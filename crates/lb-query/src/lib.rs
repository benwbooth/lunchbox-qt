use chrono::{DateTime, NaiveDate, NaiveDateTime};
use lb_domain::Game;
use std::cmp::Ordering;

mod discovery;
mod related;

pub use discovery::*;
pub use related::*;

/// Stable LaunchBox `Settings.xml` values used by the desktop Arrange By
/// control. The enum deliberately lives in the platform-neutral query crate so
/// neither QML shell nor any host OS gets a second sorting implementation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum GameSort {
    #[default]
    Title,
    SortTitle,
    Platform,
    ReleaseDate,
    DateAdded,
    DateModified,
    LastPlayed,
    PlayCount,
    PlayTime,
    StarRating,
    CommunityStarRating,
    Developer,
    Publisher,
    Genre,
    Series,
    Status,
    Favorite,
}

impl GameSort {
    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "" | "Title" => Some(Self::Title),
            "SortTitle" => Some(Self::SortTitle),
            "Platform" => Some(Self::Platform),
            "ReleaseDate" => Some(Self::ReleaseDate),
            "DateAdded" => Some(Self::DateAdded),
            "DateModified" => Some(Self::DateModified),
            "LastPlayed" => Some(Self::LastPlayed),
            "PlayCount" => Some(Self::PlayCount),
            "PlayTime" => Some(Self::PlayTime),
            "StarRating" => Some(Self::StarRating),
            "CommunityStarRating" => Some(Self::CommunityStarRating),
            "Developer" => Some(Self::Developer),
            "Publisher" => Some(Self::Publisher),
            "Genre" => Some(Self::Genre),
            "Series" => Some(Self::Series),
            "Status" => Some(Self::Status),
            "Favorite" => Some(Self::Favorite),
            _ => None,
        }
    }

    pub const fn key(self) -> &'static str {
        match self {
            Self::Title => "Title",
            Self::SortTitle => "SortTitle",
            Self::Platform => "Platform",
            Self::ReleaseDate => "ReleaseDate",
            Self::DateAdded => "DateAdded",
            Self::DateModified => "DateModified",
            Self::LastPlayed => "LastPlayed",
            Self::PlayCount => "PlayCount",
            Self::PlayTime => "PlayTime",
            Self::StarRating => "StarRating",
            Self::CommunityStarRating => "CommunityStarRating",
            Self::Developer => "Developer",
            Self::Publisher => "Publisher",
            Self::Genre => "Genre",
            Self::Series => "Series",
            Self::Status => "Status",
            Self::Favorite => "Favorite",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum GameStateFilter {
    #[default]
    Any,
    Favorite,
    NotFavorite,
    Completed,
    NotCompleted,
    Installed,
    NotInstalled,
    InstallationUnknown,
    Played,
    NeverPlayed,
    Rated,
    Unrated,
    Hidden,
    Broken,
}

impl GameStateFilter {
    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "" | "any" => Some(Self::Any),
            "favorite" => Some(Self::Favorite),
            "not-favorite" => Some(Self::NotFavorite),
            "completed" => Some(Self::Completed),
            "not-completed" => Some(Self::NotCompleted),
            "installed" => Some(Self::Installed),
            "not-installed" => Some(Self::NotInstalled),
            "installation-unknown" => Some(Self::InstallationUnknown),
            "played" => Some(Self::Played),
            "never-played" => Some(Self::NeverPlayed),
            "rated" => Some(Self::Rated),
            "unrated" => Some(Self::Unrated),
            "hidden" => Some(Self::Hidden),
            "broken" => Some(Self::Broken),
            _ => None,
        }
    }

    pub const fn key(self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::Favorite => "favorite",
            Self::NotFavorite => "not-favorite",
            Self::Completed => "completed",
            Self::NotCompleted => "not-completed",
            Self::Installed => "installed",
            Self::NotInstalled => "not-installed",
            Self::InstallationUnknown => "installation-unknown",
            Self::Played => "played",
            Self::NeverPlayed => "never-played",
            Self::Rated => "rated",
            Self::Unrated => "unrated",
            Self::Hidden => "hidden",
            Self::Broken => "broken",
        }
    }

    fn matches(self, game: &Game) -> bool {
        match self {
            Self::Any => true,
            Self::Favorite => game.favorite,
            Self::NotFavorite => !game.favorite,
            Self::Completed => game.completed,
            Self::NotCompleted => !game.completed,
            Self::Installed => game.installed == Some(true),
            Self::NotInstalled => game.installed == Some(false),
            Self::InstallationUnknown => game.installed.is_none(),
            Self::Played => game.play_count > 0 || game.last_played_date.is_some(),
            Self::NeverPlayed => game.play_count == 0 && game.last_played_date.is_none(),
            Self::Rated => game.star_rating > 0 || game.star_rating_float > 0.0,
            Self::Unrated => game.star_rating == 0 && game.star_rating_float <= 0.0,
            Self::Hidden => game.hidden,
            Self::Broken => game.broken,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MissingMediaFilter {
    #[default]
    None,
    Any,
    BackgroundImage,
    BannerImage,
    Box3dImage,
    BoxFrontImage,
    Cart3dImage,
    CartImage,
    ClearLogoImage,
    Manual,
    MarqueeImage,
    Music,
    ScreenshotImage,
    Video,
}

impl MissingMediaFilter {
    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "" | "none" => Some(Self::None),
            "any" => Some(Self::Any),
            "background-image" => Some(Self::BackgroundImage),
            "banner-image" => Some(Self::BannerImage),
            "box-3d-image" => Some(Self::Box3dImage),
            "box-front-image" => Some(Self::BoxFrontImage),
            "cart-3d-image" => Some(Self::Cart3dImage),
            "cart-image" => Some(Self::CartImage),
            "clear-logo-image" => Some(Self::ClearLogoImage),
            "manual" => Some(Self::Manual),
            "marquee-image" => Some(Self::MarqueeImage),
            "music" => Some(Self::Music),
            "screenshot-image" => Some(Self::ScreenshotImage),
            "video" => Some(Self::Video),
            _ => None,
        }
    }

    pub const fn key(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Any => "any",
            Self::BackgroundImage => "background-image",
            Self::BannerImage => "banner-image",
            Self::Box3dImage => "box-3d-image",
            Self::BoxFrontImage => "box-front-image",
            Self::Cart3dImage => "cart-3d-image",
            Self::CartImage => "cart-image",
            Self::ClearLogoImage => "clear-logo-image",
            Self::Manual => "manual",
            Self::MarqueeImage => "marquee-image",
            Self::Music => "music",
            Self::ScreenshotImage => "screenshot-image",
            Self::Video => "video",
        }
    }

    fn matches(self, game: &Game) -> bool {
        match self {
            Self::None => true,
            Self::Any => {
                game.missing_background_image
                    || game.missing_banner_image
                    || game.missing_box_3d_image
                    || game.missing_box_front_image
                    || game.missing_cart_3d_image
                    || game.missing_cart_image
                    || game.missing_clear_logo_image
                    || game.missing_manual
                    || game.missing_marquee_image
                    || game.missing_music
                    || game.missing_screenshot_image
                    || game.missing_video
            }
            Self::BackgroundImage => game.missing_background_image,
            Self::BannerImage => game.missing_banner_image,
            Self::Box3dImage => game.missing_box_3d_image,
            Self::BoxFrontImage => game.missing_box_front_image,
            Self::Cart3dImage => game.missing_cart_3d_image,
            Self::CartImage => game.missing_cart_image,
            Self::ClearLogoImage => game.missing_clear_logo_image,
            Self::Manual => game.missing_manual,
            Self::MarqueeImage => game.missing_marquee_image,
            Self::Music => game.missing_music,
            Self::ScreenshotImage => game.missing_screenshot_image,
            Self::Video => game.missing_video,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GameFilter {
    pub text: String,
    pub platform: Option<String>,
    pub include_hidden: bool,
    pub include_broken: bool,
    pub state: GameStateFilter,
    pub missing_media: MissingMediaFilter,
    pub sort: GameSort,
    pub sort_descending: bool,
}

pub fn filter_games<'a>(games: &'a [Game], filter: &GameFilter) -> Vec<&'a Game> {
    filter_game_indices(games, filter)
        .into_iter()
        .map(|index| &games[index])
        .collect()
}

