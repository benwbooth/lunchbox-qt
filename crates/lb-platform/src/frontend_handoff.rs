use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

pub const FRONTEND_PEER_EXECUTABLE_FLAG: &str = "--frontend-peer-executable";
pub const FRONTEND_SELECTED_GAME_FLAG: &str = "--select-game-id";

const LIBRARY_FLAG: &str = "--library";
const FORWARDED_SINGLE_VALUE_FLAGS: &[&str] = &[
    "--path-mappings-file",
    "--ui-state-file",
    "--model-viewer-state-file",
];
const FORWARDED_REPEATED_VALUE_FLAGS: &[&str] = &["--map-windows-drive", "--map-windows-unc"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrontendKind {
    LaunchBox,
    BigBox,
}

impl FrontendKind {
    pub const fn key(self) -> &'static str {
        match self {
            Self::LaunchBox => "launchbox",
            Self::BigBox => "bigbox",
        }
    }

    fn executable_file_name(self) -> OsString {
        OsString::from(format!("{}{}", self.key(), std::env::consts::EXE_SUFFIX))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FrontendHandoffActivity {
    pub loading: bool,
    pub importing: bool,
    pub emulator_scan: bool,
    pub emulator_update: bool,
    pub writing: bool,
    pub launching: bool,
    pub launch_session_active: bool,
    pub startup_screen_active: bool,
    pub shutdown_screen_active: bool,
    pub pause_screen_active: bool,
}

impl FrontendHandoffActivity {
    pub const fn blocker(self) -> Option<&'static str> {
        if self.loading
            || self.importing
            || self.emulator_scan
            || self.emulator_update
            || self.writing
        {
            Some("the current library operation")
        } else if self.launching
            || self.launch_session_active
            || self.startup_screen_active
            || self.shutdown_screen_active
            || self.pause_screen_active
        {
            Some("the current game session")
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrontendHandoffPlan {
    pub executable: PathBuf,
    pub arguments: Vec<OsString>,
}

impl FrontendHandoffPlan {
    pub fn from_current_process(
        target: FrontendKind,
        library_root: &Path,
        selected_game_id: Option<&str>,
    ) -> Result<Self, FrontendHandoffError> {
        let current_executable =
            std::env::current_exe().map_err(FrontendHandoffError::CurrentExecutable)?;
        Self::from_parts(
            &current_executable,
            std::env::args_os().skip(1),
            target,
            library_root,
            selected_game_id,
        )
    }

    pub fn from_parts<I>(
        current_executable: &Path,
        arguments: I,
        target: FrontendKind,
        library_root: &Path,
        selected_game_id: Option<&str>,
    ) -> Result<Self, FrontendHandoffError>
    where
        I: IntoIterator<Item = OsString>,
    {
        if library_root.as_os_str().is_empty() {
            return Err(FrontendHandoffError::EmptyLibraryPath);
        }

        let arguments: Vec<_> = arguments.into_iter().collect();
        let mut forwarded = Vec::new();
        let mut peer_executable = None;
        let mut seen_single_value_flags = Vec::<&'static str>::new();
        let mut index = 0;

        while index < arguments.len() {
            let argument = &arguments[index];
            let flag = known_value_flag(argument);
            let Some(flag) = flag else {
                index += 1;
                continue;
            };
            let value = arguments
                .get(index + 1)
                .ok_or(FrontendHandoffError::MissingFlagValue { flag })?
                .clone();
            if value.is_empty() {
                return Err(FrontendHandoffError::EmptyFlagValue { flag });
            }

            if flag == FRONTEND_PEER_EXECUTABLE_FLAG {
                if peer_executable.replace(PathBuf::from(value)).is_some() {
                    return Err(FrontendHandoffError::DuplicateFlag { flag });
                }
            } else if flag == LIBRARY_FLAG || flag == FRONTEND_SELECTED_GAME_FLAG {
                ensure_single_flag(&mut seen_single_value_flags, flag)?;
            } else if FORWARDED_SINGLE_VALUE_FLAGS.contains(&flag) {
                ensure_single_flag(&mut seen_single_value_flags, flag)?;
                forwarded.push(OsString::from(flag));
                forwarded.push(value);
            } else {
                debug_assert!(FORWARDED_REPEATED_VALUE_FLAGS.contains(&flag));
                forwarded.push(OsString::from(flag));
                forwarded.push(value);
            }
            index += 2;
        }

        let executable = match peer_executable {
            Some(path) if path.is_absolute() => path,
            Some(path) => {
                return Err(FrontendHandoffError::RelativePeerExecutable { path });
            }
            None => current_executable.with_file_name(target.executable_file_name()),
        };

        forwarded.push(OsString::from(LIBRARY_FLAG));
        forwarded.push(library_root.as_os_str().to_os_string());
        if let Some(selected_game_id) = selected_game_id
            .map(str::trim)
            .filter(|game_id| !game_id.is_empty())
        {
            forwarded.push(OsString::from(FRONTEND_SELECTED_GAME_FLAG));
            forwarded.push(OsString::from(selected_game_id));
        }

        Ok(Self {
            executable,
            arguments: forwarded,
        })
    }

    pub fn spawn(&self) -> Result<(), FrontendHandoffError> {
        Command::new(&self.executable)
            .args(&self.arguments)
            .spawn()
            .map(|_| ())
            .map_err(|source| FrontendHandoffError::Spawn {
                executable: self.executable.clone(),
                source,
            })
    }
}

fn known_value_flag(argument: &OsStr) -> Option<&'static str> {
    std::iter::once(LIBRARY_FLAG)
        .chain(std::iter::once(FRONTEND_PEER_EXECUTABLE_FLAG))
        .chain(std::iter::once(FRONTEND_SELECTED_GAME_FLAG))
        .chain(FORWARDED_SINGLE_VALUE_FLAGS.iter().copied())
        .chain(FORWARDED_REPEATED_VALUE_FLAGS.iter().copied())
        .find(|flag| argument == OsStr::new(flag))
}

fn ensure_single_flag(
    seen_single_value_flags: &mut Vec<&'static str>,
    flag: &'static str,
) -> Result<(), FrontendHandoffError> {
    if seen_single_value_flags.contains(&flag) {
        Err(FrontendHandoffError::DuplicateFlag { flag })
    } else {
        seen_single_value_flags.push(flag);
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum FrontendHandoffError {
    #[error("could not resolve the current frontend executable: {0}")]
    CurrentExecutable(#[source] std::io::Error),
    #[error("the loaded LaunchBox library path is empty")]
    EmptyLibraryPath,
    #[error("{flag} requires a value")]
    MissingFlagValue { flag: &'static str },
    #[error("{flag} requires a non-empty value")]
    EmptyFlagValue { flag: &'static str },
    #[error("{flag} may only be supplied once")]
    DuplicateFlag { flag: &'static str },
    #[error("{FRONTEND_PEER_EXECUTABLE_FLAG} requires an absolute path, received {path:?}")]
    RelativePeerExecutable { path: PathBuf },
    #[error("could not start frontend executable {executable:?}: {source}")]
    Spawn {
        executable: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn os_arguments(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn handoff_keeps_only_durable_context_and_uses_loaded_library_truth() {
        let plan = FrontendHandoffPlan::from_parts(
            Path::new("/opt/lunchbox/launchbox"),
            os_arguments(&[
                "--library",
                "/old/library",
                "--smoke-test",
                "--windowed",
                "--path-mappings-file",
                "/state/paths.json",
                "--map-windows-drive",
                "D=/mnt/games",
                "--map-windows-unc",
                "nas/roms=/srv/roms",
                "--launch-game-id",
                "must-not-cross",
                "--ui-state-file",
                "/state/ui.json",
                "--model-viewer-state-file",
                "/state/model.json",
                FRONTEND_SELECTED_GAME_FLAG,
                "stale-selection",
            ]),
            FrontendKind::BigBox,
            Path::new("/live/library"),
            Some("current-game"),
        )
        .unwrap();

        assert_eq!(plan.executable, Path::new("/opt/lunchbox/bigbox"));
        assert_eq!(
            plan.arguments,
            os_arguments(&[
                "--path-mappings-file",
                "/state/paths.json",
                "--map-windows-drive",
                "D=/mnt/games",
                "--map-windows-unc",
                "nas/roms=/srv/roms",
                "--ui-state-file",
                "/state/ui.json",
                "--model-viewer-state-file",
                "/state/model.json",
                "--library",
                "/live/library",
                "--select-game-id",
                "current-game",
            ])
        );
    }

    #[test]
    fn explicit_absolute_peer_is_used_once_but_never_forwarded() {
        let plan = FrontendHandoffPlan::from_parts(
            Path::new("/Applications/LunchBox.app/Contents/MacOS/bigbox"),
            os_arguments(&[
                FRONTEND_PEER_EXECUTABLE_FLAG,
                "/tmp/frontend-handoff-recorder",
                "--windowed",
            ]),
            FrontendKind::LaunchBox,
            Path::new("/Users/ben/LaunchBox"),
            None,
        )
        .unwrap();

        assert_eq!(plan.executable, Path::new("/tmp/frontend-handoff-recorder"));
        assert_eq!(
            plan.arguments,
            os_arguments(&["--library", "/Users/ben/LaunchBox"])
        );
    }

    #[test]
    fn executable_suffix_and_sibling_layout_are_platform_native() {
        let current = PathBuf::from(format!(
            "/opt/lunchbox/bigbox{}",
            std::env::consts::EXE_SUFFIX
        ));
        let expected = PathBuf::from(format!(
            "/opt/lunchbox/launchbox{}",
            std::env::consts::EXE_SUFFIX
        ));
        let plan = FrontendHandoffPlan::from_parts(
            &current,
            Vec::new(),
            FrontendKind::LaunchBox,
            Path::new("D:/Portable/LaunchBox"),
            None,
        )
        .unwrap();
        assert_eq!(plan.executable, expected);
    }

    #[test]
    fn malformed_or_ambiguous_context_fails_closed() {
        assert!(matches!(
            FrontendHandoffPlan::from_parts(
                Path::new("/opt/lunchbox/launchbox"),
                os_arguments(&["--path-mappings-file"]),
                FrontendKind::BigBox,
                Path::new("/library"),
                None,
            ),
            Err(FrontendHandoffError::MissingFlagValue {
                flag: "--path-mappings-file"
            })
        ));
        assert!(matches!(
            FrontendHandoffPlan::from_parts(
                Path::new("/opt/lunchbox/launchbox"),
                os_arguments(&["--library", "/one", "--library", "/two"]),
                FrontendKind::BigBox,
                Path::new("/library"),
                None,
            ),
            Err(FrontendHandoffError::DuplicateFlag { flag: "--library" })
        ));
        assert!(matches!(
            FrontendHandoffPlan::from_parts(
                Path::new("/opt/lunchbox/launchbox"),
                os_arguments(&[FRONTEND_PEER_EXECUTABLE_FLAG, "relative/bigbox"]),
                FrontendKind::BigBox,
                Path::new("/library"),
                None,
            ),
            Err(FrontendHandoffError::RelativePeerExecutable { .. })
        ));
    }

    #[test]
    fn every_activity_gate_blocks_the_handoff_in_its_expected_class() {
        let library_activity = [
            FrontendHandoffActivity {
                loading: true,
                ..Default::default()
            },
            FrontendHandoffActivity {
                importing: true,
                ..Default::default()
            },
            FrontendHandoffActivity {
                emulator_scan: true,
                ..Default::default()
            },
            FrontendHandoffActivity {
                emulator_update: true,
                ..Default::default()
            },
            FrontendHandoffActivity {
                writing: true,
                ..Default::default()
            },
        ];
        for activity in library_activity {
            assert_eq!(activity.blocker(), Some("the current library operation"));
        }

        let game_activity = [
            FrontendHandoffActivity {
                launching: true,
                ..Default::default()
            },
            FrontendHandoffActivity {
                launch_session_active: true,
                ..Default::default()
            },
            FrontendHandoffActivity {
                startup_screen_active: true,
                ..Default::default()
            },
            FrontendHandoffActivity {
                shutdown_screen_active: true,
                ..Default::default()
            },
            FrontendHandoffActivity {
                pause_screen_active: true,
                ..Default::default()
            },
        ];
        for activity in game_activity {
            assert_eq!(activity.blocker(), Some("the current game session"));
        }

        assert_eq!(FrontendHandoffActivity::default().blocker(), None);
    }
}
