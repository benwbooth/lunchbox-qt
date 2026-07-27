use crate::{
    default_platform_folders, index_game_media, GameMediaItem, GameMediaKind, LaunchPathResolver,
};
use lb_domain::{Game, PlatformFolder};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;
use unicode_normalization::UnicodeNormalization;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameMediaMigrationMove {
    pub source: PathBuf,
    pub target: PathBuf,
    pub media_type: String,
    pub game_ids: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GameMediaMigrationPlan {
    pub moves: Vec<GameMediaMigrationMove>,
}

impl GameMediaMigrationPlan {
    pub fn target_for_source(&self, source: &Path) -> Option<&Path> {
        self.moves
            .iter()
            .find(|candidate| candidate.source == source)
            .map(|candidate| candidate.target.as_path())
    }
}

#[derive(Clone, Debug)]
struct ResolvedFolder {
    media_type: String,
    path: PathBuf,
}

/// Plans the image/video portion of LaunchBox's bulk Platform change.
///
/// Every source is a currently indexed regular file owned only by the selected
/// games. Destination paths retain the relative region/subtype/ordinal
/// structure below the configured source folder. Existing targets, portable
/// case collisions, unsafe roots, shared media with an unselected game, and a
/// truncated media scan all fail closed.
pub fn plan_game_media_migration(
    launchbox_root: &Path,
    games: &[Game],
    folders: &[PlatformFolder],
    selected_game_ids: &BTreeSet<String>,
    destination_platform: &str,
    path_resolver: &dyn LaunchPathResolver,
) -> Result<GameMediaMigrationPlan, GameMediaMigrationError> {
    let root = fs::canonicalize(launchbox_root).map_err(|source| GameMediaMigrationError::Io {
        path: launchbox_root.to_path_buf(),
        source,
    })?;
    if !root.is_dir() {
        return Err(GameMediaMigrationError::UnsafeLibraryRoot { path: root });
    }
    let selected = selected_game_ids
        .iter()
        .map(|id| id.to_lowercase())
        .collect::<BTreeSet<_>>();
    if selected.is_empty() {
        return Err(GameMediaMigrationError::EmptySelection);
    }
    let games_by_id = games
        .iter()
        .map(|game| (game.id.to_lowercase(), game))
        .collect::<BTreeMap<_, _>>();
    for id in &selected {
        if !games_by_id.contains_key(id) {
            return Err(GameMediaMigrationError::GameNotFound { id: id.clone() });
        }
    }

    // Display preferences must not hide videos from a destructive migration,
    // so this intentionally uses the complete default media policy.
    let index = index_game_media(&root, games, folders, None, path_resolver);
    let truncations = index
        .report
        .truncated_configured_folders
        .saturating_add(index.report.truncated_folders)
        .saturating_add(index.report.truncated_items);
    if truncations > 0 {
        return Err(GameMediaMigrationError::TruncatedIndex { count: truncations });
    }

    let actual_destination_folders = folders
        .iter()
        .filter(|folder| folder.platform.eq_ignore_ascii_case(destination_platform))
        .cloned()
        .collect::<Vec<_>>();
    let default_destination_folders =
        default_platform_folders(destination_platform).map_err(|error| {
            GameMediaMigrationError::InvalidDestinationPlatform {
                reason: error.to_string(),
            }
        })?;
    let destination_folders = actual_destination_folders
        .iter()
        .chain(default_destination_folders.iter())
        .filter(|folder| !is_supplemental_media_type(&folder.media_type))
        .map(|folder| resolve_destination_folder(&root, folder, path_resolver))
        .collect::<Result<Vec<_>, _>>()?;

    let mut source_folders_by_platform = BTreeMap::<String, Vec<ResolvedFolder>>::new();
    for folder in folders
        .iter()
        .filter(|folder| !is_supplemental_media_type(&folder.media_type))
    {
        let resolved = path_resolver
            .resolve(&root, &folder.folder_path)
            .map_err(|error| GameMediaMigrationError::UnresolvedFolder {
                platform: folder.platform.clone(),
                media_type: folder.media_type.clone(),
                reason: error.to_string(),
            })?;
        let Ok(path) = fs::canonicalize(&resolved) else {
            continue;
        };
        if !path.starts_with(&root) {
            return Err(GameMediaMigrationError::MediaOutsideLibrary { path });
        }
        source_folders_by_platform
            .entry(folder.platform.to_lowercase())
            .or_default()
            .push(ResolvedFolder {
                media_type: folder.media_type.clone(),
                path,
            });
    }

    let mut associations = BTreeMap::<PathBuf, Vec<(&Game, &GameMediaItem)>>::new();
    for game in games {
        let Some(items) = index.items_by_game_id.get(&game.id) else {
            continue;
        };
        for item in items {
            let source =
                fs::canonicalize(&item.path).map_err(|source| GameMediaMigrationError::Io {
                    path: item.path.clone(),
                    source,
                })?;
            if !source.starts_with(&root) {
                return Err(GameMediaMigrationError::MediaOutsideLibrary { path: source });
            }
            associations.entry(source).or_default().push((game, item));
        }
    }

    let mut moves = Vec::new();
    let mut target_keys = BTreeMap::<String, PathBuf>::new();
    for (source, owners) in associations {
        if !owners
            .iter()
            .any(|(game, _)| selected.contains(&game.id.to_lowercase()))
        {
            continue;
        }
        let unselected = owners
            .iter()
            .filter(|(game, _)| !selected.contains(&game.id.to_lowercase()))
            .map(|(game, _)| game.id.clone())
            .collect::<BTreeSet<_>>();
        if !unselected.is_empty() {
            return Err(GameMediaMigrationError::SharedWithUnselectedGames {
                path: source,
                game_ids: unselected.into_iter().collect(),
            });
        }
        let selected_owners = owners
            .iter()
            .filter(|(game, _)| selected.contains(&game.id.to_lowercase()))
            .copied()
            .collect::<Vec<_>>();
        let mut proposed = BTreeMap::<String, (PathBuf, String)>::new();
        for (game, item) in &selected_owners {
            let source_folders = source_folders_by_platform
                .get(&game.platform.to_lowercase())
                .map(Vec::as_slice)
                .unwrap_or_default();
            let source_folder = best_source_folder(source_folders, &source, &item.media_type);
            let (relative, destination_media_type) = if let Some(folder) = source_folder {
                (
                    source
                        .strip_prefix(&folder.path)
                        .expect("matched source folder is a prefix")
                        .to_path_buf(),
                    folder.media_type.as_str(),
                )
            } else {
                (
                    source.file_name().map(PathBuf::from).ok_or_else(|| {
                        GameMediaMigrationError::UnmappedMedia {
                            path: source.clone(),
                            media_type: item.media_type.clone(),
                        }
                    })?,
                    item.media_type.as_str(),
                )
            };
            let destination_folder =
                best_destination_folder(&destination_folders, destination_media_type, item.kind)
                    .ok_or_else(|| GameMediaMigrationError::MissingDestinationFolder {
                        platform: destination_platform.to_string(),
                        media_type: destination_media_type.to_string(),
                    })?;
            let target = destination_folder.path.join(relative);
            proposed.insert(
                portable_path_key(&target),
                (target, item.media_type.clone()),
            );
        }
        if proposed.len() != 1 {
            return Err(GameMediaMigrationError::AmbiguousTarget {
                path: source,
                targets: proposed.into_values().map(|(target, _)| target).collect(),
            });
        }
        let (_, (target, media_type)) = proposed.into_iter().next().expect("one target");
        if target == source {
            continue;
        }
        if target.exists() {
            return Err(GameMediaMigrationError::TargetExists { path: target });
        }
        refuse_portable_case_collision(&target)?;
        let target_key = portable_path_key(&target);
        if let Some(existing) = target_keys.insert(target_key, target.clone()) {
            return Err(GameMediaMigrationError::TargetCollision {
                first: existing,
                second: target,
            });
        }
        let mut game_ids = selected_owners
            .iter()
            .map(|(game, _)| game.id.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        game_ids.sort();
        moves.push(GameMediaMigrationMove {
            source,
            target,
            media_type,
            game_ids,
        });
    }
    moves.sort_by(|left, right| left.source.cmp(&right.source));
    Ok(GameMediaMigrationPlan { moves })
}

/// Creates only the destination directories required by a validated plan.
/// Existing symlinks and non-directories are refused component by component.
/// Empty directories may remain if the later file/XML transaction is refused.
pub fn create_media_migration_directories(
    launchbox_root: &Path,
    plan: &GameMediaMigrationPlan,
) -> Result<Vec<PathBuf>, GameMediaMigrationError> {
    let root = fs::canonicalize(launchbox_root).map_err(|source| GameMediaMigrationError::Io {
        path: launchbox_root.to_path_buf(),
        source,
    })?;
    let mut parents = plan
        .moves
        .iter()
        .filter_map(|entry| entry.target.parent().map(Path::to_path_buf))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    parents.sort_by_key(|path| path.components().count());
    let mut created = Vec::new();
    for parent in parents {
        let relative = parent.strip_prefix(&root).map_err(|_| {
            GameMediaMigrationError::MediaOutsideLibrary {
                path: parent.clone(),
            }
        })?;
        let mut current = root.clone();
        for component in relative.components() {
            let Component::Normal(name) = component else {
                return Err(GameMediaMigrationError::UnsafeDestination { path: parent });
            };
            refuse_portable_directory_collision(&current, name)?;
            current.push(name);
            match fs::symlink_metadata(&current) {
                Ok(metadata)
                    if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() => {}
                Ok(_) => return Err(GameMediaMigrationError::UnsafeDestination { path: current }),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    fs::create_dir(&current).map_err(|source| GameMediaMigrationError::Io {
                        path: current.clone(),
                        source,
                    })?;
                    created.push(current.clone());
                }
                Err(source) => {
                    return Err(GameMediaMigrationError::Io {
                        path: current,
                        source,
                    })
                }
            }
        }
    }
    Ok(created)
}

fn refuse_portable_directory_collision(
    parent: &Path,
    requested_name: &OsStr,
) -> Result<(), GameMediaMigrationError> {
    let expected = portable_component_key(requested_name.to_string_lossy().as_ref());
    let entries = fs::read_dir(parent).map_err(|source| GameMediaMigrationError::Io {
        path: parent.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| GameMediaMigrationError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
        let actual_name = entry.file_name();
        if actual_name.as_os_str() != requested_name
            && portable_component_key(actual_name.to_string_lossy().as_ref()) == expected
        {
            return Err(GameMediaMigrationError::PortableDirectoryCollision {
                requested: parent.join(requested_name),
                existing: entry.path(),
            });
        }
    }
    Ok(())
}

fn resolve_destination_folder(
    root: &Path,
    folder: &PlatformFolder,
    resolver: &dyn LaunchPathResolver,
) -> Result<ResolvedFolder, GameMediaMigrationError> {
    let path = resolver
        .resolve(root, &folder.folder_path)
        .map_err(|error| GameMediaMigrationError::UnresolvedFolder {
            platform: folder.platform.clone(),
            media_type: folder.media_type.clone(),
            reason: error.to_string(),
        })?;
    if !path.is_absolute()
        || !path.starts_with(root)
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(GameMediaMigrationError::UnsafeDestination { path });
    }
    Ok(ResolvedFolder {
        media_type: folder.media_type.clone(),
        path,
    })
}

fn best_source_folder<'a>(
    folders: &'a [ResolvedFolder],
    source: &Path,
    media_type: &str,
) -> Option<&'a ResolvedFolder> {
    folders
        .iter()
        .filter(|folder| source.starts_with(&folder.path))
        .max_by_key(|folder| {
            (
                folder.media_type.eq_ignore_ascii_case(media_type),
                folder.path.components().count(),
            )
        })
}