pub fn game_matches_filter(game: &Game, filter: &GameFilter) -> bool {
    let needle = filter.text.trim().to_lowercase();
    let include_hidden = filter.include_hidden || filter.state == GameStateFilter::Hidden;
    let include_broken = filter.include_broken || filter.state == GameStateFilter::Broken;

    (include_hidden || !game.hidden)
        && (include_broken || !game.broken)
        && filter.state.matches(game)
        && filter.missing_media.matches(game)
        && filter
            .platform
            .as_deref()
            .is_none_or(|expected| game.platform == expected)
        && (needle.is_empty() || searchable_metadata(game, &needle))
}

/// Returns stable indices into the caller-owned game slice. This is the form
/// used by the Qt model so filtering does not clone records or serialize a
/// whole-library JSON snapshot.
pub fn filter_game_indices(games: &[Game], filter: &GameFilter) -> Vec<usize> {
    let mut matches: Vec<_> = games
        .iter()
        .enumerate()
        .filter(|(_, game)| game_matches_filter(game, filter))
        .map(|(index, game)| {
            (
                index,
                game_sort_value(game, filter.sort),
                normalized_text(game.display_sort_title()),
            )
        })
        .collect();

    matches.sort_by(|left, right| {
        compare_sort_values(&left.1, &right.1, filter.sort_descending)
            .then_with(|| left.2.cmp(&right.2))
            .then_with(|| games[left.0].id.cmp(&games[right.0].id))
    });
    matches.into_iter().map(|(index, _, _)| index).collect()
}

