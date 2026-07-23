use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use thiserror::Error;

const MAX_README_BYTES: u64 = 1024 * 1024;

/// Reads BigPEmu's installed version from the sibling `ReadMe.txt` used by
/// both the recovered 13.27 adapter and the current native Linux package.
///
/// This inspection is deliberately passive: the emulator is never started,
/// symlinks are rejected, and the text file has a fixed read limit.
pub fn installed_bigpemu_version(
    executable: &Path,
) -> Result<Option<String>, BigPEmuInspectionError> {
    let application_directory =
        executable
            .parent()
            .ok_or_else(|| BigPEmuInspectionError::MissingApplicationDirectory {
                executable: executable.to_path_buf(),
            })?;
    let readme = find_readme(application_directory)?;
    let Some(readme) = readme else {
        return Ok(None);
    };
    let metadata =
        fs::symlink_metadata(&readme).map_err(|source| BigPEmuInspectionError::Read {
            path: readme.clone(),
            source,
        })?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(BigPEmuInspectionError::UnsafeReadme { path: readme });
    }
    if metadata.len() > MAX_README_BYTES {
        return Err(BigPEmuInspectionError::ReadmeTooLarge {
            path: readme,
            byte_len: metadata.len(),
            max_bytes: MAX_README_BYTES,
        });
    }
    let mut bytes = Vec::with_capacity(usize::try_from(metadata.len()).unwrap_or_default());
    File::open(&readme)
        .and_then(|source| {
            source
                .take(MAX_README_BYTES.saturating_add(1))
                .read_to_end(&mut bytes)
        })
        .map_err(|source| BigPEmuInspectionError::Read {
            path: readme.clone(),
            source,
        })?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_README_BYTES {
        return Err(BigPEmuInspectionError::ReadmeTooLarge {
            path: readme,
            byte_len: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
            max_bytes: MAX_README_BYTES,
        });
    }
    let text =
        std::str::from_utf8(&bytes).map_err(|source| BigPEmuInspectionError::InvalidUtf8 {
            path: readme.clone(),
            source,
        })?;
    let Some(version) = text.lines().find_map(|line| {
        line.trim_end_matches('\r')
            .strip_prefix("Version:")
            .map(str::trim)
    }) else {
        return Ok(None);
    };
    if version.is_empty()
        || version.len() > 64
        || !version.is_ascii()
        || version
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace())
    {
        return Err(BigPEmuInspectionError::InvalidVersion {
            path: readme,
            version: version.to_string(),
        });
    }
    Ok(Some(version.to_string()))
}

fn find_readme(directory: &Path) -> Result<Option<PathBuf>, BigPEmuInspectionError> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(BigPEmuInspectionError::ReadDirectory {
                path: directory.to_path_buf(),
                source,
            });
        }
    };
    let mut matches = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| BigPEmuInspectionError::ReadDirectory {
            path: directory.to_path_buf(),
            source,
        })?;
        if entry
            .file_name()
            .to_str()
            .is_some_and(|name| name.eq_ignore_ascii_case("readme.txt"))
        {
            matches.push(entry.path());
        }
    }
    matches.sort();
    match matches.as_slice() {
        [] => Ok(None),
        [readme] => Ok(Some(readme.clone())),
        _ => Err(BigPEmuInspectionError::AmbiguousReadme {
            directory: directory.to_path_buf(),
        }),
    }
}

