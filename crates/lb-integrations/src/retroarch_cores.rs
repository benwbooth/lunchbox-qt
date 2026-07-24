use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use lb_platform::{split_windows_command_line, HostPathResolver, LaunchPathResolver};
use thiserror::Error;

// Frozen from the Retroarch/EmulatorPlatforms rows in LaunchBox 13.27's
// LaunchBox.Metadata.db. Runtime behavior never depends on the proprietary DB.
const RETROARCH_CORE_CATALOG: &str = include_str!("retroarch_core_catalog.tsv");
const RETROARCH_CORE_CATALOG_HEADER: &str = "platform\tcore\trecommended";
const MAX_RETROARCH_CONFIGURATION_BYTES: u64 = 4 * 1024 * 1024;
const MAX_RETROARCH_CORE_DIRECTORY_ENTRIES: usize = 4_096;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetroArchCoreCatalogEntry {
    pub platform: String,
    pub core: String,
    pub recommended: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetroArchCorePlatform {
    Windows,
    Linux,
    Macos,
}

impl RetroArchCorePlatform {
    pub const fn current() -> Option<Self> {
        if cfg!(target_os = "windows") {
            Some(Self::Windows)
        } else if cfg!(target_os = "macos") {
            Some(Self::Macos)
        } else if cfg!(any(target_os = "linux", target_os = "freebsd")) {
            Some(Self::Linux)
        } else {
            None
        }
    }

    pub const fn extension(self) -> &'static str {
        match self {
            Self::Windows => "dll",
            Self::Linux => "so",
            Self::Macos => "dylib",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetroArchCore {
    pub name: String,
    pub path: PathBuf,
    pub command_path: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetroArchCoreInventory {
    pub application_path: PathBuf,
    pub configuration_path: Option<PathBuf>,
    pub core_directory: Option<PathBuf>,
    pub searched_directories: Vec<PathBuf>,
    pub cores: Vec<RetroArchCore>,
    pub unsafe_entry_count: usize,
}

impl RetroArchCoreInventory {
    pub fn core(&self, name: &str) -> Option<&RetroArchCore> {
        self.cores
            .iter()
            .find(|core| core.name.eq_ignore_ascii_case(name))
    }
}

pub fn retroarch_core_catalog() -> Result<Vec<RetroArchCoreCatalogEntry>, RetroArchCoreError> {
    let mut lines = RETROARCH_CORE_CATALOG.lines();
    if lines.next() != Some(RETROARCH_CORE_CATALOG_HEADER) {
        return Err(RetroArchCoreError::InvalidCatalog {
            line: 1,
            reason: "unexpected header".into(),
        });
    }

    let mut platforms = BTreeSet::new();
    let mut entries = Vec::new();
    for (index, line) in lines.enumerate() {
        let line_number = index + 2;
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != 3 {
            return Err(RetroArchCoreError::InvalidCatalog {
                line: line_number,
                reason: format!("expected 3 tab-separated columns, found {}", columns.len()),
            });
        }
        let platform = columns[0].trim();
        let core = columns[1].trim();
        if platform.is_empty() {
            return Err(RetroArchCoreError::InvalidCatalog {
                line: line_number,
                reason: "platform is empty".into(),
            });
        }
        validate_core_name(core).map_err(|reason| RetroArchCoreError::InvalidCatalog {
            line: line_number,
            reason,
        })?;
        if !platforms.insert(platform.to_ascii_lowercase()) {
            return Err(RetroArchCoreError::InvalidCatalog {
                line: line_number,
                reason: format!("platform {platform} occurs more than once"),
            });
        }
        let recommended = match columns[2] {
            value if value.eq_ignore_ascii_case("true") => true,
            value if value.eq_ignore_ascii_case("false") => false,
            value => {
                return Err(RetroArchCoreError::InvalidCatalog {
                    line: line_number,
                    reason: format!("recommended value is not TRUE or FALSE: {value}"),
                })
            }
        };
        entries.push(RetroArchCoreCatalogEntry {
            platform: platform.to_string(),
            core: core.to_string(),
            recommended,
        });
    }
    Ok(entries)
}

pub fn suggested_retroarch_core(
    platform: &str,
) -> Result<Option<RetroArchCoreCatalogEntry>, RetroArchCoreError> {
    Ok(retroarch_core_catalog()?
        .into_iter()
        .find(|entry| entry.platform.eq_ignore_ascii_case(platform)))
}

pub fn inspect_retroarch_cores(
    application_path: &Path,
    configuration_candidates: &[PathBuf],
    fallback_directories: &[PathBuf],
    resolver: &HostPathResolver,
    platform: RetroArchCorePlatform,
) -> Result<RetroArchCoreInventory, RetroArchCoreError> {
    let application_path = canonical_regular_file(application_path, "RetroArch executable")?;
    let application_directory = application_path
        .parent()
        .ok_or_else(|| RetroArchCoreError::MissingApplicationDirectory {
            path: application_path.clone(),
        })?
        .to_path_buf();
    let (configuration_path, configured_directory) =
        configured_core_directory(&application_directory, configuration_candidates, resolver)?;

    let candidates = configured_directory
        .clone()
        .map(|directory| vec![directory])
        .unwrap_or_else(|| fallback_directories.to_vec());
    let mut searched_directories = Vec::new();
    let mut selected = None;
    for candidate in candidates {
        if searched_directories.contains(&candidate) {
            continue;
        }
        searched_directories.push(candidate.clone());
        let metadata = match fs::symlink_metadata(&candidate) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(source) => {
                return Err(RetroArchCoreError::Read {
                    path: candidate,
                    source,
                })
            }
        };
        if metadata.file_type().is_symlink() {
            return Err(RetroArchCoreError::UnsafeCoreDirectory { path: candidate });
        }
        if !metadata.is_dir() {
            return Err(RetroArchCoreError::CorePathNotDirectory { path: candidate });
        }
        selected =
            Some(
                fs::canonicalize(&candidate).map_err(|source| RetroArchCoreError::Read {
                    path: candidate,
                    source,
                })?,
            );
        break;
    }

    let Some(core_directory) = selected else {
        return Ok(RetroArchCoreInventory {
            application_path,
            configuration_path,
            core_directory: configured_directory,
            searched_directories,
            cores: Vec::new(),
            unsafe_entry_count: 0,
        });
    };
    let (cores, unsafe_entry_count) =
        scan_core_directory(&application_directory, &core_directory, platform)?;
    Ok(RetroArchCoreInventory {
        application_path,
        configuration_path,
        core_directory: Some(core_directory),
        searched_directories,
        cores,
        unsafe_entry_count,
    })
}

pub fn default_retroarch_core_directories(
    application_path: &Path,
    platform: RetroArchCorePlatform,
) -> Vec<PathBuf> {
    let application_directory = application_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(application_path);
    let mut paths = vec![application_directory.join("cores")];

    if let Some(file_name) = application_path.file_name() {
        let mut appimage_home = file_name.to_os_string();
        appimage_home.push(".home");
        paths.push(
            application_directory
                .join(appimage_home)
                .join(".config/retroarch/cores"),
        );
    }

    if application_directory
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("MacOS"))
    {
        if let Some(contents) = application_directory.parent() {
            paths.push(contents.join("Frameworks"));
            if let Some(bundle) = contents.parent() {
                paths.push(bundle.join("Frameworks"));
                paths.push(bundle.join("modules"));
            }
        }
    }

    if let Some(xdg) = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
    {
        paths.push(xdg.join("retroarch/cores"));
    }
    if let Some(home) = home_directory() {
        paths.push(home.join(".config/retroarch/cores"));
        if platform == RetroArchCorePlatform::Macos {
            paths.push(home.join("Library/Application Support/RetroArch/cores"));
        }
    }
    if let Some(app_data) = env::var_os("APPDATA")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
    {
        paths.push(app_data.join("RetroArch/cores"));
        paths.push(app_data.join("cores"));
    }
    paths.dedup();
    paths
}

pub fn command_line_with_retroarch_core(
    current: Option<&str>,
    core_command_path: &str,
) -> Result<String, RetroArchCoreError> {
    validate_command_path(core_command_path)?;
    let current = current.unwrap_or_default().trim();
    let mut arguments = split_windows_command_line(current);
    let mut core_argument = None;
    for (index, argument) in arguments.iter().enumerate() {
        let is_core_argument = argument.eq_ignore_ascii_case("-l")
            || argument.eq_ignore_ascii_case("--libretro")
            || argument
                .get(.."--libretro=".len())
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case("--libretro="));
        if is_core_argument && core_argument.replace(index).is_some() {
            return Err(RetroArchCoreError::AmbiguousCoreArgument);
        }
    }

    match core_argument {
        Some(index) if arguments[index].contains('=') => {
            arguments[index] = format!("--libretro={core_command_path}");
        }
        Some(index) => {
            let value_index = index + 1;
            if value_index >= arguments.len() || arguments[value_index].starts_with('-') {
                return Err(RetroArchCoreError::MissingCoreArgumentValue);
            }
            arguments[value_index] = core_command_path.to_string();
        }
        None => {
            let was_empty = arguments.is_empty();
            arguments.insert(0, core_command_path.to_string());
            arguments.insert(0, "-L".into());
            if was_empty {
                arguments.push("-f".into());
            }
        }
    }

    Ok(arguments
        .iter()
        .map(|argument| quote_windows_argument(argument))
        .collect::<Vec<_>>()
        .join(" "))
}

fn configured_core_directory(
    application_directory: &Path,
    configuration_candidates: &[PathBuf],
    resolver: &HostPathResolver,
) -> Result<(Option<PathBuf>, Option<PathBuf>), RetroArchCoreError> {
    for configuration in configuration_candidates
        .iter()
        .filter(|path| path.is_absolute())
    {
        let metadata = match fs::symlink_metadata(configuration) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(source) => {
                return Err(RetroArchCoreError::ReadConfiguration {
                    path: configuration.clone(),
                    source,
                })
            }
        };
        if metadata.file_type().is_symlink() {
            return Err(RetroArchCoreError::UnsafeConfiguration {
                path: configuration.clone(),
            });
        }
        if !metadata.is_file() {
            continue;
        }
        if metadata.len() > MAX_RETROARCH_CONFIGURATION_BYTES {
            return Err(RetroArchCoreError::ConfigurationTooLarge {
                path: configuration.clone(),
            });
        }
        let contents = fs::read_to_string(configuration).map_err(|source| {
            RetroArchCoreError::ReadConfiguration {
                path: configuration.clone(),
                source,
            }
        })?;
        let configured = configured_libretro_directory(&contents);
        let directory = match configured {
            Some(value)
                if !value.trim().is_empty() && !value.trim().eq_ignore_ascii_case("default") =>
            {
                Some(resolve_core_directory(
                    application_directory,
                    value,
                    resolver,
                )?)
            }
            _ => None,
        };
        return Ok((Some(configuration.clone()), directory));
    }
    Ok((None, None))
}

