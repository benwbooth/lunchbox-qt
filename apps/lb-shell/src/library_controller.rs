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
        #[qproperty(bool, import_scanning)]
        #[qproperty(bool, writing)]
        #[qproperty(bool, launching)]
        #[qproperty(bool, launch_session_active)]
        #[qproperty(bool, last_launch_succeeded)]
        #[qproperty(bool, write_conflict)]
        #[qproperty(i32, game_count)]
        #[qproperty(i32, filtered_count)]
        #[qproperty(i32, platform_entry_count)]
        #[qproperty(i32, navigation_entry_count)]
        #[qproperty(i32, big_box_navigation_entry_count)]
        #[qproperty(QString, navigation_filter_kind)]
        #[qproperty(QString, navigation_filter_key)]
        #[qproperty(i32, platform_revision)]
        #[qproperty(i32, additional_application_revision)]
        #[qproperty(i32, pending_recovery_count)]
        #[qproperty(i32, delete_blocker_count)]
        #[qproperty(QString, delete_blocker_summary)]
        #[qproperty(QString, last_added_game_id)]
        #[qproperty(QString, last_added_additional_application_id)]
        #[qproperty(QString, import_preview_json)]
        #[qproperty(i32, last_import_count)]
        #[qproperty(i32, last_import_created_file_count)]
        #[qproperty(i32, last_import_moved_file_count)]
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
        fn local_path_from_url(self: &LibraryController, value: QString) -> QString;

        #[qinvokable]
        fn preview_rom_import(self: Pin<&mut LibraryController>, request_payload: QString);

        #[qinvokable]
        fn clear_rom_import_preview(self: Pin<&mut LibraryController>);

        #[qinvokable]
        fn import_roms(self: Pin<&mut LibraryController>, selection_payload: QString);

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
        fn apply_category_filter(
            self: Pin<&mut LibraryController>,
            search_text: QString,
            category: QString,
        );

        #[qinvokable]
        fn apply_playlist_filter(
            self: Pin<&mut LibraryController>,
            search_text: QString,
            playlist_id: QString,
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
        fn platform_edit_payload(self: &LibraryController, name: QString) -> QString;

        #[qinvokable]
        fn save_platform(
            self: Pin<&mut LibraryController>,
            original_name: QString,
            edit_payload: QString,
        );

        #[qinvokable]
        fn delete_platform(self: Pin<&mut LibraryController>, name: QString);

        #[qinvokable]
        fn new_category_edit_payload(self: &LibraryController) -> QString;

        #[qinvokable]
        fn category_edit_payload(self: &LibraryController, name: QString) -> QString;

        #[qinvokable]
        fn add_category(self: Pin<&mut LibraryController>, edit_payload: QString);

        #[qinvokable]
        fn save_category(
            self: Pin<&mut LibraryController>,
            original_name: QString,
            edit_payload: QString,
        );

        #[qinvokable]
        fn delete_category(self: Pin<&mut LibraryController>, name: QString);

        #[qinvokable]
        fn new_playlist_edit_payload(self: &LibraryController) -> QString;

        #[qinvokable]
        fn playlist_edit_payload(self: &LibraryController, playlist_id: QString) -> QString;

        #[qinvokable]
        fn add_playlist(self: Pin<&mut LibraryController>, edit_payload: QString);

        #[qinvokable]
        fn save_playlist(
            self: Pin<&mut LibraryController>,
            playlist_id: QString,
            edit_payload: QString,
        );

        #[qinvokable]
        fn delete_playlist(self: Pin<&mut LibraryController>, playlist_id: QString);

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
        fn new_additional_application_edit_payload(
            self: &LibraryController,
            row: i32,
            game_id: QString,
        ) -> QString;

        #[qinvokable]
        fn additional_application_edit_payload(
            self: &LibraryController,
            row: i32,
            game_id: QString,
            application_id: QString,
        ) -> QString;

        #[qinvokable]
        fn add_additional_application(
            self: Pin<&mut LibraryController>,
            row: i32,
            game_id: QString,
            edit_payload: QString,
        );

        #[qinvokable]
        fn save_additional_application(
            self: Pin<&mut LibraryController>,
            row: i32,
            game_id: QString,
            application_id: QString,
            edit_payload: QString,
        );

        #[qinvokable]
        fn delete_additional_application(
            self: Pin<&mut LibraryController>,
            row: i32,
            game_id: QString,
            application_id: QString,
        );

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
        fn report_additional_application_crud_smoke_success(
            self: &LibraryController,
            game_id: QString,
        ) -> bool;

        #[qinvokable]
        fn report_platform_crud_smoke_success(
            self: &LibraryController,
            platform_name: QString,
            blocked_references: i32,
        ) -> bool;

        #[qinvokable]
        fn report_category_crud_smoke_success(
            self: &LibraryController,
            category_name: QString,
            detached_children: i32,
        ) -> bool;

        #[qinvokable]
        fn report_playlist_crud_smoke_success(
            self: &LibraryController,
            playlist_id: QString,
            detached_children: i32,
            removed_cache_rows: i32,
        ) -> bool;

        #[qinvokable]
        fn report_big_box_navigation_smoke_success(self: &LibraryController) -> bool;

        #[qinvokable]
        fn report_import_smoke_success(
            self: &LibraryController,
            expected_count: i32,
            expected_created_files: i32,
            expected_moved_files: i32,
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

        #[qinvokable]
        fn navigation_entry_kind_at(self: &LibraryController, index: i32) -> QString;

        #[qinvokable]
        fn navigation_entry_key_at(self: &LibraryController, index: i32) -> QString;

        #[qinvokable]
        fn navigation_entry_name_at(self: &LibraryController, index: i32) -> QString;

        #[qinvokable]
        fn navigation_entry_depth_at(self: &LibraryController, index: i32) -> i32;

        #[qinvokable]
        fn navigation_entry_game_count_at(self: &LibraryController, index: i32) -> i32;

        #[qinvokable]
        fn big_box_navigation_entry_kind_at(self: &LibraryController, index: i32) -> QString;

        #[qinvokable]
        fn big_box_navigation_entry_key_at(self: &LibraryController, index: i32) -> QString;

        #[qinvokable]
        fn big_box_navigation_entry_name_at(self: &LibraryController, index: i32) -> QString;

        #[qinvokable]
        fn big_box_navigation_entry_depth_at(self: &LibraryController, index: i32) -> i32;

        #[qinvokable]
        fn big_box_navigation_entry_game_count_at(self: &LibraryController, index: i32) -> i32;
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
    QByteArray, QHash, QHashPair_i32_QByteArray, QList, QModelIndex, QString, QUrl, QVariant,
};
use lb_domain::{
    AdditionalApplication, AdditionalApplicationEdit, AlternateName, CustomField,
    EmulatorConfiguration, Game, GameLaunchConfiguration, GameMetadata, Mount, NavigationMetadata,
    ParentRelationship, PlatformCategory, PlatformDefinition, PlatformFolder, Playlist,
    PlaylistDocument, PlaylistFilter, PlaylistGame, UNASSIGNED_EMULATOR_ID,
};
use lb_import::{
    execute_manual_import, preview_manual_import, ImportError, ManualImportReport,
    ManualImportRequest, ManualImportSelection,
};
use lb_platform::{
    default_host_path_mappings_path, default_platform_folders, execute_launch_sequence,
    navigation_document_file_name, platform_document_file_name,
    prepare_game_launch_sequence_with_mounts_context_and_resolver,
    prepare_selected_additional_application_sequence_with_mounts_context_and_resolver,
    ArchiveExtractor, HostPathMappings, HostPathResolver, LaunchContext, LaunchKind,
    LaunchSequence, LaunchSequenceEvent, LaunchSequenceReport, LaunchTarget,
};
use lb_query::{filter_game_indices, GameFilter};
use lb_storage::{
    find_game_references, find_platform_references, pending_transaction_manifests,
    recover_pending_transactions, AuxiliaryDocument, GameReference, IndexedPlatformRecordEdit,
    LaunchBoxDataIndex, LibraryIndex, LibraryTransaction, NewGame, NewGameMetadata,
    PlatformDocument, PlatformReference, StorageError, TransactionError,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
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
    import_scanning: bool,
    writing: bool,
    launching: bool,
    launch_session_active: bool,
    last_launch_succeeded: bool,
    write_conflict: bool,
    game_count: i32,
    filtered_count: i32,
    platform_entry_count: i32,
    navigation_entry_count: i32,
    big_box_navigation_entry_count: i32,
    navigation_filter_kind: QString,
    navigation_filter_key: QString,
    platform_revision: i32,
    additional_application_revision: i32,
    pending_recovery_count: i32,
    delete_blocker_count: i32,
    delete_blocker_summary: QString,
    last_added_game_id: QString,
    last_added_additional_application_id: QString,
    import_preview_json: QString,
    last_import_count: i32,
    last_import_created_file_count: i32,
    last_import_moved_file_count: i32,
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
    navigation_catalog: NavigationCatalog,
    navigation_entries: Vec<NavigationEntry>,
    big_box_navigation_entries: Vec<NavigationEntry>,
    category_platforms: BTreeMap<String, BTreeSet<String>>,
    category_game_ids: BTreeMap<String, BTreeSet<String>>,
    playlist_game_ids: BTreeMap<String, BTreeSet<String>>,
    category_filter: Option<String>,
    playlist_filter: Option<String>,
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
    additional_application_write_notifications: u64,
    category_write_notifications: u64,
    last_category_detached_children: usize,
    playlist_write_notifications: u64,
    last_playlist_detached_children: usize,
    last_playlist_cache_rows_removed: usize,
    last_imported_game_ids: Vec<String>,
}

#[derive(Clone, Debug)]
struct PlatformCount {
    name: String,
    count: usize,
}

#[derive(Clone, Debug, Default)]
struct NavigationCatalog {
    platforms: Vec<PlatformDefinition>,
    categories: Vec<PlatformCategory>,
    parents: Vec<ParentRelationship>,
    playlists: Vec<PlaylistDocument>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum NavigationNodeKey {
    Category(String),
    Platform(String),
    Playlist(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NavigationEntry {
    kind: &'static str,
    key: String,
    name: String,
    depth: usize,
    game_count: usize,
    visible_in_big_box: bool,
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
    navigation_catalog: NavigationCatalog,
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
    navigation_catalog: NavigationCatalog,
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
                navigation_catalog: NavigationCatalog::default(),
                pending_recovery_count,
                launchbox_root: None,
                emulator_configuration: None,
            });
        }

        let data = LaunchBoxDataIndex::load(&path).map_err(|error| error.to_string())?;
        let emulator_configuration = data.emulator_configuration().cloned();
        let (platform_names, platform_sources) = platform_state_from_data(&data)?;
        let navigation_catalog = NavigationCatalog {
            platforms: data
                .platform_catalog()
                .map(|catalog| catalog.platforms.clone())
                .unwrap_or_default(),
            categories: data
                .platform_catalog()
                .map(|catalog| catalog.categories.clone())
                .unwrap_or_default(),
            parents: data.parents().to_vec(),
            playlists: data.playlists().to_vec(),
        };
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
            navigation_catalog,
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

struct RomImportSuccess {
    report: ManualImportReport,
    source: PathBuf,
}

struct GameDeleteSuccess {
    game: Game,
    source: PathBuf,
    backup: PathBuf,
}

struct AdditionalApplicationWriteSuccess {
    operation: AdditionalApplicationWriteOperation,
    application: AdditionalApplication,
    source: PathBuf,
    backup: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AdditionalApplicationWriteOperation {
    Create,
    Edit,
    Delete,
}

enum AdditionalApplicationWriteRequest {
    Create {
        id: String,
        edit: AdditionalApplicationEdit,
    },
    Edit {
        id: String,
        edit: AdditionalApplicationEdit,
    },
    Delete {
        id: String,
    },
}

struct PlatformCreateSuccess {
    name: String,
    platform: PlatformDefinition,
    source: PathBuf,
    catalog_backup: PathBuf,
    folder_count: usize,
}

struct PlatformEditSuccess {
    name: String,
    platform: PlatformDefinition,
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

struct CategoryWriteSuccess {
    name: String,
    categories: Vec<PlatformCategory>,
    parents: Vec<ParentRelationship>,
    catalog_backup: PathBuf,
    parents_backup: PathBuf,
    placement_count: usize,
    removed_placements: usize,
    detached_children: usize,
}

#[derive(Clone, Copy, Debug)]
enum CategoryWriteOperation {
    Create,
    Edit,
    Delete,
}

struct PlaylistWriteSuccess {
    id: String,
    playlists: Vec<PlaylistDocument>,
    parents: Vec<ParentRelationship>,
    source: PathBuf,
    playlist_backup: Option<PathBuf>,
    parents_backup: PathBuf,
    list_cache_backup: Option<PathBuf>,
    placement_count: usize,
    removed_placements: usize,
    detached_children: usize,
    removed_cache_rows: usize,
}

#[derive(Clone, Copy, Debug)]
enum PlaylistWriteOperation {
    Create,
    Edit,
    Delete,
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

#[derive(Debug)]
enum PlatformWriteFailure {
    Conflict(String),
    PendingRecovery { count: usize, message: String },
    Referenced(Vec<PlatformReference>),
    Other(String),
}

const GAME_EDIT_PAYLOAD_VERSION: u32 = 3;
const ADDITIONAL_APPLICATION_EDIT_PAYLOAD_VERSION: u32 = 1;
const PLATFORM_EDIT_PAYLOAD_VERSION: u32 = 1;
const CATEGORY_EDIT_PAYLOAD_VERSION: u32 = 1;
const PLAYLIST_EDIT_PAYLOAD_VERSION: u32 = 1;

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

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct AdditionalApplicationEditPayload {
    version: u32,
    application: AdditionalApplicationEdit,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct PlatformFolderEditPayload {
    source_index: Option<usize>,
    media_type: String,
    folder_path: String,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct PlatformEditPayload {
    version: u32,
    platform: PlatformDefinition,
    folders: Vec<PlatformFolderEditPayload>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct PlatformCategoryEditFields {
    name: String,
    nested_name: Option<String>,
    sort_title: Option<String>,
    notes: Option<String>,
    video_path: Option<String>,
    image_type: Option<String>,
    hide_in_big_box: bool,
}

impl From<&PlatformCategory> for PlatformCategoryEditFields {
    fn from(category: &PlatformCategory) -> Self {
        Self {
            name: category.metadata.name.clone(),
            nested_name: category.metadata.nested_name.clone(),
            sort_title: category.metadata.sort_title.clone(),
            notes: category.metadata.notes.clone(),
            video_path: category.metadata.video_path.clone(),
            image_type: category.metadata.image_type.clone(),
            hide_in_big_box: category.metadata.hide_in_big_box,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "snake_case")]
enum CategoryParentKind {
    Root,
    PlatformCategory,
    Platform,
    Playlist,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct CategoryParentEditPayload {
    source_index: Option<usize>,
    target_kind: CategoryParentKind,
    target_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct CategoryParentTargetPayload {
    target_kind: CategoryParentKind,
    target_key: String,
    label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct CategoryEditPayload {
    version: u32,
    category: PlatformCategoryEditFields,
    parents: Vec<CategoryParentEditPayload>,
    available_parent_targets: Vec<CategoryParentTargetPayload>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct PlaylistEditFields {
    id: String,
    name: String,
    nested_name: Option<String>,
    sort_title: Option<String>,
    notes: Option<String>,
    video_path: Option<String>,
    image_type: Option<String>,
    category: Option<String>,
    last_game_id: Option<String>,
    big_box_view: Option<String>,
    big_box_theme: Option<String>,
    hide_in_big_box: bool,
    include_with_platforms: bool,
    auto_populate: bool,
    is_autogenerated: bool,
    sort_by: Option<String>,
}

impl From<&Playlist> for PlaylistEditFields {
    fn from(playlist: &Playlist) -> Self {
        Self {
            id: playlist.id.clone(),
            name: playlist.metadata.name.clone(),
            nested_name: playlist.metadata.nested_name.clone(),
            sort_title: playlist.metadata.sort_title.clone(),
            notes: playlist.metadata.notes.clone(),
            video_path: playlist.metadata.video_path.clone(),
            image_type: playlist.metadata.image_type.clone(),
            category: playlist.metadata.category.clone(),
            last_game_id: playlist.metadata.last_game_id.clone(),
            big_box_view: playlist.metadata.big_box_view.clone(),
            big_box_theme: playlist.metadata.big_box_theme.clone(),
            hide_in_big_box: playlist.metadata.hide_in_big_box,
            include_with_platforms: playlist.include_with_platforms,
            auto_populate: playlist.auto_populate,
            is_autogenerated: playlist.is_autogenerated,
            sort_by: playlist.sort_by.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct PlaylistFilterEditPayload {
    source_index: Option<usize>,
    field_key: String,
    comparison_type_key: String,
    value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct PlaylistGameEditPayload {
    source_index: Option<usize>,
    game_id: String,
    game_title: String,
    game_platform: String,
    game_file_name: String,
    launchbox_db_id: Option<u64>,
    manual_order: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct PlaylistAvailableGamePayload {
    game_id: String,
    title: String,
    platform: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct PlaylistEditPayload {
    version: u32,
    playlist: PlaylistEditFields,
    filters: Vec<PlaylistFilterEditPayload>,
    games: Vec<PlaylistGameEditPayload>,
    parents: Vec<CategoryParentEditPayload>,
    available_parent_targets: Vec<CategoryParentTargetPayload>,
    available_games: Vec<PlaylistAvailableGamePayload>,
}

fn parse_additional_application_edit_payload(
    payload: &str,
) -> Result<AdditionalApplicationEditPayload, String> {
    let mut payload: AdditionalApplicationEditPayload = serde_json::from_str(payload)
        .map_err(|error| format!("invalid additional-application editor payload: {error}"))?;
    if payload.version != ADDITIONAL_APPLICATION_EDIT_PAYLOAD_VERSION {
        return Err(format!(
            "unsupported additional-application editor payload version {}; expected {}",
            payload.version, ADDITIONAL_APPLICATION_EDIT_PAYLOAD_VERSION
        ));
    }
    if payload.application.name.trim().is_empty() {
        return Err("an application name cannot be empty".into());
    }
    if payload.application.priority < 0 {
        return Err("application priority cannot be negative".into());
    }
    payload.application.command_line = canonical_optional_text(payload.application.command_line);
    payload.application.emulator_id = canonical_optional_text(payload.application.emulator_id);
    payload.application.developer = canonical_optional_text(payload.application.developer);
    payload.application.publisher = canonical_optional_text(payload.application.publisher);
    payload.application.region = canonical_optional_text(payload.application.region);
    payload.application.release_date = canonical_optional_text(payload.application.release_date);
    payload.application.version = canonical_optional_text(payload.application.version);
    payload.application.status = canonical_optional_text(payload.application.status);
    payload.application.last_played = canonical_optional_text(payload.application.last_played);
    if !payload.application.use_emulator {
        payload.application.emulator_id = None;
    }
    Ok(payload)
}

fn canonicalize_additional_application_emulator(
    edit: &mut AdditionalApplicationEdit,
    configuration: Option<&EmulatorConfiguration>,
    existing_emulator_id: Option<&str>,
) -> Result<(), String> {
    if !edit.use_emulator {
        edit.emulator_id = None;
        return Ok(());
    }
    let Some(selected_id) = edit.emulator_id.as_deref() else {
        return Ok(());
    };
    if selected_id.eq_ignore_ascii_case(UNASSIGNED_EMULATOR_ID) {
        edit.use_emulator = false;
        edit.emulator_id = None;
        return Ok(());
    }
    if let Some(emulator) = configuration
        .into_iter()
        .flat_map(|configuration| &configuration.emulators)
        .find(|emulator| emulator.id.eq_ignore_ascii_case(selected_id))
    {
        edit.emulator_id = Some(emulator.id.clone());
        return Ok(());
    }
    if existing_emulator_id.is_some_and(|existing| existing.eq_ignore_ascii_case(selected_id)) {
        return Ok(());
    }
    Err(format!(
        "selected emulator {selected_id} is not present in the loaded Emulators.xml"
    ))
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

fn parse_platform_edit_payload(
    original_name: &str,
    payload: &str,
) -> Result<PlatformEditPayload, String> {
    let mut payload: PlatformEditPayload = serde_json::from_str(payload)
        .map_err(|error| format!("invalid platform editor payload: {error}"))?;
    if payload.version != PLATFORM_EDIT_PAYLOAD_VERSION {
        return Err(format!(
            "unsupported platform editor payload version {}; expected {}",
            payload.version, PLATFORM_EDIT_PAYLOAD_VERSION
        ));
    }
    if payload.platform.metadata.name != original_name {
        return Err(format!(
            "platform identity cannot be changed from {original_name} to {} without verified rename semantics",
            payload.platform.metadata.name
        ));
    }
    canonicalize_platform_definition(&mut payload.platform);
    payload
        .platform
        .validate()
        .map_err(|error| error.to_string())?;
    for folder in &payload.folders {
        PlatformFolder {
            platform: original_name.to_string(),
            media_type: folder.media_type.clone(),
            folder_path: folder.folder_path.clone(),
        }
        .validate()
        .map_err(|error| error.to_string())?;
    }
    Ok(payload)
}

fn parse_category_edit_payload(
    original_name: Option<&str>,
    payload: &str,
) -> Result<CategoryEditPayload, String> {
    let mut payload: CategoryEditPayload = serde_json::from_str(payload)
        .map_err(|error| format!("invalid platform category editor payload: {error}"))?;
    if payload.version != CATEGORY_EDIT_PAYLOAD_VERSION {
        return Err(format!(
            "unsupported platform category editor payload version {}; expected {}",
            payload.version, CATEGORY_EDIT_PAYLOAD_VERSION
        ));
    }
    if let Some(original_name) = original_name {
        if payload.category.name != original_name {
            return Err(format!(
                "platform category identity cannot be changed from {original_name} to {} without verified rename semantics",
                payload.category.name
            ));
        }
    } else {
        payload.category.name = payload.category.name.trim().to_string();
    }
    if payload.category.name.is_empty() {
        return Err("a platform category name is required".into());
    }
    payload.category.nested_name = canonical_optional_text(payload.category.nested_name);
    payload.category.sort_title = canonical_optional_text(payload.category.sort_title);
    payload.category.notes = canonical_optional_text(payload.category.notes);
    payload.category.video_path = canonical_optional_text(payload.category.video_path);
    payload.category.image_type = canonical_optional_text(payload.category.image_type);
    if payload.parents.is_empty() {
        return Err("a platform category must have at least one hierarchy placement".into());
    }
    let mut targets = BTreeSet::new();
    let mut previous = None;
    let mut saw_new = false;
    for parent in &mut payload.parents {
        parent.target_key = parent.target_key.trim().to_string();
        if parent.target_kind == CategoryParentKind::Root {
            if !parent.target_key.is_empty() {
                return Err("the root parent placement cannot have a target key".into());
            }
        } else if parent.target_key.is_empty() {
            return Err("a non-root parent placement requires a target key".into());
        }
        match parent.source_index {
            None => saw_new = true,
            Some(index) => {
                if saw_new {
                    return Err("new parent placements must follow retained source rows".into());
                }
                if previous.is_some_and(|previous| previous >= index) {
                    return Err(
                        "parent source indices must be unique and remain in source order".into(),
                    );
                }
                previous = Some(index);
            }
        }
        let target = (parent.target_kind, parent.target_key.to_lowercase());
        if !targets.insert(target) {
            return Err("a parent placement cannot occur more than once".into());
        }
    }
    Ok(payload)
}

fn parse_playlist_edit_payload(
    original: Option<&Playlist>,
    payload: &str,
) -> Result<PlaylistEditPayload, String> {
    let mut payload: PlaylistEditPayload = serde_json::from_str(payload)
        .map_err(|error| format!("invalid playlist editor payload: {error}"))?;
    if payload.version != PLAYLIST_EDIT_PAYLOAD_VERSION {
        return Err(format!(
            "unsupported playlist editor payload version {}; expected {}",
            payload.version, PLAYLIST_EDIT_PAYLOAD_VERSION
        ));
    }
    payload.playlist.id = payload.playlist.id.trim().to_string();
    payload.playlist.name = payload.playlist.name.trim().to_string();
    if payload.playlist.id.is_empty() {
        return Err("a playlist ID is required".into());
    }
    if payload.playlist.name.is_empty() {
        return Err("a playlist unique name is required".into());
    }
    if let Some(original) = original {
        if payload.playlist.id != original.id {
            return Err(format!(
                "playlist identity cannot be changed from {} to {}",
                original.id, payload.playlist.id
            ));
        }
        if payload.playlist.name != original.metadata.name {
            return Err(format!(
                "playlist unique name cannot be changed from {} to {} in the recovered 13.27 contract",
                original.metadata.name, payload.playlist.name
            ));
        }
    }
    macro_rules! canonicalize_playlist_field {
        ($($field:ident),+ $(,)?) => {
            $(payload.playlist.$field =
                canonical_optional_text(payload.playlist.$field.take());)+
        };
    }
    canonicalize_playlist_field!(
        nested_name,
        sort_title,
        notes,
        video_path,
        image_type,
        category,
        last_game_id,
        big_box_view,
        big_box_theme,
        sort_by,
    );
    validate_playlist_indexed_payload_rows(
        "filter",
        payload.filters.iter().map(|row| row.source_index),
    )?;
    for filter in &mut payload.filters {
        filter.field_key = filter.field_key.trim().to_string();
        filter.comparison_type_key = filter.comparison_type_key.trim().to_string();
        if filter.field_key.is_empty() || filter.comparison_type_key.is_empty() {
            return Err("playlist filters require a field and comparison".into());
        }
    }
    validate_playlist_indexed_payload_rows(
        "game",
        payload.games.iter().map(|row| row.source_index),
    )?;
    let mut game_ids = BTreeSet::new();
    for game in &mut payload.games {
        game.game_id = game.game_id.trim().to_string();
        if game.game_id.is_empty() {
            return Err("playlist game rows require a game ID".into());
        }
        if !game_ids.insert(game.game_id.to_lowercase()) {
            return Err(format!(
                "game {} cannot occur more than once in a playlist",
                game.game_id
            ));
        }
    }
    if payload.parents.is_empty() {
        return Err("a playlist must have at least one hierarchy placement".into());
    }
    let mut targets = BTreeSet::new();
    let mut previous = None;
    let mut saw_new = false;
    for parent in &mut payload.parents {
        parent.target_key = parent.target_key.trim().to_string();
        if parent.target_kind == CategoryParentKind::Root {
            if !parent.target_key.is_empty() {
                return Err("the root parent placement cannot have a target key".into());
            }
        } else if parent.target_key.is_empty() {
            return Err("a non-root parent placement requires a target key".into());
        }
        match parent.source_index {
            None => saw_new = true,
            Some(index) => {
                if saw_new {
                    return Err("new parent placements must follow retained source rows".into());
                }
                if previous.is_some_and(|previous| previous >= index) {
                    return Err(
                        "parent source indices must be unique and remain in source order".into(),
                    );
                }
                previous = Some(index);
            }
        }
        if !targets.insert((parent.target_kind, parent.target_key.to_lowercase())) {
            return Err("a parent placement cannot occur more than once".into());
        }
    }
    Ok(payload)
}

fn validate_playlist_indexed_payload_rows(
    record: &str,
    indices: impl IntoIterator<Item = Option<usize>>,
) -> Result<(), String> {
    let mut previous = None;
    let mut saw_new = false;
    for source_index in indices {
        match source_index {
            None => saw_new = true,
            Some(index) => {
                if saw_new {
                    return Err(format!(
                        "new playlist {record} rows must follow retained source rows"
                    ));
                }
                if previous.is_some_and(|previous| previous >= index) {
                    return Err(format!(
                        "playlist {record} source indices must be unique and remain in source order"
                    ));
                }
                previous = Some(index);
            }
        }
    }
    Ok(())
}

fn playlist_edit_fields_to_domain(
    fields: &PlaylistEditFields,
    original: Option<&Playlist>,
) -> Playlist {
    let mut playlist = original.cloned().unwrap_or_default();
    playlist.id = fields.id.clone();
    playlist.metadata.name = fields.name.clone();
    playlist.metadata.nested_name = fields.nested_name.clone();
    playlist.metadata.sort_title = fields.sort_title.clone();
    playlist.metadata.notes = fields.notes.clone();
    playlist.metadata.video_path = fields.video_path.clone();
    playlist.metadata.image_type = fields.image_type.clone();
    playlist.metadata.category = fields.category.clone();
    playlist.metadata.last_game_id = fields.last_game_id.clone();
    playlist.metadata.big_box_view = fields.big_box_view.clone();
    playlist.metadata.big_box_theme = fields.big_box_theme.clone();
    playlist.metadata.hide_in_big_box = fields.hide_in_big_box;
    playlist.include_with_platforms = fields.include_with_platforms;
    playlist.auto_populate = fields.auto_populate;
    playlist.is_autogenerated = fields.is_autogenerated;
    playlist.sort_by = fields.sort_by.clone();
    playlist
}

fn playlist_filter_edits(
    filters: &[PlaylistFilterEditPayload],
) -> Vec<IndexedPlatformRecordEdit<PlaylistFilter>> {
    filters
        .iter()
        .map(|filter| IndexedPlatformRecordEdit {
            source_index: filter.source_index,
            record: PlaylistFilter {
                field_key: filter.field_key.clone(),
                comparison_type_key: filter.comparison_type_key.clone(),
                value: filter.value.clone(),
            },
        })
        .collect()
}

fn playlist_game_edits(
    games: &[PlaylistGameEditPayload],
) -> Vec<IndexedPlatformRecordEdit<PlaylistGame>> {
    games
        .iter()
        .map(|game| IndexedPlatformRecordEdit {
            source_index: game.source_index,
            record: PlaylistGame {
                game_id: game.game_id.clone(),
                game_title: game.game_title.clone(),
                game_platform: game.game_platform.clone(),
                game_file_name: game.game_file_name.clone(),
                launchbox_db_id: game.launchbox_db_id,
                manual_order: game.manual_order,
            },
        })
        .collect()
}

fn playlist_parent_relationships(
    playlist_id: &str,
    parents: &[CategoryParentEditPayload],
) -> Vec<IndexedPlatformRecordEdit<ParentRelationship>> {
    parents
        .iter()
        .map(|parent| {
            let mut relationship = ParentRelationship {
                playlist_id: Some(playlist_id.to_string()),
                ..ParentRelationship::default()
            };
            match parent.target_kind {
                CategoryParentKind::Root => {}
                CategoryParentKind::PlatformCategory => {
                    relationship.parent_platform_category_name = Some(parent.target_key.clone());
                }
                CategoryParentKind::Platform => {
                    relationship.parent_platform_name = Some(parent.target_key.clone());
                }
                CategoryParentKind::Playlist => {
                    relationship.parent_playlist_id = Some(parent.target_key.clone());
                }
            }
            IndexedPlatformRecordEdit {
                source_index: parent.source_index,
                record: relationship,
            }
        })
        .collect()
}

fn available_playlist_games(games: &[Game]) -> Vec<PlaylistAvailableGamePayload> {
    let mut available = games
        .iter()
        .map(|game| PlaylistAvailableGamePayload {
            game_id: game.id.clone(),
            title: game.title.clone(),
            platform: game.platform.clone(),
        })
        .collect::<Vec<_>>();
    available.sort_by(|left, right| {
        left.title
            .to_lowercase()
            .cmp(&right.title.to_lowercase())
            .then_with(|| {
                left.platform
                    .to_lowercase()
                    .cmp(&right.platform.to_lowercase())
            })
            .then_with(|| left.game_id.cmp(&right.game_id))
    });
    available
}

fn category_edit_fields_to_domain(
    fields: &PlatformCategoryEditFields,
    original: Option<&PlatformCategory>,
) -> PlatformCategory {
    let mut category = original.cloned().unwrap_or_default();
    category.metadata.name = fields.name.clone();
    category.metadata.nested_name = fields.nested_name.clone();
    category.metadata.sort_title = fields.sort_title.clone();
    category.metadata.notes = fields.notes.clone();
    category.metadata.video_path = fields.video_path.clone();
    category.metadata.image_type = fields.image_type.clone();
    category.metadata.hide_in_big_box = fields.hide_in_big_box;
    category
}

fn category_parent_relationships(
    category_name: &str,
    parents: &[CategoryParentEditPayload],
) -> Vec<IndexedPlatformRecordEdit<ParentRelationship>> {
    parents
        .iter()
        .map(|parent| {
            let mut relationship = ParentRelationship {
                platform_category_name: Some(category_name.to_string()),
                ..ParentRelationship::default()
            };
            match parent.target_kind {
                CategoryParentKind::Root => {}
                CategoryParentKind::PlatformCategory => {
                    relationship.parent_platform_category_name = Some(parent.target_key.clone());
                }
                CategoryParentKind::Platform => {
                    relationship.parent_platform_name = Some(parent.target_key.clone());
                }
                CategoryParentKind::Playlist => {
                    relationship.parent_playlist_id = Some(parent.target_key.clone());
                }
            }
            IndexedPlatformRecordEdit {
                source_index: parent.source_index,
                record: relationship,
            }
        })
        .collect()
}

fn category_parent_payload(
    source_index: usize,
    relationship: &ParentRelationship,
) -> CategoryParentEditPayload {
    if let Some(name) = relationship
        .parent_platform_category_name
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        CategoryParentEditPayload {
            source_index: Some(source_index),
            target_kind: CategoryParentKind::PlatformCategory,
            target_key: name.to_string(),
        }
    } else if let Some(name) = relationship
        .parent_platform_name
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        CategoryParentEditPayload {
            source_index: Some(source_index),
            target_kind: CategoryParentKind::Platform,
            target_key: name.to_string(),
        }
    } else if let Some(id) = relationship
        .parent_playlist_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        CategoryParentEditPayload {
            source_index: Some(source_index),
            target_kind: CategoryParentKind::Playlist,
            target_key: id.to_string(),
        }
    } else {
        CategoryParentEditPayload {
            source_index: Some(source_index),
            target_kind: CategoryParentKind::Root,
            target_key: String::new(),
        }
    }
}

fn category_parent_targets(
    catalog: &NavigationCatalog,
    platform_names: &[String],
    edited_category: Option<&str>,
) -> Vec<CategoryParentTargetPayload> {
    let mut targets = vec![CategoryParentTargetPayload {
        target_kind: CategoryParentKind::Root,
        target_key: String::new(),
        label: "Root".into(),
    }];
    targets.extend(
        catalog
            .categories
            .iter()
            .filter(|category| {
                edited_category
                    .is_none_or(|edited| !category.metadata.name.eq_ignore_ascii_case(edited))
            })
            .map(|category| CategoryParentTargetPayload {
                target_kind: CategoryParentKind::PlatformCategory,
                target_key: category.metadata.name.clone(),
                label: format!("Category — {}", category.metadata.name),
            }),
    );
    targets.extend(
        platform_names
            .iter()
            .map(|platform| CategoryParentTargetPayload {
                target_kind: CategoryParentKind::Platform,
                target_key: platform.clone(),
                label: format!("Platform — {platform}"),
            }),
    );
    targets.extend(
        catalog
            .playlists
            .iter()
            .map(|playlist| CategoryParentTargetPayload {
                target_kind: CategoryParentKind::Playlist,
                target_key: playlist.playlist.id.clone(),
                label: format!("Playlist — {}", playlist.playlist.metadata.name),
            }),
    );
    targets[1..].sort_by(|left, right| {
        left.label
            .to_lowercase()
            .cmp(&right.label.to_lowercase())
            .then_with(|| left.target_key.cmp(&right.target_key))
    });
    targets
}

fn playlist_parent_targets(
    catalog: &NavigationCatalog,
    platform_names: &[String],
    edited_playlist_id: Option<&str>,
) -> Vec<CategoryParentTargetPayload> {
    let mut targets = vec![CategoryParentTargetPayload {
        target_kind: CategoryParentKind::Root,
        target_key: String::new(),
        label: "Root".into(),
    }];
    targets.extend(
        catalog
            .categories
            .iter()
            .map(|category| CategoryParentTargetPayload {
                target_kind: CategoryParentKind::PlatformCategory,
                target_key: category.metadata.name.clone(),
                label: format!("Category — {}", category.metadata.name),
            }),
    );
    targets.extend(
        platform_names
            .iter()
            .map(|platform| CategoryParentTargetPayload {
                target_kind: CategoryParentKind::Platform,
                target_key: platform.clone(),
                label: format!("Platform — {platform}"),
            }),
    );
    targets.extend(
        catalog
            .playlists
            .iter()
            .filter(|document| {
                edited_playlist_id
                    .is_none_or(|edited| !document.playlist.id.eq_ignore_ascii_case(edited))
            })
            .map(|document| CategoryParentTargetPayload {
                target_kind: CategoryParentKind::Playlist,
                target_key: document.playlist.id.clone(),
                label: format!("Playlist — {}", document.playlist.metadata.name),
            }),
    );
    targets[1..].sort_by(|left, right| {
        left.label
            .to_lowercase()
            .cmp(&right.label.to_lowercase())
            .then_with(|| left.target_key.cmp(&right.target_key))
    });
    targets
}

fn validate_category_hierarchy_edit(
    catalog: &NavigationCatalog,
    platform_names: &[String],
    payload: &CategoryEditPayload,
    creating: bool,
) -> Result<(), String> {
    let category_name = &payload.category.name;
    let existing = catalog
        .categories
        .iter()
        .find(|category| category.metadata.name.eq_ignore_ascii_case(category_name));
    if creating && existing.is_some() {
        return Err(format!("platform category already exists: {category_name}"));
    }
    if !creating && existing.is_none() {
        return Err(format!("platform category was not found: {category_name}"));
    }
    let category_names = catalog
        .categories
        .iter()
        .map(|category| category.metadata.name.as_str())
        .collect::<Vec<_>>();
    let playlist_ids = catalog
        .playlists
        .iter()
        .map(|playlist| playlist.playlist.id.as_str())
        .collect::<Vec<_>>();
    for parent in &payload.parents {
        let exists = match parent.target_kind {
            CategoryParentKind::Root => true,
            CategoryParentKind::PlatformCategory => category_names
                .iter()
                .any(|name| name.eq_ignore_ascii_case(&parent.target_key)),
            CategoryParentKind::Platform => platform_names
                .iter()
                .any(|name| name.eq_ignore_ascii_case(&parent.target_key)),
            CategoryParentKind::Playlist => playlist_ids
                .iter()
                .any(|id| id.eq_ignore_ascii_case(&parent.target_key)),
        };
        if !exists {
            return Err(format!(
                "parent target is no longer available: {:?} {}",
                parent.target_kind, parent.target_key
            ));
        }
    }

    let category_key = NavigationNodeKey::Category(category_name.to_lowercase());
    let mut parent_graph = BTreeMap::<NavigationNodeKey, BTreeSet<NavigationNodeKey>>::new();
    for relationship in &catalog.parents {
        let Some(child) = relationship_child_key(relationship) else {
            continue;
        };
        if child == category_key {
            continue;
        }
        if let Some(parent) = relationship_parent_key(relationship) {
            parent_graph.entry(child).or_default().insert(parent);
        }
    }
    for edit in category_parent_relationships(category_name, &payload.parents) {
        if let Some(parent) = relationship_parent_key(&edit.record) {
            parent_graph
                .entry(category_key.clone())
                .or_default()
                .insert(parent);
        }
    }
    let mut path = BTreeSet::new();
    if hierarchy_reaches(&category_key, &category_key, &parent_graph, &mut path, true) {
        return Err("the selected parent placements would create a hierarchy cycle".into());
    }
    Ok(())
}

fn validate_playlist_hierarchy_edit(
    catalog: &NavigationCatalog,
    platform_names: &[String],
    payload: &PlaylistEditPayload,
    creating: bool,
) -> Result<(), String> {
    let playlist_id = &payload.playlist.id;
    let existing = catalog
        .playlists
        .iter()
        .find(|document| document.playlist.id.eq_ignore_ascii_case(playlist_id));
    if creating && existing.is_some() {
        return Err(format!("playlist ID already exists: {playlist_id}"));
    }
    if !creating && existing.is_none() {
        return Err(format!("playlist was not found: {playlist_id}"));
    }
    if catalog.playlists.iter().any(|document| {
        !document.playlist.id.eq_ignore_ascii_case(playlist_id)
            && document
                .playlist
                .metadata
                .name
                .eq_ignore_ascii_case(&payload.playlist.name)
    }) {
        return Err(format!(
            "playlist unique name already exists: {}",
            payload.playlist.name
        ));
    }
    for parent in &payload.parents {
        let exists = match parent.target_kind {
            CategoryParentKind::Root => true,
            CategoryParentKind::PlatformCategory => catalog.categories.iter().any(|category| {
                category
                    .metadata
                    .name
                    .eq_ignore_ascii_case(&parent.target_key)
            }),
            CategoryParentKind::Platform => platform_names
                .iter()
                .any(|name| name.eq_ignore_ascii_case(&parent.target_key)),
            CategoryParentKind::Playlist => catalog.playlists.iter().any(|document| {
                document
                    .playlist
                    .id
                    .eq_ignore_ascii_case(&parent.target_key)
            }),
        };
        if !exists {
            return Err(format!(
                "parent target is no longer available: {:?} {}",
                parent.target_kind, parent.target_key
            ));
        }
    }
    let playlist_key = NavigationNodeKey::Playlist(playlist_id.to_lowercase());
    let mut parent_graph = BTreeMap::<NavigationNodeKey, BTreeSet<NavigationNodeKey>>::new();
    for relationship in &catalog.parents {
        let Some(child) = relationship_child_key(relationship) else {
            continue;
        };
        if child == playlist_key {
            continue;
        }
        if let Some(parent) = relationship_parent_key(relationship) {
            parent_graph.entry(child).or_default().insert(parent);
        }
    }
    for edit in playlist_parent_relationships(playlist_id, &payload.parents) {
        if let Some(parent) = relationship_parent_key(&edit.record) {
            parent_graph
                .entry(playlist_key.clone())
                .or_default()
                .insert(parent);
        }
    }
    let mut path = BTreeSet::new();
    if hierarchy_reaches(&playlist_key, &playlist_key, &parent_graph, &mut path, true) {
        return Err("the selected parent placements would create a hierarchy cycle".into());
    }
    Ok(())
}

fn canonicalize_playlist_games(
    payload: &mut PlaylistEditPayload,
    games: &[Game],
) -> Result<(), String> {
    let games_by_id = games
        .iter()
        .map(|game| (game.id.to_lowercase(), game))
        .collect::<BTreeMap<_, _>>();
    for edit in &mut payload.games {
        let Some(game) = games_by_id.get(&edit.game_id.to_lowercase()) else {
            return Err(format!(
                "playlist game is no longer available: {}",
                edit.game_id
            ));
        };
        edit.game_id = game.id.clone();
        edit.game_title = game.title.clone();
        edit.game_platform = game.platform.clone();
        edit.game_file_name = lexical_file_name(&game.application_path).to_string();
        edit.launchbox_db_id = game.database_id.map(u64::from);
    }
    Ok(())
}

fn lexical_file_name(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

fn hierarchy_reaches(
    node: &NavigationNodeKey,
    target: &NavigationNodeKey,
    parent_graph: &BTreeMap<NavigationNodeKey, BTreeSet<NavigationNodeKey>>,
    path: &mut BTreeSet<NavigationNodeKey>,
    initial: bool,
) -> bool {
    if !initial && node == target {
        return true;
    }
    if !path.insert(node.clone()) {
        return false;
    }
    let reaches = parent_graph.get(node).is_some_and(|parents| {
        parents
            .iter()
            .any(|parent| hierarchy_reaches(parent, target, parent_graph, path, false))
    });
    path.remove(node);
    reaches
}

fn canonicalize_platform_definition(platform: &mut PlatformDefinition) {
    macro_rules! canonicalize {
        ($($field:ident),+ $(,)?) => {
            $(platform.metadata.$field =
                canonical_optional_text(platform.metadata.$field.take());)+
        };
    }
    canonicalize!(
        nested_name,
        sort_title,
        notes,
        folder,
        category,
        image_type,
        scrape_as,
        last_game_id,
        last_selected_child,
        cpu,
        developer,
        display,
        graphics,
        manufacturer,
        max_controllers,
        media,
        memory,
        sound,
        android_theme_video_path,
        back_images_folder,
        banner_images_folder,
        big_box_theme,
        big_box_view,
        clear_logo_images_folder,
        fanart_images_folder,
        front_images_folder,
        manuals_folder,
        music_folder,
        screenshot_images_folder,
        steam_banner_images_folder,
        video_path,
        videos_folder,
    );
    platform.release_date = canonical_optional_text(platform.release_date.take());
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

fn describe_platform_write_failure(error: &PlatformWriteFailure) -> String {
    match error {
        PlatformWriteFailure::Conflict(message) => format!("write conflict: {message}"),
        PlatformWriteFailure::PendingRecovery { message, .. } => {
            format!("interrupted transaction requires recovery: {message}")
        }
        PlatformWriteFailure::Referenced(references) => format!(
            "{} unexpected dependent records prevented the update",
            references.len()
        ),
        PlatformWriteFailure::Other(message) => message.clone(),
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

fn write_additional_application(
    root: PathBuf,
    source: PathBuf,
    game_id: String,
    request: AdditionalApplicationWriteRequest,
) -> Result<AdditionalApplicationWriteSuccess, GameWriteFailure> {
    let mut document = PlatformDocument::load(&source)
        .map_err(|error| GameWriteFailure::Other(error.to_string()))?;
    if !document
        .library()
        .games
        .iter()
        .any(|game| game.id == game_id)
    {
        return Err(GameWriteFailure::Other(format!(
            "game {game_id} disappeared before the additional-application edit"
        )));
    }

    let (operation, application) = match request {
        AdditionalApplicationWriteRequest::Create { id, edit } => {
            let base = AdditionalApplication {
                id,
                game_id: game_id.clone(),
                ..AdditionalApplication::default()
            };
            let application = document
                .add_additional_application(edit.apply_to(&base))
                .map_err(|error| GameWriteFailure::Other(error.to_string()))?;
            (AdditionalApplicationWriteOperation::Create, application)
        }
        AdditionalApplicationWriteRequest::Edit { id, edit } => {
            let owner = document
                .library()
                .additional_applications
                .iter()
                .find(|application| application.id == id)
                .map(|application| application.game_id.as_str())
                .ok_or_else(|| {
                    GameWriteFailure::Other(format!(
                        "additional application {id} disappeared before the edit"
                    ))
                })?;
            if owner != game_id {
                return Err(GameWriteFailure::Other(format!(
                    "additional application {id} belongs to game {owner}, not {game_id}"
                )));
            }
            let application = document
                .set_additional_application(&id, edit)
                .map_err(|error| GameWriteFailure::Other(error.to_string()))?;
            (AdditionalApplicationWriteOperation::Edit, application)
        }
        AdditionalApplicationWriteRequest::Delete { id } => {
            let owner = document
                .library()
                .additional_applications
                .iter()
                .find(|application| application.id == id)
                .map(|application| application.game_id.as_str())
                .ok_or_else(|| {
                    GameWriteFailure::Other(format!(
                        "additional application {id} disappeared before deletion"
                    ))
                })?;
            if owner != game_id {
                return Err(GameWriteFailure::Other(format!(
                    "additional application {id} belongs to game {owner}, not {game_id}"
                )));
            }
            let application = document
                .remove_additional_application(&id)
                .map_err(|error| GameWriteFailure::Other(error.to_string()))?;
            (AdditionalApplicationWriteOperation::Delete, application)
        }
    };

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
    Ok(AdditionalApplicationWriteSuccess {
        operation,
        application,
        source,
        backup,
    })
}

fn platform_catalog_path(root: &Path) -> Result<PathBuf, PlatformWriteFailure> {
    [root.join("Data/Platforms.xml"), root.join("Platforms.xml")]
        .into_iter()
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| {
            PlatformWriteFailure::Other(format!(
                "could not find a writable Platforms.xml under {}",
                root.display()
            ))
        })
}

fn parents_document_path(root: &Path) -> Result<PathBuf, PlatformWriteFailure> {
    [root.join("Data/Parents.xml"), root.join("Parents.xml")]
        .into_iter()
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| {
            PlatformWriteFailure::Other(format!(
                "could not find a writable Parents.xml under {}",
                root.display()
            ))
        })
}

fn new_category_payload(
    catalog: &NavigationCatalog,
    platform_names: &[String],
) -> Result<String, PlatformWriteFailure> {
    serde_json::to_string(&CategoryEditPayload {
        version: CATEGORY_EDIT_PAYLOAD_VERSION,
        category: PlatformCategoryEditFields {
            name: String::new(),
            nested_name: None,
            sort_title: None,
            notes: None,
            video_path: None,
            image_type: None,
            hide_in_big_box: false,
        },
        parents: vec![CategoryParentEditPayload {
            source_index: None,
            target_kind: CategoryParentKind::Root,
            target_key: String::new(),
        }],
        available_parent_targets: category_parent_targets(catalog, platform_names, None),
    })
    .map_err(|error| PlatformWriteFailure::Other(error.to_string()))
}

fn load_category_edit_payload(
    root: &Path,
    name: &str,
    base_catalog: &NavigationCatalog,
    platform_names: &[String],
) -> Result<String, PlatformWriteFailure> {
    let catalog_document = AuxiliaryDocument::load(platform_catalog_path(root)?)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let parents_document = AuxiliaryDocument::load(parents_document_path(root)?)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let platform_catalog = catalog_document
        .platform_catalog()
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let category = platform_catalog
        .categories
        .iter()
        .find(|category| category.metadata.name.eq_ignore_ascii_case(name))
        .cloned()
        .ok_or_else(|| {
            PlatformWriteFailure::Other(format!("platform category was not found: {name}"))
        })?;
    let exact_name = category.metadata.name.clone();
    let mut parents = parents_document
        .parent_relationships()
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?
        .into_iter()
        .filter(|relationship| {
            relationship
                .platform_category_name
                .as_deref()
                .is_some_and(|name| name.eq_ignore_ascii_case(&exact_name))
        })
        .enumerate()
        .map(|(source_index, relationship)| category_parent_payload(source_index, &relationship))
        .collect::<Vec<_>>();
    if parents.is_empty() {
        parents.push(CategoryParentEditPayload {
            source_index: None,
            target_kind: CategoryParentKind::Root,
            target_key: String::new(),
        });
    }
    let mut current_catalog = base_catalog.clone();
    current_catalog.categories = platform_catalog.categories;
    current_catalog.parents = parents_document
        .parent_relationships()
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    serde_json::to_string(&CategoryEditPayload {
        version: CATEGORY_EDIT_PAYLOAD_VERSION,
        category: PlatformCategoryEditFields::from(&category),
        parents,
        available_parent_targets: category_parent_targets(
            &current_catalog,
            platform_names,
            Some(&exact_name),
        ),
    })
    .map_err(|error| PlatformWriteFailure::Other(error.to_string()))
}

fn create_category_in_library(
    root: PathBuf,
    payload: CategoryEditPayload,
    mut navigation_catalog: NavigationCatalog,
    platform_names: Vec<String>,
) -> Result<CategoryWriteSuccess, PlatformWriteFailure> {
    let catalog_path = platform_catalog_path(&root)?;
    let parents_path = parents_document_path(&root)?;
    let mut catalog_document = AuxiliaryDocument::load(&catalog_path)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let mut parents_document = AuxiliaryDocument::load(&parents_path)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let current_catalog = catalog_document
        .platform_catalog()
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    navigation_catalog.categories = current_catalog.categories.clone();
    navigation_catalog.parents = parents_document
        .parent_relationships()
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    validate_category_hierarchy_edit(&navigation_catalog, &platform_names, &payload, true)
        .map_err(PlatformWriteFailure::Other)?;
    let category = category_edit_fields_to_domain(&payload.category, None);
    let name = category.metadata.name.clone();
    catalog_document
        .add_platform_category(category)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let parent_edits = category_parent_relationships(&name, &payload.parents);
    let placement_count = parent_edits.len();
    parents_document
        .set_platform_category_parents(&name, parent_edits)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    finish_category_documents(
        &root,
        catalog_path,
        parents_path,
        catalog_document,
        parents_document,
        name,
        placement_count,
        0,
        0,
    )
}

fn edit_category_in_library(
    root: PathBuf,
    original_name: String,
    payload: CategoryEditPayload,
    mut navigation_catalog: NavigationCatalog,
    platform_names: Vec<String>,
) -> Result<CategoryWriteSuccess, PlatformWriteFailure> {
    let catalog_path = platform_catalog_path(&root)?;
    let parents_path = parents_document_path(&root)?;
    let mut catalog_document = AuxiliaryDocument::load(&catalog_path)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let mut parents_document = AuxiliaryDocument::load(&parents_path)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let current_catalog = catalog_document
        .platform_catalog()
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let original = current_catalog
        .categories
        .iter()
        .find(|category| category.metadata.name.eq_ignore_ascii_case(&original_name))
        .cloned()
        .ok_or_else(|| {
            PlatformWriteFailure::Other(format!("platform category was not found: {original_name}"))
        })?;
    navigation_catalog.categories = current_catalog.categories;
    navigation_catalog.parents = parents_document
        .parent_relationships()
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    validate_category_hierarchy_edit(&navigation_catalog, &platform_names, &payload, false)
        .map_err(PlatformWriteFailure::Other)?;
    let category = category_edit_fields_to_domain(&payload.category, Some(&original));
    let name = original.metadata.name;
    catalog_document
        .set_platform_category(&name, category)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let parent_edits = category_parent_relationships(&name, &payload.parents);
    let placement_count = parent_edits.len();
    parents_document
        .set_platform_category_parents(&name, parent_edits)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    finish_category_documents(
        &root,
        catalog_path,
        parents_path,
        catalog_document,
        parents_document,
        name,
        placement_count,
        0,
        0,
    )
}

fn delete_category_from_library(
    root: PathBuf,
    name: String,
) -> Result<CategoryWriteSuccess, PlatformWriteFailure> {
    let catalog_path = platform_catalog_path(&root)?;
    let parents_path = parents_document_path(&root)?;
    let mut catalog_document = AuxiliaryDocument::load(&catalog_path)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let mut parents_document = AuxiliaryDocument::load(&parents_path)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let category = catalog_document
        .remove_platform_category(&name)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let exact_name = category.metadata.name;
    let removed = parents_document
        .remove_platform_category_relationships(&exact_name)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    finish_category_documents(
        &root,
        catalog_path,
        parents_path,
        catalog_document,
        parents_document,
        exact_name,
        0,
        removed.removed_placements,
        removed.detached_children,
    )
}

#[allow(clippy::too_many_arguments)]
fn finish_category_documents(
    root: &Path,
    catalog_path: PathBuf,
    parents_path: PathBuf,
    catalog_document: AuxiliaryDocument,
    parents_document: AuxiliaryDocument,
    name: String,
    placement_count: usize,
    removed_placements: usize,
    detached_children: usize,
) -> Result<CategoryWriteSuccess, PlatformWriteFailure> {
    let categories = catalog_document
        .platform_catalog()
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?
        .categories;
    let parents = parents_document
        .parent_relationships()
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let mut transaction =
        LibraryTransaction::new(root).map_err(classify_platform_transaction_error)?;
    transaction
        .stage_auxiliary(&catalog_document)
        .map_err(classify_platform_transaction_error)?;
    transaction
        .stage_auxiliary(&parents_document)
        .map_err(classify_platform_transaction_error)?;
    let report = transaction
        .commit()
        .map_err(classify_platform_transaction_error)?;
    let catalog_backup = report
        .writes
        .iter()
        .find(|write| write.target == catalog_path)
        .map(|write| write.backup.clone())
        .ok_or_else(|| {
            PlatformWriteFailure::Other(
                "platform category transaction reported no catalog write".into(),
            )
        })?;
    let parents_backup = report
        .writes
        .iter()
        .find(|write| write.target == parents_path)
        .map(|write| write.backup.clone())
        .ok_or_else(|| {
            PlatformWriteFailure::Other(
                "platform category transaction reported no hierarchy write".into(),
            )
        })?;
    Ok(CategoryWriteSuccess {
        name,
        categories,
        parents,
        catalog_backup,
        parents_backup,
        placement_count,
        removed_placements,
        detached_children,
    })
}

fn new_playlist_payload(
    catalog: &NavigationCatalog,
    platform_names: &[String],
    games: &[Game],
) -> Result<String, PlatformWriteFailure> {
    let id = Uuid::new_v4().to_string();
    serde_json::to_string(&PlaylistEditPayload {
        version: PLAYLIST_EDIT_PAYLOAD_VERSION,
        playlist: PlaylistEditFields {
            id,
            name: String::new(),
            nested_name: None,
            sort_title: None,
            notes: None,
            video_path: None,
            image_type: None,
            category: None,
            last_game_id: None,
            big_box_view: None,
            big_box_theme: None,
            hide_in_big_box: false,
            include_with_platforms: false,
            auto_populate: false,
            is_autogenerated: false,
            sort_by: Some("Title".into()),
        },
        filters: Vec::new(),
        games: Vec::new(),
        parents: vec![CategoryParentEditPayload {
            source_index: None,
            target_kind: CategoryParentKind::Root,
            target_key: String::new(),
        }],
        available_parent_targets: playlist_parent_targets(catalog, platform_names, None),
        available_games: available_playlist_games(games),
    })
    .map_err(|error| PlatformWriteFailure::Other(error.to_string()))
}

fn load_playlist_edit_payload(
    root: &Path,
    playlist_id: &str,
    base_catalog: &NavigationCatalog,
    platform_names: &[String],
    games: &[Game],
) -> Result<String, PlatformWriteFailure> {
    let source = base_catalog
        .playlists
        .iter()
        .find(|document| document.playlist.id.eq_ignore_ascii_case(playlist_id))
        .map(|document| document.source_path.clone())
        .ok_or_else(|| {
            PlatformWriteFailure::Other(format!("playlist was not found: {playlist_id}"))
        })?;
    let document = AuxiliaryDocument::load(&source)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let playlist_document = document
        .playlist_document()
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let exact_id = playlist_document.playlist.id.clone();
    let parents_document = AuxiliaryDocument::load(parents_document_path(root)?)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let all_parents = parents_document
        .parent_relationships()
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let mut parents = all_parents
        .iter()
        .filter(|relationship| {
            relationship
                .playlist_id
                .as_deref()
                .is_some_and(|id| id.eq_ignore_ascii_case(&exact_id))
        })
        .enumerate()
        .map(|(source_index, relationship)| category_parent_payload(source_index, relationship))
        .collect::<Vec<_>>();
    if parents.is_empty() {
        parents.push(CategoryParentEditPayload {
            source_index: None,
            target_kind: CategoryParentKind::Root,
            target_key: String::new(),
        });
    }
    let filters = playlist_document
        .filters
        .iter()
        .enumerate()
        .map(|(source_index, filter)| PlaylistFilterEditPayload {
            source_index: Some(source_index),
            field_key: filter.field_key.clone(),
            comparison_type_key: filter.comparison_type_key.clone(),
            value: filter.value.clone(),
        })
        .collect();
    let playlist_games = playlist_document
        .games
        .iter()
        .enumerate()
        .map(|(source_index, game)| PlaylistGameEditPayload {
            source_index: Some(source_index),
            game_id: game.game_id.clone(),
            game_title: game.game_title.clone(),
            game_platform: game.game_platform.clone(),
            game_file_name: game.game_file_name.clone(),
            launchbox_db_id: game.launchbox_db_id,
            manual_order: game.manual_order,
        })
        .collect();
    let mut current_catalog = base_catalog.clone();
    if let Some(existing) = current_catalog
        .playlists
        .iter_mut()
        .find(|document| document.playlist.id.eq_ignore_ascii_case(&exact_id))
    {
        *existing = playlist_document.clone();
    }
    current_catalog.parents = all_parents;
    serde_json::to_string(&PlaylistEditPayload {
        version: PLAYLIST_EDIT_PAYLOAD_VERSION,
        playlist: PlaylistEditFields::from(&playlist_document.playlist),
        filters,
        games: playlist_games,
        parents,
        available_parent_targets: playlist_parent_targets(
            &current_catalog,
            platform_names,
            Some(&exact_id),
        ),
        available_games: available_playlist_games(games),
    })
    .map_err(|error| PlatformWriteFailure::Other(error.to_string()))
}

fn playlist_directory_path(root: &Path) -> Result<PathBuf, PlatformWriteFailure> {
    [root.join("Data/Playlists"), root.join("Playlists")]
        .into_iter()
        .find(|candidate| candidate.is_dir())
        .ok_or_else(|| {
            PlatformWriteFailure::Other(format!(
                "could not find a writable Playlists directory under {}",
                root.display()
            ))
        })
}

fn list_cache_document_path(root: &Path) -> Option<PathBuf> {
    [root.join("Data/ListCache.xml"), root.join("ListCache.xml")]
        .into_iter()
        .find(|candidate| candidate.is_file())
}

fn ensure_portable_playlist_target_available(
    directory: &Path,
    file_name: &str,
) -> Result<PathBuf, PlatformWriteFailure> {
    let target = directory.join(file_name);
    let entries = fs::read_dir(directory).map_err(|error| {
        PlatformWriteFailure::Other(format!(
            "could not inspect playlist directory {}: {error}",
            directory.display()
        ))
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
        if entry
            .file_name()
            .to_string_lossy()
            .eq_ignore_ascii_case(file_name)
        {
            return Err(PlatformWriteFailure::Other(format!(
                "portable playlist filename collides with existing {}",
                entry.path().display()
            )));
        }
    }
    Ok(target)
}

fn create_playlist_in_library(
    root: PathBuf,
    mut payload: PlaylistEditPayload,
    mut navigation_catalog: NavigationCatalog,
    platform_names: Vec<String>,
    games: Vec<Game>,
) -> Result<PlaylistWriteSuccess, PlatformWriteFailure> {
    let parents_path = parents_document_path(&root)?;
    let mut parents_document = AuxiliaryDocument::load(&parents_path)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    navigation_catalog.parents = parents_document
        .parent_relationships()
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    validate_playlist_hierarchy_edit(&navigation_catalog, &platform_names, &payload, true)
        .map_err(PlatformWriteFailure::Other)?;
    canonicalize_playlist_games(&mut payload, &games).map_err(PlatformWriteFailure::Other)?;
    let playlist = playlist_edit_fields_to_domain(&payload.playlist, None);
    let id = playlist.id.clone();
    let directory = playlist_directory_path(&root)?;
    let file_name = navigation_document_file_name(&playlist.metadata.name)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let source = ensure_portable_playlist_target_available(&directory, &file_name)?;
    let filter_edits = playlist_filter_edits(&payload.filters);
    let game_edits = playlist_game_edits(&payload.games);
    let document = AuxiliaryDocument::new_playlist(
        &source,
        playlist,
        filter_edits.into_iter().map(|edit| edit.record).collect(),
        game_edits.into_iter().map(|edit| edit.record).collect(),
    )
    .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let parent_edits = playlist_parent_relationships(&id, &payload.parents);
    let placement_count = parent_edits.len();
    parents_document
        .set_playlist_parents(&id, parent_edits)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let playlist_document = document
        .playlist_document()
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let parents = parents_document
        .parent_relationships()
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let mut transaction =
        LibraryTransaction::new(&root).map_err(classify_platform_transaction_error)?;
    transaction
        .stage_new_playlist(&document)
        .map_err(classify_platform_transaction_error)?;
    transaction
        .stage_auxiliary(&parents_document)
        .map_err(classify_platform_transaction_error)?;
    let report = transaction
        .commit()
        .map_err(classify_platform_transaction_error)?;
    let parents_backup = report
        .writes
        .iter()
        .find(|write| write.target == parents_path)
        .map(|write| write.backup.clone())
        .ok_or_else(|| {
            PlatformWriteFailure::Other("playlist create reported no hierarchy write".into())
        })?;
    navigation_catalog.playlists.push(playlist_document);
    Ok(PlaylistWriteSuccess {
        id,
        playlists: navigation_catalog.playlists,
        parents,
        source,
        playlist_backup: None,
        parents_backup,
        list_cache_backup: None,
        placement_count,
        removed_placements: 0,
        detached_children: 0,
        removed_cache_rows: 0,
    })
}

fn edit_playlist_in_library(
    root: PathBuf,
    playlist_id: String,
    mut payload: PlaylistEditPayload,
    mut navigation_catalog: NavigationCatalog,
    platform_names: Vec<String>,
    games: Vec<Game>,
) -> Result<PlaylistWriteSuccess, PlatformWriteFailure> {
    let source = navigation_catalog
        .playlists
        .iter()
        .find(|document| document.playlist.id.eq_ignore_ascii_case(&playlist_id))
        .map(|document| document.source_path.clone())
        .ok_or_else(|| {
            PlatformWriteFailure::Other(format!("playlist was not found: {playlist_id}"))
        })?;
    let parents_path = parents_document_path(&root)?;
    let mut document = AuxiliaryDocument::load(&source)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let original = document
        .playlist_document()
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let exact_id = original.playlist.id.clone();
    let mut parents_document = AuxiliaryDocument::load(&parents_path)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    if let Some(existing) = navigation_catalog
        .playlists
        .iter_mut()
        .find(|document| document.playlist.id.eq_ignore_ascii_case(&exact_id))
    {
        *existing = original.clone();
    }
    navigation_catalog.parents = parents_document
        .parent_relationships()
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    validate_playlist_hierarchy_edit(&navigation_catalog, &platform_names, &payload, false)
        .map_err(PlatformWriteFailure::Other)?;
    canonicalize_playlist_games(&mut payload, &games).map_err(PlatformWriteFailure::Other)?;
    let playlist = playlist_edit_fields_to_domain(&payload.playlist, Some(&original.playlist));
    document
        .set_playlist(
            &exact_id,
            playlist,
            playlist_filter_edits(&payload.filters),
            playlist_game_edits(&payload.games),
        )
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let parent_edits = playlist_parent_relationships(&exact_id, &payload.parents);
    let placement_count = parent_edits.len();
    parents_document
        .set_playlist_parents(&exact_id, parent_edits)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let playlist_document = document
        .playlist_document()
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let parents = parents_document
        .parent_relationships()
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let mut transaction =
        LibraryTransaction::new(&root).map_err(classify_platform_transaction_error)?;
    transaction
        .stage_auxiliary(&document)
        .map_err(classify_platform_transaction_error)?;
    transaction
        .stage_auxiliary(&parents_document)
        .map_err(classify_platform_transaction_error)?;
    let report = transaction
        .commit()
        .map_err(classify_platform_transaction_error)?;
    let playlist_backup = report
        .writes
        .iter()
        .find(|write| write.target == source)
        .map(|write| write.backup.clone())
        .ok_or_else(|| {
            PlatformWriteFailure::Other("playlist edit reported no playlist write".into())
        })?;
    let parents_backup = report
        .writes
        .iter()
        .find(|write| write.target == parents_path)
        .map(|write| write.backup.clone())
        .ok_or_else(|| {
            PlatformWriteFailure::Other("playlist edit reported no hierarchy write".into())
        })?;
    if let Some(existing) = navigation_catalog
        .playlists
        .iter_mut()
        .find(|document| document.playlist.id.eq_ignore_ascii_case(&exact_id))
    {
        *existing = playlist_document;
    }
    Ok(PlaylistWriteSuccess {
        id: exact_id,
        playlists: navigation_catalog.playlists,
        parents,
        source,
        playlist_backup: Some(playlist_backup),
        parents_backup,
        list_cache_backup: None,
        placement_count,
        removed_placements: 0,
        detached_children: 0,
        removed_cache_rows: 0,
    })
}

fn delete_playlist_from_library(
    root: PathBuf,
    playlist_id: String,
    mut navigation_catalog: NavigationCatalog,
) -> Result<PlaylistWriteSuccess, PlatformWriteFailure> {
    let source = navigation_catalog
        .playlists
        .iter()
        .find(|document| document.playlist.id.eq_ignore_ascii_case(&playlist_id))
        .map(|document| document.source_path.clone())
        .ok_or_else(|| {
            PlatformWriteFailure::Other(format!("playlist was not found: {playlist_id}"))
        })?;
    let document = AuxiliaryDocument::load(&source)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let exact_id = document
        .playlist_document()
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?
        .playlist
        .id;
    let parents_path = parents_document_path(&root)?;
    let mut parents_document = AuxiliaryDocument::load(&parents_path)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let removed = parents_document
        .remove_playlist_relationships(&exact_id)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let parents = parents_document
        .parent_relationships()
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let mut cache_document = list_cache_document_path(&root)
        .map(AuxiliaryDocument::load)
        .transpose()
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let removed_cache_rows = cache_document
        .as_mut()
        .map(|cache| cache.remove_playlist_list_cache_items(&exact_id))
        .transpose()
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?
        .unwrap_or_default();
    let cache_path = cache_document
        .as_ref()
        .map(|document| document.source_path().to_path_buf());
    let mut transaction =
        LibraryTransaction::new(&root).map_err(classify_platform_transaction_error)?;
    transaction
        .stage_delete_playlist(&document)
        .map_err(classify_platform_transaction_error)?;
    transaction
        .stage_auxiliary(&parents_document)
        .map_err(classify_platform_transaction_error)?;
    if removed_cache_rows > 0 {
        transaction
            .stage_auxiliary(
                cache_document
                    .as_ref()
                    .expect("removed cache rows require document"),
            )
            .map_err(classify_platform_transaction_error)?;
    }
    let report = transaction
        .commit()
        .map_err(classify_platform_transaction_error)?;
    let playlist_backup = report
        .deleted_targets
        .iter()
        .find(|deleted| deleted.target == source)
        .map(|deleted| deleted.backup.clone())
        .ok_or_else(|| {
            PlatformWriteFailure::Other("playlist delete reported no playlist deletion".into())
        })?;
    let parents_backup = report
        .writes
        .iter()
        .find(|write| write.target == parents_path)
        .map(|write| write.backup.clone())
        .ok_or_else(|| {
            PlatformWriteFailure::Other("playlist delete reported no hierarchy write".into())
        })?;
    let list_cache_backup = cache_path.and_then(|cache_path| {
        report
            .writes
            .iter()
            .find(|write| write.target == cache_path)
            .map(|write| write.backup.clone())
    });
    navigation_catalog
        .playlists
        .retain(|document| !document.playlist.id.eq_ignore_ascii_case(&exact_id));
    Ok(PlaylistWriteSuccess {
        id: exact_id,
        playlists: navigation_catalog.playlists,
        parents,
        source,
        playlist_backup: Some(playlist_backup),
        parents_backup,
        list_cache_backup,
        placement_count: 0,
        removed_placements: removed.removed_placements,
        detached_children: removed.detached_children,
        removed_cache_rows,
    })
}

fn load_platform_edit_payload(root: &Path, name: &str) -> Result<String, PlatformWriteFailure> {
    let catalog_path = platform_catalog_path(root)?;
    let document = AuxiliaryDocument::load(&catalog_path)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let catalog = document
        .platform_catalog()
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let platform = catalog
        .platforms
        .into_iter()
        .find(|platform| platform.metadata.name.eq_ignore_ascii_case(name))
        .ok_or_else(|| PlatformWriteFailure::Other(format!("platform was not found: {name}")))?;
    let exact_name = platform.metadata.name.clone();
    let folders = catalog
        .folders
        .into_iter()
        .filter(|folder| folder.platform.eq_ignore_ascii_case(&exact_name))
        .enumerate()
        .map(|(source_index, folder)| PlatformFolderEditPayload {
            source_index: Some(source_index),
            media_type: folder.media_type,
            folder_path: folder.folder_path,
        })
        .collect();
    serde_json::to_string(&PlatformEditPayload {
        version: PLATFORM_EDIT_PAYLOAD_VERSION,
        platform,
        folders,
    })
    .map_err(|error| PlatformWriteFailure::Other(error.to_string()))
}

fn write_platform_definition(
    root: PathBuf,
    original_name: String,
    payload: PlatformEditPayload,
) -> Result<PlatformEditSuccess, PlatformWriteFailure> {
    let platform = payload.platform.clone();
    let catalog_path = platform_catalog_path(&root)?;
    let mut document = AuxiliaryDocument::load(&catalog_path)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;
    let folder_edits = payload
        .folders
        .into_iter()
        .map(|folder| IndexedPlatformRecordEdit {
            source_index: folder.source_index,
            record: PlatformFolder {
                platform: original_name.clone(),
                media_type: folder.media_type,
                folder_path: folder.folder_path,
            },
        })
        .collect::<Vec<_>>();
    let folder_count = folder_edits.len();
    document
        .set_platform_definition(&original_name, payload.platform, folder_edits)
        .map_err(|error| PlatformWriteFailure::Other(error.to_string()))?;

    let mut transaction =
        LibraryTransaction::new(&root).map_err(classify_platform_transaction_error)?;
    transaction
        .stage_auxiliary(&document)
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
                "platform edit transaction reported no catalog write".into(),
            )
        })?;
    Ok(PlatformEditSuccess {
        name: original_name,
        platform,
        catalog_backup,
        folder_count,
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
    let platform_definition = PlatformDefinition {
        metadata: NavigationMetadata {
            name: name.clone(),
            scrape_as: (!scrape_as.trim().is_empty()).then_some(scrape_as),
            ..NavigationMetadata::default()
        },
        ..PlatformDefinition::default()
    };
    catalog
        .add_platform_definition(platform_definition.clone(), folders.clone())
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
        platform: platform_definition,
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
                    navigation_catalog: NavigationCatalog::default(),
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
        if *self.as_ref().import_scanning()
            || *self.as_ref().writing()
            || *self.as_ref().launching()
        {
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

    pub fn local_path_from_url(&self, value: QString) -> QString {
        let url = QUrl::from_user_input(&value, &QString::default());
        url.to_local_file().unwrap_or_default()
    }

    pub fn preview_rom_import(mut self: Pin<&mut Self>, request_payload: QString) {
        if *self.as_ref().loading()
            || *self.as_ref().import_scanning()
            || *self.as_ref().writing()
            || *self.as_ref().launching()
        {
            self.as_mut()
                .set_status_message(qstring("Wait for the current library operation to finish."));
            return;
        }
        let request =
            match serde_json::from_str::<ManualImportRequest>(&request_payload.to_string()) {
                Ok(request) => request,
                Err(error) => {
                    self.as_mut().set_status_message(qstring(format!(
                        "Could not preview ROM import: invalid request: {error}"
                    )));
                    return;
                }
            };
        let Some(root) = self.as_ref().rust().launchbox_root.clone() else {
            self.as_mut().set_status_message(qstring(
                "ROM import requires a loaded LaunchBox directory, not a standalone XML file.",
            ));
            return;
        };
        let resolver = self.as_ref().rust().path_resolver.clone();
        let generation = self.as_ref().rust().request_generation;
        self.as_mut().set_import_preview_json(QString::default());
        self.as_mut().set_import_scanning(true);
        self.as_mut().set_status_message(qstring(
            "Scanning ROM import locations in the background...",
        ));

        let qt_thread = self.as_ref().qt_thread();
        let spawn_result = std::thread::Builder::new()
            .name("launchbox-rom-import-preview".to_string())
            .spawn(move || {
                let result = match preview_manual_import(root, &resolver, request) {
                    Ok(preview) => {
                        let count = preview.importable_count;
                        serde_json::to_string(&preview)
                            .map(|json| (json, count))
                            .map_err(|error| error.to_string())
                    }
                    Err(error) => Err(error.to_string()),
                };
                qt_thread
                    .queue(move |mut controller| {
                        controller
                            .as_mut()
                            .finish_rom_import_preview(generation, result);
                    })
                    .ok();
            });
        if let Err(error) = spawn_result {
            self.as_mut().set_import_scanning(false);
            self.as_mut().set_status_message(qstring(format!(
                "Could not start ROM import scanner: {error}"
            )));
        }
    }

    pub fn clear_rom_import_preview(mut self: Pin<&mut Self>) {
        if !*self.as_ref().import_scanning() {
            self.as_mut().set_import_preview_json(QString::default());
        }
    }

    pub fn import_roms(mut self: Pin<&mut Self>, selection_payload: QString) {
        if !self.as_mut().begin_library_mutation() {
            return;
        }
        let selection =
            match serde_json::from_str::<ManualImportSelection>(&selection_payload.to_string()) {
                Ok(selection) => selection,
                Err(error) => {
                    self.as_mut().set_status_message(qstring(format!(
                        "Could not import ROMs: invalid selection: {error}"
                    )));
                    return;
                }
            };
        let platform = selection.request.platform.clone();
        let Some((source, root)) = self.as_ref().platform_write_target(&platform) else {
            self.as_mut().set_status_message(qstring(
                "The selected platform has no loaded writable platform document.",
            ));
            return;
        };
        let resolver = self.as_ref().rust().path_resolver.clone();
        let generation = self.as_ref().rust().request_generation;
        self.as_mut().set_last_import_count(0);
        self.as_mut().set_last_import_created_file_count(0);
        self.as_mut().set_last_import_moved_file_count(0);
        self.as_mut().rust_mut().last_imported_game_ids.clear();
        self.as_mut().set_writing(true);
        self.as_mut().set_status_message(qstring(format!(
            "Importing ROMs into {platform} in the background..."
        )));

        let qt_thread = self.as_ref().qt_thread();
        let spawn_result = std::thread::Builder::new()
            .name("launchbox-rom-import".to_string())
            .spawn(move || {
                let result = execute_manual_import(&root, &source, &resolver, selection)
                    .map(|report| RomImportSuccess { report, source });
                qt_thread
                    .queue(move |mut controller| {
                        controller.as_mut().finish_rom_import(generation, result);
                    })
                    .ok();
            });
        if let Err(error) = spawn_result {
            self.as_mut().set_writing(false);
            self.as_mut()
                .set_status_message(qstring(format!("Could not start ROM importer: {error}")));
        }
    }

    pub fn apply_filters(mut self: Pin<&mut Self>, search_text: QString, platform: QString) {
        let platform_key = platform.to_string();
        self.as_mut().set_search_text(search_text);
        self.as_mut().set_platform_filter(platform);
        self.as_mut()
            .set_navigation_filter_kind(qstring(if platform_key.is_empty() {
                ""
            } else {
                "platform"
            }));
        self.as_mut()
            .set_navigation_filter_key(qstring(platform_key));
        self.as_mut().rust_mut().category_filter = None;
        self.as_mut().rust_mut().playlist_filter = None;
        self.as_mut().refresh_filtered_games();
    }

    pub fn apply_category_filter(
        mut self: Pin<&mut Self>,
        search_text: QString,
        category: QString,
    ) {
        let category = category.to_string();
        let category_key = category.to_lowercase();
        if !self
            .as_ref()
            .rust()
            .category_game_ids
            .contains_key(&category_key)
        {
            self.as_mut().set_status_message(qstring(format!(
                "Platform category is no longer available: {category}"
            )));
            return;
        }
        self.as_mut().set_search_text(search_text);
        self.as_mut().set_platform_filter(QString::default());
        self.as_mut()
            .set_navigation_filter_kind(qstring("category"));
        self.as_mut().set_navigation_filter_key(qstring(&category));
        self.as_mut().rust_mut().category_filter = Some(category_key);
        self.as_mut().rust_mut().playlist_filter = None;
        self.as_mut().refresh_filtered_games();
    }

    pub fn apply_playlist_filter(
        mut self: Pin<&mut Self>,
        search_text: QString,
        playlist_id: QString,
    ) {
        let playlist_id = playlist_id.to_string();
        let playlist_key = playlist_id.to_lowercase();
        if !self
            .as_ref()
            .rust()
            .playlist_game_ids
            .contains_key(&playlist_key)
        {
            self.as_mut().set_status_message(qstring(format!(
                "Playlist is no longer available: {playlist_id}"
            )));
            return;
        }
        self.as_mut().set_search_text(search_text);
        self.as_mut().set_platform_filter(QString::default());
        self.as_mut()
            .set_navigation_filter_kind(qstring("playlist"));
        self.as_mut()
            .set_navigation_filter_key(qstring(&playlist_id));
        self.as_mut().rust_mut().category_filter = None;
        self.as_mut().rust_mut().playlist_filter = Some(playlist_key);
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

    pub fn add_additional_application(
        mut self: Pin<&mut Self>,
        row: i32,
        game_id: QString,
        edit_payload: QString,
    ) {
        if !self.as_mut().begin_library_mutation() {
            return;
        }
        let mut payload = match parse_additional_application_edit_payload(&edit_payload.to_string())
        {
            Ok(payload) => payload,
            Err(error) => {
                self.as_mut().set_status_message(qstring(format!(
                    "Could not add additional application: {error}."
                )));
                return;
            }
        };
        if let Err(error) = canonicalize_additional_application_emulator(
            &mut payload.application,
            self.as_ref().rust().emulator_configuration.as_ref(),
            None,
        ) {
            self.as_mut().set_status_message(qstring(format!(
                "Could not add additional application: {error}."
            )));
            return;
        }
        let game_id = game_id.to_string();
        let Some((source, root)) = self.as_ref().edit_target(row, &game_id) else {
            self.as_mut().set_status_message(qstring(
                "The selected game no longer matches this model; reload and try again.",
            ));
            return;
        };
        let name = payload.application.name.clone();
        let request = AdditionalApplicationWriteRequest::Create {
            id: Uuid::new_v4().to_string(),
            edit: payload.application,
        };
        self.as_mut().start_additional_application_write(
            root,
            source,
            game_id,
            request,
            format!("Adding {name} in the background..."),
        );
    }

    pub fn save_additional_application(
        mut self: Pin<&mut Self>,
        row: i32,
        game_id: QString,
        application_id: QString,
        edit_payload: QString,
    ) {
        if !self.as_mut().begin_library_mutation() {
            return;
        }
        let game_id = game_id.to_string();
        let application_id = application_id.to_string();
        let existing = self
            .as_ref()
            .additional_applications_for_model(row, &game_id)
            .and_then(|applications| {
                applications
                    .iter()
                    .find(|application| application.id == application_id)
            })
            .cloned();
        let Some(existing) = existing else {
            self.as_mut().set_status_message(qstring(
                "The selected additional application no longer matches this game; reload and try again.",
            ));
            return;
        };
        let mut payload = match parse_additional_application_edit_payload(&edit_payload.to_string())
        {
            Ok(payload) => payload,
            Err(error) => {
                self.as_mut().set_status_message(qstring(format!(
                    "Could not save additional application: {error}."
                )));
                return;
            }
        };
        if let Err(error) = canonicalize_additional_application_emulator(
            &mut payload.application,
            self.as_ref().rust().emulator_configuration.as_ref(),
            existing.emulator_id.as_deref(),
        ) {
            self.as_mut().set_status_message(qstring(format!(
                "Could not save additional application: {error}."
            )));
            return;
        }
        let Some((source, root)) = self.as_ref().edit_target(row, &game_id) else {
            self.as_mut().set_status_message(qstring(
                "The selected game no longer matches this model; reload and try again.",
            ));
            return;
        };
        let name = payload.application.name.clone();
        let request = AdditionalApplicationWriteRequest::Edit {
            id: application_id,
            edit: payload.application,
        };
        self.as_mut().start_additional_application_write(
            root,
            source,
            game_id,
            request,
            format!("Saving {name} in the background..."),
        );
    }

    pub fn delete_additional_application(
        mut self: Pin<&mut Self>,
        row: i32,
        game_id: QString,
        application_id: QString,
    ) {
        if !self.as_mut().begin_library_mutation() {
            return;
        }
        let game_id = game_id.to_string();
        let application_id = application_id.to_string();
        let existing = self
            .as_ref()
            .additional_applications_for_model(row, &game_id)
            .and_then(|applications| {
                applications
                    .iter()
                    .find(|application| application.id == application_id)
            })
            .cloned();
        let Some(existing) = existing else {
            self.as_mut().set_status_message(qstring(
                "The selected additional application no longer matches this game; reload and try again.",
            ));
            return;
        };
        let Some((source, root)) = self.as_ref().edit_target(row, &game_id) else {
            self.as_mut().set_status_message(qstring(
                "The selected game no longer matches this model; reload and try again.",
            ));
            return;
        };
        let request = AdditionalApplicationWriteRequest::Delete { id: application_id };
        self.as_mut().start_additional_application_write(
            root,
            source,
            game_id,
            request,
            format!("Deleting {} in the background...", existing.name),
        );
    }

    fn start_additional_application_write(
        mut self: Pin<&mut Self>,
        root: PathBuf,
        source: PathBuf,
        game_id: String,
        request: AdditionalApplicationWriteRequest,
        status: String,
    ) {
        let generation = self.as_ref().rust().request_generation;
        self.as_mut().set_writing(true);
        self.as_mut().set_status_message(qstring(status));
        let qt_thread = self.as_ref().qt_thread();
        let spawn_result = std::thread::Builder::new()
            .name("launchbox-additional-application-write".to_string())
            .spawn(move || {
                let result = write_additional_application(root, source, game_id, request);
                qt_thread
                    .queue(move |mut controller| {
                        controller
                            .as_mut()
                            .finish_additional_application_write(generation, result);
                    })
                    .ok();
            });
        if let Err(error) = spawn_result {
            self.as_mut().set_writing(false);
            self.as_mut().set_status_message(qstring(format!(
                "Could not start additional-application writer: {error}"
            )));
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
            emulator_id: None,
            metadata: NewGameMetadata::default(),
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

    pub fn platform_edit_payload(&self, name: QString) -> QString {
        let Some(root) = self.rust().launchbox_root.as_deref() else {
            return QString::default();
        };
        match load_platform_edit_payload(root, name.to_string().trim()) {
            Ok(payload) => qstring(payload),
            Err(error) => {
                eprintln!(
                    "Could not prepare platform editor: {}",
                    describe_platform_write_failure(&error)
                );
                QString::default()
            }
        }
    }

    pub fn save_platform(mut self: Pin<&mut Self>, original_name: QString, edit_payload: QString) {
        if !self.as_mut().begin_library_mutation() {
            return;
        }
        let original_name = original_name.to_string().trim().to_string();
        let payload = match parse_platform_edit_payload(&original_name, &edit_payload.to_string()) {
            Ok(payload) => payload,
            Err(error) => {
                self.as_mut()
                    .set_status_message(qstring(format!("Could not save platform: {error}.")));
                return;
            }
        };
        let Some(root) = self.as_ref().rust().launchbox_root.clone() else {
            self.as_mut().set_status_message(qstring(
                "Platform editing requires a loaded LaunchBox directory, not a standalone XML file.",
            ));
            return;
        };
        let generation = self.as_ref().rust().request_generation;
        self.as_mut().set_writing(true);
        self.as_mut()
            .set_status_message(qstring(format!("Saving platform {original_name}...")));

        let qt_thread = self.as_ref().qt_thread();
        let spawn_result = std::thread::Builder::new()
            .name("launchbox-platform-edit".to_string())
            .spawn(move || {
                let result = write_platform_definition(root, original_name, payload);
                qt_thread
                    .queue(move |mut controller| {
                        controller.as_mut().finish_platform_edit(generation, result);
                    })
                    .ok();
            });
        if let Err(error) = spawn_result {
            self.as_mut().set_writing(false);
            self.as_mut()
                .set_status_message(qstring(format!("Could not start platform writer: {error}")));
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

    pub fn new_category_edit_payload(&self) -> QString {
        match new_category_payload(&self.rust().navigation_catalog, &self.rust().platform_names) {
            Ok(payload) => qstring(payload),
            Err(error) => {
                eprintln!(
                    "Could not prepare new platform category editor: {}",
                    describe_platform_write_failure(&error)
                );
                QString::default()
            }
        }
    }

    pub fn category_edit_payload(&self, name: QString) -> QString {
        let Some(root) = self.rust().launchbox_root.as_deref() else {
            return QString::default();
        };
        match load_category_edit_payload(
            root,
            name.to_string().trim(),
            &self.rust().navigation_catalog,
            &self.rust().platform_names,
        ) {
            Ok(payload) => qstring(payload),
            Err(error) => {
                eprintln!(
                    "Could not prepare platform category editor: {}",
                    describe_platform_write_failure(&error)
                );
                QString::default()
            }
        }
    }

    pub fn add_category(mut self: Pin<&mut Self>, edit_payload: QString) {
        if !self.as_mut().begin_library_mutation() {
            return;
        }
        let payload = match parse_category_edit_payload(None, &edit_payload.to_string()) {
            Ok(payload) => payload,
            Err(error) => {
                self.as_mut().set_status_message(qstring(format!(
                    "Could not create platform category: {error}."
                )));
                return;
            }
        };
        if let Err(error) = validate_category_hierarchy_edit(
            &self.as_ref().rust().navigation_catalog,
            &self.as_ref().rust().platform_names,
            &payload,
            true,
        ) {
            self.as_mut().set_status_message(qstring(format!(
                "Could not create platform category: {error}."
            )));
            return;
        }
        let Some(root) = self.as_ref().rust().launchbox_root.clone() else {
            self.as_mut().set_status_message(qstring(
                "Platform category creation requires a loaded LaunchBox directory, not a standalone XML file.",
            ));
            return;
        };
        let name = payload.category.name.clone();
        let navigation_catalog = self.as_ref().rust().navigation_catalog.clone();
        let platform_names = self.as_ref().rust().platform_names.clone();
        let generation = self.as_ref().rust().request_generation;
        self.as_mut().set_writing(true);
        self.as_mut()
            .set_status_message(qstring(format!("Creating platform category {name}...")));
        let qt_thread = self.as_ref().qt_thread();
        let spawn_result = std::thread::Builder::new()
            .name("launchbox-category-create".to_string())
            .spawn(move || {
                let result =
                    create_category_in_library(root, payload, navigation_catalog, platform_names);
                qt_thread
                    .queue(move |mut controller| {
                        controller.as_mut().finish_category_write(
                            generation,
                            CategoryWriteOperation::Create,
                            result,
                        );
                    })
                    .ok();
            });
        if let Err(error) = spawn_result {
            self.as_mut().set_writing(false);
            self.as_mut().set_status_message(qstring(format!(
                "Could not start platform category creation: {error}"
            )));
        }
    }

    pub fn save_category(mut self: Pin<&mut Self>, original_name: QString, edit_payload: QString) {
        if !self.as_mut().begin_library_mutation() {
            return;
        }
        let original_name = original_name.to_string().trim().to_string();
        let payload =
            match parse_category_edit_payload(Some(&original_name), &edit_payload.to_string()) {
                Ok(payload) => payload,
                Err(error) => {
                    self.as_mut().set_status_message(qstring(format!(
                        "Could not save platform category: {error}."
                    )));
                    return;
                }
            };
        if let Err(error) = validate_category_hierarchy_edit(
            &self.as_ref().rust().navigation_catalog,
            &self.as_ref().rust().platform_names,
            &payload,
            false,
        ) {
            self.as_mut().set_status_message(qstring(format!(
                "Could not save platform category: {error}."
            )));
            return;
        }
        let Some(root) = self.as_ref().rust().launchbox_root.clone() else {
            self.as_mut().set_status_message(qstring(
                "Platform category editing requires a loaded LaunchBox directory, not a standalone XML file.",
            ));
            return;
        };
        let navigation_catalog = self.as_ref().rust().navigation_catalog.clone();
        let platform_names = self.as_ref().rust().platform_names.clone();
        let generation = self.as_ref().rust().request_generation;
        self.as_mut().set_writing(true);
        self.as_mut().set_status_message(qstring(format!(
            "Saving platform category {original_name}..."
        )));
        let qt_thread = self.as_ref().qt_thread();
        let spawn_result = std::thread::Builder::new()
            .name("launchbox-category-edit".to_string())
            .spawn(move || {
                let result = edit_category_in_library(
                    root,
                    original_name,
                    payload,
                    navigation_catalog,
                    platform_names,
                );
                qt_thread
                    .queue(move |mut controller| {
                        controller.as_mut().finish_category_write(
                            generation,
                            CategoryWriteOperation::Edit,
                            result,
                        );
                    })
                    .ok();
            });
        if let Err(error) = spawn_result {
            self.as_mut().set_writing(false);
            self.as_mut().set_status_message(qstring(format!(
                "Could not start platform category writer: {error}"
            )));
        }
    }

    pub fn delete_category(mut self: Pin<&mut Self>, name: QString) {
        if !self.as_mut().begin_library_mutation() {
            return;
        }
        let name = name.to_string().trim().to_string();
        if !self
            .as_ref()
            .rust()
            .navigation_catalog
            .categories
            .iter()
            .any(|category| category.metadata.name.eq_ignore_ascii_case(&name))
        {
            self.as_mut().set_status_message(qstring(format!(
                "Platform category is no longer available: {name}"
            )));
            return;
        }
        let Some(root) = self.as_ref().rust().launchbox_root.clone() else {
            self.as_mut().set_status_message(qstring(
                "Platform category deletion requires a loaded LaunchBox directory, not a standalone XML file.",
            ));
            return;
        };
        let generation = self.as_ref().rust().request_generation;
        self.as_mut().set_writing(true);
        self.as_mut()
            .set_status_message(qstring(format!("Deleting platform category {name}...")));
        let qt_thread = self.as_ref().qt_thread();
        let spawn_result = std::thread::Builder::new()
            .name("launchbox-category-delete".to_string())
            .spawn(move || {
                let result = delete_category_from_library(root, name);
                qt_thread
                    .queue(move |mut controller| {
                        controller.as_mut().finish_category_write(
                            generation,
                            CategoryWriteOperation::Delete,
                            result,
                        );
                    })
                    .ok();
            });
        if let Err(error) = spawn_result {
            self.as_mut().set_writing(false);
            self.as_mut().set_status_message(qstring(format!(
                "Could not start platform category deletion: {error}"
            )));
        }
    }

    pub fn new_playlist_edit_payload(&self) -> QString {
        match new_playlist_payload(
            &self.rust().navigation_catalog,
            &self.rust().platform_names,
            &self.rust().games,
        ) {
            Ok(payload) => qstring(payload),
            Err(error) => {
                eprintln!(
                    "Could not prepare new playlist editor: {}",
                    describe_platform_write_failure(&error)
                );
                QString::default()
            }
        }
    }

    pub fn playlist_edit_payload(&self, playlist_id: QString) -> QString {
        let Some(root) = self.rust().launchbox_root.as_deref() else {
            return QString::default();
        };
        match load_playlist_edit_payload(
            root,
            playlist_id.to_string().trim(),
            &self.rust().navigation_catalog,
            &self.rust().platform_names,
            &self.rust().games,
        ) {
            Ok(payload) => qstring(payload),
            Err(error) => {
                eprintln!(
                    "Could not prepare playlist editor: {}",
                    describe_platform_write_failure(&error)
                );
                QString::default()
            }
        }
    }

    pub fn add_playlist(mut self: Pin<&mut Self>, edit_payload: QString) {
        if !self.as_mut().begin_library_mutation() {
            return;
        }
        let payload = match parse_playlist_edit_payload(None, &edit_payload.to_string()) {
            Ok(payload) => payload,
            Err(error) => {
                self.as_mut()
                    .set_status_message(qstring(format!("Could not create playlist: {error}.")));
                return;
            }
        };
        if let Err(error) = validate_playlist_hierarchy_edit(
            &self.as_ref().rust().navigation_catalog,
            &self.as_ref().rust().platform_names,
            &payload,
            true,
        ) {
            self.as_mut()
                .set_status_message(qstring(format!("Could not create playlist: {error}.")));
            return;
        }
        let Some(root) = self.as_ref().rust().launchbox_root.clone() else {
            self.as_mut().set_status_message(qstring(
                "Playlist creation requires a loaded LaunchBox directory, not a standalone XML file.",
            ));
            return;
        };
        let id = payload.playlist.id.clone();
        let name = payload.playlist.name.clone();
        let navigation_catalog = self.as_ref().rust().navigation_catalog.clone();
        let platform_names = self.as_ref().rust().platform_names.clone();
        let games = self.as_ref().rust().games.clone();
        let generation = self.as_ref().rust().request_generation;
        self.as_mut().set_writing(true);
        self.as_mut()
            .set_status_message(qstring(format!("Creating playlist {name}...")));
        let qt_thread = self.as_ref().qt_thread();
        let spawn_result = std::thread::Builder::new()
            .name("launchbox-playlist-create".to_string())
            .spawn(move || {
                let result = create_playlist_in_library(
                    root,
                    payload,
                    navigation_catalog,
                    platform_names,
                    games,
                );
                qt_thread
                    .queue(move |mut controller| {
                        controller.as_mut().finish_playlist_write(
                            generation,
                            PlaylistWriteOperation::Create,
                            result,
                        );
                    })
                    .ok();
            });
        if let Err(error) = spawn_result {
            self.as_mut().set_writing(false);
            self.as_mut().set_status_message(qstring(format!(
                "Could not start playlist {id} creation: {error}"
            )));
        }
    }

    pub fn save_playlist(mut self: Pin<&mut Self>, playlist_id: QString, edit_payload: QString) {
        if !self.as_mut().begin_library_mutation() {
            return;
        }
        let playlist_id = playlist_id.to_string().trim().to_string();
        let Some(original) = self
            .as_ref()
            .rust()
            .navigation_catalog
            .playlists
            .iter()
            .find(|document| document.playlist.id.eq_ignore_ascii_case(&playlist_id))
            .map(|document| document.playlist.clone())
        else {
            self.as_mut().set_status_message(qstring(format!(
                "Playlist is no longer available: {playlist_id}"
            )));
            return;
        };
        let payload = match parse_playlist_edit_payload(Some(&original), &edit_payload.to_string())
        {
            Ok(payload) => payload,
            Err(error) => {
                self.as_mut()
                    .set_status_message(qstring(format!("Could not save playlist: {error}.")));
                return;
            }
        };
        if let Err(error) = validate_playlist_hierarchy_edit(
            &self.as_ref().rust().navigation_catalog,
            &self.as_ref().rust().platform_names,
            &payload,
            false,
        ) {
            self.as_mut()
                .set_status_message(qstring(format!("Could not save playlist: {error}.")));
            return;
        }
        let Some(root) = self.as_ref().rust().launchbox_root.clone() else {
            self.as_mut().set_status_message(qstring(
                "Playlist editing requires a loaded LaunchBox directory, not a standalone XML file.",
            ));
            return;
        };
        let navigation_catalog = self.as_ref().rust().navigation_catalog.clone();
        let platform_names = self.as_ref().rust().platform_names.clone();
        let games = self.as_ref().rust().games.clone();
        let generation = self.as_ref().rust().request_generation;
        self.as_mut().set_writing(true);
        self.as_mut()
            .set_status_message(qstring(format!("Saving playlist {playlist_id}...")));
        let qt_thread = self.as_ref().qt_thread();
        let spawn_result = std::thread::Builder::new()
            .name("launchbox-playlist-edit".to_string())
            .spawn(move || {
                let result = edit_playlist_in_library(
                    root,
                    playlist_id,
                    payload,
                    navigation_catalog,
                    platform_names,
                    games,
                );
                qt_thread
                    .queue(move |mut controller| {
                        controller.as_mut().finish_playlist_write(
                            generation,
                            PlaylistWriteOperation::Edit,
                            result,
                        );
                    })
                    .ok();
            });
        if let Err(error) = spawn_result {
            self.as_mut().set_writing(false);
            self.as_mut()
                .set_status_message(qstring(format!("Could not start playlist writer: {error}")));
        }
    }

    pub fn delete_playlist(mut self: Pin<&mut Self>, playlist_id: QString) {
        if !self.as_mut().begin_library_mutation() {
            return;
        }
        let playlist_id = playlist_id.to_string().trim().to_string();
        if !self
            .as_ref()
            .rust()
            .navigation_catalog
            .playlists
            .iter()
            .any(|document| document.playlist.id.eq_ignore_ascii_case(&playlist_id))
        {
            self.as_mut().set_status_message(qstring(format!(
                "Playlist is no longer available: {playlist_id}"
            )));
            return;
        }
        let Some(root) = self.as_ref().rust().launchbox_root.clone() else {
            self.as_mut().set_status_message(qstring(
                "Playlist deletion requires a loaded LaunchBox directory, not a standalone XML file.",
            ));
            return;
        };
        let navigation_catalog = self.as_ref().rust().navigation_catalog.clone();
        let generation = self.as_ref().rust().request_generation;
        self.as_mut().set_writing(true);
        self.as_mut()
            .set_status_message(qstring(format!("Deleting playlist {playlist_id}...")));
        let qt_thread = self.as_ref().qt_thread();
        let spawn_result = std::thread::Builder::new()
            .name("launchbox-playlist-delete".to_string())
            .spawn(move || {
                let result = delete_playlist_from_library(root, playlist_id, navigation_catalog);
                qt_thread
                    .queue(move |mut controller| {
                        controller.as_mut().finish_playlist_write(
                            generation,
                            PlaylistWriteOperation::Delete,
                            result,
                        );
                    })
                    .ok();
            });
        if let Err(error) = spawn_result {
            self.as_mut().set_writing(false);
            self.as_mut().set_status_message(qstring(format!(
                "Could not start playlist deletion: {error}"
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

    pub fn report_additional_application_crud_smoke_success(&self, game_id: QString) -> bool {
        let game_id = game_id.to_string();
        let rust = self.rust();
        let application = rust
            .additional_applications_by_game
            .get(&game_id)
            .and_then(|applications| applications.as_slice().first());
        let success = rust
            .additional_applications_by_game
            .get(&game_id)
            .is_some_and(|applications| applications.len() == 1)
            && application.is_some_and(|application| {
                application.id == "fixture-adventure-manual"
                    && application.name == "Edited Fixture Manual"
                    && application.application_path == r"Games\Fixture Adventure\edited-manual.pdf"
                    && application.command_line.as_deref() == Some("--page 3")
                    && application.auto_run_before
                    && !application.auto_run_after
                    && application.wait_for_exit
                    && !application.use_emulator
                    && application.emulator_id.is_none()
                    && !application.use_dos_box
                    && application.priority == 4
                    && application.play_count == 5
                    && application.play_time_seconds == 321
                    && application.disc == Some(2)
                    && application.side_a
                    && !application.side_b
                    && application.developer.as_deref() == Some("Qt Docs")
                    && application.publisher.as_deref() == Some("Port Press")
                    && application.region.as_deref() == Some("Europe")
                    && application.release_date.as_deref() == Some("2005-06-07")
                    && application.version.as_deref() == Some("Rev 3")
                    && application.status.as_deref() == Some("Installed")
                    && application.installed == Some(true)
                    && application.last_played.as_deref()
                        == Some("2026-07-22T13:14:15.0000000-07:00")
            })
            && rust.additional_application_write_notifications == 3
            && *self.additional_application_revision() == 3
            && self.last_added_additional_application_id().is_empty()
            && rust.model_reset_notifications == 1
            && rust.data_change_notifications == 3
            && !*self.writing()
            && !*self.write_conflict()
            && *self.pending_recovery_count() == 0;
        if success {
            eprintln!(
                "ADDITIONAL_APPLICATION_CRUD_SMOKE_COMPLETE writes={} revision={} data_changes={}",
                rust.additional_application_write_notifications,
                self.additional_application_revision(),
                rust.data_change_notifications
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

    pub fn report_category_crud_smoke_success(
        &self,
        category_name: QString,
        detached_children: i32,
    ) -> bool {
        let category_name = category_name.to_string();
        let rust = self.rust();
        let expected_detached = usize::try_from(detached_children).unwrap_or(usize::MAX);
        let success = !rust
            .navigation_catalog
            .categories
            .iter()
            .any(|category| category.metadata.name.eq_ignore_ascii_case(&category_name))
            && rust.category_write_notifications == 3
            && rust.last_category_detached_children == expected_detached
            && !*self.writing()
            && !*self.write_conflict()
            && *self.pending_recovery_count() == 0;
        if success {
            eprintln!(
                "CATEGORY_CRUD_SMOKE_COMPLETE category=\"{category_name}\" writes={} detached={detached_children} navigation_entries={}",
                rust.category_write_notifications,
                rust.navigation_entries.len()
            );
        }
        success
    }

    pub fn report_playlist_crud_smoke_success(
        &self,
        playlist_id: QString,
        detached_children: i32,
        removed_cache_rows: i32,
    ) -> bool {
        let playlist_id = playlist_id.to_string();
        let rust = self.rust();
        let expected_detached = usize::try_from(detached_children).unwrap_or(usize::MAX);
        let expected_cache_rows = usize::try_from(removed_cache_rows).unwrap_or(usize::MAX);
        let success = !rust
            .navigation_catalog
            .playlists
            .iter()
            .any(|document| document.playlist.id.eq_ignore_ascii_case(&playlist_id))
            && rust.playlist_write_notifications == 5
            && rust.last_playlist_detached_children == expected_detached
            && rust.last_playlist_cache_rows_removed == expected_cache_rows
            && !*self.writing()
            && !*self.write_conflict()
            && *self.pending_recovery_count() == 0;
        if success {
            eprintln!(
                "PLAYLIST_CRUD_SMOKE_COMPLETE playlist_id=\"{playlist_id}\" writes={} detached={detached_children} cache_rows={removed_cache_rows} navigation_entries={}",
                rust.playlist_write_notifications,
                rust.navigation_entries.len()
            );
        }
        success
    }

    pub fn report_big_box_navigation_smoke_success(&self) -> bool {
        let rust = self.rust();
        let expected = [
            ("category", "Fixture Category", 0usize, 3usize),
            ("platform", "Fixture Console", 1usize, 3usize),
            ("playlist", "fixture-playlist", 0usize, 1usize),
        ];
        let success = rust.big_box_navigation_entries.len() == expected.len()
            && rust
                .big_box_navigation_entries
                .iter()
                .zip(expected)
                .all(|(entry, expected)| {
                    (
                        entry.kind,
                        entry.key.as_str(),
                        entry.depth,
                        entry.game_count,
                    ) == expected
                })
            && rust.filtered_indices.len() == 3
            && self.navigation_filter_kind().to_string().is_empty()
            && self.navigation_filter_key().to_string().is_empty()
            && !*self.loading()
            && !*self.writing();
        if success {
            eprintln!(
                "BIGBOX_NAVIGATION_SMOKE_COMPLETE entries={} playlist=1 category=3 platform=3",
                rust.big_box_navigation_entries.len()
            );
        }
        success
    }

    pub fn report_import_smoke_success(
        &self,
        expected_count: i32,
        expected_created_files: i32,
        expected_moved_files: i32,
    ) -> bool {
        let rust = self.rust();
        let imported_games_present = rust.last_imported_game_ids.iter().all(|id| {
            rust.games.iter().any(|game| {
                game.id == *id
                    && game.platform == "Fixture Console"
                    && game.database_id == Some(4242)
                    && game.emulator_id.as_deref() == Some("fixture-emulator")
                    && game.version.as_deref() == Some("(USA)")
                    && game.region.as_deref() == Some("North America")
                    && game.status.as_deref() == Some("Imported ROM")
            })
        });
        let imported_groups = rust
            .last_imported_game_ids
            .iter()
            .filter_map(|id| {
                let game = rust.games.iter().find(|game| game.id == *id)?;
                let applications = rust.additional_applications_by_game.get(id)?;
                Some((game, applications))
            })
            .collect::<Vec<_>>();
        let imported_disc_sets = imported_groups
            .iter()
            .filter(|(game, applications)| {
                game.manual_path.as_deref()
                    == Some(
                        r"Games\Fixture Console\Fixture Saga (USA) (2002)\Fixture Sag (USA) - (Disc 1 of 2).pdf",
                    )
                    && applications.len() == 2
                    && applications.iter().enumerate().all(|(index, application)| {
                        let disc = u32::try_from(index).unwrap_or_default() + 1;
                        application.disc == Some(disc)
                            && application.name == format!("Play (USA) Disc {disc}...")
                            && application.priority == i32::try_from(disc).unwrap_or_default()
                            && application.version.as_deref() == Some("(USA)")
                            && application.region.as_deref() == Some("North America")
                            && application.status.as_deref() == Some("Imported ROM")
                            && application.use_emulator
                            && application.emulator_id.as_deref() == Some("fixture-emulator")
                    })
            })
            .count();
        let imported_version_groups = imported_groups
            .iter()
            .filter(|(game, applications)| {
                game.manual_path.is_none()
                    && applications.len() == 2
                    && applications[0].name == "Play (USA) Version..."
                    && applications[0].priority == 1
                    && applications[0].disc.is_none()
                    && applications[0].version.as_deref() == Some("(USA)")
                    && applications[0].region.as_deref() == Some("North America")
                    && applications[0].application_path == game.application_path
                    && applications[1].name == "Play (World) (Rev 1) Version..."
                    && applications[1].priority == 2
                    && applications[1].disc.is_none()
                    && applications[1].version.as_deref() == Some("(World) (Rev 1)")
                    && applications[1].region.as_deref() == Some("World")
                    && applications.iter().all(|application| {
                        application.status.as_deref() == Some("Imported ROM")
                            && application.use_emulator
                            && application.emulator_id.as_deref() == Some("fixture-emulator")
                    })
            })
            .count();
        let success = *self.last_import_count() == expected_count
            && *self.last_import_created_file_count() == expected_created_files
            && *self.last_import_moved_file_count() == expected_moved_files
            && saturating_i32(rust.last_imported_game_ids.len()) == expected_count
            && imported_games_present
            && imported_groups.len() == 2
            && imported_disc_sets == 1
            && imported_version_groups == 1
            && !*self.import_scanning()
            && !*self.loading()
            && !*self.writing()
            && !*self.write_conflict()
            && *self.pending_recovery_count() == 0;
        if success {
            eprintln!(
                "IMPORT_SMOKE_COMPLETE imported={expected_count} created={expected_created_files} moved={expected_moved_files} model_games={}",
                rust.games.len()
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

    pub fn navigation_entry_kind_at(&self, index: i32) -> QString {
        self.navigation_entry_at(index)
            .map(|entry| qstring(entry.kind))
            .unwrap_or_default()
    }

    pub fn navigation_entry_key_at(&self, index: i32) -> QString {
        self.navigation_entry_at(index)
            .map(|entry| qstring(&entry.key))
            .unwrap_or_default()
    }

    pub fn navigation_entry_name_at(&self, index: i32) -> QString {
        self.navigation_entry_at(index)
            .map(|entry| qstring(&entry.name))
            .unwrap_or_default()
    }

    pub fn navigation_entry_depth_at(&self, index: i32) -> i32 {
        self.navigation_entry_at(index)
            .map(|entry| saturating_i32(entry.depth))
            .unwrap_or_default()
    }

    pub fn navigation_entry_game_count_at(&self, index: i32) -> i32 {
        self.navigation_entry_at(index)
            .map(|entry| saturating_i32(entry.game_count))
            .unwrap_or_default()
    }

    pub fn big_box_navigation_entry_kind_at(&self, index: i32) -> QString {
        self.big_box_navigation_entry_at(index)
            .map(|entry| qstring(entry.kind))
            .unwrap_or_default()
    }

    pub fn big_box_navigation_entry_key_at(&self, index: i32) -> QString {
        self.big_box_navigation_entry_at(index)
            .map(|entry| qstring(&entry.key))
            .unwrap_or_default()
    }

    pub fn big_box_navigation_entry_name_at(&self, index: i32) -> QString {
        self.big_box_navigation_entry_at(index)
            .map(|entry| qstring(&entry.name))
            .unwrap_or_default()
    }

    pub fn big_box_navigation_entry_depth_at(&self, index: i32) -> i32 {
        self.big_box_navigation_entry_at(index)
            .map(|entry| saturating_i32(entry.depth))
            .unwrap_or_default()
    }

    pub fn big_box_navigation_entry_game_count_at(&self, index: i32) -> i32 {
        self.big_box_navigation_entry_at(index)
            .map(|entry| saturating_i32(entry.game_count))
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

    pub fn new_additional_application_edit_payload(&self, row: i32, game_id: QString) -> QString {
        let next_priority = self
            .additional_applications_for_model(row, &game_id.to_string())
            .and_then(|applications| {
                applications
                    .iter()
                    .map(|application| application.priority)
                    .max()
            })
            .unwrap_or(-1)
            .saturating_add(1)
            .max(0);
        let payload = AdditionalApplicationEditPayload {
            version: ADDITIONAL_APPLICATION_EDIT_PAYLOAD_VERSION,
            application: AdditionalApplicationEdit {
                priority: next_priority,
                ..AdditionalApplicationEdit::default()
            },
        };
        serde_json::to_string(&payload)
            .map(qstring)
            .unwrap_or_default()
    }

    pub fn additional_application_edit_payload(
        &self,
        row: i32,
        game_id: QString,
        application_id: QString,
    ) -> QString {
        let game_id = game_id.to_string();
        let application_id = application_id.to_string();
        let Some(application) = self
            .additional_applications_for_model(row, &game_id)
            .and_then(|applications| {
                applications
                    .iter()
                    .find(|application| application.id == application_id)
            })
        else {
            return QString::default();
        };
        let payload = AdditionalApplicationEditPayload {
            version: ADDITIONAL_APPLICATION_EDIT_PAYLOAD_VERSION,
            application: AdditionalApplicationEdit::from(application),
        };
        serde_json::to_string(&payload)
            .map(qstring)
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
        if *self.as_ref().loading()
            || *self.as_ref().import_scanning()
            || *self.as_ref().writing()
            || *self.as_ref().launching()
        {
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
                    navigation_catalog: loaded.navigation_catalog,
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

    fn finish_rom_import_preview(
        mut self: Pin<&mut Self>,
        generation: u64,
        result: Result<(String, usize), String>,
    ) {
        self.as_mut().set_import_scanning(false);
        if self.as_ref().rust().request_generation != generation {
            return;
        }
        match result {
            Ok((json, count)) => {
                self.as_mut().set_import_preview_json(qstring(json));
                self.as_mut().set_status_message(qstring(format!(
                    "ROM import preview ready: {count} planned game(s) selected for import."
                )));
            }
            Err(error) => {
                self.as_mut().set_import_preview_json(QString::default());
                self.as_mut()
                    .set_status_message(qstring(format!("Could not preview ROM import: {error}")));
            }
        }
    }

    fn finish_rom_import(
        mut self: Pin<&mut Self>,
        generation: u64,
        result: Result<RomImportSuccess, ImportError>,
    ) {
        self.as_mut().set_writing(false);
        if self.as_ref().rust().request_generation != generation {
            return;
        }
        match result {
            Ok(imported) => {
                let RomImportSuccess { report, source } = imported;
                let count = report.games.len();
                let created_count = report.created_files.len();
                let moved_count = report.moved_sources.len();
                let game_ids = report
                    .games
                    .iter()
                    .map(|game| game.id.clone())
                    .collect::<Vec<_>>();
                let last_id = game_ids.last().cloned().unwrap_or_default();
                {
                    let mut rust = self.as_mut().rust_mut();
                    rust.games.extend(report.games);
                    rust.game_sources.extend(std::iter::repeat_n(source, count));
                    for application in report.additional_applications {
                        rust.additional_applications_by_game
                            .entry(application.game_id.clone())
                            .or_default()
                            .push(application);
                    }
                    for applications in rust.additional_applications_by_game.values_mut() {
                        applications.sort_by(|left, right| {
                            left.priority
                                .cmp(&right.priority)
                                .then_with(|| left.id.cmp(&right.id))
                        });
                    }
                    rust.last_imported_game_ids = game_ids;
                }
                self.as_mut().refresh_filtered_games();
                self.as_mut().update_library_counts();
                self.as_mut().set_write_conflict(false);
                self.as_mut().set_last_added_game_id(qstring(last_id));
                self.as_mut().set_last_import_count(saturating_i32(count));
                self.as_mut()
                    .set_last_import_created_file_count(saturating_i32(created_count));
                self.as_mut()
                    .set_last_import_moved_file_count(saturating_i32(moved_count));
                self.as_mut().set_import_preview_json(QString::default());
                let mut message = format!(
                    "Imported {count} game(s). Exact platform backup: {}",
                    report.platform_backup.display()
                );
                if created_count > 0 {
                    message.push_str(&format!(" Created {created_count} library file(s)."));
                }
                if moved_count > 0 {
                    message.push_str(&format!(" Removed {moved_count} verified source file(s)."));
                }
                if !report.cleanup_warnings.is_empty() {
                    message.push_str(&format!(
                        " Cleanup warning(s): {}",
                        report.cleanup_warnings.join("; ")
                    ));
                }
                self.as_mut().set_status_message(qstring(message));
            }
            Err(ImportError::Transaction(TransactionError::Conflict { path, .. })) => {
                self.as_mut().set_write_conflict(true);
                self.as_mut().set_status_message(qstring(format!(
                    "Write conflict while importing ROMs at {}. Reload and preview again.",
                    path.display()
                )));
            }
            Err(ImportError::Transaction(TransactionError::PendingRecovery {
                manifests, ..
            })) => {
                self.as_mut()
                    .set_pending_recovery_count(saturating_i32(manifests.len()));
                self.as_mut().set_status_message(qstring(
                    "An interrupted transaction requires recovery before ROM import can continue.",
                ));
            }
            Err(error) => {
                self.as_mut()
                    .set_status_message(qstring(format!("Could not import ROMs: {error}")));
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

    fn finish_additional_application_write(
        mut self: Pin<&mut Self>,
        generation: u64,
        result: Result<AdditionalApplicationWriteSuccess, GameWriteFailure>,
    ) {
        self.as_mut().set_writing(false);
        if self.as_ref().rust().request_generation != generation {
            return;
        }
        match result {
            Ok(success) => {
                let AdditionalApplicationWriteSuccess {
                    operation,
                    application,
                    source,
                    backup,
                } = success;
                let game_id = application.game_id.clone();
                let actual_index = self
                    .as_ref()
                    .rust()
                    .games
                    .iter()
                    .zip(&self.as_ref().rust().game_sources)
                    .position(|(game, candidate_source)| {
                        game.id == game_id && *candidate_source == source
                    });
                let Some(actual_index) = actual_index else {
                    self.as_mut().set_status_message(qstring(
                        "The edited additional application's game is no longer present in the loaded model; reload required.",
                    ));
                    return;
                };

                match operation {
                    AdditionalApplicationWriteOperation::Create => {
                        self.as_mut()
                            .rust_mut()
                            .additional_applications_by_game
                            .entry(game_id.clone())
                            .or_default()
                            .push(application.clone());
                        self.as_mut()
                            .set_last_added_additional_application_id(qstring(&application.id));
                    }
                    AdditionalApplicationWriteOperation::Edit => {
                        let updated = {
                            let mut rust = self.as_mut().rust_mut();
                            rust.additional_applications_by_game
                                .get_mut(&game_id)
                                .and_then(|applications| {
                                    applications
                                        .iter_mut()
                                        .find(|candidate| candidate.id == application.id)
                                })
                                .map(|existing| *existing = application.clone())
                                .is_some()
                        };
                        if !updated {
                            self.as_mut().set_status_message(qstring(
                                "The edited additional application is no longer present in the loaded model; reload required.",
                            ));
                            return;
                        }
                    }
                    AdditionalApplicationWriteOperation::Delete => {
                        let deleted = {
                            let mut rust = self.as_mut().rust_mut();
                            rust.additional_applications_by_game
                                .get_mut(&game_id)
                                .map(|applications| {
                                    let previous_count = applications.len();
                                    applications.retain(|candidate| candidate.id != application.id);
                                    applications.len() != previous_count
                                })
                                .unwrap_or(false)
                        };
                        if !deleted {
                            self.as_mut().set_status_message(qstring(
                                "The deleted additional application is no longer present in the loaded model; reload required.",
                            ));
                            return;
                        }
                        if self
                            .as_ref()
                            .last_added_additional_application_id()
                            .to_string()
                            == application.id
                        {
                            self.as_mut()
                                .set_last_added_additional_application_id(QString::default());
                        }
                    }
                }
                let remove_empty_group = self
                    .as_ref()
                    .rust()
                    .additional_applications_by_game
                    .get(&game_id)
                    .is_some_and(Vec::is_empty);
                if remove_empty_group {
                    self.as_mut()
                        .rust_mut()
                        .additional_applications_by_game
                        .remove(&game_id);
                } else if let Some(applications) = self
                    .as_mut()
                    .rust_mut()
                    .additional_applications_by_game
                    .get_mut(&game_id)
                {
                    applications.sort_by(|left, right| {
                        left.priority
                            .cmp(&right.priority)
                            .then_with(|| left.id.cmp(&right.id))
                    });
                }

                let revision = self
                    .as_ref()
                    .additional_application_revision()
                    .saturating_add(1);
                self.as_mut().set_additional_application_revision(revision);
                self.as_mut()
                    .rust_mut()
                    .additional_application_write_notifications = self
                    .as_ref()
                    .rust()
                    .additional_application_write_notifications
                    .saturating_add(1);
                if let Some(filtered_row) = self
                    .as_ref()
                    .rust()
                    .filtered_indices
                    .iter()
                    .position(|index| *index == actual_index)
                {
                    let row = saturating_i32(filtered_row);
                    let parent = QModelIndex::default();
                    let index = self.as_ref().model_index(row, 0, &parent);
                    let mut roles = QList::<i32>::default();
                    roles.append(GAME_ADDITIONAL_APPLICATION_COUNT_ROLE);
                    self.as_mut().rust_mut().data_change_notifications = self
                        .as_ref()
                        .rust()
                        .data_change_notifications
                        .saturating_add(1);
                    self.as_mut().emit_data_changed(&index, &index, &roles);
                }
                self.as_mut().set_write_conflict(false);
                let verb = match operation {
                    AdditionalApplicationWriteOperation::Create => "Added",
                    AdditionalApplicationWriteOperation::Edit => "Saved",
                    AdditionalApplicationWriteOperation::Delete => "Deleted",
                };
                self.as_mut().set_status_message(qstring(format!(
                    "{verb} {}. Exact backup: {}",
                    application.name,
                    backup.display()
                )));
            }
            Err(GameWriteFailure::Conflict(message)) => {
                self.as_mut().set_write_conflict(true);
                self.as_mut().set_status_message(qstring(format!(
                    "Write conflict while changing additional application: {message}. Reload before retrying."
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
                    "Could not change additional application: {} unexpected dependent records were reported.",
                    references.len()
                )));
            }
            Err(GameWriteFailure::Other(message)) => self.as_mut().set_status_message(qstring(
                format!("Could not change additional application: {message}"),
            )),
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
                    rust.navigation_catalog
                        .platforms
                        .push(created.platform.clone());
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

    fn finish_platform_edit(
        mut self: Pin<&mut Self>,
        generation: u64,
        result: Result<PlatformEditSuccess, PlatformWriteFailure>,
    ) {
        self.as_mut().set_writing(false);
        if self.as_ref().rust().request_generation != generation {
            return;
        }
        match result {
            Ok(edited) => {
                if let Some(platform) = self
                    .as_mut()
                    .rust_mut()
                    .navigation_catalog
                    .platforms
                    .iter_mut()
                    .find(|platform| platform.metadata.name.eq_ignore_ascii_case(&edited.name))
                {
                    *platform = edited.platform.clone();
                }
                self.as_mut().update_library_counts();
                self.as_mut().set_write_conflict(false);
                self.as_mut().set_status_message(qstring(format!(
                    "Saved {} and {} media-folder records. Exact catalog backup: {}",
                    edited.name,
                    edited.folder_count,
                    edited.catalog_backup.display()
                )));
            }
            Err(PlatformWriteFailure::Conflict(message)) => {
                self.as_mut().set_write_conflict(true);
                self.as_mut().set_status_message(qstring(format!(
                    "Write conflict while editing platform: {message}. Reload before retrying."
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
                    "Could not edit platform: unexpected reference result ({})",
                    references.len()
                )))
            }
            Err(PlatformWriteFailure::Other(message)) => self
                .as_mut()
                .set_status_message(qstring(format!("Could not edit platform: {message}"))),
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
                    rust.navigation_catalog.platforms.retain(|platform| {
                        !platform.metadata.name.eq_ignore_ascii_case(&deleted.name)
                    });
                }
                if self
                    .as_ref()
                    .platform_filter()
                    .to_string()
                    .eq_ignore_ascii_case(&deleted.name)
                {
                    self.as_mut().set_platform_filter(QString::default());
                    self.as_mut().set_navigation_filter_kind(QString::default());
                    self.as_mut().set_navigation_filter_key(QString::default());
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

    fn finish_category_write(
        mut self: Pin<&mut Self>,
        generation: u64,
        operation: CategoryWriteOperation,
        result: Result<CategoryWriteSuccess, PlatformWriteFailure>,
    ) {
        self.as_mut().set_writing(false);
        if self.as_ref().rust().request_generation != generation {
            return;
        }
        let operation_label = match operation {
            CategoryWriteOperation::Create => "create",
            CategoryWriteOperation::Edit => "edit",
            CategoryWriteOperation::Delete => "delete",
        };
        match result {
            Ok(written) => {
                let CategoryWriteSuccess {
                    name,
                    categories,
                    parents,
                    catalog_backup,
                    parents_backup,
                    placement_count,
                    removed_placements,
                    detached_children,
                } = written;
                let deleting_selected_filter = matches!(operation, CategoryWriteOperation::Delete)
                    && self
                        .as_ref()
                        .rust()
                        .category_filter
                        .as_deref()
                        .is_some_and(|selected| selected == name.to_lowercase());
                {
                    let mut rust = self.as_mut().rust_mut();
                    rust.navigation_catalog.categories = categories;
                    rust.navigation_catalog.parents = parents;
                    if deleting_selected_filter {
                        rust.category_filter = None;
                    }
                    rust.category_write_notifications =
                        rust.category_write_notifications.saturating_add(1);
                    rust.last_category_detached_children = detached_children;
                }
                self.as_mut().update_library_counts();
                if deleting_selected_filter {
                    self.as_mut().set_navigation_filter_kind(QString::default());
                    self.as_mut().set_navigation_filter_key(QString::default());
                }
                if self.as_ref().rust().category_filter.is_some() || deleting_selected_filter {
                    self.as_mut().refresh_filtered_games();
                }
                self.as_mut().set_write_conflict(false);
                let detail = match operation {
                    CategoryWriteOperation::Create => format!(
                        "Created platform category {name} with {placement_count} hierarchy placement(s)."
                    ),
                    CategoryWriteOperation::Edit => format!(
                        "Saved platform category {name} with {placement_count} hierarchy placement(s)."
                    ),
                    CategoryWriteOperation::Delete => format!(
                        "Deleted platform category {name}, removed {removed_placements} placement(s), and detached {detached_children} direct child placement(s) to root. No platforms, playlists, games, or media were deleted."
                    ),
                };
                self.as_mut().set_status_message(qstring(format!(
                    "{detail} Exact catalog backup: {}. Exact hierarchy backup: {}",
                    catalog_backup.display(),
                    parents_backup.display()
                )));
            }
            Err(PlatformWriteFailure::Conflict(message)) => {
                self.as_mut().set_write_conflict(true);
                self.as_mut().set_status_message(qstring(format!(
                    "Write conflict during platform category {operation_label}: {message}. Reload before retrying."
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
                    "Could not {operation_label} platform category: unexpected reference result ({})",
                    references.len()
                )));
            }
            Err(PlatformWriteFailure::Other(message)) => self.as_mut().set_status_message(qstring(
                format!("Could not {operation_label} platform category: {message}"),
            )),
        }
    }

    fn finish_playlist_write(
        mut self: Pin<&mut Self>,
        generation: u64,
        operation: PlaylistWriteOperation,
        result: Result<PlaylistWriteSuccess, PlatformWriteFailure>,
    ) {
        self.as_mut().set_writing(false);
        if self.as_ref().rust().request_generation != generation {
            return;
        }
        let operation_label = match operation {
            PlaylistWriteOperation::Create => "create",
            PlaylistWriteOperation::Edit => "edit",
            PlaylistWriteOperation::Delete => "delete",
        };
        match result {
            Ok(written) => {
                let PlaylistWriteSuccess {
                    id,
                    playlists,
                    parents,
                    source,
                    playlist_backup,
                    parents_backup,
                    list_cache_backup,
                    placement_count,
                    removed_placements,
                    detached_children,
                    removed_cache_rows,
                } = written;
                let deleting_selected_filter = matches!(operation, PlaylistWriteOperation::Delete)
                    && self
                        .as_ref()
                        .rust()
                        .playlist_filter
                        .as_deref()
                        .is_some_and(|selected| selected == id.to_lowercase());
                {
                    let mut rust = self.as_mut().rust_mut();
                    rust.navigation_catalog.playlists = playlists;
                    rust.navigation_catalog.parents = parents;
                    if deleting_selected_filter {
                        rust.playlist_filter = None;
                    }
                    rust.playlist_write_notifications =
                        rust.playlist_write_notifications.saturating_add(1);
                    rust.last_playlist_detached_children = rust
                        .last_playlist_detached_children
                        .saturating_add(detached_children);
                    rust.last_playlist_cache_rows_removed = rust
                        .last_playlist_cache_rows_removed
                        .saturating_add(removed_cache_rows);
                }
                self.as_mut().update_library_counts();
                if deleting_selected_filter {
                    self.as_mut().set_navigation_filter_kind(QString::default());
                    self.as_mut().set_navigation_filter_key(QString::default());
                }
                if self.as_ref().rust().playlist_filter.is_some() || deleting_selected_filter {
                    self.as_mut().refresh_filtered_games();
                }
                self.as_mut().set_write_conflict(false);
                let detail = match operation {
                    PlaylistWriteOperation::Create => format!(
                        "Created playlist {id} with {placement_count} hierarchy placement(s) at {}.",
                        source.display()
                    ),
                    PlaylistWriteOperation::Edit => format!(
                        "Saved playlist {id} with {placement_count} hierarchy placement(s)."
                    ),
                    PlaylistWriteOperation::Delete => format!(
                        "Deleted all instances of playlist {id}, removed {removed_placements} placement(s), detached {detached_children} direct child placement(s) to root, and removed {removed_cache_rows} list-cache row(s). No games or media were deleted."
                    ),
                };
                let playlist_backup = playlist_backup
                    .map(|path| format!(" Exact playlist backup: {}.", path.display()))
                    .unwrap_or_default();
                let cache_backup = list_cache_backup
                    .map(|path| format!(" Exact list-cache backup: {}.", path.display()))
                    .unwrap_or_default();
                self.as_mut().set_status_message(qstring(format!(
                    "{detail}{playlist_backup} Exact hierarchy backup: {}.{cache_backup}",
                    parents_backup.display()
                )));
            }
            Err(PlatformWriteFailure::Conflict(message)) => {
                self.as_mut().set_write_conflict(true);
                self.as_mut().set_status_message(qstring(format!(
                    "Write conflict during playlist {operation_label}: {message}. Reload before retrying."
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
                    "Could not {operation_label} playlist: unexpected platform reference result ({})",
                    references.len()
                )));
            }
            Err(PlatformWriteFailure::Other(message)) => self.as_mut().set_status_message(qstring(
                format!("Could not {operation_label} playlist: {message}"),
            )),
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
        if let Some(category) = self.rust().category_filter.as_deref() {
            let visible = self
                .rust()
                .category_game_ids
                .get(category)
                .is_some_and(|ids| ids.contains(&game.id))
                || self
                    .rust()
                    .category_platforms
                    .get(category)
                    .is_some_and(|platforms| platforms.contains(&platform_key(&game.platform)));
            if !visible {
                return None;
            }
        }
        if let Some(playlist_id) = self.rust().playlist_filter.as_deref() {
            let visible = self
                .rust()
                .navigation_catalog
                .playlists
                .iter()
                .find(|document| document.playlist.id.eq_ignore_ascii_case(playlist_id))
                .is_some_and(|document| {
                    document.playlist.auto_populate
                        && auto_playlist_matches(game, &document.filters)
                });
            if !visible {
                return None;
            }
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
        let (
            game_count,
            filtered_count,
            platform_counts,
            navigation_entries,
            category_platforms,
            category_game_ids,
            playlist_game_ids,
        ) = {
            let this = self.as_ref();
            let rust = this.rust();
            let (navigation_entries, category_platforms, category_game_ids, playlist_game_ids) =
                build_navigation_entries(
                    &rust.navigation_catalog,
                    &rust.platform_names,
                    &rust.games,
                );
            (
                saturating_i32(rust.games.len()),
                saturating_i32(rust.filtered_indices.len()),
                collect_platform_counts(&rust.games, &rust.platform_names),
                navigation_entries,
                category_platforms,
                category_game_ids,
                playlist_game_ids,
            )
        };
        let platform_entry_count = saturating_i32(platform_counts.len());
        let navigation_entry_count = saturating_i32(navigation_entries.len());
        let big_box_navigation_entries = build_big_box_navigation_entries(&navigation_entries);
        let big_box_navigation_entry_count = saturating_i32(big_box_navigation_entries.len());
        {
            let mut rust = self.as_mut().rust_mut();
            rust.platform_counts = platform_counts;
            rust.navigation_entries = navigation_entries;
            rust.big_box_navigation_entries = big_box_navigation_entries;
            rust.category_platforms = category_platforms;
            rust.category_game_ids = category_game_ids;
            rust.playlist_game_ids = playlist_game_ids;
        }
        self.as_mut().set_game_count(game_count);
        self.as_mut().set_filtered_count(filtered_count);
        self.as_mut().set_platform_entry_count(platform_entry_count);
        self.as_mut()
            .set_navigation_entry_count(navigation_entry_count);
        self.as_mut()
            .set_big_box_navigation_entry_count(big_box_navigation_entry_count);
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
            navigation_catalog,
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
        let (navigation_entries, category_platforms, category_game_ids, playlist_game_ids) =
            build_navigation_entries(&navigation_catalog, &platform_names, &games);
        let navigation_entry_count = saturating_i32(navigation_entries.len());
        let big_box_navigation_entries = build_big_box_navigation_entries(&navigation_entries);
        let big_box_navigation_entry_count = saturating_i32(big_box_navigation_entries.len());
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
            rust.navigation_catalog = navigation_catalog;
            rust.navigation_entries = navigation_entries;
            rust.big_box_navigation_entries = big_box_navigation_entries;
            rust.category_platforms = category_platforms;
            rust.category_game_ids = category_game_ids;
            rust.playlist_game_ids = playlist_game_ids;
            rust.category_filter = None;
            rust.playlist_filter = None;
            rust.library_root = library_root;
            rust.launchbox_root = launchbox_root;
            rust.emulator_configuration = emulator_configuration;
            rust.model_reset_notifications = 1;
            rust.data_change_notifications = 0;
            rust.row_insert_notifications = 0;
            rust.row_remove_notifications = 0;
            rust.launch_notifications = 0;
            rust.additional_application_write_notifications = 0;
            rust.category_write_notifications = 0;
            rust.last_category_detached_children = 0;
            rust.playlist_write_notifications = 0;
            rust.last_playlist_detached_children = 0;
            rust.last_playlist_cache_rows_removed = 0;
        }
        self.as_mut().end_reset_model();
        self.as_mut().set_library_name(qstring(name));
        self.as_mut().set_status_message(qstring(message));
        self.as_mut().set_game_count(game_count);
        self.as_mut().set_filtered_count(game_count);
        self.as_mut().set_platform_entry_count(platform_entry_count);
        self.as_mut()
            .set_navigation_entry_count(navigation_entry_count);
        self.as_mut()
            .set_big_box_navigation_entry_count(big_box_navigation_entry_count);
        let revision = self.as_ref().rust().platform_revision.wrapping_add(1);
        self.as_mut().set_platform_revision(revision);
        self.as_mut()
            .set_pending_recovery_count(saturating_i32(pending_recovery_count));
        self.as_mut().set_delete_blocker_count(0);
        self.as_mut().set_delete_blocker_summary(QString::default());
        self.as_mut().set_last_added_game_id(QString::default());
        self.as_mut().set_import_scanning(false);
        self.as_mut().set_import_preview_json(QString::default());
        self.as_mut().set_last_import_count(0);
        self.as_mut().set_last_import_created_file_count(0);
        self.as_mut().set_last_import_moved_file_count(0);
        self.as_mut().rust_mut().last_imported_game_ids.clear();
        self.as_mut().set_launching(false);
        self.as_mut().set_last_launch_succeeded(false);
        self.as_mut().set_last_launch_game_id(QString::default());
        self.as_mut().set_last_launch_target_id(QString::default());
        self.as_mut().set_launch_session_active(false);
        self.as_mut().set_search_text(QString::default());
        self.as_mut().set_platform_filter(QString::default());
        self.as_mut().set_navigation_filter_kind(QString::default());
        self.as_mut().set_navigation_filter_key(QString::default());
    }

    fn refresh_filtered_games(mut self: Pin<&mut Self>) {
        let search_text = self.as_ref().search_text().to_string();
        let platform = self.as_ref().platform_filter().to_string();
        let category_filter = self.as_ref().rust().category_filter.clone();
        let playlist_filter = self.as_ref().rust().playlist_filter.clone();
        let indices = {
            let this = self.as_ref();
            let rust = this.rust();
            let filter = GameFilter {
                text: search_text,
                platform: (!platform.is_empty()).then_some(platform),
                ..GameFilter::default()
            };
            let mut indices = filter_game_indices(&rust.games, &filter);
            if let Some(category) = category_filter.as_deref() {
                if let Some(ids) = rust.category_game_ids.get(category) {
                    indices.retain(|index| ids.contains(&rust.games[*index].id));
                } else {
                    indices.clear();
                }
            }
            if let Some(playlist) = playlist_filter.as_deref() {
                if let Some(ids) = rust.playlist_game_ids.get(playlist) {
                    indices.retain(|index| ids.contains(&rust.games[*index].id));
                } else {
                    indices.clear();
                }
            }
            indices
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

    fn navigation_entry_at(&self, index: i32) -> Option<&NavigationEntry> {
        usize::try_from(index)
            .ok()
            .and_then(|index| self.rust().navigation_entries.get(index))
    }

    fn big_box_navigation_entry_at(&self, index: i32) -> Option<&NavigationEntry> {
        usize::try_from(index)
            .ok()
            .and_then(|index| self.rust().big_box_navigation_entries.get(index))
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

#[derive(Clone, Debug)]
struct NavigationNodeInfo {
    kind: &'static str,
    key: String,
    name: String,
    sort_key: String,
    visible_in_big_box: bool,
}

type NavigationBuildResult = (
    Vec<NavigationEntry>,
    BTreeMap<String, BTreeSet<String>>,
    BTreeMap<String, BTreeSet<String>>,
    BTreeMap<String, BTreeSet<String>>,
);

fn build_navigation_entries(
    catalog: &NavigationCatalog,
    platform_names: &[String],
    games: &[Game],
) -> NavigationBuildResult {
    let mut nodes = BTreeMap::<NavigationNodeKey, NavigationNodeInfo>::new();
    for category in &catalog.categories {
        let key = category.metadata.name.to_lowercase();
        nodes.insert(
            NavigationNodeKey::Category(key),
            NavigationNodeInfo {
                kind: "category",
                key: category.metadata.name.clone(),
                name: category
                    .metadata
                    .nested_name
                    .as_deref()
                    .filter(|name| !name.trim().is_empty())
                    .unwrap_or(&category.metadata.name)
                    .to_string(),
                sort_key: category
                    .metadata
                    .sort_title
                    .as_deref()
                    .filter(|title| !title.trim().is_empty())
                    .unwrap_or(&category.metadata.name)
                    .to_lowercase(),
                visible_in_big_box: !category.metadata.hide_in_big_box,
            },
        );
    }
    for platform in platform_names {
        let visible_in_big_box = catalog
            .platforms
            .iter()
            .find(|definition| definition.metadata.name.eq_ignore_ascii_case(platform))
            .is_none_or(|definition| !definition.metadata.hide_in_big_box);
        nodes.insert(
            NavigationNodeKey::Platform(platform_key(platform)),
            NavigationNodeInfo {
                kind: "platform",
                key: platform.clone(),
                name: platform.clone(),
                sort_key: platform_key(platform),
                visible_in_big_box,
            },
        );
    }
    for document in &catalog.playlists {
        let playlist = &document.playlist;
        let key = playlist.id.to_lowercase();
        nodes.insert(
            NavigationNodeKey::Playlist(key),
            NavigationNodeInfo {
                kind: "playlist",
                key: playlist.id.clone(),
                name: playlist
                    .metadata
                    .nested_name
                    .as_deref()
                    .filter(|name| !name.trim().is_empty())
                    .unwrap_or(&playlist.metadata.name)
                    .to_string(),
                sort_key: playlist
                    .metadata
                    .sort_title
                    .as_deref()
                    .filter(|title| !title.trim().is_empty())
                    .unwrap_or(&playlist.metadata.name)
                    .to_lowercase(),
                visible_in_big_box: !playlist.metadata.hide_in_big_box,
            },
        );
    }

    let mut placements = BTreeMap::<NavigationNodeKey, Vec<Option<NavigationNodeKey>>>::new();
    for relationship in &catalog.parents {
        let child = relationship_child_key(relationship);
        let Some(child) = child.filter(|child| nodes.contains_key(child)) else {
            continue;
        };
        let parent =
            relationship_parent_key(relationship).filter(|parent| nodes.contains_key(parent));
        placements.entry(child).or_default().push(parent);
    }

    let mut roots = BTreeSet::new();
    let mut children = BTreeMap::<NavigationNodeKey, BTreeSet<NavigationNodeKey>>::new();
    for node in nodes.keys() {
        match placements.get(node) {
            None => {
                roots.insert(node.clone());
            }
            Some(node_placements) => {
                for parent in node_placements {
                    if let Some(parent) = parent {
                        children
                            .entry(parent.clone())
                            .or_default()
                            .insert(node.clone());
                    } else {
                        roots.insert(node.clone());
                    }
                }
            }
        }
    }

    let mut category_platforms = BTreeMap::new();
    for node in nodes.keys() {
        let NavigationNodeKey::Category(category) = node else {
            continue;
        };
        let mut platforms = BTreeSet::new();
        let mut path = BTreeSet::new();
        collect_descendant_platforms(node, &children, &mut path, &mut platforms);
        category_platforms.insert(category.clone(), platforms);
    }
    let mut platform_game_counts = BTreeMap::<String, usize>::new();
    for game in games {
        *platform_game_counts
            .entry(platform_key(&game.platform))
            .or_default() += 1;
    }
    let playlist_game_ids = catalog
        .playlists
        .iter()
        .map(|document| {
            let ids = if document.playlist.auto_populate {
                games
                    .iter()
                    .filter(|game| auto_playlist_matches(game, &document.filters))
                    .map(|game| game.id.clone())
                    .collect()
            } else {
                document
                    .games
                    .iter()
                    .map(|game| game.game_id.clone())
                    .collect()
            };
            (document.playlist.id.to_lowercase(), ids)
        })
        .collect::<BTreeMap<String, BTreeSet<String>>>();
    let mut category_game_ids = BTreeMap::new();
    for node in nodes.keys() {
        let NavigationNodeKey::Category(category) = node else {
            continue;
        };
        let mut ids = BTreeSet::new();
        let mut path = BTreeSet::new();
        collect_descendant_game_ids(
            node,
            &children,
            games,
            &playlist_game_ids,
            &mut path,
            &mut ids,
        );
        category_game_ids.insert(category.clone(), ids);
    }

    let mut entries = Vec::new();
    let mut rendered = BTreeSet::new();
    let mut sorted_roots = roots.into_iter().collect::<Vec<_>>();
    sort_navigation_keys(&mut sorted_roots, &nodes);
    for root in sorted_roots {
        let mut path = BTreeSet::new();
        flatten_navigation_node(
            &root,
            0,
            &nodes,
            &children,
            &category_game_ids,
            &playlist_game_ids,
            &platform_game_counts,
            &mut path,
            &mut rendered,
            &mut entries,
        );
    }
    let mut orphaned = nodes
        .keys()
        .filter(|node| !rendered.contains(*node))
        .cloned()
        .collect::<Vec<_>>();
    sort_navigation_keys(&mut orphaned, &nodes);
    for root in orphaned {
        let mut path = BTreeSet::new();
        flatten_navigation_node(
            &root,
            0,
            &nodes,
            &children,
            &category_game_ids,
            &playlist_game_ids,
            &platform_game_counts,
            &mut path,
            &mut rendered,
            &mut entries,
        );
    }
    (
        entries,
        category_platforms,
        category_game_ids,
        playlist_game_ids,
    )
}

fn build_big_box_navigation_entries(entries: &[NavigationEntry]) -> Vec<NavigationEntry> {
    let mut hidden_by_depth = Vec::<bool>::new();
    let mut visible = Vec::new();
    for entry in entries {
        hidden_by_depth.truncate(entry.depth);
        let hidden_ancestors = hidden_by_depth.iter().filter(|hidden| **hidden).count();
        hidden_by_depth.push(!entry.visible_in_big_box);
        if entry.visible_in_big_box {
            let mut entry = entry.clone();
            entry.depth = entry.depth.saturating_sub(hidden_ancestors);
            visible.push(entry);
        }
    }
    visible
}

fn relationship_child_key(relationship: &ParentRelationship) -> Option<NavigationNodeKey> {
    if let Some(name) = relationship
        .platform_category_name
        .as_deref()
        .filter(|name| !name.trim().is_empty())
    {
        Some(NavigationNodeKey::Category(name.to_lowercase()))
    } else if let Some(name) = relationship
        .platform_name
        .as_deref()
        .filter(|name| !name.trim().is_empty())
    {
        Some(NavigationNodeKey::Platform(platform_key(name)))
    } else {
        relationship
            .playlist_id
            .as_deref()
            .filter(|id| !id.trim().is_empty())
            .map(|id| NavigationNodeKey::Playlist(id.to_lowercase()))
    }
}

fn relationship_parent_key(relationship: &ParentRelationship) -> Option<NavigationNodeKey> {
    if let Some(name) = relationship
        .parent_platform_category_name
        .as_deref()
        .filter(|name| !name.trim().is_empty())
    {
        Some(NavigationNodeKey::Category(name.to_lowercase()))
    } else if let Some(name) = relationship
        .parent_platform_name
        .as_deref()
        .filter(|name| !name.trim().is_empty())
    {
        Some(NavigationNodeKey::Platform(platform_key(name)))
    } else {
        relationship
            .parent_playlist_id
            .as_deref()
            .filter(|id| !id.trim().is_empty())
            .map(|id| NavigationNodeKey::Playlist(id.to_lowercase()))
    }
}

fn collect_descendant_platforms(
    node: &NavigationNodeKey,
    children: &BTreeMap<NavigationNodeKey, BTreeSet<NavigationNodeKey>>,
    path: &mut BTreeSet<NavigationNodeKey>,
    platforms: &mut BTreeSet<String>,
) {
    if !path.insert(node.clone()) {
        return;
    }
    if let Some(node_children) = children.get(node) {
        for child in node_children {
            match child {
                NavigationNodeKey::Platform(name) => {
                    platforms.insert(name.clone());
                }
                NavigationNodeKey::Category(_) => {
                    collect_descendant_platforms(child, children, path, platforms);
                }
                NavigationNodeKey::Playlist(_) => {}
            }
        }
    }
    path.remove(node);
}

fn collect_descendant_game_ids(
    node: &NavigationNodeKey,
    children: &BTreeMap<NavigationNodeKey, BTreeSet<NavigationNodeKey>>,
    games: &[Game],
    playlist_game_ids: &BTreeMap<String, BTreeSet<String>>,
    path: &mut BTreeSet<NavigationNodeKey>,
    ids: &mut BTreeSet<String>,
) {
    if !path.insert(node.clone()) {
        return;
    }
    if let Some(node_children) = children.get(node) {
        for child in node_children {
            match child {
                NavigationNodeKey::Platform(platform) => {
                    ids.extend(
                        games
                            .iter()
                            .filter(|game| platform_key(&game.platform) == *platform)
                            .map(|game| game.id.clone()),
                    );
                }
                NavigationNodeKey::Playlist(playlist) => {
                    if let Some(playlist_ids) = playlist_game_ids.get(playlist) {
                        ids.extend(playlist_ids.iter().cloned());
                    }
                }
                NavigationNodeKey::Category(_) => {}
            }
            collect_descendant_game_ids(child, children, games, playlist_game_ids, path, ids);
        }
    }
    path.remove(node);
}

fn auto_playlist_matches(game: &Game, filters: &[lb_domain::PlaylistFilter]) -> bool {
    let mut grouped = BTreeMap::<String, Vec<&lb_domain::PlaylistFilter>>::new();
    for filter in filters {
        grouped
            .entry(filter.field_key.to_lowercase())
            .or_default()
            .push(filter);
    }
    grouped.values().all(|group| {
        group
            .iter()
            .any(|filter| playlist_filter_matches(game, filter))
    })
}

fn playlist_filter_matches(game: &Game, filter: &lb_domain::PlaylistFilter) -> bool {
    let field = filter.field_key.to_lowercase();
    let comparison = filter.comparison_type_key.to_lowercase();
    let expected = filter.value.trim();
    let boolean = match field.as_str() {
        "favorite" => Some(game.favorite),
        "completed" => Some(game.completed),
        "broken" => Some(game.broken),
        "hidden" | "hide" => Some(game.hidden),
        "installed" => game.installed,
        _ => None,
    };
    if let Some(actual) = boolean {
        return match comparison.as_str() {
            "istrue" => actual,
            "isfalse" => !actual,
            "equalto" | "isequalto" => expected
                .parse::<bool>()
                .is_ok_and(|expected| actual == expected),
            "notequalto" | "isnotequalto" => expected
                .parse::<bool>()
                .is_ok_and(|expected| actual != expected),
            _ => false,
        };
    }
    if field == "lastplayed" && comparison == "recentdays" {
        let Ok(days) = expected.parse::<i64>() else {
            return false;
        };
        let Some(last_played) = game.last_played_date.as_deref() else {
            return false;
        };
        let Ok(last_played) = DateTime::parse_from_rfc3339(last_played) else {
            return false;
        };
        return Local::now().signed_duration_since(last_played).num_days() <= days;
    }
    let actual = match field.as_str() {
        "title" => Some(game.title.as_str()),
        "platform" => Some(game.platform.as_str()),
        "genre" => game.genre.as_deref(),
        "publisher" => game.publisher.as_deref(),
        "series" => game.series.as_deref(),
        "source" => game.source.as_deref(),
        "playmode" => game.play_mode.as_deref(),
        "developer" => game.developer.as_deref(),
        "status" => game.status.as_deref(),
        "region" => game.region.as_deref(),
        "rating" => game.rating.as_deref(),
        "releasetype" => game.release_type.as_deref(),
        "version" => game.version.as_deref(),
        "progress" => game.progress.as_deref(),
        _ => None,
    }
    .unwrap_or_default()
    .to_lowercase();
    let expected = expected.to_lowercase();
    match comparison.as_str() {
        "contains" => actual.contains(&expected),
        "notcontains" => !actual.contains(&expected),
        "equalto" | "isequalto" => actual == expected,
        "notequalto" | "isnotequalto" => actual != expected,
        "startswith" => actual.starts_with(&expected),
        "endswith" => actual.ends_with(&expected),
        _ => false,
    }
}

fn sort_navigation_keys(
    keys: &mut [NavigationNodeKey],
    nodes: &BTreeMap<NavigationNodeKey, NavigationNodeInfo>,
) {
    keys.sort_by(|left, right| {
        let left_info = &nodes[left];
        let right_info = &nodes[right];
        left_info
            .sort_key
            .cmp(&right_info.sort_key)
            .then_with(|| left_info.kind.cmp(right_info.kind))
            .then_with(|| left_info.key.cmp(&right_info.key))
    });
}

#[allow(clippy::too_many_arguments)]
fn flatten_navigation_node(
    node: &NavigationNodeKey,
    depth: usize,
    nodes: &BTreeMap<NavigationNodeKey, NavigationNodeInfo>,
    children: &BTreeMap<NavigationNodeKey, BTreeSet<NavigationNodeKey>>,
    category_game_ids: &BTreeMap<String, BTreeSet<String>>,
    playlist_game_ids: &BTreeMap<String, BTreeSet<String>>,
    platform_game_counts: &BTreeMap<String, usize>,
    path: &mut BTreeSet<NavigationNodeKey>,
    rendered: &mut BTreeSet<NavigationNodeKey>,
    entries: &mut Vec<NavigationEntry>,
) {
    if !path.insert(node.clone()) {
        return;
    }
    let Some(info) = nodes.get(node) else {
        path.remove(node);
        return;
    };
    rendered.insert(node.clone());
    let game_count = match node {
        NavigationNodeKey::Platform(name) => {
            platform_game_counts.get(name).copied().unwrap_or_default()
        }
        NavigationNodeKey::Category(name) => category_game_ids
            .get(name)
            .map(BTreeSet::len)
            .unwrap_or_default(),
        NavigationNodeKey::Playlist(id) => playlist_game_ids
            .get(id)
            .map(BTreeSet::len)
            .unwrap_or_default(),
    };
    entries.push(NavigationEntry {
        kind: info.kind,
        key: info.key.clone(),
        name: info.name.clone(),
        depth,
        game_count,
        visible_in_big_box: info.visible_in_big_box,
    });
    if let Some(node_children) = children.get(node) {
        let mut node_children = node_children.iter().cloned().collect::<Vec<_>>();
        sort_navigation_keys(&mut node_children, nodes);
        for child in node_children {
            flatten_navigation_node(
                &child,
                depth.saturating_add(1),
                nodes,
                children,
                category_game_ids,
                playlist_game_ids,
                platform_game_counts,
                path,
                rendered,
                entries,
            );
        }
    }
    path.remove(node);
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
    fn nested_navigation_flattens_categories_and_filters_descendant_platforms() {
        let catalog = NavigationCatalog {
            platforms: Vec::new(),
            categories: vec![
                PlatformCategory {
                    metadata: NavigationMetadata {
                        name: "Systems".into(),
                        nested_name: Some("All Systems".into()),
                        ..NavigationMetadata::default()
                    },
                    ..PlatformCategory::default()
                },
                PlatformCategory {
                    metadata: NavigationMetadata {
                        name: "Handhelds".into(),
                        ..NavigationMetadata::default()
                    },
                    ..PlatformCategory::default()
                },
            ],
            parents: vec![
                ParentRelationship {
                    platform_category_name: Some("Systems".into()),
                    ..ParentRelationship::default()
                },
                ParentRelationship {
                    platform_category_name: Some("Handhelds".into()),
                    parent_platform_category_name: Some("Systems".into()),
                    ..ParentRelationship::default()
                },
                ParentRelationship {
                    platform_name: Some("Game Boy".into()),
                    parent_platform_category_name: Some("Handhelds".into()),
                    ..ParentRelationship::default()
                },
                ParentRelationship {
                    platform_name: Some("Arcade".into()),
                    ..ParentRelationship::default()
                },
            ],
            ..NavigationCatalog::default()
        };
        let games = vec![
            Game {
                id: "handheld-game".into(),
                platform: "Game Boy".into(),
                ..Game::default()
            },
            Game {
                id: "arcade-game".into(),
                platform: "Arcade".into(),
                ..Game::default()
            },
        ];
        let (entries, category_platforms, category_game_ids, playlist_game_ids) =
            build_navigation_entries(&catalog, &["Game Boy".into(), "Arcade".into()], &games);
        assert_eq!(
            entries
                .iter()
                .map(|entry| (
                    entry.kind,
                    entry.key.as_str(),
                    entry.depth,
                    entry.game_count
                ))
                .collect::<Vec<_>>(),
            [
                ("platform", "Arcade", 0, 1),
                ("category", "Systems", 0, 1),
                ("category", "Handhelds", 1, 1),
                ("platform", "Game Boy", 2, 1),
            ]
        );
        assert_eq!(
            category_platforms["systems"]
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["game boy"]
        );
        assert_eq!(
            category_game_ids["systems"],
            BTreeSet::from(["handheld-game".into()])
        );
        assert!(playlist_game_ids.is_empty());
    }

    #[test]
    fn big_box_navigation_hides_marked_nodes_and_reparents_visible_descendants() {
        let catalog = NavigationCatalog {
            platforms: vec![
                PlatformDefinition {
                    metadata: NavigationMetadata {
                        name: "Visible Console".into(),
                        ..NavigationMetadata::default()
                    },
                    ..PlatformDefinition::default()
                },
                PlatformDefinition {
                    metadata: NavigationMetadata {
                        name: "Hidden Console".into(),
                        hide_in_big_box: true,
                        ..NavigationMetadata::default()
                    },
                    ..PlatformDefinition::default()
                },
            ],
            categories: vec![PlatformCategory {
                metadata: NavigationMetadata {
                    name: "Hidden Category".into(),
                    hide_in_big_box: true,
                    ..NavigationMetadata::default()
                },
                ..PlatformCategory::default()
            }],
            parents: vec![
                ParentRelationship {
                    platform_category_name: Some("Hidden Category".into()),
                    ..ParentRelationship::default()
                },
                ParentRelationship {
                    platform_name: Some("Visible Console".into()),
                    parent_platform_category_name: Some("Hidden Category".into()),
                    ..ParentRelationship::default()
                },
                ParentRelationship {
                    playlist_id: Some("visible-list".into()),
                    parent_platform_name: Some("Visible Console".into()),
                    ..ParentRelationship::default()
                },
            ],
            playlists: vec![
                PlaylistDocument {
                    playlist: Playlist {
                        id: "visible-list".into(),
                        metadata: NavigationMetadata {
                            name: "Visible Playlist".into(),
                            ..NavigationMetadata::default()
                        },
                        ..Playlist::default()
                    },
                    ..PlaylistDocument::default()
                },
                PlaylistDocument {
                    playlist: Playlist {
                        id: "hidden-list".into(),
                        metadata: NavigationMetadata {
                            name: "Hidden Playlist".into(),
                            hide_in_big_box: true,
                            ..NavigationMetadata::default()
                        },
                        ..Playlist::default()
                    },
                    ..PlaylistDocument::default()
                },
            ],
        };
        let (entries, _, _, _) = build_navigation_entries(
            &catalog,
            &["Visible Console".into(), "Hidden Console".into()],
            &[],
        );
        assert_eq!(entries.len(), 5);
        let big_box_entries = build_big_box_navigation_entries(&entries);
        assert_eq!(
            big_box_entries
                .iter()
                .map(|entry| (entry.kind, entry.key.as_str(), entry.depth))
                .collect::<Vec<_>>(),
            [
                ("platform", "Visible Console", 0),
                ("playlist", "visible-list", 1),
            ]
        );
    }

    #[test]
    fn playlist_navigation_uses_ids_and_launchbox_or_within_and_across_filter_groups() {
        let games = vec![
            Game {
                id: "adventure".into(),
                title: "Adventure".into(),
                platform: "Arcade".into(),
                genre: Some("Action Adventure".into()),
                favorite: true,
                ..Game::default()
            },
            Game {
                id: "racer".into(),
                title: "Racer".into(),
                platform: "Arcade".into(),
                genre: Some("Racing".into()),
                ..Game::default()
            },
            Game {
                id: "console".into(),
                title: "Console Game".into(),
                platform: "Console".into(),
                genre: Some("Racing".into()),
                ..Game::default()
            },
        ];
        let catalog = NavigationCatalog {
            platforms: Vec::new(),
            categories: vec![PlatformCategory {
                metadata: NavigationMetadata {
                    name: "Collections".into(),
                    ..NavigationMetadata::default()
                },
                ..PlatformCategory::default()
            }],
            parents: vec![
                ParentRelationship {
                    platform_category_name: Some("Collections".into()),
                    ..ParentRelationship::default()
                },
                ParentRelationship {
                    playlist_id: Some("auto-id".into()),
                    parent_platform_category_name: Some("Collections".into()),
                    ..ParentRelationship::default()
                },
                ParentRelationship {
                    playlist_id: Some("manual-id".into()),
                    ..ParentRelationship::default()
                },
            ],
            playlists: vec![
                PlaylistDocument {
                    playlist: Playlist {
                        id: "auto-id".into(),
                        metadata: NavigationMetadata {
                            name: "Immutable Auto Name".into(),
                            nested_name: Some("Arcade Genres".into()),
                            ..NavigationMetadata::default()
                        },
                        auto_populate: true,
                        ..Playlist::default()
                    },
                    filters: vec![
                        PlaylistFilter {
                            field_key: "Platform".into(),
                            comparison_type_key: "EqualTo".into(),
                            value: "Arcade".into(),
                        },
                        PlaylistFilter {
                            field_key: "Genre".into(),
                            comparison_type_key: "Contains".into(),
                            value: "Action".into(),
                        },
                        PlaylistFilter {
                            field_key: "Genre".into(),
                            comparison_type_key: "Contains".into(),
                            value: "Racing".into(),
                        },
                    ],
                    ..PlaylistDocument::default()
                },
                PlaylistDocument {
                    playlist: Playlist {
                        id: "manual-id".into(),
                        metadata: NavigationMetadata {
                            name: "Manual".into(),
                            ..NavigationMetadata::default()
                        },
                        ..Playlist::default()
                    },
                    games: vec![PlaylistGame {
                        game_id: "console".into(),
                        game_title: "Console Game".into(),
                        game_platform: "Console".into(),
                        ..PlaylistGame::default()
                    }],
                    ..PlaylistDocument::default()
                },
            ],
        };
        let (entries, _, category_game_ids, playlist_game_ids) =
            build_navigation_entries(&catalog, &[], &games);
        assert_eq!(
            playlist_game_ids["auto-id"],
            BTreeSet::from(["adventure".into(), "racer".into()])
        );
        assert_eq!(
            playlist_game_ids["manual-id"],
            BTreeSet::from(["console".into()])
        );
        assert_eq!(
            category_game_ids["collections"],
            BTreeSet::from(["adventure".into(), "racer".into()])
        );
        assert!(entries.iter().any(|entry| {
            entry.kind == "playlist"
                && entry.key == "auto-id"
                && entry.name == "Arcade Genres"
                && entry.depth == 1
                && entry.game_count == 2
        }));
        assert!(entries.iter().any(|entry| {
            entry.kind == "playlist"
                && entry.key == "manual-id"
                && entry.depth == 0
                && entry.game_count == 1
        }));
    }

    #[test]
    fn category_payload_rejects_rename_duplicates_and_hierarchy_cycles() {
        let catalog = NavigationCatalog {
            categories: vec![
                PlatformCategory {
                    metadata: NavigationMetadata {
                        name: "Parent".into(),
                        ..NavigationMetadata::default()
                    },
                    ..PlatformCategory::default()
                },
                PlatformCategory {
                    metadata: NavigationMetadata {
                        name: "Child".into(),
                        ..NavigationMetadata::default()
                    },
                    ..PlatformCategory::default()
                },
            ],
            parents: vec![ParentRelationship {
                platform_category_name: Some("Child".into()),
                parent_platform_category_name: Some("Parent".into()),
                ..ParentRelationship::default()
            }],
            ..NavigationCatalog::default()
        };
        let mut payload: CategoryEditPayload =
            serde_json::from_str(&new_category_payload(&catalog, &[]).unwrap()).unwrap();
        payload.category.name = "Parent".into();
        payload.parents[0].target_kind = CategoryParentKind::PlatformCategory;
        payload.parents[0].target_key = "Child".into();
        let serialized = serde_json::to_string(&payload).unwrap();
        let parsed = parse_category_edit_payload(Some("Parent"), &serialized).unwrap();
        assert!(validate_category_hierarchy_edit(&catalog, &[], &parsed, false).is_err());
        assert!(parse_category_edit_payload(Some("Different"), &serialized).is_err());

        payload.parents.push(payload.parents[0].clone());
        assert!(parse_category_edit_payload(
            Some("Parent"),
            &serde_json::to_string(&payload).unwrap()
        )
        .is_err());
        assert!(parse_category_edit_payload(
            Some("Parent"),
            &serialized.replace("\"version\":1", "\"version\":2")
        )
        .is_err());
        assert!(parse_category_edit_payload(
            Some("Parent"),
            &serialized.replacen("\"parents\":", "\"future\":true,\"parents\":", 1)
        )
        .is_err());
    }

    #[test]
    fn playlist_payload_rejects_identity_changes_duplicate_games_and_cycles() {
        let parent = Playlist {
            id: "parent".into(),
            metadata: NavigationMetadata {
                name: "Parent Unique Name".into(),
                ..NavigationMetadata::default()
            },
            ..Playlist::default()
        };
        let child = Playlist {
            id: "child".into(),
            metadata: NavigationMetadata {
                name: "Child Unique Name".into(),
                ..NavigationMetadata::default()
            },
            ..Playlist::default()
        };
        let catalog = NavigationCatalog {
            parents: vec![ParentRelationship {
                playlist_id: Some("child".into()),
                parent_playlist_id: Some("parent".into()),
                ..ParentRelationship::default()
            }],
            playlists: vec![
                PlaylistDocument {
                    playlist: parent.clone(),
                    ..PlaylistDocument::default()
                },
                PlaylistDocument {
                    playlist: child,
                    ..PlaylistDocument::default()
                },
            ],
            ..NavigationCatalog::default()
        };
        let mut payload: PlaylistEditPayload =
            serde_json::from_str(&new_playlist_payload(&catalog, &[], &[]).unwrap()).unwrap();
        payload.playlist = PlaylistEditFields::from(&parent);
        payload.parents[0].target_kind = CategoryParentKind::Playlist;
        payload.parents[0].target_key = "child".into();
        let serialized = serde_json::to_string(&payload).unwrap();
        let parsed = parse_playlist_edit_payload(Some(&parent), &serialized).unwrap();
        assert!(validate_playlist_hierarchy_edit(&catalog, &[], &parsed, false).is_err());

        payload.playlist.id = "changed".into();
        assert!(parse_playlist_edit_payload(
            Some(&parent),
            &serde_json::to_string(&payload).unwrap()
        )
        .is_err());
        payload.playlist = PlaylistEditFields::from(&parent);
        payload.playlist.name = "Changed Name".into();
        assert!(parse_playlist_edit_payload(
            Some(&parent),
            &serde_json::to_string(&payload).unwrap()
        )
        .is_err());
        payload.playlist = PlaylistEditFields::from(&parent);
        let duplicate = PlaylistGameEditPayload {
            source_index: None,
            game_id: "same-game".into(),
            game_title: "Game".into(),
            game_platform: "Platform".into(),
            game_file_name: String::new(),
            launchbox_db_id: None,
            manual_order: 1,
        };
        payload.games = vec![duplicate.clone(), duplicate];
        assert!(parse_playlist_edit_payload(
            Some(&parent),
            &serde_json::to_string(&payload).unwrap()
        )
        .is_err());
    }

    #[test]
    fn category_lifecycle_is_two_document_transactional_and_detaches_children() {
        let directory = tempfile::tempdir().expect("temporary library");
        let data_directory = directory.path().join("Data");
        fs::create_dir_all(&data_directory).unwrap();
        let fixture_root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/launchbox/Data");
        let catalog_path = data_directory.join("Platforms.xml");
        let parents_path = data_directory.join("Parents.xml");
        let original_catalog = fs::read_to_string(fixture_root.join("Platforms.xml"))
            .unwrap()
            .replace(
                "<Notes>A fixture category.</Notes>",
                "<Notes>A fixture category.</Notes><FutureCategoryField>keep-category</FutureCategoryField>",
            );
        let original_parents = fs::read_to_string(fixture_root.join("Parents.xml"))
            .unwrap()
            .replace(
                "<ParentPlatformCategoryName>Fixture Category</ParentPlatformCategoryName>",
                "<ParentPlatformCategoryName>Collections</ParentPlatformCategoryName><FutureChildPlacement>keep-child</FutureChildPlacement>",
            );
        fs::write(&catalog_path, &original_catalog).unwrap();
        fs::write(&parents_path, &original_parents).unwrap();
        let catalog_document = AuxiliaryDocument::load(&catalog_path).unwrap();
        let parents_document = AuxiliaryDocument::load(&parents_path).unwrap();
        let navigation_catalog = NavigationCatalog {
            platforms: Vec::new(),
            categories: catalog_document.platform_catalog().unwrap().categories,
            parents: parents_document.parent_relationships().unwrap(),
            playlists: vec![PlaylistDocument {
                playlist: lb_domain::Playlist {
                    id: "fixture-playlist".into(),
                    metadata: NavigationMetadata {
                        name: "Fixture Playlist".into(),
                        ..NavigationMetadata::default()
                    },
                    ..lb_domain::Playlist::default()
                },
                ..PlaylistDocument::default()
            }],
        };
        let mut create: CategoryEditPayload = serde_json::from_str(
            &new_category_payload(&navigation_catalog, &["Fixture Console".into()]).unwrap(),
        )
        .unwrap();
        create.category.name = "Collections".into();
        create.category.nested_name = Some("Curated Collections".into());
        create.category.notes = Some("Created by the Qt category editor.".into());
        create.parents[0].target_kind = CategoryParentKind::PlatformCategory;
        create.parents[0].target_key = "Fixture Category".into();
        let create =
            parse_category_edit_payload(None, &serde_json::to_string(&create).unwrap()).unwrap();
        let created = create_category_in_library(
            directory.path().to_path_buf(),
            create,
            navigation_catalog.clone(),
            vec!["Fixture Console".into()],
        )
        .unwrap();
        assert_eq!(created.placement_count, 1);
        assert_eq!(
            fs::read(&created.catalog_backup).unwrap(),
            original_catalog.as_bytes()
        );
        assert_eq!(
            fs::read(&created.parents_backup).unwrap(),
            original_parents.as_bytes()
        );
        let created_catalog = fs::read(&catalog_path).unwrap();
        let created_parents = fs::read(&parents_path).unwrap();

        let current_navigation = NavigationCatalog {
            platforms: Vec::new(),
            categories: created.categories.clone(),
            parents: created.parents.clone(),
            playlists: navigation_catalog.playlists.clone(),
        };
        let serialized = load_category_edit_payload(
            directory.path(),
            "collections",
            &current_navigation,
            &["Fixture Console".into()],
        )
        .unwrap();
        let mut edit = parse_category_edit_payload(Some("Collections"), &serialized).unwrap();
        assert_eq!(edit.parents[0].source_index, Some(0));
        edit.category.sort_title = Some("Collections, Curated".into());
        edit.category.video_path = Some(r"Videos\Categories\collections.mp4".into());
        edit.category.hide_in_big_box = true;
        edit.parents.push(CategoryParentEditPayload {
            source_index: None,
            target_kind: CategoryParentKind::Root,
            target_key: String::new(),
        });
        let edit = parse_category_edit_payload(
            Some("Collections"),
            &serde_json::to_string(&edit).unwrap(),
        )
        .unwrap();
        let edited = edit_category_in_library(
            directory.path().to_path_buf(),
            "Collections".into(),
            edit,
            current_navigation,
            vec!["Fixture Console".into()],
        )
        .unwrap();
        assert_eq!(edited.placement_count, 2);
        assert_eq!(fs::read(&edited.catalog_backup).unwrap(), created_catalog);
        assert_eq!(fs::read(&edited.parents_backup).unwrap(), created_parents);
        let edited_catalog = fs::read(&catalog_path).unwrap();
        let edited_parents = fs::read(&parents_path).unwrap();
        let category = edited
            .categories
            .iter()
            .find(|category| category.metadata.name == "Collections")
            .unwrap();
        assert_eq!(
            category.metadata.video_path.as_deref(),
            Some(r"Videos\Categories\collections.mp4")
        );
        assert!(category.metadata.hide_in_big_box);

        let deleted =
            delete_category_from_library(directory.path().to_path_buf(), "Collections".into())
                .unwrap();
        assert_eq!(deleted.removed_placements, 2);
        assert_eq!(deleted.detached_children, 1);
        assert_eq!(fs::read(&deleted.catalog_backup).unwrap(), edited_catalog);
        assert_eq!(fs::read(&deleted.parents_backup).unwrap(), edited_parents);
        assert!(deleted
            .categories
            .iter()
            .all(|category| category.metadata.name != "Collections"));
        let child = deleted
            .parents
            .iter()
            .find(|relationship| relationship.platform_name.as_deref() == Some("Fixture Console"))
            .unwrap();
        assert!(child.parent_platform_category_name.is_none());
        let final_catalog = fs::read_to_string(&catalog_path).unwrap();
        let final_parents = fs::read_to_string(&parents_path).unwrap();
        assert!(final_catalog.contains("FutureCategoryField"));
        assert!(final_parents.contains("FutureChildPlacement"));
        assert!(pending_transaction_manifests(directory.path())
            .unwrap()
            .is_empty());
    }

    #[test]
    fn playlist_lifecycle_is_transactional_portable_and_detaches_without_deleting_games() {
        let directory = tempfile::tempdir().expect("temporary library");
        let data_directory = directory.path().join("Data");
        let playlist_directory = data_directory.join("Playlists");
        fs::create_dir_all(&playlist_directory).unwrap();
        let fixture_root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/launchbox/Data");
        let parents_path = data_directory.join("Parents.xml");
        let cache_path = data_directory.join("ListCache.xml");
        let platform_path = data_directory.join("Platforms/Fixture Console.xml");
        fs::create_dir_all(platform_path.parent().unwrap()).unwrap();
        fs::copy(fixture_root.join("Parents.xml"), &parents_path).unwrap();
        fs::copy(fixture_root.join("ListCache.xml"), &cache_path).unwrap();
        fs::copy(
            fixture_root.join("Platforms/Fixture Console.xml"),
            &platform_path,
        )
        .unwrap();
        let original_platform = fs::read(&platform_path).unwrap();
        let parents_document = AuxiliaryDocument::load(&parents_path).unwrap();
        let navigation_catalog = NavigationCatalog {
            platforms: Vec::new(),
            categories: vec![PlatformCategory {
                metadata: NavigationMetadata {
                    name: "Fixture Category".into(),
                    ..NavigationMetadata::default()
                },
                ..PlatformCategory::default()
            }],
            parents: parents_document.parent_relationships().unwrap(),
            playlists: Vec::new(),
        };
        let games = vec![Game {
            id: "fixture-racer".into(),
            title: "Fixture Racer".into(),
            platform: "Fixture Console".into(),
            application_path: r"Games\Fixture Racer\racer.rom".into(),
            database_id: Some(4321),
            favorite: true,
            ..Game::default()
        }];
        let mut create: PlaylistEditPayload = serde_json::from_str(
            &new_playlist_payload(&navigation_catalog, &["Fixture Console".into()], &games)
                .unwrap(),
        )
        .unwrap();
        create.playlist.name = "Portable/Queue".into();
        create.playlist.nested_name = Some("Portable Queue".into());
        create.playlist.video_path = Some(r"Videos\Playlists\portable.mp4".into());
        create.games.push(PlaylistGameEditPayload {
            source_index: None,
            game_id: "fixture-racer".into(),
            game_title: String::new(),
            game_platform: String::new(),
            game_file_name: String::new(),
            launchbox_db_id: None,
            manual_order: 1,
        });
        create.parents[0].target_kind = CategoryParentKind::PlatformCategory;
        create.parents[0].target_key = "Fixture Category".into();
        let create =
            parse_playlist_edit_payload(None, &serde_json::to_string(&create).unwrap()).unwrap();
        let created = create_playlist_in_library(
            directory.path().to_path_buf(),
            create,
            navigation_catalog,
            vec!["Fixture Console".into()],
            games.clone(),
        )
        .unwrap();
        assert_eq!(created.source.file_name().unwrap(), "Portable_Queue.xml");
        assert_eq!(created.placement_count, 1);
        assert!(created.playlist_backup.is_none());
        let created_document = AuxiliaryDocument::load(&created.source)
            .unwrap()
            .playlist_document()
            .unwrap();
        assert_eq!(created_document.games[0].game_file_name, "racer.rom");
        assert_eq!(created_document.games[0].launchbox_db_id, Some(4321));
        assert_eq!(
            created_document.playlist.metadata.video_path.as_deref(),
            Some(r"Videos\Playlists\portable.mp4")
        );
        let created_playlist_bytes = fs::read(&created.source).unwrap();
        let created_parent_bytes = fs::read(&parents_path).unwrap();

        let current = NavigationCatalog {
            platforms: Vec::new(),
            categories: vec![PlatformCategory {
                metadata: NavigationMetadata {
                    name: "Fixture Category".into(),
                    ..NavigationMetadata::default()
                },
                ..PlatformCategory::default()
            }],
            parents: created.parents.clone(),
            playlists: created.playlists.clone(),
        };
        let serialized = load_playlist_edit_payload(
            directory.path(),
            &created.id,
            &current,
            &["Fixture Console".into()],
            &games,
        )
        .unwrap();
        let original_playlist = current.playlists[0].playlist.clone();
        let mut edit = parse_playlist_edit_payload(Some(&original_playlist), &serialized).unwrap();
        assert_eq!(edit.games[0].source_index, Some(0));
        assert_eq!(edit.parents[0].source_index, Some(0));
        edit.playlist.sort_title = Some("Queue, Portable".into());
        edit.playlist.auto_populate = true;
        edit.filters.push(PlaylistFilterEditPayload {
            source_index: None,
            field_key: "Favorite".into(),
            comparison_type_key: "IsTrue".into(),
            value: String::new(),
        });
        edit.parents.push(CategoryParentEditPayload {
            source_index: None,
            target_kind: CategoryParentKind::Root,
            target_key: String::new(),
        });
        let edit = parse_playlist_edit_payload(
            Some(&original_playlist),
            &serde_json::to_string(&edit).unwrap(),
        )
        .unwrap();
        let edited = edit_playlist_in_library(
            directory.path().to_path_buf(),
            created.id.clone(),
            edit,
            current,
            vec!["Fixture Console".into()],
            games,
        )
        .unwrap();
        assert_eq!(edited.placement_count, 2);
        assert_eq!(
            fs::read(edited.playlist_backup.as_ref().unwrap()).unwrap(),
            created_playlist_bytes
        );
        assert_eq!(
            fs::read(&edited.parents_backup).unwrap(),
            created_parent_bytes
        );

        let mut parents_document = AuxiliaryDocument::load(&parents_path).unwrap();
        parents_document
            .set_playlist_parents(
                "child-list",
                vec![IndexedPlatformRecordEdit {
                    source_index: None,
                    record: ParentRelationship {
                        playlist_id: Some("child-list".into()),
                        parent_playlist_id: Some(created.id.clone()),
                        ..ParentRelationship::default()
                    },
                }],
            )
            .unwrap();
        fs::write(&parents_path, parents_document.to_xml_bytes().unwrap()).unwrap();
        let cache_xml = format!(
            "<LaunchBox><ListCacheItem><PlaylistId>{}</PlaylistId><FutureCache>drop</FutureCache></ListCacheItem><ListCacheItem><PlaylistId>other</PlaylistId><FutureCache>keep</FutureCache></ListCacheItem></LaunchBox>",
            created.id
        );
        fs::write(&cache_path, cache_xml).unwrap();
        let edited_playlist_bytes = fs::read(&created.source).unwrap();
        let edited_parents_bytes = fs::read(&parents_path).unwrap();
        let deleted = delete_playlist_from_library(
            directory.path().to_path_buf(),
            created.id.clone(),
            NavigationCatalog {
                platforms: Vec::new(),
                categories: vec![],
                parents: edited.parents,
                playlists: edited.playlists,
            },
        )
        .unwrap();
        assert_eq!(deleted.removed_placements, 2);
        assert_eq!(deleted.detached_children, 1);
        assert_eq!(deleted.removed_cache_rows, 1);
        assert_eq!(
            fs::read(deleted.playlist_backup.as_ref().unwrap()).unwrap(),
            edited_playlist_bytes
        );
        assert_eq!(
            fs::read(&deleted.parents_backup).unwrap(),
            edited_parents_bytes
        );
        assert!(deleted.list_cache_backup.is_some());
        assert!(!deleted.source.exists());
        let remaining_parent = deleted
            .parents
            .iter()
            .find(|relationship| relationship.playlist_id.as_deref() == Some("child-list"))
            .unwrap();
        assert!(remaining_parent.parent_playlist_id.is_none());
        assert!(fs::read_to_string(cache_path)
            .unwrap()
            .contains("<FutureCache>keep</FutureCache>"));
        assert_eq!(fs::read(platform_path).unwrap(), original_platform);
        assert!(pending_transaction_manifests(directory.path())
            .unwrap()
            .is_empty());
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
                emulator_id: None,
                metadata: NewGameMetadata::default(),
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
    fn platform_edit_payload_round_trips_every_typed_field_and_lexical_folder_path() {
        let directory = tempfile::tempdir().expect("temporary library");
        let data_directory = directory.path().join("Data");
        let platform_directory = data_directory.join("Platforms");
        fs::create_dir_all(&platform_directory).unwrap();
        let fixture_root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/launchbox/Data");
        let original_catalog = fs::read_to_string(fixture_root.join("Platforms.xml"))
            .unwrap()
            .replace(
                "<Developer>Fixture Labs</Developer>",
                "<Developer>Fixture Labs</Developer><FuturePlatformField>keep-platform</FuturePlatformField>",
            )
            .replace(
                "<FolderPath>Videos/Fixture Console</FolderPath>",
                "<FolderPath>Videos/Fixture Console</FolderPath><FutureFolderField>keep-folder</FutureFolderField>",
            );
        let catalog_path = data_directory.join("Platforms.xml");
        fs::write(&catalog_path, original_catalog.as_bytes()).unwrap();
        fs::copy(
            fixture_root.join("Platforms/Fixture Console.xml"),
            platform_directory.join("Fixture Console.xml"),
        )
        .unwrap();

        let serialized = load_platform_edit_payload(directory.path(), "fixture console").unwrap();
        let mut payload = parse_platform_edit_payload("Fixture Console", &serialized).unwrap();
        assert_eq!(payload.folders[0].source_index, Some(0));
        payload.platform.metadata.sort_title = Some("Console, Fixture".into());
        payload.platform.metadata.notes = Some("   ".into());
        payload.platform.metadata.manufacturer = Some("Portable Systems".into());
        payload.platform.metadata.cpu = Some("Qt CPU".into());
        payload.platform.metadata.hide_in_big_box = true;
        payload.platform.release_date = Some("2001-02-03".into());
        payload.platform.disable_auto_import = true;
        payload.folders[0].folder_path = r"Videos\Fixture Console\Edited".into();
        payload.folders.push(PlatformFolderEditPayload {
            source_index: None,
            media_type: "Manual".into(),
            folder_path: r"Manuals\Fixture Console".into(),
        });
        let payload = parse_platform_edit_payload(
            "Fixture Console",
            &serde_json::to_string(&payload).unwrap(),
        )
        .unwrap();
        let edited = write_platform_definition(
            directory.path().to_path_buf(),
            "Fixture Console".into(),
            payload,
        )
        .unwrap();
        assert_eq!(edited.folder_count, 2);
        assert_eq!(
            fs::read(&edited.catalog_backup).unwrap(),
            original_catalog.as_bytes()
        );

        let bytes = fs::read(&catalog_path).unwrap();
        let xml = String::from_utf8(bytes).unwrap();
        assert!(xml.contains("<FuturePlatformField>keep-platform</FuturePlatformField>"));
        assert!(xml.contains("<FutureFolderField>keep-folder</FutureFolderField>"));
        assert!(xml.contains(r"<FolderPath>Videos\Fixture Console\Edited</FolderPath>"));
        let catalog = AuxiliaryDocument::load(&catalog_path)
            .unwrap()
            .platform_catalog()
            .unwrap();
        let platform = &catalog.platforms[0];
        assert_eq!(
            platform.metadata.sort_title.as_deref(),
            Some("Console, Fixture")
        );
        assert_eq!(platform.metadata.notes, None);
        assert_eq!(platform.metadata.cpu.as_deref(), Some("Qt CPU"));
        assert_eq!(platform.release_date.as_deref(), Some("2001-02-03"));
        assert!(platform.metadata.hide_in_big_box);
        assert!(platform.disable_auto_import);
        assert_eq!(catalog.folders[1].media_type, "Manual");
        assert_eq!(catalog.folders[1].folder_path, r"Manuals\Fixture Console");
        assert!(!directory.path().join("Manuals").exists());
    }

    #[test]
    fn platform_edit_payload_rejects_rename_unknown_fields_and_wrong_versions() {
        let payload = PlatformEditPayload {
            version: PLATFORM_EDIT_PAYLOAD_VERSION,
            platform: PlatformDefinition {
                metadata: NavigationMetadata {
                    name: "Fixture Console".into(),
                    ..NavigationMetadata::default()
                },
                ..PlatformDefinition::default()
            },
            folders: Vec::new(),
        };
        let valid = serde_json::to_string(&payload).unwrap();
        assert!(parse_platform_edit_payload("Fixture Console", &valid).is_ok());
        assert!(parse_platform_edit_payload("Different Console", &valid).is_err());
        assert!(parse_platform_edit_payload(
            "Fixture Console",
            &valid.replace("\"version\":1", "\"version\":2")
        )
        .is_err());
        assert!(parse_platform_edit_payload(
            "Fixture Console",
            &valid.replacen("\"folders\":", "\"future\":true,\"folders\":", 1)
        )
        .is_err());
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
    fn additional_application_edit_payload_is_versioned_typed_and_canonicalized() {
        let payload = AdditionalApplicationEditPayload {
            version: ADDITIONAL_APPLICATION_EDIT_PAYLOAD_VERSION,
            application: AdditionalApplicationEdit {
                name: "Fixture Manual".into(),
                application_path: r"Games\Fixture\manual.pdf".into(),
                command_line: Some("   ".into()),
                use_emulator: false,
                emulator_id: Some("ignored-for-direct-launch".into()),
                developer: Some("Fixture Labs".into()),
                publisher: Some(" ".into()),
                region: Some("Europe".into()),
                release_date: Some("2005-06-07".into()),
                version: Some("Rev 3".into()),
                status: Some("Installed".into()),
                last_played: Some(" ".into()),
                ..AdditionalApplicationEdit::default()
            },
        };
        let valid = serde_json::to_string(&payload).expect("serialize editor payload");
        let parsed =
            parse_additional_application_edit_payload(&valid).expect("valid editor payload");
        assert_eq!(parsed.application.name, "Fixture Manual");
        assert_eq!(
            parsed.application.application_path,
            r"Games\Fixture\manual.pdf"
        );
        assert_eq!(parsed.application.command_line, None);
        assert_eq!(parsed.application.emulator_id, None);
        assert_eq!(
            parsed.application.developer.as_deref(),
            Some("Fixture Labs")
        );
        assert_eq!(parsed.application.publisher, None);
        assert_eq!(parsed.application.region.as_deref(), Some("Europe"));
        assert_eq!(parsed.application.last_played, None);

        let mut wrong_version = payload.clone();
        wrong_version.version += 1;
        assert!(parse_additional_application_edit_payload(
            &serde_json::to_string(&wrong_version).unwrap()
        )
        .unwrap_err()
        .contains("unsupported additional-application editor payload version"));

        let mut missing_name = payload.clone();
        missing_name.application.name = "   ".into();
        assert!(parse_additional_application_edit_payload(
            &serde_json::to_string(&missing_name).unwrap()
        )
        .unwrap_err()
        .contains("name cannot be empty"));

        let mut negative_priority = payload;
        negative_priority.application.priority = -1;
        assert!(parse_additional_application_edit_payload(
            &serde_json::to_string(&negative_priority).unwrap()
        )
        .unwrap_err()
        .contains("priority cannot be negative"));

        assert!(parse_additional_application_edit_payload(&valid.replacen(
            "\"application\":",
            "\"future\":true,\"application\":",
            1
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
