use crate::{
    ArchiveCreationError, ArchiveExtractionError, ArchiveExtractionLimits, ArchiveExtractor,
};
use quick_xml::events::Event;
use quick_xml::Reader;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use tempfile::{Builder, TempDir};
use thiserror::Error;

const REQUIRED_ROOT_FILES: &[&str] = &[
    "BigBoxSettings.xml",
    "Emulators.xml",
    "GameControllers.xml",
    "InputBindings.xml",
    "ListCache.xml",
    "Parents.xml",
    "Platforms.xml",
    "Settings.xml",
];
const REQUIRED_ROOT_DIRECTORIES: &[&str] = &["Platforms", "Playlists"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DataBackupLimits {
    pub max_archive_bytes: u64,
    pub max_entries: usize,
    pub max_member_bytes: u64,
    pub max_total_bytes: u64,
}

impl Default for DataBackupLimits {
    fn default() -> Self {
        Self {
            max_archive_bytes: 2 * 1024 * 1024 * 1024,
            max_entries: 200_000,
            max_member_bytes: 1024 * 1024 * 1024,
            max_total_bytes: 8 * 1024 * 1024 * 1024,
        }
    }
}

impl DataBackupLimits {
    fn archive_limits(self) -> ArchiveExtractionLimits {
        ArchiveExtractionLimits {
            max_entries: self.max_entries,
            max_member_bytes: self.max_member_bytes,
            max_total_bytes: self.max_total_bytes,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataTreeRevision {
    pub file_count: u64,
    pub directory_count: u64,
    pub byte_len: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataBackupReport {
    pub archive: PathBuf,
    pub revision: DataTreeRevision,
}

#[derive(Clone, Debug)]
pub struct DataBackupService {
    extractor: ArchiveExtractor,
    limits: DataBackupLimits,
}

impl DataBackupService {
    pub fn for_launchbox_root(launchbox_root: &Path) -> Self {
        Self {
            extractor: ArchiveExtractor::for_launchbox_root(launchbox_root),
            limits: DataBackupLimits::default(),
        }
    }

    pub fn new(extractor: ArchiveExtractor, limits: DataBackupLimits) -> Self {
        Self { extractor, limits }
    }

    pub fn limits(&self) -> DataBackupLimits {
        self.limits
    }

    /// Creates and re-extracts one exact LaunchBox `Data` snapshot.
    ///
    /// The source is hashed before and after archive creation. The extracted
    /// archive must have the same complete content revision before the new
    /// backup is accepted.
    pub fn create(
        &self,
        data_directory: &Path,
        archive: &Path,
    ) -> Result<DataBackupReport, DataBackupError> {
        let initial = inspect_data_tree(data_directory, self.limits)?;
        self.extractor
            .create_7z_from_tree(data_directory, archive)?;

        let verification = self.prepare_restore(archive);
        let current = inspect_data_tree(data_directory, self.limits);
        let result = match (verification, current) {
            (Ok(_prepared), Ok(current)) if current != initial => {
                Err(DataBackupError::SourceChanged {
                    expected: initial.clone(),
                    actual: current,
                })
            }
            (Ok(prepared), Ok(_)) if prepared.revision != initial => {
                Err(DataBackupError::VerificationMismatch {
                    expected: initial.clone(),
                    actual: prepared.revision.clone(),
                })
            }
            (Ok(_), Ok(_)) => Ok(DataBackupReport {
                archive: archive.to_path_buf(),
                revision: initial,
            }),
            (Err(error), _) => Err(error),
            (_, Err(error)) => Err(error),
        };
        if result.is_err() {
            let _ = fs::remove_file(archive);
        }
        result
    }

    /// Extracts and validates a candidate 13.27-compatible `Data` snapshot
    /// into a private directory. The caller owns the final atomic replacement.
    pub fn prepare_restore(&self, archive: &Path) -> Result<PreparedDataRestore, DataBackupError> {
        let metadata = fs::symlink_metadata(archive).map_err(|source| DataBackupError::Io {
            path: archive.to_path_buf(),
            source,
        })?;
        if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
            return Err(DataBackupError::ArchiveNotRegular {
                path: archive.to_path_buf(),
            });
        }
        if !archive
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("7z"))
        {
            return Err(DataBackupError::ArchiveExtension {
                path: archive.to_path_buf(),
            });
        }
        if metadata.len() > self.limits.max_archive_bytes {
            return Err(DataBackupError::ArchiveTooLarge {
                path: archive.to_path_buf(),
                actual: metadata.len(),
                limit: self.limits.max_archive_bytes,
            });
        }

        let directory = Builder::new()
            .prefix("launchbox-data-restore-")
            .tempdir()
            .map_err(|source| DataBackupError::Io {
                path: std::env::temp_dir(),
                source,
            })?;
        self.extractor.extract_to_directory_bounded(
            archive,
            directory.path(),
            self.limits.archive_limits(),
        )?;
        let revision = inspect_data_tree(directory.path(), self.limits)?;
        Ok(PreparedDataRestore {
            directory,
            revision,
        })
    }
}

pub struct PreparedDataRestore {
    directory: TempDir,
    revision: DataTreeRevision,
}

impl PreparedDataRestore {
    pub fn data_directory(&self) -> &Path {
        self.directory.path()
    }

    pub fn revision(&self) -> &DataTreeRevision {
        &self.revision
    }
}

impl std::fmt::Debug for PreparedDataRestore {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PreparedDataRestore")
            .field("data_directory", &self.data_directory())
            .field("revision", &self.revision)
            .finish()
    }
}

pub fn inspect_data_tree(
    data_directory: &Path,
    limits: DataBackupLimits,
) -> Result<DataTreeRevision, DataBackupError> {
    let metadata = fs::symlink_metadata(data_directory).map_err(|source| DataBackupError::Io {
        path: data_directory.to_path_buf(),
        source,
    })?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(DataBackupError::DataNotDirectory {
            path: data_directory.to_path_buf(),
        });
    }
    let root = fs::canonicalize(data_directory).map_err(|source| DataBackupError::Io {
        path: data_directory.to_path_buf(),
        source,
    })?;
    validate_required_layout(&root)?;

    let mut builder = DataTreeRevisionBuilder {
        limits,
        ..Default::default()
    };
    builder.visit(&root, Path::new(""))?;
    validate_launchbox_xml(&root.join("Settings.xml"), Some(b"Settings"))?;
    validate_launchbox_xml(&root.join("BigBoxSettings.xml"), Some(b"BigBoxSettings"))?;
    for name in REQUIRED_ROOT_FILES {
        if matches!(*name, "Settings.xml" | "BigBoxSettings.xml") {
            continue;
        }
        validate_launchbox_xml(&root.join(name), None)?;
    }
    for directory in REQUIRED_ROOT_DIRECTORIES {
        validate_document_directory(&root.join(directory))?;
    }
    Ok(builder.finish())
}

#[derive(Default)]
struct DataTreeRevisionBuilder {
    limits: DataBackupLimits,
    file_count: u64,
    directory_count: u64,
    byte_len: u64,
    identities: BTreeSet<String>,
    digest: Sha256,
}

impl DataTreeRevisionBuilder {
    fn visit(&mut self, root: &Path, relative: &Path) -> Result<(), DataBackupError> {
        let directory = root.join(relative);
        let mut entries = fs::read_dir(&directory)
            .map_err(|source| DataBackupError::Io {
                path: directory.clone(),
                source,
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|source| DataBackupError::Io {
                path: directory.clone(),
                source,
            })?;
        entries.sort_by_key(|entry| entry.file_name());

        for entry in entries {
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| DataBackupError::UnsafeEntry { path: entry.path() })?;
            if !is_portable_component(&name) {
                return Err(DataBackupError::UnsafeEntry { path: entry.path() });
            }
            let child_relative = relative.join(&name);
            let portable = child_relative
                .to_str()
                .expect("all components were checked as Unicode")
                .replace('\\', "/");
            if !self.identities.insert(portable.to_ascii_lowercase()) {
                return Err(DataBackupError::DuplicateEntry { path: portable });
            }
            let entry_count = self
                .file_count
                .checked_add(self.directory_count)
                .and_then(|count| count.checked_add(1))
                .unwrap_or(u64::MAX);
            if entry_count > u64::try_from(self.limits.max_entries).unwrap_or(u64::MAX) {
                return Err(DataBackupError::EntryLimit {
                    actual: entry_count,
                    limit: self.limits.max_entries,
                });
            }

            let path = root.join(&child_relative);
            let metadata = fs::symlink_metadata(&path).map_err(|source| DataBackupError::Io {
                path: path.clone(),
                source,
            })?;
            if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
                self.directory_count = self.directory_count.saturating_add(1);
                self.update_header(b'D', &portable, 0);
                self.visit(root, &child_relative)?;
            } else if metadata.file_type().is_file() && !metadata.file_type().is_symlink() {
                let size = metadata.len();
                if size > self.limits.max_member_bytes {
                    return Err(DataBackupError::MemberSizeLimit {
                        path,
                        actual: size,
                        limit: self.limits.max_member_bytes,
                    });
                }
                self.byte_len =
                    self.byte_len
                        .checked_add(size)
                        .ok_or(DataBackupError::TotalSizeLimit {
                            actual: u64::MAX,
                            limit: self.limits.max_total_bytes,
                        })?;
                if self.byte_len > self.limits.max_total_bytes {
                    return Err(DataBackupError::TotalSizeLimit {
                        actual: self.byte_len,
                        limit: self.limits.max_total_bytes,
                    });
                }
                self.file_count = self.file_count.saturating_add(1);
                self.update_header(b'F', &portable, size);
                let mut file = fs::File::open(&path).map_err(|source| DataBackupError::Io {
                    path: path.clone(),
                    source,
                })?;
                let mut buffer = [0_u8; 64 * 1024];
                loop {
                    let read = file
                        .read(&mut buffer)
                        .map_err(|source| DataBackupError::Io {
                            path: path.clone(),
                            source,
                        })?;
                    if read == 0 {
                        break;
                    }
                    self.digest.update(&buffer[..read]);
                }
            } else {
                return Err(DataBackupError::UnsafeEntry { path });
            }
        }
        Ok(())
    }

    fn update_header(&mut self, kind: u8, path: &str, size: u64) {
        self.digest.update([kind]);
        self.digest.update((path.len() as u64).to_le_bytes());
        self.digest.update(path.as_bytes());
        self.digest.update(size.to_le_bytes());
    }

    fn finish(self) -> DataTreeRevision {
        DataTreeRevision {
            file_count: self.file_count,
            directory_count: self.directory_count,
            byte_len: self.byte_len,
            sha256: format!("{:x}", self.digest.finalize()),
        }
    }
}

