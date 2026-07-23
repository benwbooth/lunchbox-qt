use crate::pcsx2_bios::{audit_bios_file, regular_file_without_symlink, BiosFileState};
use lb_platform::{HostPathResolver, LaunchPathError, LaunchPathResolver};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

const MAX_XEMU_CONFIGURATION_BYTES: u64 = 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum XemuBiosGroupKind {
    Boot,
    Hdd,
    Flash,
}

impl XemuBiosGroupKind {
    pub const ALL: [Self; 3] = [Self::Boot, Self::Hdd, Self::Flash];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Boot => "xemu boot",
            Self::Hdd => "xemu hdd",
            Self::Flash => "xemu bios",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::Boot => "Xemu Boot ROM Image",
            Self::Hdd => "Xemu HDD Image",
            Self::Flash => "Xemu BIOS",
        }
    }

    pub const fn required(self) -> bool {
        true
    }

    pub const fn all_items_required(self) -> bool {
        false
    }

    const fn configuration_key(self) -> &'static str {
        match self {
            Self::Boot => "bootrom_path",
            Self::Hdd => "hdd_path",
            Self::Flash => "flashrom_path",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XemuBiosRequirement {
    pub group: XemuBiosGroupKind,
    pub file_name: String,
    pub file_required: bool,
    pub description: String,
    pub md5: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XemuBiosFileAudit {
    pub requirement: XemuBiosRequirement,
    pub path: PathBuf,
    pub state: BiosFileState,
    pub actual_md5: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum XemuBiosLocationSource {
    PortableConfiguration,
    NativeConfiguration,
    ApplicationDefaults,
}

impl XemuBiosLocationSource {
    pub const fn label(self) -> &'static str {
        match self {
            Self::PortableConfiguration => "portable Xemu configuration",
            Self::NativeConfiguration => "native Xemu configuration",
            Self::ApplicationDefaults => "Xemu application-directory defaults",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XemuBiosAudit {
    pub application_directory: PathBuf,
    pub configuration_path: Option<PathBuf>,
    pub location_source: XemuBiosLocationSource,
    pub group_directories: BTreeMap<XemuBiosGroupKind, PathBuf>,
    pub files: Vec<XemuBiosFileAudit>,
}

impl XemuBiosAudit {
    pub fn files_for_group(
        &self,
        group: XemuBiosGroupKind,
    ) -> impl Iterator<Item = &XemuBiosFileAudit> {
        self.files
            .iter()
            .filter(move |file| file.requirement.group == group)
    }

    pub fn valid_count(&self) -> usize {
        self.files
            .iter()
            .filter(|file| file.state == BiosFileState::Valid)
            .count()
    }

    pub fn mismatch_count(&self) -> usize {
        self.files
            .iter()
            .filter(|file| file.state == BiosFileState::HashMismatch)
            .count()
    }

    pub fn unsafe_count(&self) -> usize {
        self.files
            .iter()
            .filter(|file| file.state == BiosFileState::UnsafeEntry)
            .count()
    }

    pub fn unreadable_count(&self) -> usize {
        self.files
            .iter()
            .filter(|file| file.state == BiosFileState::Unreadable)
            .count()
    }

    pub fn missing_count(&self) -> usize {
        self.files
            .iter()
            .filter(|file| file.state == BiosFileState::Missing)
            .count()
    }

    pub fn group_satisfied(&self, group: XemuBiosGroupKind) -> bool {
        self.files_for_group(group)
            .any(|file| file.state == BiosFileState::Valid)
    }

    pub fn ready(&self) -> bool {
        XemuBiosGroupKind::ALL
            .into_iter()
            .all(|group| self.group_satisfied(group))
    }
}

#[derive(Debug, Error)]
pub enum XemuBiosError {
    #[error("Xemu executable path is not an absolute native host path: {path}")]
    NonAbsoluteExecutable { path: PathBuf },
    #[error("Xemu BIOS catalog contains a duplicate filename or MD5: {value}")]
    DuplicateCatalogValue { value: String },
    #[error("Xemu configuration is too large to audit safely: {path}")]
    ConfigurationTooLarge { path: PathBuf },
    #[error("could not read Xemu configuration {path}: {source}")]
    ReadConfiguration { path: PathBuf, source: io::Error },
    #[error("could not parse Xemu configuration {path}: {source}")]
    ParseConfiguration {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("could not resolve Xemu {key} value {value:?}: {source}")]
    ResolveConfigurationPath {
        key: &'static str,
        value: String,
        source: LaunchPathError,
    },
}

#[derive(Clone, Debug)]
struct XemuBiosLocation {
    application_directory: PathBuf,
    configuration_path: Option<PathBuf>,
    source: XemuBiosLocationSource,
    group_directories: BTreeMap<XemuBiosGroupKind, PathBuf>,
}

/// Audits the exact Xemu BIOS groups recovered from LaunchBox 13.27 without
/// starting Xemu, downloading firmware, creating directories, or rewriting
/// `xemu.toml`.
pub fn audit_xemu_bios(
    emulator_application_path: &Path,
    native_data_directories: &[PathBuf],
    path_resolver: &HostPathResolver,
) -> Result<XemuBiosAudit, XemuBiosError> {
    if !emulator_application_path.is_absolute() {
        return Err(XemuBiosError::NonAbsoluteExecutable {
            path: emulator_application_path.to_path_buf(),
        });
    }
    let requirements = xemu_bios_requirements()?;
    let location = locate_xemu_bios(
        emulator_application_path,
        native_data_directories,
        path_resolver,
    )?;
    let files = requirements
        .into_iter()
        .map(|requirement| {
            let directory = &location.group_directories[&requirement.group];
            let path = case_insensitive_catalog_path(directory, &requirement.file_name);
            let unsafe_directory = fs::symlink_metadata(directory)
                .is_ok_and(|metadata| metadata.file_type().is_symlink());
            let (state, actual_md5) = if unsafe_directory || path.ambiguous {
                (BiosFileState::UnsafeEntry, None)
            } else {
                audit_bios_file(&path.path, requirement.md5.as_deref())
            };
            XemuBiosFileAudit {
                requirement,
                path: path.path,
                state,
                actual_md5,
            }
        })
        .collect();
    Ok(XemuBiosAudit {
        application_directory: location.application_directory,
        configuration_path: location.configuration_path,
        location_source: location.source,
        group_directories: location.group_directories,
        files,
    })
}

pub fn xemu_bios_requirements() -> Result<Vec<XemuBiosRequirement>, XemuBiosError> {
    let entries = [
        (
            XemuBiosGroupKind::Boot,
            "mcpx_1.0.bin",
            true,
            "Boot ROM",
            Some("d49c52a4102f6df7bcf8d0617ac475ed"),
        ),
        (
            XemuBiosGroupKind::Hdd,
            "xbox_hdd.qcow2",
            false,
            "HDD Image",
            None,
        ),
        (
            XemuBiosGroupKind::Flash,
            "Complex_4627v1.03.bin",
            false,
            "(1mb) Complex 4627 (1.03)",
            Some("21445c6f28fca7285b0f167ea770d1e5"),
        ),
        (
            XemuBiosGroupKind::Flash,
            "Complex_4627.bin",
            false,
            "(1mb) Complex 4627 Retail (1.0)",
            Some("ec00e31e746de2473acfe7903c5a4cb7"),
        ),
        (
            XemuBiosGroupKind::Flash,
            "bios_debug_4627.bin",
            false,
            "(1mb) Complex 4627 Debug (1.02)",
            Some("19b5c6d3d42a707bba620634fe6d4baf"),
        ),
        (
            XemuBiosGroupKind::Flash,
            "bios_retail_4627.bin",
            false,
            "(1mb) Complex 4627 Retail (1.02)",
            Some("39cee882148a87f93cb440b99dde3ceb"),
        ),
        (
            XemuBiosGroupKind::Flash,
            "xbox-4627_debug.bin",
            false,
            "Complex BIOS Debug",
            Some("e8dd61cc6abdbd06aac185e371312dc1"),
        ),
    ];
    let mut file_names = BTreeSet::new();
    let mut hashes = BTreeSet::new();
    let mut requirements = Vec::with_capacity(entries.len());
    for (group, file_name, file_required, description, md5) in entries {
        if !file_names.insert(file_name.to_ascii_lowercase()) {
            return Err(XemuBiosError::DuplicateCatalogValue {
                value: file_name.into(),
            });
        }
        let md5 = md5.map(str::to_string);
        if let Some(md5) = &md5 {
            if !hashes.insert(md5.clone()) {
                return Err(XemuBiosError::DuplicateCatalogValue { value: md5.clone() });
            }
        }
        requirements.push(XemuBiosRequirement {
            group,
            file_name: file_name.into(),
            file_required,
            description: description.into(),
            md5,
        });
    }
    Ok(requirements)
}

pub fn default_xemu_data_directories() -> Vec<PathBuf> {
    default_native_xemu_root().into_iter().collect()
}

fn locate_xemu_bios(
    emulator_application_path: &Path,
    native_data_directories: &[PathBuf],
    path_resolver: &HostPathResolver,
) -> Result<XemuBiosLocation, XemuBiosError> {
    let application_directory = emulator_application_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(emulator_application_path)
        .to_path_buf();
    let mut portable_candidates = vec![application_directory.join("xemu.toml")];
    if let Some(contents) = application_directory
        .parent()
        .filter(|path| path.file_name().is_some_and(|name| name == "Contents"))
    {
        portable_candidates.push(contents.join("Resources/xemu.toml"));
    }
    if let Some(configuration) = portable_candidates
        .into_iter()
        .find(|path| regular_file_without_symlink(path))
    {
        return location_from_configuration(
            application_directory,
            configuration,
            XemuBiosLocationSource::PortableConfiguration,
            path_resolver,
        );
    }

    let mut roots = native_data_directories
        .iter()
        .filter(|path| path.is_absolute())
        .cloned()
        .collect::<Vec<_>>();
    if let Some(default) = default_native_xemu_root().filter(|path| !roots.contains(path)) {
        roots.push(default);
    }
    for root in roots {
        let configuration = root.join("xemu.toml");
        if regular_file_without_symlink(&configuration) {
            return location_from_configuration(
                application_directory,
                configuration,
                XemuBiosLocationSource::NativeConfiguration,
                path_resolver,
            );
        }
    }

    Ok(XemuBiosLocation {
        group_directories: default_group_directories(&application_directory),
        application_directory,
        configuration_path: None,
        source: XemuBiosLocationSource::ApplicationDefaults,
    })
}

fn location_from_configuration(
    application_directory: PathBuf,
    configuration: PathBuf,
    source: XemuBiosLocationSource,
    path_resolver: &HostPathResolver,
) -> Result<XemuBiosLocation, XemuBiosError> {
    let metadata =
        fs::metadata(&configuration).map_err(|source| XemuBiosError::ReadConfiguration {
            path: configuration.clone(),
            source,
        })?;
    if metadata.len() > MAX_XEMU_CONFIGURATION_BYTES {
        return Err(XemuBiosError::ConfigurationTooLarge {
            path: configuration,
        });
    }
    let contents =
        fs::read_to_string(&configuration).map_err(|source| XemuBiosError::ReadConfiguration {
            path: configuration.clone(),
            source,
        })?;
    let document =
        contents
            .parse::<toml::Table>()
            .map_err(|source| XemuBiosError::ParseConfiguration {
                path: configuration.clone(),
                source,
            })?;
    let files = document
        .get("sys")
        .and_then(toml::Value::as_table)
        .and_then(|sys| sys.get("files"))
        .and_then(toml::Value::as_table);
    let mut group_directories = default_group_directories(&application_directory);
    for group in XemuBiosGroupKind::ALL {
        let Some(value) = files
            .and_then(|files| files.get(group.configuration_key()))
            .and_then(toml::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        let path = path_resolver
            .resolve(&application_directory, value)
            .map_err(|source| XemuBiosError::ResolveConfigurationPath {
                key: group.configuration_key(),
                value: value.to_string(),
                source,
            })?;
        if path_entry_exists(&path) {
            if let Some(parent) = path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
            {
                group_directories.insert(group, parent.to_path_buf());
            }
        }
    }
    Ok(XemuBiosLocation {
        application_directory,
        configuration_path: Some(configuration),
        source,
        group_directories,
    })
}

fn default_group_directories(application_directory: &Path) -> BTreeMap<XemuBiosGroupKind, PathBuf> {
    BTreeMap::from([
        (XemuBiosGroupKind::Boot, application_directory.join("bios")),
        (XemuBiosGroupKind::Hdd, application_directory.join("saves")),
        (XemuBiosGroupKind::Flash, application_directory.join("bios")),
    ])
}

fn path_entry_exists(path: &Path) -> bool {
    match fs::symlink_metadata(path) {
        Ok(_) => true,
        Err(error) => error.kind() != io::ErrorKind::NotFound,
    }
}

struct CatalogPath {
    path: PathBuf,
    ambiguous: bool,
}

fn case_insensitive_catalog_path(directory: &Path, expected_name: &str) -> CatalogPath {
    let exact = directory.join(expected_name);
    if path_entry_exists(&exact) {
        return CatalogPath {
            path: exact,
            ambiguous: false,
        };
    }
    let Ok(entries) = fs::read_dir(directory) else {
        return CatalogPath {
            path: exact,
            ambiguous: false,
        };
    };
    let mut matching = entries
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .is_some_and(|name| name.eq_ignore_ascii_case(expected_name))
        })
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    matching.sort();
    match matching.as_slice() {
        [path] => CatalogPath {
            path: path.clone(),
            ambiguous: false,
        },
        [] => CatalogPath {
            path: exact,
            ambiguous: false,
        },
        _ => CatalogPath {
            path: exact,
            ambiguous: true,
        },
    }
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
fn default_native_xemu_root() -> Option<PathBuf> {
    native_xemu_root_for_host(
        XemuDataHost::Freedesktop,
        std::env::var_os("XDG_DATA_HOME").map(PathBuf::from),
        std::env::var_os("HOME").map(PathBuf::from),
        None,
    )
}

#[cfg(target_os = "macos")]
fn default_native_xemu_root() -> Option<PathBuf> {
    native_xemu_root_for_host(
        XemuDataHost::Macos,
        None,
        std::env::var_os("HOME").map(PathBuf::from),
        None,
    )
}

#[cfg(windows)]
fn default_native_xemu_root() -> Option<PathBuf> {
    native_xemu_root_for_host(
        XemuDataHost::Windows,
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
fn default_native_xemu_root() -> Option<PathBuf> {
    None
}

#[cfg(any(
    test,
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos",
    windows
))]
#[derive(Clone, Copy)]
enum XemuDataHost {
    #[cfg(any(test, target_os = "linux", target_os = "freebsd"))]
    Freedesktop,
    #[cfg(any(test, target_os = "macos"))]
    Macos,
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
fn native_xemu_root_for_host(
    host: XemuDataHost,
    _xdg_data_home: Option<PathBuf>,
    _home: Option<PathBuf>,
    _app_data: Option<PathBuf>,
) -> Option<PathBuf> {
    match host {
        #[cfg(any(test, target_os = "linux", target_os = "freebsd"))]
        XemuDataHost::Freedesktop => _xdg_data_home
            .filter(|path| path.is_absolute())
            .map(|path| path.join("xemu/xemu"))
            .or_else(|| _home.map(|path| path.join(".local/share/xemu/xemu"))),
        #[cfg(any(test, target_os = "macos"))]
        XemuDataHost::Macos => _home.map(|path| path.join("Library/Application Support/xemu/xemu")),
        #[cfg(any(test, windows))]
        XemuDataHost::Windows => _app_data.map(|path| path.join("xemu/xemu")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovered_catalog_has_three_exact_required_groups_and_seven_files() {
        let requirements = xemu_bios_requirements().expect("valid recovered catalog");
        assert_eq!(requirements.len(), 7);
        assert_eq!(
            requirements
                .iter()
                .filter(|requirement| requirement.group == XemuBiosGroupKind::Boot)
                .count(),
            1
        );
        assert_eq!(
            requirements
                .iter()
                .filter(|requirement| requirement.group == XemuBiosGroupKind::Hdd)
                .count(),
            1
        );
        assert_eq!(
            requirements
                .iter()
                .filter(|requirement| requirement.group == XemuBiosGroupKind::Flash)
                .count(),
            5
        );
        assert!(XemuBiosGroupKind::ALL
            .into_iter()
            .all(XemuBiosGroupKind::required));
        assert!(requirements
            .iter()
            .find(|requirement| requirement.file_name == "xbox_hdd.qcow2")
            .is_some_and(|requirement| requirement.md5.is_none()));
    }

    #[test]
    fn sdl_preference_roots_are_host_native_and_do_not_share_windows_syntax() {
        assert_eq!(
            native_xemu_root_for_host(
                XemuDataHost::Freedesktop,
                Some(PathBuf::from("/xdg-data")),
                Some(PathBuf::from("/home/player")),
                None,
            ),
            Some(PathBuf::from("/xdg-data/xemu/xemu"))
        );
        assert_eq!(
            native_xemu_root_for_host(
                XemuDataHost::Freedesktop,
                Some(PathBuf::from("relative")),
                Some(PathBuf::from("/home/player")),
                None,
            ),
            Some(PathBuf::from("/home/player/.local/share/xemu/xemu"))
        );
        assert_eq!(
            native_xemu_root_for_host(
                XemuDataHost::Macos,
                None,
                Some(PathBuf::from("/Users/player")),
                None,
            ),
            Some(PathBuf::from(
                "/Users/player/Library/Application Support/xemu/xemu"
            ))
        );
        assert_eq!(
            native_xemu_root_for_host(
                XemuDataHost::Windows,
                None,
                None,
                Some(PathBuf::from(r"C:\Users\player\AppData\Roaming")),
            ),
            Some(
                PathBuf::from(r"C:\Users\player\AppData\Roaming")
                    .join("xemu")
                    .join("xemu")
            )
        );
    }

    #[test]
    fn portable_configuration_selects_existing_group_directories_without_mutation() {
        let directory = tempfile::tempdir().expect("temporary Xemu");
        let application = directory.path().join("xemu");
        let firmware = directory.path().join("custom-firmware");
        let saves = directory.path().join("custom-saves");
        fs::create_dir_all(&firmware).expect("firmware directory");
        fs::create_dir_all(&saves).expect("save directory");
        fs::write(&application, []).expect("application fixture");
        fs::write(firmware.join("mcpx_1.0.bin"), b"not copyrighted firmware")
            .expect("boot fixture");
        fs::write(
            firmware.join("complex_4627.bin"),
            b"not copyrighted firmware",
        )
        .expect("flash fixture");
        fs::write(saves.join("xbox_hdd.qcow2"), b"readable HDD fixture").expect("HDD fixture");
        let configuration = format!(
            "[sys.files]\nbootrom_path = '{}'\nflashrom_path = '{}'\nhdd_path = '{}'\n",
            firmware.join("mcpx_1.0.bin").display(),
            firmware.join("complex_4627.bin").display(),
            saves.join("xbox_hdd.qcow2").display()
        );
        fs::write(directory.path().join("xemu.toml"), &configuration)
            .expect("portable configuration");
        let before = fs::read(directory.path().join("xemu.toml")).expect("config before");

        let audit = audit_xemu_bios(&application, &[], &HostPathResolver::default())
            .expect("portable Xemu audit");

        assert_eq!(
            audit.location_source,
            XemuBiosLocationSource::PortableConfiguration
        );
        assert_eq!(audit.group_directories[&XemuBiosGroupKind::Boot], firmware);
        assert_eq!(audit.group_directories[&XemuBiosGroupKind::Hdd], saves);
        assert_eq!(audit.valid_count(), 1);
        assert_eq!(audit.mismatch_count(), 2);
        assert_eq!(audit.missing_count(), 4);
        assert!(audit.group_satisfied(XemuBiosGroupKind::Hdd));
        assert!(!audit.group_satisfied(XemuBiosGroupKind::Boot));
        assert!(!audit.group_satisfied(XemuBiosGroupKind::Flash));
        assert!(!audit.ready());
        assert_eq!(
            fs::read(directory.path().join("xemu.toml")).expect("config after"),
            before
        );
    }

    #[test]
    fn mapped_windows_configuration_paths_are_resolved_by_the_platform_service() {
        let directory = tempfile::tempdir().expect("temporary Xemu");
        let mapped = tempfile::tempdir().expect("mapped drive");
        let application = directory.path().join("xemu");
        let firmware = mapped.path().join("firmware");
        let saves = mapped.path().join("saves");
        fs::create_dir_all(&firmware).expect("firmware directory");
        fs::create_dir_all(&saves).expect("save directory");
        fs::write(&application, []).expect("application fixture");
        fs::write(firmware.join("mcpx_1.0.bin"), b"boot fixture").expect("boot fixture");
        fs::write(saves.join("xbox_hdd.qcow2"), b"hdd fixture").expect("HDD fixture");
        fs::write(
            directory.path().join("xemu.toml"),
            "[sys.files]\nbootrom_path = 'D:\\\\firmware\\\\mcpx_1.0.bin'\nhdd_path = 'D:\\\\saves\\\\xbox_hdd.qcow2'\n",
        )
        .expect("portable configuration");
        let resolver = HostPathResolver::default()
            .with_windows_drive_mapping('D', mapped.path())
            .expect("drive mapping");

        let audit = audit_xemu_bios(&application, &[], &resolver).expect("mapped Xemu audit");

        assert_eq!(audit.group_directories[&XemuBiosGroupKind::Boot], firmware);
        assert_eq!(audit.group_directories[&XemuBiosGroupKind::Hdd], saves);
        assert_eq!(
            audit
                .files_for_group(XemuBiosGroupKind::Hdd)
                .next()
                .map(|file| file.state),
            Some(BiosFileState::Valid)
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_firmware_and_case_ambiguous_entries_are_unsafe() {
        use std::os::unix::fs::symlink;

        let directory = tempfile::tempdir().expect("temporary Xemu");
        let application = directory.path().join("xemu");
        let firmware = directory.path().join("bios");
        fs::create_dir(&firmware).expect("firmware directory");
        fs::write(&application, []).expect("application fixture");
        let target = directory.path().join("outside.bin");
        fs::write(&target, b"must not be trusted").expect("symlink target");
        symlink(&target, firmware.join("mcpx_1.0.bin")).expect("boot symlink");
        fs::write(firmware.join("complex_4627.bin"), b"first").expect("first case");
        fs::write(firmware.join("COMPLEX_4627.BIN"), b"second").expect("second case");

        let audit = audit_xemu_bios(&application, &[], &HostPathResolver::default())
            .expect("safe Xemu audit");

        assert_eq!(
            audit
                .files
                .iter()
                .find(|file| file.requirement.file_name == "mcpx_1.0.bin")
                .map(|file| file.state),
            Some(BiosFileState::UnsafeEntry)
        );
        assert_eq!(
            audit
                .files
                .iter()
                .find(|file| file.requirement.file_name == "Complex_4627.bin")
                .map(|file| file.state),
            Some(BiosFileState::UnsafeEntry)
        );
    }
}
