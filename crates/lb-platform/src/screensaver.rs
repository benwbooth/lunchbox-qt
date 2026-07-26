use crate::media::{GameMediaItem, GameMediaKind};
use lb_domain::{FrontendSettings, Game};
use std::collections::BTreeMap;
use std::path::PathBuf;

const BACKGROUND_MEDIA_TYPES: &[&str] = &[
    "Fanart - Background",
    "Uplay Background",
    "Origin Background",
    "Steam Screenshot",
    "Origin Screenshot",
    "Amazon Screenshot",
    "GOG Screenshot",
    "Screenshot - Gameplay",
    "Screenshot",
    "Epic Games Screenshot",
    "Epic Games Background",
    "Backgrounds",
];

const SCREENSHOT_MEDIA_TYPES: &[&str] = &["Screenshot - Gameplay", "Screenshot"];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BigBoxScreensaverView {
    #[default]
    Screensaver1,
    Screensaver2,
    Screensaver3,
    Screensaver4,
}

impl BigBoxScreensaverView {
    pub const fn key(self) -> &'static str {
        match self {
            Self::Screensaver1 => "Screensaver1View",
            Self::Screensaver2 => "Screensaver2View",
            Self::Screensaver3 => "Screensaver3View",
            Self::Screensaver4 => "Screensaver4View",
        }
    }

    pub const fn ordinal(self) -> u8 {
        match self {
            Self::Screensaver1 => 1,
            Self::Screensaver2 => 2,
            Self::Screensaver3 => 3,
            Self::Screensaver4 => 4,
        }
    }

    fn from_stored(value: Option<&str>) -> Self {
        let value = value.unwrap_or_default().trim();
        if value.is_empty()
            || value.eq_ignore_ascii_case("Screensaver1View")
            || value.eq_ignore_ascii_case("Screensaver 1")
        {
            Self::Screensaver1
        } else if value.eq_ignore_ascii_case("Screensaver2View")
            || value.eq_ignore_ascii_case("Screensaver 2")
        {
            Self::Screensaver2
        } else if value.eq_ignore_ascii_case("Screensaver3View")
            || value.eq_ignore_ascii_case("Screensaver 3")
        {
            Self::Screensaver3
        } else if value.eq_ignore_ascii_case("Screensaver4View")
            || value.eq_ignore_ascii_case("Screensaver 4")
        {
            Self::Screensaver4
        } else {
            Self::default()
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BigBoxScreensaverPolicy {
    pub enabled: bool,
    pub delay_seconds: u32,
    pub minimum_swap_time_ms: u32,
    pub maximum_swap_time_ms: u32,
    pub skip_games_missing_background: bool,
    pub skip_games_missing_box_art: bool,
    pub skip_games_missing_video: bool,
    pub view: BigBoxScreensaverView,
    pub video_volume_percent: u8,
    pub master_volume_percent: u8,
}

impl Default for BigBoxScreensaverPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            delay_seconds: 300,
            minimum_swap_time_ms: 30_000,
            maximum_swap_time_ms: 60_000,
            skip_games_missing_background: true,
            skip_games_missing_box_art: true,
            skip_games_missing_video: false,
            view: BigBoxScreensaverView::Screensaver1,
            video_volume_percent: 75,
            master_volume_percent: 100,
        }
    }
}

impl BigBoxScreensaverPolicy {
    pub fn from_settings(settings: Option<&FrontendSettings>) -> Self {
        let fallback = Self::default();
        let Some(settings) = settings else {
            return fallback;
        };

        let mut minimum_swap_time_ms = bounded_u32(
            settings,
            "ScreensaverMinimumSwapTime",
            100,
            3_600_000,
            fallback.minimum_swap_time_ms,
        );
        let mut maximum_swap_time_ms = bounded_u32(
            settings,
            "ScreensaverMaximumSwapTime",
            100,
            3_600_000,
            fallback.maximum_swap_time_ms,
        );
        if minimum_swap_time_ms > maximum_swap_time_ms {
            minimum_swap_time_ms = fallback.minimum_swap_time_ms;
            maximum_swap_time_ms = fallback.maximum_swap_time_ms;
        }

        Self {
            enabled: settings
                .get_bool("EnableScreensaver")
                .unwrap_or(fallback.enabled),
            delay_seconds: bounded_u32(
                settings,
                "ScreensaverDelay",
                1,
                86_400,
                fallback.delay_seconds,
            ),
            minimum_swap_time_ms,
            maximum_swap_time_ms,
            skip_games_missing_background: settings
                .get_bool("ScreensaverSkipGamesMissingBackground")
                .unwrap_or(fallback.skip_games_missing_background),
            skip_games_missing_box_art: settings
                .get_bool("ScreensaverSkipGamesMissingBoxArt")
                .unwrap_or(fallback.skip_games_missing_box_art),
            skip_games_missing_video: settings
                .get_bool("ScreensaverSkipGamesMissingVideo")
                .unwrap_or(fallback.skip_games_missing_video),
            view: BigBoxScreensaverView::from_stored(settings.get("ScreensaverView")),
            video_volume_percent: bounded_percent(
                settings,
                "VolumeVideo",
                fallback.video_volume_percent,
            ),
            master_volume_percent: bounded_percent(
                settings,
                "VolumeMaster",
                fallback.master_volume_percent,
            ),
        }
    }

