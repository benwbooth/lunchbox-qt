use crate::input::BigBoxInputAction;
use lb_domain::FrontendSettings;
use std::fmt;

/// A defensive limit for the controller-entered PIN.
///
/// LaunchBox's persisted contract is an arbitrary string, while the recovered
/// BigBox popup can enter only ASCII digits and exposes no visible length
/// rule. The native port keeps the interoperable numeric shape and applies a
/// generous bound so malformed settings cannot create an unbounded UI value.
pub const BIG_BOX_PIN_MAX_DIGITS: usize = 32;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum BigBoxSecurityPermission {
    Exit,
    SetStarRating,
    OpenGameFolder,
    OpenGameImageFolder,
    OpenEmulator,
    FavoriteGames,
    HideGames,
    MarkGamesBroken,
    ModifyProgress,
    Sleep,
    ShutDown,
    Reboot,
    ChangeView,
    ChangeImageType,
    NavigateToDiscoveryCenter,
    ChangeFilterAllGames,
    ChangeFilterPlatforms,
    ChangeFilterPlatformCategories,
    ViewRetroArchNetplayBrowser,
    ChangeFilterPlaylists,
    ChangeFilterGenres,
    ChangeFilterDevelopers,
    ChangeFilterPublishers,
    ChangeFilterSeries,
    ChangeFilterStatuses,
    ChangeFilterSources,
    ChangeFilterRatings,
    ChangeFilterPlayModes,
    ChangeFilterRegions,
    StartThemesDemo,
    Search,
    ViewAchievementProfile,
}

pub const BIG_BOX_SECURITY_PERMISSIONS: &[BigBoxSecurityPermission] = &[
    BigBoxSecurityPermission::Exit,
    BigBoxSecurityPermission::SetStarRating,
    BigBoxSecurityPermission::OpenGameFolder,
    BigBoxSecurityPermission::OpenGameImageFolder,
    BigBoxSecurityPermission::OpenEmulator,
    BigBoxSecurityPermission::FavoriteGames,
    BigBoxSecurityPermission::HideGames,
    BigBoxSecurityPermission::MarkGamesBroken,
    BigBoxSecurityPermission::ModifyProgress,
    BigBoxSecurityPermission::Sleep,
    BigBoxSecurityPermission::ShutDown,
    BigBoxSecurityPermission::Reboot,
    BigBoxSecurityPermission::ChangeView,
    BigBoxSecurityPermission::ChangeImageType,
    BigBoxSecurityPermission::NavigateToDiscoveryCenter,
    BigBoxSecurityPermission::ChangeFilterAllGames,
    BigBoxSecurityPermission::ChangeFilterPlatforms,
    BigBoxSecurityPermission::ChangeFilterPlatformCategories,
    BigBoxSecurityPermission::ViewRetroArchNetplayBrowser,
    BigBoxSecurityPermission::ChangeFilterPlaylists,
    BigBoxSecurityPermission::ChangeFilterGenres,
    BigBoxSecurityPermission::ChangeFilterDevelopers,
    BigBoxSecurityPermission::ChangeFilterPublishers,
    BigBoxSecurityPermission::ChangeFilterSeries,
    BigBoxSecurityPermission::ChangeFilterStatuses,
    BigBoxSecurityPermission::ChangeFilterSources,
    BigBoxSecurityPermission::ChangeFilterRatings,
    BigBoxSecurityPermission::ChangeFilterPlayModes,
    BigBoxSecurityPermission::ChangeFilterRegions,
    BigBoxSecurityPermission::StartThemesDemo,
    BigBoxSecurityPermission::Search,
    BigBoxSecurityPermission::ViewAchievementProfile,
];

