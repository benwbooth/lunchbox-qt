use crate::{GameControllerSupportLevel, ModelSettings};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BulkGameEditorKind {
    Text,
    MultilineText,
    Boolean,
    UnsignedInteger,
    Rating,
    Date,
    LexicalPath,
    Emulator,
    Platform,
    ControllerSupport,
    ModelSettings,
    MultiValue,
    CustomField,
}

impl BulkGameEditorKind {
    pub const fn key(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::MultilineText => "multilineText",
            Self::Boolean => "boolean",
            Self::UnsignedInteger => "unsignedInteger",
            Self::Rating => "rating",
            Self::Date => "date",
            Self::LexicalPath => "lexicalPath",
            Self::Emulator => "emulator",
            Self::Platform => "platform",
            Self::ControllerSupport => "controllerSupport",
            Self::ModelSettings => "modelSettings",
            Self::MultiValue => "multiValue",
            Self::CustomField => "customField",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BulkGameFieldDefinition {
    pub field: BulkGameField,
    pub label: &'static str,
    pub editor: BulkGameEditorKind,
    pub clearable: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BulkGameField {
    Broken,
    Completed,
    ControllerSupport,
    CustomDosBoxVersion,
    Developer,
    Emulator,
    Favorite,
    Genre,
    Hidden,
    MaxPlayers,
    ModelSettings,
    Notes,
    PauseScreenEnable,
    PauseScreenForcefulActivation,
    PauseScreenLoadStateAutoHotkeyScript,
    PauseScreenOverrideDefaultSettings,
    PauseScreenPauseGameAutoHotkeyScript,
    PauseScreenResetGameAutoHotkeyScript,
    PauseScreenResumeGameAutoHotkeyScript,
    PauseScreenSaveStateAutoHotkeyScript,
    PauseScreenSuspendGameProcessOnPause,
    PlayMode,
    Platform,
    Progress,
    Publisher,
    Rating,
    Region,
    ReleaseDate,
    ReleaseType,
    Series,
    SortTitle,
    Source,
    StarRating,
    StartupScreenAggressiveStartupWindowHiding,
    StartupScreenEnabled,
    StartupScreenHideAllNonExclusiveModeWindows,
    StartupScreenHideMouseCursorDuringGame,
    StartupScreenLoadDelay,
    StartupScreenOverrideDefaultSettings,
    StartupScreenShutdownEnabled,
    Status,
    Version,
    VideoPath,
    WikipediaUrl,
    CustomField,
}

impl BulkGameField {
    pub const fn key(self) -> &'static str {
        match self {
            Self::Broken => "broken",
            Self::Completed => "completed",
            Self::ControllerSupport => "controllerSupport",
            Self::CustomDosBoxVersion => "customDosBoxVersion",
            Self::Developer => "developer",
            Self::Emulator => "emulator",
            Self::Favorite => "favorite",
            Self::Genre => "genre",
            Self::Hidden => "hidden",
            Self::MaxPlayers => "maxPlayers",
            Self::ModelSettings => "modelSettings",
            Self::Notes => "notes",
            Self::PauseScreenEnable => "pauseScreenEnable",
            Self::PauseScreenForcefulActivation => "pauseScreenForcefulActivation",
            Self::PauseScreenLoadStateAutoHotkeyScript => "pauseScreenLoadStateAutoHotkeyScript",
            Self::PauseScreenOverrideDefaultSettings => "pauseScreenOverrideDefaultSettings",
            Self::PauseScreenPauseGameAutoHotkeyScript => "pauseScreenPauseGameAutoHotkeyScript",
            Self::PauseScreenResetGameAutoHotkeyScript => "pauseScreenResetGameAutoHotkeyScript",
            Self::PauseScreenResumeGameAutoHotkeyScript => "pauseScreenResumeGameAutoHotkeyScript",
            Self::PauseScreenSaveStateAutoHotkeyScript => "pauseScreenSaveStateAutoHotkeyScript",
            Self::PauseScreenSuspendGameProcessOnPause => "pauseScreenSuspendGameProcessOnPause",
            Self::PlayMode => "playMode",
            Self::Platform => "platform",
            Self::Progress => "progress",
            Self::Publisher => "publisher",
            Self::Rating => "rating",
            Self::Region => "region",
            Self::ReleaseDate => "releaseDate",
            Self::ReleaseType => "releaseType",
            Self::Series => "series",
            Self::SortTitle => "sortTitle",
            Self::Source => "source",
            Self::StarRating => "starRating",
            Self::StartupScreenAggressiveStartupWindowHiding => {
                "startupScreenAggressiveStartupWindowHiding"
            }
            Self::StartupScreenEnabled => "startupScreenEnabled",
            Self::StartupScreenHideAllNonExclusiveModeWindows => {
                "startupScreenHideAllNonExclusiveModeWindows"
            }
            Self::StartupScreenHideMouseCursorDuringGame => {
                "startupScreenHideMouseCursorDuringGame"
            }
            Self::StartupScreenLoadDelay => "startupScreenLoadDelay",
            Self::StartupScreenOverrideDefaultSettings => "startupScreenOverrideDefaultSettings",
            Self::StartupScreenShutdownEnabled => "startupScreenShutdownEnabled",
            Self::Status => "status",
            Self::Version => "version",
            Self::VideoPath => "videoPath",
            Self::WikipediaUrl => "wikipediaUrl",
            Self::CustomField => "customField",
        }
    }

    pub const fn supports_multi_value_operations(self) -> bool {
        matches!(
            self,
            Self::Genre | Self::PlayMode | Self::Series | Self::CustomField
        )
    }
}

pub const BULK_GAME_FIELDS: &[BulkGameFieldDefinition] = &[
    // Preserve the recovered LaunchBox 13.27 order for every implemented
    // entry. Port-owned compatibility additions that are absent from the
    // value-free oracle follow the recovered entries.
    BulkGameFieldDefinition {
        field: BulkGameField::ModelSettings,
        label: "3D Model Settings",
        editor: BulkGameEditorKind::ModelSettings,
        clearable: false,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::Broken,
        label: "Broken",
        editor: BulkGameEditorKind::Boolean,
        clearable: false,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::ControllerSupport,
        label: "Controller Support",
        editor: BulkGameEditorKind::ControllerSupport,
        clearable: false,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::CustomDosBoxVersion,
        label: "Custom DOSBox Version EXE Path",
        editor: BulkGameEditorKind::LexicalPath,
        clearable: true,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::Developer,
        label: "Developer",
        editor: BulkGameEditorKind::Text,
        clearable: true,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::Emulator,
        label: "Emulator",
        editor: BulkGameEditorKind::Emulator,
        clearable: true,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::Favorite,
        label: "Favorite",
        editor: BulkGameEditorKind::Boolean,
        clearable: false,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::Genre,
        label: "Genre",
        editor: BulkGameEditorKind::MultiValue,
        clearable: true,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::Hidden,
        label: "Hide",
        editor: BulkGameEditorKind::Boolean,
        clearable: false,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::MaxPlayers,
        label: "Max Players",
        editor: BulkGameEditorKind::Text,
        clearable: true,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::Notes,
        label: "Notes",
        editor: BulkGameEditorKind::MultilineText,
        clearable: true,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::PauseScreenEnable,
        label: "Pause Screen - Enable",
        editor: BulkGameEditorKind::Boolean,
        clearable: false,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::PauseScreenForcefulActivation,
        label: "Pause Screen - Forceful Activation",
        editor: BulkGameEditorKind::Boolean,
        clearable: false,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::PauseScreenLoadStateAutoHotkeyScript,
        label: "Pause Screen - Load State AutoHotkey Script",
        editor: BulkGameEditorKind::MultilineText,
        clearable: true,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::PauseScreenOverrideDefaultSettings,
        label: "Pause Screen - Override Default Settings",
        editor: BulkGameEditorKind::Boolean,
        clearable: false,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::PauseScreenPauseGameAutoHotkeyScript,
        label: "Pause Screen - Pause Game AutoHotkey Script",
        editor: BulkGameEditorKind::MultilineText,
        clearable: true,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::PauseScreenResetGameAutoHotkeyScript,
        label: "Pause Screen - Reset Game AutoHotkey Script",
        editor: BulkGameEditorKind::MultilineText,
        clearable: true,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::PauseScreenResumeGameAutoHotkeyScript,
        label: "Pause Screen - Resume Game AutoHotkey Script",
        editor: BulkGameEditorKind::MultilineText,
        clearable: true,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::PauseScreenSaveStateAutoHotkeyScript,
        label: "Pause Screen - Save State AutoHotkey Script",
        editor: BulkGameEditorKind::MultilineText,
        clearable: true,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::PauseScreenSuspendGameProcessOnPause,
        label: "Pause Screen - Suspend Game Process On Pause",
        editor: BulkGameEditorKind::Boolean,
        clearable: false,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::Platform,
        label: "Platform",
        editor: BulkGameEditorKind::Platform,
        clearable: false,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::PlayMode,
        label: "Play Mode",
        editor: BulkGameEditorKind::MultiValue,
        clearable: true,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::Progress,
        label: "Progress",
        editor: BulkGameEditorKind::Text,
        clearable: true,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::Publisher,
        label: "Publisher",
        editor: BulkGameEditorKind::Text,
        clearable: true,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::Rating,
        label: "Rating",
        editor: BulkGameEditorKind::Text,
        clearable: true,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::Region,
        label: "Region",
        editor: BulkGameEditorKind::Text,
        clearable: true,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::ReleaseDate,
        label: "Release Date",
        editor: BulkGameEditorKind::Date,
        clearable: true,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::ReleaseType,
        label: "Release Type",
        editor: BulkGameEditorKind::Text,
        clearable: true,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::Series,
        label: "Series",
        editor: BulkGameEditorKind::MultiValue,
        clearable: true,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::SortTitle,
        label: "Sort Title",
        editor: BulkGameEditorKind::Text,
        clearable: true,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::Source,
        label: "Source",
        editor: BulkGameEditorKind::Text,
        clearable: true,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::StarRating,
        label: "Star Rating",
        editor: BulkGameEditorKind::Rating,
        clearable: false,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::StartupScreenAggressiveStartupWindowHiding,
        label: "Startup Screen - Aggressive Startup Window Hiding",
        editor: BulkGameEditorKind::Boolean,
        clearable: false,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::StartupScreenEnabled,
        label: "Startup Screen - Enabled",
        editor: BulkGameEditorKind::Boolean,
        clearable: false,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::StartupScreenHideAllNonExclusiveModeWindows,
        label: "Startup Screen - Hide All Non-Exclusive Mode Windows",
        editor: BulkGameEditorKind::Boolean,
        clearable: false,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::StartupScreenHideMouseCursorDuringGame,
        label: "Startup Screen - Hide Mouse Cursor During Game",
        editor: BulkGameEditorKind::Boolean,
        clearable: false,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::StartupScreenLoadDelay,
        label: "Startup Screen - Load Delay",
        editor: BulkGameEditorKind::UnsignedInteger,
        clearable: false,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::StartupScreenOverrideDefaultSettings,
        label: "Startup Screen - Override Default Settings",
        editor: BulkGameEditorKind::Boolean,
        clearable: false,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::StartupScreenShutdownEnabled,
        label: "Startup Screen - Shutdown Enabled",
        editor: BulkGameEditorKind::Boolean,
        clearable: false,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::Status,
        label: "Status",
        editor: BulkGameEditorKind::Text,
        clearable: true,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::Version,
        label: "Version",
        editor: BulkGameEditorKind::Text,
        clearable: true,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::WikipediaUrl,
        label: "Wikipedia URL",
        editor: BulkGameEditorKind::Text,
        clearable: true,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::Completed,
        label: "Completed",
        editor: BulkGameEditorKind::Boolean,
        clearable: false,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::VideoPath,
        label: "Video Path",
        editor: BulkGameEditorKind::LexicalPath,
        clearable: true,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::CustomField,
        label: "Custom Field",
        editor: BulkGameEditorKind::CustomField,
        clearable: true,
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BulkGameEditOperation {
    Set,
    Clear,
    Add,
    Remove,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BulkGameEdit {
    pub field: BulkGameField,
    pub operation: BulkGameEditOperation,
    pub text: Option<String>,
    pub boolean: Option<bool>,
    pub number: Option<f64>,
    pub custom_field_name: Option<String>,
}

impl BulkGameEdit {
    pub fn validate(&self) -> Result<(), BulkGameEditError> {
        let definition = BULK_GAME_FIELDS
            .iter()
            .find(|definition| definition.field == self.field)
            .ok_or(BulkGameEditError::UnsupportedField)?;
        match definition.editor {
            BulkGameEditorKind::Boolean => {
                if self.operation != BulkGameEditOperation::Set || self.boolean.is_none() {
                    return Err(BulkGameEditError::BooleanValueRequired);
                }
                self.require_unused(false, true, false, false)?;
            }
            BulkGameEditorKind::UnsignedInteger => {
                if self.operation != BulkGameEditOperation::Set {
                    return Err(BulkGameEditError::SetOperationRequired);
                }
                let number = self.number.ok_or(BulkGameEditError::NumberValueRequired)?;
                if !number.is_finite()
                    || number.fract() != 0.0
                    || !(0.0..=f64::from(u32::MAX)).contains(&number)
                {
                    return Err(BulkGameEditError::InvalidUnsignedInteger);
                }
                self.require_unused(false, false, true, false)?;
            }
            BulkGameEditorKind::Rating => {
                if self.operation != BulkGameEditOperation::Set {
                    return Err(BulkGameEditError::SetOperationRequired);
                }
                let rating = self.number.ok_or(BulkGameEditError::NumberValueRequired)?;
                let scaled = rating * 2.0;
                if !rating.is_finite()
                    || !(0.0..=5.0).contains(&rating)
                    || (scaled - scaled.round()).abs() > 1.0e-9
                {
                    return Err(BulkGameEditError::InvalidStarRating);
                }
                self.require_unused(false, false, true, false)?;
            }
            BulkGameEditorKind::ControllerSupport | BulkGameEditorKind::ModelSettings => {
                if self.operation != BulkGameEditOperation::Set {
                    return Err(BulkGameEditError::SetOperationRequired);
                }
                self.require_unused(false, false, false, false)?;
            }
            _ => {
                if matches!(
                    self.operation,
                    BulkGameEditOperation::Add | BulkGameEditOperation::Remove
                ) && !self.field.supports_multi_value_operations()
                {
                    return Err(BulkGameEditError::MultiValueOperationNotSupported);
                }
                if self.operation == BulkGameEditOperation::Clear && !definition.clearable {
                    return Err(BulkGameEditError::ClearNotSupported);
                }
                if self.operation != BulkGameEditOperation::Clear
                    && self
                        .text
                        .as_deref()
                        .is_none_or(|value| value.trim().is_empty())
                {
                    return Err(BulkGameEditError::TextValueRequired);
                }
                if self.field == BulkGameField::MaxPlayers
                    && self.operation == BulkGameEditOperation::Set
                    && self
                        .text
                        .as_deref()
                        .and_then(|value| value.trim().parse::<u32>().ok())
                        .is_none_or(|value| value == 0)
                {
                    return Err(BulkGameEditError::InvalidMaxPlayers);
                }
                if self.field == BulkGameField::CustomField
                    && self
                        .custom_field_name
                        .as_deref()
                        .is_none_or(|name| name.trim().is_empty())
                {
                    return Err(BulkGameEditError::CustomFieldNameRequired);
                }
                self.require_unused(
                    self.operation != BulkGameEditOperation::Clear,
                    false,
                    false,
                    self.field == BulkGameField::CustomField,
                )?;
            }
        }
        Ok(())
    }

    fn require_unused(
        &self,
        text: bool,
        boolean: bool,
        number: bool,
        custom_field_name: bool,
    ) -> Result<(), BulkGameEditError> {
        if self.text.is_some() != text
            || self.boolean.is_some() != boolean
            || self.number.is_some() != number
            || self.custom_field_name.is_some() != custom_field_name
        {
            return Err(BulkGameEditError::UnexpectedValue);
        }
        Ok(())
    }
}

/// One LaunchBox bulk 3D-model-settings change.
///
/// The recovered 13.27 surface has an "override default model settings"
/// checkbox. An enabled checkbox supplies one identity-free whole-record
/// template that is copied to every selected game; a disabled checkbox removes
/// each selected game's override so normal platform/built-in inheritance
/// resumes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BulkModelSettingsEdit {
    pub settings: Option<ModelSettings>,
}

impl BulkModelSettingsEdit {
    pub fn validate(&self) -> Result<(), BulkGameEditError> {
        let Some(settings) = &self.settings else {
            return Ok(());
        };
        settings
            .validate()
            .map_err(|error| BulkGameEditError::InvalidBulkModelSettings {
                reason: error.to_string(),
            })?;
        if settings.game_id.is_some() || settings.platform_name.is_some() {
            return Err(BulkGameEditError::BulkModelSettingsIdentityNotAllowed);
        }
        if settings.model_type.is_none() {
            return Err(BulkGameEditError::BulkModelSettingsTypeRequired);
        }
        Ok(())
    }
}

/// One LaunchBox bulk Controller Support change.
///
/// The recovered 13.27 surface presents independent remove and add
/// multi-selectors. One support level applies to every added controller. A
/// controller cannot be selected on both sides of the same operation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BulkControllerSupportEdit {
    pub add_controller_ids: Vec<String>,
    pub remove_controller_ids: Vec<String>,
    pub support_level: Option<GameControllerSupportLevel>,
}

impl BulkControllerSupportEdit {
    pub fn validate(&self) -> Result<(), BulkGameEditError> {
        if self.add_controller_ids.is_empty() && self.remove_controller_ids.is_empty() {
            return Err(BulkGameEditError::ControllerSupportChangeRequired);
        }
        if !self.add_controller_ids.is_empty() && self.support_level.is_none() {
            return Err(BulkGameEditError::ControllerSupportLevelRequired);
        }
        if self.add_controller_ids.is_empty() && self.support_level.is_some() {
            return Err(BulkGameEditError::UnexpectedControllerSupportLevel);
        }

        let add = validate_controller_ids(&self.add_controller_ids)?;
        let remove = validate_controller_ids(&self.remove_controller_ids)?;
        if let Some(id) = add.intersection(&remove).next() {
            return Err(BulkGameEditError::ConflictingControllerSupportId { id: id.clone() });
        }
        Ok(())
    }
}

fn validate_controller_ids(ids: &[String]) -> Result<BTreeSet<String>, BulkGameEditError> {
    let mut keys = BTreeSet::new();
    for id in ids {
        let id = id.trim();
        if id.is_empty() {
            return Err(BulkGameEditError::EmptyControllerSupportId);
        }
        if !keys.insert(id.to_lowercase()) {
            return Err(BulkGameEditError::DuplicateControllerSupportId { id: id.to_string() });
        }
    }
    Ok(keys)
}

pub fn mutate_multi_value_text(
    original: Option<&str>,
    operation: BulkGameEditOperation,
    value: Option<&str>,
) -> Result<Option<String>, BulkGameEditError> {
    if operation == BulkGameEditOperation::Clear {
        return Ok(None);
    }
    let value = value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(BulkGameEditError::TextValueRequired)?;
    if operation == BulkGameEditOperation::Set {
        return Ok(Some(value.to_string()));
    }
    let mut values = original
        .unwrap_or_default()
        .split(';')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    match operation {
        BulkGameEditOperation::Add => {
            if !values.iter().any(|item| item.eq_ignore_ascii_case(value)) {
                values.push(value.to_string());
            }
        }
        BulkGameEditOperation::Remove => {
            values.retain(|item| !item.eq_ignore_ascii_case(value));
        }
        BulkGameEditOperation::Set | BulkGameEditOperation::Clear => unreachable!(),
    }
    Ok((!values.is_empty()).then(|| values.join("; ")))
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum BulkGameEditError {
    #[error("the bulk-edit field is not supported")]
    UnsupportedField,
    #[error("this field requires the set operation")]
    SetOperationRequired,
    #[error("this field does not support clearing")]
    ClearNotSupported,
    #[error("this field does not support add or remove")]
    MultiValueOperationNotSupported,
    #[error("a text value is required")]
    TextValueRequired,
    #[error("a boolean value is required")]
    BooleanValueRequired,
    #[error("a numeric value is required")]
    NumberValueRequired,
    #[error("the value must be a whole number from 0 through 4294967295")]
    InvalidUnsignedInteger,
    #[error("star rating must be a half-star value from 0 to 5")]
    InvalidStarRating,
    #[error("max players must be a positive whole number")]
    InvalidMaxPlayers,
    #[error("a custom-field name is required")]
    CustomFieldNameRequired,
    #[error("bulk 3D model settings cannot carry a game or platform identity")]
    BulkModelSettingsIdentityNotAllowed,
    #[error("bulk 3D model settings require a model type")]
    BulkModelSettingsTypeRequired,
    #[error("invalid bulk 3D model settings: {reason}")]
    InvalidBulkModelSettings { reason: String },
    #[error("select at least one controller to add or remove")]
    ControllerSupportChangeRequired,
    #[error("a support level is required when adding controllers")]
    ControllerSupportLevelRequired,
    #[error("a support level is only valid when adding controllers")]
    UnexpectedControllerSupportLevel,
    #[error("controller-support IDs cannot be empty")]
    EmptyControllerSupportId,
    #[error("controller {id} appears more than once in the bulk controller-support change")]
    DuplicateControllerSupportId { id: String },
    #[error("controller {id} cannot be added and removed in the same bulk change")]
    ConflictingControllerSupportId { id: String },
    #[error("the request supplied a value that is not used by this field")]
    UnexpectedValue,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_keys_are_unique_and_named_13_27_additions_are_present() {
        let keys = BULK_GAME_FIELDS
            .iter()
            .map(|definition| definition.field.key())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(keys.len(), BULK_GAME_FIELDS.len());
        assert!(keys.contains("broken"));
        assert!(keys.contains("controllerSupport"));
        assert!(keys.contains("customDosBoxVersion"));
        assert!(keys.contains("hidden"));
        assert!(keys.contains("modelSettings"));
        assert!(keys.contains("starRating"));
        assert!(keys.contains("videoPath"));
        assert!(keys.contains("customField"));
        assert!(keys.contains("platform"));
        assert_eq!(BULK_GAME_FIELDS.len(), 45);
        assert_eq!(
            BULK_GAME_FIELDS[11..20]
                .iter()
                .map(|definition| definition.label)
                .collect::<Vec<_>>(),
            [
                "Pause Screen - Enable",
                "Pause Screen - Forceful Activation",
                "Pause Screen - Load State AutoHotkey Script",
                "Pause Screen - Override Default Settings",
                "Pause Screen - Pause Game AutoHotkey Script",
                "Pause Screen - Reset Game AutoHotkey Script",
                "Pause Screen - Resume Game AutoHotkey Script",
                "Pause Screen - Save State AutoHotkey Script",
                "Pause Screen - Suspend Game Process On Pause",
            ]
        );
        assert_eq!(
            BULK_GAME_FIELDS[32..39]
                .iter()
                .map(|definition| definition.label)
                .collect::<Vec<_>>(),
            [
                "Startup Screen - Aggressive Startup Window Hiding",
                "Startup Screen - Enabled",
                "Startup Screen - Hide All Non-Exclusive Mode Windows",
                "Startup Screen - Hide Mouse Cursor During Game",
                "Startup Screen - Load Delay",
                "Startup Screen - Override Default Settings",
                "Startup Screen - Shutdown Enabled",
            ]
        );
    }

    #[test]
    fn typed_requests_reject_cross_editor_values() {
        let valid = BulkGameEdit {
            field: BulkGameField::StarRating,
            operation: BulkGameEditOperation::Set,
            text: None,
            boolean: None,
            number: Some(2.5),
            custom_field_name: None,
        };
        assert_eq!(valid.validate(), Ok(()));
        assert_eq!(
            BulkGameEdit {
                number: Some(2.25),
                ..valid.clone()
            }
            .validate(),
            Err(BulkGameEditError::InvalidStarRating)
        );
        assert_eq!(
            BulkGameEdit {
                field: BulkGameField::Publisher,
                text: Some("Publisher".into()),
                number: Some(2.5),
                ..valid
            }
            .validate(),
            Err(BulkGameEditError::UnexpectedValue)
        );

        let delay = BulkGameEdit {
            field: BulkGameField::StartupScreenLoadDelay,
            operation: BulkGameEditOperation::Set,
            text: None,
            boolean: None,
            number: Some(1250.0),
            custom_field_name: None,
        };
        assert_eq!(delay.validate(), Ok(()));
        assert_eq!(
            BulkGameEdit {
                number: Some(1250.5),
                ..delay.clone()
            }
            .validate(),
            Err(BulkGameEditError::InvalidUnsignedInteger)
        );
        assert_eq!(
            BulkGameEdit {
                number: Some(f64::from(u32::MAX) + 1.0),
                ..delay
            }
            .validate(),
            Err(BulkGameEditError::InvalidUnsignedInteger)
        );
    }

    #[test]
    fn multi_value_add_remove_is_case_insensitive_and_stable() {
        assert_eq!(
            mutate_multi_value_text(
                Some("Action; Adventure"),
                BulkGameEditOperation::Add,
                Some("action")
            ),
            Ok(Some("Action; Adventure".into()))
        );
        assert_eq!(
            mutate_multi_value_text(
                Some("Action; Adventure"),
                BulkGameEditOperation::Remove,
                Some("ACTION")
            ),
            Ok(Some("Adventure".into()))
        );
        assert_eq!(
            mutate_multi_value_text(
                Some("Action"),
                BulkGameEditOperation::Remove,
                Some("Action")
            ),
            Ok(None)
        );
    }

    #[test]
    fn controller_support_bulk_edit_requires_a_typed_non_conflicting_change() {
        let valid = BulkControllerSupportEdit {
            add_controller_ids: vec!["fixture-controller".into()],
            remove_controller_ids: vec!["legacy-controller".into()],
            support_level: Some(GameControllerSupportLevel::Required),
        };
        assert_eq!(valid.validate(), Ok(()));
        assert_eq!(
            BulkControllerSupportEdit {
                add_controller_ids: vec!["FIXTURE-CONTROLLER".into()],
                remove_controller_ids: vec!["fixture-controller".into()],
                support_level: Some(GameControllerSupportLevel::FullSupport),
            }
            .validate(),
            Err(BulkGameEditError::ConflictingControllerSupportId {
                id: "fixture-controller".into()
            })
        );
        assert_eq!(
            BulkControllerSupportEdit {
                add_controller_ids: vec!["fixture-controller".into()],
                remove_controller_ids: Vec::new(),
                support_level: None,
            }
            .validate(),
            Err(BulkGameEditError::ControllerSupportLevelRequired)
        );
        assert_eq!(
            BulkControllerSupportEdit {
                add_controller_ids: Vec::new(),
                remove_controller_ids: Vec::new(),
                support_level: None,
            }
            .validate(),
            Err(BulkGameEditError::ControllerSupportChangeRequired)
        );
    }

    #[test]
    fn bulk_model_settings_are_identity_free_or_remove_the_override() {
        assert_eq!(BulkModelSettingsEdit { settings: None }.validate(), Ok(()));

        let valid = BulkModelSettingsEdit {
            settings: Some(ModelSettings::long_jewel_case_defaults()),
        };
        assert_eq!(valid.validate(), Ok(()));

        let mut identified = ModelSettings::box_defaults();
        identified.game_id = Some("fixture-game".into());
        assert_eq!(
            BulkModelSettingsEdit {
                settings: Some(identified),
            }
            .validate(),
            Err(BulkGameEditError::BulkModelSettingsIdentityNotAllowed)
        );

        assert_eq!(
            BulkModelSettingsEdit {
                settings: Some(ModelSettings::default()),
            }
            .validate(),
            Err(BulkGameEditError::BulkModelSettingsTypeRequired)
        );
    }
}
