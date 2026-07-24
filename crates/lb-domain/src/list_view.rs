use crate::FrontendSettings;
use std::collections::BTreeSet;
use thiserror::Error;

/// The 35 desktop game-list columns recovered from LaunchBox 13.27.
///
/// The numeric index is the stable WPF `DataGrid.Columns` index used by
/// `ListViewVisibleColumnIndexPriorities`. It is deliberately independent of
/// display order: Badges is column 33 but LaunchBox puts it first by default.
pub const LAUNCHBOX_LIST_VIEW_COLUMNS: [(&str, usize); 35] = [
    ("Title", 0),
    ("Platform", 1),
    ("Developer", 2),
    ("Publisher", 3),
    ("Release Date", 4),
    ("Rating", 5),
    ("Genre", 6),
    ("Series", 7),
    ("Region", 8),
    ("Play Mode", 9),
    ("Version", 10),
    ("Status", 11),
    ("Source", 12),
    ("Last Played", 13),
    ("Added", 14),
    ("Modified", 15),
    ("Play Count", 16),
    ("Favorite", 17),
    ("Completed", 18),
    ("Broken", 19),
    ("Portable", 20),
    ("Hide", 21),
    ("Star Rating", 22),
    ("Community Star Rating", 23),
    ("Community Star Rating Count", 24),
    ("Alternate Names", 25),
    ("Wikipedia URL", 26),
    ("Max Players", 27),
    ("Release Type", 28),
    ("Video URL", 29),
    ("Installed", 30),
    ("Application Path", 31),
    ("Launchbox Database ID", 32),
    ("Badges", 33),
    ("Play Time", 34),
];

pub const LAUNCHBOX_LIST_VIEW_DEFAULT_ORDER: [&str; 35] = [
    "Badges",
    "Title",
    "Platform",
    "Developer",
    "Publisher",
    "Release Date",
    "Rating",
    "Genre",
    "Series",
    "Region",
    "Play Mode",
    "Version",
    "Status",
    "Source",
    "Last Played",
    "Added",
    "Modified",
    "Play Count",
    "Favorite",
    "Completed",
    "Broken",
    "Portable",
    "Hide",
    "Star Rating",
    "Community Star Rating",
    "Community Star Rating Count",
    "Alternate Names",
    "Wikipedia URL",
    "Max Players",
    "Release Type",
    "Video URL",
    "Installed",
    "Application Path",
    "Launchbox Database ID",
    "Play Time",
];