fn configured_libretro_directory(contents: &str) -> Option<&str> {
    contents.lines().find_map(|line| {
        let line = line.trim_start();
        if line.starts_with('#') {
            return None;
        }
        let (key, value) = line.split_once('=')?;
        key.trim()
            .eq_ignore_ascii_case("libretro_directory")
            .then_some(value.trim())
    })
}

fn resolve_core_directory(
    application_directory: &Path,
    configured: &str,
    resolver: &HostPathResolver,
) -> Result<PathBuf, RetroArchCoreError> {
    let value = configured
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(configured)
        .trim();
    if value.is_empty() {
        return Err(RetroArchCoreError::InvalidConfiguredDirectory);
    }
    if value == "~" {
        return home_directory().ok_or(RetroArchCoreError::MissingHomeDirectory);
    }
    if let Some(remainder) = value
        .strip_prefix("~/")
        .or_else(|| value.strip_prefix("~\\"))
    {
        return Ok(home_directory()
            .ok_or(RetroArchCoreError::MissingHomeDirectory)?
            .join(native_relative_path(remainder)?));
    }
    let value = value
        .strip_prefix(":\\")
        .or_else(|| value.strip_prefix(":/"))
        .unwrap_or(value);
    resolver
        .resolve(application_directory, value)
        .map_err(|source| RetroArchCoreError::ConfiguredDirectory {
            value: configured.to_string(),
            reason: source.to_string(),
        })
}

