use crate::{HostPathResolver, LaunchPathError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

pub const HOST_PATH_MAPPINGS_VERSION: u32 = 1;

/// Returns the platform-native, port-owned location for host path mappings.
/// LaunchBox XML paths never participate in this decision.
#[cfg(windows)]
pub fn default_host_path_mappings_path() -> Result<PathBuf, HostPathMappingsError> {
    absolute_environment_root("APPDATA")
        .map(|root| root.join("LaunchBox Port").join("path-mappings.json"))
        .ok_or(HostPathMappingsError::DefaultConfigurationRootUnavailable {
            variables: "APPDATA",
        })
}

/// Returns the platform-native, port-owned location for host path mappings.
/// LaunchBox XML paths never participate in this decision.
#[cfg(target_os = "macos")]
pub fn default_host_path_mappings_path() -> Result<PathBuf, HostPathMappingsError> {
    absolute_environment_root("HOME")
        .map(|root| {
            root.join("Library")
                .join("Application Support")
                .join("LaunchBox Port")
                .join("path-mappings.json")
        })
        .ok_or(HostPathMappingsError::DefaultConfigurationRootUnavailable { variables: "HOME" })
}

/// Returns the platform-native, port-owned location for host path mappings.
/// LaunchBox XML paths never participate in this decision.
#[cfg(all(not(windows), not(target_os = "macos")))]
pub fn default_host_path_mappings_path() -> Result<PathBuf, HostPathMappingsError> {
    if let Some(root) = absolute_environment_root("XDG_CONFIG_HOME") {
        return Ok(root.join("launchbox-port").join("path-mappings.json"));
    }
    absolute_environment_root("HOME")
        .map(|root| {
            root.join(".config")
                .join("launchbox-port")
                .join("path-mappings.json")
        })
        .ok_or(HostPathMappingsError::DefaultConfigurationRootUnavailable {
            variables: "XDG_CONFIG_HOME or HOME",
        })
}

fn absolute_environment_root(variable: &str) -> Option<PathBuf> {
    std::env::var_os(variable)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .filter(|root| root.is_absolute())
}

/// Port-owned configuration for translating paths from a Windows LaunchBox
/// library. It intentionally lives outside LaunchBox's XML so that loading the
/// same portable library on another host never rewrites the source paths.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HostPathMappings {
    version: u32,
    #[serde(default)]
    windows_drives: Vec<WindowsDriveMapping>,
    #[serde(default)]
    windows_unc: Vec<WindowsUncMapping>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WindowsDriveMapping {
    drive: char,
    host_root: PathBuf,
}

impl WindowsDriveMapping {
    pub fn drive(&self) -> char {
        self.drive
    }

    pub fn host_root(&self) -> &Path {
        &self.host_root
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WindowsUncMapping {
    server: String,
    share: String,
    host_root: PathBuf,
}

impl WindowsUncMapping {
    pub fn server(&self) -> &str {
        &self.server
    }

    pub fn share(&self) -> &str {
        &self.share
    }

    pub fn host_root(&self) -> &Path {
        &self.host_root
    }
}

impl Default for HostPathMappings {
    fn default() -> Self {
        Self {
            version: HOST_PATH_MAPPINGS_VERSION,
            windows_drives: Vec::new(),
            windows_unc: Vec::new(),
        }
    }
}

impl HostPathMappings {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, HostPathMappingsError> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|source| HostPathMappingsError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let settings = serde_json::from_slice::<Self>(&bytes).map_err(|source| {
            HostPathMappingsError::Parse {
                path: path.to_path_buf(),
                source,
            }
        })?;
        settings.validate()?;
        Ok(settings)
    }

    pub fn load_or_default(path: impl AsRef<Path>) -> Result<Self, HostPathMappingsError> {
        let path = path.as_ref();
        match Self::load(path) {
            Ok(settings) => Ok(settings),
            Err(HostPathMappingsError::Read { source, .. })
                if source.kind() == std::io::ErrorKind::NotFound =>
            {
                Ok(Self::default())
            }
            Err(error) => Err(error),
        }
    }

    pub fn save_atomic(&self, path: impl AsRef<Path>) -> Result<(), HostPathMappingsError> {
        self.validate()?;
        let path = path.as_ref();
        let parent = path.parent().unwrap_or(Path::new("."));
        fs::create_dir_all(parent).map_err(|source| HostPathMappingsError::Write {
            path: parent.to_path_buf(),
            source,
        })?;

        let mut bytes = serde_json::to_vec_pretty(self)
            .map_err(|source| HostPathMappingsError::Serialize { source })?;
        bytes.push(b'\n');
        let (temporary_path, mut temporary) = create_unique_sibling(path)?;
        let result = (|| {
            temporary
                .write_all(&bytes)
                .and_then(|()| temporary.sync_all())
                .map_err(|source| HostPathMappingsError::Write {
                    path: temporary_path.clone(),
                    source,
                })?;
            drop(temporary);
            replace_file(&temporary_path, path).map_err(|source| {
                HostPathMappingsError::Replace {
                    temporary: temporary_path.clone(),
                    target: path.to_path_buf(),
                    source,
                }
            })?;
            sync_parent_directory(path).map_err(|source| HostPathMappingsError::Write {
                path: parent.to_path_buf(),
                source,
            })?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary_path);
        }
        result
    }

    pub fn windows_drives(&self) -> &[WindowsDriveMapping] {
        &self.windows_drives
    }

    pub fn windows_unc(&self) -> &[WindowsUncMapping] {
        &self.windows_unc
    }

    pub fn len(&self) -> usize {
        self.windows_drives.len() + self.windows_unc.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn set_windows_drive(
        &mut self,
        drive: char,
        host_root: impl Into<PathBuf>,
    ) -> Result<(), HostPathMappingsError> {
        let host_root = host_root.into();
        let normalized =
            HostPathResolver::default().with_windows_drive_mapping(drive, host_root.clone())?;
        let _ = normalized;
        let drive = drive.to_ascii_uppercase();
        self.windows_drives.retain(|mapping| mapping.drive != drive);
        self.windows_drives
            .push(WindowsDriveMapping { drive, host_root });
        self.windows_drives.sort_by_key(|mapping| mapping.drive);
        Ok(())
    }

    pub fn remove_windows_drive(&mut self, drive: char) -> bool {
        let drive = drive.to_ascii_uppercase();
        let previous = self.windows_drives.len();
        self.windows_drives.retain(|mapping| mapping.drive != drive);
        self.windows_drives.len() != previous
    }

    pub fn set_windows_unc(
        &mut self,
        server: impl Into<String>,
        share: impl Into<String>,
        host_root: impl Into<PathBuf>,
    ) -> Result<(), HostPathMappingsError> {
        let server = server.into().trim().to_string();
        let share = share.into().trim().to_string();
        let host_root = host_root.into();
        let normalized = HostPathResolver::default().with_windows_unc_mapping(
            server.clone(),
            share.clone(),
            host_root.clone(),
        )?;
        let _ = normalized;
        self.windows_unc.retain(|mapping| {
            !mapping.server.eq_ignore_ascii_case(&server)
                || !mapping.share.eq_ignore_ascii_case(&share)
        });
        self.windows_unc.push(WindowsUncMapping {
            server,
            share,
            host_root,
        });
        self.windows_unc.sort_by(|left, right| {
            left.server
                .to_ascii_lowercase()
                .cmp(&right.server.to_ascii_lowercase())
                .then_with(|| {
                    left.share
                        .to_ascii_lowercase()
                        .cmp(&right.share.to_ascii_lowercase())
                })
        });
        Ok(())
    }

    pub fn remove_windows_unc(&mut self, server: &str, share: &str) -> bool {
        let previous = self.windows_unc.len();
        self.windows_unc.retain(|mapping| {
            !mapping.server.eq_ignore_ascii_case(server)
                || !mapping.share.eq_ignore_ascii_case(share)
        });
        self.windows_unc.len() != previous
    }

    pub fn resolver(&self) -> Result<HostPathResolver, HostPathMappingsError> {
        self.validate()?;
        let mut resolver = HostPathResolver::default();
        for mapping in &self.windows_drives {
            resolver =
                resolver.with_windows_drive_mapping(mapping.drive, mapping.host_root.clone())?;
        }
        for mapping in &self.windows_unc {
            resolver = resolver.with_windows_unc_mapping(
                mapping.server.clone(),
                mapping.share.clone(),
                mapping.host_root.clone(),
            )?;
        }
        Ok(resolver)
    }

    fn validate(&self) -> Result<(), HostPathMappingsError> {
        if self.version != HOST_PATH_MAPPINGS_VERSION {
            return Err(HostPathMappingsError::UnsupportedVersion {
                version: self.version,
            });
        }
        let mut drives = Vec::with_capacity(self.windows_drives.len());
        for mapping in &self.windows_drives {
            HostPathResolver::default()
                .with_windows_drive_mapping(mapping.drive, mapping.host_root.clone())?;
            let drive = mapping.drive.to_ascii_uppercase();
            if drives.contains(&drive) {
                return Err(HostPathMappingsError::DuplicateWindowsDrive { drive });
            }
            drives.push(drive);
        }
        let mut unc_roots = Vec::<(String, String)>::with_capacity(self.windows_unc.len());
        for mapping in &self.windows_unc {
            HostPathResolver::default().with_windows_unc_mapping(
                mapping.server.clone(),
                mapping.share.clone(),
                mapping.host_root.clone(),
            )?;
            let key = (
                mapping.server.to_ascii_lowercase(),
                mapping.share.to_ascii_lowercase(),
            );
            if unc_roots.contains(&key) {
                return Err(HostPathMappingsError::DuplicateWindowsUnc {
                    server: mapping.server.clone(),
                    share: mapping.share.clone(),
                });
            }
            unc_roots.push(key);
        }
        Ok(())
    }
}

fn create_unique_sibling(target: &Path) -> Result<(PathBuf, fs::File), HostPathMappingsError> {
    let parent = target.parent().unwrap_or(Path::new("."));
    let file_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("path-mappings.json");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    for attempt in 0..1000 {
        let candidate = parent.join(format!(
            ".{file_name}.lbport-{}-{timestamp}-{attempt}.tmp",
            std::process::id()
        ));
        match fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&candidate)
        {
            Ok(file) => return Ok((candidate, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(source) => {
                return Err(HostPathMappingsError::Write {
                    path: candidate,
                    source,
                });
            }
        }
    }
    Err(HostPathMappingsError::UniqueTemporaryExhausted {
        path: target.to_path_buf(),
    })
}

#[cfg(not(windows))]
fn replace_file(temporary: &Path, target: &Path) -> Result<(), std::io::Error> {
    fs::rename(temporary, target)
}

#[cfg(windows)]
fn replace_file(temporary: &Path, target: &Path) -> Result<(), std::io::Error> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };

    let temporary = temporary
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let target = target
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let replaced = unsafe {
        MoveFileExW(
            temporary.as_ptr(),
            target.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if replaced == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(unix)]
fn sync_parent_directory(target: &Path) -> Result<(), std::io::Error> {
    fs::File::open(target.parent().unwrap_or(Path::new(".")))?.sync_all()
}

#[cfg(not(unix))]
fn sync_parent_directory(_target: &Path) -> Result<(), std::io::Error> {
    Ok(())
}

#[derive(Debug, Error)]
pub enum HostPathMappingsError {
    #[error(
        "{variables} does not provide an absolute configuration root; pass --path-mappings-file explicitly"
    )]
    DefaultConfigurationRootUnavailable { variables: &'static str },
    #[error("could not read host path mappings from {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not parse host path mappings from {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("could not serialize host path mappings: {source}")]
    Serialize {
        #[source]
        source: serde_json::Error,
    },
    #[error("could not write host path mappings at {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not replace {target} with host path mapping temporary {temporary}: {source}")]
    Replace {
        temporary: PathBuf,
        target: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("host path mapping version {version} is unsupported")]
    UnsupportedVersion { version: u32 },
    #[error("Windows drive {drive}: occurs more than once in host path mappings")]
    DuplicateWindowsDrive { drive: char },
    #[error("Windows UNC share \\\\{server}\\{share} occurs more than once in host path mappings")]
    DuplicateWindowsUnc { server: String, share: String },
    #[error("could not allocate a unique temporary beside {path}")]
    UniqueTemporaryExhausted { path: PathBuf },
    #[error(transparent)]
    InvalidMapping(#[from] LaunchPathError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LaunchPathResolver;

    #[test]
    fn mappings_round_trip_and_resolve_after_reload() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("settings/path-mappings.json");
        let mut mappings = HostPathMappings::default();
        mappings
            .set_windows_drive('e', directory.path().join("roms"))
            .expect("drive mapping");
        mappings
            .set_windows_unc("NAS", "Games", directory.path().join("network-games"))
            .expect("UNC mapping");
        mappings.save_atomic(&path).expect("save mappings");

        let loaded = HostPathMappings::load(&path).expect("reload mappings");
        assert_eq!(loaded, mappings);
        let resolver = loaded.resolver().expect("build resolver");
        #[cfg(not(windows))]
        {
            assert_eq!(
                resolver
                    .resolve(Path::new("/launchbox"), r"E:\Console\game.rom")
                    .expect("resolve drive"),
                directory.path().join("roms/Console/game.rom")
            );
            assert_eq!(
                resolver
                    .resolve(Path::new("/launchbox"), r"\\nas\games\Arcade\game.zip")
                    .expect("resolve UNC"),
                directory.path().join("network-games/Arcade/game.zip")
            );
        }
    }

    #[test]
    fn a_second_save_atomically_replaces_the_first_document() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("path-mappings.json");
        let mut mappings = HostPathMappings::default();
        mappings
            .set_windows_drive('D', directory.path().join("old"))
            .expect("first mapping");
        mappings.save_atomic(&path).expect("first save");
        mappings
            .set_windows_drive('D', directory.path().join("new"))
            .expect("replacement mapping");
        mappings.save_atomic(&path).expect("replacement save");

        let loaded = HostPathMappings::load(&path).expect("reload replacement");
        assert_eq!(
            loaded.windows_drives()[0].host_root(),
            directory.path().join("new")
        );
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    #[test]
    fn malformed_or_ambiguous_documents_are_rejected() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("path-mappings.json");
        fs::write(
            &path,
            r#"{
  "version": 1,
  "windows_drives": [
    {"drive": "D", "host_root": "/mnt/one"},
    {"drive": "d", "host_root": "/mnt/two"}
  ],
  "windows_unc": []
}"#,
        )
        .expect("write malformed settings");
        assert!(matches!(
            HostPathMappings::load(&path),
            Err(HostPathMappingsError::DuplicateWindowsDrive { drive: 'D' })
        ));

        fs::write(
            &path,
            r#"{"version": 99, "windows_drives": [], "windows_unc": []}"#,
        )
        .expect("write future settings");
        assert!(matches!(
            HostPathMappings::load(&path),
            Err(HostPathMappingsError::UnsupportedVersion { version: 99 })
        ));
    }
}
