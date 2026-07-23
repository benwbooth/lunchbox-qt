use crate::{LaunchPathError, LaunchPathResolver, LaunchRequest};
use lb_domain::Game;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub(crate) fn build_request(
    launchbox_root: &Path,
    game: &Game,
    path_resolver: &dyn LaunchPathResolver,
) -> Result<LaunchRequest, ScummVmPlanError> {
    let data_folder = game
        .scumm_vm_game_data_folder_path
        .as_deref()
        .filter(|path| !path.trim().is_empty())
        .ok_or_else(|| ScummVmPlanError::MissingGameDataFolder {
            game_id: game.id.clone(),
        })?;
    let data_folder = path_resolver
        .resolve(launchbox_root, data_folder)
        .map_err(|source| ScummVmPlanError::GameDataFolderPath {
            game_id: game.id.clone(),
            source,
        })?;
    if !data_folder.is_dir() {
        return Err(ScummVmPlanError::GameDataFolderDoesNotExist {
            game_id: game.id.clone(),
            path: data_folder,
        });
    }

    let executable = resolve_executable(launchbox_root, game, path_resolver)?;
    let mut request = LaunchRequest::new(&executable);
    request.working_directory = executable
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(Path::to_path_buf);
    request.hide_console = true;

    // These are the semantic argv boundaries recovered from LaunchBox's
    // legacy command string. OsString::push keeps non-Unicode native paths
    // representable instead of converting them through a Windows-shaped
    // string in the UI or domain layers.
    request.arguments.push(OsString::from("--no-console"));
    request
        .arguments
        .push(path_option("--savepath=", &data_folder));
    request
        .arguments
        .push(path_option("--extrapath=", &data_folder));
    request.arguments.push(OsString::from("-p"));
    request.arguments.push(data_folder.as_os_str().to_owned());
    if game.scumm_vm_fullscreen {
        request.arguments.push(OsString::from("-f"));
    }
    if game.scumm_vm_aspect_correction {
        request.arguments.push(OsString::from("--aspect-ratio"));
    }
    if let Some(game_type) = game
        .scumm_vm_game_type
        .as_deref()
        .filter(|game_type| !game_type.trim().is_empty())
    {
        if game_type.starts_with('-') || game_type.chars().any(char::is_control) {
            return Err(ScummVmPlanError::InvalidGameType {
                game_id: game.id.clone(),
                game_type: game_type.to_string(),
            });
        }
        request.arguments.push(OsString::from(game_type));
    }
    Ok(request)
}

fn path_option(name: &str, path: &Path) -> OsString {
    let mut option = OsString::from(name);
    option.push(path.as_os_str());
    option
}

fn resolve_executable(
    launchbox_root: &Path,
    game: &Game,
    path_resolver: &dyn LaunchPathResolver,
) -> Result<PathBuf, ScummVmPlanError> {
    #[cfg(windows)]
    {
        let executable = path_resolver
            .resolve(launchbox_root, r"ThirdParty\ScummVM\ScummVM.exe")
            .map_err(|source| ScummVmPlanError::ExecutablePath {
                game_id: game.id.clone(),
                source,
            })?;
        if !executable.is_file() {
            return Err(ScummVmPlanError::ExecutableDoesNotExist {
                game_id: game.id.clone(),
                path: executable,
            });
        }
        Ok(executable)
    }
    #[cfg(not(windows))]
    {
        let _ = (launchbox_root, game, path_resolver);
        Ok(PathBuf::from("scummvm"))
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ScummVmPlanError {
    #[error("ScummVM game {game_id} has no game-data folder")]
    MissingGameDataFolder { game_id: String },
    #[error("ScummVM game-data folder for game {game_id} cannot be used on this host: {source}")]
    GameDataFolderPath {
        game_id: String,
        #[source]
        source: LaunchPathError,
    },
    #[error("ScummVM game-data folder for game {game_id} does not exist: {path}")]
    GameDataFolderDoesNotExist { game_id: String, path: PathBuf },
    #[error("ScummVM executable for game {game_id} cannot be used on this host: {source}")]
    ExecutablePath {
        game_id: String,
        #[source]
        source: LaunchPathError,
    },
    #[error("ScummVM executable for game {game_id} does not exist: {path}")]
    ExecutableDoesNotExist { game_id: String, path: PathBuf },
    #[error("ScummVM game {game_id} has invalid target ID {game_type:?}")]
    InvalidGameType { game_id: String, game_type: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HostPathResolver;
    use std::fs;

    fn game(data_folder: &str) -> Game {
        Game {
            id: "scumm-game".into(),
            title: "ScummVM Fixture".into(),
            platform: "ScummVM".into(),
            use_scumm_vm: true,
            scumm_vm_game_data_folder_path: Some(data_folder.into()),
            scumm_vm_game_type: Some("monkey2".into()),
            scumm_vm_fullscreen: true,
            scumm_vm_aspect_correction: true,
            ..Game::default()
        }
    }

    #[test]
    fn resolves_mixed_persisted_separators_into_native_semantic_arguments() {
        let directory = tempfile::tempdir().expect("temporary library");
        let data_folder = directory.path().join("Games/Monkey Island 2");
        fs::create_dir_all(&data_folder).expect("game-data folder");
        let request = build_request(
            directory.path(),
            &game(r"Games\Monkey Island 2"),
            &HostPathResolver::default(),
        )
        .expect("ScummVM plan");
        assert_eq!(request.executable, PathBuf::from("scummvm"));
        assert_eq!(
            request.arguments,
            [
                OsString::from("--no-console"),
                path_option("--savepath=", &data_folder),
                path_option("--extrapath=", &data_folder),
                OsString::from("-p"),
                data_folder.as_os_str().to_owned(),
                OsString::from("-f"),
                OsString::from("--aspect-ratio"),
                OsString::from("monkey2"),
            ]
        );
    }

    #[test]
    fn requires_the_game_data_folder_to_exist() {
        let directory = tempfile::tempdir().expect("temporary library");
        assert!(matches!(
            build_request(
                directory.path(),
                &game("Games/Missing"),
                &HostPathResolver::default(),
            ),
            Err(ScummVmPlanError::GameDataFolderDoesNotExist { .. })
        ));
    }

    #[test]
    fn empty_legacy_target_opens_scummvm_without_inventing_auto_detection() {
        let directory = tempfile::tempdir().expect("temporary library");
        fs::create_dir(directory.path().join("Game")).expect("game-data folder");
        let mut game = game("Game");
        game.scumm_vm_game_type = None;
        let request = build_request(directory.path(), &game, &HostPathResolver::default())
            .expect("ScummVM plan without a target");
        assert_eq!(
            request.arguments.last(),
            Some(&OsString::from("--aspect-ratio"))
        );
    }
}
