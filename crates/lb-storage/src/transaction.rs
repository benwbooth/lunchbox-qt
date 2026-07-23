use super::{
    create_unique_sibling, replace_file, sync_parent_directory, AtomicSaveReport,
    AuxiliaryDocument, PlatformDocument, StorageError,
};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::fs;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

const MANIFEST_VERSION: u32 = 2;
const LEGACY_MANIFEST_VERSION: u32 = 1;
const MANIFEST_PREFIX: &str = ".lbport-transaction-";
const MANIFEST_SUFFIX: &str = ".json";
const LOCK_FILE_NAME: &str = ".lbport-transaction.lock";

/// Exact source revision used for optimistic conflict detection. Timestamps
/// are deliberately excluded because copies and Windows filesystem timestamp
/// granularity can otherwise hide a changed document.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FileRevision {
    pub byte_len: u64,
    pub sha256: String,
}

impl FileRevision {
    pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        let digest = Sha256::digest(bytes);
        let mut sha256 = String::with_capacity(digest.len() * 2);
        for byte in digest {
            write!(&mut sha256, "{byte:02x}").expect("writing to a String cannot fail");
        }
        Self {
            byte_len: bytes.len().try_into().unwrap_or(u64::MAX),
            sha256,
        }
    }

    pub fn read(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let path = path.as_ref();
        let mut file = fs::File::open(path).map_err(|source| StorageError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let mut digest = Sha256::new();
        let mut byte_len = 0_u64;
        let mut buffer = vec![0_u8; 1024 * 1024];
        loop {
            let read = file
                .read(&mut buffer)
                .map_err(|source| StorageError::Read {
                    path: path.to_path_buf(),
                    source,
                })?;
            if read == 0 {
                break;
            }
            digest.update(&buffer[..read]);
            byte_len = byte_len.saturating_add(read.try_into().unwrap_or(u64::MAX));
        }
        let digest = digest.finalize();
        let mut sha256 = String::with_capacity(digest.len() * 2);
        for byte in digest {
            write!(&mut sha256, "{byte:02x}").expect("writing to a String cannot fail");
        }
        Ok(Self { byte_len, sha256 })
    }
}

#[derive(Clone, Debug)]
struct PendingChange {
    target: PathBuf,
    operation: TransactionOperation,
    candidate: Option<PendingCandidate>,
    expected: Option<FileRevision>,
}