fn scan_core_directory(
    application_directory: &Path,
    core_directory: &Path,
    platform: RetroArchCorePlatform,
) -> Result<(Vec<RetroArchCore>, usize), RetroArchCoreError> {
    let entries = fs::read_dir(core_directory).map_err(|source| RetroArchCoreError::Read {
        path: core_directory.to_path_buf(),
        source,
    })?;
    let mut names = BTreeSet::new();
    let mut cores = Vec::new();
    let mut unsafe_entry_count = 0;
    for (index, entry) in entries.enumerate() {
        if index >= MAX_RETROARCH_CORE_DIRECTORY_ENTRIES {
            return Err(RetroArchCoreError::TooManyCoreEntries {
                path: core_directory.to_path_buf(),
                maximum: MAX_RETROARCH_CORE_DIRECTORY_ENTRIES,
            });
        }
        let entry = entry.map_err(|source| RetroArchCoreError::Read {
            path: core_directory.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|source| RetroArchCoreError::Read {
                path: path.clone(),
                source,
            })?;
        if file_type.is_symlink() {
            unsafe_entry_count += 1;
            continue;
        }
        if !file_type.is_file()
            || !path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case(platform.extension()))
        {
            continue;
        }
        let Some(name) = path.file_stem().and_then(|stem| stem.to_str()) else {
            unsafe_entry_count += 1;
            continue;
        };
        if validate_core_name(name).is_err() {
            unsafe_entry_count += 1;
            continue;
        }
        let key = name.to_ascii_lowercase();
        if !names.insert(key) {
            return Err(RetroArchCoreError::AmbiguousCoreName {
                name: name.to_string(),
                path: core_directory.to_path_buf(),
            });
        }
        let canonical = fs::canonicalize(&path).map_err(|source| RetroArchCoreError::Read {
            path: path.clone(),
            source,
        })?;
        cores.push(RetroArchCore {
            name: name.to_string(),
            command_path: command_path(application_directory, &canonical)?,
            path: canonical,
        });
    }
    cores.sort_by(|left, right| {
        left.name
            .to_ascii_lowercase()
            .cmp(&right.name.to_ascii_lowercase())
            .then_with(|| left.name.cmp(&right.name))
    });
    Ok((cores, unsafe_entry_count))
}