/// Uses the same ordering contract as [`filter_game_indices`] for incremental
/// model insertion. Missing primary values remain last in either direction,
/// and title/ID tie-breaks always remain ascending and deterministic.
pub fn compare_games(left: &Game, right: &Game, filter: &GameFilter) -> Ordering {
    compare_sort_values(
        &game_sort_value(left, filter.sort),
        &game_sort_value(right, filter.sort),
        filter.sort_descending,
    )
    .then_with(|| {
        normalized_text(left.display_sort_title()).cmp(&normalized_text(right.display_sort_title()))
    })
    .then_with(|| left.id.cmp(&right.id))
}

/// Returns true when an in-place record update can add/remove the game or move
/// it within the active query result.
pub fn game_query_result_may_change(previous: &Game, next: &Game, filter: &GameFilter) -> bool {
    if game_matches_filter(previous, filter) != game_matches_filter(next, filter) {
        return true;
    }
    game_sort_value(previous, filter.sort) != game_sort_value(next, filter.sort)
        || normalized_text(previous.display_sort_title())
            != normalized_text(next.display_sort_title())
        || previous.id != next.id
}

/// Selects a visible model row using caller-provided entropy. Injecting entropy
/// keeps this pure and fully testable on every supported host. When more than
/// one valid row exists, the requested current game is excluded.
pub fn select_random_filtered_row(
    games: &[Game],
    filtered_indices: &[usize],
    avoid_game_id: Option<&str>,
    entropy: u64,
) -> Option<usize> {
    let valid_rows = filtered_indices
        .iter()
        .enumerate()
        .filter_map(|(row, index)| games.get(*index).map(|game| (row, game)))
        .collect::<Vec<_>>();
    if valid_rows.is_empty() {
        return None;
    }
    let candidates = if valid_rows.len() > 1 {
        let without_current = valid_rows
            .iter()
            .copied()
            .filter(|(_, game)| Some(game.id.as_str()) != avoid_game_id)
            .collect::<Vec<_>>();
        if without_current.is_empty() {
            valid_rows
        } else {
            without_current
        }
    } else {
        valid_rows
    };
    let selected = usize::try_from(entropy % candidates.len() as u64).unwrap_or_default();
    Some(candidates[selected].0)
}

#[derive(Clone, Debug, PartialEq)]
enum GameSortValue {
    Missing,
    Text(String),
    Signed(i64),
    Unsigned(u64),
    Float(f64),
    Boolean(bool),
}

