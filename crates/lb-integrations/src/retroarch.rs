use crate::{DiscoveredEmulatorSave, EmulatorSaveKind};
use lb_platform::{split_windows_command_line, LaunchPathResolver};
use md5::{Digest, Md5};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use thiserror::Error;

const SATURN_EXTENSIONS: [&str; 3] = ["bcr", "bkr", "smpc"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetroArchContent {
    pub game_id: String,
    pub additional_application_id: Option<String>,
    pub content_path: PathBuf,
    pub effective_command_line: String,
    pub platform: String,
    pub scrape_as: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct RetroArchConfiguration {
    values: BTreeMap<String, String>,
}

impl RetroArchConfiguration {
    fn load(path: &Path) -> Result<Self, RetroArchError> {
        let contents = fs::read_to_string(path).map_err(|source| {
            if source.kind() == std::io::ErrorKind::NotFound {
                RetroArchError::ConfigurationNotFound {
                    path: path.to_path_buf(),
                }
            } else {
                RetroArchError::Read {
                    path: path.to_path_buf(),
                    source,
                }
            }
        })?;
        let mut values = BTreeMap::new();
        for (line_number, raw) in contents.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                return Err(RetroArchError::InvalidConfigurationLine {
                    path: path.to_path_buf(),
                    line: line_number + 1,
                });
            };
            let key = key.trim();
            if key.is_empty() {
                return Err(RetroArchError::InvalidConfigurationLine {
                    path: path.to_path_buf(),
                    line: line_number + 1,
                });
            }
            let value = value.trim();
            let value = value
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
                .unwrap_or(value);
            values.insert(key.to_ascii_lowercase(), value.to_string());
        }
        Ok(Self { values })
    }

    fn enabled(&self, key: &str) -> bool {
        self.values
            .get(key)
            .is_some_and(|value| value.eq_ignore_ascii_case("true"))
    }

    fn value(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }
}

