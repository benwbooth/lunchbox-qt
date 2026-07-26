use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

pub const MODEL_VIEWER_STATE_VERSION: u32 = 1;

/// The axis policy used by LaunchBox's interactive model viewer.
///
/// The public keys deliberately describe the user's gesture rather than the
/// underlying 3D axis: horizontal input rotates around Y, while vertical
/// input rotates around X.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelRotationLock {
    #[default]
    Free,
    Horizontal,
    Vertical,
}

impl ModelRotationLock {
    pub const fn key(self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }

    pub fn from_key(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "free" => Some(Self::Free),
            "horizontal" => Some(Self::Horizontal),
            "vertical" => Some(Self::Vertical),
            _ => None,
        }
    }
}

/// Host-local state shared by LaunchBox and BigBox model viewers.
///
/// It is intentionally separate from portable LaunchBox XML and desktop
/// window geometry, allowing the same library to be opened on Linux, Windows,
/// and macOS without writing host preferences into the library.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelViewerState {
    pub version: u32,
    pub rotation_lock: ModelRotationLock,
}

impl Default for ModelViewerState {
    fn default() -> Self {
        Self {
            version: MODEL_VIEWER_STATE_VERSION,
            rotation_lock: ModelRotationLock::Free,
        }
    }
}

impl ModelViewerState {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ModelViewerStateError> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|source| ModelViewerStateError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let state = serde_json::from_slice::<Self>(&bytes).map_err(|source| {
            ModelViewerStateError::Parse {
                path: path.to_path_buf(),
                source,
            }
        })?;
        state.validate()?;
        Ok(state)
    }

    pub fn load_or_default(path: impl AsRef<Path>) -> Result<Self, ModelViewerStateError> {
        let path = path.as_ref();
        match Self::load(path) {
            Ok(state) => Ok(state),
            Err(ModelViewerStateError::Read { source, .. })
                if source.kind() == std::io::ErrorKind::NotFound =>
            {
                Ok(Self::default())
            }
            Err(error) => Err(error),
        }
    }

    pub fn save_atomic(&self, path: impl AsRef<Path>) -> Result<(), ModelViewerStateError> {
        self.validate()?;
        let path = path.as_ref();
        let parent = path.parent().unwrap_or(Path::new("."));
        fs::create_dir_all(parent).map_err(|source| ModelViewerStateError::Write {
            path: parent.to_path_buf(),
            source,
        })?;

        let mut bytes = serde_json::to_vec_pretty(self)
            .map_err(|source| ModelViewerStateError::Serialize { source })?;
        bytes.push(b'\n');
        let (temporary_path, mut temporary) = create_unique_sibling(path)?;
        let result = (|| {
            temporary
                .write_all(&bytes)
                .and_then(|()| temporary.sync_all())
                .map_err(|source| ModelViewerStateError::Write {
                    path: temporary_path.clone(),
                    source,
                })?;
            drop(temporary);
            replace_file(&temporary_path, path).map_err(|source| {
                ModelViewerStateError::Replace {
                    temporary: temporary_path.clone(),
                    target: path.to_path_buf(),
                    source,
                }
            })?;
            sync_parent_directory(path).map_err(|source| ModelViewerStateError::Write {
                path: parent.to_path_buf(),
                source,
            })
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary_path);
        }
        result
    }

    pub fn validate(&self) -> Result<(), ModelViewerStateError> {
        if self.version != MODEL_VIEWER_STATE_VERSION {
            return Err(ModelViewerStateError::UnsupportedVersion {
                version: self.version,
            });
        }
        Ok(())
    }
}

/// Returns the platform-native, port-owned model-viewer state location.
#[cfg(windows)]
pub fn default_model_viewer_state_path() -> Result<PathBuf, ModelViewerStateError> {
    absolute_environment_root("APPDATA")
        .map(|root| root.join("LaunchBox Port").join("model-viewer-state.json"))
        .ok_or(ModelViewerStateError::DefaultConfigurationRootUnavailable {
            variables: "APPDATA",
        })
}

/// Returns the platform-native, port-owned model-viewer state location.
#[cfg(target_os = "macos")]
pub fn default_model_viewer_state_path() -> Result<PathBuf, ModelViewerStateError> {
    absolute_environment_root("HOME")
        .map(|root| {
            root.join("Library")
                .join("Application Support")
                .join("LaunchBox Port")
                .join("model-viewer-state.json")
        })
        .ok_or(ModelViewerStateError::DefaultConfigurationRootUnavailable { variables: "HOME" })
}