fn command_path(
    application_directory: &Path,
    core_path: &Path,
) -> Result<String, RetroArchCoreError> {
    let path = core_path
        .strip_prefix(application_directory)
        .unwrap_or(core_path);
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => {
                parts.push(prefix.as_os_str().to_string_lossy().into_owned());
            }
            Component::RootDir => {
                if parts.is_empty() {
                    parts.push(String::new());
                }
            }
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(RetroArchCoreError::UnsafeCommandPath {
                    path: path.to_path_buf(),
                })
            }
            Component::Normal(part) => {
                let part = part
                    .to_str()
                    .ok_or_else(|| RetroArchCoreError::NonUnicodePath {
                        path: path.to_path_buf(),
                    })?;
                parts.push(part.to_string());
            }
        }
    }
    let command_path = parts.join("/");
    validate_command_path(&command_path)?;
    Ok(command_path)
}

fn canonical_regular_file(path: &Path, kind: &'static str) -> Result<PathBuf, RetroArchCoreError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| RetroArchCoreError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(RetroArchCoreError::NotRegularFile {
            kind,
            path: path.to_path_buf(),
        });
    }
    fs::canonicalize(path).map_err(|source| RetroArchCoreError::Read {
        path: path.to_path_buf(),
        source,
    })
}

fn validate_core_name(core: &str) -> Result<(), String> {
    if !core.ends_with("_libretro") {
        return Err(format!("core name does not end in _libretro: {core}"));
    }
    if !core
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'+'))
    {
        return Err(format!("core name contains unsafe characters: {core}"));
    }
    Ok(())
}