impl BigBoxSecurityPermission {
    pub const fn setting_key(self) -> &'static str {
        match self {
            // This is the 13.27 serialized name even though its option label
            // says "Allow Exit While Locked".
            Self::Exit => "AllowExitWhileUnlocked",
            Self::SetStarRating => "AllowSettingStarRatingsWhileLocked",
            Self::OpenGameFolder => "AllowOpeningGameFoldersWhileLocked",
            Self::OpenGameImageFolder => "AllowOpeningGameImageFoldersWhileLocked",
            Self::OpenEmulator => "AllowOpeningEmulatorsWhileLocked",
            Self::FavoriteGames => "AllowFavoritingGamesWhileLocked",
            Self::HideGames => "AllowHidingGamesWhileLocked",
            Self::MarkGamesBroken => "AllowMarkingGamesAsBrokenWhileLocked",
            Self::ModifyProgress => "AllowModifyingProgressWhileLocked",
            Self::Sleep => "AllowSleepWhileLocked",
            Self::ShutDown => "AllowShutDownWhileLocked",
            Self::Reboot => "AllowRebootWhileLocked",
            Self::ChangeView => "AllowChangeViewWhileLocked",
            Self::ChangeImageType => "AllowChangeImageTypeWhileLocked",
            Self::NavigateToDiscoveryCenter => "AllowNavigateToGameDiscoveryCenterWhileLocked",
            Self::ChangeFilterAllGames => "AllowChangeFilterAllGamesWhileLocked",
            Self::ChangeFilterPlatforms => "AllowChangeFilterPlatformsWhileLocked",
            Self::ChangeFilterPlatformCategories => {
                "AllowChangeFilterPlatformCategoriesWhileLocked"
            }
            Self::ViewRetroArchNetplayBrowser => "AllowViewRetroarchNetplayBrowserWhileLocked",
            Self::ChangeFilterPlaylists => "AllowChangeFilterPlaylistsWhileLocked",
            Self::ChangeFilterGenres => "AllowChangeFilterGenresWhileLocked",
            Self::ChangeFilterDevelopers => "AllowChangeFilterDevelopersWhileLocked",
            Self::ChangeFilterPublishers => "AllowChangeFilterPublishersWhileLocked",
            Self::ChangeFilterSeries => "AllowChangeFilterSeriesWhileLocked",
            Self::ChangeFilterStatuses => "AllowChangeFilterStatusesWhileLocked",
            Self::ChangeFilterSources => "AllowChangeFilterSourcesWhileLocked",
            Self::ChangeFilterRatings => "AllowChangeFilterRatingsWhileLocked",
            Self::ChangeFilterPlayModes => "AllowChangeFilterPlayModesWhileLocked",
            Self::ChangeFilterRegions => "AllowChangeFilterRegionsWhileLocked",
            Self::StartThemesDemo => "AllowThemesDemoWhileLocked",
            Self::Search => "AllowSearchWhileLocked",
            Self::ViewAchievementProfile => "AllowViewAchievementProfileWhileLocked",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Exit => "Allow exit",
            Self::SetStarRating => "Allow setting star ratings",
            Self::OpenGameFolder => "Allow opening game folders",
            Self::OpenGameImageFolder => "Allow opening game image folders",
            Self::OpenEmulator => "Allow opening emulators",
            Self::FavoriteGames => "Allow favoriting games",
            Self::HideGames => "Allow hiding games",
            Self::MarkGamesBroken => "Allow marking games as broken",
            Self::ModifyProgress => "Allow updating game progress",
            Self::Sleep => "Allow sleep",
            Self::ShutDown => "Allow shut down",
            Self::Reboot => "Allow reboot",
            Self::ChangeView => "Allow changing views",
            Self::ChangeImageType => "Allow changing image types",
            Self::NavigateToDiscoveryCenter => "Allow Game Discovery Center",
            Self::ChangeFilterAllGames => "Allow All Games filter",
            Self::ChangeFilterPlatforms => "Allow Platforms filter",
            Self::ChangeFilterPlatformCategories => "Allow Platform Categories filter",
            Self::ViewRetroArchNetplayBrowser => "Allow RetroArch Netplay browser",
            Self::ChangeFilterPlaylists => "Allow Playlists filter",
            Self::ChangeFilterGenres => "Allow Genres filter",
            Self::ChangeFilterDevelopers => "Allow Developers filter",
            Self::ChangeFilterPublishers => "Allow Publishers filter",
            Self::ChangeFilterSeries => "Allow Series filter",
            Self::ChangeFilterStatuses => "Allow Statuses filter",
            Self::ChangeFilterSources => "Allow Sources filter",
            Self::ChangeFilterRatings => "Allow Ratings filter",
            Self::ChangeFilterPlayModes => "Allow Play Modes filter",
            Self::ChangeFilterRegions => "Allow Regions filter",
            Self::StartThemesDemo => "Allow Themes Demo",
            Self::Search => "Allow global search",
            Self::ViewAchievementProfile => "Allow achievement profile",
        }
    }

    pub const fn default_allowed(self) -> bool {
        matches!(
            self,
            Self::NavigateToDiscoveryCenter
                | Self::ChangeFilterAllGames
                | Self::ChangeFilterPlatforms
                | Self::ChangeFilterPlatformCategories
                | Self::ViewRetroArchNetplayBrowser
                | Self::ChangeFilterPlaylists
                | Self::ChangeFilterGenres
                | Self::ChangeFilterDevelopers
                | Self::ChangeFilterPublishers
                | Self::ChangeFilterSeries
                | Self::ChangeFilterStatuses
                | Self::ChangeFilterSources
                | Self::ChangeFilterRatings
                | Self::ChangeFilterPlayModes
                | Self::ChangeFilterRegions
                | Self::StartThemesDemo
                | Self::Search
        )
    }

    pub fn from_setting_key(value: &str) -> Option<Self> {
        BIG_BOX_SECURITY_PERMISSIONS
            .iter()
            .copied()
            .find(|permission| permission.setting_key() == value)
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct BigBoxSecurityPolicy {
    pin: String,
    permissions: [bool; BIG_BOX_SECURITY_PERMISSIONS.len()],
    pub show_game_lock_unlock: bool,
}

impl fmt::Debug for BigBoxSecurityPolicy {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BigBoxSecurityPolicy")
            .field("pin_configured", &self.pin_configured())
            .field("permissions", &self.permissions)
            .field("show_game_lock_unlock", &self.show_game_lock_unlock)
            .finish()
    }
}