pub fn discover_retroarch_saves(
    emulator_application_path: &Path,
    targets: &[RetroArchContent],
    configured_path_resolver: &dyn LaunchPathResolver,
) -> Result<Vec<DiscoveredEmulatorSave>, RetroArchError> {
    let emulator_application_path =
        canonical_regular_file(emulator_application_path, "RetroArch executable")?;
    let emulator_root = emulator_application_path.parent().ok_or_else(|| {
        RetroArchError::MissingEmulatorDirectory {
            path: emulator_application_path.clone(),
        }
    })?;
    let configuration_path = emulator_root.join("retroarch.cfg");
    let configuration = RetroArchConfiguration::load(&configuration_path)?;
    let emulator_file_name = emulator_application_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| RetroArchError::NonUnicodePath {
            path: emulator_application_path.clone(),
        })?
        .to_string();

    let mut discovered = Vec::new();
    let mut identities = BTreeSet::new();
    for target in targets {
        let content_path = canonical_regular_file(&target.content_path, "content file")?;
        let content_stem = content_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .filter(|stem| !stem.is_empty())
            .ok_or_else(|| RetroArchError::MissingContentStem {
                path: content_path.clone(),
            })?;
        let core = retroarch_core_name(&target.effective_command_line).unwrap_or_default();
        let saturn = is_saturn_context(target, &core);

        if let Some(save_directory) = resolve_data_directory(
            &configuration,
            configured_path_resolver,
            emulator_root,
            &content_path,
            &core,
            "savefiles_in_content_dir",
            "savefile_directory",
            "sort_savefiles_by_content_enable",
            "sort_savefiles_enable",
        )? {
            let files = matching_files(&save_directory, content_stem, SaveFileMatch::Game)?;
            let mut processed = BTreeSet::new();
            if saturn {
                for (stem, set) in saturn_sets(&files) {
                    let Some(primary) = set.first().cloned() else {
                        continue;
                    };
                    for path in &set {
                        processed.insert(path.clone());
                    }
                    let companions = set.into_iter().filter(|path| path != &primary).collect();
                    push_unique(
                        &mut discovered,
                        &mut identities,
                        DiscoveredEmulatorSave {
                            game_id: target.game_id.clone(),
                            additional_application_id: target.additional_application_id.clone(),
                            emulator_file_name: emulator_file_name.clone(),
                            emulator_core: core.clone(),
                            kind: EmulatorSaveKind::Game,
                            primary_path: primary,
                            companion_paths: companions,
                            save_group_id: Some(saturn_group_id(&stem)),
                            save_group_name: "My Save File".to_string(),
                            display_chip_text: None,
                            container_save: None,
                        },
                    );
                }
            }
            let ordinary = files
                .into_iter()
                .filter(|path| !processed.contains(path))
                .collect::<Vec<_>>();
            for path in select_ordinary_game_saves(ordinary) {
                push_unique(
                    &mut discovered,
                    &mut identities,
                    DiscoveredEmulatorSave {
                        game_id: target.game_id.clone(),
                        additional_application_id: target.additional_application_id.clone(),
                        emulator_file_name: emulator_file_name.clone(),
                        emulator_core: core.clone(),
                        kind: EmulatorSaveKind::Game,
                        primary_path: path,
                        companion_paths: Vec::new(),
                        save_group_id: None,
                        save_group_name: "My Save File".to_string(),
                        display_chip_text: None,
                        container_save: None,
                    },
                );
            }
        }

        if let Some(state_directory) = resolve_data_directory(
            &configuration,
            configured_path_resolver,
            emulator_root,
            &content_path,
            &core,
            "savestates_in_content_dir",
            "savestate_directory",
            "sort_savestates_by_content_enable",
            "sort_savestates_enable",
        )? {
            for path in matching_files(&state_directory, content_stem, SaveFileMatch::State)? {
                let Some(slot) = state_slot(&path, content_stem) else {
                    continue;
                };
                push_unique(
                    &mut discovered,
                    &mut identities,
                    DiscoveredEmulatorSave {
                        game_id: target.game_id.clone(),
                        additional_application_id: target.additional_application_id.clone(),
                        emulator_file_name: emulator_file_name.clone(),
                        emulator_core: core.clone(),
                        kind: EmulatorSaveKind::State { slot },
                        primary_path: path,
                        companion_paths: Vec::new(),
                        save_group_id: None,
                        save_group_name: "My Save State".to_string(),
                        display_chip_text: None,
                        container_save: None,
                    },
                );
            }
        }
    }

    discovered.sort_by(|left, right| {
        left.game_id
            .cmp(&right.game_id)
            .then_with(|| {
                left.additional_application_id
                    .cmp(&right.additional_application_id)
            })
            .then_with(|| {
                path_sort_key(&left.primary_path).cmp(&path_sort_key(&right.primary_path))
            })
    });
    Ok(discovered)
}

pub fn retroarch_core_name(command_line: &str) -> Option<String> {
    let arguments = split_windows_command_line(command_line);
    arguments
        .iter()
        .enumerate()
        .rev()
        .find_map(|(index, argument)| {
            let value = if argument.eq_ignore_ascii_case("-l")
                || argument.eq_ignore_ascii_case("--libretro")
            {
                arguments.get(index + 1)?
            } else if argument
                .get(.."--libretro=".len())
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case("--libretro="))
            {
                argument.get("--libretro=".len()..)?
            } else {
                return None;
            };
            let normalized = value.replace('\\', "/");
            let file_name = normalized.rsplit('/').next()?;
            let stem = file_name
                .rsplit_once('.')
                .filter(|(_, extension)| {
                    ["dll", "so", "dylib"]
                        .iter()
                        .any(|candidate| extension.eq_ignore_ascii_case(candidate))
                })
                .map(|(stem, _)| stem)
                .unwrap_or(file_name);
            (!stem.trim().is_empty()).then(|| stem.to_string())
        })
}

pub fn is_retroarch_emulator(title: &str, application_path: &Path) -> bool {
    title.trim().eq_ignore_ascii_case("retroarch")
        || application_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .is_some_and(|stem| stem.eq_ignore_ascii_case("retroarch"))
}

pub fn is_saturn_companion_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            SATURN_EXTENSIONS
                .iter()
                .any(|candidate| extension.eq_ignore_ascii_case(candidate))
        })
}

