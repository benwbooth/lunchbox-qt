use crate::FrontendSettings;
use serde::{Deserialize, Serialize};

/// The notification presentation modes persisted by LaunchBox 13.27.
///
/// The integer values are part of the Settings.xml contract and match
/// `Unbroken.LaunchBox.Windows.NotificationCenter.NotificationTypes`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DesktopNotificationType {
    #[default]
    LaunchBoxNotifications,
    WindowsNotifications,
    MessageBoxes,
}

impl DesktopNotificationType {
    pub const ALL: [Self; 3] = [
        Self::LaunchBoxNotifications,
        Self::WindowsNotifications,
        Self::MessageBoxes,
    ];

    pub const fn persisted_value(self) -> i64 {
        match self {
            Self::LaunchBoxNotifications => 0,
            Self::WindowsNotifications => 1,
            Self::MessageBoxes => 2,
        }
    }

    pub const fn key(self) -> &'static str {
        match self {
            Self::LaunchBoxNotifications => "launchBoxNotifications",
            Self::WindowsNotifications => "windowsNotifications",
            Self::MessageBoxes => "messageBoxes",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::LaunchBoxNotifications => "LaunchBox Notifications",
            Self::WindowsNotifications => "Windows Notifications",
            Self::MessageBoxes => "Message Boxes",
        }
    }

    pub fn from_persisted_value(value: i64) -> Option<Self> {
        match value {
            0 => Some(Self::LaunchBoxNotifications),
            1 => Some(Self::WindowsNotifications),
            2 => Some(Self::MessageBoxes),
            _ => None,
        }
    }

    pub fn from_key(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|candidate| candidate.key() == value)
    }
}

/// Desktop tray policy recovered from LaunchBox 13.27's Settings type.
///
/// Fresh 13.27 settings default every tray boolean to false, while
/// `DontSendTrayReminder` also defaults to false; the latter is exposed here
/// as the positive `show_sent_to_tray_notification` flag.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopTrayPolicy {
    pub enabled: bool,
    pub minimize_to_tray: bool,
    pub close_to_tray: bool,
    pub show_sent_to_tray_notification: bool,
    pub notification_type: DesktopNotificationType,
}

impl Default for DesktopTrayPolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            minimize_to_tray: false,
            close_to_tray: false,
            show_sent_to_tray_notification: true,
            notification_type: DesktopNotificationType::LaunchBoxNotifications,
        }
    }
}

impl DesktopTrayPolicy {
    pub fn from_settings(settings: Option<&FrontendSettings>) -> Self {
        let Some(settings) = settings else {
            return Self::default();
        };
        Self {
            enabled: settings.get_bool("EnableSystemTray").unwrap_or(false),
            minimize_to_tray: settings.get_bool("MinimizeToSystemTray").unwrap_or(false),
            close_to_tray: settings.get_bool("CloseToSystemTray").unwrap_or(false),
            show_sent_to_tray_notification: !settings
                .get_bool("DontSendTrayReminder")
                .unwrap_or(false),
            notification_type: settings
                .get_i64("NotificationType")
                .and_then(DesktopNotificationType::from_persisted_value)
                .unwrap_or_default(),
        }
    }

    pub const fn intercepts_minimize(self) -> bool {
        self.enabled && self.minimize_to_tray
    }

    pub const fn intercepts_close(self) -> bool {
        self.enabled && self.close_to_tray
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SettingEntry;

    fn settings(entries: &[(&str, &str)]) -> FrontendSettings {
        FrontendSettings {
            entries: entries
                .iter()
                .map(|(key, value)| SettingEntry {
                    key: (*key).to_string(),
                    value: (*value).to_string(),
                })
                .collect(),
            ..FrontendSettings::default()
        }
    }

    #[test]
    fn fresh_13_27_defaults_match_the_managed_settings_type() {
        assert_eq!(
            DesktopTrayPolicy::from_settings(None),
            DesktopTrayPolicy {
                enabled: false,
                minimize_to_tray: false,
                close_to_tray: false,
                show_sent_to_tray_notification: true,
                notification_type: DesktopNotificationType::LaunchBoxNotifications,
            }
        );
    }

    #[test]
    fn reads_all_five_settings_and_inverts_the_negative_reminder_field() {
        let policy = DesktopTrayPolicy::from_settings(Some(&settings(&[
            ("EnableSystemTray", "true"),
            ("MinimizeToSystemTray", "TRUE"),
            ("CloseToSystemTray", "true"),
            ("DontSendTrayReminder", "true"),
            ("NotificationType", "2"),
        ])));
        assert_eq!(
            policy,
            DesktopTrayPolicy {
                enabled: true,
                minimize_to_tray: true,
                close_to_tray: true,
                show_sent_to_tray_notification: false,
                notification_type: DesktopNotificationType::MessageBoxes,
            }
        );
        assert!(policy.intercepts_minimize());
        assert!(policy.intercepts_close());
    }

    #[test]
    fn malformed_values_fall_back_to_fresh_defaults() {
        let policy = DesktopTrayPolicy::from_settings(Some(&settings(&[
            ("EnableSystemTray", "yes"),
            ("MinimizeToSystemTray", ""),
            ("CloseToSystemTray", "1"),
            ("DontSendTrayReminder", "no"),
            ("NotificationType", "9"),
        ])));
        assert_eq!(policy, DesktopTrayPolicy::default());
    }

    #[test]
    fn disabled_policy_preserves_choices_but_does_not_intercept_windows() {
        let policy = DesktopTrayPolicy {
            enabled: false,
            minimize_to_tray: true,
            close_to_tray: true,
            ..DesktopTrayPolicy::default()
        };
        assert!(!policy.intercepts_minimize());
        assert!(!policy.intercepts_close());
    }

    #[test]
    fn notification_type_keys_labels_and_values_are_stable() {
        assert_eq!(
            DesktopNotificationType::ALL.map(DesktopNotificationType::persisted_value),
            [0, 1, 2]
        );
        assert_eq!(
            DesktopNotificationType::ALL.map(DesktopNotificationType::key),
            [
                "launchBoxNotifications",
                "windowsNotifications",
                "messageBoxes"
            ]
        );
        assert_eq!(
            DesktopNotificationType::ALL.map(DesktopNotificationType::label),
            [
                "LaunchBox Notifications",
                "Windows Notifications",
                "Message Boxes"
            ]
        );
    }
}
