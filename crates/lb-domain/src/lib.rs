use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

mod box_size;
mod catalog;
mod list_view;

pub use box_size::*;
pub use catalog::*;
pub use list_view::*;

/// LaunchBox persists this ID when a game is explicitly configured not to use
/// an emulator. It is distinct from a missing `<Emulator>` field, which allows
/// the platform's default emulator mapping to apply.
pub const UNASSIGNED_EMULATOR_ID: &str = "00000000-0000-0000-0000-000000000000";

pub fn is_unassigned_emulator_id(id: &str) -> bool {
    id.eq_ignore_ascii_case(UNASSIGNED_EMULATOR_ID)
}

/// Every field observed on a LaunchBox 13.24 `<Game>` record. String fields
/// remain strings so the storage layer can preserve LaunchBox's exact path,
/// timestamp, URL, and free-form metadata spelling.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub title: String,
    pub sort_title: Option<String>,
    pub platform: String,
    pub application_path: String,
    pub command_line: Option<String>,
    pub emulator_id: Option<String>,
    pub notes: Option<String>,
    pub clone_of: Option<String>,
    pub database_id: Option<u32>,
    pub date_added: String,
    pub date_modified: String,
    pub developer: Option<String>,
    pub genre: Option<String>,
    pub max_players: Option<u32>,
    pub play_mode: Option<String>,
    pub progress: Option<String>,
    pub publisher: Option<String>,
    pub rating: Option<String>,
    pub region: Option<String>,
    pub release_date: Option<String>,
    pub release_type: Option<String>,
    pub series: Option<String>,
    pub source: Option<String>,
    pub status: Option<String>,
    pub version: Option<String>,
    pub wikipedia_url: Option<String>,
    pub favorite: bool,
    pub completed: bool,
    pub hidden: bool,
    pub broken: bool,
    pub portable: bool,
    pub installed: Option<bool>,
    pub play_count: u32,
    pub play_time_seconds: u64,
    pub star_rating: u8,
    pub star_rating_float: f64,
    pub community_star_rating: f64,
    pub community_star_rating_total_votes: u32,
    pub last_played_date: Option<String>,
    pub aggressive_window_hiding: bool,
    pub disable_shutdown_screen: bool,
    pub forceful_pause_screen_activation: bool,
    pub hide_all_non_exclusive_fullscreen_windows: bool,
    pub hide_mouse_cursor_in_game: bool,
    pub override_default_pause_screen_settings: bool,
    pub override_default_startup_screen_settings: bool,
    pub suspend_process_on_pause: bool,
    pub use_pause_screen: bool,
    pub use_startup_screen: bool,
    pub startup_load_delay: u32,
    pub configuration_command_line: Option<String>,
    pub configuration_path: Option<String>,
    pub load_state_auto_hotkey_script: Option<String>,
    pub pause_auto_hotkey_script: Option<String>,
    pub reset_auto_hotkey_script: Option<String>,
    pub resume_auto_hotkey_script: Option<String>,
    pub save_state_auto_hotkey_script: Option<String>,
    pub swap_discs_auto_hotkey_script: Option<String>,
    pub use_dos_box: bool,
    pub custom_dos_box_version_path: Option<String>,
    pub dos_box_configuration_path: Option<String>,
    pub use_scumm_vm: bool,
    pub scumm_vm_aspect_correction: bool,
    pub scumm_vm_fullscreen: bool,
    pub scumm_vm_game_data_folder_path: Option<String>,
    pub scumm_vm_game_type: Option<String>,
    pub manual_path: Option<String>,
    pub music_path: Option<String>,
    pub root_folder: Option<String>,
    pub theme_video_path: Option<String>,
    pub video_path: Option<String>,
    pub video_url: Option<String>,
    pub missing_background_image: bool,
    pub missing_banner_image: bool,
    pub missing_box_3d_image: bool,
    pub missing_box_front_image: bool,
    pub missing_cart_3d_image: bool,
    pub missing_cart_image: bool,
    pub missing_clear_logo_image: bool,
    pub missing_manual: bool,
    pub missing_marquee_image: bool,
    pub missing_music: bool,
    pub missing_screenshot_image: bool,
    pub missing_video: bool,
    pub gog_app_id: Option<String>,
    pub origin_app_id: Option<String>,
    pub origin_install_path: Option<String>,
    pub has_cloud_synced: bool,
    pub has_gog_achievements: Option<bool>,
    pub has_steam_achievements: Option<bool>,
    pub last_steam_scan: Option<String>,
    pub retro_achievements_beaten_hardcore: bool,
    pub retro_achievements_beaten_softcore: bool,
    pub retro_achievements_hash: Option<String>,
    pub retro_achievements_id: Option<u32>,
    pub android_background_path: Option<String>,
    pub android_background_thumb_path: Option<String>,
    pub android_box_front_full_path: Option<String>,
    pub android_box_front_thumb_path: Option<String>,
    pub android_clear_logo_full_path: Option<String>,
    pub android_clear_logo_thumb_path: Option<String>,
    pub android_game_title_screenshot_path: Option<String>,
    pub android_game_title_screenshot_thumb_path: Option<String>,
    pub android_gameplay_screenshot_path: Option<String>,
    pub android_gameplay_screenshot_thumb_path: Option<String>,
    pub android_video_path: Option<String>,
}