#[derive(Debug, Error)]
pub enum BigPEmuInspectionError {
    #[error("BigPEmu executable has no application directory: {executable}")]
    MissingApplicationDirectory { executable: PathBuf },
    #[error("could not inspect BigPEmu application directory {path}: {source}")]
    ReadDirectory { path: PathBuf, source: io::Error },
    #[error("BigPEmu application directory has ambiguous readme.txt entries: {directory}")]
    AmbiguousReadme { directory: PathBuf },
    #[error("BigPEmu readme is not a safe regular file: {path}")]
    UnsafeReadme { path: PathBuf },
    #[error("BigPEmu readme {path} is {byte_len} bytes, above the {max_bytes}-byte limit")]
    ReadmeTooLarge {
        path: PathBuf,
        byte_len: u64,
        max_bytes: u64,
    },
    #[error("could not read BigPEmu readme {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("BigPEmu readme {path} is not valid UTF-8: {source}")]
    InvalidUtf8 {
        path: PathBuf,
        source: std::str::Utf8Error,
    },
    #[error("BigPEmu readme {path} contains an invalid version value: {version:?}")]
    InvalidVersion { path: PathBuf, version: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_current_linux_and_legacy_windows_readme_casing_without_execution() {
        for name in ["ReadMe.txt", "readme.txt"] {
            let directory = tempfile::tempdir().expect("temporary BigPEmu");
            let executable = directory.path().join("bigpemu");
            fs::write(&executable, b"not an executable image").expect("fixture executable");
            fs::write(
                directory.path().join(name),
                b"Title: BigPEmu\r\nVersion: 1.221\r\nAuthor: Rich Whitehouse\r\n",
            )
            .expect("fixture readme");
            assert_eq!(
                installed_bigpemu_version(&executable).expect("version inspection"),
                Some("1.221".into())
            );
            assert_eq!(
                fs::read(&executable).expect("unchanged executable"),
                b"not an executable image"
            );
        }
    }

    #[test]
    fn missing_version_is_not_invented_and_malformed_values_are_rejected() {
        let directory = tempfile::tempdir().expect("temporary BigPEmu");
        let executable = directory.path().join("bigpemu");
        fs::write(directory.path().join("ReadMe.txt"), b"Title: BigPEmu\n")
            .expect("fixture readme");
        assert_eq!(
            installed_bigpemu_version(&executable).expect("missing version"),
            None
        );

        fs::write(
            directory.path().join("ReadMe.txt"),
            b"Version: unsafe value\n",
        )
        .expect("malformed readme");
        assert!(matches!(
            installed_bigpemu_version(&executable),
            Err(BigPEmuInspectionError::InvalidVersion { .. })
        ));
    }

    #[test]
    fn oversized_and_non_utf8_readmes_are_rejected_before_version_parsing() {
        let directory = tempfile::tempdir().expect("temporary BigPEmu");
        let executable = directory.path().join("bigpemu");
        let readme = directory.path().join("ReadMe.txt");
        let oversized = File::create(&readme).expect("oversized readme");
        oversized
            .set_len(MAX_README_BYTES + 1)
            .expect("extend readme");
        assert!(matches!(
            installed_bigpemu_version(&executable),
            Err(BigPEmuInspectionError::ReadmeTooLarge { .. })
        ));

        fs::write(&readme, [0xff, 0xfe]).expect("non-UTF-8 readme");
        assert!(matches!(
            installed_bigpemu_version(&executable),
            Err(BigPEmuInspectionError::InvalidUtf8 { .. })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn readme_symlinks_and_case_ambiguous_files_are_rejected() {
        use std::os::unix::fs::symlink;

        let directory = tempfile::tempdir().expect("temporary BigPEmu");
        let executable = directory.path().join("bigpemu");
        let external = directory.path().join("external.txt");
        fs::write(&external, b"Version: 1.221\n").expect("external readme");
        symlink(&external, directory.path().join("ReadMe.txt")).expect("readme link");
        assert!(matches!(
            installed_bigpemu_version(&executable),
            Err(BigPEmuInspectionError::UnsafeReadme { .. })
        ));

        fs::remove_file(directory.path().join("ReadMe.txt")).expect("remove link");
        fs::write(directory.path().join("ReadMe.txt"), b"Version: 1.221\n").expect("first readme");
        fs::write(directory.path().join("readme.txt"), b"Version: 1.22\n").expect("second readme");
        assert!(matches!(
            installed_bigpemu_version(&executable),
            Err(BigPEmuInspectionError::AmbiguousReadme { .. })
        ));
    }
}