#[derive(Clone, Debug)]
enum PendingCandidate {
    Bytes(Vec<u8>),
    SourceFile {
        path: PathBuf,
        expected: Option<FileRevision>,
    },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TransactionOperation {
    #[default]
    Replace,
    Create,
    Delete,
}

/// A recoverable set of platform and auxiliary document replacements.
/// Documents must have been loaded from files so every write has an exact
/// source revision.
#[derive(Debug)]
pub struct LibraryTransaction {
    root: PathBuf,
    changes: Vec<PendingChange>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionReport {
    pub manifest: PathBuf,
    pub writes: Vec<AtomicSaveReport>,
    pub created_targets: Vec<PathBuf>,
    pub deleted_targets: Vec<AtomicSaveReport>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryReport {
    pub manifest: PathBuf,
    pub restored_targets: Vec<PathBuf>,
}

impl LibraryTransaction {
    pub fn new(root: impl AsRef<Path>) -> Result<Self, TransactionError> {
        let supplied = root.as_ref();
        let root = fs::canonicalize(supplied).map_err(|source| TransactionError::Io {
            path: supplied.to_path_buf(),
            source,
        })?;
        if !root.is_dir() {
            return Err(TransactionError::RootNotDirectory { path: root });
        }
        Ok(Self {
            root,
            changes: Vec::new(),
        })
    }

    pub fn stage_platform(&mut self, document: &PlatformDocument) -> Result<(), TransactionError> {
        let expected = document.source_revision().cloned().ok_or_else(|| {
            TransactionError::UnversionedDocument {
                path: document.source_path().to_path_buf(),
            }
        })?;
        let candidate = document.to_xml_bytes()?;
        self.stage_replace(document.source_path(), candidate, expected)
    }

    pub fn stage_auxiliary(
        &mut self,
        document: &AuxiliaryDocument,
    ) -> Result<(), TransactionError> {
        let actual = super::AuxiliaryDocumentKind::infer(document.source_path())?;
        if actual != document.kind() {
            return Err(StorageError::AuxiliaryDocumentKindMismatch {
                path: document.source_path().to_path_buf(),
                expected: document.kind(),
                actual,
            }
            .into());
        }
        let expected = document.source_revision().cloned().ok_or_else(|| {
            TransactionError::UnversionedDocument {
                path: document.source_path().to_path_buf(),
            }
        })?;
        let candidate = document.to_xml_bytes()?;
        self.stage_replace(document.source_path(), candidate, expected)
    }

    /// Stages a newly constructed playlist document. Its parent directory
    /// must already exist and the portable target must remain absent through
    /// commit.
    pub fn stage_new_playlist(
        &mut self,
        document: &AuxiliaryDocument,
    ) -> Result<(), TransactionError> {
        if document.kind() != super::AuxiliaryDocumentKind::Playlist {
            return Err(StorageError::UnsupportedAuxiliaryOperation {
                operation: "create playlist transaction",
                expected: super::AuxiliaryDocumentKind::Playlist,
                actual: document.kind(),
            }
            .into());
        }
        if document.source_revision().is_some() {
            return Err(TransactionError::VersionedNewDocument {
                path: document.source_path().to_path_buf(),
            });
        }
        let candidate = document.to_xml_bytes()?;
        self.stage_create(document.source_path(), candidate)
    }

    /// Stages deletion of one existing playlist document. The playlist's
    /// membership/filter rows belong to the playlist; deleting this file does
    /// not delete any game or media record.
    pub fn stage_delete_playlist(
        &mut self,
        document: &AuxiliaryDocument,
    ) -> Result<(), TransactionError> {
        if document.kind() != super::AuxiliaryDocumentKind::Playlist {
            return Err(StorageError::UnsupportedAuxiliaryOperation {
                operation: "delete playlist transaction",
                expected: super::AuxiliaryDocumentKind::Playlist,
                actual: document.kind(),
            }
            .into());
        }
        let expected = document.source_revision().cloned().ok_or_else(|| {
            TransactionError::UnversionedDocument {
                path: document.source_path().to_path_buf(),
            }
        })?;
        self.stage_delete(document.source_path(), expected)
    }

    /// Stages a newly constructed platform document. The target must not
    /// exist both when staged and immediately before commit.
    pub fn stage_new_platform(
        &mut self,
        document: &PlatformDocument,
    ) -> Result<(), TransactionError> {
        if document.source_revision().is_some() {
            return Err(TransactionError::VersionedNewDocument {
                path: document.source_path().to_path_buf(),
            });
        }
        let candidate = document.to_xml_bytes()?;
        self.stage_create(document.source_path(), candidate)
    }

    /// Stages deletion of an existing empty platform document. Semantic
    /// emptiness is checked here as a second safety boundary.
    pub fn stage_delete_platform(
        &mut self,
        document: &PlatformDocument,
    ) -> Result<(), TransactionError> {
        if !document.library().games.is_empty()
            || !document.library().additional_applications.is_empty()
            || !document.library().mounts.is_empty()
            || !document.library().alternate_names.is_empty()
            || !document.library().custom_fields.is_empty()
            || !document.library().controller_support.is_empty()
            || !document.library().game_saves.is_empty()
        {
            return Err(TransactionError::PlatformDocumentNotEmpty {
                path: document.source_path().to_path_buf(),
            });
        }
        let expected = document.source_revision().cloned().ok_or_else(|| {
            TransactionError::UnversionedDocument {
                path: document.source_path().to_path_buf(),
            }
        })?;
        self.stage_delete(document.source_path(), expected)
    }

    /// Stages a streamed copy of one regular file into a new path under the
    /// library root. The source may live outside the library. Its bytes are
    /// copied into the transaction's durable staging file during commit, so
    /// large ROM and disc images are never buffered in memory.
    pub fn stage_file_copy(
        &mut self,
        source: impl AsRef<Path>,
        target: impl AsRef<Path>,
    ) -> Result<(), TransactionError> {
        let supplied_source = source.as_ref();
        let metadata =
            fs::symlink_metadata(supplied_source).map_err(|source| TransactionError::Io {
                path: supplied_source.to_path_buf(),
                source,
            })?;
        if !metadata.file_type().is_file() {
            return Err(TransactionError::SourceNotFile {
                path: supplied_source.to_path_buf(),
            });
        }
        let source = fs::canonicalize(supplied_source).map_err(|source| TransactionError::Io {
            path: supplied_source.to_path_buf(),
            source,
        })?;
        let target = self.checked_new_target(target.as_ref())?;
        self.push_change(PendingChange {
            target,
            operation: TransactionOperation::Create,
            candidate: Some(PendingCandidate::SourceFile {
                path: source,
                expected: None,
            }),
            expected: None,
        })
    }

    /// Stages a streamed file creation while requiring the copied bytes to
    /// match a revision already inspected by the caller. This lets metadata
    /// such as hashes and sizes be derived from one exact source snapshot
    /// without trusting a path that may change before the transaction is
    /// durably prepared.
    pub fn stage_file_copy_with_revision(
        &mut self,
        source: impl AsRef<Path>,
        target: impl AsRef<Path>,
        expected: FileRevision,
    ) -> Result<(), TransactionError> {
        let supplied_source = source.as_ref();
        let metadata =
            fs::symlink_metadata(supplied_source).map_err(|source| TransactionError::Io {
                path: supplied_source.to_path_buf(),
                source,
            })?;
        if !metadata.file_type().is_file() {
            return Err(TransactionError::SourceNotFile {
                path: supplied_source.to_path_buf(),
            });
        }
        let source = fs::canonicalize(supplied_source).map_err(|source| TransactionError::Io {
            path: supplied_source.to_path_buf(),
            source,
        })?;
        let target = self.checked_new_target(target.as_ref())?;
        self.push_change(PendingChange {
            target,
            operation: TransactionOperation::Create,
            candidate: Some(PendingCandidate::SourceFile {
                path: source,
                expected: Some(expected),
            }),
            expected: None,
        })
    }

    pub fn len(&self) -> usize {
        self.changes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    pub fn commit(self) -> Result<TransactionReport, TransactionError> {
        self.commit_with_apply(|_index, entry| apply_prepared_change(entry), true)
    }

    fn stage_replace(
        &mut self,
        target: &Path,
        candidate: Vec<u8>,
        expected: FileRevision,
    ) -> Result<(), TransactionError> {
        let target = self.checked_target(target)?;
        self.push_change(PendingChange {
            target,
            operation: TransactionOperation::Replace,
            candidate: Some(PendingCandidate::Bytes(candidate)),
            expected: Some(expected),
        })
    }

    fn stage_create(&mut self, target: &Path, candidate: Vec<u8>) -> Result<(), TransactionError> {
        let target = self.checked_new_target(target)?;
        self.push_change(PendingChange {
            target,
            operation: TransactionOperation::Create,
            candidate: Some(PendingCandidate::Bytes(candidate)),
            expected: None,
        })
    }

    fn stage_delete(
        &mut self,
        target: &Path,
        expected: FileRevision,
    ) -> Result<(), TransactionError> {
        let target = self.checked_target(target)?;
        self.push_change(PendingChange {
            target,
            operation: TransactionOperation::Delete,
            candidate: None,
            expected: Some(expected),
        })
    }

    fn push_change(&mut self, change: PendingChange) -> Result<(), TransactionError> {
        if self
            .changes
            .iter()
            .any(|existing| existing.target == change.target)
        {
            return Err(TransactionError::DuplicateTarget {
                path: change.target.clone(),
            });
        }
        self.changes.push(change);
        Ok(())
    }

    fn checked_target(&self, target: &Path) -> Result<PathBuf, TransactionError> {
        let metadata = fs::symlink_metadata(target).map_err(|source| TransactionError::Io {
            path: target.to_path_buf(),
            source,
        })?;
        if !metadata.file_type().is_file() {
            return Err(TransactionError::TargetNotFile {
                path: target.to_path_buf(),
            });
        }
        let target = fs::canonicalize(target).map_err(|source| TransactionError::Io {
            path: target.to_path_buf(),
            source,
        })?;
        if !target.starts_with(&self.root) {
            return Err(TransactionError::TargetOutsideRoot {
                root: self.root.clone(),
                path: target,
            });
        }
        Ok(target)
    }

    fn checked_new_target(&self, target: &Path) -> Result<PathBuf, TransactionError> {
        match fs::symlink_metadata(target) {
            Ok(_) => {
                return Err(TransactionError::NewTargetAlreadyExists {
                    path: target.to_path_buf(),
                })
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(source) => {
                return Err(TransactionError::Io {
                    path: target.to_path_buf(),
                    source,
                })
            }
        }
        let file_name = target
            .file_name()
            .ok_or_else(|| TransactionError::UnsafeNewTarget {
                path: target.to_path_buf(),
            })?;
        let supplied_parent = target
            .parent()
            .ok_or_else(|| TransactionError::UnsafeNewTarget {
                path: target.to_path_buf(),
            })?;
        let parent = fs::canonicalize(supplied_parent).map_err(|source| TransactionError::Io {
            path: supplied_parent.to_path_buf(),
            source,
        })?;
        if !parent.starts_with(&self.root) {
            return Err(TransactionError::TargetOutsideRoot {
                root: self.root.clone(),
                path: parent.join(file_name),
            });
        }
        Ok(parent.join(file_name))
    }

    #[cfg(test)]
    fn commit_with_replace<F>(
        self,
        mut replace: F,
        recover_on_failure: bool,
    ) -> Result<TransactionReport, TransactionError>
    where
        F: FnMut(usize, &Path, &Path) -> Result<(), std::io::Error>,
    {
        self.commit_with_apply(
            |index, entry| match entry.operation {
                TransactionOperation::Replace | TransactionOperation::Create => replace(
                    index,
                    entry
                        .staged
                        .as_deref()
                        .expect("replace/create transaction entry has staged data"),
                    &entry.target,
                ),
                TransactionOperation::Delete => apply_prepared_change(entry),
            },
            recover_on_failure,
        )
    }

    fn commit_with_apply<F>(
        self,
        mut apply: F,
        recover_on_failure: bool,
    ) -> Result<TransactionReport, TransactionError>
    where
        F: FnMut(usize, &PreparedChange) -> Result<(), std::io::Error>,
    {
        if self.changes.is_empty() {
            return Err(TransactionError::Empty);
        }
        let _lock = TransactionLock::acquire(&self.root)?;
        let pending_manifests = pending_manifest_paths(&self.root)?;
        if !pending_manifests.is_empty() {
            return Err(TransactionError::PendingRecovery {
                root: self.root,
                manifests: pending_manifests,
            });
        }
        verify_pending_changes(&self.changes)?;

        let mut prepared = Vec::with_capacity(self.changes.len());
        for change in &self.changes {
            match prepare_change(change) {
                Ok(entry) => prepared.push(entry),
                Err(error) => {
                    cleanup_prepared(&prepared);
                    return Err(error);
                }
            }
        }

        if let Err(error) = verify_pending_changes(&self.changes) {
            cleanup_prepared(&prepared);
            return Err(error);
        }

        let manifest = TransactionManifest {
            version: MANIFEST_VERSION,
            entries: prepared,
        };
        let manifest_path = match write_manifest(&self.root, &manifest) {
            Ok(path) => path,
            Err(error) => {
                cleanup_prepared(&manifest.entries);
                return Err(error);
            }
        };

        for (index, entry) in manifest.entries.iter().enumerate() {
            let result = apply(index, entry).and_then(|()| sync_parent_directory(&entry.target));
            if let Err(source) = result {
                let commit_error = source.to_string();
                if !recover_on_failure {
                    return Err(TransactionError::RecoveryRequired {
                        manifest: manifest_path,
                        commit_error,
                        recovery_error: "automatic recovery intentionally skipped".to_string(),
                    });
                }
                return match rollback_manifest(&self.root, &manifest_path, &manifest) {
                    Ok(_) => Err(TransactionError::CommitRolledBack { commit_error }),
                    Err(recovery) => Err(TransactionError::RecoveryRequired {
                        manifest: manifest_path,
                        commit_error,
                        recovery_error: recovery.to_string(),
                    }),
                };
            }
        }

        remove_manifest(&self.root, &manifest_path)?;
        let mut writes = Vec::new();
        let mut created_targets = Vec::new();
        let mut deleted_targets = Vec::new();
        for entry in manifest.entries {
            match entry.operation {
                TransactionOperation::Replace => writes.push(AtomicSaveReport {
                    target: entry.target,
                    backup: entry
                        .backup
                        .expect("replace transaction entry has a backup"),
                }),
                TransactionOperation::Create => created_targets.push(entry.target),
                TransactionOperation::Delete => deleted_targets.push(AtomicSaveReport {
                    target: entry.target,
                    backup: entry.backup.expect("delete transaction entry has a backup"),
                }),
            }
        }
        Ok(TransactionReport {
            manifest: manifest_path,
            writes,
            created_targets,
            deleted_targets,
        })
    }
}

/// Lists durable transaction manifests directly under a library root without
/// changing them. Callers can require an explicit user recovery decision
/// before attempting another write.
pub fn pending_transaction_manifests(
    root: impl AsRef<Path>,
) -> Result<Vec<PathBuf>, TransactionError> {
    let supplied = root.as_ref();
    let root = fs::canonicalize(supplied).map_err(|source| TransactionError::Io {
        path: supplied.to_path_buf(),
        source,
    })?;
    if !root.is_dir() {
        return Err(TransactionError::RootNotDirectory { path: root });
    }
    pending_manifest_paths(&root)
}

/// Rolls back every durable transaction manifest found directly under `root`.
/// A target whose contents match neither the recorded original nor candidate
/// revision is left untouched and reported as a divergence.
pub fn recover_pending_transactions(
    root: impl AsRef<Path>,
) -> Result<Vec<RecoveryReport>, TransactionError> {
    let supplied = root.as_ref();
    let root = fs::canonicalize(supplied).map_err(|source| TransactionError::Io {
        path: supplied.to_path_buf(),
        source,
    })?;
    let _lock = TransactionLock::acquire(&root)?;
    let manifests = pending_manifest_paths(&root)?;

    manifests
        .into_iter()
        .map(|path| {
            let manifest = read_manifest(&root, &path)?;
            rollback_manifest(&root, &path, &manifest)
        })
        .collect()
}

fn pending_manifest_paths(root: &Path) -> Result<Vec<PathBuf>, TransactionError> {
    let mut manifests = fs::read_dir(root)
        .map_err(|source| TransactionError::Io {
            path: root.to_path_buf(),
            source,
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| is_manifest_path(path))
        .collect::<Vec<_>>();
    manifests.sort();
    Ok(manifests)
}

fn verify_pending_changes(changes: &[PendingChange]) -> Result<(), TransactionError> {
    for change in changes {
        match change.operation {
            TransactionOperation::Replace | TransactionOperation::Delete => {
                let expected = change
                    .expected
                    .as_ref()
                    .expect("replace/delete pending change has an expected revision");
                let actual = FileRevision::read(&change.target)?;
                if actual != *expected {
                    return Err(TransactionError::Conflict {
                        path: change.target.clone(),
                        expected: expected.clone(),
                        actual,
                    });
                }
            }
            TransactionOperation::Create => {
                if fs::symlink_metadata(&change.target).is_ok() {
                    return Err(TransactionError::NewTargetAlreadyExists {
                        path: change.target.clone(),
                    });
                }
            }
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TransactionManifest {
    version: u32,
    entries: Vec<PreparedChange>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PreparedChange {
    #[serde(default)]
    operation: TransactionOperation,
    target: PathBuf,
    backup: Option<PathBuf>,
    staged: Option<PathBuf>,
    original_revision: Option<FileRevision>,
    candidate_revision: Option<FileRevision>,
}

fn prepare_change(change: &PendingChange) -> Result<PreparedChange, TransactionError> {
    match change.operation {
        TransactionOperation::Replace => {
            let metadata = regular_target_metadata(&change.target)?;
            let candidate = change
                .candidate
                .as_ref()
                .expect("replace pending change has candidate data");
            let (staged, candidate_revision) =
                prepare_staged_candidate(&change.target, candidate, Some(metadata.permissions()))?;
            let backup = match prepare_backup(&change.target, &metadata) {
                Ok(backup) => backup,
                Err(error) => {
                    let _ = fs::remove_file(&staged);
                    return Err(error);
                }
            };
            Ok(PreparedChange {
                operation: change.operation,
                target: change.target.clone(),
                backup: Some(backup),
                staged: Some(staged),
                original_revision: change.expected.clone(),
                candidate_revision: Some(candidate_revision),
            })
        }
        TransactionOperation::Create => {
            let candidate = change
                .candidate
                .as_ref()
                .expect("create pending change has candidate data");
            let (staged, candidate_revision) =
                prepare_staged_candidate(&change.target, candidate, None)?;
            Ok(PreparedChange {
                operation: change.operation,
                target: change.target.clone(),
                backup: None,
                staged: Some(staged),
                original_revision: None,
                candidate_revision: Some(candidate_revision),
            })
        }
        TransactionOperation::Delete => {
            let metadata = regular_target_metadata(&change.target)?;
            let backup = prepare_backup(&change.target, &metadata)?;
            Ok(PreparedChange {
                operation: change.operation,
                target: change.target.clone(),
                backup: Some(backup),
                staged: None,
                original_revision: change.expected.clone(),
                candidate_revision: None,
            })
        }
    }
}

fn regular_target_metadata(path: &Path) -> Result<fs::Metadata, TransactionError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| TransactionError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if !metadata.file_type().is_file() {
        return Err(TransactionError::TargetNotFile {
            path: path.to_path_buf(),
        });
    }
    Ok(metadata)
}

fn prepare_staged_candidate(
    target: &Path,
    candidate: &PendingCandidate,
    permissions: Option<fs::Permissions>,
) -> Result<(PathBuf, FileRevision), TransactionError> {
    let (staged_path, mut staged) = create_unique_sibling(target, "transaction-stage", true)?;
    let result = (|| {
        let revision = match candidate {
            PendingCandidate::Bytes(bytes) => {
                staged.write_all(bytes)?;
                FileRevision::from_bytes(bytes)
            }
            PendingCandidate::SourceFile { path, .. } => {
                let mut source_file = fs::File::open(path)?;
                let mut digest = Sha256::new();
                let mut byte_len = 0_u64;
                let mut buffer = vec![0_u8; 1024 * 1024];
                loop {
                    let read = source_file.read(&mut buffer)?;
                    if read == 0 {
                        break;
                    }
                    staged.write_all(&buffer[..read])?;
                    digest.update(&buffer[..read]);
                    byte_len = byte_len.saturating_add(read.try_into().unwrap_or(u64::MAX));
                }
                let digest = digest.finalize();
                let mut sha256 = String::with_capacity(digest.len() * 2);
                for byte in digest {
                    write!(&mut sha256, "{byte:02x}").expect("writing to a String cannot fail");
                }
                FileRevision { byte_len, sha256 }
            }
        };
        staged.flush()?;
        if let Some(permissions) = permissions {
            staged.set_permissions(permissions)?;
        }
        staged.sync_all()?;
        Ok::<_, std::io::Error>(revision)
    })();
    let revision = match result {
        Ok(revision) => revision,
        Err(source) => {
            let _ = fs::remove_file(&staged_path);
            return Err(TransactionError::Io {
                path: staged_path,
                source,
            });
        }
    };
    if let PendingCandidate::SourceFile {
        path,
        expected: Some(expected),
    } = candidate
    {
        if revision != *expected {
            let _ = fs::remove_file(&staged_path);
            return Err(TransactionError::SourceConflict {
                path: path.clone(),
                expected: expected.clone(),
                actual: revision,
            });
        }
    }
    drop(staged);
    Ok((staged_path, revision))
}

fn prepare_backup(target: &Path, metadata: &fs::Metadata) -> Result<PathBuf, TransactionError> {
    let (backup_path, mut backup) = create_unique_sibling(target, "transaction-backup", false)?;
    let result = fs::File::open(target)
        .and_then(|mut original| std::io::copy(&mut original, &mut backup))
        .and_then(|_| backup.flush())
        .and_then(|()| backup.set_permissions(metadata.permissions()))
        .and_then(|()| backup.sync_all());
    if let Err(source) = result {
        let _ = fs::remove_file(&backup_path);
        return Err(TransactionError::Io {
            path: backup_path,
            source,
        });
    }
    drop(backup);
    Ok(backup_path)
}

fn apply_prepared_change(entry: &PreparedChange) -> Result<(), std::io::Error> {
    match entry.operation {
        TransactionOperation::Replace | TransactionOperation::Create => replace_file(
            entry
                .staged
                .as_deref()
                .expect("replace/create transaction entry has staged data"),
            &entry.target,
        ),
        TransactionOperation::Delete => fs::remove_file(&entry.target),
    }
}

fn cleanup_prepared(entries: &[PreparedChange]) {
    for entry in entries {
        if let Some(staged) = &entry.staged {
            let _ = fs::remove_file(staged);
        }
        if let Some(backup) = &entry.backup {
            let _ = fs::remove_file(backup);
        }
    }
}

fn write_manifest(
    root: &Path,
    manifest: &TransactionManifest,
) -> Result<PathBuf, TransactionError> {
    let bytes = serde_json::to_vec_pretty(manifest)?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    for attempt in 0..1000 {
        let path = root.join(format!(
            "{MANIFEST_PREFIX}{}-{timestamp}-{attempt}{MANIFEST_SUFFIX}",
            std::process::id()
        ));
        let mut file = match fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
        {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(source) => return Err(TransactionError::Io { path, source }),
        };
        file.write_all(&bytes)
            .and_then(|()| file.flush())
            .and_then(|()| file.sync_all())
            .map_err(|source| TransactionError::Io {
                path: path.clone(),
                source,
            })?;
        sync_parent_directory(&path).map_err(|source| TransactionError::Io {
            path: root.to_path_buf(),
            source,
        })?;
        return Ok(path);
    }
    Err(TransactionError::ManifestNameExhausted {
        root: root.to_path_buf(),
    })
}

fn read_manifest(root: &Path, path: &Path) -> Result<TransactionManifest, TransactionError> {
    validate_regular_artifact(root, path, path, true)?;
    let mut bytes = Vec::new();
    fs::File::open(path)
        .and_then(|mut file| file.read_to_end(&mut bytes))
        .map_err(|source| TransactionError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    let manifest: TransactionManifest =
        serde_json::from_slice(&bytes).map_err(|source| TransactionError::InvalidManifest {
            path: path.to_path_buf(),
            source,
        })?;
    if ![LEGACY_MANIFEST_VERSION, MANIFEST_VERSION].contains(&manifest.version) {
        return Err(TransactionError::UnsupportedManifestVersion {
            path: path.to_path_buf(),
            version: manifest.version,
        });
    }
    for entry in &manifest.entries {
        let target_must_exist = entry.operation == TransactionOperation::Replace;
        validate_regular_artifact(root, path, &entry.target, target_must_exist)?;
        if let Some(backup) = &entry.backup {
            validate_regular_artifact(root, path, backup, true)?;
        } else if entry.operation != TransactionOperation::Create {
            return Err(TransactionError::IncompleteManifestEntry {
                manifest: path.to_path_buf(),
                target: entry.target.clone(),
            });
        }
        if let Some(staged) = &entry.staged {
            validate_regular_artifact(root, path, staged, false)?;
        } else if entry.operation != TransactionOperation::Delete {
            return Err(TransactionError::IncompleteManifestEntry {
                manifest: path.to_path_buf(),
                target: entry.target.clone(),
            });
        }
        match entry.operation {
            TransactionOperation::Replace
                if entry.original_revision.is_none() || entry.candidate_revision.is_none() =>
            {
                return Err(TransactionError::IncompleteManifestEntry {
                    manifest: path.to_path_buf(),
                    target: entry.target.clone(),
                });
            }
            TransactionOperation::Create if entry.candidate_revision.is_none() => {
                return Err(TransactionError::IncompleteManifestEntry {
                    manifest: path.to_path_buf(),
                    target: entry.target.clone(),
                });
            }
            TransactionOperation::Delete if entry.original_revision.is_none() => {
                return Err(TransactionError::IncompleteManifestEntry {
                    manifest: path.to_path_buf(),
                    target: entry.target.clone(),
                });
            }
            _ => {}
        }
    }
    Ok(manifest)
}

fn validate_regular_artifact(
    root: &Path,
    manifest: &Path,
    artifact: &Path,
    must_exist: bool,
) -> Result<(), TransactionError> {
    let is_normalized_absolute = artifact.is_absolute()
        && !artifact
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir));
    let parent_in_root = artifact
        .parent()
        .filter(|_| is_normalized_absolute)
        .and_then(|parent| fs::canonicalize(parent).ok())
        .is_some_and(|parent| parent.starts_with(root));
    if !parent_in_root {
        return Err(TransactionError::UnsafeManifestPath {
            manifest: manifest.to_path_buf(),
            path: artifact.to_path_buf(),
        });
    }

    match fs::symlink_metadata(artifact) {
        Ok(metadata) if metadata.file_type().is_file() => Ok(()),
        Err(error) if !must_exist && error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Ok(_) | Err(_) => Err(TransactionError::UnsafeManifestPath {
            manifest: manifest.to_path_buf(),
            path: artifact.to_path_buf(),
        }),
    }
}

fn rollback_manifest(
    root: &Path,
    manifest_path: &Path,
    manifest: &TransactionManifest,
) -> Result<RecoveryReport, TransactionError> {
    let mut restored_targets = Vec::new();
    for entry in &manifest.entries {
        let actual = optional_file_revision(&entry.target)?;
        if actual == entry.original_revision {
            continue;
        }
        if actual != entry.candidate_revision {
            return Err(TransactionError::RecoveryDiverged {
                path: entry.target.clone(),
                original: entry.original_revision.clone(),
                candidate: entry.candidate_revision.clone(),
                actual,
            });
        }
        match entry.operation {
            TransactionOperation::Replace | TransactionOperation::Delete => {
                restore_entry(entry)?;
            }
            TransactionOperation::Create => {
                fs::remove_file(&entry.target).map_err(|source| TransactionError::Io {
                    path: entry.target.clone(),
                    source,
                })?;
                sync_parent_directory(&entry.target).map_err(|source| TransactionError::Io {
                    path: entry.target.clone(),
                    source,
                })?;
            }
        }
        restored_targets.push(entry.target.clone());
    }

    for entry in &manifest.entries {
        if let Some(staged) = &entry.staged {
            if staged.is_file() {
                fs::remove_file(staged).map_err(|source| TransactionError::Io {
                    path: staged.clone(),
                    source,
                })?;
            }
        }
    }
    remove_manifest(root, manifest_path)?;
    Ok(RecoveryReport {
        manifest: manifest_path.to_path_buf(),
        restored_targets,
    })
}

fn optional_file_revision(path: &Path) -> Result<Option<FileRevision>, TransactionError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => Ok(Some(FileRevision::read(path)?)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Ok(_) => Err(TransactionError::TargetNotFile {
            path: path.to_path_buf(),
        }),
        Err(source) => Err(TransactionError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn restore_entry(entry: &PreparedChange) -> Result<(), TransactionError> {
    let backup = entry
        .backup
        .as_ref()
        .expect("replace/delete transaction entry has a backup");
    let original_revision = entry
        .original_revision
        .as_ref()
        .expect("replace/delete transaction entry has an original revision");
    let backup_bytes = fs::read(backup).map_err(|source| TransactionError::Io {
        path: backup.clone(),
        source,
    })?;
    let backup_revision = FileRevision::from_bytes(&backup_bytes);
    if backup_revision != *original_revision {
        return Err(TransactionError::BackupMismatch {
            path: backup.clone(),
            expected: original_revision.clone(),
            actual: backup_revision,
        });
    }
    let permissions = fs::metadata(backup)
        .map_err(|source| TransactionError::Io {
            path: backup.clone(),
            source,
        })?
        .permissions();
    let (temporary_path, mut temporary) =
        create_unique_sibling(&entry.target, "transaction-restore", true)?;
    if let Err(source) = temporary
        .write_all(&backup_bytes)
        .and_then(|()| temporary.flush())
        .and_then(|()| temporary.set_permissions(permissions))
        .and_then(|()| temporary.sync_all())
    {
        let _ = fs::remove_file(&temporary_path);
        return Err(TransactionError::Io {
            path: temporary_path,
            source,
        });
    }
    drop(temporary);
    replace_file(&temporary_path, &entry.target).map_err(|source| TransactionError::Io {
        path: entry.target.clone(),
        source,
    })?;
    sync_parent_directory(&entry.target).map_err(|source| TransactionError::Io {
        path: entry.target.clone(),
        source,
    })?;
    Ok(())
}

fn remove_manifest(root: &Path, manifest: &Path) -> Result<(), TransactionError> {
    fs::remove_file(manifest).map_err(|source| TransactionError::Io {
        path: manifest.to_path_buf(),
        source,
    })?;
    sync_parent_directory(manifest).map_err(|source| TransactionError::Io {
        path: root.to_path_buf(),
        source,
    })
}

fn is_manifest_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with(MANIFEST_PREFIX) && name.ends_with(MANIFEST_SUFFIX))
}

struct TransactionLock {
    file: fs::File,
}

impl TransactionLock {
    fn acquire(root: &Path) -> Result<Self, TransactionError> {
        let path = root.join(LOCK_FILE_NAME);
        let file = fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|source| TransactionError::Io {
                path: path.clone(),
                source,
            })?;
        let lock_target = fs::canonicalize(&path).map_err(|source| TransactionError::Io {
            path: path.clone(),
            source,
        })?;
        let metadata = fs::symlink_metadata(&path).map_err(|source| TransactionError::Io {
            path: path.clone(),
            source,
        })?;
        if !metadata.file_type().is_file() || !lock_target.starts_with(root) {
            return Err(TransactionError::UnsafeLockFile { path });
        }
        file.lock_exclusive()
            .map_err(|source| TransactionError::Io {
                path: root.join(LOCK_FILE_NAME),
                source,
            })?;
        Ok(Self { file })
    }
}

impl Drop for TransactionLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}

#[derive(Debug, Error)]
pub enum TransactionError {
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error("transaction root is not a directory: {path}")]
    RootNotDirectory { path: PathBuf },
    #[error("transaction target is not a regular file: {path}")]
    TargetNotFile { path: PathBuf },
    #[error("transaction copy source is not a regular file: {path}")]
    SourceNotFile { path: PathBuf },
    #[error("transaction target {path} is outside root {root}")]
    TargetOutsideRoot { root: PathBuf, path: PathBuf },
    #[error("document {path} was not loaded from a file and has no source revision")]
    UnversionedDocument { path: PathBuf },
    #[error("document {path} was loaded from a file and cannot be staged as new")]
    VersionedNewDocument { path: PathBuf },
    #[error("new transaction target already exists: {path}")]
    NewTargetAlreadyExists { path: PathBuf },
    #[error("new transaction target is not a safe file path: {path}")]
    UnsafeNewTarget { path: PathBuf },
    #[error("platform document is not empty and cannot be deleted: {path}")]
    PlatformDocumentNotEmpty { path: PathBuf },
    #[error("document {path} was staged more than once")]
    DuplicateTarget { path: PathBuf },
    #[error("cannot commit an empty library transaction")]
    Empty,
    #[error("pending transaction recovery is required under {root}: {manifests:?}")]
    PendingRecovery {
        root: PathBuf,
        manifests: Vec<PathBuf>,
    },
    #[error("document changed since it was loaded: {path}")]
    Conflict {
        path: PathBuf,
        expected: FileRevision,
        actual: FileRevision,
    },
    #[error("copy source changed after it was inspected: {path}")]
    SourceConflict {
        path: PathBuf,
        expected: FileRevision,
        actual: FileRevision,
    },
    #[error("transaction commit failed and was rolled back: {commit_error}")]
    CommitRolledBack { commit_error: String },
    #[error(
        "transaction commit failed and recovery is required from {manifest}: {commit_error}; recovery error: {recovery_error}"
    )]
    RecoveryRequired {
        manifest: PathBuf,
        commit_error: String,
        recovery_error: String,
    },
    #[error("target diverged during transaction recovery: {path}")]
    RecoveryDiverged {
        path: PathBuf,
        original: Option<FileRevision>,
        candidate: Option<FileRevision>,
        actual: Option<FileRevision>,
    },
    #[error("transaction backup revision does not match its manifest: {path}")]
    BackupMismatch {
        path: PathBuf,
        expected: FileRevision,
        actual: FileRevision,
    },
    #[error("unsafe path {path} in transaction manifest {manifest}")]
    UnsafeManifestPath { manifest: PathBuf, path: PathBuf },
    #[error("incomplete transaction entry for {target} in manifest {manifest}")]
    IncompleteManifestEntry { manifest: PathBuf, target: PathBuf },
    #[error("transaction lock path is not a regular file within the library root: {path}")]
    UnsafeLockFile { path: PathBuf },
    #[error("unsupported transaction manifest version {version} in {path}")]
    UnsupportedManifestVersion { path: PathBuf, version: u32 },
    #[error("invalid transaction manifest {path}: {source}")]
    InvalidManifest {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to serialize transaction manifest: {0}")]
    SerializeManifest(#[from] serde_json::Error),
    #[error("could not allocate a transaction manifest name under {root}")]
    ManifestNameExhausted { root: PathBuf },
    #[error("I/O failure at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AuxiliaryDocumentKind, PlatformDocument};
    use std::collections::{BTreeMap, BTreeSet};
    use xmltree::{Element, XMLNode};

    const PLATFORM: &str =
        include_str!("../../../fixtures/launchbox/Data/Platforms/Fixture Console.xml");
    const SETTINGS: &str = include_str!("../../../fixtures/launchbox/Data/Settings.xml");

    fn fixture_tree() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let directory = tempfile::tempdir().expect("temporary directory");
        let platform = directory.path().join("Data/Platforms/Fixture Console.xml");
        let settings = directory.path().join("Data/Settings.xml");
        fs::create_dir_all(platform.parent().expect("platform parent"))
            .expect("create platform directory");
        fs::write(&platform, PLATFORM).expect("write platform");
        fs::write(&settings, SETTINGS).expect("write settings");
        (directory, platform, settings)
    }

    fn edited_documents(
        platform_path: &Path,
        settings_path: &Path,
    ) -> (PlatformDocument, AuxiliaryDocument) {
        let mut platform = PlatformDocument::load(platform_path).expect("load platform");
        platform
            .set_game_title("fixture-adventure", "Transactional Adventure")
            .expect("edit game");
        let mut settings = AuxiliaryDocument::load(settings_path).expect("load settings");
        settings
            .set_single_record_field("Settings", "Theme", "Transactional Theme")
            .expect("edit settings");
        (platform, settings)
    }

    #[derive(Debug, Serialize)]
    struct GoldenChange {
        document: String,
        path: String,
        before: Option<String>,
        after: Option<String>,
    }

    fn collect_leaf_values(element: &Element, path: &str, values: &mut BTreeMap<String, String>) {
        for (name, value) in &element.attributes {
            values.insert(format!("{path}/@{name}"), value.clone());
        }

        let children = element
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .collect::<Vec<_>>();
        if children.is_empty() {
            values.insert(
                path.to_string(),
                element
                    .get_text()
                    .map_or_else(String::new, |value| value.into_owned()),
            );
            return;
        }

        let mut indices = BTreeMap::<&str, usize>::new();
        for child in children {
            let index = indices.entry(child.name.as_str()).or_default();
            let child_path = format!("{path}/{}[{index}]", child.name);
            *index += 1;
            collect_leaf_values(child, &child_path, values);
        }
    }

    fn semantic_changes(document: &str, before: &Element, after: &Element) -> Vec<GoldenChange> {
        let mut before_values = BTreeMap::new();
        let mut after_values = BTreeMap::new();
        collect_leaf_values(before, &format!("/{}", before.name), &mut before_values);
        collect_leaf_values(after, &format!("/{}", after.name), &mut after_values);
        let paths = before_values
            .keys()
            .chain(after_values.keys())
            .cloned()
            .collect::<BTreeSet<_>>();

        paths
            .into_iter()
            .filter_map(|path| {
                let before = before_values.get(&path).cloned();
                let after = after_values.get(&path).cloned();
                (before != after).then(|| GoldenChange {
                    document: document.to_string(),
                    path,
                    before,
                    after,
                })
            })
            .collect()
    }

    #[test]
    fn commits_two_documents_with_exact_backups() {
        let (directory, platform_path, settings_path) = fixture_tree();
        let original_platform = fs::read(&platform_path).expect("read platform");
        let original_settings = fs::read(&settings_path).expect("read settings");
        let (platform, settings) = edited_documents(&platform_path, &settings_path);

        let mut transaction = LibraryTransaction::new(directory.path()).expect("transaction");
        transaction
            .stage_platform(&platform)
            .expect("stage platform");
        transaction
            .stage_auxiliary(&settings)
            .expect("stage settings");
        let report = transaction.commit().expect("commit transaction");

        assert_eq!(report.writes.len(), 2);
        assert!(!report.manifest.exists());
        assert_eq!(
            fs::read(&report.writes[0].backup).unwrap(),
            original_platform
        );
        assert_eq!(
            fs::read(&report.writes[1].backup).unwrap(),
            original_settings
        );
        assert_eq!(
            PlatformDocument::load(&platform_path)
                .unwrap()
                .library()
                .games[0]
                .title,
            "Transactional Adventure"
        );
        let settings = AuxiliaryDocument::load_as(AuxiliaryDocumentKind::Settings, &settings_path)
            .expect("reload settings");
        assert!(settings
            .to_xml_bytes()
            .expect("serialize settings")
            .windows("Transactional Theme".len())
            .any(|window| window == b"Transactional Theme"));
    }

    #[test]
    fn committed_semantic_diff_matches_the_two_document_golden_file() {
        let (directory, platform_path, settings_path) = fixture_tree();
        let before_platform = PlatformDocument::load(&platform_path).unwrap();
        let before_settings = AuxiliaryDocument::load(&settings_path).unwrap();
        let (platform, settings) = edited_documents(&platform_path, &settings_path);
        let mut transaction = LibraryTransaction::new(directory.path()).unwrap();
        transaction.stage_platform(&platform).unwrap();
        transaction.stage_auxiliary(&settings).unwrap();
        transaction.commit().unwrap();

        let after_platform = PlatformDocument::load(&platform_path).unwrap();
        let after_settings = AuxiliaryDocument::load(&settings_path).unwrap();
        let mut changes = semantic_changes(
            "Data/Platforms/Fixture Console.xml",
            &before_platform.root,
            &after_platform.root,
        );
        changes.extend(semantic_changes(
            "Data/Settings.xml",
            &before_settings.root,
            &after_settings.root,
        ));

        let actual = serde_json::to_value(changes).unwrap();
        let expected: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/transactions/two-document-edit.expected.json"
        ))
        .unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_unversioned_and_out_of_root_documents() {
        let (directory, platform_path, _) = fixture_tree();
        let unversioned =
            PlatformDocument::from_reader(&platform_path, PLATFORM.as_bytes()).unwrap();
        let mut transaction = LibraryTransaction::new(directory.path()).unwrap();
        assert!(matches!(
            transaction.stage_platform(&unversioned),
            Err(TransactionError::UnversionedDocument { .. })
        ));

        let outside_directory = tempfile::tempdir().unwrap();
        let outside_path = outside_directory.path().join("Outside.xml");
        fs::write(&outside_path, PLATFORM).unwrap();
        let outside = PlatformDocument::load(&outside_path).unwrap();
        assert!(matches!(
            transaction.stage_platform(&outside),
            Err(TransactionError::TargetOutsideRoot { .. })
        ));
    }

    #[test]
    fn pending_manifest_blocks_a_new_commit_until_recovery() {
        let (directory, platform_path, _) = fixture_tree();
        let original = fs::read(&platform_path).unwrap();
        let manifest = directory.path().join(".lbport-transaction-pending.json");
        fs::write(&manifest, "{}").unwrap();

        assert_eq!(
            pending_transaction_manifests(directory.path()).unwrap(),
            vec![manifest.clone()]
        );

        let mut platform = PlatformDocument::load(&platform_path).unwrap();
        platform
            .set_game_state("fixture-adventure", false, true, 2)
            .unwrap();
        let mut transaction = LibraryTransaction::new(directory.path()).unwrap();
        transaction.stage_platform(&platform).unwrap();
        assert!(matches!(
            transaction.commit(),
            Err(TransactionError::PendingRecovery { manifests, .. })
                if manifests == vec![manifest]
        ));
        assert_eq!(fs::read(&platform_path).unwrap(), original);
    }

    #[test]
    fn recovery_rejects_a_manifest_path_that_lexically_escapes_the_root() {
        let (directory, _, _) = fixture_tree();
        let manifest_path = directory.path().join(".lbport-transaction-malicious.json");
        let escaping_target = directory.path().join("subdirectory/../../outside.xml");
        let manifest = serde_json::json!({
            "version": MANIFEST_VERSION,
            "entries": [{
                "target": escaping_target,
                "backup": directory.path().join("backup.xml"),
                "staged": directory.path().join("staged.xml"),
                "original_revision": { "byte_len": 0, "sha256": "00" },
                "candidate_revision": { "byte_len": 0, "sha256": "11" }
            }]
        });
        fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();

        assert!(matches!(
            recover_pending_transactions(directory.path()),
            Err(TransactionError::UnsafeManifestPath { manifest, path })
                if manifest == manifest_path && path == escaping_target
        ));
    }

    #[test]
    fn detects_external_change_before_creating_transaction_artifacts() {
        let (directory, platform_path, settings_path) = fixture_tree();
        let (platform, settings) = edited_documents(&platform_path, &settings_path);
        fs::write(
            &settings_path,
            SETTINGS.replace("Fixture Theme", "External Theme"),
        )
        .expect("external edit");

        let mut transaction = LibraryTransaction::new(directory.path()).expect("transaction");
        transaction.stage_platform(&platform).unwrap();
        transaction.stage_auxiliary(&settings).unwrap();
        assert!(matches!(
            transaction.commit(),
            Err(TransactionError::Conflict { path, .. }) if path == fs::canonicalize(&settings_path).unwrap()
        ));
        assert_eq!(
            fs::read_dir(directory.path())
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| is_manifest_path(&entry.path()))
                .count(),
            0
        );
    }

    #[test]
    fn injected_second_replace_failure_rolls_back_first_document() {
        let (directory, platform_path, settings_path) = fixture_tree();
        let original_platform = fs::read(&platform_path).unwrap();
        let original_settings = fs::read(&settings_path).unwrap();
        let (platform, settings) = edited_documents(&platform_path, &settings_path);
        let mut transaction = LibraryTransaction::new(directory.path()).unwrap();
        transaction.stage_platform(&platform).unwrap();
        transaction.stage_auxiliary(&settings).unwrap();

        assert!(matches!(
            transaction.commit_with_replace(
                |index, staged, target| {
                    if index == 1 {
                        Err(std::io::Error::other("injected second replace failure"))
                    } else {
                        replace_file(staged, target)
                    }
                },
                true,
            ),
            Err(TransactionError::CommitRolledBack { .. })
        ));
        assert_eq!(fs::read(&platform_path).unwrap(), original_platform);
        assert_eq!(fs::read(&settings_path).unwrap(), original_settings);
    }

    #[test]
    fn durable_manifest_recovers_a_simulated_crash_after_first_replace() {
        let (directory, platform_path, settings_path) = fixture_tree();
        let original_platform = fs::read(&platform_path).unwrap();
        let original_settings = fs::read(&settings_path).unwrap();
        let (platform, settings) = edited_documents(&platform_path, &settings_path);
        let mut transaction = LibraryTransaction::new(directory.path()).unwrap();
        transaction.stage_platform(&platform).unwrap();
        transaction.stage_auxiliary(&settings).unwrap();

        let manifest = match transaction.commit_with_replace(
            |index, staged, target| {
                if index == 1 {
                    Err(std::io::Error::other("simulated process death"))
                } else {
                    replace_file(staged, target)
                }
            },
            false,
        ) {
            Err(TransactionError::RecoveryRequired { manifest, .. }) => manifest,
            other => panic!("unexpected result: {other:?}"),
        };
        assert!(manifest.is_file());
        assert_ne!(fs::read(&platform_path).unwrap(), original_platform);
        assert_eq!(fs::read(&settings_path).unwrap(), original_settings);

        let reports = recover_pending_transactions(directory.path()).expect("recover transaction");
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].restored_targets.len(), 1);
        assert_eq!(fs::read(&platform_path).unwrap(), original_platform);
        assert_eq!(fs::read(&settings_path).unwrap(), original_settings);
        assert!(!manifest.exists());
    }

