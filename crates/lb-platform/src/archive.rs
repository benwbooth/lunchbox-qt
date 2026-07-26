use std::ffi::{OsStr, OsString};
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::{
    collections::BTreeSet,
    io::{Read, Write},
};

use tempfile::{Builder, TempDir};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveExtractor {
    executable: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArchiveExtractionLimits {
    pub max_entries: usize,
    pub max_member_bytes: u64,
    pub max_total_bytes: u64,
}

impl ArchiveExtractor {
    pub fn new(executable: impl Into<PathBuf>) -> Self {
        Self {
            executable: executable.into(),
        }
    }

    /// Uses LaunchBox's bundled 7-Zip on Windows and the packaged `7z`
    /// command on Unix hosts. The executable is always invoked directly;
    /// persisted paths and archive names never pass through a command shell.
    pub fn for_launchbox_root(launchbox_root: &Path) -> Self {
        #[cfg(windows)]
        {
            let bundled = launchbox_root
                .join("ThirdParty")
                .join("7-Zip")
                .join("7z.exe");
            if bundled.is_file() {
                return Self::new(bundled);
            }
        }
        #[cfg(not(windows))]
        let _ = launchbox_root;

        Self::new("7z")
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }

    /// Creates one flat, non-encrypted `.7z` archive from a real directory.
    ///
    /// This is the recovered LaunchBox 13.27 storage shape for emulator save
    /// adapters that back up one logical container member as several files.
    /// Source names are passed directly to 7-Zip without a command shell and
    /// the resulting member listing is checked before success is returned.
    pub fn create_7z_from_directory(
        &self,
        source_directory: &Path,
        archive: &Path,
    ) -> Result<(), ArchiveCreationError> {
        let source_metadata =
            fs::symlink_metadata(source_directory).map_err(|source| ArchiveCreationError::Io {
                path: source_directory.to_path_buf(),
                source,
            })?;
        if !source_metadata.file_type().is_dir() || source_metadata.file_type().is_symlink() {
            return Err(ArchiveCreationError::SourceNotDirectory {
                path: source_directory.to_path_buf(),
            });
        }
        let source_directory =
            fs::canonicalize(source_directory).map_err(|source| ArchiveCreationError::Io {
                path: source_directory.to_path_buf(),
                source,
            })?;
        let archive_name = archive
            .file_name()
            .filter(|name| !name.is_empty())
            .ok_or_else(|| ArchiveCreationError::InvalidTarget {
                path: archive.to_path_buf(),
            })?;
        if !archive
            .extension()
            .and_then(OsStr::to_str)
            .is_some_and(|extension| extension.eq_ignore_ascii_case("7z"))
        {
            return Err(ArchiveCreationError::InvalidTarget {
                path: archive.to_path_buf(),
            });
        }
        let archive_parent =
            archive
                .parent()
                .ok_or_else(|| ArchiveCreationError::InvalidTarget {
                    path: archive.to_path_buf(),
                })?;
        let archive_parent =
            fs::canonicalize(archive_parent).map_err(|source| ArchiveCreationError::Io {
                path: archive_parent.to_path_buf(),
                source,
            })?;
        let archive = archive_parent.join(archive_name);
        match fs::symlink_metadata(&archive) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Ok(_) => return Err(ArchiveCreationError::TargetExists { path: archive }),
            Err(source) => {
                return Err(ArchiveCreationError::Io {
                    path: archive,
                    source,
                })
            }
        }

        let mut members = Vec::<(String, OsString)>::new();
        let mut identities = BTreeSet::new();
        let entries =
            fs::read_dir(&source_directory).map_err(|source| ArchiveCreationError::Io {
                path: source_directory.clone(),
                source,
            })?;
        for entry in entries {
            let entry = entry.map_err(|source| ArchiveCreationError::Io {
                path: source_directory.clone(),
                source,
            })?;
            let path = entry.path();
            let metadata =
                fs::symlink_metadata(&path).map_err(|source| ArchiveCreationError::Io {
                    path: path.clone(),
                    source,
                })?;
            if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
                return Err(ArchiveCreationError::UnsupportedSourceEntry { path });
            }
            let name = entry.file_name();
            let name_text = name
                .to_str()
                .filter(|name| is_safe_archive_member(name))
                .ok_or_else(|| ArchiveCreationError::UnsafeSourceName { path: path.clone() })?
                .to_string();
            if !identities.insert(name_text.to_ascii_lowercase()) {
                return Err(ArchiveCreationError::DuplicateSourceName { name: name_text });
            }
            members.push((name_text, name));
        }
        members.sort_by(|left, right| {
            left.0
                .to_ascii_lowercase()
                .cmp(&right.0.to_ascii_lowercase())
                .then_with(|| left.0.cmp(&right.0))
        });
        if members.is_empty() {
            return Err(ArchiveCreationError::EmptySource {
                path: source_directory,
            });
        }

        let output = Command::new(&self.executable)
            .current_dir(&source_directory)
            .arg("a")
            .arg("-t7z")
            .arg("-mx=9")
            .arg("-bd")
            .arg("-bb0")
            .arg("-y")
            .arg("-sccUTF-8")
            .arg(&archive)
            .arg("--")
            .args(members.iter().map(|(_, name)| name))
            .stdin(Stdio::null())
            .output()
            .map_err(|error| ArchiveCreationError::ToolStart {
                executable: self.executable.clone(),
                archive: archive.clone(),
                message: error.to_string(),
            })?;
        if !output.status.success() {
            let _ = fs::remove_file(&archive);
            return Err(ArchiveCreationError::ToolFailed {
                executable: self.executable.clone(),
                archive,
                message: tool_output_message(&output.stdout, &output.stderr),
            });
        }
        let archive_metadata =
            fs::symlink_metadata(&archive).map_err(|source| ArchiveCreationError::Io {
                path: archive.clone(),
                source,
            })?;
        if !archive_metadata.file_type().is_file() || archive_metadata.file_type().is_symlink() {
            let _ = fs::remove_file(&archive);
            return Err(ArchiveCreationError::InvalidCreatedArchive { path: archive });
        }
        let listed = match self.list_entries(&archive).and_then(|entries| {
            validate_entries(&archive, &entries)?;
            Ok(entries)
        }) {
            Ok(entries) => entries,
            Err(source) => {
                let _ = fs::remove_file(&archive);
                return Err(ArchiveCreationError::Verification {
                    archive,
                    source: Box::new(source),
                });
            }
        };
        let listed = listed
            .into_iter()
            .map(|entry| entry.path.to_ascii_lowercase())
            .collect::<BTreeSet<_>>();
        let expected = members
            .iter()
            .map(|(name, _)| name.to_ascii_lowercase())
            .collect::<BTreeSet<_>>();
        if listed != expected {
            let _ = fs::remove_file(&archive);
            return Err(ArchiveCreationError::MemberMismatch {
                archive,
                expected: expected.into_iter().collect(),
                actual: listed.into_iter().collect(),
            });
        }
        let mut archive_file =
            fs::OpenOptions::new()
                .write(true)
                .open(&archive)
                .map_err(|source| ArchiveCreationError::Io {
                    path: archive.clone(),
                    source,
                })?;
        archive_file
            .flush()
            .and_then(|()| archive_file.sync_all())
            .map_err(|source| ArchiveCreationError::Io {
                path: archive,
                source,
            })
    }

    /// Creates one recursively verified `.7z` archive from a real directory.
    ///
    /// Every source entry is walked before 7-Zip starts. Symlinks, special
    /// files, non-Unicode names, unsafe portable components, and
    /// case-insensitive path collisions are rejected. The archive is created
    /// from explicit top-level arguments without a command shell, then its
    /// complete technical listing is compared with the preflight tree.
    pub fn create_7z_from_tree(
        &self,
        source_directory: &Path,
        archive: &Path,
    ) -> Result<(), ArchiveCreationError> {
        let source_metadata =
            fs::symlink_metadata(source_directory).map_err(|source| ArchiveCreationError::Io {
                path: source_directory.to_path_buf(),
                source,
            })?;
        if !source_metadata.file_type().is_dir() || source_metadata.file_type().is_symlink() {
            return Err(ArchiveCreationError::SourceNotDirectory {
                path: source_directory.to_path_buf(),
            });
        }
        let source_directory =
            fs::canonicalize(source_directory).map_err(|source| ArchiveCreationError::Io {
                path: source_directory.to_path_buf(),
                source,
            })?;
        let archive_name = archive
            .file_name()
            .filter(|name| !name.is_empty())
            .ok_or_else(|| ArchiveCreationError::InvalidTarget {
                path: archive.to_path_buf(),
            })?;
        if !archive
            .extension()
            .and_then(OsStr::to_str)
            .is_some_and(|extension| extension.eq_ignore_ascii_case("7z"))
        {
            return Err(ArchiveCreationError::InvalidTarget {
                path: archive.to_path_buf(),
            });
        }
        let archive_parent =
            archive
                .parent()
                .ok_or_else(|| ArchiveCreationError::InvalidTarget {
                    path: archive.to_path_buf(),
                })?;
        let archive_parent =
            fs::canonicalize(archive_parent).map_err(|source| ArchiveCreationError::Io {
                path: archive_parent.to_path_buf(),
                source,
            })?;
        if archive_parent.starts_with(&source_directory) {
            return Err(ArchiveCreationError::InvalidTarget {
                path: archive.to_path_buf(),
            });
        }
        let archive = archive_parent.join(archive_name);
        match fs::symlink_metadata(&archive) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Ok(_) => return Err(ArchiveCreationError::TargetExists { path: archive }),
            Err(source) => {
                return Err(ArchiveCreationError::Io {
                    path: archive,
                    source,
                });
            }
        }

        let mut top_level = Vec::<(String, OsString)>::new();
        let mut expected = BTreeSet::new();
        collect_archive_tree(
            &source_directory,
            &source_directory,
            &mut top_level,
            &mut expected,
        )?;
        top_level.sort_by(|left, right| {
            left.0
                .to_ascii_lowercase()
                .cmp(&right.0.to_ascii_lowercase())
                .then_with(|| left.0.cmp(&right.0))
        });
        if expected.is_empty() {
            return Err(ArchiveCreationError::EmptySource {
                path: source_directory,
            });
        }

        let output = Command::new(&self.executable)
            .current_dir(&source_directory)
            .arg("a")
            .arg("-t7z")
            .arg("-mx=9")
            .arg("-bd")
            .arg("-bb0")
            .arg("-y")
            .arg("-sccUTF-8")
            .arg(&archive)
            .arg("--")
            .args(top_level.iter().map(|(_, name)| name))
            .stdin(Stdio::null())
            .output()
            .map_err(|error| ArchiveCreationError::ToolStart {
                executable: self.executable.clone(),
                archive: archive.clone(),
                message: error.to_string(),
            })?;
        if !output.status.success() {
            let _ = fs::remove_file(&archive);
            return Err(ArchiveCreationError::ToolFailed {
                executable: self.executable.clone(),
                archive,
                message: tool_output_message(&output.stdout, &output.stderr),
            });
        }
        let archive_metadata =
            fs::symlink_metadata(&archive).map_err(|source| ArchiveCreationError::Io {
                path: archive.clone(),
                source,
            })?;
        if !archive_metadata.file_type().is_file() || archive_metadata.file_type().is_symlink() {
            let _ = fs::remove_file(&archive);
            return Err(ArchiveCreationError::InvalidCreatedArchive { path: archive });
        }
        let listed = match self.list_entries(&archive).and_then(|entries| {
            validate_entries(&archive, &entries)?;
            Ok(entries)
        }) {
            Ok(entries) => entries,
            Err(source) => {
                let _ = fs::remove_file(&archive);
                return Err(ArchiveCreationError::Verification {
                    archive,
                    source: Box::new(source),
                });
            }
        };
        let listed = listed
            .into_iter()
            .map(|entry| entry.path.to_ascii_lowercase())
            .collect::<BTreeSet<_>>();
        if listed != expected {
            let _ = fs::remove_file(&archive);
            return Err(ArchiveCreationError::MemberMismatch {
                archive,
                expected: expected.into_iter().collect(),
                actual: listed.into_iter().collect(),
            });
        }
        let mut archive_file =
            fs::OpenOptions::new()
                .write(true)
                .open(&archive)
                .map_err(|source| ArchiveCreationError::Io {
                    path: archive.clone(),
                    source,
                })?;
        archive_file
            .flush()
            .and_then(|()| archive_file.sync_all())
            .map_err(|source| ArchiveCreationError::Io {
                path: archive,
                source,
            })
    }

    /// Safely extracts every member into an existing empty real directory.
    ///
    /// Unlike launch preparation, this does not choose a runnable file and
    /// does not own the destination lifetime. It is intended for adapters
    /// whose archive is a logical multi-file save rather than a game image.
    pub fn extract_to_directory(
        &self,
        archive: &Path,
        destination: &Path,
    ) -> Result<Vec<PathBuf>, ArchiveExtractionError> {
        self.extract_to_directory_with_limits(archive, destination, None)
    }

    /// Safely extracts an archive while enforcing its declared and resulting
    /// member counts and sizes.
    ///
    /// The technical listing is checked before extraction, so a data archive
    /// cannot silently expand beyond its configured boundary. The extracted
    /// regular files are checked again before they are returned.
    pub fn extract_to_directory_bounded(
        &self,
        archive: &Path,
        destination: &Path,
        limits: ArchiveExtractionLimits,
    ) -> Result<Vec<PathBuf>, ArchiveExtractionError> {
        self.extract_to_directory_with_limits(archive, destination, Some(limits))
    }

    fn extract_to_directory_with_limits(
        &self,
        archive: &Path,
        destination: &Path,
        limits: Option<ArchiveExtractionLimits>,
    ) -> Result<Vec<PathBuf>, ArchiveExtractionError> {
        if !archive.is_file() {
            return Err(ArchiveExtractionError::ArchiveNotFound {
                archive: archive.to_path_buf(),
            });
        }
        let entries = self.list_entries(archive)?;
        validate_entries(archive, &entries)?;
        if let Some(limits) = limits {
            validate_entry_limits(archive, &entries, limits)?;
        }
        let metadata = fs::symlink_metadata(destination).map_err(|error| {
            ArchiveExtractionError::ExtractedTree {
                archive: archive.to_path_buf(),
                path: destination.to_path_buf(),
                message: error.to_string(),
            }
        })?;
        if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
            return Err(ArchiveExtractionError::ExtractedTree {
                archive: archive.to_path_buf(),
                path: destination.to_path_buf(),
                message: "destination is not a real directory".into(),
            });
        }
        let destination = fs::canonicalize(destination).map_err(|error| {
            ArchiveExtractionError::ExtractedTree {
                archive: archive.to_path_buf(),
                path: destination.to_path_buf(),
                message: error.to_string(),
            }
        })?;
        let mut destination_entries =
            fs::read_dir(&destination).map_err(|error| ArchiveExtractionError::ExtractedTree {
                archive: archive.to_path_buf(),
                path: destination.clone(),
                message: error.to_string(),
            })?;
        if destination_entries.next().is_some() {
            return Err(ArchiveExtractionError::ExtractedTree {
                archive: archive.to_path_buf(),
                path: destination,
                message: "destination directory is not empty".into(),
            });
        }
        let mut output_directory_argument = OsString::from("-o");
        output_directory_argument.push(&destination);
        let output = Command::new(&self.executable)
            .arg("x")
            .arg("-y")
            .arg("-aos")
            .arg("-bd")
            .arg("-bb0")
            .arg(output_directory_argument)
            .arg("--")
            .arg(archive)
            .stdin(Stdio::null())
            .output()
            .map_err(|error| ArchiveExtractionError::ToolStart {
                executable: self.executable.clone(),
                operation: "extract",
                archive: archive.to_path_buf(),
                message: error.to_string(),
            })?;
        if !output.status.success() {
            return Err(ArchiveExtractionError::ToolFailed {
                executable: self.executable.clone(),
                operation: "extract",
                archive: archive.to_path_buf(),
                message: tool_output_message(&output.stdout, &output.stderr),
            });
        }
        let files = audit_extracted_tree(archive, &destination)?;
        if let Some(limits) = limits {
            validate_extracted_limits(archive, &files, limits)?;
        }
        Ok(files)
    }

    /// Extracts one bounded `.tar.xz` payload through two audited stages.
    ///
    /// XZ is a single compressed stream and does not carry a member pathname,
    /// so 7-Zip's normal technical listing has no `Path` record to validate.
    /// This boundary instead requires an exact `.tar.xz` name, verifies the XZ
    /// signature and declared unpacked size, extracts the one derived `.tar`
    /// file into an empty real directory, and then sends that tar through the
    /// ordinary traversal/link/special-file checks. Both 7-Zip invocations are
    /// direct process calls with argument vectors; no command shell is used.
    pub fn extract_tar_xz_to_directory(
        &self,
        archive: &Path,
        stream_directory: &Path,
        destination: &Path,
        max_tar_bytes: u64,
    ) -> Result<Vec<PathBuf>, ArchiveExtractionError> {
        let metadata =
            fs::symlink_metadata(archive).map_err(|_| ArchiveExtractionError::ArchiveNotFound {
                archive: archive.to_path_buf(),
            })?;
        if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
            return Err(ArchiveExtractionError::ArchiveNotFound {
                archive: archive.to_path_buf(),
            });
        }
        let archive_name =
            archive
                .file_name()
                .ok_or_else(|| ArchiveExtractionError::MissingArchiveFileName {
                    archive: archive.to_path_buf(),
                })?;
        let archive_name_text = archive_name.to_string_lossy();
        if !archive_name_text.to_ascii_lowercase().ends_with(".tar.xz") {
            return Err(ArchiveExtractionError::InvalidCompressedTar {
                archive: archive.to_path_buf(),
                message: "the source name does not end in .tar.xz".into(),
            });
        }
        let expected_tar_name = Path::new(archive_name)
            .file_stem()
            .filter(|name| {
                Path::new(name)
                    .extension()
                    .and_then(OsStr::to_str)
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("tar"))
            })
            .ok_or_else(|| ArchiveExtractionError::InvalidCompressedTar {
                archive: archive.to_path_buf(),
                message: "the source has no derived .tar filename".into(),
            })?
            .to_os_string();

        let mut signature = [0_u8; 6];
        fs::File::open(archive)
            .and_then(|mut file| file.read_exact(&mut signature))
            .map_err(|error| ArchiveExtractionError::InvalidCompressedTar {
                archive: archive.to_path_buf(),
                message: format!("could not read the XZ signature: {error}"),
            })?;
        if signature != [0xfd, b'7', b'z', b'X', b'Z', 0x00] {
            return Err(ArchiveExtractionError::InvalidCompressedTar {
                archive: archive.to_path_buf(),
                message: "the source does not have an XZ stream signature".into(),
            });
        }
        if max_tar_bytes == 0 {
            return Err(ArchiveExtractionError::InvalidCompressedTar {
                archive: archive.to_path_buf(),
                message: "the permitted unpacked size is zero".into(),
            });
        }

        let output = Command::new(&self.executable)
            .arg("l")
            .arg("-slt")
            .arg("-sccUTF-8")
            .arg("-ba")
            .arg("--")
            .arg(archive)
            .stdin(Stdio::null())
            .output()
            .map_err(|error| ArchiveExtractionError::ToolStart {
                executable: self.executable.clone(),
                operation: "list",
                archive: archive.to_path_buf(),
                message: error.to_string(),
            })?;
        if !output.status.success() {
            return Err(ArchiveExtractionError::ToolFailed {
                executable: self.executable.clone(),
                operation: "list",
                archive: archive.to_path_buf(),
                message: tool_output_message(&output.stdout, &output.stderr),
            });
        }
        let listing = String::from_utf8(output.stdout).map_err(|error| {
            ArchiveExtractionError::InvalidToolOutput {
                archive: archive.to_path_buf(),
                message: error.to_string(),
            }
        })?;
        if !parse_technical_listing(&listing).is_empty() {
            return Err(ArchiveExtractionError::InvalidCompressedTar {
                archive: archive.to_path_buf(),
                message: "the XZ listing unexpectedly contains named members".into(),
            });
        }
        let unpacked_sizes = listing
            .lines()
            .filter_map(|line| line.trim_end_matches('\r').strip_prefix("Size = "))
            .collect::<Vec<_>>();
        let [unpacked_size] = unpacked_sizes.as_slice() else {
            return Err(ArchiveExtractionError::InvalidCompressedTar {
                archive: archive.to_path_buf(),
                message: "the XZ listing does not contain exactly one unpacked size".into(),
            });
        };
        let unpacked_size = unpacked_size.parse::<u64>().map_err(|error| {
            ArchiveExtractionError::InvalidCompressedTar {
                archive: archive.to_path_buf(),
                message: format!("the XZ unpacked size is invalid: {error}"),
            }
        })?;
        if unpacked_size == 0 || unpacked_size > max_tar_bytes {
            return Err(ArchiveExtractionError::CompressedTarTooLarge {
                archive: archive.to_path_buf(),
                unpacked_size,
                max_tar_bytes,
            });
        }

        let stream_directory =
            empty_real_directory(archive, stream_directory, "stream extraction")?;
        let destination = empty_real_directory(archive, destination, "tar extraction")?;
        if stream_directory == destination {
            return Err(ArchiveExtractionError::InvalidCompressedTar {
                archive: archive.to_path_buf(),
                message: "the stream and tar extraction directories are identical".into(),
            });
        }
        let mut output_directory_argument = OsString::from("-o");
        output_directory_argument.push(&stream_directory);
        let output = Command::new(&self.executable)
            .arg("x")
            .arg("-y")
            .arg("-aos")
            .arg("-bd")
            .arg("-bb0")
            .arg(output_directory_argument)
            .arg("--")
            .arg(archive)
            .stdin(Stdio::null())
            .output()
            .map_err(|error| ArchiveExtractionError::ToolStart {
                executable: self.executable.clone(),
                operation: "extract XZ stream",
                archive: archive.to_path_buf(),
                message: error.to_string(),
            })?;
        if !output.status.success() {
            return Err(ArchiveExtractionError::ToolFailed {
                executable: self.executable.clone(),
                operation: "extract XZ stream",
                archive: archive.to_path_buf(),
                message: tool_output_message(&output.stdout, &output.stderr),
            });
        }
        let stream_files = audit_extracted_tree(archive, &stream_directory)?;
        let [tar_path] = stream_files.as_slice() else {
            return Err(ArchiveExtractionError::InvalidCompressedTar {
                archive: archive.to_path_buf(),
                message: format!(
                    "the XZ stream produced {} files instead of one",
                    stream_files.len()
                ),
            });
        };
        if tar_path.file_name() != Some(expected_tar_name.as_os_str()) {
            return Err(ArchiveExtractionError::InvalidCompressedTar {
                archive: archive.to_path_buf(),
                message: format!(
                    "the XZ stream produced unexpected file {}",
                    tar_path.display()
                ),
            });
        }
        let actual_size = fs::symlink_metadata(tar_path)
            .map_err(|error| ArchiveExtractionError::InvalidCompressedTar {
                archive: archive.to_path_buf(),
                message: format!("could not inspect extracted tar: {error}"),
            })?
            .len();
        if actual_size != unpacked_size {
            return Err(ArchiveExtractionError::InvalidCompressedTar {
                archive: archive.to_path_buf(),
                message: format!(
                    "the extracted tar size is {actual_size}, expected {unpacked_size}"
                ),
            });
        }

        self.extract_to_directory(tar_path, &destination)
    }

    /// Extracts one bounded `.tar.gz` payload through two audited stages.
    ///
    /// GZip carries one stream filename, which must be exactly the `.tar`
    /// derived from the source archive name. The declared unpacked size is
    /// checked before extraction, the one stream output is checked again by
    /// size and name, and all tar members then pass through the ordinary
    /// traversal/link/special-file boundary. Every tool is started directly
    /// with an argument vector; no command shell is involved.
    pub fn extract_tar_gz_to_directory(
        &self,
        archive: &Path,
        stream_directory: &Path,
        destination: &Path,
        max_tar_bytes: u64,
    ) -> Result<Vec<PathBuf>, ArchiveExtractionError> {
        let metadata =
            fs::symlink_metadata(archive).map_err(|_| ArchiveExtractionError::ArchiveNotFound {
                archive: archive.to_path_buf(),
            })?;
        if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
            return Err(ArchiveExtractionError::ArchiveNotFound {
                archive: archive.to_path_buf(),
            });
        }
        let archive_name =
            archive
                .file_name()
                .ok_or_else(|| ArchiveExtractionError::MissingArchiveFileName {
                    archive: archive.to_path_buf(),
                })?;
        let archive_name_text = archive_name.to_string_lossy();
        if !archive_name_text.to_ascii_lowercase().ends_with(".tar.gz") {
            return Err(ArchiveExtractionError::InvalidCompressedTar {
                archive: archive.to_path_buf(),
                message: "the source name does not end in .tar.gz".into(),
            });
        }
        let expected_tar_name = Path::new(archive_name)
            .file_stem()
            .filter(|name| {
                Path::new(name)
                    .extension()
                    .and_then(OsStr::to_str)
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("tar"))
            })
            .ok_or_else(|| ArchiveExtractionError::InvalidCompressedTar {
                archive: archive.to_path_buf(),
                message: "the source has no derived .tar filename".into(),
            })?
            .to_os_string();

        let mut signature = [0_u8; 2];
        fs::File::open(archive)
            .and_then(|mut file| file.read_exact(&mut signature))
            .map_err(|error| ArchiveExtractionError::InvalidCompressedTar {
                archive: archive.to_path_buf(),
                message: format!("could not read the GZip signature: {error}"),
            })?;
        if signature != [0x1f, 0x8b] {
            return Err(ArchiveExtractionError::InvalidCompressedTar {
                archive: archive.to_path_buf(),
                message: "the source does not have a GZip stream signature".into(),
            });
        }
        if max_tar_bytes == 0 {
            return Err(ArchiveExtractionError::InvalidCompressedTar {
                archive: archive.to_path_buf(),
                message: "the permitted unpacked size is zero".into(),
            });
        }

        let output = Command::new(&self.executable)
            .arg("l")
            .arg("-slt")
            .arg("-sccUTF-8")
            .arg("-ba")
            .arg("--")
            .arg(archive)
            .stdin(Stdio::null())
            .output()
            .map_err(|error| ArchiveExtractionError::ToolStart {
                executable: self.executable.clone(),
                operation: "list",
                archive: archive.to_path_buf(),
                message: error.to_string(),
            })?;
        if !output.status.success() {
            return Err(ArchiveExtractionError::ToolFailed {
                executable: self.executable.clone(),
                operation: "list",
                archive: archive.to_path_buf(),
                message: tool_output_message(&output.stdout, &output.stderr),
            });
        }
        let listing = String::from_utf8(output.stdout).map_err(|error| {
            ArchiveExtractionError::InvalidToolOutput {
                archive: archive.to_path_buf(),
                message: error.to_string(),
            }
        })?;
        let entries = parse_technical_listing(&listing);
        let [stream_entry] = entries.as_slice() else {
            return Err(ArchiveExtractionError::InvalidCompressedTar {
                archive: archive.to_path_buf(),
                message: format!(
                    "the GZip listing contains {} stream names instead of one",
                    entries.len()
                ),
            });
        };
        if stream_entry.encrypted
            || Path::new(&stream_entry.path).file_name() != Some(expected_tar_name.as_os_str())
            || Path::new(&stream_entry.path)
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err(ArchiveExtractionError::InvalidCompressedTar {
                archive: archive.to_path_buf(),
                message: format!(
                    "the GZip stream name is unsafe or unexpected: {}",
                    stream_entry.path
                ),
            });
        }
        let unpacked_sizes = listing
            .lines()
            .filter_map(|line| line.trim_end_matches('\r').strip_prefix("Size = "))
            .collect::<Vec<_>>();
        let [unpacked_size] = unpacked_sizes.as_slice() else {
            return Err(ArchiveExtractionError::InvalidCompressedTar {
                archive: archive.to_path_buf(),
                message: "the GZip listing does not contain exactly one unpacked size".into(),
            });
        };
        let unpacked_size = unpacked_size.parse::<u64>().map_err(|error| {
            ArchiveExtractionError::InvalidCompressedTar {
                archive: archive.to_path_buf(),
                message: format!("the GZip unpacked size is invalid: {error}"),
            }
        })?;
        if unpacked_size == 0 || unpacked_size > max_tar_bytes {
            return Err(ArchiveExtractionError::CompressedTarTooLarge {
                archive: archive.to_path_buf(),
                unpacked_size,
                max_tar_bytes,
            });
        }

        let stream_directory =
            empty_real_directory(archive, stream_directory, "stream extraction")?;
        let destination = empty_real_directory(archive, destination, "tar extraction")?;
        if stream_directory == destination {
            return Err(ArchiveExtractionError::InvalidCompressedTar {
                archive: archive.to_path_buf(),
                message: "the stream and tar extraction directories are identical".into(),
            });
        }
        let mut output_directory_argument = OsString::from("-o");
        output_directory_argument.push(&stream_directory);
        let output = Command::new(&self.executable)
            .arg("x")
            .arg("-y")
            .arg("-aos")
            .arg("-bd")
            .arg("-bb0")
            .arg(output_directory_argument)
            .arg("--")
            .arg(archive)
            .stdin(Stdio::null())
            .output()
            .map_err(|error| ArchiveExtractionError::ToolStart {
                executable: self.executable.clone(),
                operation: "extract GZip stream",
                archive: archive.to_path_buf(),
                message: error.to_string(),
            })?;
        if !output.status.success() {
            return Err(ArchiveExtractionError::ToolFailed {
                executable: self.executable.clone(),
                operation: "extract GZip stream",
                archive: archive.to_path_buf(),
                message: tool_output_message(&output.stdout, &output.stderr),
            });
        }
        let stream_files = audit_extracted_tree(archive, &stream_directory)?;
        let [tar_path] = stream_files.as_slice() else {
            return Err(ArchiveExtractionError::InvalidCompressedTar {
                archive: archive.to_path_buf(),
                message: format!(
                    "the GZip stream produced {} files instead of one",
                    stream_files.len()
                ),
            });
        };
        if tar_path.file_name() != Some(expected_tar_name.as_os_str()) {
            return Err(ArchiveExtractionError::InvalidCompressedTar {
                archive: archive.to_path_buf(),
                message: format!(
                    "the GZip stream produced unexpected file {}",
                    tar_path.display()
                ),
            });
        }
        let actual_size = fs::symlink_metadata(tar_path)
            .map_err(|error| ArchiveExtractionError::InvalidCompressedTar {
                archive: archive.to_path_buf(),
                message: format!("could not inspect extracted tar: {error}"),
            })?
            .len();
        if actual_size != unpacked_size {
            return Err(ArchiveExtractionError::InvalidCompressedTar {
                archive: archive.to_path_buf(),
                message: format!(
                    "the extracted tar size is {actual_size}, expected {unpacked_size}"
                ),
            });
        }

        self.extract_to_directory(tar_path, &destination)
    }

    pub(crate) fn extract(
        &self,
        archive: &Path,
    ) -> Result<PreparedArchive, ArchiveExtractionError> {
        if !archive.is_file() {
            return Err(ArchiveExtractionError::ArchiveNotFound {
                archive: archive.to_path_buf(),
            });
        }

        let entries = self.list_entries(archive)?;
        validate_entries(archive, &entries)?;

        let temporary_directory =
            Builder::new()
                .prefix("launchbox-port-")
                .tempdir()
                .map_err(|error| ArchiveExtractionError::TemporaryDirectory {
                    archive: archive.to_path_buf(),
                    message: error.to_string(),
                })?;
        let archive_stem = archive
            .file_stem()
            .filter(|stem| !stem.is_empty())
            .ok_or_else(|| ArchiveExtractionError::MissingArchiveFileName {
                archive: archive.to_path_buf(),
            })?;
        let extraction_root = temporary_directory.path().join(archive_stem);
        fs::create_dir(&extraction_root).map_err(|error| {
            ArchiveExtractionError::TemporaryDirectory {
                archive: archive.to_path_buf(),
                message: error.to_string(),
            }
        })?;

        let mut output_directory_argument = OsString::from("-o");
        output_directory_argument.push(&extraction_root);
        let output = Command::new(&self.executable)
            .arg("x")
            .arg("-y")
            .arg("-bd")
            .arg("-bb0")
            .arg(output_directory_argument)
            .arg("--")
            .arg(archive)
            .stdin(Stdio::null())
            .output()
            .map_err(|error| ArchiveExtractionError::ToolStart {
                executable: self.executable.clone(),
                operation: "extract",
                archive: archive.to_path_buf(),
                message: error.to_string(),
            })?;
        if !output.status.success() {
            return Err(ArchiveExtractionError::ToolFailed {
                executable: self.executable.clone(),
                operation: "extract",
                archive: archive.to_path_buf(),
                message: tool_output_message(&output.stdout, &output.stderr),
            });
        }

        let files = audit_extracted_tree(archive, &extraction_root)?;
        let launch_path = select_launch_file(archive, &files)?;
        Ok(PreparedArchive {
            launch_path,
            lease: LaunchResourceLease {
                inner: Arc::new(TemporaryLaunchResource {
                    directory: temporary_directory,
                }),
            },
        })
    }

    fn list_entries(&self, archive: &Path) -> Result<Vec<ArchiveEntry>, ArchiveExtractionError> {
        let output = Command::new(&self.executable)
            .arg("l")
            .arg("-slt")
            .arg("-sccUTF-8")
            .arg("-ba")
            .arg("--")
            .arg(archive)
            .stdin(Stdio::null())
            .output()
            .map_err(|error| ArchiveExtractionError::ToolStart {
                executable: self.executable.clone(),
                operation: "list",
                archive: archive.to_path_buf(),
                message: error.to_string(),
            })?;
        if !output.status.success() {
            return Err(ArchiveExtractionError::ToolFailed {
                executable: self.executable.clone(),
                operation: "list",
                archive: archive.to_path_buf(),
                message: tool_output_message(&output.stdout, &output.stderr),
            });
        }
        let listing = String::from_utf8(output.stdout).map_err(|error| {
            ArchiveExtractionError::InvalidToolOutput {
                archive: archive.to_path_buf(),
                message: error.to_string(),
            }
        })?;
        let entries = parse_technical_listing(&listing);
        if entries.is_empty() {
            return Err(ArchiveExtractionError::EmptyArchive {
                archive: archive.to_path_buf(),
            });
        }
        Ok(entries)
    }
}

