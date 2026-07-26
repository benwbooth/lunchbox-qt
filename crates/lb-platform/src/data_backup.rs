use crate::{
    ArchiveCreationError, ArchiveExtractionError, ArchiveExtractionLimits, ArchiveExtractor,
};
use chrono::NaiveDateTime;
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
pub const AUTOMATIC_DATA_BACKUP_RETENTION: usize = 25;
const AUTOMATIC_DATA_BACKUP_TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H-%M-%S";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AutomaticDataBackupKind {
    LaunchBoxStartup,
    LaunchBoxShutdown,
    BigBoxStartup,
    BigBoxShutdown,
}

impl AutomaticDataBackupKind {
    pub const ALL: [Self; 4] = [
        Self::LaunchBoxStartup,
        Self::LaunchBoxShutdown,
        Self::BigBoxStartup,
        Self::BigBoxShutdown,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::LaunchBoxStartup => "Automatic LaunchBox Startup Data Backup",
            Self::LaunchBoxShutdown => "Automatic LaunchBox Shutdown Data Backup",
            Self::BigBoxStartup => "Automatic Big Box Startup Data Backup",
            Self::BigBoxShutdown => "Automatic Big Box Shutdown Data Backup",
        }
    }

    pub const fn event_label(self) -> &'static str {
        match self {
            Self::LaunchBoxStartup | Self::BigBoxStartup => "startup",
            Self::LaunchBoxShutdown | Self::BigBoxShutdown => "shutdown",
        }
    }

    pub const fn frontend_label(self) -> &'static str {
        match self {
            Self::LaunchBoxStartup | Self::LaunchBoxShutdown => "launchbox",
            Self::BigBoxStartup | Self::BigBoxShutdown => "bigbox",
        }
    }

    pub fn archive_name(self, timestamp: NaiveDateTime) -> String {
        format!(
            "{} {}.7z",
            self.label(),
            timestamp.format(AUTOMATIC_DATA_BACKUP_TIMESTAMP_FORMAT)
        )
    }

    fn parse_name(name: &str) -> Option<(Self, NaiveDateTime)> {
        let stem = name.strip_suffix(".7z")?;
        Self::ALL.into_iter().find_map(|kind| {
            let timestamp = stem.strip_prefix(kind.label())?.strip_prefix(' ')?;
            let timestamp =
                NaiveDateTime::parse_from_str(timestamp, AUTOMATIC_DATA_BACKUP_TIMESTAMP_FORMAT)
                    .ok()?;
            Some((kind, timestamp))
        })
    }
}

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutomaticDataBackupReport {
    pub backup: DataBackupReport,
    pub kind: AutomaticDataBackupKind,
    pub removed_archives: Vec<PathBuf>,
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

    /// Creates one observed-name automatic archive and retains only the 25
    /// newest recognized automatic data backups.
    ///
    /// Manual and unrecognized files are never retention candidates.
    pub fn create_automatic(
        &self,
        data_directory: &Path,
        backup_directory: &Path,
        kind: AutomaticDataBackupKind,
        timestamp: NaiveDateTime,
    ) -> Result<AutomaticDataBackupReport, DataBackupError> {
        ensure_real_backup_directory(backup_directory)?;
        let archive = backup_directory.join(kind.archive_name(timestamp));
        let backup = self.create(data_directory, &archive)?;
        let removed_archives =
            prune_automatic_data_backups(backup_directory, AUTOMATIC_DATA_BACKUP_RETENTION)?;
        Ok(AutomaticDataBackupReport {
            backup,
            kind,
            removed_archives,
        })
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

fn ensure_real_backup_directory(directory: &Path) -> Result<(), DataBackupError> {
    match fs::symlink_metadata(directory) {
        Ok(metadata) if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() => {
            Ok(())
        }
        Ok(_) => Err(DataBackupError::BackupDirectoryNotReal {
            path: directory.to_path_buf(),
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => fs::create_dir(directory)
            .map_err(|source| DataBackupError::Io {
                path: directory.to_path_buf(),
                source,
            }),
        Err(source) => Err(DataBackupError::Io {
            path: directory.to_path_buf(),
            source,
        }),
    }
}

fn prune_automatic_data_backups(
    directory: &Path,
    retention: usize,
) -> Result<Vec<PathBuf>, DataBackupError> {
    let mut candidates = fs::read_dir(directory)
        .map_err(|source| DataBackupError::Io {
            path: directory.to_path_buf(),
            source,
        })?
        .filter_map(|result| result.ok())
        .filter_map(|entry| {
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).ok()?;
            if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
                return None;
            }
            let name = entry.file_name().into_string().ok()?;
            let (kind, timestamp) = AutomaticDataBackupKind::parse_name(&name)?;
            Some((timestamp, kind, name, path))
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| left.2.cmp(&right.2))
    });

    let remove_count = candidates.len().saturating_sub(retention);
    let mut removed = Vec::with_capacity(remove_count);
    for (_, _, _, path) in candidates.into_iter().take(remove_count) {
        let metadata =
            fs::symlink_metadata(&path).map_err(|source| DataBackupError::AutomaticRetention {
                path: path.clone(),
                source,
            })?;
        if !metadata.file_type().is_file()
            || metadata.file_type().is_symlink()
            || path
                .file_name()
                .and_then(|name| name.to_str())
                .and_then(AutomaticDataBackupKind::parse_name)
                .is_none()
        {
            return Err(DataBackupError::AutomaticRetentionChanged { path });
        }
        fs::remove_file(&path).map_err(|source| DataBackupError::AutomaticRetention {
            path: path.clone(),
            source,
        })?;
        removed.push(path);
    }
    Ok(removed)
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
    #[error("automatic data-backup directory is not a real directory: {path}")]
    BackupDirectoryNotReal { path: PathBuf },
    #[error("automatic data-backup retention candidate changed before deletion: {path}")]
    AutomaticRetentionChanged { path: PathBuf },
    #[error("could not remove expired automatic data backup {path}: {source}")]
    AutomaticRetention {
        path: PathBuf,
        #[source]
        source: std::io::Error,
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

    #[test]
    fn automatic_names_are_exact_and_retention_ignores_manual_unknown_and_links() {
        let directory = tempfile::tempdir().unwrap();
        let backups = directory.path().join("Backups");
        fs::create_dir(&backups).unwrap();
        for day in 1..=27 {
            let timestamp = NaiveDateTime::parse_from_str(
                &format!("2026-01-{day:02} 12-00-00"),
                AUTOMATIC_DATA_BACKUP_TIMESTAMP_FORMAT,
            )
            .unwrap();
            let kind = AutomaticDataBackupKind::ALL[(day - 1) % 4];
            fs::write(backups.join(kind.archive_name(timestamp)), [day as u8]).unwrap();
        }
        let manual = backups.join("Manual Data Backup.7z");
        let unknown = backups.join("Automatic LaunchBox Startup Data Backup not-a-date.7z");
        fs::write(&manual, b"manual").unwrap();
        fs::write(&unknown, b"unknown").unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(
                &manual,
                backups.join("Automatic LaunchBox Startup Data Backup 2025-01-01 00-00-00.7z"),
            )
            .unwrap();
        }

        let removed =
            prune_automatic_data_backups(&backups, AUTOMATIC_DATA_BACKUP_RETENTION).unwrap();
        assert_eq!(removed.len(), 2);
        assert!(removed.iter().all(|path| !path.exists()));
        assert_eq!(fs::read(&manual).unwrap(), b"manual");
        assert_eq!(fs::read(&unknown).unwrap(), b"unknown");
        assert_eq!(
            fs::read_dir(&backups)
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| {
                    entry.file_type().is_ok_and(|file_type| file_type.is_file())
                        && entry
                            .file_name()
                            .to_str()
                            .and_then(AutomaticDataBackupKind::parse_name)
                            .is_some()
                })
                .count(),
            AUTOMATIC_DATA_BACKUP_RETENTION
        );
    }

    #[test]
    fn creates_an_exact_automatic_backup_and_prunes_only_recognized_archives() {
        let fixture = tempfile::tempdir().unwrap();
        write_fixture(fixture.path(), "Original");
        let backups = fixture.path().join("Backups");
        fs::create_dir(&backups).unwrap();
        for day in 1..=AUTOMATIC_DATA_BACKUP_RETENTION {
            let timestamp = NaiveDateTime::parse_from_str(
                &format!("2025-01-{day:02} 00-00-00"),
                AUTOMATIC_DATA_BACKUP_TIMESTAMP_FORMAT,
            )
            .unwrap();
            fs::write(
                backups.join(AutomaticDataBackupKind::LaunchBoxStartup.archive_name(timestamp)),
                b"old",
            )
            .unwrap();
        }
        let manual = backups.join("Custom LaunchBox Data Backup 2025.7z");
        fs::write(&manual, b"manual").unwrap();
        let service =
            DataBackupService::new(ArchiveExtractor::new("7z"), DataBackupLimits::default());
        let timestamp =
            NaiveDateTime::parse_from_str("2026-07-26 17-30-00", "%Y-%m-%d %H-%M-%S").unwrap();

        let report = service
            .create_automatic(
                &fixture.path().join("Data"),
                &backups,
                AutomaticDataBackupKind::BigBoxShutdown,
                timestamp,
            )
            .unwrap();

        assert_eq!(report.kind, AutomaticDataBackupKind::BigBoxShutdown);
        assert_eq!(report.removed_archives.len(), 1);
        assert_eq!(
            report.backup.archive.file_name().unwrap(),
            "Automatic Big Box Shutdown Data Backup 2026-07-26 17-30-00.7z"
        );
        assert!(report.backup.archive.is_file());
        assert_eq!(fs::read(manual).unwrap(), b"manual");
    }
}