    #[test]
    fn recovery_refuses_to_overwrite_a_diverged_target() {
        let (directory, platform_path, settings_path) = fixture_tree();
        let (platform, settings) = edited_documents(&platform_path, &settings_path);
        let mut transaction = LibraryTransaction::new(directory.path()).unwrap();
        transaction.stage_platform(&platform).unwrap();
        transaction.stage_auxiliary(&settings).unwrap();

        let manifest = match transaction.commit_with_replace(
            |index, staged, target| {
                if index == 1 {
                    Err(std::io::Error::other("simulated process death"))
                } else {
                    replace_file(staged, target)
                }
            },
            false,
        ) {
            Err(TransactionError::RecoveryRequired { manifest, .. }) => manifest,
            other => panic!("unexpected result: {other:?}"),
        };
        let external = PLATFORM.replace("Fixture Racer", "Recovery Conflict Racer");
        fs::write(&platform_path, &external).expect("write recovery conflict");

        assert!(matches!(
            recover_pending_transactions(directory.path()),
            Err(TransactionError::RecoveryDiverged { path, .. })
                if path == fs::canonicalize(&platform_path).unwrap()
        ));
        assert_eq!(
            fs::read_to_string(&platform_path).expect("read diverged target"),
            external
        );
        assert!(manifest.is_file());
    }

