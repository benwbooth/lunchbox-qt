use crate::{DiscoveredContainerSave, DiscoveredEmulatorSave, EmulatorSaveKind};
use std::collections::BTreeSet;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

const COMPRESSED_DISC_EXTENSIONS: [&str; 4] = ["gcz", "rvz", "wia", "wbfs"];
const DEFAULT_GAMECUBE_REGIONS: [&str; 5] = ["USA", "EUR", "JAP", "JPN", "KOR"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DolphinContent {
    pub game_id: String,
    pub additional_application_id: Option<String>,
    pub content_path: PathBuf,
    pub platform: String,
}

pub fn is_dolphin_emulator(title: &str, application_path: &Path) -> bool {
    title.trim().eq_ignore_ascii_case("dolphin")
        || application_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .is_some_and(|stem| stem.eq_ignore_ascii_case("dolphin"))
}

/// Returns existing native Dolphin user directories without interpreting a
/// LaunchBox-persisted path. Portable installs are checked beside the resolved
/// emulator executable, followed by the native per-user location.
pub fn default_dolphin_user_directories(emulator_application_path: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(parent) = emulator_application_path.parent() {
        candidates.push(parent.join("User"));
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(data_home) = std::env::var_os("XDG_DATA_HOME") {
            candidates.push(PathBuf::from(data_home).join("dolphin-emu"));
        } else if let Some(home) = std::env::var_os("HOME") {
            candidates.push(
                PathBuf::from(home)
                    .join(".local")
                    .join("share")
                    .join("dolphin-emu"),
            );
        }
    }
    #[cfg(target_os = "macos")]
    if let Some(home) = std::env::var_os("HOME") {
        candidates.push(
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("Dolphin"),
        );
    }
    #[cfg(windows)]
    if let Some(profile) = std::env::var_os("USERPROFILE") {
        candidates.push(
            PathBuf::from(profile)
                .join("Documents")
                .join("Dolphin Emulator"),
        );
    }

    existing_unique_directories(candidates)
}

/// Discovers LaunchBox 13.27's Dolphin save locations: GameCube GCI/SAV/RAW
/// files, Wii title `data` directories, and `.sNN` save states.
pub fn discover_dolphin_saves(
    emulator_application_path: &Path,
    targets: &[DolphinContent],
    user_directories: &[PathBuf],
) -> Result<Vec<DiscoveredEmulatorSave>, DolphinError> {
    let emulator_application_path =
        canonical_regular_file(emulator_application_path, "Dolphin executable")?;
    let emulator_file_name = unicode_file_name(&emulator_application_path)?.to_string();
    let user_directories = existing_unique_directories(user_directories.iter().cloned());

    let mut discovered = Vec::new();
    let mut identities = BTreeSet::new();
    for target in targets {
        let content_path = canonical_regular_file(&target.content_path, "Dolphin content")?;
        let Some(disc_id) = dolphin_disc_id(&content_path, &emulator_application_path)? else {
            continue;
        };

        if is_gamecube_context(&target.platform, &content_path) {
            discover_gamecube_files(
                &mut discovered,
                &mut identities,
                &emulator_file_name,
                target,
                &disc_id,
                &user_directories,
            )?;
        }
        if is_wii_context(&target.platform, &content_path) {
            discover_wii_directories(
                &mut discovered,
                &mut identities,
                &emulator_file_name,
                target,
                &content_path,
                &disc_id,
                &user_directories,
            )?;
        }
        discover_save_states(
            &mut discovered,
            &mut identities,
            &emulator_file_name,
            target,
            &disc_id,
            &user_directories,
        )?;
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

/// Extracts the high and low title IDs from the stable group used for a
/// discovered Dolphin Wii directory.
pub fn dolphin_wii_group_ids(group: &str) -> Option<(&str, &str)> {
    let parts = group.split(':').collect::<Vec<_>>();
    if parts.len() != 5
        || parts[0] != "dolphin"
        || parts[1] != "wii"
        || parts[2].is_empty()
        || !is_hex_title_component(parts[3])
        || !is_hex_title_component(parts[4])
    {
        return None;
    }
    Some((parts[3], parts[4]))
}

fn is_hex_title_component(value: &str) -> bool {
    value.len() == 8 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[allow(clippy::too_many_arguments)]
fn discover_wii_directories(
    discovered: &mut Vec<DiscoveredEmulatorSave>,
    identities: &mut BTreeSet<PathBuf>,
    emulator_file_name: &str,
    target: &DolphinContent,
    content_path: &Path,
    disc_id: &str,
    user_directories: &[PathBuf],
) -> Result<(), DolphinError> {
    let title_ids = if content_path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("wad"))
    {
        let Some(title_id) = wad_title_id(content_path)? else {
            return Ok(());
        };
        vec![(
            hex_title_component(&title_id[..4]),
            hex_title_component(&title_id[4..]),
        )]
    } else {
        let Some(low_bytes) = disc_id.as_bytes().get(..4) else {
            return Ok(());
        };
        let low = hex_title_component(low_bytes);
        vec![
            ("00010000".to_string(), low.clone()),
            ("00010004".to_string(), low),
        ]
    };

    for user_directory in user_directories {
        for (high, low) in &title_ids {
            let data = user_directory
                .join("Wii")
                .join("title")
                .join(high)
                .join(low)
                .join("data");
            let Some(metadata) = inspect_wii_save_directory(&data)? else {
                continue;
            };
            if !identities.insert(metadata.canonical.clone()) {
                continue;
            }
            discovered.push(DiscoveredEmulatorSave {
                game_id: target.game_id.clone(),
                additional_application_id: target.additional_application_id.clone(),
                emulator_file_name: emulator_file_name.to_string(),
                emulator_core: String::new(),
                kind: EmulatorSaveKind::Game,
                primary_path: metadata.canonical,
                companion_paths: Vec::new(),
                save_group_id: Some(format!(
                    "dolphin:wii:{}:{}:{}",
                    target.game_id,
                    high.to_ascii_lowercase(),
                    low.to_ascii_lowercase()
                )),
                save_group_name: "My Save File".to_string(),
                display_chip_text: match high.as_str() {
                    "00010000" => Some("Disc Save".to_string()),
                    "00010004" => Some("Channel Save".to_string()),
                    _ => None,
                },
                container_save: Some(DiscoveredContainerSave {
                    original_file_name: "data".to_string(),
                    reported_file_size_bytes: Some(
                        i64::try_from(metadata.byte_len).unwrap_or(i64::MAX),
                    ),
                    reported_last_modified: metadata.last_modified,
                }),
            });
        }
    }
    Ok(())
}

struct WiiDirectoryMetadata {
    canonical: PathBuf,
    byte_len: u64,
    last_modified: Option<std::time::SystemTime>,
}

fn inspect_wii_save_directory(path: &Path) -> Result<Option<WiiDirectoryMetadata>, DolphinError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(DolphinError::Read {
                path: path.to_path_buf(),
                source,
            });
        }
    };
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(DolphinError::UnsafeDirectoryEntry {
            path: path.to_path_buf(),
        });
    }
    let canonical = fs::canonicalize(path).map_err(|source| DolphinError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let mut byte_len = 0_u64;
    let mut last_modified = metadata.modified().ok();
    inspect_wii_save_directory_contents(&canonical, &mut byte_len, &mut last_modified)?;
    Ok(Some(WiiDirectoryMetadata {
        canonical,
        byte_len,
        last_modified,
    }))
}