impl Default for ArchiveExtractor {
    fn default() -> Self {
        Self::new("7z")
    }
}

pub(crate) struct PreparedArchive {
    pub(crate) launch_path: PathBuf,
    pub(crate) lease: LaunchResourceLease,
}

#[derive(Clone)]
pub struct LaunchResourceLease {
    inner: Arc<TemporaryLaunchResource>,
}

impl LaunchResourceLease {
    pub fn path(&self) -> &Path {
        self.inner.directory.path()
    }

    pub(crate) fn temporary(prefix: &str) -> Result<Self, std::io::Error> {
        Ok(Self {
            inner: Arc::new(TemporaryLaunchResource {
                directory: Builder::new().prefix(prefix).tempdir()?,
            }),
        })
    }
}

impl fmt::Debug for LaunchResourceLease {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LaunchResourceLease")
            .field("path", &self.path())
            .finish()
    }
}

impl PartialEq for LaunchResourceLease {
    fn eq(&self, other: &Self) -> bool {
        self.path() == other.path()
    }
}

impl Eq for LaunchResourceLease {}

struct TemporaryLaunchResource {
    directory: TempDir,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ArchiveEntry {
    path: String,
    encrypted: bool,
    size: Option<u64>,
    directory: bool,
}

fn parse_technical_listing(listing: &str) -> Vec<ArchiveEntry> {
    let mut entries = Vec::new();
    let mut path = None;
    let mut encrypted = false;
    let mut size = None;
    let mut directory = false;
    for raw_line in listing.lines().chain(std::iter::once("")) {
        let line = raw_line.trim_end_matches('\r');
        if line.is_empty() {
            if let Some(path) = path.take() {
                entries.push(ArchiveEntry {
                    path,
                    encrypted,
                    size,
                    directory,
                });
            }
            encrypted = false;
            size = None;
            directory = false;
            continue;
        }
        let Some((name, value)) = line.split_once(" = ") else {
            continue;
        };
        match name {
            "Path" => path = Some(value.to_string()),
            "Encrypted" => encrypted = value == "+",
            "Size" => size = value.parse().ok(),
            "Folder" => directory = value == "+",
            "Attributes" => directory |= value.starts_with('D'),
            _ => {}
        }
    }
    entries
}

fn collect_archive_tree(
    root: &Path,
    directory: &Path,
    top_level: &mut Vec<(String, OsString)>,
    expected: &mut BTreeSet<String>,
) -> Result<(), ArchiveCreationError> {
    let mut entries = fs::read_dir(directory)
        .map_err(|source| ArchiveCreationError::Io {
            path: directory.to_path_buf(),
            source,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| ArchiveCreationError::Io {
            path: directory.to_path_buf(),
            source,
        })?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let name = entry.file_name();
        let name_text = name
            .to_str()
            .filter(|name| is_portable_archive_component(name))
            .ok_or_else(|| ArchiveCreationError::UnsafeSourceName { path: path.clone() })?
            .to_string();
        let relative = path
            .strip_prefix(root)
            .map_err(|_| ArchiveCreationError::UnsafeSourceName { path: path.clone() })?;
        if relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err(ArchiveCreationError::UnsafeSourceName { path });
        }
        let portable = relative
            .to_str()
            .ok_or_else(|| ArchiveCreationError::UnsafeSourceName { path: path.clone() })?
            .replace('\\', "/");
        let identity = portable.to_ascii_lowercase();
        if !expected.insert(identity) {
            return Err(ArchiveCreationError::DuplicateSourceName { name: portable });
        }
        let metadata = fs::symlink_metadata(&path).map_err(|source| ArchiveCreationError::Io {
            path: path.clone(),
            source,
        })?;
        if directory == root {
            top_level.push((name_text, name));
        }
        if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
            collect_archive_tree(root, &path, top_level, expected)?;
        } else if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
            return Err(ArchiveCreationError::UnsupportedSourceEntry { path });
        }
    }
    Ok(())
}

