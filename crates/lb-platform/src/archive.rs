use std::ffi::{OsStr, OsString};
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;

use tempfile::{Builder, TempDir};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveExtractor {
    executable: PathBuf,
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

#[cfg(test)]
pub(crate) fn temporary_launch_resource_for_test() -> LaunchResourceLease {
    LaunchResourceLease::temporary("launchbox-port-resource-test-")
        .expect("create temporary launch resource")
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ArchiveEntry {
    path: String,
    encrypted: bool,
}

fn parse_technical_listing(listing: &str) -> Vec<ArchiveEntry> {
    let mut entries = Vec::new();
    let mut path = None;
    let mut encrypted = false;
    for raw_line in listing.lines().chain(std::iter::once("")) {
        let line = raw_line.trim_end_matches('\r');
        if line.is_empty() {
            if let Some(path) = path.take() {
                entries.push(ArchiveEntry { path, encrypted });
            }
            encrypted = false;
            continue;
        }
        let Some((name, value)) = line.split_once(" = ") else {
            continue;
        };
        match name {
            "Path" => path = Some(value.to_string()),
            "Encrypted" => encrypted = value == "+",
            _ => {}
        }
    }
    entries
}

fn validate_entries(
    archive: &Path,
    entries: &[ArchiveEntry],
) -> Result<(), ArchiveExtractionError> {
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
    #[error("encrypted archive member is unsupported in {archive}: {entry}")]
    EncryptedEntry { archive: PathBuf, entry: String },
    #[error("unsafe archive member path in {archive}: {entry}")]
    UnsafeEntry { archive: PathBuf, entry: String },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_7zip_technical_records() {
        let entries = parse_technical_listing(
            "Path = Disc/Game.cue\r\nFolder = -\r\nEncrypted = -\r\n\r\nPath = Disc/track.bin\r\nFolder = -\r\nEncrypted = +\r\n",
        );
        assert_eq!(
            entries,
            [
                ArchiveEntry {
                    path: "Disc/Game.cue".into(),
                    encrypted: false,
                },
                ArchiveEntry {
                    path: "Disc/track.bin".into(),
                    encrypted: true,
                },
            ]
        );
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
}
