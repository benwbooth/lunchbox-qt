use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

mod catalog;

pub use catalog::*;

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

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct GameSave {
    pub game_id: String,
    pub additional_application_id: Option<String>,
    pub emulator_core: String,
    pub emulator_file_name: String,
    pub file_path: String,
    pub slot: Option<i32>,
    pub title: Option<String>,
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
}
