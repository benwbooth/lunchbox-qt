use lb_domain::{
    is_unassigned_emulator_id, AdditionalApplication, AdditionalApplicationEdit, AlternateName,
    CatalogValidationError, CustomField, Emulator, EmulatorConfiguration, EmulatorPlatform, Game,
    GameControllerSupport, GameLaunchConfiguration, GameMetadata, GameSave, GameSaveMetadataEdit,
    Mount, NavigationMetadata, ParentRelationship, PlatformCatalog, PlatformCategory,
    PlatformDefinition, PlatformFolder, PlatformLibrary, Playlist, PlaylistDocument,
    PlaylistFilter, PlaylistGame, ValidationError,
};
#[cfg(test)]
use lb_domain::{
    EMULATOR_PLATFORM_XML_FIELDS, EMULATOR_XML_FIELDS, GAME_SAVE_XML_FIELDS, GAME_XML_FIELDS,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use xmltree::{Element, EmitterConfig, XMLNode};

mod data_index;
mod transaction;

pub use data_index::*;
pub use transaction::*;

/// Read-optimized representation used by the library browser. It retains the
/// LaunchBox fields currently modeled by `Game`, but does not build an XML DOM.
/// Open an individual `PlatformDocument` when an edit must preserve unknown XML.
#[derive(Clone, Debug)]
pub struct LibraryIndex {
    root: PathBuf,
    platforms: Vec<PlatformLibrary>,
}

impl LibraryIndex {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let path = path.as_ref();
        if path.is_file() {
            let platform = load_platform_index(path)?;
            let root = path.parent().unwrap_or(Path::new(".")).to_path_buf();
            return Ok(Self {
                root,
                platforms: vec![platform],
            });
        }

        let platform_dir = find_platform_directory(path)?;
        let platform_files = platform_files(&platform_dir)?;
        let platforms = platform_files
            .iter()
            .map(|file| load_platform_index(file))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            root: path.to_path_buf(),
            platforms,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn platforms(&self) -> &[PlatformLibrary] {
        &self.platforms
    }

    pub fn games(&self) -> impl Iterator<Item = &Game> {
        self.platforms
            .iter()
            .flat_map(|platform| platform.games.iter())
    }

    pub fn additional_applications(&self) -> impl Iterator<Item = &AdditionalApplication> {
        self.platforms
            .iter()
            .flat_map(|platform| platform.additional_applications.iter())
    }

    pub fn mounts(&self) -> impl Iterator<Item = &Mount> {
        self.platforms
            .iter()
            .flat_map(|platform| platform.mounts.iter())
    }

    pub fn alternate_names(&self) -> impl Iterator<Item = &AlternateName> {
        self.platforms
            .iter()
            .flat_map(|platform| platform.alternate_names.iter())
    }

    pub fn custom_fields(&self) -> impl Iterator<Item = &CustomField> {
        self.platforms
            .iter()
            .flat_map(|platform| platform.custom_fields.iter())
    }

    pub fn controller_support(&self) -> impl Iterator<Item = &GameControllerSupport> {
        self.platforms
            .iter()
            .flat_map(|platform| platform.controller_support.iter())
    }

    pub fn game_saves(&self) -> impl Iterator<Item = &GameSave> {
        self.platforms
            .iter()
            .flat_map(|platform| platform.game_saves.iter())
    }
}

#[derive(Clone, Debug)]
pub struct PlatformDocument {
    source_path: PathBuf,
    source_revision: Option<FileRevision>,
    root: Element,
    library: PlatformLibrary,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtomicSaveReport {
    pub target: PathBuf,
    pub backup: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectoryRevision {
    pub file_count: u64,
    pub directory_count: u64,
    pub byte_len: u64,
    pub sha256: String,
}

impl DirectoryRevision {
    pub fn read(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let supplied = path.as_ref();
        let metadata = fs::symlink_metadata(supplied).map_err(|source| StorageError::Read {
            path: supplied.to_path_buf(),
            source,
        })?;
        if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
            return Err(StorageError::AtomicTargetNotDirectory {
                path: supplied.to_path_buf(),
            });
        }
        let root = fs::canonicalize(supplied).map_err(|source| StorageError::Read {
            path: supplied.to_path_buf(),
            source,
        })?;
        let mut revision = DirectoryRevisionBuilder::default();
        revision.visit(&root, Path::new(""))?;
        Ok(revision.finish())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtomicDirectorySaveReport {
    pub target: PathBuf,
    pub backup: PathBuf,
}

#[derive(Default)]
struct DirectoryRevisionBuilder {
    file_count: u64,
    directory_count: u64,
    byte_len: u64,
    digest: Sha256,
}

impl DirectoryRevisionBuilder {
    fn visit(&mut self, root: &Path, relative: &Path) -> Result<(), StorageError> {
        let current = root.join(relative);
        let mut entries = fs::read_dir(&current)
            .map_err(|source| StorageError::Read {
                path: current.clone(),
                source,
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|source| StorageError::Read {
                path: current.clone(),
                source,
            })?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| StorageError::AtomicDirectoryNonUnicode { path: entry.path() })?;
            let child_relative = relative.join(name);
            let child = root.join(&child_relative);
            let metadata = fs::symlink_metadata(&child).map_err(|source| StorageError::Read {
                path: child.clone(),
                source,
            })?;
            let portable = child_relative
                .to_str()
                .expect("all directory components were checked as Unicode")
                .replace('\\', "/");
            if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
                self.directory_count = self.directory_count.saturating_add(1);
                self.update_entry(b'D', &portable, 0);
                self.visit(root, &child_relative)?;
            } else if metadata.file_type().is_file() && !metadata.file_type().is_symlink() {
                self.file_count = self.file_count.saturating_add(1);
                self.byte_len = self.byte_len.saturating_add(metadata.len());
                self.update_entry(b'F', &portable, metadata.len());
                let mut file = fs::File::open(&child).map_err(|source| StorageError::Read {
                    path: child.clone(),
                    source,
                })?;
                let mut buffer = vec![0_u8; 1024 * 1024];
                loop {
                    let read = file
                        .read(&mut buffer)
                        .map_err(|source| StorageError::Read {
                            path: child.clone(),
                            source,
                        })?;
                    if read == 0 {
                        break;
                    }
                    self.digest.update(&buffer[..read]);
                }
            } else {
                return Err(StorageError::AtomicDirectoryUnsafeEntry { path: child });
            }
        }
        Ok(())
    }

    fn update_entry(&mut self, kind: u8, path: &str, byte_len: u64) {
        self.digest.update([kind]);
        self.digest
            .update(u64::try_from(path.len()).unwrap_or(u64::MAX).to_le_bytes());
        self.digest.update(path.as_bytes());
        self.digest.update(byte_len.to_le_bytes());
    }

    fn finish(self) -> DirectoryRevision {
        let digest = self.digest.finalize();
        let mut sha256 = String::with_capacity(digest.len() * 2);
        for byte in digest {
            write!(&mut sha256, "{byte:02x}").expect("writing to a String cannot fail");
        }
        DirectoryRevision {
            file_count: self.file_count,
            directory_count: self.directory_count,
            byte_len: self.byte_len,
            sha256,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NewGameMetadata {
    pub database_id: Option<u32>,
    pub manual_path: Option<String>,
    pub notes: Option<String>,
    pub developer: Option<String>,
    pub genre: Option<String>,
    pub max_players: Option<u32>,
    pub play_mode: Option<String>,
    pub publisher: Option<String>,
    pub rating: Option<String>,
    pub region: Option<String>,
    pub release_date: Option<String>,
    pub release_type: Option<String>,
    pub status: Option<String>,
    pub version: Option<String>,
    pub wikipedia_url: Option<String>,
    pub video_url: Option<String>,
    pub community_star_rating: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewGame {
    pub id: String,
    pub title: String,
    pub platform: String,
    pub application_path: String,
    /// `None` inherits the platform default. The all-zero LaunchBox sentinel
    /// explicitly launches the game directly, while any other value pins a
    /// configured emulator.
    pub emulator_id: Option<String>,
    pub metadata: NewGameMetadata,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemovedPlatformCatalogRecords {
    pub platform: PlatformDefinition,
    pub folders: Vec<PlatformFolder>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemovedEmulatorConfigurationRecords {
    pub emulator: Emulator,
    pub platforms: Vec<EmulatorPlatform>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemovedPlatformCategoryRelationships {
    pub removed_placements: usize,
    pub detached_children: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemovedPlaylistRelationships {
    pub removed_placements: usize,
    pub detached_children: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CombinedGames {
    pub root_game: Game,
    pub removed_games: Vec<Game>,
    pub created_version_applications: Vec<AdditionalApplication>,
    pub retargeted_additional_applications: usize,
    pub retargeted_game_records: usize,
    pub retargeted_clone_relationships: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExpandedGames {
    pub root_game: Game,
    pub created_games: Vec<Game>,
    pub removed_version_applications: Vec<AdditionalApplication>,
    pub reassigned_game_saves: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CombinedGameReferenceRemap {
    pub navigation_references: usize,
    pub playlist_rows_retargeted: usize,
    pub playlist_rows_deduplicated: usize,
    pub blacklist_rows_removed: usize,
}

/// One row submitted by an editor for a repeated platform-document record.
/// Existing rows carry their per-game source ordinal so the lossless editor
/// can update or remove the exact XML element without discarding unknown
/// children. New rows use `None` and must follow all retained source rows.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexedPlatformRecordEdit<T> {
    pub source_index: Option<usize>,
    pub record: T,
}

/// A metadata-only edit for one existing `<GameSave>` row. The source index is
/// scoped to the owning game's save rows, in document order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexedGameSaveMetadataEdit {
    pub source_index: usize,
    pub metadata: GameSaveMetadataEdit,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum GameReferenceKind {
    AdditionalApplication,
    Mount,
    AlternateName,
    CustomField,
    ControllerSupport,
    GameSave,
    CloneOf,
    PlaylistGame,
    NavigationLastGame,
    ImportBlacklist,
}

impl std::fmt::Display for GameReferenceKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::AdditionalApplication => "additional application",
            Self::Mount => "DOSBox mount",
            Self::AlternateName => "alternate name",
            Self::CustomField => "custom field",
            Self::ControllerSupport => "controller support",
            Self::GameSave => "game save",
            Self::CloneOf => "clone relationship",
            Self::PlaylistGame => "playlist entry",
            Self::NavigationLastGame => "navigation last-game selection",
            Self::ImportBlacklist => "import blacklist entry",
        };
        formatter.write_str(name)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameReference {
    pub kind: GameReferenceKind,
    pub source_path: PathBuf,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PlatformReferenceKind {
    Game,
    EmulatorMapping,
    EmulatorDefault,
    ParentChild,
    ParentTarget,
    PlaylistGame,
    PlaylistFilter,
    NavigationLastSelectedChild,
    ControllerAssociation,
    FrontendSetting,
}

impl std::fmt::Display for PlatformReferenceKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Game => "game",
            Self::EmulatorMapping => "emulator mapping",
            Self::EmulatorDefault => "emulator default",
            Self::ParentChild => "parent relationship child",
            Self::ParentTarget => "parent relationship target",
            Self::PlaylistGame => "playlist game",
            Self::PlaylistFilter => "playlist filter",
            Self::NavigationLastSelectedChild => "navigation selection",
            Self::ControllerAssociation => "controller association",
            Self::FrontendSetting => "frontend setting",
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformReference {
    pub kind: PlatformReferenceKind,
    pub source_path: PathBuf,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EmulatorReferenceKind {
    Game,
    AdditionalApplication,
}

impl std::fmt::Display for EmulatorReferenceKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Game => "game",
            Self::AdditionalApplication => "additional application",
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmulatorReference {
    pub kind: EmulatorReferenceKind,
    pub source_path: PathBuf,
    pub detail: String,
}

/// Freshly scans every modeled document that can refer to a platform. The
/// platform's own catalog row and owned `PlatformFolder` rows are intentionally
/// excluded because a lifecycle transaction removes those records itself.
pub fn find_platform_references(
    scope: impl AsRef<Path>,
    platform_name: &str,
) -> Result<Vec<PlatformReference>, StorageError> {
    let scope = scope.as_ref();
    let mut references = if scope.is_file() {
        platform_game_platform_references(&LibraryIndex::load(scope)?, platform_name)
    } else {
        let data = LaunchBoxDataIndex::load(scope)?;
        let mut references = platform_game_platform_references(data.platforms(), platform_name);

        if let Some(configuration) = data.emulator_configuration() {
            for mapping in configuration
                .platforms
                .iter()
                .filter(|mapping| platform_names_equal(&mapping.platform, platform_name))
            {
                references.push(PlatformReference {
                    kind: PlatformReferenceKind::EmulatorMapping,
                    source_path: configuration.source_path.clone(),
                    detail: mapping.emulator_id.clone(),
                });
            }
            for emulator in configuration.emulators.iter().filter(|emulator| {
                emulator
                    .default_platform
                    .as_deref()
                    .is_some_and(|name| platform_names_equal(name, platform_name))
            }) {
                references.push(PlatformReference {
                    kind: PlatformReferenceKind::EmulatorDefault,
                    source_path: configuration.source_path.clone(),
                    detail: emulator.title.clone(),
                });
            }
        }

        for relationship in data.parents() {
            if relationship
                .platform_name
                .as_deref()
                .is_some_and(|name| platform_names_equal(name, platform_name))
            {
                references.push(PlatformReference {
                    kind: PlatformReferenceKind::ParentChild,
                    source_path: data.data_root().join("Parents.xml"),
                    detail: platform_name.to_string(),
                });
            }
            if relationship
                .parent_platform_name
                .as_deref()
                .is_some_and(|name| platform_names_equal(name, platform_name))
            {
                references.push(PlatformReference {
                    kind: PlatformReferenceKind::ParentTarget,
                    source_path: data.data_root().join("Parents.xml"),
                    detail: platform_name.to_string(),
                });
            }
        }

        for document in data.playlists() {
            for game in document
                .games
                .iter()
                .filter(|game| platform_names_equal(&game.game_platform, platform_name))
            {
                references.push(PlatformReference {
                    kind: PlatformReferenceKind::PlaylistGame,
                    source_path: document.source_path.clone(),
                    detail: format!("{} ({})", document.playlist.metadata.name, game.game_title),
                });
            }
            for filter in document.filters.iter().filter(|filter| {
                filter.field_key.eq_ignore_ascii_case("Platform")
                    && platform_names_equal(&filter.value, platform_name)
            }) {
                references.push(PlatformReference {
                    kind: PlatformReferenceKind::PlaylistFilter,
                    source_path: document.source_path.clone(),
                    detail: format!(
                        "{} ({})",
                        document.playlist.metadata.name, filter.comparison_type_key
                    ),
                });
            }
            if document
                .playlist
                .metadata
                .last_selected_child
                .as_deref()
                .is_some_and(|name| platform_names_equal(name, platform_name))
            {
                references.push(PlatformReference {
                    kind: PlatformReferenceKind::NavigationLastSelectedChild,
                    source_path: document.source_path.clone(),
                    detail: document.playlist.metadata.name.clone(),
                });
            }
        }

        if let Some(catalog) = data.platform_catalog() {
            for metadata in catalog
                .platforms
                .iter()
                .filter(|platform| !platform_names_equal(&platform.metadata.name, platform_name))
                .map(|platform| &platform.metadata)
                .chain(catalog.categories.iter().map(|category| &category.metadata))
                .filter(|metadata| {
                    metadata
                        .last_selected_child
                        .as_deref()
                        .is_some_and(|name| platform_names_equal(name, platform_name))
                })
            {
                references.push(PlatformReference {
                    kind: PlatformReferenceKind::NavigationLastSelectedChild,
                    source_path: catalog.source_path.clone(),
                    detail: metadata.name.clone(),
                });
            }
        }

        for controller in data.game_controllers().iter().filter(|controller| {
            controller
                .associated_platforms
                .as_deref()
                .is_some_and(|names| {
                    names
                        .split(';')
                        .any(|name| platform_names_equal(name.trim(), platform_name))
                })
        }) {
            references.push(PlatformReference {
                kind: PlatformReferenceKind::ControllerAssociation,
                source_path: data.data_root().join("GameControllers.xml"),
                detail: controller.name.clone(),
            });
        }

        for settings in [data.settings(), data.big_box_settings()]
            .into_iter()
            .flatten()
        {
            for entry in settings.entries.iter().filter(|entry| {
                entry.key.to_ascii_lowercase().contains("platform")
                    && platform_names_equal(&entry.value, platform_name)
            }) {
                references.push(PlatformReference {
                    kind: PlatformReferenceKind::FrontendSetting,
                    source_path: settings.source_path.clone(),
                    detail: entry.key.clone(),
                });
            }
        }
        references
    };
    references.sort_by(|left, right| {
        left.source_path
            .cmp(&right.source_path)
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.detail.cmp(&right.detail))
    });
    Ok(references)
}

fn platform_names_equal(left: &str, right: &str) -> bool {
    left.to_lowercase() == right.to_lowercase()
}

fn platform_game_platform_references(
    library: &LibraryIndex,
    platform_name: &str,
) -> Vec<PlatformReference> {
    library
        .platforms()
        .iter()
        .flat_map(|platform| {
            platform
                .games
                .iter()
                .filter(|game| platform_names_equal(&game.platform, platform_name))
                .map(|game| PlatformReference {
                    kind: PlatformReferenceKind::Game,
                    source_path: platform.source_path.clone(),
                    detail: format!("{} ({})", game.title, game.id),
                })
        })
        .collect()
}

/// Freshly scans every modeled platform document for records that pin an
/// emulator ID. Owned `<EmulatorPlatform>` rows are excluded because deleting
/// an emulator removes those mappings in the same transaction.
pub fn find_emulator_references(
    scope: impl AsRef<Path>,
    emulator_id: &str,
) -> Result<Vec<EmulatorReference>, StorageError> {
    let scope = scope.as_ref();
    let library = if scope.is_file() {
        LibraryIndex::load(scope)?
    } else {
        LaunchBoxDataIndex::load(scope)?.platforms().clone()
    };
    let mut references = Vec::new();
    for platform in library.platforms() {
        references.extend(
            platform
                .games
                .iter()
                .filter(|game| {
                    game.emulator_id.as_deref().is_some_and(|id| {
                        !is_unassigned_emulator_id(id) && id.eq_ignore_ascii_case(emulator_id)
                    })
                })
                .map(|game| EmulatorReference {
                    kind: EmulatorReferenceKind::Game,
                    source_path: platform.source_path.clone(),
                    detail: format!("{} ({})", game.title, game.id),
                }),
        );
        references.extend(
            platform
                .additional_applications
                .iter()
                .filter(|application| {
                    application
                        .emulator_id
                        .as_deref()
                        .is_some_and(|id| id.eq_ignore_ascii_case(emulator_id))
                })
                .map(|application| EmulatorReference {
                    kind: EmulatorReferenceKind::AdditionalApplication,
                    source_path: platform.source_path.clone(),
                    detail: format!("{} ({})", application.name, application.id),
                }),
        );
    }
    references.sort_by(|left, right| {
        left.source_path
            .cmp(&right.source_path)
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.detail.cmp(&right.detail))
    });
    Ok(references)
}

/// Freshly scans every modeled reference-bearing document in a library. A
/// direct platform-file scope checks only that platform document.
pub fn find_game_references(
    scope: impl AsRef<Path>,
    game_id: &str,
) -> Result<Vec<GameReference>, StorageError> {
    let scope = scope.as_ref();
    let mut references = if scope.is_file() {
        let library = LibraryIndex::load(scope)?;
        platform_game_references(&library, game_id)
    } else {
        let data = LaunchBoxDataIndex::load(scope)?;
        let mut references = platform_game_references(data.platforms(), game_id);
        for document in data.playlists() {
            if document.playlist.metadata.last_game_id.as_deref() == Some(game_id) {
                references.push(GameReference {
                    kind: GameReferenceKind::NavigationLastGame,
                    source_path: document.source_path.clone(),
                    detail: document.playlist.metadata.name.clone(),
                });
            }
            for game in document.games.iter().filter(|game| game.game_id == game_id) {
                references.push(GameReference {
                    kind: GameReferenceKind::PlaylistGame,
                    source_path: document.source_path.clone(),
                    detail: format!("{} ({})", document.playlist.metadata.name, game.game_title),
                });
            }
        }
        if let Some(catalog) = data.platform_catalog() {
            for metadata in catalog
                .platforms
                .iter()
                .map(|platform| &platform.metadata)
                .chain(catalog.categories.iter().map(|category| &category.metadata))
                .filter(|metadata| metadata.last_game_id.as_deref() == Some(game_id))
            {
                references.push(GameReference {
                    kind: GameReferenceKind::NavigationLastGame,
                    source_path: catalog.source_path.clone(),
                    detail: metadata.name.clone(),
                });
            }
        }
        if data.ignored_game_ids().iter().any(|id| id == game_id) {
            references.push(GameReference {
                kind: GameReferenceKind::ImportBlacklist,
                source_path: data.data_root().join("ImportBlacklist.xml"),
                detail: game_id.to_string(),
            });
        }
        references
    };
    references.sort_by(|left, right| {
        left.source_path
            .cmp(&right.source_path)
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.detail.cmp(&right.detail))
    });
    Ok(references)
}

fn platform_game_references(library: &LibraryIndex, game_id: &str) -> Vec<GameReference> {
    let mut references = Vec::new();
    for platform in library.platforms() {
        references.extend(platform_library_game_references(platform, game_id));
    }
    references
}

fn platform_library_game_references(
    platform: &PlatformLibrary,
    game_id: &str,
) -> Vec<GameReference> {
    let mut references = Vec::new();
    for game in platform
        .games
        .iter()
        .filter(|game| game.clone_of.as_deref() == Some(game_id))
    {
        references.push(GameReference {
            kind: GameReferenceKind::CloneOf,
            source_path: platform.source_path.clone(),
            detail: format!("{} ({})", game.title, game.id),
        });
    }
    for application in platform
        .additional_applications
        .iter()
        .filter(|application| application.game_id == game_id)
    {
        references.push(GameReference {
            kind: GameReferenceKind::AdditionalApplication,
            source_path: platform.source_path.clone(),
            detail: application.name.clone(),
        });
    }
    for mount in platform
        .mounts
        .iter()
        .filter(|mount| mount.game_id == game_id)
    {
        references.push(GameReference {
            kind: GameReferenceKind::Mount,
            source_path: platform.source_path.clone(),
            detail: format!("drive {} ({})", mount.drive_letter, mount.path),
        });
    }
    for alternate in platform
        .alternate_names
        .iter()
        .filter(|alternate| alternate.game_id == game_id)
    {
        references.push(GameReference {
            kind: GameReferenceKind::AlternateName,
            source_path: platform.source_path.clone(),
            detail: alternate.name.clone(),
        });
    }
    for field in platform
        .custom_fields
        .iter()
        .filter(|field| field.game_id == game_id)
    {
        references.push(GameReference {
            kind: GameReferenceKind::CustomField,
            source_path: platform.source_path.clone(),
            detail: field.name.clone(),
        });
    }
    for support in platform
        .controller_support
        .iter()
        .filter(|support| support.game_id == game_id)
    {
        references.push(GameReference {
            kind: GameReferenceKind::ControllerSupport,
            source_path: platform.source_path.clone(),
            detail: support.controller_id.clone(),
        });
    }
    for save in platform
        .game_saves
        .iter()
        .filter(|save| save.game_id == game_id)
    {
        references.push(GameReference {
            kind: GameReferenceKind::GameSave,
            source_path: platform.source_path.clone(),
            detail: save.title.clone().unwrap_or_else(|| "untitled save".into()),
        });
    }
    references
}

fn summarize_reference_kinds(references: &[GameReference]) -> String {
    let mut counts = std::collections::BTreeMap::<GameReferenceKind, usize>::new();
    for reference in references {
        *counts.entry(reference.kind).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(kind, count)| format!("{count} {kind}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Lossless editor for every non-platform XML document family in LaunchBox's
/// `Data` directory. Unknown elements and their ordering stay in the DOM, while
/// every mutation and save is checked by the same typed parser used by
/// `LaunchBoxDataIndex`.
#[derive(Clone, Debug)]
pub struct AuxiliaryDocument {
    kind: AuxiliaryDocumentKind,
    source_path: PathBuf,
    source_revision: Option<FileRevision>,
    root: Element,
}

impl AuxiliaryDocument {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let path = path.as_ref();
        let kind = AuxiliaryDocumentKind::infer(path)?;
        Self::load_as(kind, path)
    }

    pub fn load_as(
        kind: AuxiliaryDocumentKind,
        path: impl AsRef<Path>,
    ) -> Result<Self, StorageError> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|source| StorageError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let revision = FileRevision::from_bytes(&bytes);
        let mut document = Self::from_reader(kind, path, bytes.as_slice())?;
        document.source_revision = Some(revision);
        Ok(document)
    }

    pub fn from_reader(
        kind: AuxiliaryDocumentKind,
        source_path: impl Into<PathBuf>,
        reader: impl Read,
    ) -> Result<Self, StorageError> {
        let source_path = source_path.into();
        let root = Element::parse(reader).map_err(|source| StorageError::Parse {
            path: source_path.clone(),
            source,
        })?;
        data_index::validate_auxiliary_root(kind, &source_path, &root)?;
        Ok(Self {
            kind,
            source_path,
            source_revision: None,
            root,
        })
    }

    pub fn kind(&self) -> AuxiliaryDocumentKind {
        self.kind
    }

    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    pub fn source_revision(&self) -> Option<&FileRevision> {
        self.source_revision.as_ref()
    }

    pub fn root(&self) -> &Element {
        &self.root
    }

    pub fn record_count(&self, record_name: &str) -> usize {
        elements_named(&self.root, record_name).count()
    }

    pub fn platform_catalog(&self) -> Result<PlatformCatalog, StorageError> {
        self.ensure_operation_kind("read platform catalog", AuxiliaryDocumentKind::Platforms)?;
        data_index::parse_platform_catalog(&self.source_path, &self.root)
    }

    pub fn emulator_configuration(&self) -> Result<EmulatorConfiguration, StorageError> {
        self.ensure_operation_kind(
            "read emulator configuration",
            AuxiliaryDocumentKind::Emulators,
        )?;
        data_index::parse_emulators(&self.source_path, &self.root)
    }

    /// Adds one complete emulator and its per-platform mappings without
    /// interpreting the stored executable path. A newly selected default
    /// mapping clears the former default for that platform in the same DOM
    /// mutation, matching LaunchBox's one-default-per-platform invariant.
    pub fn add_emulator(
        &mut self,
        emulator: Emulator,
        platforms: Vec<EmulatorPlatform>,
    ) -> Result<(), StorageError> {
        self.ensure_operation_kind("add emulator", AuxiliaryDocumentKind::Emulators)?;
        emulator.validate()?;
        let configuration = self.emulator_configuration()?;
        if configuration
            .emulators
            .iter()
            .any(|existing| existing.id.eq_ignore_ascii_case(&emulator.id))
        {
            return Err(StorageError::DuplicateEmulatorId {
                id: emulator.id.clone(),
            });
        }
        validate_emulator_platform_records(&emulator.id, &platforms)?;
        let default_platforms = platforms
            .iter()
            .filter(|mapping| mapping.default)
            .map(|mapping| mapping.platform.to_lowercase())
            .collect::<BTreeSet<_>>();

        self.mutate(move |root| {
            clear_peer_default_emulators(root, &emulator.id, &default_platforms);
            let insertion = emulator_record_insertion_index(root, "Emulator");
            root.children
                .insert(insertion, XMLNode::Element(emulator_element(&emulator)));
            for platform in platforms {
                let insertion = emulator_record_insertion_index(root, "EmulatorPlatform");
                root.children.insert(
                    insertion,
                    XMLNode::Element(emulator_platform_element(&platform)),
                );
            }
            Ok(())
        })
    }

    /// Replaces every mutable emulator field and its ordered, source-indexed
    /// mappings. Retained mapping elements keep unknown children and original
    /// document order; new rows follow retained rows.
    pub fn set_emulator(
        &mut self,
        emulator_id: &str,
        emulator: Emulator,
        platform_edits: Vec<IndexedPlatformRecordEdit<EmulatorPlatform>>,
    ) -> Result<(), StorageError> {
        self.ensure_operation_kind("edit emulator", AuxiliaryDocumentKind::Emulators)?;
        emulator.validate()?;
        let configuration = self.emulator_configuration()?;
        let original = configuration
            .emulators
            .iter()
            .find(|candidate| candidate.id.eq_ignore_ascii_case(emulator_id))
            .cloned()
            .ok_or_else(|| StorageError::EmulatorNotFound {
                id: emulator_id.to_string(),
            })?;
        if emulator.id != original.id {
            return Err(StorageError::ImmutableEmulatorId {
                expected: original.id,
                actual: emulator.id,
            });
        }
        let exact_id = original.id.clone();
        let original_platforms = configuration
            .platforms
            .iter()
            .filter(|platform| platform.emulator_id.eq_ignore_ascii_case(&exact_id))
            .cloned()
            .collect::<Vec<_>>();
        validate_emulator_platform_edits(&exact_id, original_platforms.len(), &platform_edits)?;
        let edited_platforms = platform_edits
            .iter()
            .map(|edit| edit.record.clone())
            .collect::<Vec<_>>();
        validate_emulator_platform_records(&exact_id, &edited_platforms)?;
        let default_platforms = edited_platforms
            .iter()
            .filter(|mapping| mapping.default)
            .map(|mapping| mapping.platform.to_lowercase())
            .collect::<BTreeSet<_>>();

        self.mutate(move |root| {
            let emulator_indices = root
                .children
                .iter()
                .enumerate()
                .filter_map(|(index, node)| {
                    let element = node.as_element()?;
                    (element.name == "Emulator"
                        && child_text(element, "ID")
                            .is_some_and(|id| id.eq_ignore_ascii_case(&exact_id)))
                    .then_some(index)
                })
                .collect::<Vec<_>>();
            let emulator_index = exactly_one_emulator_record(&exact_id, &emulator_indices)?;
            let element = root.children[emulator_index]
                .as_mut_element()
                .expect("emulator index must identify an element");
            update_emulator_element(element, &original, &emulator);

            let root_indices = emulator_platform_indices(root, &exact_id);
            if root_indices.len() != original_platforms.len() {
                return Err(StorageError::InvalidEmulatorPlatformEdit {
                    id: exact_id.clone(),
                    reason: format!(
                        "typed/XML source count mismatch ({} versus {})",
                        original_platforms.len(),
                        root_indices.len()
                    ),
                });
            }
            let mut retained = vec![false; original_platforms.len()];
            for edit in &platform_edits {
                let Some(source_index) = edit.source_index else {
                    continue;
                };
                retained[source_index] = true;
                let element = root.children[root_indices[source_index]]
                    .as_mut_element()
                    .expect("emulator platform index must identify an element");
                update_emulator_platform_element(
                    element,
                    &original_platforms[source_index],
                    &edit.record,
                );
            }
            for source_index in (0..root_indices.len()).rev() {
                if !retained[source_index] {
                    root.children.remove(root_indices[source_index]);
                }
            }
            for edit in platform_edits
                .iter()
                .filter(|edit| edit.source_index.is_none())
            {
                let insertion = emulator_record_insertion_index(root, "EmulatorPlatform");
                root.children.insert(
                    insertion,
                    XMLNode::Element(emulator_platform_element(&edit.record)),
                );
            }
            clear_peer_default_emulators(root, &exact_id, &default_platforms);
            Ok(())
        })
    }

    /// Removes an emulator and all mappings it owns. Callers must perform a
    /// fresh external reference scan before committing the changed document.
    pub fn remove_emulator(
        &mut self,
        emulator_id: &str,
    ) -> Result<RemovedEmulatorConfigurationRecords, StorageError> {
        self.ensure_operation_kind("remove emulator", AuxiliaryDocumentKind::Emulators)?;
        let configuration = self.emulator_configuration()?;
        let emulator = configuration
            .emulators
            .iter()
            .find(|candidate| candidate.id.eq_ignore_ascii_case(emulator_id))
            .cloned()
            .ok_or_else(|| StorageError::EmulatorNotFound {
                id: emulator_id.to_string(),
            })?;
        let exact_id = emulator.id.clone();
        let platforms = configuration
            .platforms
            .iter()
            .filter(|mapping| mapping.emulator_id.eq_ignore_ascii_case(&exact_id))
            .cloned()
            .collect::<Vec<_>>();
        self.mutate(|root| {
            root.children.retain(|node| {
                let Some(element) = node.as_element() else {
                    return true;
                };
                match element.name.as_str() {
                    "Emulator" => child_text(element, "ID")
                        .is_none_or(|id| !id.eq_ignore_ascii_case(&exact_id)),
                    "EmulatorPlatform" => child_text(element, "Emulator")
                        .is_none_or(|id| !id.eq_ignore_ascii_case(&exact_id)),
                    _ => true,
                }
            });
            Ok(())
        })?;
        Ok(RemovedEmulatorConfigurationRecords {
            emulator,
            platforms,
        })
    }

    pub fn new_playlist(
        source_path: impl Into<PathBuf>,
        playlist: Playlist,
        filters: Vec<PlaylistFilter>,
        games: Vec<PlaylistGame>,
    ) -> Result<Self, StorageError> {
        let source_path = source_path.into();
        if AuxiliaryDocumentKind::infer(&source_path)? != AuxiliaryDocumentKind::Playlist {
            return Err(StorageError::UnsupportedAuxiliaryDocument { path: source_path });
        }
        let document = PlaylistDocument {
            source_path: source_path.clone(),
            playlist,
            filters,
            games,
        };
        document.validate()?;
        validate_playlist_row_identity(&document)?;
        let mut root = Element::new("LaunchBox");
        root.children
            .push(XMLNode::Element(playlist_element(&document.playlist)));
        root.children.extend(
            document
                .filters
                .iter()
                .map(playlist_filter_element)
                .map(XMLNode::Element),
        );
        root.children.extend(
            document
                .games
                .iter()
                .map(playlist_game_element)
                .map(XMLNode::Element),
        );
        data_index::validate_auxiliary_root(AuxiliaryDocumentKind::Playlist, &source_path, &root)?;
        Ok(Self {
            kind: AuxiliaryDocumentKind::Playlist,
            source_path,
            source_revision: None,
            root,
        })
    }

    pub fn playlist_document(&self) -> Result<PlaylistDocument, StorageError> {
        self.ensure_operation_kind("read playlist", AuxiliaryDocumentKind::Playlist)?;
        data_index::parse_playlist(&self.source_path, &self.root)
    }

    /// Rewrites modeled references after several standalone games are
    /// combined into one retained root game.
    ///
    /// Playlist membership is deduplicated instead of leaving multiple rows
    /// for the same root ID. Import-blacklist entries for removed game IDs are
    /// discarded; mapping them to the retained root would incorrectly hide a
    /// still-present game from future import review.
    pub fn remap_combined_game_references(
        &mut self,
        removed_game_ids: &[String],
        root_game: &Game,
    ) -> Result<CombinedGameReferenceRemap, StorageError> {
        let removed = removed_game_ids
            .iter()
            .map(|id| id.to_lowercase())
            .collect::<std::collections::BTreeSet<_>>();
        if removed.is_empty() {
            return Ok(CombinedGameReferenceRemap::default());
        }
        let root_id = root_game.id.clone();
        let root_title = root_game.title.clone();
        let root_platform = root_game.platform.clone();
        let kind = self.kind;
        let mut report = CombinedGameReferenceRemap::default();
        self.mutate(|root| {
            match kind {
                AuxiliaryDocumentKind::Platforms => {
                    for element in root.children.iter_mut().filter_map(XMLNode::as_mut_element) {
                        if !matches!(element.name.as_str(), "Platform" | "PlatformCategory") {
                            continue;
                        }
                        if optional_child(element, "LastGameId")
                            .is_some_and(|id| removed.contains(&id.to_lowercase()))
                        {
                            set_child_text(element, "LastGameId", &root_id);
                            report.navigation_references =
                                report.navigation_references.saturating_add(1);
                        }
                    }
                }
                AuxiliaryDocumentKind::Playlist => {
                    for element in root.children.iter_mut().filter_map(XMLNode::as_mut_element) {
                        if element.name == "Playlist"
                            && optional_child(element, "LastGameId")
                                .is_some_and(|id| removed.contains(&id.to_lowercase()))
                        {
                            set_child_text(element, "LastGameId", &root_id);
                            report.navigation_references =
                                report.navigation_references.saturating_add(1);
                        }
                    }

                    let root_row_exists = root.children.iter().any(|node| {
                        node.as_element().is_some_and(|element| {
                            element.name == "PlaylistGame"
                                && optional_child(element, "GameId")
                                    .is_some_and(|id| id.eq_ignore_ascii_case(&root_id))
                        })
                    });
                    let mut retained_retargeted_row = root_row_exists;
                    root.children.retain_mut(|node| {
                        let Some(element) = node.as_mut_element() else {
                            return true;
                        };
                        if element.name != "PlaylistGame" {
                            return true;
                        }
                        let is_removed = optional_child(element, "GameId")
                            .is_some_and(|id| removed.contains(&id.to_lowercase()));
                        if !is_removed {
                            if optional_child(element, "GameId")
                                .is_some_and(|id| id.eq_ignore_ascii_case(&root_id))
                            {
                                set_child_text(element, "GameTitle", &root_title);
                                set_child_text(element, "GamePlatform", &root_platform);
                            }
                            return true;
                        }
                        if retained_retargeted_row {
                            report.playlist_rows_deduplicated =
                                report.playlist_rows_deduplicated.saturating_add(1);
                            return false;
                        }
                        set_child_text(element, "GameId", &root_id);
                        set_child_text(element, "GameTitle", &root_title);
                        set_child_text(element, "GamePlatform", &root_platform);
                        retained_retargeted_row = true;
                        report.playlist_rows_retargeted =
                            report.playlist_rows_retargeted.saturating_add(1);
                        true
                    });
                }
                AuxiliaryDocumentKind::ImportBlacklist => {
                    root.children.retain(|node| {
                        let matches = node.as_element().is_some_and(|element| {
                            element.name == "IgnoredGameId"
                                && optional_child(element, "GameId")
                                    .is_some_and(|id| removed.contains(&id.to_lowercase()))
                        });
                        if matches {
                            report.blacklist_rows_removed =
                                report.blacklist_rows_removed.saturating_add(1);
                        }
                        !matches
                    });
                }
                actual => {
                    return Err(StorageError::UnsupportedAuxiliaryOperation {
                        operation: "remap combined game references",
                        expected: AuxiliaryDocumentKind::Playlist,
                        actual,
                    });
                }
            }
            Ok(())
        })?;
        Ok(report)
    }

    /// Updates one playlist and its ordered filter/game rows without
    /// rebuilding retained XML elements. Playlist ID and unique name are kept
    /// immutable because the recovered 13.27 plugin contract exposes both as
    /// getter-only identity fields.
    pub fn set_playlist(
        &mut self,
        playlist_id: &str,
        playlist: Playlist,
        filter_edits: Vec<IndexedPlatformRecordEdit<PlaylistFilter>>,
        game_edits: Vec<IndexedPlatformRecordEdit<PlaylistGame>>,
    ) -> Result<(), StorageError> {
        self.ensure_operation_kind("edit playlist", AuxiliaryDocumentKind::Playlist)?;
        playlist.validate()?;
        let original = self.playlist_document()?;
        if original.playlist.id != playlist_id {
            return Err(StorageError::PlaylistNotFound {
                id: playlist_id.to_string(),
            });
        }
        if playlist.id != original.playlist.id {
            return Err(StorageError::ImmutablePlaylistId {
                expected: original.playlist.id,
                actual: playlist.id,
            });
        }
        if playlist.metadata.name != original.playlist.metadata.name {
            return Err(StorageError::ImmutablePlaylistName {
                expected: original.playlist.metadata.name,
                actual: playlist.metadata.name,
            });
        }
        validate_indexed_playlist_filters(original.filters.len(), &filter_edits)?;
        validate_indexed_playlist_games(playlist_id, original.games.len(), &game_edits)?;

        self.mutate(move |root| {
            let playlist_indices = record_indices(root, "Playlist", None);
            let playlist_index = exactly_one_editable_record("Playlist", None, &playlist_indices)?;
            let element = root.children[playlist_index]
                .as_mut_element()
                .expect("playlist index must identify an element");
            update_playlist_element(element, &original.playlist, &playlist);

            replace_playlist_filter_rows(root, &original.filters, &filter_edits)?;
            replace_playlist_game_rows(root, &original.games, &game_edits)?;
            Ok(())
        })
    }

    /// Adds one typed platform and all of its folder records in a single
    /// lossless DOM mutation. `FolderPath` values are lexical LaunchBox data;
    /// this method never interprets them as native paths.
    pub fn add_platform_definition(
        &mut self,
        platform: PlatformDefinition,
        folders: Vec<PlatformFolder>,
    ) -> Result<(), StorageError> {
        self.ensure_operation_kind("add platform", AuxiliaryDocumentKind::Platforms)?;
        platform.validate()?;
        let name = platform.metadata.name.clone();
        let catalog = self.platform_catalog()?;
        if catalog
            .platforms
            .iter()
            .any(|existing| existing.metadata.name.eq_ignore_ascii_case(&name))
        {
            return Err(StorageError::DuplicatePlatformName { name });
        }

        let mut media_types = std::collections::BTreeSet::new();
        for folder in &folders {
            folder.validate()?;
            if folder.platform != name {
                return Err(StorageError::PlatformFolderOwnerMismatch {
                    expected: name,
                    actual: folder.platform.clone(),
                });
            }
            if !media_types.insert(folder.media_type.to_lowercase()) {
                return Err(StorageError::DuplicatePlatformFolderMediaType {
                    platform: name,
                    media_type: folder.media_type.clone(),
                });
            }
        }

        self.mutate(move |root| {
            let insertion = catalog_record_insertion_index(root, "Platform");
            root.children.insert(
                insertion,
                XMLNode::Element(platform_definition_element(&platform)),
            );
            for folder in folders {
                let insertion = catalog_record_insertion_index(root, "PlatformFolder");
                root.children.insert(
                    insertion,
                    XMLNode::Element(platform_folder_element(&folder)),
                );
            }
            Ok(())
        })
    }

    /// Replaces the mutable fields of one platform and its ordered folder rows
    /// without rebuilding retained XML elements. The public LaunchBox 13.27
    /// contract exposes `IPlatform.Name` as getter-only, so identity changes
    /// are deliberately rejected until a runtime oracle establishes the full
    /// cross-document rename behavior.
    pub fn set_platform_definition(
        &mut self,
        platform_name: &str,
        platform: PlatformDefinition,
        folder_edits: Vec<IndexedPlatformRecordEdit<PlatformFolder>>,
    ) -> Result<(), StorageError> {
        self.ensure_operation_kind("edit platform", AuxiliaryDocumentKind::Platforms)?;
        platform.validate()?;
        let catalog = self.platform_catalog()?;
        let original = catalog
            .platforms
            .iter()
            .find(|candidate| candidate.metadata.name.eq_ignore_ascii_case(platform_name))
            .cloned()
            .ok_or_else(|| StorageError::PlatformNotFound {
                name: platform_name.to_string(),
            })?;
        let exact_name = original.metadata.name.clone();
        if platform.metadata.name != exact_name {
            return Err(StorageError::ImmutablePlatformName {
                expected: exact_name,
                actual: platform.metadata.name,
            });
        }

        let original_folders = catalog
            .folders
            .iter()
            .filter(|folder| folder.platform.eq_ignore_ascii_case(&exact_name))
            .cloned()
            .collect::<Vec<_>>();
        validate_indexed_platform_folder_edits(&exact_name, original_folders.len(), &folder_edits)?;
        let mut media_types = std::collections::BTreeSet::new();
        for edit in &folder_edits {
            edit.record.validate()?;
            if edit.record.platform != exact_name {
                return Err(StorageError::PlatformFolderOwnerMismatch {
                    expected: exact_name,
                    actual: edit.record.platform.clone(),
                });
            }
            if !media_types.insert(edit.record.media_type.to_lowercase()) {
                return Err(StorageError::DuplicatePlatformFolderMediaType {
                    platform: exact_name,
                    media_type: edit.record.media_type.clone(),
                });
            }
        }

        self.mutate(move |root| {
            let platform_indices = root
                .children
                .iter()
                .enumerate()
                .filter_map(|(index, node)| {
                    let element = node.as_element()?;
                    (element.name == "Platform"
                        && child_text(element, "Name")
                            .is_some_and(|name| name.eq_ignore_ascii_case(&exact_name)))
                    .then_some(index)
                })
                .collect::<Vec<_>>();
            let platform_index = match platform_indices.as_slice() {
                [index] => *index,
                [] => {
                    return Err(StorageError::PlatformNotFound {
                        name: exact_name.clone(),
                    })
                }
                _ => {
                    return Err(StorageError::DuplicatePlatformName {
                        name: exact_name.clone(),
                    })
                }
            };
            let element = root.children[platform_index]
                .as_mut_element()
                .expect("platform index must identify an element");
            update_platform_definition_element(element, &original, &platform);

            let root_indices = root
                .children
                .iter()
                .enumerate()
                .filter_map(|(index, node)| {
                    let element = node.as_element()?;
                    (element.name == "PlatformFolder"
                        && child_text(element, "Platform")
                            .is_some_and(|name| name.eq_ignore_ascii_case(&exact_name)))
                    .then_some(index)
                })
                .collect::<Vec<_>>();
            if root_indices.len() != original_folders.len() {
                return Err(StorageError::InvalidPlatformFolderEdit {
                    platform: exact_name.clone(),
                    reason: format!(
                        "typed/XML source count mismatch ({} versus {})",
                        original_folders.len(),
                        root_indices.len()
                    ),
                });
            }

            let mut retained = vec![false; original_folders.len()];
            for edit in &folder_edits {
                let Some(source_index) = edit.source_index else {
                    continue;
                };
                retained[source_index] = true;
                let original_folder = &original_folders[source_index];
                let element = root.children[root_indices[source_index]]
                    .as_mut_element()
                    .expect("platform folder index must identify an element");
                if original_folder.media_type != edit.record.media_type {
                    set_child_text(element, "MediaType", &edit.record.media_type);
                }
                if original_folder.folder_path != edit.record.folder_path {
                    set_child_text(element, "FolderPath", &edit.record.folder_path);
                }
            }
            for source_index in (0..root_indices.len()).rev() {
                if !retained[source_index] {
                    root.children.remove(root_indices[source_index]);
                }
            }
            for edit in folder_edits
                .iter()
                .filter(|edit| edit.source_index.is_none())
            {
                let insertion = catalog_record_insertion_index(root, "PlatformFolder");
                root.children.insert(
                    insertion,
                    XMLNode::Element(platform_folder_element(&edit.record)),
                );
            }
            Ok(())
        })
    }

    /// Removes one platform definition and every folder record it owns while
    /// preserving unrelated and unknown XML nodes byte-semantically in the DOM.
    pub fn remove_platform_definition(
        &mut self,
        platform_name: &str,
    ) -> Result<RemovedPlatformCatalogRecords, StorageError> {
        self.ensure_operation_kind("remove platform", AuxiliaryDocumentKind::Platforms)?;
        let catalog = self.platform_catalog()?;
        let platform = catalog
            .platforms
            .iter()
            .find(|platform| platform.metadata.name.eq_ignore_ascii_case(platform_name))
            .cloned()
            .ok_or_else(|| StorageError::PlatformNotFound {
                name: platform_name.to_string(),
            })?;
        let exact_name = platform.metadata.name.clone();
        let folders = catalog
            .folders
            .iter()
            .filter(|folder| folder.platform.eq_ignore_ascii_case(&exact_name))
            .cloned()
            .collect::<Vec<_>>();

        self.mutate(|root| {
            root.children.retain(|node| {
                let Some(element) = node.as_element() else {
                    return true;
                };
                match element.name.as_str() {
                    "Platform" => child_text(element, "Name")
                        .is_none_or(|name| !name.eq_ignore_ascii_case(&exact_name)),
                    "PlatformFolder" => child_text(element, "Platform")
                        .is_none_or(|name| !name.eq_ignore_ascii_case(&exact_name)),
                    _ => true,
                }
            });
            Ok(())
        })?;
        Ok(RemovedPlatformCatalogRecords { platform, folders })
    }

    /// Adds a category definition without inventing hierarchy placement.
    /// Callers stage the corresponding `Parents.xml` mutation in the same
    /// library transaction.
    pub fn add_platform_category(
        &mut self,
        category: PlatformCategory,
    ) -> Result<(), StorageError> {
        self.ensure_operation_kind("add platform category", AuxiliaryDocumentKind::Platforms)?;
        category.validate()?;
        let name = category.metadata.name.clone();
        let catalog = self.platform_catalog()?;
        if catalog
            .categories
            .iter()
            .any(|existing| existing.metadata.name.eq_ignore_ascii_case(&name))
        {
            return Err(StorageError::DuplicatePlatformCategoryName { name });
        }
        self.mutate(move |root| {
            let insertion = catalog_record_insertion_index(root, "PlatformCategory");
            root.children.insert(
                insertion,
                XMLNode::Element(platform_category_element(&category)),
            );
            Ok(())
        })
    }

    /// Updates the mutable category fields in place. The public 13.27
    /// `IPlatformCategory.Name` contract is getter-only, matching platform
    /// identity, so renaming remains gated on a runtime-proven cascade.
    pub fn set_platform_category(
        &mut self,
        category_name: &str,
        category: PlatformCategory,
    ) -> Result<(), StorageError> {
        self.ensure_operation_kind("edit platform category", AuxiliaryDocumentKind::Platforms)?;
        category.validate()?;
        let catalog = self.platform_catalog()?;
        let original = catalog
            .categories
            .iter()
            .find(|candidate| candidate.metadata.name.eq_ignore_ascii_case(category_name))
            .cloned()
            .ok_or_else(|| StorageError::PlatformCategoryNotFound {
                name: category_name.to_string(),
            })?;
        let exact_name = original.metadata.name.clone();
        if category.metadata.name != exact_name {
            return Err(StorageError::ImmutablePlatformCategoryName {
                expected: exact_name,
                actual: category.metadata.name,
            });
        }

        self.mutate(move |root| {
            let matches = root
                .children
                .iter()
                .enumerate()
                .filter_map(|(index, node)| {
                    let element = node.as_element()?;
                    (element.name == "PlatformCategory"
                        && child_text(element, "Name")
                            .is_some_and(|name| name.eq_ignore_ascii_case(&exact_name)))
                    .then_some(index)
                })
                .collect::<Vec<_>>();
            let index = match matches.as_slice() {
                [index] => *index,
                [] => {
                    return Err(StorageError::PlatformCategoryNotFound {
                        name: exact_name.clone(),
                    })
                }
                _ => {
                    return Err(StorageError::DuplicatePlatformCategoryName {
                        name: exact_name.clone(),
                    })
                }
            };
            let element = root.children[index]
                .as_mut_element()
                .expect("category index must identify an element");
            update_platform_category_element(element, &original, &category);
            Ok(())
        })
    }

    pub fn remove_platform_category(
        &mut self,
        category_name: &str,
    ) -> Result<PlatformCategory, StorageError> {
        self.ensure_operation_kind("remove platform category", AuxiliaryDocumentKind::Platforms)?;
        let catalog = self.platform_catalog()?;
        let category = catalog
            .categories
            .iter()
            .find(|candidate| candidate.metadata.name.eq_ignore_ascii_case(category_name))
            .cloned()
            .ok_or_else(|| StorageError::PlatformCategoryNotFound {
                name: category_name.to_string(),
            })?;
        let exact_name = category.metadata.name.clone();
        self.mutate(|root| {
            root.children.retain(|node| {
                let Some(element) = node.as_element() else {
                    return true;
                };
                element.name != "PlatformCategory"
                    || child_text(element, "Name")
                        .is_none_or(|name| !name.eq_ignore_ascii_case(&exact_name))
            });
            Ok(())
        })?;
        Ok(category)
    }

    pub fn parent_relationships(&self) -> Result<Vec<ParentRelationship>, StorageError> {
        self.ensure_operation_kind("read parent relationships", AuxiliaryDocumentKind::Parents)?;
        data_index::parse_parents(&self.root)
    }

    /// Replaces only the hierarchy placements whose child is the selected
    /// category. Retained source-indexed rows keep unknown XML children.
    pub fn set_platform_category_parents(
        &mut self,
        category_name: &str,
        edits: Vec<IndexedPlatformRecordEdit<ParentRelationship>>,
    ) -> Result<(), StorageError> {
        self.ensure_operation_kind(
            "edit platform category parents",
            AuxiliaryDocumentKind::Parents,
        )?;
        let original = self
            .parent_relationships()?
            .into_iter()
            .filter(|relationship| {
                relationship
                    .platform_category_name
                    .as_deref()
                    .is_some_and(|name| name.eq_ignore_ascii_case(category_name))
            })
            .collect::<Vec<_>>();
        validate_indexed_category_parent_edits(category_name, original.len(), &edits)?;

        self.mutate(move |root| {
            let root_indices = root
                .children
                .iter()
                .enumerate()
                .filter_map(|(index, node)| {
                    let element = node.as_element()?;
                    (element.name == "Parent"
                        && child_text(element, "PlatformCategoryName")
                            .is_some_and(|name| name.eq_ignore_ascii_case(category_name)))
                    .then_some(index)
                })
                .collect::<Vec<_>>();
            if root_indices.len() != original.len() {
                return Err(StorageError::InvalidPlatformCategoryParentEdit {
                    category: category_name.to_string(),
                    reason: format!(
                        "typed/XML source count mismatch ({} versus {})",
                        original.len(),
                        root_indices.len()
                    ),
                });
            }

            let mut retained = vec![false; original.len()];
            for edit in &edits {
                let Some(source_index) = edit.source_index else {
                    continue;
                };
                retained[source_index] = true;
                let element = root.children[root_indices[source_index]]
                    .as_mut_element()
                    .expect("parent index must identify an element");
                update_parent_relationship_element(element, &original[source_index], &edit.record);
            }
            for source_index in (0..root_indices.len()).rev() {
                if !retained[source_index] {
                    root.children.remove(root_indices[source_index]);
                }
            }
            for edit in edits.iter().filter(|edit| edit.source_index.is_none()) {
                root.children
                    .push(XMLNode::Element(parent_relationship_element(&edit.record)));
            }
            Ok(())
        })
    }

    /// Removes the deleted category's own placements and detaches each direct
    /// child placement to the root, matching LaunchBox's deletion warning.
    /// Other fields and unknown XML on those child rows are retained.
    pub fn remove_platform_category_relationships(
        &mut self,
        category_name: &str,
    ) -> Result<RemovedPlatformCategoryRelationships, StorageError> {
        self.ensure_operation_kind(
            "remove platform category relationships",
            AuxiliaryDocumentKind::Parents,
        )?;
        let mut removed_placements = 0usize;
        let mut detached_children = 0usize;
        self.mutate(|root| {
            root.children.retain_mut(|node| {
                let Some(element) = node.as_mut_element() else {
                    return true;
                };
                if element.name != "Parent" {
                    return true;
                }
                if child_text(element, "PlatformCategoryName")
                    .is_some_and(|name| name.eq_ignore_ascii_case(category_name))
                {
                    removed_placements = removed_placements.saturating_add(1);
                    return false;
                }
                if child_text(element, "ParentPlatformCategoryName")
                    .is_some_and(|name| name.eq_ignore_ascii_case(category_name))
                {
                    clear_child_text_preserving_element(element, "ParentPlatformCategoryName");
                    detached_children = detached_children.saturating_add(1);
                }
                true
            });
            Ok(())
        })?;
        Ok(RemovedPlatformCategoryRelationships {
            removed_placements,
            detached_children,
        })
    }

    /// Replaces only hierarchy placements whose child is the selected
    /// playlist. Source indices are local to that playlist's retained rows.
    pub fn set_playlist_parents(
        &mut self,
        playlist_id: &str,
        edits: Vec<IndexedPlatformRecordEdit<ParentRelationship>>,
    ) -> Result<(), StorageError> {
        self.ensure_operation_kind("edit playlist parents", AuxiliaryDocumentKind::Parents)?;
        let original = self
            .parent_relationships()?
            .into_iter()
            .filter(|relationship| {
                relationship
                    .playlist_id
                    .as_deref()
                    .is_some_and(|id| id.eq_ignore_ascii_case(playlist_id))
            })
            .collect::<Vec<_>>();
        validate_indexed_playlist_parent_edits(playlist_id, original.len(), &edits)?;

        self.mutate(move |root| {
            let root_indices = root
                .children
                .iter()
                .enumerate()
                .filter_map(|(index, node)| {
                    let element = node.as_element()?;
                    (element.name == "Parent"
                        && child_text(element, "PlaylistId")
                            .is_some_and(|id| id.eq_ignore_ascii_case(playlist_id)))
                    .then_some(index)
                })
                .collect::<Vec<_>>();
            if root_indices.len() != original.len() {
                return Err(StorageError::InvalidPlaylistParentEdit {
                    id: playlist_id.to_string(),
                    reason: format!(
                        "typed/XML source count mismatch ({} versus {})",
                        original.len(),
                        root_indices.len()
                    ),
                });
            }

            let mut retained = vec![false; original.len()];
            for edit in &edits {
                let Some(source_index) = edit.source_index else {
                    continue;
                };
                retained[source_index] = true;
                let element = root.children[root_indices[source_index]]
                    .as_mut_element()
                    .expect("parent index must identify an element");
                update_parent_relationship_element(element, &original[source_index], &edit.record);
            }
            for source_index in (0..root_indices.len()).rev() {
                if !retained[source_index] {
                    root.children.remove(root_indices[source_index]);
                }
            }
            for edit in edits.iter().filter(|edit| edit.source_index.is_none()) {
                root.children
                    .push(XMLNode::Element(parent_relationship_element(&edit.record)));
            }
            Ok(())
        })
    }

    /// Removes every placement of the deleted playlist and detaches direct
    /// child placements to root. Playlist games are only membership records;
    /// this never deletes game records or media.
    pub fn remove_playlist_relationships(
        &mut self,
        playlist_id: &str,
    ) -> Result<RemovedPlaylistRelationships, StorageError> {
        self.ensure_operation_kind(
            "remove playlist relationships",
            AuxiliaryDocumentKind::Parents,
        )?;
        let mut removed_placements = 0usize;
        let mut detached_children = 0usize;
        self.mutate(|root| {
            root.children.retain_mut(|node| {
                let Some(element) = node.as_mut_element() else {
                    return true;
                };
                if element.name != "Parent" {
                    return true;
                }
                if child_text(element, "PlaylistId")
                    .is_some_and(|id| id.eq_ignore_ascii_case(playlist_id))
                {
                    removed_placements = removed_placements.saturating_add(1);
                    return false;
                }
                if child_text(element, "ParentPlaylistId")
                    .is_some_and(|id| id.eq_ignore_ascii_case(playlist_id))
                {
                    clear_child_text_preserving_element(element, "ParentPlaylistId");
                    detached_children = detached_children.saturating_add(1);
                }
                true
            });
            Ok(())
        })?;
        Ok(RemovedPlaylistRelationships {
            removed_placements,
            detached_children,
        })
    }

    /// Removes cache rows owned by a playlist that is being deleted. Cache
    /// documents are optional transaction participants; unrelated rows remain
    /// byte-semantically present in the lossless DOM.
    pub fn remove_playlist_list_cache_items(
        &mut self,
        playlist_id: &str,
    ) -> Result<usize, StorageError> {
        self.ensure_operation_kind(
            "remove playlist list-cache items",
            AuxiliaryDocumentKind::ListCache,
        )?;
        let mut removed = 0usize;
        self.mutate(|root| {
            root.children.retain(|node| {
                let Some(element) = node.as_element() else {
                    return true;
                };
                let matches = element.name == "ListCacheItem"
                    && child_text(element, "PlaylistId")
                        .is_some_and(|id| id.eq_ignore_ascii_case(playlist_id));
                if matches {
                    removed = removed.saturating_add(1);
                }
                !matches
            });
            Ok(())
        })?;
        Ok(removed)
    }

    /// Updates a field on the only record with `record_name`.
    pub fn set_single_record_field(
        &mut self,
        record_name: &str,
        field: &str,
        value: &str,
    ) -> Result<(), StorageError> {
        self.mutate(|root| {
            let matches = record_indices(root, record_name, None);
            let index = exactly_one_editable_record(record_name, None, &matches)?;
            let record = root.children[index]
                .as_mut_element()
                .expect("record index always identifies an element");
            set_child_text(record, field, value);
            Ok(())
        })
    }

    /// Updates a field on exactly one record selected by a child field/value.
    pub fn set_record_field(
        &mut self,
        record_name: &str,
        selector_field: &str,
        selector_value: &str,
        field: &str,
        value: &str,
    ) -> Result<(), StorageError> {
        self.mutate(|root| {
            let selector = Some((selector_field, selector_value));
            let matches = record_indices(root, record_name, selector);
            let index = exactly_one_editable_record(record_name, selector, &matches)?;
            let record = root.children[index]
                .as_mut_element()
                .expect("record index always identifies an element");
            set_child_text(record, field, value);
            Ok(())
        })
    }

    /// Adds a complete record and rejects the mutation if the typed document
    /// parser considers the resulting document invalid.
    pub fn append_record(&mut self, record: Element) -> Result<(), StorageError> {
        self.mutate(move |root| {
            root.children.push(XMLNode::Element(record));
            Ok(())
        })
    }

    /// Removes exactly one selected record. Required singleton records cannot
    /// be removed because semantic validation runs before the mutation commits.
    pub fn remove_record(
        &mut self,
        record_name: &str,
        selector_field: &str,
        selector_value: &str,
    ) -> Result<(), StorageError> {
        self.mutate(|root| {
            let selector = Some((selector_field, selector_value));
            let matches = record_indices(root, record_name, selector);
            let index = exactly_one_editable_record(record_name, selector, &matches)?;
            root.children.remove(index);
            Ok(())
        })
    }

    pub fn write_to(&self, writer: impl Write) -> Result<(), StorageError> {
        write_xml_root(&self.root, writer)
    }

    pub fn to_xml_bytes(&self) -> Result<Vec<u8>, StorageError> {
        let mut bytes = Vec::new();
        self.write_to(&mut bytes)?;
        data_index::validate_auxiliary_root(
            self.kind,
            &self.source_path,
            &parse_xml_root(&self.source_path, bytes.as_slice())?,
        )?;
        Ok(bytes)
    }

    pub fn save_new(&self, path: impl AsRef<Path>) -> Result<(), StorageError> {
        let path = path.as_ref();
        self.ensure_target_kind(path)?;
        save_new_bytes(path, &self.to_xml_bytes()?)
    }

    pub fn save_atomic(&self) -> Result<AtomicSaveReport, StorageError> {
        self.save_atomic_to(&self.source_path)
    }

    pub fn save_atomic_to(
        &self,
        target: impl AsRef<Path>,
    ) -> Result<AtomicSaveReport, StorageError> {
        let target = target.as_ref();
        self.ensure_target_kind(target)?;
        let bytes = self.to_xml_bytes()?;
        let expected = if target == self.source_path {
            self.source_revision.as_ref()
        } else {
            None
        };
        save_atomic_bytes_if_revision(target, &bytes, expected, |candidate| {
            let root = parse_xml_root(target, candidate)?;
            data_index::validate_auxiliary_root(self.kind, target, &root)
        })
    }

    fn mutate(
        &mut self,
        edit: impl FnOnce(&mut Element) -> Result<(), StorageError>,
    ) -> Result<(), StorageError> {
        let mut candidate = self.root.clone();
        edit(&mut candidate)?;
        data_index::validate_auxiliary_root(self.kind, &self.source_path, &candidate)?;
        self.root = candidate;
        Ok(())
    }

    fn ensure_target_kind(&self, target: &Path) -> Result<(), StorageError> {
        let actual = AuxiliaryDocumentKind::infer(target)?;
        if actual != self.kind {
            return Err(StorageError::AuxiliaryDocumentKindMismatch {
                path: target.to_path_buf(),
                expected: self.kind,
                actual,
            });
        }
        Ok(())
    }

    fn ensure_operation_kind(
        &self,
        operation: &'static str,
        expected: AuxiliaryDocumentKind,
    ) -> Result<(), StorageError> {
        if self.kind == expected {
            Ok(())
        } else {
            Err(StorageError::UnsupportedAuxiliaryOperation {
                operation,
                expected,
                actual: self.kind,
            })
        }
    }
}

pub fn restore_auxiliary_backup(
    backup: impl AsRef<Path>,
    target: impl AsRef<Path>,
) -> Result<AtomicSaveReport, StorageError> {
    let backup = backup.as_ref();
    let target = target.as_ref();
    let kind = AuxiliaryDocumentKind::infer(target)?;
    let bytes = fs::read(backup).map_err(|source| StorageError::Read {
        path: backup.to_path_buf(),
        source,
    })?;
    save_atomic_bytes(target, &bytes, |candidate| {
        let root = parse_xml_root(target, candidate)?;
        data_index::validate_auxiliary_root(kind, target, &root)
    })
}

/// Restores a lossless platform backup through the same validated atomic path.
/// The file being replaced is itself retained as a new backup, so recovery is
/// reversible.
pub fn restore_platform_backup(
    backup: impl AsRef<Path>,
    target: impl AsRef<Path>,
) -> Result<AtomicSaveReport, StorageError> {
    let backup = backup.as_ref();
    let target = target.as_ref();
    let bytes = fs::read(backup).map_err(|source| StorageError::Read {
        path: backup.to_path_buf(),
        source,
    })?;
    save_atomic_bytes(target, &bytes, |candidate| {
        PlatformDocument::from_reader(target, candidate).map(|_| ())
    })
}

/// Atomically replaces one existing regular file from another regular file
/// while requiring exact pre-inspected revisions for both paths.
///
/// This is the narrow filesystem primitive used by regular-file save
/// adapters. It rejects symlinks and directories, streams through a sibling
/// staging file, preserves target permissions, and retains the replaced target
/// as an exact sibling backup.
pub fn replace_regular_file_from_source_if_revisions(
    source: impl AsRef<Path>,
    source_expected: &FileRevision,
    target: impl AsRef<Path>,
    target_expected: &FileRevision,
) -> Result<AtomicSaveReport, StorageError> {
    let supplied_source = source.as_ref();
    let source_metadata =
        fs::symlink_metadata(supplied_source).map_err(|source| StorageError::Read {
            path: supplied_source.to_path_buf(),
            source,
        })?;
    if !source_metadata.file_type().is_file() {
        return Err(StorageError::AtomicSourceNotFile {
            path: supplied_source.to_path_buf(),
        });
    }
    let source = fs::canonicalize(supplied_source).map_err(|source| StorageError::Read {
        path: supplied_source.to_path_buf(),
        source,
    })?;

    let supplied_target = target.as_ref();
    let target_metadata =
        fs::symlink_metadata(supplied_target).map_err(|source| StorageError::Write {
            path: supplied_target.to_path_buf(),
            source,
        })?;
    if !target_metadata.file_type().is_file() {
        return Err(StorageError::AtomicTargetNotFile {
            path: supplied_target.to_path_buf(),
        });
    }
    let target = fs::canonicalize(supplied_target).map_err(|source| StorageError::Read {
        path: supplied_target.to_path_buf(),
        source,
    })?;
    if source == target {
        return Err(StorageError::AtomicSourceEqualsTarget { path: target });
    }

    let source_actual = FileRevision::read(&source)?;
    if source_actual != *source_expected {
        return Err(StorageError::AtomicSourceConflict {
            path: source,
            expected: source_expected.clone(),
            actual: source_actual,
        });
    }
    let target_actual = FileRevision::read(&target)?;
    if target_actual != *target_expected {
        return Err(StorageError::WriteConflict {
            path: target,
            expected: target_expected.clone(),
            actual: target_actual,
        });
    }

    let (temporary_path, mut temporary) = create_unique_sibling(&target, "temporary", true)?;
    let staged = fs::File::open(&source)
        .and_then(|mut source_file| std::io::copy(&mut source_file, &mut temporary))
        .and_then(|_| temporary.flush())
        .and_then(|()| temporary.set_permissions(target_metadata.permissions()))
        .and_then(|()| temporary.sync_all());
    if let Err(source) = staged {
        let _ = fs::remove_file(&temporary_path);
        return Err(StorageError::Write {
            path: temporary_path,
            source,
        });
    }
    drop(temporary);
    let staged_revision = FileRevision::read(&temporary_path)?;
    if staged_revision != *source_expected {
        let _ = fs::remove_file(&temporary_path);
        return Err(StorageError::AtomicSourceConflict {
            path: source,
            expected: source_expected.clone(),
            actual: staged_revision,
        });
    }

    let target_actual = FileRevision::read(&target)?;
    if target_actual != *target_expected {
        let _ = fs::remove_file(&temporary_path);
        return Err(StorageError::WriteConflict {
            path: target,
            expected: target_expected.clone(),
            actual: target_actual,
        });
    }
    let (backup_path, mut backup) = match create_unique_sibling(&target, "backup", false) {
        Ok(value) => value,
        Err(error) => {
            let _ = fs::remove_file(&temporary_path);
            return Err(error);
        }
    };
    let backup_result = fs::File::open(&target)
        .and_then(|mut original| std::io::copy(&mut original, &mut backup))
        .and_then(|_| backup.flush())
        .and_then(|()| backup.set_permissions(target_metadata.permissions()))
        .and_then(|()| backup.sync_all());
    if let Err(source) = backup_result {
        let _ = fs::remove_file(&temporary_path);
        let _ = fs::remove_file(&backup_path);
        return Err(StorageError::Write {
            path: backup_path,
            source,
        });
    }
    drop(backup);
    let backup_revision = FileRevision::read(&backup_path)?;
    if backup_revision != *target_expected {
        let _ = fs::remove_file(&temporary_path);
        let _ = fs::remove_file(&backup_path);
        return Err(StorageError::WriteConflict {
            path: target,
            expected: target_expected.clone(),
            actual: backup_revision,
        });
    }
    let target_actual = FileRevision::read(&target)?;
    if target_actual != *target_expected {
        let _ = fs::remove_file(&temporary_path);
        let _ = fs::remove_file(&backup_path);
        return Err(StorageError::WriteConflict {
            path: target,
            expected: target_expected.clone(),
            actual: target_actual,
        });
    }

    if let Err(source) = replace_file(&temporary_path, &target) {
        let _ = fs::remove_file(&temporary_path);
        return Err(StorageError::AtomicReplace {
            path: target,
            backup: backup_path,
            source,
        });
    }
    if let Err(source) = sync_parent_directory(&target) {
        return Err(StorageError::AtomicDirectorySync {
            path: target,
            backup: backup_path,
            source,
        });
    }
    Ok(AtomicSaveReport {
        target,
        backup: backup_path,
    })
}

/// Replaces one existing directory tree from another exact directory snapshot.
///
/// Both trees are rejected if they contain symlinks, special files, or
/// non-Unicode names. The source is copied into a durable sibling staging
/// directory, both revisions are rechecked, and the original target is moved
/// into a retained sibling recovery directory before the staged tree is moved
/// into place. A failed second move is rolled back before returning.
pub fn replace_directory_from_source_if_revisions(
    source: impl AsRef<Path>,
    source_expected: &DirectoryRevision,
    target: impl AsRef<Path>,
    target_expected: &DirectoryRevision,
) -> Result<AtomicDirectorySaveReport, StorageError> {
    replace_directory_from_source_if_revisions_with(
        source.as_ref(),
        source_expected,
        target.as_ref(),
        target_expected,
        |from, to| fs::rename(from, to),
    )
}

fn replace_directory_from_source_if_revisions_with(
    supplied_source: &Path,
    source_expected: &DirectoryRevision,
    supplied_target: &Path,
    target_expected: &DirectoryRevision,
    mut rename: impl FnMut(&Path, &Path) -> std::io::Result<()>,
) -> Result<AtomicDirectorySaveReport, StorageError> {
    let source_metadata =
        fs::symlink_metadata(supplied_source).map_err(|source| StorageError::Read {
            path: supplied_source.to_path_buf(),
            source,
        })?;
    if !source_metadata.file_type().is_dir() || source_metadata.file_type().is_symlink() {
        return Err(StorageError::AtomicSourceNotDirectory {
            path: supplied_source.to_path_buf(),
        });
    }
    let source = fs::canonicalize(supplied_source).map_err(|source| StorageError::Read {
        path: supplied_source.to_path_buf(),
        source,
    })?;

    let target_metadata =
        fs::symlink_metadata(supplied_target).map_err(|source| StorageError::Read {
            path: supplied_target.to_path_buf(),
            source,
        })?;
    if !target_metadata.file_type().is_dir() || target_metadata.file_type().is_symlink() {
        return Err(StorageError::AtomicTargetNotDirectory {
            path: supplied_target.to_path_buf(),
        });
    }
    let target = fs::canonicalize(supplied_target).map_err(|source| StorageError::Read {
        path: supplied_target.to_path_buf(),
        source,
    })?;
    if source == target || source.starts_with(&target) || target.starts_with(&source) {
        return Err(StorageError::AtomicDirectorySourceAliasesTarget {
            source_path: source,
            target_path: target,
        });
    }

    let source_actual = DirectoryRevision::read(&source)?;
    if source_actual != *source_expected {
        return Err(StorageError::AtomicDirectorySourceConflict {
            path: source,
            expected: Box::new(source_expected.clone()),
            actual: Box::new(source_actual),
        });
    }
    let target_actual = DirectoryRevision::read(&target)?;
    if target_actual != *target_expected {
        return Err(StorageError::AtomicDirectoryTargetConflict {
            path: target,
            expected: Box::new(target_expected.clone()),
            actual: Box::new(target_actual),
        });
    }

    let staging = create_unique_sibling_directory(&target, "directory-temporary", true)?;
    if let Err(error) = copy_directory_contents(&source, &staging) {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }
    let staged_revision = DirectoryRevision::read(&staging)?;
    if staged_revision != *source_expected {
        let _ = fs::remove_dir_all(&staging);
        return Err(StorageError::AtomicDirectorySourceConflict {
            path: source,
            expected: Box::new(source_expected.clone()),
            actual: Box::new(staged_revision),
        });
    }
    let source_actual = DirectoryRevision::read(&source)?;
    if source_actual != *source_expected {
        let _ = fs::remove_dir_all(&staging);
        return Err(StorageError::AtomicDirectorySourceConflict {
            path: source,
            expected: Box::new(source_expected.clone()),
            actual: Box::new(source_actual),
        });
    }
    let target_actual = DirectoryRevision::read(&target)?;
    if target_actual != *target_expected {
        let _ = fs::remove_dir_all(&staging);
        return Err(StorageError::AtomicDirectoryTargetConflict {
            path: target,
            expected: Box::new(target_expected.clone()),
            actual: Box::new(target_actual),
        });
    }

    let backup_root = match create_unique_sibling_directory(&target, "directory-backup", false) {
        Ok(path) => path,
        Err(error) => {
            let _ = fs::remove_dir_all(&staging);
            return Err(error);
        }
    };
    let backup = backup_root.join(
        target
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("directory")),
    );
    if let Err(source) = rename(&target, &backup) {
        let _ = fs::remove_dir_all(&staging);
        let _ = fs::remove_dir_all(&backup_root);
        return Err(StorageError::Write {
            path: target,
            source,
        });
    }
    if let Err(source) =
        sync_parent_directory(&target).and_then(|()| sync_parent_directory(&backup))
    {
        let rollback = rename(&backup, &target)
            .and_then(|()| sync_parent_directory(&target))
            .and_then(|()| fs::remove_dir(&backup_root));
        return match rollback {
            Ok(()) => {
                let _ = fs::remove_dir_all(&staging);
                Err(StorageError::AtomicDirectoryReplaceRolledBack {
                    path: target,
                    source,
                })
            }
            Err(recovery) => Err(StorageError::AtomicDirectoryReplaceRecoveryRequired {
                path: Box::new(target),
                backup: Box::new(backup),
                staged: Box::new(staging),
                replace_error: source.to_string(),
                recovery_error: recovery.to_string(),
            }),
        };
    }
    if let Err(source) = rename(&staging, &target) {
        let rollback = rename(&backup, &target)
            .and_then(|()| sync_parent_directory(&target))
            .and_then(|()| fs::remove_dir(&backup_root));
        return match rollback {
            Ok(()) => {
                let _ = fs::remove_dir_all(&staging);
                Err(StorageError::AtomicDirectoryReplaceRolledBack {
                    path: target,
                    source,
                })
            }
            Err(recovery) => Err(StorageError::AtomicDirectoryReplaceRecoveryRequired {
                path: Box::new(target),
                backup: Box::new(backup),
                staged: Box::new(staging),
                replace_error: source.to_string(),
                recovery_error: recovery.to_string(),
            }),
        };
    }
    if let Err(source) = sync_parent_directory(&target) {
        return Err(StorageError::AtomicDirectorySync {
            path: target,
            backup,
            source,
        });
    }
    Ok(AtomicDirectorySaveReport { target, backup })
}

/// Deletes one exact directory tree while retaining the complete tree in a
/// unique sibling recovery directory.
///
/// The target and every nested entry must be a real directory or regular file.
/// Its recursive revision is checked before and immediately before the rename,
/// so callers cannot accidentally delete a save tree that changed after it was
/// presented to the user.
pub fn delete_directory_if_revision(
    target: impl AsRef<Path>,
    expected: &DirectoryRevision,
) -> Result<AtomicDirectorySaveReport, StorageError> {
    let supplied_target = target.as_ref();
    let target_metadata =
        fs::symlink_metadata(supplied_target).map_err(|source| StorageError::Read {
            path: supplied_target.to_path_buf(),
            source,
        })?;
    if !target_metadata.file_type().is_dir() || target_metadata.file_type().is_symlink() {
        return Err(StorageError::AtomicTargetNotDirectory {
            path: supplied_target.to_path_buf(),
        });
    }
    let target = fs::canonicalize(supplied_target).map_err(|source| StorageError::Read {
        path: supplied_target.to_path_buf(),
        source,
    })?;
    let actual = DirectoryRevision::read(&target)?;
    if actual != *expected {
        return Err(StorageError::AtomicDirectoryTargetConflict {
            path: target,
            expected: Box::new(expected.clone()),
            actual: Box::new(actual),
        });
    }

    let backup_root = create_unique_sibling_directory(&target, "directory-delete-backup", false)?;
    let backup = backup_root.join(
        target
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("directory")),
    );
    let actual = match DirectoryRevision::read(&target) {
        Ok(actual) => actual,
        Err(error) => {
            let _ = fs::remove_dir(&backup_root);
            return Err(error);
        }
    };
    if actual != *expected {
        let _ = fs::remove_dir(&backup_root);
        return Err(StorageError::AtomicDirectoryTargetConflict {
            path: target,
            expected: Box::new(expected.clone()),
            actual: Box::new(actual),
        });
    }

    if let Err(source) = fs::rename(&target, &backup) {
        let _ = fs::remove_dir(&backup_root);
        return Err(StorageError::Write {
            path: target,
            source,
        });
    }
    if let Err(source) =
        sync_parent_directory(&target).and_then(|()| sync_parent_directory(&backup))
    {
        return Err(StorageError::AtomicDirectorySync {
            path: target,
            backup,
            source,
        });
    }

    Ok(AtomicDirectorySaveReport { target, backup })
}

/// Deletes an exact set of regular files, including files outside a LaunchBox
/// library root, while retaining one exact sibling recovery copy per file.
///
/// All targets and revisions are validated and all recovery copies are made
/// durable before the first deletion. A failure after deletion starts rolls
/// every attempted target back in reverse order. This deliberately remains a
/// narrow filesystem primitive: callers that also mutate library metadata must
/// first commit a durable vault copy so process death cannot lose save data.
pub fn delete_regular_files_if_revisions(
    files: &[(PathBuf, FileRevision)],
) -> Result<Vec<AtomicSaveReport>, StorageError> {
    delete_regular_files_if_revisions_with(files, |_, target| fs::remove_file(target))
}

fn delete_regular_files_if_revisions_with(
    files: &[(PathBuf, FileRevision)],
    mut remove: impl FnMut(usize, &Path) -> Result<(), std::io::Error>,
) -> Result<Vec<AtomicSaveReport>, StorageError> {
    if files.is_empty() {
        return Err(StorageError::AtomicDeleteSetEmpty);
    }

    let mut seen = BTreeSet::new();
    let mut targets = Vec::with_capacity(files.len());
    for (supplied, expected) in files {
        let metadata = fs::symlink_metadata(supplied).map_err(|source| StorageError::Read {
            path: supplied.clone(),
            source,
        })?;
        if !metadata.file_type().is_file() {
            return Err(StorageError::AtomicTargetNotFile {
                path: supplied.clone(),
            });
        }
        let target = fs::canonicalize(supplied).map_err(|source| StorageError::Read {
            path: supplied.clone(),
            source,
        })?;
        if !seen.insert(target.clone()) {
            return Err(StorageError::AtomicDeleteSetDuplicate { path: target });
        }
        let actual = FileRevision::read(&target)?;
        if actual != *expected {
            return Err(StorageError::WriteConflict {
                path: target,
                expected: expected.clone(),
                actual,
            });
        }
        targets.push((target, expected.clone(), metadata.permissions()));
    }

    let mut reports = Vec::with_capacity(targets.len());
    for (target, expected, permissions) in &targets {
        let (backup_path, mut backup) = match create_unique_sibling(target, "delete-backup", false)
        {
            Ok(backup) => backup,
            Err(error) => {
                cleanup_delete_backups(&reports);
                return Err(error);
            }
        };
        let backup_result = fs::File::open(target)
            .and_then(|mut original| std::io::copy(&mut original, &mut backup))
            .and_then(|_| backup.flush())
            .and_then(|()| backup.set_permissions(permissions.clone()))
            .and_then(|()| backup.sync_all());
        if let Err(source) = backup_result {
            let _ = fs::remove_file(&backup_path);
            cleanup_delete_backups(&reports);
            return Err(StorageError::Write {
                path: backup_path,
                source,
            });
        }
        drop(backup);
        let backup_revision = match FileRevision::read(&backup_path) {
            Ok(revision) => revision,
            Err(error) => {
                let _ = fs::remove_file(&backup_path);
                cleanup_delete_backups(&reports);
                return Err(error);
            }
        };
        if backup_revision != *expected {
            let _ = fs::remove_file(&backup_path);
            cleanup_delete_backups(&reports);
            return Err(StorageError::WriteConflict {
                path: target.clone(),
                expected: expected.clone(),
                actual: backup_revision,
            });
        }
        if let Err(source) = sync_parent_directory(&backup_path) {
            let _ = fs::remove_file(&backup_path);
            cleanup_delete_backups(&reports);
            return Err(StorageError::Write {
                path: backup_path,
                source,
            });
        }
        reports.push(AtomicSaveReport {
            target: target.clone(),
            backup: backup_path,
        });
    }

    for ((target, expected, _), report) in targets.iter().zip(&reports) {
        let actual = match FileRevision::read(target) {
            Ok(actual) => actual,
            Err(error) => {
                cleanup_delete_backups(&reports);
                return Err(error);
            }
        };
        if actual != *expected {
            cleanup_delete_backups(&reports);
            return Err(StorageError::WriteConflict {
                path: report.target.clone(),
                expected: expected.clone(),
                actual,
            });
        }
    }

    for index in 0..targets.len() {
        let target = &targets[index].0;
        let apply_result = remove(index, target).and_then(|()| sync_parent_directory(target));
        if let Err(source) = apply_result {
            let mut recovery_error = None;
            for attempted in (0..=index).rev() {
                if let Err(error) =
                    restore_deleted_regular_file(&reports[attempted], &targets[attempted].1)
                {
                    recovery_error.get_or_insert_with(|| error.to_string());
                }
            }
            if let Some(recovery_error) = recovery_error {
                return Err(StorageError::AtomicDeleteSetRecoveryRequired {
                    path: target.clone(),
                    backup: reports[index].backup.clone(),
                    delete_error: source.to_string(),
                    recovery_error,
                });
            }
            return Err(StorageError::AtomicDeleteSetRolledBack {
                path: target.clone(),
                backup: reports[index].backup.clone(),
                source,
            });
        }
    }

    Ok(reports)
}

fn cleanup_delete_backups(reports: &[AtomicSaveReport]) {
    for report in reports {
        let _ = fs::remove_file(&report.backup);
    }
}

fn restore_deleted_regular_file(
    report: &AtomicSaveReport,
    expected: &FileRevision,
) -> Result<(), StorageError> {
    match fs::symlink_metadata(&report.target) {
        Ok(metadata) if metadata.file_type().is_file() => {
            let actual = FileRevision::read(&report.target)?;
            if actual == *expected {
                return Ok(());
            }
            return Err(StorageError::WriteConflict {
                path: report.target.clone(),
                expected: expected.clone(),
                actual,
            });
        }
        Ok(_) => {
            return Err(StorageError::AtomicTargetNotFile {
                path: report.target.clone(),
            })
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(source) => {
            return Err(StorageError::Read {
                path: report.target.clone(),
                source,
            })
        }
    }

    let backup_revision = FileRevision::read(&report.backup)?;
    if backup_revision != *expected {
        return Err(StorageError::WriteConflict {
            path: report.backup.clone(),
            expected: expected.clone(),
            actual: backup_revision,
        });
    }
    let permissions = fs::metadata(&report.backup)
        .map_err(|source| StorageError::Read {
            path: report.backup.clone(),
            source,
        })?
        .permissions();
    let (temporary_path, mut temporary) =
        create_unique_sibling(&report.target, "delete-restore", true)?;
    let restore_result = fs::File::open(&report.backup)
        .and_then(|mut backup| std::io::copy(&mut backup, &mut temporary))
        .and_then(|_| temporary.flush())
        .and_then(|()| temporary.set_permissions(permissions))
        .and_then(|()| temporary.sync_all());
    if let Err(source) = restore_result {
        let _ = fs::remove_file(&temporary_path);
        return Err(StorageError::Write {
            path: temporary_path,
            source,
        });
    }
    drop(temporary);
    replace_file(&temporary_path, &report.target).map_err(|source| {
        StorageError::AtomicReplace {
            path: report.target.clone(),
            backup: report.backup.clone(),
            source,
        }
    })?;
    sync_parent_directory(&report.target).map_err(|source| StorageError::AtomicDirectorySync {
        path: report.target.clone(),
        backup: report.backup.clone(),
        source,
    })
}

impl PlatformDocument {
    /// Constructs a semantically empty platform document at a native host
    /// path. The platform name remains domain data; callers should obtain the
    /// filename from the platform service instead of treating the name as a
    /// `Path` component directly.
    pub fn new_empty(
        source_path: impl Into<PathBuf>,
        platform_name: impl Into<String>,
    ) -> Result<Self, StorageError> {
        let source_path = source_path.into();
        let library = PlatformLibrary {
            name: platform_name.into(),
            source_path: source_path.clone(),
            games: Vec::new(),
            additional_applications: Vec::new(),
            mounts: Vec::new(),
            alternate_names: Vec::new(),
            custom_fields: Vec::new(),
            controller_support: Vec::new(),
            game_saves: Vec::new(),
        };
        library.validate()?;
        Ok(Self {
            source_path,
            source_revision: None,
            root: Element::new("LaunchBox"),
            library,
        })
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|source| StorageError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let revision = FileRevision::from_bytes(&bytes);
        let mut document = Self::from_reader(path, bytes.as_slice())?;
        document.source_revision = Some(revision);
        Ok(document)
    }

    /// Loads a platform document using its catalog display name when the XML
    /// contains no games. This is required when a portable filename was
    /// coerced (for example `Dragon 32_64.xml` for `Dragon 32/64`).
    pub fn load_for_platform(
        path: impl AsRef<Path>,
        platform_name: impl Into<String>,
    ) -> Result<Self, StorageError> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|source| StorageError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let revision = FileRevision::from_bytes(&bytes);
        let mut document = Self::from_reader_for_platform(path, bytes.as_slice(), platform_name)?;
        document.source_revision = Some(revision);
        Ok(document)
    }

    pub fn from_reader(
        source_path: impl Into<PathBuf>,
        reader: impl Read,
    ) -> Result<Self, StorageError> {
        let source_path = source_path.into();
        let fallback_platform = source_path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown Platform")
            .to_string();
        Self::from_reader_with_fallback(source_path, reader, fallback_platform)
    }

    pub fn from_reader_for_platform(
        source_path: impl Into<PathBuf>,
        reader: impl Read,
        platform_name: impl Into<String>,
    ) -> Result<Self, StorageError> {
        Self::from_reader_with_fallback(source_path.into(), reader, platform_name.into())
    }

    fn from_reader_with_fallback(
        source_path: PathBuf,
        reader: impl Read,
        fallback_platform: String,
    ) -> Result<Self, StorageError> {
        let root = Element::parse(reader).map_err(|source| StorageError::Parse {
            path: source_path.clone(),
            source,
        })?;
        if root.name != "LaunchBox" {
            return Err(StorageError::InvalidRoot {
                path: source_path,
                actual: root.name,
            });
        }

        let games = root
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .filter(|element| element.name == "Game")
            .map(|element| parse_game(element, &fallback_platform))
            .collect::<Result<Vec<_>, _>>()?;
        let additional_applications = elements_named(&root, "AdditionalApplication")
            .map(parse_additional_application)
            .collect::<Result<Vec<_>, _>>()?;
        let mounts = elements_named(&root, "Mount")
            .map(parse_mount)
            .collect::<Result<Vec<_>, _>>()?;
        let alternate_names = elements_named(&root, "AlternateName")
            .map(parse_alternate_name)
            .collect::<Result<Vec<_>, _>>()?;
        let custom_fields = elements_named(&root, "CustomField")
            .map(parse_custom_field)
            .collect::<Result<Vec<_>, _>>()?;
        let controller_support = elements_named(&root, "GameControllerSupport")
            .map(parse_controller_support)
            .collect::<Result<Vec<_>, _>>()?;
        let game_saves = elements_named(&root, "GameSave")
            .map(parse_game_save)
            .collect::<Result<Vec<_>, _>>()?;

        let library = PlatformLibrary {
            name: games
                .first()
                .map(|game| game.platform.clone())
                .filter(|name| !name.trim().is_empty())
                .unwrap_or(fallback_platform),
            source_path: source_path.clone(),
            games,
            additional_applications,
            mounts,
            alternate_names,
            custom_fields,
            controller_support,
            game_saves,
        };
        library.validate()?;

        Ok(Self {
            source_path,
            source_revision: None,
            root,
            library,
        })
    }

    pub fn library(&self) -> &PlatformLibrary {
        &self.library
    }

    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    pub fn source_revision(&self) -> Option<&FileRevision> {
        self.source_revision.as_ref()
    }

    pub fn add_game(&mut self, new_game: NewGame) -> Result<Game, StorageError> {
        if new_game.application_path.trim().is_empty() {
            return Err(StorageError::EmptyGameApplicationPath { id: new_game.id });
        }
        if new_game.platform != self.library.name {
            return Err(StorageError::GamePlatformMismatch {
                expected: self.library.name.clone(),
                actual: new_game.platform,
            });
        }
        if self.library.games.iter().any(|game| game.id == new_game.id) {
            return Err(StorageError::DuplicateGameId { id: new_game.id });
        }
        let game = Game {
            id: new_game.id,
            title: new_game.title,
            platform: new_game.platform,
            application_path: new_game.application_path,
            emulator_id: new_game.emulator_id,
            database_id: new_game.metadata.database_id,
            manual_path: new_game.metadata.manual_path,
            notes: new_game.metadata.notes,
            developer: new_game.metadata.developer,
            genre: new_game.metadata.genre,
            max_players: new_game.metadata.max_players,
            play_mode: new_game.metadata.play_mode,
            publisher: new_game.metadata.publisher,
            rating: new_game.metadata.rating,
            region: new_game.metadata.region,
            release_date: new_game.metadata.release_date,
            release_type: new_game.metadata.release_type,
            status: new_game.metadata.status,
            version: new_game.metadata.version,
            wikipedia_url: new_game.metadata.wikipedia_url,
            video_url: new_game.metadata.video_url,
            community_star_rating: new_game.metadata.community_star_rating.unwrap_or_default(),
            ..Game::default()
        };
        game.validate()?;
        let element = minimal_game_element(&game);
        parse_game(&element, &self.library.name)?;

        let insertion = self
            .root
            .children
            .iter()
            .rposition(|node| {
                node.as_element()
                    .is_some_and(|element| element.name == "Game")
            })
            .map_or(0, |index| index + 1);
        self.root
            .children
            .insert(insertion, XMLNode::Element(element));
        self.library.games.push(game.clone());
        Ok(game)
    }

    /// Adds a new additional application without rewriting any existing
    /// platform records. The owner must already exist in this document and
    /// IDs remain unique across the additional-application family.
    pub fn add_additional_application(
        &mut self,
        application: AdditionalApplication,
    ) -> Result<AdditionalApplication, StorageError> {
        self.require_game(&application.game_id)?;
        application.validate()?;
        if self
            .library
            .additional_applications
            .iter()
            .any(|existing| existing.id == application.id)
        {
            return Err(StorageError::DuplicateAdditionalApplicationId { id: application.id });
        }

        let element = additional_application_element(&application);
        parse_additional_application(&element)?;
        let insertion = platform_record_insertion_index(&self.root, "AdditionalApplication");
        self.root
            .children
            .insert(insertion, XMLNode::Element(element));
        self.library
            .additional_applications
            .push(application.clone());
        self.library.validate()?;
        Ok(application)
    }

    /// Combines standalone games from this platform into one retained root
    /// game and losslessly retargets every platform-document record that
    /// refers to a removed game.
    ///
    /// Each standalone launch target remains selectable through an additional
    /// application. Existing version/helper applications keep their IDs and
    /// unknown XML children. Main-game save rows from removed games are moved
    /// to the generated representative application so per-version save
    /// ownership remains explicit.
    pub fn combine_games(
        &mut self,
        root_game_id: &str,
        selected_game_ids: &[String],
        new_application_ids: &[String],
    ) -> Result<CombinedGames, StorageError> {
        let selected = selected_game_ids
            .iter()
            .map(|id| id.to_lowercase())
            .collect::<std::collections::BTreeSet<_>>();
        if selected.len() < 2 || !selected.contains(&root_game_id.to_lowercase()) {
            return Err(StorageError::InvalidGameGrouping {
                reason: "combine requires at least two unique games including the selected root"
                    .into(),
            });
        }
        let mut selected_games = Vec::with_capacity(selected.len());
        let root_game = self
            .library
            .games
            .iter()
            .find(|game| game.id.eq_ignore_ascii_case(root_game_id))
            .cloned()
            .ok_or_else(|| StorageError::GameNotFound {
                id: root_game_id.to_string(),
            })?;
        selected_games.push(root_game.clone());
        for id in selected_game_ids {
            if id.eq_ignore_ascii_case(root_game_id)
                || selected_games
                    .iter()
                    .any(|game| game.id.eq_ignore_ascii_case(id))
            {
                continue;
            }
            let game = self
                .library
                .games
                .iter()
                .find(|game| game.id.eq_ignore_ascii_case(id))
                .cloned()
                .ok_or_else(|| StorageError::GameNotFound { id: id.clone() })?;
            if !game.platform.eq_ignore_ascii_case(&root_game.platform) {
                return Err(StorageError::InvalidGameGrouping {
                    reason: format!(
                        "{} belongs to {}, not {}",
                        game.title, game.platform, root_game.platform
                    ),
                });
            }
            selected_games.push(game);
        }
        if selected_games.len() != selected.len() {
            return Err(StorageError::InvalidGameGrouping {
                reason: "one or more selected game IDs were not resolved exactly once".into(),
            });
        }

        let mut representative_by_game = std::collections::BTreeMap::<String, String>::new();
        for game in &selected_games {
            if let Some(application) =
                self.library
                    .additional_applications
                    .iter()
                    .find(|application| {
                        application.game_id.eq_ignore_ascii_case(&game.id)
                            && application.is_likely_game_version()
                            && additional_application_matches_game_launch(application, game)
                    })
            {
                representative_by_game.insert(game.id.to_lowercase(), application.id.clone());
            }
        }
        let missing_representatives = selected_games
            .iter()
            .filter(|game| !representative_by_game.contains_key(&game.id.to_lowercase()))
            .count();
        if new_application_ids.len() < missing_representatives {
            return Err(StorageError::InvalidGameGrouping {
                reason: format!(
                    "combine requires at least {missing_representatives} fresh version application ID(s), received {}",
                    new_application_ids.len()
                ),
            });
        }
        let existing_application_ids = self
            .library
            .additional_applications
            .iter()
            .map(|application| application.id.to_lowercase())
            .collect::<std::collections::BTreeSet<_>>();
        let mut supplied_ids = std::collections::BTreeSet::new();
        for id in new_application_ids.iter().take(missing_representatives) {
            if id.trim().is_empty()
                || existing_application_ids.contains(&id.to_lowercase())
                || !supplied_ids.insert(id.to_lowercase())
            {
                return Err(StorageError::InvalidGameGrouping {
                    reason: format!("version application ID is empty or already in use: {id}"),
                });
            }
        }

        let mut next_priority = self
            .library
            .additional_applications
            .iter()
            .filter(|application| selected.contains(&application.game_id.to_lowercase()))
            .map(|application| application.priority)
            .max()
            .unwrap_or_default()
            .saturating_add(1);
        let mut supplied = new_application_ids.iter();
        let mut created_version_applications = Vec::new();
        for game in &selected_games {
            if representative_by_game.contains_key(&game.id.to_lowercase()) {
                continue;
            }
            let id = supplied
                .next()
                .expect("fresh application ID count was validated")
                .clone();
            let application =
                AdditionalApplication::from_game_version(id, &root_game.id, game, next_priority);
            next_priority = next_priority.saturating_add(1);
            representative_by_game.insert(game.id.to_lowercase(), application.id.clone());
            created_version_applications.push(application);
        }

        let removed_games = selected_games
            .iter()
            .filter(|game| !game.id.eq_ignore_ascii_case(&root_game.id))
            .cloned()
            .collect::<Vec<_>>();
        let removed_ids = removed_games
            .iter()
            .map(|game| game.id.to_lowercase())
            .collect::<std::collections::BTreeSet<_>>();
        let mut candidate = self.root.clone();
        let mut retargeted_additional_applications = 0usize;
        let mut retargeted_game_records = 0usize;
        let mut retargeted_clone_relationships = 0usize;
        candidate.children.retain_mut(|node| {
            let Some(element) = node.as_mut_element() else {
                return true;
            };
            if element.name == "Game" {
                let id = optional_child(element, "ID").unwrap_or_default();
                if removed_ids.contains(&id.to_lowercase()) {
                    return false;
                }
                if optional_child(element, "CloneOf")
                    .is_some_and(|clone| removed_ids.contains(&clone.to_lowercase()))
                {
                    if id.eq_ignore_ascii_case(&root_game.id) {
                        set_optional_child_text(element, "CloneOf", None);
                    } else {
                        set_child_text(element, "CloneOf", &root_game.id);
                    }
                    retargeted_clone_relationships =
                        retargeted_clone_relationships.saturating_add(1);
                }
                return true;
            }

            let owner_field = match element.name.as_str() {
                "AdditionalApplication" | "Mount" | "AlternateName" | "CustomField" => {
                    Some("GameID")
                }
                "GameControllerSupport" | "GameSave" => Some("GameId"),
                _ => None,
            };
            let Some(owner_field) = owner_field else {
                return true;
            };
            let owner = optional_child(element, owner_field).unwrap_or_default();
            if !removed_ids.contains(&owner.to_lowercase()) {
                return true;
            }
            set_child_text(element, owner_field, &root_game.id);
            if element.name == "AdditionalApplication" {
                retargeted_additional_applications =
                    retargeted_additional_applications.saturating_add(1);
            } else {
                retargeted_game_records = retargeted_game_records.saturating_add(1);
            }
            if element.name == "GameSave"
                && optional_child(element, "AdditionalApplicationId").is_none()
            {
                if let Some(representative) = representative_by_game.get(&owner.to_lowercase()) {
                    set_child_text(element, "AdditionalApplicationId", representative);
                }
            }
            true
        });
        for application in &created_version_applications {
            let insertion = platform_record_insertion_index(&candidate, "AdditionalApplication");
            candidate.children.insert(
                insertion,
                XMLNode::Element(additional_application_element(application)),
            );
        }
        self.replace_platform_root(candidate)?;

        Ok(CombinedGames {
            root_game,
            removed_games,
            created_version_applications,
            retargeted_additional_applications,
            retargeted_game_records,
            retargeted_clone_relationships,
        })
    }

    /// Expands every launchable version application into a standalone game.
    ///
    /// A version row that exactly represents the retained root's default
    /// launch target is consumed without creating a duplicate game. Save rows
    /// explicitly owned by an expanded application move to the corresponding
    /// new game and drop the now-removed application ID.
    pub fn expand_game_versions(
        &mut self,
        root_game_id: &str,
        new_game_ids: &[String],
    ) -> Result<ExpandedGames, StorageError> {
        let root_game = self
            .library
            .games
            .iter()
            .find(|game| game.id.eq_ignore_ascii_case(root_game_id))
            .cloned()
            .ok_or_else(|| StorageError::GameNotFound {
                id: root_game_id.to_string(),
            })?;
        let version_applications = self
            .library
            .additional_applications
            .iter()
            .filter(|application| {
                application.game_id.eq_ignore_ascii_case(root_game_id)
                    && application.is_likely_game_version()
            })
            .cloned()
            .collect::<Vec<_>>();
        let expandable_count = version_applications
            .iter()
            .filter(|application| {
                !additional_application_matches_game_launch(application, &root_game)
            })
            .count();
        if expandable_count == 0 {
            return Err(StorageError::InvalidGameGrouping {
                reason:
                    "the selected game has no additional launch target distinct from its default"
                        .into(),
            });
        }
        if new_game_ids.len() < expandable_count {
            return Err(StorageError::InvalidGameGrouping {
                reason: format!(
                    "expand requires at least {expandable_count} fresh game ID(s), received {}",
                    new_game_ids.len()
                ),
            });
        }
        let existing_game_ids = self
            .library
            .games
            .iter()
            .map(|game| game.id.to_lowercase())
            .collect::<std::collections::BTreeSet<_>>();
        let mut supplied_ids = std::collections::BTreeSet::new();
        for id in new_game_ids.iter().take(expandable_count) {
            if id.trim().is_empty()
                || existing_game_ids.contains(&id.to_lowercase())
                || !supplied_ids.insert(id.to_lowercase())
            {
                return Err(StorageError::InvalidGameGrouping {
                    reason: format!("expanded game ID is empty or already in use: {id}"),
                });
            }
        }

        let mut supplied = new_game_ids.iter();
        let mut application_to_game = std::collections::BTreeMap::<String, Option<String>>::new();
        let mut created_games = Vec::with_capacity(expandable_count);
        for application in &version_applications {
            if additional_application_matches_game_launch(application, &root_game) {
                application_to_game.insert(application.id.to_lowercase(), None);
                continue;
            }
            let id = supplied
                .next()
                .expect("fresh expanded game ID count was validated")
                .clone();
            let game = Game::from_additional_application_version(id, application, &root_game);
            game.validate()?;
            application_to_game.insert(application.id.to_lowercase(), Some(game.id.clone()));
            created_games.push(game);
        }

        let root_element = self
            .root
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .find(|element| {
                element.name == "Game"
                    && optional_child(element, "ID")
                        .is_some_and(|id| id.eq_ignore_ascii_case(root_game_id))
            })
            .cloned()
            .ok_or_else(|| StorageError::GameNotFound {
                id: root_game_id.to_string(),
            })?;
        let root_mounts = self
            .root
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .filter(|element| {
                element.name == "Mount"
                    && optional_child(element, "GameID")
                        .is_some_and(|id| id.eq_ignore_ascii_case(root_game_id))
            })
            .cloned()
            .collect::<Vec<_>>();
        let removed_application_ids = application_to_game
            .keys()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        let mut candidate = self.root.clone();
        let mut reassigned_game_saves = 0usize;
        candidate.children.retain_mut(|node| {
            let Some(element) = node.as_mut_element() else {
                return true;
            };
            if element.name == "AdditionalApplication" {
                return !optional_child(element, "Id")
                    .is_some_and(|id| removed_application_ids.contains(&id.to_lowercase()));
            }
            if element.name != "GameSave" {
                return true;
            }
            let Some(application_id) = optional_child(element, "AdditionalApplicationId") else {
                return true;
            };
            let Some(new_game_id) = application_to_game.get(&application_id.to_lowercase()) else {
                return true;
            };
            set_child_text(
                element,
                "GameId",
                new_game_id.as_deref().unwrap_or(&root_game.id),
            );
            set_optional_child_text(element, "AdditionalApplicationId", None);
            reassigned_game_saves = reassigned_game_saves.saturating_add(1);
            true
        });

        for game in &created_games {
            let mut element = root_element.clone();
            update_expanded_game_element(&mut element, game);
            let insertion = platform_record_insertion_index(&candidate, "Game");
            candidate
                .children
                .insert(insertion, XMLNode::Element(element));
            if game.use_dos_box {
                for mount in &root_mounts {
                    let mut mount = mount.clone();
                    set_child_text(&mut mount, "GameID", &game.id);
                    let insertion = platform_record_insertion_index(&candidate, "Mount");
                    candidate
                        .children
                        .insert(insertion, XMLNode::Element(mount));
                }
            }
        }
        self.replace_platform_root(candidate)?;

        Ok(ExpandedGames {
            root_game,
            created_games,
            removed_version_applications: version_applications,
            reassigned_game_saves,
        })
    }

    /// Retargets clone relationships in another platform document during a
    /// cross-document combine transaction.
    pub fn remap_clone_references(
        &mut self,
        removed_game_ids: &[String],
        root_game_id: &str,
    ) -> Result<usize, StorageError> {
        let removed = removed_game_ids
            .iter()
            .map(|id| id.to_lowercase())
            .collect::<std::collections::BTreeSet<_>>();
        let mut candidate = self.root.clone();
        let mut changed = 0usize;
        for element in candidate
            .children
            .iter_mut()
            .filter_map(XMLNode::as_mut_element)
            .filter(|element| element.name == "Game")
        {
            if optional_child(element, "CloneOf")
                .is_some_and(|id| removed.contains(&id.to_lowercase()))
            {
                set_child_text(element, "CloneOf", root_game_id);
                changed = changed.saturating_add(1);
            }
        }
        if changed > 0 {
            self.replace_platform_root(candidate)?;
        }
        Ok(changed)
    }

    fn replace_platform_root(&mut self, root: Element) -> Result<(), StorageError> {
        let mut bytes = Vec::new();
        write_xml_root(&root, &mut bytes)?;
        let replacement = Self::from_reader_for_platform(
            self.source_path.clone(),
            bytes.as_slice(),
            self.library.name.clone(),
        )?;
        self.root = replacement.root;
        self.library = replacement.library;
        Ok(())
    }

    /// Replaces the 13.27 editor surface for one additional application while
    /// retaining its immutable identity, owner, storefront/cloud fields, and
    /// every unknown XML child on the source record.
    pub fn set_additional_application(
        &mut self,
        id: &str,
        edit: AdditionalApplicationEdit,
    ) -> Result<AdditionalApplication, StorageError> {
        let application_index = self
            .library
            .additional_applications
            .iter()
            .position(|application| application.id == id)
            .ok_or_else(|| StorageError::AdditionalApplicationNotFound { id: id.to_string() })?;
        let original = self.library.additional_applications[application_index].clone();
        let updated = edit.apply_to(&original);
        updated.validate()?;

        let element = find_record_element_mut(&mut self.root, "AdditionalApplication", "Id", id)
            .ok_or_else(|| StorageError::AdditionalApplicationNotFound { id: id.to_string() })?;
        macro_rules! update_text {
            ($field:ident, $element_name:literal) => {
                if original.$field != updated.$field {
                    set_child_text(element, $element_name, &updated.$field.to_string());
                }
            };
        }
        macro_rules! update_optional_text {
            ($field:ident, $element_name:literal) => {
                if original.$field != updated.$field {
                    set_optional_child_text(element, $element_name, updated.$field.as_deref());
                }
            };
        }

        if original.name != updated.name {
            set_child_text(element, "Name", &updated.name);
        }
        if original.application_path != updated.application_path {
            set_child_text(element, "ApplicationPath", &updated.application_path);
        }
        update_optional_text!(command_line, "CommandLine");
        update_text!(auto_run_before, "AutoRunBefore");
        update_text!(auto_run_after, "AutoRunAfter");
        update_text!(wait_for_exit, "WaitForExit");
        update_text!(use_emulator, "UseEmulator");
        update_optional_text!(emulator_id, "EmulatorId");
        update_text!(use_dos_box, "UseDosBox");
        update_text!(priority, "Priority");
        update_text!(play_count, "PlayCount");
        update_text!(play_time_seconds, "PlayTime");
        if original.disc != updated.disc {
            let disc = updated.disc.map(|value| value.to_string());
            set_optional_child_text(element, "Disc", disc.as_deref());
        }
        update_text!(side_a, "SideA");
        update_text!(side_b, "SideB");
        update_optional_text!(developer, "Developer");
        update_optional_text!(publisher, "Publisher");
        update_optional_text!(region, "Region");
        update_optional_text!(release_date, "ReleaseDate");
        update_optional_text!(version, "Version");
        update_optional_text!(status, "Status");
        if original.installed != updated.installed {
            let installed = updated.installed.map(|value| value.to_string());
            set_optional_child_text(element, "Installed", installed.as_deref());
        }
        update_optional_text!(last_played, "LastPlayed");

        self.library.additional_applications[application_index] = updated.clone();
        self.library.validate()?;
        Ok(updated)
    }

    /// Copies one additional application's shared launch/version fields to its
    /// owning game while retaining both records and all game-only/unknown XML.
    pub fn make_additional_application_default(&mut self, id: &str) -> Result<Game, StorageError> {
        let application = self
            .library
            .additional_applications
            .iter()
            .find(|application| application.id == id)
            .cloned()
            .ok_or_else(|| StorageError::AdditionalApplicationNotFound { id: id.to_string() })?;
        let game_index = self
            .library
            .games
            .iter()
            .position(|game| game.id == application.game_id)
            .ok_or_else(|| StorageError::GameNotFound {
                id: application.game_id.clone(),
            })?;
        let original = self.library.games[game_index].clone();
        let updated = application.apply_as_default_to(&original);
        GameLaunchConfiguration::from(&updated).validate_for_game(&updated.id)?;
        updated.validate()?;

        let element = find_record_element_mut(&mut self.root, "Game", "ID", &updated.id)
            .ok_or_else(|| StorageError::GameNotFound {
                id: updated.id.clone(),
            })?;
        macro_rules! update_text {
            ($field:ident, $element_name:literal) => {
                if original.$field != updated.$field {
                    set_child_text(element, $element_name, &updated.$field.to_string());
                }
            };
        }
        macro_rules! update_optional_text {
            ($field:ident, $element_name:literal) => {
                if original.$field != updated.$field {
                    set_optional_child_text(element, $element_name, updated.$field.as_deref());
                }
            };
        }

        if original.application_path != updated.application_path {
            set_child_text(element, "ApplicationPath", &updated.application_path);
        }
        update_optional_text!(command_line, "CommandLine");
        update_optional_text!(emulator_id, "Emulator");
        update_text!(use_dos_box, "UseDosBox");
        update_text!(use_scumm_vm, "UseScummVM");
        update_optional_text!(developer, "Developer");
        update_optional_text!(publisher, "Publisher");
        update_optional_text!(region, "Region");
        update_optional_text!(release_date, "ReleaseDate");
        update_optional_text!(version, "Version");
        update_optional_text!(status, "Status");
        if original.installed != updated.installed {
            let installed = updated.installed.map(|value| value.to_string());
            set_optional_child_text(element, "Installed", installed.as_deref());
        }
        update_text!(play_count, "PlayCount");
        update_text!(play_time_seconds, "PlayTime");
        update_optional_text!(last_played_date, "LastPlayedDate");
        update_optional_text!(gog_app_id, "GogAppId");
        update_optional_text!(origin_app_id, "OriginAppId");
        update_optional_text!(origin_install_path, "OriginInstallPath");
        update_text!(has_cloud_synced, "HasCloudSynced");

        self.library.games[game_index] = updated.clone();
        self.library.validate()?;
        Ok(updated)
    }

    /// Removes one additional application without deleting files or save
    /// history. LaunchBox game-save records can explicitly own an additional
    /// application ID, so those references must be remediated first.
    pub fn remove_additional_application(
        &mut self,
        id: &str,
    ) -> Result<AdditionalApplication, StorageError> {
        let reference_count = self
            .library
            .game_saves
            .iter()
            .filter(|save| save.additional_application_id.as_deref() == Some(id))
            .count();
        if reference_count > 0 {
            return Err(StorageError::AdditionalApplicationHasReferences {
                id: id.to_string(),
                count: reference_count,
            });
        }
        let application_index = self
            .library
            .additional_applications
            .iter()
            .position(|application| application.id == id)
            .ok_or_else(|| StorageError::AdditionalApplicationNotFound { id: id.to_string() })?;
        let element_index = self
            .root
            .children
            .iter()
            .position(|node| {
                node.as_element().is_some_and(|element| {
                    element.name == "AdditionalApplication"
                        && child_text(element, "Id").as_deref() == Some(id)
                })
            })
            .ok_or_else(|| StorageError::AdditionalApplicationNotFound { id: id.to_string() })?;
        self.root.children.remove(element_index);
        let application = self
            .library
            .additional_applications
            .remove(application_index);
        self.library.validate()?;
        Ok(application)
    }

    pub fn remove_game(&mut self, id: &str) -> Result<Game, StorageError> {
        let references = platform_library_game_references(&self.library, id);
        if !references.is_empty() {
            return Err(StorageError::GameHasReferences {
                id: id.to_string(),
                summary: summarize_reference_kinds(&references),
                count: references.len(),
            });
        }
        let game_index = self
            .library
            .games
            .iter()
            .position(|game| game.id == id)
            .ok_or_else(|| StorageError::GameNotFound { id: id.to_string() })?;
        let element_index = self
            .root
            .children
            .iter()
            .position(|node| {
                node.as_element().is_some_and(|element| {
                    element.name == "Game" && child_text(element, "ID").as_deref() == Some(id)
                })
            })
            .ok_or_else(|| StorageError::GameNotFound { id: id.to_string() })?;
        self.root.children.remove(element_index);
        Ok(self.library.games.remove(game_index))
    }

    pub fn set_game_title(&mut self, id: &str, title: &str) -> Result<(), StorageError> {
        if title.trim().is_empty() {
            return Err(StorageError::EmptyGameTitle { id: id.to_string() });
        }

        let game = self
            .library
            .games
            .iter_mut()
            .find(|game| game.id == id)
            .ok_or_else(|| StorageError::GameNotFound { id: id.to_string() })?;
        game.title = title.to_string();

        let element = self
            .root
            .children
            .iter_mut()
            .filter_map(XMLNode::as_mut_element)
            .filter(|element| element.name == "Game")
            .find(|element| child_text(element, "ID").as_deref() == Some(id))
            .ok_or_else(|| StorageError::GameNotFound { id: id.to_string() })?;
        set_child_text(element, "Title", title);
        Ok(())
    }

    /// Replaces the user-editable descriptive metadata for one game while
    /// retaining every field outside that edit surface, including unknown XML.
    /// `None` removes an optional LaunchBox element; unchanged optional values
    /// are not rewritten, so existing empty-element spelling remains intact.
    pub fn set_game_metadata(
        &mut self,
        id: &str,
        metadata: GameMetadata,
    ) -> Result<Game, StorageError> {
        if metadata.title.trim().is_empty() {
            return Err(StorageError::EmptyGameTitle { id: id.to_string() });
        }

        let game_index = self
            .library
            .games
            .iter()
            .position(|game| game.id == id)
            .ok_or_else(|| StorageError::GameNotFound { id: id.to_string() })?;
        let original = self.library.games[game_index].clone();
        let mut updated = original.clone();
        updated.title = metadata.title;
        updated.sort_title = metadata.sort_title;
        updated.notes = metadata.notes;
        updated.developer = metadata.developer;
        updated.genre = metadata.genre;
        updated.max_players = metadata.max_players;
        updated.play_mode = metadata.play_mode;
        updated.progress = metadata.progress;
        updated.publisher = metadata.publisher;
        updated.rating = metadata.rating;
        updated.region = metadata.region;
        updated.release_date = metadata.release_date;
        updated.release_type = metadata.release_type;
        updated.series = metadata.series;
        updated.source = metadata.source;
        updated.status = metadata.status;
        updated.version = metadata.version;
        updated.wikipedia_url = metadata.wikipedia_url;
        updated.validate()?;

        let element = find_record_element_mut(&mut self.root, "Game", "ID", id)
            .ok_or_else(|| StorageError::GameNotFound { id: id.to_string() })?;
        if original.title != updated.title {
            set_child_text(element, "Title", &updated.title);
        }
        macro_rules! update_optional_text {
            ($field:ident, $element_name:literal) => {
                if original.$field != updated.$field {
                    set_optional_child_text(element, $element_name, updated.$field.as_deref());
                }
            };
        }
        update_optional_text!(sort_title, "SortTitle");
        update_optional_text!(notes, "Notes");
        update_optional_text!(developer, "Developer");
        update_optional_text!(genre, "Genre");
        if original.max_players != updated.max_players {
            let max_players = updated.max_players.map(|value| value.to_string());
            set_optional_child_text(element, "MaxPlayers", max_players.as_deref());
        }
        update_optional_text!(play_mode, "PlayMode");
        update_optional_text!(progress, "Progress");
        update_optional_text!(publisher, "Publisher");
        update_optional_text!(rating, "Rating");
        update_optional_text!(region, "Region");
        update_optional_text!(release_date, "ReleaseDate");
        update_optional_text!(release_type, "ReleaseType");
        update_optional_text!(series, "Series");
        update_optional_text!(source, "Source");
        update_optional_text!(status, "Status");
        update_optional_text!(version, "Version");
        update_optional_text!(wikipedia_url, "WikipediaURL");

        self.library.games[game_index] = updated.clone();
        Ok(updated)
    }

    /// Replaces the persisted executable/backend settings for one game.
    /// Stored paths remain lexical LaunchBox data and are never interpreted as
    /// native paths by this storage operation.
    pub fn set_game_launch_configuration(
        &mut self,
        id: &str,
        configuration: GameLaunchConfiguration,
    ) -> Result<Game, StorageError> {
        let game_index = self
            .library
            .games
            .iter()
            .position(|game| game.id == id)
            .ok_or_else(|| StorageError::GameNotFound { id: id.to_string() })?;
        let original = self.library.games[game_index].clone();
        if GameLaunchConfiguration::from(&original) == configuration {
            return Ok(original);
        }
        configuration.validate_for_game(id)?;

        let mut updated = original.clone();
        updated.application_path = configuration.application_path;
        updated.command_line = configuration.command_line;
        updated.emulator_id = configuration.emulator_id;
        updated.use_dos_box = configuration.use_dos_box;
        updated.custom_dos_box_version_path = configuration.custom_dos_box_version_path;
        updated.dos_box_configuration_path = configuration.dos_box_configuration_path;
        updated.use_scumm_vm = configuration.use_scumm_vm;
        updated.scumm_vm_aspect_correction = configuration.scumm_vm_aspect_correction;
        updated.scumm_vm_fullscreen = configuration.scumm_vm_fullscreen;
        updated.scumm_vm_game_data_folder_path = configuration.scumm_vm_game_data_folder_path;
        updated.scumm_vm_game_type = configuration.scumm_vm_game_type;
        updated.validate()?;

        let element = find_record_element_mut(&mut self.root, "Game", "ID", id)
            .ok_or_else(|| StorageError::GameNotFound { id: id.to_string() })?;
        if original.application_path != updated.application_path {
            set_child_text(element, "ApplicationPath", &updated.application_path);
        }
        macro_rules! update_optional_text {
            ($field:ident, $element_name:literal) => {
                if original.$field != updated.$field {
                    set_optional_child_text(element, $element_name, updated.$field.as_deref());
                }
            };
        }
        update_optional_text!(command_line, "CommandLine");
        update_optional_text!(emulator_id, "Emulator");
        if original.use_dos_box != updated.use_dos_box {
            set_child_text(element, "UseDosBox", &updated.use_dos_box.to_string());
        }
        update_optional_text!(custom_dos_box_version_path, "CustomDosBoxVersionPath");
        update_optional_text!(dos_box_configuration_path, "DosBoxConfigurationPath");
        if original.use_scumm_vm != updated.use_scumm_vm {
            set_child_text(element, "UseScummVM", &updated.use_scumm_vm.to_string());
        }
        if original.scumm_vm_aspect_correction != updated.scumm_vm_aspect_correction {
            set_child_text(
                element,
                "ScummVMAspectCorrection",
                &updated.scumm_vm_aspect_correction.to_string(),
            );
        }
        if original.scumm_vm_fullscreen != updated.scumm_vm_fullscreen {
            set_child_text(
                element,
                "ScummVMFullscreen",
                &updated.scumm_vm_fullscreen.to_string(),
            );
        }
        update_optional_text!(scumm_vm_game_data_folder_path, "ScummVMGameDataFolderPath");
        update_optional_text!(scumm_vm_game_type, "ScummVMGameType");

        self.library.games[game_index] = updated.clone();
        Ok(updated)
    }

    /// Replaces one game's alternate-name rows while preserving unknown XML
    /// children on every retained row. Source indices are scoped to that game,
    /// not to the whole platform document.
    pub fn set_game_alternate_names(
        &mut self,
        id: &str,
        edits: Vec<IndexedPlatformRecordEdit<AlternateName>>,
    ) -> Result<Vec<AlternateName>, StorageError> {
        self.require_game(id)?;
        let originals = self
            .library
            .alternate_names
            .iter()
            .filter(|record| record.game_id == id)
            .cloned()
            .collect::<Vec<_>>();
        validate_indexed_record_edits("alternate name", id, originals.len(), &edits)?;
        for edit in &edits {
            if edit.record.game_id != id {
                return Err(invalid_game_record_edit(
                    "alternate name",
                    id,
                    "record owner does not match the edited game",
                ));
            }
            edit.record.validate()?;
        }
        let root_indices = matching_record_indices(&self.root, "AlternateName", "GameID", id);
        ensure_record_index_alignment("alternate name", id, originals.len(), root_indices.len())?;

        let mut retained = vec![false; originals.len()];
        for edit in &edits {
            let Some(source_index) = edit.source_index else {
                continue;
            };
            retained[source_index] = true;
            let original = &originals[source_index];
            let element = self.root.children[root_indices[source_index]]
                .as_mut_element()
                .expect("matching record index must identify an element");
            if original.name != edit.record.name {
                set_child_text(element, "Name", &edit.record.name);
            }
            if original.region != edit.record.region {
                set_optional_child_text(element, "Region", edit.record.region.as_deref());
            }
        }
        for source_index in (0..root_indices.len()).rev() {
            if !retained[source_index] {
                self.root.children.remove(root_indices[source_index]);
            }
        }
        for edit in edits.iter().filter(|edit| edit.source_index.is_none()) {
            let mut element = Element::new("AlternateName");
            set_child_text(&mut element, "GameID", id);
            set_child_text(&mut element, "Name", &edit.record.name);
            set_optional_child_text(&mut element, "Region", edit.record.region.as_deref());
            let insertion = platform_record_insertion_index(&self.root, "AlternateName");
            self.root
                .children
                .insert(insertion, XMLNode::Element(element));
        }
        self.library.alternate_names = elements_named(&self.root, "AlternateName")
            .map(parse_alternate_name)
            .collect::<Result<Vec<_>, _>>()?;
        self.library.validate()?;
        Ok(self
            .library
            .alternate_names
            .iter()
            .filter(|record| record.game_id == id)
            .cloned()
            .collect())
    }

    /// Replaces one game's custom-field rows with the same source-indexed,
    /// unknown-XML-preserving behavior as alternate-name editing.
    pub fn set_game_custom_fields(
        &mut self,
        id: &str,
        edits: Vec<IndexedPlatformRecordEdit<CustomField>>,
    ) -> Result<Vec<CustomField>, StorageError> {
        self.require_game(id)?;
        let originals = self
            .library
            .custom_fields
            .iter()
            .filter(|record| record.game_id == id)
            .cloned()
            .collect::<Vec<_>>();
        validate_indexed_record_edits("custom field", id, originals.len(), &edits)?;
        for edit in &edits {
            if edit.record.game_id != id {
                return Err(invalid_game_record_edit(
                    "custom field",
                    id,
                    "record owner does not match the edited game",
                ));
            }
            edit.record.validate()?;
        }
        let root_indices = matching_record_indices(&self.root, "CustomField", "GameID", id);
        ensure_record_index_alignment("custom field", id, originals.len(), root_indices.len())?;

        let mut retained = vec![false; originals.len()];
        for edit in &edits {
            let Some(source_index) = edit.source_index else {
                continue;
            };
            retained[source_index] = true;
            let original = &originals[source_index];
            let element = self.root.children[root_indices[source_index]]
                .as_mut_element()
                .expect("matching record index must identify an element");
            if original.name != edit.record.name {
                set_child_text(element, "Name", &edit.record.name);
            }
            if original.value != edit.record.value {
                set_child_text(element, "Value", &edit.record.value);
            }
        }
        for source_index in (0..root_indices.len()).rev() {
            if !retained[source_index] {
                self.root.children.remove(root_indices[source_index]);
            }
        }
        for edit in edits.iter().filter(|edit| edit.source_index.is_none()) {
            let mut element = Element::new("CustomField");
            set_child_text(&mut element, "GameID", id);
            set_child_text(&mut element, "Name", &edit.record.name);
            set_child_text(&mut element, "Value", &edit.record.value);
            let insertion = platform_record_insertion_index(&self.root, "CustomField");
            self.root
                .children
                .insert(insertion, XMLNode::Element(element));
        }
        self.library.custom_fields = elements_named(&self.root, "CustomField")
            .map(parse_custom_field)
            .collect::<Result<Vec<_>, _>>()?;
        self.library.validate()?;
        Ok(self
            .library
            .custom_fields
            .iter()
            .filter(|record| record.game_id == id)
            .cloned()
            .collect())
    }

    /// Applies metadata-only changes to existing save rows without creating,
    /// deleting, moving, or otherwise claiming ownership of save files.
    ///
    /// Source indices are scoped to this game's `<GameSave>` rows in document
    /// order. Every non-editable child, including unknown future LaunchBox
    /// fields, remains byte-for-byte represented in the XML tree.
    pub fn set_game_save_metadata(
        &mut self,
        id: &str,
        edits: Vec<IndexedGameSaveMetadataEdit>,
    ) -> Result<Vec<GameSave>, StorageError> {
        self.require_game(id)?;
        let originals = self
            .library
            .game_saves
            .iter()
            .filter(|record| record.game_id == id)
            .cloned()
            .collect::<Vec<_>>();
        validate_game_save_metadata_edits(id, originals.len(), &edits)?;
        let root_indices = matching_record_indices(&self.root, "GameSave", "GameId", id);
        ensure_record_index_alignment("game save", id, originals.len(), root_indices.len())?;

        for edit in &edits {
            let original = &originals[edit.source_index];
            let updated = edit.metadata.apply_to(original);
            let element = self.root.children[root_indices[edit.source_index]]
                .as_mut_element()
                .expect("matching record index must identify an element");
            if original.title != updated.title {
                set_optional_child_text(element, "Title", updated.title.as_deref());
            }
            if original.save_group_name != updated.save_group_name {
                set_optional_child_text(
                    element,
                    "SaveGroupName",
                    updated.save_group_name.as_deref(),
                );
            }
            if original.save_group_id != updated.save_group_id {
                set_optional_child_text(element, "SaveGroupId", updated.save_group_id.as_deref());
            }
        }

        self.library.game_saves = elements_named(&self.root, "GameSave")
            .map(parse_game_save)
            .collect::<Result<Vec<_>, _>>()?;
        self.library.validate()?;
        Ok(self
            .library
            .game_saves
            .iter()
            .filter(|record| record.game_id == id)
            .cloned()
            .collect())
    }

    /// Appends one filesystem-backed save record while preserving every
    /// existing row and unknown XML field. The caller owns file creation and
    /// must stage this document in the same transaction as that file.
    pub fn add_game_save(&mut self, save: GameSave) -> Result<Vec<GameSave>, StorageError> {
        let game_id = save.game_id.clone();
        self.require_game(&game_id)?;
        save.validate()?;
        if let Some(application_id) = save.additional_application_id.as_deref() {
            let owned = self
                .library
                .additional_applications
                .iter()
                .any(|application| {
                    application.id == application_id && application.game_id == save.game_id
                });
            if !owned {
                return Err(StorageError::AdditionalApplicationNotFound {
                    id: application_id.to_string(),
                });
            }
        }

        let element = game_save_element(&save);
        parse_game_save(&element)?;
        let insertion = platform_record_insertion_index(&self.root, "GameSave");
        self.root
            .children
            .insert(insertion, XMLNode::Element(element));
        self.library.game_saves.push(save);
        self.library.validate()?;
        Ok(self
            .library
            .game_saves
            .iter()
            .filter(|record| record.game_id == game_id)
            .cloned()
            .collect())
    }

    /// Removes one persisted save row without interpreting or deleting its
    /// filesystem path. Source indices are scoped to this game's
    /// `<GameSave>` rows in document order.
    ///
    /// Callers that own a vault file must stage that file deletion in the
    /// same transaction as this document.
    pub fn remove_game_save(
        &mut self,
        id: &str,
        source_index: usize,
    ) -> Result<Vec<GameSave>, StorageError> {
        self.require_game(id)?;
        let originals = self
            .library
            .game_saves
            .iter()
            .filter(|record| record.game_id == id)
            .collect::<Vec<_>>();
        let root_indices = matching_record_indices(&self.root, "GameSave", "GameId", id);
        ensure_record_index_alignment("game save", id, originals.len(), root_indices.len())?;
        let root_index = root_indices.get(source_index).copied().ok_or_else(|| {
            StorageError::InvalidGameRecordEdit {
                record: "game save",
                game_id: id.to_string(),
                reason: format!(
                    "source index {source_index} is outside the current {} rows",
                    originals.len()
                ),
            }
        })?;

        self.root.children.remove(root_index);
        self.library.game_saves = elements_named(&self.root, "GameSave")
            .map(parse_game_save)
            .collect::<Result<Vec<_>, _>>()?;
        self.library.validate()?;
        Ok(self
            .library
            .game_saves
            .iter()
            .filter(|record| record.game_id == id)
            .cloned()
            .collect())
    }

    fn require_game(&self, id: &str) -> Result<(), StorageError> {
        self.library
            .games
            .iter()
            .any(|game| game.id == id)
            .then_some(())
            .ok_or_else(|| StorageError::GameNotFound { id: id.to_string() })
    }

    pub fn set_game_state(
        &mut self,
        id: &str,
        favorite: bool,
        completed: bool,
        star_rating: u8,
    ) -> Result<(), StorageError> {
        if star_rating > 5 {
            return Err(ValidationError::InvalidStarRating {
                id: id.to_string(),
                rating: star_rating,
            }
            .into());
        }

        let game_index = self
            .library
            .games
            .iter()
            .position(|game| game.id == id)
            .ok_or_else(|| StorageError::GameNotFound { id: id.to_string() })?;
        let element = self
            .root
            .children
            .iter_mut()
            .filter_map(XMLNode::as_mut_element)
            .filter(|element| element.name == "Game")
            .find(|element| child_text(element, "ID").as_deref() == Some(id))
            .ok_or_else(|| StorageError::GameNotFound { id: id.to_string() })?;

        let game = &mut self.library.games[game_index];
        game.favorite = favorite;
        game.completed = completed;
        game.star_rating = star_rating;
        game.validate()?;

        set_child_text(element, "Favorite", &favorite.to_string());
        set_child_text(element, "Completed", &completed.to_string());
        set_child_text(element, "StarRating", &star_rating.to_string());
        Ok(())
    }

    /// Records the point at which a main game successfully starts. LaunchBox
    /// increments the count and persists the local-offset timestamp before it
    /// later knows how long the process will remain active.
    pub fn record_game_play_start(
        &mut self,
        id: &str,
        last_played: &str,
    ) -> Result<Game, StorageError> {
        validate_last_played_timestamp(last_played)?;
        let game_index = self
            .library
            .games
            .iter()
            .position(|game| game.id == id)
            .ok_or_else(|| StorageError::GameNotFound { id: id.to_string() })?;
        let play_count = self.library.games[game_index]
            .play_count
            .checked_add(1)
            .ok_or_else(|| StorageError::PlayStatisticOverflow {
                record: "game",
                id: id.to_string(),
                field: "PlayCount",
            })?;
        let element = find_record_element_mut(&mut self.root, "Game", "ID", id)
            .ok_or_else(|| StorageError::GameNotFound { id: id.to_string() })?;

        let game = &mut self.library.games[game_index];
        game.play_count = play_count;
        game.last_played_date = Some(last_played.to_string());
        set_child_text(element, "PlayCount", &play_count.to_string());
        set_child_text(element, "LastPlayedDate", last_played);
        Ok(game.clone())
    }

    /// Adds observed whole seconds after a main-game process exits. A zero
    /// duration is a typed no-op so short launcher processes do not invent a
    /// second of playtime.
    pub fn record_game_play_time(
        &mut self,
        id: &str,
        elapsed_seconds: u64,
    ) -> Result<Game, StorageError> {
        let game_index = self
            .library
            .games
            .iter()
            .position(|game| game.id == id)
            .ok_or_else(|| StorageError::GameNotFound { id: id.to_string() })?;
        if elapsed_seconds == 0 {
            return Ok(self.library.games[game_index].clone());
        }
        let play_time = self.library.games[game_index]
            .play_time_seconds
            .checked_add(elapsed_seconds)
            .ok_or_else(|| StorageError::PlayStatisticOverflow {
                record: "game",
                id: id.to_string(),
                field: "PlayTime",
            })?;
        let element = find_record_element_mut(&mut self.root, "Game", "ID", id)
            .ok_or_else(|| StorageError::GameNotFound { id: id.to_string() })?;
        let game = &mut self.library.games[game_index];
        game.play_time_seconds = play_time;
        set_child_text(element, "PlayTime", &play_time.to_string());
        Ok(game.clone())
    }

    pub fn record_additional_application_play_start(
        &mut self,
        id: &str,
        last_played: &str,
    ) -> Result<AdditionalApplication, StorageError> {
        validate_last_played_timestamp(last_played)?;
        let application_index = self
            .library
            .additional_applications
            .iter()
            .position(|application| application.id == id)
            .ok_or_else(|| StorageError::AdditionalApplicationNotFound { id: id.to_string() })?;
        let play_count = self.library.additional_applications[application_index]
            .play_count
            .checked_add(1)
            .ok_or_else(|| StorageError::PlayStatisticOverflow {
                record: "additional application",
                id: id.to_string(),
                field: "PlayCount",
            })?;
        let element = find_record_element_mut(&mut self.root, "AdditionalApplication", "Id", id)
            .ok_or_else(|| StorageError::AdditionalApplicationNotFound { id: id.to_string() })?;
        let application = &mut self.library.additional_applications[application_index];
        application.play_count = play_count;
        application.last_played = Some(last_played.to_string());
        set_child_text(element, "PlayCount", &play_count.to_string());
        set_child_text(element, "LastPlayed", last_played);
        Ok(application.clone())
    }

    pub fn record_additional_application_play_time(
        &mut self,
        id: &str,
        elapsed_seconds: u64,
    ) -> Result<AdditionalApplication, StorageError> {
        let application_index = self
            .library
            .additional_applications
            .iter()
            .position(|application| application.id == id)
            .ok_or_else(|| StorageError::AdditionalApplicationNotFound { id: id.to_string() })?;
        if elapsed_seconds == 0 {
            return Ok(self.library.additional_applications[application_index].clone());
        }
        let play_time = self.library.additional_applications[application_index]
            .play_time_seconds
            .checked_add(elapsed_seconds)
            .ok_or_else(|| StorageError::PlayStatisticOverflow {
                record: "additional application",
                id: id.to_string(),
                field: "PlayTime",
            })?;
        let element = find_record_element_mut(&mut self.root, "AdditionalApplication", "Id", id)
            .ok_or_else(|| StorageError::AdditionalApplicationNotFound { id: id.to_string() })?;
        let application = &mut self.library.additional_applications[application_index];
        application.play_time_seconds = play_time;
        set_child_text(element, "PlayTime", &play_time.to_string());
        Ok(application.clone())
    }

    pub fn write_to(&self, writer: impl Write) -> Result<(), StorageError> {
        write_xml_root(&self.root, writer)
    }

    pub fn to_xml_bytes(&self) -> Result<Vec<u8>, StorageError> {
        let mut bytes = Vec::new();
        self.write_to(&mut bytes)?;
        Ok(bytes)
    }

    /// Writes a new document without replacing an existing user file.
    pub fn save_new(&self, path: impl AsRef<Path>) -> Result<(), StorageError> {
        save_new_bytes(path.as_ref(), &self.to_xml_bytes()?)
    }

    /// Replaces an existing platform document only after serializing and
    /// reparsing the complete output. A uniquely named sibling backup is made
    /// durable before the same-directory atomic replacement.
    pub fn save_atomic(&self) -> Result<AtomicSaveReport, StorageError> {
        self.save_atomic_to(&self.source_path)
    }

    pub fn save_atomic_to(
        &self,
        target: impl AsRef<Path>,
    ) -> Result<AtomicSaveReport, StorageError> {
        let target = target.as_ref();
        let bytes = self.to_xml_bytes()?;
        let expected = if target == self.source_path {
            self.source_revision.as_ref()
        } else {
            None
        };
        save_atomic_bytes_if_revision(target, &bytes, expected, |candidate| {
            PlatformDocument::from_reader(target, candidate).map(|_| ())
        })
    }
}

fn record_indices(root: &Element, record_name: &str, selector: Option<(&str, &str)>) -> Vec<usize> {
    root.children
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            let record = node.as_element()?;
            (record.name == record_name
                && selector.is_none_or(|(field, value)| {
                    child_text(record, field).as_deref() == Some(value)
                }))
            .then_some(index)
        })
        .collect()
}

fn matching_record_indices(
    root: &Element,
    record_name: &str,
    id_field: &str,
    id: &str,
) -> Vec<usize> {
    record_indices(root, record_name, Some((id_field, id)))
}

fn invalid_game_record_edit(
    record: &'static str,
    game_id: &str,
    reason: impl Into<String>,
) -> StorageError {
    StorageError::InvalidGameRecordEdit {
        record,
        game_id: game_id.to_string(),
        reason: reason.into(),
    }
}

fn validate_indexed_record_edits<T>(
    record: &'static str,
    game_id: &str,
    original_count: usize,
    edits: &[IndexedPlatformRecordEdit<T>],
) -> Result<(), StorageError> {
    let mut previous = None;
    let mut saw_new = false;
    for edit in edits {
        match edit.source_index {
            None => saw_new = true,
            Some(index) => {
                if saw_new {
                    return Err(invalid_game_record_edit(
                        record,
                        game_id,
                        "new rows must follow retained source rows",
                    ));
                }
                if index >= original_count {
                    return Err(invalid_game_record_edit(
                        record,
                        game_id,
                        format!("source index {index} is outside 0..{original_count}"),
                    ));
                }
                if previous.is_some_and(|previous| previous >= index) {
                    return Err(invalid_game_record_edit(
                        record,
                        game_id,
                        "source indices must be unique and remain in source order",
                    ));
                }
                previous = Some(index);
            }
        }
    }
    Ok(())
}

fn validate_game_save_metadata_edits(
    game_id: &str,
    original_count: usize,
    edits: &[IndexedGameSaveMetadataEdit],
) -> Result<(), StorageError> {
    let mut previous = None;
    for edit in edits {
        if edit.source_index >= original_count {
            return Err(invalid_game_record_edit(
                "game save",
                game_id,
                format!(
                    "source index {} is outside 0..{original_count}",
                    edit.source_index
                ),
            ));
        }
        if previous.is_some_and(|previous| previous >= edit.source_index) {
            return Err(invalid_game_record_edit(
                "game save",
                game_id,
                "source indices must be unique and remain in source order",
            ));
        }
        if [
            edit.metadata.title.as_deref(),
            edit.metadata.save_group_name.as_deref(),
            edit.metadata.save_group_id.as_deref(),
        ]
        .into_iter()
        .flatten()
        .any(str::is_empty)
        {
            return Err(invalid_game_record_edit(
                "game save",
                game_id,
                "optional metadata must use None instead of an empty string",
            ));
        }
        previous = Some(edit.source_index);
    }
    Ok(())
}

fn validate_playlist_row_identity(document: &PlaylistDocument) -> Result<(), StorageError> {
    let mut game_ids = std::collections::BTreeSet::new();
    for game in &document.games {
        if !game_ids.insert(game.game_id.to_lowercase()) {
            return Err(StorageError::DuplicatePlaylistGame {
                id: document.playlist.id.clone(),
                game_id: game.game_id.clone(),
            });
        }
    }
    Ok(())
}

fn validate_indexed_playlist_filters(
    original_count: usize,
    edits: &[IndexedPlatformRecordEdit<PlaylistFilter>],
) -> Result<(), StorageError> {
    validate_playlist_source_indices("filter", original_count, edits)?;
    for edit in edits {
        edit.record.validate()?;
    }
    Ok(())
}

fn validate_indexed_playlist_games(
    playlist_id: &str,
    original_count: usize,
    edits: &[IndexedPlatformRecordEdit<PlaylistGame>],
) -> Result<(), StorageError> {
    validate_playlist_source_indices("game", original_count, edits)?;
    let mut game_ids = std::collections::BTreeSet::new();
    for edit in edits {
        edit.record.validate()?;
        if !game_ids.insert(edit.record.game_id.to_lowercase()) {
            return Err(StorageError::DuplicatePlaylistGame {
                id: playlist_id.to_string(),
                game_id: edit.record.game_id.clone(),
            });
        }
    }
    Ok(())
}

fn validate_playlist_source_indices<T>(
    record: &'static str,
    original_count: usize,
    edits: &[IndexedPlatformRecordEdit<T>],
) -> Result<(), StorageError> {
    let mut previous = None;
    let mut saw_new = false;
    for edit in edits {
        match edit.source_index {
            None => saw_new = true,
            Some(index) => {
                let reason = if saw_new {
                    Some("new rows must follow retained source rows".to_string())
                } else if index >= original_count {
                    Some(format!(
                        "source index {index} is outside 0..{original_count}"
                    ))
                } else if previous.is_some_and(|previous| previous >= index) {
                    Some("source indices must be unique and remain in source order".to_string())
                } else {
                    None
                };
                if let Some(reason) = reason {
                    return Err(StorageError::InvalidPlaylistRecordEdit { record, reason });
                }
                previous = Some(index);
            }
        }
    }
    Ok(())
}

fn validate_emulator_platform_edits(
    emulator_id: &str,
    original_count: usize,
    edits: &[IndexedPlatformRecordEdit<EmulatorPlatform>],
) -> Result<(), StorageError> {
    let mut previous = None;
    let mut saw_new = false;
    for edit in edits {
        match edit.source_index {
            None => saw_new = true,
            Some(index) => {
                let reason = if saw_new {
                    Some("new rows must follow retained source rows".to_string())
                } else if index >= original_count {
                    Some(format!(
                        "source index {index} is outside 0..{original_count}"
                    ))
                } else if previous.is_some_and(|previous| previous >= index) {
                    Some("source indices must be unique and remain in source order".to_string())
                } else {
                    None
                };
                if let Some(reason) = reason {
                    return Err(StorageError::InvalidEmulatorPlatformEdit {
                        id: emulator_id.to_string(),
                        reason,
                    });
                }
                previous = Some(index);
            }
        }
    }
    Ok(())
}

fn validate_emulator_platform_records(
    emulator_id: &str,
    platforms: &[EmulatorPlatform],
) -> Result<(), StorageError> {
    let mut names = BTreeSet::new();
    let mut defaults = BTreeSet::new();
    for platform in platforms {
        platform.validate()?;
        if platform.emulator_id != emulator_id {
            return Err(StorageError::EmulatorPlatformOwnerMismatch {
                expected: emulator_id.to_string(),
                actual: platform.emulator_id.clone(),
            });
        }
        let key = platform.platform.to_lowercase();
        if platform.default && !defaults.insert(key.clone()) {
            return Err(StorageError::DuplicateEmulatorPlatformDefault {
                id: emulator_id.to_string(),
                platform: platform.platform.clone(),
            });
        }
        if !names.insert(key) {
            return Err(StorageError::DuplicateEmulatorPlatform {
                id: emulator_id.to_string(),
                platform: platform.platform.clone(),
            });
        }
    }
    Ok(())
}

fn replace_playlist_filter_rows(
    root: &mut Element,
    original: &[PlaylistFilter],
    edits: &[IndexedPlatformRecordEdit<PlaylistFilter>],
) -> Result<(), StorageError> {
    let root_indices = record_indices(root, "PlaylistFilter", None);
    if root_indices.len() != original.len() {
        return Err(StorageError::InvalidPlaylistRecordEdit {
            record: "filter",
            reason: format!(
                "typed/XML source count mismatch ({} versus {})",
                original.len(),
                root_indices.len()
            ),
        });
    }
    let mut retained = vec![false; original.len()];
    for edit in edits {
        let Some(source_index) = edit.source_index else {
            continue;
        };
        retained[source_index] = true;
        let element = root.children[root_indices[source_index]]
            .as_mut_element()
            .expect("playlist filter index must identify an element");
        update_playlist_filter_element(element, &original[source_index], &edit.record);
    }
    for source_index in (0..root_indices.len()).rev() {
        if !retained[source_index] {
            root.children.remove(root_indices[source_index]);
        }
    }
    for edit in edits.iter().filter(|edit| edit.source_index.is_none()) {
        root.children
            .push(XMLNode::Element(playlist_filter_element(&edit.record)));
    }
    Ok(())
}

fn replace_playlist_game_rows(
    root: &mut Element,
    original: &[PlaylistGame],
    edits: &[IndexedPlatformRecordEdit<PlaylistGame>],
) -> Result<(), StorageError> {
    let root_indices = record_indices(root, "PlaylistGame", None);
    if root_indices.len() != original.len() {
        return Err(StorageError::InvalidPlaylistRecordEdit {
            record: "game",
            reason: format!(
                "typed/XML source count mismatch ({} versus {})",
                original.len(),
                root_indices.len()
            ),
        });
    }
    let mut retained = vec![false; original.len()];
    for edit in edits {
        let Some(source_index) = edit.source_index else {
            continue;
        };
        retained[source_index] = true;
        let element = root.children[root_indices[source_index]]
            .as_mut_element()
            .expect("playlist game index must identify an element");
        update_playlist_game_element(element, &original[source_index], &edit.record);
    }
    for source_index in (0..root_indices.len()).rev() {
        if !retained[source_index] {
            root.children.remove(root_indices[source_index]);
        }
    }
    for edit in edits.iter().filter(|edit| edit.source_index.is_none()) {
        root.children
            .push(XMLNode::Element(playlist_game_element(&edit.record)));
    }
    Ok(())
}

fn validate_indexed_platform_folder_edits(
    platform: &str,
    original_count: usize,
    edits: &[IndexedPlatformRecordEdit<PlatformFolder>],
) -> Result<(), StorageError> {
    let mut previous = None;
    let mut saw_new = false;
    for edit in edits {
        match edit.source_index {
            None => saw_new = true,
            Some(index) => {
                let reason = if saw_new {
                    Some("new rows must follow retained source rows".to_string())
                } else if index >= original_count {
                    Some(format!(
                        "source index {index} is outside 0..{original_count}"
                    ))
                } else if previous.is_some_and(|previous| previous >= index) {
                    Some("source indices must be unique and remain in source order".to_string())
                } else {
                    None
                };
                if let Some(reason) = reason {
                    return Err(StorageError::InvalidPlatformFolderEdit {
                        platform: platform.to_string(),
                        reason,
                    });
                }
                previous = Some(index);
            }
        }
    }
    Ok(())
}

fn validate_indexed_category_parent_edits(
    category: &str,
    original_count: usize,
    edits: &[IndexedPlatformRecordEdit<ParentRelationship>],
) -> Result<(), StorageError> {
    let invalid = |reason: String| StorageError::InvalidPlatformCategoryParentEdit {
        category: category.to_string(),
        reason,
    };
    let mut previous = None;
    let mut saw_new = false;
    let mut targets = std::collections::BTreeSet::new();
    for edit in edits {
        match edit.source_index {
            None => saw_new = true,
            Some(index) => {
                if saw_new {
                    return Err(invalid(
                        "new rows must follow retained source rows".to_string(),
                    ));
                }
                if index >= original_count {
                    return Err(invalid(format!(
                        "source index {index} is outside 0..{original_count}"
                    )));
                }
                if previous.is_some_and(|previous| previous >= index) {
                    return Err(invalid(
                        "source indices must be unique and remain in source order".to_string(),
                    ));
                }
                previous = Some(index);
            }
        }
        edit.record.validate()?;
        if edit.record.platform_category_name.as_deref() != Some(category)
            || edit
                .record
                .platform_name
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
            || edit
                .record
                .playlist_id
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
        {
            return Err(invalid(
                "every row must identify the exact edited category as its only child".to_string(),
            ));
        }
        if edit
            .record
            .parent_platform_category_name
            .as_deref()
            .is_some_and(|name| name.eq_ignore_ascii_case(category))
        {
            return Err(invalid("a category cannot be its own parent".to_string()));
        }
        let target = if let Some(name) = edit
            .record
            .parent_platform_category_name
            .as_deref()
            .filter(|name| !name.trim().is_empty())
        {
            format!("category:{}", name.to_lowercase())
        } else if let Some(name) = edit
            .record
            .parent_platform_name
            .as_deref()
            .filter(|name| !name.trim().is_empty())
        {
            format!("platform:{}", name.to_lowercase())
        } else if let Some(id) = edit
            .record
            .parent_playlist_id
            .as_deref()
            .filter(|id| !id.trim().is_empty())
        {
            format!("playlist:{}", id.to_lowercase())
        } else {
            "root".to_string()
        };
        if !targets.insert(target.clone()) {
            return Err(invalid(format!(
                "the parent placement occurs more than once: {target}"
            )));
        }
    }
    if edits.is_empty() {
        return Err(invalid(
            "a category must retain at least one hierarchy placement".to_string(),
        ));
    }
    Ok(())
}

fn validate_indexed_playlist_parent_edits(
    playlist_id: &str,
    original_count: usize,
    edits: &[IndexedPlatformRecordEdit<ParentRelationship>],
) -> Result<(), StorageError> {
    let invalid = |reason: String| StorageError::InvalidPlaylistParentEdit {
        id: playlist_id.to_string(),
        reason,
    };
    let mut previous = None;
    let mut saw_new = false;
    let mut targets = std::collections::BTreeSet::new();
    for edit in edits {
        match edit.source_index {
            None => saw_new = true,
            Some(index) => {
                if saw_new {
                    return Err(invalid(
                        "new rows must follow retained source rows".to_string(),
                    ));
                }
                if index >= original_count {
                    return Err(invalid(format!(
                        "source index {index} is outside 0..{original_count}"
                    )));
                }
                if previous.is_some_and(|previous| previous >= index) {
                    return Err(invalid(
                        "source indices must be unique and remain in source order".to_string(),
                    ));
                }
                previous = Some(index);
            }
        }
        edit.record.validate()?;
        if edit.record.playlist_id.as_deref() != Some(playlist_id)
            || edit
                .record
                .platform_name
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
            || edit
                .record
                .platform_category_name
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
        {
            return Err(invalid(
                "every row must identify the exact edited playlist as its only child".to_string(),
            ));
        }
        if edit
            .record
            .parent_playlist_id
            .as_deref()
            .is_some_and(|id| id.eq_ignore_ascii_case(playlist_id))
        {
            return Err(invalid("a playlist cannot be its own parent".to_string()));
        }
        let target = if let Some(name) = edit
            .record
            .parent_platform_category_name
            .as_deref()
            .filter(|name| !name.trim().is_empty())
        {
            format!("category:{}", name.to_lowercase())
        } else if let Some(name) = edit
            .record
            .parent_platform_name
            .as_deref()
            .filter(|name| !name.trim().is_empty())
        {
            format!("platform:{}", name.to_lowercase())
        } else if let Some(id) = edit
            .record
            .parent_playlist_id
            .as_deref()
            .filter(|id| !id.trim().is_empty())
        {
            format!("playlist:{}", id.to_lowercase())
        } else {
            "root".to_string()
        };
        if !targets.insert(target.clone()) {
            return Err(invalid(format!(
                "the parent placement occurs more than once: {target}"
            )));
        }
    }
    if edits.is_empty() {
        return Err(invalid(
            "a playlist must retain at least one hierarchy placement".to_string(),
        ));
    }
    Ok(())
}

fn ensure_record_index_alignment(
    record: &'static str,
    game_id: &str,
    typed_count: usize,
    xml_count: usize,
) -> Result<(), StorageError> {
    if typed_count == xml_count {
        Ok(())
    } else {
        Err(invalid_game_record_edit(
            record,
            game_id,
            format!("typed/XML source count mismatch ({typed_count} versus {xml_count})"),
        ))
    }
}

fn additional_application_matches_game_launch(
    application: &AdditionalApplication,
    game: &Game,
) -> bool {
    let game_uses_emulator = game
        .emulator_id
        .as_deref()
        .is_none_or(|id| !lb_domain::is_unassigned_emulator_id(id));
    application.application_path == game.application_path
        && application.command_line == game.command_line
        && application.use_dos_box == game.use_dos_box
        && application.use_emulator == game_uses_emulator
        && (!game_uses_emulator || application.emulator_id == game.emulator_id)
}

fn update_expanded_game_element(element: &mut Element, game: &Game) {
    set_child_text(element, "ID", &game.id);
    set_child_text(element, "Title", &game.title);
    set_child_text(element, "Platform", &game.platform);
    set_child_text(element, "ApplicationPath", &game.application_path);
    set_optional_child_text(element, "CommandLine", game.command_line.as_deref());
    set_optional_child_text(element, "Emulator", game.emulator_id.as_deref());
    set_optional_child_text(element, "CloneOf", game.clone_of.as_deref());
    set_child_text(element, "UseDosBox", &game.use_dos_box.to_string());
    set_child_text(element, "UseScummVM", &game.use_scumm_vm.to_string());
    set_optional_child_text(element, "Developer", game.developer.as_deref());
    set_optional_child_text(element, "Publisher", game.publisher.as_deref());
    set_optional_child_text(element, "Region", game.region.as_deref());
    set_optional_child_text(element, "ReleaseDate", game.release_date.as_deref());
    set_optional_child_text(element, "Version", game.version.as_deref());
    set_optional_child_text(element, "Status", game.status.as_deref());
    set_optional_child_text(
        element,
        "Installed",
        game.installed.map(|value| value.to_string()).as_deref(),
    );
    set_child_text(element, "PlayCount", &game.play_count.to_string());
    set_child_text(element, "PlayTime", &game.play_time_seconds.to_string());
    set_optional_child_text(element, "LastPlayedDate", game.last_played_date.as_deref());
    set_optional_child_text(element, "GogAppId", game.gog_app_id.as_deref());
    set_optional_child_text(element, "OriginAppId", game.origin_app_id.as_deref());
    set_optional_child_text(
        element,
        "OriginInstallPath",
        game.origin_install_path.as_deref(),
    );
    set_child_text(
        element,
        "HasCloudSynced",
        &game.has_cloud_synced.to_string(),
    );
}

fn emulator_element(emulator: &Emulator) -> Element {
    let mut element = Element::new("Emulator");
    set_child_text(&mut element, "ID", &emulator.id);
    set_child_text(&mut element, "Title", &emulator.title);
    set_child_text(&mut element, "ApplicationPath", &emulator.application_path);
    set_optional_child_text(
        &mut element,
        "CommandLine",
        emulator.command_line.as_deref(),
    );
    set_optional_child_text(
        &mut element,
        "DefaultPlatform",
        emulator.default_platform.as_deref(),
    );
    set_child_text(
        &mut element,
        "AutoExtract",
        &emulator.auto_extract.to_string(),
    );
    set_child_text(
        &mut element,
        "AggressiveWindowHiding",
        &emulator.aggressive_window_hiding.to_string(),
    );
    set_child_text(
        &mut element,
        "DefaultPauseSettingsPushed",
        &emulator.default_pause_settings_pushed.to_string(),
    );
    set_child_text(
        &mut element,
        "DisableShutdownScreen",
        &emulator.disable_shutdown_screen.to_string(),
    );
    set_child_text(
        &mut element,
        "EnableHardcoreAchievements",
        &emulator.enable_hardcore_achievements.to_string(),
    );
    set_child_text(
        &mut element,
        "FileNameWithoutExtensionAndPath",
        &emulator.file_name_without_extension_and_path.to_string(),
    );
    set_child_text(
        &mut element,
        "ForcefulPauseScreenActivation",
        &emulator.forceful_pause_screen_activation.to_string(),
    );
    set_child_text(
        &mut element,
        "HideAllNonExclusiveFullscreenWindows",
        &emulator
            .hide_all_non_exclusive_fullscreen_windows
            .to_string(),
    );
    set_child_text(
        &mut element,
        "HideConsole",
        &emulator.hide_console.to_string(),
    );
    set_child_text(
        &mut element,
        "HideMouseCursorInGame",
        &emulator.hide_mouse_cursor_in_game.to_string(),
    );
    set_child_text(
        &mut element,
        "LoginToCheevoOnGameLaunch",
        &emulator.login_to_cheevo_on_game_launch.to_string(),
    );
    set_child_text(&mut element, "NoQuotes", &emulator.no_quotes.to_string());
    set_child_text(&mut element, "NoSpace", &emulator.no_space.to_string());
    set_child_text(
        &mut element,
        "SkipVersionCheck",
        &emulator.skip_version_check.to_string(),
    );
    set_child_text(
        &mut element,
        "StartupLoadDelay",
        &emulator.startup_load_delay.to_string(),
    );
    set_child_text(
        &mut element,
        "SuspendProcessOnPause",
        &emulator.suspend_process_on_pause.to_string(),
    );
    set_child_text(
        &mut element,
        "UsePauseScreen",
        &emulator.use_pause_screen.to_string(),
    );
    set_child_text(
        &mut element,
        "UseStartupScreen",
        &emulator.use_startup_screen.to_string(),
    );
    set_optional_child_text(
        &mut element,
        "AutoHotkeyScript",
        emulator.auto_hotkey_script.as_deref(),
    );
    set_optional_child_text(
        &mut element,
        "ExitAutoHotkeyScript",
        emulator.exit_auto_hotkey_script.as_deref(),
    );
    set_optional_child_text(
        &mut element,
        "LoadStateAutoHotkeyScript",
        emulator.load_state_auto_hotkey_script.as_deref(),
    );
    set_optional_child_text(
        &mut element,
        "PauseAutoHotkeyScript",
        emulator.pause_auto_hotkey_script.as_deref(),
    );
    set_optional_child_text(
        &mut element,
        "ResetAutoHotkeyScript",
        emulator.reset_auto_hotkey_script.as_deref(),
    );
    set_optional_child_text(
        &mut element,
        "ResumeAutoHotkeyScript",
        emulator.resume_auto_hotkey_script.as_deref(),
    );
    set_optional_child_text(
        &mut element,
        "SaveStateAutoHotkeyScript",
        emulator.save_state_auto_hotkey_script.as_deref(),
    );
    set_optional_child_text(
        &mut element,
        "SwapDiscsAutoHotkeyScript",
        emulator.swap_discs_auto_hotkey_script.as_deref(),
    );
    element
}

fn update_emulator_element(element: &mut Element, original: &Emulator, updated: &Emulator) {
    macro_rules! update_text {
        ($field:ident, $xml:literal) => {
            if original.$field != updated.$field {
                set_child_text(element, $xml, &updated.$field.to_string());
            }
        };
    }
    macro_rules! update_optional_text {
        ($field:ident, $xml:literal) => {
            if original.$field != updated.$field {
                set_optional_child_text(element, $xml, updated.$field.as_deref());
            }
        };
    }

    update_text!(title, "Title");
    update_text!(application_path, "ApplicationPath");
    update_optional_text!(command_line, "CommandLine");
    update_optional_text!(default_platform, "DefaultPlatform");
    update_text!(auto_extract, "AutoExtract");
    update_text!(aggressive_window_hiding, "AggressiveWindowHiding");
    update_text!(default_pause_settings_pushed, "DefaultPauseSettingsPushed");
    update_text!(disable_shutdown_screen, "DisableShutdownScreen");
    update_text!(enable_hardcore_achievements, "EnableHardcoreAchievements");
    update_text!(
        file_name_without_extension_and_path,
        "FileNameWithoutExtensionAndPath"
    );
    update_text!(
        forceful_pause_screen_activation,
        "ForcefulPauseScreenActivation"
    );
    update_text!(
        hide_all_non_exclusive_fullscreen_windows,
        "HideAllNonExclusiveFullscreenWindows"
    );
    update_text!(hide_console, "HideConsole");
    update_text!(hide_mouse_cursor_in_game, "HideMouseCursorInGame");
    update_text!(login_to_cheevo_on_game_launch, "LoginToCheevoOnGameLaunch");
    update_text!(no_quotes, "NoQuotes");
    update_text!(no_space, "NoSpace");
    update_text!(skip_version_check, "SkipVersionCheck");
    update_text!(startup_load_delay, "StartupLoadDelay");
    update_text!(suspend_process_on_pause, "SuspendProcessOnPause");
    update_text!(use_pause_screen, "UsePauseScreen");
    update_text!(use_startup_screen, "UseStartupScreen");
    update_optional_text!(auto_hotkey_script, "AutoHotkeyScript");
    update_optional_text!(exit_auto_hotkey_script, "ExitAutoHotkeyScript");
    update_optional_text!(load_state_auto_hotkey_script, "LoadStateAutoHotkeyScript");
    update_optional_text!(pause_auto_hotkey_script, "PauseAutoHotkeyScript");
    update_optional_text!(reset_auto_hotkey_script, "ResetAutoHotkeyScript");
    update_optional_text!(resume_auto_hotkey_script, "ResumeAutoHotkeyScript");
    update_optional_text!(save_state_auto_hotkey_script, "SaveStateAutoHotkeyScript");
    update_optional_text!(swap_discs_auto_hotkey_script, "SwapDiscsAutoHotkeyScript");
}

fn emulator_platform_element(platform: &EmulatorPlatform) -> Element {
    let mut element = Element::new("EmulatorPlatform");
    set_child_text(&mut element, "Emulator", &platform.emulator_id);
    set_child_text(&mut element, "Platform", &platform.platform);
    set_optional_child_text(
        &mut element,
        "CommandLine",
        platform.command_line.as_deref(),
    );
    set_child_text(&mut element, "Default", &platform.default.to_string());
    set_optional_child_text(
        &mut element,
        "AutoExtract",
        platform
            .auto_extract
            .map(|value| value.to_string())
            .as_deref(),
    );
    set_child_text(
        &mut element,
        "M3uDiscLoadEnabled",
        &platform.m3u_disc_load_enabled.to_string(),
    );
    element
}

fn update_emulator_platform_element(
    element: &mut Element,
    original: &EmulatorPlatform,
    updated: &EmulatorPlatform,
) {
    if original.platform != updated.platform {
        set_child_text(element, "Platform", &updated.platform);
    }
    if original.command_line != updated.command_line {
        set_optional_child_text(element, "CommandLine", updated.command_line.as_deref());
    }
    if original.default != updated.default {
        set_child_text(element, "Default", &updated.default.to_string());
    }
    if original.auto_extract != updated.auto_extract {
        set_optional_child_text(
            element,
            "AutoExtract",
            updated
                .auto_extract
                .map(|value| value.to_string())
                .as_deref(),
        );
    }
    if original.m3u_disc_load_enabled != updated.m3u_disc_load_enabled {
        set_child_text(
            element,
            "M3uDiscLoadEnabled",
            &updated.m3u_disc_load_enabled.to_string(),
        );
    }
}

fn emulator_platform_indices(root: &Element, emulator_id: &str) -> Vec<usize> {
    root.children
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            let element = node.as_element()?;
            (element.name == "EmulatorPlatform"
                && child_text(element, "Emulator")
                    .is_some_and(|id| id.eq_ignore_ascii_case(emulator_id)))
            .then_some(index)
        })
        .collect()
}

fn exactly_one_emulator_record(
    emulator_id: &str,
    matches: &[usize],
) -> Result<usize, StorageError> {
    match matches {
        [] => Err(StorageError::EmulatorNotFound {
            id: emulator_id.to_string(),
        }),
        [index] => Ok(*index),
        _ => Err(StorageError::DuplicateEmulatorId {
            id: emulator_id.to_string(),
        }),
    }
}

fn clear_peer_default_emulators(
    root: &mut Element,
    retained_emulator_id: &str,
    platforms: &BTreeSet<String>,
) {
    if platforms.is_empty() {
        return;
    }
    for element in root.children.iter_mut().filter_map(XMLNode::as_mut_element) {
        if element.name != "EmulatorPlatform"
            || child_text(element, "Emulator")
                .is_some_and(|id| id.eq_ignore_ascii_case(retained_emulator_id))
        {
            continue;
        }
        let is_target = child_text(element, "Platform")
            .is_some_and(|platform| platforms.contains(&platform.to_lowercase()));
        let is_default =
            child_text(element, "Default").is_some_and(|value| value.eq_ignore_ascii_case("true"));
        if is_target && is_default {
            set_child_text(element, "Default", "false");
        }
    }
}

fn emulator_record_insertion_index(root: &Element, record_name: &str) -> usize {
    fn rank(name: &str) -> Option<usize> {
        ["Emulator", "EmulatorPlatform"]
            .iter()
            .position(|candidate| *candidate == name)
    }

    let target_rank = rank(record_name).expect("editable emulator record family has a rank");
    let after_existing = root
        .children
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            let element = node.as_element()?;
            rank(&element.name)
                .filter(|rank| *rank <= target_rank)
                .map(|_| index + 1)
        })
        .max();
    after_existing.unwrap_or_else(|| {
        root.children
            .iter()
            .position(|node| {
                node.as_element()
                    .and_then(|element| rank(&element.name))
                    .is_some_and(|rank| rank > target_rank)
            })
            .unwrap_or(root.children.len())
    })
}

fn platform_record_insertion_index(root: &Element, record_name: &str) -> usize {
    fn rank(name: &str) -> Option<usize> {
        [
            "Game",
            "AdditionalApplication",
            "AlternateName",
            "CustomField",
            "Mount",
            "GameControllerSupport",
            "GameSave",
        ]
        .iter()
        .position(|candidate| *candidate == name)
    }

    let target_rank = rank(record_name).expect("editable platform record family has a rank");
    root.children
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            let element = node.as_element()?;
            rank(&element.name)
                .filter(|rank| *rank <= target_rank)
                .map(|_| index + 1)
        })
        .max()
        .unwrap_or_default()
}

fn catalog_record_insertion_index(root: &Element, record_name: &str) -> usize {
    fn rank(name: &str) -> Option<usize> {
        ["Platform", "PlatformCategory", "PlatformFolder"]
            .iter()
            .position(|candidate| *candidate == name)
    }

    let target_rank = rank(record_name).expect("editable catalog record family has a rank");
    let after_existing = root
        .children
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            let element = node.as_element()?;
            rank(&element.name)
                .filter(|rank| *rank <= target_rank)
                .map(|_| index + 1)
        })
        .max();
    after_existing.unwrap_or_else(|| {
        root.children
            .iter()
            .position(|node| {
                node.as_element()
                    .and_then(|element| rank(&element.name))
                    .is_some_and(|rank| rank > target_rank)
            })
            .unwrap_or(root.children.len())
    })
}

fn exactly_one_editable_record(
    record_name: &str,
    selector: Option<(&str, &str)>,
    matches: &[usize],
) -> Result<usize, StorageError> {
    let selector = selector
        .map(|(field, value)| format!(" where {field}={value}"))
        .unwrap_or_default();
    match matches {
        [] => Err(StorageError::EditableRecordNotFound {
            record: record_name.to_string(),
            selector,
        }),
        [index] => Ok(*index),
        _ => Err(StorageError::EditableRecordAmbiguous {
            record: record_name.to_string(),
            selector,
            count: matches.len(),
        }),
    }
}

fn parse_xml_root(path: &Path, reader: impl Read) -> Result<Element, StorageError> {
    Element::parse(reader).map_err(|source| StorageError::Parse {
        path: path.to_path_buf(),
        source,
    })
}

fn write_xml_root(root: &Element, writer: impl Write) -> Result<(), StorageError> {
    let config = EmitterConfig::new()
        .perform_indent(true)
        .write_document_declaration(true);
    root.write_with_config(writer, config)
        .map_err(StorageError::WriteXml)
}

fn save_new_bytes(path: &Path, bytes: &[u8]) -> Result<(), StorageError> {
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|source| StorageError::Write {
            path: path.to_path_buf(),
            source,
        })?;
    file.write_all(bytes)
        .and_then(|()| file.flush())
        .and_then(|()| file.sync_all())
        .map_err(|source| StorageError::Write {
            path: path.to_path_buf(),
            source,
        })
}

fn save_atomic_bytes(
    target: &Path,
    bytes: &[u8],
    validate: impl FnOnce(&[u8]) -> Result<(), StorageError>,
) -> Result<AtomicSaveReport, StorageError> {
    save_atomic_bytes_with_operations(target, bytes, validate, replace_file, sync_parent_directory)
}

fn save_atomic_bytes_if_revision(
    target: &Path,
    bytes: &[u8],
    expected: Option<&FileRevision>,
    validate: impl FnOnce(&[u8]) -> Result<(), StorageError>,
) -> Result<AtomicSaveReport, StorageError> {
    if let Some(expected) = expected {
        let actual = FileRevision::read(target)?;
        if actual != *expected {
            return Err(StorageError::WriteConflict {
                path: target.to_path_buf(),
                expected: expected.clone(),
                actual,
            });
        }
    }
    save_atomic_bytes(target, bytes, validate)
}

fn save_atomic_bytes_with_operations(
    target: &Path,
    bytes: &[u8],
    validate: impl FnOnce(&[u8]) -> Result<(), StorageError>,
    replace: impl FnOnce(&Path, &Path) -> Result<(), std::io::Error>,
    sync_parent: impl FnOnce(&Path) -> Result<(), std::io::Error>,
) -> Result<AtomicSaveReport, StorageError> {
    let metadata = fs::symlink_metadata(target).map_err(|source| StorageError::Write {
        path: target.to_path_buf(),
        source,
    })?;
    if !metadata.file_type().is_file() {
        return Err(StorageError::AtomicTargetNotFile {
            path: target.to_path_buf(),
        });
    }

    validate(bytes).map_err(|source| StorageError::AtomicValidation {
        path: target.to_path_buf(),
        source: Box::new(source),
    })?;

    let (temporary_path, mut temporary) = create_unique_sibling(target, "temporary", true)?;
    let staged = temporary
        .write_all(bytes)
        .and_then(|()| temporary.flush())
        .and_then(|()| temporary.set_permissions(metadata.permissions()))
        .and_then(|()| temporary.sync_all())
        .map_err(|source| StorageError::Write {
            path: temporary_path.clone(),
            source,
        });
    if let Err(error) = staged {
        let _ = fs::remove_file(&temporary_path);
        return Err(error);
    }
    drop(temporary);

    let (backup_path, mut backup) = match create_unique_sibling(target, "backup", false) {
        Ok(backup) => backup,
        Err(error) => {
            let _ = fs::remove_file(&temporary_path);
            return Err(error);
        }
    };
    let backup_result = match fs::File::open(target) {
        Ok(mut original) => std::io::copy(&mut original, &mut backup)
            .and_then(|_| backup.flush())
            .and_then(|()| backup.set_permissions(metadata.permissions()))
            .and_then(|()| backup.sync_all())
            .map(|_| ())
            .map_err(|source| StorageError::Write {
                path: backup_path.clone(),
                source,
            }),
        Err(source) => Err(StorageError::Read {
            path: target.to_path_buf(),
            source,
        }),
    };
    if let Err(error) = backup_result {
        let _ = fs::remove_file(&temporary_path);
        let _ = fs::remove_file(&backup_path);
        return Err(error);
    }
    drop(backup);

    if let Err(source) = replace(&temporary_path, target) {
        let _ = fs::remove_file(&temporary_path);
        return Err(StorageError::AtomicReplace {
            path: target.to_path_buf(),
            backup: backup_path,
            source,
        });
    }
    if let Err(source) = sync_parent(target) {
        return Err(StorageError::AtomicDirectorySync {
            path: target.to_path_buf(),
            backup: backup_path,
            source,
        });
    }

    Ok(AtomicSaveReport {
        target: target.to_path_buf(),
        backup: backup_path,
    })
}

fn create_unique_sibling(
    target: &Path,
    kind: &str,
    hidden: bool,
) -> Result<(PathBuf, fs::File), StorageError> {
    let parent = target.parent().unwrap_or(Path::new("."));
    let file_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("platform.xml");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    for attempt in 0..1000 {
        let leading_dot = if hidden { "." } else { "" };
        let candidate = parent.join(format!(
            "{leading_dot}{file_name}.lbport-{kind}-{}-{timestamp}-{attempt}",
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
                return Err(StorageError::Write {
                    path: candidate,
                    source,
                });
            }
        }
    }
    Err(StorageError::UniqueSiblingExhausted {
        path: target.to_path_buf(),
        kind: kind.to_string(),
    })
}

fn create_unique_sibling_directory(
    target: &Path,
    kind: &str,
    hidden: bool,
) -> Result<PathBuf, StorageError> {
    let parent = target.parent().unwrap_or(Path::new("."));
    let file_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("directory");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    for attempt in 0..1000 {
        let leading_dot = if hidden { "." } else { "" };
        let candidate = parent.join(format!(
            "{leading_dot}{file_name}.lbport-{kind}-{}-{timestamp}-{attempt}",
            std::process::id()
        ));
        match fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(source) => {
                return Err(StorageError::Write {
                    path: candidate,
                    source,
                });
            }
        }
    }
    Err(StorageError::UniqueSiblingExhausted {
        path: target.to_path_buf(),
        kind: kind.to_string(),
    })
}

fn copy_directory_contents(source: &Path, destination: &Path) -> Result<(), StorageError> {
    let source_metadata =
        fs::symlink_metadata(source).map_err(|source_error| StorageError::Read {
            path: source.to_path_buf(),
            source: source_error,
        })?;
    if !source_metadata.file_type().is_dir() || source_metadata.file_type().is_symlink() {
        return Err(StorageError::AtomicSourceNotDirectory {
            path: source.to_path_buf(),
        });
    }
    let mut entries = fs::read_dir(source)
        .map_err(|source_error| StorageError::Read {
            path: source.to_path_buf(),
            source: source_error,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source_error| StorageError::Read {
            path: source.to_path_buf(),
            source: source_error,
        })?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        if entry.file_name().to_str().is_none() {
            return Err(StorageError::AtomicDirectoryNonUnicode { path: entry.path() });
        }
        let source_path = entry.path();
        let target_path = destination.join(entry.file_name());
        let metadata =
            fs::symlink_metadata(&source_path).map_err(|source_error| StorageError::Read {
                path: source_path.clone(),
                source: source_error,
            })?;
        if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
            fs::create_dir(&target_path).map_err(|source| StorageError::Write {
                path: target_path.clone(),
                source,
            })?;
            copy_directory_contents(&source_path, &target_path)?;
            set_path_metadata(&target_path, &metadata)?;
            sync_directory(&target_path).map_err(|source| StorageError::Write {
                path: target_path,
                source,
            })?;
        } else if metadata.file_type().is_file() && !metadata.file_type().is_symlink() {
            let mut input = fs::File::open(&source_path).map_err(|source| StorageError::Read {
                path: source_path.clone(),
                source,
            })?;
            let mut output = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&target_path)
                .map_err(|source| StorageError::Write {
                    path: target_path.clone(),
                    source,
                })?;
            std::io::copy(&mut input, &mut output)
                .and_then(|_| output.flush())
                .and_then(|()| output.set_permissions(metadata.permissions()))
                .and_then(|()| output.set_times(file_times(&metadata)))
                .and_then(|()| output.sync_all())
                .map_err(|source| StorageError::Write {
                    path: target_path,
                    source,
                })?;
        } else {
            return Err(StorageError::AtomicDirectoryUnsafeEntry { path: source_path });
        }
    }
    set_path_metadata(destination, &source_metadata)?;
    sync_directory(destination).map_err(|source| StorageError::Write {
        path: destination.to_path_buf(),
        source,
    })
}

fn file_times(metadata: &fs::Metadata) -> fs::FileTimes {
    let mut times = fs::FileTimes::new();
    if let Ok(accessed) = metadata.accessed() {
        times = times.set_accessed(accessed);
    }
    if let Ok(modified) = metadata.modified() {
        times = times.set_modified(modified);
    }
    times
}

fn set_path_metadata(path: &Path, metadata: &fs::Metadata) -> Result<(), StorageError> {
    fs::set_permissions(path, metadata.permissions()).map_err(|source| StorageError::Write {
        path: path.to_path_buf(),
        source,
    })?;
    fs::File::open(path)
        .and_then(|file| file.set_times(file_times(metadata)))
        .or_else(|error| if path.is_dir() { Ok(()) } else { Err(error) })
        .map_err(|source| StorageError::Write {
            path: path.to_path_buf(),
            source,
        })
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), std::io::Error> {
    fs::File::open(path).and_then(|directory| directory.sync_all())
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> Result<(), std::io::Error> {
    Ok(())
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
    let parent = target.parent().unwrap_or(Path::new("."));
    fs::File::open(parent).and_then(|directory| directory.sync_all())
}

#[cfg(not(unix))]
fn sync_parent_directory(_target: &Path) -> Result<(), std::io::Error> {
    Ok(())
}

#[derive(Clone, Debug)]
pub struct LaunchBoxLibrary {
    root: PathBuf,
    documents: Vec<PlatformDocument>,
}

impl LaunchBoxLibrary {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let path = path.as_ref();
        if path.is_file() {
            let document = PlatformDocument::load(path)?;
            let root = path.parent().unwrap_or(Path::new(".")).to_path_buf();
            return Ok(Self {
                root,
                documents: vec![document],
            });
        }

        let platform_dir = find_platform_directory(path)?;
        let platform_files = platform_files(&platform_dir)?;
        let documents = platform_files
            .into_iter()
            .map(PlatformDocument::load)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            root: path.to_path_buf(),
            documents,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn documents(&self) -> &[PlatformDocument] {
        &self.documents
    }

    pub fn games(&self) -> impl Iterator<Item = &Game> {
        self.documents
            .iter()
            .flat_map(|document| document.library.games.iter())
    }
}

fn find_platform_directory(path: &Path) -> Result<PathBuf, StorageError> {
    [
        path.join("Data").join("Platforms"),
        path.join("Platforms"),
        path.to_path_buf(),
    ]
    .into_iter()
    .find(|candidate| candidate.is_dir())
    .ok_or_else(|| StorageError::NoPlatformDirectory {
        path: path.to_path_buf(),
    })
}

fn platform_files(platform_dir: &Path) -> Result<Vec<PathBuf>, StorageError> {
    let mut files = fs::read_dir(platform_dir)
        .map_err(|source| StorageError::Read {
            path: platform_dir.to_path_buf(),
            source,
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|entry| {
            entry
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("xml"))
        })
        .collect::<Vec<_>>();
    files.sort();
    if files.is_empty() {
        return Err(StorageError::NoPlatformDocuments {
            path: platform_dir.to_path_buf(),
        });
    }
    Ok(files)
}

#[derive(Debug, Deserialize)]
#[serde(rename = "LaunchBox")]
struct RawLaunchBox {
    #[serde(rename = "Game", default)]
    games: Vec<RawGame>,
    #[serde(rename = "AdditionalApplication", default)]
    additional_applications: Vec<RawAdditionalApplication>,
    #[serde(rename = "Mount", default)]
    mounts: Vec<RawMount>,
    #[serde(rename = "AlternateName", default)]
    alternate_names: Vec<RawAlternateName>,
    #[serde(rename = "CustomField", default)]
    custom_fields: Vec<RawCustomField>,
    #[serde(rename = "GameControllerSupport", default)]
    controller_support: Vec<RawGameControllerSupport>,
    #[serde(rename = "GameSave", default)]
    game_saves: Vec<RawGameSave>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RawGame {
    #[serde(rename = "AggressiveWindowHiding")]
    aggressive_window_hiding: bool,
    #[serde(rename = "AndroidBackgroundPath")]
    android_background_path: Option<String>,
    #[serde(rename = "AndroidBackgroundThumbPath")]
    android_background_thumb_path: Option<String>,
    #[serde(rename = "AndroidBoxFrontFullPath")]
    android_box_front_full_path: Option<String>,
    #[serde(rename = "AndroidBoxFrontThumbPath")]
    android_box_front_thumb_path: Option<String>,
    #[serde(rename = "AndroidClearLogoFullPath")]
    android_clear_logo_full_path: Option<String>,
    #[serde(rename = "AndroidClearLogoThumbPath")]
    android_clear_logo_thumb_path: Option<String>,
    #[serde(rename = "AndroidGameTitleScreenshotPath")]
    android_game_title_screenshot_path: Option<String>,
    #[serde(rename = "AndroidGameTitleScreenshotThumbPath")]
    android_game_title_screenshot_thumb_path: Option<String>,
    #[serde(rename = "AndroidGameplayScreenshotPath")]
    android_gameplay_screenshot_path: Option<String>,
    #[serde(rename = "AndroidGameplayScreenshotThumbPath")]
    android_gameplay_screenshot_thumb_path: Option<String>,
    #[serde(rename = "AndroidVideoPath")]
    android_video_path: Option<String>,
    #[serde(rename = "ApplicationPath")]
    application_path: String,
    #[serde(rename = "Broken")]
    broken: bool,
    #[serde(rename = "CloneOf")]
    clone_of: Option<String>,
    #[serde(rename = "CommandLine")]
    command_line: Option<String>,
    #[serde(rename = "CommunityStarRating")]
    community_star_rating: f64,
    #[serde(rename = "CommunityStarRatingTotalVotes")]
    community_star_rating_total_votes: u32,
    #[serde(rename = "Completed")]
    completed: bool,
    #[serde(rename = "ConfigurationCommandLine")]
    configuration_command_line: Option<String>,
    #[serde(rename = "ConfigurationPath")]
    configuration_path: Option<String>,
    #[serde(rename = "CustomDosBoxVersionPath")]
    custom_dos_box_version_path: Option<String>,
    #[serde(rename = "DatabaseID")]
    database_id: Option<u32>,
    #[serde(rename = "DateAdded")]
    date_added: String,
    #[serde(rename = "DateModified")]
    date_modified: String,
    #[serde(rename = "Developer")]
    developer: Option<String>,
    #[serde(rename = "DisableShutdownScreen")]
    disable_shutdown_screen: bool,
    #[serde(rename = "DosBoxConfigurationPath")]
    dos_box_configuration_path: Option<String>,
    #[serde(rename = "Emulator")]
    emulator_id: Option<String>,
    #[serde(rename = "Favorite")]
    favorite: bool,
    #[serde(rename = "ForcefulPauseScreenActivation")]
    forceful_pause_screen_activation: bool,
    #[serde(rename = "Genre")]
    genre: Option<String>,
    #[serde(rename = "GogAppId")]
    gog_app_id: Option<String>,
    #[serde(rename = "HasCloudSynced")]
    has_cloud_synced: bool,
    #[serde(rename = "HasGogAchievements")]
    has_gog_achievements: Option<bool>,
    #[serde(rename = "HasSteamAchievements")]
    has_steam_achievements: Option<bool>,
    #[serde(rename = "Hide")]
    hidden: bool,
    #[serde(rename = "HideAllNonExclusiveFullscreenWindows")]
    hide_all_non_exclusive_fullscreen_windows: bool,
    #[serde(rename = "HideMouseCursorInGame")]
    hide_mouse_cursor_in_game: bool,
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "Installed")]
    installed: Option<bool>,
    #[serde(rename = "LastPlayedDate")]
    last_played_date: Option<String>,
    #[serde(rename = "LastSteamScan")]
    last_steam_scan: Option<String>,
    #[serde(rename = "LoadStateAutoHotkeyScript")]
    load_state_auto_hotkey_script: Option<String>,
    #[serde(rename = "ManualPath")]
    manual_path: Option<String>,
    #[serde(rename = "MaxPlayers")]
    max_players: Option<u32>,
    #[serde(rename = "MissingBackgroundImage")]
    missing_background_image: bool,
    #[serde(rename = "MissingBannerImage")]
    missing_banner_image: bool,
    #[serde(rename = "MissingBox3dImage")]
    missing_box_3d_image: bool,
    #[serde(rename = "MissingBoxFrontImage")]
    missing_box_front_image: bool,
    #[serde(rename = "MissingCart3dImage")]
    missing_cart_3d_image: bool,
    #[serde(rename = "MissingCartImage")]
    missing_cart_image: bool,
    #[serde(rename = "MissingClearLogoImage")]
    missing_clear_logo_image: bool,
    #[serde(rename = "MissingManual")]
    missing_manual: bool,
    #[serde(rename = "MissingMarqueeImage")]
    missing_marquee_image: bool,
    #[serde(rename = "MissingMusic")]
    missing_music: bool,
    #[serde(rename = "MissingScreenshotImage")]
    missing_screenshot_image: bool,
    #[serde(rename = "MissingVideo")]
    missing_video: bool,
    #[serde(rename = "MusicPath")]
    music_path: Option<String>,
    #[serde(rename = "Notes")]
    notes: Option<String>,
    #[serde(rename = "OriginAppId")]
    origin_app_id: Option<String>,
    #[serde(rename = "OriginInstallPath")]
    origin_install_path: Option<String>,
    #[serde(rename = "OverrideDefaultPauseScreenSettings")]
    override_default_pause_screen_settings: bool,
    #[serde(rename = "OverrideDefaultStartupScreenSettings")]
    override_default_startup_screen_settings: bool,
    #[serde(rename = "PauseAutoHotkeyScript")]
    pause_auto_hotkey_script: Option<String>,
    #[serde(rename = "Platform")]
    platform: Option<String>,
    #[serde(rename = "PlayCount")]
    play_count: u32,
    #[serde(rename = "PlayMode")]
    play_mode: Option<String>,
    #[serde(rename = "PlayTime")]
    play_time_seconds: u64,
    #[serde(rename = "Portable")]
    portable: bool,
    #[serde(rename = "Progress")]
    progress: Option<String>,
    #[serde(rename = "Publisher")]
    publisher: Option<String>,
    #[serde(rename = "Rating")]
    rating: Option<String>,
    #[serde(rename = "Region")]
    region: Option<String>,
    #[serde(rename = "ReleaseDate")]
    release_date: Option<String>,
    #[serde(rename = "ReleaseType")]
    release_type: Option<String>,
    #[serde(rename = "ResetAutoHotkeyScript")]
    reset_auto_hotkey_script: Option<String>,
    #[serde(rename = "ResumeAutoHotkeyScript")]
    resume_auto_hotkey_script: Option<String>,
    #[serde(rename = "RetroAchievementsBeatenHardcore")]
    retro_achievements_beaten_hardcore: bool,
    #[serde(rename = "RetroAchievementsBeatenSoftcore")]
    retro_achievements_beaten_softcore: bool,
    #[serde(rename = "RetroAchievementsHash")]
    retro_achievements_hash: Option<String>,
    #[serde(rename = "RetroAchievementsId")]
    retro_achievements_id: Option<u32>,
    #[serde(rename = "RootFolder")]
    root_folder: Option<String>,
    #[serde(rename = "SaveStateAutoHotkeyScript")]
    save_state_auto_hotkey_script: Option<String>,
    #[serde(rename = "ScummVMAspectCorrection")]
    scumm_vm_aspect_correction: bool,
    #[serde(rename = "ScummVMFullscreen")]
    scumm_vm_fullscreen: bool,
    #[serde(rename = "ScummVMGameDataFolderPath")]
    scumm_vm_game_data_folder_path: Option<String>,
    #[serde(rename = "ScummVMGameType")]
    scumm_vm_game_type: Option<String>,
    #[serde(rename = "Series")]
    series: Option<String>,
    #[serde(rename = "SortTitle")]
    sort_title: Option<String>,
    #[serde(rename = "Source")]
    source: Option<String>,
    #[serde(rename = "StarRating")]
    star_rating: u8,
    #[serde(rename = "StarRatingFloat")]
    star_rating_float: f64,
    #[serde(rename = "StartupLoadDelay")]
    startup_load_delay: u32,
    #[serde(rename = "Status")]
    status: Option<String>,
    #[serde(rename = "SuspendProcessOnPause")]
    suspend_process_on_pause: bool,
    #[serde(rename = "SwapDiscsAutoHotkeyScript")]
    swap_discs_auto_hotkey_script: Option<String>,
    #[serde(rename = "ThemeVideoPath")]
    theme_video_path: Option<String>,
    #[serde(rename = "Title")]
    title: String,
    #[serde(rename = "UseDosBox")]
    use_dos_box: bool,
    #[serde(rename = "UsePauseScreen")]
    use_pause_screen: bool,
    #[serde(rename = "UseScummVM")]
    use_scumm_vm: bool,
    #[serde(rename = "UseStartupScreen")]
    use_startup_screen: bool,
    #[serde(rename = "Version")]
    version: Option<String>,
    #[serde(rename = "VideoPath")]
    video_path: Option<String>,
    #[serde(rename = "VideoUrl")]
    video_url: Option<String>,
    #[serde(rename = "WikipediaURL")]
    wikipedia_url: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RawAdditionalApplication {
    #[serde(rename = "Id")]
    id: String,
    #[serde(rename = "GameID")]
    game_id: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "ApplicationPath")]
    application_path: String,
    #[serde(rename = "CommandLine")]
    command_line: Option<String>,
    #[serde(rename = "AutoRunBefore")]
    auto_run_before: bool,
    #[serde(rename = "AutoRunAfter")]
    auto_run_after: bool,
    #[serde(rename = "WaitForExit")]
    wait_for_exit: bool,
    #[serde(rename = "UseEmulator")]
    use_emulator: bool,
    #[serde(rename = "EmulatorId")]
    emulator_id: Option<String>,
    #[serde(rename = "UseDosBox")]
    use_dos_box: bool,
    #[serde(rename = "Priority")]
    priority: i32,
    #[serde(rename = "PlayCount")]
    play_count: u32,
    #[serde(rename = "PlayTime")]
    play_time_seconds: u64,
    #[serde(rename = "Disc")]
    disc: Option<u32>,
    #[serde(rename = "SideA")]
    side_a: bool,
    #[serde(rename = "SideB")]
    side_b: bool,
    #[serde(rename = "Developer")]
    developer: Option<String>,
    #[serde(rename = "Publisher")]
    publisher: Option<String>,
    #[serde(rename = "Region")]
    region: Option<String>,
    #[serde(rename = "ReleaseDate")]
    release_date: Option<String>,
    #[serde(rename = "Version")]
    version: Option<String>,
    #[serde(rename = "Status")]
    status: Option<String>,
    #[serde(rename = "Installed")]
    installed: Option<bool>,
    #[serde(rename = "LastPlayed")]
    last_played: Option<String>,
    #[serde(rename = "GogAppId")]
    gog_app_id: Option<String>,
    #[serde(rename = "OriginAppId")]
    origin_app_id: Option<String>,
    #[serde(rename = "OriginInstallPath")]
    origin_install_path: Option<String>,
    #[serde(rename = "HasCloudSynced")]
    has_cloud_synced: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RawMount {
    #[serde(rename = "GameID")]
    game_id: String,
    #[serde(rename = "DriveLetter")]
    drive_letter: String,
    #[serde(rename = "Filesystem")]
    filesystem: String,
    #[serde(rename = "MountType")]
    mount_type: String,
    #[serde(rename = "Path")]
    path: String,
    #[serde(rename = "Type")]
    media_type: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RawAlternateName {
    #[serde(rename = "GameID")]
    game_id: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Region")]
    region: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RawCustomField {
    #[serde(rename = "GameID")]
    game_id: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Value")]
    value: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RawGameControllerSupport {
    #[serde(rename = "ControllerId")]
    controller_id: String,
    #[serde(rename = "GameId")]
    game_id: String,
    #[serde(rename = "SupportLevel")]
    support_level: Option<i32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RawGameSave {
    #[serde(rename = "GameId")]
    game_id: String,
    #[serde(rename = "AdditionalApplicationId")]
    additional_application_id: Option<String>,
    #[serde(rename = "EmulatorCore")]
    emulator_core: String,
    #[serde(rename = "EmulatorFileName")]
    emulator_file_name: String,
    #[serde(rename = "Title")]
    title: Option<String>,
    #[serde(rename = "SaveGroupName")]
    save_group_name: Option<String>,
    #[serde(rename = "DisplayChipText")]
    display_chip_text: Option<String>,
    #[serde(rename = "SaveGroupId")]
    save_group_id: Option<String>,
    #[serde(rename = "MatchLineageId")]
    match_lineage_id: Option<String>,
    #[serde(rename = "MigrationFamilyId")]
    migration_family_id: Option<String>,
    #[serde(rename = "FilePath")]
    file_path: String,
    #[serde(rename = "OriginalFileName")]
    original_file_name: Option<String>,
    #[serde(rename = "Slot")]
    slot: Option<i32>,
    #[serde(rename = "ReportedFileSizeBytes")]
    reported_file_size_bytes: Option<i64>,
    #[serde(rename = "ReportedLastModifiedUtc")]
    reported_last_modified_utc: Option<String>,
    #[serde(rename = "Md5")]
    md5: Option<String>,
}

fn load_platform_index(path: &Path) -> Result<PlatformLibrary, StorageError> {
    let file = fs::File::open(path).map_err(|source| StorageError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let raw: RawLaunchBox = quick_xml::de::from_reader(BufReader::new(file)).map_err(|source| {
        StorageError::ReadIndex {
            path: path.to_path_buf(),
            source,
        }
    })?;
    let fallback_platform = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("Unknown Platform")
        .to_string();
    let games = raw
        .games
        .into_iter()
        .map(|raw| raw.into_game(&fallback_platform))
        .collect::<Result<Vec<_>, _>>()?;
    let additional_applications = raw
        .additional_applications
        .into_iter()
        .map(RawAdditionalApplication::into_domain)
        .collect::<Result<Vec<_>, _>>()?;
    let mounts = raw
        .mounts
        .into_iter()
        .map(RawMount::into_domain)
        .collect::<Result<Vec<_>, _>>()?;
    let alternate_names = raw
        .alternate_names
        .into_iter()
        .map(RawAlternateName::into_domain)
        .collect::<Result<Vec<_>, _>>()?;
    let custom_fields = raw
        .custom_fields
        .into_iter()
        .map(RawCustomField::into_domain)
        .collect::<Result<Vec<_>, _>>()?;
    let controller_support = raw
        .controller_support
        .into_iter()
        .map(RawGameControllerSupport::into_domain)
        .collect::<Result<Vec<_>, _>>()?;
    let game_saves = raw
        .game_saves
        .into_iter()
        .map(RawGameSave::into_domain)
        .collect::<Result<Vec<_>, _>>()?;
    let platform = PlatformLibrary {
        name: games
            .first()
            .map(|game| game.platform.clone())
            .filter(|name| !name.trim().is_empty())
            .unwrap_or(fallback_platform),
        source_path: path.to_path_buf(),
        games,
        additional_applications,
        mounts,
        alternate_names,
        custom_fields,
        controller_support,
        game_saves,
    };
    platform.validate()?;
    Ok(platform)
}

impl RawGame {
    fn into_game(self, fallback_platform: &str) -> Result<Game, StorageError> {
        let game = Game {
            id: self.id,
            title: self.title,
            sort_title: non_empty(self.sort_title),
            platform: non_empty(self.platform).unwrap_or_else(|| fallback_platform.to_string()),
            application_path: self.application_path,
            command_line: non_empty(self.command_line),
            emulator_id: non_empty(self.emulator_id),
            notes: non_empty(self.notes),
            clone_of: non_empty(self.clone_of),
            database_id: self.database_id,
            date_added: self.date_added,
            date_modified: self.date_modified,
            developer: non_empty(self.developer),
            genre: non_empty(self.genre),
            max_players: self.max_players,
            play_mode: non_empty(self.play_mode),
            progress: non_empty(self.progress),
            publisher: non_empty(self.publisher),
            rating: non_empty(self.rating),
            region: non_empty(self.region),
            release_date: non_empty(self.release_date),
            release_type: non_empty(self.release_type),
            series: non_empty(self.series),
            source: non_empty(self.source),
            status: non_empty(self.status),
            version: non_empty(self.version),
            wikipedia_url: non_empty(self.wikipedia_url),
            favorite: self.favorite,
            completed: self.completed,
            hidden: self.hidden,
            broken: self.broken,
            portable: self.portable,
            installed: self.installed,
            play_count: self.play_count,
            play_time_seconds: self.play_time_seconds,
            star_rating: self.star_rating,
            star_rating_float: self.star_rating_float,
            community_star_rating: self.community_star_rating,
            community_star_rating_total_votes: self.community_star_rating_total_votes,
            last_played_date: non_empty(self.last_played_date),
            aggressive_window_hiding: self.aggressive_window_hiding,
            disable_shutdown_screen: self.disable_shutdown_screen,
            forceful_pause_screen_activation: self.forceful_pause_screen_activation,
            hide_all_non_exclusive_fullscreen_windows: self
                .hide_all_non_exclusive_fullscreen_windows,
            hide_mouse_cursor_in_game: self.hide_mouse_cursor_in_game,
            override_default_pause_screen_settings: self.override_default_pause_screen_settings,
            override_default_startup_screen_settings: self.override_default_startup_screen_settings,
            suspend_process_on_pause: self.suspend_process_on_pause,
            use_pause_screen: self.use_pause_screen,
            use_startup_screen: self.use_startup_screen,
            startup_load_delay: self.startup_load_delay,
            configuration_command_line: non_empty(self.configuration_command_line),
            configuration_path: non_empty(self.configuration_path),
            load_state_auto_hotkey_script: non_empty(self.load_state_auto_hotkey_script),
            pause_auto_hotkey_script: non_empty(self.pause_auto_hotkey_script),
            reset_auto_hotkey_script: non_empty(self.reset_auto_hotkey_script),
            resume_auto_hotkey_script: non_empty(self.resume_auto_hotkey_script),
            save_state_auto_hotkey_script: non_empty(self.save_state_auto_hotkey_script),
            swap_discs_auto_hotkey_script: non_empty(self.swap_discs_auto_hotkey_script),
            use_dos_box: self.use_dos_box,
            custom_dos_box_version_path: non_empty(self.custom_dos_box_version_path),
            dos_box_configuration_path: non_empty(self.dos_box_configuration_path),
            use_scumm_vm: self.use_scumm_vm,
            scumm_vm_aspect_correction: self.scumm_vm_aspect_correction,
            scumm_vm_fullscreen: self.scumm_vm_fullscreen,
            scumm_vm_game_data_folder_path: non_empty(self.scumm_vm_game_data_folder_path),
            scumm_vm_game_type: non_empty(self.scumm_vm_game_type),
            manual_path: non_empty(self.manual_path),
            music_path: non_empty(self.music_path),
            root_folder: non_empty(self.root_folder),
            theme_video_path: non_empty(self.theme_video_path),
            video_path: non_empty(self.video_path),
            video_url: non_empty(self.video_url),
            missing_background_image: self.missing_background_image,
            missing_banner_image: self.missing_banner_image,
            missing_box_3d_image: self.missing_box_3d_image,
            missing_box_front_image: self.missing_box_front_image,
            missing_cart_3d_image: self.missing_cart_3d_image,
            missing_cart_image: self.missing_cart_image,
            missing_clear_logo_image: self.missing_clear_logo_image,
            missing_manual: self.missing_manual,
            missing_marquee_image: self.missing_marquee_image,
            missing_music: self.missing_music,
            missing_screenshot_image: self.missing_screenshot_image,
            missing_video: self.missing_video,
            gog_app_id: non_empty(self.gog_app_id),
            origin_app_id: non_empty(self.origin_app_id),
            origin_install_path: non_empty(self.origin_install_path),
            has_cloud_synced: self.has_cloud_synced,
            has_gog_achievements: self.has_gog_achievements,
            has_steam_achievements: self.has_steam_achievements,
            last_steam_scan: non_empty(self.last_steam_scan),
            retro_achievements_beaten_hardcore: self.retro_achievements_beaten_hardcore,
            retro_achievements_beaten_softcore: self.retro_achievements_beaten_softcore,
            retro_achievements_hash: non_empty(self.retro_achievements_hash),
            retro_achievements_id: self.retro_achievements_id,
            android_background_path: non_empty(self.android_background_path),
            android_background_thumb_path: non_empty(self.android_background_thumb_path),
            android_box_front_full_path: non_empty(self.android_box_front_full_path),
            android_box_front_thumb_path: non_empty(self.android_box_front_thumb_path),
            android_clear_logo_full_path: non_empty(self.android_clear_logo_full_path),
            android_clear_logo_thumb_path: non_empty(self.android_clear_logo_thumb_path),
            android_game_title_screenshot_path: non_empty(self.android_game_title_screenshot_path),
            android_game_title_screenshot_thumb_path: non_empty(
                self.android_game_title_screenshot_thumb_path,
            ),
            android_gameplay_screenshot_path: non_empty(self.android_gameplay_screenshot_path),
            android_gameplay_screenshot_thumb_path: non_empty(
                self.android_gameplay_screenshot_thumb_path,
            ),
            android_video_path: non_empty(self.android_video_path),
        };
        game.validate()?;
        Ok(game)
    }
}

impl RawAdditionalApplication {
    fn into_domain(self) -> Result<AdditionalApplication, StorageError> {
        let application = AdditionalApplication {
            id: self.id,
            game_id: self.game_id,
            name: self.name,
            application_path: self.application_path,
            command_line: non_empty(self.command_line),
            auto_run_before: self.auto_run_before,
            auto_run_after: self.auto_run_after,
            wait_for_exit: self.wait_for_exit,
            use_emulator: self.use_emulator,
            emulator_id: non_empty(self.emulator_id),
            use_dos_box: self.use_dos_box,
            priority: self.priority,
            play_count: self.play_count,
            play_time_seconds: self.play_time_seconds,
            disc: self.disc,
            side_a: self.side_a,
            side_b: self.side_b,
            developer: non_empty(self.developer),
            publisher: non_empty(self.publisher),
            region: non_empty(self.region),
            release_date: non_empty(self.release_date),
            version: non_empty(self.version),
            status: non_empty(self.status),
            installed: self.installed,
            last_played: non_empty(self.last_played),
            gog_app_id: non_empty(self.gog_app_id),
            origin_app_id: non_empty(self.origin_app_id),
            origin_install_path: non_empty(self.origin_install_path),
            has_cloud_synced: self.has_cloud_synced,
        };
        application.validate()?;
        Ok(application)
    }
}

impl RawMount {
    fn into_domain(self) -> Result<Mount, StorageError> {
        mount_from_fields(
            self.game_id,
            self.drive_letter,
            self.filesystem,
            self.mount_type,
            self.path,
            self.media_type,
        )
    }
}

impl RawAlternateName {
    fn into_domain(self) -> Result<AlternateName, StorageError> {
        let alternate_name = AlternateName {
            game_id: self.game_id,
            name: self.name,
            region: non_empty(self.region),
        };
        alternate_name.validate()?;
        Ok(alternate_name)
    }
}

impl RawCustomField {
    fn into_domain(self) -> Result<CustomField, StorageError> {
        let field = CustomField {
            game_id: self.game_id,
            name: self.name,
            value: self.value,
        };
        field.validate()?;
        Ok(field)
    }
}

impl RawGameControllerSupport {
    fn into_domain(self) -> Result<GameControllerSupport, StorageError> {
        let support = GameControllerSupport {
            controller_id: self.controller_id,
            game_id: self.game_id,
            support_level: self.support_level,
        };
        support.validate()?;
        Ok(support)
    }
}

impl RawGameSave {
    fn into_domain(self) -> Result<GameSave, StorageError> {
        let save = GameSave {
            game_id: self.game_id,
            additional_application_id: non_empty(self.additional_application_id),
            emulator_core: self.emulator_core,
            emulator_file_name: self.emulator_file_name,
            title: non_empty(self.title),
            save_group_name: non_empty(self.save_group_name),
            display_chip_text: non_empty(self.display_chip_text),
            save_group_id: non_empty(self.save_group_id),
            match_lineage_id: non_empty(self.match_lineage_id),
            migration_family_id: non_empty(self.migration_family_id),
            file_path: self.file_path,
            original_file_name: non_empty(self.original_file_name),
            slot: self.slot,
            reported_file_size_bytes: self.reported_file_size_bytes,
            reported_last_modified_utc: non_empty(self.reported_last_modified_utc),
            md5: non_empty(self.md5),
        };
        save.validate()?;
        Ok(save)
    }
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.is_empty())
}

fn validate_last_played_timestamp(timestamp: &str) -> Result<(), StorageError> {
    if timestamp.trim().is_empty() {
        return Err(StorageError::EmptyLastPlayedTimestamp);
    }
    Ok(())
}

fn find_record_element_mut<'a>(
    root: &'a mut Element,
    record: &str,
    id_field: &str,
    id: &str,
) -> Option<&'a mut Element> {
    root.children
        .iter_mut()
        .filter_map(XMLNode::as_mut_element)
        .filter(|element| element.name == record)
        .find(|element| child_text(element, id_field).as_deref() == Some(id))
}

fn parse_game(element: &Element, fallback_platform: &str) -> Result<Game, StorageError> {
    let mut bytes = Vec::new();
    element.write(&mut bytes).map_err(StorageError::WriteXml)?;
    let raw: RawGame = quick_xml::de::from_reader(bytes.as_slice()).map_err(|source| {
        StorageError::ReadEmbeddedRecord {
            record: "Game",
            source,
        }
    })?;
    raw.into_game(fallback_platform)
}

fn elements_named<'a>(root: &'a Element, name: &'a str) -> impl Iterator<Item = &'a Element> {
    root.children
        .iter()
        .filter_map(XMLNode::as_element)
        .filter(move |element| element.name == name)
}

fn parse_additional_application(element: &Element) -> Result<AdditionalApplication, StorageError> {
    let application = AdditionalApplication {
        id: required_record_child(element, "AdditionalApplication", "Id")?,
        game_id: required_record_child(element, "AdditionalApplication", "GameID")?,
        name: required_record_child(element, "AdditionalApplication", "Name")?,
        application_path: optional_child(element, "ApplicationPath").unwrap_or_default(),
        command_line: optional_child(element, "CommandLine"),
        auto_run_before: bool_child(element, "AutoRunBefore"),
        auto_run_after: bool_child(element, "AutoRunAfter"),
        wait_for_exit: bool_child(element, "WaitForExit"),
        use_emulator: bool_child(element, "UseEmulator"),
        emulator_id: optional_child(element, "EmulatorId"),
        use_dos_box: bool_child(element, "UseDosBox"),
        priority: number_child(element, "Priority").unwrap_or(0),
        play_count: number_child(element, "PlayCount").unwrap_or(0),
        play_time_seconds: number_child(element, "PlayTime").unwrap_or(0),
        disc: number_child(element, "Disc"),
        side_a: bool_child(element, "SideA"),
        side_b: bool_child(element, "SideB"),
        developer: optional_child(element, "Developer"),
        publisher: optional_child(element, "Publisher"),
        region: optional_child(element, "Region"),
        release_date: optional_child(element, "ReleaseDate"),
        version: optional_child(element, "Version"),
        status: optional_child(element, "Status"),
        installed: optional_bool_child(element, "Installed"),
        last_played: optional_child(element, "LastPlayed"),
        gog_app_id: optional_child(element, "GogAppId"),
        origin_app_id: optional_child(element, "OriginAppId"),
        origin_install_path: optional_child(element, "OriginInstallPath"),
        has_cloud_synced: bool_child(element, "HasCloudSynced"),
    };
    application.validate()?;
    Ok(application)
}

fn parse_mount(element: &Element) -> Result<Mount, StorageError> {
    mount_from_fields(
        required_record_child(element, "Mount", "GameID")?,
        required_record_child(element, "Mount", "DriveLetter")?,
        optional_child(element, "Filesystem").unwrap_or_default(),
        required_record_child(element, "Mount", "MountType")?,
        required_record_child(element, "Mount", "Path")?,
        optional_child(element, "Type").unwrap_or_default(),
    )
}

fn mount_from_fields(
    game_id: String,
    drive_letter: String,
    filesystem: String,
    mount_type: String,
    path: String,
    media_type: String,
) -> Result<Mount, StorageError> {
    let mut characters = drive_letter.chars();
    let drive_letter = characters
        .next()
        .filter(|_| characters.next().is_none())
        .unwrap_or('\0');
    let mount = Mount {
        game_id,
        drive_letter,
        filesystem,
        mount_type,
        path,
        media_type,
    };
    mount.validate()?;
    Ok(mount)
}

fn parse_alternate_name(element: &Element) -> Result<AlternateName, StorageError> {
    let alternate_name = AlternateName {
        game_id: required_record_child(element, "AlternateName", "GameID")?,
        name: required_record_child(element, "AlternateName", "Name")?,
        region: optional_child(element, "Region"),
    };
    alternate_name.validate()?;
    Ok(alternate_name)
}

fn parse_custom_field(element: &Element) -> Result<CustomField, StorageError> {
    let field = CustomField {
        game_id: required_record_child(element, "CustomField", "GameID")?,
        name: required_record_child(element, "CustomField", "Name")?,
        value: present_record_child(element, "CustomField", "Value")?,
    };
    field.validate()?;
    Ok(field)
}

fn parse_controller_support(element: &Element) -> Result<GameControllerSupport, StorageError> {
    let support = GameControllerSupport {
        controller_id: required_record_child(element, "GameControllerSupport", "ControllerId")?,
        game_id: required_record_child(element, "GameControllerSupport", "GameId")?,
        support_level: number_child(element, "SupportLevel"),
    };
    support.validate()?;
    Ok(support)
}

fn parse_game_save(element: &Element) -> Result<GameSave, StorageError> {
    let save = GameSave {
        game_id: required_record_child(element, "GameSave", "GameId")?,
        additional_application_id: optional_child(element, "AdditionalApplicationId"),
        emulator_core: optional_child(element, "EmulatorCore").unwrap_or_default(),
        emulator_file_name: optional_child(element, "EmulatorFileName").unwrap_or_default(),
        title: optional_child(element, "Title"),
        save_group_name: optional_child(element, "SaveGroupName"),
        display_chip_text: optional_child(element, "DisplayChipText"),
        save_group_id: optional_child(element, "SaveGroupId"),
        match_lineage_id: optional_child(element, "MatchLineageId"),
        migration_family_id: optional_child(element, "MigrationFamilyId"),
        file_path: required_record_child(element, "GameSave", "FilePath")?,
        original_file_name: optional_child(element, "OriginalFileName"),
        slot: number_child(element, "Slot"),
        reported_file_size_bytes: number_child(element, "ReportedFileSizeBytes"),
        reported_last_modified_utc: optional_child(element, "ReportedLastModifiedUtc"),
        md5: optional_child(element, "Md5"),
    };
    save.validate()?;
    Ok(save)
}

fn child_text(element: &Element, name: &str) -> Option<String> {
    element
        .get_child(name)
        .and_then(Element::get_text)
        .map(|text| text.into_owned())
}

fn optional_child(element: &Element, name: &str) -> Option<String> {
    child_text(element, name).filter(|value| !value.is_empty())
}

fn required_record_child(
    element: &Element,
    record: &'static str,
    field: &'static str,
) -> Result<String, StorageError> {
    optional_child(element, field).ok_or(StorageError::MissingRecordField { record, field })
}

fn present_record_child(
    element: &Element,
    record: &'static str,
    field: &'static str,
) -> Result<String, StorageError> {
    element
        .get_child(field)
        .map(|child| {
            child
                .get_text()
                .map(|value| value.into_owned())
                .unwrap_or_default()
        })
        .ok_or(StorageError::MissingRecordField { record, field })
}

fn bool_child(element: &Element, name: &str) -> bool {
    child_text(element, name).is_some_and(|value| value.eq_ignore_ascii_case("true"))
}

fn optional_bool_child(element: &Element, name: &str) -> Option<bool> {
    child_text(element, name).map(|value| value.eq_ignore_ascii_case("true"))
}

fn number_child<T: std::str::FromStr>(element: &Element, name: &str) -> Option<T> {
    child_text(element, name)?.parse().ok()
}

fn set_child_text(element: &mut Element, name: &str, value: &str) {
    if let Some(child) = element.get_mut_child(name) {
        child.children.clear();
        child.children.push(XMLNode::Text(value.to_string()));
    } else {
        let mut child = Element::new(name);
        child.children.push(XMLNode::Text(value.to_string()));
        element.children.push(XMLNode::Element(child));
    }
}

fn set_optional_child_text(element: &mut Element, name: &str, value: Option<&str>) {
    if let Some(value) = value {
        set_child_text(element, name, value);
    } else {
        element.children.retain(|node| {
            node.as_element()
                .is_none_or(|child| child.name.as_str() != name)
        });
    }
}

fn clear_child_text_preserving_element(element: &mut Element, name: &str) {
    if let Some(child) = element.get_mut_child(name) {
        child.children.clear();
    }
}

fn update_navigation_metadata_element(
    element: &mut Element,
    original: &NavigationMetadata,
    updated: &NavigationMetadata,
) {
    macro_rules! update_optional_text {
        ($field:ident, $element_name:literal) => {
            if original.$field != updated.$field {
                set_optional_child_text(element, $element_name, updated.$field.as_deref());
            }
        };
    }
    update_optional_text!(nested_name, "NestedName");
    update_optional_text!(sort_title, "SortTitle");
    update_optional_text!(notes, "Notes");
    update_optional_text!(folder, "Folder");
    update_optional_text!(category, "Category");
    update_optional_text!(image_type, "ImageType");
    update_optional_text!(scrape_as, "ScrapeAs");
    if original.hide_in_big_box != updated.hide_in_big_box {
        set_child_text(
            element,
            "HideInBigBox",
            &updated.hide_in_big_box.to_string(),
        );
    }
    if original.local_db_parsed != updated.local_db_parsed {
        set_child_text(
            element,
            "LocalDbParsed",
            &updated.local_db_parsed.to_string(),
        );
    }
    update_optional_text!(last_game_id, "LastGameId");
    update_optional_text!(last_selected_child, "LastSelectedChild");
    update_optional_text!(cpu, "Cpu");
    update_optional_text!(developer, "Developer");
    update_optional_text!(display, "Display");
    update_optional_text!(graphics, "Graphics");
    update_optional_text!(manufacturer, "Manufacturer");
    update_optional_text!(max_controllers, "MaxControllers");
    update_optional_text!(media, "Media");
    update_optional_text!(memory, "Memory");
    update_optional_text!(sound, "Sound");
    update_optional_text!(android_theme_video_path, "AndroidThemeVideoPath");
    update_optional_text!(back_images_folder, "BackImagesFolder");
    update_optional_text!(banner_images_folder, "BannerImagesFolder");
    update_optional_text!(big_box_theme, "BigBoxTheme");
    update_optional_text!(big_box_view, "BigBoxView");
    update_optional_text!(clear_logo_images_folder, "ClearLogoImagesFolder");
    update_optional_text!(fanart_images_folder, "FanartImagesFolder");
    update_optional_text!(front_images_folder, "FrontImagesFolder");
    update_optional_text!(manuals_folder, "ManualsFolder");
    update_optional_text!(music_folder, "MusicFolder");
    update_optional_text!(screenshot_images_folder, "ScreenshotImagesFolder");
    update_optional_text!(steam_banner_images_folder, "SteamBannerImagesFolder");
    update_optional_text!(video_path, "VideoPath");
    update_optional_text!(videos_folder, "VideosFolder");
}

fn update_platform_definition_element(
    element: &mut Element,
    original: &PlatformDefinition,
    updated: &PlatformDefinition,
) {
    let original_metadata = &original.metadata;
    let updated_metadata = &updated.metadata;
    update_navigation_metadata_element(element, original_metadata, updated_metadata);
    if original.release_date != updated.release_date {
        set_optional_child_text(element, "ReleaseDate", updated.release_date.as_deref());
    }
    if original.disable_auto_import != updated.disable_auto_import {
        set_child_text(
            element,
            "DisableAutoImport",
            &updated.disable_auto_import.to_string(),
        );
    }
}

fn update_platform_category_element(
    element: &mut Element,
    original: &PlatformCategory,
    updated: &PlatformCategory,
) {
    update_navigation_metadata_element(element, &original.metadata, &updated.metadata);
    if original.is_autogenerated != updated.is_autogenerated {
        set_child_text(
            element,
            "IsAutogenerated",
            &updated.is_autogenerated.to_string(),
        );
    }
    if original.disable_auto_import != updated.disable_auto_import {
        set_child_text(
            element,
            "DisableAutoImport",
            &updated.disable_auto_import.to_string(),
        );
    }
}

fn update_playlist_element(element: &mut Element, original: &Playlist, updated: &Playlist) {
    update_navigation_metadata_element(element, &original.metadata, &updated.metadata);
    if original.auto_populate != updated.auto_populate {
        set_child_text(element, "AutoPopulate", &updated.auto_populate.to_string());
    }
    if original.include_with_platforms != updated.include_with_platforms {
        set_child_text(
            element,
            "IncludeWithPlatforms",
            &updated.include_with_platforms.to_string(),
        );
    }
    if original.is_autogenerated != updated.is_autogenerated {
        set_child_text(
            element,
            "IsAutogenerated",
            &updated.is_autogenerated.to_string(),
        );
    }
    if original.sort_by != updated.sort_by {
        set_optional_child_text(element, "SortBy", updated.sort_by.as_deref());
    }
}

fn update_playlist_filter_element(
    element: &mut Element,
    original: &PlaylistFilter,
    updated: &PlaylistFilter,
) {
    if original.field_key != updated.field_key {
        set_child_text(element, "FieldKey", &updated.field_key);
    }
    if original.comparison_type_key != updated.comparison_type_key {
        set_child_text(element, "ComparisonTypeKey", &updated.comparison_type_key);
    }
    if original.value != updated.value {
        set_child_text(element, "Value", &updated.value);
    }
}

fn update_playlist_game_element(
    element: &mut Element,
    original: &PlaylistGame,
    updated: &PlaylistGame,
) {
    if original.game_id != updated.game_id {
        set_child_text(element, "GameId", &updated.game_id);
    }
    if original.game_title != updated.game_title {
        set_child_text(element, "GameTitle", &updated.game_title);
    }
    if original.game_platform != updated.game_platform {
        set_child_text(element, "GamePlatform", &updated.game_platform);
    }
    if original.game_file_name != updated.game_file_name {
        set_optional_child_text(
            element,
            "GameFileName",
            (!updated.game_file_name.is_empty()).then_some(updated.game_file_name.as_str()),
        );
    }
    if original.launchbox_db_id != updated.launchbox_db_id {
        set_optional_child_text(
            element,
            "LaunchBoxDbId",
            updated
                .launchbox_db_id
                .map(|value| value.to_string())
                .as_deref(),
        );
    }
    if original.manual_order != updated.manual_order {
        set_child_text(element, "ManualOrder", &updated.manual_order.to_string());
    }
}

fn update_parent_relationship_element(
    element: &mut Element,
    original: &ParentRelationship,
    updated: &ParentRelationship,
) {
    macro_rules! update_optional_text {
        ($field:ident, $element_name:literal) => {
            if original.$field != updated.$field {
                if let Some(value) = updated.$field.as_deref() {
                    set_child_text(element, $element_name, value);
                } else {
                    clear_child_text_preserving_element(element, $element_name);
                }
            }
        };
    }
    update_optional_text!(platform_name, "PlatformName");
    update_optional_text!(playlist_id, "PlaylistId");
    update_optional_text!(platform_category_name, "PlatformCategoryName");
    update_optional_text!(parent_platform_name, "ParentPlatformName");
    update_optional_text!(parent_playlist_id, "ParentPlaylistId");
    update_optional_text!(parent_platform_category_name, "ParentPlatformCategoryName");
}

fn platform_definition_element(platform: &PlatformDefinition) -> Element {
    let metadata = &platform.metadata;
    let mut element = Element::new("Platform");
    set_child_text(&mut element, "Name", &metadata.name);
    for (field, value) in [
        ("NestedName", metadata.nested_name.as_deref()),
        ("SortTitle", metadata.sort_title.as_deref()),
        ("Notes", metadata.notes.as_deref()),
        ("Folder", metadata.folder.as_deref()),
        ("Category", metadata.category.as_deref()),
        ("ImageType", metadata.image_type.as_deref()),
        ("ScrapeAs", metadata.scrape_as.as_deref()),
        ("ReleaseDate", platform.release_date.as_deref()),
    ] {
        set_optional_child_text(&mut element, field, value);
    }
    set_child_text(
        &mut element,
        "HideInBigBox",
        &metadata.hide_in_big_box.to_string(),
    );
    set_child_text(
        &mut element,
        "LocalDbParsed",
        &metadata.local_db_parsed.to_string(),
    );
    for (field, value) in [
        ("LastGameId", metadata.last_game_id.as_deref()),
        ("LastSelectedChild", metadata.last_selected_child.as_deref()),
        ("Cpu", metadata.cpu.as_deref()),
        ("Developer", metadata.developer.as_deref()),
        ("Display", metadata.display.as_deref()),
        ("Graphics", metadata.graphics.as_deref()),
        ("Manufacturer", metadata.manufacturer.as_deref()),
        ("MaxControllers", metadata.max_controllers.as_deref()),
        ("Media", metadata.media.as_deref()),
        ("Memory", metadata.memory.as_deref()),
        ("Sound", metadata.sound.as_deref()),
        (
            "AndroidThemeVideoPath",
            metadata.android_theme_video_path.as_deref(),
        ),
        ("BackImagesFolder", metadata.back_images_folder.as_deref()),
        (
            "BannerImagesFolder",
            metadata.banner_images_folder.as_deref(),
        ),
        ("BigBoxTheme", metadata.big_box_theme.as_deref()),
        ("BigBoxView", metadata.big_box_view.as_deref()),
        (
            "ClearLogoImagesFolder",
            metadata.clear_logo_images_folder.as_deref(),
        ),
        (
            "FanartImagesFolder",
            metadata.fanart_images_folder.as_deref(),
        ),
        ("FrontImagesFolder", metadata.front_images_folder.as_deref()),
        ("ManualsFolder", metadata.manuals_folder.as_deref()),
        ("MusicFolder", metadata.music_folder.as_deref()),
        (
            "ScreenshotImagesFolder",
            metadata.screenshot_images_folder.as_deref(),
        ),
        (
            "SteamBannerImagesFolder",
            metadata.steam_banner_images_folder.as_deref(),
        ),
        ("VideoPath", metadata.video_path.as_deref()),
        ("VideosFolder", metadata.videos_folder.as_deref()),
    ] {
        set_optional_child_text(&mut element, field, value);
    }
    set_child_text(
        &mut element,
        "DisableAutoImport",
        &platform.disable_auto_import.to_string(),
    );
    element
}

fn platform_category_element(category: &PlatformCategory) -> Element {
    let metadata = &category.metadata;
    let mut element = Element::new("PlatformCategory");
    set_child_text(&mut element, "Name", &metadata.name);
    for (field, value) in [
        ("NestedName", metadata.nested_name.as_deref()),
        ("Notes", metadata.notes.as_deref()),
        ("VideoPath", metadata.video_path.as_deref()),
        ("SortTitle", metadata.sort_title.as_deref()),
    ] {
        set_optional_child_text(&mut element, field, value);
    }
    set_child_text(
        &mut element,
        "IsAutogenerated",
        &category.is_autogenerated.to_string(),
    );
    for (field, value) in [
        ("Category", metadata.category.as_deref()),
        ("LastSelectedChild", metadata.last_selected_child.as_deref()),
        ("Developer", metadata.developer.as_deref()),
        ("Manufacturer", metadata.manufacturer.as_deref()),
        ("Cpu", metadata.cpu.as_deref()),
        ("Memory", metadata.memory.as_deref()),
        ("Graphics", metadata.graphics.as_deref()),
        ("Sound", metadata.sound.as_deref()),
        ("Display", metadata.display.as_deref()),
        ("Media", metadata.media.as_deref()),
        ("MaxControllers", metadata.max_controllers.as_deref()),
        ("Folder", metadata.folder.as_deref()),
        ("VideosFolder", metadata.videos_folder.as_deref()),
        ("FrontImagesFolder", metadata.front_images_folder.as_deref()),
        ("BackImagesFolder", metadata.back_images_folder.as_deref()),
        (
            "ClearLogoImagesFolder",
            metadata.clear_logo_images_folder.as_deref(),
        ),
        (
            "FanartImagesFolder",
            metadata.fanart_images_folder.as_deref(),
        ),
        (
            "ScreenshotImagesFolder",
            metadata.screenshot_images_folder.as_deref(),
        ),
        (
            "BannerImagesFolder",
            metadata.banner_images_folder.as_deref(),
        ),
        (
            "SteamBannerImagesFolder",
            metadata.steam_banner_images_folder.as_deref(),
        ),
        ("ManualsFolder", metadata.manuals_folder.as_deref()),
        ("MusicFolder", metadata.music_folder.as_deref()),
        ("ScrapeAs", metadata.scrape_as.as_deref()),
        ("ImageType", metadata.image_type.as_deref()),
        ("LastGameId", metadata.last_game_id.as_deref()),
        ("BigBoxView", metadata.big_box_view.as_deref()),
        ("BigBoxTheme", metadata.big_box_theme.as_deref()),
        (
            "AndroidThemeVideoPath",
            metadata.android_theme_video_path.as_deref(),
        ),
    ] {
        set_optional_child_text(&mut element, field, value);
    }
    set_child_text(
        &mut element,
        "LocalDbParsed",
        &metadata.local_db_parsed.to_string(),
    );
    set_child_text(
        &mut element,
        "HideInBigBox",
        &metadata.hide_in_big_box.to_string(),
    );
    set_child_text(
        &mut element,
        "DisableAutoImport",
        &category.disable_auto_import.to_string(),
    );
    element
}

fn playlist_element(playlist: &Playlist) -> Element {
    let metadata = &playlist.metadata;
    let mut element = Element::new("Playlist");
    set_child_text(&mut element, "PlaylistId", &playlist.id);
    set_child_text(&mut element, "Name", &metadata.name);
    for (field, value) in [
        ("NestedName", metadata.nested_name.as_deref()),
        ("SortTitle", metadata.sort_title.as_deref()),
        ("Notes", metadata.notes.as_deref()),
        ("VideoPath", metadata.video_path.as_deref()),
        ("ImageType", metadata.image_type.as_deref()),
        ("Category", metadata.category.as_deref()),
        ("LastGameId", metadata.last_game_id.as_deref()),
        ("BigBoxView", metadata.big_box_view.as_deref()),
        ("BigBoxTheme", metadata.big_box_theme.as_deref()),
        ("SortBy", playlist.sort_by.as_deref()),
    ] {
        set_optional_child_text(&mut element, field, value);
    }
    set_child_text(
        &mut element,
        "HideInBigBox",
        &metadata.hide_in_big_box.to_string(),
    );
    set_child_text(
        &mut element,
        "LocalDbParsed",
        &metadata.local_db_parsed.to_string(),
    );
    set_child_text(
        &mut element,
        "AutoPopulate",
        &playlist.auto_populate.to_string(),
    );
    set_child_text(
        &mut element,
        "IncludeWithPlatforms",
        &playlist.include_with_platforms.to_string(),
    );
    set_child_text(
        &mut element,
        "IsAutogenerated",
        &playlist.is_autogenerated.to_string(),
    );
    element
}

fn playlist_filter_element(filter: &PlaylistFilter) -> Element {
    let mut element = Element::new("PlaylistFilter");
    set_child_text(&mut element, "FieldKey", &filter.field_key);
    set_child_text(
        &mut element,
        "ComparisonTypeKey",
        &filter.comparison_type_key,
    );
    set_child_text(&mut element, "Value", &filter.value);
    element
}

fn playlist_game_element(game: &PlaylistGame) -> Element {
    let mut element = Element::new("PlaylistGame");
    set_child_text(&mut element, "GameId", &game.game_id);
    set_child_text(&mut element, "GameTitle", &game.game_title);
    set_child_text(&mut element, "GamePlatform", &game.game_platform);
    set_optional_child_text(
        &mut element,
        "GameFileName",
        (!game.game_file_name.is_empty()).then_some(game.game_file_name.as_str()),
    );
    set_optional_child_text(
        &mut element,
        "LaunchBoxDbId",
        game.launchbox_db_id
            .map(|value| value.to_string())
            .as_deref(),
    );
    set_child_text(&mut element, "ManualOrder", &game.manual_order.to_string());
    element
}

fn parent_relationship_element(relationship: &ParentRelationship) -> Element {
    let mut element = Element::new("Parent");
    for (field, value) in [
        ("PlatformName", relationship.platform_name.as_deref()),
        ("PlaylistId", relationship.playlist_id.as_deref()),
        (
            "PlatformCategoryName",
            relationship.platform_category_name.as_deref(),
        ),
        (
            "ParentPlatformName",
            relationship.parent_platform_name.as_deref(),
        ),
        (
            "ParentPlaylistId",
            relationship.parent_playlist_id.as_deref(),
        ),
        (
            "ParentPlatformCategoryName",
            relationship.parent_platform_category_name.as_deref(),
        ),
    ] {
        set_child_text(&mut element, field, value.unwrap_or_default());
    }
    element
}

fn platform_folder_element(folder: &PlatformFolder) -> Element {
    let mut element = Element::new("PlatformFolder");
    set_child_text(&mut element, "Platform", &folder.platform);
    set_child_text(&mut element, "MediaType", &folder.media_type);
    set_child_text(&mut element, "FolderPath", &folder.folder_path);
    element
}

fn minimal_game_element(game: &Game) -> Element {
    let mut element = Element::new("Game");
    for (field, value) in [
        ("ApplicationPath", game.application_path.as_str()),
        ("Broken", "false"),
        ("Completed", "false"),
        ("Favorite", "false"),
        ("Hide", "false"),
        ("ID", game.id.as_str()),
        ("Platform", game.platform.as_str()),
        ("PlayCount", "0"),
        ("PlayTime", "0"),
        ("StarRating", "0"),
        ("Title", game.title.as_str()),
    ] {
        set_child_text(&mut element, field, value);
    }
    set_optional_child_text(&mut element, "Emulator", game.emulator_id.as_deref());
    set_optional_child_text(
        &mut element,
        "DatabaseID",
        game.database_id.map(|value| value.to_string()).as_deref(),
    );
    set_optional_child_text(&mut element, "ManualPath", game.manual_path.as_deref());
    set_optional_child_text(&mut element, "Notes", game.notes.as_deref());
    set_optional_child_text(&mut element, "Developer", game.developer.as_deref());
    set_optional_child_text(&mut element, "Genre", game.genre.as_deref());
    set_optional_child_text(
        &mut element,
        "MaxPlayers",
        game.max_players.map(|value| value.to_string()).as_deref(),
    );
    set_optional_child_text(&mut element, "PlayMode", game.play_mode.as_deref());
    set_optional_child_text(&mut element, "Publisher", game.publisher.as_deref());
    set_optional_child_text(&mut element, "Rating", game.rating.as_deref());
    set_optional_child_text(&mut element, "Region", game.region.as_deref());
    set_optional_child_text(&mut element, "ReleaseDate", game.release_date.as_deref());
    set_optional_child_text(&mut element, "ReleaseType", game.release_type.as_deref());
    set_optional_child_text(&mut element, "Status", game.status.as_deref());
    set_optional_child_text(&mut element, "Version", game.version.as_deref());
    set_optional_child_text(&mut element, "WikipediaURL", game.wikipedia_url.as_deref());
    set_optional_child_text(&mut element, "VideoUrl", game.video_url.as_deref());
    set_optional_child_text(
        &mut element,
        "CommunityStarRating",
        (game.community_star_rating != 0.0)
            .then(|| game.community_star_rating.to_string())
            .as_deref(),
    );
    element
}

fn additional_application_element(application: &AdditionalApplication) -> Element {
    let mut element = Element::new("AdditionalApplication");
    set_child_text(
        &mut element,
        "ApplicationPath",
        &application.application_path,
    );
    set_child_text(
        &mut element,
        "AutoRunAfter",
        &application.auto_run_after.to_string(),
    );
    set_child_text(
        &mut element,
        "AutoRunBefore",
        &application.auto_run_before.to_string(),
    );
    set_optional_child_text(
        &mut element,
        "CommandLine",
        application.command_line.as_deref(),
    );
    set_optional_child_text(
        &mut element,
        "EmulatorId",
        application.emulator_id.as_deref(),
    );
    set_child_text(&mut element, "GameID", &application.game_id);
    set_optional_child_text(&mut element, "GogAppId", application.gog_app_id.as_deref());
    set_child_text(
        &mut element,
        "HasCloudSynced",
        &application.has_cloud_synced.to_string(),
    );
    set_child_text(&mut element, "Id", &application.id);
    let installed = application.installed.map(|value| value.to_string());
    set_optional_child_text(&mut element, "Installed", installed.as_deref());
    set_optional_child_text(
        &mut element,
        "LastPlayed",
        application.last_played.as_deref(),
    );
    set_child_text(&mut element, "Name", &application.name);
    set_optional_child_text(
        &mut element,
        "OriginAppId",
        application.origin_app_id.as_deref(),
    );
    set_optional_child_text(
        &mut element,
        "OriginInstallPath",
        application.origin_install_path.as_deref(),
    );
    set_child_text(
        &mut element,
        "PlayCount",
        &application.play_count.to_string(),
    );
    set_child_text(
        &mut element,
        "PlayTime",
        &application.play_time_seconds.to_string(),
    );
    set_child_text(&mut element, "Priority", &application.priority.to_string());
    set_child_text(&mut element, "SideA", &application.side_a.to_string());
    set_child_text(&mut element, "SideB", &application.side_b.to_string());
    set_child_text(
        &mut element,
        "UseDosBox",
        &application.use_dos_box.to_string(),
    );
    set_child_text(
        &mut element,
        "UseEmulator",
        &application.use_emulator.to_string(),
    );
    set_child_text(
        &mut element,
        "WaitForExit",
        &application.wait_for_exit.to_string(),
    );
    if let Some(disc) = application.disc {
        set_child_text(&mut element, "Disc", &disc.to_string());
    }
    for (field, value) in [
        ("Developer", application.developer.as_deref()),
        ("Publisher", application.publisher.as_deref()),
        ("Region", application.region.as_deref()),
        ("ReleaseDate", application.release_date.as_deref()),
        ("Status", application.status.as_deref()),
        ("Version", application.version.as_deref()),
    ] {
        set_optional_child_text(&mut element, field, value);
    }
    element
}

fn game_save_element(save: &GameSave) -> Element {
    let mut element = Element::new("GameSave");
    set_child_text(&mut element, "GameId", &save.game_id);
    set_optional_child_text(
        &mut element,
        "AdditionalApplicationId",
        save.additional_application_id.as_deref(),
    );
    set_optional_child_text(
        &mut element,
        "EmulatorFileName",
        (!save.emulator_file_name.is_empty()).then_some(save.emulator_file_name.as_str()),
    );
    set_optional_child_text(
        &mut element,
        "EmulatorCore",
        (!save.emulator_core.is_empty()).then_some(save.emulator_core.as_str()),
    );
    set_optional_child_text(&mut element, "Title", save.title.as_deref());
    set_optional_child_text(
        &mut element,
        "SaveGroupName",
        save.save_group_name.as_deref(),
    );
    set_optional_child_text(
        &mut element,
        "DisplayChipText",
        save.display_chip_text.as_deref(),
    );
    set_optional_child_text(&mut element, "SaveGroupId", save.save_group_id.as_deref());
    set_optional_child_text(
        &mut element,
        "MatchLineageId",
        save.match_lineage_id.as_deref(),
    );
    set_optional_child_text(
        &mut element,
        "MigrationFamilyId",
        save.migration_family_id.as_deref(),
    );
    set_child_text(&mut element, "FilePath", &save.file_path);
    set_optional_child_text(
        &mut element,
        "OriginalFileName",
        save.original_file_name.as_deref(),
    );
    set_optional_child_text(
        &mut element,
        "Slot",
        save.slot.map(|value| value.to_string()).as_deref(),
    );
    set_optional_child_text(
        &mut element,
        "ReportedFileSizeBytes",
        save.reported_file_size_bytes
            .map(|value| value.to_string())
            .as_deref(),
    );
    set_optional_child_text(
        &mut element,
        "ReportedLastModifiedUtc",
        save.reported_last_modified_utc.as_deref(),
    );
    set_optional_child_text(&mut element, "Md5", save.md5.as_deref());
    element
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse XML {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: xmltree::ParseError,
    },
    #[error("failed to read library index XML {path}: {source}")]
    ReadIndex {
        path: PathBuf,
        #[source]
        source: quick_xml::DeError,
    },
    #[error("failed to parse embedded {record} record: {source}")]
    ReadEmbeddedRecord {
        record: &'static str,
        #[source]
        source: quick_xml::DeError,
    },
    #[error("expected <LaunchBox> root in {path}, found <{actual}>")]
    InvalidRoot { path: PathBuf, actual: String },
    #[error("no LaunchBox Data directory found under {path}")]
    NoDataDirectory { path: PathBuf },
    #[error("cannot infer a supported LaunchBox auxiliary document kind from {path}")]
    UnsupportedAuxiliaryDocument { path: PathBuf },
    #[error("refusing to write {expected:?} document to {path}, whose name identifies {actual:?}")]
    AuxiliaryDocumentKindMismatch {
        path: PathBuf,
        expected: AuxiliaryDocumentKind,
        actual: AuxiliaryDocumentKind,
    },
    #[error("cannot {operation} through {actual:?}; expected a {expected:?} document")]
    UnsupportedAuxiliaryOperation {
        operation: &'static str,
        expected: AuxiliaryDocumentKind,
        actual: AuxiliaryDocumentKind,
    },
    #[error("platform name already exists: {name}")]
    DuplicatePlatformName { name: String },
    #[error("platform was not found: {name}")]
    PlatformNotFound { name: String },
    #[error("platform name is immutable in the recovered 13.27 contract; expected {expected}, got {actual}")]
    ImmutablePlatformName { expected: String, actual: String },
    #[error("platform folder owner {actual} does not match platform {expected}")]
    PlatformFolderOwnerMismatch { expected: String, actual: String },
    #[error("platform {platform} has more than one folder for media type {media_type}")]
    DuplicatePlatformFolderMediaType {
        platform: String,
        media_type: String,
    },
    #[error("invalid platform folder edit for {platform}: {reason}")]
    InvalidPlatformFolderEdit { platform: String, reason: String },
    #[error("emulator ID already exists: {id}")]
    DuplicateEmulatorId { id: String },
    #[error("emulator was not found: {id}")]
    EmulatorNotFound { id: String },
    #[error("emulator ID is immutable; expected {expected}, got {actual}")]
    ImmutableEmulatorId { expected: String, actual: String },
    #[error("emulator platform owner {actual} does not match emulator {expected}")]
    EmulatorPlatformOwnerMismatch { expected: String, actual: String },
    #[error("emulator {id} contains more than one mapping for platform {platform}")]
    DuplicateEmulatorPlatform { id: String, platform: String },
    #[error("emulator {id} has more than one default mapping for platform {platform}")]
    DuplicateEmulatorPlatformDefault { id: String, platform: String },
    #[error("invalid emulator platform edit for {id}: {reason}")]
    InvalidEmulatorPlatformEdit { id: String, reason: String },
    #[error("platform category name already exists: {name}")]
    DuplicatePlatformCategoryName { name: String },
    #[error("platform category was not found: {name}")]
    PlatformCategoryNotFound { name: String },
    #[error("platform category name is immutable in the recovered 13.27 contract; expected {expected}, got {actual}")]
    ImmutablePlatformCategoryName { expected: String, actual: String },
    #[error("invalid parent placement edit for platform category {category}: {reason}")]
    InvalidPlatformCategoryParentEdit { category: String, reason: String },
    #[error("playlist was not found: {id}")]
    PlaylistNotFound { id: String },
    #[error("playlist ID is immutable; expected {expected}, got {actual}")]
    ImmutablePlaylistId { expected: String, actual: String },
    #[error("playlist unique name is immutable in the recovered 13.27 contract; expected {expected}, got {actual}")]
    ImmutablePlaylistName { expected: String, actual: String },
    #[error("invalid playlist {record} edit: {reason}")]
    InvalidPlaylistRecordEdit {
        record: &'static str,
        reason: String,
    },
    #[error("playlist {id} contains game {game_id} more than once")]
    DuplicatePlaylistGame { id: String, game_id: String },
    #[error("invalid parent placement edit for playlist {id}: {reason}")]
    InvalidPlaylistParentEdit { id: String, reason: String },
    #[error("{path} has no {record} record")]
    MissingDocumentRecord { path: PathBuf, record: &'static str },
    #[error("{path} has more than one {record} record")]
    DuplicateDocumentRecord { path: PathBuf, record: &'static str },
    #[error("{record}.{field} is not a valid {expected}")]
    InvalidRecordField {
        record: &'static str,
        field: &'static str,
        expected: &'static str,
    },
    #[error("setting {field} in {record} unexpectedly contains nested XML")]
    NestedSettingField { record: String, field: String },
    #[error("{record} is missing required {field} element")]
    MissingRecordField {
        record: &'static str,
        field: &'static str,
    },
    #[error(transparent)]
    InvalidDomain(#[from] ValidationError),
    #[error(transparent)]
    InvalidCatalog(#[from] CatalogValidationError),
    #[error("no LaunchBox platform directory found under {path}")]
    NoPlatformDirectory { path: PathBuf },
    #[error("no platform XML documents found in {path}")]
    NoPlatformDocuments { path: PathBuf },
    #[error("game {id} was not found")]
    GameNotFound { id: String },
    #[error("additional application {id} was not found")]
    AdditionalApplicationNotFound { id: String },
    #[error("additional application ID {id} already exists")]
    DuplicateAdditionalApplicationId { id: String },
    #[error("additional application {id} has {count} dependent game-save records")]
    AdditionalApplicationHasReferences { id: String, count: usize },
    #[error("last-played timestamp cannot be empty")]
    EmptyLastPlayedTimestamp,
    #[error("{record} {id} {field} would overflow its persisted integer type")]
    PlayStatisticOverflow {
        record: &'static str,
        id: String,
        field: &'static str,
    },
    #[error("game {id} cannot have an empty title")]
    EmptyGameTitle { id: String },
    #[error("game {id} cannot have an empty application path")]
    EmptyGameApplicationPath { id: String },
    #[error("game ID {id} already exists")]
    DuplicateGameId { id: String },
    #[error("invalid game combine/expand request: {reason}")]
    InvalidGameGrouping { reason: String },
    #[error("game platform {actual} does not match platform document {expected}")]
    GamePlatformMismatch { expected: String, actual: String },
    #[error("invalid {record} edit for game {game_id}: {reason}")]
    InvalidGameRecordEdit {
        record: &'static str,
        game_id: String,
        reason: String,
    },
    #[error("game {id} has {count} dependent records ({summary})")]
    GameHasReferences {
        id: String,
        count: usize,
        summary: String,
    },
    #[error("no {record} record matched{selector}")]
    EditableRecordNotFound { record: String, selector: String },
    #[error("{count} {record} records matched{selector}; expected exactly one")]
    EditableRecordAmbiguous {
        record: String,
        selector: String,
        count: usize,
    },
    #[error("failed to serialize XML: {0}")]
    WriteXml(#[source] xmltree::Error),
    #[error("failed to write {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("atomic save target {path} is not a regular file")]
    AtomicTargetNotFile { path: PathBuf },
    #[error("atomic save source {path} is not a regular file")]
    AtomicSourceNotFile { path: PathBuf },
    #[error("atomic directory target {path} is not a real directory")]
    AtomicTargetNotDirectory { path: PathBuf },
    #[error("atomic directory source {path} is not a real directory")]
    AtomicSourceNotDirectory { path: PathBuf },
    #[error("atomic directory tree contains a symlink or special entry: {path}")]
    AtomicDirectoryUnsafeEntry { path: PathBuf },
    #[error("atomic directory tree contains a non-Unicode path: {path}")]
    AtomicDirectoryNonUnicode { path: PathBuf },
    #[error("atomic directory source {source_path} aliases target {target_path}")]
    AtomicDirectorySourceAliasesTarget {
        source_path: PathBuf,
        target_path: PathBuf,
    },
    #[error("refusing to copy directory {path} because it changed after inspection")]
    AtomicDirectorySourceConflict {
        path: PathBuf,
        expected: Box<DirectoryRevision>,
        actual: Box<DirectoryRevision>,
    },
    #[error("refusing to overwrite directory {path} because it changed after inspection")]
    AtomicDirectoryTargetConflict {
        path: PathBuf,
        expected: Box<DirectoryRevision>,
        actual: Box<DirectoryRevision>,
    },
    #[error(
        "directory replacement of {path} failed, but the original tree was restored: {source}"
    )]
    AtomicDirectoryReplaceRolledBack {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(
        "directory replacement of {path} failed and requires recovery from {backup}; staged tree remains at {staged}; replacement error: {replace_error}; recovery error: {recovery_error}"
    )]
    AtomicDirectoryReplaceRecoveryRequired {
        path: Box<PathBuf>,
        backup: Box<PathBuf>,
        staged: Box<PathBuf>,
        replace_error: String,
        recovery_error: String,
    },
    #[error("atomic save source and target resolve to the same file: {path}")]
    AtomicSourceEqualsTarget { path: PathBuf },
    #[error("cannot delete an empty regular-file set")]
    AtomicDeleteSetEmpty,
    #[error("regular-file delete set contains {path} more than once")]
    AtomicDeleteSetDuplicate { path: PathBuf },
    #[error("refusing to copy {path} because it changed after inspection")]
    AtomicSourceConflict {
        path: PathBuf,
        expected: FileRevision,
        actual: FileRevision,
    },
    #[error("refusing to overwrite {path} because it changed since this document was loaded")]
    WriteConflict {
        path: PathBuf,
        expected: FileRevision,
        actual: FileRevision,
    },
    #[error("serialized replacement for {path} failed validation: {source}")]
    AtomicValidation {
        path: PathBuf,
        #[source]
        source: Box<StorageError>,
    },
    #[error("failed to atomically replace {path}; original backup remains at {backup}: {source}")]
    AtomicReplace {
        path: PathBuf,
        backup: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(
        "replaced {path}, but could not sync its directory; backup remains at {backup}: {source}"
    )]
    AtomicDirectorySync {
        path: PathBuf,
        backup: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(
        "failed to delete {path}, but every attempted deletion was rolled back; recovery copy remains at {backup}: {source}"
    )]
    AtomicDeleteSetRolledBack {
        path: PathBuf,
        backup: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(
        "failed to delete {path} and could not fully roll the set back; recovery copy remains at {backup}; delete error: {delete_error}; recovery error: {recovery_error}"
    )]
    AtomicDeleteSetRecoveryRequired {
        path: PathBuf,
        backup: PathBuf,
        delete_error: String,
        recovery_error: String,
    },
    #[error("could not allocate a unique {kind} sibling for {path}")]
    UniqueSiblingExhausted { path: PathBuf, kind: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str =
        include_str!("../../../fixtures/launchbox/Data/Platforms/Fixture Console.xml");
    const AUXILIARY_FIXTURES: &[(AuxiliaryDocumentKind, &str, &str)] = &[
        (
            AuxiliaryDocumentKind::Playlist,
            "Playlists/Fixture Playlist.xml",
            include_str!("../../../fixtures/launchbox/Data/Playlists/Fixture Playlist.xml"),
        ),
        (
            AuxiliaryDocumentKind::Emulators,
            "Emulators.xml",
            include_str!("../../../fixtures/launchbox/Data/Emulators.xml"),
        ),
        (
            AuxiliaryDocumentKind::Platforms,
            "Platforms.xml",
            include_str!("../../../fixtures/launchbox/Data/Platforms.xml"),
        ),
        (
            AuxiliaryDocumentKind::Parents,
            "Parents.xml",
            include_str!("../../../fixtures/launchbox/Data/Parents.xml"),
        ),
        (
            AuxiliaryDocumentKind::GameControllers,
            "GameControllers.xml",
            include_str!("../../../fixtures/launchbox/Data/GameControllers.xml"),
        ),
        (
            AuxiliaryDocumentKind::InputBindings,
            "InputBindings.xml",
            include_str!("../../../fixtures/launchbox/Data/InputBindings.xml"),
        ),
        (
            AuxiliaryDocumentKind::ImportBlacklist,
            "ImportBlacklist.xml",
            include_str!("../../../fixtures/launchbox/Data/ImportBlacklist.xml"),
        ),
        (
            AuxiliaryDocumentKind::ListCache,
            "ListCache.xml",
            include_str!("../../../fixtures/launchbox/Data/ListCache.xml"),
        ),
        (
            AuxiliaryDocumentKind::Settings,
            "Settings.xml",
            include_str!("../../../fixtures/launchbox/Data/Settings.xml"),
        ),
        (
            AuxiliaryDocumentKind::BigBoxSettings,
            "BigBoxSettings.xml",
            include_str!("../../../fixtures/launchbox/Data/BigBoxSettings.xml"),
        ),
    ];

    fn full_game_save_fixture() -> String {
        FIXTURE
            .replace(
                "    <EmulatorCore>fixture-core</EmulatorCore>",
                "    <AdditionalApplicationId>fixture-adventure-manual</AdditionalApplicationId>\n    <EmulatorCore>fixture-core</EmulatorCore>",
            )
            .replace(
                "    <FilePath>Saves\\Fixture Adventure\\slot1.sav</FilePath>",
                "    <SaveGroupName>Final Puzzle</SaveGroupName>\n    <DisplayChipText>Slot 1</DisplayChipText>\n    <SaveGroupId>save-group-alpha</SaveGroupId>\n    <MatchLineageId>lineage-alpha</MatchLineageId>\n    <MigrationFamilyId>migration-alpha</MigrationFamilyId>\n    <FilePath>Saves\\Fixture Adventure\\slot1.sav</FilePath>\n    <OriginalFileName>slot1-original.sav</OriginalFileName>",
            )
            .replace(
                "    <Title>Before the Final Puzzle</Title>",
                "    <Title>Before the Final Puzzle</Title>\n    <ReportedFileSizeBytes>9223372036854770000</ReportedFileSizeBytes>\n    <ReportedLastModifiedUtc>2026-07-22T01:02:03.4567890Z</ReportedLastModifiedUtc>\n    <Md5>0123456789abcdef0123456789abcdef</Md5>\n    <FutureGameSaveField>keep-save-data</FutureGameSaveField>",
            )
    }

    fn grouped_game_save_fixture() -> String {
        full_game_save_fixture().replace(
            "  <FutureRootElement>preserve-me</FutureRootElement>",
            "  <GameSave>\n    <EmulatorCore>fixture-core</EmulatorCore>\n    <EmulatorFileName>fixture-emulator</EmulatorFileName>\n    <Title>Earlier Checkpoint</Title>\n    <SaveGroupName>Earlier Run</SaveGroupName>\n    <SaveGroupId>save-group-beta</SaveGroupId>\n    <FilePath>C:\\Users\\Ben\\RetroArch\\saves\\fixture-backup.srm</FilePath>\n    <GameId>fixture-adventure</GameId>\n    <OriginalFileName>fixture.srm</OriginalFileName>\n    <ReportedFileSizeBytes>4096</ReportedFileSizeBytes>\n    <Md5>fedcba9876543210fedcba9876543210</Md5>\n    <FutureSecondSaveField>keep-second-save-data</FutureSecondSaveField>\n  </GameSave>\n  <FutureRootElement>preserve-me</FutureRootElement>",
        )
    }

    #[test]
    fn loads_real_launchbox_shaped_platform_document() {
        let document = PlatformDocument::from_reader("Fixture Console.xml", FIXTURE.as_bytes())
            .expect("parse fixture");
        assert_eq!(document.library().name, "Fixture Console");
        assert_eq!(document.library().games.len(), 3);
        assert_eq!(document.library().games[0].id, "fixture-adventure");
        assert!(document.library().games[0].favorite);
        assert_eq!(document.library().additional_applications.len(), 1);
        assert_eq!(
            document.library().additional_applications[0].id,
            "fixture-adventure-manual"
        );
        assert_eq!(document.library().alternate_names.len(), 1);
        assert_eq!(document.library().custom_fields.len(), 1);
        assert_eq!(document.library().controller_support.len(), 1);
        assert_eq!(document.library().game_saves.len(), 1);
    }

    #[test]
    fn combine_and_expand_games_preserve_versions_references_and_unknown_xml() {
        let fixture = FIXTURE
            .replace(
                "    <Title>Fixture Racer</Title>",
                "    <Region>Europe</Region>\n    <Version>Rev 2</Version>\n    <Title>Fixture Racer</Title>\n    <FutureRacerField>keep-racer-data</FutureRacerField>",
            )
            .replace(
                "    <Notes>Searchable puzzle fixture.</Notes>",
                "    <CloneOf>fixture-racer</CloneOf>\n    <Notes>Searchable puzzle fixture.</Notes>",
            )
            .replace(
                "  <FutureRootElement>preserve-me</FutureRootElement>",
                "  <GameSave>\n    <FilePath>Saves\\Fixture Racer\\career.sav</FilePath>\n    <GameId>fixture-racer</GameId>\n    <Title>Racer Career</Title>\n    <FutureRacerSaveField>keep-racer-save-data</FutureRacerSaveField>\n  </GameSave>\n  <FutureRootElement>preserve-me</FutureRootElement>",
            );
        let mut document =
            PlatformDocument::from_reader("Fixture Console.xml", fixture.as_bytes()).unwrap();
        let combined = document
            .combine_games(
                "fixture-adventure",
                &["fixture-adventure".into(), "fixture-racer".into()],
                &["version-root".into(), "version-racer".into()],
            )
            .unwrap();

        assert_eq!(combined.root_game.id, "fixture-adventure");
        assert_eq!(
            combined
                .removed_games
                .iter()
                .map(|game| game.id.as_str())
                .collect::<Vec<_>>(),
            ["fixture-racer"]
        );
        assert_eq!(combined.created_version_applications.len(), 2);
        assert_eq!(combined.retargeted_game_records, 2);
        assert_eq!(combined.retargeted_clone_relationships, 1);
        assert_eq!(document.library().games.len(), 2);
        assert!(document
            .library()
            .games
            .iter()
            .all(|game| game.id != "fixture-racer"));
        assert_eq!(
            document
                .library()
                .games
                .iter()
                .find(|game| game.id == "fixture-puzzle")
                .unwrap()
                .clone_of
                .as_deref(),
            Some("fixture-adventure")
        );
        assert_eq!(document.library().additional_applications.len(), 3);
        assert!(document
            .library()
            .controller_support
            .iter()
            .all(|support| support.game_id == "fixture-adventure"));
        let racer_save = document
            .library()
            .game_saves
            .iter()
            .find(|save| save.title.as_deref() == Some("Racer Career"))
            .unwrap();
        assert_eq!(racer_save.game_id, "fixture-adventure");
        assert_eq!(
            racer_save.additional_application_id.as_deref(),
            Some("version-racer")
        );

        let combined_xml = String::from_utf8(document.to_xml_bytes().unwrap()).unwrap();
        assert!(combined_xml
            .contains("<FutureRacerSaveField>keep-racer-save-data</FutureRacerSaveField>"));
        assert!(!combined_xml.contains("<FutureRacerField>keep-racer-data</FutureRacerField>"));

        let expanded = document
            .expand_game_versions("fixture-adventure", &["expanded-racer".into()])
            .unwrap();
        assert_eq!(expanded.created_games.len(), 1);
        assert_eq!(expanded.removed_version_applications.len(), 2);
        assert_eq!(expanded.reassigned_game_saves, 1);
        let racer = document
            .library()
            .games
            .iter()
            .find(|game| game.id == "expanded-racer")
            .unwrap();
        assert_eq!(racer.title, "Fixture Adventure");
        assert_eq!(racer.application_path, r"Games\Fixture Racer\racer.rom");
        assert_eq!(racer.region.as_deref(), Some("Europe"));
        assert_eq!(racer.version.as_deref(), Some("Rev 2"));
        assert_eq!(racer.play_count, 8);
        assert_eq!(
            document
                .library()
                .game_saves
                .iter()
                .find(|save| save.title.as_deref() == Some("Racer Career"))
                .unwrap()
                .game_id,
            "expanded-racer"
        );
        assert!(document
            .library()
            .game_saves
            .iter()
            .find(|save| save.title.as_deref() == Some("Racer Career"))
            .unwrap()
            .additional_application_id
            .is_none());
        assert_eq!(document.library().additional_applications.len(), 1);
        assert_eq!(
            document.library().additional_applications[0].id,
            "fixture-adventure-manual"
        );
        let expanded_xml = String::from_utf8(document.to_xml_bytes().unwrap()).unwrap();
        assert_eq!(
            expanded_xml
                .matches("<TestOnlyUnknownGameElement>keep-this-too</TestOnlyUnknownGameElement>")
                .count(),
            2
        );
        assert!(expanded_xml
            .contains("<FutureRacerSaveField>keep-racer-save-data</FutureRacerSaveField>"));
    }

    #[test]
    fn combined_game_reference_remap_updates_and_deduplicates_auxiliary_documents() {
        let root_game = Game {
            id: "fixture-adventure".into(),
            title: "Fixture Adventure".into(),
            platform: "Fixture Console".into(),
            application_path: "Games/adventure.rom".into(),
            ..Game::default()
        };
        let removed = vec!["fixture-racer".to_string()];

        let playlist_fixture =
            include_str!("../../../fixtures/launchbox/Data/Playlists/Fixture Playlist.xml")
                .replace(
                    "    <Notes>A deterministic playlist fixture.</Notes>",
                    "    <Notes>A deterministic playlist fixture.</Notes>\n    <LastGameId>fixture-racer</LastGameId>",
                )
                .replace(
                    "</LaunchBox>",
                    "  <PlaylistGame>\n    <GameId>fixture-racer</GameId>\n    <GameTitle>Fixture Racer</GameTitle>\n    <GamePlatform>Fixture Console</GamePlatform>\n    <FuturePlaylistGameField>keep-playlist-data</FuturePlaylistGameField>\n  </PlaylistGame>\n</LaunchBox>",
                );
        let mut playlist = AuxiliaryDocument::from_reader(
            AuxiliaryDocumentKind::Playlist,
            "Fixture Playlist.xml",
            playlist_fixture.as_bytes(),
        )
        .unwrap();
        let playlist_report = playlist
            .remap_combined_game_references(&removed, &root_game)
            .unwrap();
        assert_eq!(playlist_report.navigation_references, 1);
        assert_eq!(playlist_report.playlist_rows_retargeted, 0);
        assert_eq!(playlist_report.playlist_rows_deduplicated, 1);
        let reparsed = playlist.playlist_document().unwrap();
        assert_eq!(
            reparsed.playlist.metadata.last_game_id.as_deref(),
            Some("fixture-adventure")
        );
        assert_eq!(reparsed.games.len(), 1);
        assert_eq!(reparsed.games[0].game_id, "fixture-adventure");

        let platforms_fixture = include_str!("../../../fixtures/launchbox/Data/Platforms.xml")
            .replace(
                "    <Name>Fixture Console</Name>",
                "    <Name>Fixture Console</Name>\n    <LastGameId>fixture-racer</LastGameId>",
            );
        let mut platforms = AuxiliaryDocument::from_reader(
            AuxiliaryDocumentKind::Platforms,
            "Platforms.xml",
            platforms_fixture.as_bytes(),
        )
        .unwrap();
        let platforms_report = platforms
            .remap_combined_game_references(&removed, &root_game)
            .unwrap();
        assert_eq!(platforms_report.navigation_references, 1);
        assert!(String::from_utf8(platforms.to_xml_bytes().unwrap())
            .unwrap()
            .contains("<LastGameId>fixture-adventure</LastGameId>"));

        let blacklist_fixture =
            include_str!("../../../fixtures/launchbox/Data/ImportBlacklist.xml")
                .replace("fixture-ignored-game", "fixture-racer");
        let mut blacklist = AuxiliaryDocument::from_reader(
            AuxiliaryDocumentKind::ImportBlacklist,
            "ImportBlacklist.xml",
            blacklist_fixture.as_bytes(),
        )
        .unwrap();
        let blacklist_report = blacklist
            .remap_combined_game_references(&removed, &root_game)
            .unwrap();
        assert_eq!(blacklist_report.blacklist_rows_removed, 1);
        assert_eq!(blacklist.record_count("IgnoredGameId"), 0);
    }

    #[test]
    fn both_platform_readers_cover_the_full_1327_game_save_contract() {
        let fixture = full_game_save_fixture();
        let document =
            PlatformDocument::from_reader("Fixture Console.xml", fixture.as_bytes()).unwrap();
        let save = &document.library().game_saves[0];
        assert_eq!(
            save.additional_application_id.as_deref(),
            Some("fixture-adventure-manual")
        );
        assert_eq!(save.save_group_name.as_deref(), Some("Final Puzzle"));
        assert_eq!(save.display_chip_text.as_deref(), Some("Slot 1"));
        assert_eq!(save.save_group_id.as_deref(), Some("save-group-alpha"));
        assert_eq!(save.match_lineage_id.as_deref(), Some("lineage-alpha"));
        assert_eq!(save.migration_family_id.as_deref(), Some("migration-alpha"));
        assert_eq!(
            save.original_file_name.as_deref(),
            Some("slot1-original.sav")
        );
        assert_eq!(
            save.reported_file_size_bytes,
            Some(9_223_372_036_854_770_000)
        );
        assert_eq!(
            save.reported_last_modified_utc.as_deref(),
            Some("2026-07-22T01:02:03.4567890Z")
        );
        assert_eq!(
            save.md5.as_deref(),
            Some("0123456789abcdef0123456789abcdef")
        );

        let root = Element::parse(fixture.as_bytes()).unwrap();
        let game_save = elements_named(&root, "GameSave").next().unwrap();
        let mut fixture_fields = game_save
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .map(|child| child.name.as_str())
            .filter(|name| *name != "FutureGameSaveField")
            .collect::<Vec<_>>();
        fixture_fields.sort_unstable();
        let mut modeled_fields = GAME_SAVE_XML_FIELDS.to_vec();
        modeled_fields.sort_unstable();
        assert_eq!(fixture_fields, modeled_fields);

        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("Fixture Console.xml");
        fs::write(&path, fixture).unwrap();
        let index = LibraryIndex::load(&path).unwrap();
        assert_eq!(index.game_saves().next(), Some(save));
    }

    #[test]
    fn appends_a_full_game_save_record_without_rewriting_existing_xml() {
        let mut document =
            PlatformDocument::from_reader("Fixture Console.xml", FIXTURE.as_bytes()).unwrap();
        let original = document.library().game_saves[0].clone();
        let backup = GameSave {
            game_id: "fixture-adventure".into(),
            additional_application_id: Some("fixture-adventure-manual".into()),
            emulator_core: "fixture-core".into(),
            emulator_file_name: "fixture-emulator".into(),
            title: Some("Manual Backup".into()),
            save_group_name: Some("Final Run".into()),
            display_chip_text: Some("Slot 2".into()),
            save_group_id: Some("save-group-alpha".into()),
            match_lineage_id: Some("lineage-alpha".into()),
            migration_family_id: Some("migration-alpha".into()),
            file_path: r"Saves\Fixture Console\fixture-adventure.sav".into(),
            original_file_name: Some("fixture-adventure.sav".into()),
            slot: Some(2),
            reported_file_size_bytes: Some(8192),
            reported_last_modified_utc: Some("2026-07-23T01:02:03.4567890Z".into()),
            md5: Some("0123456789ABCDEF0123456789ABCDEF".into()),
        };

        let saves = document.add_game_save(backup.clone()).unwrap();
        assert_eq!(saves, vec![original, backup.clone()]);
        let xml = String::from_utf8(document.to_xml_bytes().unwrap()).unwrap();
        assert!(xml.contains("<FutureRootElement>preserve-me</FutureRootElement>"));
        assert!(xml.contains(r"<FilePath>Saves\Fixture Console\fixture-adventure.sav</FilePath>"));

        let reparsed =
            PlatformDocument::from_reader("Fixture Console.xml", xml.as_bytes()).unwrap();
        assert_eq!(reparsed.library().game_saves.last(), Some(&backup));
    }

    #[test]
    fn removes_one_source_indexed_game_save_without_touching_peer_xml() {
        let fixture = grouped_game_save_fixture();
        let mut document =
            PlatformDocument::from_reader("Fixture Console.xml", fixture.as_bytes()).unwrap();
        let retained = document.library().game_saves[0].clone();

        let saves = document.remove_game_save("fixture-adventure", 1).unwrap();
        assert_eq!(saves, vec![retained]);
        let xml = String::from_utf8(document.to_xml_bytes().unwrap()).unwrap();
        assert!(xml.contains("<FutureGameSaveField>keep-save-data</FutureGameSaveField>"));
        assert!(
            !xml.contains("<FutureSecondSaveField>keep-second-save-data</FutureSecondSaveField>")
        );
        assert!(xml.contains("<FutureRootElement>preserve-me</FutureRootElement>"));

        let before_invalid = document.to_xml_bytes().unwrap();
        assert!(matches!(
            document.remove_game_save("fixture-adventure", 1),
            Err(StorageError::InvalidGameRecordEdit {
                record: "game save",
                ..
            })
        ));
        assert_eq!(document.to_xml_bytes().unwrap(), before_invalid);
    }

    #[test]
    fn rejects_a_game_save_for_an_unknown_additional_application_without_mutation() {
        let mut document =
            PlatformDocument::from_reader("Fixture Console.xml", FIXTURE.as_bytes()).unwrap();
        let before = document.to_xml_bytes().unwrap();
        let save = GameSave {
            game_id: "fixture-adventure".into(),
            additional_application_id: Some("missing-app".into()),
            file_path: r"Saves\Fixture Console\fixture.sav".into(),
            ..GameSave::default()
        };
        assert!(matches!(
            document.add_game_save(save),
            Err(StorageError::AdditionalApplicationNotFound {
                id
            }) if id == "missing-app"
        ));
        assert_eq!(document.to_xml_bytes().unwrap(), before);
    }

    #[test]
    fn game_save_metadata_edit_is_source_indexed_and_lossless() {
        let fixture = grouped_game_save_fixture();
        let mut document =
            PlatformDocument::from_reader("Fixture Console.xml", fixture.as_bytes()).unwrap();
        let originals = document.library().game_saves.clone();
        let edits = vec![
            IndexedGameSaveMetadataEdit {
                source_index: 0,
                metadata: GameSaveMetadataEdit {
                    title: Some("Renamed Version".into()),
                    save_group_name: Some("Combined Run".into()),
                    save_group_id: Some("save-group-alpha".into()),
                },
            },
            IndexedGameSaveMetadataEdit {
                source_index: 1,
                metadata: GameSaveMetadataEdit {
                    title: originals[1].title.clone(),
                    save_group_name: Some("Combined Run".into()),
                    save_group_id: Some("save-group-alpha".into()),
                },
            },
        ];
        let updated = document
            .set_game_save_metadata("fixture-adventure", edits.clone())
            .unwrap();
        assert_eq!(updated.len(), 2);
        assert_eq!(updated[0], edits[0].metadata.apply_to(&originals[0]));
        assert_eq!(updated[1], edits[1].metadata.apply_to(&originals[1]));
        assert_eq!(updated[0].file_path, r"Saves\Fixture Adventure\slot1.sav");
        assert_eq!(
            updated[1].file_path,
            r"C:\Users\Ben\RetroArch\saves\fixture-backup.srm"
        );

        let xml = String::from_utf8(document.to_xml_bytes().unwrap()).unwrap();
        assert!(xml.contains("<FutureGameSaveField>keep-save-data</FutureGameSaveField>"));
        assert!(
            xml.contains("<FutureSecondSaveField>keep-second-save-data</FutureSecondSaveField>")
        );
        assert!(xml.contains("<FutureRootElement>preserve-me</FutureRootElement>"));
        let reparsed =
            PlatformDocument::from_reader("Fixture Console.xml", xml.as_bytes()).unwrap();
        assert_eq!(reparsed.library().game_saves, updated);
    }

    #[test]
    fn invalid_game_save_metadata_edits_do_not_mutate_the_document() {
        let fixture = grouped_game_save_fixture();
        let mut document =
            PlatformDocument::from_reader("Fixture Console.xml", fixture.as_bytes()).unwrap();
        let before = document.to_xml_bytes().unwrap();
        let metadata = GameSaveMetadataEdit {
            title: Some("Changed".into()),
            save_group_name: Some("Group".into()),
            save_group_id: Some("group".into()),
        };
        let duplicate = vec![
            IndexedGameSaveMetadataEdit {
                source_index: 0,
                metadata: metadata.clone(),
            },
            IndexedGameSaveMetadataEdit {
                source_index: 0,
                metadata: metadata.clone(),
            },
        ];
        assert!(matches!(
            document.set_game_save_metadata("fixture-adventure", duplicate),
            Err(StorageError::InvalidGameRecordEdit {
                record: "game save",
                ..
            })
        ));
        assert_eq!(document.to_xml_bytes().unwrap(), before);

        assert!(matches!(
            document.set_game_save_metadata(
                "fixture-adventure",
                vec![IndexedGameSaveMetadataEdit {
                    source_index: 2,
                    metadata,
                }],
            ),
            Err(StorageError::InvalidGameRecordEdit {
                record: "game save",
                ..
            })
        ));
        assert_eq!(document.to_xml_bytes().unwrap(), before);
    }

    #[test]
    fn fixture_and_model_cover_every_observed_game_field() {
        let root = Element::parse(FIXTURE.as_bytes()).expect("parse fixture XML");
        let game = elements_named(&root, "Game").next().expect("fixture game");
        let mut fixture_fields = game
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .map(|child| child.name.as_str())
            .filter(|name| *name != "TestOnlyUnknownGameElement")
            .collect::<Vec<_>>();
        fixture_fields.sort_unstable();

        let mut modeled_fields = GAME_XML_FIELDS.to_vec();
        modeled_fields.sort_unstable();
        assert_eq!(fixture_fields, modeled_fields);

        let document = PlatformDocument::from_reader("Fixture Console.xml", FIXTURE.as_bytes())
            .expect("parse every modeled field");
        let game = &document.library().games[0];
        assert_eq!(game.database_id, Some(1234));
        assert_eq!(game.community_star_rating, 4.25);
        assert_eq!(game.star_rating_float, 4.5);
        assert_eq!(game.max_players, Some(4));
        assert_eq!(game.has_gog_achievements, Some(true));
        assert_eq!(game.retro_achievements_id, Some(5678));
        assert_eq!(
            game.android_video_path.as_deref(),
            Some(r"Android\Videos\fixture-adventure.mp4")
        );
    }

    #[test]
    fn modeled_game_fields_match_the_frozen_real_install_schema() {
        let schema: serde_json::Value =
            serde_json::from_str(include_str!("../../../analysis/real-install-schema.json"))
                .expect("parse value-free real-install schema");
        let mut observed = schema["document_groups"]["Platforms/*.xml"]["record_fields"]["Game"]
            .as_array()
            .expect("Game field inventory")
            .iter()
            .map(|field| field.as_str().expect("field name"))
            .collect::<Vec<_>>();
        observed.sort_unstable();

        let mut modeled = GAME_XML_FIELDS.to_vec();
        modeled.sort_unstable();
        assert_eq!(modeled, observed);
    }

    #[test]
    fn indexed_reader_matches_lossless_reader_for_modeled_fields() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("Fixture Console.xml");
        fs::write(&path, FIXTURE).expect("write fixture");
        let indexed = LibraryIndex::load(&path).expect("load indexed fixture");
        let lossless = PlatformDocument::load(&path).expect("load lossless fixture");
        assert_eq!(
            indexed.platforms(),
            std::slice::from_ref(lossless.library())
        );
    }

    #[test]
    fn dosbox_mounts_are_typed_indexed_losslessly_and_block_game_deletion() {
        const DOSBOX_XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<LaunchBox>
  <Game>
    <ID>dos-game</ID>
    <Title>DOS Fixture</Title>
    <Platform>MS-DOS</Platform>
    <ApplicationPath>Games\DOS Fixture\PLAY.BAT</ApplicationPath>
    <UseDosBox>true</UseDosBox>
  </Game>
  <Mount>
    <GameID>dos-game</GameID>
    <DriveLetter>D</DriveLetter>
    <Filesystem>ISO</Filesystem>
    <MountType>Folder</MountType>
    <Path>Media\CD Files</Path>
    <Type>CD-ROM/ISO</Type>
    <FutureMountField>preserve-this</FutureMountField>
  </Mount>
  <Mount>
    <GameID>dos-game</GameID>
    <DriveLetter>A</DriveLetter>
    <Filesystem>FAT</Filesystem>
    <MountType>File</MountType>
    <Path>Media/Disk One.img</Path>
    <Type>Floppy</Type>
  </Mount>
</LaunchBox>"#;
        let document = PlatformDocument::from_reader("MS-DOS.xml", DOSBOX_XML.as_bytes())
            .expect("parse DOSBox mounts");
        assert_eq!(
            document.library().mounts,
            [
                Mount {
                    game_id: "dos-game".into(),
                    drive_letter: 'D',
                    filesystem: "ISO".into(),
                    mount_type: "Folder".into(),
                    path: r"Media\CD Files".into(),
                    media_type: "CD-ROM/ISO".into(),
                },
                Mount {
                    game_id: "dos-game".into(),
                    drive_letter: 'A',
                    filesystem: "FAT".into(),
                    mount_type: "File".into(),
                    path: "Media/Disk One.img".into(),
                    media_type: "Floppy".into(),
                },
            ]
        );
        let serialized = document.to_xml_bytes().expect("serialize DOSBox fixture");
        let root = Element::parse(serialized.as_slice()).expect("reparse DOSBox fixture");
        assert_eq!(
            elements_named(&root, "Mount")
                .next()
                .and_then(|mount| child_text(mount, "FutureMountField"))
                .as_deref(),
            Some("preserve-this")
        );

        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("MS-DOS.xml");
        fs::write(&path, DOSBOX_XML).expect("write DOSBox fixture");
        let indexed = LibraryIndex::load(&path).expect("index DOSBox mounts");
        assert_eq!(indexed.mounts().count(), 2);
        assert_eq!(
            find_game_references(&path, "dos-game")
                .expect("scan mount references")
                .iter()
                .map(|reference| reference.kind)
                .collect::<Vec<_>>(),
            [GameReferenceKind::Mount, GameReferenceKind::Mount]
        );
        let mut document = PlatformDocument::load(&path).expect("load DOSBox fixture");
        assert!(matches!(
            document.remove_game("dos-game"),
            Err(StorageError::GameHasReferences { count: 2, .. })
        ));
    }

    #[test]
    fn edit_preserves_unknown_game_and_root_elements() {
        let mut document = PlatformDocument::from_reader("Fixture Console.xml", FIXTURE.as_bytes())
            .expect("parse fixture");
        document
            .set_game_title("fixture-adventure", "Renamed Adventure")
            .expect("rename game");
        let bytes = document.to_xml_bytes().expect("serialize fixture");
        let reparsed = Element::parse(bytes.as_slice()).expect("reparse output");
        assert_eq!(
            reparsed
                .get_child("FutureRootElement")
                .and_then(Element::get_text)
                .as_deref(),
            Some("preserve-me")
        );
        let game = reparsed
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .find(|element| {
                element.name == "Game"
                    && child_text(element, "ID").as_deref() == Some("fixture-adventure")
            })
            .expect("edited game");
        assert_eq!(
            child_text(game, "Title").as_deref(),
            Some("Renamed Adventure")
        );
        assert_eq!(
            child_text(game, "TestOnlyUnknownGameElement").as_deref(),
            Some("keep-this-too")
        );
        let additional_application = reparsed
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .find(|element| element.name == "AdditionalApplication")
            .expect("additional application");
        assert_eq!(
            child_text(additional_application, "FutureAdditionalApplicationElement").as_deref(),
            Some("keep-additional-app-data")
        );
    }

    #[test]
    fn add_and_remove_game_are_lossless_and_validate_identity() {
        let mut document = PlatformDocument::from_reader("Fixture Console.xml", FIXTURE.as_bytes())
            .expect("parse fixture");
        let added = NewGame {
            id: "fixture-added".into(),
            title: "Fixture Added".into(),
            platform: "Fixture Console".into(),
            application_path: r"Games\Fixture Added\added.rom".into(),
            emulator_id: None,
            metadata: NewGameMetadata {
                database_id: Some(4242),
                manual_path: Some(r"Manuals\Fixture Console\Fixture Added.pdf".into()),
                notes: Some("Imported overview".into()),
                developer: Some("Fixture Forge".into()),
                genre: Some("Strategy".into()),
                max_players: Some(2),
                play_mode: Some("Cooperative; Multiplayer".into()),
                publisher: Some("Fixture Press".into()),
                rating: Some("E10+".into()),
                region: Some("North America".into()),
                release_date: Some("2002-03-04".into()),
                release_type: Some("Released".into()),
                status: Some("Imported ROM".into()),
                version: Some("(USA)".into()),
                wikipedia_url: Some("https://example.org/fixture".into()),
                video_url: Some("https://video.example/fixture".into()),
                community_star_rating: Some(4.75),
            },
        };
        let game = document.add_game(added.clone()).expect("add game");
        assert_eq!(game.id, "fixture-added");
        assert!(matches!(
            document.add_game(added),
            Err(StorageError::DuplicateGameId { .. })
        ));

        let bytes = document.to_xml_bytes().expect("serialize added game");
        let reparsed = PlatformDocument::from_reader("Fixture Console.xml", bytes.as_slice())
            .expect("reparse added game");
        assert_eq!(reparsed.library().games.len(), 4);
        let reparsed_game = reparsed
            .library()
            .games
            .iter()
            .find(|game| game.id == "fixture-added")
            .expect("reparsed added game");
        assert_eq!(
            reparsed_game.application_path,
            r"Games\Fixture Added\added.rom"
        );
        assert_eq!(reparsed_game.database_id, Some(4242));
        assert_eq!(
            reparsed_game.manual_path.as_deref(),
            Some(r"Manuals\Fixture Console\Fixture Added.pdf")
        );
        assert_eq!(reparsed_game.notes.as_deref(), Some("Imported overview"));
        assert_eq!(reparsed_game.developer.as_deref(), Some("Fixture Forge"));
        assert_eq!(reparsed_game.genre.as_deref(), Some("Strategy"));
        assert_eq!(reparsed_game.max_players, Some(2));
        assert_eq!(
            reparsed_game.play_mode.as_deref(),
            Some("Cooperative; Multiplayer")
        );
        assert_eq!(reparsed_game.publisher.as_deref(), Some("Fixture Press"));
        assert_eq!(reparsed_game.rating.as_deref(), Some("E10+"));
        assert_eq!(reparsed_game.release_date.as_deref(), Some("2002-03-04"));
        assert_eq!(reparsed_game.release_type.as_deref(), Some("Released"));
        assert_eq!(
            reparsed_game.wikipedia_url.as_deref(),
            Some("https://example.org/fixture")
        );
        assert_eq!(
            reparsed_game.video_url.as_deref(),
            Some("https://video.example/fixture")
        );
        assert_eq!(reparsed_game.community_star_rating, 4.75);

        let removed = document.remove_game("fixture-added").expect("remove game");
        assert_eq!(removed.id, "fixture-added");
        let final_bytes = document.to_xml_bytes().expect("serialize removal");
        let root = Element::parse(final_bytes.as_slice()).expect("parse removal");
        assert!(elements_named(&root, "Game")
            .all(|element| child_text(element, "ID").as_deref() != Some("fixture-added")));
        assert_eq!(
            root.get_child("FutureRootElement")
                .and_then(Element::get_text)
                .as_deref(),
            Some("preserve-me")
        );
    }

    #[test]
    fn adding_an_additional_application_is_typed_and_lossless() {
        let mut document = PlatformDocument::from_reader("Fixture Console.xml", FIXTURE.as_bytes())
            .expect("parse fixture");
        let application = AdditionalApplication {
            id: "fixture-disc-two".into(),
            game_id: "fixture-adventure".into(),
            name: "Play Disc 2".into(),
            application_path: r"Games\Fixture Adventure\disc-2.rom".into(),
            use_emulator: true,
            emulator_id: Some("fixture-emulator".into()),
            priority: 2,
            disc: Some(2),
            ..AdditionalApplication::default()
        };
        assert_eq!(
            document
                .add_additional_application(application.clone())
                .expect("add additional application"),
            application
        );
        assert!(matches!(
            document.add_additional_application(application),
            Err(StorageError::DuplicateAdditionalApplicationId { .. })
        ));

        let bytes = document.to_xml_bytes().expect("serialize");
        let reparsed = PlatformDocument::from_reader("Fixture Console.xml", bytes.as_slice())
            .expect("reparse");
        let added = reparsed
            .library()
            .additional_applications
            .iter()
            .find(|application| application.id == "fixture-disc-two")
            .expect("find added application");
        assert_eq!(added.disc, Some(2));
        assert_eq!(added.priority, 2);
        assert_eq!(added.emulator_id.as_deref(), Some("fixture-emulator"));
        assert_eq!(
            String::from_utf8_lossy(&bytes)
                .matches("<FutureAdditionalApplicationElement>")
                .count(),
            1
        );
    }

    #[test]
    fn additional_application_editor_preserves_identity_provider_and_unknown_xml() {
        let mut document = PlatformDocument::from_reader("Fixture Console.xml", FIXTURE.as_bytes())
            .expect("parse fixture");
        let original = document
            .library()
            .additional_applications
            .iter()
            .find(|application| application.id == "fixture-adventure-manual")
            .expect("fixture application")
            .clone();
        let edit = AdditionalApplicationEdit {
            name: "Fixture Bonus Campaign".into(),
            application_path: r"Games\Fixture Adventure\bonus.rom".into(),
            command_line: Some("--bonus \"two words\"".into()),
            auto_run_before: true,
            auto_run_after: false,
            wait_for_exit: true,
            use_emulator: true,
            emulator_id: Some("fixture-emulator".into()),
            use_dos_box: false,
            priority: 3,
            play_count: 7,
            play_time_seconds: 1234,
            disc: Some(2),
            side_a: false,
            side_b: true,
            developer: Some("Fixture Forge".into()),
            publisher: Some("Fixture Press".into()),
            region: Some("Europe".into()),
            release_date: Some("2004-05-06".into()),
            version: Some("Rev 2".into()),
            status: Some("Imported ROM".into()),
            installed: Some(true),
            last_played: Some("2026-07-22T12:34:56.0000000-07:00".into()),
        };
        let updated = document
            .set_additional_application("fixture-adventure-manual", edit.clone())
            .expect("edit additional application");
        assert_eq!(AdditionalApplicationEdit::from(&updated), edit);
        assert_eq!(updated.id, original.id);
        assert_eq!(updated.game_id, original.game_id);
        assert_eq!(updated.gog_app_id, original.gog_app_id);
        assert_eq!(updated.origin_app_id, original.origin_app_id);
        assert_eq!(updated.origin_install_path, original.origin_install_path);
        assert_eq!(updated.has_cloud_synced, original.has_cloud_synced);

        let bytes = document.to_xml_bytes().expect("serialize edit");
        let xml = String::from_utf8_lossy(&bytes);
        assert!(xml.contains(
            "<FutureAdditionalApplicationElement>keep-additional-app-data</FutureAdditionalApplicationElement>"
        ));
        let reparsed = PlatformDocument::from_reader("Fixture Console.xml", bytes.as_slice())
            .expect("reparse edit");
        let reparsed = reparsed
            .library()
            .additional_applications
            .iter()
            .find(|application| application.id == "fixture-adventure-manual")
            .expect("reparsed application");
        assert_eq!(AdditionalApplicationEdit::from(reparsed), edit);
    }

    #[test]
    fn additional_application_becomes_default_without_removing_either_record() {
        let mut document = PlatformDocument::from_reader("Fixture Console.xml", FIXTURE.as_bytes())
            .expect("parse fixture");
        let application = AdditionalApplication {
            id: "fixture-default-version".into(),
            game_id: "fixture-adventure".into(),
            name: "Play European Version...".into(),
            application_path: r"Games\Fixture Adventure\europe.rom".into(),
            command_line: Some("--region europe".into()),
            use_emulator: false,
            emulator_id: Some("ignored-for-direct-launch".into()),
            use_dos_box: false,
            play_count: 11,
            play_time_seconds: 9876,
            developer: Some("Version Developer".into()),
            publisher: Some("Version Publisher".into()),
            region: Some("Europe".into()),
            release_date: Some("2004-05-06".into()),
            version: Some("Rev 4".into()),
            status: Some("Installed".into()),
            installed: Some(true),
            last_played: Some("2026-07-22T16:17:18.0000000-07:00".into()),
            gog_app_id: Some("gog-version".into()),
            origin_app_id: Some("origin-version".into()),
            origin_install_path: Some(r"C:\Games\Fixture Version".into()),
            has_cloud_synced: true,
            ..AdditionalApplication::default()
        };
        document
            .add_additional_application(application.clone())
            .expect("add selectable version");

        let updated = document
            .make_additional_application_default(&application.id)
            .expect("make version default");
        assert_eq!(updated.id, "fixture-adventure");
        assert_eq!(updated.title, "Fixture Adventure");
        assert_eq!(
            updated.application_path,
            r"Games\Fixture Adventure\europe.rom"
        );
        assert_eq!(updated.command_line.as_deref(), Some("--region europe"));
        assert_eq!(
            updated.emulator_id.as_deref(),
            Some(lb_domain::UNASSIGNED_EMULATOR_ID)
        );
        assert!(!updated.use_dos_box);
        assert!(!updated.use_scumm_vm);
        assert_eq!(updated.developer.as_deref(), Some("Version Developer"));
        assert_eq!(updated.publisher.as_deref(), Some("Version Publisher"));
        assert_eq!(updated.region.as_deref(), Some("Europe"));
        assert_eq!(updated.release_date.as_deref(), Some("2004-05-06"));
        assert_eq!(updated.version.as_deref(), Some("Rev 4"));
        assert_eq!(updated.status.as_deref(), Some("Installed"));
        assert_eq!(updated.installed, Some(true));
        assert_eq!(updated.play_count, 11);
        assert_eq!(updated.play_time_seconds, 9876);
        assert_eq!(
            updated.last_played_date.as_deref(),
            Some("2026-07-22T16:17:18.0000000-07:00")
        );
        assert_eq!(updated.gog_app_id.as_deref(), Some("gog-version"));
        assert_eq!(updated.origin_app_id.as_deref(), Some("origin-version"));
        assert_eq!(
            updated.origin_install_path.as_deref(),
            Some(r"C:\Games\Fixture Version")
        );
        assert!(updated.has_cloud_synced);
        assert_eq!(
            document
                .library()
                .additional_applications
                .iter()
                .find(|candidate| candidate.id == application.id),
            Some(&application)
        );

        let bytes = document.to_xml_bytes().expect("serialize default change");
        let xml = String::from_utf8_lossy(&bytes);
        assert!(
            xml.contains("<TestOnlyUnknownGameElement>keep-this-too</TestOnlyUnknownGameElement>")
        );
        assert!(xml.contains(
            "<FutureAdditionalApplicationElement>keep-additional-app-data</FutureAdditionalApplicationElement>"
        ));
        let reparsed = PlatformDocument::from_reader("Fixture Console.xml", bytes.as_slice())
            .expect("reparse default change");
        let reparsed_game = reparsed
            .library()
            .games
            .iter()
            .find(|game| game.id == "fixture-adventure")
            .expect("reparsed owner");
        assert_eq!(reparsed_game, &updated);
        assert_eq!(reparsed.library().additional_applications.len(), 2);
    }

    #[test]
    fn additional_application_removal_refuses_game_save_references() {
        let referenced = FIXTURE.replacen(
            "<GameId>fixture-adventure</GameId>",
            "<GameId>fixture-adventure</GameId><AdditionalApplicationId>fixture-adventure-manual</AdditionalApplicationId>",
            1,
        );
        let mut document =
            PlatformDocument::from_reader("Fixture Console.xml", referenced.as_bytes())
                .expect("parse referenced fixture");
        assert!(matches!(
            document.remove_additional_application("fixture-adventure-manual"),
            Err(StorageError::AdditionalApplicationHasReferences { count: 1, .. })
        ));
        assert_eq!(document.library().additional_applications.len(), 1);

        let mut document = PlatformDocument::from_reader("Fixture Console.xml", FIXTURE.as_bytes())
            .expect("parse unreferenced fixture");
        let removed = document
            .remove_additional_application("fixture-adventure-manual")
            .expect("remove unreferenced application");
        assert_eq!(removed.id, "fixture-adventure-manual");
        assert!(document.library().additional_applications.is_empty());
        let bytes = document.to_xml_bytes().expect("serialize removal");
        assert!(!String::from_utf8_lossy(&bytes).contains("<AdditionalApplication>"));
        assert!(String::from_utf8_lossy(&bytes)
            .contains("<FutureRootElement>preserve-me</FutureRootElement>"));
    }

    #[test]
    fn delete_reference_scan_covers_platform_records_and_playlists() {
        let directory = tempfile::tempdir().expect("temporary library");
        let platforms = directory.path().join("Data/Platforms");
        let playlists = directory.path().join("Data/Playlists");
        fs::create_dir_all(&platforms).expect("create platform directory");
        fs::create_dir_all(&playlists).expect("create playlist directory");
        fs::write(platforms.join("Fixture Console.xml"), FIXTURE).expect("write platform");
        fs::write(
            playlists.join("Fixture Playlist.xml"),
            include_str!("../../../fixtures/launchbox/Data/Playlists/Fixture Playlist.xml"),
        )
        .expect("write playlist");
        let platform_catalog = include_str!("../../../fixtures/launchbox/Data/Platforms.xml")
            .replacen(
                "</Platform>",
                "<LastGameId>fixture-puzzle</LastGameId></Platform>",
                1,
            );
        fs::write(
            directory.path().join("Data/Platforms.xml"),
            platform_catalog,
        )
        .expect("write platform catalog");
        fs::write(
            directory.path().join("Data/ImportBlacklist.xml"),
            include_str!("../../../fixtures/launchbox/Data/ImportBlacklist.xml"),
        )
        .expect("write import blacklist");

        let references =
            find_game_references(directory.path(), "fixture-adventure").expect("scan references");
        assert_eq!(
            references
                .iter()
                .map(|reference| reference.kind)
                .collect::<Vec<_>>(),
            [
                GameReferenceKind::AdditionalApplication,
                GameReferenceKind::AlternateName,
                GameReferenceKind::CustomField,
                GameReferenceKind::GameSave,
                GameReferenceKind::PlaylistGame,
            ]
        );

        let mut document =
            PlatformDocument::load(platforms.join("Fixture Console.xml")).expect("load platform");
        assert!(matches!(
            document.remove_game("fixture-adventure"),
            Err(StorageError::GameHasReferences { count: 4, .. })
        ));
        assert!(find_game_references(directory.path(), "fixture-puzzle")
            .expect("scan unreferenced game")
            .iter()
            .any(|reference| reference.kind == GameReferenceKind::NavigationLastGame));
        assert_eq!(
            find_game_references(directory.path(), "fixture-racer")
                .expect("scan controller support")
                .iter()
                .map(|reference| reference.kind)
                .collect::<Vec<_>>(),
            [GameReferenceKind::ControllerSupport]
        );
        assert_eq!(
            find_game_references(directory.path(), "fixture-prototype")
                .expect("scan clone relation")
                .iter()
                .map(|reference| reference.kind)
                .collect::<Vec<_>>(),
            [GameReferenceKind::CloneOf]
        );
        assert_eq!(
            find_game_references(directory.path(), "fixture-ignored-game")
                .expect("scan import blacklist")
                .iter()
                .map(|reference| reference.kind)
                .collect::<Vec<_>>(),
            [GameReferenceKind::ImportBlacklist]
        );
    }

    #[test]
    fn platform_reference_scan_covers_every_modeled_dependency_family() {
        let directory = tempfile::tempdir().expect("temporary library");
        let data = directory.path().join("Data");
        let platforms = data.join("Platforms");
        let playlists = data.join("Playlists");
        fs::create_dir_all(&platforms).unwrap();
        fs::create_dir_all(&playlists).unwrap();
        fs::write(platforms.join("Fixture Console.xml"), FIXTURE).unwrap();

        let playlist =
            include_str!("../../../fixtures/launchbox/Data/Playlists/Fixture Playlist.xml")
                .replace(
                    "<FieldKey>Favorite</FieldKey>",
                    "<FieldKey>Platform</FieldKey>",
                )
                .replace("<Value>true</Value>", "<Value>Fixture Console</Value>");
        fs::write(playlists.join("Fixture Playlist.xml"), playlist).unwrap();

        let catalog = include_str!("../../../fixtures/launchbox/Data/Platforms.xml").replace(
            "</PlatformCategory>",
            "<LastSelectedChild>Fixture Console</LastSelectedChild></PlatformCategory>",
        );
        fs::write(data.join("Platforms.xml"), catalog).unwrap();

        let emulators = include_str!("../../../fixtures/launchbox/Data/Emulators.xml").replace(
            "<Title>Fixture Emulator</Title>",
            "<Title>Fixture Emulator</Title><DefaultPlatform>Fixture Console</DefaultPlatform>",
        );
        fs::write(data.join("Emulators.xml"), emulators).unwrap();

        let parents = include_str!("../../../fixtures/launchbox/Data/Parents.xml").replace(
            "</LaunchBox>",
            "<Parent><ParentPlatformName>Fixture Console</ParentPlatformName><PlaylistId>fixture-playlist</PlaylistId></Parent></LaunchBox>",
        );
        fs::write(data.join("Parents.xml"), parents).unwrap();

        let controllers = include_str!("../../../fixtures/launchbox/Data/GameControllers.xml")
            .replace(
                "<AssociatedPlatforms />",
                "<AssociatedPlatforms>Other;Fixture Console</AssociatedPlatforms>",
            );
        fs::write(data.join("GameControllers.xml"), controllers).unwrap();

        let settings = include_str!("../../../fixtures/launchbox/Data/Settings.xml").replace(
            "</Settings>",
            "<SelectedPlatform>Fixture Console</SelectedPlatform></Settings>",
        );
        fs::write(data.join("Settings.xml"), settings).unwrap();

        let kinds = find_platform_references(directory.path(), "fixture console")
            .unwrap()
            .into_iter()
            .map(|reference| reference.kind)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            kinds,
            std::collections::BTreeSet::from([
                PlatformReferenceKind::Game,
                PlatformReferenceKind::EmulatorMapping,
                PlatformReferenceKind::EmulatorDefault,
                PlatformReferenceKind::ParentChild,
                PlatformReferenceKind::ParentTarget,
                PlatformReferenceKind::PlaylistGame,
                PlatformReferenceKind::PlaylistFilter,
                PlatformReferenceKind::NavigationLastSelectedChild,
                PlatformReferenceKind::ControllerAssociation,
                PlatformReferenceKind::FrontendSetting,
            ])
        );
        assert!(find_platform_references(directory.path(), "No References")
            .unwrap()
            .is_empty());
    }

    #[test]
    fn modeled_emulator_fields_match_the_frozen_real_install_schema() {
        let schema: serde_json::Value =
            serde_json::from_str(include_str!("../../../analysis/real-install-schema.json"))
                .expect("parse value-free real-install schema");
        let fields = &schema["document_groups"]["Emulators.xml"]["record_fields"];
        let mut observed_emulators = fields["Emulator"]
            .as_array()
            .expect("Emulator field inventory")
            .iter()
            .map(|field| field.as_str().expect("field name"))
            .collect::<Vec<_>>();
        observed_emulators.sort_unstable();
        let mut modeled_emulators = EMULATOR_XML_FIELDS.to_vec();
        modeled_emulators.sort_unstable();
        assert_eq!(modeled_emulators, observed_emulators);

        let mut observed_platforms = fields["EmulatorPlatform"]
            .as_array()
            .expect("EmulatorPlatform field inventory")
            .iter()
            .map(|field| field.as_str().expect("field name"))
            .collect::<Vec<_>>();
        observed_platforms.sort_unstable();
        let mut modeled_platforms = EMULATOR_PLATFORM_XML_FIELDS.to_vec();
        modeled_platforms.sort_unstable();
        assert_eq!(modeled_platforms, observed_platforms);
    }

    #[test]
    fn emulator_crud_is_typed_source_indexed_lossless_and_keeps_paths_lexical() {
        let fixture = include_str!("../../../fixtures/launchbox/Data/Emulators.xml")
            .replace(
                "  </Emulator>\n  <EmulatorPlatform>",
                "    <FutureEmulatorField>keep-emulator-data</FutureEmulatorField>\n  </Emulator>\n  <EmulatorPlatform>",
            )
            .replace(
                "  </EmulatorPlatform>\n</LaunchBox>",
                "    <FutureMappingField>keep-mapping-data</FutureMappingField>\n  </EmulatorPlatform>\n</LaunchBox>",
            );
        let mut document = AuxiliaryDocument::from_reader(
            AuxiliaryDocumentKind::Emulators,
            "Emulators.xml",
            fixture.as_bytes(),
        )
        .expect("load emulator fixture");
        let original = document
            .emulator_configuration()
            .expect("typed configuration")
            .emulators[0]
            .clone();
        let second = Emulator {
            id: "second-emulator".into(),
            title: "Second Emulator".into(),
            application_path: r"C:\Portable\Second\second.exe".into(),
            command_line: Some("--fullscreen".into()),
            use_startup_screen: true,
            ..Emulator::default()
        };
        document
            .add_emulator(
                second.clone(),
                vec![EmulatorPlatform {
                    emulator_id: second.id.clone(),
                    platform: "Fixture Console".into(),
                    command_line: Some("--second".into()),
                    default: true,
                    auto_extract: Some(false),
                    m3u_disc_load_enabled: true,
                }],
            )
            .expect("add emulator and swap default");
        let configuration = document.emulator_configuration().unwrap();
        assert_eq!(configuration.emulators.len(), 2);
        assert!(!configuration.platforms[0].default);
        assert!(configuration.platforms[1].default);

        let edited = Emulator {
            title: "Edited Fixture Emulator".into(),
            application_path: r"Emulators\Fixture\fixture.exe".into(),
            auto_hotkey_script: Some("Run lexical script".into()),
            ..original.clone()
        };
        document
            .set_emulator(
                &original.id,
                edited.clone(),
                vec![IndexedPlatformRecordEdit {
                    source_index: Some(0),
                    record: EmulatorPlatform {
                        emulator_id: original.id.clone(),
                        platform: "Fixture Console".into(),
                        command_line: Some("--edited-platform".into()),
                        default: true,
                        auto_extract: None,
                        m3u_disc_load_enabled: true,
                    },
                }],
            )
            .expect("edit source-indexed mapping and restore default");
        let configuration = document.emulator_configuration().unwrap();
        assert_eq!(configuration.emulators[0], edited);
        assert!(configuration.platforms[0].default);
        assert!(!configuration.platforms[1].default);

        let removed = document
            .remove_emulator(&second.id)
            .expect("remove emulator and owned mappings");
        assert_eq!(removed.emulator, second);
        assert_eq!(removed.platforms.len(), 1);
        let xml = String::from_utf8(document.to_xml_bytes().unwrap()).unwrap();
        assert!(xml.contains("<FutureEmulatorField>keep-emulator-data</FutureEmulatorField>"));
        assert!(xml.contains("<FutureMappingField>keep-mapping-data</FutureMappingField>"));
        assert!(xml.contains(r"<ApplicationPath>Emulators\Fixture\fixture.exe</ApplicationPath>"));
        assert!(!xml.contains(r"C:\Portable\Second\second.exe"));
    }

    #[test]
    fn emulator_reference_scan_covers_games_and_additional_applications_only() {
        let directory = tempfile::tempdir().expect("temporary library");
        let data = directory.path().join("Data");
        let platforms = data.join("Platforms");
        fs::create_dir_all(&platforms).unwrap();
        let platform = FIXTURE.replace(
            "<EmulatorId />",
            "<EmulatorId>fixture-emulator</EmulatorId>",
        );
        fs::write(platforms.join("Fixture Console.xml"), platform).unwrap();
        fs::write(
            data.join("Emulators.xml"),
            include_str!("../../../fixtures/launchbox/Data/Emulators.xml"),
        )
        .unwrap();

        let references = find_emulator_references(directory.path(), "FIXTURE-EMULATOR")
            .expect("scan emulator references");
        assert_eq!(
            references
                .iter()
                .map(|reference| reference.kind)
                .collect::<Vec<_>>(),
            [
                EmulatorReferenceKind::Game,
                EmulatorReferenceKind::AdditionalApplication,
            ]
        );
        assert!(
            find_emulator_references(directory.path(), "missing-emulator")
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn game_state_edit_updates_typed_and_lossless_views() {
        let mut document = PlatformDocument::from_reader("Fixture Console.xml", FIXTURE.as_bytes())
            .expect("parse fixture");
        document
            .set_game_state("fixture-adventure", false, true, 2)
            .expect("edit game state");

        let game = document
            .library()
            .games
            .iter()
            .find(|game| game.id == "fixture-adventure")
            .expect("typed game");
        assert!(!game.favorite);
        assert!(game.completed);
        assert_eq!(game.star_rating, 2);

        let bytes = document.to_xml_bytes().expect("serialize fixture");
        let reparsed = PlatformDocument::from_reader("Fixture Console.xml", bytes.as_slice())
            .expect("reparse edited fixture");
        let game = reparsed
            .library()
            .games
            .iter()
            .find(|game| game.id == "fixture-adventure")
            .expect("reparsed game");
        assert!(!game.favorite);
        assert!(game.completed);
        assert_eq!(game.star_rating, 2);
        assert!(String::from_utf8(bytes)
            .expect("UTF-8 XML")
            .contains("<TestOnlyUnknownGameElement>keep-this-too</TestOnlyUnknownGameElement>"));

        assert!(matches!(
            document.set_game_state("fixture-adventure", false, true, 6),
            Err(StorageError::InvalidDomain(
                ValidationError::InvalidStarRating { rating: 6, .. }
            ))
        ));
    }

    #[test]
    fn game_metadata_edit_updates_known_fields_and_preserves_unknown_xml() {
        let mut document = PlatformDocument::from_reader("Fixture Console.xml", FIXTURE.as_bytes())
            .expect("parse fixture");
        let original = document
            .library()
            .games
            .iter()
            .find(|game| game.id == "fixture-adventure")
            .expect("fixture game");
        let mut metadata = GameMetadata::from(original);
        metadata.title = "Edited Adventure".into();
        metadata.sort_title = Some("Adventure, Edited".into());
        metadata.notes = Some("Edited multiline-ready notes.".into());
        metadata.developer = Some("New Developer".into());
        metadata.genre = Some("Puzzle Adventure".into());
        metadata.max_players = Some(6);
        metadata.play_mode = Some("Local Cooperative".into());
        metadata.progress = Some("75%".into());
        metadata.publisher = Some("New Publisher".into());
        metadata.rating = Some("T".into());
        metadata.region = Some("Europe".into());
        metadata.release_date = Some("2001-02-03".into());
        metadata.release_type = Some("Homebrew".into());
        metadata.series = None;
        metadata.source = Some("Physical Media".into());
        metadata.status = Some("Imported".into());
        metadata.version = Some("2.0".into());
        metadata.wikipedia_url = None;

        let updated = document
            .set_game_metadata("fixture-adventure", metadata.clone())
            .expect("edit metadata");
        assert_eq!(GameMetadata::from(&updated), metadata);

        let bytes = document.to_xml_bytes().expect("serialize metadata edit");
        let xml = String::from_utf8(bytes.clone()).expect("UTF-8 XML");
        assert!(
            xml.contains("<TestOnlyUnknownGameElement>keep-this-too</TestOnlyUnknownGameElement>")
        );
        assert!(!xml.contains("<Series>"));
        assert!(!xml.contains("<WikipediaURL>"));

        let reparsed = PlatformDocument::from_reader("Fixture Console.xml", bytes.as_slice())
            .expect("reparse metadata edit");
        let reparsed = reparsed
            .library()
            .games
            .iter()
            .find(|game| game.id == "fixture-adventure")
            .expect("reparsed game");
        assert_eq!(GameMetadata::from(reparsed), metadata);

        let before_invalid = document
            .to_xml_bytes()
            .expect("serialize before invalid edit");
        let mut invalid = metadata;
        invalid.title = "   ".into();
        assert!(matches!(
            document.set_game_metadata("fixture-adventure", invalid),
            Err(StorageError::EmptyGameTitle { .. })
        ));
        assert_eq!(
            document
                .to_xml_bytes()
                .expect("serialize after invalid edit"),
            before_invalid
        );
    }

    #[test]
    fn launch_configuration_edit_is_typed_lossless_and_keeps_paths_lexical() {
        let mut document = PlatformDocument::from_reader("Fixture Console.xml", FIXTURE.as_bytes())
            .expect("parse fixture");
        let original = document
            .library()
            .games
            .iter()
            .find(|game| game.id == "fixture-adventure")
            .expect("fixture game");
        let mut configuration = GameLaunchConfiguration::from(original);
        configuration.application_path = r"Runtime\edited-recorder".into();
        configuration.command_line = Some(r#"--edited "%gameid%" "two words""#.into());
        configuration.emulator_id = Some(lb_domain::UNASSIGNED_EMULATOR_ID.into());
        configuration.use_dos_box = false;
        configuration.custom_dos_box_version_path = None;
        configuration.dos_box_configuration_path = None;
        configuration.use_scumm_vm = false;
        configuration.scumm_vm_aspect_correction = false;
        configuration.scumm_vm_fullscreen = false;
        configuration.scumm_vm_game_data_folder_path = None;
        configuration.scumm_vm_game_type = None;

        let updated = document
            .set_game_launch_configuration("fixture-adventure", configuration.clone())
            .expect("edit launch configuration");
        assert_eq!(GameLaunchConfiguration::from(&updated), configuration);
        let bytes = document.to_xml_bytes().expect("serialize launch edit");
        let xml = String::from_utf8(bytes.clone()).expect("UTF-8 XML");
        assert!(xml.contains(r"<ApplicationPath>Runtime\edited-recorder</ApplicationPath>"));
        assert!(xml.contains("<UseDosBox>false</UseDosBox>"));
        assert!(xml.contains("<UseScummVM>false</UseScummVM>"));
        assert!(!xml.contains("<CustomDosBoxVersionPath>"));
        assert!(!xml.contains("<ScummVMGameDataFolderPath>"));
        assert!(
            xml.contains("<TestOnlyUnknownGameElement>keep-this-too</TestOnlyUnknownGameElement>")
        );

        let reparsed = PlatformDocument::from_reader("Fixture Console.xml", bytes.as_slice())
            .expect("reparse launch edit");
        let reparsed = reparsed
            .library()
            .games
            .iter()
            .find(|game| game.id == "fixture-adventure")
            .expect("reparsed game");
        assert_eq!(GameLaunchConfiguration::from(reparsed), configuration);

        let before_invalid = document.to_xml_bytes().expect("serialize valid edit");
        let mut invalid = configuration;
        invalid.use_dos_box = true;
        invalid.use_scumm_vm = true;
        assert!(matches!(
            document.set_game_launch_configuration("fixture-adventure", invalid),
            Err(StorageError::InvalidDomain(
                ValidationError::ConflictingGameLaunchModes { .. }
            ))
        ));
        assert_eq!(
            document
                .to_xml_bytes()
                .expect("serialize after invalid edit"),
            before_invalid
        );
    }

    #[test]
    fn repeated_game_metadata_edits_preserve_exact_source_rows_and_unknown_xml() {
        let mut document = PlatformDocument::from_reader("Fixture Console.xml", FIXTURE.as_bytes())
            .expect("parse fixture");

        let alternate_names = document
            .set_game_alternate_names(
                "fixture-adventure",
                vec![
                    IndexedPlatformRecordEdit {
                        source_index: Some(0),
                        record: AlternateName {
                            game_id: "fixture-adventure".into(),
                            name: "Adventure, The Fixture".into(),
                            region: None,
                        },
                    },
                    IndexedPlatformRecordEdit {
                        source_index: None,
                        record: AlternateName {
                            game_id: "fixture-adventure".into(),
                            name: "Aventure de test".into(),
                            region: Some("France".into()),
                        },
                    },
                ],
            )
            .expect("edit alternate names");
        assert_eq!(alternate_names.len(), 2);
        assert_eq!(alternate_names[0].region, None);

        let custom_fields = document
            .set_game_custom_fields(
                "fixture-adventure",
                vec![
                    IndexedPlatformRecordEdit {
                        source_index: Some(0),
                        record: CustomField {
                            game_id: "fixture-adventure".into(),
                            name: "Cabinet Style".into(),
                            value: "Cocktail".into(),
                        },
                    },
                    IndexedPlatformRecordEdit {
                        source_index: None,
                        record: CustomField {
                            game_id: "fixture-adventure".into(),
                            name: "PCB".into(),
                            value: String::new(),
                        },
                    },
                ],
            )
            .expect("edit custom fields");
        assert_eq!(custom_fields.len(), 2);

        let bytes = document
            .to_xml_bytes()
            .expect("serialize repeated metadata");
        let xml = String::from_utf8(bytes.clone()).expect("UTF-8 XML");
        assert!(xml.contains("<Name>Adventure, The Fixture</Name>"));
        assert!(xml.contains("<Name>Aventure de test</Name>"));
        assert!(xml.contains("<Region>France</Region>"));
        assert!(xml.contains(
            "<FutureAlternateNameElement>keep-alternate-name-data</FutureAlternateNameElement>"
        ));
        assert!(xml.contains("<Name>Cabinet Style</Name>"));
        assert!(xml.contains("<Value>Cocktail</Value>"));
        assert!(xml.contains("<Name>PCB</Name>"));
        assert!(xml.contains(
            "<FutureCustomFieldElement>keep-custom-field-data</FutureCustomFieldElement>"
        ));
        let root = Element::parse(bytes.as_slice()).expect("parse repeated metadata XML");
        let empty_field = elements_named(&root, "CustomField")
            .find(|field| child_text(field, "Name").as_deref() == Some("PCB"))
            .expect("new empty-valued custom field");
        assert!(empty_field.get_child("Value").is_some());

        let reparsed = PlatformDocument::from_reader("Fixture Console.xml", bytes.as_slice())
            .expect("reparse repeated metadata");
        assert_eq!(reparsed.library().alternate_names, alternate_names);
        assert_eq!(reparsed.library().custom_fields, custom_fields);

        let before_invalid = document
            .to_xml_bytes()
            .expect("serialize before invalid source ordinals");
        let duplicate_source = vec![
            IndexedPlatformRecordEdit {
                source_index: Some(0),
                record: alternate_names[0].clone(),
            },
            IndexedPlatformRecordEdit {
                source_index: Some(0),
                record: alternate_names[0].clone(),
            },
        ];
        assert!(matches!(
            document.set_game_alternate_names("fixture-adventure", duplicate_source),
            Err(StorageError::InvalidGameRecordEdit { .. })
        ));
        assert_eq!(
            document
                .to_xml_bytes()
                .expect("serialize after rejected source ordinals"),
            before_invalid
        );
    }

    #[test]
    fn play_session_stats_update_games_and_additional_apps_losslessly() {
        let mut document = PlatformDocument::from_reader("Fixture Console.xml", FIXTURE.as_bytes())
            .expect("load platform fixture");
        let timestamp = "2026-07-22T12:34:56.1234567-07:00";

        let game = document
            .record_game_play_start("fixture-racer", timestamp)
            .expect("record game start");
        assert_eq!(game.play_count, 9);
        assert_eq!(game.last_played_date.as_deref(), Some(timestamp));
        let game = document
            .record_game_play_time("fixture-racer", 61)
            .expect("record game duration");
        assert_eq!(game.play_time_seconds, 14_461);

        let application = document
            .record_additional_application_play_start("fixture-adventure-manual", timestamp)
            .expect("record additional application start");
        assert_eq!(application.play_count, 1);
        assert_eq!(application.last_played.as_deref(), Some(timestamp));
        let application = document
            .record_additional_application_play_time("fixture-adventure-manual", 7)
            .expect("record additional application duration");
        assert_eq!(application.play_time_seconds, 7);

        let bytes = document.to_xml_bytes().expect("serialize session stats");
        let reparsed = PlatformDocument::from_reader("Fixture Console.xml", bytes.as_slice())
            .expect("reparse session stats");
        let game = reparsed
            .library()
            .games
            .iter()
            .find(|game| game.id == "fixture-racer")
            .expect("updated game");
        assert_eq!(game.play_count, 9);
        assert_eq!(game.play_time_seconds, 14_461);
        assert_eq!(game.last_played_date.as_deref(), Some(timestamp));
        let application = reparsed
            .library()
            .additional_applications
            .iter()
            .find(|application| application.id == "fixture-adventure-manual")
            .expect("updated additional application");
        assert_eq!(application.play_count, 1);
        assert_eq!(application.play_time_seconds, 7);
        assert_eq!(application.last_played.as_deref(), Some(timestamp));

        let xml = String::from_utf8(bytes).expect("UTF-8 XML");
        assert!(
            xml.contains("<TestOnlyUnknownGameElement>keep-this-too</TestOnlyUnknownGameElement>")
        );
        assert!(xml.contains("<FutureAdditionalApplicationElement>keep-additional-app-data</FutureAdditionalApplicationElement>"));
        assert!(xml.contains("<FutureRootElement>preserve-me</FutureRootElement>"));
    }

    #[test]
    fn play_session_stats_reject_empty_timestamps_and_integer_overflow() {
        let mut document = PlatformDocument::from_reader("Fixture Console.xml", FIXTURE.as_bytes())
            .expect("load platform fixture");
        assert!(matches!(
            document.record_game_play_start("fixture-racer", ""),
            Err(StorageError::EmptyLastPlayedTimestamp)
        ));

        document
            .library
            .games
            .iter_mut()
            .find(|game| game.id == "fixture-racer")
            .expect("fixture game")
            .play_count = u32::MAX;
        assert!(matches!(
            document.record_game_play_start("fixture-racer", "2026-07-22T12:34:56.1234567-07:00"),
            Err(StorageError::PlayStatisticOverflow {
                field: "PlayCount",
                ..
            })
        ));

        document
            .library
            .additional_applications
            .iter_mut()
            .find(|application| application.id == "fixture-adventure-manual")
            .expect("fixture additional application")
            .play_time_seconds = u64::MAX;
        assert!(matches!(
            document.record_additional_application_play_time("fixture-adventure-manual", 1),
            Err(StorageError::PlayStatisticOverflow {
                field: "PlayTime",
                ..
            })
        ));
    }

    #[test]
    fn save_new_refuses_to_overwrite_existing_data() {
        let document = PlatformDocument::from_reader("Fixture Console.xml", FIXTURE.as_bytes())
            .expect("parse fixture");
        let directory = tempfile::tempdir().expect("temporary directory");
        let output = directory.path().join("platform.xml");
        document.save_new(&output).expect("first save");
        assert!(matches!(
            document.save_new(&output),
            Err(StorageError::Write { .. })
        ));
    }

    #[test]
    fn atomic_save_keeps_exact_backup_and_valid_replacement() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let target = directory.path().join("Fixture Console.xml");
        fs::write(&target, FIXTURE).expect("write fixture");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&target, fs::Permissions::from_mode(0o640))
                .expect("set fixture permissions");
        }

        let original = fs::read(&target).expect("read original fixture");
        let mut document = PlatformDocument::load(&target).expect("load fixture");
        document
            .set_game_title("fixture-adventure", "Atomic Adventure")
            .expect("rename fixture");
        let report = document.save_atomic().expect("atomic save");

        assert_eq!(report.target, target);
        assert_eq!(fs::read(&report.backup).expect("read backup"), original);
        let replacement = PlatformDocument::load(&target).expect("load replacement");
        assert_eq!(replacement.library().games[0].title, "Atomic Adventure");
        assert_eq!(replacement.library().additional_applications.len(), 1);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&target)
                    .expect("replacement metadata")
                    .permissions()
                    .mode()
                    & 0o777,
                0o640
            );
        }

        let temporary_files = fs::read_dir(directory.path())
            .expect("list directory")
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().contains("temporary"))
            .count();
        assert_eq!(temporary_files, 0);
    }

    #[test]
    fn revision_checked_regular_file_replace_streams_and_keeps_the_exact_target() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("vault.sav");
        let target = directory.path().join("active.sav");
        let source_bytes = vec![0x5a; 2 * 1024 * 1024 + 7];
        fs::write(&source, &source_bytes).unwrap();
        fs::write(&target, b"previous active save").unwrap();
        let source_revision = FileRevision::read(&source).unwrap();
        let target_revision = FileRevision::read(&target).unwrap();

        let report = replace_regular_file_from_source_if_revisions(
            &source,
            &source_revision,
            &target,
            &target_revision,
        )
        .unwrap();
        assert_eq!(fs::read(&target).unwrap(), source_bytes);
        assert_eq!(fs::read(&report.backup).unwrap(), b"previous active save");
        assert_eq!(fs::read(&source).unwrap(), source_bytes);
        assert_eq!(
            fs::read_dir(directory.path())
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| entry.file_name().to_string_lossy().contains("temporary"))
                .count(),
            0
        );
    }

    #[test]
    fn revision_checked_regular_file_replace_refuses_changed_source_or_target() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("vault.sav");
        let target = directory.path().join("active.sav");
        fs::write(&source, b"selected vault save").unwrap();
        fs::write(&target, b"active save").unwrap();
        let source_revision = FileRevision::read(&source).unwrap();
        let target_revision = FileRevision::read(&target).unwrap();

        fs::write(&source, b"changed vault save").unwrap();
        assert!(matches!(
            replace_regular_file_from_source_if_revisions(
                &source,
                &source_revision,
                &target,
                &target_revision,
            ),
            Err(StorageError::AtomicSourceConflict { .. })
        ));
        assert_eq!(fs::read(&target).unwrap(), b"active save");

        fs::write(&source, b"selected vault save").unwrap();
        fs::write(&target, b"changed active save").unwrap();
        assert!(matches!(
            replace_regular_file_from_source_if_revisions(
                &source,
                &source_revision,
                &target,
                &target_revision,
            ),
            Err(StorageError::WriteConflict { .. })
        ));
        assert_eq!(fs::read(&target).unwrap(), b"changed active save");
    }

    #[test]
    fn revision_checked_directory_replace_preserves_the_source_and_retains_the_target() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("working-card.ps2");
        let target = directory.path().join("active-card.ps2");
        fs::create_dir_all(source.join("BASLUS-RESTORED/nested")).unwrap();
        fs::create_dir_all(source.join("EMPTY")).unwrap();
        fs::write(source.join("BASLUS-RESTORED/icon.sys"), b"restored icon").unwrap();
        fs::write(
            source.join("BASLUS-RESTORED/nested/progress.dat"),
            b"restored progress",
        )
        .unwrap();
        fs::create_dir_all(target.join("BASLUS-ACTIVE")).unwrap();
        fs::write(target.join("BASLUS-ACTIVE/icon.sys"), b"active icon").unwrap();
        fs::write(target.join("unrelated.bin"), b"active unrelated member").unwrap();

        let source_revision = DirectoryRevision::read(&source).unwrap();
        let target_revision = DirectoryRevision::read(&target).unwrap();
        let report = replace_directory_from_source_if_revisions(
            &source,
            &source_revision,
            &target,
            &target_revision,
        )
        .unwrap();

        assert_eq!(DirectoryRevision::read(&target).unwrap(), source_revision);
        assert_eq!(DirectoryRevision::read(&source).unwrap(), source_revision);
        assert_eq!(
            DirectoryRevision::read(&report.backup).unwrap(),
            target_revision
        );
        assert!(target.join("EMPTY").is_dir());
        assert_eq!(
            fs::read(target.join("BASLUS-RESTORED/nested/progress.dat")).unwrap(),
            b"restored progress"
        );
        assert_eq!(
            fs::read(report.backup.join("unrelated.bin")).unwrap(),
            b"active unrelated member"
        );
        assert_eq!(
            fs::read_dir(directory.path())
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| entry.file_name().to_string_lossy().contains("temporary"))
                .count(),
            0
        );
    }

    #[test]
    fn revision_checked_directory_replace_refuses_changed_source_or_target() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("working-card.ps2");
        let target = directory.path().join("active-card.ps2");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&target).unwrap();
        fs::write(source.join("save.bin"), b"selected save").unwrap();
        fs::write(target.join("save.bin"), b"active save").unwrap();
        let source_revision = DirectoryRevision::read(&source).unwrap();
        let target_revision = DirectoryRevision::read(&target).unwrap();

        fs::write(source.join("save.bin"), b"changed selected save").unwrap();
        assert!(matches!(
            replace_directory_from_source_if_revisions(
                &source,
                &source_revision,
                &target,
                &target_revision,
            ),
            Err(StorageError::AtomicDirectorySourceConflict { .. })
        ));
        assert_eq!(fs::read(target.join("save.bin")).unwrap(), b"active save");

        fs::write(source.join("save.bin"), b"selected save").unwrap();
        fs::write(target.join("save.bin"), b"changed active save").unwrap();
        assert!(matches!(
            replace_directory_from_source_if_revisions(
                &source,
                &source_revision,
                &target,
                &target_revision,
            ),
            Err(StorageError::AtomicDirectoryTargetConflict { .. })
        ));
        assert_eq!(
            fs::read(target.join("save.bin")).unwrap(),
            b"changed active save"
        );
    }

    #[test]
    fn revision_checked_directory_replace_rolls_back_a_failed_second_move() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("working-card.ps2");
        let target = directory.path().join("active-card.ps2");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&target).unwrap();
        fs::write(source.join("save.bin"), b"replacement").unwrap();
        fs::write(target.join("save.bin"), b"original").unwrap();
        let source_revision = DirectoryRevision::read(&source).unwrap();
        let target_revision = DirectoryRevision::read(&target).unwrap();
        let mut rename_count = 0;

        let error = replace_directory_from_source_if_revisions_with(
            &source,
            &source_revision,
            &target,
            &target_revision,
            |from, to| {
                rename_count += 1;
                if rename_count == 2 {
                    return Err(std::io::Error::other("injected second move failure"));
                }
                fs::rename(from, to)
            },
        )
        .unwrap_err();

        assert!(matches!(
            error,
            StorageError::AtomicDirectoryReplaceRolledBack { .. }
        ));
        assert_eq!(fs::read(target.join("save.bin")).unwrap(), b"original");
        assert_eq!(fs::read(source.join("save.bin")).unwrap(), b"replacement");
        assert_eq!(
            fs::read_dir(directory.path())
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| {
                    let name = entry.file_name();
                    let name = name.to_string_lossy();
                    name.contains("temporary") || name.contains("backup")
                })
                .count(),
            0
        );
    }

    #[test]
    fn revision_checked_directory_delete_retains_the_exact_tree() {
        let directory = tempfile::tempdir().unwrap();
        let target = directory.path().join("data");
        fs::create_dir_all(target.join("nested/empty")).unwrap();
        fs::write(target.join("banner.bin"), b"banner").unwrap();
        fs::write(target.join("nested/progress.dat"), b"progress").unwrap();
        let revision = DirectoryRevision::read(&target).unwrap();

        let report = delete_directory_if_revision(&target, &revision).unwrap();

        assert_eq!(report.target, target);
        assert!(!target.exists());
        assert_eq!(DirectoryRevision::read(&report.backup).unwrap(), revision);
        assert_eq!(
            fs::read(report.backup.join("nested/progress.dat")).unwrap(),
            b"progress"
        );
        assert!(report.backup.join("nested/empty").is_dir());
    }

    #[test]
    fn revision_checked_directory_delete_refuses_a_changed_tree() {
        let directory = tempfile::tempdir().unwrap();
        let target = directory.path().join("data");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("progress.dat"), b"before").unwrap();
        let revision = DirectoryRevision::read(&target).unwrap();
        fs::write(target.join("progress.dat"), b"after").unwrap();

        assert!(matches!(
            delete_directory_if_revision(&target, &revision),
            Err(StorageError::AtomicDirectoryTargetConflict { .. })
        ));
        assert_eq!(fs::read(target.join("progress.dat")).unwrap(), b"after");
        assert_eq!(
            fs::read_dir(directory.path())
                .unwrap()
                .filter_map(Result::ok)
                .count(),
            1
        );
    }

    #[cfg(unix)]
    #[test]
    fn directory_revision_rejects_symlinks() {
        use std::os::unix::fs::symlink;

        let directory = tempfile::tempdir().unwrap();
        let card = directory.path().join("card.ps2");
        fs::create_dir_all(&card).unwrap();
        fs::write(directory.path().join("outside.bin"), b"outside").unwrap();
        symlink(
            directory.path().join("outside.bin"),
            card.join("unsafe-link.bin"),
        )
        .unwrap();

        assert!(matches!(
            DirectoryRevision::read(&card),
            Err(StorageError::AtomicDirectoryUnsafeEntry { .. })
        ));
    }

    #[test]
    fn revision_checked_file_set_delete_is_exact_and_rolls_back_partial_failure() {
        let rollback_directory = tempfile::tempdir().unwrap();
        let rollback_targets = ["adventure.bcr", "adventure.bkr", "adventure.smpc"]
            .map(|name| rollback_directory.path().join(name));
        let bytes = [
            b"active cartridge".as_slice(),
            b"active backup ram".as_slice(),
            b"active clock".as_slice(),
        ];
        for (target, bytes) in rollback_targets.iter().zip(bytes) {
            fs::write(target, bytes).unwrap();
        }
        let rollback_files = rollback_targets
            .iter()
            .map(|target| (target.clone(), FileRevision::read(target).unwrap()))
            .collect::<Vec<_>>();

        assert!(matches!(
            delete_regular_files_if_revisions_with(&rollback_files, |index, target| {
                if index == 1 {
                    Err(std::io::Error::other("injected companion delete failure"))
                } else {
                    fs::remove_file(target)
                }
            }),
            Err(StorageError::AtomicDeleteSetRolledBack { .. })
        ));
        for (target, bytes) in rollback_targets.iter().zip(bytes) {
            assert_eq!(fs::read(target).unwrap(), bytes);
            let prefix = format!(
                "{}.lbport-delete-backup-",
                target.file_name().unwrap().to_string_lossy()
            );
            let recovery = fs::read_dir(rollback_directory.path())
                .unwrap()
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .find(|candidate| {
                    candidate
                        .file_name()
                        .is_some_and(|name| name.to_string_lossy().starts_with(&prefix))
                })
                .unwrap();
            assert_eq!(fs::read(recovery).unwrap(), bytes);
        }

        let success_directory = tempfile::tempdir().unwrap();
        let success_targets = ["adventure.bcr", "adventure.bkr", "adventure.smpc"]
            .map(|name| success_directory.path().join(name));
        for (target, bytes) in success_targets.iter().zip(bytes) {
            fs::write(target, bytes).unwrap();
        }
        let success_files = success_targets
            .iter()
            .map(|target| (target.clone(), FileRevision::read(target).unwrap()))
            .collect::<Vec<_>>();
        let reports = delete_regular_files_if_revisions(&success_files).unwrap();
        assert_eq!(reports.len(), 3);
        for ((target, bytes), report) in success_targets.iter().zip(bytes).zip(reports) {
            assert!(!target.exists());
            assert_eq!(report.target, *target);
            assert_eq!(fs::read(report.backup).unwrap(), bytes);
        }
    }

    #[test]
    fn atomic_save_rejects_a_file_changed_since_load() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let target = directory.path().join("Fixture Console.xml");
        fs::write(&target, FIXTURE).expect("write fixture");
        let mut document = PlatformDocument::load(&target).expect("load fixture");
        document
            .set_game_title("fixture-adventure", "Stale Adventure")
            .expect("edit fixture");
        let external = FIXTURE.replace("Fixture Racer", "Externally Edited Racer");
        fs::write(&target, &external).expect("external edit");

        assert!(matches!(
            document.save_atomic(),
            Err(StorageError::WriteConflict { .. })
        ));
        assert_eq!(fs::read_to_string(&target).expect("read target"), external);
        assert_eq!(
            fs::read_dir(directory.path())
                .expect("list directory")
                .count(),
            1
        );
    }

    #[test]
    fn atomic_save_rejects_non_file_target() {
        let document = PlatformDocument::from_reader("Fixture Console.xml", FIXTURE.as_bytes())
            .expect("parse fixture");
        let directory = tempfile::tempdir().expect("temporary directory");
        assert!(matches!(
            document.save_atomic_to(directory.path()),
            Err(StorageError::AtomicTargetNotFile { .. })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn atomic_save_rejects_symlinks_without_touching_the_referent() {
        use std::os::unix::fs::symlink;

        let document = PlatformDocument::from_reader("Fixture Console.xml", FIXTURE.as_bytes())
            .expect("parse fixture");
        let directory = tempfile::tempdir().expect("temporary directory");
        let referent = directory.path().join("real.xml");
        let target = directory.path().join("linked.xml");
        fs::write(&referent, FIXTURE).expect("write referent");
        symlink(&referent, &target).expect("create symlink");

        assert!(matches!(
            document.save_atomic_to(&target),
            Err(StorageError::AtomicTargetNotFile { .. })
        ));
        assert_eq!(
            fs::read(&referent).expect("read referent"),
            FIXTURE.as_bytes()
        );
    }

    #[test]
    fn candidate_validation_failure_creates_no_backup_or_temporary_file() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let target = directory.path().join("Fixture Console.xml");
        fs::write(&target, FIXTURE).expect("write fixture");
        let original = fs::read(&target).expect("read fixture");

        assert!(matches!(
            save_atomic_bytes(&target, b"<not-launchbox />", |candidate| {
                PlatformDocument::from_reader(&target, candidate).map(|_| ())
            }),
            Err(StorageError::AtomicValidation { .. })
        ));
        assert_eq!(fs::read(&target).expect("read target"), original);
        assert_eq!(
            fs::read_dir(directory.path())
                .expect("list directory")
                .count(),
            1
        );
    }

    #[test]
    fn injected_replace_failure_leaves_original_and_exact_backup() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let target = directory.path().join("Fixture Console.xml");
        fs::write(&target, FIXTURE).expect("write fixture");
        let original = fs::read(&target).expect("read fixture");
        let mut document = PlatformDocument::load(&target).expect("load fixture");
        document
            .set_game_title("fixture-adventure", "Interrupted Adventure")
            .expect("edit fixture");
        let candidate = document.to_xml_bytes().expect("serialize candidate");

        let error = save_atomic_bytes_with_operations(
            &target,
            &candidate,
            |bytes| PlatformDocument::from_reader(&target, bytes).map(|_| ()),
            |_temporary, _target| Err(std::io::Error::other("injected replace failure")),
            |_target| Ok(()),
        )
        .expect_err("replace must fail");
        let backup = match error {
            StorageError::AtomicReplace { backup, .. } => backup,
            other => panic!("unexpected error: {other}"),
        };
        assert_eq!(fs::read(&target).expect("read target"), original);
        assert_eq!(fs::read(backup).expect("read backup"), original);
        assert_eq!(
            fs::read_dir(directory.path())
                .expect("list directory")
                .filter_map(Result::ok)
                .filter(|entry| entry.file_name().to_string_lossy().contains("temporary"))
                .count(),
            0
        );
    }

    #[test]
    fn injected_directory_sync_failure_reports_recoverable_backup() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let target = directory.path().join("Fixture Console.xml");
        fs::write(&target, FIXTURE).expect("write fixture");
        let original = fs::read(&target).expect("read fixture");
        let mut document = PlatformDocument::load(&target).expect("load fixture");
        document
            .set_game_title("fixture-adventure", "Unsynced Adventure")
            .expect("edit fixture");
        let candidate = document.to_xml_bytes().expect("serialize candidate");

        let error = save_atomic_bytes_with_operations(
            &target,
            &candidate,
            |bytes| PlatformDocument::from_reader(&target, bytes).map(|_| ()),
            replace_file,
            |_target| Err(std::io::Error::other("injected directory sync failure")),
        )
        .expect_err("directory sync must fail");
        let backup = match error {
            StorageError::AtomicDirectorySync { backup, .. } => backup,
            other => panic!("unexpected error: {other}"),
        };
        assert_eq!(fs::read(backup).expect("read backup"), original);
        let replacement = PlatformDocument::load(&target).expect("load replacement");
        assert_eq!(replacement.library().games[0].title, "Unsynced Adventure");
    }

    #[test]
    fn backup_restore_is_atomic_and_reversible() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let target = directory.path().join("Fixture Console.xml");
        fs::write(&target, FIXTURE).expect("write fixture");

        let mut changed = PlatformDocument::load(&target).expect("load fixture");
        changed
            .set_game_title("fixture-adventure", "Changed Adventure")
            .expect("rename fixture");
        let changed_report = changed.save_atomic().expect("save changed fixture");

        let restore_report = restore_platform_backup(&changed_report.backup, &target)
            .expect("restore original backup");
        let restored = PlatformDocument::load(&target).expect("load restored fixture");
        assert_eq!(restored.library().games[0].title, "Fixture Adventure");

        let replaced = PlatformDocument::load(&restore_report.backup)
            .expect("load backup created during restore");
        assert_eq!(replaced.library().games[0].title, "Changed Adventure");
    }

    #[test]
    fn every_auxiliary_family_has_a_lossless_typed_round_trip() {
        for (kind, path, fixture) in AUXILIARY_FIXTURES {
            let document = AuxiliaryDocument::from_reader(*kind, path, fixture.as_bytes())
                .unwrap_or_else(|error| panic!("parse {path}: {error}"));
            let bytes = document
                .to_xml_bytes()
                .unwrap_or_else(|error| panic!("serialize {path}: {error}"));
            AuxiliaryDocument::from_reader(*kind, path, bytes.as_slice())
                .unwrap_or_else(|error| panic!("reparse {path}: {error}"));
        }
    }

    #[test]
    fn every_auxiliary_family_has_a_validated_atomic_mutation_path() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let data = directory.path().join("Data");

        for (kind, relative_path, fixture) in AUXILIARY_FIXTURES {
            let target = data.join(relative_path);
            fs::create_dir_all(target.parent().expect("fixture parent"))
                .expect("create fixture directory");
            fs::write(&target, fixture).expect("write fixture");
            let original = fs::read(&target).expect("read original");
            let mut document = AuxiliaryDocument::load(&target)
                .unwrap_or_else(|error| panic!("load {relative_path}: {error}"));

            match kind {
                AuxiliaryDocumentKind::Playlist => document.set_record_field(
                    "Playlist",
                    "PlaylistId",
                    "fixture-playlist",
                    "Name",
                    "Edited Playlist",
                ),
                AuxiliaryDocumentKind::Emulators => document.set_record_field(
                    "Emulator",
                    "ID",
                    "fixture-emulator",
                    "Title",
                    "Edited Emulator",
                ),
                AuxiliaryDocumentKind::Platforms => document.set_record_field(
                    "Platform",
                    "Name",
                    "Fixture Console",
                    "Notes",
                    "Edited platform notes",
                ),
                AuxiliaryDocumentKind::Parents => document.set_record_field(
                    "Parent",
                    "PlatformName",
                    "Fixture Console",
                    "ParentPlatformCategoryName",
                    "Edited Category",
                ),
                AuxiliaryDocumentKind::GameControllers => document.set_record_field(
                    "GameController",
                    "Id",
                    "fixture-controller",
                    "Name",
                    "Edited Controller",
                ),
                AuxiliaryDocumentKind::InputBindings => document.set_record_field(
                    "InputBinding",
                    "InputAction",
                    "Play",
                    "ControllerBinding",
                    "ButtonB",
                ),
                AuxiliaryDocumentKind::ImportBlacklist => document.set_record_field(
                    "IgnoredGameId",
                    "GameId",
                    "fixture-ignored-game",
                    "GameId",
                    "fixture-edited-ignored-game",
                ),
                AuxiliaryDocumentKind::ListCache => document.set_record_field(
                    "ListCacheItem",
                    "PlaylistId",
                    "fixture-playlist",
                    "LaunchBoxCount",
                    "2",
                ),
                AuxiliaryDocumentKind::Settings => {
                    document.set_single_record_field("Settings", "Theme", "Edited Theme")
                }
                AuxiliaryDocumentKind::BigBoxSettings => document.set_single_record_field(
                    "BigBoxSettings",
                    "Theme",
                    "Edited BigBox Theme",
                ),
            }
            .unwrap_or_else(|error| panic!("mutate {relative_path}: {error}"));

            let report = document
                .save_atomic()
                .unwrap_or_else(|error| panic!("save {relative_path}: {error}"));
            assert_eq!(
                fs::read(&report.backup).expect("read backup"),
                original,
                "backup for {relative_path}"
            );
            AuxiliaryDocument::load(&target)
                .unwrap_or_else(|error| panic!("reload {relative_path}: {error}"));
        }
    }

    #[test]
    fn auxiliary_mutation_is_semantically_validated_before_commit() {
        let fixture = include_str!("../../../fixtures/launchbox/Data/ListCache.xml");
        let mut document = AuxiliaryDocument::from_reader(
            AuxiliaryDocumentKind::ListCache,
            "ListCache.xml",
            fixture.as_bytes(),
        )
        .expect("load list cache");
        let before = document.to_xml_bytes().expect("serialize original");

        assert!(matches!(
            document.set_record_field(
                "ListCacheItem",
                "PlaylistId",
                "fixture-playlist",
                "LaunchBoxIncludeBroken",
                "not-a-boolean"
            ),
            Err(StorageError::InvalidRecordField {
                record: "ListCacheItem",
                field: "LaunchBoxIncludeBroken",
                expected: "boolean"
            })
        ));
        assert_eq!(document.to_xml_bytes().expect("serialize after"), before);
    }

    #[test]
    fn auxiliary_writer_refuses_a_different_document_family_target() {
        let settings = include_str!("../../../fixtures/launchbox/Data/Settings.xml");
        let document = AuxiliaryDocument::from_reader(
            AuxiliaryDocumentKind::Settings,
            "Settings.xml",
            settings.as_bytes(),
        )
        .expect("load settings");
        let directory = tempfile::tempdir().expect("temporary directory");
        let target = directory.path().join("Emulators.xml");
        let emulators = include_str!("../../../fixtures/launchbox/Data/Emulators.xml");
        fs::write(&target, emulators).expect("write emulator target");

        assert!(matches!(
            document.save_atomic_to(&target),
            Err(StorageError::AuxiliaryDocumentKindMismatch {
                expected: AuxiliaryDocumentKind::Settings,
                actual: AuxiliaryDocumentKind::Emulators,
                ..
            })
        ));
        assert_eq!(
            fs::read(&target).expect("read untouched target"),
            emulators.as_bytes()
        );
    }

    #[test]
    fn auxiliary_atomic_save_keeps_exact_backup_and_can_be_restored() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let data = directory.path().join("Data");
        fs::create_dir(&data).expect("create Data directory");
        let target = data.join("Settings.xml");
        let fixture = include_str!("../../../fixtures/launchbox/Data/Settings.xml");
        fs::write(&target, fixture).expect("write fixture");
        let original = fs::read(&target).expect("read fixture");

        let mut document = AuxiliaryDocument::load(&target).expect("load settings");
        document
            .set_single_record_field("Settings", "Theme", "Updated Fixture Theme")
            .expect("edit theme");
        let report = document.save_atomic().expect("save settings atomically");
        assert_eq!(fs::read(&report.backup).expect("read backup"), original);

        let replacement = AuxiliaryDocument::load(&target).expect("load replacement");
        let settings = elements_named(replacement.root(), "Settings")
            .next()
            .expect("Settings record");
        assert_eq!(
            child_text(settings, "Theme").as_deref(),
            Some("Updated Fixture Theme")
        );

        let restore_report =
            restore_auxiliary_backup(&report.backup, &target).expect("restore settings backup");
        assert_eq!(fs::read(&target).expect("read restored target"), original);
        let changed_backup =
            AuxiliaryDocument::load_as(AuxiliaryDocumentKind::Settings, &restore_report.backup)
                .expect("load backup created during restore");
        let settings = elements_named(changed_backup.root(), "Settings")
            .next()
            .expect("Settings record");
        assert_eq!(
            child_text(settings, "Theme").as_deref(),
            Some("Updated Fixture Theme")
        );
    }

    #[test]
    fn auxiliary_unknown_records_survive_edit_and_serialization() {
        let fixture = include_str!("../../../fixtures/launchbox/Data/Settings.xml");
        let mut document = AuxiliaryDocument::from_reader(
            AuxiliaryDocumentKind::Settings,
            "Settings.xml",
            fixture.as_bytes(),
        )
        .expect("load settings");
        let mut future = Element::new("FutureRecord");
        set_child_text(&mut future, "Key", "fixture-future");
        set_child_text(&mut future, "Payload", "preserve-this");
        document
            .append_record(future)
            .expect("append future record");
        document
            .set_single_record_field("Settings", "Theme", "Edited Theme")
            .expect("edit known record");

        let bytes = document.to_xml_bytes().expect("serialize document");
        let reparsed = Element::parse(bytes.as_slice()).expect("reparse document");
        let future = elements_named(&reparsed, "FutureRecord")
            .next()
            .expect("future record");
        assert_eq!(
            child_text(future, "Payload").as_deref(),
            Some("preserve-this")
        );
    }

    #[test]
    fn empty_platform_uses_catalog_name_independently_of_native_filename() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("Dragon 32_64.xml");
        let document = PlatformDocument::new_empty(&path, "Dragon 32/64").unwrap();
        assert_eq!(document.library().name, "Dragon 32/64");
        fs::write(&path, document.to_xml_bytes().unwrap()).unwrap();

        assert_eq!(
            PlatformDocument::load(&path).unwrap().library().name,
            "Dragon 32_64"
        );
        let named = PlatformDocument::load_for_platform(&path, "Dragon 32/64").unwrap();
        assert_eq!(named.library().name, "Dragon 32/64");
        assert!(named.library().games.is_empty());
    }

    #[test]
    fn platform_catalog_add_remove_is_lossless_and_uses_typed_folders() {
        let fixture = include_str!("../../../fixtures/launchbox/Data/Platforms.xml");
        let mut document = AuxiliaryDocument::from_reader(
            AuxiliaryDocumentKind::Platforms,
            "Platforms.xml",
            fixture.as_bytes(),
        )
        .unwrap();
        let mut future = Element::new("FuturePlatformPolicy");
        set_child_text(&mut future, "Payload", "preserve-this");
        document.append_record(future).unwrap();
        let original = document.to_xml_bytes().unwrap();

        let name = "Dragon 32/64";
        let platform = PlatformDefinition {
            metadata: lb_domain::NavigationMetadata {
                name: name.into(),
                scrape_as: Some("Dragon 32/64".into()),
                ..lb_domain::NavigationMetadata::default()
            },
            ..PlatformDefinition::default()
        };
        let folders = lb_platform::default_platform_folders(name).unwrap();
        document
            .add_platform_definition(platform.clone(), folders.clone())
            .unwrap();

        let catalog = document.platform_catalog().unwrap();
        assert_eq!(catalog.platforms.last(), Some(&platform));
        assert_eq!(
            catalog
                .folders
                .iter()
                .filter(|folder| folder.platform == name)
                .count(),
            51
        );
        assert!(catalog
            .folders
            .iter()
            .filter(|folder| folder.platform == name)
            .all(|folder| !folder.folder_path.contains('/')));

        let removed = document.remove_platform_definition("dragon 32/64").unwrap();
        assert_eq!(removed.platform, platform);
        assert_eq!(removed.folders, folders);
        assert_eq!(document.to_xml_bytes().unwrap(), original);
    }

    #[test]
    fn duplicate_platform_add_is_rejected_without_mutating_catalog() {
        let fixture = include_str!("../../../fixtures/launchbox/Data/Platforms.xml");
        let mut document = AuxiliaryDocument::from_reader(
            AuxiliaryDocumentKind::Platforms,
            "Platforms.xml",
            fixture.as_bytes(),
        )
        .unwrap();
        let before = document.to_xml_bytes().unwrap();
        let platform = PlatformDefinition {
            metadata: lb_domain::NavigationMetadata {
                name: "fixture console".into(),
                ..lb_domain::NavigationMetadata::default()
            },
            ..PlatformDefinition::default()
        };

        assert!(matches!(
            document.add_platform_definition(platform, Vec::new()),
            Err(StorageError::DuplicatePlatformName { name }) if name == "fixture console"
        ));
        assert_eq!(document.to_xml_bytes().unwrap(), before);
    }

    #[test]
    fn platform_edit_is_typed_source_indexed_and_preserves_unknown_xml() {
        let fixture = include_str!("../../../fixtures/launchbox/Data/Platforms.xml")
            .replace(
                "<Developer>Fixture Labs</Developer>",
                "<Developer>Fixture Labs</Developer><FuturePlatformField>keep-platform</FuturePlatformField>",
            )
            .replace(
                "<FolderPath>Videos/Fixture Console</FolderPath>",
                "<FolderPath>Videos/Fixture Console</FolderPath><FutureFolderField>keep-folder</FutureFolderField>",
            );
        let mut document = AuxiliaryDocument::from_reader(
            AuxiliaryDocumentKind::Platforms,
            "Platforms.xml",
            fixture.as_bytes(),
        )
        .unwrap();
        let catalog = document.platform_catalog().unwrap();
        let mut platform = catalog.platforms[0].clone();
        platform.metadata.sort_title = Some("Console, Fixture".into());
        platform.metadata.notes = None;
        platform.metadata.developer = Some("Qt Forge".into());
        platform.metadata.manufacturer = Some("Portable Systems".into());
        platform.metadata.hide_in_big_box = true;
        platform.release_date = Some("2001-02-03".into());
        platform.disable_auto_import = true;
        document
            .set_platform_definition(
                "fixture console",
                platform.clone(),
                vec![
                    IndexedPlatformRecordEdit {
                        source_index: Some(0),
                        record: PlatformFolder {
                            platform: "Fixture Console".into(),
                            media_type: "Video".into(),
                            folder_path: r"Videos\Fixture Console\Edited".into(),
                        },
                    },
                    IndexedPlatformRecordEdit {
                        source_index: None,
                        record: PlatformFolder {
                            platform: "Fixture Console".into(),
                            media_type: "Manual".into(),
                            folder_path: r"Manuals\Fixture Console".into(),
                        },
                    },
                ],
            )
            .unwrap();

        let updated = document.platform_catalog().unwrap();
        assert_eq!(updated.platforms, [platform]);
        assert_eq!(updated.folders.len(), 2);
        assert_eq!(
            updated.folders[0].folder_path,
            r"Videos\Fixture Console\Edited"
        );
        let bytes = document.to_xml_bytes().unwrap();
        let xml = String::from_utf8(bytes).unwrap();
        assert!(xml.contains("<FuturePlatformField>keep-platform</FuturePlatformField>"));
        assert!(xml.contains("<FutureFolderField>keep-folder</FutureFolderField>"));
        assert!(!xml.contains("A platform catalog fixture."));
    }

    #[test]
    fn platform_edit_rejects_identity_changes_and_invalid_folder_indices_atomically() {
        let fixture = include_str!("../../../fixtures/launchbox/Data/Platforms.xml");
        let mut document = AuxiliaryDocument::from_reader(
            AuxiliaryDocumentKind::Platforms,
            "Platforms.xml",
            fixture.as_bytes(),
        )
        .unwrap();
        let catalog = document.platform_catalog().unwrap();
        let mut renamed = catalog.platforms[0].clone();
        renamed.metadata.name = "Renamed Console".into();
        let before = document.to_xml_bytes().unwrap();
        assert!(matches!(
            document.set_platform_definition("Fixture Console", renamed, Vec::new()),
            Err(StorageError::ImmutablePlatformName { .. })
        ));
        assert_eq!(document.to_xml_bytes().unwrap(), before);

        let invalid = IndexedPlatformRecordEdit {
            source_index: Some(catalog.folders.len()),
            record: catalog.folders[0].clone(),
        };
        assert!(matches!(
            document.set_platform_definition(
                "Fixture Console",
                catalog.platforms[0].clone(),
                vec![invalid]
            ),
            Err(StorageError::InvalidPlatformFolderEdit { .. })
        ));
        assert_eq!(document.to_xml_bytes().unwrap(), before);
    }

    #[test]
    fn platform_category_create_delete_and_detach_are_lossless_across_documents() {
        let platform_fixture = include_str!("../../../fixtures/launchbox/Data/Platforms.xml");
        let parent_fixture = include_str!("../../../fixtures/launchbox/Data/Parents.xml");
        let mut catalog = AuxiliaryDocument::from_reader(
            AuxiliaryDocumentKind::Platforms,
            "Platforms.xml",
            platform_fixture.as_bytes(),
        )
        .unwrap();
        let mut parents = AuxiliaryDocument::from_reader(
            AuxiliaryDocumentKind::Parents,
            "Parents.xml",
            parent_fixture.as_bytes(),
        )
        .unwrap();
        let original_catalog = catalog.to_xml_bytes().unwrap();
        let original_parents = parents.to_xml_bytes().unwrap();
        let category = PlatformCategory {
            metadata: NavigationMetadata {
                name: "Portable Systems".into(),
                nested_name: Some("Portable Systems".into()),
                notes: Some("Handheld and transportable systems.".into()),
                ..NavigationMetadata::default()
            },
            ..PlatformCategory::default()
        };
        catalog.add_platform_category(category.clone()).unwrap();
        parents
            .set_platform_category_parents(
                "Portable Systems",
                vec![IndexedPlatformRecordEdit {
                    source_index: None,
                    record: ParentRelationship {
                        platform_category_name: Some("Portable Systems".into()),
                        ..ParentRelationship::default()
                    },
                }],
            )
            .unwrap();
        assert_eq!(
            catalog.platform_catalog().unwrap().categories.last(),
            Some(&category)
        );
        assert!(parents
            .parent_relationships()
            .unwrap()
            .iter()
            .any(
                |relationship| relationship.platform_category_name.as_deref()
                    == Some("Portable Systems")
                    && relationship.parent_platform_category_name.is_none()
                    && relationship.parent_platform_name.is_none()
                    && relationship.parent_playlist_id.is_none()
            ));

        assert_eq!(
            catalog
                .remove_platform_category("portable systems")
                .unwrap(),
            category
        );
        let removed = parents
            .remove_platform_category_relationships("portable systems")
            .unwrap();
        assert_eq!(removed.removed_placements, 1);
        assert_eq!(removed.detached_children, 0);
        assert_eq!(catalog.to_xml_bytes().unwrap(), original_catalog);
        assert_eq!(parents.to_xml_bytes().unwrap(), original_parents);
    }

    #[test]
    fn platform_category_edit_preserves_unknown_rows_and_delete_detaches_children() {
        let platform_fixture = include_str!("../../../fixtures/launchbox/Data/Platforms.xml")
            .replace(
                "<Notes>A fixture category.</Notes>",
                "<Notes>A fixture category.</Notes><FutureCategoryField>keep-category</FutureCategoryField>",
            );
        let parent_fixture = r#"<?xml version="1.0" encoding="utf-8"?>
<LaunchBox>
  <Parent><PlatformCategoryName>Fixture Category</PlatformCategoryName><ParentPlatformName>Fixture Console</ParentPlatformName><FuturePlacement>keep-placement</FuturePlacement></Parent>
  <Parent><ParentPlatformCategoryName>Fixture Category</ParentPlatformCategoryName><PlatformName>Fixture Console</PlatformName><FutureChildPlacement>keep-child</FutureChildPlacement></Parent>
</LaunchBox>"#;
        let mut catalog = AuxiliaryDocument::from_reader(
            AuxiliaryDocumentKind::Platforms,
            "Platforms.xml",
            platform_fixture.as_bytes(),
        )
        .unwrap();
        let mut parents = AuxiliaryDocument::from_reader(
            AuxiliaryDocumentKind::Parents,
            "Parents.xml",
            parent_fixture.as_bytes(),
        )
        .unwrap();
        let mut category = catalog.platform_catalog().unwrap().categories[0].clone();
        category.metadata.nested_name = Some("Fixture Collection".into());
        category.metadata.notes = None;
        category.metadata.sort_title = Some("Collection, Fixture".into());
        category.metadata.video_path = Some(r"Videos\Categories\fixture.mp4".into());
        category.metadata.hide_in_big_box = true;
        catalog
            .set_platform_category("fixture category", category.clone())
            .unwrap();
        parents
            .set_platform_category_parents(
                "Fixture Category",
                vec![
                    IndexedPlatformRecordEdit {
                        source_index: Some(0),
                        record: ParentRelationship {
                            platform_category_name: Some("Fixture Category".into()),
                            ..ParentRelationship::default()
                        },
                    },
                    IndexedPlatformRecordEdit {
                        source_index: None,
                        record: ParentRelationship {
                            platform_category_name: Some("Fixture Category".into()),
                            parent_playlist_id: Some("fixture-playlist".into()),
                            ..ParentRelationship::default()
                        },
                    },
                ],
            )
            .unwrap();
        assert_eq!(catalog.platform_catalog().unwrap().categories[0], category);
        let edited_xml = String::from_utf8(catalog.to_xml_bytes().unwrap()).unwrap();
        assert!(edited_xml.contains("<FutureCategoryField>keep-category</FutureCategoryField>"));
        assert!(!edited_xml.contains("A fixture category."));
        let parent_xml = String::from_utf8(parents.to_xml_bytes().unwrap()).unwrap();
        assert!(parent_xml.contains("<FuturePlacement>keep-placement</FuturePlacement>"));
        assert!(parent_xml.contains("<ParentPlaylistId>fixture-playlist</ParentPlaylistId>"));

        let before = catalog.to_xml_bytes().unwrap();
        let mut renamed = category.clone();
        renamed.metadata.name = "Renamed Category".into();
        assert!(matches!(
            catalog.set_platform_category("Fixture Category", renamed),
            Err(StorageError::ImmutablePlatformCategoryName { .. })
        ));
        assert_eq!(catalog.to_xml_bytes().unwrap(), before);

        catalog
            .remove_platform_category("Fixture Category")
            .unwrap();
        let removed = parents
            .remove_platform_category_relationships("Fixture Category")
            .unwrap();
        assert_eq!(removed.removed_placements, 2);
        assert_eq!(removed.detached_children, 1);
        let remaining = parents.parent_relationships().unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(
            remaining[0].platform_name.as_deref(),
            Some("Fixture Console")
        );
        assert!(remaining[0].parent_platform_category_name.is_none());
        let remaining_xml = String::from_utf8(parents.to_xml_bytes().unwrap()).unwrap();
        assert!(remaining_xml.contains("<FutureChildPlacement>keep-child</FutureChildPlacement>"));
        assert!(!remaining_xml.contains("keep-placement"));
    }

    #[test]
    fn playlist_edit_preserves_unknown_xml_and_replaces_indexed_rows() {
        let fixture = include_str!(
            "../../../fixtures/launchbox/Data/Playlists/Fixture Playlist.xml"
        )
        .replace(
            "<Notes>A deterministic playlist fixture.</Notes>",
            "<Notes>A deterministic playlist fixture.</Notes><FuturePlaylistField>keep-playlist</FuturePlaylistField>",
        )
        .replace(
            "<Value>true</Value>",
            "<Value>true</Value><FutureFilterField>keep-filter</FutureFilterField>",
        )
        .replace(
            "<ManualOrder>1</ManualOrder>",
            "<ManualOrder>1</ManualOrder><FutureGameField>keep-game</FutureGameField>",
        );
        let mut document = AuxiliaryDocument::from_reader(
            AuxiliaryDocumentKind::Playlist,
            "Playlists/Fixture Playlist.xml",
            fixture.as_bytes(),
        )
        .unwrap();
        let original = document.playlist_document().unwrap();
        let mut playlist = original.playlist.clone();
        playlist.metadata.nested_name = Some("Edited Favorites".into());
        playlist.metadata.notes = None;
        playlist.metadata.video_path = Some(r"Videos\Playlists\favorite.mp4".into());
        playlist.metadata.hide_in_big_box = true;
        playlist.auto_populate = false;
        playlist.sort_by = Some("ManualOrder".into());
        document
            .set_playlist(
                "fixture-playlist",
                playlist.clone(),
                vec![IndexedPlatformRecordEdit {
                    source_index: Some(0),
                    record: PlaylistFilter {
                        field_key: "Genre".into(),
                        comparison_type_key: "Contains".into(),
                        value: "Adventure".into(),
                    },
                }],
                vec![
                    IndexedPlatformRecordEdit {
                        source_index: Some(0),
                        record: PlaylistGame {
                            game_title: "Edited Adventure".into(),
                            manual_order: 2,
                            ..original.games[0].clone()
                        },
                    },
                    IndexedPlatformRecordEdit {
                        source_index: None,
                        record: PlaylistGame {
                            game_id: "fixture-racer".into(),
                            game_title: "Fixture Racer".into(),
                            game_platform: "Fixture Console".into(),
                            game_file_name: r"Games\Fixture\racer.rom".into(),
                            launchbox_db_id: Some(4321),
                            manual_order: 3,
                        },
                    },
                ],
            )
            .unwrap();

        let updated = document.playlist_document().unwrap();
        assert_eq!(updated.playlist, playlist);
        assert_eq!(updated.filters[0].field_key, "Genre");
        assert_eq!(updated.games.len(), 2);
        assert_eq!(updated.games[1].game_file_name, r"Games\Fixture\racer.rom");
        let xml = String::from_utf8(document.to_xml_bytes().unwrap()).unwrap();
        assert!(xml.contains("<FuturePlaylistField>keep-playlist</FuturePlaylistField>"));
        assert!(xml.contains("<FutureFilterField>keep-filter</FutureFilterField>"));
        assert!(xml.contains("<FutureGameField>keep-game</FutureGameField>"));
        assert!(!xml.contains("A deterministic playlist fixture."));

        let before = document.to_xml_bytes().unwrap();
        let mut renamed = playlist;
        renamed.metadata.name = "Renamed Unique Name".into();
        assert!(matches!(
            document.set_playlist("fixture-playlist", renamed, Vec::new(), Vec::new()),
            Err(StorageError::ImmutablePlaylistName { .. })
        ));
        assert_eq!(document.to_xml_bytes().unwrap(), before);
    }

    #[test]
    fn playlist_create_and_relationship_cleanup_are_lossless() {
        let path = PathBuf::from("Data/Playlists/Portable List.xml");
        let playlist = Playlist {
            id: "portable-list".into(),
            metadata: NavigationMetadata {
                name: "Portable List".into(),
                video_path: Some(r"Videos\Playlists\portable.mp4".into()),
                ..NavigationMetadata::default()
            },
            ..Playlist::default()
        };
        let created = AuxiliaryDocument::new_playlist(
            &path,
            playlist.clone(),
            Vec::new(),
            vec![PlaylistGame {
                game_id: "fixture-racer".into(),
                game_title: "Fixture Racer".into(),
                game_platform: "Fixture Console".into(),
                game_file_name: r"Games\Fixture\racer.rom".into(),
                manual_order: 1,
                ..PlaylistGame::default()
            }],
        )
        .unwrap();
        assert_eq!(created.playlist_document().unwrap().playlist, playlist);

        let parent_fixture = r#"<?xml version="1.0" encoding="utf-8"?>
<LaunchBox>
  <Parent><PlaylistId>portable-list</PlaylistId><ParentPlatformCategoryName>Fixture Category</ParentPlatformCategoryName><FuturePlacement>drop</FuturePlacement></Parent>
  <Parent><PlaylistId>portable-list</PlaylistId><FutureRootPlacement>drop-root</FutureRootPlacement></Parent>
  <Parent><PlaylistId>child-list</PlaylistId><ParentPlaylistId>portable-list</ParentPlaylistId><FutureChildPlacement>keep-child</FutureChildPlacement></Parent>
</LaunchBox>"#;
        let mut parents = AuxiliaryDocument::from_reader(
            AuxiliaryDocumentKind::Parents,
            "Data/Parents.xml",
            parent_fixture.as_bytes(),
        )
        .unwrap();
        let removed = parents
            .remove_playlist_relationships("PORTABLE-LIST")
            .unwrap();
        assert_eq!(removed.removed_placements, 2);
        assert_eq!(removed.detached_children, 1);
        let remaining = parents.parent_relationships().unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].playlist_id.as_deref(), Some("child-list"));
        assert!(remaining[0].parent_playlist_id.is_none());
        let parent_xml = String::from_utf8(parents.to_xml_bytes().unwrap()).unwrap();
        assert!(parent_xml.contains("<FutureChildPlacement>keep-child</FutureChildPlacement>"));
        assert!(!parent_xml.contains("drop-root"));

        let cache_fixture = r#"<LaunchBox>
  <ListCacheItem><PlaylistId>portable-list</PlaylistId><FutureCache>drop</FutureCache></ListCacheItem>
  <ListCacheItem><PlaylistId>other-list</PlaylistId><FutureCache>keep-cache</FutureCache></ListCacheItem>
</LaunchBox>"#;
        let mut cache = AuxiliaryDocument::from_reader(
            AuxiliaryDocumentKind::ListCache,
            "Data/ListCache.xml",
            cache_fixture.as_bytes(),
        )
        .unwrap();
        assert_eq!(
            cache
                .remove_playlist_list_cache_items("Portable-List")
                .unwrap(),
            1
        );
        let cache_xml = String::from_utf8(cache.to_xml_bytes().unwrap()).unwrap();
        assert!(cache_xml.contains("keep-cache"));
        assert!(!cache_xml.contains("<FutureCache>drop</FutureCache>"));
    }
}
