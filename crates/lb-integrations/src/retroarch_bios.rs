use crate::pcsx2_bios::{audit_bios_file, BiosFileState};
use crate::retroarch::retroarch_core_name;
use lb_platform::{HostPathResolver, LaunchPathError, LaunchPathResolver};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

const RETROARCH_BIOS_CATALOG: &str = include_str!("retroarch_bios_catalog.tsv");
const RETROARCH_BIOS_CATALOG_HEADER: &str =
    "CoreName\tPlatform\tBiosDesc\tBiosPath\tMD5\tRequired\tGroupId\tGroupDesc\tIsGroupRequired\tGroupRequirements(None|Any|All)";
const MAX_RETROARCH_CONFIGURATION_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetroArchBiosGroupRule {
    None,
    Any,
    All,
}

impl RetroArchBiosGroupRule {
    pub const fn all_items_required(self) -> bool {
        matches!(self, Self::All)
    }

    pub const fn id(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Any => "any",
            Self::All => "all",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetroArchBiosGroup {
    pub id: String,
    pub description: String,
    pub required: bool,
    pub rule: RetroArchBiosGroupRule,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetroArchBiosRequirement {
    pub core: String,
    pub platform: Option<String>,
    pub description: String,
    pub relative_path: String,
    pub md5: Option<String>,
    pub required: bool,
    pub group: Option<RetroArchBiosGroup>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetroArchBiosTarget {
    pub platform: String,
    pub command_line: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetroArchBiosTargetAudit {
    pub platform: String,
    pub core: String,
    pub requirement_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetroArchBiosGroupAudit {
    pub id: String,
    pub description: String,
    pub required: bool,
    pub rule: RetroArchBiosGroupRule,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetroArchBiosFileAudit {
    pub group_id: String,
    pub requirement: RetroArchBiosRequirement,
    pub path: PathBuf,
    pub state: BiosFileState,
    pub actual_md5: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetroArchBiosLocationSource {
    ApplicationConfiguration,
    NativeConfiguration,
    ApplicationDefault,
}

impl RetroArchBiosLocationSource {
    pub const fn label(self) -> &'static str {
        match self {
            Self::ApplicationConfiguration => "application-local RetroArch configuration",
            Self::NativeConfiguration => "host-native RetroArch configuration",
            Self::ApplicationDefault => "RetroArch application-directory default",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetroArchBiosAudit {
    pub application_directory: PathBuf,
    pub system_directory: PathBuf,
    pub configuration_path: Option<PathBuf>,
    pub location_source: RetroArchBiosLocationSource,
    pub targets: Vec<RetroArchBiosTargetAudit>,
    pub groups: Vec<RetroArchBiosGroupAudit>,
    pub files: Vec<RetroArchBiosFileAudit>,
}

impl RetroArchBiosAudit {
    pub fn files_for_group<'a>(
        &'a self,
        group_id: &'a str,
    ) -> impl Iterator<Item = &'a RetroArchBiosFileAudit> + 'a {
        self.files
            .iter()
            .filter(move |file| file.group_id == group_id)
    }

    pub fn group_satisfied(&self, group: &RetroArchBiosGroupAudit) -> bool {
        if !group.required || group.rule == RetroArchBiosGroupRule::None {
            return true;
        }
        let mut files = self.files_for_group(&group.id).peekable();
        match group.rule {
            RetroArchBiosGroupRule::None => true,
            RetroArchBiosGroupRule::Any => files.any(|file| file.state == BiosFileState::Valid),
            RetroArchBiosGroupRule::All => {
                files.peek().is_some() && files.all(|file| file.state == BiosFileState::Valid)
            }
        }
    }

    pub fn ready(&self) -> bool {
        self.groups.iter().all(|group| self.group_satisfied(group))
    }

    pub fn valid_count(&self) -> usize {
        self.count_state(BiosFileState::Valid)
    }

    pub fn mismatch_count(&self) -> usize {
        self.count_state(BiosFileState::HashMismatch)
    }

    pub fn unsafe_count(&self) -> usize {
        self.count_state(BiosFileState::UnsafeEntry)
    }

    pub fn unreadable_count(&self) -> usize {
        self.count_state(BiosFileState::Unreadable)
    }

    pub fn missing_count(&self) -> usize {
        self.count_state(BiosFileState::Missing)
    }

    fn count_state(&self, expected: BiosFileState) -> usize {
        self.files
            .iter()
            .filter(|file| file.state == expected)
            .count()
    }
}

#[derive(Debug, Error)]
pub enum RetroArchBiosError {
    #[error("RetroArch executable path is not an absolute native host path: {path}")]
    NonAbsoluteExecutable { path: PathBuf },
    #[error("RetroArch BIOS catalog line {line} is malformed: {reason}")]
    InvalidCatalog { line: usize, reason: String },
    #[error("RetroArch BIOS catalog contains a duplicate row at line {line}")]
    DuplicateCatalogRow { line: usize },
    #[error("no configured RetroArch platform mapping identifies a libretro core")]
    NoConfiguredCore,
    #[error("RetroArch configuration is a symlink and will not be followed: {path}")]
    UnsafeConfiguration { path: PathBuf },
    #[error("RetroArch configuration is too large to audit safely: {path}")]
    ConfigurationTooLarge { path: PathBuf },
    #[error("could not read RetroArch configuration {path}: {source}")]
    ReadConfiguration { path: PathBuf, source: io::Error },
    #[error(
        "RetroArch system_directory is set to \"default\", which resolves per content and has no single auditable BIOS root"
    )]
    DynamicSystemDirectory,
    #[error("RetroArch system_directory uses ~ but no absolute home directory is available")]
    MissingHomeDirectory,
    #[error("could not resolve RetroArch system_directory value {value:?}: {source}")]
    ResolveSystemDirectory {
        value: String,
        source: LaunchPathError,
    },
}

#[derive(Clone, Debug)]
struct RetroArchBiosLocation {
    application_directory: PathBuf,
    system_directory: PathBuf,
    configuration_path: Option<PathBuf>,
    source: RetroArchBiosLocationSource,
}

/// Audits every BIOS requirement selected by the configured RetroArch
/// platform/core mappings. The executable, configuration, system directory,
/// and firmware are never executed or changed.
pub fn audit_retroarch_bios(
    emulator_application_path: &Path,
    targets: &[RetroArchBiosTarget],
    configuration_candidates: &[PathBuf],
    path_resolver: &HostPathResolver,
) -> Result<RetroArchBiosAudit, RetroArchBiosError> {
    if !emulator_application_path.is_absolute() {
        return Err(RetroArchBiosError::NonAbsoluteExecutable {
            path: emulator_application_path.to_path_buf(),
        });
    }
    let catalog = retroarch_bios_catalog()?;
    let location = locate_retroarch_bios(
        emulator_application_path,
        configuration_candidates,
        path_resolver,
    )?;
    let selected_targets = selected_targets(targets)?;
    let mut target_audits = Vec::with_capacity(selected_targets.len());
    let mut groups = Vec::<RetroArchBiosGroupAudit>::new();
    let mut files = Vec::new();

    for (platform, core) in selected_targets {
        let requirements = catalog
            .iter()
            .filter(|requirement| {
                requirement.core.eq_ignore_ascii_case(&core)
                    && requirement
                        .platform
                        .as_ref()
                        .is_none_or(|required_platform| {
                            required_platform.eq_ignore_ascii_case(&platform)
                        })
            })
            .cloned()
            .collect::<Vec<_>>();
        target_audits.push(RetroArchBiosTargetAudit {
            platform: platform.clone(),
            core: core.clone(),
            requirement_count: requirements.len(),
        });
        let mut target_groups = BTreeMap::<String, usize>::new();
        for requirement in requirements {
            let (group_id, group_description, group_required, group_rule) =
                requirement_group(&platform, &core, &requirement);
            match target_groups.get(&group_id).copied() {
                Some(index) => {
                    let existing = &groups[index];
                    if existing.description != group_description
                        || existing.required != group_required
                        || existing.rule != group_rule
                    {
                        return Err(RetroArchBiosError::InvalidCatalog {
                            line: 0,
                            reason: format!(
                                "core {core} platform {platform} redefines BIOS group {group_id}"
                            ),
                        });
                    }
                }
                None => {
                    target_groups.insert(group_id.clone(), groups.len());
                    groups.push(RetroArchBiosGroupAudit {
                        id: group_id.clone(),
                        description: group_description,
                        required: group_required,
                        rule: group_rule,
                    });
                }
            }
            let resolved =
                resolve_catalog_path(&location.system_directory, &requirement.relative_path);
            let (state, actual_md5) = match resolved.override_state {
                Some(state) => (state, None),
                None => audit_bios_file(&resolved.path, requirement.md5.as_deref()),
            };
            files.push(RetroArchBiosFileAudit {
                group_id,
                requirement,
                path: resolved.path,
                state,
                actual_md5,
            });
        }
    }

    groups.sort_by(|left, right| {
        left.description
            .to_ascii_lowercase()
            .cmp(&right.description.to_ascii_lowercase())
            .then_with(|| left.id.cmp(&right.id))
    });
    files.sort_by(|left, right| {
        left.group_id
            .cmp(&right.group_id)
            .then_with(|| {
                left.requirement
                    .relative_path
                    .to_ascii_lowercase()
                    .cmp(&right.requirement.relative_path.to_ascii_lowercase())
            })
            .then_with(|| {
                left.requirement
                    .relative_path
                    .cmp(&right.requirement.relative_path)
            })
    });

    Ok(RetroArchBiosAudit {
        application_directory: location.application_directory,
        system_directory: location.system_directory,
        configuration_path: location.configuration_path,
        location_source: location.source,
        targets: target_audits,
        groups,
        files,
    })
}

pub fn retroarch_bios_catalog() -> Result<Vec<RetroArchBiosRequirement>, RetroArchBiosError> {
    let mut lines = RETROARCH_BIOS_CATALOG.lines();
    if lines.next() != Some(RETROARCH_BIOS_CATALOG_HEADER) {
        return Err(RetroArchBiosError::InvalidCatalog {
            line: 1,
            reason: "unexpected header".into(),
        });
    }
    let mut rows = BTreeSet::new();
    let mut requirements = Vec::new();
    for (index, line) in lines.enumerate() {
        let line_number = index + 2;
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != 10 {
            return Err(RetroArchBiosError::InvalidCatalog {
                line: line_number,
                reason: format!("expected 10 tab-separated columns, found {}", columns.len()),
            });
        }
        let core = required_catalog_value(columns[0], line_number, "core")?;
        let description = required_catalog_value(columns[2], line_number, "description")?;
        let relative_path = required_catalog_value(columns[3], line_number, "BIOS path")?;
        validate_catalog_path(relative_path, line_number)?;
        let md5 = parse_catalog_md5(columns[4], line_number)?;
        let required = parse_catalog_bool(columns[5], line_number, "Required")?;
        let group = parse_catalog_group(&columns, line_number)?;
        let row_key = (
            core.to_ascii_lowercase(),
            columns[1].to_ascii_lowercase(),
            relative_path.to_ascii_lowercase(),
            columns[4].to_ascii_lowercase(),
            columns[6].to_ascii_lowercase(),
        );
        if !rows.insert(row_key) {
            return Err(RetroArchBiosError::DuplicateCatalogRow { line: line_number });
        }
        requirements.push(RetroArchBiosRequirement {
            core: core.to_string(),
            platform: (!columns[1].is_empty()).then(|| columns[1].to_string()),
            description: description.to_string(),
            relative_path: relative_path.to_string(),
            md5,
            required,
            group,
        });
    }
    Ok(requirements)
}

pub fn default_retroarch_configuration_paths(emulator_application_path: &Path) -> Vec<PathBuf> {
    let application_directory = emulator_application_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(emulator_application_path);
    let mut paths = vec![application_directory.join("retroarch.cfg")];
    for path in default_native_retroarch_configuration_paths() {
        if !paths.contains(&path) {
            paths.push(path);
        }
    }
    paths
}

fn required_catalog_value<'a>(
    value: &'a str,
    line: usize,
    field: &str,
) -> Result<&'a str, RetroArchBiosError> {
    if value.is_empty() {
        return Err(RetroArchBiosError::InvalidCatalog {
            line,
            reason: format!("{field} is empty"),
        });
    }
    Ok(value)
}

fn parse_catalog_md5(value: &str, line: usize) -> Result<Option<String>, RetroArchBiosError> {
    if value.is_empty() {
        return Ok(None);
    }
    if value.len() != 32 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(RetroArchBiosError::InvalidCatalog {
            line,
            reason: "MD5 is not 32 hexadecimal characters".into(),
        });
    }
    Ok(Some(value.to_ascii_lowercase()))
}

fn parse_catalog_bool(value: &str, line: usize, field: &str) -> Result<bool, RetroArchBiosError> {
    if value.eq_ignore_ascii_case("true") {
        Ok(true)
    } else if value.eq_ignore_ascii_case("false") {
        Ok(false)
    } else {
        Err(RetroArchBiosError::InvalidCatalog {
            line,
            reason: format!("{field} is not TRUE or FALSE"),
        })
    }
}

fn parse_catalog_group(
    columns: &[&str],
    line: usize,
) -> Result<Option<RetroArchBiosGroup>, RetroArchBiosError> {
    if columns[6].is_empty() {
        if columns[7..].iter().any(|column| !column.is_empty()) {
            return Err(RetroArchBiosError::InvalidCatalog {
                line,
                reason: "group metadata is present without a group ID".into(),
            });
        }
        return Ok(None);
    }
    let description = required_catalog_value(columns[7], line, "group description")?;
    let required = parse_catalog_bool(columns[8], line, "IsGroupRequired")?;
    let rule = if columns[9].eq_ignore_ascii_case("none") {
        RetroArchBiosGroupRule::None
    } else if columns[9].eq_ignore_ascii_case("any") {
        RetroArchBiosGroupRule::Any
    } else if columns[9].eq_ignore_ascii_case("all") {
        RetroArchBiosGroupRule::All
    } else {
        return Err(RetroArchBiosError::InvalidCatalog {
            line,
            reason: "group rule is not None, Any, or All".into(),
        });
    };
    Ok(Some(RetroArchBiosGroup {
        id: columns[6].to_string(),
        description: description.to_string(),
        required,
        rule,
    }))
}

fn validate_catalog_path(path: &str, line: usize) -> Result<(), RetroArchBiosError> {
    let components = path.split(['/', '\\']).collect::<Vec<_>>();
    if components.is_empty()
        || components
            .iter()
            .any(|component| component.is_empty() || *component == "." || *component == "..")
        || path.starts_with('/')
        || path.starts_with('\\')
        || lb_platform::is_windows_absolute_path(path)
    {
        return Err(RetroArchBiosError::InvalidCatalog {
            line,
            reason: format!("BIOS path {path:?} is not a safe relative path"),
        });
    }
    Ok(())
}

fn selected_targets(
    targets: &[RetroArchBiosTarget],
) -> Result<Vec<(String, String)>, RetroArchBiosError> {
    let mut seen = BTreeSet::new();
    let mut selected = Vec::new();
    for target in targets {
        let Some(core) = retroarch_core_name(&target.command_line) else {
            continue;
        };
        let platform = target.platform.trim();
        if platform.is_empty() {
            continue;
        }
        let key = (platform.to_ascii_lowercase(), core.to_ascii_lowercase());
        if seen.insert(key) {
            selected.push((platform.to_string(), core));
        }
    }
    if selected.is_empty() {
        return Err(RetroArchBiosError::NoConfiguredCore);
    }
    selected.sort_by(|left, right| {
        left.0
            .to_ascii_lowercase()
            .cmp(&right.0.to_ascii_lowercase())
            .then_with(|| {
                left.1
                    .to_ascii_lowercase()
                    .cmp(&right.1.to_ascii_lowercase())
            })
    });
    Ok(selected)
}

fn requirement_group(
    platform: &str,
    core: &str,
    requirement: &RetroArchBiosRequirement,
) -> (String, String, bool, RetroArchBiosGroupRule) {
    let context = format!("{platform} · {core}");
    match &requirement.group {
        Some(group) => (
            format!("{platform}|{core}|group:{}", group.id),
            format!("{context} · {}", group.description),
            group.required,
            group.rule,
        ),
        None if requirement.required => (
            format!("{platform}|{core}|required"),
            format!("{context} · Required BIOS files"),
            true,
            RetroArchBiosGroupRule::All,
        ),
        None => (
            format!("{platform}|{core}|optional"),
            format!("{context} · Optional BIOS files"),
            false,
            RetroArchBiosGroupRule::None,
        ),
    }
}

fn locate_retroarch_bios(
    emulator_application_path: &Path,
    configuration_candidates: &[PathBuf],
    path_resolver: &HostPathResolver,
) -> Result<RetroArchBiosLocation, RetroArchBiosError> {
    let application_directory = emulator_application_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(emulator_application_path)
        .to_path_buf();
    let application_configuration = application_directory.join("retroarch.cfg");
    for configuration in configuration_candidates
        .iter()
        .filter(|path| path.is_absolute())
    {
        let metadata = match fs::symlink_metadata(configuration) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(source) => {
                return Err(RetroArchBiosError::ReadConfiguration {
                    path: configuration.clone(),
                    source,
                });
            }
        };
        if metadata.file_type().is_symlink() {
            return Err(RetroArchBiosError::UnsafeConfiguration {
                path: configuration.clone(),
            });
        }
        if !metadata.is_file() {
            continue;
        }
        if metadata.len() > MAX_RETROARCH_CONFIGURATION_BYTES {
            return Err(RetroArchBiosError::ConfigurationTooLarge {
                path: configuration.clone(),
            });
        }
        let contents = fs::read_to_string(configuration).map_err(|source| {
            RetroArchBiosError::ReadConfiguration {
                path: configuration.clone(),
                source,
            }
        })?;
        let configured = configured_system_directory(&contents);
        let system_directory = match configured {
            Some(value) if !value.trim().is_empty() => resolve_system_directory(
                &application_directory,
                value,
                path_resolver,
                host_home_directory(),
            )?,
            _ => application_directory.join("system"),
        };
        return Ok(RetroArchBiosLocation {
            application_directory,
            system_directory,
            configuration_path: Some(configuration.clone()),
            source: if configuration == &application_configuration {
                RetroArchBiosLocationSource::ApplicationConfiguration
            } else {
                RetroArchBiosLocationSource::NativeConfiguration
            },
        });
    }
    Ok(RetroArchBiosLocation {
        system_directory: application_directory.join("system"),
        application_directory,
        configuration_path: None,
        source: RetroArchBiosLocationSource::ApplicationDefault,
    })
}

fn configured_system_directory(contents: &str) -> Option<&str> {
    contents.lines().find_map(|line| {
        let line = line.trim_start();
        if line.starts_with('#') {
            return None;
        }
        let (key, value) = line.split_once('=')?;
        key.trim()
            .eq_ignore_ascii_case("system_directory")
            .then_some(value.trim())
    })
}

fn resolve_system_directory(
    application_directory: &Path,
    configured: &str,
    path_resolver: &HostPathResolver,
    home: Option<PathBuf>,
) -> Result<PathBuf, RetroArchBiosError> {
    let mut value = configured.replace('"', "");
    value = value.trim().to_string();
    if value.eq_ignore_ascii_case("default") {
        return Err(RetroArchBiosError::DynamicSystemDirectory);
    }
    if value.starts_with(":\\") || value.starts_with(":/") {
        value.drain(..2);
    }
    if value == "~" || value.starts_with("~/") || value.starts_with("~\\") {
        let home = home
            .filter(|path| path.is_absolute())
            .ok_or(RetroArchBiosError::MissingHomeDirectory)?;
        let suffix = value
            .strip_prefix("~/")
            .or_else(|| value.strip_prefix("~\\"))
            .unwrap_or("");
        return path_resolver
            .resolve(&home, suffix)
            .map_err(|source| RetroArchBiosError::ResolveSystemDirectory { value, source });
    }
    path_resolver
        .resolve(application_directory, &value)
        .map_err(|source| RetroArchBiosError::ResolveSystemDirectory { value, source })
}

#[cfg(windows)]
fn host_home_directory() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}

#[cfg(not(windows))]
fn host_home_directory() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

#[derive(Debug)]
struct ResolvedCatalogPath {
    path: PathBuf,
    override_state: Option<BiosFileState>,
}

fn resolve_catalog_path(root: &Path, relative_path: &str) -> ResolvedCatalogPath {
    let components = relative_path.split(['/', '\\']).collect::<Vec<_>>();
    let mut current = root.to_path_buf();
    for (index, component) in components.iter().enumerate() {
        let metadata = match fs::symlink_metadata(&current) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                for remaining in &components[index..] {
                    current.push(remaining);
                }
                return ResolvedCatalogPath {
                    path: current,
                    override_state: None,
                };
            }
            Err(_) => {
                for remaining in &components[index..] {
                    current.push(remaining);
                }
                return ResolvedCatalogPath {
                    path: current,
                    override_state: Some(BiosFileState::Unreadable),
                };
            }
        };
        if metadata.file_type().is_symlink() {
            for remaining in &components[index..] {
                current.push(remaining);
            }
            return ResolvedCatalogPath {
                path: current,
                override_state: Some(BiosFileState::UnsafeEntry),
            };
        }
        if !metadata.is_dir() {
            for remaining in &components[index..] {
                current.push(remaining);
            }
            return ResolvedCatalogPath {
                path: current,
                override_state: Some(BiosFileState::Unreadable),
            };
        }
        let entries = match fs::read_dir(&current) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                for remaining in &components[index..] {
                    current.push(remaining);
                }
                return ResolvedCatalogPath {
                    path: current,
                    override_state: None,
                };
            }
            Err(_) => {
                for remaining in &components[index..] {
                    current.push(remaining);
                }
                return ResolvedCatalogPath {
                    path: current,
                    override_state: Some(BiosFileState::Unreadable),
                };
            }
        };
        let mut matches = entries
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.eq_ignore_ascii_case(component))
            })
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        matches.sort();
        match matches.as_slice() {
            [] => {
                for remaining in &components[index..] {
                    current.push(remaining);
                }
                return ResolvedCatalogPath {
                    path: current,
                    override_state: None,
                };
            }
            [matched] => current = matched.clone(),
            _ => {
                current.push(component);
                return ResolvedCatalogPath {
                    path: current,
                    override_state: Some(BiosFileState::UnsafeEntry),
                };
            }
        }
    }
    ResolvedCatalogPath {
        path: current,
        override_state: None,
    }
}

