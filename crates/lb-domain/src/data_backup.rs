use crate::FrontendSettings;
use serde::{Deserialize, Serialize};

/// LaunchBox application-data backup policy recovered from 13.27.
///
/// A fresh Settings.xml persists `AutoBackup=true`. Missing or malformed
/// values deliberately fall back to that fresh-install recommendation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationDataBackupPolicy {
    pub enabled: bool,
}

impl Default for ApplicationDataBackupPolicy {
    fn default() -> Self {
        Self { enabled: true }
    }
}

impl ApplicationDataBackupPolicy {
    pub fn from_settings(settings: Option<&FrontendSettings>) -> Self {
        Self {
            enabled: settings
                .and_then(|settings| settings.get_bool("AutoBackup"))
                .unwrap_or(true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SettingEntry;

    fn settings(value: &str) -> FrontendSettings {
        FrontendSettings {
            entries: vec![SettingEntry {
                key: "AutoBackup".into(),
                value: value.into(),
            }],
            ..FrontendSettings::default()
        }
    }

    #[test]
    fn fresh_missing_and_malformed_values_remain_recommended() {
        assert_eq!(
            ApplicationDataBackupPolicy::from_settings(None),
            ApplicationDataBackupPolicy { enabled: true }
        );
        assert!(ApplicationDataBackupPolicy::from_settings(Some(&settings("invalid"))).enabled);
    }

    #[test]
    fn reads_both_persisted_boolean_values() {
        assert!(ApplicationDataBackupPolicy::from_settings(Some(&settings("true"))).enabled);
        assert!(!ApplicationDataBackupPolicy::from_settings(Some(&settings("false"))).enabled);
    }
}
