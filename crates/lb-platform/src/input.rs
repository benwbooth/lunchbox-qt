use lb_domain::{FrontendSettings, InputBinding};
use std::collections::{BTreeSet, HashMap, VecDeque};

/// The complete BigBox portion of LaunchBox 13.27's persisted `InputAction`
/// enum, in its recovered declaration order.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BigBoxInputAction {
    Search,
    ShowGameDetails,
    SwitchImageType,
    SwitchView,
    Exit,
    ShowImages,
    PlayMusic,
    FlipBox,
    PageDown,
    PageUp,
    PlayGame,
    Back,
    Select,
    NextMusicTrack,
    PreviousMusicTrack,
    ShowDiscoveryCenter,
    ShowAllGames,
    ShowGenres,
    ShowPlatforms,
    ShowPlaylists,
    ShowDevelopers,
    ShowPublishers,
    ShowRatings,
    ShowPlayModes,
    ShowRegions,
    ShowSeries,
    ShowStatuses,
    ShowSources,
    ShowPlatformCategories,
    Filter,
    StartAttractMode,
    WheelSpin,
    SwitchTheme,
    ShowAchievements,
    ShowAchievementProfile,
    SetStarRating,
    ZoomIn,
    ZoomOut,
    LockUnlock,
    ShowPauseScreen,
    ExitGame,
    FocusInterface,
    VolumeUp,
    VolumeDown,
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    ShowHighScores,
    Screenshot,
    ViewSystemMenu,
    OpenIndex,
    RotateModelUp,
    RotateModelDown,
    RotateModelLeft,
    RotateModelRight,
    ShowModel,
    RandomGame,
    StartScreensaver,
}

pub const BIG_BOX_INPUT_ACTIONS: &[BigBoxInputAction] = &[
    BigBoxInputAction::Search,
    BigBoxInputAction::ShowGameDetails,
    BigBoxInputAction::SwitchImageType,
    BigBoxInputAction::SwitchView,
    BigBoxInputAction::Exit,
    BigBoxInputAction::ShowImages,
    BigBoxInputAction::PlayMusic,
    BigBoxInputAction::FlipBox,
    BigBoxInputAction::PageDown,
    BigBoxInputAction::PageUp,
    BigBoxInputAction::PlayGame,
    BigBoxInputAction::Back,
    BigBoxInputAction::Select,
    BigBoxInputAction::NextMusicTrack,
    BigBoxInputAction::PreviousMusicTrack,
    BigBoxInputAction::ShowDiscoveryCenter,
    BigBoxInputAction::ShowAllGames,
    BigBoxInputAction::ShowGenres,
    BigBoxInputAction::ShowPlatforms,
    BigBoxInputAction::ShowPlaylists,
    BigBoxInputAction::ShowDevelopers,
    BigBoxInputAction::ShowPublishers,
    BigBoxInputAction::ShowRatings,
    BigBoxInputAction::ShowPlayModes,
    BigBoxInputAction::ShowRegions,
    BigBoxInputAction::ShowSeries,
    BigBoxInputAction::ShowStatuses,
    BigBoxInputAction::ShowSources,
    BigBoxInputAction::ShowPlatformCategories,
    BigBoxInputAction::Filter,
    BigBoxInputAction::StartAttractMode,
    BigBoxInputAction::WheelSpin,
    BigBoxInputAction::SwitchTheme,
    BigBoxInputAction::ShowAchievements,
    BigBoxInputAction::ShowAchievementProfile,
    BigBoxInputAction::SetStarRating,
    BigBoxInputAction::ZoomIn,
    BigBoxInputAction::ZoomOut,
    BigBoxInputAction::LockUnlock,
    BigBoxInputAction::ShowPauseScreen,
    BigBoxInputAction::ExitGame,
    BigBoxInputAction::FocusInterface,
    BigBoxInputAction::VolumeUp,
    BigBoxInputAction::VolumeDown,
    BigBoxInputAction::NavigateUp,
    BigBoxInputAction::NavigateDown,
    BigBoxInputAction::NavigateLeft,
    BigBoxInputAction::NavigateRight,
    BigBoxInputAction::ShowHighScores,
    BigBoxInputAction::Screenshot,
    BigBoxInputAction::ViewSystemMenu,
    BigBoxInputAction::OpenIndex,
    BigBoxInputAction::RotateModelUp,
    BigBoxInputAction::RotateModelDown,
    BigBoxInputAction::RotateModelLeft,
    BigBoxInputAction::RotateModelRight,
    BigBoxInputAction::ShowModel,
    BigBoxInputAction::RandomGame,
    BigBoxInputAction::StartScreensaver,
];

