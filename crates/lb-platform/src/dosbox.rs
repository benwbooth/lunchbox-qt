use crate::{LaunchRequest, LaunchResourceLease};
use lb_domain::{Game, Mount};
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

use crate::{LaunchPathError, LaunchPathResolver};

pub(crate) struct DosBoxPlan {
    pub request: LaunchRequest,
    pub resource_leases: Vec<LaunchResourceLease>,
}

pub(crate) fn build_plan(
    launchbox_root: &Path,
    game: &Game,
    mounts: &[Mount],
    path_resolver: &dyn LaunchPathResolver,
) -> Result<DosBoxPlan, DosBoxPlanError> {
    let executable = resolve_executable(launchbox_root, game, path_resolver)?;
    let mut request = LaunchRequest::new(&executable);
    request.working_directory = executable
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(Path::to_path_buf);
    request.hide_console = true;
    request.arguments.push(OsString::from("-noconsole"));

    let configuration = resolve_configuration(launchbox_root, game, path_resolver)?;
    if let Some(configuration) = &configuration {
        request.arguments.push(OsString::from("-conf"));
        request.arguments.push(configuration.as_os_str().to_owned());
    }

    // LaunchBox permits an empty application path when the custom DOSBox
    // configuration owns the complete startup sequence in its [autoexec]
    // section. In that mode, -noautoexec and generated commands must be absent.
    if game.application_path.trim().is_empty() {
        if configuration.is_none() {
            return Err(DosBoxPlanError::MissingApplicationAndConfiguration {
                game_id: game.id.clone(),
            });
        }
        return Ok(DosBoxPlan {
            request,
            resource_leases: Vec::new(),
        });
    }

    let application = path_resolver
        .resolve(launchbox_root, &game.application_path)
        .map_err(|source| DosBoxPlanError::ApplicationPath {
            game_id: game.id.clone(),
            source,
        })?;
    let root = match game
        .root_folder
        .as_deref()
        .filter(|root| !root.trim().is_empty())
    {
        Some(root) => path_resolver
            .resolve(launchbox_root, root)
            .map_err(|source| DosBoxPlanError::RootPath {
                game_id: game.id.clone(),
                source,
            })?,
        None => application.parent().map(Path::to_path_buf).ok_or_else(|| {
            DosBoxPlanError::ApplicationHasNoParent {
                game_id: game.id.clone(),
                path: application.clone(),
            }
        })?,
    };
    let relative_application =
        application
            .strip_prefix(&root)
            .map_err(|_| DosBoxPlanError::ApplicationOutsideRoot {
                game_id: game.id.clone(),
                application: application.clone(),
                root: root.clone(),
            })?;
    validate_guest_relative_path(game, relative_application)?;

    let mut used_drives = BTreeSet::from(['C']);
    for mount in mounts.iter().filter(|mount| mount.game_id == game.id) {
        if !mount.drive_letter.is_ascii_alphabetic() {
            return Err(DosBoxPlanError::InvalidDrive {
                game_id: game.id.clone(),
                drive: mount.drive_letter,
            });
        }
        let drive = mount.drive_letter.to_ascii_uppercase();
        if !used_drives.insert(drive) {
            return Err(DosBoxPlanError::DuplicateDrive {
                game_id: game.id.clone(),
                drive,
            });
        }
    }

    request.arguments.push(OsString::from("-noautoexec"));
    push_command(&mut request, "@ECHO OFF");
    push_command(&mut request, "CLS");
    let root = quoted_host_path("DOSBox C drive", &root)?;
    push_command(&mut request, format!("MOUNT C {root}"));
    for mount in mounts.iter().filter(|mount| mount.game_id == game.id) {
        push_command(
            &mut request,
            mount_command(launchbox_root, mount, path_resolver)?,
        );
    }
    push_command(&mut request, "C:");

    let file_name = relative_application.file_name().ok_or_else(|| {
        DosBoxPlanError::MissingApplicationFileName {
            game_id: game.id.clone(),
            path: application.clone(),
        }
    })?;
    if let Some(parent) = relative_application
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        let guest_directory = guest_path(parent, game)?;
        push_command(
            &mut request,
            format!(
                "CD {}",
                quote_dos_value("application directory", &guest_directory)?
            ),
        );
    }
    let file_name = file_name
        .to_str()
        .ok_or_else(|| DosBoxPlanError::NonUnicodeGuestPath {
            game_id: game.id.clone(),
            path: relative_application.to_path_buf(),
        })?;
    let program = quote_dos_value("application file name", file_name)?;
    let mut run = if Path::new(file_name)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("bat"))
    {
        format!("CALL {program}")
    } else {
        program
    };
    if let Some(arguments) = game
        .command_line
        .as_deref()
        .filter(|arguments| !arguments.trim().is_empty())
    {
        if arguments.contains(['\r', '\n']) {
            return Err(DosBoxPlanError::InvalidCommandLine {
                game_id: game.id.clone(),
            });
        }
        run.push(' ');
        run.push_str(arguments);
    }
    push_command(&mut request, run);
    push_command(&mut request, "EXIT");

    Ok(DosBoxPlan {
        request,
        resource_leases: Vec::new(),
    })
}