/// User-editable descriptive metadata that belongs to one game record.
///
/// Platform moves and launch configuration are intentionally excluded: moving
/// a game is a cross-document operation, while persisted executable paths must
/// pass through the platform path service before they become host paths.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameMetadata {
    pub title: String,
    pub sort_title: Option<String>,
    pub notes: Option<String>,
    pub developer: Option<String>,
    pub genre: Option<String>,
    pub max_players: Option<u32>,
    pub play_mode: Option<String>,
    pub progress: Option<String>,
    pub publisher: Option<String>,
    pub rating: Option<String>,
    pub region: Option<String>,
    pub release_date: Option<String>,
    pub release_type: Option<String>,
    pub series: Option<String>,
    pub source: Option<String>,
    pub status: Option<String>,
    pub version: Option<String>,
    pub wikipedia_url: Option<String>,
}

impl From<&Game> for GameMetadata {
    fn from(game: &Game) -> Self {
        Self {
            title: game.title.clone(),
            sort_title: game.sort_title.clone(),
            notes: game.notes.clone(),
            developer: game.developer.clone(),
            genre: game.genre.clone(),
            max_players: game.max_players,
            play_mode: game.play_mode.clone(),
            progress: game.progress.clone(),
            publisher: game.publisher.clone(),
            rating: game.rating.clone(),
            region: game.region.clone(),
            release_date: game.release_date.clone(),
            release_type: game.release_type.clone(),
            series: game.series.clone(),
            source: game.source.clone(),
            status: game.status.clone(),
            version: game.version.clone(),
            wikipedia_url: game.wikipedia_url.clone(),
        }
    }
}

/// Persisted fields that select and configure the executable launch backend.
/// Paths remain LaunchBox strings here; resolving them into native host paths
/// is exclusively the responsibility of `lb-platform`.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameLaunchConfiguration {
    pub application_path: String,
    pub command_line: Option<String>,
    pub emulator_id: Option<String>,
    pub use_dos_box: bool,
    pub custom_dos_box_version_path: Option<String>,
    pub dos_box_configuration_path: Option<String>,
    pub use_scumm_vm: bool,
    pub scumm_vm_aspect_correction: bool,
    pub scumm_vm_fullscreen: bool,
    pub scumm_vm_game_data_folder_path: Option<String>,
    pub scumm_vm_game_type: Option<String>,
}

impl GameLaunchConfiguration {
    pub fn validate_for_game(&self, game_id: &str) -> Result<(), ValidationError> {
        if self.use_dos_box && self.use_scumm_vm {
            return Err(ValidationError::ConflictingGameLaunchModes {
                id: game_id.to_string(),
            });
        }
        if self.use_scumm_vm
            && self
                .scumm_vm_game_data_folder_path
                .as_deref()
                .is_none_or(|path| path.trim().is_empty())
        {
            return Err(ValidationError::MissingScummVmGameDataPath {
                id: game_id.to_string(),
            });
        }
        if self.use_dos_box
            && self.application_path.trim().is_empty()
            && self
                .dos_box_configuration_path
                .as_deref()
                .is_none_or(|path| path.trim().is_empty())
        {
            return Err(ValidationError::MissingDosBoxApplicationOrConfiguration {
                id: game_id.to_string(),
            });
        }
        if !self.use_dos_box && !self.use_scumm_vm && self.application_path.trim().is_empty() {
            return Err(ValidationError::MissingGameApplicationPath {
                id: game_id.to_string(),
            });
        }
        Ok(())
    }
}

impl From<&Game> for GameLaunchConfiguration {
    fn from(game: &Game) -> Self {
        Self {
            application_path: game.application_path.clone(),
            command_line: game.command_line.clone(),
            emulator_id: game.emulator_id.clone(),
            use_dos_box: game.use_dos_box,
            custom_dos_box_version_path: game.custom_dos_box_version_path.clone(),
            dos_box_configuration_path: game.dos_box_configuration_path.clone(),
            use_scumm_vm: game.use_scumm_vm,
            scumm_vm_aspect_correction: game.scumm_vm_aspect_correction,
            scumm_vm_fullscreen: game.scumm_vm_fullscreen,
            scumm_vm_game_data_folder_path: game.scumm_vm_game_data_folder_path.clone(),
            scumm_vm_game_type: game.scumm_vm_game_type.clone(),
        }
    }
}

