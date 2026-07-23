#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("QtCore/QAbstractListModel");
        type QAbstractListModel;

        include!("cxx-qt-lib/qbytearray.h");
        type QByteArray = cxx_qt_lib::QByteArray;
        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;
        include!("cxx-qt-lib/core/qlist/qlist_i32.h");
        type QList_i32 = cxx_qt_lib::QList<i32>;
        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        #[qproperty(QString, library_path)]
        #[qproperty(QString, library_name)]
        #[qproperty(QString, status_message)]
        #[qproperty(QString, search_text)]
        #[qproperty(QString, platform_filter)]
        #[qproperty(bool, loading)]
        #[qproperty(bool, writing)]
        #[qproperty(bool, launching)]
        #[qproperty(bool, launch_session_active)]
        #[qproperty(bool, last_launch_succeeded)]
        #[qproperty(bool, write_conflict)]
        #[qproperty(i32, game_count)]
        #[qproperty(i32, filtered_count)]
        #[qproperty(i32, platform_entry_count)]
        #[qproperty(i32, platform_revision)]
        #[qproperty(i32, pending_recovery_count)]
        #[qproperty(i32, delete_blocker_count)]
        #[qproperty(QString, delete_blocker_summary)]
        #[qproperty(QString, last_added_game_id)]
        #[qproperty(QString, last_launch_game_id)]
        #[qproperty(QString, last_launch_target_id)]
        #[qproperty(QString, path_mapping_settings_path)]
        #[qproperty(i32, path_mapping_count)]
        type LibraryController = super::LibraryControllerRust;

        #[qinvokable]
        fn initialize_host_path_mappings(self: Pin<&mut LibraryController>) -> bool;

        #[qinvokable]
        fn load_fixture(self: Pin<&mut LibraryController>);

        #[qinvokable]
        fn load_library(self: Pin<&mut LibraryController>, path: QString);

        #[qinvokable]
        fn configure_windows_drive_mapping(
            self: Pin<&mut LibraryController>,
            drive: QString,
            host_root: QString,
        ) -> bool;

        #[qinvokable]
        fn configure_windows_unc_mapping(
            self: Pin<&mut LibraryController>,
            server: QString,
            share: QString,
            host_root: QString,
        ) -> bool;

        #[qinvokable]
        fn save_windows_drive_mapping(
            self: Pin<&mut LibraryController>,
            drive: QString,
            host_root: QString,
        ) -> bool;

        #[qinvokable]
        fn save_windows_unc_mapping(
            self: Pin<&mut LibraryController>,
            server: QString,
            share: QString,
            host_root: QString,
        ) -> bool;

        #[qinvokable]
        fn remove_path_mapping(self: Pin<&mut LibraryController>, index: i32) -> bool;

        #[qinvokable]
        fn path_mapping_kind_at(self: &LibraryController, index: i32) -> QString;

        #[qinvokable]
        fn path_mapping_windows_root_at(self: &LibraryController, index: i32) -> QString;

        #[qinvokable]
        fn path_mapping_host_root_at(self: &LibraryController, index: i32) -> QString;

        #[qinvokable]
        fn emulator_entry_count(self: &LibraryController) -> i32;

        #[qinvokable]
        fn emulator_id_at(self: &LibraryController, index: i32) -> QString;

        #[qinvokable]
        fn emulator_title_at(self: &LibraryController, index: i32) -> QString;

        #[qinvokable]
        fn apply_filters(
            self: Pin<&mut LibraryController>,
            search_text: QString,
            platform: QString,
        );

        #[qinvokable]
        fn save_game(
            self: Pin<&mut LibraryController>,
            row: i32,
            game_id: QString,
            edit_payload: QString,
        );

        #[qinvokable]
        fn add_game(
            self: Pin<&mut LibraryController>,
            title: QString,
            application_path: QString,
            platform: QString,
        );

        #[qinvokable]
        fn add_platform(self: Pin<&mut LibraryController>, name: QString, scrape_as: QString);

        #[qinvokable]
        fn delete_platform(self: Pin<&mut LibraryController>, name: QString);

        #[qinvokable]
        fn delete_game(self: Pin<&mut LibraryController>, row: i32, game_id: QString);

        #[qinvokable]
        fn launch_game(self: Pin<&mut LibraryController>, row: i32, game_id: QString);

        #[qinvokable]
        fn launch_additional_application(
            self: Pin<&mut LibraryController>,
            row: i32,
            game_id: QString,
            application_id: QString,
        );

        #[qinvokable]
        fn additional_application_count(
            self: &LibraryController,
            row: i32,
            game_id: QString,
        ) -> i32;

        #[qinvokable]
        fn additional_application_id_at(
            self: &LibraryController,
            row: i32,
            game_id: QString,
            index: i32,
        ) -> QString;

        #[qinvokable]
        fn additional_application_name_at(
            self: &LibraryController,
            row: i32,
            game_id: QString,
            index: i32,
        ) -> QString;

        #[qinvokable]
        fn alternate_name_count(self: &LibraryController, row: i32, game_id: QString) -> i32;

        #[qinvokable]
        fn alternate_name_name_at(
            self: &LibraryController,
            row: i32,
            game_id: QString,
            index: i32,
        ) -> QString;

        #[qinvokable]
        fn alternate_name_region_at(
            self: &LibraryController,
            row: i32,
            game_id: QString,
            index: i32,
        ) -> QString;

        #[qinvokable]
        fn custom_field_count(self: &LibraryController, row: i32, game_id: QString) -> i32;

        #[qinvokable]
        fn custom_field_name_at(
            self: &LibraryController,
            row: i32,
            game_id: QString,
            index: i32,
        ) -> QString;

        #[qinvokable]
        fn custom_field_value_at(
            self: &LibraryController,
            row: i32,
            game_id: QString,
            index: i32,
        ) -> QString;

        #[qinvokable]
        fn dismiss_delete_blocker(self: Pin<&mut LibraryController>);

        #[qinvokable]
        fn recover_pending_changes(self: Pin<&mut LibraryController>);

        #[qinvokable]
        fn reload_library(self: Pin<&mut LibraryController>);

        #[qinvokable]
        fn report_model_smoke_success(self: &LibraryController, rows: i32);

        #[qinvokable]
        fn report_load_smoke_success(
            self: &LibraryController,
            games: i32,
            platforms: i32,
            heartbeats: i32,
        );

        #[qinvokable]
        fn report_state_edit_smoke_success(self: &LibraryController, game_id: QString) -> bool;

        #[qinvokable]
        fn report_title_edit_smoke_success(
            self: &LibraryController,
            game_id: QString,
            expected_title: QString,
        ) -> bool;

        #[qinvokable]
        fn report_crud_smoke_success(
            self: &LibraryController,
            added_game_id: QString,
            blocked_references: i32,
        ) -> bool;

        #[qinvokable]
        fn report_platform_crud_smoke_success(
            self: &LibraryController,
            platform_name: QString,
            blocked_references: i32,
        ) -> bool;

        #[qinvokable]
        fn report_launch_smoke_success(self: &LibraryController, game_id: QString) -> bool;

        #[qinvokable]
        fn report_additional_application_launch_smoke_success(
            self: &LibraryController,
            game_id: QString,
            application_id: QString,
        ) -> bool;

        #[qinvokable]
        fn report_path_mapping_smoke_success(self: &LibraryController, expected_count: i32)
            -> bool;

        #[qinvokable]
        fn row_for_game_id(self: &LibraryController, game_id: QString) -> i32;

        #[qinvokable]
        fn game_id_at(self: &LibraryController, row: i32) -> QString;

        #[qinvokable]
        fn platform_name_at(self: &LibraryController, index: i32) -> QString;

        #[qinvokable]
        fn platform_game_count_at(self: &LibraryController, index: i32) -> i32;
    }

    unsafe extern "RustQt" {
        #[qinvokable]
        #[cxx_name = "rowCount"]
        #[cxx_override]
        fn row_count(self: &LibraryController, parent: &QModelIndex) -> i32;

        #[qinvokable]
        #[cxx_override]
        fn data(self: &LibraryController, index: &QModelIndex, role: i32) -> QVariant;

        #[qinvokable]
        #[cxx_name = "roleNames"]
        #[cxx_override]
        fn role_names(self: &LibraryController) -> QHash_i32_QByteArray;

        #[cxx_name = "beginResetModel"]
        #[inherit]
        fn begin_reset_model(self: Pin<&mut LibraryController>);

        #[cxx_name = "endResetModel"]
        #[inherit]
        fn end_reset_model(self: Pin<&mut LibraryController>);

        #[cxx_name = "beginInsertRows"]
        #[inherit]
        fn begin_insert_rows(
            self: Pin<&mut LibraryController>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );

        #[cxx_name = "endInsertRows"]
        #[inherit]
        fn end_insert_rows(self: Pin<&mut LibraryController>);

        #[cxx_name = "beginRemoveRows"]
        #[inherit]
        fn begin_remove_rows(
            self: Pin<&mut LibraryController>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );

        #[cxx_name = "endRemoveRows"]
        #[inherit]
        fn end_remove_rows(self: Pin<&mut LibraryController>);

        #[cxx_name = "index"]
        #[inherit]
        fn model_index(
            self: &LibraryController,
            row: i32,
            column: i32,
            parent: &QModelIndex,
        ) -> QModelIndex;

        #[cxx_name = "dataChanged"]
        #[inherit]
        fn emit_data_changed(
            self: Pin<&mut LibraryController>,
            top_left: &QModelIndex,
            bottom_right: &QModelIndex,
            roles: &QList_i32,
        );
    }

    impl cxx_qt::Threading for LibraryController {}
}

use chrono::{DateTime, Local};
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{
    QByteArray, QHash, QHashPair_i32_QByteArray, QList, QModelIndex, QString, QVariant,
};
use lb_domain::{
    AdditionalApplication, AlternateName, CustomField, EmulatorConfiguration, Game,
    GameLaunchConfiguration, GameMetadata, Mount, NavigationMetadata, PlatformDefinition,
    UNASSIGNED_EMULATOR_ID,
};
use lb_platform::{
    default_host_path_mappings_path, default_platform_folders, execute_launch_sequence,
    platform_document_file_name, prepare_game_launch_sequence_with_mounts_context_and_resolver,
    prepare_selected_additional_application_sequence_with_mounts_context_and_resolver,
    ArchiveExtractor, HostPathMappings, HostPathResolver, LaunchContext, LaunchKind,
    LaunchSequence, LaunchSequenceEvent, LaunchSequenceReport, LaunchTarget,
};
use lb_query::{filter_game_indices, GameFilter};
use lb_storage::{
    find_game_references, find_platform_references, pending_transaction_manifests,
    recover_pending_transactions, AuxiliaryDocument, GameReference, IndexedPlatformRecordEdit,
    LaunchBoxDataIndex, LibraryIndex, LibraryTransaction, NewGame, PlatformDocument,
    PlatformReference, StorageError, TransactionError,
};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};
use uuid::Uuid;

const FIXTURE: &str =
    include_str!("../../../fixtures/launchbox/Data/Platforms/Fixture Console.xml");

type RoleNames = QHash<QHashPair_i32_QByteArray>;

const DISPLAY_ROLE: i32 = 0;
const GAME_ID_ROLE: i32 = 257;
const GAME_TITLE_ROLE: i32 = 258;
const GAME_PLATFORM_ROLE: i32 = 259;
const GAME_FAVORITE_ROLE: i32 = 260;
const GAME_COMPLETED_ROLE: i32 = 261;
const GAME_PLAY_COUNT_ROLE: i32 = 262;
const GAME_STAR_RATING_ROLE: i32 = 263;
const GAME_ADDITIONAL_APPLICATION_COUNT_ROLE: i32 = 264;
const GAME_SORT_TITLE_ROLE: i32 = 265;
const GAME_NOTES_ROLE: i32 = 266;
const GAME_DEVELOPER_ROLE: i32 = 267;
const GAME_GENRE_ROLE: i32 = 268;
const GAME_MAX_PLAYERS_ROLE: i32 = 269;
const GAME_PLAY_MODE_ROLE: i32 = 270;
const GAME_PROGRESS_ROLE: i32 = 271;
const GAME_PUBLISHER_ROLE: i32 = 272;
const GAME_RATING_ROLE: i32 = 273;
const GAME_REGION_ROLE: i32 = 274;
const GAME_RELEASE_DATE_ROLE: i32 = 275;
const GAME_RELEASE_TYPE_ROLE: i32 = 276;
const GAME_SERIES_ROLE: i32 = 277;
const GAME_SOURCE_ROLE: i32 = 278;
const GAME_STATUS_ROLE: i32 = 279;
const GAME_VERSION_ROLE: i32 = 280;
const GAME_WIKIPEDIA_URL_ROLE: i32 = 281;
const GAME_APPLICATION_PATH_ROLE: i32 = 282;
const GAME_COMMAND_LINE_ROLE: i32 = 283;
const GAME_EMULATOR_ID_ROLE: i32 = 284;
const GAME_USE_DOS_BOX_ROLE: i32 = 285;
const GAME_CUSTOM_DOS_BOX_VERSION_PATH_ROLE: i32 = 286;
const GAME_DOS_BOX_CONFIGURATION_PATH_ROLE: i32 = 287;
const GAME_USE_SCUMM_VM_ROLE: i32 = 288;
const GAME_SCUMM_VM_ASPECT_CORRECTION_ROLE: i32 = 289;
const GAME_SCUMM_VM_FULLSCREEN_ROLE: i32 = 290;
const GAME_SCUMM_VM_GAME_DATA_FOLDER_PATH_ROLE: i32 = 291;
const GAME_SCUMM_VM_GAME_TYPE_ROLE: i32 = 292;

const GAME_ROLES: [(i32, &str); 36] = [
    (GAME_ID_ROLE, "gameId"),
    (GAME_TITLE_ROLE, "gameTitle"),
    (GAME_PLATFORM_ROLE, "gamePlatform"),
    (GAME_FAVORITE_ROLE, "gameFavorite"),
    (GAME_COMPLETED_ROLE, "gameCompleted"),
    (GAME_PLAY_COUNT_ROLE, "gamePlayCount"),
    (GAME_STAR_RATING_ROLE, "gameStarRating"),
    (
        GAME_ADDITIONAL_APPLICATION_COUNT_ROLE,
        "gameAdditionalApplicationCount",
    ),
    (GAME_SORT_TITLE_ROLE, "gameSortTitle"),
    (GAME_NOTES_ROLE, "gameNotes"),
    (GAME_DEVELOPER_ROLE, "gameDeveloper"),
    (GAME_GENRE_ROLE, "gameGenre"),
    (GAME_MAX_PLAYERS_ROLE, "gameMaxPlayers"),
    (GAME_PLAY_MODE_ROLE, "gamePlayMode"),
    (GAME_PROGRESS_ROLE, "gameProgress"),
    (GAME_PUBLISHER_ROLE, "gamePublisher"),
    (GAME_RATING_ROLE, "gameRating"),
    (GAME_REGION_ROLE, "gameRegion"),
    (GAME_RELEASE_DATE_ROLE, "gameReleaseDate"),
    (GAME_RELEASE_TYPE_ROLE, "gameReleaseType"),
    (GAME_SERIES_ROLE, "gameSeries"),
    (GAME_SOURCE_ROLE, "gameSource"),
    (GAME_STATUS_ROLE, "gameStatus"),
    (GAME_VERSION_ROLE, "gameVersion"),
    (GAME_WIKIPEDIA_URL_ROLE, "gameWikipediaUrl"),
    (GAME_APPLICATION_PATH_ROLE, "gameApplicationPath"),
    (GAME_COMMAND_LINE_ROLE, "gameCommandLine"),
    (GAME_EMULATOR_ID_ROLE, "gameEmulatorId"),
    (GAME_USE_DOS_BOX_ROLE, "gameUseDosBox"),
    (
        GAME_CUSTOM_DOS_BOX_VERSION_PATH_ROLE,
        "gameCustomDosBoxVersionPath",
    ),
    (
        GAME_DOS_BOX_CONFIGURATION_PATH_ROLE,
        "gameDosBoxConfigurationPath",
    ),
    (GAME_USE_SCUMM_VM_ROLE, "gameUseScummVm"),
    (
        GAME_SCUMM_VM_ASPECT_CORRECTION_ROLE,
        "gameScummVmAspectCorrection",
    ),
    (GAME_SCUMM_VM_FULLSCREEN_ROLE, "gameScummVmFullscreen"),
    (
        GAME_SCUMM_VM_GAME_DATA_FOLDER_PATH_ROLE,
        "gameScummVmGameDataFolderPath",
    ),
    (GAME_SCUMM_VM_GAME_TYPE_ROLE, "gameScummVmGameType"),
];

const EDITABLE_GAME_ROLES: [i32; 32] = [
    GAME_TITLE_ROLE,
    GAME_SORT_TITLE_ROLE,
    GAME_NOTES_ROLE,
    GAME_DEVELOPER_ROLE,
    GAME_GENRE_ROLE,
    GAME_MAX_PLAYERS_ROLE,
    GAME_PLAY_MODE_ROLE,
    GAME_PROGRESS_ROLE,
    GAME_PUBLISHER_ROLE,
    GAME_RATING_ROLE,
    GAME_REGION_ROLE,
    GAME_RELEASE_DATE_ROLE,
    GAME_RELEASE_TYPE_ROLE,
    GAME_SERIES_ROLE,
    GAME_SOURCE_ROLE,
    GAME_STATUS_ROLE,
    GAME_VERSION_ROLE,
    GAME_WIKIPEDIA_URL_ROLE,
    GAME_FAVORITE_ROLE,
    GAME_COMPLETED_ROLE,
    GAME_STAR_RATING_ROLE,
    GAME_APPLICATION_PATH_ROLE,
    GAME_COMMAND_LINE_ROLE,
    GAME_EMULATOR_ID_ROLE,
    GAME_USE_DOS_BOX_ROLE,
    GAME_CUSTOM_DOS_BOX_VERSION_PATH_ROLE,
    GAME_DOS_BOX_CONFIGURATION_PATH_ROLE,
    GAME_USE_SCUMM_VM_ROLE,
    GAME_SCUMM_VM_ASPECT_CORRECTION_ROLE,
    GAME_SCUMM_VM_FULLSCREEN_ROLE,
    GAME_SCUMM_VM_GAME_DATA_FOLDER_PATH_ROLE,
    GAME_SCUMM_VM_GAME_TYPE_ROLE,
];

