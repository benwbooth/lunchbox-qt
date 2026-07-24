use crate::FrontendSettings;
use thiserror::Error;

/// LaunchBox 13.27's modern desktop-theme box-size slider contract.
///
/// The value is a normalized single-precision scale stored in
/// `Settings.xml` as `NextBoxSize`. The stock theme uses a 0.05 through 0.50
/// slider, with 0.001 small changes and 0.01 large changes.
pub const MIN_BOX_SIZE: f64 = 0.05;
pub const MAX_BOX_SIZE: f64 = 0.50;
pub const BOX_SIZE_STEP: f64 = 0.001;
pub const BOX_SIZE_LARGE_STEP: f64 = 0.01;
pub const DEFAULT_BOX_SIZE: f64 = 0.172_142_86;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxSize(f32);

impl Default for BoxSize {
    fn default() -> Self {
        Self(DEFAULT_BOX_SIZE as f32)
    }
}

impl BoxSize {
    pub fn new(value: f64) -> Result<Self, BoxSizeError> {
        if !value.is_finite() || !(MIN_BOX_SIZE..=MAX_BOX_SIZE).contains(&value) {
            return Err(BoxSizeError::InvalidValue {
                value: value.to_string(),
            });
        }
        Ok(Self(value as f32))
    }

    pub fn from_settings(settings: Option<&FrontendSettings>) -> Self {
        settings
            .and_then(|settings| settings.get("NextBoxSize"))
            .and_then(|value| value.parse::<f32>().ok())
            .filter(|value| {
                value.is_finite() && (MIN_BOX_SIZE as f32..=MAX_BOX_SIZE as f32).contains(value)
            })
            .map(Self)
            .unwrap_or_default()
    }

    pub fn value(self) -> f64 {
        f64::from(self.0)
    }

    pub fn setting_value(self) -> String {
        self.0.to_string()
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum BoxSizeError {
    #[error("box size {value} is outside LaunchBox 13.27's supported 0.05 through 0.50 range")]
    InvalidValue { value: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SettingEntry;

    fn settings(value: &str) -> FrontendSettings {
        FrontendSettings {
            entries: vec![SettingEntry {
                key: "NextBoxSize".into(),
                value: value.into(),
            }],
            ..FrontendSettings::default()
        }
    }

    #[test]
    fn recovered_1327_default_and_slider_contract_are_exact() {
        let size = BoxSize::default();
        assert_eq!(size.setting_value(), "0.17214286");
        assert!((size.value() - DEFAULT_BOX_SIZE).abs() < f64::from(f32::EPSILON));
        assert_eq!(MIN_BOX_SIZE, 0.05);
        assert_eq!(MAX_BOX_SIZE, 0.50);
        assert_eq!(BOX_SIZE_STEP, 0.001);
        assert_eq!(BOX_SIZE_LARGE_STEP, 0.01);
    }

    #[test]
    fn settings_accept_bounded_single_precision_values() {
        assert_eq!(
            BoxSize::from_settings(Some(&settings("0.31"))).setting_value(),
            "0.31"
        );
        assert_eq!(
            BoxSize::from_settings(Some(&settings("0.05"))).setting_value(),
            "0.05"
        );
        assert_eq!(
            BoxSize::from_settings(Some(&settings("0.5"))).setting_value(),
            "0.5"
        );
    }

    #[test]
    fn missing_malformed_non_finite_and_out_of_range_values_fail_closed() {
        for value in ["", "future", "NaN", "inf", "0.049", "0.501"] {
            assert_eq!(
                BoxSize::from_settings(Some(&settings(value))),
                BoxSize::default(),
                "{value}"
            );
        }
        assert_eq!(BoxSize::from_settings(None), BoxSize::default());
        assert!(BoxSize::new(f64::NAN).is_err());
        assert!(BoxSize::new(0.049).is_err());
        assert!(BoxSize::new(0.501).is_err());
    }
}