/// Canonical field inventory used by schema-audit tooling. Keep this in XML
/// spelling so new LaunchBox releases can be compared without inspecting data
/// values.
pub const GAME_XML_FIELDS: &[&str] = &[
    "AggressiveWindowHiding",
    "AndroidBackgroundPath",
    "AndroidBackgroundThumbPath",
    "AndroidBoxFrontFullPath",
    "AndroidBoxFrontThumbPath",
    "AndroidClearLogoFullPath",
    "AndroidClearLogoThumbPath",
    "AndroidGameTitleScreenshotPath",
    "AndroidGameTitleScreenshotThumbPath",
    "AndroidGameplayScreenshotPath",
    "AndroidGameplayScreenshotThumbPath",
    "AndroidVideoPath",
    "ApplicationPath",
    "Broken",
    "CloneOf",
    "CommandLine",
    "CommunityStarRating",
    "CommunityStarRatingTotalVotes",
    "Completed",
    "ConfigurationCommandLine",
    "ConfigurationPath",
    "CustomDosBoxVersionPath",
    "DatabaseID",
    "DateAdded",
    "DateModified",
    "Developer",
    "DisableShutdownScreen",
    "DosBoxConfigurationPath",
    "Emulator",
    "Favorite",
    "ForcefulPauseScreenActivation",
    "Genre",
    "GogAppId",
    "HasCloudSynced",
    "HasGogAchievements",
    "HasSteamAchievements",
    "Hide",
    "HideAllNonExclusiveFullscreenWindows",
    "HideMouseCursorInGame",
    "ID",
    "Installed",
    "LastPlayedDate",
    "LastSteamScan",
    "LoadStateAutoHotkeyScript",
    "ManualPath",
    "MaxPlayers",
    "MissingBackgroundImage",
    "MissingBannerImage",
    "MissingBox3dImage",
    "MissingBoxFrontImage",
    "MissingCart3dImage",
    "MissingCartImage",
    "MissingClearLogoImage",
    "MissingManual",
    "MissingMarqueeImage",
    "MissingMusic",
    "MissingScreenshotImage",
    "MissingVideo",
    "MusicPath",
    "Notes",
    "OriginAppId",
    "OriginInstallPath",
    "OverrideDefaultPauseScreenSettings",
    "OverrideDefaultStartupScreenSettings",
    "PauseAutoHotkeyScript",
    "Platform",
    "PlayCount",
    "PlayMode",
    "PlayTime",
    "Portable",
    "Progress",
    "Publisher",
    "Rating",
    "Region",
    "ReleaseDate",
    "ReleaseType",
    "ResetAutoHotkeyScript",
    "ResumeAutoHotkeyScript",
    "RetroAchievementsBeatenHardcore",
    "RetroAchievementsBeatenSoftcore",
    "RetroAchievementsHash",
    "RetroAchievementsId",
    "RootFolder",
    "SaveStateAutoHotkeyScript",
    "ScummVMAspectCorrection",
    "ScummVMFullscreen",
    "ScummVMGameDataFolderPath",
    "ScummVMGameType",
    "Series",
    "SortTitle",
    "Source",
    "StarRating",
    "StarRatingFloat",
    "StartupLoadDelay",
    "Status",
    "SuspendProcessOnPause",
    "SwapDiscsAutoHotkeyScript",
    "ThemeVideoPath",
    "Title",
    "UseDosBox",
    "UsePauseScreen",
    "UseScummVM",
    "UseStartupScreen",
    "Version",
    "VideoPath",
    "VideoUrl",
    "WikipediaURL",
];

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AdditionalApplication {
    pub id: String,
    pub game_id: String,
    pub name: String,
    pub application_path: String,
    pub command_line: Option<String>,
    pub auto_run_before: bool,
    pub auto_run_after: bool,
    pub wait_for_exit: bool,
    pub use_emulator: bool,
    pub emulator_id: Option<String>,
    pub use_dos_box: bool,
    pub priority: i32,
    pub play_count: u32,
    pub play_time_seconds: u64,
    pub disc: Option<u32>,
    pub side_a: bool,
    pub side_b: bool,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub region: Option<String>,
    pub release_date: Option<String>,
    pub version: Option<String>,
    pub status: Option<String>,
    pub installed: Option<bool>,
    pub last_played: Option<String>,
    pub gog_app_id: Option<String>,
    pub origin_app_id: Option<String>,
    pub origin_install_path: Option<String>,
    pub has_cloud_synced: bool,
}