fn validate_command_path(path: &str) -> Result<(), RetroArchCoreError> {
    if path.is_empty()
        || path
            .chars()
            .any(|character| matches!(character, '\r' | '\n' | '\0' | '"'))
        || Path::new(path)
            .components()
            .any(|component| component == Component::ParentDir)
    {
        return Err(RetroArchCoreError::UnsafeCommandPath {
            path: PathBuf::from(path),
        });
    }
    Ok(())
}

fn native_relative_path(path: &str) -> Result<PathBuf, RetroArchCoreError> {
    let mut output = PathBuf::new();
    for component in path.split(['/', '\\']) {
        match component {
            "" | "." => {}
            ".." => {
                return Err(RetroArchCoreError::UnsafeCommandPath {
                    path: PathBuf::from(path),
                })
            }
            value => output.push(value),
        }
    }
    Ok(output)
}

fn quote_windows_argument(argument: &str) -> String {
    if !argument.is_empty()
        && !argument
            .bytes()
            .any(|byte| byte.is_ascii_whitespace() || byte == b'"')
    {
        return argument.to_string();
    }

    let mut output = String::from("\"");
    let mut backslashes = 0;
    for character in argument.chars() {
        if character == '\\' {
            backslashes += 1;
            continue;
        }
        if character == '"' {
            output.extend(std::iter::repeat_n('\\', backslashes * 2 + 1));
            output.push('"');
        } else {
            output.extend(std::iter::repeat_n('\\', backslashes));
            output.push(character);
        }
        backslashes = 0;
    }
    output.extend(std::iter::repeat_n('\\', backslashes * 2));
    output.push('"');
    output
}

fn home_directory() -> Option<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
}