#[cfg(any(
    test,
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos",
    windows
))]
#[derive(Clone, Copy)]
enum RetroArchConfigHost {
    #[cfg(any(test, target_os = "linux", target_os = "freebsd", target_os = "macos"))]
    Unix,
    #[cfg(any(test, windows))]
    Windows,
}

#[cfg(any(
    test,
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos",
    windows
))]
fn native_configuration_paths_for_host(
    host: RetroArchConfigHost,
    _xdg_config_home: Option<PathBuf>,
    _home: Option<PathBuf>,
    _app_data: Option<PathBuf>,
) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    match host {
        #[cfg(any(test, target_os = "linux", target_os = "freebsd", target_os = "macos"))]
        RetroArchConfigHost::Unix => {
            if let Some(xdg) = _xdg_config_home.filter(|path| path.is_absolute()) {
                paths.push(xdg.join("retroarch/retroarch.cfg"));
            }
            if let Some(home) = _home.filter(|path| path.is_absolute()) {
                paths.push(home.join(".config/retroarch/retroarch.cfg"));
                paths.push(home.join(".retroarch.cfg"));
            }
            paths.push(PathBuf::from("/etc/retroarch.cfg"));
        }
        #[cfg(any(test, windows))]
        RetroArchConfigHost::Windows => {
            if let Some(app_data) = _app_data {
                paths.push(app_data.join("retroarch.cfg"));
            }
        }
    }
    paths.dedup();
    paths
}