impl Default for BigBoxSecurityPolicy {
    fn default() -> Self {
        Self {
            pin: String::new(),
            permissions: std::array::from_fn(|index| {
                BIG_BOX_SECURITY_PERMISSIONS[index].default_allowed()
            }),
            show_game_lock_unlock: true,
        }
    }
}

impl BigBoxSecurityPolicy {
    pub fn from_settings(settings: Option<&FrontendSettings>) -> Self {
        let fallback = Self::default();
        let Some(settings) = settings else {
            return fallback;
        };
        let pin = settings
            .get("LockPin")
            .filter(|value| valid_pin(value))
            .unwrap_or_default()
            .to_string();
        Self {
            pin,
            permissions: std::array::from_fn(|index| {
                let permission = BIG_BOX_SECURITY_PERMISSIONS[index];
                settings
                    .get_bool(permission.setting_key())
                    .unwrap_or(permission.default_allowed())
            }),
            show_game_lock_unlock: settings.get_bool("ShowGameLockUnlock").unwrap_or(true),
        }
    }

    pub fn pin_configured(&self) -> bool {
        !self.pin.is_empty()
    }

    pub fn verify_pin(&self, candidate: &str) -> bool {
        if !self.pin_configured() || candidate.len() != self.pin.len() {
            return false;
        }
        self.pin
            .as_bytes()
            .iter()
            .zip(candidate.as_bytes())
            .fold(0_u8, |difference, (expected, actual)| {
                difference | (expected ^ actual)
            })
            == 0
    }

    pub fn permission_allowed(&self, permission: BigBoxSecurityPermission) -> bool {
        BIG_BOX_SECURITY_PERMISSIONS
            .iter()
            .position(|candidate| *candidate == permission)
            .is_some_and(|index| self.permissions[index])
    }

    pub fn set_permission(&mut self, permission: BigBoxSecurityPermission, allowed: bool) {
        if let Some(index) = BIG_BOX_SECURITY_PERMISSIONS
            .iter()
            .position(|candidate| *candidate == permission)
        {
            self.permissions[index] = allowed;
        }
    }

    pub fn set_pin(&mut self, pin: &str) -> Result<(), String> {
        if !valid_pin(pin) {
            return Err(format!(
                "BigBox PIN must contain 1 to {BIG_BOX_PIN_MAX_DIGITS} ASCII digits"
            ));
        }
        self.pin.clear();
        self.pin.push_str(pin);
        Ok(())
    }

    pub fn clear_pin(&mut self) {
        self.pin.clear();
    }

