use std::fs;
use std::path::{Path, PathBuf};

use lb_domain::AdditionalApplication;
use thiserror::Error;

use crate::{
    is_archive_path, ArchiveExtractionError, ArchiveExtractor, LaunchPathError, LaunchPathResolver,
    LaunchResourceLease,
};

pub(crate) struct PreparedM3u {
    pub(crate) launch_path: PathBuf,
    pub(crate) resource_leases: Vec<LaunchResourceLease>,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn prepare_m3u(
    launchbox_root: &Path,
    game_id: &str,
    game_path: &Path,
    additional_applications: &[&AdditionalApplication],
    path_resolver: &dyn LaunchPathResolver,
    auto_extract: bool,
    archive_extractor: &ArchiveExtractor,
) -> Result<Option<PreparedM3u>, M3uPreparationError> {
    let mut discs = additional_applications
        .iter()
        .copied()
        .filter(|application| application.disc.is_some())
        .collect::<Vec<_>>();
    discs.sort_by(|left, right| {
        left.priority
            .cmp(&right.priority)
            .then_with(|| left.id.cmp(&right.id))
    });
    let Some(first_disc) = discs.first() else {
        return Ok(None);
    };

    let first_path = resolve_disc_path(launchbox_root, first_disc, path_resolver)?;
    if first_path != game_path {
        return Err(M3uPreparationError::PrimaryApplicationMismatch {
            game_id: game_id.to_string(),
            game_path: game_path.to_path_buf(),
            first_application_id: first_disc.id.clone(),
            first_path,
        });
    }

    let mut paths = Vec::with_capacity(discs.len());
    let mut resource_leases = Vec::new();
    for application in discs {
        let path = resolve_disc_path(launchbox_root, application, path_resolver)?;
        if auto_extract && is_archive_path(&path) {
            let prepared = archive_extractor.extract(&path).map_err(|source| {
                M3uPreparationError::ArchiveExtraction {
                    application_id: application.id.clone(),
                    path: path.clone(),
                    source: Box::new(source),
                }
            })?;
            paths.push(prepared.launch_path);
            resource_leases.push(prepared.lease);
        } else {
            paths.push(path);
        }
    }

    let playlist_lease =
        LaunchResourceLease::temporary("launchbox-port-m3u-").map_err(|error| {
            M3uPreparationError::TemporaryDirectory {
                game_id: game_id.to_string(),
                message: error.to_string(),
            }
        })?;
    let playlist_file_name = game_path
        .file_stem()
        .filter(|stem| !stem.is_empty())
        .ok_or_else(|| M3uPreparationError::MissingPrimaryFileName {
            game_id: game_id.to_string(),
            path: game_path.to_path_buf(),
        })?;
    let mut playlist_path = playlist_lease.path().join(playlist_file_name);
    playlist_path.set_extension("m3u");

    let mut contents = String::new();
    for path in &paths {
        let line = path
            .to_str()
            .ok_or_else(|| M3uPreparationError::NonUnicodeDiscPath { path: path.clone() })?;
        if line.contains(['\r', '\n']) {
            return Err(M3uPreparationError::UnsafeDiscPath { path: path.clone() });
        }
        contents.push_str(line);
        contents.push('\n');
    }
    fs::write(&playlist_path, contents).map_err(|error| M3uPreparationError::WritePlaylist {
        path: playlist_path.clone(),
        message: error.to_string(),
    })?;
    resource_leases.push(playlist_lease);

    Ok(Some(PreparedM3u {
        launch_path: playlist_path,
        resource_leases,
    }))
}

fn resolve_disc_path(
    launchbox_root: &Path,
    application: &AdditionalApplication,
    path_resolver: &dyn LaunchPathResolver,
) -> Result<PathBuf, M3uPreparationError> {
    if application.application_path.trim().is_empty() {
        return Err(M3uPreparationError::MissingDiscPath {
            application_id: application.id.clone(),
            disc: application.disc,
        });
    }
    path_resolver
        .resolve(launchbox_root, &application.application_path)
        .map_err(|source| M3uPreparationError::DiscPath {
            application_id: application.id.clone(),
            disc: application.disc,
            source,
        })
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum M3uPreparationError {
    #[error("disc application {application_id} (disc {disc:?}) has no application path")]
    MissingDiscPath {
        application_id: String,
        disc: Option<u32>,
    },
    #[error(
        "disc application {application_id} (disc {disc:?}) path cannot be used on this host: {source}"
    )]
    DiscPath {
        application_id: String,
        disc: Option<u32>,
        #[source]
        source: LaunchPathError,
    },
    #[error(
        "game {game_id} path {game_path} does not match the first disc application {first_application_id} at {first_path}"
    )]
    PrimaryApplicationMismatch {
        game_id: String,
        game_path: PathBuf,
        first_application_id: String,
        first_path: PathBuf,
    },
    #[error("game {game_id} primary path has no file name: {path}")]
    MissingPrimaryFileName { game_id: String, path: PathBuf },
    #[error("disc path is not Unicode and cannot be represented in an M3U playlist: {path:?}")]
    NonUnicodeDiscPath { path: PathBuf },
    #[error("disc path contains a line break and cannot be represented safely in M3U: {path:?}")]
    UnsafeDiscPath { path: PathBuf },
    #[error("could not create a temporary M3U directory for game {game_id}: {message}")]
    TemporaryDirectory { game_id: String, message: String },
    #[error("could not write M3U playlist {path}: {message}")]
    WritePlaylist { path: PathBuf, message: String },
    #[error("could not prepare archived disc application {application_id} at {path}: {source}")]
    ArchiveExtraction {
        application_id: String,
        path: PathBuf,
        #[source]
        source: Box<ArchiveExtractionError>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HostPathResolver;

    fn disc(id: &str, path: &str, disc: u32, priority: i32) -> AdditionalApplication {
        AdditionalApplication {
            id: id.into(),
            game_id: "game-id".into(),
            name: format!("Disc {disc}"),
            application_path: path.into(),
            use_emulator: true,
            priority,
            disc: Some(disc),
            ..AdditionalApplication::default()
        }
    }

    #[test]
    fn builds_priority_ordered_playlist_with_a_stable_rom_stem() {
        let root = tempfile::tempdir().expect("create library root");
        let game_path = root.path().join("Games/Game Name (Disc 1).chd");
        let second = disc("disc-2", r"Games\Game Name (Disc 2).chd", 2, 2);
        let first = disc("disc-1", r"Games\Game Name (Disc 1).chd", 1, 1);
        let prepared = prepare_m3u(
            root.path(),
            "game-id",
            &game_path,
            &[&second, &first],
            &HostPathResolver::default(),
            false,
            &ArchiveExtractor::default(),
        )
        .expect("prepare playlist")
        .expect("disc metadata creates a playlist");

        assert_eq!(
            prepared.launch_path.file_name(),
            Some(std::ffi::OsStr::new("Game Name (Disc 1).m3u"))
        );
        assert_eq!(
            fs::read_to_string(&prepared.launch_path).expect("read playlist"),
            format!(
                "{}\n{}\n",
                root.path().join("Games/Game Name (Disc 1).chd").display(),
                root.path().join("Games/Game Name (Disc 2).chd").display(),
            )
        );
    }

    #[test]
    fn requires_the_primary_game_path_to_match_the_first_disc() {
        let root = tempfile::tempdir().expect("create library root");
        let first = disc("disc-1", "Games/Other (Disc 1).chd", 1, 1);
        assert!(matches!(
            prepare_m3u(
                root.path(),
                "game-id",
                &root.path().join("Games/Game (Disc 1).chd"),
                &[&first],
                &HostPathResolver::default(),
                false,
                &ArchiveExtractor::default(),
            ),
            Err(M3uPreparationError::PrimaryApplicationMismatch { .. })
        ));
    }

    #[test]
    fn skips_playlist_creation_without_explicit_disc_metadata() {
        let root = tempfile::tempdir().expect("create library root");
        let application = AdditionalApplication {
            id: "manual".into(),
            game_id: "game-id".into(),
            name: "Manual".into(),
            application_path: "Manual/readme.pdf".into(),
            ..AdditionalApplication::default()
        };
        assert!(prepare_m3u(
            root.path(),
            "game-id",
            &root.path().join("Games/Game.chd"),
            &[&application],
            &HostPathResolver::default(),
            false,
            &ArchiveExtractor::default(),
        )
        .expect("inspect disc metadata")
        .is_none());
    }
}