fn resolve_executable(
    launchbox_root: &Path,
    game: &Game,
    path_resolver: &dyn LaunchPathResolver,
) -> Result<PathBuf, DosBoxPlanError> {
    if let Some(executable) = game
        .custom_dos_box_version_path
        .as_deref()
        .filter(|path| !path.trim().is_empty())
    {
        return path_resolver
            .resolve(launchbox_root, executable)
            .map_err(|source| DosBoxPlanError::ExecutablePath {
                game_id: game.id.clone(),
                source,
            });
    }

    #[cfg(windows)]
    {
        path_resolver
            .resolve(launchbox_root, r"ThirdParty\DOSBox\DOSBox.exe")
            .map_err(|source| DosBoxPlanError::ExecutablePath {
                game_id: game.id.clone(),
                source,
            })
    }
    #[cfg(not(windows))]
    {
        Ok(PathBuf::from("dosbox"))
    }
}

fn resolve_configuration(
    launchbox_root: &Path,
    game: &Game,
    path_resolver: &dyn LaunchPathResolver,
) -> Result<Option<PathBuf>, DosBoxPlanError> {
    if let Some(configuration) = game
        .dos_box_configuration_path
        .as_deref()
        .filter(|path| !path.trim().is_empty())
    {
        return path_resolver
            .resolve(launchbox_root, configuration)
            .map(Some)
            .map_err(|source| DosBoxPlanError::ConfigurationPath {
                game_id: game.id.clone(),
                source,
            });
    }
    let bundled = path_resolver
        .resolve(launchbox_root, r"ThirdParty\DOSBox\dosbox.conf")
        .map_err(|source| DosBoxPlanError::ConfigurationPath {
            game_id: game.id.clone(),
            source,
        })?;
    Ok(bundled.is_file().then_some(bundled))
}

