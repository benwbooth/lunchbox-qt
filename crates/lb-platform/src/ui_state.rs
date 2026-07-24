use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

pub const LAUNCHBOX_UI_STATE_VERSION: u32 = 1;

pub const MIN_GAME_DETAILS_PANE_WIDTH: u32 = 300;
pub const MAX_GAME_DETAILS_PANE_WIDTH: u32 = 1_600;
pub const MIN_GAME_DETAILS_WINDOW_WIDTH: u32 = 320;
pub const MAX_GAME_DETAILS_WINDOW_WIDTH: u32 = 7_680;
pub const MIN_GAME_DETAILS_WINDOW_HEIGHT: u32 = 320;
pub const MAX_GAME_DETAILS_WINDOW_HEIGHT: u32 = 4_320;
const MIN_WINDOW_POSITION: i32 = -100_000;
const MAX_WINDOW_POSITION: i32 = 100_000;

/// Host-specific desktop layout state. It intentionally lives outside the
/// LaunchBox library so opening one portable library on several operating
/// systems never rewrites shared XML with machine-specific window geometry.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LaunchBoxUiState {
    pub version: u32,
    pub show_game_details: bool,
    pub game_details_popped_out: bool,
    pub game_details_pane_width: u32,
    pub game_details_window: GameDetailsWindowState,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GameDetailsWindowState {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
}

impl Default for LaunchBoxUiState {
    fn default() -> Self {
        Self {
            version: LAUNCHBOX_UI_STATE_VERSION,
            show_game_details: true,
            game_details_popped_out: false,
            game_details_pane_width: 360,
            game_details_window: GameDetailsWindowState {
                x: 120,
                y: 80,
                width: 480,
                height: 640,
                maximized: false,
            },
        }
    }
}

impl LaunchBoxUiState {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, LaunchBoxUiStateError> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|source| LaunchBoxUiStateError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let state = serde_json::from_slice::<Self>(&bytes).map_err(|source| {
            LaunchBoxUiStateError::Parse {
                path: path.to_path_buf(),
                source,
            }
        })?;
        state.validate()?;
        Ok(state)
    }

    pub fn load_or_default(path: impl AsRef<Path>) -> Result<Self, LaunchBoxUiStateError> {
        let path = path.as_ref();
        match Self::load(path) {
            Ok(state) => Ok(state),
            Err(LaunchBoxUiStateError::Read { source, .. })
                if source.kind() == std::io::ErrorKind::NotFound =>
            {
                Ok(Self::default())
            }
            Err(error) => Err(error),
        }
    }

    pub fn save_atomic(&self, path: impl AsRef<Path>) -> Result<(), LaunchBoxUiStateError> {
        self.validate()?;
        let path = path.as_ref();
        let parent = path.parent().unwrap_or(Path::new("."));
        fs::create_dir_all(parent).map_err(|source| LaunchBoxUiStateError::Write {
            path: parent.to_path_buf(),
            source,
        })?;

        let mut bytes = serde_json::to_vec_pretty(self)
            .map_err(|source| LaunchBoxUiStateError::Serialize { source })?;
        bytes.push(b'\n');
        let (temporary_path, mut temporary) = create_unique_sibling(path)?;
        let result = (|| {
            temporary
                .write_all(&bytes)
                .and_then(|()| temporary.sync_all())
                .map_err(|source| LaunchBoxUiStateError::Write {
                    path: temporary_path.clone(),
                    source,
                })?;
            drop(temporary);
            replace_file(&temporary_path, path).map_err(|source| {
                LaunchBoxUiStateError::Replace {
                    temporary: temporary_path.clone(),
                    target: path.to_path_buf(),
                    source,
                }
            })?;
            sync_parent_directory(path).map_err(|source| LaunchBoxUiStateError::Write {
                path: parent.to_path_buf(),
                source,
            })
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary_path);
        }
        result
    }

    pub fn validate(&self) -> Result<(), LaunchBoxUiStateError> {
        if self.version != LAUNCHBOX_UI_STATE_VERSION {
            return Err(LaunchBoxUiStateError::UnsupportedVersion {
                version: self.version,
            });
        }
        validate_range(
            "game_details_pane_width",
            self.game_details_pane_width,
            MIN_GAME_DETAILS_PANE_WIDTH,
            MAX_GAME_DETAILS_PANE_WIDTH,
        )?;
        validate_range(
            "game_details_window.width",
            self.game_details_window.width,
            MIN_GAME_DETAILS_WINDOW_WIDTH,
            MAX_GAME_DETAILS_WINDOW_WIDTH,
        )?;
        validate_range(
            "game_details_window.height",
            self.game_details_window.height,
            MIN_GAME_DETAILS_WINDOW_HEIGHT,
            MAX_GAME_DETAILS_WINDOW_HEIGHT,
        )?;
        for (field, value) in [
            ("game_details_window.x", self.game_details_window.x),
            ("game_details_window.y", self.game_details_window.y),
        ] {
            if !(MIN_WINDOW_POSITION..=MAX_WINDOW_POSITION).contains(&value) {
                return Err(LaunchBoxUiStateError::InvalidPosition { field, value });
            }
        }
        Ok(())
    }
}

fn validate_range(
    field: &'static str,
    value: u32,
    minimum: u32,
    maximum: u32,
) -> Result<(), LaunchBoxUiStateError> {
    if (minimum..=maximum).contains(&value) {
        Ok(())
    } else {
        Err(LaunchBoxUiStateError::InvalidDimension {
            field,
            value,
            minimum,
            maximum,
        })
    }
}