fn inspect_wii_save_directory_contents(
    directory: &Path,
    byte_len: &mut u64,
    last_modified: &mut Option<std::time::SystemTime>,
) -> Result<(), DolphinError> {
    let mut entries = fs::read_dir(directory)
        .map_err(|source| DolphinError::Read {
            path: directory.to_path_buf(),
            source,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| DolphinError::Read {
            path: directory.to_path_buf(),
            source,
        })?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        if entry.file_name().to_str().is_none() {
            return Err(DolphinError::NonUnicodePath { path: entry.path() });
        }
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|source| DolphinError::Read {
            path: path.clone(),
            source,
        })?;
        if let Ok(modified) = metadata.modified() {
            if last_modified.is_none_or(|current| modified > current) {
                *last_modified = Some(modified);
            }
        }
        if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
            inspect_wii_save_directory_contents(&path, byte_len, last_modified)?;
        } else if metadata.file_type().is_file() && !metadata.file_type().is_symlink() {
            *byte_len = byte_len.saturating_add(metadata.len());
        } else {
            return Err(DolphinError::UnsafeDirectoryEntry { path });
        }
    }
    Ok(())
}

fn hex_title_component(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
    }
    output
}

fn discover_gamecube_files(
    discovered: &mut Vec<DiscoveredEmulatorSave>,
    identities: &mut BTreeSet<PathBuf>,
    emulator_file_name: &str,
    target: &DolphinContent,
    disc_id: &str,
    user_directories: &[PathBuf],
) -> Result<(), DolphinError> {
    for user_directory in user_directories {
        let gamecube_root = user_directory.join("GC");
        if !gamecube_root.is_dir() {
            continue;
        }
        for region in gamecube_region_candidates(disc_id) {
            let region_path = gamecube_root.join(region);
            if !region_path.is_dir() {
                continue;
            }

            let folder = region_path.join(disc_id);
            for path in preferred_gamecube_files(&folder)? {
                push_gamecube_file(
                    discovered,
                    identities,
                    emulator_file_name,
                    target,
                    disc_id,
                    path,
                    "Folder",
                    None,
                )?;
            }

            let prefix = disc_id.get(..4).unwrap_or(disc_id);
            for (directory, chip) in [("Card A", "Card A"), ("Card B", "Card B")] {
                let card = region_path.join(directory);
                for path in files_with_extension(&card, "gci")? {
                    let stem = path
                        .file_stem()
                        .and_then(|stem| stem.to_str())
                        .unwrap_or_default();
                    if stem
                        .to_ascii_lowercase()
                        .contains(&prefix.to_ascii_lowercase())
                    {
                        push_gamecube_file(
                            discovered,
                            identities,
                            emulator_file_name,
                            target,
                            disc_id,
                            path,
                            directory.replace(' ', "").as_str(),
                            Some(chip.to_string()),
                        )?;
                    }
                }
            }
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn push_gamecube_file(
    discovered: &mut Vec<DiscoveredEmulatorSave>,
    identities: &mut BTreeSet<PathBuf>,
    emulator_file_name: &str,
    target: &DolphinContent,
    disc_id: &str,
    path: PathBuf,
    source_token: &str,
    display_chip_text: Option<String>,
) -> Result<(), DolphinError> {
    let primary_path = canonical_regular_file(&path, "Dolphin GameCube save")?;
    if !identities.insert(primary_path.clone()) {
        return Ok(());
    }
    let original_file_name = unicode_file_name(&primary_path)?.to_string();
    discovered.push(DiscoveredEmulatorSave {
        game_id: target.game_id.clone(),
        additional_application_id: target.additional_application_id.clone(),
        emulator_file_name: emulator_file_name.to_string(),
        emulator_core: String::new(),
        kind: EmulatorSaveKind::Game,
        primary_path,
        companion_paths: Vec::new(),
        save_group_id: Some(format!(
            "dolphin:gc:{}:{}:{source_token}:{original_file_name}",
            target.game_id,
            disc_id.to_ascii_uppercase()
        )),
        save_group_name: "My Save File".to_string(),
        display_chip_text,
        container_save: None,
    });
    Ok(())
}

fn discover_save_states(
    discovered: &mut Vec<DiscoveredEmulatorSave>,
    identities: &mut BTreeSet<PathBuf>,
    emulator_file_name: &str,
    target: &DolphinContent,
    disc_id: &str,
    user_directories: &[PathBuf],
) -> Result<(), DolphinError> {
    for user_directory in user_directories {
        let state_root = user_directory.join("StateSaves");
        if !state_root.is_dir() {
            continue;
        }
        let mut entries = fs::read_dir(&state_root)
            .map_err(|source| DolphinError::Read {
                path: state_root.clone(),
                source,
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|source| DolphinError::Read {
                path: state_root.clone(),
                source,
            })?;
        entries.sort_by_key(|entry| entry.file_name().to_string_lossy().to_ascii_lowercase());
        for entry in entries {
            let file_name = entry.file_name();
            let Some(file_name) = file_name.to_str() else {
                continue;
            };
            let Some(slot) = dolphin_state_slot(file_name, disc_id) else {
                continue;
            };
            let primary_path = canonical_regular_file(&entry.path(), "Dolphin save state")?;
            if !identities.insert(primary_path.clone()) {
                continue;
            }
            discovered.push(DiscoveredEmulatorSave {
                game_id: target.game_id.clone(),
                additional_application_id: target.additional_application_id.clone(),
                emulator_file_name: emulator_file_name.to_string(),
                emulator_core: String::new(),
                kind: EmulatorSaveKind::State { slot },
                primary_path,
                companion_paths: Vec::new(),
                save_group_id: Some(format!("{}-{}-State-{slot}", target.game_id, disc_id)),
                save_group_name: "My Save State".to_string(),
                display_chip_text: None,
                container_save: None,
            });
        }
    }
    Ok(())
}

fn dolphin_state_slot(file_name: &str, disc_id: &str) -> Option<i32> {
    let expected_prefix = format!("{disc_id}.s");
    if file_name.len() != expected_prefix.len() + 2
        || !file_name
            .get(..expected_prefix.len())?
            .eq_ignore_ascii_case(&expected_prefix)
    {
        return None;
    }
    let slot = file_name.get(expected_prefix.len()..)?;
    (slot.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| slot.parse().ok())
        .flatten()
}

fn dolphin_disc_id(
    content_path: &Path,
    emulator_application_path: &Path,
) -> Result<Option<String>, DolphinError> {
    let extension = content_path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default();
    if extension.eq_ignore_ascii_case("wad") {
        return wad_title_id(content_path)
            .map(|title_id| title_id.and_then(|title_id| normalize_disc_id_bytes(&title_id[4..])));
    }
    if COMPRESSED_DISC_EXTENSIONS
        .iter()
        .any(|candidate| extension.eq_ignore_ascii_case(candidate))
    {
        return dolphin_tool_disc_id(content_path, emulator_application_path);
    }

    let mut file = fs::File::open(content_path).map_err(|source| DolphinError::Read {
        path: content_path.to_path_buf(),
        source,
    })?;
    let mut bytes = [0_u8; 6];
    let read = file.read(&mut bytes).map_err(|source| DolphinError::Read {
        path: content_path.to_path_buf(),
        source,
    })?;
    if read != bytes.len() {
        return Ok(None);
    }
    Ok(normalize_disc_id_bytes(&bytes))
}

fn dolphin_tool_disc_id(
    content_path: &Path,
    emulator_application_path: &Path,
) -> Result<Option<String>, DolphinError> {
    let Some(emulator_root) = emulator_application_path.parent() else {
        return Ok(None);
    };
    let tool = ["DolphinTool.exe", "dolphin-tool", "DolphinTool"]
        .into_iter()
        .map(|name| emulator_root.join(name))
        .find(|path| path.is_file());
    let Some(tool) = tool else {
        return Ok(None);
    };
    for mode in ["header", "verify"] {
        let mut command = Command::new(&tool);
        command.arg(mode).arg("-i").arg(content_path);
        if mode == "verify" {
            command.arg("-a").arg("info");
        }
        let output = command.output().map_err(|source| DolphinError::Tool {
            path: tool.clone(),
            source,
        })?;
        let combined = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        if let Some(disc_id) = parse_dolphin_tool_disc_id(&combined) {
            return Ok(Some(disc_id));
        }
    }
    Ok(None)
}

fn parse_dolphin_tool_disc_id(output: &str) -> Option<String> {
    output.lines().find_map(|line| {
        let line = line.trim();
        let (key, value) = line.split_once(':')?;
        (key.trim().eq_ignore_ascii_case("Game ID") || key.trim().eq_ignore_ascii_case("Title ID"))
            .then(|| normalize_disc_id(value.trim()))?
    })
}

fn normalize_disc_id(value: &str) -> Option<String> {
    let value = value.trim().trim_end_matches('\0');
    ((value.len() == 4 || value.len() == 6)
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric()))
    .then(|| value.to_ascii_uppercase())
}

fn normalize_disc_id_bytes(bytes: &[u8]) -> Option<String> {
    std::str::from_utf8(bytes).ok().and_then(normalize_disc_id)
}

fn wad_title_id(path: &Path) -> Result<Option<[u8; 8]>, DolphinError> {
    let mut file = fs::File::open(path).map_err(|source| DolphinError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    file.seek(SeekFrom::Start(8))
        .map_err(|source| DolphinError::Read {
            path: path.to_path_buf(),
            source,
        })?;
    let mut lengths = [0_u8; 8];
    if file.read_exact(&mut lengths).is_err() {
        return Ok(None);
    }
    let certificate_len = u32::from_be_bytes(lengths[..4].try_into().unwrap()) as u64;
    let ticket_len = u32::from_be_bytes(lengths[4..].try_into().unwrap()) as u64;
    let offset =
        align_64(align_64(64_u64.saturating_add(certificate_len)).saturating_add(ticket_len))
            .saturating_add(476);
    let length = file
        .metadata()
        .map_err(|source| DolphinError::Read {
            path: path.to_path_buf(),
            source,
        })?
        .len();
    if length < offset.saturating_add(8) {
        return Ok(None);
    }
    file.seek(SeekFrom::Start(offset))
        .map_err(|source| DolphinError::Read {
            path: path.to_path_buf(),
            source,
        })?;
    let mut title_id = [0_u8; 8];
    file.read_exact(&mut title_id)
        .map_err(|source| DolphinError::Read {
            path: path.to_path_buf(),
            source,
        })?;
    Ok(Some(title_id))
}

fn align_64(value: u64) -> u64 {
    let remainder = value % 64;
    if remainder == 0 {
        value
    } else {
        value.saturating_add(64 - remainder)
    }
}

fn is_gamecube_context(platform: &str, path: &Path) -> bool {
    if platform.to_ascii_lowercase().contains("gamecube") {
        return true;
    }
    if platform.to_ascii_lowercase().contains("wii") {
        return false;
    }
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            ["iso", "gcm", "gcz"]
                .iter()
                .any(|candidate| extension.eq_ignore_ascii_case(candidate))
        })
}

fn is_wii_context(platform: &str, path: &Path) -> bool {
    platform.to_ascii_lowercase().contains("wii")
        || path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("wad"))
}

fn gamecube_region_candidates(disc_id: &str) -> Vec<&'static str> {
    let preferred: &[&str] = match disc_id.as_bytes().get(3).map(u8::to_ascii_uppercase) {
        Some(b'E') => &["USA"],
        Some(b'P') => &["EUR"],
        Some(b'J') => &["JAP", "JPN"],
        Some(b'K') => &["KOR"],
        Some(b'D') => &["GER"],
        Some(b'F') => &["FRA"],
        Some(b'S') => &["ESP"],
        _ => &[],
    };
    let mut regions = Vec::new();
    for region in preferred.iter().copied().chain(DEFAULT_GAMECUBE_REGIONS) {
        if !regions.contains(&region) {
            regions.push(region);
        }
    }
    regions
}

