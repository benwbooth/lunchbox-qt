#[cfg(test)]
use lb_domain::GAME_XML_FIELDS;
use lb_domain::{
    AdditionalApplication, AlternateName, CatalogValidationError, CustomField, Game,
    GameControllerSupport, GameLaunchConfiguration, GameMetadata, GameSave, Mount, PlatformCatalog,
    PlatformDefinition, PlatformFolder, PlatformLibrary, ValidationError,
};
use serde::Deserialize;
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
pub struct NewGame {
    pub id: String,
    pub title: String,
    pub platform: String,
    pub application_path: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemovedPlatformCatalogRecords {
    pub platform: PlatformDefinition,
    pub folders: Vec<PlatformFolder>,
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
    #[serde(rename = "FilePath")]
    file_path: String,
    #[serde(rename = "Slot")]
    slot: Option<i32>,
    #[serde(rename = "Title")]
    title: Option<String>,
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
            file_path: self.file_path,
            slot: self.slot,
            title: non_empty(self.title),
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
        file_path: required_record_child(element, "GameSave", "FilePath")?,
        slot: number_child(element, "Slot"),
        title: optional_child(element, "Title"),
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
    #[error("platform folder owner {actual} does not match platform {expected}")]
    PlatformFolderOwnerMismatch { expected: String, actual: String },
    #[error("platform {platform} has more than one folder for media type {media_type}")]
    DuplicatePlatformFolderMediaType {
        platform: String,
        media_type: String,
    },
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
        assert_eq!(
            reparsed
                .library()
                .games
                .iter()
                .find(|game| game.id == "fixture-added")
                .map(|game| game.application_path.as_str()),
            Some(r"Games\Fixture Added\added.rom")
        );

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
}
