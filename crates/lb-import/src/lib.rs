use lb_domain::{is_unassigned_emulator_id, EmulatorConfiguration, Game, UNASSIGNED_EMULATOR_ID};
use lb_platform::{
    portable_storage_name, portable_stored_path, HostPathResolver, LaunchPathError,
    LaunchPathResolver, PlatformPathError,
};
use lb_storage::{
    FileRevision, LaunchBoxDataIndex, LibraryTransaction, NewGame, PlatformDocument, StorageError,
    TransactionError,
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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
}

impl ManualImportPreviewRow {
    pub fn is_importable(&self, duplicate_policy: ImportDuplicatePolicy) -> bool {
        self.state == ImportRowState::Ready
            || (self.state == ImportRowState::Duplicate
                && duplicate_policy == ImportDuplicatePolicy::Import)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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
    let existing_paths = existing_game_paths(&launchbox_root, resolver, library.games());
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

        let (destination_path, application_path) = match request.file_policy {
            ImportFilePolicy::Leave => (
                None,
                resolver.stored_path_for_host_path(&launchbox_root, &source)?,
            ),
            ImportFilePolicy::Copy | ImportFilePolicy::Move => {
                let file_name =
                    source
                        .file_name()
                        .ok_or_else(|| ImportError::InvalidSourcePath {
                            path: source.clone(),
                        })?;
                let file_name =
                    file_name
                        .to_str()
                        .ok_or_else(|| LaunchPathError::NonUnicodeHostPath {
                            path: source.clone(),
                        })?;
                let file_name = portable_storage_name(file_name)?;
                let destination = destination_directory.join(&file_name);
                let destination_key = file_name.to_lowercase();
                let destination_eligible = state == ImportRowState::Ready
                    || (state == ImportRowState::Duplicate
                        && request.duplicate_policy == ImportDuplicatePolicy::Import);
                if destination_eligible {
                    if destination.exists()
                        || case_insensitive_name_exists(&destination_directory, &file_name)?
                    {
                        state = ImportRowState::DestinationExists;
                        message = format!("Destination already exists: {}", destination.display());
                    } else if !planned_destinations.insert(destination_key) {
                        state = ImportRowState::DestinationExists;
                        message = "Another selected file has the same portable destination name"
                            .to_string();
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
        });
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
    let preview = preview_manual_import(launchbox_root, resolver, selection.request.clone())?;
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
    for (row, title) in &imports {
        games.push(document.add_game(NewGame {
            id: next_id(),
            title: title.clone(),
            platform: preview.request.platform.clone(),
            application_path: row.application_path.clone(),
            emulator_id: preview.request.emulator_id.clone(),
        })?);
    }

    for (row, _) in &imports {
        if let Some(destination) = &row.destination_path {
            let parent = destination
                .parent()
                .ok_or_else(|| ImportError::InvalidSourcePath {
                    path: destination.clone(),
                })?;
            fs::create_dir_all(parent).map_err(|source| ImportError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
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
            transaction.stage_file_copy(
                &row.source_path,
                row.destination_path
                    .as_ref()
                    .expect("copy/move preview row has a destination"),
            )?;
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
            let destination = row
                .destination_path
                .as_ref()
                .expect("move preview row has a destination");
            let source_revision = match FileRevision::read(&row.source_path) {
                Ok(revision) => revision,
                Err(error) => {
                    cleanup_warnings.push(format!(
                        "Imported {}, but could not verify the source before cleanup: {error}",
                        row.source_path.display()
                    ));
                    continue;
                }
            };
            let destination_revision = match FileRevision::read(destination) {
                Ok(revision) => revision,
                Err(error) => {
                    cleanup_warnings.push(format!(
                        "Imported {}, but could not verify its committed destination before cleanup: {error}",
                        row.source_path.display()
                    ));
                    continue;
                }
            };
            if source_revision != destination_revision {
                cleanup_warnings.push(format!(
                    "Kept {} because the committed copy did not match it",
                    row.source_path.display()
                ));
                continue;
            }
            match fs::remove_file(&row.source_path) {
                Ok(()) => moved_sources.push(row.source_path.clone()),
                Err(error) => cleanup_warnings.push(format!(
                    "Imported {}, but could not remove the source: {error}",
                    row.source_path.display()
                )),
            }
        }
    }

    Ok(ManualImportReport {
        games,
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
    let value = if use_folder_names {
        path.parent().and_then(Path::file_name)
    } else {
        path.file_stem().or_else(|| path.file_name())
    };
    value
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn existing_game_paths<'a>(
    launchbox_root: &Path,
    resolver: &HostPathResolver,
    games: impl Iterator<Item = &'a Game>,
) -> BTreeSet<PathBuf> {
    games
        .filter_map(|game| {
            resolver
                .resolve(launchbox_root, &game.application_path)
                .ok()
        })
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
    #[error("at least one importable row must be selected")]
    EmptySelection,
    #[error("the committed transaction did not report a platform XML backup")]
    MissingPlatformBackup,
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
        fs::write(&source, b"move bytes").unwrap();
        let preview = preview_manual_import(
            library.path(),
            &HostPathResolver::default(),
            request(
                vec![ImportLocation {
                    path: source.clone(),
                    kind: ImportLocationKind::File,
                }],
                ImportFilePolicy::Move,
            ),
        )
        .unwrap();
        let report = execute_manual_import_with_ids(
            library.path(),
            &platform,
            &HostPathResolver::default(),
            selection(&preview),
            || "moved-game".into(),
        )
        .unwrap();

        assert_eq!(
            report.moved_sources,
            vec![fs::canonicalize(&source).unwrap_or(source.clone())]
        );
        assert!(report.cleanup_warnings.is_empty());
        assert!(!source.exists());
        assert_eq!(
            fs::read(library.path().join("Games/Fixture Console/Move Me.rom")).unwrap(),
            b"move bytes"
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
