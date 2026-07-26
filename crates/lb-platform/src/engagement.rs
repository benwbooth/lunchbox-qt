use lb_domain::{FrontendSettings, Game};
use thiserror::Error;

const HALF_STAR_SCALE: f64 = 2.0;
const HALF_STAR_EPSILON: f64 = 1.0e-9;

/// Recovered LaunchBox 13.27 BigBox settings that govern favorites and ratings.
///
/// All six values are `true` in three independent fresh 13.27 installations.
/// Missing or malformed settings therefore fall back to the observed defaults.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BigBoxGameActionPolicy {
    pub show_star_next_to_favorited_games: bool,
    pub show_favorited_games_first: bool,
    pub show_game_favorite: bool,
    pub show_game_menu_favorite: bool,
    pub show_game_menu_star_rating: bool,
    pub show_game_star_rating: bool,
}

impl Default for BigBoxGameActionPolicy {
    fn default() -> Self {
        Self {
            show_star_next_to_favorited_games: true,
            show_favorited_games_first: true,
            show_game_favorite: true,
            show_game_menu_favorite: true,
            show_game_menu_star_rating: true,
            show_game_star_rating: true,
        }
    }
}

impl BigBoxGameActionPolicy {
    pub fn from_settings(settings: Option<&FrontendSettings>) -> Self {
        let fallback = Self::default();
        let Some(settings) = settings else {
            return fallback;
        };
        Self {
            show_star_next_to_favorited_games: settings
                .get_bool("ShowStarNextToFavoritedGames")
                .unwrap_or(fallback.show_star_next_to_favorited_games),
            show_favorited_games_first: settings
                .get_bool("ShowFavoritedGamesFirst")
                .unwrap_or(fallback.show_favorited_games_first),
            show_game_favorite: settings
                .get_bool("ShowGameFavorite")
                .unwrap_or(fallback.show_game_favorite),
            show_game_menu_favorite: settings
                .get_bool("ShowGameMenuFavorite")
                .unwrap_or(fallback.show_game_menu_favorite),
            show_game_menu_star_rating: settings
                .get_bool("ShowGameMenuStarRating")
                .unwrap_or(fallback.show_game_menu_star_rating),
            show_game_star_rating: settings
                .get_bool("ShowGameStarRating")
                .unwrap_or(fallback.show_game_star_rating),
        }
    }
}

#[derive(Clone, Copy, Debug, Error, PartialEq)]
pub enum BigBoxStarRatingError {
    #[error("star rating must be a finite value from 0 to 5")]
    OutOfRange,
    #[error("star rating must use half-star increments")]
    NotHalfStar,
}

/// Validates the half-star values stored by LaunchBox and returns the integer
/// companion value used in its legacy `<StarRating>` field.
pub fn normalize_big_box_star_rating(value: f64) -> Result<(u8, f64), BigBoxStarRatingError> {
    if !value.is_finite() || !(0.0..=5.0).contains(&value) {
        return Err(BigBoxStarRatingError::OutOfRange);
    }
    let scaled = value * HALF_STAR_SCALE;
    let rounded = scaled.round();
    if (scaled - rounded).abs() > HALF_STAR_EPSILON {
        return Err(BigBoxStarRatingError::NotHalfStar);
    }
    let normalized = rounded / HALF_STAR_SCALE;
    Ok((normalized.floor() as u8, normalized))
}

/// Stable-partitions an already sorted result set so BigBox can show favorites
/// first without changing the configured order within either group.
pub fn prioritize_favorite_game_indices(games: &[Game], indices: &mut Vec<usize>) {
    let mut favorites = Vec::with_capacity(indices.len());
    let mut remaining = Vec::with_capacity(indices.len());
    for index in indices.drain(..) {
        if games.get(index).is_some_and(|game| game.favorite) {
            favorites.push(index);
        } else {
            remaining.push(index);
        }
    }
    favorites.extend(remaining);
    *indices = favorites;
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

    fn game(id: &str, favorite: bool) -> Game {
        Game {
            id: id.into(),
            title: id.into(),
            favorite,
            ..Game::default()
        }
    }

    #[test]
    fn defaults_match_three_fresh_launchbox_13_27_installations() {
        let policy = BigBoxGameActionPolicy::from_settings(None);
        assert!(policy.show_star_next_to_favorited_games);
        assert!(policy.show_favorited_games_first);
        assert!(policy.show_game_favorite);
        assert!(policy.show_game_menu_favorite);
        assert!(policy.show_game_menu_star_rating);
        assert!(policy.show_game_star_rating);
    }

    #[test]
    fn reads_false_values_and_falls_back_for_malformed_values() {
        let policy = BigBoxGameActionPolicy::from_settings(Some(&settings(&[
            ("ShowStarNextToFavoritedGames", "false"),
            ("ShowFavoritedGamesFirst", "FALSE"),
            ("ShowGameFavorite", "sometimes"),
            ("ShowGameMenuFavorite", "false"),
            ("ShowGameMenuStarRating", "false"),
            ("ShowGameStarRating", "false"),
        ])));
        assert!(!policy.show_star_next_to_favorited_games);
        assert!(!policy.show_favorited_games_first);
        assert!(policy.show_game_favorite);
        assert!(!policy.show_game_menu_favorite);
        assert!(!policy.show_game_menu_star_rating);
        assert!(!policy.show_game_star_rating);
    }

    #[test]
    fn accepts_only_bounded_half_star_values_and_derives_integer_companion() {
        assert_eq!(normalize_big_box_star_rating(0.0), Ok((0, 0.0)));
        assert_eq!(normalize_big_box_star_rating(2.5), Ok((2, 2.5)));
        assert_eq!(normalize_big_box_star_rating(4.5), Ok((4, 4.5)));
        assert_eq!(normalize_big_box_star_rating(5.0), Ok((5, 5.0)));
        assert_eq!(
            normalize_big_box_star_rating(2.25),
            Err(BigBoxStarRatingError::NotHalfStar)
        );
        for value in [-0.5, 5.5, f64::NAN, f64::INFINITY] {
            assert_eq!(
                normalize_big_box_star_rating(value),
                Err(BigBoxStarRatingError::OutOfRange)
            );
        }
    }

    #[test]
    fn favorites_first_is_stable_inside_both_partitions() {
        let games = [
            game("alpha", false),
            game("bravo", true),
            game("charlie", false),
            game("delta", true),
        ];
        let mut indices = vec![2, 1, 0, 3];
        prioritize_favorite_game_indices(&games, &mut indices);
        assert_eq!(indices, [1, 3, 2, 0]);
    }
}