fn preferred_gamecube_files(directory: &Path) -> Result<Vec<PathBuf>, DolphinError> {
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    for extension in ["gci", "sav", "raw"] {
        let files = files_with_extension(directory, extension)?;
        if !files.is_empty() {
            return Ok(files);
        }
    }
    directory_files(directory)
}

fn files_with_extension(directory: &Path, extension: &str) -> Result<Vec<PathBuf>, DolphinError> {
    Ok(directory_files(directory)?
        .into_iter()
        .filter(|path| {
            path.extension()
                .and_then(|candidate| candidate.to_str())
                .is_some_and(|candidate| candidate.eq_ignore_ascii_case(extension))
        })
        .collect())
}

fn directory_files(directory: &Path) -> Result<Vec<PathBuf>, DolphinError> {
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    let mut paths = fs::read_dir(directory)
        .map_err(|source| DolphinError::Read {
            path: directory.to_path_buf(),
            source,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| DolphinError::Read {
            path: directory.to_path_buf(),
            source,
        })?
        .into_iter()
        .filter_map(|entry| {
            entry
                .file_type()
                .ok()
                .filter(|file_type| file_type.is_file())
                .map(|_| entry.path())
        })
        .collect::<Vec<_>>();
    paths.sort_by_key(|path| path_sort_key(path));
    Ok(paths)
}

fn existing_unique_directories(candidates: impl IntoIterator<Item = PathBuf>) -> Vec<PathBuf> {
    let mut seen = BTreeSet::new();
    candidates
        .into_iter()
        .filter_map(|path| fs::canonicalize(path).ok())
        .filter(|path| path.is_dir() && seen.insert(path.clone()))
        .collect()
}

fn canonical_regular_file(path: &Path, kind: &'static str) -> Result<PathBuf, DolphinError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| DolphinError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    if !metadata.file_type().is_file() {
        return Err(DolphinError::NotRegularFile {
            kind,
            path: path.to_path_buf(),
        });
    }
    fs::canonicalize(path).map_err(|source| DolphinError::Read {
        path: path.to_path_buf(),
        source,
    })
}