impl AdditionalApplication {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id.trim().is_empty() {
            return Err(ValidationError::MissingAdditionalApplicationId);
        }
        if self.game_id.trim().is_empty() {
            return Err(ValidationError::MissingAdditionalApplicationGameId {
                id: self.id.clone(),
            });
        }
        if self.name.trim().is_empty() {
            return Err(ValidationError::MissingAdditionalApplicationName {
                id: self.id.clone(),
            });
        }
        Ok(())
    }

    /// Applies the fields represented by an additional application to the
    /// owning game's default launch/version record.
    ///
    /// LaunchBox retains the additional-application row when it is made
    /// default. Game-only identity, presentation, media, and input fields stay
    /// on the game, while the shared launch, version metadata, provider/cloud,
    /// and per-version statistics come from the selected application.
    pub fn apply_as_default_to(&self, game: &Game) -> Game {
        let mut updated = game.clone();
        updated.application_path = self.application_path.clone();
        updated.command_line = self.command_line.clone();
        updated.emulator_id = if self.use_emulator {
            self.emulator_id.clone()
        } else {
            Some(UNASSIGNED_EMULATOR_ID.to_string())
        };
        updated.use_dos_box = self.use_dos_box;
        updated.use_scumm_vm = false;
        updated.developer = self.developer.clone();
        updated.publisher = self.publisher.clone();
        updated.region = self.region.clone();
        updated.release_date = self.release_date.clone();
        updated.version = self.version.clone();
        updated.status = self.status.clone();
        updated.installed = self.installed;
        updated.play_count = self.play_count;
        updated.play_time_seconds = self.play_time_seconds;
        updated.last_played_date = self.last_played.clone();
        updated.gog_app_id = self.gog_app_id.clone();
        updated.origin_app_id = self.origin_app_id.clone();
        updated.origin_install_path = self.origin_install_path.clone();
        updated.has_cloud_synced = self.has_cloud_synced;
        updated
    }

    /// Converts one standalone game into the version/application shape used
    /// by LaunchBox's manual Combine Games operation.
    ///
    /// The protected 13.27 implementation exposes
    /// `AdditionalApplication.GetFromGame(newGameId, game, priority, region,
    /// version)`. Only fields representable by the concrete additional-
    /// application record are copied. Game-only presentation, media, input,
    /// DOSBox mount, and ScummVM fields remain on the selected root game.
    pub fn from_game_version(
        id: impl Into<String>,
        root_game_id: impl Into<String>,
        game: &Game,
        priority: i32,
    ) -> Self {
        let version_label = game
            .version
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                game.region
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
            })
            .unwrap_or(&game.title);
        let use_emulator = game
            .emulator_id
            .as_deref()
            .is_none_or(|id| !is_unassigned_emulator_id(id));
        Self {
            id: id.into(),
            game_id: root_game_id.into(),
            name: format!("Play {version_label} Version..."),
            application_path: game.application_path.clone(),
            command_line: game.command_line.clone(),
            use_emulator,
            emulator_id: if use_emulator {
                game.emulator_id.clone()
            } else {
                None
            },
            use_dos_box: game.use_dos_box,
            priority,
            play_count: game.play_count,
            play_time_seconds: game.play_time_seconds,
            developer: game.developer.clone(),
            publisher: game.publisher.clone(),
            region: game.region.clone(),
            release_date: game.release_date.clone(),
            version: game.version.clone(),
            status: game.status.clone(),
            installed: game.installed,
            last_played: game.last_played_date.clone(),
            gog_app_id: game.gog_app_id.clone(),
            origin_app_id: game.origin_app_id.clone(),
            origin_install_path: game.origin_install_path.clone(),
            has_cloud_synced: game.has_cloud_synced,
            ..Self::default()
        }
    }

    /// Matches the semantic boundary used by Expand Games: launchable
    /// versions are expandable, while automatic helpers and documents are
    /// retained on the original game.
    pub fn is_likely_game_version(&self) -> bool {
        if self.auto_run_before || self.auto_run_after || self.application_path.trim().is_empty() {
            return false;
        }
        let lowercase_path = self.application_path.to_ascii_lowercase();
        ![".pdf", ".txt", ".doc", ".docx", ".htm", ".html", ".url"]
            .iter()
            .any(|extension| lowercase_path.ends_with(extension))
    }
}

impl Game {
    /// Converts one expandable additional application back into a standalone
    /// game while using the original game as the template for fields that an
    /// additional application cannot represent.
    ///
    /// This mirrors the surviving 13.27 signature
    /// `Game.GetFromAdditionalApplication(app, title, region, version,
    /// platform, originalGame)`.
    pub fn from_additional_application_version(
        id: impl Into<String>,
        application: &AdditionalApplication,
        original: &Game,
    ) -> Self {
        let mut game = application.apply_as_default_to(original);
        game.id = id.into();
        game.title = original.title.clone();
        game.platform = original.platform.clone();
        game.clone_of = None;
        game
    }
}