    #[test]
    fn commits_replacement_and_new_platform_as_one_transaction() {
        let (directory, _, settings_path) = fixture_tree();
        let original_settings = fs::read(&settings_path).unwrap();
        let mut settings = AuxiliaryDocument::load(&settings_path).unwrap();
        settings
            .set_single_record_field("Settings", "Theme", "Created Platform Theme")
            .unwrap();
        let new_path = directory.path().join("Data/Platforms/New Console.xml");
        let new_platform =
            PlatformDocument::from_reader(&new_path, &b"<?xml version=\"1.0\"?><LaunchBox />"[..])
                .unwrap();

        let mut transaction = LibraryTransaction::new(directory.path()).unwrap();
        transaction.stage_auxiliary(&settings).unwrap();
        transaction.stage_new_platform(&new_platform).unwrap();
        let report = transaction.commit().unwrap();

        assert_eq!(report.writes.len(), 1);
        assert_eq!(report.created_targets, vec![new_path.clone()]);
        assert!(report.deleted_targets.is_empty());
        assert_eq!(
            fs::read(&report.writes[0].backup).unwrap(),
            original_settings
        );
        assert!(new_path.is_file());
        assert!(PlatformDocument::load(&new_path)
            .unwrap()
            .library()
            .games
            .is_empty());
    }