fn mount_command(
    launchbox_root: &Path,
    mount: &Mount,
    path_resolver: &dyn LaunchPathResolver,
) -> Result<String, DosBoxPlanError> {
    let drive = mount.drive_letter.to_ascii_uppercase();
    let path = path_resolver
        .resolve(launchbox_root, &mount.path)
        .map_err(|source| DosBoxPlanError::MountPath {
            game_id: mount.game_id.clone(),
            drive,
            source,
        })?;
    let path = quoted_host_path("DOSBox mount", &path)?;
    let mount_type = mount.mount_type.trim();
    let media_type = mount.media_type.trim();
    let filesystem = mount.filesystem.trim();

    if mount_type.eq_ignore_ascii_case("Folder") {
        let mut command = format!("MOUNT {drive} {path}");
        if media_type.is_empty() || media_type.eq_ignore_ascii_case("Hard Disk") {
            validate_filesystem(mount, &["", "FAT"])?;
        } else if media_type.eq_ignore_ascii_case("Floppy") {
            validate_filesystem(mount, &["", "FAT"])?;
            command.push_str(" -t floppy");
        } else if media_type.eq_ignore_ascii_case("CD-ROM/ISO") {
            validate_filesystem(mount, &["", "ISO"])?;
            command.push_str(" -t cdrom -fs iso");
        } else {
            return Err(DosBoxPlanError::UnsupportedMountMediaType {
                game_id: mount.game_id.clone(),
                drive,
                media_type: mount.media_type.clone(),
            });
        }
        return Ok(command);
    }

    if mount_type.eq_ignore_ascii_case("File") {
        let mut command = format!("IMGMOUNT {drive} {path}");
        if media_type.eq_ignore_ascii_case("Floppy") {
            validate_filesystem(mount, &["", "FAT"])?;
            command.push_str(" -t floppy");
        } else if media_type.eq_ignore_ascii_case("CD-ROM/ISO") {
            validate_filesystem(mount, &["", "ISO"])?;
            command.push_str(" -t iso");
        } else if media_type.eq_ignore_ascii_case("Hard Disk") {
            validate_filesystem(mount, &["", "FAT"])?;
            command.push_str(" -t hdd");
        } else {
            return Err(DosBoxPlanError::UnsupportedMountMediaType {
                game_id: mount.game_id.clone(),
                drive,
                media_type: mount.media_type.clone(),
            });
        }
        if filesystem.eq_ignore_ascii_case("FAT") {
            command.push_str(" -fs fat");
        } else if filesystem.eq_ignore_ascii_case("ISO") {
            command.push_str(" -fs iso");
        }
        return Ok(command);
    }

    Err(DosBoxPlanError::UnsupportedMountType {
        game_id: mount.game_id.clone(),
        drive,
        mount_type: mount.mount_type.clone(),
    })
}

fn validate_filesystem(mount: &Mount, allowed: &[&str]) -> Result<(), DosBoxPlanError> {
    if allowed
        .iter()
        .any(|allowed| mount.filesystem.trim().eq_ignore_ascii_case(allowed))
    {
        Ok(())
    } else {
        Err(DosBoxPlanError::UnsupportedMountFilesystem {
            game_id: mount.game_id.clone(),
            drive: mount.drive_letter.to_ascii_uppercase(),
            filesystem: mount.filesystem.clone(),
            media_type: mount.media_type.clone(),
        })
    }
}

fn validate_guest_relative_path(game: &Game, path: &Path) -> Result<(), DosBoxPlanError> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_) | Component::CurDir))
    {
        return Err(DosBoxPlanError::InvalidGuestApplicationPath {
            game_id: game.id.clone(),
            path: path.to_path_buf(),
        });
    }
    Ok(())
}