/// Fields exposed by LaunchBox 13.27's additional-application editor.
///
/// Identity, ownership, storefront identifiers, and cloud state are excluded:
/// those values are retained from the source record rather than being
/// user-editable. Persisted paths and timestamps remain lexical LaunchBox
/// strings; the platform layer interprets paths only when launching.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdditionalApplicationEdit {
    pub name: String,
    pub application_path: String,
    pub command_line: Option<String>,
    pub auto_run_before: bool,
    pub auto_run_after: bool,
    pub wait_for_exit: bool,
    pub use_emulator: bool,
    pub emulator_id: Option<String>,
    pub use_dos_box: bool,
    pub priority: i32,
    pub play_count: u32,
    pub play_time_seconds: u64,
    pub disc: Option<u32>,
    pub side_a: bool,
    pub side_b: bool,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub region: Option<String>,
    pub release_date: Option<String>,
    pub version: Option<String>,
    pub status: Option<String>,
    pub installed: Option<bool>,
    pub last_played: Option<String>,
}

impl AdditionalApplicationEdit {
    pub fn apply_to(&self, application: &AdditionalApplication) -> AdditionalApplication {
        AdditionalApplication {
            id: application.id.clone(),
            game_id: application.game_id.clone(),
            name: self.name.clone(),
            application_path: self.application_path.clone(),
            command_line: self.command_line.clone(),
            auto_run_before: self.auto_run_before,
            auto_run_after: self.auto_run_after,
            wait_for_exit: self.wait_for_exit,
            use_emulator: self.use_emulator,
            emulator_id: self.emulator_id.clone(),
            use_dos_box: self.use_dos_box,
            priority: self.priority,
            play_count: self.play_count,
            play_time_seconds: self.play_time_seconds,
            disc: self.disc,
            side_a: self.side_a,
            side_b: self.side_b,
            developer: self.developer.clone(),
            publisher: self.publisher.clone(),
            region: self.region.clone(),
            release_date: self.release_date.clone(),
            version: self.version.clone(),
            status: self.status.clone(),
            installed: self.installed,
            last_played: self.last_played.clone(),
            gog_app_id: application.gog_app_id.clone(),
            origin_app_id: application.origin_app_id.clone(),
            origin_install_path: application.origin_install_path.clone(),
            has_cloud_synced: application.has_cloud_synced,
        }
    }
}

impl From<&AdditionalApplication> for AdditionalApplicationEdit {
    fn from(application: &AdditionalApplication) -> Self {
        Self {
            name: application.name.clone(),
            application_path: application.application_path.clone(),
            command_line: application.command_line.clone(),
            auto_run_before: application.auto_run_before,
            auto_run_after: application.auto_run_after,
            wait_for_exit: application.wait_for_exit,
            use_emulator: application.use_emulator,
            emulator_id: application.emulator_id.clone(),
            use_dos_box: application.use_dos_box,
            priority: application.priority,
            play_count: application.play_count,
            play_time_seconds: application.play_time_seconds,
            disc: application.disc,
            side_a: application.side_a,
            side_b: application.side_b,
            developer: application.developer.clone(),
            publisher: application.publisher.clone(),
            region: application.region.clone(),
            release_date: application.release_date.clone(),
            version: application.version.clone(),
            status: application.status.clone(),
            installed: application.installed,
            last_played: application.last_played.clone(),
        }
    }
}

/// A DOSBox drive or image mount associated with a game. These strings remain
/// in LaunchBox's persisted vocabulary; host-path interpretation belongs to
/// the launch platform layer.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Mount {
    pub game_id: String,
    pub drive_letter: char,
    pub filesystem: String,
    pub mount_type: String,
    pub path: String,
    pub media_type: String,
}

impl Mount {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.game_id.trim().is_empty() {
            return Err(ValidationError::MissingMountGameId);
        }
        if !self.drive_letter.is_ascii_alphabetic() {
            return Err(ValidationError::InvalidMountDriveLetter {
                game_id: self.game_id.clone(),
                drive_letter: self.drive_letter,
            });
        }
        if self.mount_type.trim().is_empty() {
            return Err(ValidationError::MissingMountType {
                game_id: self.game_id.clone(),
                drive_letter: self.drive_letter,
            });
        }
        if self.path.trim().is_empty() {
            return Err(ValidationError::MissingMountPath {
                game_id: self.game_id.clone(),
                drive_letter: self.drive_letter,
            });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AlternateName {
    pub game_id: String,
    pub name: String,
    pub region: Option<String>,
}

/// A LaunchBox name/value metadata entry associated with one game.
///
/// LaunchBox 13.27 exposes this exact shape through `ICustomField` and stores
/// it as a top-level `<CustomField>` record in the owning platform document.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct CustomField {
    pub game_id: String,
    pub name: String,
    pub value: String,
}

impl CustomField {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.game_id.trim().is_empty() {
            return Err(ValidationError::MissingCustomFieldGameId);
        }
        if self.name.trim().is_empty() {
            return Err(ValidationError::MissingCustomFieldName {
                game_id: self.game_id.clone(),
            });
        }
        Ok(())
    }
}