    #[test]
    fn commits_replacement_and_empty_platform_deletion_as_one_transaction() {
        let (directory, _, settings_path) = fixture_tree();
        let empty_path = directory.path().join("Data/Platforms/Empty Console.xml");
        let original_empty =
            b"<?xml version=\"1.0\"?><LaunchBox><Unknown>keep</Unknown></LaunchBox>";
        fs::write(&empty_path, original_empty).unwrap();
        let empty_platform = PlatformDocument::load(&empty_path).unwrap();
        let mut settings = AuxiliaryDocument::load(&settings_path).unwrap();
        settings
            .set_single_record_field("Settings", "Theme", "Deleted Platform Theme")
            .unwrap();

        let mut transaction = LibraryTransaction::new(directory.path()).unwrap();
        transaction.stage_auxiliary(&settings).unwrap();
        transaction.stage_delete_platform(&empty_platform).unwrap();
        let report = transaction.commit().unwrap();

        assert_eq!(report.writes.len(), 1);
        assert!(report.created_targets.is_empty());
        assert_eq!(report.deleted_targets.len(), 1);
        assert_eq!(report.deleted_targets[0].target, empty_path);
        assert_eq!(
            fs::read(&report.deleted_targets[0].backup).unwrap(),
            original_empty
        );
        assert!(!report.deleted_targets[0].target.exists());
    }

