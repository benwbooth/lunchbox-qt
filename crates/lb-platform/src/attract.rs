use lb_domain::FrontendSettings;

/// Number of wheel advances in one port-owned Attract Mode spin.
///
/// LaunchBox exposes the minimum and maximum timer intervals, but the
/// protected 13.27 implementation does not expose its exact acceleration
/// curve or step count. Keeping the curve here makes the native Qt behavior
/// deterministic and independently testable on every host.
pub const BIG_BOX_ATTRACT_MODE_WHEEL_STEPS: u32 = 16;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BigBoxAttractModePolicy {
    pub enabled: bool,
    pub switch_filters: bool,
    pub delay_seconds: u32,
    pub time_per_movement_seconds: u32,
    pub maximum_speed_ms: u32,
    pub minimum_speed_ms: u32,
    pub play_move_sound: bool,
    pub navigation_sound_volume_percent: u8,
    pub master_volume_percent: u8,
}

impl Default for BigBoxAttractModePolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            switch_filters: true,
            delay_seconds: 120,
            time_per_movement_seconds: 5,
            maximum_speed_ms: 20,
            minimum_speed_ms: 200,
            play_move_sound: false,
            navigation_sound_volume_percent: 15,
            master_volume_percent: 100,
        }
    }
}

impl BigBoxAttractModePolicy {
    pub fn from_settings(settings: Option<&FrontendSettings>) -> Self {
        let fallback = Self::default();
        let Some(settings) = settings else {
            return fallback;
        };

        let mut maximum_speed_ms = bounded_u32(
            settings,
            "AttractModeMaximumSpeed",
            1,
            5_000,
            fallback.maximum_speed_ms,
        );
        let mut minimum_speed_ms = bounded_u32(
            settings,
            "AttractModeMinimumSpeed",
            1,
            5_000,
            fallback.minimum_speed_ms,
        );
        if maximum_speed_ms > minimum_speed_ms {
            maximum_speed_ms = fallback.maximum_speed_ms;
            minimum_speed_ms = fallback.minimum_speed_ms;
        }

        Self {
            enabled: settings
                .get_bool("EnableAttractMode")
                .unwrap_or(fallback.enabled),
            switch_filters: settings
                .get_bool("AttractModeSwitchFilters")
                .unwrap_or(fallback.switch_filters),
            delay_seconds: bounded_u32(
                settings,
                "AttractModeDelay",
                1,
                3_600,
                fallback.delay_seconds,
            ),
            time_per_movement_seconds: bounded_u32(
                settings,
                "AttractModeTimePerMovement",
                1,
                300,
                fallback.time_per_movement_seconds,
            ),
            maximum_speed_ms,
            minimum_speed_ms,
            play_move_sound: settings
                .get_bool("PlayMoveInAttractMode")
                .unwrap_or(fallback.play_move_sound),
            navigation_sound_volume_percent: bounded_percent(
                settings,
                "VolumeAttractModeNavigationSound",
                fallback.navigation_sound_volume_percent,
            ),
            master_volume_percent: bounded_percent(
                settings,
                "VolumeAttractModeMaster",
                fallback.master_volume_percent,
            ),
        }
    }

    /// Returns the timer interval for one step of the deterministic native
    /// wheel spin. Smaller settings represent faster movement, matching the
    /// recovered 20 ms maximum-speed and 200 ms minimum-speed defaults.
    pub fn wheel_interval_ms(&self, step: u32) -> u32 {
        let last_step = BIG_BOX_ATTRACT_MODE_WHEEL_STEPS.saturating_sub(1);
        let bounded_step = step.min(last_step);
        let distance_from_end = bounded_step.min(last_step.saturating_sub(bounded_step));
        let peak_distance = last_step / 2;
        if peak_distance == 0 || self.minimum_speed_ms <= self.maximum_speed_ms {
            return self.minimum_speed_ms;
        }

        let speed_range = u64::from(self.minimum_speed_ms - self.maximum_speed_ms);
        let distance = u64::from(distance_from_end);
        let peak = u64::from(peak_distance);
        let acceleration = speed_range.saturating_mul(distance.saturating_mul(distance))
            / peak.saturating_mul(peak);
        self.minimum_speed_ms
            .saturating_sub(u32::try_from(acceleration).unwrap_or(u32::MAX))
            .max(self.maximum_speed_ms)
    }
}