impl AlternateName {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.game_id.trim().is_empty() {
            return Err(ValidationError::MissingAlternateNameGameId);
        }
        if self.name.trim().is_empty() {
            return Err(ValidationError::MissingAlternateName {
                game_id: self.game_id.clone(),
            });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct GameControllerSupport {
    pub controller_id: String,
    pub game_id: String,
    pub support_level: Option<i32>,
}

impl GameControllerSupport {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.controller_id.trim().is_empty() {
            return Err(ValidationError::MissingControllerId);
        }
        if self.game_id.trim().is_empty() {
            return Err(ValidationError::MissingControllerSupportGameId {
                controller_id: self.controller_id.clone(),
            });
        }
        Ok(())
    }
}

/// Persisted fields recovered from LaunchBox 13.27's concrete `GameSave`
/// contract. This excludes runtime-only properties from `GameSaveBase`, such
/// as `IsDirectory` and computed timestamps.
pub const GAME_SAVE_XML_FIELDS: &[&str] = &[
    "GameId",
    "AdditionalApplicationId",
    "EmulatorFileName",
    "EmulatorCore",
    "Title",
    "SaveGroupName",
    "DisplayChipText",
    "SaveGroupId",
    "MatchLineageId",
    "MigrationFamilyId",
    "FilePath",
    "OriginalFileName",
    "Slot",
    "ReportedFileSizeBytes",
    "ReportedLastModifiedUtc",
    "Md5",
];

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct GameSave {
    pub game_id: String,
    pub additional_application_id: Option<String>,
    pub emulator_core: String,
    pub emulator_file_name: String,
    pub title: Option<String>,
    pub save_group_name: Option<String>,
    pub display_chip_text: Option<String>,
    pub save_group_id: Option<String>,
    pub match_lineage_id: Option<String>,
    pub migration_family_id: Option<String>,
    pub file_path: String,
    pub original_file_name: Option<String>,
    pub slot: Option<i32>,
    pub reported_file_size_bytes: Option<i64>,
    pub reported_last_modified_utc: Option<String>,
    pub md5: Option<String>,
}

impl GameSave {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.game_id.trim().is_empty() {
            return Err(ValidationError::MissingGameSaveGameId);
        }
        if self.file_path.trim().is_empty() {
            return Err(ValidationError::MissingGameSavePath {
                game_id: self.game_id.clone(),
            });
        }
        Ok(())
    }
}

/// The user-editable portion of a persisted LaunchBox 13.27 save record.
///
/// File ownership, emulator matching, hashes, timestamps, and migration
/// lineage are deliberately excluded: changing those requires a separate
/// filesystem-aware transaction instead of a metadata-only XML edit.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct GameSaveMetadataEdit {
    pub title: Option<String>,
    pub save_group_name: Option<String>,
    pub save_group_id: Option<String>,
}

impl GameSaveMetadataEdit {
    pub fn apply_to(&self, save: &GameSave) -> GameSave {
        let mut updated = save.clone();
        updated.title = self.title.clone();
        updated.save_group_name = self.save_group_name.clone();
        updated.save_group_id = self.save_group_id.clone();
        updated
    }
}