pub fn saturn_group_id(base_name: &str) -> String {
    let base_name = base_name.trim();
    if base_name.is_empty() {
        "saturn".to_string()
    } else {
        format!("saturn-{}", base_name.to_lowercase())
    }
}

pub fn inspect_saturn_save_set(primary_path: &Path) -> Result<Vec<PathBuf>, RetroArchError> {
    if !is_saturn_companion_path(primary_path) {
        return Err(RetroArchError::NotSaturnSave {
            path: primary_path.to_path_buf(),
        });
    }
    let primary = canonical_regular_file(primary_path, "Saturn primary save")?;
    let parent = primary
        .parent()
        .ok_or_else(|| RetroArchError::MissingSaveDirectory {
            path: primary.clone(),
        })?;
    let stem = primary
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| RetroArchError::NonUnicodePath {
            path: primary.clone(),
        })?;
    let mut paths = Vec::new();
    for extension in SATURN_EXTENSIONS {
        let candidate = find_case_insensitive_sibling(parent, stem, extension)?;
        if let Some(candidate) = candidate {
            paths.push(canonical_regular_file(&candidate, "Saturn companion save")?);
        }
    }
    if paths.is_empty() {
        return Err(RetroArchError::NotSaturnSave { path: primary });
    }
    Ok(paths)
}

/// Computes the signature used by LaunchBox 13.27's RetroArch adapter.
///
/// Ordinary saves use the uppercase MD5 of the primary file. Saturn sets hash
/// a CRLF-delimited manifest containing each lower-case filename and uppercase
/// member MD5, ordered case-insensitively by full path.
pub fn retroarch_save_signature(save: &DiscoveredEmulatorSave) -> Result<String, RetroArchError> {
    if save.companion_paths.is_empty() && !is_saturn_companion_path(&save.primary_path) {
        return md5_file(&save.primary_path);
    }
    let mut paths = save.all_paths().cloned().collect::<Vec<_>>();
    paths.sort_by_key(|path| path_sort_key(path));
    let mut manifest = String::new();
    for path in paths {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| RetroArchError::NonUnicodePath { path: path.clone() })?;
        manifest.push_str(&file_name.to_ascii_lowercase());
        manifest.push('|');
        manifest.push_str(&md5_file(&path)?);
        manifest.push_str("\r\n");
    }
    Ok(format!("{:X}", Md5::digest(manifest.as_bytes())))
}

#[allow(clippy::too_many_arguments)]
fn resolve_data_directory(
    configuration: &RetroArchConfiguration,
    configured_path_resolver: &dyn LaunchPathResolver,
    emulator_root: &Path,
    content_path: &Path,
    core: &str,
    in_content_key: &str,
    directory_key: &str,
    sort_content_key: &str,
    sort_core_key: &str,
) -> Result<Option<PathBuf>, RetroArchError> {
    let mut directory = if configuration.enabled(in_content_key) {
        content_path
            .parent()
            .ok_or_else(|| RetroArchError::MissingContentDirectory {
                path: content_path.to_path_buf(),
            })?
            .to_path_buf()
    } else {
        let Some(configured) = configuration.value(directory_key) else {
            return Ok(None);
        };
        let configured = configured
            .strip_prefix(":\\")
            .or_else(|| configured.strip_prefix(":/"))
            .unwrap_or(configured);
        configured_path_resolver
            .resolve(emulator_root, configured)
            .map_err(|source| RetroArchError::ConfiguredPath {
                key: directory_key.to_string(),
                value: configured.to_string(),
                reason: source.to_string(),
            })?
    };

    if configuration.enabled(sort_content_key) {
        let parent_name = content_path
            .parent()
            .and_then(Path::file_name)
            .ok_or_else(|| RetroArchError::MissingContentDirectory {
                path: content_path.to_path_buf(),
            })?;
        directory.push(parent_name);
    }
    if configuration.enabled(sort_core_key) {
        if core.is_empty() {
            return Ok(None);
        }
        let info_path = emulator_root.join("info").join(format!("{core}.info"));
        let Some(core_name) = core_display_name(&info_path)? else {
            return Ok(None);
        };
        directory.push(core_name);
    }
    match fs::canonicalize(&directory) {
        Ok(path) if path.is_dir() => Ok(Some(path)),
        Ok(_) => Err(RetroArchError::DataDirectoryNotDirectory { path: directory }),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(RetroArchError::Read {
            path: directory,
            source,
        }),
    }
}