fn bounded_u32(
    settings: &FrontendSettings,
    key: &str,
    minimum: u32,
    maximum: u32,
    fallback: u32,
) -> u32 {
    settings
        .get_i64(key)
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| (*value >= minimum) && (*value <= maximum))
        .unwrap_or(fallback)
}

fn bounded_percent(settings: &FrontendSettings, key: &str, fallback: u8) -> u8 {
    settings
        .get_i64(key)
        .and_then(|value| u8::try_from(value).ok())
        .filter(|value| *value <= 100)
        .unwrap_or(fallback)
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
    fn defaults_match_fresh_and_existing_launchbox_13_27_installations() {
        assert_eq!(
            BigBoxAttractModePolicy::from_settings(None),
            BigBoxAttractModePolicy {
                enabled: false,
                switch_filters: true,
                delay_seconds: 120,
                time_per_movement_seconds: 5,
                maximum_speed_ms: 20,
                minimum_speed_ms: 200,
                play_move_sound: false,
                navigation_sound_volume_percent: 15,
                master_volume_percent: 100,
            }
        );
    }

    #[test]
    fn reads_every_recovered_attract_mode_scalar() {
        let configured = settings(&[
            ("EnableAttractMode", "true"),
            ("AttractModeSwitchFilters", "false"),
            ("AttractModeDelay", "90"),
            ("AttractModeTimePerMovement", "7"),
            ("AttractModeMaximumSpeed", "12"),
            ("AttractModeMinimumSpeed", "240"),
            ("PlayMoveInAttractMode", "true"),
            ("VolumeAttractModeNavigationSound", "37"),
            ("VolumeAttractModeMaster", "84"),
        ]);
        assert_eq!(
            BigBoxAttractModePolicy::from_settings(Some(&configured)),
            BigBoxAttractModePolicy {
                enabled: true,
                switch_filters: false,
                delay_seconds: 90,
                time_per_movement_seconds: 7,
                maximum_speed_ms: 12,
                minimum_speed_ms: 240,
                play_move_sound: true,
                navigation_sound_volume_percent: 37,
                master_volume_percent: 84,
            }
        );
    }

    #[test]
    fn malformed_out_of_range_and_inverted_values_fail_to_safe_defaults() {
        let malformed = settings(&[
            ("EnableAttractMode", "sometimes"),
            ("AttractModeSwitchFilters", ""),
            ("AttractModeDelay", "0"),
            ("AttractModeTimePerMovement", "301"),
            ("AttractModeMaximumSpeed", "400"),
            ("AttractModeMinimumSpeed", "40"),
            ("PlayMoveInAttractMode", "yes"),
            ("VolumeAttractModeNavigationSound", "-1"),
            ("VolumeAttractModeMaster", "101"),
        ]);
        assert_eq!(
            BigBoxAttractModePolicy::from_settings(Some(&malformed)),
            BigBoxAttractModePolicy::default()
        );
    }

    #[test]
    fn native_wheel_curve_is_bounded_symmetric_and_accelerates_then_decelerates() {
        let policy = BigBoxAttractModePolicy::default();
        let intervals = (0..BIG_BOX_ATTRACT_MODE_WHEEL_STEPS)
            .map(|step| policy.wheel_interval_ms(step))
            .collect::<Vec<_>>();
        assert_eq!(intervals.first(), Some(&policy.minimum_speed_ms));
        assert_eq!(intervals.last(), Some(&policy.minimum_speed_ms));
        assert_eq!(
            intervals,
            intervals.iter().copied().rev().collect::<Vec<_>>()
        );
        assert!(intervals.windows(2).take(7).all(|pair| pair[1] < pair[0]));
        assert!(intervals
            .iter()
            .all(|value| *value >= policy.maximum_speed_ms && *value <= policy.minimum_speed_ms));
        assert_eq!(
            policy.wheel_interval_ms(BIG_BOX_ATTRACT_MODE_WHEEL_STEPS + 100),
            policy.minimum_speed_ms
        );
    }
}
