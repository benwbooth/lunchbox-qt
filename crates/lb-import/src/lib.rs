use lb_domain::{
    is_unassigned_emulator_id, AdditionalApplication, EmulatorConfiguration, Game,
    UNASSIGNED_EMULATOR_ID,
};
use lb_metadata::{MetadataDatabase, MetadataError, MetadataGame, MetadataMatchKind};
use lb_platform::{
    portable_storage_name, portable_stored_path, HostPathResolver, LaunchPathError,
    LaunchPathResolver, PlatformPathError,
};
use lb_storage::{
    FileRevision, LaunchBoxDataIndex, LibraryTransaction, NewGame, NewGameMetadata,
    PlatformDocument, StorageError, TransactionError,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportLocationKind {
    File,
    Folder,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ImportLocation {
    pub path: PathBuf,
    pub kind: ImportLocationKind,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportFilePolicy {
    Leave,
    Copy,
    Move,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportDuplicatePolicy {
    Skip,
    Import,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManualImportRequest {
    pub platform: String,
    pub locations: Vec<ImportLocation>,
    #[serde(default)]
    pub recursive: bool,
    #[serde(default)]
    pub use_folder_names: bool,
    pub file_policy: ImportFilePolicy,
    pub duplicate_policy: ImportDuplicatePolicy,
    #[serde(default)]
    pub extensions: Vec<String>,
    /// When copying or moving, include regular files in the same directory
    /// whose filename stem matches the selected ROM but whose extension
    /// differs. This follows LaunchBox's recovered "same name but different
    /// file extensions" option; it does not parse descriptor-file contents.
    #[serde(default)]
    pub copy_files_with_same_name: bool,
    /// When copying or moving, create one portable directory named from the
    /// final game title and, when known, its metadata release year.
    #[serde(default)]
    pub copy_to_subfolders: bool,
    /// Combine only complete, unambiguous filename-derived disc sets. The
    /// planner recognizes `(Disc N)` and `(Disc N of M)` within one folder and
    /// extension; incomplete or colliding sets remain separate games.
    #[serde(default)]
    pub combine_disc_sets: bool,
    /// Combine import rows whose final, metadata-resolved titles match. The
    /// first source in deterministic preview order remains the primary game;
    /// every ROM, including that primary source, is persisted as a selectable
    /// LaunchBox additional application.
    #[serde(default)]
    pub combine_matching_titles: bool,
    /// Search LaunchBox's local SQLite metadata database. Exact primary or
    /// alternate titles win; when none match, use the recovered partial-word
    /// fallback. Only one result is applied automatically, while missing or
    /// ambiguous results remain explicit in the preview.
    #[serde(default)]
    pub search_local_metadata: bool,
    /// Look in each imported game's source directory for a PDF that can be
    /// linked as its manual. An exact ROM-stem match wins; otherwise a sole
    /// PDF is accepted. Ambiguous folders remain explicit in the preview.
    #[serde(default)]
    pub look_for_pdf_manuals: bool,
    /// `None` inherits the platform default. LaunchBox's all-zero sentinel
    /// explicitly selects direct launch; any other value selects a configured
    /// emulator by ID.
    #[serde(default)]
    pub emulator_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportRowState {
    Ready,
    Duplicate,
    DestinationExists,
    UnsupportedExtension,
    InvalidTitle,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ManualImportMetadataMatchKind {
    Exact,
    Partial,
}

impl From<MetadataMatchKind> for ManualImportMetadataMatchKind {
    fn from(kind: MetadataMatchKind) -> Self {
        match kind {
            MetadataMatchKind::Exact => Self::Exact,
            MetadataMatchKind::Partial => Self::Partial,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManualImportPreviewRow {
    pub source_path: PathBuf,
    pub title: String,
    pub extension: String,
    pub destination_path: Option<PathBuf>,
    pub application_path: String,
    pub state: ImportRowState,
    pub included: bool,
    pub message: String,
    pub disc: Option<u32>,
    pub version: Option<String>,
    pub region: Option<String>,
    #[serde(default)]
    pub same_name_files: Vec<ManualImportCompanion>,
    #[serde(default)]
    pub additional_roms: Vec<ManualImportRom>,
    pub metadata: Option<ManualImportMetadata>,
    pub metadata_candidate_count: usize,
    pub metadata_match_kind: Option<ManualImportMetadataMatchKind>,
    #[serde(default)]
    pub metadata_candidates: Vec<ManualImportMetadataCandidate>,
    pub manual: Option<ManualImportManual>,
    pub manual_candidate_count: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManualImportMetadataCandidate {
    pub database_id: u32,
    pub title: String,
    pub platform: String,
    pub release_year: Option<i32>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManualImportManual {
    pub source_path: PathBuf,
    pub stored_path: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManualImportMetadata {
    pub database_id: u32,
    pub notes: Option<String>,
    pub developer: Option<String>,
    pub genre: Option<String>,
    pub max_players: Option<u32>,
    pub play_mode: Option<String>,
    pub publisher: Option<String>,
    pub rating: Option<String>,
    pub release_date: Option<String>,
    pub release_type: Option<String>,
    pub wikipedia_url: Option<String>,
    pub video_url: Option<String>,
    pub community_star_rating: Option<f64>,
}

impl From<ManualImportMetadata> for NewGameMetadata {
    fn from(metadata: ManualImportMetadata) -> Self {
        Self {
            database_id: Some(metadata.database_id),
            manual_path: None,
            notes: metadata.notes,
            developer: metadata.developer,
            genre: metadata.genre,
            max_players: metadata.max_players,
            play_mode: metadata.play_mode,
            publisher: metadata.publisher,
            rating: metadata.rating,
            region: None,
            release_date: metadata.release_date,
            release_type: metadata.release_type,
            status: None,
            version: None,
            wikipedia_url: metadata.wikipedia_url,
            video_url: metadata.video_url,
            community_star_rating: metadata.community_star_rating,
        }
    }
}

impl ManualImportPreviewRow {
    pub fn is_importable(&self, duplicate_policy: ImportDuplicatePolicy) -> bool {
        self.state == ImportRowState::Ready
            || (self.state == ImportRowState::Duplicate
                && duplicate_policy == ImportDuplicatePolicy::Import)
    }

    pub fn file_count(&self) -> usize {
        1usize.saturating_add(self.additional_roms.len())
    }

    pub fn companion_file_count(&self) -> usize {
        self.same_name_files.len()
            + self
                .additional_roms
                .iter()
                .map(|rom| rom.same_name_files.len())
                .sum::<usize>()
    }

    pub fn transfer_file_count(&self) -> usize {
        self.file_count()
            .saturating_add(self.companion_file_count())
    }

    fn import_files(&self) -> impl Iterator<Item = ImportFileRef<'_>> {
        std::iter::once(ImportFileRef {
            source_path: &self.source_path,
            application_path: &self.application_path,
            disc: self.disc,
            version: self.version.as_deref(),
            region: self.region.as_deref(),
            metadata: self.metadata.as_ref(),
        })
        .chain(self.additional_roms.iter().map(|rom| ImportFileRef {
            source_path: &rom.source_path,
            application_path: &rom.application_path,
            disc: rom.disc,
            version: rom.version.as_deref(),
            region: rom.region.as_deref(),
            metadata: rom.metadata.as_ref(),
        }))
    }

    fn transfer_files(&self) -> Vec<TransferFileRef<'_>> {
        let mut files = Vec::with_capacity(self.transfer_file_count());
        files.push(TransferFileRef {
            source_path: &self.source_path,
            destination_path: self.destination_path.as_deref(),
        });
        files.extend(
            self.same_name_files
                .iter()
                .map(ManualImportCompanion::transfer_file),
        );
        for rom in &self.additional_roms {
            files.push(TransferFileRef {
                source_path: &rom.source_path,
                destination_path: rom.destination_path.as_deref(),
            });
            files.extend(
                rom.same_name_files
                    .iter()
                    .map(ManualImportCompanion::transfer_file),
            );
        }
        files
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManualImportCompanion {
    pub source_path: PathBuf,
    pub extension: String,
    pub destination_path: Option<PathBuf>,
}

impl ManualImportCompanion {
    fn transfer_file(&self) -> TransferFileRef<'_> {
        TransferFileRef {
            source_path: &self.source_path,
            destination_path: self.destination_path.as_deref(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManualImportRom {
    pub source_path: PathBuf,
    pub extension: String,
    pub destination_path: Option<PathBuf>,
    pub application_path: String,
    pub disc: Option<u32>,
    pub version: Option<String>,
    pub region: Option<String>,
    #[serde(default)]
    pub same_name_files: Vec<ManualImportCompanion>,
    pub metadata: Option<ManualImportMetadata>,
}

#[derive(Clone, Copy, Debug)]
struct ImportFileRef<'a> {
    source_path: &'a Path,
    application_path: &'a str,
    disc: Option<u32>,
    version: Option<&'a str>,
    region: Option<&'a str>,
    metadata: Option<&'a ManualImportMetadata>,
}

#[derive(Clone, Copy, Debug)]
struct TransferFileRef<'a> {
    source_path: &'a Path,
    destination_path: Option<&'a Path>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManualImportPreview {
    pub request: ManualImportRequest,
    pub rows: Vec<ManualImportPreviewRow>,
    pub importable_count: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManualImportRowSelection {
    pub source_path: PathBuf,
    pub title: String,
    pub included: bool,
    #[serde(default)]
    pub metadata_database_id: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManualImportSelection {
    pub request: ManualImportRequest,
    pub rows: Vec<ManualImportRowSelection>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ManualImportReport {
    pub games: Vec<Game>,
    pub additional_applications: Vec<AdditionalApplication>,
    pub platform_backup: PathBuf,
    pub created_files: Vec<PathBuf>,
    pub moved_sources: Vec<PathBuf>,
    pub cleanup_warnings: Vec<String>,
}

pub fn preview_manual_import(
    launchbox_root: impl AsRef<Path>,
    resolver: &HostPathResolver,
    mut request: ManualImportRequest,
) -> Result<ManualImportPreview, ImportError> {
    let supplied_root = launchbox_root.as_ref();
    let launchbox_root = fs::canonicalize(supplied_root).map_err(|source| ImportError::Io {
        path: supplied_root.to_path_buf(),
        source,
    })?;
    if request.platform.trim().is_empty() {
        return Err(ImportError::EmptyPlatform);
    }
    if request.locations.is_empty() {
        return Err(ImportError::NoLocations);
    }

    let data = LaunchBoxDataIndex::load(&launchbox_root)?;
    let library = data.platforms();
    if !library
        .platforms()
        .iter()
        .any(|platform| platform.name.eq_ignore_ascii_case(request.platform.trim()))
    {
        return Err(ImportError::UnknownPlatform {
            platform: request.platform.clone(),
        });
    }
    request.emulator_id = normalize_emulator_selection(
        data.emulator_configuration(),
        request.emulator_id.as_deref(),
    )?;

    let extensions = normalize_extensions(&request.extensions);
    let sources = collect_sources(&request.locations, request.recursive)?;
    let independent_sources = sources
        .iter()
        .filter(|source| {
            let extension = source
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            extensions.is_empty() || extensions.contains(&extension)
        })
        .cloned()
        .collect::<BTreeSet<_>>();
    let existing_paths = existing_library_paths(
        &launchbox_root,
        resolver,
        library.games(),
        library.additional_applications(),
    );
    let destination_directory = launchbox_root
        .join("Games")
        .join(portable_storage_name(&request.platform)?);
    let mut planned_destinations = BTreeSet::new();
    let mut rows = Vec::with_capacity(sources.len());

    for source in sources {
        let extension = source
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let title = derive_title(&source, request.use_folder_names);
        let version = rom_version(&source, request.use_folder_names);
        let region = rom_region(&source, request.use_folder_names);
        let mut state = ImportRowState::Ready;
        let mut message = "Ready to import".to_string();
        if !extensions.is_empty() && !extensions.contains(&extension) {
            state = ImportRowState::UnsupportedExtension;
            message = if extension.is_empty() {
                "Excluded because the file has no extension".to_string()
            } else {
                format!("Excluded by the extension filter (.{extension})")
            };
        } else if title.trim().is_empty() {
            state = ImportRowState::InvalidTitle;
            message = "A non-empty title is required".to_string();
        } else if existing_paths.contains(&source) {
            state = ImportRowState::Duplicate;
            message = "A library game already resolves to this file".to_string();
        }

        let mut same_name_files = Vec::new();
        let (destination_path, application_path) = match request.file_policy {
            ImportFilePolicy::Leave => (
                None,
                resolver.stored_path_for_host_path(&launchbox_root, &source)?,
            ),
            ImportFilePolicy::Copy | ImportFilePolicy::Move => {
                let file_name = portable_source_file_name(&source)?;
                let destination = destination_directory.join(&file_name);
                let destination_eligible = state == ImportRowState::Ready
                    || (state == ImportRowState::Duplicate
                        && request.duplicate_policy == ImportDuplicatePolicy::Import);
                if destination_eligible {
                    if request.copy_files_with_same_name {
                        for companion in same_name_companions(&source)? {
                            if independent_sources.contains(&companion) {
                                continue;
                            }
                            let companion_name = portable_source_file_name(&companion)?;
                            same_name_files.push(ManualImportCompanion {
                                extension: normalized_extension(&companion),
                                destination_path: Some(destination_directory.join(companion_name)),
                                source_path: companion,
                            });
                        }
                    }

                    if !request.copy_to_subfolders {
                        let mut row_destinations = BTreeSet::new();
                        let companion_destinations = same_name_files.iter().map(|companion| {
                            let destination = companion
                                .destination_path
                                .as_ref()
                                .expect("copy/move companion has a destination");
                            let name = destination
                                .file_name()
                                .and_then(|name| name.to_str())
                                .expect("planned companion destination has a Unicode filename");
                            (name, destination.as_path())
                        });
                        for candidate in
                            std::iter::once((file_name.as_str(), destination.as_path()))
                                .chain(companion_destinations)
                        {
                            let destination_key = candidate.0.to_lowercase();
                            if candidate.1.exists()
                                || case_insensitive_name_exists(
                                    &destination_directory,
                                    candidate.0,
                                )?
                            {
                                state = ImportRowState::DestinationExists;
                                message = format!(
                                    "Destination already exists: {}",
                                    candidate.1.display()
                                );
                                break;
                            }
                            if planned_destinations.contains(&destination_key)
                                || !row_destinations.insert(destination_key)
                            {
                                state = ImportRowState::DestinationExists;
                                message = format!(
                                    "Another selected file would use destination: {}",
                                    candidate.1.display()
                                );
                                break;
                            }
                        }
                        if state != ImportRowState::DestinationExists {
                            planned_destinations.extend(row_destinations);
                        }
                    }
                }
                let relative = Path::new("Games")
                    .join(portable_storage_name(&request.platform)?)
                    .join(file_name);
                (
                    Some(destination),
                    portable_stored_path(&relative).map_err(ImportError::Path)?,
                )
            }
        };

        let included = match state {
            ImportRowState::Ready => true,
            ImportRowState::Duplicate => request.duplicate_policy == ImportDuplicatePolicy::Import,
            ImportRowState::DestinationExists
            | ImportRowState::UnsupportedExtension
            | ImportRowState::InvalidTitle => false,
        };
        rows.push(ManualImportPreviewRow {
            source_path: source,
            title,
            extension,
            destination_path,
            application_path,
            state,
            included,
            message,
            disc: None,
            version,
            region,
            same_name_files,
            additional_roms: Vec::new(),
            metadata: None,
            metadata_candidate_count: 0,
            metadata_match_kind: None,
            metadata_candidates: Vec::new(),
            manual: None,
            manual_candidate_count: 0,
        });
    }
    if request.combine_disc_sets {
        rows = combine_complete_disc_sets(rows, request.use_folder_names, request.duplicate_policy);
    }
    if request.search_local_metadata {
        apply_local_metadata(
            &launchbox_root,
            &request.platform,
            request.duplicate_policy,
            &mut rows,
        )?;
    }
    if request.combine_matching_titles {
        rows = combine_matching_title_rows(rows, request.duplicate_policy);
    }
    if request.copy_to_subfolders
        && matches!(
            request.file_policy,
            ImportFilePolicy::Copy | ImportFilePolicy::Move
        )
    {
        replan_subfolder_destinations(
            &launchbox_root,
            &request.platform,
            request.duplicate_policy,
            &mut rows,
        )?;
    }
    if request.look_for_pdf_manuals {
        apply_pdf_manuals(
            &launchbox_root,
            resolver,
            request.duplicate_policy,
            &mut rows,
        )?;
    }
    let importable_count = rows.iter().filter(|row| row.included).count();
    Ok(ManualImportPreview {
        request,
        rows,
        importable_count,
    })
}

pub fn execute_manual_import(
    launchbox_root: impl AsRef<Path>,
    platform_document: impl AsRef<Path>,
    resolver: &HostPathResolver,
    selection: ManualImportSelection,
) -> Result<ManualImportReport, ImportError> {
    execute_manual_import_with_ids(
        launchbox_root.as_ref(),
        platform_document.as_ref(),
        resolver,
        selection,
        || Uuid::new_v4().to_string(),
    )
}

fn execute_manual_import_with_ids<F>(
    launchbox_root: &Path,
    platform_document: &Path,
    resolver: &HostPathResolver,
    selection: ManualImportSelection,
    mut next_id: F,
) -> Result<ManualImportReport, ImportError>
where
    F: FnMut() -> String,
{
    let mut preview = preview_manual_import(launchbox_root, resolver, selection.request.clone())?;
    let selected = selection
        .rows
        .into_iter()
        .map(|row| (row.source_path.clone(), row))
        .collect::<BTreeMap<_, _>>();
    if selected.len() != preview.rows.len()
        || preview
            .rows
            .iter()
            .any(|row| !selected.contains_key(&row.source_path))
    {
        return Err(ImportError::PreviewChanged);
    }
    apply_selected_local_metadata(
        launchbox_root,
        &preview.request.platform,
        preview.request.search_local_metadata,
        &selected,
        &mut preview.rows,
    )?;
    if preview.request.copy_to_subfolders
        && matches!(
            preview.request.file_policy,
            ImportFilePolicy::Copy | ImportFilePolicy::Move
        )
    {
        for row in &mut preview.rows {
            if let Some(selection) = selected.get(&row.source_path) {
                let title = selection.title.trim();
                if !title.is_empty() {
                    row.title = title.to_string();
                }
            }
        }
        replan_subfolder_destinations(
            launchbox_root,
            &preview.request.platform,
            preview.request.duplicate_policy,
            &mut preview.rows,
        )?;
        if preview.request.look_for_pdf_manuals {
            apply_pdf_manuals(
                launchbox_root,
                resolver,
                preview.request.duplicate_policy,
                &mut preview.rows,
            )?;
        }
    }

    let mut imports = Vec::new();
    for row in &preview.rows {
        let selection = &selected[&row.source_path];
        if !selection.included {
            continue;
        }
        if !row.is_importable(preview.request.duplicate_policy) {
            return Err(ImportError::UnimportableSelection {
                path: row.source_path.clone(),
                state: row.state,
            });
        }
        let title = selection.title.trim();
        if title.is_empty() {
            return Err(ImportError::EmptyTitle {
                path: row.source_path.clone(),
            });
        }
        imports.push((row, title.to_string()));
    }
    if imports.is_empty() {
        return Err(ImportError::EmptySelection);
    }

    let mut document =
        PlatformDocument::load_for_platform(platform_document, &preview.request.platform)?;
    let mut games = Vec::with_capacity(imports.len());
    let mut additional_applications = Vec::new();
    for (row, title) in &imports {
        let game_id = next_id();
        let mut metadata: NewGameMetadata =
            row.metadata.clone().map(Into::into).unwrap_or_default();
        metadata.manual_path = row.manual.as_ref().map(|manual| manual.stored_path.clone());
        metadata.region = row.region.clone();
        metadata.version = row.version.clone();
        metadata.status = Some("Imported ROM".to_string());
        games.push(document.add_game(NewGame {
            id: game_id.clone(),
            title: title.clone(),
            platform: preview.request.platform.clone(),
            application_path: row.application_path.clone(),
            emulator_id: preview.request.emulator_id.clone(),
            metadata,
        })?);
        if row.file_count() > 1 {
            let use_emulator = preview
                .request
                .emulator_id
                .as_deref()
                .is_none_or(|id| !is_unassigned_emulator_id(id));
            for (index, file) in row.import_files().enumerate() {
                let priority_number = index.saturating_add(1);
                let priority = i32::try_from(priority_number).map_err(|_| {
                    ImportError::CombinedRomPriorityTooLarge {
                        priority: priority_number,
                    }
                })?;
                let file_metadata = file.metadata.or(row.metadata.as_ref());
                let application = AdditionalApplication {
                    id: next_id(),
                    game_id: game_id.clone(),
                    name: combined_rom_application_name(file),
                    application_path: file.application_path.to_string(),
                    use_emulator,
                    emulator_id: if use_emulator {
                        preview.request.emulator_id.clone()
                    } else {
                        None
                    },
                    priority,
                    disc: file.disc,
                    version: file.version.map(ToOwned::to_owned),
                    region: file.region.map(ToOwned::to_owned),
                    developer: file_metadata.and_then(|metadata| metadata.developer.clone()),
                    publisher: file_metadata.and_then(|metadata| metadata.publisher.clone()),
                    release_date: file_metadata.and_then(|metadata| metadata.release_date.clone()),
                    status: Some("Imported ROM".to_string()),
                    ..AdditionalApplication::default()
                };
                additional_applications.push(document.add_additional_application(application)?);
            }
        }
    }

    for (row, _) in &imports {
        for file in row.transfer_files() {
            if let Some(destination) = file.destination_path {
                let parent =
                    destination
                        .parent()
                        .ok_or_else(|| ImportError::InvalidSourcePath {
                            path: destination.to_path_buf(),
                        })?;
                fs::create_dir_all(parent).map_err(|source| ImportError::Io {
                    path: parent.to_path_buf(),
                    source,
                })?;
            }
        }
    }

    let launchbox_root = fs::canonicalize(launchbox_root).map_err(|source| ImportError::Io {
        path: launchbox_root.to_path_buf(),
        source,
    })?;
    let platform_target =
        fs::canonicalize(platform_document).map_err(|source| ImportError::Io {
            path: platform_document.to_path_buf(),
            source,
        })?;
    let mut transaction = LibraryTransaction::new(&launchbox_root)?;
    if preview.request.file_policy != ImportFilePolicy::Leave {
        for (row, _) in &imports {
            for file in row.transfer_files() {
                transaction.stage_file_copy(
                    file.source_path,
                    file.destination_path
                        .expect("copy/move preview file has a destination"),
                )?;
            }
        }
    }
    transaction.stage_platform(&document)?;
    let report = transaction.commit()?;
    let platform_backup = report
        .writes
        .into_iter()
        .find(|write| write.target == platform_target)
        .map(|write| write.backup)
        .ok_or(ImportError::MissingPlatformBackup)?;

    let mut moved_sources = Vec::new();
    let mut cleanup_warnings = Vec::new();
    if preview.request.file_policy == ImportFilePolicy::Move {
        for (row, _) in &imports {
            for file in row.transfer_files() {
                let destination = file
                    .destination_path
                    .expect("move preview file has a destination");
                let source_revision = match FileRevision::read(file.source_path) {
                    Ok(revision) => revision,
                    Err(error) => {
                        cleanup_warnings.push(format!(
                            "Imported {}, but could not verify the source before cleanup: {error}",
                            file.source_path.display()
                        ));
                        continue;
                    }
                };
                let destination_revision = match FileRevision::read(destination) {
                    Ok(revision) => revision,
                    Err(error) => {
                        cleanup_warnings.push(format!(
                            "Imported {}, but could not verify its committed destination before cleanup: {error}",
                            file.source_path.display()
                        ));
                        continue;
                    }
                };
                if source_revision != destination_revision {
                    cleanup_warnings.push(format!(
                        "Kept {} because the committed copy did not match it",
                        file.source_path.display()
                    ));
                    continue;
                }
                match fs::remove_file(file.source_path) {
                    Ok(()) => moved_sources.push(file.source_path.to_path_buf()),
                    Err(error) => cleanup_warnings.push(format!(
                        "Imported {}, but could not remove the source: {error}",
                        file.source_path.display()
                    )),
                }
            }
        }
    }

    Ok(ManualImportReport {
        games,
        additional_applications,
        platform_backup,
        created_files: report.created_targets,
        moved_sources,
        cleanup_warnings,
    })
}

fn normalize_extensions(extensions: &[String]) -> BTreeSet<String> {
    extensions
        .iter()
        .map(|extension| {
            extension
                .trim()
                .trim_start_matches("*.")
                .trim_start_matches('.')
                .to_ascii_lowercase()
        })
        .filter(|extension| !extension.is_empty())
        .collect()
}

fn combined_rom_application_name(file: ImportFileRef<'_>) -> String {
    match (
        file.version.filter(|version| !version.trim().is_empty()),
        file.disc,
    ) {
        (Some(version), Some(disc)) => format!("Play {version} Disc {disc}..."),
        (Some(version), None) => format!("Play {version} Version..."),
        (None, Some(disc)) => format!("Play Disc {disc}"),
        (None, None) => {
            let label = file
                .source_path
                .file_stem()
                .or_else(|| file.source_path.file_name())
                .and_then(|value| value.to_str())
                .unwrap_or("ROM");
            format!("Play {label} Version...")
        }
    }
}

fn normalize_emulator_selection(
    configuration: Option<&EmulatorConfiguration>,
    selected_id: Option<&str>,
) -> Result<Option<String>, ImportError> {
    let Some(selected_id) = selected_id else {
        return Ok(None);
    };
    let selected_id = selected_id.trim();
    if selected_id.is_empty() {
        return Err(ImportError::EmptyEmulatorId);
    }
    if is_unassigned_emulator_id(selected_id) {
        return Ok(Some(UNASSIGNED_EMULATOR_ID.to_string()));
    }

    let matches = configuration
        .into_iter()
        .flat_map(|configuration| &configuration.emulators)
        .filter(|emulator| emulator.id.eq_ignore_ascii_case(selected_id))
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [emulator] => Ok(Some(emulator.id.clone())),
        [] => Err(ImportError::UnknownEmulator {
            id: selected_id.to_string(),
        }),
        _ => Err(ImportError::AmbiguousEmulator {
            id: selected_id.to_string(),
            count: matches.len(),
        }),
    }
}

#[derive(Clone, Debug)]
struct DiscDescriptor {
    number: u32,
    total: Option<u32>,
    base_title: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum MatchingTitleKey {
    Database(u32),
    Title(String),
}

fn combine_complete_disc_sets(
    mut rows: Vec<ManualImportPreviewRow>,
    use_folder_names: bool,
    duplicate_policy: ImportDuplicatePolicy,
) -> Vec<ManualImportPreviewRow> {
    let mut candidates = BTreeMap::<(PathBuf, String, String), Vec<(usize, DiscDescriptor)>>::new();
    for (index, row) in rows.iter().enumerate() {
        if !row.is_importable(duplicate_policy) || !row.included {
            continue;
        }
        let Some(descriptor) = disc_descriptor(&row.source_path) else {
            continue;
        };
        let parent = row
            .source_path
            .parent()
            .unwrap_or(Path::new(""))
            .to_path_buf();
        candidates
            .entry((
                parent,
                descriptor.base_title.to_lowercase(),
                row.extension.clone(),
            ))
            .or_default()
            .push((index, descriptor));
    }

    let mut removed = BTreeSet::new();
    for candidates in candidates.values_mut() {
        candidates.sort_by_key(|(_, descriptor)| descriptor.number);
        if !is_complete_disc_set(candidates) {
            continue;
        }
        let primary_index = candidates[0].0;
        let any_duplicate = candidates
            .iter()
            .any(|(index, _)| rows[*index].state == ImportRowState::Duplicate);
        let additional_roms = candidates
            .iter()
            .skip(1)
            .map(|(index, descriptor)| {
                let row = &rows[*index];
                removed.insert(*index);
                ManualImportRom {
                    source_path: row.source_path.clone(),
                    extension: row.extension.clone(),
                    destination_path: row.destination_path.clone(),
                    application_path: row.application_path.clone(),
                    disc: Some(descriptor.number),
                    version: row.version.clone(),
                    region: row.region.clone(),
                    same_name_files: row.same_name_files.clone(),
                    metadata: row.metadata.clone(),
                }
            })
            .collect::<Vec<_>>();
        let primary = &mut rows[primary_index];
        primary.disc = Some(1);
        primary.additional_roms = additional_roms;
        if !use_folder_names {
            primary.title = candidates[0].1.base_title.clone();
        }
        primary.state = if any_duplicate {
            ImportRowState::Duplicate
        } else {
            ImportRowState::Ready
        };
        primary.message = if any_duplicate {
            format!(
                "Disc set includes a referenced file; import as one {}-disc game",
                primary.file_count()
            )
        } else {
            format!("Ready to import as one {}-disc game", primary.file_count())
        };
    }

    rows.into_iter()
        .enumerate()
        .filter_map(|(index, row)| (!removed.contains(&index)).then_some(row))
        .collect()
}

fn combine_matching_title_rows(
    mut rows: Vec<ManualImportPreviewRow>,
    duplicate_policy: ImportDuplicatePolicy,
) -> Vec<ManualImportPreviewRow> {
    let mut candidates = BTreeMap::<MatchingTitleKey, Vec<usize>>::new();
    for (index, row) in rows.iter().enumerate() {
        if !row.is_importable(duplicate_policy) || !row.included || row.metadata_candidate_count > 1
        {
            continue;
        }
        let key = match row.metadata.as_ref() {
            Some(metadata) => MatchingTitleKey::Database(metadata.database_id),
            None => {
                let title = row
                    .title
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .to_lowercase();
                if title.is_empty() {
                    continue;
                }
                MatchingTitleKey::Title(title)
            }
        };
        candidates.entry(key).or_default().push(index);
    }

    let mut removed = BTreeSet::new();
    for indices in candidates.values().filter(|indices| indices.len() > 1) {
        let primary_index = indices[0];
        let any_duplicate = indices
            .iter()
            .any(|index| rows[*index].state == ImportRowState::Duplicate);
        let mut additional_roms = Vec::new();
        for index in indices.iter().copied().skip(1) {
            let secondary = &rows[index];
            removed.insert(index);
            additional_roms.push(ManualImportRom {
                source_path: secondary.source_path.clone(),
                extension: secondary.extension.clone(),
                destination_path: secondary.destination_path.clone(),
                application_path: secondary.application_path.clone(),
                disc: secondary.disc,
                version: secondary.version.clone(),
                region: secondary.region.clone(),
                same_name_files: secondary.same_name_files.clone(),
                metadata: secondary.metadata.clone(),
            });
            additional_roms.extend(secondary.additional_roms.iter().cloned().map(|mut rom| {
                if rom.metadata.is_none() {
                    rom.metadata.clone_from(&secondary.metadata);
                }
                rom
            }));
        }

        let primary = &mut rows[primary_index];
        primary.additional_roms.extend(additional_roms);
        primary.state = if any_duplicate {
            ImportRowState::Duplicate
        } else {
            ImportRowState::Ready
        };
        primary.included = primary.is_importable(duplicate_policy);
        primary.message.push_str(&format!(
            "; combined {} matching-title ROMs as selectable versions",
            primary.file_count()
        ));
    }

    rows.into_iter()
        .enumerate()
        .filter_map(|(index, row)| (!removed.contains(&index)).then_some(row))
        .collect()
}

fn apply_local_metadata(
    launchbox_root: &Path,
    platform: &str,
    duplicate_policy: ImportDuplicatePolicy,
    rows: &mut [ManualImportPreviewRow],
) -> Result<(), ImportError> {
    let database_path = launchbox_root
        .join("Metadata")
        .join("LaunchBox.Metadata.db");
    let database = MetadataDatabase::open(database_path)?;
    for row in rows.iter_mut() {
        if !row.included || !row.is_importable(duplicate_policy) {
            continue;
        }
        let search = database.search(platform, &row.title, Some(&row.source_path))?;
        row.metadata_match_kind = search.kind.map(Into::into);
        let matches = search.games;
        row.metadata_candidate_count = matches.len();
        row.metadata_candidates = matches
            .iter()
            .map(manual_import_metadata_candidate)
            .collect::<Result<_, _>>()?;
        match matches.as_slice() {
            [game] => {
                row.title = game.name.clone();
                row.metadata = Some(manual_import_metadata(game)?);
                let kind = metadata_match_label(row.metadata_match_kind);
                row.message
                    .push_str(&format!("; unique {kind} local metadata match"));
            }
            [] => row.message.push_str("; no local metadata match"),
            candidates => {
                let kind = metadata_match_label(row.metadata_match_kind);
                row.message.push_str(&format!(
                    "; {} {kind} local metadata matches require review",
                    candidates.len()
                ));
            }
        }
    }
    Ok(())
}

fn apply_selected_local_metadata(
    launchbox_root: &Path,
    platform: &str,
    search_local_metadata: bool,
    selections: &BTreeMap<PathBuf, ManualImportRowSelection>,
    rows: &mut [ManualImportPreviewRow],
) -> Result<(), ImportError> {
    let selected_count = selections
        .values()
        .filter(|selection| selection.metadata_database_id.is_some())
        .count();
    if selected_count == 0 {
        return Ok(());
    }
    if !search_local_metadata {
        let selection = selections
            .values()
            .find(|selection| selection.metadata_database_id.is_some())
            .expect("selected_count proves a metadata selection exists");
        return Err(ImportError::InvalidMetadataSelection {
            path: selection.source_path.clone(),
            database_id: selection
                .metadata_database_id
                .expect("selection was filtered for a database ID"),
        });
    }

    let database_path = launchbox_root
        .join("Metadata")
        .join("LaunchBox.Metadata.db");
    let database = MetadataDatabase::open(database_path)?;
    for row in rows {
        let selection = &selections[&row.source_path];
        let Some(database_id) = selection.metadata_database_id else {
            continue;
        };
        let search = database.search(platform, &row.title, Some(&row.source_path))?;
        let Some(game) = search
            .games
            .iter()
            .find(|game| u32::try_from(game.database_id) == Ok(database_id))
        else {
            return Err(ImportError::InvalidMetadataSelection {
                path: row.source_path.clone(),
                database_id,
            });
        };
        row.metadata = Some(manual_import_metadata(game)?);
        if row.metadata_candidate_count > 1 {
            row.message
                .push_str(&format!("; selected local metadata game {database_id}"));
        }
    }
    Ok(())
}

fn metadata_match_label(kind: Option<ManualImportMetadataMatchKind>) -> &'static str {
    match kind {
        Some(ManualImportMetadataMatchKind::Exact) => "exact",
        Some(ManualImportMetadataMatchKind::Partial) => "partial",
        None => "unknown",
    }
}

fn apply_pdf_manuals(
    launchbox_root: &Path,
    resolver: &HostPathResolver,
    duplicate_policy: ImportDuplicatePolicy,
    rows: &mut [ManualImportPreviewRow],
) -> Result<(), ImportError> {
    for row in rows {
        if !row.included || !row.is_importable(duplicate_policy) {
            continue;
        }
        let candidates = pdf_manual_candidates(row)?;
        row.manual_candidate_count = candidates.len();
        let matching_stems = std::iter::once(row.source_path.as_path())
            .chain(
                row.additional_roms
                    .iter()
                    .map(|rom| rom.source_path.as_path()),
            )
            .filter_map(|path| {
                path.file_stem()
                    .and_then(|stem| stem.to_str())
                    .map(str::to_ascii_lowercase)
            })
            .collect::<BTreeSet<_>>();
        let exact = candidates
            .iter()
            .filter(|candidate| {
                candidate
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .is_some_and(|stem| matching_stems.contains(&stem.to_ascii_lowercase()))
            })
            .collect::<Vec<_>>();
        let (manual_source, exact_match) = match (exact.as_slice(), candidates.as_slice()) {
            ([candidate], _) => ((*candidate).clone(), true),
            ([], [candidate]) => (candidate.clone(), false),
            _ => {
                if candidates.is_empty() {
                    row.message
                        .push_str("; no PDF manual found in the game folder");
                } else {
                    row.message.push_str(&format!(
                        "; {} PDF manual candidates require review",
                        candidates.len()
                    ));
                }
                continue;
            }
        };
        let linked_host_path = row
            .transfer_files()
            .into_iter()
            .find(|file| file.source_path == manual_source)
            .and_then(|file| file.destination_path)
            .unwrap_or(&manual_source);
        let stored_path = resolver.stored_path_for_host_path(launchbox_root, linked_host_path)?;
        row.manual = Some(ManualImportManual {
            source_path: manual_source,
            stored_path,
        });
        if exact_match {
            row.message.push_str("; linked same-name PDF manual");
        } else {
            row.message
                .push_str("; linked the game folder's sole PDF manual");
        }
    }
    Ok(())
}

fn pdf_manual_candidates(row: &ManualImportPreviewRow) -> Result<Vec<PathBuf>, ImportError> {
    let Some(parent) = row.source_path.parent() else {
        return Ok(Vec::new());
    };
    let mut candidates = BTreeSet::new();
    let entries = fs::read_dir(parent)
        .map_err(|source| ImportError::Io {
            path: parent.to_path_buf(),
            source,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| ImportError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type().map_err(|source| ImportError::Io {
            path: path.clone(),
            source,
        })?;
        if file_type.is_file()
            && path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"))
        {
            let candidate = canonical_regular_file(&path)?;
            if candidate != row.source_path {
                candidates.insert(candidate);
            }
        }
    }
    Ok(candidates.into_iter().collect())
}

fn replan_subfolder_destinations(
    launchbox_root: &Path,
    platform: &str,
    duplicate_policy: ImportDuplicatePolicy,
    rows: &mut [ManualImportPreviewRow],
) -> Result<(), ImportError> {
    let platform_directory = launchbox_root
        .join("Games")
        .join(portable_storage_name(platform)?);
    let mut planned_destinations = BTreeSet::new();
    for row in rows {
        if !row.included || !row.is_importable(duplicate_policy) {
            continue;
        }
        let folder_name = portable_storage_name(&game_subfolder_name(row))?;
        let destination_directory = platform_directory.join(&folder_name);
        if (destination_directory.exists() && !destination_directory.is_dir())
            || (!destination_directory.exists()
                && case_insensitive_name_exists(&platform_directory, &folder_name)?)
        {
            row.state = ImportRowState::DestinationExists;
            row.included = false;
            row.message = format!(
                "Portable subfolder name collides with an existing entry: {}",
                destination_directory.display()
            );
            continue;
        }
        assign_subfolder_destination(
            &row.source_path,
            &destination_directory,
            platform,
            &folder_name,
            &mut row.destination_path,
            &mut row.application_path,
        )?;
        for companion in &mut row.same_name_files {
            assign_companion_subfolder_destination(companion, &destination_directory)?;
        }
        for rom in &mut row.additional_roms {
            assign_subfolder_destination(
                &rom.source_path,
                &destination_directory,
                platform,
                &folder_name,
                &mut rom.destination_path,
                &mut rom.application_path,
            )?;
            for companion in &mut rom.same_name_files {
                assign_companion_subfolder_destination(companion, &destination_directory)?;
            }
        }

        let mut row_destinations = BTreeSet::new();
        let destinations = row
            .transfer_files()
            .into_iter()
            .map(|file| {
                file.destination_path
                    .expect("copy/move subfolder transfer has a destination")
                    .to_path_buf()
            })
            .collect::<Vec<_>>();
        for destination in destinations {
            let file_name = destination
                .file_name()
                .and_then(|name| name.to_str())
                .expect("planned import destination has a Unicode filename");
            let parent = destination
                .parent()
                .expect("planned import destination has a parent");
            let key = destination.to_string_lossy().to_lowercase();
            if destination.exists() || case_insensitive_name_exists(parent, file_name)? {
                row.state = ImportRowState::DestinationExists;
                row.included = false;
                row.message = format!("Destination already exists: {}", destination.display());
                break;
            }
            if planned_destinations.contains(&key) || !row_destinations.insert(key) {
                row.state = ImportRowState::DestinationExists;
                row.included = false;
                row.message = format!(
                    "Another selected file would use destination: {}",
                    destination.display()
                );
                break;
            }
        }
        if row.state != ImportRowState::DestinationExists {
            planned_destinations.extend(row_destinations);
        }
    }
    Ok(())
}

fn assign_subfolder_destination(
    source_path: &Path,
    destination_directory: &Path,
    platform: &str,
    folder_name: &str,
    destination_path: &mut Option<PathBuf>,
    application_path: &mut String,
) -> Result<(), ImportError> {
    let file_name = portable_source_file_name(source_path)?;
    *destination_path = Some(destination_directory.join(&file_name));
    let relative = Path::new("Games")
        .join(portable_storage_name(platform)?)
        .join(folder_name)
        .join(file_name);
    *application_path = portable_stored_path(&relative).map_err(ImportError::Path)?;
    Ok(())
}

fn assign_companion_subfolder_destination(
    companion: &mut ManualImportCompanion,
    destination_directory: &Path,
) -> Result<(), ImportError> {
    let file_name = portable_source_file_name(&companion.source_path)?;
    companion.destination_path = Some(destination_directory.join(file_name));
    Ok(())
}

fn game_subfolder_name(row: &ManualImportPreviewRow) -> String {
    let year = row
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.release_date.as_deref())
        .and_then(|date| date.get(..4))
        .filter(|year| year.bytes().all(|byte| byte.is_ascii_digit()))
        .filter(|year| *year != "0000");
    match year {
        Some(year) => format!("{} ({year})", row.title.trim()),
        None => row.title.trim().to_string(),
    }
}

fn manual_import_metadata(game: &MetadataGame) -> Result<ManualImportMetadata, ImportError> {
    let database_id = metadata_database_id(game)?;
    let max_players = game
        .max_players
        .map(|value| {
            u32::try_from(value).map_err(|_| ImportError::MetadataValueOutOfRange {
                database_id: game.database_id,
                field: "MaxPlayers",
                value: value.to_string(),
            })
        })
        .transpose()?;
    let community_star_rating = game
        .community_rating
        .map(|value| {
            if value.is_finite() && (0.0..=5.0).contains(&value) {
                Ok(value)
            } else {
                Err(ImportError::MetadataValueOutOfRange {
                    database_id: game.database_id,
                    field: "CommunityRating",
                    value: value.to_string(),
                })
            }
        })
        .transpose()?;
    let play_mode = if game.cooperative {
        Some("Cooperative; Multiplayer".to_string())
    } else {
        match max_players {
            Some(1) => Some("Single Player".to_string()),
            Some(value) if value > 1 => Some("Multiplayer".to_string()),
            _ => None,
        }
    };
    Ok(ManualImportMetadata {
        database_id,
        notes: non_empty_metadata_value(game.overview.as_deref()),
        developer: non_empty_metadata_value(game.developer.as_deref()),
        genre: non_empty_metadata_value(Some(&game.genres)),
        max_players,
        play_mode,
        publisher: non_empty_metadata_value(game.publisher.as_deref()),
        rating: non_empty_metadata_value(game.esrb.as_deref()),
        release_date: metadata_release_date(game),
        release_type: non_empty_metadata_value(game.release_type.as_deref()),
        wikipedia_url: non_empty_metadata_value(game.wikipedia_url.as_deref()),
        video_url: non_empty_metadata_value(game.video_url.as_deref()),
        community_star_rating,
    })
}

fn manual_import_metadata_candidate(
    game: &MetadataGame,
) -> Result<ManualImportMetadataCandidate, ImportError> {
    Ok(ManualImportMetadataCandidate {
        database_id: metadata_database_id(game)?,
        title: game.name.clone(),
        platform: game.platform.clone(),
        release_year: game.release_year,
        developer: non_empty_metadata_value(game.developer.as_deref()),
        publisher: non_empty_metadata_value(game.publisher.as_deref()),
    })
}

fn metadata_database_id(game: &MetadataGame) -> Result<u32, ImportError> {
    u32::try_from(game.database_id).map_err(|_| ImportError::MetadataValueOutOfRange {
        database_id: game.database_id,
        field: "DatabaseID",
        value: game.database_id.to_string(),
    })
}

fn non_empty_metadata_value(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn metadata_release_date(game: &MetadataGame) -> Option<String> {
    if let Some(value) = game
        .release_date
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let prefix = value.get(..10);
        if prefix.is_some_and(is_iso_date_prefix) {
            return prefix.map(ToOwned::to_owned);
        }
        return Some(value.to_string());
    }
    game.release_year
        .filter(|year| (1..=9999).contains(year))
        .map(|year| format!("{year:04}-01-01"))
}

fn is_iso_date_prefix(value: &str) -> bool {
    value.len() == 10
        && value.as_bytes()[4] == b'-'
        && value.as_bytes()[7] == b'-'
        && value
            .bytes()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
}

fn is_complete_disc_set(candidates: &[(usize, DiscDescriptor)]) -> bool {
    if candidates.len() < 2 {
        return false;
    }
    for (expected, (_, descriptor)) in (1u32..).zip(candidates) {
        if descriptor.number != expected {
            return false;
        }
    }
    let totals = candidates
        .iter()
        .map(|(_, descriptor)| descriptor.total)
        .collect::<Vec<_>>();
    if totals.iter().all(Option::is_none) {
        return true;
    }
    let expected_total = u32::try_from(candidates.len()).ok();
    totals
        .iter()
        .all(|total| total.is_some() && *total == expected_total)
}

fn disc_descriptor(path: &Path) -> Option<DiscDescriptor> {
    let stem = path.file_stem()?.to_str()?;
    let lower = stem.to_ascii_lowercase();
    for (start, _) in lower.rmatch_indices("(disc ") {
        let suffix = &lower[start + "(disc ".len()..];
        let Some(close) = suffix.find(')') else {
            continue;
        };
        let tokens = suffix[..close].split_whitespace().collect::<Vec<_>>();
        let (number, total) = match tokens.as_slice() {
            [number] => {
                let Ok(number) = number.parse::<u32>() else {
                    continue;
                };
                (number, None)
            }
            [number, "of", total] => {
                let (Ok(number), Ok(total)) = (number.parse::<u32>(), total.parse::<u32>()) else {
                    continue;
                };
                (number, Some(total))
            }
            _ => continue,
        };
        if number == 0 || total.is_some_and(|total| total < number) {
            continue;
        }
        let marker_end = start + "(disc ".len() + close + 1;
        let joined = format!("{} {}", &stem[..start], &stem[marker_end..]);
        let base_title = clean_rom_title(&joined);
        if !base_title.is_empty() {
            return Some(DiscDescriptor {
                number,
                total,
                base_title,
            });
        }
    }
    None
}

fn portable_source_file_name(path: &Path) -> Result<String, ImportError> {
    let file_name = path
        .file_name()
        .ok_or_else(|| ImportError::InvalidSourcePath {
            path: path.to_path_buf(),
        })?
        .to_str()
        .ok_or_else(|| LaunchPathError::NonUnicodeHostPath {
            path: path.to_path_buf(),
        })?;
    portable_storage_name(file_name).map_err(ImportError::PlatformPath)
}

fn normalized_extension(path: &Path) -> String {
    path.extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
}

fn same_name_companions(source: &Path) -> Result<Vec<PathBuf>, ImportError> {
    let Some(parent) = source.parent() else {
        return Ok(Vec::new());
    };
    let Some(source_stem) = source.file_stem().and_then(|stem| stem.to_str()) else {
        return Ok(Vec::new());
    };
    let Some(source_extension) = source.extension().and_then(|extension| extension.to_str()) else {
        return Ok(Vec::new());
    };

    let mut companions = BTreeSet::new();
    let entries = fs::read_dir(parent)
        .map_err(|source| ImportError::Io {
            path: parent.to_path_buf(),
            source,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| ImportError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type().map_err(|source| ImportError::Io {
            path: path.clone(),
            source,
        })?;
        if !file_type.is_file() {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
            continue;
        };
        if stem.eq_ignore_ascii_case(source_stem)
            && !extension.eq_ignore_ascii_case(source_extension)
        {
            companions.insert(canonical_regular_file(&path)?);
        }
    }
    Ok(companions.into_iter().collect())
}

fn collect_sources(
    locations: &[ImportLocation],
    recursive: bool,
) -> Result<Vec<PathBuf>, ImportError> {
    let mut sources = BTreeSet::new();
    for location in locations {
        match location.kind {
            ImportLocationKind::File => {
                let path = canonical_regular_file(&location.path)?;
                sources.insert(path);
            }
            ImportLocationKind::Folder => {
                let folder =
                    fs::canonicalize(&location.path).map_err(|source| ImportError::Io {
                        path: location.path.clone(),
                        source,
                    })?;
                if !folder.is_dir() {
                    return Err(ImportError::LocationNotFolder { path: folder });
                }
                collect_folder_files(&folder, recursive, &mut sources)?;
            }
        }
    }
    Ok(sources.into_iter().collect())
}

fn collect_folder_files(
    folder: &Path,
    recursive: bool,
    sources: &mut BTreeSet<PathBuf>,
) -> Result<(), ImportError> {
    let mut entries = fs::read_dir(folder)
        .map_err(|source| ImportError::Io {
            path: folder.to_path_buf(),
            source,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| ImportError::Io {
            path: folder.to_path_buf(),
            source,
        })?;
    entries.sort_by_key(fs::DirEntry::path);
    for entry in entries {
        let file_type = entry.file_type().map_err(|source| ImportError::Io {
            path: entry.path(),
            source,
        })?;
        if file_type.is_file() {
            sources.insert(canonical_regular_file(&entry.path())?);
        } else if recursive && file_type.is_dir() {
            collect_folder_files(&entry.path(), true, sources)?;
        }
    }
    Ok(())
}

fn canonical_regular_file(path: &Path) -> Result<PathBuf, ImportError> {
    let canonical = fs::canonicalize(path).map_err(|source| ImportError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if !canonical.is_file() {
        return Err(ImportError::LocationNotFile { path: canonical });
    }
    Ok(canonical)
}

fn derive_title(path: &Path, use_folder_names: bool) -> String {
    import_name(path, use_folder_names)
        .map(clean_rom_title)
        .unwrap_or_default()
}

fn rom_version(path: &Path, use_folder_names: bool) -> Option<String> {
    let name = import_name(path, use_folder_names)?;
    let version = delimited_qualifiers(name)
        .into_iter()
        .filter(|qualifier| !is_disc_qualifier(qualifier))
        .filter(|qualifier| !qualifier_body(qualifier).eq_ignore_ascii_case("bios"))
        .collect::<Vec<_>>()
        .join(" ");
    (!version.is_empty()).then_some(version)
}

fn rom_region(path: &Path, use_folder_names: bool) -> Option<String> {
    let name = import_name(path, use_folder_names)?;
    let mut regions = Vec::new();
    for qualifier in delimited_qualifiers(name) {
        for token in qualifier_body(&qualifier).split(',') {
            let Some(region) = canonical_rom_region(token.trim()) else {
                continue;
            };
            if !regions
                .iter()
                .any(|existing: &String| existing.eq_ignore_ascii_case(region))
            {
                regions.push(region.to_string());
            }
        }
    }
    (!regions.is_empty()).then(|| regions.join(", "))
}

fn import_name(path: &Path, use_folder_names: bool) -> Option<&str> {
    let lexical = path.to_str()?;
    let value = if use_folder_names {
        let file_start = lexical.rfind(['/', '\\'])?;
        let parent = lexical[..file_start].trim_end_matches(['/', '\\']);
        parent
            .rfind(['/', '\\'])
            .map_or(parent, |start| &parent[start + 1..])
    } else {
        let file_name = lexical
            .rsplit(['/', '\\'])
            .next()
            .filter(|value| !value.is_empty())?;
        file_name
            .rfind('.')
            .filter(|index| *index > 0)
            .map_or(file_name, |index| &file_name[..index])
    };
    let value = value.trim();
    let value = value.strip_suffix('.').unwrap_or(value).trim();
    (!value.is_empty()).then_some(value)
}

fn clean_rom_title(value: &str) -> String {
    let mut title = value.to_string();
    for qualifier in delimited_qualifiers(value) {
        title = title.replacen(&qualifier, " ", 1);
    }
    title
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_matches(['-', '_'])
        .trim()
        .to_string()
}

fn delimited_qualifiers(value: &str) -> Vec<String> {
    let mut qualifiers = Vec::new();
    let mut open = None;
    for (index, character) in value.char_indices() {
        match (open, character) {
            (None, '(') => open = Some((index, ')')),
            (None, '[') => open = Some((index, ']')),
            (Some((start, expected)), closing) if closing == expected => {
                qualifiers.push(value[start..index + closing.len_utf8()].to_string());
                open = None;
            }
            _ => {}
        }
    }
    qualifiers
}

fn qualifier_body(qualifier: &str) -> &str {
    qualifier
        .strip_prefix(['(', '['])
        .and_then(|value| value.strip_suffix([')', ']']))
        .unwrap_or(qualifier)
        .trim()
}

fn is_disc_qualifier(qualifier: &str) -> bool {
    let body = qualifier_body(qualifier).to_ascii_lowercase();
    ["disc ", "disk ", "cd "]
        .iter()
        .any(|prefix| body.strip_prefix(prefix).is_some_and(starts_with_number))
        || matches!(body.as_str(), "side a" | "side b")
}

fn starts_with_number(value: &str) -> bool {
    value
        .trim_start()
        .bytes()
        .next()
        .is_some_and(|byte| byte.is_ascii_digit())
}

fn canonical_rom_region(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "usa" | "u.s.a." | "us" | "north america" | "united states" => Some("North America"),
        "europe" | "eur" => Some("Europe"),
        "japan" | "jpn" => Some("Japan"),
        "world" => Some("World"),
        "asia" => Some("Asia"),
        "australia" => Some("Australia"),
        "brazil" => Some("Brazil"),
        "canada" => Some("Canada"),
        "china" => Some("China"),
        "finland" => Some("Finland"),
        "france" => Some("France"),
        "germany" => Some("Germany"),
        "greece" => Some("Greece"),
        "holland" => Some("Holland"),
        "hong kong" => Some("Hong Kong"),
        "italy" => Some("Italy"),
        "korea" => Some("Korea"),
        "netherlands" | "the netherlands" => Some("The Netherlands"),
        "norway" => Some("Norway"),
        "oceania" => Some("Oceania"),
        "russia" => Some("Russia"),
        "south america" => Some("South America"),
        "spain" => Some("Spain"),
        "sweden" => Some("Sweden"),
        "thailand" => Some("Thailand"),
        "uk" | "united kingdom" => Some("United Kingdom"),
        _ => None,
    }
}

fn existing_library_paths<'a>(
    launchbox_root: &Path,
    resolver: &HostPathResolver,
    games: impl Iterator<Item = &'a Game>,
    additional_applications: impl Iterator<Item = &'a AdditionalApplication>,
) -> BTreeSet<PathBuf> {
    games
        .map(|game| game.application_path.as_str())
        .chain(additional_applications.map(|application| application.application_path.as_str()))
        .filter_map(|application_path| resolver.resolve(launchbox_root, application_path).ok())
        .filter_map(|path| fs::canonicalize(path).ok())
        .collect()
}

fn case_insensitive_name_exists(directory: &Path, candidate: &str) -> Result<bool, ImportError> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(source) => {
            return Err(ImportError::Io {
                path: directory.to_path_buf(),
                source,
            })
        }
    };
    for entry in entries {
        let entry = entry.map_err(|source| ImportError::Io {
            path: directory.to_path_buf(),
            source,
        })?;
        if entry
            .file_name()
            .to_str()
            .is_some_and(|name| name.eq_ignore_ascii_case(candidate))
        {
            return Ok(true);
        }
    }
    Ok(false)
}

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("a platform is required")]
    EmptyPlatform,
    #[error("at least one file or folder location is required")]
    NoLocations,
    #[error("platform is not present in the loaded library: {platform}")]
    UnknownPlatform { platform: String },
    #[error("an emulator selection cannot be empty; omit it to use the platform default")]
    EmptyEmulatorId,
    #[error("emulator is not present in the loaded LaunchBox configuration: {id}")]
    UnknownEmulator { id: String },
    #[error("emulator ID {id} appears {count} times in the LaunchBox configuration")]
    AmbiguousEmulator { id: String, count: usize },
    #[error("combined ROM priority {priority} cannot be represented by LaunchBox")]
    CombinedRomPriorityTooLarge { priority: usize },
    #[error("import location is not a regular file: {path}")]
    LocationNotFile { path: PathBuf },
    #[error("import location is not a folder: {path}")]
    LocationNotFolder { path: PathBuf },
    #[error("import source has no usable filename: {path}")]
    InvalidSourcePath { path: PathBuf },
    #[error("the import preview changed; preview the locations again")]
    PreviewChanged,
    #[error("selected row is not importable ({state:?}): {path}")]
    UnimportableSelection {
        path: PathBuf,
        state: ImportRowState,
    },
    #[error("selected import title is empty: {path}")]
    EmptyTitle { path: PathBuf },
    #[error("metadata game {database_id} is no longer a search candidate for {path}")]
    InvalidMetadataSelection { path: PathBuf, database_id: u32 },
    #[error("at least one importable row must be selected")]
    EmptySelection,
    #[error("the committed transaction did not report a platform XML backup")]
    MissingPlatformBackup,
    #[error("metadata game {database_id} has an out-of-range {field} value: {value}")]
    MetadataValueOutOfRange {
        database_id: i64,
        field: &'static str,
        value: String,
    },
    #[error(transparent)]
    Metadata(#[from] MetadataError),
    #[error(transparent)]
    Path(#[from] LaunchPathError),
    #[error(transparent)]
    PlatformPath(#[from] PlatformPathError),
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error(transparent)]
    Transaction(#[from] TransactionError),
    #[error("I/O failure at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLATFORM: &str =
        include_str!("../../../fixtures/launchbox/Data/Platforms/Fixture Console.xml");

    fn library() -> (tempfile::TempDir, PathBuf) {
        let directory = tempfile::tempdir().unwrap();
        let platform = directory.path().join("Data/Platforms/Fixture Console.xml");
        fs::create_dir_all(platform.parent().unwrap()).unwrap();
        fs::write(&platform, PLATFORM).unwrap();
        (directory, platform)
    }

    fn configure_fixture_emulator(library: &Path) {
        fs::write(
            library.join("Data/Emulators.xml"),
            include_str!("../../../fixtures/launchbox/Data/Emulators.xml"),
        )
        .unwrap();
    }

    fn configure_fixture_metadata(library: &Path) -> PathBuf {
        let metadata_path = library.join("Metadata/LaunchBox.Metadata.db");
        fs::create_dir_all(metadata_path.parent().unwrap()).unwrap();
        let connection = rusqlite::Connection::open(&metadata_path).unwrap();
        connection
            .execute_batch(include_str!(
                "../../../fixtures/launchbox/Metadata/fixture.sql"
            ))
            .unwrap();
        metadata_path
    }

    fn request(
        locations: Vec<ImportLocation>,
        file_policy: ImportFilePolicy,
    ) -> ManualImportRequest {
        ManualImportRequest {
            platform: "Fixture Console".into(),
            locations,
            recursive: true,
            use_folder_names: false,
            file_policy,
            duplicate_policy: ImportDuplicatePolicy::Skip,
            extensions: vec!["rom".into(), ".zip".into()],
            copy_files_with_same_name: false,
            copy_to_subfolders: false,
            combine_disc_sets: false,
            combine_matching_titles: false,
            search_local_metadata: false,
            look_for_pdf_manuals: false,
            emulator_id: None,
        }
    }

    fn selection(preview: &ManualImportPreview) -> ManualImportSelection {
        ManualImportSelection {
            request: preview.request.clone(),
            rows: preview
                .rows
                .iter()
                .map(|row| ManualImportRowSelection {
                    source_path: row.source_path.clone(),
                    title: row.title.clone(),
                    included: row.included,
                    metadata_database_id: row
                        .metadata
                        .as_ref()
                        .map(|metadata| metadata.database_id),
                })
                .collect(),
        }
    }

    #[test]
    fn folder_preview_is_sorted_filtered_editable_and_duplicate_aware() {
        let (library, _) = library();
        let source = library.path().join("incoming");
        fs::create_dir_all(source.join("nested/Folder Title")).unwrap();
        fs::write(source.join("zeta.rom"), b"zeta").unwrap();
        fs::write(source.join("alpha.zip"), b"alpha").unwrap();
        fs::write(source.join("ignored.txt"), b"ignored").unwrap();
        fs::write(source.join("nested/Folder Title/deep.rom"), b"deep").unwrap();
        let mut import_request = request(
            vec![ImportLocation {
                path: source,
                kind: ImportLocationKind::Folder,
            }],
            ImportFilePolicy::Leave,
        );
        import_request.use_folder_names = true;
        let preview =
            preview_manual_import(library.path(), &HostPathResolver::default(), import_request)
                .unwrap();

        assert_eq!(preview.rows.len(), 4);
        assert_eq!(preview.importable_count, 3);
        assert_eq!(
            preview
                .rows
                .iter()
                .find(|row| row.extension == "txt")
                .unwrap()
                .state,
            ImportRowState::UnsupportedExtension
        );
        assert_eq!(
            preview
                .rows
                .iter()
                .find(|row| row.source_path.ends_with("deep.rom"))
                .unwrap()
                .title,
            "Folder Title"
        );

        let existing = library.path().join("Games/Fixture Adventure/adventure.rom");
        fs::create_dir_all(existing.parent().unwrap()).unwrap();
        fs::write(&existing, b"existing").unwrap();
        let duplicate = preview_manual_import(
            library.path(),
            &HostPathResolver::default(),
            request(
                vec![ImportLocation {
                    path: existing,
                    kind: ImportLocationKind::File,
                }],
                ImportFilePolicy::Leave,
            ),
        )
        .unwrap();
        assert_eq!(duplicate.rows[0].state, ImportRowState::Duplicate);
        assert!(!duplicate.rows[0].included);

        let additional_path = library.path().join("Games/Fixture Adventure/manual.pdf");
        fs::create_dir_all(additional_path.parent().unwrap()).unwrap();
        fs::write(&additional_path, b"manual").unwrap();
        let mut additional_request = request(
            vec![ImportLocation {
                path: additional_path,
                kind: ImportLocationKind::File,
            }],
            ImportFilePolicy::Leave,
        );
        additional_request.extensions = vec!["pdf".into()];
        let additional_duplicate = preview_manual_import(
            library.path(),
            &HostPathResolver::default(),
            additional_request,
        )
        .unwrap();
        assert_eq!(
            additional_duplicate.rows[0].state,
            ImportRowState::Duplicate
        );
    }

    #[test]
    fn unique_exact_local_metadata_match_is_previewed_and_persisted() {
        let (library, platform) = library();
        configure_fixture_metadata(library.path());
        let source_directory = tempfile::tempdir().unwrap();
        let source = source_directory.path().join("Fixture Saga (USA).rom");
        fs::write(&source, b"metadata rom").unwrap();
        let mut import_request = request(
            vec![ImportLocation {
                path: source,
                kind: ImportLocationKind::File,
            }],
            ImportFilePolicy::Copy,
        );
        import_request.search_local_metadata = true;
        import_request.copy_to_subfolders = true;

        let preview =
            preview_manual_import(library.path(), &HostPathResolver::default(), import_request)
                .unwrap();
        assert_eq!(preview.rows[0].title, "Fixture Saga (USA)");
        assert_eq!(preview.rows[0].metadata_candidate_count, 1);
        assert_eq!(
            preview.rows[0].metadata_match_kind,
            Some(ManualImportMetadataMatchKind::Exact)
        );
        assert!(preview.rows[0]
            .destination_path
            .as_deref()
            .is_some_and(|path| path.ends_with(
                "Games/Fixture Console/Fixture Saga (USA) (2002)/Fixture Saga (USA).rom"
            )));
        assert_eq!(
            preview.rows[0].application_path,
            r"Games\Fixture Console\Fixture Saga (USA) (2002)\Fixture Saga (USA).rom"
        );
        assert!(preview.rows[0]
            .message
            .contains("unique exact local metadata match"));
        assert_eq!(
            preview.rows[0].metadata,
            Some(ManualImportMetadata {
                database_id: 4242,
                notes: Some("Recovered local metadata overview.".into()),
                developer: Some("Fixture Forge".into()),
                genre: Some("Role-Playing; Strategy".into()),
                max_players: Some(2),
                play_mode: Some("Cooperative; Multiplayer".into()),
                publisher: Some("Fixture Press".into()),
                rating: Some("E10+".into()),
                release_date: Some("2002-03-04".into()),
                release_type: Some("Released".into()),
                wikipedia_url: Some("https://example.org/wiki/Fixture_Saga".into()),
                video_url: Some("https://video.example/fixture-saga".into()),
                community_star_rating: Some(4.75),
            })
        );

        let report = execute_manual_import_with_ids(
            library.path(),
            &platform,
            &HostPathResolver::default(),
            selection(&preview),
            || "metadata-game".into(),
        )
        .unwrap();
        let game = &report.games[0];
        assert_eq!(game.database_id, Some(4242));
        assert_eq!(game.release_date.as_deref(), Some("2002-03-04"));
        assert_eq!(game.play_mode.as_deref(), Some("Cooperative; Multiplayer"));
        assert_eq!(game.community_star_rating, 4.75);
        assert_eq!(report.created_files.len(), 1);
        assert_eq!(
            fs::read(
                library
                    .path()
                    .join("Games/Fixture Console/Fixture Saga (USA) (2002)/Fixture Saga (USA).rom")
            )
            .unwrap(),
            b"metadata rom"
        );

        let xml = fs::read_to_string(platform).unwrap();
        for value in [
            "<DatabaseID>4242</DatabaseID>",
            "<Notes>Recovered local metadata overview.</Notes>",
            "<Developer>Fixture Forge</Developer>",
            "<Genre>Role-Playing; Strategy</Genre>",
            "<MaxPlayers>2</MaxPlayers>",
            "<PlayMode>Cooperative; Multiplayer</PlayMode>",
            "<Publisher>Fixture Press</Publisher>",
            "<Rating>E10+</Rating>",
            "<ReleaseDate>2002-03-04</ReleaseDate>",
            "<ReleaseType>Released</ReleaseType>",
            "<WikipediaURL>https://example.org/wiki/Fixture_Saga</WikipediaURL>",
            "<VideoUrl>https://video.example/fixture-saga</VideoUrl>",
            "<CommunityStarRating>4.75</CommunityStarRating>",
        ] {
            assert!(xml.contains(value), "missing persisted metadata: {value}");
        }
    }

    #[test]
    fn unique_partial_local_metadata_fallback_is_previewed_and_persisted() {
        let (library, platform) = library();
        configure_fixture_metadata(library.path());
        let source_directory = tempfile::tempdir().unwrap();
        let source = source_directory.path().join("Fixture Sag (USA).rom");
        fs::write(&source, b"partial metadata rom").unwrap();
        let mut import_request = request(
            vec![ImportLocation {
                path: source,
                kind: ImportLocationKind::File,
            }],
            ImportFilePolicy::Leave,
        );
        import_request.search_local_metadata = true;

        let preview =
            preview_manual_import(library.path(), &HostPathResolver::default(), import_request)
                .unwrap();
        assert_eq!(preview.rows[0].title, "Fixture Saga (USA)");
        assert_eq!(preview.rows[0].metadata_candidate_count, 1);
        assert_eq!(
            preview.rows[0].metadata_match_kind,
            Some(ManualImportMetadataMatchKind::Partial)
        );
        assert_eq!(
            preview.rows[0]
                .metadata
                .as_ref()
                .map(|metadata| metadata.database_id),
            Some(4242)
        );
        assert!(preview.rows[0]
            .message
            .contains("unique partial local metadata match"));

        let report = execute_manual_import_with_ids(
            library.path(),
            &platform,
            &HostPathResolver::default(),
            selection(&preview),
            || "partial-metadata-game".into(),
        )
        .unwrap();
        assert_eq!(report.games[0].database_id, Some(4242));
        assert_eq!(report.games[0].title, "Fixture Saga (USA)");
        assert_eq!(
            report.games[0].application_path,
            source_directory
                .path()
                .join("Fixture Sag (USA).rom")
                .to_string_lossy()
        );
    }

    #[test]
    fn ambiguous_partial_local_metadata_match_can_be_selected_and_persisted() {
        let (library, platform) = library();
        let database_path = configure_fixture_metadata(library.path());
        let connection = rusqlite::Connection::open(database_path).unwrap();
        connection
            .execute(
                "INSERT INTO Games VALUES (
                    4343, 'Fixture Saga (Japan)', 'FIXTURE SAGA', NULL, 2003,
                    'Japan overview', 1, 'Released', 0, NULL, 4.25, NULL,
                    'Fixture Console', 'E', 'Role-Playing', 'Japan Forge',
                    'Japan Press'
                )",
                [],
            )
            .unwrap();
        let source_directory = tempfile::tempdir().unwrap();
        let source = source_directory.path().join("Fixture Sag.rom");
        fs::write(&source, b"ambiguous rom").unwrap();
        fs::write(
            source_directory.path().join("Fixture Sag.pdf"),
            b"selected metadata manual",
        )
        .unwrap();
        let mut import_request = request(
            vec![ImportLocation {
                path: source,
                kind: ImportLocationKind::File,
            }],
            ImportFilePolicy::Copy,
        );
        import_request.search_local_metadata = true;
        import_request.copy_to_subfolders = true;
        import_request.copy_files_with_same_name = true;
        import_request.look_for_pdf_manuals = true;

        let preview =
            preview_manual_import(library.path(), &HostPathResolver::default(), import_request)
                .unwrap();
        assert_eq!(preview.rows[0].title, "Fixture Sag");
        assert_eq!(preview.rows[0].metadata_candidate_count, 2);
        assert_eq!(
            preview.rows[0].metadata_match_kind,
            Some(ManualImportMetadataMatchKind::Partial)
        );
        assert_eq!(
            preview.rows[0].metadata_candidates,
            vec![
                ManualImportMetadataCandidate {
                    database_id: 4242,
                    title: "Fixture Saga (USA)".into(),
                    platform: "Fixture Console".into(),
                    release_year: Some(2002),
                    developer: Some("Fixture Forge".into()),
                    publisher: Some("Fixture Press".into()),
                },
                ManualImportMetadataCandidate {
                    database_id: 4343,
                    title: "Fixture Saga (Japan)".into(),
                    platform: "Fixture Console".into(),
                    release_year: Some(2003),
                    developer: Some("Japan Forge".into()),
                    publisher: Some("Japan Press".into()),
                },
            ]
        );
        assert_eq!(preview.rows[0].metadata, None);
        assert!(preview.rows[0]
            .message
            .contains("2 partial local metadata matches require review"));

        let mut selected = selection(&preview);
        selected.rows[0].title = "Fixture Saga (Japan)".into();
        selected.rows[0].metadata_database_id = Some(4343);
        let report = execute_manual_import_with_ids(
            library.path(),
            &platform,
            &HostPathResolver::default(),
            selected,
            || "selected-metadata-game".into(),
        )
        .unwrap();
        assert_eq!(report.games[0].title, "Fixture Saga (Japan)");
        assert_eq!(report.games[0].database_id, Some(4343));
        assert_eq!(report.games[0].release_date.as_deref(), Some("2003-01-01"));
        assert_eq!(report.games[0].developer.as_deref(), Some("Japan Forge"));
        assert_eq!(
            report.games[0].manual_path,
            Some(r"Games\Fixture Console\Fixture Saga (Japan) (2003)\Fixture Sag.pdf".into())
        );
        assert_eq!(report.created_files.len(), 2);
        assert_eq!(
            report.games[0].application_path,
            r"Games\Fixture Console\Fixture Saga (Japan) (2003)\Fixture Sag.rom"
        );
        assert_eq!(
            fs::read(
                library
                    .path()
                    .join("Games/Fixture Console/Fixture Saga (Japan) (2003)/Fixture Sag.rom")
            )
            .unwrap(),
            b"ambiguous rom"
        );
        assert_eq!(
            fs::read(
                library
                    .path()
                    .join("Games/Fixture Console/Fixture Saga (Japan) (2003)/Fixture Sag.pdf")
            )
            .unwrap(),
            b"selected metadata manual"
        );
    }

    #[test]
    fn removed_metadata_candidate_is_rejected_before_writing() {
        let (library, platform) = library();
        let database_path = configure_fixture_metadata(library.path());
        let connection = rusqlite::Connection::open(&database_path).unwrap();
        connection
            .execute(
                "INSERT INTO Games VALUES (
                    4343, 'Fixture Saga (Japan)', 'FIXTURE SAGA', NULL, 2003,
                    'Japan overview', 1, 'Released', 0, NULL, 4.25, NULL,
                    'Fixture Console', 'E', 'Role-Playing', 'Japan Forge',
                    'Japan Press'
                )",
                [],
            )
            .unwrap();
        let source_directory = tempfile::tempdir().unwrap();
        let source = source_directory.path().join("Fixture Saga.rom");
        fs::write(&source, b"stale metadata rom").unwrap();
        let mut import_request = request(
            vec![ImportLocation {
                path: source.clone(),
                kind: ImportLocationKind::File,
            }],
            ImportFilePolicy::Leave,
        );
        import_request.search_local_metadata = true;
        let preview =
            preview_manual_import(library.path(), &HostPathResolver::default(), import_request)
                .unwrap();
        let mut selected = selection(&preview);
        selected.rows[0].metadata_database_id = Some(4343);
        connection
            .execute("DELETE FROM Games WHERE DatabaseID = 4343", [])
            .unwrap();
        drop(connection);

        assert!(matches!(
            execute_manual_import_with_ids(
                library.path(),
                &platform,
                &HostPathResolver::default(),
                selected,
                || "must-not-be-used".into(),
            ),
            Err(ImportError::InvalidMetadataSelection {
                path,
                database_id: 4343
            }) if path == source
        ));
        assert_eq!(fs::read_to_string(platform).unwrap(), PLATFORM);
    }

    #[test]
    fn portable_subfolder_planning_uses_distinct_final_titles() {
        let (library, platform) = library();
        let source_directory = tempfile::tempdir().unwrap();
        let alpha = source_directory.path().join("Alpha");
        let beta = source_directory.path().join("Beta");
        fs::create_dir_all(&alpha).unwrap();
        fs::create_dir_all(&beta).unwrap();
        let alpha_rom = alpha.join("disc.rom");
        let beta_rom = beta.join("disc.rom");
        fs::write(&alpha_rom, b"alpha").unwrap();
        fs::write(&beta_rom, b"beta").unwrap();
        let mut import_request = request(
            vec![
                ImportLocation {
                    path: alpha_rom,
                    kind: ImportLocationKind::File,
                },
                ImportLocation {
                    path: beta_rom,
                    kind: ImportLocationKind::File,
                },
            ],
            ImportFilePolicy::Copy,
        );
        import_request.use_folder_names = true;
        import_request.copy_to_subfolders = true;

        let preview =
            preview_manual_import(library.path(), &HostPathResolver::default(), import_request)
                .unwrap();
        assert_eq!(preview.importable_count, 2);
        assert_eq!(
            preview
                .rows
                .iter()
                .map(|row| row.application_path.as_str())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([
                r"Games\Fixture Console\Alpha\disc.rom",
                r"Games\Fixture Console\Beta\disc.rom",
            ])
        );

        let mut import_selection = selection(&preview);
        import_selection.rows[0].title = "Edited: Alpha/Title".into();
        let mut ids = ["portable-alpha", "portable-beta"].into_iter();
        let report = execute_manual_import_with_ids(
            library.path(),
            &platform,
            &HostPathResolver::default(),
            import_selection,
            || ids.next().unwrap().into(),
        )
        .unwrap();
        assert_eq!(report.games.len(), 2);
        assert!(report.games.iter().any(|game| {
            game.title == "Edited: Alpha/Title"
                && game.application_path == r"Games\Fixture Console\Edited_ Alpha_Title\disc.rom"
        }));
        assert!(library
            .path()
            .join("Games/Fixture Console/Edited_ Alpha_Title/disc.rom")
            .is_file());
        assert!(library
            .path()
            .join("Games/Fixture Console/Beta/disc.rom")
            .is_file());
    }

    #[test]
    fn portable_subfolder_planning_rejects_existing_case_variant_directory() {
        let (library, _) = library();
        let existing = library.path().join("Games/Fixture Console/alpha");
        fs::create_dir_all(&existing).unwrap();
        fs::write(existing.join("unrelated.txt"), b"keep").unwrap();
        let source_directory = tempfile::tempdir().unwrap();
        let source = source_directory.path().join("Alpha.rom");
        fs::write(&source, b"rom").unwrap();
        let mut import_request = request(
            vec![ImportLocation {
                path: source,
                kind: ImportLocationKind::File,
            }],
            ImportFilePolicy::Copy,
        );
        import_request.copy_to_subfolders = true;

        let preview =
            preview_manual_import(library.path(), &HostPathResolver::default(), import_request)
                .unwrap();
        assert_eq!(preview.rows[0].state, ImportRowState::DestinationExists);
        assert!(!preview.rows[0].included);
        assert!(preview.rows[0]
            .message
            .contains("Portable subfolder name collides"));
    }

    #[test]
    fn enabled_local_metadata_search_requires_a_readable_database() {
        let (library, _) = library();
        let source_directory = tempfile::tempdir().unwrap();
        let source = source_directory.path().join("Missing Metadata.rom");
        fs::write(&source, b"rom").unwrap();
        let mut import_request = request(
            vec![ImportLocation {
                path: source,
                kind: ImportLocationKind::File,
            }],
            ImportFilePolicy::Leave,
        );
        import_request.search_local_metadata = true;

        assert!(matches!(
            preview_manual_import(library.path(), &HostPathResolver::default(), import_request),
            Err(ImportError::Metadata(MetadataError::Open { .. }))
        ));
    }

    #[test]
    fn copy_batch_commits_roms_and_platform_xml_together() {
        let (library, platform) = library();
        let source_directory = tempfile::tempdir().unwrap();
        let first = source_directory.path().join("First Game.rom");
        let second = source_directory.path().join("Second Game.zip");
        fs::write(&first, b"first rom").unwrap();
        fs::write(&second, b"second rom").unwrap();
        let preview = preview_manual_import(
            library.path(),
            &HostPathResolver::default(),
            request(
                vec![
                    ImportLocation {
                        path: first.clone(),
                        kind: ImportLocationKind::File,
                    },
                    ImportLocation {
                        path: second.clone(),
                        kind: ImportLocationKind::File,
                    },
                ],
                ImportFilePolicy::Copy,
            ),
        )
        .unwrap();
        let mut ids = ["import-one", "import-two"].into_iter();
        let report = execute_manual_import_with_ids(
            library.path(),
            &platform,
            &HostPathResolver::default(),
            selection(&preview),
            || ids.next().unwrap().to_string(),
        )
        .unwrap();

        assert_eq!(report.games.len(), 2);
        assert_eq!(report.created_files.len(), 2);
        assert_eq!(
            fs::read(library.path().join("Games/Fixture Console/First Game.rom")).unwrap(),
            b"first rom"
        );
        let document = PlatformDocument::load(&platform).unwrap();
        let imported = document
            .library()
            .games
            .iter()
            .filter(|game| game.id.starts_with("import-"))
            .collect::<Vec<_>>();
        assert_eq!(imported.len(), 2);
        assert_eq!(
            imported[0].application_path,
            r"Games\Fixture Console\First Game.rom"
        );
    }

    #[test]
    fn same_stem_different_extension_companions_share_the_copy_transaction() {
        let (library, platform) = library();
        let source_directory = tempfile::tempdir().unwrap();
        let cue = source_directory.path().join("Companion Game.cue");
        let bin = source_directory.path().join("Companion Game.bin");
        let sub = source_directory.path().join("COMPANION GAME.sub");
        let unrelated = source_directory.path().join("Other Game.txt");
        fs::write(&cue, b"cue bytes").unwrap();
        fs::write(&bin, b"bin bytes").unwrap();
        fs::write(&sub, b"sub bytes").unwrap();
        fs::write(&unrelated, b"unrelated bytes").unwrap();
        let mut import_request = request(
            vec![ImportLocation {
                path: cue.clone(),
                kind: ImportLocationKind::File,
            }],
            ImportFilePolicy::Copy,
        );
        import_request.extensions = vec!["cue".into()];
        import_request.copy_files_with_same_name = true;

        let preview =
            preview_manual_import(library.path(), &HostPathResolver::default(), import_request)
                .unwrap();
        assert_eq!(preview.rows.len(), 1);
        assert_eq!(preview.rows[0].companion_file_count(), 2);
        assert_eq!(preview.rows[0].transfer_file_count(), 3);
        assert_eq!(
            preview.rows[0]
                .same_name_files
                .iter()
                .map(|file| file.extension.as_str())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from(["bin", "sub"])
        );

        let report = execute_manual_import_with_ids(
            library.path(),
            &platform,
            &HostPathResolver::default(),
            selection(&preview),
            || "companion-game".into(),
        )
        .unwrap();
        assert_eq!(report.created_files.len(), 3);
        for (name, bytes) in [
            ("Companion Game.cue", b"cue bytes".as_slice()),
            ("Companion Game.bin", b"bin bytes".as_slice()),
            ("COMPANION GAME.sub", b"sub bytes".as_slice()),
        ] {
            assert_eq!(
                fs::read(library.path().join("Games/Fixture Console").join(name)).unwrap(),
                bytes
            );
        }
        assert!(!library
            .path()
            .join("Games/Fixture Console/Other Game.txt")
            .exists());
        assert_eq!(
            report.games[0].application_path,
            r"Games\Fixture Console\Companion Game.cue"
        );
    }

    #[test]
    fn companion_destination_collision_blocks_the_whole_game() {
        let (library, _) = library();
        let source_directory = tempfile::tempdir().unwrap();
        let rom = source_directory.path().join("Collision.rom");
        let companion = source_directory.path().join("Collision.dat");
        fs::write(&rom, b"rom").unwrap();
        fs::write(&companion, b"companion").unwrap();
        let destination = library.path().join("Games/Fixture Console/Collision.dat");
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        fs::write(&destination, b"keep").unwrap();
        let mut import_request = request(
            vec![ImportLocation {
                path: rom,
                kind: ImportLocationKind::File,
            }],
            ImportFilePolicy::Copy,
        );
        import_request.copy_files_with_same_name = true;

        let preview =
            preview_manual_import(library.path(), &HostPathResolver::default(), import_request)
                .unwrap();
        assert_eq!(preview.rows[0].state, ImportRowState::DestinationExists);
        assert!(!preview.rows[0].included);
        assert!(preview.rows[0].message.contains("Collision.dat"));
        assert!(!library
            .path()
            .join("Games/Fixture Console/Collision.rom")
            .exists());
    }

    #[test]
    fn same_name_pdf_manual_wins_and_is_persisted_with_multiple_pdfs() {
        let (library, platform) = library();
        let source_directory = tempfile::tempdir().unwrap();
        let rom = source_directory.path().join("Manual Game.rom");
        let manual = source_directory.path().join("MANUAL GAME.PDF");
        let unrelated = source_directory.path().join("Reference.pdf");
        fs::write(&rom, b"rom").unwrap();
        fs::write(&manual, b"manual").unwrap();
        fs::write(&unrelated, b"reference").unwrap();
        let mut import_request = request(
            vec![ImportLocation {
                path: rom,
                kind: ImportLocationKind::File,
            }],
            ImportFilePolicy::Leave,
        );
        import_request.look_for_pdf_manuals = true;

        let preview =
            preview_manual_import(library.path(), &HostPathResolver::default(), import_request)
                .unwrap();
        assert_eq!(preview.rows[0].manual_candidate_count, 2);
        let planned_manual = preview.rows[0].manual.as_ref().unwrap();
        assert_eq!(
            planned_manual.source_path,
            fs::canonicalize(&manual).unwrap()
        );
        assert_eq!(planned_manual.stored_path, manual.to_string_lossy());
        assert!(preview.rows[0].message.contains("same-name PDF manual"));
        let manual_stored = manual.to_string_lossy().to_string();

        let report = execute_manual_import_with_ids(
            library.path(),
            &platform,
            &HostPathResolver::default(),
            selection(&preview),
            || "manual-game".into(),
        )
        .unwrap();
        assert_eq!(
            report.games[0].manual_path.as_deref(),
            Some(manual_stored.as_str())
        );
        let persisted = PlatformDocument::load(&platform).unwrap();
        assert_eq!(
            persisted
                .library()
                .games
                .iter()
                .find(|game| game.id == "manual-game")
                .and_then(|game| game.manual_path.as_deref()),
            Some(manual_stored.as_str())
        );
    }

    #[test]
    fn copied_same_name_pdf_manual_uses_its_portable_committed_path() {
        let (library, platform) = library();
        let source_directory = tempfile::tempdir().unwrap();
        let rom = source_directory.path().join("Portable Manual.rom");
        let manual = source_directory.path().join("Portable Manual.pdf");
        fs::write(&rom, b"rom").unwrap();
        fs::write(&manual, b"pdf").unwrap();
        let mut import_request = request(
            vec![ImportLocation {
                path: rom,
                kind: ImportLocationKind::File,
            }],
            ImportFilePolicy::Copy,
        );
        import_request.copy_files_with_same_name = true;
        import_request.look_for_pdf_manuals = true;

        let preview =
            preview_manual_import(library.path(), &HostPathResolver::default(), import_request)
                .unwrap();
        assert_eq!(
            preview.rows[0]
                .manual
                .as_ref()
                .map(|manual| manual.stored_path.as_str()),
            Some(r"Games\Fixture Console\Portable Manual.pdf")
        );
        assert_eq!(preview.rows[0].transfer_file_count(), 2);

        let report = execute_manual_import_with_ids(
            library.path(),
            &platform,
            &HostPathResolver::default(),
            selection(&preview),
            || "portable-manual-game".into(),
        )
        .unwrap();
        assert_eq!(report.created_files.len(), 2);
        assert_eq!(
            fs::read(
                library
                    .path()
                    .join("Games/Fixture Console/Portable Manual.pdf")
            )
            .unwrap(),
            b"pdf"
        );
        assert_eq!(
            report.games[0].manual_path.as_deref(),
            Some(r"Games\Fixture Console\Portable Manual.pdf")
        );
        assert!(manual.is_file(), "copy keeps the original manual");
    }

    #[test]
    fn ambiguous_pdf_manuals_are_reported_without_guessing() {
        let (library, _) = library();
        let source_directory = tempfile::tempdir().unwrap();
        let rom = source_directory.path().join("Ambiguous.rom");
        fs::write(&rom, b"rom").unwrap();
        fs::write(source_directory.path().join("Manual.pdf"), b"one").unwrap();
        fs::write(source_directory.path().join("Reference.PDF"), b"two").unwrap();
        let mut import_request = request(
            vec![ImportLocation {
                path: rom,
                kind: ImportLocationKind::File,
            }],
            ImportFilePolicy::Leave,
        );
        import_request.look_for_pdf_manuals = true;

        let preview =
            preview_manual_import(library.path(), &HostPathResolver::default(), import_request)
                .unwrap();
        assert_eq!(preview.rows[0].manual_candidate_count, 2);
        assert!(preview.rows[0].manual.is_none());
        assert!(preview.rows[0]
            .message
            .contains("2 PDF manual candidates require review"));
    }

    #[test]
    fn sole_pdf_manual_uses_reversible_windows_mapping() {
        let (library, _) = library();
        let source_directory = tempfile::tempdir().unwrap();
        let game_directory = source_directory.path().join("Games/Manual Test");
        fs::create_dir_all(&game_directory).unwrap();
        let rom = game_directory.join("game.rom");
        let manual = game_directory.join("instructions.pdf");
        fs::write(&rom, b"rom").unwrap();
        fs::write(&manual, b"manual").unwrap();
        let resolver = HostPathResolver::default()
            .with_windows_drive_mapping('D', source_directory.path())
            .unwrap();
        let mut import_request = request(
            vec![ImportLocation {
                path: rom,
                kind: ImportLocationKind::File,
            }],
            ImportFilePolicy::Leave,
        );
        import_request.look_for_pdf_manuals = true;

        let preview = preview_manual_import(library.path(), &resolver, import_request).unwrap();
        assert_eq!(
            preview.rows[0]
                .manual
                .as_ref()
                .map(|manual| manual.stored_path.as_str()),
            Some(r"D:\Games\Manual Test\instructions.pdf")
        );
        assert!(preview.rows[0]
            .message
            .contains("game folder's sole PDF manual"));
    }

    #[test]
    fn complete_disc_set_is_one_game_with_all_discs_as_additional_applications() {
        let (library, platform) = library();
        configure_fixture_emulator(library.path());
        let source_directory = tempfile::tempdir().unwrap();
        let first = source_directory
            .path()
            .join("Fixture Saga (USA) - (Disc 1 of 2).rom");
        let second = source_directory
            .path()
            .join("Fixture Saga (USA) - (Disc 2 of 2).rom");
        let first_companion = source_directory
            .path()
            .join("Fixture Saga (USA) - (Disc 1 of 2).dat");
        let second_companion = source_directory
            .path()
            .join("Fixture Saga (USA) - (Disc 2 of 2).dat");
        fs::write(&first, b"disc one").unwrap();
        fs::write(&second, b"disc two").unwrap();
        fs::write(&first_companion, b"disc one companion").unwrap();
        fs::write(&second_companion, b"disc two companion").unwrap();
        let mut import_request = request(
            vec![
                ImportLocation {
                    path: second.clone(),
                    kind: ImportLocationKind::File,
                },
                ImportLocation {
                    path: first.clone(),
                    kind: ImportLocationKind::File,
                },
            ],
            ImportFilePolicy::Copy,
        );
        import_request.copy_files_with_same_name = true;
        import_request.combine_disc_sets = true;
        import_request.emulator_id = Some("fixture-emulator".into());

        let preview =
            preview_manual_import(library.path(), &HostPathResolver::default(), import_request)
                .unwrap();
        assert_eq!(preview.rows.len(), 1);
        assert_eq!(preview.importable_count, 1);
        assert_eq!(preview.rows[0].title, "Fixture Saga");
        assert_eq!(preview.rows[0].disc, Some(1));
        assert_eq!(preview.rows[0].version.as_deref(), Some("(USA)"));
        assert_eq!(preview.rows[0].region.as_deref(), Some("North America"));
        assert_eq!(preview.rows[0].file_count(), 2);
        assert_eq!(preview.rows[0].companion_file_count(), 2);
        assert_eq!(preview.rows[0].transfer_file_count(), 4);
        assert_eq!(preview.rows[0].additional_roms[0].disc, Some(2));

        let mut ids = ["disc-game", "disc-app-one", "disc-app-two"].into_iter();
        let report = execute_manual_import_with_ids(
            library.path(),
            &platform,
            &HostPathResolver::default(),
            selection(&preview),
            || ids.next().unwrap().to_string(),
        )
        .unwrap();
        assert_eq!(report.games.len(), 1);
        assert_eq!(report.additional_applications.len(), 2);
        assert_eq!(report.created_files.len(), 4);
        assert_eq!(
            fs::read(
                library
                    .path()
                    .join("Games/Fixture Console/Fixture Saga (USA) - (Disc 1 of 2).rom")
            )
            .unwrap(),
            b"disc one"
        );
        assert_eq!(
            fs::read(
                library
                    .path()
                    .join("Games/Fixture Console/Fixture Saga (USA) - (Disc 2 of 2).rom")
            )
            .unwrap(),
            b"disc two"
        );
        assert_eq!(
            fs::read(
                library
                    .path()
                    .join("Games/Fixture Console/Fixture Saga (USA) - (Disc 1 of 2).dat")
            )
            .unwrap(),
            b"disc one companion"
        );
        assert_eq!(
            fs::read(
                library
                    .path()
                    .join("Games/Fixture Console/Fixture Saga (USA) - (Disc 2 of 2).dat")
            )
            .unwrap(),
            b"disc two companion"
        );

        let persisted = PlatformDocument::load(&platform).unwrap();
        let game = persisted
            .library()
            .games
            .iter()
            .find(|game| game.id == "disc-game")
            .unwrap();
        assert_eq!(
            game.application_path,
            r"Games\Fixture Console\Fixture Saga (USA) - (Disc 1 of 2).rom"
        );
        let applications = persisted
            .library()
            .additional_applications
            .iter()
            .filter(|application| application.game_id == "disc-game")
            .collect::<Vec<_>>();
        assert_eq!(applications.len(), 2);
        assert_eq!(
            applications
                .iter()
                .map(|application| (
                    application.disc,
                    application.priority,
                    application.version.as_deref(),
                    application.region.as_deref(),
                ))
                .collect::<Vec<_>>(),
            vec![
                (Some(1), 1, Some("(USA)"), Some("North America")),
                (Some(2), 2, Some("(USA)"), Some("North America")),
            ]
        );
        assert_eq!(applications[0].application_path, game.application_path);
        assert!(applications.iter().all(|application| {
            application.use_emulator
                && application.emulator_id.as_deref() == Some("fixture-emulator")
        }));
    }

    #[test]
    fn metadata_resolved_matching_titles_become_selectable_version_applications() {
        let (library, platform) = library();
        configure_fixture_metadata(library.path());
        configure_fixture_emulator(library.path());
        let source_directory = tempfile::tempdir().unwrap();
        let usa = source_directory.path().join("Fixture Saga (USA).rom");
        let world = source_directory
            .path()
            .join("Fixture Saga (World) (Rev 1).rom");
        fs::write(&usa, b"north american rom").unwrap();
        fs::write(&world, b"world revision rom").unwrap();
        let mut import_request = request(
            vec![
                ImportLocation {
                    path: world.clone(),
                    kind: ImportLocationKind::File,
                },
                ImportLocation {
                    path: usa.clone(),
                    kind: ImportLocationKind::File,
                },
            ],
            ImportFilePolicy::Leave,
        );
        import_request.search_local_metadata = true;
        import_request.combine_matching_titles = true;
        import_request.emulator_id = Some("fixture-emulator".into());

        let preview =
            preview_manual_import(library.path(), &HostPathResolver::default(), import_request)
                .unwrap();
        assert_eq!(preview.rows.len(), 1);
        assert_eq!(preview.importable_count, 1);
        let row = &preview.rows[0];
        assert_eq!(row.source_path, fs::canonicalize(&usa).unwrap());
        assert_eq!(row.title, "Fixture Saga (USA)");
        assert_eq!(row.version.as_deref(), Some("(USA)"));
        assert_eq!(row.region.as_deref(), Some("North America"));
        assert_eq!(row.file_count(), 2);
        assert_eq!(
            row.additional_roms[0].source_path,
            fs::canonicalize(&world).unwrap()
        );
        assert_eq!(
            row.additional_roms[0].version.as_deref(),
            Some("(World) (Rev 1)")
        );
        assert_eq!(row.additional_roms[0].region.as_deref(), Some("World"));
        assert!(row
            .message
            .contains("combined 2 matching-title ROMs as selectable versions"));

        let mut ids = ["version-game", "version-app-usa", "version-app-world"].into_iter();
        let report = execute_manual_import_with_ids(
            library.path(),
            &platform,
            &HostPathResolver::default(),
            selection(&preview),
            || ids.next().unwrap().to_string(),
        )
        .unwrap();
        assert_eq!(report.games.len(), 1);
        assert_eq!(report.additional_applications.len(), 2);

        let persisted = PlatformDocument::load(&platform).unwrap();
        let game = persisted
            .library()
            .games
            .iter()
            .find(|game| game.id == "version-game")
            .unwrap();
        assert_eq!(game.version.as_deref(), Some("(USA)"));
        assert_eq!(game.region.as_deref(), Some("North America"));
        assert_eq!(game.status.as_deref(), Some("Imported ROM"));
        let applications = persisted
            .library()
            .additional_applications
            .iter()
            .filter(|application| application.game_id == "version-game")
            .collect::<Vec<_>>();
        assert_eq!(
            applications
                .iter()
                .map(|application| (
                    application.name.as_str(),
                    application.priority,
                    application.version.as_deref(),
                    application.region.as_deref(),
                    application.developer.as_deref(),
                    application.publisher.as_deref(),
                    application.status.as_deref(),
                ))
                .collect::<Vec<_>>(),
            vec![
                (
                    "Play (USA) Version...",
                    1,
                    Some("(USA)"),
                    Some("North America"),
                    Some("Fixture Forge"),
                    Some("Fixture Press"),
                    Some("Imported ROM"),
                ),
                (
                    "Play (World) (Rev 1) Version...",
                    2,
                    Some("(World) (Rev 1)"),
                    Some("World"),
                    Some("Fixture Forge"),
                    Some("Fixture Press"),
                    Some("Imported ROM"),
                ),
            ]
        );
        assert_eq!(applications[0].application_path, game.application_path);
        assert!(applications.iter().all(|application| {
            application.use_emulator
                && application.emulator_id.as_deref() == Some("fixture-emulator")
                && application.disc.is_none()
        }));
    }

    #[test]
    fn exact_cleaned_titles_combine_without_metadata() {
        let (library, _) = library();
        let source_directory = tempfile::tempdir().unwrap();
        let europe = source_directory
            .path()
            .join("Unlisted Adventure (Europe).rom");
        let usa = source_directory.path().join("Unlisted Adventure (USA).rom");
        fs::write(&europe, b"europe").unwrap();
        fs::write(&usa, b"usa").unwrap();
        let mut import_request = request(
            vec![
                ImportLocation {
                    path: usa,
                    kind: ImportLocationKind::File,
                },
                ImportLocation {
                    path: europe,
                    kind: ImportLocationKind::File,
                },
            ],
            ImportFilePolicy::Leave,
        );
        import_request.combine_matching_titles = true;

        let preview =
            preview_manual_import(library.path(), &HostPathResolver::default(), import_request)
                .unwrap();
        assert_eq!(preview.rows.len(), 1);
        assert_eq!(preview.rows[0].title, "Unlisted Adventure");
        assert_eq!(preview.rows[0].version.as_deref(), Some("(Europe)"));
        assert_eq!(preview.rows[0].region.as_deref(), Some("Europe"));
        assert_eq!(preview.rows[0].additional_roms.len(), 1);
        assert_eq!(
            preview.rows[0].additional_roms[0].version.as_deref(),
            Some("(USA)")
        );
        assert_eq!(
            preview.rows[0].additional_roms[0].region.as_deref(),
            Some("North America")
        );
    }

    #[test]
    fn ambiguous_metadata_rows_remain_separate_for_review() {
        let (library, _) = library();
        let metadata_path = configure_fixture_metadata(library.path());
        let connection = rusqlite::Connection::open(metadata_path).unwrap();
        connection
            .execute(
                "INSERT INTO Games VALUES (
                    4343, 'Fixture Saga Collector (USA)', 'FIXTURE SAGA COLLECTOR',
                    NULL, 2004, 'Collector overview', 4, 'Released', 0, NULL,
                    4.0, NULL, 'Fixture Console', 'E10+', 'Role-Playing',
                    'Collector Forge', 'Collector Press'
                )",
                [],
            )
            .unwrap();
        let source_directory = tempfile::tempdir().unwrap();
        let europe = source_directory.path().join("Fixture Sag (Europe).rom");
        let usa = source_directory.path().join("Fixture Sag (USA).rom");
        fs::write(&europe, b"europe").unwrap();
        fs::write(&usa, b"usa").unwrap();
        let mut import_request = request(
            vec![
                ImportLocation {
                    path: usa,
                    kind: ImportLocationKind::File,
                },
                ImportLocation {
                    path: europe,
                    kind: ImportLocationKind::File,
                },
            ],
            ImportFilePolicy::Leave,
        );
        import_request.search_local_metadata = true;
        import_request.combine_matching_titles = true;

        let preview =
            preview_manual_import(library.path(), &HostPathResolver::default(), import_request)
                .unwrap();
        assert_eq!(preview.rows.len(), 2);
        assert!(preview.rows.iter().all(|row| {
            row.metadata.is_none()
                && row.metadata_candidate_count == 2
                && row.metadata_match_kind == Some(ManualImportMetadataMatchKind::Partial)
                && row.additional_roms.is_empty()
        }));
    }

    #[test]
    fn filename_title_version_and_region_recovery_is_platform_neutral() {
        let path = Path::new(r"C:\Roms\Fixture Saga (Japan, USA) (En) (Rev 2) (Disc 1 of 2).chd");
        assert_eq!(derive_title(path, false), "Fixture Saga");
        assert_eq!(
            rom_version(path, false).as_deref(),
            Some("(Japan, USA) (En) (Rev 2)")
        );
        assert_eq!(
            rom_region(path, false).as_deref(),
            Some("Japan, North America")
        );
    }

    #[test]
    fn incomplete_or_ambiguous_disc_sets_are_not_combined() {
        let (library, _) = library();
        let source_directory = tempfile::tempdir().unwrap();
        let paths = [
            "Incomplete (Disc 1 of 3).rom",
            "Incomplete (Disc 2 of 3).rom",
            "Collision (Disc 1).rom",
            "Collision (Disc 1 of 2).rom",
            "Collision (Disc 2).rom",
        ]
        .map(|name| source_directory.path().join(name));
        for path in &paths {
            fs::write(path, b"rom").unwrap();
        }
        let mut import_request = request(
            paths
                .into_iter()
                .map(|path| ImportLocation {
                    path,
                    kind: ImportLocationKind::File,
                })
                .collect(),
            ImportFilePolicy::Leave,
        );
        import_request.combine_disc_sets = true;

        let preview =
            preview_manual_import(library.path(), &HostPathResolver::default(), import_request)
                .unwrap();
        assert_eq!(preview.rows.len(), 5);
        assert!(preview.rows.iter().all(|row| row.disc.is_none()));
        assert!(preview
            .rows
            .iter()
            .all(|row| row.additional_roms.is_empty()));
    }

    #[test]
    fn explicit_emulator_is_validated_canonicalized_and_persisted() {
        let (library, platform) = library();
        configure_fixture_emulator(library.path());
        let source_directory = tempfile::tempdir().unwrap();
        let source = source_directory.path().join("Pinned Emulator.rom");
        fs::write(&source, b"rom").unwrap();
        let mut import_request = request(
            vec![ImportLocation {
                path: source,
                kind: ImportLocationKind::File,
            }],
            ImportFilePolicy::Leave,
        );
        import_request.emulator_id = Some("FIXTURE-EMULATOR".into());

        let preview =
            preview_manual_import(library.path(), &HostPathResolver::default(), import_request)
                .unwrap();
        assert_eq!(
            preview.request.emulator_id.as_deref(),
            Some("fixture-emulator")
        );
        let report = execute_manual_import_with_ids(
            library.path(),
            &platform,
            &HostPathResolver::default(),
            selection(&preview),
            || "pinned-emulator-game".into(),
        )
        .unwrap();

        assert_eq!(
            report.games[0].emulator_id.as_deref(),
            Some("fixture-emulator")
        );
        let persisted = PlatformDocument::load(&platform).unwrap();
        assert_eq!(
            persisted
                .library()
                .games
                .iter()
                .find(|game| game.id == "pinned-emulator-game")
                .and_then(|game| game.emulator_id.as_deref()),
            Some("fixture-emulator")
        );
    }

    #[test]
    fn direct_launch_sentinel_is_valid_without_emulator_configuration() {
        let (library, _) = library();
        let source_directory = tempfile::tempdir().unwrap();
        let source = source_directory.path().join("Direct.rom");
        fs::write(&source, b"rom").unwrap();
        let mut import_request = request(
            vec![ImportLocation {
                path: source,
                kind: ImportLocationKind::File,
            }],
            ImportFilePolicy::Leave,
        );
        import_request.emulator_id = Some(UNASSIGNED_EMULATOR_ID.to_ascii_uppercase());

        let preview =
            preview_manual_import(library.path(), &HostPathResolver::default(), import_request)
                .unwrap();
        assert_eq!(
            preview.request.emulator_id.as_deref(),
            Some(UNASSIGNED_EMULATOR_ID)
        );
    }

    #[test]
    fn unknown_or_empty_emulator_selection_is_rejected_during_preview() {
        let (library, _) = library();
        configure_fixture_emulator(library.path());
        let source_directory = tempfile::tempdir().unwrap();
        let source = source_directory.path().join("Unknown Emulator.rom");
        fs::write(&source, b"rom").unwrap();
        let location = ImportLocation {
            path: source,
            kind: ImportLocationKind::File,
        };

        let mut unknown = request(vec![location.clone()], ImportFilePolicy::Leave);
        unknown.emulator_id = Some("missing-emulator".into());
        assert!(matches!(
            preview_manual_import(library.path(), &HostPathResolver::default(), unknown),
            Err(ImportError::UnknownEmulator { id }) if id == "missing-emulator"
        ));

        let mut empty = request(vec![location], ImportFilePolicy::Leave);
        empty.emulator_id = Some("  ".into());
        assert!(matches!(
            preview_manual_import(library.path(), &HostPathResolver::default(), empty),
            Err(ImportError::EmptyEmulatorId)
        ));
    }

    #[test]
    fn move_deletes_sources_only_after_verified_commit() {
        let (library, platform) = library();
        let source_directory = tempfile::tempdir().unwrap();
        let source = source_directory.path().join("Move Me.rom");
        let companion = source_directory.path().join("Move Me.dat");
        fs::write(&source, b"move bytes").unwrap();
        fs::write(&companion, b"move companion bytes").unwrap();
        let mut import_request = request(
            vec![ImportLocation {
                path: source.clone(),
                kind: ImportLocationKind::File,
            }],
            ImportFilePolicy::Move,
        );
        import_request.copy_files_with_same_name = true;
        let preview =
            preview_manual_import(library.path(), &HostPathResolver::default(), import_request)
                .unwrap();
        let report = execute_manual_import_with_ids(
            library.path(),
            &platform,
            &HostPathResolver::default(),
            selection(&preview),
            || "moved-game".into(),
        )
        .unwrap();

        assert_eq!(report.moved_sources.len(), 2);
        assert!(report.moved_sources.contains(&source));
        assert!(report.moved_sources.contains(&companion));
        assert!(report.cleanup_warnings.is_empty());
        assert!(!source.exists());
        assert!(!companion.exists());
        assert_eq!(
            fs::read(library.path().join("Games/Fixture Console/Move Me.rom")).unwrap(),
            b"move bytes"
        );
        assert_eq!(
            fs::read(library.path().join("Games/Fixture Console/Move Me.dat")).unwrap(),
            b"move companion bytes"
        );
    }

    #[test]
    fn changed_preview_is_refused_before_any_write() {
        let (library, platform) = library();
        let source_directory = tempfile::tempdir().unwrap();
        let source = source_directory.path().join("Only.rom");
        fs::write(&source, b"rom").unwrap();
        let preview = preview_manual_import(
            library.path(),
            &HostPathResolver::default(),
            request(
                vec![ImportLocation {
                    path: source,
                    kind: ImportLocationKind::File,
                }],
                ImportFilePolicy::Copy,
            ),
        )
        .unwrap();
        let mut changed = selection(&preview);
        changed.rows.clear();
        assert!(matches!(
            execute_manual_import_with_ids(
                library.path(),
                &platform,
                &HostPathResolver::default(),
                changed,
                || "unused".into(),
            ),
            Err(ImportError::PreviewChanged)
        ));
        assert!(!library.path().join("Games").exists());
    }
}