fn unicode_file_name(path: &Path) -> Result<&str, DolphinError> {
    path.file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| DolphinError::NonUnicodePath {
            path: path.to_path_buf(),
        })
}

fn path_sort_key(path: &Path) -> String {
    path.to_string_lossy().to_ascii_lowercase()
}

#[derive(Debug, Error)]
pub enum DolphinError {
    #[error("could not read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{kind} is not a regular file: {path}")]
    NotRegularFile { kind: &'static str, path: PathBuf },
    #[error("path is not valid Unicode: {path}")]
    NonUnicodePath { path: PathBuf },
    #[error("Dolphin Wii save contains a symlink or special entry: {path}")]
    UnsafeDirectoryEntry { path: PathBuf },
    #[error("could not run Dolphin tool {path}: {source}")]
    Tool {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_fixture_wad(path: &Path, title_id: [u8; 8]) {
        let mut file = fs::File::create(path).unwrap();
        let certificate_len = 32_u32;
        let ticket_len = 64_u32;
        file.write_all(&[0; 8]).unwrap();
        file.write_all(&certificate_len.to_be_bytes()).unwrap();
        file.write_all(&ticket_len.to_be_bytes()).unwrap();
        let offset =
            align_64(align_64(64 + u64::from(certificate_len)) + u64::from(ticket_len)) + 476;
        file.seek(SeekFrom::Start(offset)).unwrap();
        file.write_all(&title_id).unwrap();
        file.sync_all().unwrap();
    }

    #[test]
    fn discovers_gamecube_folder_card_and_state_files_without_windows_paths() {
        let directory = tempfile::tempdir().unwrap();
        let emulator_root = directory.path().join("Dolphin");
        let user = emulator_root.join("User");
        let folder = user.join("GC/USA/GALE01");
        let card_a = user.join("GC/USA/Card A");
        let states = user.join("StateSaves");
        fs::create_dir_all(&folder).unwrap();
        fs::create_dir_all(&card_a).unwrap();
        fs::create_dir_all(&states).unwrap();
        let emulator = emulator_root.join("Dolphin.exe");
        fs::write(&emulator, b"dolphin").unwrap();
        let content = directory.path().join("adventure.iso");
        fs::write(&content, b"GALE01 fixture disc bytes").unwrap();
        fs::write(folder.join("01-GALE-adventure.gci"), b"folder save").unwrap();
        fs::write(card_a.join("01-GALE-card.gci"), b"card save").unwrap();
        fs::write(card_a.join("01-OTHER-card.gci"), b"other game").unwrap();
        fs::write(states.join("GALE01.s00"), b"slot zero").unwrap();
        fs::write(states.join("gale01.S09"), b"slot nine").unwrap();
        fs::write(states.join("GALE01.s100"), b"unsupported slot").unwrap();

        let user_directories = default_dolphin_user_directories(&emulator);
        assert_eq!(
            user_directories,
            vec![fs::canonicalize(&user).expect("portable Dolphin user root")]
        );
        let saves = discover_dolphin_saves(
            &emulator,
            &[DolphinContent {
                game_id: "game".into(),
                additional_application_id: None,
                content_path: content,
                platform: "Nintendo GameCube".into(),
            }],
            &user_directories,
        )
        .unwrap();

        assert_eq!(saves.len(), 4);
        assert!(saves.iter().any(|save| {
            save.primary_path.ends_with("01-GALE-adventure.gci")
                && save.save_group_id.as_deref()
                    == Some("dolphin:gc:game:GALE01:Folder:01-GALE-adventure.gci")
                && save.display_chip_text.is_none()
        }));
        assert!(saves.iter().any(|save| {
            save.primary_path.ends_with("01-GALE-card.gci")
                && save.save_group_id.as_deref()
                    == Some("dolphin:gc:game:GALE01:CardA:01-GALE-card.gci")
                && save.display_chip_text.as_deref() == Some("Card A")
        }));
        assert!(saves.iter().any(|save| {
            save.slot() == Some(0) && save.save_group_id.as_deref() == Some("game-GALE01-State-0")
        }));
        assert!(saves.iter().any(|save| {
            save.slot() == Some(9) && save.save_group_id.as_deref() == Some("game-GALE01-State-9")
        }));
    }

    #[test]
    fn parses_wad_title_ids_and_dolphin_tool_output_like_1327() {
        let directory = tempfile::tempdir().unwrap();
        let wad = directory.path().join("channel.wad");
        write_fixture_wad(&wad, [0x00, 0x01, 0x00, 0x01, b'G', b'A', b'M', b'E']);

        assert_eq!(
            wad_title_id(&wad).unwrap(),
            Some([0x00, 0x01, 0x00, 0x01, b'G', b'A', b'M', b'E'])
        );
        assert_eq!(
            dolphin_disc_id(&wad, directory.path().join("Dolphin.exe").as_path()).unwrap(),
            Some("GAME".into())
        );
        assert_eq!(
            parse_dolphin_tool_disc_id("Format: RVZ\nGame ID: gale01\n"),
            Some("GALE01".into())
        );
        assert_eq!(
            parse_dolphin_tool_disc_id("Title ID: GAME\n"),
            Some("GAME".into())
        );
        assert_eq!(parse_dolphin_tool_disc_id("Game ID: ../bad"), None);
    }

    #[test]
    fn discovers_both_recovered_wii_disc_title_directories() {
        let directory = tempfile::tempdir().unwrap();
        let emulator_root = directory.path().join("Dolphin");
        let user = emulator_root.join("User");
        let disc_data = user.join("Wii/title/00010000/47414d45/data");
        let channel_data = user.join("Wii/title/00010004/47414d45/data");
        fs::create_dir_all(disc_data.join("nested/empty")).unwrap();
        fs::create_dir_all(&channel_data).unwrap();
        fs::write(disc_data.join("banner.bin"), b"banner").unwrap();
        fs::write(disc_data.join("nested/progress.dat"), b"progress").unwrap();
        fs::write(channel_data.join("channel.dat"), b"channel").unwrap();
        let emulator = emulator_root.join("Dolphin.exe");
        fs::write(&emulator, b"dolphin").unwrap();
        let content = directory.path().join("adventure.iso");
        fs::write(&content, b"GAME01 fixture disc bytes").unwrap();

        let saves = discover_dolphin_saves(
            &emulator,
            &[DolphinContent {
                game_id: "game".into(),
                additional_application_id: None,
                content_path: content,
                platform: "Nintendo Wii".into(),
            }],
            &[user],
        )
        .unwrap();

        assert_eq!(saves.len(), 2);
        let disc = saves
            .iter()
            .find(|save| {
                save.save_group_id.as_deref() == Some("dolphin:wii:game:00010000:47414d45")
            })
            .unwrap();
        assert_eq!(disc.display_chip_text.as_deref(), Some("Disc Save"));
        assert!(disc.primary_path.ends_with("00010000/47414d45/data"));
        assert_eq!(
            disc.container_save
                .as_ref()
                .and_then(|save| save.reported_file_size_bytes),
            Some(14)
        );
        assert_eq!(
            disc.container_save
                .as_ref()
                .map(|save| save.original_file_name.as_str()),
            Some("data")
        );
        assert!(disc
            .container_save
            .as_ref()
            .and_then(|save| save.reported_last_modified)
            .is_some());

        let channel = saves
            .iter()
            .find(|save| {
                save.save_group_id.as_deref() == Some("dolphin:wii:game:00010004:47414d45")
            })
            .unwrap();
        assert_eq!(channel.display_chip_text.as_deref(), Some("Channel Save"));
        assert_eq!(
            dolphin_wii_group_ids(channel.save_group_id.as_deref().unwrap()),
            Some(("00010004", "47414d45"))
        );
    }

    #[test]
    fn discovers_the_exact_high_and_low_wad_title_directory() {
        let directory = tempfile::tempdir().unwrap();
        let emulator_root = directory.path().join("Dolphin");
        let user = emulator_root.join("User");
        let data = user.join("Wii/title/00010002/41424344/data");
        fs::create_dir_all(&data).unwrap();
        fs::write(data.join("progress.dat"), b"wad progress").unwrap();
        let emulator = emulator_root.join("Dolphin.exe");
        fs::write(&emulator, b"dolphin").unwrap();
        let wad = directory.path().join("channel.wad");
        write_fixture_wad(&wad, [0x00, 0x01, 0x00, 0x02, b'A', b'B', b'C', b'D']);

        let saves = discover_dolphin_saves(
            &emulator,
            &[DolphinContent {
                game_id: "channel".into(),
                additional_application_id: None,
                content_path: wad,
                platform: "Nintendo Wii".into(),
            }],
            &[user],
        )
        .unwrap();

        assert_eq!(saves.len(), 1);
        assert_eq!(
            saves[0].save_group_id.as_deref(),
            Some("dolphin:wii:channel:00010002:41424344")
        );
        assert!(saves[0].display_chip_text.is_none());
        assert!(saves[0].primary_path.ends_with("00010002/41424344/data"));
    }

    #[cfg(unix)]
    #[test]
    fn refuses_symlinks_inside_wii_save_directories() {
        use std::os::unix::fs::symlink;

        let directory = tempfile::tempdir().unwrap();
        let emulator_root = directory.path().join("Dolphin");
        let user = emulator_root.join("User");
        let data = user.join("Wii/title/00010000/47414d45/data");
        fs::create_dir_all(&data).unwrap();
        fs::write(data.join("progress.dat"), b"progress").unwrap();
        symlink(data.join("progress.dat"), data.join("linked.dat")).unwrap();
        let emulator = emulator_root.join("Dolphin.exe");
        fs::write(&emulator, b"dolphin").unwrap();
        let content = directory.path().join("adventure.iso");
        fs::write(&content, b"GAME01 fixture disc bytes").unwrap();

        assert!(matches!(
            discover_dolphin_saves(
                &emulator,
                &[DolphinContent {
                    game_id: "game".into(),
                    additional_application_id: None,
                    content_path: content,
                    platform: "Nintendo Wii".into(),
                }],
                &[user],
            ),
            Err(DolphinError::UnsafeDirectoryEntry { .. })
        ));
    }
}