    pub fn allows_input_action(&self, action: BigBoxInputAction) -> bool {
        let permission = match action {
            BigBoxInputAction::Exit => Some(BigBoxSecurityPermission::Exit),
            BigBoxInputAction::SetStarRating => Some(BigBoxSecurityPermission::SetStarRating),
            BigBoxInputAction::SwitchView => Some(BigBoxSecurityPermission::ChangeView),
            BigBoxInputAction::SwitchImageType | BigBoxInputAction::FlipBox => {
                Some(BigBoxSecurityPermission::ChangeImageType)
            }
            BigBoxInputAction::ShowDiscoveryCenter => {
                Some(BigBoxSecurityPermission::NavigateToDiscoveryCenter)
            }
            BigBoxInputAction::ShowAllGames => Some(BigBoxSecurityPermission::ChangeFilterAllGames),
            BigBoxInputAction::ShowPlatforms => {
                Some(BigBoxSecurityPermission::ChangeFilterPlatforms)
            }
            BigBoxInputAction::ShowPlatformCategories => {
                Some(BigBoxSecurityPermission::ChangeFilterPlatformCategories)
            }
            BigBoxInputAction::ShowPlaylists => {
                Some(BigBoxSecurityPermission::ChangeFilterPlaylists)
            }
            BigBoxInputAction::ShowGenres => Some(BigBoxSecurityPermission::ChangeFilterGenres),
            BigBoxInputAction::ShowDevelopers => {
                Some(BigBoxSecurityPermission::ChangeFilterDevelopers)
            }
            BigBoxInputAction::ShowPublishers => {
                Some(BigBoxSecurityPermission::ChangeFilterPublishers)
            }
            BigBoxInputAction::ShowSeries => Some(BigBoxSecurityPermission::ChangeFilterSeries),
            BigBoxInputAction::ShowStatuses => Some(BigBoxSecurityPermission::ChangeFilterStatuses),
            BigBoxInputAction::ShowSources => Some(BigBoxSecurityPermission::ChangeFilterSources),
            BigBoxInputAction::ShowRatings => Some(BigBoxSecurityPermission::ChangeFilterRatings),
            BigBoxInputAction::ShowPlayModes => {
                Some(BigBoxSecurityPermission::ChangeFilterPlayModes)
            }
            BigBoxInputAction::ShowRegions => Some(BigBoxSecurityPermission::ChangeFilterRegions),
            BigBoxInputAction::SwitchTheme => Some(BigBoxSecurityPermission::StartThemesDemo),
            BigBoxInputAction::Search | BigBoxInputAction::Filter => {
                Some(BigBoxSecurityPermission::Search)
            }
            BigBoxInputAction::ShowAchievementProfile => {
                Some(BigBoxSecurityPermission::ViewAchievementProfile)
            }
            _ => None,
        };
        permission.is_none_or(|permission| self.permission_allowed(permission))
    }

    /// Handles both recovered BigBox input actions and menu-only commands.
    /// Unknown command keys fail closed while BigBox is locked.
    pub fn allows_action_key(&self, action_key: &str) -> bool {
        if let Some(action) = BigBoxInputAction::from_key(action_key) {
            return self.allows_input_action(action);
        }
        match action_key {
            "BigBoxFavoriteGames" => {
                self.permission_allowed(BigBoxSecurityPermission::FavoriteGames)
            }
            _ => false,
        }
    }

    pub fn allows_navigation_kind(&self, kind: &str) -> bool {
        let permission = match kind {
            "" | "all" => BigBoxSecurityPermission::ChangeFilterAllGames,
            "platform" => BigBoxSecurityPermission::ChangeFilterPlatforms,
            "category" => BigBoxSecurityPermission::ChangeFilterPlatformCategories,
            "playlist" => BigBoxSecurityPermission::ChangeFilterPlaylists,
            _ => return false,
        };
        self.permission_allowed(permission)
    }
}