fn game_sort_value(game: &Game, sort: GameSort) -> GameSortValue {
    match sort {
        GameSort::Title => text_value(Some(game.display_sort_title())),
        GameSort::SortTitle => text_value(game.sort_title.as_deref()),
        GameSort::Platform => text_value(Some(&game.platform)),
        GameSort::ReleaseDate => date_value(game.release_date.as_deref()),
        GameSort::DateAdded => date_value(Some(&game.date_added)),
        GameSort::DateModified => date_value(Some(&game.date_modified)),
        GameSort::LastPlayed => date_value(game.last_played_date.as_deref()),
        GameSort::PlayCount => GameSortValue::Unsigned(u64::from(game.play_count)),
        GameSort::PlayTime => GameSortValue::Unsigned(game.play_time_seconds),
        GameSort::StarRating => {
            let value = if game.star_rating_float > 0.0 {
                game.star_rating_float
            } else {
                f64::from(game.star_rating)
            };
            GameSortValue::Float(value)
        }
        GameSort::CommunityStarRating => GameSortValue::Float(game.community_star_rating),
        GameSort::Developer => text_value(game.developer.as_deref()),
        GameSort::Publisher => text_value(game.publisher.as_deref()),
        GameSort::Genre => text_value(game.genre.as_deref()),
        GameSort::Series => text_value(game.series.as_deref()),
        GameSort::Status => text_value(game.status.as_deref()),
        GameSort::Favorite => GameSortValue::Boolean(game.favorite),
    }
}

fn text_value(value: Option<&str>) -> GameSortValue {
    value
        .filter(|value| !value.trim().is_empty())
        .map(normalized_text)
        .map(GameSortValue::Text)
        .unwrap_or(GameSortValue::Missing)
}

fn date_value(value: Option<&str>) -> GameSortValue {
    value
        .filter(|value| !value.trim().is_empty())
        .and_then(parse_timestamp)
        .map(GameSortValue::Signed)
        .unwrap_or(GameSortValue::Missing)
}

fn parse_timestamp(value: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(value)
        .map(|value| value.timestamp())
        .ok()
        .or_else(|| {
            NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f")
                .map(|value| value.and_utc().timestamp())
                .ok()
        })
        .or_else(|| {
            NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .ok()
                .and_then(|value| value.and_hms_opt(0, 0, 0))
                .map(|value| value.and_utc().timestamp())
        })
}

fn normalized_text(value: &str) -> String {
    value.trim().to_lowercase()
}

fn compare_sort_values(left: &GameSortValue, right: &GameSortValue, descending: bool) -> Ordering {
    match (left, right) {
        (GameSortValue::Missing, GameSortValue::Missing) => Ordering::Equal,
        (GameSortValue::Missing, _) => Ordering::Greater,
        (_, GameSortValue::Missing) => Ordering::Less,
        _ => {
            let ordering = match (left, right) {
                (GameSortValue::Text(left), GameSortValue::Text(right)) => left.cmp(right),
                (GameSortValue::Signed(left), GameSortValue::Signed(right)) => left.cmp(right),
                (GameSortValue::Unsigned(left), GameSortValue::Unsigned(right)) => left.cmp(right),
                (GameSortValue::Float(left), GameSortValue::Float(right)) => left.total_cmp(right),
                (GameSortValue::Boolean(left), GameSortValue::Boolean(right)) => left.cmp(right),
                _ => Ordering::Equal,
            };
            if descending {
                ordering.reverse()
            } else {
                ordering
            }
        }
    }
}