fn validate_required_layout(root: &Path) -> Result<(), DataBackupError> {
    for name in REQUIRED_ROOT_FILES {
        let path = root.join(name);
        let metadata = fs::symlink_metadata(&path)
            .map_err(|_| DataBackupError::MissingRequiredEntry { path: path.clone() })?;
        if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
            return Err(DataBackupError::MissingRequiredEntry { path });
        }
    }
    for name in REQUIRED_ROOT_DIRECTORIES {
        let path = root.join(name);
        let metadata = fs::symlink_metadata(&path)
            .map_err(|_| DataBackupError::MissingRequiredEntry { path: path.clone() })?;
        if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
            return Err(DataBackupError::MissingRequiredEntry { path });
        }
    }
    Ok(())
}

fn validate_document_directory(directory: &Path) -> Result<(), DataBackupError> {
    let entries = fs::read_dir(directory).map_err(|source| DataBackupError::Io {
        path: directory.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| DataBackupError::Io {
            path: directory.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|source| DataBackupError::Io {
            path: path.clone(),
            source,
        })?;
        if metadata.file_type().is_file()
            && path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("xml"))
        {
            validate_launchbox_xml(&path, None)?;
        }
    }
    Ok(())
}

fn validate_launchbox_xml(
    path: &Path,
    required_child: Option<&[u8]>,
) -> Result<(), DataBackupError> {
    let file = fs::File::open(path).map_err(|source| DataBackupError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut reader = Reader::from_reader(BufReader::new(file));
    let mut buffer = Vec::new();
    let mut depth = 0_usize;
    let mut root_seen = false;
    let mut child_seen = required_child.is_none();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(|source| DataBackupError::InvalidXml {
                path: path.to_path_buf(),
                message: source.to_string(),
            })? {
            Event::Start(element) => {
                if depth == 0 {
                    if root_seen || element.name().as_ref() != b"LaunchBox" {
                        return Err(DataBackupError::InvalidXml {
                            path: path.to_path_buf(),
                            message: "the document root is not LaunchBox".into(),
                        });
                    }
                    root_seen = true;
                } else if depth == 1
                    && required_child.is_some_and(|required| element.name().as_ref() == required)
                {
                    child_seen = true;
                }
                depth = depth.saturating_add(1);
            }
            Event::Empty(element) => {
                if depth == 0 {
                    if root_seen || element.name().as_ref() != b"LaunchBox" {
                        return Err(DataBackupError::InvalidXml {
                            path: path.to_path_buf(),
                            message: "the document root is not LaunchBox".into(),
                        });
                    }
                    root_seen = true;
                } else if depth == 1
                    && required_child.is_some_and(|required| element.name().as_ref() == required)
                {
                    child_seen = true;
                }
            }
            Event::End(_) => depth = depth.saturating_sub(1),
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    if !root_seen {
        return Err(DataBackupError::InvalidXml {
            path: path.to_path_buf(),
            message: "the document has no root element".into(),
        });
    }
    if !child_seen {
        return Err(DataBackupError::InvalidXml {
            path: path.to_path_buf(),
            message: format!(
                "the LaunchBox document has no required {} element",
                String::from_utf8_lossy(required_child.expect("missing child was required"))
            ),
        });
    }
    Ok(())
}

fn is_portable_component(component: &str) -> bool {
    if component.is_empty()
        || component.len() > 255
        || matches!(component, "." | "..")
        || component.ends_with(['.', ' '])
        || component
            .chars()
            .any(|character| character.is_control() || r#"<>:"|?*\/"#.contains(character))
    {
        return false;
    }
    let stem = component
        .split_once('.')
        .map_or(component, |(stem, _)| stem)
        .to_ascii_lowercase();
    !matches!(stem.as_str(), "con" | "prn" | "aux" | "nul")
        && !(stem.len() == 4
            && (stem.starts_with("com") || stem.starts_with("lpt"))
            && matches!(stem.as_bytes()[3], b'1'..=b'9'))
}

#[derive(Debug, Error)]
pub enum DataBackupError {
    #[error(transparent)]
    ArchiveCreation(#[from] ArchiveCreationError),
    #[error(transparent)]
    ArchiveExtraction(#[from] ArchiveExtractionError),
    #[error("LaunchBox data source is not a real directory: {path}")]
    DataNotDirectory { path: PathBuf },
    #[error("data backup archive is not a real regular file: {path}")]
    ArchiveNotRegular { path: PathBuf },
    #[error("data backup archive must use the .7z extension: {path}")]
    ArchiveExtension { path: PathBuf },
    #[error("data backup archive {path} is {actual} bytes, above the {limit}-byte limit")]
    ArchiveTooLarge {
        path: PathBuf,
        actual: u64,
        limit: u64,
    },
    #[error("LaunchBox data snapshot is missing required file or directory {path}")]
    MissingRequiredEntry { path: PathBuf },
    #[error("LaunchBox data snapshot contains an unsafe entry: {path}")]
    UnsafeEntry { path: PathBuf },
    #[error("LaunchBox data snapshot contains a case-insensitive duplicate entry: {path}")]
    DuplicateEntry { path: String },
    #[error("LaunchBox data snapshot contains {actual} entries, above the {limit}-entry limit")]
    EntryLimit { actual: u64, limit: usize },
    #[error("LaunchBox data file {path} is {actual} bytes, above the {limit}-byte member limit")]
    MemberSizeLimit {
        path: PathBuf,
        actual: u64,
        limit: u64,
    },
    #[error(
        "LaunchBox data snapshot is at least {actual} bytes, above the {limit}-byte total limit"
    )]
    TotalSizeLimit { actual: u64, limit: u64 },
    #[error("invalid LaunchBox XML file {path}: {message}")]
    InvalidXml { path: PathBuf, message: String },
    #[error("LaunchBox Data changed while its backup was being created")]
    SourceChanged {
        expected: DataTreeRevision,
        actual: DataTreeRevision,
    },
    #[error("the verified archive does not match the LaunchBox Data snapshot")]
    VerificationMismatch {
        expected: DataTreeRevision,
        actual: DataTreeRevision,
    },
    #[error("could not read or write {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_fixture(root: &Path, title: &str) {
        let data = root.join("Data");
        fs::create_dir_all(data.join("Platforms")).unwrap();
        fs::create_dir_all(data.join("Playlists")).unwrap();
        for name in REQUIRED_ROOT_FILES {
            let body = match *name {
                "Settings.xml" => {
                    "<LaunchBox><Settings><AutoBackup>true</AutoBackup></Settings></LaunchBox>"
                        .to_string()
                }
                "BigBoxSettings.xml" => {
                    "<LaunchBox><BigBoxSettings><Theme>Default</Theme></BigBoxSettings></LaunchBox>"
                        .to_string()
                }
                _ => "<LaunchBox />".to_string(),
            };
            fs::write(data.join(name), body).unwrap();
        }
        fs::write(
            data.join("Platforms/Fixture Console.xml"),
            format!("<LaunchBox><Game><Title>{title}</Title></Game></LaunchBox>"),
        )
        .unwrap();
        fs::write(
            data.join("Playlists/Favorites.xml"),
            "<LaunchBox><Playlist><Title>Favorites</Title></Playlist></LaunchBox>",
        )
        .unwrap();
        fs::write(data.join("FuturePlugin.bin"), b"preserve unknown data").unwrap();
    }

    #[test]
    fn creates_and_reopens_an_exact_data_archive_without_a_shell() {
        let fixture = tempfile::tempdir().unwrap();
        write_fixture(fixture.path(), "Original");
        let backups = fixture.path().join("Backups");
        fs::create_dir(&backups).unwrap();
        let archive = backups.join("Manual Data Backup.7z");
        let service =
            DataBackupService::new(ArchiveExtractor::new("7z"), DataBackupLimits::default());

        let report = service
            .create(&fixture.path().join("Data"), &archive)
            .unwrap();
        assert!(archive.is_file());
        assert_eq!(report.revision.file_count, 11);
        let prepared = service.prepare_restore(&archive).unwrap();
        assert_eq!(prepared.revision(), &report.revision);
        assert_eq!(
            fs::read(prepared.data_directory().join("FuturePlugin.bin")).unwrap(),
            b"preserve unknown data"
        );
    }

    #[test]
    fn rejects_missing_core_files_malformed_xml_and_unsafe_entries() {
        let fixture = tempfile::tempdir().unwrap();
        write_fixture(fixture.path(), "Original");
        let data = fixture.path().join("Data");
        fs::remove_file(data.join("Settings.xml")).unwrap();
        assert!(matches!(
            inspect_data_tree(&data, DataBackupLimits::default()),
            Err(DataBackupError::MissingRequiredEntry { .. })
        ));

        write_fixture(fixture.path(), "Original");
        fs::write(data.join("Settings.xml"), "<Other />").unwrap();
        assert!(matches!(
            inspect_data_tree(&data, DataBackupLimits::default()),
            Err(DataBackupError::InvalidXml { .. })
        ));

        write_fixture(fixture.path(), "Original");
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(
                data.join("Settings.xml"),
                data.join("Platforms/unsafe.xml"),
            )
            .unwrap();
            assert!(matches!(
                inspect_data_tree(&data, DataBackupLimits::default()),
                Err(DataBackupError::UnsafeEntry { .. })
            ));
        }

        let archive_fixture = tempfile::tempdir().unwrap();
        let incomplete = archive_fixture.path().join("Incomplete");
        let backups = archive_fixture.path().join("Backups");
        fs::create_dir_all(incomplete.join("Platforms")).unwrap();
        fs::create_dir_all(incomplete.join("Playlists")).unwrap();
        fs::create_dir(&backups).unwrap();
        fs::write(incomplete.join("Settings.xml"), "<LaunchBox />").unwrap();
        let archive = backups.join("Incomplete.7z");
        let extractor = ArchiveExtractor::new("7z");
        extractor
            .create_7z_from_tree(&incomplete, &archive)
            .unwrap();
        let service = DataBackupService::new(extractor, DataBackupLimits::default());
        assert!(matches!(
            service.prepare_restore(&archive),
            Err(DataBackupError::MissingRequiredEntry { .. })
        ));
    }

    #[test]
    fn enforces_snapshot_size_limits_before_archiving() {
        let fixture = tempfile::tempdir().unwrap();
        write_fixture(fixture.path(), "Original");
        let limits = DataBackupLimits {
            max_member_bytes: 8,
            ..DataBackupLimits::default()
        };
        assert!(matches!(
            inspect_data_tree(&fixture.path().join("Data"), limits),
            Err(DataBackupError::MemberSizeLimit { .. })
        ));
    }
}
