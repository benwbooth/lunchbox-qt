use lb_domain::Game;

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
        .map(|(index, game)| (index, game.display_sort_title().to_lowercase()))
        .collect();

    matches.sort_by(|left, right| {
        left.1
            .cmp(&right.1)
            .then_with(|| games[left.0].id.cmp(&games[right.0].id))
    });
    matches.into_iter().map(|(index, _)| index).collect()
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
}