fn empty_real_directory(
    archive: &Path,
    directory: &Path,
    label: &str,
) -> Result<PathBuf, ArchiveExtractionError> {
    let metadata =
        fs::symlink_metadata(directory).map_err(|error| ArchiveExtractionError::ExtractedTree {
            archive: archive.to_path_buf(),
            path: directory.to_path_buf(),
            message: format!("{label} directory cannot be inspected: {error}"),
        })?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(ArchiveExtractionError::ExtractedTree {
            archive: archive.to_path_buf(),
            path: directory.to_path_buf(),
            message: format!("{label} destination is not a real directory"),
        });
    }
    let directory =
        fs::canonicalize(directory).map_err(|error| ArchiveExtractionError::ExtractedTree {
            archive: archive.to_path_buf(),
            path: directory.to_path_buf(),
            message: format!("{label} directory cannot be resolved: {error}"),
        })?;
    let mut entries =
        fs::read_dir(&directory).map_err(|error| ArchiveExtractionError::ExtractedTree {
            archive: archive.to_path_buf(),
            path: directory.clone(),
            message: format!("{label} directory cannot be read: {error}"),
        })?;
    if entries.next().is_some() {
        return Err(ArchiveExtractionError::ExtractedTree {
            archive: archive.to_path_buf(),
            path: directory,
            message: format!("{label} destination is not empty"),
        });
    }
    Ok(directory)
}