#[derive(Default)]
pub struct LibraryControllerRust {
    library_path: QString,
    library_name: QString,
    status_message: QString,
    search_text: QString,
    platform_filter: QString,
    loading: bool,
    writing: bool,
    launching: bool,
    launch_session_active: bool,
    last_launch_succeeded: bool,
    write_conflict: bool,
    game_count: i32,
    filtered_count: i32,
    platform_entry_count: i32,
    platform_revision: i32,
    pending_recovery_count: i32,
    delete_blocker_count: i32,
    delete_blocker_summary: QString,
    last_added_game_id: QString,
    last_launch_game_id: QString,
    last_launch_target_id: QString,
    path_mapping_settings_path: QString,
    path_mapping_count: i32,
    games: Vec<Game>,
    game_sources: Vec<PathBuf>,
    additional_applications_by_game: BTreeMap<String, Vec<AdditionalApplication>>,
    mounts_by_game: BTreeMap<String, Vec<Mount>>,
    alternate_names_by_game: BTreeMap<String, Vec<AlternateName>>,
    custom_fields_by_game: BTreeMap<String, Vec<CustomField>>,
    filtered_indices: Vec<usize>,
    platform_counts: Vec<PlatformCount>,
    platform_names: Vec<String>,
    platform_sources: BTreeMap<String, PathBuf>,
    library_root: Option<PathBuf>,
    launchbox_root: Option<PathBuf>,
    emulator_configuration: Option<EmulatorConfiguration>,
    path_mapping_settings_file: Option<PathBuf>,
    path_mappings: HostPathMappings,
    path_mappings_initialized: bool,
    path_resolver: HostPathResolver,
    request_generation: u64,
    model_reset_notifications: u64,
    data_change_notifications: u64,
    row_insert_notifications: u64,
    row_remove_notifications: u64,
    launch_notifications: u64,
    session_stats_writes: u64,
    session_stats_error: Option<String>,
}

#[derive(Clone, Debug)]
struct PlatformCount {
    name: String,
    count: usize,
}

struct LoadedLibrary {
    path: String,
    root: PathBuf,
    name: String,
    message: String,
    games: Vec<Game>,
    game_sources: Vec<PathBuf>,
    additional_applications_by_game: BTreeMap<String, Vec<AdditionalApplication>>,
    mounts_by_game: BTreeMap<String, Vec<Mount>>,
    alternate_names_by_game: BTreeMap<String, Vec<AlternateName>>,
    custom_fields_by_game: BTreeMap<String, Vec<CustomField>>,
    platform_names: Vec<String>,
    platform_sources: BTreeMap<String, PathBuf>,
    pending_recovery_count: usize,
    launchbox_root: Option<PathBuf>,
    emulator_configuration: Option<EmulatorConfiguration>,
}

struct LibraryReplacement {
    games: Vec<Game>,
    game_sources: Vec<PathBuf>,
    additional_applications_by_game: BTreeMap<String, Vec<AdditionalApplication>>,
    mounts_by_game: BTreeMap<String, Vec<Mount>>,
    alternate_names_by_game: BTreeMap<String, Vec<AlternateName>>,
    custom_fields_by_game: BTreeMap<String, Vec<CustomField>>,
    platform_names: Vec<String>,
    platform_sources: BTreeMap<String, PathBuf>,
    library_root: Option<PathBuf>,
    launchbox_root: Option<PathBuf>,
    emulator_configuration: Option<EmulatorConfiguration>,
    name: String,
    message: String,
    pending_recovery_count: usize,
}

impl LoadedLibrary {
    fn load(path: String) -> Result<Self, String> {
        let started = Instant::now();
        if Path::new(&path).is_file() {
            let library = LibraryIndex::load(&path).map_err(|error| error.to_string())?;
            let platform_count = library.platforms().len();
            let (games, game_sources) = collect_games_and_sources(&library);
            let additional_applications_by_game = collect_additional_applications_by_game(&library);
            let additional_application_count = additional_applications_by_game
                .values()
                .map(Vec::len)
                .sum::<usize>();
            let mounts_by_game = collect_mounts_by_game(&library);
            let mount_count = mounts_by_game.values().map(Vec::len).sum::<usize>();
            let alternate_names_by_game = collect_alternate_names_by_game(&library);
            let custom_fields_by_game = collect_custom_fields_by_game(&library);
            let (platform_names, platform_sources) = platform_state_from_library(&library);
            let name = library
                .platforms()
                .first()
                .map(|platform| platform.name.clone())
                .unwrap_or_else(|| "LaunchBox Library".to_string());
            let message = format!(
                "Loaded {} games, {additional_application_count} additional applications, and {mount_count} DOSBox mounts from {platform_count} platform file in {:.3}s.",
                games.len(),
                started.elapsed().as_secs_f64()
            );
            let root = Path::new(&path)
                .parent()
                .unwrap_or(Path::new("."))
                .to_path_buf();
            let pending_recovery_count = pending_transaction_manifests(&root)
                .map_err(|error| error.to_string())?
                .len();
            return Ok(Self {
                path,
                root,
                name,
                message,
                games,
                game_sources,
                additional_applications_by_game,
                mounts_by_game,
                alternate_names_by_game,
                custom_fields_by_game,
                platform_names,
                platform_sources,
                pending_recovery_count,
                launchbox_root: None,
                emulator_configuration: None,
            });
        }

        let data = LaunchBoxDataIndex::load(&path).map_err(|error| error.to_string())?;
        let emulator_configuration = data.emulator_configuration().cloned();
        let (platform_names, platform_sources) = platform_state_from_data(&data)?;
        let platform_count = platform_names.len();
        let (games, game_sources) = collect_games_and_sources(data.platforms());
        let additional_applications_by_game =
            collect_additional_applications_by_game(data.platforms());
        let additional_application_count = additional_applications_by_game
            .values()
            .map(Vec::len)
            .sum::<usize>();
        let mounts_by_game = collect_mounts_by_game(data.platforms());
        let mount_count = mounts_by_game.values().map(Vec::len).sum::<usize>();
        let alternate_names_by_game = collect_alternate_names_by_game(data.platforms());
        let custom_fields_by_game = collect_custom_fields_by_game(data.platforms());
        let playlist_count = data.playlists().len();
        let emulator_count = data
            .emulator_configuration()
            .map(|configuration| configuration.emulators.len())
            .unwrap_or_default();
        let message = format!(
            "Loaded {} games, {additional_application_count} additional applications, {mount_count} DOSBox mounts, {playlist_count} playlists, and {emulator_count} emulators from {platform_count} platforms in {:.3}s.",
            games.len(),
            started.elapsed().as_secs_f64()
        );
        let root = PathBuf::from(&path);
        let launchbox_root = Some(root.clone());
        let pending_recovery_count = pending_transaction_manifests(&root)
            .map_err(|error| error.to_string())?
            .len();
        Ok(Self {
            path,
            root,
            name: format!("LaunchBox Library ({platform_count} platforms)"),
            message,
            games,
            game_sources,
            additional_applications_by_game,
            mounts_by_game,
            alternate_names_by_game,
            custom_fields_by_game,
            platform_names,
            platform_sources,
            pending_recovery_count,
            launchbox_root,
            emulator_configuration,
        })
    }
}

fn platform_key(name: &str) -> String {
    name.to_lowercase()
}

fn platform_state_from_library(library: &LibraryIndex) -> (Vec<String>, BTreeMap<String, PathBuf>) {
    let mut names = library
        .platforms()
        .iter()
        .map(|platform| platform.name.clone())
        .collect::<Vec<_>>();
    names.sort_by_key(|name| platform_key(name));
    names.dedup_by(|left, right| platform_key(left) == platform_key(right));
    let sources = library
        .platforms()
        .iter()
        .map(|platform| (platform_key(&platform.name), platform.source_path.clone()))
        .collect();
    (names, sources)
}

fn platform_state_from_data(
    data: &LaunchBoxDataIndex,
) -> Result<(Vec<String>, BTreeMap<String, PathBuf>), String> {
    let mut names = Vec::new();
    let mut sources = BTreeMap::new();
    let mut claimed_sources = std::collections::BTreeSet::new();
    if let Some(catalog) = data.platform_catalog() {
        for platform in &catalog.platforms {
            let name = platform.metadata.name.clone();
            let expected = data
                .data_root()
                .join("Platforms")
                .join(platform_document_file_name(&name).map_err(|error| error.to_string())?);
            let source = data
                .platforms()
                .platforms()
                .iter()
                .find(|document| platform_key(&document.name) == platform_key(&name))
                .map(|document| document.source_path.clone())
                .or_else(|| expected.is_file().then_some(expected));
            if let Some(source) = source {
                claimed_sources.insert(source.clone());
                sources.insert(platform_key(&name), source);
            }
            names.push(name);
        }
    }
    for document in data.platforms().platforms() {
        if claimed_sources.contains(&document.source_path) {
            continue;
        }
        let key = platform_key(&document.name);
        if !names.iter().any(|name| platform_key(name) == key) {
            names.push(document.name.clone());
        }
        sources
            .entry(key)
            .or_insert_with(|| document.source_path.clone());
    }
    names.sort_by_key(|name| platform_key(name));
    Ok((names, sources))
}

fn collect_games_and_sources(library: &LibraryIndex) -> (Vec<Game>, Vec<PathBuf>) {
    let game_count = library.games().count();
    let mut games = Vec::with_capacity(game_count);
    let mut sources = Vec::with_capacity(game_count);
    for platform in library.platforms() {
        for game in &platform.games {
            games.push(game.clone());
            sources.push(platform.source_path.clone());
        }
    }
    (games, sources)
}

fn collect_additional_applications_by_game(
    library: &LibraryIndex,
) -> BTreeMap<String, Vec<AdditionalApplication>> {
    index_additional_applications(library.additional_applications())
}

fn collect_mounts_by_game(library: &LibraryIndex) -> BTreeMap<String, Vec<Mount>> {
    index_mounts(library.mounts())
}

fn collect_alternate_names_by_game(library: &LibraryIndex) -> BTreeMap<String, Vec<AlternateName>> {
    index_alternate_names(library.alternate_names())
}

fn collect_custom_fields_by_game(library: &LibraryIndex) -> BTreeMap<String, Vec<CustomField>> {
    index_custom_fields(library.custom_fields())
}

fn index_mounts<'a>(mounts: impl IntoIterator<Item = &'a Mount>) -> BTreeMap<String, Vec<Mount>> {
    let mut by_game = BTreeMap::<String, Vec<Mount>>::new();
    for mount in mounts {
        by_game
            .entry(mount.game_id.clone())
            .or_default()
            .push(mount.clone());
    }
    by_game
}

fn index_alternate_names<'a>(
    alternate_names: impl IntoIterator<Item = &'a AlternateName>,
) -> BTreeMap<String, Vec<AlternateName>> {
    let mut by_game = BTreeMap::<String, Vec<AlternateName>>::new();
    for alternate_name in alternate_names {
        by_game
            .entry(alternate_name.game_id.clone())
            .or_default()
            .push(alternate_name.clone());
    }
    by_game
}

fn index_custom_fields<'a>(
    custom_fields: impl IntoIterator<Item = &'a CustomField>,
) -> BTreeMap<String, Vec<CustomField>> {
    let mut by_game = BTreeMap::<String, Vec<CustomField>>::new();
    for custom_field in custom_fields {
        by_game
            .entry(custom_field.game_id.clone())
            .or_default()
            .push(custom_field.clone());
    }
    by_game
}

fn index_additional_applications<'a>(
    applications: impl IntoIterator<Item = &'a AdditionalApplication>,
) -> BTreeMap<String, Vec<AdditionalApplication>> {
    let mut by_game = BTreeMap::<String, Vec<AdditionalApplication>>::new();
    for application in applications {
        by_game
            .entry(application.game_id.clone())
            .or_default()
            .push(application.clone());
    }
    for applications in by_game.values_mut() {
        applications.sort_by(|left, right| {
            left.priority
                .cmp(&right.priority)
                .then_with(|| left.id.cmp(&right.id))
        });
    }
    by_game
}

struct GameWriteSuccess {
    game: Game,
    alternate_names: Vec<AlternateName>,
    custom_fields: Vec<CustomField>,
    source: PathBuf,
    backup: PathBuf,
}

struct GameAddSuccess {
    game: Game,
    source: PathBuf,
    backup: PathBuf,
}

struct GameDeleteSuccess {
    game: Game,
    source: PathBuf,
    backup: PathBuf,
}

struct PlatformCreateSuccess {
    name: String,
    source: PathBuf,
    catalog_backup: PathBuf,
    folder_count: usize,
}

struct PlatformDeleteSuccess {
    name: String,
    source: PathBuf,
    catalog_backup: PathBuf,
    platform_backup: PathBuf,
    folder_count: usize,
}

#[derive(Clone)]
struct GameLaunchSuccess {
    game_id: String,
    game_title: String,
    target_id: String,
    kind: String,
    executable: PathBuf,
    pid: u32,
}

enum PlaySessionStatsRecord {
    Game(Box<Game>),
    AdditionalApplication(Box<AdditionalApplication>),
}

struct PlaySessionStatsWriteSuccess {
    record: PlaySessionStatsRecord,
    backup: PathBuf,
}

enum LaunchSelection {
    MainGame {
        automatic_applications: Vec<AdditionalApplication>,
    },
    AdditionalApplication(Box<AdditionalApplication>),
}

fn primary_launch_template(sequence: &LaunchSequence) -> Result<GameLaunchSuccess, String> {
    let step = sequence
        .steps
        .iter()
        .find(|step| step.role.is_primary())
        .ok_or_else(|| "launch sequence has no primary target".to_string())?;
    let (target_id, target_label) = match &step.plan.target {
        LaunchTarget::MainGame => (sequence.game_id.clone(), None),
        LaunchTarget::AdditionalApplication {
            application_id,
            application_name,
        } => (
            application_id.clone(),
            Some(format!("additional application {application_name}")),
        ),
    };
    let process_label = match &step.plan.kind {
        LaunchKind::Direct => "direct process".to_string(),
        LaunchKind::DosBox => "DOSBox".to_string(),
        LaunchKind::ScummVm => "ScummVM".to_string(),
        LaunchKind::Emulator { title, .. } => format!("emulator {title}"),
    };
    let kind = target_label
        .map(|target| format!("{target} using {process_label}"))
        .unwrap_or(process_label);
    Ok(GameLaunchSuccess {
        game_id: sequence.game_id.clone(),
        game_title: sequence.game_title.clone(),
        target_id,
        kind,
        executable: step.plan.request.executable.clone(),
        pid: 0,
    })
}

enum GameWriteFailure {
    Conflict(String),
    PendingRecovery { count: usize, message: String },
    Referenced(Vec<GameReference>),
    Other(String),
}

enum PlatformWriteFailure {
    Conflict(String),
    PendingRecovery { count: usize, message: String },
    Referenced(Vec<PlatformReference>),
    Other(String),
}