pub const LAUNCHBOX_LIST_VIEW_DEFAULT_VISIBLE_INDEXES: [usize; 35] = [
    33, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
    25, 26, 27, 28, 29, 30, 31, 32, 34,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListViewColumnLayout {
    pub ordered_columns: Vec<String>,
    pub visible_column_indexes: Vec<usize>,
}

impl Default for ListViewColumnLayout {
    fn default() -> Self {
        Self {
            ordered_columns: LAUNCHBOX_LIST_VIEW_DEFAULT_ORDER
                .into_iter()
                .map(str::to_string)
                .collect(),
            visible_column_indexes: LAUNCHBOX_LIST_VIEW_DEFAULT_VISIBLE_INDEXES.to_vec(),
        }
    }
}

impl ListViewColumnLayout {
    pub fn from_settings(settings: Option<&FrontendSettings>) -> Self {
        settings
            .and_then(|settings| {
                Self::parse(
                    settings.get("ListViewOrderedColumnPriorities")?,
                    settings.get("ListViewVisibleColumnIndexPriorities")?,
                )
                .ok()
            })
            .unwrap_or_default()
    }

    pub fn parse(ordered: &str, visible_indexes: &str) -> Result<Self, ListViewLayoutError> {
        let ordered_columns = ordered
            .split(',')
            .map(str::trim)
            .map(str::to_string)
            .collect::<Vec<_>>();
        let visible_column_indexes = if visible_indexes.trim().is_empty() {
            Vec::new()
        } else {
            visible_indexes
                .split(',')
                .map(str::trim)
                .map(|value| {
                    value.parse::<usize>().map_err(|_| {
                        ListViewLayoutError::InvalidVisibleColumnIndex {
                            value: value.to_string(),
                        }
                    })
                })
                .collect::<Result<Vec<_>, _>>()?
        };
        let layout = Self {
            ordered_columns,
            visible_column_indexes,
        };
        layout.validate()?;
        Ok(layout)
    }

    pub fn validate(&self) -> Result<(), ListViewLayoutError> {
        if self.ordered_columns.len() != LAUNCHBOX_LIST_VIEW_COLUMNS.len() {
            return Err(ListViewLayoutError::WrongOrderedColumnCount {
                actual: self.ordered_columns.len(),
            });
        }
        let mut names = BTreeSet::new();
        for name in &self.ordered_columns {
            if list_view_column_index(name).is_none() {
                return Err(ListViewLayoutError::UnknownColumn { name: name.clone() });
            }
            if !names.insert(name.as_str()) {
                return Err(ListViewLayoutError::DuplicateColumn { name: name.clone() });
            }
        }
        if self.visible_column_indexes.is_empty() {
            return Err(ListViewLayoutError::NoVisibleColumns);
        }
        let mut indexes = BTreeSet::new();
        for index in &self.visible_column_indexes {
            if list_view_column_name(*index).is_none() {
                return Err(ListViewLayoutError::UnknownVisibleColumnIndex { index: *index });
            }
            if !indexes.insert(*index) {
                return Err(ListViewLayoutError::DuplicateVisibleColumnIndex { index: *index });
            }
        }
        Ok(())
    }

    pub fn ordered_setting(&self) -> String {
        self.ordered_columns.join(",")
    }

    pub fn visible_indexes_setting(&self) -> String {
        self.visible_column_indexes
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}

pub fn list_view_column_index(name: &str) -> Option<usize> {
    LAUNCHBOX_LIST_VIEW_COLUMNS
        .iter()
        .find_map(|(candidate, index)| (*candidate == name).then_some(*index))
}

pub fn list_view_column_name(index: usize) -> Option<&'static str> {
    LAUNCHBOX_LIST_VIEW_COLUMNS
        .iter()
        .find_map(|(name, candidate)| (*candidate == index).then_some(*name))
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ListViewLayoutError {
    #[error("LaunchBox list layout has {actual} ordered columns instead of the required 35")]
    WrongOrderedColumnCount { actual: usize },
    #[error("LaunchBox list layout contains unknown column {name}")]
    UnknownColumn { name: String },
    #[error("LaunchBox list layout repeats column {name}")]
    DuplicateColumn { name: String },
    #[error("LaunchBox list layout has no visible columns")]
    NoVisibleColumns,
    #[error("LaunchBox list layout contains invalid visible column index {value}")]
    InvalidVisibleColumnIndex { value: String },
    #[error("LaunchBox list layout contains unknown visible column index {index}")]
    UnknownVisibleColumnIndex { index: usize },
    #[error("LaunchBox list layout repeats visible column index {index}")]
    DuplicateVisibleColumnIndex { index: usize },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SettingEntry;

    #[test]
    fn recovered_default_uses_stable_wpf_indexes_independently_of_display_order() {
        let layout = ListViewColumnLayout::default();
        assert_eq!(layout.ordered_columns[0], "Badges");
        assert_eq!(layout.visible_column_indexes[0], 33);
        assert_eq!(list_view_column_index("Title"), Some(0));
        assert_eq!(list_view_column_index("Badges"), Some(33));
        assert_eq!(list_view_column_name(34), Some("Play Time"));
        assert_eq!(
            layout.ordered_setting(),
            LAUNCHBOX_LIST_VIEW_DEFAULT_ORDER.join(",")
        );
        assert_eq!(
            layout.visible_indexes_setting(),
            LAUNCHBOX_LIST_VIEW_DEFAULT_VISIBLE_INDEXES
                .map(|index| index.to_string())
                .join(",")
        );
    }

    #[test]
    fn settings_preserve_reordering_and_hidden_stable_indexes() {
        let mut expected = ListViewColumnLayout::default();
        expected.ordered_columns.swap(1, 17);
        expected.visible_column_indexes.retain(|index| *index != 3);
        let settings = FrontendSettings {
            entries: vec![
                SettingEntry {
                    key: "ListViewOrderedColumnPriorities".into(),
                    value: expected.ordered_setting(),
                },
                SettingEntry {
                    key: "ListViewVisibleColumnIndexPriorities".into(),
                    value: expected.visible_indexes_setting(),
                },
            ],
            ..FrontendSettings::default()
        };
        assert_eq!(
            ListViewColumnLayout::from_settings(Some(&settings)),
            expected
        );
    }

    #[test]
    fn missing_future_duplicate_and_empty_settings_fail_closed_to_defaults() {
        assert_eq!(
            ListViewColumnLayout::from_settings(None),
            Default::default()
        );
        assert_eq!(
            ListViewColumnLayout::parse("Title,Future", "0"),
            Err(ListViewLayoutError::WrongOrderedColumnCount { actual: 2 })
        );

        let mut duplicate = ListViewColumnLayout::default();
        duplicate.ordered_columns[1] = "Badges".into();
        assert!(matches!(
            duplicate.validate(),
            Err(ListViewLayoutError::DuplicateColumn { .. })
        ));

        let mut hidden = ListViewColumnLayout::default();
        hidden.visible_column_indexes.clear();
        assert_eq!(
            hidden.validate(),
            Err(ListViewLayoutError::NoVisibleColumns)
        );
    }
}