fn validate_entries(
    archive: &Path,
    entries: &[ArchiveEntry],
) -> Result<(), ArchiveExtractionError> {
    let mut identities = BTreeSet::new();
    for entry in entries {
        if entry.encrypted {
            return Err(ArchiveExtractionError::EncryptedEntry {
                archive: archive.to_path_buf(),
                entry: entry.path.clone(),
            });
        }
        if !is_safe_archive_member(&entry.path) {
            return Err(ArchiveExtractionError::UnsafeEntry {
                archive: archive.to_path_buf(),
                entry: entry.path.clone(),
            });
        }
        let identity = entry.path.replace('\\', "/").to_ascii_lowercase();
        if !identities.insert(identity) {
            return Err(ArchiveExtractionError::DuplicateEntry {
                archive: archive.to_path_buf(),
                entry: entry.path.clone(),
            });
        }
    }
    Ok(())
}

fn validate_entry_limits(
    archive: &Path,
    entries: &[ArchiveEntry],
    limits: ArchiveExtractionLimits,
) -> Result<(), ArchiveExtractionError> {
    if entries.len() > limits.max_entries {
        return Err(ArchiveExtractionError::EntryLimit {
            archive: archive.to_path_buf(),
            actual: entries.len(),
            limit: limits.max_entries,
        });
    }
    let mut total_bytes = 0_u64;
    for entry in entries {
        if entry.directory {
            continue;
        }
        let size = entry
            .size
            .ok_or_else(|| ArchiveExtractionError::MissingEntrySize {
                archive: archive.to_path_buf(),
                entry: entry.path.clone(),
            })?;
        if size > limits.max_member_bytes {
            return Err(ArchiveExtractionError::MemberSizeLimit {
                archive: archive.to_path_buf(),
                entry: entry.path.clone(),
                actual: size,
                limit: limits.max_member_bytes,
            });
        }
        total_bytes = total_bytes.checked_add(size).ok_or_else(|| {
            ArchiveExtractionError::TotalSizeLimit {
                archive: archive.to_path_buf(),
                actual: u64::MAX,
                limit: limits.max_total_bytes,
            }
        })?;
        if total_bytes > limits.max_total_bytes {
            return Err(ArchiveExtractionError::TotalSizeLimit {
                archive: archive.to_path_buf(),
                actual: total_bytes,
                limit: limits.max_total_bytes,
            });
        }
    }
    Ok(())
}