fn guest_path(path: &Path, game: &Game) -> Result<String, DosBoxPlanError> {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value),
            Component::CurDir => None,
            _ => unreachable!("validated guest path contains only relative components"),
        })
        .map(|component| {
            component.to_str().map(str::to_owned).ok_or_else(|| {
                DosBoxPlanError::NonUnicodeGuestPath {
                    game_id: game.id.clone(),
                    path: path.to_path_buf(),
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|components| components.join("\\"))
}

fn quoted_host_path(purpose: &'static str, path: &Path) -> Result<String, DosBoxPlanError> {
    let path = path
        .to_str()
        .ok_or_else(|| DosBoxPlanError::NonUnicodeHostPath {
            purpose,
            path: path.to_path_buf(),
        })?;
    quote_dos_value(purpose, path)
}

fn quote_dos_value(purpose: &'static str, value: &str) -> Result<String, DosBoxPlanError> {
    if value.contains(['"', '\r', '\n']) {
        return Err(DosBoxPlanError::UnrepresentableDosBoxValue { purpose });
    }
    Ok(format!("\"{value}\""))
}

fn push_command(request: &mut LaunchRequest, command: impl Into<OsString>) {
    request.arguments.push(OsString::from("-c"));
    request.arguments.push(command.into());
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum DosBoxPlanError {
    #[error("DOSBox executable for game {game_id} cannot be used on this host: {source}")]
    ExecutablePath {
        game_id: String,
        #[source]
        source: LaunchPathError,
    },
    #[error("DOSBox configuration for game {game_id} cannot be used on this host: {source}")]
    ConfigurationPath {
        game_id: String,
        #[source]
        source: LaunchPathError,
    },
    #[error("DOSBox application for game {game_id} cannot be used on this host: {source}")]
    ApplicationPath {
        game_id: String,
        #[source]
        source: LaunchPathError,
    },
    #[error("DOSBox C-drive root for game {game_id} cannot be used on this host: {source}")]
    RootPath {
        game_id: String,
        #[source]
        source: LaunchPathError,
    },
    #[error("DOSBox mount {drive}: for game {game_id} cannot be used on this host: {source}")]
    MountPath {
        game_id: String,
        drive: char,
        #[source]
        source: LaunchPathError,
    },
    #[error(
        "DOSBox game {game_id} has neither an application nor a configuration with [autoexec]"
    )]
    MissingApplicationAndConfiguration { game_id: String },
    #[error("DOSBox application for game {game_id} has no parent directory: {path}")]
    ApplicationHasNoParent { game_id: String, path: PathBuf },
    #[error(
        "DOSBox application for game {game_id} is outside its C-drive root {root}: {application}"
    )]
    ApplicationOutsideRoot {
        game_id: String,
        application: PathBuf,
        root: PathBuf,
    },
    #[error("DOSBox application for game {game_id} has an unsafe guest-relative path: {path}")]
    InvalidGuestApplicationPath { game_id: String, path: PathBuf },
    #[error("DOSBox application for game {game_id} has no file name: {path}")]
    MissingApplicationFileName { game_id: String, path: PathBuf },
    #[error("DOSBox guest path for game {game_id} is not Unicode: {path}")]
    NonUnicodeGuestPath { game_id: String, path: PathBuf },
    #[error("{purpose} path is not Unicode and cannot be represented in a DOSBox command: {path}")]
    NonUnicodeHostPath {
        purpose: &'static str,
        path: PathBuf,
    },
    #[error(
        "{purpose} contains a quote or newline that cannot be represented in a DOSBox command"
    )]
    UnrepresentableDosBoxValue { purpose: &'static str },
    #[error("DOSBox game {game_id} uses drive {drive}: more than once (C: is reserved for the game root)")]
    DuplicateDrive { game_id: String, drive: char },
    #[error("DOSBox game {game_id} uses invalid drive letter {drive:?}")]
    InvalidDrive { game_id: String, drive: char },
    #[error("DOSBox mount {drive}: for game {game_id} uses unsupported mount type {mount_type:?}")]
    UnsupportedMountType {
        game_id: String,
        drive: char,
        mount_type: String,
    },
    #[error("DOSBox mount {drive}: for game {game_id} uses unsupported media type {media_type:?}")]
    UnsupportedMountMediaType {
        game_id: String,
        drive: char,
        media_type: String,
    },
    #[error("DOSBox mount {drive}: for game {game_id} uses filesystem {filesystem:?}, which is incompatible with media type {media_type:?}")]
    UnsupportedMountFilesystem {
        game_id: String,
        drive: char,
        filesystem: String,
        media_type: String,
    },
    #[error("DOSBox command line for game {game_id} contains a newline")]
    InvalidCommandLine { game_id: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HostPathResolver;

    fn game() -> Game {
        Game {
            id: "dos-game".into(),
            title: "DOS Fixture".into(),
            platform: "DOS".into(),
            application_path: r"Games\DOS Fixture/BIN\PLAY.BAT".into(),
            command_line: Some("-fast".into()),
            use_dos_box: true,
            custom_dos_box_version_path: Some(r"Runtime\dosbox-recorder".into()),
            dos_box_configuration_path: Some(r"Config/dosbox.conf".into()),
            root_folder: Some(r"Games/DOS Fixture".into()),
            ..Game::default()
        }
    }

    fn mount(drive: char, mount_type: &str, path: &str, media: &str, fs: &str) -> Mount {
        Mount {
            game_id: "dos-game".into(),
            drive_letter: drive,
            filesystem: fs.into(),
            mount_type: mount_type.into(),
            path: path.into(),
            media_type: media.into(),
        }
    }

    #[test]
    fn creates_native_host_paths_and_dos_guest_paths_without_a_shell() {
        let root = Path::new("/library");
        let mounts = [
            mount('d', "Folder", r"Media\CD Files", "CD-ROM/ISO", "ISO"),
            mount('A', "File", "Media/Disk One.img", "Floppy", "FAT"),
            mount('E', "File", r"Media\Game.iso", "CD-ROM/ISO", "ISO"),
        ];
        let plan =
            build_plan(root, &game(), &mounts, &HostPathResolver::default()).expect("DOSBox plan");

        assert_eq!(
            plan.request.executable,
            PathBuf::from("/library/Runtime/dosbox-recorder")
        );
        assert_eq!(
            plan.request.arguments,
            [
                "-noconsole",
                "-conf",
                "/library/Config/dosbox.conf",
                "-noautoexec",
                "-c",
                "@ECHO OFF",
                "-c",
                "CLS",
                "-c",
                "MOUNT C \"/library/Games/DOS Fixture\"",
                "-c",
                "MOUNT D \"/library/Media/CD Files\" -t cdrom -fs iso",
                "-c",
                "IMGMOUNT A \"/library/Media/Disk One.img\" -t floppy -fs fat",
                "-c",
                "IMGMOUNT E \"/library/Media/Game.iso\" -t iso -fs iso",
                "-c",
                "C:",
                "-c",
                "CD \"BIN\"",
                "-c",
                "CALL \"PLAY.BAT\" -fast",
                "-c",
                "EXIT",
            ]
            .into_iter()
            .map(OsString::from)
            .collect::<Vec<_>>()
        );
    }

    #[test]
    fn permits_configuration_owned_autoexec_without_an_application() {
        let mut game = game();
        game.application_path.clear();
        let plan = build_plan(
            Path::new("/library"),
            &game,
            &[],
            &HostPathResolver::default(),
        )
        .expect("autoexec plan");
        assert_eq!(
            plan.request.arguments,
            ["-noconsole", "-conf", "/library/Config/dosbox.conf"]
                .into_iter()
                .map(OsString::from)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn rejects_duplicate_and_reserved_drives() {
        let error = build_plan(
            Path::new("/library"),
            &game(),
            &[mount('c', "Folder", "Media", "", "")],
            &HostPathResolver::default(),
        )
        .err()
        .expect("reserved drive must fail");
        assert!(matches!(
            error,
            DosBoxPlanError::DuplicateDrive { drive: 'C', .. }
        ));
    }

    #[test]
    fn rejects_parent_traversal_from_the_mounted_root() {
        let mut game = game();
        game.application_path = r"Games/DOS Fixture/../outside.exe".into();
        let error = build_plan(
            Path::new("/library"),
            &game,
            &[],
            &HostPathResolver::default(),
        )
        .err()
        .expect("parent traversal must fail");
        assert!(matches!(
            error,
            DosBoxPlanError::InvalidGuestApplicationPath { .. }
        ));
    }
}