/// Returns the platform-native, port-owned model-viewer state location.
#[cfg(all(not(windows), not(target_os = "macos")))]
pub fn default_model_viewer_state_path() -> Result<PathBuf, ModelViewerStateError> {
    if let Some(root) = absolute_environment_root("XDG_CONFIG_HOME") {
        return Ok(root.join("launchbox-port").join("model-viewer-state.json"));
    }
    absolute_environment_root("HOME")
        .map(|root| {
            root.join(".config")
                .join("launchbox-port")
                .join("model-viewer-state.json")
        })
        .ok_or(ModelViewerStateError::DefaultConfigurationRootUnavailable {
            variables: "XDG_CONFIG_HOME or HOME",
        })
}

fn absolute_environment_root(variable: &str) -> Option<PathBuf> {
    std::env::var_os(variable)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .filter(|root| root.is_absolute())
}

fn create_unique_sibling(target: &Path) -> Result<(PathBuf, fs::File), ModelViewerStateError> {
    let parent = target.parent().unwrap_or(Path::new("."));
    let file_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("model-viewer-state.json");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    for attempt in 0..1_000 {
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
                return Err(ModelViewerStateError::Write {
                    path: candidate,
                    source,
                });
            }
        }
    }
    Err(ModelViewerStateError::UniqueTemporaryExhausted {
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
    let result = unsafe {
        MoveFileExW(
            temporary.as_ptr(),
            target.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if result == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(unix)]
fn sync_parent_directory(path: &Path) -> Result<(), std::io::Error> {
    path.parent()
        .map(fs::File::open)
        .transpose()?
        .map_or(Ok(()), |directory| directory.sync_all())
}

#[cfg(not(unix))]
fn sync_parent_directory(_path: &Path) -> Result<(), std::io::Error> {
    Ok(())
}

#[derive(Debug, Error)]
pub enum ModelViewerStateError {
    #[error("could not read model-viewer state {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not parse model-viewer state {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("model-viewer state version {version} is not supported")]
    UnsupportedVersion { version: u32 },
    #[error("could not serialize model-viewer state: {source}")]
    Serialize {
        #[source]
        source: serde_json::Error,
    },
    #[error("could not write model-viewer state {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not replace model-viewer state {target} with {temporary}: {source}")]
    Replace {
        temporary: PathBuf,
        target: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("no absolute model-viewer configuration root is available from {variables}")]
    DefaultConfigurationRootUnavailable { variables: &'static str },
    #[error("could not allocate a unique model-viewer temporary beside {path}")]
    UniqueTemporaryExhausted { path: PathBuf },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_free_rotation() {
        let state = ModelViewerState::default();
        state.validate().expect("default state");
        assert_eq!(state.rotation_lock, ModelRotationLock::Free);
        assert_eq!(state.rotation_lock.key(), "free");
    }

    #[test]
    fn rotation_locks_round_trip_through_atomic_replacement() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("settings/model-viewer-state.json");
        let mut state = ModelViewerState {
            rotation_lock: ModelRotationLock::Horizontal,
            ..ModelViewerState::default()
        };
        state.save_atomic(&path).expect("first save");

        state.rotation_lock = ModelRotationLock::Vertical;
        state.save_atomic(&path).expect("replacement save");

        assert_eq!(ModelViewerState::load(&path).expect("reload"), state);
        assert_eq!(
            fs::read_dir(path.parent().unwrap())
                .expect("settings directory")
                .count(),
            1
        );
    }

    #[test]
    fn future_unknown_and_invalid_lock_documents_are_rejected() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("model-viewer-state.json");
        for payload in [
            r#"{"version":99,"rotation_lock":"free"}"#,
            r#"{"version":1,"rotation_lock":"diagonal"}"#,
            r#"{"version":1,"rotation_lock":"free","future":true}"#,
        ] {
            fs::write(&path, payload).expect("state fixture");
            assert!(ModelViewerState::load(&path).is_err(), "{payload}");
        }
    }

    #[test]
    fn lock_keys_are_strict_and_case_insensitive() {
        assert_eq!(
            ModelRotationLock::from_key(" Horizontal "),
            Some(ModelRotationLock::Horizontal)
        );
        assert_eq!(
            ModelRotationLock::from_key("VERTICAL"),
            Some(ModelRotationLock::Vertical)
        );
        assert_eq!(ModelRotationLock::from_key("x"), None);
    }
}