    /// Maps caller-provided entropy to the inclusive recovered swap-time range.
    ///
    /// LaunchBox 13.27 exposes a random-number generator in the protected swap
    /// loop but not its exact rounding rule. This native rule is deterministic
    /// for tests and includes both configured endpoints.
    pub fn swap_time_ms(&self, entropy: u64) -> u32 {
        if self.minimum_swap_time_ms >= self.maximum_swap_time_ms {
            return self.minimum_swap_time_ms;
        }
        let width = u64::from(self.maximum_swap_time_ms - self.minimum_swap_time_ms) + 1;
        self.minimum_swap_time_ms
            .saturating_add(u32::try_from(entropy % width).unwrap_or_default())
    }

    pub fn allows_media(&self, media: &BigBoxScreensaverMedia) -> bool {
        (!self.skip_games_missing_background || media.background_path.is_some())
            && (!self.skip_games_missing_box_art || media.box_art_path.is_some())
            && (!self.skip_games_missing_video || media.video_path.is_some())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BigBoxScreensaverMedia {
    pub background_path: Option<PathBuf>,
    pub box_art_path: Option<PathBuf>,
    pub screenshot_path: Option<PathBuf>,
    pub video_path: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BigBoxScreensaverCandidate {
    pub game_id: String,
    pub title: String,
    pub platform: String,
    pub genre: String,
    pub release_date: String,
    pub play_time_seconds: u64,
    pub star_rating: f64,
    pub media: BigBoxScreensaverMedia,
}

/// Projects the full library into the bounded media records consumed by the
/// native Qt screensaver. Hidden games remain hidden, stable IDs survive any
/// list sorting, and every path comes from the existing guarded media index.
pub fn project_big_box_screensaver_candidates(
    games: &[Game],
    media_by_game_id: &BTreeMap<String, Vec<GameMediaItem>>,
    front_paths_by_game_id: &BTreeMap<String, PathBuf>,
    policy: &BigBoxScreensaverPolicy,
) -> Vec<BigBoxScreensaverCandidate> {
    games
        .iter()
        .filter(|game| !game.hidden)
        .filter_map(|game| {
            let items = media_by_game_id
                .get(&game.id)
                .map(Vec::as_slice)
                .unwrap_or_default();
            let media = BigBoxScreensaverMedia {
                background_path: first_image_path(items, BACKGROUND_MEDIA_TYPES),
                box_art_path: front_paths_by_game_id.get(&game.id).cloned(),
                screenshot_path: first_image_path(items, SCREENSHOT_MEDIA_TYPES),
                video_path: items
                    .iter()
                    .find(|item| item.kind == GameMediaKind::Video)
                    .map(|item| item.path.clone()),
            };
            policy
                .allows_media(&media)
                .then(|| BigBoxScreensaverCandidate {
                    game_id: game.id.clone(),
                    title: game.title.clone(),
                    platform: game.platform.clone(),
                    genre: game.genre.clone().unwrap_or_default(),
                    release_date: game.release_date.clone().unwrap_or_default(),
                    play_time_seconds: game.play_time_seconds,
                    star_rating: effective_star_rating(game),
                    media,
                })
        })
        .collect()
}

/// Selects one candidate index from injected entropy and avoids an active game
/// whenever at least two candidates exist.
pub fn select_big_box_screensaver_candidate(
    candidates: &[BigBoxScreensaverCandidate],
    avoid_game_id: Option<&str>,
    entropy: u64,
) -> Option<usize> {
    if candidates.is_empty() {
        return None;
    }
    let indices = candidates
        .iter()
        .enumerate()
        .filter_map(|(index, candidate)| {
            (candidates.len() == 1 || Some(candidate.game_id.as_str()) != avoid_game_id)
                .then_some(index)
        })
        .collect::<Vec<_>>();
    let index = usize::try_from(entropy % indices.len() as u64).unwrap_or_default();
    indices.get(index).copied()
}

fn first_image_path(items: &[GameMediaItem], priorities: &[&str]) -> Option<PathBuf> {
    priorities.iter().find_map(|priority| {
        items
            .iter()
            .find(|item| {
                item.kind == GameMediaKind::Image && item.media_type.eq_ignore_ascii_case(priority)
            })
            .map(|item| item.path.clone())
    })
}

fn effective_star_rating(game: &Game) -> f64 {
    if game.community_star_rating.is_finite() && game.community_star_rating > 0.0 {
        game.community_star_rating
    } else if game.star_rating_float.is_finite() && game.star_rating_float > 0.0 {
        game.star_rating_float
    } else {
        f64::from(game.star_rating)
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
    use std::path::Path;

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

    fn media(kind: GameMediaKind, media_type: &str, path: &str) -> GameMediaItem {
        GameMediaItem {
            kind,
            media_type: media_type.into(),
            path: Path::new(path).to_path_buf(),
            region: None,
            ordinal: 0,
        }
    }

    #[test]
    fn defaults_match_fresh_and_existing_launchbox_13_27_installations() {
        assert_eq!(
            BigBoxScreensaverPolicy::from_settings(None),
            BigBoxScreensaverPolicy {
                enabled: true,
                delay_seconds: 300,
                minimum_swap_time_ms: 30_000,
                maximum_swap_time_ms: 60_000,
                skip_games_missing_background: true,
                skip_games_missing_box_art: true,
                skip_games_missing_video: false,
                view: BigBoxScreensaverView::Screensaver1,
                video_volume_percent: 75,
                master_volume_percent: 100,
            }
        );
    }

    #[test]
    fn reads_every_recovered_screensaver_scalar_and_all_four_view_keys() {
        for (stored, expected) in [
            ("", BigBoxScreensaverView::Screensaver1),
            ("Screensaver1View", BigBoxScreensaverView::Screensaver1),
            ("Screensaver2View", BigBoxScreensaverView::Screensaver2),
            ("Screensaver3View", BigBoxScreensaverView::Screensaver3),
            ("Screensaver4View", BigBoxScreensaverView::Screensaver4),
        ] {
            let configured = settings(&[
                ("EnableScreensaver", "false"),
                ("ScreensaverDelay", "90"),
                ("ScreensaverMinimumSwapTime", "1200"),
                ("ScreensaverMaximumSwapTime", "3400"),
                ("ScreensaverSkipGamesMissingBackground", "false"),
                ("ScreensaverSkipGamesMissingBoxArt", "false"),
                ("ScreensaverSkipGamesMissingVideo", "true"),
                ("ScreensaverView", stored),
                ("VolumeVideo", "61"),
                ("VolumeMaster", "84"),
            ]);
            assert_eq!(
                BigBoxScreensaverPolicy::from_settings(Some(&configured)),
                BigBoxScreensaverPolicy {
                    enabled: false,
                    delay_seconds: 90,
                    minimum_swap_time_ms: 1200,
                    maximum_swap_time_ms: 3400,
                    skip_games_missing_background: false,
                    skip_games_missing_box_art: false,
                    skip_games_missing_video: true,
                    view: expected,
                    video_volume_percent: 61,
                    master_volume_percent: 84,
                }
            );
        }
    }

    #[test]
    fn malformed_out_of_range_inverted_and_unknown_values_fail_to_defaults() {
        let malformed = settings(&[
            ("EnableScreensaver", "sometimes"),
            ("ScreensaverDelay", "0"),
            ("ScreensaverMinimumSwapTime", "90000"),
            ("ScreensaverMaximumSwapTime", "80000"),
            ("ScreensaverSkipGamesMissingBackground", ""),
            ("ScreensaverSkipGamesMissingBoxArt", "yes"),
            ("ScreensaverSkipGamesMissingVideo", "no"),
            ("ScreensaverView", "Screensaver99View"),
            ("VolumeVideo", "-1"),
            ("VolumeMaster", "101"),
        ]);
        assert_eq!(
            BigBoxScreensaverPolicy::from_settings(Some(&malformed)),
            BigBoxScreensaverPolicy::default()
        );
    }

    #[test]
    fn random_swap_time_is_inclusive_and_fixed_ranges_stay_fixed() {
        let mut policy = BigBoxScreensaverPolicy {
            minimum_swap_time_ms: 300,
            maximum_swap_time_ms: 302,
            ..BigBoxScreensaverPolicy::default()
        };
        assert_eq!(policy.swap_time_ms(0), 300);
        assert_eq!(policy.swap_time_ms(1), 301);
        assert_eq!(policy.swap_time_ms(2), 302);
        assert_eq!(policy.swap_time_ms(3), 300);
        policy.maximum_swap_time_ms = 300;
        assert_eq!(policy.swap_time_ms(u64::MAX), 300);
    }

    #[test]
    fn candidate_projection_uses_safe_indexed_media_and_all_three_skip_switches() {
        let visible = Game {
            id: "visible".into(),
            title: "Visible Game".into(),
            platform: "Console".into(),
            genre: Some("Arcade".into()),
            release_date: Some("1999-03-04".into()),
            play_time_seconds: 7200,
            community_star_rating: 4.25,
            ..Game::default()
        };
        let hidden = Game {
            id: "hidden".into(),
            title: "Hidden Game".into(),
            hidden: true,
            ..Game::default()
        };
        let media_items = BTreeMap::from([(
            visible.id.clone(),
            vec![
                media(
                    GameMediaKind::Image,
                    "Screenshot - Gameplay",
                    "/library/screenshot.png",
                ),
                media(
                    GameMediaKind::Image,
                    "Fanart - Background",
                    "/library/background.png",
                ),
                media(GameMediaKind::Video, "Theme Video", "/library/video.mp4"),
            ],
        )]);
        let fronts = BTreeMap::from([(
            visible.id.clone(),
            Path::new("/library/box.png").to_path_buf(),
        )]);
        let projected = project_big_box_screensaver_candidates(
            &[visible, hidden],
            &media_items,
            &fronts,
            &BigBoxScreensaverPolicy {
                skip_games_missing_video: true,
                ..BigBoxScreensaverPolicy::default()
            },
        );
        assert_eq!(projected.len(), 1);
        assert_eq!(projected[0].game_id, "visible");
        assert_eq!(
            projected[0].media,
            BigBoxScreensaverMedia {
                background_path: Some(Path::new("/library/background.png").to_path_buf()),
                box_art_path: Some(Path::new("/library/box.png").to_path_buf()),
                screenshot_path: Some(Path::new("/library/screenshot.png").to_path_buf()),
                video_path: Some(Path::new("/library/video.mp4").to_path_buf()),
            }
        );
        assert_eq!(projected[0].star_rating, 4.25);

        let absent = project_big_box_screensaver_candidates(
            &[Game {
                id: "absent".into(),
                title: "No Media".into(),
                ..Game::default()
            }],
            &BTreeMap::new(),
            &BTreeMap::new(),
            &BigBoxScreensaverPolicy::default(),
        );
        assert!(absent.is_empty());

        let retained = project_big_box_screensaver_candidates(
            &[Game {
                id: "absent".into(),
                title: "No Media".into(),
                ..Game::default()
            }],
            &BTreeMap::new(),
            &BTreeMap::new(),
            &BigBoxScreensaverPolicy {
                skip_games_missing_background: false,
                skip_games_missing_box_art: false,
                ..BigBoxScreensaverPolicy::default()
            },
        );
        assert_eq!(retained.len(), 1);
    }

    #[test]
    fn candidate_selection_is_bounded_deterministic_and_avoids_current_game() {
        let candidate = |id: &str| BigBoxScreensaverCandidate {
            game_id: id.into(),
            title: id.into(),
            platform: String::new(),
            genre: String::new(),
            release_date: String::new(),
            play_time_seconds: 0,
            star_rating: 0.0,
            media: BigBoxScreensaverMedia::default(),
        };
        let candidates = [candidate("a"), candidate("b"), candidate("c")];
        assert_eq!(
            select_big_box_screensaver_candidate(&candidates, Some("a"), 0),
            Some(1)
        );
        assert_eq!(
            select_big_box_screensaver_candidate(&candidates, Some("a"), 1),
            Some(2)
        );
        assert_eq!(
            select_big_box_screensaver_candidate(&candidates[..1], Some("a"), 99),
            Some(0)
        );
        assert_eq!(
            select_big_box_screensaver_candidate(&[], Some("a"), 0),
            None
        );
    }
}