#[derive(Debug, Error)]
pub enum RetroArchCoreError {
    #[error("RetroArch core catalog is invalid at line {line}: {reason}")]
    InvalidCatalog { line: usize, reason: String },
    #[error("could not inspect {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("RetroArch {kind} is not a safe regular file: {path}")]
    NotRegularFile { kind: &'static str, path: PathBuf },
    #[error("RetroArch executable has no application directory: {path}")]
    MissingApplicationDirectory { path: PathBuf },
    #[error("RetroArch configuration is a symbolic link and was refused: {path}")]
    UnsafeConfiguration { path: PathBuf },
    #[error("RetroArch configuration exceeds the safety limit: {path}")]
    ConfigurationTooLarge { path: PathBuf },
    #[error("could not read RetroArch configuration {path}: {source}")]
    ReadConfiguration { path: PathBuf, source: io::Error },
    #[error("RetroArch libretro_directory is empty")]
    InvalidConfiguredDirectory,
    #[error("RetroArch libretro_directory {value:?} cannot be resolved: {reason}")]
    ConfiguredDirectory { value: String, reason: String },
    #[error("the home directory required by RetroArch libretro_directory is unavailable")]
    MissingHomeDirectory,
    #[error("RetroArch core directory is a symbolic link and was refused: {path}")]
    UnsafeCoreDirectory { path: PathBuf },
    #[error("RetroArch core path is not a directory: {path}")]
    CorePathNotDirectory { path: PathBuf },
    #[error("RetroArch core directory {path} exceeds {maximum} entries")]
    TooManyCoreEntries { path: PathBuf, maximum: usize },
    #[error("RetroArch core name {name} is ambiguous in {path}")]
    AmbiguousCoreName { name: String, path: PathBuf },
    #[error("RetroArch core path is not Unicode: {path}")]
    NonUnicodePath { path: PathBuf },
    #[error("RetroArch core command path is unsafe: {path}")]
    UnsafeCommandPath { path: PathBuf },
    #[error("RetroArch command line contains more than one core argument")]
    AmbiguousCoreArgument,
    #[error("RetroArch command line has a core flag without a value")]
    MissingCoreArgumentValue,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_1327_catalog_has_all_56_unique_platform_suggestions() {
        let entries = retroarch_core_catalog().unwrap();
        assert_eq!(entries.len(), 56);
        assert_eq!(entries.iter().filter(|entry| entry.recommended).count(), 54);
        assert_eq!(
            suggested_retroarch_core("Sega CD").unwrap(),
            Some(RetroArchCoreCatalogEntry {
                platform: "Sega CD".into(),
                core: "genesis_plus_gx_libretro".into(),
                recommended: true,
            })
        );
        assert_eq!(
            suggested_retroarch_core("Atari Jaguar")
                .unwrap()
                .map(|entry| (entry.core, entry.recommended)),
            Some(("virtualjaguar_libretro".into(), false))
        );
        assert!(suggested_retroarch_core("Unknown Console")
            .unwrap()
            .is_none());
    }

    #[test]
    fn scans_only_safe_native_core_files_from_configured_directory() {
        let directory = tempfile::tempdir().unwrap();
        let application = directory.path().join("RetroArch/retroarch");
        let cores = directory.path().join("Configured Cores");
        fs::create_dir_all(application.parent().unwrap()).unwrap();
        fs::create_dir_all(&cores).unwrap();
        fs::write(&application, b"frontend").unwrap();
        fs::write(
            application.parent().unwrap().join("retroarch.cfg"),
            format!("libretro_directory = \"{}\"\n", cores.display()),
        )
        .unwrap();
        fs::write(cores.join("snes9x_libretro.so"), b"core").unwrap();
        fs::write(cores.join("wrong_libretro.dll"), b"foreign").unwrap();
        fs::write(cores.join("notes.txt"), b"notes").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(
            cores.join("snes9x_libretro.so"),
            cores.join("linked_libretro.so"),
        )
        .unwrap();

        let inventory = inspect_retroarch_cores(
            &application,
            &[application.parent().unwrap().join("retroarch.cfg")],
            &[application.parent().unwrap().join("cores")],
            &HostPathResolver::default(),
            RetroArchCorePlatform::Linux,
        )
        .unwrap();
        let configuration = application.parent().unwrap().join("retroarch.cfg");
        assert_eq!(
            inventory.configuration_path.as_deref(),
            Some(configuration.as_path())
        );
        let canonical_cores = cores.canonicalize().unwrap();
        assert_eq!(
            inventory.core_directory.as_deref(),
            Some(canonical_cores.as_path())
        );
        assert_eq!(inventory.cores.len(), 1);
        assert_eq!(inventory.cores[0].name, "snes9x_libretro");
        assert_eq!(
            inventory.cores[0].command_path,
            cores
                .join("snes9x_libretro.so")
                .to_string_lossy()
                .replace('\\', "/")
        );
        #[cfg(unix)]
        assert_eq!(inventory.unsafe_entry_count, 1);
    }

    #[test]
    fn appimage_home_is_a_portable_relative_core_location() {
        let directory = tempfile::tempdir().unwrap();
        let application = directory.path().join("RetroArch/RetroArch.AppImage");
        let cores = directory
            .path()
            .join("RetroArch/RetroArch.AppImage.home/.config/retroarch/cores");
        fs::create_dir_all(&cores).unwrap();
        fs::write(&application, b"frontend").unwrap();
        fs::write(cores.join("mesen_libretro.so"), b"core").unwrap();

        let fallbacks =
            default_retroarch_core_directories(&application, RetroArchCorePlatform::Linux);
        let inventory = inspect_retroarch_cores(
            &application,
            &[],
            &fallbacks,
            &HostPathResolver::default(),
            RetroArchCorePlatform::Linux,
        )
        .unwrap();
        assert_eq!(
            inventory.cores[0].command_path,
            "RetroArch.AppImage.home/.config/retroarch/cores/mesen_libretro.so"
        );
    }

    #[test]
    fn macos_bundle_inventory_accepts_only_dylib_cores() {
        let directory = tempfile::tempdir().unwrap();
        let application = directory
            .path()
            .join("RetroArch.app/Contents/MacOS/RetroArch");
        let cores = directory.path().join("RetroArch.app/modules");
        fs::create_dir_all(application.parent().unwrap()).unwrap();
        fs::create_dir_all(&cores).unwrap();
        fs::write(&application, b"frontend").unwrap();
        fs::write(cores.join("snes9x_libretro.dylib"), b"mac core").unwrap();
        fs::write(cores.join("foreign_libretro.so"), b"linux core").unwrap();

        let fallbacks =
            default_retroarch_core_directories(&application, RetroArchCorePlatform::Macos);
        let inventory = inspect_retroarch_cores(
            &application,
            &[],
            &fallbacks,
            &HostPathResolver::default(),
            RetroArchCorePlatform::Macos,
        )
        .unwrap();
        assert_eq!(inventory.cores.len(), 1);
        assert_eq!(inventory.cores[0].name, "snes9x_libretro");
        assert!(inventory.cores[0]
            .command_path
            .ends_with("RetroArch.app/modules/snes9x_libretro.dylib"));
    }

    #[cfg(unix)]
    #[test]
    fn refuses_symlinked_configuration_and_core_directories() {
        use std::os::unix::fs::symlink;

        let directory = tempfile::tempdir().unwrap();
        let application = directory.path().join("retroarch");
        let real_configuration = directory.path().join("real.cfg");
        let linked_configuration = directory.path().join("retroarch.cfg");
        fs::write(&application, b"frontend").unwrap();
        fs::write(&real_configuration, "libretro_directory = \"cores\"\n").unwrap();
        symlink(&real_configuration, &linked_configuration).unwrap();
        assert!(matches!(
            inspect_retroarch_cores(
                &application,
                &[linked_configuration],
                &[],
                &HostPathResolver::default(),
                RetroArchCorePlatform::Linux,
            ),
            Err(RetroArchCoreError::UnsafeConfiguration { .. })
        ));

        let real_cores = directory.path().join("real-cores");
        let linked_cores = directory.path().join("cores");
        fs::create_dir(&real_cores).unwrap();
        symlink(&real_cores, &linked_cores).unwrap();
        assert!(matches!(
            inspect_retroarch_cores(
                &application,
                &[],
                &[linked_cores],
                &HostPathResolver::default(),
                RetroArchCorePlatform::Linux,
            ),
            Err(RetroArchCoreError::UnsafeCoreDirectory { .. })
        ));
    }

    #[test]
    fn replaces_only_the_semantic_core_argument_and_round_trips_quotes() {
        let changed = command_line_with_retroarch_core(
            Some(r#"-f -L "cores\old_libretro.dll" --appendconfig "My Config.cfg""#),
            "cores/new_libretro.dll",
        )
        .unwrap();
        assert_eq!(
            split_windows_command_line(&changed),
            [
                "-f",
                "-L",
                "cores/new_libretro.dll",
                "--appendconfig",
                "My Config.cfg"
            ]
        );
        let created =
            command_line_with_retroarch_core(None, "Portable Home/cores/new_libretro.so").unwrap();
        assert_eq!(
            split_windows_command_line(&created),
            ["-L", "Portable Home/cores/new_libretro.so", "-f"]
        );
        let long = command_line_with_retroarch_core(
            Some("--libretro=cores/old_libretro.so --verbose"),
            "cores/new_libretro.so",
        )
        .unwrap();
        assert_eq!(
            split_windows_command_line(&long),
            ["--libretro=cores/new_libretro.so", "--verbose"]
        );
    }

    #[test]
    fn rejects_ambiguous_or_incomplete_core_arguments() {
        assert!(matches!(
            command_line_with_retroarch_core(
                Some("-L one_libretro.so -L two_libretro.so"),
                "three_libretro.so",
            ),
            Err(RetroArchCoreError::AmbiguousCoreArgument)
        ));
        assert!(matches!(
            command_line_with_retroarch_core(Some("-f -L"), "three_libretro.so"),
            Err(RetroArchCoreError::MissingCoreArgumentValue)
        ));
        assert!(matches!(
            command_line_with_retroarch_core(Some("-L --verbose"), "three_libretro.so"),
            Err(RetroArchCoreError::MissingCoreArgumentValue)
        ));
    }
}
