use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BulkGameEditorKind {
    Text,
    MultilineText,
    Boolean,
    Rating,
    Date,
    LexicalPath,
    Emulator,
    MultiValue,
    CustomField,
}

impl BulkGameEditorKind {
    pub const fn key(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::MultilineText => "multilineText",
            Self::Boolean => "boolean",
            Self::Rating => "rating",
            Self::Date => "date",
            Self::LexicalPath => "lexicalPath",
            Self::Emulator => "emulator",
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
    Developer,
    Emulator,
    Favorite,
    Genre,
    Hidden,
    MaxPlayers,
    Notes,
    PlayMode,
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
            Self::Developer => "developer",
            Self::Emulator => "emulator",
            Self::Favorite => "favorite",
            Self::Genre => "genre",
            Self::Hidden => "hidden",
            Self::MaxPlayers => "maxPlayers",
            Self::Notes => "notes",
            Self::PlayMode => "playMode",
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
    BulkGameFieldDefinition {
        field: BulkGameField::Broken,
        label: "Broken",
        editor: BulkGameEditorKind::Boolean,
        clearable: false,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::Completed,
        label: "Completed",
        editor: BulkGameEditorKind::Boolean,
        clearable: false,
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
        field: BulkGameField::VideoPath,
        label: "Video Path",
        editor: BulkGameEditorKind::LexicalPath,
        clearable: true,
    },
    BulkGameFieldDefinition {
        field: BulkGameField::WikipediaUrl,
        label: "Wikipedia URL",
        editor: BulkGameEditorKind::Text,
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
    #[error("star rating must be a half-star value from 0 to 5")]
    InvalidStarRating,
    #[error("max players must be a positive whole number")]
    InvalidMaxPlayers,
    #[error("a custom-field name is required")]
    CustomFieldNameRequired,
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
        assert!(keys.contains("hidden"));
        assert!(keys.contains("starRating"));
        assert!(keys.contains("videoPath"));
        assert!(keys.contains("customField"));
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
}