fn best_destination_folder<'a>(
    folders: &'a [ResolvedFolder],
    media_type: &str,
    kind: GameMediaKind,
) -> Option<&'a ResolvedFolder> {
    folders
        .iter()
        .find(|folder| folder.media_type.eq_ignore_ascii_case(media_type))
        .or_else(|| {
            (kind == GameMediaKind::Video)
                .then(|| {
                    folders
                        .iter()
                        .find(|folder| folder.media_type.eq_ignore_ascii_case("Video"))
                })
                .flatten()
        })
}

fn refuse_portable_case_collision(target: &Path) -> Result<(), GameMediaMigrationError> {
    let Some(parent) = target.parent() else {
        return Err(GameMediaMigrationError::UnsafeDestination {
            path: target.to_path_buf(),
        });
    };
    let Some(file_name) = target.file_name() else {
        return Err(GameMediaMigrationError::UnsafeDestination {
            path: target.to_path_buf(),
        });
    };
    let Ok(entries) = fs::read_dir(parent) else {
        return Ok(());
    };
    let expected = portable_component_key(file_name.to_string_lossy().as_ref());
    for entry in entries {
        let entry = entry.map_err(|source| GameMediaMigrationError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
        if portable_component_key(entry.file_name().to_string_lossy().as_ref()) == expected {
            return Err(GameMediaMigrationError::TargetExists { path: entry.path() });
        }
    }
    Ok(())
}

fn portable_path_key(path: &Path) -> String {
    path.components()
        .map(|component| portable_component_key(component.as_os_str().to_string_lossy().as_ref()))
        .collect::<Vec<_>>()
        .join("/")
}

fn portable_component_key(value: &str) -> String {
    value.nfkc().flat_map(char::to_lowercase).collect()
}

fn is_supplemental_media_type(media_type: &str) -> bool {
    media_type.eq_ignore_ascii_case("Manual") || media_type.eq_ignore_ascii_case("Music")
}

#[derive(Debug, Error)]
pub enum GameMediaMigrationError {
    #[error("the media-migration selection is empty")]
    EmptySelection,
    #[error("selected game {id} was not found")]
    GameNotFound { id: String },
    #[error("the media index was truncated by {count} item(s); refusing a partial migration")]
    TruncatedIndex { count: usize },
    #[error("invalid destination platform: {reason}")]
    InvalidDestinationPlatform { reason: String },
    #[error("could not resolve {platform} {media_type} folder: {reason}")]
    UnresolvedFolder {
        platform: String,
        media_type: String,
        reason: String,
    },
    #[error("media path is outside the transactional LaunchBox root: {path}")]
    MediaOutsideLibrary { path: PathBuf },
    #[error("unsafe media migration destination: {path}")]
    UnsafeDestination { path: PathBuf },
    #[error("unsafe LaunchBox root for media migration: {path}")]
    UnsafeLibraryRoot { path: PathBuf },
    #[error("could not map {media_type} file to a configured source folder: {path}")]
    UnmappedMedia { path: PathBuf, media_type: String },
    #[error("destination platform {platform} has no {media_type} folder")]
    MissingDestinationFolder {
        platform: String,
        media_type: String,
    },
    #[error("media file {path} is also associated with unselected games {game_ids:?}")]
    SharedWithUnselectedGames {
        path: PathBuf,
        game_ids: Vec<String>,
    },
    #[error("media file {path} maps to more than one destination: {targets:?}")]
    AmbiguousTarget {
        path: PathBuf,
        targets: Vec<PathBuf>,
    },
    #[error("media migration target already exists: {path}")]
    TargetExists { path: PathBuf },
    #[error("media migration targets collide portably: {first} and {second}")]
    TargetCollision { first: PathBuf, second: PathBuf },
    #[error(
        "media migration directory collides portably: requested {requested}, existing {existing}"
    )]
    PortableDirectoryCollision {
        requested: PathBuf,
        existing: PathBuf,
    },
    #[error("media migration I/O failed at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HostPathResolver;

    fn game(id: &str, title: &str, platform: &str) -> Game {
        Game {
            id: id.into(),
            title: title.into(),
            platform: platform.into(),
            application_path: format!(r"Games\{platform}\{id}.rom"),
            ..Game::default()
        }
    }

    #[test]
    fn plans_region_and_video_subfolders_without_host_separator_assumptions() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let source_image = root.join("Images/Source/Box - Front/North America");
        let source_video = root.join("Videos/Source/Trailer");
        fs::create_dir_all(&source_image).unwrap();
        fs::create_dir_all(&source_video).unwrap();
        fs::write(source_image.join("Shared Game-01.png"), b"image").unwrap();
        fs::write(source_video.join("Shared Game.mp4"), b"video").unwrap();
        let games = [game("selected", "Shared Game", "Source")];
        let folders = [
            PlatformFolder {
                platform: "Source".into(),
                media_type: "Box - Front".into(),
                folder_path: r"Images\Source\Box - Front".into(),
            },
            PlatformFolder {
                platform: "Source".into(),
                media_type: "Video".into(),
                folder_path: "Videos/Source".into(),
            },
            PlatformFolder {
                platform: "Target".into(),
                media_type: "Box - Front".into(),
                folder_path: "Images/Target/Box - Front".into(),
            },
            PlatformFolder {
                platform: "Target".into(),
                media_type: "Video".into(),
                folder_path: r"Videos\Target".into(),
            },
        ];
        let plan = plan_game_media_migration(
            root,
            &games,
            &folders,
            &["selected".into()].into_iter().collect(),
            "Target",
            &HostPathResolver::default(),
        )
        .unwrap();
        assert_eq!(plan.moves.len(), 2);
        assert!(plan.moves.iter().any(|entry| {
            entry.target == root.join("Images/Target/Box - Front/North America/Shared Game-01.png")
        }));
        assert!(plan
            .moves
            .iter()
            .any(|entry| entry.target == root.join("Videos/Target/Trailer/Shared Game.mp4")));
        let created = create_media_migration_directories(root, &plan).unwrap();
        assert!(!created.is_empty());
        assert!(root.join("Videos/Target/Trailer").is_dir());
    }

    #[test]
    fn refuses_media_shared_with_an_unselected_same_title_game() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        fs::create_dir_all(root.join("Images/Source/Box - Front")).unwrap();
        fs::write(
            root.join("Images/Source/Box - Front/Shared Game.png"),
            b"shared",
        )
        .unwrap();
        let games = [
            game("selected", "Shared Game", "Source"),
            game("retained", "Shared Game", "Source"),
        ];
        let folders = [PlatformFolder {
            platform: "Source".into(),
            media_type: "Box - Front".into(),
            folder_path: r"Images\Source\Box - Front".into(),
        }];
        assert!(matches!(
            plan_game_media_migration(
                root,
                &games,
                &folders,
                &["selected".into()].into_iter().collect(),
                "Target",
                &HostPathResolver::default(),
            ),
            Err(GameMediaMigrationError::SharedWithUnselectedGames { .. })
        ));
    }

    #[test]
    fn refuses_an_existing_destination_before_creating_directories() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let source_directory = root.join("Images/Source/Box - Front");
        let target_directory = root.join("Images/Target/Box - Front");
        fs::create_dir_all(&source_directory).unwrap();
        fs::create_dir_all(&target_directory).unwrap();
        fs::write(source_directory.join("Collision Game.png"), b"source").unwrap();
        let target = target_directory.join("Collision Game.png");
        fs::write(&target, b"target").unwrap();
        let games = [game("collision", "Collision Game", "Source")];
        let folders = [
            PlatformFolder {
                platform: "Source".into(),
                media_type: "Box - Front".into(),
                folder_path: r"Images\Source\Box - Front".into(),
            },
            PlatformFolder {
                platform: "Target".into(),
                media_type: "Box - Front".into(),
                folder_path: r"Images\Target\Box - Front".into(),
            },
        ];

        assert!(matches!(
            plan_game_media_migration(
                root,
                &games,
                &folders,
                &["collision".into()].into_iter().collect(),
                "Target",
                &HostPathResolver::default(),
            ),
            Err(GameMediaMigrationError::TargetExists { path }) if path == target
        ));
        assert_eq!(fs::read(&target).unwrap(), b"target");
    }

    #[cfg(unix)]
    #[test]
    fn refuses_a_symlink_in_a_destination_directory_chain() {
        use std::os::unix::fs::symlink;

        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let outside = tempfile::tempdir().unwrap();
        fs::create_dir(root.join("Images")).unwrap();
        symlink(outside.path(), root.join("Images/Target")).unwrap();
        let plan = GameMediaMigrationPlan {
            moves: vec![GameMediaMigrationMove {
                source: root.join("source.png"),
                target: root.join("Images/Target/Box - Front/game.png"),
                media_type: "Box - Front".into(),
                game_ids: vec!["game".into()],
            }],
        };
        assert!(matches!(
            create_media_migration_directories(root, &plan),
            Err(GameMediaMigrationError::UnsafeDestination { .. })
        ));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn refuses_a_portable_case_collision_in_a_destination_directory_chain() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        fs::create_dir_all(root.join("Images/target")).unwrap();
        let plan = GameMediaMigrationPlan {
            moves: vec![GameMediaMigrationMove {
                source: root.join("source.png"),
                target: root.join("Images/Target/Box - Front/game.png"),
                media_type: "Box - Front".into(),
                game_ids: vec!["game".into()],
            }],
        };
        assert!(matches!(
            create_media_migration_directories(root, &plan),
            Err(GameMediaMigrationError::PortableDirectoryCollision {
                requested,
                existing,
            }) if requested == root.join("Images/Target")
                && existing == root.join("Images/target")
        ));
        assert!(!root.join("Images/Target").exists());
    }
}