#[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "macos"))]
fn default_native_retroarch_configuration_paths() -> Vec<PathBuf> {
    native_configuration_paths_for_host(
        RetroArchConfigHost::Unix,
        std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
        std::env::var_os("HOME").map(PathBuf::from),
        None,
    )
}

#[cfg(windows)]
fn default_native_retroarch_configuration_paths() -> Vec<PathBuf> {
    native_configuration_paths_for_host(
        RetroArchConfigHost::Windows,
        None,
        None,
        std::env::var_os("APPDATA").map(PathBuf::from),
    )
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos",
    windows
)))]
fn default_native_retroarch_configuration_paths() -> Vec<PathBuf> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn recovered_catalog_has_all_630_rows_and_exact_group_rules() {
        let catalog = retroarch_bios_catalog().expect("valid recovered catalog");
        assert_eq!(catalog.len(), 630);
        assert_eq!(
            catalog
                .iter()
                .map(|requirement| requirement.core.to_ascii_lowercase())
                .collect::<BTreeSet<_>>()
                .len(),
            103
        );
        assert!(catalog.iter().any(|requirement| {
            requirement.core == "4do_libretro"
                && requirement.relative_path == "panafz1.bin"
                && requirement.md5.as_deref() == Some("f47264dd47fe30f73ab3c010015c155b")
                && requirement.group.as_ref().is_some_and(|group| {
                    group.description == "System BIOS"
                        && group.required
                        && group.rule == RetroArchBiosGroupRule::Any
                })
        }));
        assert!(catalog.iter().any(|requirement| {
            requirement.core == "bsnes_libretro"
                && requirement.group.as_ref().is_some_and(|group| {
                    !group.required && group.rule == RetroArchBiosGroupRule::All
                })
        }));
        assert!(catalog.iter().any(|requirement| {
            requirement.core == "4do_libretro"
                && requirement.group.as_ref().is_some_and(|group| {
                    !group.required && group.rule == RetroArchBiosGroupRule::None
                })
        }));
    }

    #[test]
    fn required_any_all_and_optional_group_rules_control_readiness() {
        fn file(
            group: &RetroArchBiosGroupAudit,
            name: &str,
            state: BiosFileState,
        ) -> RetroArchBiosFileAudit {
            RetroArchBiosFileAudit {
                group_id: group.id.clone(),
                requirement: RetroArchBiosRequirement {
                    core: "fixture_libretro".into(),
                    platform: Some("Fixture Platform".into()),
                    description: name.into(),
                    relative_path: name.into(),
                    md5: None,
                    required: false,
                    group: Some(RetroArchBiosGroup {
                        id: group.id.clone(),
                        description: group.description.clone(),
                        required: group.required,
                        rule: group.rule,
                    }),
                },
                path: PathBuf::from("/fixture/system").join(name),
                state,
                actual_md5: None,
            }
        }

        let any = RetroArchBiosGroupAudit {
            id: "any".into(),
            description: "Any alternative".into(),
            required: true,
            rule: RetroArchBiosGroupRule::Any,
        };
        let all = RetroArchBiosGroupAudit {
            id: "all".into(),
            description: "All files".into(),
            required: true,
            rule: RetroArchBiosGroupRule::All,
        };
        let optional = RetroArchBiosGroupAudit {
            id: "optional".into(),
            description: "Optional files".into(),
            required: false,
            rule: RetroArchBiosGroupRule::None,
        };
        let mut audit = RetroArchBiosAudit {
            application_directory: PathBuf::from("/fixture"),
            system_directory: PathBuf::from("/fixture/system"),
            configuration_path: None,
            location_source: RetroArchBiosLocationSource::ApplicationDefault,
            targets: Vec::new(),
            groups: vec![any.clone(), all.clone(), optional.clone()],
            files: vec![
                file(&any, "any-valid.bin", BiosFileState::Valid),
                file(&any, "any-missing.bin", BiosFileState::Missing),
                file(&all, "all-valid.bin", BiosFileState::Valid),
                file(&all, "all-missing.bin", BiosFileState::Missing),
                file(&optional, "optional-missing.bin", BiosFileState::Missing),
            ],
        };

        assert!(audit.group_satisfied(&any));
        assert!(!audit.group_satisfied(&all));
        assert!(audit.group_satisfied(&optional));
        assert!(!audit.ready());

        audit.files[3].state = BiosFileState::Valid;
        assert!(audit.group_satisfied(&all));
        assert!(audit.ready());
    }

    #[test]
    fn portable_relative_system_directory_audits_required_core_file_without_mutation() {
        let directory = tempfile::tempdir().expect("temporary RetroArch");
        let application = directory.path().join("retroarch");
        let configuration = directory.path().join("retroarch.cfg");
        let system = directory.path().join("firmware");
        fs::create_dir(&system).expect("system directory");
        fs::write(&application, []).expect("application fixture");
        fs::write(&configuration, "system_directory = \":\\\\firmware\"\n")
            .expect("configuration fixture");
        fs::write(system.join("disksys.rom"), b"fixture BIOS").expect("BIOS fixture");
        let before = fs::read(&configuration).expect("configuration snapshot");

        let audit = audit_retroarch_bios(
            &application,
            &[RetroArchBiosTarget {
                platform: "Nintendo Famicom Disk System".into(),
                command_line: r#"-L "cores\mesen_libretro.dll""#.into(),
            }],
            std::slice::from_ref(&configuration),
            &HostPathResolver::default(),
        )
        .expect("RetroArch BIOS audit");

        assert_eq!(audit.system_directory, system);
        assert_eq!(
            audit.location_source,
            RetroArchBiosLocationSource::ApplicationConfiguration
        );
        assert_eq!(audit.targets.len(), 1);
        assert_eq!(audit.targets[0].core, "mesen_libretro");
        assert_eq!(audit.files.len(), 1);
        assert_eq!(audit.files[0].state, BiosFileState::Valid);
        assert!(audit.ready());
        assert_eq!(
            fs::read(configuration).expect("configuration after audit"),
            before
        );
    }

    #[test]
    fn mapped_windows_system_directory_and_case_insensitive_nested_paths_are_supported() {
        let directory = tempfile::tempdir().expect("temporary RetroArch");
        let mapped = tempfile::tempdir().expect("mapped drive");
        let application = directory.path().join("retroarch");
        let configuration = directory.path().join("retroarch.cfg");
        fs::write(&application, []).expect("application fixture");
        fs::write(
            &configuration,
            "system_directory = \"D:\\\\RetroArch\\\\System\"\n",
        )
        .expect("configuration fixture");
        let database = mapped.path().join("RetroArch/System/DATABASES");
        fs::create_dir_all(&database).expect("nested system directory");
        fs::write(database.join("MSXROMDB.XML"), b"fixture database")
            .expect("required BIOS fixture");
        let resolver = HostPathResolver::default()
            .with_windows_drive_mapping('D', mapped.path())
            .expect("drive mapping");

        let audit = audit_retroarch_bios(
            &application,
            &[RetroArchBiosTarget {
                platform: "Microsoft MSX".into(),
                command_line: "-L cores/bluemsx_libretro.so".into(),
            }],
            &[configuration],
            &resolver,
        )
        .expect("mapped RetroArch audit");

        assert!(audit.files.iter().any(|file| {
            file.requirement.relative_path == "Databases/msxromdb.xml"
                && file.state == BiosFileState::Valid
                && file.path.ends_with("DATABASES/MSXROMDB.XML")
        }));
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_or_case_ambiguous_catalog_components_are_unsafe() {
        use std::os::unix::fs::symlink;

        let directory = tempfile::tempdir().expect("temporary RetroArch");
        let application = directory.path().join("retroarch");
        let system = directory.path().join("system");
        let outside = directory.path().join("outside");
        fs::create_dir(&system).expect("system directory");
        fs::create_dir(&outside).expect("outside directory");
        fs::write(&application, []).expect("application fixture");
        fs::write(outside.join("msxromdb.xml"), b"outside").expect("outside fixture");
        symlink(&outside, system.join("Databases")).expect("directory symlink");
        fs::create_dir(system.join("Machines")).expect("first case directory");
        fs::create_dir(system.join("MACHINES")).expect("second case directory");

        let audit = audit_retroarch_bios(
            &application,
            &[RetroArchBiosTarget {
                platform: "Microsoft MSX".into(),
                command_line: "-L cores/bluemsx_libretro.dylib".into(),
            }],
            &[],
            &HostPathResolver::default(),
        )
        .expect("safe RetroArch audit");

        assert_eq!(
            audit
                .files
                .iter()
                .find(|file| file.requirement.relative_path == "Databases/msxromdb.xml")
                .map(|file| file.state),
            Some(BiosFileState::UnsafeEntry)
        );
        assert_eq!(
            audit
                .files
                .iter()
                .find(|file| file.requirement.relative_path.starts_with("Machines/"))
                .map(|file| file.state),
            Some(BiosFileState::UnsafeEntry)
        );
    }

    #[test]
    fn dynamic_content_directory_is_reported_instead_of_guessed() {
        let directory = tempfile::tempdir().expect("temporary RetroArch");
        let application = directory.path().join("retroarch");
        let configuration = directory.path().join("retroarch.cfg");
        fs::write(&application, []).expect("application fixture");
        fs::write(&configuration, "system_directory = \"default\"\n")
            .expect("configuration fixture");

        assert!(matches!(
            audit_retroarch_bios(
                &application,
                &[RetroArchBiosTarget {
                    platform: "Sega Saturn".into(),
                    command_line: "-L cores/mednafen_saturn_libretro.so".into(),
                }],
                &[configuration],
                &HostPathResolver::default(),
            ),
            Err(RetroArchBiosError::DynamicSystemDirectory)
        ));
    }

    #[test]
    fn native_configuration_candidates_are_host_specific_and_deduplicated() {
        assert_eq!(
            native_configuration_paths_for_host(
                RetroArchConfigHost::Unix,
                Some(PathBuf::from("/xdg")),
                Some(PathBuf::from("/home/player")),
                None,
            ),
            vec![
                PathBuf::from("/xdg/retroarch/retroarch.cfg"),
                PathBuf::from("/home/player/.config/retroarch/retroarch.cfg"),
                PathBuf::from("/home/player/.retroarch.cfg"),
                PathBuf::from("/etc/retroarch.cfg"),
            ]
        );
        assert_eq!(
            native_configuration_paths_for_host(
                RetroArchConfigHost::Windows,
                None,
                None,
                Some(PathBuf::from(r"C:\Users\Player\AppData\Roaming")),
            ),
            vec![PathBuf::from(
                r"C:\Users\Player\AppData\Roaming/retroarch.cfg"
            )]
        );
    }
}