pub fn valid_pin(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= BIG_BOX_PIN_MAX_DIGITS
        && value.bytes().all(|byte| byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lb_domain::SettingEntry;

    fn settings(entries: &[(&str, &str)]) -> FrontendSettings {
        FrontendSettings {
            entries: entries
                .iter()
                .map(|(key, value)| SettingEntry {
                    key: (*key).into(),
                    value: (*value).into(),
                })
                .collect(),
            ..FrontendSettings::default()
        }
    }

    #[test]
    fn defaults_match_three_fresh_launchbox_13_27_installations() {
        let policy = BigBoxSecurityPolicy::from_settings(None);
        assert!(!policy.pin_configured());
        assert!(policy.show_game_lock_unlock);
        assert_eq!(BIG_BOX_SECURITY_PERMISSIONS.len(), 32);
        assert_eq!(
            BIG_BOX_SECURITY_PERMISSIONS
                .iter()
                .filter(|permission| policy.permission_allowed(**permission))
                .count(),
            17
        );
        assert!(!policy.permission_allowed(BigBoxSecurityPermission::Exit));
        assert!(policy.permission_allowed(BigBoxSecurityPermission::Search));
        assert!(!policy.permission_allowed(BigBoxSecurityPermission::ViewAchievementProfile));
    }

    #[test]
    fn reads_every_permission_and_redacts_the_pin_from_debug_output() {
        let mut entries = vec![("LockPin", "2580"), ("ShowGameLockUnlock", "false")];
        for permission in BIG_BOX_SECURITY_PERMISSIONS {
            entries.push((
                permission.setting_key(),
                if permission.default_allowed() {
                    "false"
                } else {
                    "true"
                },
            ));
        }
        let policy = BigBoxSecurityPolicy::from_settings(Some(&settings(&entries)));
        assert!(policy.pin_configured());
        assert!(policy.verify_pin("2580"));
        assert!(!policy.verify_pin("2581"));
        assert!(!policy.show_game_lock_unlock);
        for permission in BIG_BOX_SECURITY_PERMISSIONS {
            assert_ne!(
                policy.permission_allowed(*permission),
                permission.default_allowed()
            );
        }
        let debug = format!("{policy:?}");
        assert!(debug.contains("pin_configured: true"));
        assert!(!debug.contains("2580"));
    }

    #[test]
    fn malformed_pin_and_values_do_not_lock_out_the_frontend() {
        for malformed in ["", "12x4", "１２３４", "123456789012345678901234567890123"] {
            let policy = BigBoxSecurityPolicy::from_settings(Some(&settings(&[
                ("LockPin", malformed),
                ("AllowExitWhileUnlocked", "sometimes"),
                ("ShowGameLockUnlock", "yes"),
            ])));
            assert!(!policy.pin_configured());
            assert!(!policy.permission_allowed(BigBoxSecurityPermission::Exit));
            assert!(policy.show_game_lock_unlock);
        }
    }

    #[test]
    fn locked_input_and_navigation_gates_cover_recovered_actions() {
        let mut policy = BigBoxSecurityPolicy::default();
        assert!(policy.allows_input_action(BigBoxInputAction::PlayGame));
        assert!(!policy.allows_input_action(BigBoxInputAction::Exit));
        assert!(!policy.allows_input_action(BigBoxInputAction::FlipBox));
        assert!(policy.allows_input_action(BigBoxInputAction::Search));
        assert!(policy.allows_navigation_kind("platform"));
        assert!(!policy.allows_navigation_kind("future"));

        policy.set_permission(BigBoxSecurityPermission::ChangeFilterPlatforms, false);
        policy.set_permission(BigBoxSecurityPermission::Exit, true);
        assert!(!policy.allows_navigation_kind("platform"));
        assert!(policy.allows_input_action(BigBoxInputAction::Exit));
        assert!(policy.allows_action_key("BigBoxExit"));
        assert!(!policy.allows_action_key("BigBoxFavoriteGames"));
        policy.set_permission(BigBoxSecurityPermission::FavoriteGames, true);
        assert!(policy.allows_action_key("BigBoxFavoriteGames"));
        policy.set_permission(BigBoxSecurityPermission::FavoriteGames, false);
        assert!(!policy.allows_action_key("BigBoxFavoriteGames"));
        assert!(!policy.allows_action_key("FutureMenuCommand"));
    }

    #[test]
    fn pin_replacement_and_clear_are_bounded_and_verifiable() {
        let mut policy = BigBoxSecurityPolicy::default();
        assert!(policy.set_pin("8642").is_ok());
        assert!(policy.pin_configured());
        assert!(policy.verify_pin("8642"));
        assert!(!policy.verify_pin("86420"));
        assert!(policy.set_pin("86 42").is_err());
        assert!(policy.verify_pin("8642"));
        policy.clear_pin();
        assert!(!policy.pin_configured());
        assert!(!policy.verify_pin(""));
    }
}