fn validate_extracted_limits(
    archive: &Path,
    files: &[PathBuf],
    limits: ArchiveExtractionLimits,
) -> Result<(), ArchiveExtractionError> {
    if files.len() > limits.max_entries {
        return Err(ArchiveExtractionError::EntryLimit {
            archive: archive.to_path_buf(),
            actual: files.len(),
            limit: limits.max_entries,
        });
    }
    let mut total_bytes = 0_u64;
    for path in files {
        let size = fs::symlink_metadata(path)
            .map_err(|error| ArchiveExtractionError::ExtractedTree {
                archive: archive.to_path_buf(),
                path: path.clone(),
                message: error.to_string(),
            })?
            .len();
        if size > limits.max_member_bytes {
            return Err(ArchiveExtractionError::MemberSizeLimit {
                archive: archive.to_path_buf(),
                entry: path.display().to_string(),
                actual: size,
                limit: limits.max_member_bytes,
            });
        }
        total_bytes = total_bytes.checked_add(size).ok_or_else(|| {
            ArchiveExtractionError::TotalSizeLimit {
                archive: archive.to_path_buf(),
                actual: u64::MAX,
                limit: limits.max_total_bytes,
            }
        })?;
        if total_bytes > limits.max_total_bytes {
            return Err(ArchiveExtractionError::TotalSizeLimit {
                archive: archive.to_path_buf(),
                actual: total_bytes,
                limit: limits.max_total_bytes,
            });
        }
    }
    Ok(())
}