fn searchable_metadata(game: &Game, needle: &str) -> bool {
    [
        Some(game.title.as_str()),
        game.sort_title.as_deref(),
        game.notes.as_deref(),
        game.developer.as_deref(),
        game.genre.as_deref(),
        game.play_mode.as_deref(),
        game.progress.as_deref(),
        game.publisher.as_deref(),
        game.rating.as_deref(),
        game.region.as_deref(),
        game.release_date.as_deref(),
        game.release_type.as_deref(),
        game.series.as_deref(),
        game.source.as_deref(),
        game.status.as_deref(),
        game.version.as_deref(),
        game.wikipedia_url.as_deref(),
    ]
    .into_iter()
    .flatten()
    .any(|value| value.to_lowercase().contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game(id: &str, title: &str, platform: &str) -> Game {
        Game {
            id: id.into(),
            title: title.into(),
            platform: platform.into(),
            application_path: format!("Games/{id}.rom"),
            ..Game::default()
        }
    }

    #[test]
    fn filters_case_insensitively_and_sorts() {
        let games = vec![
            game("2", "Zebra Racer", "Arcade"),
            game("1", "Adventure", "Console"),
            game("3", "Another Adventure", "Arcade"),
        ];
        let matches = filter_games(
            &games,
            &GameFilter {
                text: "ADVENTURE".into(),
                ..GameFilter::default()
            },
        );
        assert_eq!(
            matches
                .iter()
                .map(|game| game.id.as_str())
                .collect::<Vec<_>>(),
            ["1", "3"]
        );
        assert_eq!(
            filter_game_indices(
                &games,
                &GameFilter {
                    text: "ADVENTURE".into(),
                    ..GameFilter::default()
                },
            ),
            [1, 2]
        );
    }

    #[test]
    fn excludes_hidden_and_broken_by_default() {
        let mut hidden = game("hidden", "Hidden", "Console");
        hidden.hidden = true;
        let mut broken = game("broken", "Broken", "Console");
        broken.broken = true;
        let visible = game("visible", "Visible", "Console");
        let games = vec![hidden, broken, visible];
        let matches = filter_games(&games, &GameFilter::default());
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, "visible");
    }

    #[test]
    fn searches_across_descriptive_metadata() {
        let mut metadata = game("metadata", "Ordinary Title", "Console");
        metadata.developer = Some("Moon Studio".into());
        metadata.genre = Some("Action Puzzle".into());
        metadata.publisher = Some("Sun Press".into());
        metadata.series = Some("Constellation Saga".into());
        metadata.region = Some("Oceania".into());
        metadata.version = Some("Director's Cut".into());
        let games = [metadata];

        for needle in [
            "moon",
            "puzzle",
            "sun press",
            "constellation",
            "oceania",
            "director",
        ] {
            assert_eq!(
                filter_game_indices(
                    &games,
                    &GameFilter {
                        text: needle.into(),
                        ..GameFilter::default()
                    }
                ),
                [0],
                "metadata search missed {needle}"
            );
        }
    }

    #[test]
    fn filters_every_persisted_library_state_without_losing_visibility_safety() {
        let mut favorite = game("favorite", "Favorite", "Console");
        favorite.favorite = true;
        favorite.installed = Some(true);
        favorite.play_count = 1;
        favorite.star_rating = 4;

        let mut hidden = game("hidden", "Hidden", "Console");
        hidden.hidden = true;

        let mut broken = game("broken", "Broken", "Console");
        broken.broken = true;
        broken.completed = true;
        broken.installed = Some(false);

        let unknown = game("unknown", "Unknown", "Console");
        let games = [favorite, hidden, broken, unknown];

        let ids_for = |state| {
            filter_games(
                &games,
                &GameFilter {
                    state,
                    ..GameFilter::default()
                },
            )
            .into_iter()
            .map(|game| game.id.as_str())
            .collect::<Vec<_>>()
        };

        assert_eq!(ids_for(GameStateFilter::Favorite), ["favorite"]);
        assert_eq!(ids_for(GameStateFilter::Completed), Vec::<&str>::new());
        assert_eq!(ids_for(GameStateFilter::Installed), ["favorite"]);
        assert_eq!(ids_for(GameStateFilter::NotInstalled), Vec::<&str>::new());
        assert_eq!(ids_for(GameStateFilter::InstallationUnknown), ["unknown"]);
        assert_eq!(ids_for(GameStateFilter::Played), ["favorite"]);
        assert_eq!(ids_for(GameStateFilter::NeverPlayed), ["unknown"]);
        assert_eq!(ids_for(GameStateFilter::Rated), ["favorite"]);
        assert_eq!(ids_for(GameStateFilter::Unrated), ["unknown"]);
        assert_eq!(ids_for(GameStateFilter::Hidden), ["hidden"]);
        assert_eq!(ids_for(GameStateFilter::Broken), ["broken"]);

        assert_eq!(
            filter_games(
                &games,
                &GameFilter {
                    state: GameStateFilter::Completed,
                    include_broken: true,
                    ..GameFilter::default()
                },
            )[0]
            .id,
            "broken"
        );
    }

    #[test]
    fn filters_all_twelve_persisted_missing_media_families() {
        let mut missing = game("missing", "Missing", "Console");
        missing.missing_background_image = true;
        missing.missing_banner_image = true;
        missing.missing_box_3d_image = true;
        missing.missing_box_front_image = true;
        missing.missing_cart_3d_image = true;
        missing.missing_cart_image = true;
        missing.missing_clear_logo_image = true;
        missing.missing_manual = true;
        missing.missing_marquee_image = true;
        missing.missing_music = true;
        missing.missing_screenshot_image = true;
        missing.missing_video = true;
        let complete = game("complete", "Complete", "Console");
        let games = [missing, complete];

        for missing_media in [
            MissingMediaFilter::Any,
            MissingMediaFilter::BackgroundImage,
            MissingMediaFilter::BannerImage,
            MissingMediaFilter::Box3dImage,
            MissingMediaFilter::BoxFrontImage,
            MissingMediaFilter::Cart3dImage,
            MissingMediaFilter::CartImage,
            MissingMediaFilter::ClearLogoImage,
            MissingMediaFilter::Manual,
            MissingMediaFilter::MarqueeImage,
            MissingMediaFilter::Music,
            MissingMediaFilter::ScreenshotImage,
            MissingMediaFilter::Video,
        ] {
            let matches = filter_games(
                &games,
                &GameFilter {
                    missing_media,
                    ..GameFilter::default()
                },
            );
            assert_eq!(matches.len(), 1, "filter {}", missing_media.key());
            assert_eq!(matches[0].id, "missing", "filter {}", missing_media.key());
        }
    }

    #[test]
    fn stable_filter_keys_round_trip_and_reject_unknown_values() {
        for state in [
            GameStateFilter::Any,
            GameStateFilter::Favorite,
            GameStateFilter::NotFavorite,
            GameStateFilter::Completed,
            GameStateFilter::NotCompleted,
            GameStateFilter::Installed,
            GameStateFilter::NotInstalled,
            GameStateFilter::InstallationUnknown,
            GameStateFilter::Played,
            GameStateFilter::NeverPlayed,
            GameStateFilter::Rated,
            GameStateFilter::Unrated,
            GameStateFilter::Hidden,
            GameStateFilter::Broken,
        ] {
            assert_eq!(GameStateFilter::from_key(state.key()), Some(state));
        }
        for missing_media in [
            MissingMediaFilter::None,
            MissingMediaFilter::Any,
            MissingMediaFilter::BackgroundImage,
            MissingMediaFilter::BannerImage,
            MissingMediaFilter::Box3dImage,
            MissingMediaFilter::BoxFrontImage,
            MissingMediaFilter::Cart3dImage,
            MissingMediaFilter::CartImage,
            MissingMediaFilter::ClearLogoImage,
            MissingMediaFilter::Manual,
            MissingMediaFilter::MarqueeImage,
            MissingMediaFilter::Music,
            MissingMediaFilter::ScreenshotImage,
            MissingMediaFilter::Video,
        ] {
            assert_eq!(
                MissingMediaFilter::from_key(missing_media.key()),
                Some(missing_media)
            );
        }
        assert_eq!(GameStateFilter::from_key("favorites"), None);
        assert_eq!(MissingMediaFilter::from_key("box"), None);
    }

    #[test]
    fn launchbox_sort_keys_round_trip_and_reject_unknown_values() {
        for sort in [
            GameSort::Title,
            GameSort::SortTitle,
            GameSort::Platform,
            GameSort::ReleaseDate,
            GameSort::DateAdded,
            GameSort::DateModified,
            GameSort::LastPlayed,
            GameSort::PlayCount,
            GameSort::PlayTime,
            GameSort::StarRating,
            GameSort::CommunityStarRating,
            GameSort::Developer,
            GameSort::Publisher,
            GameSort::Genre,
            GameSort::Series,
            GameSort::Status,
            GameSort::Favorite,
        ] {
            assert_eq!(GameSort::from_key(sort.key()), Some(sort));
        }
        assert_eq!(GameSort::from_key("Random"), None);
        assert_eq!(GameSort::from_key("title"), None);
    }

    #[test]
    fn sorts_every_typed_value_with_stable_title_and_id_ties() {
        let mut alpha = game("2", "Alpha", "Console");
        alpha.play_count = 3;
        alpha.play_time_seconds = 20;
        alpha.star_rating_float = 4.5;
        alpha.favorite = true;
        alpha.developer = Some("Zed".into());
        alpha.release_date = Some("2020-01-02".into());

        let mut beta = game("1", "Beta", "Arcade");
        beta.play_count = 1;
        beta.play_time_seconds = 40;
        beta.star_rating = 2;
        beta.developer = Some("Able".into());
        beta.release_date = Some("2019-01-02T00:00:00Z".into());

        let games = [alpha, beta];
        let ids = |sort, sort_descending| {
            filter_game_indices(
                &games,
                &GameFilter {
                    sort,
                    sort_descending,
                    ..GameFilter::default()
                },
            )
            .into_iter()
            .map(|index| games[index].id.as_str())
            .collect::<Vec<_>>()
        };

        assert_eq!(ids(GameSort::Platform, false), ["1", "2"]);
        assert_eq!(ids(GameSort::Developer, false), ["1", "2"]);
        assert_eq!(ids(GameSort::ReleaseDate, false), ["1", "2"]);
        assert_eq!(ids(GameSort::PlayCount, false), ["1", "2"]);
        assert_eq!(ids(GameSort::PlayTime, false), ["2", "1"]);
        assert_eq!(ids(GameSort::StarRating, false), ["1", "2"]);
        assert_eq!(ids(GameSort::Favorite, false), ["1", "2"]);
        assert_eq!(ids(GameSort::PlayCount, true), ["2", "1"]);
    }

    #[test]
    fn date_sort_parses_offsets_and_keeps_missing_or_invalid_values_last() {
        let mut later_in_local_spelling = game("later", "Later", "Console");
        later_in_local_spelling.release_date = Some("2024-01-01T01:00:00+01:00".into());
        let mut earlier_utc = game("earlier", "Earlier", "Console");
        earlier_utc.release_date = Some("2023-12-31T23:30:00Z".into());
        let mut missing = game("missing", "Missing", "Console");
        missing.release_date = None;
        let mut invalid = game("invalid", "Invalid", "Console");
        invalid.release_date = Some("not a date".into());
        let games = [missing, later_in_local_spelling, invalid, earlier_utc];

        for sort_descending in [false, true] {
            let indices = filter_game_indices(
                &games,
                &GameFilter {
                    sort: GameSort::ReleaseDate,
                    sort_descending,
                    ..GameFilter::default()
                },
            );
            let ids = indices
                .into_iter()
                .map(|index| games[index].id.as_str())
                .collect::<Vec<_>>();
            let expected = if sort_descending {
                vec!["later", "earlier", "invalid", "missing"]
            } else {
                vec!["earlier", "later", "invalid", "missing"]
            };
            assert_eq!(ids, expected);
        }
    }

    #[test]
    fn query_change_detection_covers_filter_membership_and_active_sort_values() {
        let previous = game("id", "Title", "Console");
        let mut next = previous.clone();
        next.play_count = 1;
        assert!(!game_query_result_may_change(
            &previous,
            &next,
            &GameFilter::default()
        ));
        assert!(game_query_result_may_change(
            &previous,
            &next,
            &GameFilter {
                sort: GameSort::PlayCount,
                ..GameFilter::default()
            }
        ));
        assert!(game_query_result_may_change(
            &previous,
            &next,
            &GameFilter {
                state: GameStateFilter::NeverPlayed,
                ..GameFilter::default()
            }
        ));
    }

    #[test]
    fn random_selection_is_bounded_deterministic_and_avoids_the_current_game() {
        let games = [
            game("a", "Alpha", "Console"),
            game("b", "Beta", "Console"),
            game("c", "Gamma", "Console"),
        ];
        let visible = [2, 0, 1];
        assert_eq!(
            select_random_filtered_row(&games, &visible, Some("c"), 0),
            Some(1)
        );
        assert_eq!(
            select_random_filtered_row(&games, &visible, Some("c"), 1),
            Some(2)
        );
        assert_eq!(
            select_random_filtered_row(&games, &visible[..1], Some("c"), 99),
            Some(0)
        );
        assert_eq!(select_random_filtered_row(&games, &[], Some("c"), 0), None);
        assert_eq!(
            select_random_filtered_row(&games, &[99, 1], Some("b"), 0),
            Some(1)
        );
    }
}