    #[test]
    fn failed_create_rolls_back_an_already_replaced_document() {
        let (directory, _, settings_path) = fixture_tree();
        let original_settings = fs::read(&settings_path).unwrap();
        let mut settings = AuxiliaryDocument::load(&settings_path).unwrap();
        settings
            .set_single_record_field("Settings", "Theme", "Rollback Theme")
            .unwrap();
        let new_path = directory.path().join("Data/Platforms/Rollback Console.xml");
        let new_platform = PlatformDocument::from_reader(&new_path, &b"<LaunchBox />"[..]).unwrap();
        let mut transaction = LibraryTransaction::new(directory.path()).unwrap();
        transaction.stage_auxiliary(&settings).unwrap();
        transaction.stage_new_platform(&new_platform).unwrap();

        assert!(matches!(
            transaction.commit_with_apply(
                |index, entry| {
                    if index == 1 {
                        Err(std::io::Error::other("injected create failure"))
                    } else {
                        apply_prepared_change(entry)
                    }
                },
                true,
            ),
            Err(TransactionError::CommitRolledBack { .. })
        ));
        assert_eq!(fs::read(&settings_path).unwrap(), original_settings);
        assert!(!new_path.exists());
    }

    #[test]
    fn crash_recovery_removes_a_created_platform_and_restores_catalog_peer() {
        let (directory, _, settings_path) = fixture_tree();
        let original_settings = fs::read(&settings_path).unwrap();
        let mut settings = AuxiliaryDocument::load(&settings_path).unwrap();
        settings
            .set_single_record_field("Settings", "Theme", "Crash Theme")
            .unwrap();
        let new_path = directory.path().join("Data/Platforms/Crash Console.xml");
        let new_platform = PlatformDocument::from_reader(&new_path, &b"<LaunchBox />"[..]).unwrap();
        let mut transaction = LibraryTransaction::new(directory.path()).unwrap();
        transaction.stage_auxiliary(&settings).unwrap();
        transaction.stage_new_platform(&new_platform).unwrap();

        let manifest = match transaction.commit_with_apply(
            |index, entry| {
                apply_prepared_change(entry).and_then(|()| {
                    if index == 1 {
                        Err(std::io::Error::other("simulated death after create"))
                    } else {
                        Ok(())
                    }
                })
            },
            false,
        ) {
            Err(TransactionError::RecoveryRequired { manifest, .. }) => manifest,
            other => panic!("unexpected result: {other:?}"),
        };
        assert!(new_path.is_file());
        assert_ne!(fs::read(&settings_path).unwrap(), original_settings);

        let reports = recover_pending_transactions(directory.path()).unwrap();
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].restored_targets.len(), 2);
        assert_eq!(fs::read(&settings_path).unwrap(), original_settings);
        assert!(!new_path.exists());
        assert!(!manifest.exists());
    }

    #[test]
    fn failed_peer_write_restores_an_already_deleted_platform() {
        let (directory, _, settings_path) = fixture_tree();
        let empty_path = directory.path().join("Data/Platforms/Delete Rollback.xml");
        let original_empty = b"<LaunchBox><Unknown>preserve exactly</Unknown></LaunchBox>";
        fs::write(&empty_path, original_empty).unwrap();
        let empty_platform = PlatformDocument::load(&empty_path).unwrap();
        let mut settings = AuxiliaryDocument::load(&settings_path).unwrap();
        settings
            .set_single_record_field("Settings", "Theme", "Never Committed")
            .unwrap();
        let mut transaction = LibraryTransaction::new(directory.path()).unwrap();
        transaction.stage_delete_platform(&empty_platform).unwrap();
        transaction.stage_auxiliary(&settings).unwrap();

        assert!(matches!(
            transaction.commit_with_apply(
                |index, entry| {
                    if index == 1 {
                        Err(std::io::Error::other("injected peer failure"))
                    } else {
                        apply_prepared_change(entry)
                    }
                },
                true,
            ),
            Err(TransactionError::CommitRolledBack { .. })
        ));
        assert_eq!(fs::read(&empty_path).unwrap(), original_empty);
    }

    #[test]
    fn streams_file_creation_in_the_same_recoverable_transaction_as_xml() {
        let (directory, platform_path, _) = fixture_tree();
        let source_directory = tempfile::tempdir().unwrap();
        let source = source_directory.path().join("disc-image.bin");
        let mut expected = vec![0_u8; 2 * 1024 * 1024 + 17];
        for (index, byte) in expected.iter_mut().enumerate() {
            *byte = (index % 251) as u8;
        }
        fs::write(&source, &expected).unwrap();
        let target_directory = directory.path().join("Games/Fixture Console");
        fs::create_dir_all(&target_directory).unwrap();
        let target = target_directory.join("disc-image.bin");
        let mut platform = PlatformDocument::load(&platform_path).unwrap();
        platform
            .set_game_title("fixture-adventure", "Imported Adventure")
            .unwrap();

        let mut transaction = LibraryTransaction::new(directory.path()).unwrap();
        transaction.stage_file_copy(&source, &target).unwrap();
        transaction.stage_platform(&platform).unwrap();
        let report = transaction.commit().unwrap();

        assert_eq!(report.created_targets, vec![target.clone()]);
        assert_eq!(fs::read(&target).unwrap(), expected);
        assert_eq!(
            PlatformDocument::load(&platform_path)
                .unwrap()
                .library()
                .games[0]
                .title,
            "Imported Adventure"
        );
    }

    #[test]
    fn refuses_a_streamed_copy_when_the_source_changed_after_inspection() {
        let (directory, _, _) = fixture_tree();
        let source_directory = tempfile::tempdir().unwrap();
        let source = source_directory.path().join("active-save.srm");
        fs::write(&source, b"inspected save bytes").unwrap();
        let expected = FileRevision::read(&source).unwrap();
        fs::write(&source, b"new emulator save bytes").unwrap();
        let target_directory = directory.path().join("Saves/Fixture Console");
        fs::create_dir_all(&target_directory).unwrap();
        let target = target_directory.join("fixture-adventure.srm");

        let mut transaction = LibraryTransaction::new(directory.path()).unwrap();
        transaction
            .stage_file_copy_with_revision(&source, &target, expected.clone())
            .unwrap();
        assert!(matches!(
            transaction.commit(),
            Err(TransactionError::SourceConflict {
                path,
                expected: actual_expected,
                ..
            }) if path == fs::canonicalize(&source).unwrap() && actual_expected == expected
        ));
        assert!(!target.exists());
    }

    #[test]
    fn peer_failure_removes_a_streamed_file_and_restores_xml() {
        let (directory, platform_path, _) = fixture_tree();
        let original_platform = fs::read(&platform_path).unwrap();
        let source_directory = tempfile::tempdir().unwrap();
        let source = source_directory.path().join("game.rom");
        fs::write(&source, b"source rom bytes").unwrap();
        let target_directory = directory.path().join("Games/Fixture Console");
        fs::create_dir_all(&target_directory).unwrap();
        let target = target_directory.join("game.rom");
        let mut platform = PlatformDocument::load(&platform_path).unwrap();
        platform
            .set_game_title("fixture-adventure", "Never Committed")
            .unwrap();

        let mut transaction = LibraryTransaction::new(directory.path()).unwrap();
        transaction.stage_file_copy(&source, &target).unwrap();
        transaction.stage_platform(&platform).unwrap();
        assert!(matches!(
            transaction.commit_with_apply(
                |index, entry| {
                    if index == 1 {
                        Err(std::io::Error::other("injected XML failure"))
                    } else {
                        apply_prepared_change(entry)
                    }
                },
                true,
            ),
            Err(TransactionError::CommitRolledBack { .. })
        ));

        assert!(!target.exists());
        assert_eq!(fs::read(&platform_path).unwrap(), original_platform);
        assert_eq!(fs::read(&source).unwrap(), b"source rom bytes");
    }

    #[test]
    fn create_conflict_is_detected_again_at_commit_time() {
        let (directory, _, _) = fixture_tree();
        let new_path = directory.path().join("Data/Platforms/Raced Console.xml");
        let new_platform = PlatformDocument::from_reader(&new_path, &b"<LaunchBox />"[..]).unwrap();
        let mut transaction = LibraryTransaction::new(directory.path()).unwrap();
        transaction.stage_new_platform(&new_platform).unwrap();
        fs::write(&new_path, b"external creator").unwrap();

        assert!(matches!(
            transaction.commit(),
            Err(TransactionError::NewTargetAlreadyExists { path }) if path == new_path
        ));
        assert_eq!(fs::read(&new_path).unwrap(), b"external creator");
    }

    #[test]
    fn non_empty_platform_cannot_be_staged_for_deletion() {
        let (directory, platform_path, _) = fixture_tree();
        let platform = PlatformDocument::load(&platform_path).unwrap();
        let mut transaction = LibraryTransaction::new(directory.path()).unwrap();
        assert!(matches!(
            transaction.stage_delete_platform(&platform),
            Err(TransactionError::PlatformDocumentNotEmpty { path }) if path == platform_path
        ));
    }

    #[test]
    fn commits_playlist_create_edit_and_delete_with_hierarchy_peers() {
        let directory = tempfile::tempdir().expect("temporary library");
        let data = directory.path().join("Data");
        let playlists = data.join("Playlists");
        fs::create_dir_all(&playlists).unwrap();
        let parents_path = data.join("Parents.xml");
        fs::write(
            &parents_path,
            b"<LaunchBox><FutureRoot>keep</FutureRoot></LaunchBox>",
        )
        .unwrap();
        let playlist_path = playlists.join("Portable List.xml");
        let playlist = lb_domain::Playlist {
            id: "portable-list".into(),
            metadata: lb_domain::NavigationMetadata {
                name: "Portable List".into(),
                ..lb_domain::NavigationMetadata::default()
            },
            ..lb_domain::Playlist::default()
        };
        let new_playlist = AuxiliaryDocument::new_playlist(
            &playlist_path,
            playlist.clone(),
            Vec::new(),
            Vec::new(),
        )
        .unwrap();
        let mut parents = AuxiliaryDocument::load(&parents_path).unwrap();
        parents
            .set_playlist_parents(
                "portable-list",
                vec![crate::IndexedPlatformRecordEdit {
                    source_index: None,
                    record: lb_domain::ParentRelationship {
                        playlist_id: Some("portable-list".into()),
                        ..lb_domain::ParentRelationship::default()
                    },
                }],
            )
            .unwrap();
        let original_parents = fs::read(&parents_path).unwrap();
        let mut create = LibraryTransaction::new(directory.path()).unwrap();
        create.stage_new_playlist(&new_playlist).unwrap();
        create.stage_auxiliary(&parents).unwrap();
        let created = create.commit().unwrap();
        assert_eq!(
            created.created_targets.as_slice(),
            std::slice::from_ref(&playlist_path)
        );
        assert_eq!(created.writes.len(), 1);
        assert_eq!(
            fs::read(&created.writes[0].backup).unwrap(),
            original_parents
        );

        let original_playlist = fs::read(&playlist_path).unwrap();
        let mut edited_playlist = AuxiliaryDocument::load(&playlist_path).unwrap();
        let mut edited = playlist.clone();
        edited.metadata.nested_name = Some("Portable Collection".into());
        edited_playlist
            .set_playlist("portable-list", edited, Vec::new(), Vec::new())
            .unwrap();
        let mut edit = LibraryTransaction::new(directory.path()).unwrap();
        edit.stage_auxiliary(&edited_playlist).unwrap();
        let edited_report = edit.commit().unwrap();
        assert_eq!(edited_report.writes.len(), 1);
        assert_eq!(
            fs::read(&edited_report.writes[0].backup).unwrap(),
            original_playlist
        );

        let loaded_playlist = AuxiliaryDocument::load(&playlist_path).unwrap();
        let mut deleted_parents = AuxiliaryDocument::load(&parents_path).unwrap();
        let removed = deleted_parents
            .remove_playlist_relationships("portable-list")
            .unwrap();
        assert_eq!(removed.removed_placements, 1);
        let before_delete = fs::read(&playlist_path).unwrap();
        let mut delete = LibraryTransaction::new(directory.path()).unwrap();
        delete.stage_delete_playlist(&loaded_playlist).unwrap();
        delete.stage_auxiliary(&deleted_parents).unwrap();
        let deleted = delete.commit().unwrap();
        assert_eq!(deleted.deleted_targets.len(), 1);
        assert_eq!(deleted.deleted_targets[0].target, playlist_path);
        assert_eq!(
            fs::read(&deleted.deleted_targets[0].backup).unwrap(),
            before_delete
        );
        assert!(!deleted.deleted_targets[0].target.exists());
        assert!(fs::read_to_string(parents_path)
            .unwrap()
            .contains("<FutureRoot>keep</FutureRoot>"));
    }
}