fn is_safe_archive_member(path: &str) -> bool {
    if path.is_empty()
        || path.starts_with(['/', '\\'])
        || path
            .as_bytes()
            .get(1)
            .is_some_and(|separator| *separator == b':')
    {
        return false;
    }

    path.split(['/', '\\'])
        .all(|component| !component.is_empty() && component != "." && component != "..")
}

fn is_portable_archive_component(component: &str) -> bool {
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
        .map_or(component, |(stem, _extension)| stem)
        .to_ascii_lowercase();
    !matches!(stem.as_str(), "con" | "prn" | "aux" | "nul")
        && !(stem.len() == 4
            && (stem.starts_with("com") || stem.starts_with("lpt"))
            && matches!(stem.as_bytes()[3], b'1'..=b'9'))
}

fn audit_extracted_tree(
    archive: &Path,
    extraction_root: &Path,
) -> Result<Vec<PathBuf>, ArchiveExtractionError> {
    fn visit(
        archive: &Path,
        root: &Path,
        directory: &Path,
        files: &mut Vec<PathBuf>,
    ) -> Result<(), ArchiveExtractionError> {
        let entries =
            fs::read_dir(directory).map_err(|error| ArchiveExtractionError::ExtractedTree {
                archive: archive.to_path_buf(),
                path: directory.to_path_buf(),
                message: error.to_string(),
            })?;
        for entry in entries {
            let entry = entry.map_err(|error| ArchiveExtractionError::ExtractedTree {
                archive: archive.to_path_buf(),
                path: directory.to_path_buf(),
                message: error.to_string(),
            })?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).map_err(|error| {
                ArchiveExtractionError::ExtractedTree {
                    archive: archive.to_path_buf(),
                    path: path.clone(),
                    message: error.to_string(),
                }
            })?;
            if metadata.file_type().is_symlink() {
                return Err(ArchiveExtractionError::ExtractedLink {
                    archive: archive.to_path_buf(),
                    path,
                });
            }
            if metadata.is_dir() {
                visit(archive, root, &path, files)?;
            } else if metadata.is_file() {
                let relative = path.strip_prefix(root).map_err(|error| {
                    ArchiveExtractionError::ExtractedTree {
                        archive: archive.to_path_buf(),
                        path: path.clone(),
                        message: error.to_string(),
                    }
                })?;
                if relative
                    .components()
                    .any(|component| !matches!(component, Component::Normal(_)))
                {
                    return Err(ArchiveExtractionError::ExtractedTree {
                        archive: archive.to_path_buf(),
                        path,
                        message: "extracted file is not below its private extraction root".into(),
                    });
                }
                files.push(path);
            } else {
                return Err(ArchiveExtractionError::ExtractedSpecialFile {
                    archive: archive.to_path_buf(),
                    path,
                });
            }
        }
        Ok(())
    }

    let mut files = Vec::new();
    visit(archive, extraction_root, extraction_root, &mut files)?;
    files.sort();
    if files.is_empty() {
        return Err(ArchiveExtractionError::EmptyArchive {
            archive: archive.to_path_buf(),
        });
    }
    Ok(files)
}

fn select_launch_file(
    archive: &Path,
    files: &[PathBuf],
) -> Result<PathBuf, ArchiveExtractionError> {
    let archive_stem = archive
        .file_stem()
        .and_then(OsStr::to_str)
        .unwrap_or_default();
    let viable = files
        .iter()
        .filter(|path| !is_auxiliary_file(path))
        .collect::<Vec<_>>();

    let matching_stem = viable
        .iter()
        .copied()
        .filter(|path| {
            path.file_stem()
                .and_then(OsStr::to_str)
                .is_some_and(|stem| stem.eq_ignore_ascii_case(archive_stem))
        })
        .collect::<Vec<_>>();
    if let [path] = matching_stem.as_slice() {
        return Ok((*path).clone());
    }

    for extension in ["m3u", "cue", "gdi", "ccd"] {
        let matching_extension = viable
            .iter()
            .copied()
            .filter(|path| has_extension(path, extension))
            .collect::<Vec<_>>();
        if let [path] = matching_extension.as_slice() {
            return Ok((*path).clone());
        }
        if matching_extension.len() > 1 {
            break;
        }
    }

    if let [path] = viable.as_slice() {
        return Ok((*path).clone());
    }

    Err(ArchiveExtractionError::AmbiguousLaunchFile {
        archive: archive.to_path_buf(),
        candidates: viable
            .iter()
            .map(|path| (*path).clone())
            .collect::<Vec<_>>(),
    })
}

fn is_auxiliary_file(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "txt"
                    | "nfo"
                    | "md"
                    | "pdf"
                    | "jpg"
                    | "jpeg"
                    | "png"
                    | "gif"
                    | "xml"
                    | "dat"
                    | "sha1"
                    | "sha256"
                    | "md5"
                    | "sfv"
            )
        })
}

fn has_extension(path: &Path, expected: &str) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| extension.eq_ignore_ascii_case(expected))
}

fn tool_output_message(stdout: &[u8], stderr: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr);
    let stdout = String::from_utf8_lossy(stdout);
    let message = if stderr.trim().is_empty() {
        stdout.trim()
    } else {
        stderr.trim()
    };
    if message.is_empty() {
        "7-Zip exited unsuccessfully without diagnostics".into()
    } else {
        message.to_string()
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ArchiveExtractionError {
    #[error("archive does not exist or is not a regular file: {archive}")]
    ArchiveNotFound { archive: PathBuf },
    #[error("archive has no usable file name: {archive}")]
    MissingArchiveFileName { archive: PathBuf },
    #[error("could not create a temporary directory for {archive}: {message}")]
    TemporaryDirectory { archive: PathBuf, message: String },
    #[error("could not start {executable} to {operation} {archive}: {message}")]
    ToolStart {
        executable: PathBuf,
        operation: &'static str,
        archive: PathBuf,
        message: String,
    },
    #[error("{executable} could not {operation} {archive}: {message}")]
    ToolFailed {
        executable: PathBuf,
        operation: &'static str,
        archive: PathBuf,
        message: String,
    },
    #[error("7-Zip returned non-UTF-8 technical output for {archive}: {message}")]
    InvalidToolOutput { archive: PathBuf, message: String },
    #[error("archive is empty: {archive}")]
    EmptyArchive { archive: PathBuf },
    #[error("archive {archive} is not a valid single-stream .tar.xz payload: {message}")]
    InvalidCompressedTar { archive: PathBuf, message: String },
    #[error(
        "archive {archive} expands to {unpacked_size} bytes, above the {max_tar_bytes}-byte tar limit"
    )]
    CompressedTarTooLarge {
        archive: PathBuf,
        unpacked_size: u64,
        max_tar_bytes: u64,
    },
    #[error("encrypted archive member is unsupported in {archive}: {entry}")]
    EncryptedEntry { archive: PathBuf, entry: String },
    #[error("archive {archive} contains a case-insensitive duplicate member: {entry}")]
    DuplicateEntry { archive: PathBuf, entry: String },
    #[error("unsafe archive member path in {archive}: {entry}")]
    UnsafeEntry { archive: PathBuf, entry: String },
    #[error("archive {archive} contains {actual} entries, above the {limit}-entry limit")]
    EntryLimit {
        archive: PathBuf,
        actual: usize,
        limit: usize,
    },
    #[error("archive {archive} does not declare a size for regular member {entry}")]
    MissingEntrySize { archive: PathBuf, entry: String },
    #[error(
        "archive member {entry} in {archive} is {actual} bytes, above the {limit}-byte member limit"
    )]
    MemberSizeLimit {
        archive: PathBuf,
        entry: String,
        actual: u64,
        limit: u64,
    },
    #[error(
        "archive {archive} expands to at least {actual} bytes, above the {limit}-byte total limit"
    )]
    TotalSizeLimit {
        archive: PathBuf,
        actual: u64,
        limit: u64,
    },
    #[error("archive {archive} extracted a symbolic link: {path}")]
    ExtractedLink { archive: PathBuf, path: PathBuf },
    #[error("archive {archive} extracted an unsupported special file: {path}")]
    ExtractedSpecialFile { archive: PathBuf, path: PathBuf },
    #[error("could not audit extracted archive {archive} at {path}: {message}")]
    ExtractedTree {
        archive: PathBuf,
        path: PathBuf,
        message: String,
    },
    #[error("archive {archive} has no unambiguous launch file; candidates: {candidates:?}")]
    AmbiguousLaunchFile {
        archive: PathBuf,
        candidates: Vec<PathBuf>,
    },
}