/// Returns the platform-native, port-owned desktop-layout location.
#[cfg(windows)]
pub fn default_launchbox_ui_state_path() -> Result<PathBuf, LaunchBoxUiStateError> {
    absolute_environment_root("APPDATA")
        .map(|root| root.join("LaunchBox Port").join("ui-state.json"))
        .ok_or(LaunchBoxUiStateError::DefaultConfigurationRootUnavailable {
            variables: "APPDATA",
        })
}

/// Returns the platform-native, port-owned desktop-layout location.
#[cfg(target_os = "macos")]
pub fn default_launchbox_ui_state_path() -> Result<PathBuf, LaunchBoxUiStateError> {
    absolute_environment_root("HOME")
        .map(|root| {
            root.join("Library")
                .join("Application Support")
                .join("LaunchBox Port")
                .join("ui-state.json")
        })
        .ok_or(LaunchBoxUiStateError::DefaultConfigurationRootUnavailable { variables: "HOME" })
}

/// Returns the platform-native, port-owned desktop-layout location.
#[cfg(all(not(windows), not(target_os = "macos")))]
pub fn default_launchbox_ui_state_path() -> Result<PathBuf, LaunchBoxUiStateError> {
    if let Some(root) = absolute_environment_root("XDG_CONFIG_HOME") {
        return Ok(root.join("launchbox-port").join("ui-state.json"));
    }
    absolute_environment_root("HOME")
        .map(|root| {
            root.join(".config")
                .join("launchbox-port")
                .join("ui-state.json")
        })
        .ok_or(LaunchBoxUiStateError::DefaultConfigurationRootUnavailable {
            variables: "XDG_CONFIG_HOME or HOME",
        })
}

fn absolute_environment_root(variable: &str) -> Option<PathBuf> {
    std::env::var_os(variable)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .filter(|root| root.is_absolute())
}

fn create_unique_sibling(target: &Path) -> Result<(PathBuf, fs::File), LaunchBoxUiStateError> {
    let parent = target.parent().unwrap_or(Path::new("."));
    let file_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("ui-state.json");
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
                return Err(LaunchBoxUiStateError::Write {
                    path: candidate,
                    source,
                });
            }
        }
    }
    Err(LaunchBoxUiStateError::UniqueTemporaryExhausted {
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
pub enum LaunchBoxUiStateError {
    #[error(
        "{variables} does not provide an absolute configuration root; pass --ui-state-file explicitly"
    )]
    DefaultConfigurationRootUnavailable { variables: &'static str },
    #[error("could not read LaunchBox UI state from {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not parse LaunchBox UI state from {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("could not serialize LaunchBox UI state: {source}")]
    Serialize {
        #[source]
        source: serde_json::Error,
    },
    #[error("could not write LaunchBox UI state at {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not replace {target} with UI state temporary {temporary}: {source}")]
    Replace {
        temporary: PathBuf,
        target: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("LaunchBox UI state version {version} is unsupported")]
    UnsupportedVersion { version: u32 },
    #[error("{field} value {value} is outside the supported {minimum}..={maximum} range")]
    InvalidDimension {
        field: &'static str,
        value: u32,
        minimum: u32,
        maximum: u32,
    },
    #[error("{field} position {value} is outside the supported desktop coordinate range")]
    InvalidPosition { field: &'static str, value: i32 },
    #[error("could not allocate a unique UI state temporary beside {path}")]
    UniqueTemporaryExhausted { path: PathBuf },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_visible_docked_and_within_supported_geometry() {
        let state = LaunchBoxUiState::default();
        state.validate().expect("default state");
        assert!(state.show_game_details);
        assert!(!state.game_details_popped_out);
        assert_eq!(state.game_details_pane_width, 360);
        assert_eq!(state.game_details_window.width, 480);
        assert_eq!(state.game_details_window.height, 640);
    }

    #[test]
    fn state_round_trips_and_atomically_replaces_an_existing_document() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("settings/ui-state.json");
        let mut state = LaunchBoxUiState::default();
        state.save_atomic(&path).expect("first save");

        state.game_details_popped_out = true;
        state.game_details_pane_width = 444;
        state.game_details_window = GameDetailsWindowState {
            x: -640,
            y: 72,
            width: 720,
            height: 900,
            maximized: true,
        };
        state.save_atomic(&path).expect("replacement save");

        assert_eq!(LaunchBoxUiState::load(&path).expect("reload"), state);
        assert_eq!(
            fs::read_dir(path.parent().unwrap())
                .expect("settings directory")
                .count(),
            1
        );
    }

    #[test]
    fn malformed_future_and_unsafe_geometry_documents_are_rejected() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("ui-state.json");
        fs::write(
            &path,
            r#"{"version":99,"show_game_details":true,"game_details_popped_out":false,"game_details_pane_width":360,"game_details_window":{"x":0,"y":0,"width":480,"height":640,"maximized":false}}"#,
        )
        .expect("future state");
        assert!(matches!(
            LaunchBoxUiState::load(&path),
            Err(LaunchBoxUiStateError::UnsupportedVersion { version: 99 })
        ));

        let mut state = LaunchBoxUiState::default();
        state.game_details_window.width = 1;
        assert!(matches!(
            state.validate(),
            Err(LaunchBoxUiStateError::InvalidDimension {
                field: "game_details_window.width",
                ..
            })
        ));

        fs::write(
            &path,
            r#"{"version":1,"show_game_details":true,"game_details_popped_out":false,"game_details_pane_width":360,"game_details_window":{"x":0,"y":0,"width":480,"height":640,"maximized":false},"future":true}"#,
        )
        .expect("unknown field state");
        assert!(matches!(
            LaunchBoxUiState::load(&path),
            Err(LaunchBoxUiStateError::Parse { .. })
        ));
    }
}
