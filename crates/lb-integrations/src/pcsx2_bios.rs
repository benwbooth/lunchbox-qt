use lb_platform::is_windows_absolute_path;
use md5::{Digest, Md5};
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use thiserror::Error;

const PCSX2_BIOS_CATALOG: &str = include_str!("pcsx2_bios_catalog.tsv");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pcsx2BiosRequirement {
    pub file_name: String,
    pub description: String,
    pub md5: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BiosFileState {
    Missing,
    Valid,
    HashMismatch,
    UnsafeEntry,
    Unreadable,
}

impl BiosFileState {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Valid => "valid",
            Self::HashMismatch => "hash_mismatch",
            Self::UnsafeEntry => "unsafe_entry",
            Self::Unreadable => "unreadable",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pcsx2BiosFileAudit {
    pub requirement: Pcsx2BiosRequirement,
    pub path: PathBuf,
    pub state: BiosFileState,
    pub actual_md5: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pcsx2BiosLocationSource {
    PortableConfiguration,
    PortableDefault,
    NativeConfiguration,
    NativeDefault,
    ApplicationFallback,
}

impl Pcsx2BiosLocationSource {
    pub const fn label(self) -> &'static str {
        match self {
            Self::PortableConfiguration => "portable PCSX2 configuration",
            Self::PortableDefault => "portable PCSX2 default",
            Self::NativeConfiguration => "native PCSX2 configuration",
            Self::NativeDefault => "native PCSX2 default",
            Self::ApplicationFallback => "executable-directory fallback",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pcsx2BiosAudit {
    pub bios_directory: PathBuf,
    pub configuration_path: Option<PathBuf>,
    pub location_source: Pcsx2BiosLocationSource,
    pub files: Vec<Pcsx2BiosFileAudit>,
}

impl Pcsx2BiosAudit {
    pub const fn group_id(&self) -> &'static str {
        "ps2 bios"
    }

    pub const fn group_description(&self) -> &'static str {
        "PlayStation 2 BIOS"
    }

    pub const fn group_required(&self) -> bool {
        true
    }

    pub const fn all_items_required(&self) -> bool {
        false
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

    pub fn group_satisfied(&self) -> bool {
        self.valid_count() > 0
    }
}

#[derive(Debug, Error)]
pub enum Pcsx2BiosError {
    #[error("PCSX2 executable path is not an absolute native host path: {path}")]
    NonAbsoluteExecutable { path: PathBuf },
    #[error("PCSX2 BIOS catalog line {line} is malformed")]
    InvalidCatalog { line: usize },
    #[error("PCSX2 BIOS catalog contains a duplicate filename or MD5: {value}")]
    DuplicateCatalogValue { value: String },
    #[error("could not read PCSX2 configuration {path}: {source}")]
    ReadConfiguration { path: PathBuf, source: io::Error },
    #[error("PCSX2 configuration contains a foreign absolute BIOS path on this host: {path}")]
    ForeignAbsoluteConfigurationPath { path: String },
}

#[derive(Clone, Debug)]
struct Pcsx2BiosLocation {
    bios_directory: PathBuf,
    configuration_path: Option<PathBuf>,
    source: Pcsx2BiosLocationSource,
}

/// Audits the complete LaunchBox 13.27 PCSX2 BIOS alternative group without
/// executing PCSX2 or changing its configuration, firmware, or directories.
pub fn audit_pcsx2_bios(
    emulator_application_path: &Path,
    data_directories: &[PathBuf],
) -> Result<Pcsx2BiosAudit, Pcsx2BiosError> {
    if !emulator_application_path.is_absolute() {
        return Err(Pcsx2BiosError::NonAbsoluteExecutable {
            path: emulator_application_path.to_path_buf(),
        });
    }
    let requirements = pcsx2_bios_requirements()?;
    let location = locate_pcsx2_bios(emulator_application_path, data_directories)?;
    let files = audit_bios_directory(&location.bios_directory, &requirements);
    Ok(Pcsx2BiosAudit {
        bios_directory: location.bios_directory,
        configuration_path: location.configuration_path,
        location_source: location.source,
        files,
    })
}

pub fn pcsx2_bios_requirements() -> Result<Vec<Pcsx2BiosRequirement>, Pcsx2BiosError> {
    let mut requirements = Vec::new();
    let mut file_names = BTreeSet::new();
    let mut hashes = BTreeSet::new();
    for (index, line) in PCSX2_BIOS_CATALOG.lines().enumerate() {
        let mut columns = line.split('\t');
        let (Some(file_name), Some(md5), Some(description), None) = (
            columns.next(),
            columns.next(),
            columns.next(),
            columns.next(),
        ) else {
            return Err(Pcsx2BiosError::InvalidCatalog { line: index + 1 });
        };
        if file_name.is_empty()
            || description.is_empty()
            || md5.len() != 32
            || !md5.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(Pcsx2BiosError::InvalidCatalog { line: index + 1 });
        }
        let file_key = file_name.to_ascii_lowercase();
        if !file_names.insert(file_key) {
            return Err(Pcsx2BiosError::DuplicateCatalogValue {
                value: file_name.to_string(),
            });
        }
        let md5 = md5.to_ascii_lowercase();
        if !hashes.insert(md5.clone()) {
            return Err(Pcsx2BiosError::DuplicateCatalogValue { value: md5 });
        }
        requirements.push(Pcsx2BiosRequirement {
            file_name: file_name.to_string(),
            description: description.to_string(),
            md5,
        });
    }
    Ok(requirements)
}

fn locate_pcsx2_bios(
    emulator_application_path: &Path,
    data_directories: &[PathBuf],
) -> Result<Pcsx2BiosLocation, Pcsx2BiosError> {
    let application_directory = emulator_application_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(emulator_application_path);
    let portable = regular_file_without_symlink(&application_directory.join("portable.ini"));
    if portable {
        let configuration = application_directory.join("inis/PCSX2.ini");
        if regular_file_without_symlink(&configuration) {
            if let Some(path) = configured_bios_directory(&configuration, application_directory)? {
                return Ok(Pcsx2BiosLocation {
                    bios_directory: path,
                    configuration_path: Some(configuration),
                    source: Pcsx2BiosLocationSource::PortableConfiguration,
                });
            }
            return Ok(Pcsx2BiosLocation {
                bios_directory: application_directory.join("bios"),
                configuration_path: Some(configuration),
                source: Pcsx2BiosLocationSource::PortableDefault,
            });
        }
        return Ok(Pcsx2BiosLocation {
            bios_directory: application_directory.join("bios"),
            configuration_path: None,
            source: Pcsx2BiosLocationSource::PortableDefault,
        });
    }

    let application_inis = application_directory.join("inis");
    let mut roots = Vec::new();
    for path in data_directories.iter().filter(|path| {
        path.is_absolute()
            && path.as_path() != application_directory
            && path.as_path() != application_inis.as_path()
    }) {
        if !roots.contains(path) {
            roots.push(path.clone());
        }
    }
    let native_default = default_native_pcsx2_root();
    if let Some(root) = native_default.as_ref().filter(|root| !roots.contains(root)) {
        roots.push(root.clone());
    }
    for root in &roots {
        for configuration in [root.join("inis/PCSX2.ini"), root.join("PCSX2.ini")] {
            if !regular_file_without_symlink(&configuration) {
                continue;
            }
            if let Some(path) = configured_bios_directory(&configuration, root)? {
                return Ok(Pcsx2BiosLocation {
                    bios_directory: path,
                    configuration_path: Some(configuration),
                    source: Pcsx2BiosLocationSource::NativeConfiguration,
                });
            }
            return Ok(Pcsx2BiosLocation {
                bios_directory: root.join("bios"),
                configuration_path: Some(configuration),
                source: Pcsx2BiosLocationSource::NativeDefault,
            });
        }
    }
    if let Some(root) = roots.iter().find(|root| root.join("bios").is_dir()) {
        return Ok(Pcsx2BiosLocation {
            bios_directory: root.join("bios"),
            configuration_path: None,
            source: Pcsx2BiosLocationSource::NativeDefault,
        });
    }
    if let Some(root) = native_default {
        return Ok(Pcsx2BiosLocation {
            bios_directory: root.join("bios"),
            configuration_path: None,
            source: Pcsx2BiosLocationSource::NativeDefault,
        });
    }
    if let Some(root) = roots.first() {
        return Ok(Pcsx2BiosLocation {
            bios_directory: root.join("bios"),
            configuration_path: None,
            source: Pcsx2BiosLocationSource::NativeDefault,
        });
    }
    Ok(Pcsx2BiosLocation {
        bios_directory: application_directory.join("bios"),
        configuration_path: None,
        source: Pcsx2BiosLocationSource::ApplicationFallback,
    })
}

fn configured_bios_directory(
    configuration: &Path,
    relative_root: &Path,
) -> Result<Option<PathBuf>, Pcsx2BiosError> {
    let contents =
        fs::read_to_string(configuration).map_err(|source| Pcsx2BiosError::ReadConfiguration {
            path: configuration.to_path_buf(),
            source,
        })?;
    for line in contents.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if !key.trim().eq_ignore_ascii_case("bios") {
            continue;
        }
        let value = value.trim();
        if value.is_empty() {
            return Ok(None);
        }
        let path = PathBuf::from(value);
        if path.is_absolute() {
            return Ok(Some(path));
        }
        if is_windows_absolute_path(value) {
            return Err(Pcsx2BiosError::ForeignAbsoluteConfigurationPath {
                path: value.to_string(),
            });
        }
        return Ok(Some(relative_root.join(path)));
    }
    Ok(None)
}

fn audit_bios_directory(
    bios_directory: &Path,
    requirements: &[Pcsx2BiosRequirement],
) -> Vec<Pcsx2BiosFileAudit> {
    let unsafe_directory = fs::symlink_metadata(bios_directory)
        .is_ok_and(|metadata| metadata.file_type().is_symlink());
    requirements
        .iter()
        .cloned()
        .map(|requirement| {
            let path = bios_directory.join(&requirement.file_name);
            let (state, actual_md5) = if unsafe_directory {
                (BiosFileState::UnsafeEntry, None)
            } else {
                audit_file(&path, &requirement.md5)
            };
            Pcsx2BiosFileAudit {
                requirement,
                path,
                state,
                actual_md5,
            }
        })
        .collect()
}

fn audit_file(path: &Path, expected_md5: &str) -> (BiosFileState, Option<String>) {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return (BiosFileState::Missing, None);
        }
        Err(_) => return (BiosFileState::Unreadable, None),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return (BiosFileState::UnsafeEntry, None);
    }
    let Ok(actual_md5) = md5_file(path) else {
        return (BiosFileState::Unreadable, None);
    };
    let state = if actual_md5.eq_ignore_ascii_case(expected_md5) {
        BiosFileState::Valid
    } else {
        BiosFileState::HashMismatch
    };
    (state, Some(actual_md5))
}