impl Game {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id.trim().is_empty() {
            return Err(ValidationError::MissingGameId);
        }
        if self.title.trim().is_empty() {
            return Err(ValidationError::MissingGameTitle {
                id: self.id.clone(),
            });
        }
        if self.star_rating > 5 {
            return Err(ValidationError::InvalidStarRating {
                id: self.id.clone(),
                rating: self.star_rating,
            });
        }
        if !self.star_rating_float.is_finite() || !(0.0..=5.0).contains(&self.star_rating_float) {
            return Err(ValidationError::InvalidFloatingStarRating {
                id: self.id.clone(),
                rating: self.star_rating_float,
            });
        }
        if !self.community_star_rating.is_finite()
            || !(0.0..=5.0).contains(&self.community_star_rating)
        {
            return Err(ValidationError::InvalidCommunityStarRating {
                id: self.id.clone(),
                rating: self.community_star_rating,
            });
        }
        Ok(())
    }

    pub fn display_sort_title(&self) -> &str {
        self.sort_title
            .as_deref()
            .filter(|title| !title.trim().is_empty())
            .unwrap_or(&self.title)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlatformLibrary {
    pub name: String,
    pub source_path: PathBuf,
    pub games: Vec<Game>,
    pub additional_applications: Vec<AdditionalApplication>,
    pub mounts: Vec<Mount>,
    pub alternate_names: Vec<AlternateName>,
    pub custom_fields: Vec<CustomField>,
    pub controller_support: Vec<GameControllerSupport>,
    pub game_saves: Vec<GameSave>,
}

impl PlatformLibrary {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.name.trim().is_empty() {
            return Err(ValidationError::MissingPlatformName);
        }
        for game in &self.games {
            game.validate()?;
        }
        for application in &self.additional_applications {
            application.validate()?;
        }
        for mount in &self.mounts {
            mount.validate()?;
        }
        for alternate_name in &self.alternate_names {
            alternate_name.validate()?;
        }
        for custom_field in &self.custom_fields {
            custom_field.validate()?;
        }
        for support in &self.controller_support {
            support.validate()?;
        }
        for save in &self.game_saves {
            save.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum ValidationError {
    #[error("game ID is missing")]
    MissingGameId,
    #[error("game {id} has no title")]
    MissingGameTitle { id: String },
    #[error("game {id} has invalid star rating {rating}; expected 0 through 5")]
    InvalidStarRating { id: String, rating: u8 },
    #[error("game {id} has invalid floating star rating {rating}; expected 0 through 5")]
    InvalidFloatingStarRating { id: String, rating: f64 },
    #[error("game {id} has invalid community star rating {rating}; expected 0 through 5")]
    InvalidCommunityStarRating { id: String, rating: f64 },
    #[error("game {id} enables both DOSBox and ScummVM")]
    ConflictingGameLaunchModes { id: String },
    #[error("game {id} has no application path for an ordinary launch")]
    MissingGameApplicationPath { id: String },
    #[error("DOSBox game {id} has neither an application nor a configuration path")]
    MissingDosBoxApplicationOrConfiguration { id: String },
    #[error("ScummVM game {id} has no game-data folder path")]
    MissingScummVmGameDataPath { id: String },
    #[error("platform name is missing")]
    MissingPlatformName,
    #[error("additional application ID is missing")]
    MissingAdditionalApplicationId,
    #[error("additional application {id} has no game ID")]
    MissingAdditionalApplicationGameId { id: String },
    #[error("additional application {id} has no name")]
    MissingAdditionalApplicationName { id: String },
    #[error("DOSBox mount has no game ID")]
    MissingMountGameId,
    #[error("DOSBox mount for game {game_id} has invalid drive letter {drive_letter:?}")]
    InvalidMountDriveLetter { game_id: String, drive_letter: char },
    #[error("DOSBox mount for game {game_id} drive {drive_letter} has no mount type")]
    MissingMountType { game_id: String, drive_letter: char },
    #[error("DOSBox mount for game {game_id} drive {drive_letter} has no path")]
    MissingMountPath { game_id: String, drive_letter: char },
    #[error("alternate name has no game ID")]
    MissingAlternateNameGameId,
    #[error("alternate name for game {game_id} is empty")]
    MissingAlternateName { game_id: String },
    #[error("custom field has no game ID")]
    MissingCustomFieldGameId,
    #[error("custom field for game {game_id} has no name")]
    MissingCustomFieldName { game_id: String },
    #[error("controller support record has no controller ID")]
    MissingControllerId,
    #[error("controller {controller_id} support record has no game ID")]
    MissingControllerSupportGameId { controller_id: String },
    #[error("game save has no game ID")]
    MissingGameSaveGameId,
    #[error("game save for game {game_id} has no file path")]
    MissingGameSavePath { game_id: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game() -> Game {
        Game {
            id: "game-1".into(),
            title: "Fixture Game".into(),
            platform: "Fixture Platform".into(),
            application_path: "Games/fixture.rom".into(),
            ..Game::default()
        }
    }

    #[test]
    fn sort_title_falls_back_to_title() {
        let mut game = game();
        assert_eq!(game.display_sort_title(), "Fixture Game");
        game.sort_title = Some("Game, Fixture".into());
        assert_eq!(game.display_sort_title(), "Game, Fixture");
    }

    #[test]
    fn validates_star_rating_range() {
        let mut game = game();
        game.star_rating = 6;
        assert_eq!(
            game.validate(),
            Err(ValidationError::InvalidStarRating {
                id: "game-1".into(),
                rating: 6,
            })
        );
    }

    #[test]
    fn recognizes_the_explicit_unassigned_emulator_sentinel() {
        assert!(is_unassigned_emulator_id(UNASSIGNED_EMULATOR_ID));
        assert!(is_unassigned_emulator_id(
            "00000000-0000-0000-0000-000000000000"
        ));
        assert!(!is_unassigned_emulator_id("emulator-id"));
    }

    #[test]
    fn additional_application_default_maps_launch_fields_without_replacing_identity() {
        let original = Game {
            id: "game-1".into(),
            title: "Fixture Game".into(),
            platform: "Fixture Platform".into(),
            application_path: r"Games\Fixture\original.rom".into(),
            notes: Some("keep game-only metadata".into()),
            use_scumm_vm: true,
            scumm_vm_game_type: Some("keep-latent-scummvm-settings".into()),
            ..Game::default()
        };
        let mut application = AdditionalApplication {
            id: "version-1".into(),
            game_id: "game-1".into(),
            name: "Alternate Version".into(),
            application_path: r"Games\Fixture\alternate.rom".into(),
            command_line: Some("--alternate".into()),
            use_emulator: false,
            emulator_id: Some("ignored-while-direct".into()),
            developer: Some("Version Developer".into()),
            play_count: 7,
            ..AdditionalApplication::default()
        };

        let direct = application.apply_as_default_to(&original);
        assert_eq!(direct.id, original.id);
        assert_eq!(direct.title, original.title);
        assert_eq!(direct.platform, original.platform);
        assert_eq!(direct.notes, original.notes);
        assert_eq!(direct.scumm_vm_game_type, original.scumm_vm_game_type);
        assert_eq!(direct.application_path, application.application_path);
        assert_eq!(direct.command_line, application.command_line);
        assert_eq!(direct.emulator_id.as_deref(), Some(UNASSIGNED_EMULATOR_ID));
        assert!(!direct.use_scumm_vm);
        assert_eq!(direct.developer, application.developer);
        assert_eq!(direct.play_count, 7);

        application.use_emulator = true;
        application.emulator_id = None;
        assert_eq!(application.apply_as_default_to(&original).emulator_id, None);

        application.emulator_id = Some("specific-emulator".into());
        assert_eq!(
            application
                .apply_as_default_to(&original)
                .emulator_id
                .as_deref(),
            Some("specific-emulator")
        );
    }

    #[test]
    fn game_and_additional_application_version_conversion_is_reversible() {
        let original = Game {
            id: "regional-game".into(),
            title: "Fixture Adventure".into(),
            platform: "Fixture Console".into(),
            application_path: r"Games\Fixture Console\adventure-eu.rom".into(),
            command_line: Some("--region eu".into()),
            emulator_id: Some("fixture-emulator".into()),
            notes: Some("shared presentation template".into()),
            developer: Some("Fixture Developer".into()),
            region: Some("Europe".into()),
            version: Some("Rev 2".into()),
            play_count: 7,
            play_time_seconds: 31,
            last_played_date: Some("2026-07-23T12:34:56.0000000-07:00".into()),
            ..Game::default()
        };

        let application =
            AdditionalApplication::from_game_version("version-app", "root-game", &original, 2);
        assert_eq!(application.id, "version-app");
        assert_eq!(application.game_id, "root-game");
        assert_eq!(application.name, "Play Rev 2 Version...");
        assert_eq!(application.application_path, original.application_path);
        assert_eq!(application.command_line, original.command_line);
        assert_eq!(application.emulator_id, original.emulator_id);
        assert_eq!(application.region, original.region);
        assert_eq!(application.version, original.version);
        assert_eq!(application.play_count, 7);
        assert!(application.is_likely_game_version());

        let template = Game {
            id: "root-game".into(),
            title: "Fixture Adventure".into(),
            platform: "Fixture Console".into(),
            application_path: r"Games\Fixture Console\adventure-us.rom".into(),
            notes: Some("shared presentation template".into()),
            clone_of: Some("another-game".into()),
            ..Game::default()
        };
        let expanded =
            Game::from_additional_application_version("expanded-game", &application, &template);
        assert_eq!(expanded.id, "expanded-game");
        assert_eq!(expanded.title, template.title);
        assert_eq!(expanded.platform, template.platform);
        assert_eq!(expanded.notes, template.notes);
        assert_eq!(expanded.application_path, original.application_path);
        assert_eq!(expanded.command_line, original.command_line);
        assert_eq!(expanded.emulator_id, original.emulator_id);
        assert_eq!(expanded.region, original.region);
        assert_eq!(expanded.version, original.version);
        assert_eq!(expanded.clone_of, None);

        let automatic = AdditionalApplication {
            auto_run_before: true,
            ..application.clone()
        };
        assert!(!automatic.is_likely_game_version());
        let document = AdditionalApplication {
            application_path: "Docs/guide.pdf".into(),
            ..application
        };
        assert!(!document.is_likely_game_version());
    }
}