impl BigBoxInputAction {
    pub const fn key(self) -> &'static str {
        match self {
            Self::Search => "BigBoxSearch",
            Self::ShowGameDetails => "BigBoxShowGameDetails",
            Self::SwitchImageType => "BigBoxSwitchImageType",
            Self::SwitchView => "BigBoxSwitchView",
            Self::Exit => "BigBoxExit",
            Self::ShowImages => "BigBoxShowImages",
            Self::PlayMusic => "BigBoxPlayMusic",
            Self::FlipBox => "BigBoxFlipBox",
            Self::PageDown => "BigBoxPageDown",
            Self::PageUp => "BigBoxPageUp",
            Self::PlayGame => "BigBoxPlayGame",
            Self::Back => "BigBoxBack",
            Self::Select => "BigBoxSelect",
            Self::NextMusicTrack => "BigBoxNextMusicTrack",
            Self::PreviousMusicTrack => "BigBoxPreviousMusicTrack",
            Self::ShowDiscoveryCenter => "BigBoxShowDiscoveryCenter",
            Self::ShowAllGames => "BigBoxShowAllGames",
            Self::ShowGenres => "BigBoxShowGenres",
            Self::ShowPlatforms => "BigBoxShowPlatforms",
            Self::ShowPlaylists => "BigBoxShowPlaylists",
            Self::ShowDevelopers => "BigBoxShowDevelopers",
            Self::ShowPublishers => "BigBoxShowPublishers",
            Self::ShowRatings => "BigBoxShowRatings",
            Self::ShowPlayModes => "BigBoxShowPlayModes",
            Self::ShowRegions => "BigBoxShowRegions",
            Self::ShowSeries => "BigBoxShowSeries",
            Self::ShowStatuses => "BigBoxShowStatuses",
            Self::ShowSources => "BigBoxShowSources",
            Self::ShowPlatformCategories => "BigBoxShowPlatformCategories",
            Self::Filter => "BigBoxFilter",
            Self::StartAttractMode => "BigBoxStartAttractMode",
            Self::WheelSpin => "BigBoxWheelSpin",
            Self::SwitchTheme => "BigBoxSwitchTheme",
            Self::ShowAchievements => "BigBoxShowAchievements",
            Self::ShowAchievementProfile => "BigBoxShowAchievementProfile",
            Self::SetStarRating => "BigBoxSetStarRating",
            Self::ZoomIn => "BigBoxZoomIn",
            Self::ZoomOut => "BigBoxZoomOut",
            Self::LockUnlock => "BigBoxLockUnlock",
            Self::ShowPauseScreen => "BigBoxShowPauseScreen",
            Self::ExitGame => "BigBoxExitGame",
            Self::FocusInterface => "BigBoxFocusInterface",
            Self::VolumeUp => "BigBoxVolumeUp",
            Self::VolumeDown => "BigBoxVolumeDown",
            Self::NavigateUp => "BigBoxNavigateUp",
            Self::NavigateDown => "BigBoxNavigateDown",
            Self::NavigateLeft => "BigBoxNavigateLeft",
            Self::NavigateRight => "BigBoxNavigateRight",
            Self::ShowHighScores => "BigBoxShowHighScores",
            Self::Screenshot => "BigBoxScreenshot",
            Self::ViewSystemMenu => "BigBoxViewSystemMenu",
            Self::OpenIndex => "BigBoxOpenIndex",
            Self::RotateModelUp => "BigBoxRotateModelUp",
            Self::RotateModelDown => "BigBoxRotateModelDown",
            Self::RotateModelLeft => "BigBoxRotateModelLeft",
            Self::RotateModelRight => "BigBoxRotateModelRight",
            Self::ShowModel => "BigBoxShowModel",
            Self::RandomGame => "BigBoxRandomGame",
            Self::StartScreensaver => "BigBoxStartScreensaver",
        }
    }

    pub fn from_key(value: &str) -> Option<Self> {
        BIG_BOX_INPUT_ACTIONS
            .iter()
            .copied()
            .find(|action| action.key() == value)
    }

    pub fn keyboard_slot_count(self) -> usize {
        self.keyboard_mapping().map_or(0, |mapping| mapping.slots)
    }

    pub fn keyboard_setting_key(self, slot: usize) -> Option<String> {
        self.keyboard_mapping()?.key(slot)
    }

    fn keyboard_mapping(self) -> Option<KeyboardMapping> {
        let mapping = match self {
            Self::Search => KeyboardMapping::standard("KeyboardSearch"),
            Self::ShowGameDetails => KeyboardMapping::standard("KeyboardShowGameDetailsScreen"),
            Self::SwitchImageType => KeyboardMapping::standard("KeyboardSwitchImageType"),
            Self::SwitchView => KeyboardMapping::standard("KeyboardSwitchView"),
            Self::Exit => KeyboardMapping::standard("KeyboardExit"),
            Self::ShowImages => KeyboardMapping::standard("KeyboardViewImages"),
            Self::PlayMusic => KeyboardMapping::standard("KeyboardPlayMusic"),
            Self::FlipBox => KeyboardMapping::standard("KeyboardFlipBox"),
            Self::PageDown => KeyboardMapping::standard("KeyboardPageDown"),
            Self::PageUp => KeyboardMapping::standard("KeyboardPageUp"),
            Self::PlayGame => KeyboardMapping::standard("KeyboardPlay"),
            Self::Back => KeyboardMapping::standard("KeyboardBack"),
            Self::Select => KeyboardMapping::standard("KeyboardSelect"),
            Self::NextMusicTrack => KeyboardMapping::standard("KeyboardNextMusicTrack"),
            Self::PreviousMusicTrack => KeyboardMapping::standard("KeyboardPreviousMusicTrack"),
            Self::ShowDiscoveryCenter => KeyboardMapping::standard("KeyboardShowDiscoveryCenter"),
            Self::ShowAllGames => KeyboardMapping::standard("KeyboardShowAllGames"),
            Self::ShowGenres => KeyboardMapping::standard("KeyboardShowGenres"),
            Self::ShowPlatforms => KeyboardMapping::standard("KeyboardShowPlatforms"),
            Self::ShowPlaylists => KeyboardMapping::standard("KeyboardShowPlaylists"),
            Self::ShowDevelopers => KeyboardMapping::standard("KeyboardShowDevelopers"),
            Self::ShowPublishers => KeyboardMapping::standard("KeyboardShowPublishers"),
            Self::ShowRatings => KeyboardMapping::standard("KeyboardShowRatings"),
            Self::ShowPlayModes => KeyboardMapping::standard("KeyboardShowPlayModes"),
            Self::ShowRegions => KeyboardMapping::standard("KeyboardShowRegions"),
            Self::ShowSeries => KeyboardMapping::standard("KeyboardShowSeries"),
            Self::ShowStatuses => KeyboardMapping::standard("KeyboardShowStatuses"),
            Self::ShowSources => KeyboardMapping::standard("KeyboardShowSources"),
            Self::ShowPlatformCategories => {
                KeyboardMapping::standard("KeyboardShowPlatformCategories")
            }
            Self::Filter => KeyboardMapping::standard("KeyboardFilter"),
            Self::StartAttractMode => KeyboardMapping::standard("KeyboardStartAttractMode"),
            Self::WheelSpin => KeyboardMapping::standard("KeyboardWheelSpin"),
            Self::SwitchTheme => KeyboardMapping::standard("KeyboardSwitchTheme"),
            Self::ShowAchievements => KeyboardMapping::standard("KeyboardShowAchievements"),
            Self::ShowAchievementProfile => {
                KeyboardMapping::standard("KeyboardShowAchievementProfile")
            }
            Self::SetStarRating => KeyboardMapping::standard("KeyboardSetStarRating"),
            Self::ZoomIn => KeyboardMapping::standard("KeyboardPdfReaderZoomIn"),
            Self::ZoomOut => KeyboardMapping::standard("KeyboardPdfReaderZoomOut"),
            Self::LockUnlock => KeyboardMapping::standard("KeyboardLockUnlock"),
            Self::ShowPauseScreen => KeyboardMapping::single("KeyboardGamePause"),
            Self::VolumeUp => KeyboardMapping::standard("KeyboardVolumeUp"),
            Self::VolumeDown => KeyboardMapping::standard("KeyboardVolumeDown"),
            Self::NavigateUp => KeyboardMapping::standard("KeyboardUp"),
            Self::NavigateDown => KeyboardMapping::standard("KeyboardDown"),
            Self::NavigateLeft => KeyboardMapping::standard("KeyboardLeft"),
            Self::NavigateRight => KeyboardMapping::standard("KeyboardRight"),
            Self::ShowHighScores => KeyboardMapping::standard("KeyboardShowHighScores"),
            Self::ViewSystemMenu => KeyboardMapping::numbered("KeyboardViewSystemMenu"),
            Self::OpenIndex => KeyboardMapping::numbered("KeyboardOpenIndex"),
            Self::RotateModelUp => KeyboardMapping::numbered("KeyboardRotateModelUp"),
            Self::RotateModelDown => KeyboardMapping::numbered("KeyboardRotateModelDown"),
            Self::RotateModelLeft => KeyboardMapping::numbered("KeyboardRotateModelLeft"),
            Self::RotateModelRight => KeyboardMapping::numbered("KeyboardRotateModelRight"),
            Self::ShowModel => KeyboardMapping::standard("KeyboardViewModel"),
            Self::RandomGame => KeyboardMapping::standard("KeyboardSelectRandomGame"),
            Self::StartScreensaver => KeyboardMapping::standard("KeyboardStartScreensaver"),
            Self::ExitGame | Self::FocusInterface | Self::Screenshot => return None,
        };
        Some(mapping)
    }

    const fn default_wpf_key(self) -> i64 {
        match self {
            Self::NavigateLeft => 23,
            Self::NavigateUp => 24,
            Self::NavigateRight => 25,
            Self::NavigateDown => 26,
            Self::Select => 6,
            Self::Back => 13,
            Self::PlayGame => 59,
            Self::PageUp => 19,
            Self::PageDown => 20,
            Self::FlipBox => 49,
            Self::PlayMusic => 56,
            Self::ShowImages => 52,
            Self::Exit => 67,
            Self::VolumeUp | Self::ZoomIn => 85,
            Self::VolumeDown => 57,
            Self::ZoomOut => 87,
            _ => 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct KeyboardMapping {
    base: &'static str,
    first_numbered: bool,
    slots: usize,
}

impl KeyboardMapping {
    const fn standard(base: &'static str) -> Self {
        Self {
            base,
            first_numbered: false,
            slots: 4,
        }
    }

    const fn numbered(base: &'static str) -> Self {
        Self {
            base,
            first_numbered: true,
            slots: 4,
        }
    }

    const fn single(base: &'static str) -> Self {
        Self {
            base,
            first_numbered: false,
            slots: 1,
        }
    }

    fn key(self, slot: usize) -> Option<String> {
        if slot >= self.slots {
            return None;
        }
        let suffix = if slot == 0 && !self.first_numbered {
            String::new()
        } else {
            (slot + 1).to_string()
        };
        Some(format!("{}{suffix}", self.base))
    }
}

/// LaunchBox's semantic controller names. They are deliberately independent
/// from platform-specific scan codes and physical device paths.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ControllerBinding {
    Button(u8),
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    LeftStickUp,
    LeftStickDown,
    LeftStickLeft,
    LeftStickRight,
    RightStickUp,
    RightStickDown,
    RightStickLeft,
    RightStickRight,
    TriggerLeft,
    TriggerRight,
}

impl ControllerBinding {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "DPadUp" => Some(Self::DPadUp),
            "DPadDown" => Some(Self::DPadDown),
            "DPadLeft" => Some(Self::DPadLeft),
            "DPadRight" => Some(Self::DPadRight),
            "LeftStickUp" => Some(Self::LeftStickUp),
            "LeftStickDown" => Some(Self::LeftStickDown),
            "LeftStickLeft" => Some(Self::LeftStickLeft),
            "LeftStickRight" => Some(Self::LeftStickRight),
            "RightStickUp" => Some(Self::RightStickUp),
            "RightStickDown" => Some(Self::RightStickDown),
            "RightStickLeft" => Some(Self::RightStickLeft),
            "RightStickRight" => Some(Self::RightStickRight),
            "TriggerLeft" => Some(Self::TriggerLeft),
            "TriggerRight" => Some(Self::TriggerRight),
            value => value
                .strip_prefix("Button")
                .and_then(|number| number.parse::<u8>().ok())
                .filter(|number| (1..=32).contains(number))
                .map(Self::Button),
        }
    }

    pub fn key(self) -> String {
        match self {
            Self::Button(number) => format!("Button{number}"),
            Self::DPadUp => "DPadUp".into(),
            Self::DPadDown => "DPadDown".into(),
            Self::DPadLeft => "DPadLeft".into(),
            Self::DPadRight => "DPadRight".into(),
            Self::LeftStickUp => "LeftStickUp".into(),
            Self::LeftStickDown => "LeftStickDown".into(),
            Self::LeftStickLeft => "LeftStickLeft".into(),
            Self::LeftStickRight => "LeftStickRight".into(),
            Self::RightStickUp => "RightStickUp".into(),
            Self::RightStickDown => "RightStickDown".into(),
            Self::RightStickLeft => "RightStickLeft".into(),
            Self::RightStickRight => "RightStickRight".into(),
            Self::TriggerLeft => "TriggerLeft".into(),
            Self::TriggerRight => "TriggerRight".into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ControllerRule {
    action: BigBoxInputAction,
    binding: ControllerBinding,
    hold: Option<ControllerBinding>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct KeyboardSlot {
    wpf_key: i64,
    sequence: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BigBoxInputPolicy {
    pub gamepad_enabled: bool,
    pub use_all_controllers: bool,
    keyboard: HashMap<BigBoxInputAction, [KeyboardSlot; 4]>,
    controller_rules: Vec<ControllerRule>,
    pub unsupported_controller_rule_count: usize,
}

impl BigBoxInputPolicy {
    pub fn from_settings(settings: Option<&FrontendSettings>, bindings: &[InputBinding]) -> Self {
        Self::from_settings_with_controller_defaults(settings, bindings, true)
    }

    /// Builds a policy from an explicitly persisted controller rule set. Unlike
    /// `from_settings`, an empty slice remains empty instead of receiving fresh
    /// installation defaults.
    pub fn from_persisted_settings(
        settings: Option<&FrontendSettings>,
        bindings: &[InputBinding],
    ) -> Self {
        Self::from_settings_with_controller_defaults(settings, bindings, false)
    }

    fn from_settings_with_controller_defaults(
        settings: Option<&FrontendSettings>,
        bindings: &[InputBinding],
        use_controller_defaults: bool,
    ) -> Self {
        let gamepad_enabled = setting_bool(settings, "EnableGamepad", true);
        let use_all_controllers = setting_bool(settings, "UseAllControllers", false);
        let mut keyboard = HashMap::new();
        for action in BIG_BOX_INPUT_ACTIONS {
            let mut slots: [KeyboardSlot; 4] = std::array::from_fn(|_| KeyboardSlot::default());
            if let Some(mapping) = action.keyboard_mapping() {
                for (slot, keyboard_slot) in slots.iter_mut().enumerate() {
                    let Some(key) = mapping.key(slot) else {
                        continue;
                    };
                    let default = if slot == 0 {
                        action.default_wpf_key()
                    } else {
                        0
                    };
                    let value = setting_i64(settings, &key, default);
                    keyboard_slot.wpf_key = value;
                    keyboard_slot.sequence = wpf_key_to_qt_portable_text(value);
                }
            }
            keyboard.insert(*action, slots);
        }

        let mut unsupported_controller_rule_count = 0;
        let mut controller_rules = Vec::new();
        for binding in bindings {
            let Some(action) = BigBoxInputAction::from_key(&binding.input_action) else {
                if binding.input_action.starts_with("BigBox") {
                    unsupported_controller_rule_count += 1;
                }
                continue;
            };
            let Some(controller_binding) =
                ControllerBinding::parse(binding.controller_binding.trim())
            else {
                unsupported_controller_rule_count += 1;
                continue;
            };
            let hold = match binding.controller_hold_binding.trim() {
                "" | "None" => None,
                value => match ControllerBinding::parse(value) {
                    Some(value) => Some(value),
                    None => {
                        unsupported_controller_rule_count += 1;
                        continue;
                    }
                },
            };
            let rule = ControllerRule {
                action,
                binding: controller_binding,
                hold,
            };
            if !controller_rules.contains(&rule) {
                controller_rules.push(rule);
            }
        }
        if controller_rules.is_empty() && use_controller_defaults {
            controller_rules.extend(default_big_box_controller_rules());
        }

        Self {
            gamepad_enabled,
            use_all_controllers,
            keyboard,
            controller_rules,
            unsupported_controller_rule_count,
        }
    }

    pub fn keyboard_sequence(&self, action: BigBoxInputAction, slot: usize) -> Option<&str> {
        self.keyboard
            .get(&action)
            .and_then(|slots| slots.get(slot))
            .and_then(|slot| slot.sequence.as_deref())
    }

    pub fn keyboard_wpf_key(&self, action: BigBoxInputAction, slot: usize) -> Option<i64> {
        (slot < action.keyboard_slot_count())
            .then(|| {
                self.keyboard
                    .get(&action)
                    .and_then(|slots| slots.get(slot))
                    .map(|slot| slot.wpf_key)
            })
            .flatten()
    }

    pub fn controller_rule_count(&self) -> usize {
        self.controller_rules.len()
    }

    pub fn controller_rule(
        &self,
        index: usize,
    ) -> Option<(
        BigBoxInputAction,
        ControllerBinding,
        Option<ControllerBinding>,
    )> {
        self.controller_rules
            .get(index)
            .map(|rule| (rule.action, rule.binding, rule.hold))
    }

    pub fn keyboard_bindings(&self) -> Vec<(String, Vec<BigBoxInputAction>)> {
        let mut bindings = Vec::<(String, Vec<BigBoxInputAction>)>::new();
        for action in BIG_BOX_INPUT_ACTIONS {
            for sequence in self
                .keyboard
                .get(action)
                .into_iter()
                .flat_map(|slots| slots.iter())
                .filter_map(|slot| slot.sequence.as_ref())
            {
                if let Some((_, actions)) = bindings
                    .iter_mut()
                    .find(|(candidate, _)| candidate == sequence)
                {
                    if !actions.contains(action) {
                        actions.push(*action);
                    }
                } else {
                    bindings.push((sequence.clone(), vec![*action]));
                }
            }
        }
        bindings
    }
}

impl Default for BigBoxInputPolicy {
    fn default() -> Self {
        Self::from_settings(None, &[])
    }
}

fn setting_bool(settings: Option<&FrontendSettings>, key: &str, default: bool) -> bool {
    settings
        .and_then(|settings| settings.get(key))
        .and_then(|value| match value.trim() {
            value if value.eq_ignore_ascii_case("true") => Some(true),
            value if value.eq_ignore_ascii_case("false") => Some(false),
            _ => None,
        })
        .unwrap_or(default)
}

fn setting_i64(settings: Option<&FrontendSettings>, key: &str, default: i64) -> i64 {
    settings
        .and_then(|settings| settings.get(key))
        .and_then(|value| value.trim().parse::<i64>().ok())
        .unwrap_or(default)
}

fn default_big_box_controller_rules() -> Vec<ControllerRule> {
    use BigBoxInputAction as Action;
    use ControllerBinding as Binding;
    [
        (Action::Back, Binding::Button(2)),
        (Action::NavigateDown, Binding::LeftStickDown),
        (Action::NavigateDown, Binding::DPadDown),
        (Action::NavigateLeft, Binding::DPadLeft),
        (Action::NavigateLeft, Binding::LeftStickLeft),
        (Action::NavigateRight, Binding::LeftStickRight),
        (Action::NavigateRight, Binding::DPadRight),
        (Action::NavigateUp, Binding::LeftStickUp),
        (Action::NavigateUp, Binding::DPadUp),
        (Action::PageDown, Binding::Button(6)),
        (Action::PageUp, Binding::Button(5)),
        (Action::PlayGame, Binding::Button(3)),
        (Action::RotateModelDown, Binding::RightStickDown),
        (Action::RotateModelLeft, Binding::RightStickLeft),
        (Action::RotateModelRight, Binding::RightStickRight),
        (Action::RotateModelUp, Binding::RightStickUp),
        (Action::Select, Binding::Button(1)),
        (Action::ShowImages, Binding::Button(4)),
    ]
    .into_iter()
    .map(|(action, binding)| ControllerRule {
        action,
        binding,
        hold: None,
    })
    .collect()
}

/// Convert WPF's persisted `System.Windows.Input.Key` integer to a Qt
/// portable key-sequence string. `None`/zero and unknown future values are
/// intentionally left unbound.
pub fn wpf_key_to_qt_portable_text(value: i64) -> Option<String> {
    let fixed = match value {
        1 => "Cancel",
        2 => "Backspace",
        3 => "Tab",
        5 => "Clear",
        6 => "Return",
        7 => "Pause",
        8 => "CapsLock",
        13 => "Esc",
        18 => "Space",
        19 => "PgUp",
        20 => "PgDown",
        21 => "End",
        22 => "Home",
        23 => "Left",
        24 => "Up",
        25 => "Right",
        26 => "Down",
        30 => "Print",
        31 => "Ins",
        32 => "Del",
        33 => "Help",
        70 | 71 => "Meta",
        72 => "Menu",
        84 => "Num+*",
        85 => "Num++",
        86 => "Num+,",
        87 => "Num+-",
        88 => "Num+.",
        89 => "Num+/",
        114 => "NumLock",
        115 => "ScrollLock",
        116 | 117 => "Shift",
        118 | 119 => "Ctrl",
        120 | 121 => "Alt",
        122 => "Back",
        123 => "Forward",
        124 => "Refresh",
        125 => "Stop",
        126 => "Search",
        127 => "Favorites",
        128 => "Home Page",
        129 => "Volume Mute",
        130 => "Volume Down",
        131 => "Volume Up",
        132 => "Media Next",
        133 => "Media Previous",
        134 => "Media Stop",
        135 => "Media Play",
        136 => "Launch Mail",
        137 => "Launch Media",
        138 => "Launch 0",
        139 => "Launch 1",
        140 => ";",
        141 => "=",
        142 => ",",
        143 => "-",
        144 => ".",
        145 => "/",
        146 => "`",
        149 => "[",
        150 => "\\",
        151 => "]",
        152 => "'",
        171 => "Clear",
        _ => "",
    };
    if !fixed.is_empty() {
        return Some(fixed.to_string());
    }
    match value {
        34..=43 => char::from_u32(u32::try_from(value - 34).ok()? + u32::from(b'0'))
            .map(|value| value.to_string()),
        44..=69 => char::from_u32(u32::try_from(value - 44).ok()? + u32::from(b'A'))
            .map(|value| value.to_string()),
        74..=83 => Some(format!("Num+{}", value - 74)),
        90..=113 => Some(format!("F{}", value - 89)),
        _ => None,
    }
}

/// Convert a Qt `KeyEvent.key` value to the persisted WPF `Key` integer used
/// by LaunchBox. The conversion is intentionally based on logical Qt keys, not
/// operating-system scan codes, so the editor behaves the same on Linux,
/// Windows, and macOS.
pub fn qt_key_to_wpf_key(value: i32) -> Option<i64> {
    qt_key_to_wpf_key_with_modifiers(value, 0)
}

pub fn qt_key_to_wpf_key_with_modifiers(value: i32, modifiers: i32) -> Option<i64> {
    const QT_KEY_ESCAPE: i32 = 0x0100_0000;
    const QT_KEY_TAB: i32 = 0x0100_0001;
    const QT_KEY_BACKTAB: i32 = 0x0100_0002;
    const QT_KEY_BACKSPACE: i32 = 0x0100_0003;
    const QT_KEY_RETURN: i32 = 0x0100_0004;
    const QT_KEY_ENTER: i32 = 0x0100_0005;
    const QT_KEY_INSERT: i32 = 0x0100_0006;
    const QT_KEY_DELETE: i32 = 0x0100_0007;
    const QT_KEY_PAUSE: i32 = 0x0100_0008;
    const QT_KEY_PRINT: i32 = 0x0100_0009;
    const QT_KEY_CLEAR: i32 = 0x0100_000b;
    const QT_KEY_HOME: i32 = 0x0100_0010;
    const QT_KEY_END: i32 = 0x0100_0011;
    const QT_KEY_LEFT: i32 = 0x0100_0012;
    const QT_KEY_UP: i32 = 0x0100_0013;
    const QT_KEY_RIGHT: i32 = 0x0100_0014;
    const QT_KEY_DOWN: i32 = 0x0100_0015;
    const QT_KEY_PAGE_UP: i32 = 0x0100_0016;
    const QT_KEY_PAGE_DOWN: i32 = 0x0100_0017;
    const QT_KEY_SHIFT: i32 = 0x0100_0020;
    const QT_KEY_CONTROL: i32 = 0x0100_0021;
    const QT_KEY_META: i32 = 0x0100_0022;
    const QT_KEY_ALT: i32 = 0x0100_0023;
    const QT_KEY_CAPS_LOCK: i32 = 0x0100_0024;
    const QT_KEY_NUM_LOCK: i32 = 0x0100_0025;
    const QT_KEY_SCROLL_LOCK: i32 = 0x0100_0026;
    const QT_KEY_F1: i32 = 0x0100_0030;
    const QT_KEY_F24: i32 = QT_KEY_F1 + 23;
    const QT_KEYPAD_MODIFIER: i32 = 0x2000_0000;

    if modifiers & QT_KEYPAD_MODIFIER != 0 {
        return match value {
            0x30..=0x39 => Some(i64::from(value - 0x30 + 74)),
            0x2a => Some(84),
            0x2b => Some(85),
            0x2c => Some(86),
            0x2d => Some(87),
            0x2e => Some(88),
            0x2f => Some(89),
            _ => None,
        };
    }

    let fixed = match value {
        QT_KEY_ESCAPE => 13,
        QT_KEY_TAB | QT_KEY_BACKTAB => 3,
        QT_KEY_BACKSPACE => 2,
        QT_KEY_RETURN | QT_KEY_ENTER => 6,
        QT_KEY_INSERT => 31,
        QT_KEY_DELETE => 32,
        QT_KEY_PAUSE => 7,
        QT_KEY_PRINT => 30,
        QT_KEY_CLEAR => 5,
        QT_KEY_HOME => 22,
        QT_KEY_END => 21,
        QT_KEY_LEFT => 23,
        QT_KEY_UP => 24,
        QT_KEY_RIGHT => 25,
        QT_KEY_DOWN => 26,
        QT_KEY_PAGE_UP => 19,
        QT_KEY_PAGE_DOWN => 20,
        QT_KEY_SHIFT => 116,
        QT_KEY_CONTROL => 118,
        QT_KEY_META => 70,
        QT_KEY_ALT => 120,
        QT_KEY_CAPS_LOCK => 8,
        QT_KEY_NUM_LOCK => 114,
        QT_KEY_SCROLL_LOCK => 115,
        0x20 => 18,
        0x3b | 0x3a => 140,
        0x3d | 0x2b => 141,
        0x2c | 0x3c => 142,
        0x2d | 0x5f => 143,
        0x2e | 0x3e => 144,
        0x2f | 0x3f => 145,
        0x60 | 0x7e => 146,
        0x5b | 0x7b => 149,
        0x5c | 0x7c => 150,
        0x5d | 0x7d => 151,
        0x27 | 0x22 => 152,
        _ => 0,
    };
    if fixed != 0 {
        return Some(fixed);
    }
    match value {
        0x30..=0x39 => Some(i64::from(value - 0x30 + 34)),
        0x41..=0x5a => Some(i64::from(value - 0x41 + 44)),
        QT_KEY_F1..=QT_KEY_F24 => Some(i64::from(value - QT_KEY_F1 + 90)),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GamepadInputEvent {
    Pressed {
        device_id: u64,
        binding: ControllerBinding,
    },
    Released {
        device_id: u64,
        binding: ControllerBinding,
    },
    Disconnected {
        device_id: u64,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GamepadBackendStatus {
    NotInitialized,
    Ready,
    Unavailable(String),
}

impl GamepadBackendStatus {
    pub fn label(&self) -> String {
        match self {
            Self::NotInitialized => "Not initialized".into(),
            Self::Ready => "Ready".into(),
            Self::Unavailable(error) => format!("Unavailable: {error}"),
        }
    }
}

#[derive(Default)]
struct ControllerRouter {
    pressed: HashMap<u64, BTreeSet<ControllerBinding>>,
    active_device: Option<u64>,
}

impl ControllerRouter {
    fn reset(&mut self) {
        self.pressed.clear();
        self.active_device = None;
    }

    fn route(
        &mut self,
        policy: &BigBoxInputPolicy,
        event: GamepadInputEvent,
    ) -> Vec<BigBoxInputAction> {
        let (device_id, binding, pressed) = match event {
            GamepadInputEvent::Pressed { device_id, binding } => (device_id, binding, true),
            GamepadInputEvent::Released { device_id, binding } => (device_id, binding, false),
            GamepadInputEvent::Disconnected { device_id } => {
                self.pressed.remove(&device_id);
                if self.active_device == Some(device_id) {
                    self.active_device = None;
                }
                return Vec::new();
            }
        };
        if !policy.gamepad_enabled {
            return Vec::new();
        }
        if !policy.use_all_controllers {
            if self.active_device.is_none() && pressed {
                self.active_device = Some(device_id);
            }
            if self.active_device != Some(device_id) {
                return Vec::new();
            }
        }

        let device_pressed = self.pressed.entry(device_id).or_default();
        if pressed {
            if !device_pressed.insert(binding) {
                return Vec::new();
            }
        } else {
            device_pressed.remove(&binding);
            return Vec::new();
        }

        policy
            .controller_rules
            .iter()
            .filter(|rule| {
                rule.binding == binding
                    && rule.hold.is_none_or(|hold| device_pressed.contains(&hold))
            })
            .map(|rule| rule.action)
            .collect()
    }
}

pub struct BigBoxInputEngine {
    policy: BigBoxInputPolicy,
    router: ControllerRouter,
    native: NativeGamepadInput,
    pending_actions: VecDeque<BigBoxInputAction>,
}

impl BigBoxInputEngine {
    pub fn new(policy: BigBoxInputPolicy) -> Self {
        Self {
            policy,
            router: ControllerRouter::default(),
            native: NativeGamepadInput::default(),
            pending_actions: VecDeque::new(),
        }
    }

    pub fn set_policy(&mut self, policy: BigBoxInputPolicy) {
        self.policy = policy;
        self.router.reset();
        self.pending_actions.clear();
    }

    pub fn policy(&self) -> &BigBoxInputPolicy {
        &self.policy
    }

    pub fn poll_action(&mut self) -> Option<BigBoxInputAction> {
        if let Some(action) = self.pending_actions.pop_front() {
            return Some(action);
        }
        if self.policy.gamepad_enabled {
            for event in self.native.poll_events() {
                self.pending_actions
                    .extend(self.router.route(&self.policy, event));
            }
        }
        self.pending_actions.pop_front()
    }

    /// Feed the same semantic boundary used by the native backend. This is
    /// useful for controller-mapping previews and deterministic UI tests.
    pub fn submit_semantic_event(&mut self, event: GamepadInputEvent) {
        self.pending_actions
            .extend(self.router.route(&self.policy, event));
    }

    pub fn backend_status(&self) -> &GamepadBackendStatus {
        self.native.status()
    }

    pub fn connected_gamepad_count(&self) -> usize {
        self.native.connected_count()
    }
}

impl Default for BigBoxInputEngine {
    fn default() -> Self {
        Self::new(BigBoxInputPolicy::default())
    }
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum StickAxis {
    LeftX,
    LeftY,
    RightX,
    RightY,
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
struct NativeGamepadInput {
    gilrs: Option<gilrs::Gilrs>,
    status: GamepadBackendStatus,
    connected_count: usize,
    axis_bindings: HashMap<(u64, StickAxis), ControllerBinding>,
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
impl NativeGamepadInput {
    fn initialize(&mut self) {
        if !matches!(self.status, GamepadBackendStatus::NotInitialized) {
            return;
        }
        match gilrs::GilrsBuilder::new()
            .with_force_feedback(false)
            .set_axis_to_btn(0.75, 0.65)
            .build()
        {
            Ok(gilrs) => {
                self.connected_count = gilrs.gamepads().count();
                self.gilrs = Some(gilrs);
                self.status = GamepadBackendStatus::Ready;
            }
            Err(error) => {
                self.status = GamepadBackendStatus::Unavailable(error.to_string());
            }
        }
    }

    fn poll_events(&mut self) -> Vec<GamepadInputEvent> {
        self.initialize();
        let Some(gilrs) = self.gilrs.as_mut() else {
            return Vec::new();
        };
        let mut events = Vec::new();
        for _ in 0..256 {
            let Some(event) = gilrs.next_event() else {
                break;
            };
            let device_id = usize::from(event.id) as u64;
            match event.event {
                gilrs::EventType::ButtonPressed(button, _) => {
                    if let Some(binding) = controller_binding_for_button(button) {
                        events.push(GamepadInputEvent::Pressed { device_id, binding });
                    }
                }
                gilrs::EventType::ButtonReleased(button, _) => {
                    if let Some(binding) = controller_binding_for_button(button) {
                        events.push(GamepadInputEvent::Released { device_id, binding });
                    }
                }
                gilrs::EventType::AxisChanged(axis, value, _) => {
                    if let Some((axis, negative, positive)) = controller_bindings_for_axis(axis) {
                        let key = (device_id, axis);
                        let previous = self.axis_bindings.get(&key).copied();
                        let next = if value <= -0.75 {
                            Some(negative)
                        } else if value >= 0.75 {
                            Some(positive)
                        } else if value.abs() <= 0.65 {
                            None
                        } else {
                            continue;
                        };
                        if previous != next {
                            if let Some(binding) = previous {
                                events.push(GamepadInputEvent::Released { device_id, binding });
                                self.axis_bindings.remove(&key);
                            }
                            if let Some(binding) = next {
                                events.push(GamepadInputEvent::Pressed { device_id, binding });
                                self.axis_bindings.insert(key, binding);
                            }
                        }
                    }
                }
                gilrs::EventType::Disconnected => {
                    self.axis_bindings
                        .retain(|(candidate, _), _| *candidate != device_id);
                    events.push(GamepadInputEvent::Disconnected { device_id });
                }
                _ => {}
            }
        }
        self.connected_count = gilrs.gamepads().count();
        events
    }

    fn status(&self) -> &GamepadBackendStatus {
        &self.status
    }

    fn connected_count(&self) -> usize {
        self.connected_count
    }
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
impl Default for NativeGamepadInput {
    fn default() -> Self {
        Self {
            gilrs: None,
            status: GamepadBackendStatus::NotInitialized,
            connected_count: 0,
            axis_bindings: HashMap::new(),
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
fn controller_binding_for_button(button: gilrs::Button) -> Option<ControllerBinding> {
    use gilrs::Button;
    match button {
        Button::South => Some(ControllerBinding::Button(1)),
        Button::East => Some(ControllerBinding::Button(2)),
        Button::West => Some(ControllerBinding::Button(3)),
        Button::North => Some(ControllerBinding::Button(4)),
        Button::LeftTrigger => Some(ControllerBinding::Button(5)),
        Button::RightTrigger => Some(ControllerBinding::Button(6)),
        Button::Select => Some(ControllerBinding::Button(7)),
        Button::Start => Some(ControllerBinding::Button(8)),
        Button::LeftThumb => Some(ControllerBinding::Button(9)),
        Button::RightThumb => Some(ControllerBinding::Button(10)),
        Button::Mode => Some(ControllerBinding::Button(11)),
        Button::C => Some(ControllerBinding::Button(12)),
        Button::Z => Some(ControllerBinding::Button(13)),
        Button::DPadUp => Some(ControllerBinding::DPadUp),
        Button::DPadDown => Some(ControllerBinding::DPadDown),
        Button::DPadLeft => Some(ControllerBinding::DPadLeft),
        Button::DPadRight => Some(ControllerBinding::DPadRight),
        Button::LeftTrigger2 => Some(ControllerBinding::TriggerLeft),
        Button::RightTrigger2 => Some(ControllerBinding::TriggerRight),
        Button::Unknown => None,
    }
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
fn controller_bindings_for_axis(
    axis: gilrs::Axis,
) -> Option<(StickAxis, ControllerBinding, ControllerBinding)> {
    match axis {
        gilrs::Axis::LeftStickX => Some((
            StickAxis::LeftX,
            ControllerBinding::LeftStickLeft,
            ControllerBinding::LeftStickRight,
        )),
        gilrs::Axis::LeftStickY => Some((
            StickAxis::LeftY,
            ControllerBinding::LeftStickDown,
            ControllerBinding::LeftStickUp,
        )),
        gilrs::Axis::RightStickX => Some((
            StickAxis::RightX,
            ControllerBinding::RightStickLeft,
            ControllerBinding::RightStickRight,
        )),
        gilrs::Axis::RightStickY => Some((
            StickAxis::RightY,
            ControllerBinding::RightStickDown,
            ControllerBinding::RightStickUp,
        )),
        _ => None,
    }
}

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
struct NativeGamepadInput {
    status: GamepadBackendStatus,
}

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
impl Default for NativeGamepadInput {
    fn default() -> Self {
        Self {
            status: GamepadBackendStatus::Unavailable(
                "native gamepads are unsupported on this target".into(),
            ),
        }
    }
}

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
impl NativeGamepadInput {
    fn poll_events(&mut self) -> Vec<GamepadInputEvent> {
        Vec::new()
    }

    fn status(&self) -> &GamepadBackendStatus {
        &self.status
    }

    fn connected_count(&self) -> usize {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lb_domain::SettingEntry;
    use std::path::PathBuf;

    fn settings(entries: &[(&str, &str)]) -> FrontendSettings {
        FrontendSettings {
            source_path: PathBuf::from("Data/BigBoxSettings.xml"),
            record_name: "BigBoxSettings".into(),
            entries: entries
                .iter()
                .map(|(key, value)| SettingEntry {
                    key: (*key).into(),
                    value: (*value).into(),
                })
                .collect(),
            image_type_settings: Vec::new(),
        }
    }

    fn binding(action: &str, controller: &str, hold: &str) -> InputBinding {
        InputBinding {
            input_action: action.into(),
            controller_binding: controller.into(),
            controller_hold_binding: hold.into(),
        }
    }

    #[test]
    fn recovered_action_catalog_is_complete_and_ordered() {
        assert_eq!(BIG_BOX_INPUT_ACTIONS.len(), 59);
        assert_eq!(BIG_BOX_INPUT_ACTIONS[0].key(), "BigBoxSearch");
        assert_eq!(
            BIG_BOX_INPUT_ACTIONS.last().map(|action| action.key()),
            Some("BigBoxStartScreensaver")
        );
        let keys = BIG_BOX_INPUT_ACTIONS
            .iter()
            .map(|action| action.key())
            .collect::<BTreeSet<_>>();
        assert_eq!(keys.len(), BIG_BOX_INPUT_ACTIONS.len());
    }

    #[test]
    fn recovered_keyboard_defaults_and_all_four_slots_are_portable() {
        let settings = settings(&[
            ("KeyboardSelect", "44"),
            ("KeyboardSelect2", "45"),
            ("KeyboardSelect3", "90"),
            ("KeyboardSelect4", "113"),
            ("KeyboardRotateModelLeft1", "23"),
            ("KeyboardRotateModelLeft4", "25"),
        ]);
        let policy = BigBoxInputPolicy::from_settings(Some(&settings), &[]);
        assert_eq!(
            policy.keyboard_sequence(BigBoxInputAction::Select, 0),
            Some("A")
        );
        assert_eq!(
            policy.keyboard_sequence(BigBoxInputAction::Select, 1),
            Some("B")
        );
        assert_eq!(
            policy.keyboard_sequence(BigBoxInputAction::Select, 2),
            Some("F1")
        );
        assert_eq!(
            policy.keyboard_sequence(BigBoxInputAction::Select, 3),
            Some("F24")
        );
        assert_eq!(
            policy.keyboard_wpf_key(BigBoxInputAction::Select, 3),
            Some(113)
        );
        assert_eq!(
            policy.keyboard_sequence(BigBoxInputAction::RotateModelLeft, 0),
            Some("Left")
        );
        assert_eq!(
            policy.keyboard_sequence(BigBoxInputAction::RotateModelLeft, 3),
            Some("Right")
        );
        assert_eq!(
            policy.keyboard_sequence(BigBoxInputAction::PlayGame, 0),
            Some("P")
        );
    }

    #[test]
    fn malformed_settings_fall_back_without_inventing_unknown_keys() {
        let settings = settings(&[
            ("EnableGamepad", "maybe"),
            ("UseAllControllers", "TRUE"),
            ("KeyboardBack", "not-a-key"),
            ("KeyboardSelect", "999"),
        ]);
        let policy = BigBoxInputPolicy::from_settings(Some(&settings), &[]);
        assert!(policy.gamepad_enabled);
        assert!(policy.use_all_controllers);
        assert_eq!(
            policy.keyboard_sequence(BigBoxInputAction::Back, 0),
            Some("Esc")
        );
        assert_eq!(policy.keyboard_sequence(BigBoxInputAction::Select, 0), None);
        assert_eq!(
            policy.keyboard_wpf_key(BigBoxInputAction::Select, 0),
            Some(999)
        );
        assert_eq!(
            policy.keyboard_wpf_key(BigBoxInputAction::ExitGame, 0),
            None
        );
    }

    #[test]
    fn recovered_default_controller_rules_route_distinct_select_and_play() {
        let mut engine = BigBoxInputEngine::default();
        assert_eq!(engine.policy().controller_rule_count(), 18);
        engine.submit_semantic_event(GamepadInputEvent::Pressed {
            device_id: 7,
            binding: ControllerBinding::Button(1),
        });
        assert_eq!(engine.poll_action(), Some(BigBoxInputAction::Select));
        engine.submit_semantic_event(GamepadInputEvent::Pressed {
            device_id: 7,
            binding: ControllerBinding::Button(3),
        });
        assert_eq!(engine.poll_action(), Some(BigBoxInputAction::PlayGame));
    }

    #[test]
    fn explicitly_persisted_empty_controller_map_stays_empty() {
        assert_eq!(
            BigBoxInputPolicy::from_persisted_settings(None, &[]).controller_rule_count(),
            0
        );
        assert_eq!(BigBoxInputPolicy::default().controller_rule_count(), 18);
    }

    #[test]
    fn controller_holds_and_edges_use_the_same_semantic_router() {
        let bindings = [
            binding("BigBoxExit", "Button8", "Button7"),
            binding("BigBoxSelect", "Button1", "None"),
        ];
        let mut engine = BigBoxInputEngine::new(BigBoxInputPolicy::from_settings(None, &bindings));
        engine.submit_semantic_event(GamepadInputEvent::Pressed {
            device_id: 1,
            binding: ControllerBinding::Button(8),
        });
        assert_eq!(engine.poll_action(), None);
        engine.submit_semantic_event(GamepadInputEvent::Released {
            device_id: 1,
            binding: ControllerBinding::Button(8),
        });
        engine.submit_semantic_event(GamepadInputEvent::Pressed {
            device_id: 1,
            binding: ControllerBinding::Button(7),
        });
        engine.submit_semantic_event(GamepadInputEvent::Pressed {
            device_id: 1,
            binding: ControllerBinding::Button(8),
        });
        assert_eq!(engine.poll_action(), Some(BigBoxInputAction::Exit));
        assert_eq!(engine.poll_action(), None);
    }

    #[test]
    fn one_controller_mode_locks_to_first_active_device_until_disconnect() {
        let mut engine = BigBoxInputEngine::default();
        engine.submit_semantic_event(GamepadInputEvent::Pressed {
            device_id: 2,
            binding: ControllerBinding::Button(1),
        });
        engine.submit_semantic_event(GamepadInputEvent::Pressed {
            device_id: 3,
            binding: ControllerBinding::Button(3),
        });
        assert_eq!(engine.poll_action(), Some(BigBoxInputAction::Select));
        assert_eq!(engine.poll_action(), None);
        engine.submit_semantic_event(GamepadInputEvent::Disconnected { device_id: 2 });
        engine.submit_semantic_event(GamepadInputEvent::Pressed {
            device_id: 3,
            binding: ControllerBinding::Button(1),
        });
        assert_eq!(engine.poll_action(), Some(BigBoxInputAction::Select));
    }

    #[test]
    fn disabled_gamepad_refuses_semantic_events() {
        let settings = settings(&[("EnableGamepad", "false")]);
        let mut engine =
            BigBoxInputEngine::new(BigBoxInputPolicy::from_settings(Some(&settings), &[]));
        engine.submit_semantic_event(GamepadInputEvent::Pressed {
            device_id: 1,
            binding: ControllerBinding::Button(1),
        });
        assert_eq!(engine.poll_action(), None);
        assert!(matches!(
            engine.backend_status(),
            GamepadBackendStatus::NotInitialized
        ));
    }

    #[test]
    fn unsupported_big_box_rules_are_counted_and_valid_rules_are_deduplicated() {
        let bindings = [
            binding("BigBoxSelect", "Button1", "None"),
            binding("BigBoxSelect", "Button1", "None"),
            binding("BigBoxFutureAction", "Button2", "None"),
            binding("BigBoxBack", "Button99", "None"),
            binding("LaunchBoxSelect", "Button1", "None"),
        ];
        let policy = BigBoxInputPolicy::from_settings(None, &bindings);
        assert_eq!(policy.controller_rule_count(), 1);
        assert_eq!(policy.unsupported_controller_rule_count, 2);
    }

    #[test]
    fn wpf_key_conversion_covers_recovered_defaults_and_ranges() {
        assert_eq!(wpf_key_to_qt_portable_text(0), None);
        assert_eq!(wpf_key_to_qt_portable_text(6).as_deref(), Some("Return"));
        assert_eq!(wpf_key_to_qt_portable_text(13).as_deref(), Some("Esc"));
        assert_eq!(wpf_key_to_qt_portable_text(23).as_deref(), Some("Left"));
        assert_eq!(wpf_key_to_qt_portable_text(34).as_deref(), Some("0"));
        assert_eq!(wpf_key_to_qt_portable_text(69).as_deref(), Some("Z"));
        assert_eq!(wpf_key_to_qt_portable_text(74).as_deref(), Some("Num+0"));
        assert_eq!(wpf_key_to_qt_portable_text(85).as_deref(), Some("Num++"));
        assert_eq!(wpf_key_to_qt_portable_text(90).as_deref(), Some("F1"));
        assert_eq!(wpf_key_to_qt_portable_text(113).as_deref(), Some("F24"));
        assert_eq!(wpf_key_to_qt_portable_text(140).as_deref(), Some(";"));
        assert_eq!(wpf_key_to_qt_portable_text(150).as_deref(), Some("\\"));
        assert_eq!(wpf_key_to_qt_portable_text(999), None);
    }

    #[test]
    fn qt_key_capture_maps_logical_keys_without_platform_scan_codes() {
        assert_eq!(qt_key_to_wpf_key(0x0100_0000), Some(13));
        assert_eq!(qt_key_to_wpf_key(0x0100_0004), Some(6));
        assert_eq!(qt_key_to_wpf_key(0x0100_0012), Some(23));
        assert_eq!(qt_key_to_wpf_key(i32::from(b'0')), Some(34));
        assert_eq!(qt_key_to_wpf_key(i32::from(b'Z')), Some(69));
        assert_eq!(qt_key_to_wpf_key(0x0100_0030), Some(90));
        assert_eq!(qt_key_to_wpf_key(0x0100_0047), Some(113));
        assert_eq!(qt_key_to_wpf_key(i32::from(b';')), Some(140));
        assert_eq!(qt_key_to_wpf_key(i32::from(b'\\')), Some(150));
        assert_eq!(
            qt_key_to_wpf_key_with_modifiers(i32::from(b'7'), 0x2000_0000),
            Some(81)
        );
        assert_eq!(
            qt_key_to_wpf_key_with_modifiers(i32::from(b'+'), 0x2000_0000),
            Some(85)
        );
        assert_eq!(qt_key_to_wpf_key(0x0100_0050), None);
    }

    #[test]
    fn action_metadata_exposes_exact_persisted_keyboard_slots() {
        assert_eq!(BigBoxInputAction::Select.keyboard_slot_count(), 4);
        assert_eq!(
            BigBoxInputAction::Select.keyboard_setting_key(0).as_deref(),
            Some("KeyboardSelect")
        );
        assert_eq!(
            BigBoxInputAction::Select.keyboard_setting_key(3).as_deref(),
            Some("KeyboardSelect4")
        );
        assert_eq!(
            BigBoxInputAction::RotateModelLeft
                .keyboard_setting_key(0)
                .as_deref(),
            Some("KeyboardRotateModelLeft1")
        );
        assert_eq!(
            BigBoxInputAction::ShowPauseScreen
                .keyboard_setting_key(0)
                .as_deref(),
            Some("KeyboardGamePause")
        );
        assert_eq!(BigBoxInputAction::ExitGame.keyboard_slot_count(), 0);
    }

    #[test]
    fn duplicate_keyboard_sequences_are_grouped_for_one_qt_shortcut() {
        let policy = BigBoxInputPolicy::default();
        let bindings = policy.keyboard_bindings();
        let (_, actions) = bindings
            .iter()
            .find(|(sequence, _)| sequence == "Num++")
            .expect("recovered plus binding");
        assert_eq!(
            actions,
            &[BigBoxInputAction::ZoomIn, BigBoxInputAction::VolumeUp]
        );
        assert_eq!(
            bindings
                .iter()
                .filter(|(sequence, _)| sequence == "Num++")
                .count(),
            1
        );
    }
}