fn md5_file(path: &Path) -> io::Result<String> {
    let mut source = File::open(path)?;
    let mut digest = Md5::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = source.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn regular_file_without_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
fn default_native_pcsx2_root() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .map(|path| path.join("PCSX2"))
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config/PCSX2")))
}

#[cfg(target_os = "macos")]
fn default_native_pcsx2_root() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| {
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("PCSX2")
    })
}

#[cfg(windows)]
fn default_native_pcsx2_root() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE").map(|profile| PathBuf::from(profile).join("Documents/PCSX2"))
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos",
    windows
)))]
fn default_native_pcsx2_root() -> Option<PathBuf> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn recovered_catalog_has_all_unique_alternatives() {
        let requirements = pcsx2_bios_requirements().expect("valid recovered catalog");
        assert_eq!(requirements.len(), 73);
        assert_eq!(
            requirements.first().map(|entry| entry.file_name.as_str()),
            Some("ps2-0100jd-20000117.bin")
        );
        assert_eq!(
            requirements.last().map(|entry| entry.file_name.as_str()),
            Some("ps2-0250j-20100415.bin")
        );
    }

    #[test]
    fn generic_group_audit_distinguishes_valid_mismatch_missing_and_unsafe() {
        let directory = tempfile::tempdir().expect("temporary BIOS directory");
        let valid_bytes = b"fixture BIOS bytes";
        let valid_hash = format!("{:x}", Md5::digest(valid_bytes));
        fs::write(directory.path().join("valid.bin"), valid_bytes).expect("valid BIOS fixture");
        fs::write(directory.path().join("mismatch.bin"), b"wrong").expect("mismatch fixture");
        fs::create_dir(directory.path().join("unsafe.bin")).expect("unsafe directory fixture");
        let requirements = vec![
            requirement("valid.bin", &valid_hash),
            requirement("mismatch.bin", "00000000000000000000000000000000"),
            requirement("missing.bin", "00000000000000000000000000000000"),
            requirement("unsafe.bin", "00000000000000000000000000000000"),
        ];

        let files = audit_bios_directory(directory.path(), &requirements);
        assert_eq!(files[0].state, BiosFileState::Valid);
        assert_eq!(files[0].actual_md5.as_deref(), Some(valid_hash.as_str()));
        assert_eq!(files[1].state, BiosFileState::HashMismatch);
        assert!(files[1].actual_md5.is_some());
        assert_eq!(files[2].state, BiosFileState::Missing);
        assert_eq!(files[3].state, BiosFileState::UnsafeEntry);
    }

    #[test]
    fn portable_configuration_resolves_relative_bios_directory_without_writing() {
        let directory = tempfile::tempdir().expect("temporary PCSX2");
        let application = directory.path().join("pcsx2-qt");
        fs::write(&application, b"executable").expect("application fixture");
        fs::write(directory.path().join("portable.ini"), b"").expect("portable marker");
        fs::create_dir_all(directory.path().join("inis")).expect("configuration directory");
        fs::write(
            directory.path().join("inis/PCSX2.ini"),
            b"[Folders]\nBios = firmware/custom\n",
        )
        .expect("configuration fixture");
        let before = snapshot(directory.path());

        let audit = audit_pcsx2_bios(&application, &[]).expect("portable BIOS audit");

        assert_eq!(
            audit.bios_directory,
            directory.path().join("firmware/custom")
        );
        assert_eq!(
            audit.configuration_path.as_deref(),
            Some(directory.path().join("inis/PCSX2.ini").as_path())
        );
        assert_eq!(
            audit.location_source,
            Pcsx2BiosLocationSource::PortableConfiguration
        );
        assert_eq!(audit.files.len(), 73);
        assert_eq!(audit.missing_count(), 73);
        assert_eq!(snapshot(directory.path()), before);
    }

    #[test]
    fn native_configuration_uses_its_data_root_and_rejects_foreign_paths() {
        let directory = tempfile::tempdir().expect("temporary PCSX2");
        let application_directory = directory.path().join("app");
        let data_root = directory.path().join("config/PCSX2");
        fs::create_dir_all(&application_directory).expect("application directory");
        fs::create_dir_all(application_directory.join("inis"))
            .expect("application-local configuration directory");
        fs::create_dir_all(data_root.join("inis")).expect("configuration directory");
        let application = application_directory.join("pcsx2-qt");
        fs::write(&application, b"executable").expect("application fixture");
        fs::write(
            application_directory.join("inis/PCSX2.ini"),
            b"Bios = must-not-win-without-portable-marker\n",
        )
        .expect("ignored application-local configuration");
        fs::write(data_root.join("inis/PCSX2.ini"), b"Bios = ../firmware\n")
            .expect("configuration fixture");

        let audit =
            audit_pcsx2_bios(&application, std::slice::from_ref(&data_root)).expect("native audit");
        assert_eq!(audit.bios_directory, data_root.join("../firmware"));
        assert_eq!(
            audit.location_source,
            Pcsx2BiosLocationSource::NativeConfiguration
        );

        fs::write(data_root.join("inis/PCSX2.ini"), b"Bios = C:\\firmware\n")
            .expect("foreign configuration");
        assert!(matches!(
            audit_pcsx2_bios(&application, &[data_root]),
            Err(Pcsx2BiosError::ForeignAbsoluteConfigurationPath { .. })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn bios_symlinks_are_reported_without_following_them() {
        use std::os::unix::fs::symlink;

        let directory = tempfile::tempdir().expect("temporary BIOS directory");
        let target = directory.path().join("outside.bin");
        fs::write(&target, b"outside").expect("symlink target");
        let link = directory.path().join("firmware.bin");
        symlink(&target, &link).expect("BIOS symlink");
        let files = audit_bios_directory(
            directory.path(),
            &[requirement(
                "firmware.bin",
                "00000000000000000000000000000000",
            )],
        );
        assert_eq!(files[0].state, BiosFileState::UnsafeEntry);
        assert_eq!(fs::read(target).expect("target remains"), b"outside");

        let real_directory = directory.path().join("real-bios");
        fs::create_dir(&real_directory).expect("real BIOS directory");
        fs::write(real_directory.join("firmware.bin"), b"outside directory")
            .expect("firmware behind directory symlink");
        let linked_directory = directory.path().join("linked-bios");
        symlink(&real_directory, &linked_directory).expect("BIOS directory symlink");
        let files = audit_bios_directory(
            &linked_directory,
            &[requirement(
                "firmware.bin",
                "00000000000000000000000000000000",
            )],
        );
        assert_eq!(files[0].state, BiosFileState::UnsafeEntry);
        assert_eq!(
            fs::read(real_directory.join("firmware.bin")).expect("directory target remains"),
            b"outside directory"
        );
    }

    fn requirement(file_name: &str, md5: &str) -> Pcsx2BiosRequirement {
        Pcsx2BiosRequirement {
            file_name: file_name.into(),
            description: "Fixture firmware".into(),
            md5: md5.into(),
        }
    }

    fn snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
        fn visit(root: &Path, directory: &Path, entries: &mut Vec<(PathBuf, Vec<u8>)>) {
            let mut children = fs::read_dir(directory)
                .expect("read snapshot directory")
                .map(|entry| entry.expect("snapshot entry"))
                .collect::<Vec<_>>();
            children.sort_by_key(fs::DirEntry::file_name);
            for child in children {
                let path = child.path();
                let relative = path
                    .strip_prefix(root)
                    .expect("relative snapshot")
                    .to_path_buf();
                if child.file_type().expect("snapshot type").is_dir() {
                    entries.push((relative.clone(), Vec::new()));
                    visit(root, &path, entries);
                } else {
                    entries.push((relative, fs::read(path).expect("snapshot file")));
                }
            }
        }
        let mut entries = Vec::new();
        visit(root, root, &mut entries);
        entries
    }
}