fn core_display_name(path: &Path) -> Result<Option<String>, RetroArchError> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(RetroArchError::Read {
                path: path.to_path_buf(),
                source,
            })
        }
    };
    Ok(contents.lines().find_map(|line| {
        let (key, value) = line.split_once('=')?;
        if !key.trim().eq_ignore_ascii_case("corename") {
            return None;
        }
        let value = value.trim();
        Some(
            value
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
                .unwrap_or(value)
                .to_string(),
        )
    }))
}

#[derive(Clone, Copy)]
enum SaveFileMatch {
    Game,
    State,
}

fn matching_files(
    directory: &Path,
    content_stem: &str,
    kind: SaveFileMatch,
) -> Result<Vec<PathBuf>, RetroArchError> {
    let entries = fs::read_dir(directory).map_err(|source| RetroArchError::Read {
        path: directory.to_path_buf(),
        source,
    })?;
    let mut files = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| RetroArchError::Read {
            path: directory.to_path_buf(),
            source,
        })?;
        let file_type = entry.file_type().map_err(|source| RetroArchError::Read {
            path: entry.path(),
            source,
        })?;
        if !file_type.is_file() {
            continue;
        }
        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        let matches = match kind {
            SaveFileMatch::Game => {
                name.len() > content_stem.len()
                    && name
                        .get(..content_stem.len())
                        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(content_stem))
                    && Path::new(&name).extension().is_some()
            }
            SaveFileMatch::State => {
                let prefix = format!("{content_stem}.state");
                name.get(..prefix.len())
                    .is_some_and(|value| value.eq_ignore_ascii_case(&prefix))
            }
        };
        if matches {
            files.push(entry.path());
        }
    }
    files.sort_by_key(|path| path_sort_key(path));
    Ok(files)
}

fn saturn_sets(files: &[PathBuf]) -> Vec<(String, Vec<PathBuf>)> {
    let mut groups = BTreeMap::<String, (String, Vec<PathBuf>)>::new();
    for path in files {
        if !is_saturn_companion_path(path) {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        groups
            .entry(stem.to_ascii_lowercase())
            .or_insert_with(|| (stem.to_string(), Vec::new()))
            .1
            .push(path.clone());
    }
    groups
        .into_values()
        .map(|(stem, mut paths)| {
            paths.sort_by_key(|path| {
                SATURN_EXTENSIONS
                    .iter()
                    .position(|extension| {
                        path.extension()
                            .and_then(|value| value.to_str())
                            .is_some_and(|value| value.eq_ignore_ascii_case(extension))
                    })
                    .unwrap_or(usize::MAX)
            });
            (stem, paths)
        })
        .collect()
}

fn select_ordinary_game_saves(files: Vec<PathBuf>) -> Vec<PathBuf> {
    if files.len() <= 1 {
        return files;
    }
    let preferred = ["srm", "mcr"]
        .into_iter()
        .filter_map(|extension| {
            files.iter().find(|path| {
                path.extension()
                    .and_then(|value| value.to_str())
                    .is_some_and(|value| value.eq_ignore_ascii_case(extension))
            })
        })
        .cloned()
        .collect::<Vec<_>>();
    if preferred.is_empty() {
        files
    } else {
        preferred
    }
}

fn state_slot(path: &Path, content_stem: &str) -> Option<i32> {
    let name = path.file_name()?.to_str()?;
    let prefix = format!("{content_stem}.state");
    let suffix = name.get(prefix.len()..)?;
    if suffix.is_empty() {
        Some(0)
    } else if suffix.eq_ignore_ascii_case(".auto") || suffix.eq_ignore_ascii_case("auto") {
        Some(-1)
    } else {
        suffix.parse().ok()
    }
}

fn is_saturn_context(target: &RetroArchContent, core: &str) -> bool {
    core.to_ascii_lowercase().contains("saturn")
        || target.platform.to_ascii_lowercase().contains("saturn")
        || target
            .scrape_as
            .as_deref()
            .is_some_and(|name| name.to_ascii_lowercase().contains("saturn"))
}

fn push_unique(
    saves: &mut Vec<DiscoveredEmulatorSave>,
    identities: &mut BTreeSet<(String, Option<String>, PathBuf)>,
    save: DiscoveredEmulatorSave,
) {
    let identity = (
        save.game_id.clone(),
        save.additional_application_id.clone(),
        save.primary_path.clone(),
    );
    if identities.insert(identity) {
        saves.push(save);
    }
}

fn canonical_regular_file(path: &Path, kind: &'static str) -> Result<PathBuf, RetroArchError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| RetroArchError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    if !metadata.file_type().is_file() {
        return Err(RetroArchError::NotRegularFile {
            kind,
            path: path.to_path_buf(),
        });
    }
    fs::canonicalize(path).map_err(|source| RetroArchError::Read {
        path: path.to_path_buf(),
        source,
    })
}