const GAME_EDIT_PAYLOAD_VERSION: u32 = 3;

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct AlternateNameEditPayload {
    source_index: Option<usize>,
    name: String,
    region: Option<String>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct CustomFieldEditPayload {
    source_index: Option<usize>,
    name: String,
    value: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct GameEditPayload {
    version: u32,
    metadata: GameMetadata,
    launch_configuration: GameLaunchConfiguration,
    alternate_names: Vec<AlternateNameEditPayload>,
    custom_fields: Vec<CustomFieldEditPayload>,
    favorite: bool,
    completed: bool,
    star_rating: u8,
}

fn parse_game_edit_payload(payload: &str) -> Result<GameEditPayload, String> {
    let mut payload: GameEditPayload = serde_json::from_str(payload)
        .map_err(|error| format!("invalid game editor payload: {error}"))?;
    if payload.version != GAME_EDIT_PAYLOAD_VERSION {
        return Err(format!(
            "unsupported game editor payload version {}; expected {}",
            payload.version, GAME_EDIT_PAYLOAD_VERSION
        ));
    }
    if payload.metadata.title.trim().is_empty() {
        return Err("a game title cannot be empty".into());
    }
    if payload.star_rating > 5 {
        return Err("star rating must be between 0 and 5".into());
    }
    payload.metadata.sort_title = canonical_optional_text(payload.metadata.sort_title);
    payload.metadata.notes = canonical_optional_text(payload.metadata.notes);
    payload.metadata.developer = canonical_optional_text(payload.metadata.developer);
    payload.metadata.genre = canonical_optional_text(payload.metadata.genre);
    payload.metadata.max_players = payload.metadata.max_players.filter(|value| *value > 0);
    payload.metadata.play_mode = canonical_optional_text(payload.metadata.play_mode);
    payload.metadata.progress = canonical_optional_text(payload.metadata.progress);
    payload.metadata.publisher = canonical_optional_text(payload.metadata.publisher);
    payload.metadata.rating = canonical_optional_text(payload.metadata.rating);
    payload.metadata.region = canonical_optional_text(payload.metadata.region);
    payload.metadata.release_date = canonical_optional_text(payload.metadata.release_date);
    payload.metadata.release_type = canonical_optional_text(payload.metadata.release_type);
    payload.metadata.series = canonical_optional_text(payload.metadata.series);
    payload.metadata.source = canonical_optional_text(payload.metadata.source);
    payload.metadata.status = canonical_optional_text(payload.metadata.status);
    payload.metadata.version = canonical_optional_text(payload.metadata.version);
    payload.metadata.wikipedia_url = canonical_optional_text(payload.metadata.wikipedia_url);
    payload.launch_configuration.command_line =
        canonical_optional_text(payload.launch_configuration.command_line);
    payload.launch_configuration.emulator_id =
        canonical_optional_text(payload.launch_configuration.emulator_id);
    payload.launch_configuration.custom_dos_box_version_path =
        canonical_optional_text(payload.launch_configuration.custom_dos_box_version_path);
    payload.launch_configuration.dos_box_configuration_path =
        canonical_optional_text(payload.launch_configuration.dos_box_configuration_path);
    payload.launch_configuration.scumm_vm_game_data_folder_path =
        canonical_optional_text(payload.launch_configuration.scumm_vm_game_data_folder_path);
    payload.launch_configuration.scumm_vm_game_type =
        canonical_optional_text(payload.launch_configuration.scumm_vm_game_type);
    for alternate_name in &mut payload.alternate_names {
        if alternate_name.name.trim().is_empty() {
            return Err("an alternate name cannot be empty".into());
        }
        alternate_name.region = canonical_optional_text(alternate_name.region.take());
    }
    for custom_field in &payload.custom_fields {
        if custom_field.name.trim().is_empty() {
            return Err("a custom field name cannot be empty".into());
        }
    }
    Ok(payload)
}

fn canonical_optional_text(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.trim().is_empty())
}

fn launchbox_local_timestamp(started_at: SystemTime) -> String {
    let local: DateTime<Local> = started_at.into();
    format!(
        "{}.{:07}{}",
        local.format("%Y-%m-%dT%H:%M:%S"),
        local.timestamp_subsec_nanos() / 100,
        local.format("%:z")
    )
}

fn write_play_session_start(
    root: PathBuf,
    source: PathBuf,
    target: &LaunchTarget,
    game_id: &str,
    started_at: SystemTime,
) -> Result<PlaySessionStatsWriteSuccess, GameWriteFailure> {
    let mut document = PlatformDocument::load(&source)
        .map_err(|error| GameWriteFailure::Other(error.to_string()))?;
    let timestamp = launchbox_local_timestamp(started_at);
    let record = match target {
        LaunchTarget::MainGame => document
            .record_game_play_start(game_id, &timestamp)
            .map(|game| PlaySessionStatsRecord::Game(Box::new(game))),
        LaunchTarget::AdditionalApplication { application_id, .. } => document
            .record_additional_application_play_start(application_id, &timestamp)
            .map(|application| {
                PlaySessionStatsRecord::AdditionalApplication(Box::new(application))
            }),
    }
    .map_err(|error| GameWriteFailure::Other(error.to_string()))?;
    commit_play_session_stats(root, source, document, record)
}

fn write_play_session_time(
    root: PathBuf,
    source: PathBuf,
    target: &LaunchTarget,
    game_id: &str,
    elapsed: Duration,
) -> Result<PlaySessionStatsWriteSuccess, GameWriteFailure> {
    let elapsed_seconds = elapsed.as_secs();
    debug_assert!(elapsed_seconds > 0);
    let mut document = PlatformDocument::load(&source)
        .map_err(|error| GameWriteFailure::Other(error.to_string()))?;
    let record = match target {
        LaunchTarget::MainGame => document
            .record_game_play_time(game_id, elapsed_seconds)
            .map(|game| PlaySessionStatsRecord::Game(Box::new(game))),
        LaunchTarget::AdditionalApplication { application_id, .. } => document
            .record_additional_application_play_time(application_id, elapsed_seconds)
            .map(|application| {
                PlaySessionStatsRecord::AdditionalApplication(Box::new(application))
            }),
    }
    .map_err(|error| GameWriteFailure::Other(error.to_string()))?;
    commit_play_session_stats(root, source, document, record)
}

fn commit_play_session_stats(
    root: PathBuf,
    source: PathBuf,
    document: PlatformDocument,
    record: PlaySessionStatsRecord,
) -> Result<PlaySessionStatsWriteSuccess, GameWriteFailure> {
    let mut transaction = LibraryTransaction::new(&root).map_err(classify_transaction_error)?;
    transaction
        .stage_platform(&document)
        .map_err(classify_transaction_error)?;
    let report = transaction.commit().map_err(classify_transaction_error)?;
    let backup = report
        .writes
        .into_iter()
        .find(|write| write.target == source)
        .map(|write| write.backup)
        .ok_or_else(|| {
            GameWriteFailure::Other("play-statistics transaction reported no platform write".into())
        })?;
    Ok(PlaySessionStatsWriteSuccess { record, backup })
}

fn describe_game_write_failure(error: &GameWriteFailure) -> String {
    match error {
        GameWriteFailure::Conflict(message) => format!("write conflict: {message}"),
        GameWriteFailure::PendingRecovery { message, .. } => {
            format!("interrupted transaction requires recovery: {message}")
        }
        GameWriteFailure::Referenced(references) => format!(
            "{} unexpected dependent records prevented the update",
            references.len()
        ),
        GameWriteFailure::Other(message) => message.clone(),
    }
}

fn write_game(
    root: PathBuf,
    source: PathBuf,
    game_id: String,
    edit: GameEditPayload,
) -> Result<GameWriteSuccess, GameWriteFailure> {
    let GameEditPayload {
        version: _,
        metadata,
        launch_configuration,
        alternate_names,
        custom_fields,
        favorite,
        completed,
        star_rating,
    } = edit;
    let mut document = PlatformDocument::load(&source)
        .map_err(|error| GameWriteFailure::Other(error.to_string()))?;
    document
        .set_game_metadata(&game_id, metadata)
        .map_err(|error| GameWriteFailure::Other(error.to_string()))?;
    document
        .set_game_launch_configuration(&game_id, launch_configuration)
        .map_err(|error| GameWriteFailure::Other(error.to_string()))?;
    let alternate_names = document
        .set_game_alternate_names(
            &game_id,
            alternate_names
                .into_iter()
                .map(|edit| IndexedPlatformRecordEdit {
                    source_index: edit.source_index,
                    record: AlternateName {
                        game_id: game_id.clone(),
                        name: edit.name,
                        region: edit.region,
                    },
                })
                .collect(),
        )
        .map_err(|error| GameWriteFailure::Other(error.to_string()))?;
    let custom_fields = document
        .set_game_custom_fields(
            &game_id,
            custom_fields
                .into_iter()
                .map(|edit| IndexedPlatformRecordEdit {
                    source_index: edit.source_index,
                    record: CustomField {
                        game_id: game_id.clone(),
                        name: edit.name,
                        value: edit.value,
                    },
                })
                .collect(),
        )
        .map_err(|error| GameWriteFailure::Other(error.to_string()))?;
    document
        .set_game_state(&game_id, favorite, completed, star_rating)
        .map_err(|error| GameWriteFailure::Other(error.to_string()))?;
    let game = document
        .library()
        .games
        .iter()
        .find(|game| game.id == game_id)
        .cloned()
        .ok_or_else(|| {
            GameWriteFailure::Other(format!("game {game_id} disappeared during edit"))
        })?;

    let mut transaction = LibraryTransaction::new(&root).map_err(classify_transaction_error)?;
    transaction
        .stage_platform(&document)
        .map_err(classify_transaction_error)?;
    let report = transaction.commit().map_err(classify_transaction_error)?;
    let backup = report
        .writes
        .into_iter()
        .next()
        .map(|write| write.backup)
        .ok_or_else(|| GameWriteFailure::Other("transaction reported no platform write".into()))?;
    Ok(GameWriteSuccess {
        game,
        alternate_names,
        custom_fields,
        source,
        backup,
    })
}

fn add_game_to_platform(
    root: PathBuf,
    source: PathBuf,
    new_game: NewGame,
) -> Result<GameAddSuccess, GameWriteFailure> {
    let platform_name = new_game.platform.clone();
    let mut document = PlatformDocument::load_for_platform(&source, platform_name)
        .map_err(|error| GameWriteFailure::Other(error.to_string()))?;
    let game = document
        .add_game(new_game)
        .map_err(|error| GameWriteFailure::Other(error.to_string()))?;
    let mut transaction = LibraryTransaction::new(&root).map_err(classify_transaction_error)?;
    transaction
        .stage_platform(&document)
        .map_err(classify_transaction_error)?;
    let report = transaction.commit().map_err(classify_transaction_error)?;
    let backup = report
        .writes
        .into_iter()
        .next()
        .map(|write| write.backup)
        .ok_or_else(|| GameWriteFailure::Other("transaction reported no platform write".into()))?;
    Ok(GameAddSuccess {
        game,
        source,
        backup,
    })
}

fn create_platform_in_library(
    root: PathBuf,
    name: String,
    scrape_as: String,
) -> Result<PlatformCreateSuccess, PlatformWriteFailure> {
    let data = LaunchBoxDataIndex::load(&root)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let data_root = data.data_root().to_path_buf();
    let catalog_path = data_root.join("Platforms.xml");
    let target =
        data_root
            .join("Platforms")
            .join(platform_document_file_name(&name).map_err(|error| {
                PlatformWriteFailure::Other(format!("invalid platform name: {error}"))
            })?);

    let mut catalog = AuxiliaryDocument::load(&catalog_path)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let folders = default_platform_folders(&name)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    catalog
        .add_platform_definition(
            PlatformDefinition {
                metadata: NavigationMetadata {
                    name: name.clone(),
                    scrape_as: (!scrape_as.trim().is_empty()).then_some(scrape_as),
                    ..NavigationMetadata::default()
                },
                ..PlatformDefinition::default()
            },
            folders.clone(),
        )
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    ensure_portable_target_is_absent(&target)?;

    let platform = PlatformDocument::new_empty(&target, &name)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let mut transaction =
        LibraryTransaction::new(&root).map_err(classify_platform_transaction_error)?;
    transaction
        .stage_auxiliary(&catalog)
        .map_err(classify_platform_transaction_error)?;
    transaction
        .stage_new_platform(&platform)
        .map_err(classify_platform_transaction_error)?;
    let report = transaction
        .commit()
        .map_err(classify_platform_transaction_error)?;
    let catalog_backup = report
        .writes
        .into_iter()
        .find(|write| write.target == catalog_path)
        .map(|write| write.backup)
        .ok_or_else(|| {
            PlatformWriteFailure::Other(
                "platform creation transaction reported no catalog write".into(),
            )
        })?;
    if !report
        .created_targets
        .iter()
        .any(|created| created == &target)
    {
        return Err(PlatformWriteFailure::Other(
            "platform creation transaction reported no created document".into(),
        ));
    }

    Ok(PlatformCreateSuccess {
        name,
        source: target,
        catalog_backup,
        folder_count: folders.len(),
    })
}

fn delete_platform_from_library(
    root: PathBuf,
    source: PathBuf,
    name: String,
) -> Result<PlatformDeleteSuccess, PlatformWriteFailure> {
    let references = find_platform_references(&root, &name)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    if !references.is_empty() {
        return Err(PlatformWriteFailure::Referenced(references));
    }

    let data = LaunchBoxDataIndex::load(&root)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let catalog_path = data.data_root().join("Platforms.xml");
    let mut catalog = AuxiliaryDocument::load(&catalog_path)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let removed = catalog
        .remove_platform_definition(&name)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let exact_name = removed.platform.metadata.name;
    let folder_count = removed.folders.len();
    let platform = PlatformDocument::load_for_platform(&source, &exact_name)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;

    let mut transaction =
        LibraryTransaction::new(&root).map_err(classify_platform_transaction_error)?;
    transaction
        .stage_auxiliary(&catalog)
        .map_err(classify_platform_transaction_error)?;
    transaction
        .stage_delete_platform(&platform)
        .map_err(classify_platform_transaction_error)?;
    let report = transaction
        .commit()
        .map_err(classify_platform_transaction_error)?;
    let catalog_backup = report
        .writes
        .into_iter()
        .find(|write| write.target == catalog_path)
        .map(|write| write.backup)
        .ok_or_else(|| {
            PlatformWriteFailure::Other(
                "platform deletion transaction reported no catalog write".into(),
            )
        })?;
    let platform_backup = report
        .deleted_targets
        .into_iter()
        .find(|write| write.target == source)
        .map(|write| write.backup)
        .ok_or_else(|| {
            PlatformWriteFailure::Other(
                "platform deletion transaction reported no platform deletion".into(),
            )
        })?;

    Ok(PlatformDeleteSuccess {
        name: exact_name,
        source,
        catalog_backup,
        platform_backup,
        folder_count,
    })
}

fn ensure_portable_target_is_absent(target: &Path) -> Result<(), PlatformWriteFailure> {
    let parent = target.parent().ok_or_else(|| {
        PlatformWriteFailure::Other(format!(
            "platform document has no parent directory: {}",
            target.display()
        ))
    })?;
    let expected_name = target.file_name().ok_or_else(|| {
        PlatformWriteFailure::Other(format!(
            "platform document has no filename: {}",
            target.display()
        ))
    })?;
    for entry in fs::read_dir(parent).map_err(|error| {
        PlatformWriteFailure::Other(format!("could not read {}: {error}", parent.display()))
    })? {
        let entry = entry.map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
        if entry
            .file_name()
            .to_string_lossy()
            .eq_ignore_ascii_case(&expected_name.to_string_lossy())
        {
            return Err(PlatformWriteFailure::Other(format!(
                "portable platform filename collides with existing {}",
                entry.path().display()
            )));
        }
    }
    Ok(())
}

fn delete_game_from_platform(
    root: PathBuf,
    reference_scope: PathBuf,
    source: PathBuf,
    game_id: String,
) -> Result<GameDeleteSuccess, GameWriteFailure> {
    let references = find_game_references(&reference_scope, &game_id)
        .map_err(|error| GameWriteFailure::Other(error.to_string()))?;
    if !references.is_empty() {
        return Err(GameWriteFailure::Referenced(references));
    }
    let mut document = PlatformDocument::load(&source)
        .map_err(|error| GameWriteFailure::Other(error.to_string()))?;
    let game = document
        .remove_game(&game_id)
        .map_err(|error| GameWriteFailure::Other(error.to_string()))?;
    let mut transaction = LibraryTransaction::new(&root).map_err(classify_transaction_error)?;
    transaction
        .stage_platform(&document)
        .map_err(classify_transaction_error)?;
    let report = transaction.commit().map_err(classify_transaction_error)?;
    let backup = report
        .writes
        .into_iter()
        .next()
        .map(|write| write.backup)
        .ok_or_else(|| GameWriteFailure::Other("transaction reported no platform write".into()))?;
    Ok(GameDeleteSuccess {
        game,
        source,
        backup,
    })
}

fn classify_transaction_error(error: TransactionError) -> GameWriteFailure {
    let message = error.to_string();
    match error {
        TransactionError::Conflict { .. }
        | TransactionError::Storage(StorageError::WriteConflict { .. }) => {
            GameWriteFailure::Conflict(message)
        }
        TransactionError::PendingRecovery { manifests, .. } => GameWriteFailure::PendingRecovery {
            count: manifests.len(),
            message,
        },
        TransactionError::RecoveryRequired { .. } => {
            GameWriteFailure::PendingRecovery { count: 1, message }
        }
        _ => GameWriteFailure::Other(message),
    }
}

fn classify_platform_transaction_error(error: TransactionError) -> PlatformWriteFailure {
    let message = error.to_string();
    match error {
        TransactionError::Conflict { .. }
        | TransactionError::Storage(StorageError::WriteConflict { .. }) => {
            PlatformWriteFailure::Conflict(message)
        }
        TransactionError::PendingRecovery { manifests, .. } => {
            PlatformWriteFailure::PendingRecovery {
                count: manifests.len(),
                message,
            }
        }
        TransactionError::RecoveryRequired { .. } => {
            PlatformWriteFailure::PendingRecovery { count: 1, message }
        }
        _ => PlatformWriteFailure::Other(message),
    }
}

fn path_resolver_from_command_line(
    mut resolver: HostPathResolver,
) -> Result<HostPathResolver, String> {
    let mut arguments = std::env::args_os();
    while let Some(argument) = arguments.next() {
        if argument == "--map-windows-drive" {
            let mapping = arguments
                .next()
                .ok_or_else(|| "--map-windows-drive requires DRIVE=HOST_ROOT".to_string())?;
            let mapping = mapping.to_str().ok_or_else(|| {
                "--map-windows-drive must be representable as Unicode".to_string()
            })?;
            let (drive, host_root) = mapping
                .split_once('=')
                .ok_or_else(|| "--map-windows-drive requires DRIVE=HOST_ROOT".to_string())?;
            let mut characters = drive.chars();
            let drive = characters
                .next()
                .filter(|_| characters.next().is_none())
                .ok_or_else(|| "a Windows drive mapping requires one drive letter".to_string())?;
            if host_root.is_empty() {
                return Err("a Windows drive mapping requires a host root".to_string());
            }
            resolver = resolver
                .with_windows_drive_mapping(drive, PathBuf::from(host_root))
                .map_err(|error| error.to_string())?;
        } else if argument == "--map-windows-unc" {
            let mapping = arguments
                .next()
                .ok_or_else(|| "--map-windows-unc requires SERVER/SHARE=HOST_ROOT".to_string())?;
            let mapping = mapping
                .to_str()
                .ok_or_else(|| "--map-windows-unc must be representable as Unicode".to_string())?;
            let (windows_root, host_root) = mapping
                .split_once('=')
                .ok_or_else(|| "--map-windows-unc requires SERVER/SHARE=HOST_ROOT".to_string())?;
            let (server, share) = windows_root
                .split_once('/')
                .ok_or_else(|| "--map-windows-unc requires SERVER/SHARE=HOST_ROOT".to_string())?;
            if host_root.is_empty() {
                return Err("a Windows UNC mapping requires a host root".to_string());
            }
            resolver = resolver
                .with_windows_unc_mapping(server, share, PathBuf::from(host_root))
                .map_err(|error| error.to_string())?;
        }
    }
    Ok(resolver)
}

fn path_mapping_settings_path_from_command_line() -> Result<PathBuf, String> {
    let mut arguments = std::env::args_os();
    let mut explicit_path = None;
    while let Some(argument) = arguments.next() {
        if argument == "--path-mappings-file" {
            let path = arguments
                .next()
                .ok_or_else(|| "--path-mappings-file requires a file path".to_string())?;
            if path.is_empty() {
                return Err("--path-mappings-file requires a non-empty file path".to_string());
            }
            if explicit_path.replace(PathBuf::from(path)).is_some() {
                return Err("--path-mappings-file may only be supplied once".to_string());
            }
        }
    }
    explicit_path.map_or_else(
        || default_host_path_mappings_path().map_err(|error| error.to_string()),
        Ok,
    )
}

fn load_host_path_mappings() -> Result<(PathBuf, HostPathMappings, HostPathResolver), String> {
    let path = path_mapping_settings_path_from_command_line()?;
    let mappings = HostPathMappings::load_or_default(&path).map_err(|error| error.to_string())?;
    let resolver = mappings.resolver().map_err(|error| error.to_string())?;
    let resolver = path_resolver_from_command_line(resolver)?;
    Ok((path, mappings, resolver))
}

enum PathMappingKey {
    WindowsDrive(char),
    WindowsUnc { server: String, share: String },
}

fn path_mapping_key(mappings: &HostPathMappings, index: i32) -> Option<PathMappingKey> {
    let index = usize::try_from(index).ok()?;
    if let Some(mapping) = mappings.windows_drives().get(index) {
        return Some(PathMappingKey::WindowsDrive(mapping.drive()));
    }
    let unc_index = index.checked_sub(mappings.windows_drives().len())?;
    mappings
        .windows_unc()
        .get(unc_index)
        .map(|mapping| PathMappingKey::WindowsUnc {
            server: mapping.server().to_string(),
            share: mapping.share().to_string(),
        })
}

impl qobject::LibraryController {
    pub fn initialize_host_path_mappings(mut self: Pin<&mut Self>) -> bool {
        if self.as_ref().rust().path_mappings_initialized {
            return true;
        }
        match load_host_path_mappings() {
            Ok((path, mappings, resolver)) => {
                let count = i32::try_from(mappings.len()).unwrap_or(i32::MAX);
                self.as_mut().rust_mut().path_mapping_settings_file = Some(path.clone());
                self.as_mut().rust_mut().path_mappings = mappings;
                self.as_mut().rust_mut().path_resolver = resolver;
                self.as_mut().rust_mut().path_mappings_initialized = true;
                self.as_mut()
                    .set_path_mapping_settings_path(qstring(path.to_string_lossy()));
                self.as_mut().set_path_mapping_count(count);
                true
            }
            Err(error) => {
                self.as_mut().set_status_message(qstring(format!(
                    "Could not load host path mappings: {error}"
                )));
                false
            }
        }
    }

    pub fn configure_windows_drive_mapping(
        mut self: Pin<&mut Self>,
        drive: QString,
        host_root: QString,
    ) -> bool {
        if !self.as_mut().initialize_host_path_mappings() {
            return false;
        }
        let drive = drive.to_string();
        let host_root = host_root.to_string();
        let mut characters = drive.chars();
        let Some(drive) = characters.next() else {
            self.as_mut()
                .set_status_message(qstring("A Windows drive mapping requires a drive letter."));
            return false;
        };
        if characters.next().is_some() || host_root.trim().is_empty() {
            self.as_mut().set_status_message(qstring(
                "A Windows drive mapping requires one drive letter and a host root.",
            ));
            return false;
        }
        let resolver = self.as_ref().rust().path_resolver.clone();
        match resolver.with_windows_drive_mapping(drive, PathBuf::from(&host_root)) {
            Ok(resolver) => {
                self.as_mut().rust_mut().path_resolver = resolver;
                true
            }
            Err(error) => {
                self.as_mut()
                    .set_status_message(qstring(format!("Invalid Windows drive mapping: {error}")));
                false
            }
        }
    }

    pub fn save_windows_drive_mapping(
        mut self: Pin<&mut Self>,
        drive: QString,
        host_root: QString,
    ) -> bool {
        if !self.as_mut().initialize_host_path_mappings() {
            return false;
        }
        let drive = drive.to_string();
        let mut characters = drive.trim().chars();
        let Some(drive) = characters.next() else {
            self.as_mut()
                .set_status_message(qstring("A Windows drive mapping requires a drive letter."));
            return false;
        };
        if characters.next().is_some() {
            self.as_mut().set_status_message(qstring(
                "A Windows drive mapping requires exactly one drive letter.",
            ));
            return false;
        }
        let mut mappings = self.as_ref().rust().path_mappings.clone();
        if let Err(error) = mappings.set_windows_drive(drive, PathBuf::from(host_root.to_string()))
        {
            self.as_mut()
                .set_status_message(qstring(format!("Invalid Windows drive mapping: {error}")));
            return false;
        }
        self.as_mut().persist_path_mappings(
            mappings,
            format!(
                "Saved host mapping for Windows drive {}:.",
                drive.to_ascii_uppercase()
            ),
        )
    }

    pub fn save_windows_unc_mapping(
        mut self: Pin<&mut Self>,
        server: QString,
        share: QString,
        host_root: QString,
    ) -> bool {
        if !self.as_mut().initialize_host_path_mappings() {
            return false;
        }
        let server = server.to_string();
        let share = share.to_string();
        let mut mappings = self.as_ref().rust().path_mappings.clone();
        if let Err(error) = mappings.set_windows_unc(
            server.clone(),
            share.clone(),
            PathBuf::from(host_root.to_string()),
        ) {
            self.as_mut()
                .set_status_message(qstring(format!("Invalid Windows UNC mapping: {error}")));
            return false;
        }
        self.as_mut().persist_path_mappings(
            mappings,
            format!(
                "Saved host mapping for \\\\{}\\{}.",
                server.trim(),
                share.trim()
            ),
        )
    }

    pub fn remove_path_mapping(mut self: Pin<&mut Self>, index: i32) -> bool {
        if !self.as_mut().initialize_host_path_mappings() {
            return false;
        }
        let mut mappings = self.as_ref().rust().path_mappings.clone();
        let Some(key) = path_mapping_key(&mappings, index) else {
            self.as_mut()
                .set_status_message(qstring("The selected host path mapping no longer exists."));
            return false;
        };
        let label = match key {
            PathMappingKey::WindowsDrive(drive) => {
                mappings.remove_windows_drive(drive);
                format!("Windows drive {drive}:")
            }
            PathMappingKey::WindowsUnc { server, share } => {
                mappings.remove_windows_unc(&server, &share);
                format!(r"\\{server}\{share}")
            }
        };
        self.as_mut()
            .persist_path_mappings(mappings, format!("Removed host mapping for {label}."))
    }

    pub fn path_mapping_kind_at(&self, index: i32) -> QString {
        qstring(match path_mapping_key(&self.rust().path_mappings, index) {
            Some(PathMappingKey::WindowsDrive(_)) => "Drive",
            Some(PathMappingKey::WindowsUnc { .. }) => "UNC",
            None => "",
        })
    }

    pub fn path_mapping_windows_root_at(&self, index: i32) -> QString {
        qstring(match path_mapping_key(&self.rust().path_mappings, index) {
            Some(PathMappingKey::WindowsDrive(drive)) => format!("{drive}:\\"),
            Some(PathMappingKey::WindowsUnc { server, share }) => {
                format!(r"\\{server}\{share}")
            }
            None => String::new(),
        })
    }

    pub fn path_mapping_host_root_at(&self, index: i32) -> QString {
        let index = match usize::try_from(index) {
            Ok(index) => index,
            Err(_) => return QString::default(),
        };
        let mappings = &self.rust().path_mappings;
        let root = mappings
            .windows_drives()
            .get(index)
            .map(|mapping| mapping.host_root())
            .or_else(|| {
                index
                    .checked_sub(mappings.windows_drives().len())
                    .and_then(|index| mappings.windows_unc().get(index))
                    .map(|mapping| mapping.host_root())
            });
        root.map_or_else(QString::default, |root| qstring(root.to_string_lossy()))
    }

    pub fn configure_windows_unc_mapping(
        mut self: Pin<&mut Self>,
        server: QString,
        share: QString,
        host_root: QString,
    ) -> bool {
        if !self.as_mut().initialize_host_path_mappings() {
            return false;
        }
        let server = server.to_string();
        let share = share.to_string();
        let host_root = host_root.to_string();
        if host_root.trim().is_empty() {
            self.as_mut()
                .set_status_message(qstring("A Windows UNC mapping requires a host root."));
            return false;
        }
        let resolver = self.as_ref().rust().path_resolver.clone();
        match resolver.with_windows_unc_mapping(server, share, PathBuf::from(&host_root)) {
            Ok(resolver) => {
                self.as_mut().rust_mut().path_resolver = resolver;
                true
            }
            Err(error) => {
                self.as_mut()
                    .set_status_message(qstring(format!("Invalid Windows UNC mapping: {error}")));
                false
            }
        }
    }

    pub fn load_fixture(mut self: Pin<&mut Self>) {
        self.as_mut().advance_generation();
        self.as_mut().set_loading(false);
        self.as_mut().set_writing(false);
        self.as_mut().set_write_conflict(false);
        self.as_mut().set_pending_recovery_count(0);
        match PlatformDocument::from_reader("Fixture Console.xml", FIXTURE.as_bytes()) {
            Ok(document) => {
                let games = document.library().games.clone();
                let additional_applications_by_game = index_additional_applications(
                    document.library().additional_applications.iter(),
                );
                let mounts_by_game = index_mounts(document.library().mounts.iter());
                let alternate_names_by_game =
                    index_alternate_names(document.library().alternate_names.iter());
                let custom_fields_by_game =
                    index_custom_fields(document.library().custom_fields.iter());
                self.as_mut().replace_library(LibraryReplacement {
                    games,
                    game_sources: Vec::new(),
                    additional_applications_by_game,
                    mounts_by_game,
                    alternate_names_by_game,
                    custom_fields_by_game,
                    platform_names: vec![document.library().name.clone()],
                    platform_sources: BTreeMap::new(),
                    library_root: None,
                    launchbox_root: None,
                    emulator_configuration: None,
                    name: "Fixture Console".into(),
                    message: "Embedded compatibility fixture".into(),
                    pending_recovery_count: 0,
                });
            }
            Err(error) => self
                .as_mut()
                .set_status_message(qstring(format!("Fixture failed to load: {error}"))),
        }
    }

    pub fn load_library(mut self: Pin<&mut Self>, path: QString) {
        if *self.as_ref().writing() || *self.as_ref().launching() {
            self.as_mut()
                .set_status_message(qstring("Wait for the current library operation to finish."));
            return;
        }
        let path = path.to_string();
        if path.trim().is_empty() {
            self.as_mut().set_status_message(qstring(
                "Choose a LaunchBox directory or platform XML file.",
            ));
            return;
        }
        if !self.as_mut().initialize_host_path_mappings() {
            return;
        }

        let generation = self.as_mut().advance_generation();
        self.as_mut().set_loading(true);
        self.as_mut()
            .set_status_message(qstring(format!("Loading {path} in the background...")));

        let qt_thread = self.as_ref().qt_thread();
        eprintln!("Started background LaunchBox library load.");
        let spawn_result = std::thread::Builder::new()
            .name("launchbox-library-index".to_string())
            .spawn(move || {
                let loaded = LoadedLibrary::load(path);
                match &loaded {
                    Ok(library) => eprintln!("Background index ready: {}", library.message),
                    Err(error) => eprintln!("Background index failed: {error}"),
                }
                qt_thread
                    .queue(move |mut controller| {
                        controller
                            .as_mut()
                            .finish_background_load(generation, loaded);
                    })
                    .ok();
            });
        if let Err(error) = spawn_result {
            self.as_mut().set_loading(false);
            self.as_mut()
                .set_status_message(qstring(format!("Could not start library loader: {error}")));
        }
    }

    pub fn apply_filters(mut self: Pin<&mut Self>, search_text: QString, platform: QString) {
        self.as_mut().set_search_text(search_text);
        self.as_mut().set_platform_filter(platform);
        self.as_mut().refresh_filtered_games();
    }

    pub fn save_game(mut self: Pin<&mut Self>, row: i32, game_id: QString, edit_payload: QString) {
        if *self.as_ref().loading() || *self.as_ref().writing() || *self.as_ref().launching() {
            self.as_mut()
                .set_status_message(qstring("Wait for the current library operation to finish."));
            return;
        }
        if *self.as_ref().pending_recovery_count() > 0 {
            self.as_mut().set_status_message(qstring(
                "Recover the interrupted transaction before editing this library.",
            ));
            return;
        }
        if *self.as_ref().write_conflict() {
            self.as_mut().set_status_message(qstring(
                "Reload the library before retrying an edit after a write conflict.",
            ));
            return;
        }
        let edit = match parse_game_edit_payload(&edit_payload.to_string()) {
            Ok(edit) => edit,
            Err(error) => {
                self.as_mut()
                    .set_status_message(qstring(format!("Could not save game: {error}.")));
                return;
            }
        };
        let title = edit.metadata.title.clone();

        let game_id = game_id.to_string();
        let Some((source, root)) = self.edit_target(row, &game_id) else {
            self.as_mut().set_status_message(qstring(
                "The selected game no longer matches this model; reload and try again.",
            ));
            return;
        };
        let generation = self.as_ref().rust().request_generation;
        self.as_mut().set_writing(true);
        self.as_mut()
            .set_status_message(qstring(format!("Saving {title} in the background...")));

        let qt_thread = self.as_ref().qt_thread();
        let spawn_result = std::thread::Builder::new()
            .name("launchbox-game-write".to_string())
            .spawn(move || {
                let result = write_game(root, source, game_id, edit);
                qt_thread
                    .queue(move |mut controller| {
                        controller.as_mut().finish_game_write(generation, result);
                    })
                    .ok();
            });
        if let Err(error) = spawn_result {
            self.as_mut().set_writing(false);
            self.as_mut()
                .set_status_message(qstring(format!("Could not start game writer: {error}")));
        }
    }

    pub fn add_game(
        mut self: Pin<&mut Self>,
        title: QString,
        application_path: QString,
        platform: QString,
    ) {
        if !self.as_mut().begin_library_mutation() {
            return;
        }
        let title = title.to_string();
        let application_path = application_path.to_string();
        let platform = platform.to_string();
        if title.trim().is_empty() || application_path.trim().is_empty() {
            self.as_mut().set_status_message(qstring(
                "A title and application path are required to add a game.",
            ));
            return;
        }
        let Some((source, root)) = self.as_ref().platform_write_target(&platform) else {
            self.as_mut().set_status_message(qstring(
                "The selected platform has no loaded writable platform document.",
            ));
            return;
        };
        let id = Uuid::new_v4().to_string();
        let new_game = NewGame {
            id: id.clone(),
            title,
            platform,
            application_path,
        };
        let generation = self.as_ref().rust().request_generation;
        self.as_mut().set_delete_blocker_count(0);
        self.as_mut().set_delete_blocker_summary(QString::default());
        self.as_mut().set_writing(true);
        self.as_mut()
            .set_status_message(qstring(format!("Adding game {id} in the background...")));

        let qt_thread = self.as_ref().qt_thread();
        let spawn_result = std::thread::Builder::new()
            .name("launchbox-game-add".to_string())
            .spawn(move || {
                let result = add_game_to_platform(root, source, new_game);
                qt_thread
                    .queue(move |mut controller| {
                        controller.as_mut().finish_game_add(generation, result);
                    })
                    .ok();
            });
        if let Err(error) = spawn_result {
            self.as_mut().set_writing(false);
            self.as_mut()
                .set_status_message(qstring(format!("Could not start game add: {error}")));
        }
    }

    pub fn add_platform(mut self: Pin<&mut Self>, name: QString, scrape_as: QString) {
        if !self.as_mut().begin_library_mutation() {
            return;
        }
        let name = name.to_string().trim().to_string();
        let scrape_as = scrape_as.to_string().trim().to_string();
        if name.is_empty() {
            self.as_mut()
                .set_status_message(qstring("A platform name is required."));
            return;
        }
        let Some(root) = self.as_ref().rust().launchbox_root.clone() else {
            self.as_mut().set_status_message(qstring(
                "Platform creation requires a loaded LaunchBox directory, not a standalone XML file.",
            ));
            return;
        };
        let generation = self.as_ref().rust().request_generation;
        self.as_mut().set_delete_blocker_count(0);
        self.as_mut().set_delete_blocker_summary(QString::default());
        self.as_mut().set_writing(true);
        self.as_mut()
            .set_status_message(qstring(format!("Creating platform {name}...")));

        let qt_thread = self.as_ref().qt_thread();
        let spawn_result = std::thread::Builder::new()
            .name("launchbox-platform-create".to_string())
            .spawn(move || {
                let result = create_platform_in_library(root, name, scrape_as);
                qt_thread
                    .queue(move |mut controller| {
                        controller
                            .as_mut()
                            .finish_platform_create(generation, result);
                    })
                    .ok();
            });
        if let Err(error) = spawn_result {
            self.as_mut().set_writing(false);
            self.as_mut().set_status_message(qstring(format!(
                "Could not start platform creation: {error}"
            )));
        }
    }

    pub fn delete_platform(mut self: Pin<&mut Self>, name: QString) {
        if !self.as_mut().begin_library_mutation() {
            return;
        }
        let name = name.to_string().trim().to_string();
        let Some(root) = self.as_ref().rust().launchbox_root.clone() else {
            self.as_mut().set_status_message(qstring(
                "Platform deletion requires a loaded LaunchBox directory, not a standalone XML file.",
            ));
            return;
        };
        let Some(source) = self
            .as_ref()
            .rust()
            .platform_sources
            .get(&platform_key(&name))
            .cloned()
        else {
            self.as_mut().set_status_message(qstring(
                "The selected platform has no loaded writable platform document.",
            ));
            return;
        };
        let generation = self.as_ref().rust().request_generation;
        self.as_mut().set_delete_blocker_count(0);
        self.as_mut().set_delete_blocker_summary(QString::default());
        self.as_mut().set_writing(true);
        self.as_mut().set_status_message(qstring(format!(
            "Checking references before deleting platform {name}..."
        )));

        let qt_thread = self.as_ref().qt_thread();
        let spawn_result = std::thread::Builder::new()
            .name("launchbox-platform-delete".to_string())
            .spawn(move || {
                let result = delete_platform_from_library(root, source, name);
                qt_thread
                    .queue(move |mut controller| {
                        controller
                            .as_mut()
                            .finish_platform_delete(generation, result);
                    })
                    .ok();
            });
        if let Err(error) = spawn_result {
            self.as_mut().set_writing(false);
            self.as_mut().set_status_message(qstring(format!(
                "Could not start platform deletion: {error}"
            )));
        }
    }

    pub fn delete_game(mut self: Pin<&mut Self>, row: i32, game_id: QString) {
        if !self.as_mut().begin_library_mutation() {
            return;
        }
        let game_id = game_id.to_string();
        let Some((source, root)) = self.as_ref().edit_target(row, &game_id) else {
            self.as_mut().set_status_message(qstring(
                "The selected game no longer matches this model; reload and try again.",
            ));
            return;
        };
        let reference_scope = PathBuf::from(self.as_ref().library_path().to_string());
        let generation = self.as_ref().rust().request_generation;
        self.as_mut().set_delete_blocker_count(0);
        self.as_mut().set_delete_blocker_summary(QString::default());
        self.as_mut().set_writing(true);
        self.as_mut().set_status_message(qstring(format!(
            "Checking references before deleting {game_id}..."
        )));

        let qt_thread = self.as_ref().qt_thread();
        let spawn_result = std::thread::Builder::new()
            .name("launchbox-game-delete".to_string())
            .spawn(move || {
                let result = delete_game_from_platform(root, reference_scope, source, game_id);
                qt_thread
                    .queue(move |mut controller| {
                        controller.as_mut().finish_game_delete(generation, result);
                    })
                    .ok();
            });
        if let Err(error) = spawn_result {
            self.as_mut().set_writing(false);
            self.as_mut()
                .set_status_message(qstring(format!("Could not start game delete: {error}")));
        }
    }

    pub fn launch_game(mut self: Pin<&mut Self>, row: i32, game_id: QString) {
        if *self.as_ref().loading()
            || *self.as_ref().writing()
            || *self.as_ref().launching()
            || *self.as_ref().launch_session_active()
        {
            self.as_mut()
                .set_status_message(qstring("Wait for the current library operation to finish."));
            return;
        }
        if *self.as_ref().pending_recovery_count() > 0 {
            self.as_mut().set_status_message(qstring(
                "Recover the interrupted transaction before launching from this library.",
            ));
            return;
        }
        let game_id = game_id.to_string();
        let Some(game) = self.as_ref().filtered_game(row).cloned() else {
            self.as_mut().set_status_message(qstring(
                "The selected game is no longer present; reload and try again.",
            ));
            return;
        };
        if game.id != game_id {
            self.as_mut().set_status_message(qstring(
                "The selected game no longer matches this model; reload and try again.",
            ));
            return;
        }
        let automatic_applications = self
            .as_ref()
            .rust()
            .additional_applications_by_game
            .get(&game.id)
            .cloned()
            .unwrap_or_default();
        self.as_mut().start_launch(
            game,
            LaunchSelection::MainGame {
                automatic_applications,
            },
        );
    }

    pub fn launch_additional_application(
        mut self: Pin<&mut Self>,
        row: i32,
        game_id: QString,
        application_id: QString,
    ) {
        if *self.as_ref().loading()
            || *self.as_ref().writing()
            || *self.as_ref().launching()
            || *self.as_ref().launch_session_active()
        {
            self.as_mut()
                .set_status_message(qstring("Wait for the current launch operation to finish."));
            return;
        }
        if *self.as_ref().pending_recovery_count() > 0 {
            self.as_mut().set_status_message(qstring(
                "Recover the interrupted transaction before launching from this library.",
            ));
            return;
        }
        let game_id = game_id.to_string();
        let application_id = application_id.to_string();
        let Some(game) = self.as_ref().filtered_game(row).cloned() else {
            self.as_mut().set_status_message(qstring(
                "The selected game is no longer present; reload and try again.",
            ));
            return;
        };
        if game.id != game_id {
            self.as_mut().set_status_message(qstring(
                "The selected game no longer matches this model; reload and try again.",
            ));
            return;
        }
        let application = self
            .as_ref()
            .rust()
            .additional_applications_by_game
            .get(&game.id)
            .and_then(|applications| {
                applications
                    .iter()
                    .find(|application| application.id == application_id)
            })
            .cloned();
        let Some(application) = application else {
            self.as_mut().set_status_message(qstring(
                "The selected additional application is no longer present; reload and try again.",
            ));
            return;
        };
        self.as_mut().start_launch(
            game,
            LaunchSelection::AdditionalApplication(Box::new(application)),
        );
    }

    fn start_launch(mut self: Pin<&mut Self>, game: Game, selection: LaunchSelection) {
        let (root, source, configuration, path_resolver, mounts) = {
            let this = self.as_ref();
            let rust = this.rust();
            let Some(root) = rust.launchbox_root.clone() else {
                self.as_mut().set_status_message(qstring(
                    "Load a LaunchBox directory, not a standalone platform XML file, before launching games.",
                ));
                return;
            };
            let Some(source) = rust
                .games
                .iter()
                .position(|candidate| candidate.id == game.id)
                .and_then(|index| rust.game_sources.get(index))
                .cloned()
            else {
                self.as_mut().set_status_message(qstring(
                    "The selected game's platform document is unavailable; reload and try again.",
                ));
                return;
            };
            (
                root,
                source,
                rust.emulator_configuration.clone(),
                rust.path_resolver.clone(),
                rust.mounts_by_game
                    .get(&game.id)
                    .cloned()
                    .unwrap_or_default(),
            )
        };
        let generation = self.as_ref().rust().request_generation;
        let title = game.title.clone();
        self.as_mut().set_launching(true);
        self.as_mut().set_launch_session_active(true);
        self.as_mut().set_last_launch_succeeded(false);
        self.as_mut().set_last_launch_game_id(QString::default());
        self.as_mut().set_last_launch_target_id(QString::default());
        self.as_mut().rust_mut().session_stats_writes = 0;
        self.as_mut().rust_mut().session_stats_error = None;
        self.as_mut()
            .set_status_message(qstring(format!("Launching {title} in the background...")));

        let qt_thread = self.as_ref().qt_thread();
        let event_thread = qt_thread.clone();
        let spawn_result = std::thread::Builder::new()
            .name("launchbox-game-launch".to_string())
            .spawn(move || {
                let archive_extractor = ArchiveExtractor::for_launchbox_root(&root);
                let sequence = std::env::current_exe()
                    .map_err(|error| {
                        format!(
                            "could not locate the running frontend executable: {error}"
                        )
                    })
                    .and_then(|frontend_executable| {
                        let context = LaunchContext {
                            frontend_executable: Some(frontend_executable),
                        };
                        match &selection {
                            LaunchSelection::MainGame {
                                automatic_applications,
                            } => prepare_game_launch_sequence_with_mounts_context_and_resolver(
                                &root,
                                &game,
                                automatic_applications,
                                &mounts,
                                configuration.as_ref(),
                                &context,
                                &path_resolver,
                                &archive_extractor,
                            ),
                            LaunchSelection::AdditionalApplication(application) => {
                                prepare_selected_additional_application_sequence_with_mounts_context_and_resolver(
                                    &root,
                                    &game,
                                    application,
                                    &mounts,
                                    configuration.as_ref(),
                                    &context,
                                    &path_resolver,
                                    &archive_extractor,
                                )
                            }
                        }
                        .map_err(|error| error.to_string())
                    });

                let mut primary_started = false;
                let mut session_stats_errors = Vec::new();
                let result = sequence.and_then(|sequence| {
                    let template = primary_launch_template(&sequence)?;
                    let report = execute_launch_sequence(&sequence, |event| match event {
                        LaunchSequenceEvent::StepStarted {
                            role,
                            target,
                            pid,
                            started_at,
                            ..
                        } if role.is_primary() => {
                            primary_started = true;
                            let mut success = template.clone();
                            success.pid = pid;
                            event_thread
                                .queue(move |mut controller| {
                                    controller
                                        .as_mut()
                                        .finish_game_launch(generation, Ok(success));
                                })
                                .ok();

                            let stats_result = write_play_session_start(
                                root.clone(),
                                source.clone(),
                                &target,
                                &game.id,
                                started_at,
                            );
                            if let Err(error) = &stats_result {
                                session_stats_errors.push(describe_game_write_failure(error));
                            }
                            event_thread
                                .queue(move |mut controller| {
                                    controller
                                        .as_mut()
                                        .finish_play_session_stats(generation, stats_result);
                                })
                                .ok();
                        }
                        LaunchSequenceEvent::BeforeWaitTimedOut { target, timeout } => {
                            eprintln!(
                                "Before-app {target:?} exceeded its {:.0}s wait ceiling; continuing.",
                                timeout.as_secs_f64()
                            );
                        }
                        _ => {}
                    })
                    .map_err(|error| error.to_string())?;

                    if report.primary_runtime.as_secs() > 0 {
                        let stats_result = write_play_session_time(
                            root.clone(),
                            source.clone(),
                            &report.primary_target,
                            &report.game_id,
                            report.primary_runtime,
                        );
                        if let Err(error) = &stats_result {
                            session_stats_errors.push(describe_game_write_failure(error));
                        }
                        event_thread
                            .queue(move |mut controller| {
                                controller
                                    .as_mut()
                                    .finish_play_session_stats(generation, stats_result);
                            })
                            .ok();
                    }
                    Ok(report)
                });
                qt_thread
                    .queue(move |mut controller| {
                        controller.as_mut().finish_launch_session(
                            generation,
                            result,
                            primary_started,
                            session_stats_errors,
                        );
                    })
                    .ok();
            });
        if let Err(error) = spawn_result {
            self.as_mut().set_launching(false);
            self.as_mut().set_launch_session_active(false);
            self.as_mut()
                .set_status_message(qstring(format!("Could not start game launcher: {error}")));
        }
    }

    pub fn dismiss_delete_blocker(mut self: Pin<&mut Self>) {
        self.as_mut().set_delete_blocker_count(0);
        self.as_mut().set_delete_blocker_summary(QString::default());
    }

    pub fn recover_pending_changes(mut self: Pin<&mut Self>) {
        if *self.as_ref().loading() || *self.as_ref().writing() || *self.as_ref().launching() {
            self.as_mut()
                .set_status_message(qstring("Wait for the current library operation to finish."));
            return;
        }
        if *self.as_ref().pending_recovery_count() == 0 {
            self.as_mut().set_status_message(qstring(
                "This library has no pending transaction to recover.",
            ));
            return;
        }
        let Some(root) = self.as_ref().rust().library_root.clone() else {
            self.as_mut()
                .set_status_message(qstring("No writable library root is loaded."));
            return;
        };

        self.as_mut().set_writing(true);
        self.as_mut()
            .set_status_message(qstring("Recovering the interrupted transaction..."));
        let qt_thread = self.as_ref().qt_thread();
        let spawn_result = std::thread::Builder::new()
            .name("launchbox-transaction-recovery".to_string())
            .spawn(move || {
                let result = recover_pending_transactions(&root)
                    .map(|reports| reports.len())
                    .map_err(|error| error.to_string());
                qt_thread
                    .queue(move |mut controller| {
                        controller.as_mut().finish_recovery(result);
                    })
                    .ok();
            });
        if let Err(error) = spawn_result {
            self.as_mut().set_writing(false);
            self.as_mut().set_status_message(qstring(format!(
                "Could not start transaction recovery: {error}"
            )));
        }
    }

    pub fn reload_library(mut self: Pin<&mut Self>) {
        let path = self.as_ref().library_path().clone();
        if path.is_empty() {
            self.as_mut()
                .set_status_message(qstring("No user library is loaded."));
            return;
        }
        self.as_mut().load_library(path);
    }

    pub fn report_model_smoke_success(&self, rows: i32) {
        eprintln!("MODEL_ROLE_SMOKE_COMPLETE rows={rows}");
    }

    pub fn report_load_smoke_success(&self, games: i32, platforms: i32, heartbeats: i32) {
        eprintln!(
            "LOAD_SMOKE_COMPLETE games={games} platforms={platforms} heartbeats={heartbeats}"
        );
    }

    pub fn report_state_edit_smoke_success(&self, game_id: QString) -> bool {
        let game_id = game_id.to_string();
        let rust = self.rust();
        let state_matches = rust.games.iter().any(|game| {
            game.id == game_id && !game.favorite && game.completed && game.star_rating == 2
        });
        let success = state_matches
            && rust.model_reset_notifications == 1
            && rust.data_change_notifications == 1
            && !*self.writing()
            && !*self.write_conflict()
            && *self.pending_recovery_count() == 0;
        if success {
            eprintln!(
                "STATE_EDIT_SMOKE_COMPLETE id={game_id} resets={} data_changes={}",
                rust.model_reset_notifications, rust.data_change_notifications
            );
        }
        success
    }

    pub fn report_title_edit_smoke_success(
        &self,
        game_id: QString,
        expected_title: QString,
    ) -> bool {
        let game_id = game_id.to_string();
        let expected_title = expected_title.to_string();
        let rust = self.rust();
        let state_matches = rust.games.iter().any(|game| {
            game.id == game_id
                && game.title == expected_title
                && game.sort_title.as_deref() == Some("Adventure, Renamed")
                && game.notes.as_deref() == Some("Edited notes from Qt.")
                && game.developer.as_deref() == Some("Qt Forge")
                && game.genre.as_deref() == Some("Action Adventure")
                && game.max_players == Some(6)
                && game.play_mode.as_deref() == Some("Local Cooperative")
                && game.progress.as_deref() == Some("75%")
                && game.publisher.as_deref() == Some("Port Press")
                && game.rating.as_deref() == Some("T")
                && game.region.as_deref() == Some("Europe")
                && game.release_date.as_deref() == Some("2001-02-03")
                && game.release_type.as_deref() == Some("Homebrew")
                && game.series.is_none()
                && game.source.as_deref() == Some("Physical Media")
                && game.status.as_deref() == Some("Imported")
                && game.version.as_deref() == Some("2.0")
                && game.wikipedia_url.is_none()
                && game.application_path == r"Runtime\edited-recorder"
                && game.command_line.as_deref() == Some(r#"--edited "%gameid%" "two words""#)
                && game.emulator_id.as_deref() == Some(UNASSIGNED_EMULATOR_ID)
                && !game.use_dos_box
                && game.custom_dos_box_version_path.is_none()
                && game.dos_box_configuration_path.is_none()
                && !game.use_scumm_vm
                && !game.scumm_vm_aspect_correction
                && !game.scumm_vm_fullscreen
                && game.scumm_vm_game_data_folder_path.is_none()
                && game.scumm_vm_game_type.is_none()
                && !game.favorite
                && game.completed
                && game.star_rating == 2
        });
        let repeated_metadata_matches = rust.alternate_names_by_game.get(&game_id)
            == Some(&vec![
                AlternateName {
                    game_id: game_id.clone(),
                    name: "Adventure, Renamed Alias".into(),
                    region: Some("Europe".into()),
                },
                AlternateName {
                    game_id: game_id.clone(),
                    name: "Aventure Qt".into(),
                    region: Some("France".into()),
                },
            ])
            && rust.custom_fields_by_game.get(&game_id)
                == Some(&vec![
                    CustomField {
                        game_id: game_id.clone(),
                        name: "Cabinet Style".into(),
                        value: "Cocktail".into(),
                    },
                    CustomField {
                        game_id: game_id.clone(),
                        name: "Port Status".into(),
                        value: "Native Qt".into(),
                    },
                ]);
        let success = state_matches
            && repeated_metadata_matches
            && rust.filtered_indices.is_empty()
            && rust.model_reset_notifications == 3
            && rust.data_change_notifications == 1
            && self.search_text().to_string() == "Fixture Adventure"
            && !*self.writing()
            && !*self.write_conflict()
            && *self.pending_recovery_count() == 0;
        if success {
            eprintln!(
                "EDIT_SMOKE_COMPLETE id={game_id} title=\"{expected_title}\" resets={} data_changes={} filtered={}",
                rust.model_reset_notifications,
                rust.data_change_notifications,
                rust.filtered_indices.len()
            );
        }
        success
    }

    pub fn report_crud_smoke_success(
        &self,
        added_game_id: QString,
        blocked_references: i32,
    ) -> bool {
        let added_game_id = added_game_id.to_string();
        let rust = self.rust();
        let success = !rust.games.iter().any(|game| game.id == added_game_id)
            && rust.games.len() == 3
            && rust.model_reset_notifications == 1
            && rust.row_insert_notifications == 1
            && rust.row_remove_notifications == 1
            && rust.data_change_notifications == 0
            && blocked_references == 5
            && !*self.writing()
            && !*self.write_conflict()
            && *self.pending_recovery_count() == 0;
        if success {
            eprintln!(
                "CRUD_SMOKE_COMPLETE blocked={blocked_references} inserts={} removes={} games={}",
                rust.row_insert_notifications,
                rust.row_remove_notifications,
                rust.games.len()
            );
        }
        success
    }

    pub fn report_platform_crud_smoke_success(
        &self,
        platform_name: QString,
        blocked_references: i32,
    ) -> bool {
        let platform_name = platform_name.to_string();
        let rust = self.rust();
        let success = !rust
            .platform_names
            .iter()
            .any(|name| name.eq_ignore_ascii_case(&platform_name))
            && !rust
                .games
                .iter()
                .any(|game| game.platform.eq_ignore_ascii_case(&platform_name))
            && rust.games.len() == 3
            && rust.platform_names.len() == 1
            && rust.model_reset_notifications == 1
            && rust.row_insert_notifications == 1
            && rust.row_remove_notifications == 1
            && blocked_references == 1
            && *self.platform_entry_count() == 1
            && !*self.writing()
            && !*self.write_conflict()
            && *self.pending_recovery_count() == 0;
        if success {
            eprintln!(
                "PLATFORM_CRUD_SMOKE_COMPLETE platform=\"{platform_name}\" blocked={blocked_references} inserts={} removes={} games={} platforms={}",
                rust.row_insert_notifications,
                rust.row_remove_notifications,
                rust.games.len(),
                rust.platform_names.len()
            );
        }
        success
    }

    pub fn report_launch_smoke_success(&self, game_id: QString) -> bool {
        let game_id = game_id.to_string();
        let rust = self.rust();
        let success = rust.launch_notifications == 1
            && rust.session_stats_writes >= 1
            && rust.session_stats_error.is_none()
            && self.last_launch_game_id().to_string() == game_id
            && self.last_launch_target_id().to_string() == game_id
            && *self.last_launch_succeeded()
            && !*self.launching()
            && !*self.launch_session_active();
        if success {
            eprintln!(
                "LAUNCH_SMOKE_COMPLETE id={game_id} launches={} stats_writes={}",
                rust.launch_notifications, rust.session_stats_writes,
            );
        }
        success
    }

    pub fn report_additional_application_launch_smoke_success(
        &self,
        game_id: QString,
        application_id: QString,
    ) -> bool {
        let game_id = game_id.to_string();
        let application_id = application_id.to_string();
        let rust = self.rust();
        let success = rust.launch_notifications == 1
            && rust.session_stats_writes >= 1
            && rust.session_stats_error.is_none()
            && self.last_launch_game_id().to_string() == game_id
            && self.last_launch_target_id().to_string() == application_id
            && *self.last_launch_succeeded()
            && !*self.launching()
            && !*self.launch_session_active();
        if success {
            eprintln!(
                "ADDITIONAL_APP_LAUNCH_SMOKE_COMPLETE game={game_id} application={application_id} launches={} stats_writes={}",
                rust.launch_notifications,
                rust.session_stats_writes,
            );
        }
        success
    }

    pub fn report_path_mapping_smoke_success(&self, expected_count: i32) -> bool {
        let success = self.rust().path_mappings_initialized
            && *self.path_mapping_count() == expected_count
            && self.rust().path_mappings.len() == usize::try_from(expected_count).unwrap_or(0)
            && !self.path_mapping_settings_path().is_empty();
        if success {
            eprintln!(
                "PATH_MAPPING_SMOKE_COMPLETE mappings={expected_count} settings={}",
                self.path_mapping_settings_path()
            );
        }
        success
    }

    pub fn row_for_game_id(&self, game_id: QString) -> i32 {
        let game_id = game_id.to_string();
        self.rust()
            .filtered_indices
            .iter()
            .position(|index| self.rust().games[*index].id == game_id)
            .map(saturating_i32)
            .unwrap_or(-1)
    }

    pub fn game_id_at(&self, row: i32) -> QString {
        self.filtered_game(row)
            .map(|game| qstring(&game.id))
            .unwrap_or_default()
    }

    pub fn row_count(&self, parent: &QModelIndex) -> i32 {
        if parent.is_valid() {
            0
        } else {
            saturating_i32(self.rust().filtered_indices.len())
        }
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        if !index.is_valid() || index.column() != 0 {
            return QVariant::default();
        }
        let Some(game) = self.filtered_game(index.row()) else {
            return QVariant::default();
        };

        match role {
            DISPLAY_ROLE | GAME_TITLE_ROLE => QVariant::from(&qstring(&game.title)),
            GAME_ID_ROLE => QVariant::from(&qstring(&game.id)),
            GAME_PLATFORM_ROLE => QVariant::from(&qstring(&game.platform)),
            GAME_FAVORITE_ROLE => QVariant::from(&game.favorite),
            GAME_COMPLETED_ROLE => QVariant::from(&game.completed),
            GAME_PLAY_COUNT_ROLE => QVariant::from(&saturating_i32(game.play_count as usize)),
            GAME_STAR_RATING_ROLE => QVariant::from(&i32::from(game.star_rating)),
            GAME_ADDITIONAL_APPLICATION_COUNT_ROLE => QVariant::from(&saturating_i32(
                self.rust()
                    .additional_applications_by_game
                    .get(&game.id)
                    .map(Vec::len)
                    .unwrap_or_default(),
            )),
            GAME_SORT_TITLE_ROLE => {
                QVariant::from(&qstring(game.sort_title.as_deref().unwrap_or_default()))
            }
            GAME_NOTES_ROLE => QVariant::from(&qstring(game.notes.as_deref().unwrap_or_default())),
            GAME_DEVELOPER_ROLE => {
                QVariant::from(&qstring(game.developer.as_deref().unwrap_or_default()))
            }
            GAME_GENRE_ROLE => QVariant::from(&qstring(game.genre.as_deref().unwrap_or_default())),
            GAME_MAX_PLAYERS_ROLE => QVariant::from(
                &game
                    .max_players
                    .and_then(|value| i32::try_from(value).ok())
                    .unwrap_or_default(),
            ),
            GAME_PLAY_MODE_ROLE => {
                QVariant::from(&qstring(game.play_mode.as_deref().unwrap_or_default()))
            }
            GAME_PROGRESS_ROLE => {
                QVariant::from(&qstring(game.progress.as_deref().unwrap_or_default()))
            }
            GAME_PUBLISHER_ROLE => {
                QVariant::from(&qstring(game.publisher.as_deref().unwrap_or_default()))
            }
            GAME_RATING_ROLE => {
                QVariant::from(&qstring(game.rating.as_deref().unwrap_or_default()))
            }
            GAME_REGION_ROLE => {
                QVariant::from(&qstring(game.region.as_deref().unwrap_or_default()))
            }
            GAME_RELEASE_DATE_ROLE => {
                QVariant::from(&qstring(game.release_date.as_deref().unwrap_or_default()))
            }
            GAME_RELEASE_TYPE_ROLE => {
                QVariant::from(&qstring(game.release_type.as_deref().unwrap_or_default()))
            }
            GAME_SERIES_ROLE => {
                QVariant::from(&qstring(game.series.as_deref().unwrap_or_default()))
            }
            GAME_SOURCE_ROLE => {
                QVariant::from(&qstring(game.source.as_deref().unwrap_or_default()))
            }
            GAME_STATUS_ROLE => {
                QVariant::from(&qstring(game.status.as_deref().unwrap_or_default()))
            }
            GAME_VERSION_ROLE => {
                QVariant::from(&qstring(game.version.as_deref().unwrap_or_default()))
            }
            GAME_WIKIPEDIA_URL_ROLE => {
                QVariant::from(&qstring(game.wikipedia_url.as_deref().unwrap_or_default()))
            }
            GAME_APPLICATION_PATH_ROLE => QVariant::from(&qstring(&game.application_path)),
            GAME_COMMAND_LINE_ROLE => {
                QVariant::from(&qstring(game.command_line.as_deref().unwrap_or_default()))
            }
            GAME_EMULATOR_ID_ROLE => {
                QVariant::from(&qstring(game.emulator_id.as_deref().unwrap_or_default()))
            }
            GAME_USE_DOS_BOX_ROLE => QVariant::from(&game.use_dos_box),
            GAME_CUSTOM_DOS_BOX_VERSION_PATH_ROLE => QVariant::from(&qstring(
                game.custom_dos_box_version_path
                    .as_deref()
                    .unwrap_or_default(),
            )),
            GAME_DOS_BOX_CONFIGURATION_PATH_ROLE => QVariant::from(&qstring(
                game.dos_box_configuration_path
                    .as_deref()
                    .unwrap_or_default(),
            )),
            GAME_USE_SCUMM_VM_ROLE => QVariant::from(&game.use_scumm_vm),
            GAME_SCUMM_VM_ASPECT_CORRECTION_ROLE => {
                QVariant::from(&game.scumm_vm_aspect_correction)
            }
            GAME_SCUMM_VM_FULLSCREEN_ROLE => QVariant::from(&game.scumm_vm_fullscreen),
            GAME_SCUMM_VM_GAME_DATA_FOLDER_PATH_ROLE => QVariant::from(&qstring(
                game.scumm_vm_game_data_folder_path
                    .as_deref()
                    .unwrap_or_default(),
            )),
            GAME_SCUMM_VM_GAME_TYPE_ROLE => QVariant::from(&qstring(
                game.scumm_vm_game_type.as_deref().unwrap_or_default(),
            )),
            _ => QVariant::default(),
        }
    }

    pub fn role_names(&self) -> RoleNames {
        let mut roles = RoleNames::default();
        for (role, name) in GAME_ROLES {
            roles.insert(role, QByteArray::from(name));
        }
        roles
    }

    pub fn platform_name_at(&self, index: i32) -> QString {
        usize::try_from(index)
            .ok()
            .and_then(|index| self.rust().platform_counts.get(index))
            .map(|platform| qstring(&platform.name))
            .unwrap_or_default()
    }

    pub fn platform_game_count_at(&self, index: i32) -> i32 {
        usize::try_from(index)
            .ok()
            .and_then(|index| self.rust().platform_counts.get(index))
            .map(|platform| saturating_i32(platform.count))
            .unwrap_or_default()
    }

    pub fn emulator_entry_count(&self) -> i32 {
        let configured = self
            .rust()
            .emulator_configuration
            .as_ref()
            .map(|configuration| configuration.emulators.len())
            .unwrap_or_default();
        saturating_i32(configured.saturating_add(2))
    }

    pub fn emulator_id_at(&self, index: i32) -> QString {
        match index {
            0 => QString::default(),
            1 => qstring(UNASSIGNED_EMULATOR_ID),
            _ => usize::try_from(index - 2)
                .ok()
                .and_then(|index| {
                    self.rust()
                        .emulator_configuration
                        .as_ref()?
                        .emulators
                        .get(index)
                })
                .map(|emulator| qstring(&emulator.id))
                .unwrap_or_default(),
        }
    }

    pub fn emulator_title_at(&self, index: i32) -> QString {
        match index {
            0 => qstring("Platform default"),
            1 => qstring("No emulator (direct)"),
            _ => usize::try_from(index - 2)
                .ok()
                .and_then(|index| {
                    self.rust()
                        .emulator_configuration
                        .as_ref()?
                        .emulators
                        .get(index)
                })
                .map(|emulator| qstring(&emulator.title))
                .unwrap_or_default(),
        }
    }

    pub fn additional_application_count(&self, row: i32, game_id: QString) -> i32 {
        self.additional_applications_for_model(row, &game_id.to_string())
            .map(|applications| saturating_i32(applications.len()))
            .unwrap_or_default()
    }

    pub fn additional_application_id_at(&self, row: i32, game_id: QString, index: i32) -> QString {
        self.additional_application_at(row, &game_id.to_string(), index)
            .map(|application| qstring(&application.id))
            .unwrap_or_default()
    }

    pub fn additional_application_name_at(
        &self,
        row: i32,
        game_id: QString,
        index: i32,
    ) -> QString {
        self.additional_application_at(row, &game_id.to_string(), index)
            .map(|application| qstring(&application.name))
            .unwrap_or_default()
    }

    pub fn alternate_name_count(&self, row: i32, game_id: QString) -> i32 {
        self.alternate_names_for_model(row, &game_id.to_string())
            .map(|alternate_names| saturating_i32(alternate_names.len()))
            .unwrap_or_default()
    }

    pub fn alternate_name_name_at(&self, row: i32, game_id: QString, index: i32) -> QString {
        self.alternate_name_at(row, &game_id.to_string(), index)
            .map(|alternate_name| qstring(&alternate_name.name))
            .unwrap_or_default()
    }

    pub fn alternate_name_region_at(&self, row: i32, game_id: QString, index: i32) -> QString {
        self.alternate_name_at(row, &game_id.to_string(), index)
            .map(|alternate_name| qstring(alternate_name.region.as_deref().unwrap_or_default()))
            .unwrap_or_default()
    }

    pub fn custom_field_count(&self, row: i32, game_id: QString) -> i32 {
        self.custom_fields_for_model(row, &game_id.to_string())
            .map(|custom_fields| saturating_i32(custom_fields.len()))
            .unwrap_or_default()
    }

    pub fn custom_field_name_at(&self, row: i32, game_id: QString, index: i32) -> QString {
        self.custom_field_at(row, &game_id.to_string(), index)
            .map(|custom_field| qstring(&custom_field.name))
            .unwrap_or_default()
    }

    pub fn custom_field_value_at(&self, row: i32, game_id: QString, index: i32) -> QString {
        self.custom_field_at(row, &game_id.to_string(), index)
            .map(|custom_field| qstring(&custom_field.value))
            .unwrap_or_default()
    }

    fn persist_path_mappings(
        mut self: Pin<&mut Self>,
        mappings: HostPathMappings,
        success_message: String,
    ) -> bool {
        let Some(path) = self.as_ref().rust().path_mapping_settings_file.clone() else {
            self.as_mut().set_status_message(qstring(
                "Host path mapping settings have not been initialized.",
            ));
            return false;
        };
        let resolver = match mappings
            .resolver()
            .map_err(|error| error.to_string())
            .and_then(path_resolver_from_command_line)
        {
            Ok(resolver) => resolver,
            Err(error) => {
                self.as_mut().set_status_message(qstring(format!(
                    "Could not apply host path mappings: {error}"
                )));
                return false;
            }
        };
        if let Err(error) = mappings.save_atomic(&path) {
            self.as_mut().set_status_message(qstring(format!(
                "Could not save host path mappings: {error}"
            )));
            return false;
        }
        let count = i32::try_from(mappings.len()).unwrap_or(i32::MAX);
        self.as_mut().rust_mut().path_mappings = mappings;
        self.as_mut().rust_mut().path_resolver = resolver;
        self.as_mut().set_path_mapping_count(count);
        self.as_mut().set_status_message(qstring(success_message));
        true
    }

    fn advance_generation(mut self: Pin<&mut Self>) -> u64 {
        let mut rust = self.as_mut().rust_mut();
        rust.request_generation = rust.request_generation.wrapping_add(1);
        rust.request_generation
    }

    fn begin_library_mutation(mut self: Pin<&mut Self>) -> bool {
        if *self.as_ref().loading() || *self.as_ref().writing() || *self.as_ref().launching() {
            self.as_mut()
                .set_status_message(qstring("Wait for the current library operation to finish."));
            return false;
        }
        if *self.as_ref().pending_recovery_count() > 0 {
            self.as_mut().set_status_message(qstring(
                "Recover the interrupted transaction before editing this library.",
            ));
            return false;
        }
        if *self.as_ref().write_conflict() {
            self.as_mut().set_status_message(qstring(
                "Reload the library before retrying an edit after a write conflict.",
            ));
            return false;
        }
        true
    }

    fn finish_background_load(
        mut self: Pin<&mut Self>,
        generation: u64,
        loaded: Result<LoadedLibrary, String>,
    ) {
        if self.as_ref().rust().request_generation != generation {
            return;
        }
        self.as_mut().set_loading(false);
        match loaded {
            Ok(loaded) => {
                eprintln!("{}", loaded.message);
                let path = loaded.path;
                self.as_mut().replace_library(LibraryReplacement {
                    games: loaded.games,
                    game_sources: loaded.game_sources,
                    additional_applications_by_game: loaded.additional_applications_by_game,
                    mounts_by_game: loaded.mounts_by_game,
                    alternate_names_by_game: loaded.alternate_names_by_game,
                    custom_fields_by_game: loaded.custom_fields_by_game,
                    platform_names: loaded.platform_names,
                    platform_sources: loaded.platform_sources,
                    library_root: Some(loaded.root),
                    launchbox_root: loaded.launchbox_root,
                    emulator_configuration: loaded.emulator_configuration,
                    name: loaded.name,
                    message: loaded.message,
                    pending_recovery_count: loaded.pending_recovery_count,
                });
                self.as_mut().set_library_path(qstring(path));
                self.as_mut().set_write_conflict(false);
            }
            Err(error) => {
                eprintln!("Could not load library: {error}");
                self.as_mut()
                    .set_status_message(qstring(format!("Could not load library: {error}")));
            }
        }
    }

    fn finish_game_write(
        mut self: Pin<&mut Self>,
        generation: u64,
        result: Result<GameWriteSuccess, GameWriteFailure>,
    ) {
        self.as_mut().set_writing(false);
        if self.as_ref().rust().request_generation != generation {
            return;
        }

        match result {
            Ok(written) => {
                let actual_index = {
                    let this = self.as_ref();
                    let rust = this.rust();
                    rust.games
                        .iter()
                        .zip(&rust.game_sources)
                        .position(|(game, source)| {
                            game.id == written.game.id && *source == written.source
                        })
                };
                let Some(actual_index) = actual_index else {
                    self.as_mut().set_status_message(qstring(
                        "The saved game is no longer present in the loaded model; reload required.",
                    ));
                    return;
                };
                let GameWriteSuccess {
                    game,
                    alternate_names,
                    custom_fields,
                    source: _,
                    backup,
                } = written;
                let filtered_row = self
                    .as_ref()
                    .rust()
                    .filtered_indices
                    .iter()
                    .position(|index| *index == actual_index);
                let metadata_changed =
                    GameMetadata::from(&self.as_ref().rust().games[actual_index])
                        != GameMetadata::from(&game);
                let game_id = game.id.clone();
                {
                    let mut rust = self.as_mut().rust_mut();
                    rust.games[actual_index] = game;
                    if alternate_names.is_empty() {
                        rust.alternate_names_by_game.remove(&game_id);
                    } else {
                        rust.alternate_names_by_game
                            .insert(game_id.clone(), alternate_names);
                    }
                    if custom_fields.is_empty() {
                        rust.custom_fields_by_game.remove(&game_id);
                    } else {
                        rust.custom_fields_by_game.insert(game_id, custom_fields);
                    }
                }
                self.as_mut().set_write_conflict(false);
                self.as_mut().set_status_message(qstring(format!(
                    "Saved game. Exact backup: {}",
                    backup.display()
                )));

                if metadata_changed {
                    self.as_mut().refresh_filtered_games();
                } else if let Some(filtered_row) = filtered_row {
                    let row = saturating_i32(filtered_row);
                    let parent = QModelIndex::default();
                    let index = self.as_ref().model_index(row, 0, &parent);
                    let mut roles = QList::<i32>::default();
                    for role in EDITABLE_GAME_ROLES {
                        roles.append_clone(&role);
                    }
                    self.as_mut().rust_mut().data_change_notifications += 1;
                    self.as_mut().emit_data_changed(&index, &index, &roles);
                }
            }
            Err(GameWriteFailure::Conflict(message)) => {
                self.as_mut().set_write_conflict(true);
                self.as_mut().set_status_message(qstring(format!(
                    "Write conflict: {message}. Reload before retrying."
                )));
            }
            Err(GameWriteFailure::PendingRecovery { count, message }) => {
                self.as_mut()
                    .set_pending_recovery_count(saturating_i32(count));
                self.as_mut().set_status_message(qstring(format!(
                    "Interrupted transaction requires recovery: {message}"
                )));
            }
            Err(GameWriteFailure::Other(message)) => {
                self.as_mut()
                    .set_status_message(qstring(format!("Could not save game state: {message}")));
            }
            Err(GameWriteFailure::Referenced(references)) => {
                self.as_mut().set_status_message(qstring(format!(
                    "Could not save game: {} dependent records were reported unexpectedly.",
                    references.len()
                )));
            }
        }
    }

    fn finish_game_launch(
        mut self: Pin<&mut Self>,
        generation: u64,
        result: Result<GameLaunchSuccess, String>,
    ) {
        self.as_mut().set_launching(false);
        if self.as_ref().rust().request_generation != generation {
            return;
        }
        match result {
            Ok(launched) => {
                self.as_mut().rust_mut().launch_notifications =
                    self.as_ref().rust().launch_notifications.saturating_add(1);
                self.as_mut().set_last_launch_succeeded(true);
                self.as_mut()
                    .set_last_launch_game_id(qstring(&launched.game_id));
                self.as_mut()
                    .set_last_launch_target_id(qstring(&launched.target_id));
                self.as_mut().set_status_message(qstring(format!(
                    "Launched {} with {} (PID {}) from {}.",
                    launched.game_title,
                    launched.kind,
                    launched.pid,
                    launched.executable.display()
                )));
            }
            Err(error) => {
                self.as_mut().set_last_launch_succeeded(false);
                self.as_mut().set_last_launch_game_id(QString::default());
                self.as_mut().set_last_launch_target_id(QString::default());
                self.as_mut()
                    .set_status_message(qstring(format!("Could not launch game: {error}")));
            }
        }
    }

    fn finish_play_session_stats(
        mut self: Pin<&mut Self>,
        generation: u64,
        result: Result<PlaySessionStatsWriteSuccess, GameWriteFailure>,
    ) {
        if self.as_ref().rust().request_generation != generation {
            return;
        }
        match result {
            Ok(updated) => {
                self.as_mut().rust_mut().session_stats_writes =
                    self.as_ref().rust().session_stats_writes.saturating_add(1);
                let _backup = updated.backup;
                match updated.record {
                    PlaySessionStatsRecord::Game(game) => {
                        let Some(actual_index) = self
                            .as_ref()
                            .rust()
                            .games
                            .iter()
                            .position(|candidate| candidate.id == game.id)
                        else {
                            self.as_mut().rust_mut().session_stats_error = Some(format!(
                                "updated game {} disappeared from the loaded model",
                                game.id
                            ));
                            return;
                        };
                        let play_count_changed =
                            self.as_ref().rust().games[actual_index].play_count != game.play_count;
                        self.as_mut().rust_mut().games[actual_index] = *game;
                        if play_count_changed {
                            let filtered_row = self
                                .as_ref()
                                .rust()
                                .filtered_indices
                                .iter()
                                .position(|index| *index == actual_index);
                            if let Some(row) = filtered_row {
                                let parent = QModelIndex::default();
                                let index =
                                    self.as_ref().model_index(saturating_i32(row), 0, &parent);
                                let mut roles = QList::<i32>::default();
                                roles.append(GAME_PLAY_COUNT_ROLE);
                                self.as_mut().rust_mut().data_change_notifications = self
                                    .as_ref()
                                    .rust()
                                    .data_change_notifications
                                    .saturating_add(1);
                                self.as_mut().emit_data_changed(&index, &index, &roles);
                            }
                        }
                    }
                    PlaySessionStatsRecord::AdditionalApplication(application) => {
                        let game_id = application.game_id.clone();
                        let parent_loaded = self
                            .as_ref()
                            .rust()
                            .additional_applications_by_game
                            .contains_key(&game_id);
                        let application_index = self
                            .as_ref()
                            .rust()
                            .additional_applications_by_game
                            .get(&game_id)
                            .and_then(|applications| {
                                applications
                                    .iter()
                                    .position(|candidate| candidate.id == application.id)
                            });
                        let Some(application_index) = application_index else {
                            let message = if parent_loaded {
                                format!(
                                    "updated additional application {} disappeared from the loaded model",
                                    application.id
                                )
                            } else {
                                format!(
                                    "updated additional application {} has no loaded parent",
                                    application.id
                                )
                            };
                            self.as_mut().rust_mut().session_stats_error = Some(message);
                            return;
                        };
                        self.as_mut()
                            .rust_mut()
                            .additional_applications_by_game
                            .get_mut(&game_id)
                            .expect("additional-application parent checked above")
                            [application_index] = *application;
                    }
                }
            }
            Err(error) => {
                let message = describe_game_write_failure(&error);
                self.as_mut().rust_mut().session_stats_error = Some(message.clone());
                match error {
                    GameWriteFailure::Conflict(_) => self.as_mut().set_write_conflict(true),
                    GameWriteFailure::PendingRecovery { count, .. } => self
                        .as_mut()
                        .set_pending_recovery_count(saturating_i32(count)),
                    GameWriteFailure::Referenced(_) | GameWriteFailure::Other(_) => {}
                }
                self.as_mut().set_status_message(qstring(format!(
                    "The application started, but play statistics could not be saved: {message}"
                )));
            }
        }
    }

    fn finish_launch_session(
        mut self: Pin<&mut Self>,
        generation: u64,
        result: Result<LaunchSequenceReport, String>,
        primary_started: bool,
        session_stats_errors: Vec<String>,
    ) {
        if self.as_ref().rust().request_generation != generation {
            return;
        }
        self.as_mut().set_launch_session_active(false);
        if !primary_started {
            self.as_mut().set_launching(false);
        }
        match result {
            Ok(report) => {
                let target = match &report.primary_target {
                    LaunchTarget::MainGame => report.game_title.clone(),
                    LaunchTarget::AdditionalApplication {
                        application_name, ..
                    } => format!("{} ({application_name})", report.game_title),
                };
                let exit = if report.primary_exit_success {
                    "successfully"
                } else {
                    "with an unsuccessful status"
                };
                let mut message = format!(
                    "Session for {target} ended {exit} after {} second(s).",
                    report.primary_runtime.as_secs()
                );
                if report.automatic_before_started > 0
                    || report.automatic_after_started > 0
                    || report.before_wait_timeouts > 0
                {
                    message.push_str(&format!(
                        " Started {} before-app(s) and {} after-app(s); {} before-app wait(s) reached the 30-second ceiling.",
                        report.automatic_before_started,
                        report.automatic_after_started,
                        report.before_wait_timeouts,
                    ));
                }
                if !session_stats_errors.is_empty() {
                    message.push_str(&format!(
                        " Play statistics were not fully saved: {}.",
                        session_stats_errors.join("; ")
                    ));
                }
                self.as_mut().set_status_message(qstring(message));
            }
            Err(error) if primary_started => {
                self.as_mut().set_status_message(qstring(format!(
                    "The primary application started, but its launch lifecycle failed: {error}"
                )));
            }
            Err(error) => {
                self.as_mut().set_last_launch_succeeded(false);
                self.as_mut().set_last_launch_game_id(QString::default());
                self.as_mut().set_last_launch_target_id(QString::default());
                self.as_mut()
                    .set_status_message(qstring(format!("Could not launch game: {error}")));
            }
        }
    }

    fn finish_game_add(
        mut self: Pin<&mut Self>,
        generation: u64,
        result: Result<GameAddSuccess, GameWriteFailure>,
    ) {
        self.as_mut().set_writing(false);
        if self.as_ref().rust().request_generation != generation {
            return;
        }
        match result {
            Ok(added) => {
                let actual_index = self.as_ref().rust().games.len();
                let insertion_row = self.as_ref().game_insertion_row(&added.game);
                if let Some(row) = insertion_row {
                    let row = saturating_i32(row);
                    let parent = QModelIndex::default();
                    self.as_mut().begin_insert_rows(&parent, row, row);
                }
                {
                    let mut rust = self.as_mut().rust_mut();
                    rust.games.push(added.game.clone());
                    rust.game_sources.push(added.source);
                    if let Some(row) = insertion_row {
                        rust.filtered_indices.insert(row, actual_index);
                        rust.row_insert_notifications += 1;
                    }
                }
                if insertion_row.is_some() {
                    self.as_mut().end_insert_rows();
                }
                self.as_mut().update_library_counts();
                self.as_mut().set_write_conflict(false);
                self.as_mut()
                    .set_last_added_game_id(qstring(&added.game.id));
                self.as_mut().set_status_message(qstring(format!(
                    "Added {}. Exact backup: {}",
                    added.game.title,
                    added.backup.display()
                )));
            }
            Err(GameWriteFailure::Conflict(message)) => {
                self.as_mut().set_write_conflict(true);
                self.as_mut().set_status_message(qstring(format!(
                    "Write conflict while adding game: {message}. Reload before retrying."
                )));
            }
            Err(GameWriteFailure::PendingRecovery { count, message }) => {
                self.as_mut()
                    .set_pending_recovery_count(saturating_i32(count));
                self.as_mut().set_status_message(qstring(format!(
                    "Interrupted transaction requires recovery: {message}"
                )));
            }
            Err(GameWriteFailure::Referenced(references)) => {
                self.as_mut().set_status_message(qstring(format!(
                    "Could not add game: unexpected reference result ({})",
                    references.len()
                )));
            }
            Err(GameWriteFailure::Other(message)) => self
                .as_mut()
                .set_status_message(qstring(format!("Could not add game: {message}"))),
        }
    }

    fn finish_platform_create(
        mut self: Pin<&mut Self>,
        generation: u64,
        result: Result<PlatformCreateSuccess, PlatformWriteFailure>,
    ) {
        self.as_mut().set_writing(false);
        if self.as_ref().rust().request_generation != generation {
            return;
        }
        match result {
            Ok(created) => {
                {
                    let mut rust = self.as_mut().rust_mut();
                    rust.platform_names.push(created.name.clone());
                    rust.platform_names.sort_by_key(|name| platform_key(name));
                    rust.platform_names
                        .dedup_by(|left, right| platform_key(left) == platform_key(right));
                    rust.platform_sources
                        .insert(platform_key(&created.name), created.source.clone());
                }
                self.as_mut().update_library_counts();
                let platform_count = self.as_ref().rust().platform_names.len();
                self.as_mut().set_library_name(qstring(format!(
                    "LaunchBox Library ({platform_count} platforms)"
                )));
                self.as_mut().set_write_conflict(false);
                self.as_mut().set_status_message(qstring(format!(
                    "Created {} with {} default media-folder records at {}. Exact catalog backup: {}. No media directories were created.",
                    created.name,
                    created.folder_count,
                    created.source.display(),
                    created.catalog_backup.display()
                )));
            }
            Err(PlatformWriteFailure::Conflict(message)) => {
                self.as_mut().set_write_conflict(true);
                self.as_mut().set_status_message(qstring(format!(
                    "Write conflict while creating platform: {message}. Reload before retrying."
                )));
            }
            Err(PlatformWriteFailure::PendingRecovery { count, message }) => {
                self.as_mut()
                    .set_pending_recovery_count(saturating_i32(count));
                self.as_mut().set_status_message(qstring(format!(
                    "Interrupted transaction requires recovery: {message}"
                )));
            }
            Err(PlatformWriteFailure::Referenced(references)) => {
                self.as_mut().set_status_message(qstring(format!(
                    "Could not create platform: unexpected reference result ({})",
                    references.len()
                )));
            }
            Err(PlatformWriteFailure::Other(message)) => self
                .as_mut()
                .set_status_message(qstring(format!("Could not create platform: {message}"))),
        }
    }

    fn finish_platform_delete(
        mut self: Pin<&mut Self>,
        generation: u64,
        result: Result<PlatformDeleteSuccess, PlatformWriteFailure>,
    ) {
        self.as_mut().set_writing(false);
        if self.as_ref().rust().request_generation != generation {
            return;
        }
        match result {
            Ok(deleted) => {
                {
                    let mut rust = self.as_mut().rust_mut();
                    rust.platform_names
                        .retain(|name| !name.eq_ignore_ascii_case(&deleted.name));
                    rust.platform_sources.remove(&platform_key(&deleted.name));
                }
                if self
                    .as_ref()
                    .platform_filter()
                    .to_string()
                    .eq_ignore_ascii_case(&deleted.name)
                {
                    self.as_mut().set_platform_filter(QString::default());
                    self.as_mut().refresh_filtered_games();
                }
                self.as_mut().update_library_counts();
                let platform_count = self.as_ref().rust().platform_names.len();
                self.as_mut().set_library_name(qstring(format!(
                    "LaunchBox Library ({platform_count} platforms)"
                )));
                self.as_mut().set_write_conflict(false);
                self.as_mut().set_status_message(qstring(format!(
                    "Deleted {} and {} owned media-folder records. Exact catalog backup: {}. Exact platform backup: {}. No media files or directories were deleted (former document: {}).",
                    deleted.name,
                    deleted.folder_count,
                    deleted.catalog_backup.display(),
                    deleted.platform_backup.display(),
                    deleted.source.display()
                )));
            }
            Err(PlatformWriteFailure::Referenced(references)) => {
                let summary = summarize_platform_references(&references);
                self.as_mut()
                    .set_delete_blocker_count(saturating_i32(references.len()));
                self.as_mut().set_delete_blocker_summary(qstring(&summary));
                self.as_mut().set_status_message(qstring(format!(
                    "Platform delete blocked by {} dependent record(s): {summary}",
                    references.len()
                )));
            }
            Err(PlatformWriteFailure::Conflict(message)) => {
                self.as_mut().set_write_conflict(true);
                self.as_mut().set_status_message(qstring(format!(
                    "Write conflict while deleting platform: {message}. Reload before retrying."
                )));
            }
            Err(PlatformWriteFailure::PendingRecovery { count, message }) => {
                self.as_mut()
                    .set_pending_recovery_count(saturating_i32(count));
                self.as_mut().set_status_message(qstring(format!(
                    "Interrupted transaction requires recovery: {message}"
                )));
            }
            Err(PlatformWriteFailure::Other(message)) => self
                .as_mut()
                .set_status_message(qstring(format!("Could not delete platform: {message}"))),
        }
    }

    fn finish_game_delete(
        mut self: Pin<&mut Self>,
        generation: u64,
        result: Result<GameDeleteSuccess, GameWriteFailure>,
    ) {
        self.as_mut().set_writing(false);
        if self.as_ref().rust().request_generation != generation {
            return;
        }
        match result {
            Ok(deleted) => {
                let actual_index = {
                    let this = self.as_ref();
                    let rust = this.rust();
                    rust.games
                        .iter()
                        .zip(&rust.game_sources)
                        .position(|(game, source)| {
                            game.id == deleted.game.id && *source == deleted.source
                        })
                };
                let Some(actual_index) = actual_index else {
                    self.as_mut().set_status_message(qstring(
                        "The deleted game is no longer present in the loaded model; reload required.",
                    ));
                    return;
                };
                let filtered_row = self
                    .as_ref()
                    .rust()
                    .filtered_indices
                    .iter()
                    .position(|index| *index == actual_index);
                if let Some(row) = filtered_row {
                    let row = saturating_i32(row);
                    let parent = QModelIndex::default();
                    self.as_mut().begin_remove_rows(&parent, row, row);
                }
                {
                    let mut rust = self.as_mut().rust_mut();
                    rust.games.remove(actual_index);
                    rust.game_sources.remove(actual_index);
                    rust.filtered_indices.retain(|index| *index != actual_index);
                    for index in &mut rust.filtered_indices {
                        if *index > actual_index {
                            *index -= 1;
                        }
                    }
                    if filtered_row.is_some() {
                        rust.row_remove_notifications += 1;
                    }
                }
                if filtered_row.is_some() {
                    self.as_mut().end_remove_rows();
                }
                self.as_mut().update_library_counts();
                self.as_mut().set_write_conflict(false);
                self.as_mut().set_status_message(qstring(format!(
                    "Deleted {}. Exact backup: {}",
                    deleted.game.title,
                    deleted.backup.display()
                )));
            }
            Err(GameWriteFailure::Referenced(references)) => {
                let summary = summarize_game_references(&references);
                self.as_mut()
                    .set_delete_blocker_count(saturating_i32(references.len()));
                self.as_mut().set_delete_blocker_summary(qstring(&summary));
                self.as_mut().set_status_message(qstring(format!(
                    "Delete blocked by {} dependent record(s): {summary}",
                    references.len()
                )));
            }
            Err(GameWriteFailure::Conflict(message)) => {
                self.as_mut().set_write_conflict(true);
                self.as_mut().set_status_message(qstring(format!(
                    "Write conflict while deleting game: {message}. Reload before retrying."
                )));
            }
            Err(GameWriteFailure::PendingRecovery { count, message }) => {
                self.as_mut()
                    .set_pending_recovery_count(saturating_i32(count));
                self.as_mut().set_status_message(qstring(format!(
                    "Interrupted transaction requires recovery: {message}"
                )));
            }
            Err(GameWriteFailure::Other(message)) => self
                .as_mut()
                .set_status_message(qstring(format!("Could not delete game: {message}"))),
        }
    }

    fn finish_recovery(mut self: Pin<&mut Self>, result: Result<usize, String>) {
        self.as_mut().set_writing(false);
        match result {
            Ok(count) => {
                self.as_mut().set_pending_recovery_count(0);
                self.as_mut().set_write_conflict(false);
                eprintln!("Recovered {count} pending library transaction(s).");
                self.as_mut().reload_library();
            }
            Err(error) => {
                self.as_mut().set_status_message(qstring(format!(
                    "Transaction recovery stopped without overwriting divergent data: {error}"
                )));
            }
        }
    }

    fn edit_target(&self, row: i32, game_id: &str) -> Option<(PathBuf, PathBuf)> {
        let filtered_row = usize::try_from(row).ok()?;
        let actual_index = *self.rust().filtered_indices.get(filtered_row)?;
        let game = self.rust().games.get(actual_index)?;
        if game.id != game_id {
            return None;
        }
        let source = self.rust().game_sources.get(actual_index)?.clone();
        let root = self.rust().library_root.clone()?;
        Some((source, root))
    }

    fn platform_write_target(&self, platform: &str) -> Option<(PathBuf, PathBuf)> {
        let source = self
            .rust()
            .platform_sources
            .get(&platform_key(platform))
            .cloned()?;
        let root = self.rust().library_root.clone()?;
        Some((source, root))
    }

    fn game_insertion_row(&self, game: &Game) -> Option<usize> {
        let filter = self.current_filter();
        if filter_game_indices(std::slice::from_ref(game), &filter).is_empty() {
            return None;
        }
        let key = game.display_sort_title().to_lowercase();
        Some(
            self.rust()
                .filtered_indices
                .iter()
                .position(|actual| {
                    let existing = &self.rust().games[*actual];
                    let existing_key = existing.display_sort_title().to_lowercase();
                    key < existing_key || (key == existing_key && game.id < existing.id)
                })
                .unwrap_or(self.rust().filtered_indices.len()),
        )
    }

    fn current_filter(&self) -> GameFilter {
        let platform = self.platform_filter().to_string();
        GameFilter {
            text: self.search_text().to_string(),
            platform: (!platform.is_empty()).then_some(platform),
            ..GameFilter::default()
        }
    }

    fn update_library_counts(mut self: Pin<&mut Self>) {
        let (game_count, filtered_count, platform_counts) = {
            let this = self.as_ref();
            let rust = this.rust();
            (
                saturating_i32(rust.games.len()),
                saturating_i32(rust.filtered_indices.len()),
                collect_platform_counts(&rust.games, &rust.platform_names),
            )
        };
        let platform_entry_count = saturating_i32(platform_counts.len());
        self.as_mut().rust_mut().platform_counts = platform_counts;
        self.as_mut().set_game_count(game_count);
        self.as_mut().set_filtered_count(filtered_count);
        self.as_mut().set_platform_entry_count(platform_entry_count);
        let revision = self.as_ref().rust().platform_revision.wrapping_add(1);
        self.as_mut().set_platform_revision(revision);
    }

    fn replace_library(mut self: Pin<&mut Self>, replacement: LibraryReplacement) {
        let LibraryReplacement {
            games,
            game_sources,
            additional_applications_by_game,
            mounts_by_game,
            alternate_names_by_game,
            custom_fields_by_game,
            platform_names,
            platform_sources,
            library_root,
            launchbox_root,
            emulator_configuration,
            name,
            message,
            pending_recovery_count,
        } = replacement;
        debug_assert!(game_sources.is_empty() || game_sources.len() == games.len());
        let game_count = saturating_i32(games.len());
        let platform_counts = collect_platform_counts(&games, &platform_names);
        let platform_entry_count = saturating_i32(platform_counts.len());
        let filtered_indices = (0..games.len()).collect();
        self.as_mut().begin_reset_model();
        {
            let mut rust = self.as_mut().rust_mut();
            rust.games = games;
            rust.game_sources = game_sources;
            rust.additional_applications_by_game = additional_applications_by_game;
            rust.mounts_by_game = mounts_by_game;
            rust.alternate_names_by_game = alternate_names_by_game;
            rust.custom_fields_by_game = custom_fields_by_game;
            rust.filtered_indices = filtered_indices;
            rust.platform_counts = platform_counts;
            rust.platform_names = platform_names;
            rust.platform_sources = platform_sources;
            rust.library_root = library_root;
            rust.launchbox_root = launchbox_root;
            rust.emulator_configuration = emulator_configuration;
            rust.model_reset_notifications = 1;
            rust.data_change_notifications = 0;
            rust.row_insert_notifications = 0;
            rust.row_remove_notifications = 0;
            rust.launch_notifications = 0;
        }
        self.as_mut().end_reset_model();
        self.as_mut().set_library_name(qstring(name));
        self.as_mut().set_status_message(qstring(message));
        self.as_mut().set_game_count(game_count);
        self.as_mut().set_filtered_count(game_count);
        self.as_mut().set_platform_entry_count(platform_entry_count);
        let revision = self.as_ref().rust().platform_revision.wrapping_add(1);
        self.as_mut().set_platform_revision(revision);
        self.as_mut()
            .set_pending_recovery_count(saturating_i32(pending_recovery_count));
        self.as_mut().set_delete_blocker_count(0);
        self.as_mut().set_delete_blocker_summary(QString::default());
        self.as_mut().set_last_added_game_id(QString::default());
        self.as_mut().set_launching(false);
        self.as_mut().set_last_launch_succeeded(false);
        self.as_mut().set_last_launch_game_id(QString::default());
        self.as_mut().set_last_launch_target_id(QString::default());
        self.as_mut().set_launch_session_active(false);
        self.as_mut().set_search_text(QString::default());
        self.as_mut().set_platform_filter(QString::default());
    }

    fn refresh_filtered_games(mut self: Pin<&mut Self>) {
        let search_text = self.as_ref().search_text().to_string();
        let platform = self.as_ref().platform_filter().to_string();
        let indices = {
            let this = self.as_ref();
            let rust = this.rust();
            let filter = GameFilter {
                text: search_text,
                platform: (!platform.is_empty()).then_some(platform),
                ..GameFilter::default()
            };
            filter_game_indices(&rust.games, &filter)
        };
        let count = saturating_i32(indices.len());
        self.as_mut().begin_reset_model();
        {
            let mut rust = self.as_mut().rust_mut();
            rust.filtered_indices = indices;
            rust.model_reset_notifications = rust.model_reset_notifications.saturating_add(1);
        }
        self.as_mut().end_reset_model();
        self.as_mut().set_filtered_count(count);
    }

    fn filtered_game(&self, index: i32) -> Option<&Game> {
        let index = usize::try_from(index).ok()?;
        let game_index = *self.rust().filtered_indices.get(index)?;
        self.rust().games.get(game_index)
    }

    fn additional_applications_for_model(
        &self,
        row: i32,
        game_id: &str,
    ) -> Option<&[AdditionalApplication]> {
        let game = self.filtered_game(row)?;
        if game.id != game_id {
            return None;
        }
        Some(
            self.rust()
                .additional_applications_by_game
                .get(game_id)
                .map(Vec::as_slice)
                .unwrap_or_default(),
        )
    }

    fn additional_application_at(
        &self,
        row: i32,
        game_id: &str,
        index: i32,
    ) -> Option<&AdditionalApplication> {
        let index = usize::try_from(index).ok()?;
        self.additional_applications_for_model(row, game_id)?
            .get(index)
    }

    fn alternate_names_for_model(&self, row: i32, game_id: &str) -> Option<&[AlternateName]> {
        let game = self.filtered_game(row)?;
        if game.id != game_id {
            return None;
        }
        Some(
            self.rust()
                .alternate_names_by_game
                .get(game_id)
                .map(Vec::as_slice)
                .unwrap_or_default(),
        )
    }

    fn alternate_name_at(&self, row: i32, game_id: &str, index: i32) -> Option<&AlternateName> {
        let index = usize::try_from(index).ok()?;
        self.alternate_names_for_model(row, game_id)?.get(index)
    }

    fn custom_fields_for_model(&self, row: i32, game_id: &str) -> Option<&[CustomField]> {
        let game = self.filtered_game(row)?;
        if game.id != game_id {
            return None;
        }
        Some(
            self.rust()
                .custom_fields_by_game
                .get(game_id)
                .map(Vec::as_slice)
                .unwrap_or_default(),
        )
    }

    fn custom_field_at(&self, row: i32, game_id: &str, index: i32) -> Option<&CustomField> {
        let index = usize::try_from(index).ok()?;
        self.custom_fields_for_model(row, game_id)?.get(index)
    }
}

fn collect_platform_counts(games: &[Game], platform_names: &[String]) -> Vec<PlatformCount> {
    let mut counts = BTreeMap::<String, (String, usize)>::new();
    for name in platform_names {
        counts
            .entry(platform_key(name))
            .or_insert_with(|| (name.clone(), 0));
    }
    for game in games {
        let entry = counts
            .entry(platform_key(&game.platform))
            .or_insert_with(|| (game.platform.clone(), 0));
        entry.1 += 1;
    }
    counts
        .into_iter()
        .map(|(_, (name, count))| PlatformCount { name, count })
        .collect()
}

fn summarize_game_references(references: &[GameReference]) -> String {
    let mut counts = BTreeMap::new();
    for reference in references {
        *counts.entry(reference.kind).or_insert(0usize) += 1;
    }
    counts
        .into_iter()
        .map(|(kind, count)| format!("{count} {kind}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn summarize_platform_references(references: &[PlatformReference]) -> String {
    let mut counts = BTreeMap::new();
    for reference in references {
        *counts.entry(reference.kind).or_insert(0usize) += 1;
    }
    counts
        .into_iter()
        .map(|(kind, count)| format!("{count} {kind}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn saturating_i32(value: usize) -> i32 {
    value.try_into().unwrap_or(i32::MAX)
}

fn qstring(value: impl AsRef<str>) -> QString {
    QString::from(value.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lb_storage::FileRevision;
    use std::fs;
    use std::time::UNIX_EPOCH;

    #[test]
    fn transaction_conflicts_are_distinct_from_generic_write_failures() {
        let revision = FileRevision {
            byte_len: 1,
            sha256: "00".into(),
        };
        let conflict = classify_transaction_error(TransactionError::Conflict {
            path: PathBuf::from("platform.xml"),
            expected: revision.clone(),
            actual: revision,
        });
        assert!(matches!(conflict, GameWriteFailure::Conflict(_)));
        assert!(matches!(
            classify_transaction_error(TransactionError::Empty),
            GameWriteFailure::Other(_)
        ));
    }

    #[test]
    fn pending_recovery_preserves_the_manifest_count_for_the_ui() {
        let failure = classify_transaction_error(TransactionError::PendingRecovery {
            root: PathBuf::from("library"),
            manifests: vec![PathBuf::from("one.json"), PathBuf::from("two.json")],
        });
        assert!(matches!(
            failure,
            GameWriteFailure::PendingRecovery { count: 2, .. }
        ));
    }

    #[test]
    fn platform_counts_retain_catalog_entries_without_games() {
        let games = vec![Game {
            platform: "fixture console".into(),
            ..Game::default()
        }];
        let counts =
            collect_platform_counts(&games, &["Fixture Console".into(), "Empty Console".into()]);
        assert_eq!(
            counts
                .iter()
                .map(|entry| (entry.name.as_str(), entry.count))
                .collect::<Vec<_>>(),
            [("Empty Console", 0), ("Fixture Console", 1)]
        );
    }

    #[test]
    fn platform_lifecycle_is_transactional_portable_and_reference_safe() {
        let directory = tempfile::tempdir().expect("temporary library");
        let data_directory = directory.path().join("Data");
        let platform_directory = data_directory.join("Platforms");
        fs::create_dir_all(&platform_directory).expect("create platform directory");
        let fixture_root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/launchbox/Data");
        let catalog_path = data_directory.join("Platforms.xml");
        let mut original_catalog =
            fs::read_to_string(fixture_root.join("Platforms.xml")).expect("read catalog fixture");
        original_catalog = original_catalog.replace(
            "</LaunchBox>",
            "  <FutureCatalogRecord><KeepMe>yes</KeepMe></FutureCatalogRecord>\n</LaunchBox>",
        );
        fs::write(&catalog_path, original_catalog.as_bytes()).expect("write catalog fixture");
        fs::copy(
            fixture_root.join("Platforms/Fixture Console.xml"),
            platform_directory.join("Fixture Console.xml"),
        )
        .expect("copy platform fixture");

        let created = match create_platform_in_library(
            directory.path().to_path_buf(),
            "Dragon 32/64".into(),
            "Dragon 32/64".into(),
        ) {
            Ok(created) => created,
            Err(_) => panic!("platform creation failed"),
        };
        assert_eq!(created.source.file_name().unwrap(), "Dragon 32_64.xml");
        assert_eq!(created.folder_count, 51);
        assert_eq!(
            fs::read(&created.catalog_backup).expect("read catalog backup"),
            original_catalog.as_bytes()
        );
        assert!(created.source.is_file());
        assert!(!directory.path().join("Images").exists());
        assert!(!directory.path().join("Videos").exists());

        let created_catalog_bytes = fs::read(&catalog_path).expect("read created catalog");
        let catalog = AuxiliaryDocument::load(&catalog_path)
            .expect("load created catalog")
            .platform_catalog()
            .expect("parse created catalog");
        let dragon = catalog
            .platforms
            .iter()
            .find(|platform| platform.metadata.name == "Dragon 32/64")
            .expect("created platform definition");
        assert_eq!(dragon.metadata.scrape_as.as_deref(), Some("Dragon 32/64"));
        assert_eq!(
            catalog
                .folders
                .iter()
                .filter(|folder| folder.platform == "Dragon 32/64")
                .count(),
            51
        );
        assert!(catalog.folders.iter().any(|folder| {
            folder.platform == "Dragon 32/64"
                && folder.folder_path == "Images\\Dragon 32_64\\Box - Front"
        }));
        assert!(String::from_utf8_lossy(&created_catalog_bytes).contains("FutureCatalogRecord"));
        let loaded = LoadedLibrary::load(directory.path().to_string_lossy().into_owned())
            .expect("load library with empty portable platform");
        assert_eq!(loaded.games.len(), 3);
        assert!(loaded
            .platform_names
            .iter()
            .any(|name| name == "Dragon 32/64"));
        assert_eq!(
            loaded.platform_sources.get(&platform_key("Dragon 32/64")),
            Some(&created.source)
        );

        let collision = create_platform_in_library(
            directory.path().to_path_buf(),
            "Dragon 32\\64".into(),
            String::new(),
        );
        assert!(matches!(
            collision,
            Err(PlatformWriteFailure::Other(message)) if message.contains("collides")
        ));

        let added = match add_game_to_platform(
            directory.path().to_path_buf(),
            created.source.clone(),
            NewGame {
                id: "dragon-test-game".into(),
                title: "Dragon Test".into(),
                platform: "Dragon 32/64".into(),
                application_path: "Games\\Dragon 32_64\\test.vdk".into(),
            },
        ) {
            Ok(added) => added,
            Err(error) => panic!(
                "add to empty platform failed: {}",
                describe_game_write_failure(&error)
            ),
        };
        assert_eq!(added.game.platform, "Dragon 32/64");
        let blocked = delete_platform_from_library(
            directory.path().to_path_buf(),
            created.source.clone(),
            "Dragon 32/64".into(),
        );
        assert!(matches!(
            blocked,
            Err(PlatformWriteFailure::Referenced(references))
                if references.iter().any(|reference| reference.kind == lb_storage::PlatformReferenceKind::Game)
        ));

        delete_game_from_platform(
            directory.path().to_path_buf(),
            directory.path().to_path_buf(),
            created.source.clone(),
            added.game.id,
        )
        .unwrap_or_else(|error| {
            panic!(
                "delete temporary game failed: {}",
                describe_game_write_failure(&error)
            )
        });
        let deleted = match delete_platform_from_library(
            directory.path().to_path_buf(),
            created.source.clone(),
            "Dragon 32/64".into(),
        ) {
            Ok(deleted) => deleted,
            Err(_) => panic!("platform deletion failed"),
        };
        assert_eq!(deleted.folder_count, 51);
        assert!(!created.source.exists());
        assert_eq!(
            fs::read(&deleted.catalog_backup).expect("read pre-delete catalog backup"),
            created_catalog_bytes
        );
        let deleted_document =
            PlatformDocument::load_for_platform(&deleted.platform_backup, "Dragon 32/64")
                .expect("load deleted platform backup");
        assert!(deleted_document.library().games.is_empty());

        let final_catalog_bytes = fs::read(&catalog_path).expect("read final catalog");
        assert!(String::from_utf8_lossy(&final_catalog_bytes).contains("FutureCatalogRecord"));
        let final_catalog = AuxiliaryDocument::load(&catalog_path)
            .expect("load final catalog")
            .platform_catalog()
            .expect("parse final catalog");
        assert!(final_catalog
            .platforms
            .iter()
            .all(|platform| platform.metadata.name != "Dragon 32/64"));
        assert!(final_catalog
            .folders
            .iter()
            .all(|folder| folder.platform != "Dragon 32/64"));
        assert!(pending_transaction_manifests(directory.path())
            .expect("inspect pending transactions")
            .is_empty());
    }

    #[test]
    fn game_edit_payload_is_versioned_typed_and_canonicalized() {
        let valid = r#"{
            "version": 3,
            "metadata": {
                "title": "Fixture",
                "sort_title": "   ",
                "developer": "Fixture Labs",
                "max_players": 0
            },
            "launch_configuration": {
                "application_path": "Games/Fixture/game.rom",
                "command_line": "  ",
                "use_dos_box": false,
                "use_scumm_vm": false,
                "scumm_vm_aspect_correction": false,
                "scumm_vm_fullscreen": false
            },
            "alternate_names": [
                {
                    "source_index": 0,
                    "name": "Fixture Alias",
                    "region": "   "
                }
            ],
            "custom_fields": [
                {
                    "source_index": null,
                    "name": "Cabinet",
                    "value": ""
                }
            ],
            "favorite": true,
            "completed": false,
            "star_rating": 4
        }"#;
        let parsed = parse_game_edit_payload(valid).expect("valid editor payload");
        assert_eq!(parsed.metadata.title, "Fixture");
        assert_eq!(parsed.metadata.sort_title, None);
        assert_eq!(parsed.metadata.developer.as_deref(), Some("Fixture Labs"));
        assert_eq!(parsed.metadata.max_players, None);
        assert_eq!(parsed.launch_configuration.command_line, None);
        assert_eq!(parsed.alternate_names[0].source_index, Some(0));
        assert_eq!(parsed.alternate_names[0].region, None);
        assert_eq!(parsed.custom_fields[0].value, "");

        assert!(
            parse_game_edit_payload(&valid.replace("\"version\": 3", "\"version\": 4"))
                .unwrap_err()
                .contains("unsupported game editor payload version")
        );
        assert!(parse_game_edit_payload(
            &valid.replace("\"star_rating\": 4", "\"star_rating\": 6")
        )
        .unwrap_err()
        .contains("star rating"));
        assert!(parse_game_edit_payload(&valid.replace(
            "\"favorite\": true",
            "\"unknown\": true, \"favorite\": true"
        ))
        .unwrap_err()
        .contains("unknown field"));
    }

    #[test]
    fn additional_applications_are_indexed_by_game_and_priority() {
        let applications = [
            AdditionalApplication {
                id: "later".into(),
                game_id: "game".into(),
                priority: 20,
                ..AdditionalApplication::default()
            },
            AdditionalApplication {
                id: "other".into(),
                game_id: "other-game".into(),
                priority: 0,
                ..AdditionalApplication::default()
            },
            AdditionalApplication {
                id: "first".into(),
                game_id: "game".into(),
                priority: 10,
                ..AdditionalApplication::default()
            },
        ];
        let indexed = index_additional_applications(&applications);
        assert_eq!(indexed.len(), 2);
        assert_eq!(
            indexed["game"]
                .iter()
                .map(|application| application.id.as_str())
                .collect::<Vec<_>>(),
            ["first", "later"]
        );
    }

    #[test]
    fn path_mapping_rows_are_drives_then_unc_shares() {
        let mut mappings = HostPathMappings::default();
        mappings
            .set_windows_drive('D', "/mnt/windows")
            .expect("drive mapping");
        mappings
            .set_windows_unc("server", "roms", "/mnt/network")
            .expect("UNC mapping");
        assert!(matches!(
            path_mapping_key(&mappings, 0),
            Some(PathMappingKey::WindowsDrive('D'))
        ));
        assert!(matches!(
            path_mapping_key(&mappings, 1),
            Some(PathMappingKey::WindowsUnc { server, share })
                if server == "server" && share == "roms"
        ));
        assert!(path_mapping_key(&mappings, -1).is_none());
        assert!(path_mapping_key(&mappings, 2).is_none());
    }

    #[test]
    fn play_session_helpers_commit_launchbox_compatible_statistics() {
        let directory = tempfile::tempdir().expect("temporary library");
        let platform_directory = directory.path().join("Data/Platforms");
        fs::create_dir_all(&platform_directory).expect("create platform directory");
        let source = platform_directory.join("Fixture Console.xml");
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/launchbox/Data/Platforms/Fixture Console.xml");
        let original = fs::read(&fixture).expect("read platform fixture");
        fs::write(&source, &original).expect("write platform fixture");

        let started_at = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let timestamp = launchbox_local_timestamp(started_at);
        let parsed = DateTime::parse_from_rfc3339(&timestamp).expect("RFC 3339 timestamp");
        assert_eq!(parsed.timestamp(), 1_700_000_000);
        assert_eq!(
            timestamp
                .split_once('.')
                .and_then(|(_, suffix)| suffix.get(..7)),
            Some("0000000")
        );

        let started = write_play_session_start(
            directory.path().to_path_buf(),
            source.clone(),
            &LaunchTarget::MainGame,
            "fixture-racer",
            started_at,
        )
        .unwrap_or_else(|error| {
            panic!(
                "start statistics failed: {}",
                describe_game_write_failure(&error)
            )
        });
        let PlaySessionStatsRecord::Game(started_game) = started.record else {
            panic!("main-game session returned additional-application statistics");
        };
        assert_eq!(started_game.play_count, 9);
        assert_eq!(
            started_game.last_played_date.as_deref(),
            Some(timestamp.as_str())
        );
        assert_eq!(
            fs::read(&started.backup).expect("read first backup"),
            original
        );

        let timed = write_play_session_time(
            directory.path().to_path_buf(),
            source.clone(),
            &LaunchTarget::MainGame,
            "fixture-racer",
            Duration::from_secs(3),
        )
        .unwrap_or_else(|error| {
            panic!(
                "time statistics failed: {}",
                describe_game_write_failure(&error)
            )
        });
        let PlaySessionStatsRecord::Game(timed_game) = timed.record else {
            panic!("main-game duration returned additional-application statistics");
        };
        assert_eq!(timed_game.play_count, 9);
        assert_eq!(timed_game.play_time_seconds, 14_403);

        let persisted = PlatformDocument::load(&source).expect("reload written platform");
        let game = persisted
            .library()
            .games
            .iter()
            .find(|game| game.id == "fixture-racer")
            .expect("persisted fixture racer");
        assert_eq!(game.play_count, 9);
        assert_eq!(game.play_time_seconds, 14_403);
        assert_eq!(game.last_played_date.as_deref(), Some(timestamp.as_str()));
        let persisted_xml = fs::read_to_string(source).expect("read persisted XML");
        assert!(persisted_xml.contains("<FutureRootElement>preserve-me</FutureRootElement>"));
    }
}