#[derive(Debug, Error)]
pub enum ArchiveCreationError {
    #[error("archive source is not a real directory: {path}")]
    SourceNotDirectory { path: PathBuf },
    #[error("archive target must be a named .7z file below an existing directory: {path}")]
    InvalidTarget { path: PathBuf },
    #[error("archive target already exists: {path}")]
    TargetExists { path: PathBuf },
    #[error("archive source contains an unsupported entry: {path}")]
    UnsupportedSourceEntry { path: PathBuf },
    #[error("archive source contains an unsafe or non-Unicode filename: {path}")]
    UnsafeSourceName { path: PathBuf },
    #[error("archive source has a case-insensitive duplicate filename: {name}")]
    DuplicateSourceName { name: String },
    #[error("archive source contains no files: {path}")]
    EmptySource { path: PathBuf },
    #[error("could not read or write {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not start {executable} to create {archive}: {message}")]
    ToolStart {
        executable: PathBuf,
        archive: PathBuf,
        message: String,
    },
    #[error("{executable} could not create {archive}: {message}")]
    ToolFailed {
        executable: PathBuf,
        archive: PathBuf,
        message: String,
    },
    #[error("7-Zip did not create a real archive file: {path}")]
    InvalidCreatedArchive { path: PathBuf },
    #[error("could not verify created archive {archive}: {source}")]
    Verification {
        archive: PathBuf,
        #[source]
        source: Box<ArchiveExtractionError>,
    },
    #[error(
        "created archive {archive} has different members; expected {expected:?}, found {actual:?}"
    )]
    MemberMismatch {
        archive: PathBuf,
        expected: Vec<String>,
        actual: Vec<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_7zip_technical_records() {
        let entries = parse_technical_listing(
            "Path = Disc/Game.cue\r\nSize = 42\r\nFolder = -\r\nEncrypted = -\r\n\r\nPath = Disc/track.bin\r\nSize = 1024\r\nFolder = -\r\nEncrypted = +\r\n",
        );
        assert_eq!(
            entries,
            [
                ArchiveEntry {
                    path: "Disc/Game.cue".into(),
                    encrypted: false,
                    size: Some(42),
                    directory: false,
                },
                ArchiveEntry {
                    path: "Disc/track.bin".into(),
                    encrypted: true,
                    size: Some(1024),
                    directory: false,
                },
            ]
        );
    }

    #[test]
    fn bounded_manifest_rejects_duplicates_and_oversized_members_before_extraction() {
        let archive = Path::new("/tmp/data.7z");
        let duplicate = [
            ArchiveEntry {
                path: "Platforms/Arcade.xml".into(),
                encrypted: false,
                size: Some(10),
                directory: false,
            },
            ArchiveEntry {
                path: "platforms/ARCADE.XML".into(),
                encrypted: false,
                size: Some(10),
                directory: false,
            },
        ];
        assert!(matches!(
            validate_entries(archive, &duplicate),
            Err(ArchiveExtractionError::DuplicateEntry { .. })
        ));

        let oversized = [ArchiveEntry {
            path: "Platforms/Arcade.xml".into(),
            encrypted: false,
            size: Some(101),
            directory: false,
        }];
        assert!(matches!(
            validate_entry_limits(
                archive,
                &oversized,
                ArchiveExtractionLimits {
                    max_entries: 10,
                    max_member_bytes: 100,
                    max_total_bytes: 1_000,
                },
            ),
            Err(ArchiveExtractionError::MemberSizeLimit { actual: 101, .. })
        ));
    }

    #[test]
    fn rejects_cross_platform_absolute_and_parent_paths() {
        for path in [
            "../escape.rom",
            "folder/../../escape.rom",
            "/absolute.rom",
            r"C:\absolute.rom",
            r"\\server\share\escape.rom",
            r"folder\..\escape.rom",
        ] {
            assert!(!is_safe_archive_member(path), "accepted {path:?}");
        }
        assert!(is_safe_archive_member("Disc One/Game.cue"));
        assert!(is_safe_archive_member(r"Disc One\track.bin"));
    }

    #[test]
    fn launch_selection_prefers_archive_stem_and_disc_descriptors() {
        let files = [
            PathBuf::from("/tmp/Game/readme.txt"),
            PathBuf::from("/tmp/Game/Game.rom"),
            PathBuf::from("/tmp/Game/other.bin"),
        ];
        assert_eq!(
            select_launch_file(Path::new("/library/Game.zip"), &files),
            Ok(PathBuf::from("/tmp/Game/Game.rom"))
        );

        let disc_files = [
            PathBuf::from("/tmp/Disc/track01.bin"),
            PathBuf::from("/tmp/Disc/layout.cue"),
        ];
        assert_eq!(
            select_launch_file(Path::new("/library/Disc.7z"), &disc_files),
            Ok(PathBuf::from("/tmp/Disc/layout.cue"))
        );
    }

    #[test]
    fn launch_selection_refuses_to_guess_between_roms() {
        let files = [
            PathBuf::from("/tmp/Collection/one.rom"),
            PathBuf::from("/tmp/Collection/two.rom"),
        ];
        assert!(matches!(
            select_launch_file(Path::new("/library/Collection.rar"), &files),
            Err(ArchiveExtractionError::AmbiguousLaunchFile { .. })
        ));
    }

    #[test]
    fn creates_and_verifies_flat_7z_archives_without_a_shell() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("member");
        fs::create_dir(&source).unwrap();
        fs::write(source.join("icon.sys"), b"icon bytes").unwrap();
        fs::write(source.join("save.bin"), b"save bytes").unwrap();
        let archive = directory.path().join("member.7z");

        let tool = ArchiveExtractor::new("7z");
        tool.create_7z_from_directory(&source, &archive).unwrap();

        let entries = tool.list_entries(&archive).unwrap();
        assert_eq!(
            entries
                .into_iter()
                .map(|entry| entry.path)
                .collect::<BTreeSet<_>>(),
            BTreeSet::from(["icon.sys".into(), "save.bin".into()])
        );
        assert!(matches!(
            tool.create_7z_from_directory(&source, &archive),
            Err(ArchiveCreationError::TargetExists { .. })
        ));
        let extracted = directory.path().join("extracted");
        fs::create_dir(&extracted).unwrap();
        let files = tool.extract_to_directory(&archive, &extracted).unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(fs::read(extracted.join("icon.sys")).unwrap(), b"icon bytes");
        assert_eq!(fs::read(extracted.join("save.bin")).unwrap(), b"save bytes");
    }

    #[test]
    fn creates_and_verifies_nested_7z_trees_without_a_shell() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("data");
        fs::create_dir_all(source.join("slot/nested")).unwrap();
        fs::create_dir(source.join("empty")).unwrap();
        fs::write(source.join("settings.dat"), b"root settings").unwrap();
        fs::write(source.join("slot/nested/progress.bin"), b"nested progress").unwrap();
        let archive = directory.path().join("wii-save.7z");
        let tool = ArchiveExtractor::new("7z");
        tool.create_7z_from_tree(&source, &archive).unwrap();

        let entries = tool
            .list_entries(&archive)
            .unwrap()
            .into_iter()
            .map(|entry| entry.path)
            .collect::<BTreeSet<_>>();
        assert_eq!(
            entries,
            BTreeSet::from([
                "empty".into(),
                "settings.dat".into(),
                "slot".into(),
                "slot/nested".into(),
                "slot/nested/progress.bin".into(),
            ])
        );
        let extracted = directory.path().join("extracted-tree");
        fs::create_dir(&extracted).unwrap();
        let files = tool.extract_to_directory(&archive, &extracted).unwrap();
        assert_eq!(files.len(), 2);
        assert!(extracted.join("empty").is_dir());
        assert_eq!(
            fs::read(extracted.join("slot/nested/progress.bin")).unwrap(),
            b"nested progress"
        );

        fs::write(source.join("CON.txt"), b"not portable to Windows").unwrap();
        assert!(matches!(
            tool.create_7z_from_tree(&source, &directory.path().join("reserved.7z")),
            Err(ArchiveCreationError::UnsafeSourceName { .. })
        ));
        fs::remove_file(source.join("CON.txt")).unwrap();

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(source.join("settings.dat"), source.join("unsafe-link"))
                .unwrap();
            assert!(matches!(
                tool.create_7z_from_tree(&source, &directory.path().join("unsafe.7z")),
                Err(ArchiveCreationError::UnsupportedSourceEntry { .. })
            ));
        }
    }

    #[test]
    fn extracts_bounded_tar_xz_through_two_audited_stages() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("source");
        let bundle = source.join("Fixture.app");
        fs::create_dir_all(bundle.join("Contents/MacOS")).unwrap();
        fs::create_dir_all(bundle.join("Contents/Resources")).unwrap();
        fs::write(bundle.join("Contents/MacOS/Fixture"), b"fixture executable").unwrap();
        fs::write(bundle.join("Contents/Resources/default.ini"), b"setting=1").unwrap();

        let tar = directory.path().join("fixture.tar");
        let tar_output = Command::new("7z")
            .current_dir(&source)
            .arg("a")
            .arg("-ttar")
            .arg("-bd")
            .arg("-bb0")
            .arg("-y")
            .arg(&tar)
            .arg("--")
            .arg("Fixture.app")
            .output()
            .unwrap();
        assert!(
            tar_output.status.success(),
            "{}",
            tool_output_message(&tar_output.stdout, &tar_output.stderr)
        );
        let archive = directory.path().join("fixture.tar.xz");
        let xz_output = Command::new("7z")
            .current_dir(directory.path())
            .arg("a")
            .arg("-txz")
            .arg("-bd")
            .arg("-bb0")
            .arg("-y")
            .arg(&archive)
            .arg("--")
            .arg(tar.file_name().unwrap())
            .output()
            .unwrap();
        assert!(
            xz_output.status.success(),
            "{}",
            tool_output_message(&xz_output.stdout, &xz_output.stderr)
        );
        fs::remove_file(&tar).unwrap();

        let rejected_stream = directory.path().join("rejected-stream");
        let rejected_destination = directory.path().join("rejected-destination");
        fs::create_dir(&rejected_stream).unwrap();
        fs::create_dir(&rejected_destination).unwrap();
        let extractor = ArchiveExtractor::new("7z");
        assert!(matches!(
            extractor.extract_tar_xz_to_directory(
                &archive,
                &rejected_stream,
                &rejected_destination,
                1
            ),
            Err(ArchiveExtractionError::CompressedTarTooLarge { .. })
        ));
        assert_eq!(fs::read_dir(&rejected_stream).unwrap().count(), 0);
        assert_eq!(fs::read_dir(&rejected_destination).unwrap().count(), 0);

        let stream = directory.path().join("stream");
        let destination = directory.path().join("destination");
        fs::create_dir(&stream).unwrap();
        fs::create_dir(&destination).unwrap();
        let files = extractor
            .extract_tar_xz_to_directory(&archive, &stream, &destination, 1024 * 1024)
            .unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(
            fs::read(destination.join("Fixture.app/Contents/MacOS/Fixture")).unwrap(),
            b"fixture executable"
        );
        assert_eq!(
            fs::read(destination.join("Fixture.app/Contents/Resources/default.ini")).unwrap(),
            b"setting=1"
        );
        assert_eq!(
            fs::read_dir(&stream)
                .unwrap()
                .map(|entry| entry.unwrap().file_name())
                .collect::<Vec<_>>(),
            [tar.file_name().unwrap().to_os_string()]
        );

        let invalid = directory.path().join("invalid.tar.xz");
        fs::write(&invalid, b"not xz").unwrap();
        let invalid_stream = directory.path().join("invalid-stream");
        let invalid_destination = directory.path().join("invalid-destination");
        fs::create_dir(&invalid_stream).unwrap();
        fs::create_dir(&invalid_destination).unwrap();
        assert!(matches!(
            extractor.extract_tar_xz_to_directory(
                &invalid,
                &invalid_stream,
                &invalid_destination,
                1024
            ),
            Err(ArchiveExtractionError::InvalidCompressedTar { .. })
        ));
    }

    #[test]
    fn extracts_bounded_tar_gz_through_two_audited_stages() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("source");
        let payload = source.join("bigpemu");
        fs::create_dir_all(payload.join("Data")).unwrap();
        fs::write(payload.join("bigpemu"), b"fixture executable").unwrap();
        fs::write(payload.join("Data/default.ini"), b"setting=1").unwrap();

        let tar = directory.path().join("BigPEmu_Linux64_v1221.tar");
        let tar_output = Command::new("7z")
            .current_dir(&source)
            .arg("a")
            .arg("-ttar")
            .arg("-bd")
            .arg("-bb0")
            .arg("-y")
            .arg(&tar)
            .arg("--")
            .arg("bigpemu")
            .output()
            .unwrap();
        assert!(
            tar_output.status.success(),
            "{}",
            tool_output_message(&tar_output.stdout, &tar_output.stderr)
        );
        let archive = directory.path().join("BigPEmu_Linux64_v1221.tar.gz");
        let gzip_output = Command::new("7z")
            .current_dir(directory.path())
            .arg("a")
            .arg("-tgzip")
            .arg("-bd")
            .arg("-bb0")
            .arg("-y")
            .arg(&archive)
            .arg("--")
            .arg(tar.file_name().unwrap())
            .output()
            .unwrap();
        assert!(
            gzip_output.status.success(),
            "{}",
            tool_output_message(&gzip_output.stdout, &gzip_output.stderr)
        );
        fs::remove_file(&tar).unwrap();

        let rejected_stream = directory.path().join("rejected-stream");
        let rejected_destination = directory.path().join("rejected-destination");
        fs::create_dir(&rejected_stream).unwrap();
        fs::create_dir(&rejected_destination).unwrap();
        let extractor = ArchiveExtractor::new("7z");
        assert!(matches!(
            extractor.extract_tar_gz_to_directory(
                &archive,
                &rejected_stream,
                &rejected_destination,
                1
            ),
            Err(ArchiveExtractionError::CompressedTarTooLarge { .. })
        ));
        assert_eq!(fs::read_dir(&rejected_stream).unwrap().count(), 0);
        assert_eq!(fs::read_dir(&rejected_destination).unwrap().count(), 0);

        let stream = directory.path().join("stream");
        let destination = directory.path().join("destination");
        fs::create_dir(&stream).unwrap();
        fs::create_dir(&destination).unwrap();
        let files = extractor
            .extract_tar_gz_to_directory(&archive, &stream, &destination, 1024 * 1024)
            .unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(
            fs::read(destination.join("bigpemu/bigpemu")).unwrap(),
            b"fixture executable"
        );
        assert_eq!(
            fs::read(destination.join("bigpemu/Data/default.ini")).unwrap(),
            b"setting=1"
        );

        let invalid = directory.path().join("invalid.tar.gz");
        fs::write(&invalid, b"not gzip").unwrap();
        let invalid_stream = directory.path().join("invalid-stream");
        let invalid_destination = directory.path().join("invalid-destination");
        fs::create_dir(&invalid_stream).unwrap();
        fs::create_dir(&invalid_destination).unwrap();
        assert!(matches!(
            extractor.extract_tar_gz_to_directory(
                &invalid,
                &invalid_stream,
                &invalid_destination,
                1024
            ),
            Err(ArchiveExtractionError::InvalidCompressedTar { .. })
        ));
    }
}