fn find_case_insensitive_sibling(
    parent: &Path,
    stem: &str,
    extension: &str,
) -> Result<Option<PathBuf>, RetroArchError> {
    let expected = format!("{stem}.{extension}");
    let expected_key = expected.to_lowercase();
    let entries = fs::read_dir(parent).map_err(|source| RetroArchError::Read {
        path: parent.to_path_buf(),
        source,
    })?;
    let mut matches = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| RetroArchError::Read {
            path: parent.to_path_buf(),
            source,
        })?;
        if entry
            .file_name()
            .to_str()
            .is_some_and(|name| name.to_lowercase() == expected_key)
        {
            matches.push(entry.path());
        }
    }
    matches.sort_by_key(|path| path_sort_key(path));
    match matches.as_slice() {
        [] => Ok(None),
        [path] => Ok(Some(path.clone())),
        _ => Err(RetroArchError::AmbiguousCompanion {
            directory: parent.to_path_buf(),
            name: expected,
        }),
    }
}

fn path_sort_key(path: &Path) -> String {
    path.to_string_lossy().to_ascii_lowercase()
}

fn md5_file(path: &Path) -> Result<String, RetroArchError> {
    let mut file = fs::File::open(path).map_err(|source| RetroArchError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let mut digest = Md5::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|source| RetroArchError::Read {
                path: path.to_path_buf(),
                source,
            })?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(format!("{:X}", digest.finalize()))
}

#[derive(Debug, Error)]
pub enum RetroArchError {
    #[error("RetroArch configuration was not found at {path}")]
    ConfigurationNotFound { path: PathBuf },
    #[error("RetroArch configuration {path} has an invalid line at {line}")]
    InvalidConfigurationLine { path: PathBuf, line: usize },
    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{kind} is not a regular file: {path}")]
    NotRegularFile { kind: &'static str, path: PathBuf },
    #[error("RetroArch executable has no parent directory: {path}")]
    MissingEmulatorDirectory { path: PathBuf },
    #[error("content file has no parent directory: {path}")]
    MissingContentDirectory { path: PathBuf },
    #[error("content file has no usable filename stem: {path}")]
    MissingContentStem { path: PathBuf },
    #[error("configured RetroArch path {key}={value:?} cannot be resolved: {reason}")]
    ConfiguredPath {
        key: String,
        value: String,
        reason: String,
    },
    #[error("configured RetroArch data path is not a directory: {path}")]
    DataDirectoryNotDirectory { path: PathBuf },
    #[error("save path has no parent directory: {path}")]
    MissingSaveDirectory { path: PathBuf },
    #[error("path cannot be represented as Unicode: {path}")]
    NonUnicodePath { path: PathBuf },
    #[error("save is not a RetroArch Saturn primary or companion file: {path}")]
    NotSaturnSave { path: PathBuf },
    #[error("multiple case-insensitive Saturn companions named {name} exist in {directory}")]
    AmbiguousCompanion { directory: PathBuf, name: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use lb_platform::HostPathResolver;
    use std::fs;
    use tempfile::tempdir;

    fn create_file(path: &Path, bytes: &[u8]) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    #[test]
    fn discovers_regular_saves_states_and_core_sorted_directories() {
        let root = tempdir().unwrap();
        let emulator = root.path().join("retroarch/retroarch");
        let content = root.path().join("roms/System/Fixture Game.rom");
        create_file(&emulator, b"executable");
        create_file(&content, b"rom");
        fs::write(
            root.path().join("retroarch/retroarch.cfg"),
            concat!(
                "savefile_directory = \":/saves\"\n",
                "savestate_directory = \":/states\"\n",
                "sort_savefiles_enable = \"true\"\n",
                "sort_savestates_by_content_enable = \"true\"\n",
            ),
        )
        .unwrap();
        fs::create_dir_all(root.path().join("retroarch/info")).unwrap();
        fs::write(
            root.path().join("retroarch/info/mesen_libretro.info"),
            "display_name = \"Mesen\"\ncorename = \"Mesen Core\"\n",
        )
        .unwrap();
        create_file(
            &root
                .path()
                .join("retroarch/saves/Mesen Core/Fixture Game.srm"),
            b"save",
        );
        create_file(
            &root
                .path()
                .join("retroarch/states/System/Fixture Game.state"),
            b"state",
        );
        create_file(
            &root
                .path()
                .join("retroarch/states/System/Fixture Game.state.auto"),
            b"auto",
        );
        create_file(
            &root
                .path()
                .join("retroarch/states/System/Fixture Game.state3"),
            b"slot",
        );
        create_file(
            &root
                .path()
                .join("retroarch/states/System/Fixture Game.state.bad"),
            b"ignore",
        );

        let target = RetroArchContent {
            game_id: "game".into(),
            additional_application_id: None,
            content_path: content,
            effective_command_line: r#"-f -L "cores\mesen_libretro.dll" --appendconfig game.cfg"#
                .into(),
            platform: "Fixture Console".into(),
            scrape_as: None,
        };
        let saves =
            discover_retroarch_saves(&emulator, &[target], &HostPathResolver::default()).unwrap();

        assert_eq!(saves.len(), 4);
        assert_eq!(
            saves
                .iter()
                .map(DiscoveredEmulatorSave::slot)
                .collect::<Vec<_>>(),
            vec![None, Some(0), Some(-1), Some(3)]
        );
        assert!(saves
            .iter()
            .all(|save| save.emulator_core == "mesen_libretro"));
        assert!(saves.iter().all(|save| save.companion_paths.is_empty()));
    }

    #[test]
    fn saturn_discovery_uses_one_primary_and_ordered_companions() {
        let root = tempdir().unwrap();
        let emulator = root.path().join("retroarch/retroarch");
        let content = root.path().join("roms/Saturn/Nights.chd");
        create_file(&emulator, b"executable");
        create_file(&content, b"rom");
        fs::write(
            root.path().join("retroarch/retroarch.cfg"),
            "savefile_directory = \":/saves\"\n",
        )
        .unwrap();
        create_file(&root.path().join("retroarch/saves/Nights.smpc"), b"smpc");
        create_file(&root.path().join("retroarch/saves/Nights.bkr"), b"bkr");
        create_file(&root.path().join("retroarch/saves/Nights.bcr"), b"bcr");

        let target = RetroArchContent {
            game_id: "saturn-game".into(),
            additional_application_id: Some("disc-two".into()),
            content_path: content,
            effective_command_line: "-L cores/beetle_saturn_libretro.so".into(),
            platform: "Sega Saturn".into(),
            scrape_as: None,
        };
        let saves =
            discover_retroarch_saves(&emulator, &[target], &HostPathResolver::default()).unwrap();

        assert_eq!(saves.len(), 1);
        let save = &saves[0];
        assert_eq!(
            save.primary_path.file_name().unwrap().to_str(),
            Some("Nights.bcr")
        );
        assert_eq!(
            save.companion_paths
                .iter()
                .map(|path| path.file_name().unwrap().to_str().unwrap())
                .collect::<Vec<_>>(),
            vec!["Nights.bkr", "Nights.smpc"]
        );
        assert_eq!(save.save_group_id.as_deref(), Some("saturn-nights"));
        assert_eq!(save.save_group_name, "My Save File");
    }

    #[test]
    fn configured_windows_directory_uses_the_shared_host_mapping() {
        #[cfg(not(windows))]
        {
            let root = tempdir().unwrap();
            let mapped = root.path().join("mapped-c");
            let emulator = root.path().join("retroarch/retroarch");
            let content = root.path().join("roms/Fixture.rom");
            create_file(&emulator, b"executable");
            create_file(&content, b"rom");
            fs::write(
                root.path().join("retroarch/retroarch.cfg"),
                "savefile_directory = \"C:\\\\RetroArch\\\\saves\"\n",
            )
            .unwrap();
            create_file(&mapped.join("RetroArch/saves/Fixture.srm"), b"save");
            let resolver = HostPathResolver::default()
                .with_windows_drive_mapping('C', mapped)
                .unwrap();
            let target = RetroArchContent {
                game_id: "game".into(),
                additional_application_id: None,
                content_path: content,
                effective_command_line: String::new(),
                platform: "Fixture".into(),
                scrape_as: None,
            };

            let saves = discover_retroarch_saves(&emulator, &[target], &resolver).unwrap();
            assert_eq!(saves.len(), 1);
            assert_eq!(
                saves[0].primary_path.file_name().unwrap().to_str(),
                Some("Fixture.srm")
            );
        }
    }

    #[test]
    fn core_parser_accepts_windows_linux_and_macos_library_names() {
        assert_eq!(
            retroarch_core_name(r#"-L "cores\mesen_libretro.dll""#).as_deref(),
            Some("mesen_libretro")
        );
        assert_eq!(
            retroarch_core_name("-f -L /usr/lib/libretro/snes9x_libretro.so").as_deref(),
            Some("snes9x_libretro")
        );
        assert_eq!(
            retroarch_core_name("-L cores/parallel_n64_libretro.dylib").as_deref(),
            Some("parallel_n64_libretro")
        );
        assert_eq!(
            retroarch_core_name("--libretro=cores/genesis_plus_gx_libretro.so").as_deref(),
            Some("genesis_plus_gx_libretro")
        );
        assert_eq!(
            retroarch_core_name("--libretro cores/mgba_libretro.dll").as_deref(),
            Some("mgba_libretro")
        );
        assert!(is_retroarch_emulator(
            "Custom RetroArch",
            Path::new("/opt/retroarch")
        ));
        assert!(is_retroarch_emulator(
            "RetroArch",
            Path::new("/opt/custom-emulator")
        ));
        assert!(!is_retroarch_emulator(
            "Fixture Emulator",
            Path::new("/opt/fixture")
        ));
    }

    #[test]
    fn saturn_set_inspection_is_case_insensitive_and_rejects_symlinks() {
        let root = tempdir().unwrap();
        create_file(&root.path().join("Game.BCR"), b"primary");
        create_file(&root.path().join("Game.bKr"), b"backup");
        create_file(&root.path().join("Game.SMPC"), b"clock");
        let paths = inspect_saturn_save_set(&root.path().join("Game.BCR")).unwrap();
        assert_eq!(paths.len(), 3);
        let signature = retroarch_save_signature(&DiscoveredEmulatorSave {
            game_id: "game".into(),
            additional_application_id: None,
            emulator_file_name: "retroarch".into(),
            emulator_core: "beetle_saturn_libretro".into(),
            kind: EmulatorSaveKind::Game,
            primary_path: paths[0].clone(),
            companion_paths: paths[1..].to_vec(),
            save_group_id: Some("saturn-game".into()),
            save_group_name: "Game".into(),
            display_chip_text: None,
            container_save: None,
        })
        .unwrap();
        assert_eq!(signature, "CADCB510A38A6691B4F15362B17F12EE");

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let other = tempdir().unwrap();
            create_file(&other.path().join("Other.bcr"), b"other");
            symlink(
                other.path().join("Other.bcr"),
                root.path().join("Linked.bcr"),
            )
            .unwrap();
            assert!(matches!(
                inspect_saturn_save_set(&root.path().join("Linked.bcr")),
                Err(RetroArchError::NotRegularFile { .. })
            ));
        }
    }
}
