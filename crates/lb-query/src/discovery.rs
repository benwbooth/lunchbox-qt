use super::parse_timestamp;
use chrono::{DateTime, Duration, Utc};
use lb_domain::Game;
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::BTreeMap;

pub const BIG_BOX_DISCOVERY_PAYLOAD_VERSION: u8 = 1;
pub const BIG_BOX_DISCOVERY_MAXIMUM_GAME_ITEMS: usize = 25;
pub const BIG_BOX_DISCOVERY_RECENTLY_ADDED_DAYS: i64 = 360;
pub const BIG_BOX_DISCOVERY_RECENTLY_ADDED_MINIMUM_ITEMS: usize = 5;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BigBoxDiscoveryPayload {
    pub version: u8,
    pub contract_source: String,
    pub sections: Vec<BigBoxDiscoverySection>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BigBoxDiscoverySection {
    pub key: String,
    pub title: String,
    pub list_type: String,
    pub source: String,
    pub available: bool,
    pub displayable: bool,
    pub minimum_items: usize,
    pub maximum_items: Option<usize>,
    pub items: Vec<BigBoxDiscoveryItem>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BigBoxDiscoveryItem {
    pub kind: String,
    pub game_id: Option<String>,
    pub platform_key: Option<String>,
    pub representative_game_id: Option<String>,
    pub title: String,
    pub subtitle: String,
    pub platform: String,
    pub rating: f64,
    pub rating_source: String,
    pub favorite: bool,
    pub game_count: usize,
}

/// Projects the six ordered content-list slots in LaunchBox 13.27's embedded
/// `DiscoveryPageView`.
///
/// The view contract and the Recently Added criteria are recovered directly.
/// The protected view-model's other ranking implementations are unavailable,
/// so this projection deliberately uses documented stable native policies.
/// MAME high scores retain their recovered slot but remain unavailable until
/// that separate adapter is implemented.
pub fn project_big_box_discovery(games: &[Game], now: DateTime<Utc>) -> BigBoxDiscoveryPayload {
    let visible = games
        .iter()
        .filter(|game| !game.hidden && !game.broken)
        .collect::<Vec<_>>();

    let mut highly_rated = visible
        .iter()
        .filter_map(|game| {
            let (rating, source) = effective_rating(game);
            (rating > 0.0).then_some((*game, rating, source))
        })
        .collect::<Vec<_>>();
    highly_rated.sort_by(|(left, left_rating, _), (right, right_rating, _)| {
        right_rating
            .total_cmp(left_rating)
            .then_with(|| {
                right
                    .community_star_rating_total_votes
                    .cmp(&left.community_star_rating_total_votes)
            })
            .then_with(|| compare_game_identity(left, right))
    });
    let highly_rated = highly_rated
        .into_iter()
        .take(BIG_BOX_DISCOVERY_MAXIMUM_GAME_ITEMS)
        .map(|(game, rating, source)| {
            game_item(
                game,
                format!("{}  •  ★ {rating:.1}", game.platform),
                rating,
                source,
            )
        })
        .collect::<Vec<_>>();

    let mut recently_played = visible
        .iter()
        .filter_map(|game| {
            game.last_played_date
                .as_deref()
                .and_then(parse_timestamp)
                .map(|timestamp| (*game, timestamp))
        })
        .collect::<Vec<_>>();
    recently_played.sort_by(|(left, left_timestamp), (right, right_timestamp)| {
        right_timestamp
            .cmp(left_timestamp)
            .then_with(|| compare_game_identity(left, right))
    });
    let recently_played = recently_played
        .into_iter()
        .take(BIG_BOX_DISCOVERY_MAXIMUM_GAME_ITEMS)
        .map(|(game, _)| {
            game_item(
                game,
                format!(
                    "{}  •  Last played {}",
                    game.platform,
                    game.last_played_date.as_deref().unwrap_or_default()
                ),
                effective_rating(game).0,
                effective_rating(game).1,
            )
        })
        .collect::<Vec<_>>();

    let recent_boundary = now
        .checked_sub_signed(Duration::days(BIG_BOX_DISCOVERY_RECENTLY_ADDED_DAYS))
        .map(|value| value.timestamp())
        .unwrap_or(i64::MIN);
    let now_timestamp = now.timestamp();
    let mut recently_added = visible
        .iter()
        .filter_map(|game| {
            parse_timestamp(&game.date_added)
                .filter(|timestamp| *timestamp >= recent_boundary && *timestamp <= now_timestamp)
                .map(|timestamp| (*game, timestamp))
        })
        .collect::<Vec<_>>();
    recently_added.sort_by(|(left, left_timestamp), (right, right_timestamp)| {
        right_timestamp
            .cmp(left_timestamp)
            .then_with(|| compare_game_identity(left, right))
    });
    let recently_added = recently_added
        .into_iter()
        .take(BIG_BOX_DISCOVERY_MAXIMUM_GAME_ITEMS)
        .map(|(game, _)| {
            game_item(
                game,
                format!("{}  •  Added {}", game.platform, game.date_added),
                effective_rating(game).0,
                effective_rating(game).1,
            )
        })
        .collect::<Vec<_>>();

    let mut games_by_platform = BTreeMap::<String, Vec<&Game>>::new();
    for game in &visible {
        games_by_platform
            .entry(game.platform.trim().to_lowercase())
            .or_default()
            .push(game);
    }
    let platforms = games_by_platform
        .into_iter()
        .filter(|(platform_key, games)| !platform_key.is_empty() && !games.is_empty())
        .map(|(_, mut games)| {
            games.sort_by(|left, right| compare_game_identity(left, right));
            let platform = games
                .iter()
                .map(|game| game.platform.trim())
                .min()
                .unwrap_or_default()
                .to_string();
            let game_count = games.len();
            BigBoxDiscoveryItem {
                kind: "platform".to_string(),
                game_id: None,
                platform_key: Some(platform.clone()),
                representative_game_id: games.first().map(|game| game.id.clone()),
                title: platform.clone(),
                subtitle: format!(
                    "{game_count} {}",
                    if game_count == 1 { "game" } else { "games" }
                ),
                platform,
                rating: 0.0,
                rating_source: "none".to_string(),
                favorite: false,
                game_count,
            }
        })
        .collect::<Vec<_>>();

    let mut favorites = visible
        .iter()
        .filter(|game| game.favorite)
        .copied()
        .collect::<Vec<_>>();
    favorites.sort_by(|left, right| compare_game_identity(left, right));
    let favorites = favorites
        .into_iter()
        .take(BIG_BOX_DISCOVERY_MAXIMUM_GAME_ITEMS)
        .map(|game| {
            game_item(
                game,
                format!("{}  •  Favorite", game.platform),
                effective_rating(game).0,
                effective_rating(game).1,
            )
        })
        .collect::<Vec<_>>();

    let sections = vec![
        game_section(
            "highlyRated",
            "Highly Rated",
            "HighlyRated",
            "recoveredViewModelPortRanking",
            1,
            highly_rated,
        ),
        game_section(
            "recentlyPlayed",
            "Recently Played",
            "RecentlyPlayed",
            "recoveredViewModelPortRanking",
            1,
            recently_played,
        ),
        game_section(
            "recentlyAdded",
            "Recently Added",
            "RecentlyAdded",
            "recoveredThemeContract",
            BIG_BOX_DISCOVERY_RECENTLY_ADDED_MINIMUM_ITEMS,
            recently_added,
        ),
        BigBoxDiscoverySection {
            key: "platforms".to_string(),
            title: "Platforms".to_string(),
            list_type: "Platforms".to_string(),
            source: "recoveredViewModelPortProjection".to_string(),
            available: true,
            displayable: !platforms.is_empty(),
            minimum_items: 1,
            maximum_items: None,
            items: platforms,
        },
        game_section(
            "favorites",
            "Favorites",
            "Favorites",
            "recoveredViewModelPortRanking",
            1,
            favorites,
        ),
        BigBoxDiscoverySection {
            key: "mameHighScores".to_string(),
            title: "MAME High Scores".to_string(),
            list_type: "MameHighScores".to_string(),
            source: "recoveredViewModelAdapterPending".to_string(),
            available: false,
            displayable: false,
            minimum_items: 1,
            maximum_items: Some(BIG_BOX_DISCOVERY_MAXIMUM_GAME_ITEMS),
            items: Vec::new(),
        },
    ];

    BigBoxDiscoveryPayload {
        version: BIG_BOX_DISCOVERY_PAYLOAD_VERSION,
        contract_source: "launchBox13.27EmbeddedDefaultView".to_string(),
        sections,
    }
}

fn game_section(
    key: &str,
    title: &str,
    list_type: &str,
    source: &str,
    minimum_items: usize,
    items: Vec<BigBoxDiscoveryItem>,
) -> BigBoxDiscoverySection {
    BigBoxDiscoverySection {
        key: key.to_string(),
        title: title.to_string(),
        list_type: list_type.to_string(),
        source: source.to_string(),
        available: true,
        displayable: items.len() >= minimum_items,
        minimum_items,
        maximum_items: Some(BIG_BOX_DISCOVERY_MAXIMUM_GAME_ITEMS),
        items,
    }
}

fn game_item(
    game: &Game,
    subtitle: String,
    rating: f64,
    rating_source: &'static str,
) -> BigBoxDiscoveryItem {
    BigBoxDiscoveryItem {
        kind: "game".to_string(),
        game_id: Some(game.id.clone()),
        platform_key: None,
        representative_game_id: Some(game.id.clone()),
        title: game.title.clone(),
        subtitle,
        platform: game.platform.clone(),
        rating,
        rating_source: rating_source.to_string(),
        favorite: game.favorite,
        game_count: 1,
    }
}

fn effective_rating(game: &Game) -> (f64, &'static str) {
    if game.community_star_rating.is_finite() && game.community_star_rating > 0.0 {
        (game.community_star_rating, "community")
    } else if game.star_rating_float.is_finite() && game.star_rating_float > 0.0 {
        (game.star_rating_float, "localFloat")
    } else if game.star_rating > 0 {
        (f64::from(game.star_rating), "localLegacy")
    } else {
        (0.0, "none")
    }
}

fn compare_game_identity(left: &Game, right: &Game) -> Ordering {
    left.display_sort_title()
        .trim()
        .to_lowercase()
        .cmp(&right.display_sort_title().trim().to_lowercase())
        .then_with(|| {
            left.title
                .trim()
                .to_lowercase()
                .cmp(&right.title.trim().to_lowercase())
        })
        .then_with(|| left.id.cmp(&right.id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn game(id: &str, title: &str, platform: &str) -> Game {
        Game {
            id: id.to_string(),
            title: title.to_string(),
            platform: platform.to_string(),
            application_path: format!("Games/{id}.rom"),
            date_added: "2026-01-01T00:00:00Z".to_string(),
            ..Game::default()
        }
    }

    #[test]
    fn freezes_recovered_section_order_and_recently_added_contract() {
        let games = (0..6)
            .map(|index| game(&format!("g{index}"), &format!("Game {index}"), "Console"))
            .collect::<Vec<_>>();
        let payload =
            project_big_box_discovery(&games, Utc.with_ymd_and_hms(2026, 7, 26, 0, 0, 0).unwrap());

        assert_eq!(payload.version, 1);
        assert_eq!(
            payload
                .sections
                .iter()
                .map(|section| section.key.as_str())
                .collect::<Vec<_>>(),
            [
                "highlyRated",
                "recentlyPlayed",
                "recentlyAdded",
                "platforms",
                "favorites",
                "mameHighScores",
            ]
        );
        let recently_added = &payload.sections[2];
        assert_eq!(
            recently_added.minimum_items,
            BIG_BOX_DISCOVERY_RECENTLY_ADDED_MINIMUM_ITEMS
        );
        assert_eq!(
            recently_added.maximum_items,
            Some(BIG_BOX_DISCOVERY_MAXIMUM_GAME_ITEMS)
        );
        assert!(recently_added.displayable);
        assert!(!payload.sections[5].available);
        assert!(!payload.sections[5].displayable);
    }

    #[test]
    fn ratings_use_community_then_float_then_legacy_with_stable_ties() {
        let mut community = game("community", "Community", "Console");
        community.community_star_rating = 4.5;
        community.community_star_rating_total_votes = 10;
        community.star_rating_float = 5.0;
        let mut float = game("float", "Float", "Console");
        float.star_rating_float = 4.5;
        let mut legacy = game("legacy", "Legacy", "Console");
        legacy.star_rating = 4;
        let payload = project_big_box_discovery(
            &[legacy, float, community],
            Utc.with_ymd_and_hms(2026, 7, 26, 0, 0, 0).unwrap(),
        );
        let items = &payload.sections[0].items;

        assert_eq!(
            items
                .iter()
                .map(|item| (item.game_id.as_deref(), item.rating_source.as_str()))
                .collect::<Vec<_>>(),
            [
                (Some("community"), "community"),
                (Some("float"), "localFloat"),
                (Some("legacy"), "localLegacy"),
            ]
        );
    }

    #[test]
    fn recently_added_obeys_inclusive_window_minimum_and_cap() {
        let now = Utc.with_ymd_and_hms(2026, 7, 26, 0, 0, 0).unwrap();
        let mut games = (0..30)
            .map(|index| {
                let mut game = game(&format!("g{index:02}"), &format!("Game {index:02}"), "A");
                game.date_added = (now - Duration::days(index)).to_rfc3339();
                game
            })
            .collect::<Vec<_>>();
        let mut old = game("old", "Old", "A");
        old.date_added = (now - Duration::days(361)).to_rfc3339();
        games.push(old);
        let payload = project_big_box_discovery(&games, now);
        let recent = &payload.sections[2];
        assert_eq!(recent.items.len(), 25);
        assert_eq!(recent.items[0].game_id.as_deref(), Some("g00"));
        assert_eq!(recent.items[24].game_id.as_deref(), Some("g24"));
        assert!(recent
            .items
            .iter()
            .all(|item| item.game_id.as_deref() != Some("old")));

        let below_minimum = project_big_box_discovery(&games[..4], now);
        assert!(!below_minimum.sections[2].displayable);
        assert_eq!(below_minimum.sections[2].items.len(), 4);
    }

    #[test]
    fn local_projections_and_visibility_are_stable_and_safe() {
        let mut hidden = game("hidden", "Hidden", "Arcade");
        hidden.hidden = true;
        let mut broken = game("broken", "Broken", "Arcade");
        broken.broken = true;
        let mut beta = game("beta", "Beta", "console");
        beta.favorite = true;
        beta.last_played_date = Some("2026-07-20T00:00:00Z".to_string());
        let mut alpha = game("alpha", "Alpha", "Console");
        alpha.favorite = true;
        alpha.last_played_date = Some("2026-07-25T00:00:00Z".to_string());
        let arcade = game("arcade", "Arcade", "Arcade");
        let payload = project_big_box_discovery(
            &[hidden, beta, broken, alpha, arcade],
            Utc.with_ymd_and_hms(2026, 7, 26, 0, 0, 0).unwrap(),
        );
        let platforms = &payload.sections[3].items;

        assert_eq!(
            platforms
                .iter()
                .map(|item| (
                    item.platform_key.as_deref(),
                    item.game_count,
                    item.representative_game_id.as_deref()
                ))
                .collect::<Vec<_>>(),
            [
                (Some("Arcade"), 1, Some("arcade")),
                (Some("Console"), 2, Some("alpha")),
            ]
        );
        assert_eq!(
            payload.sections[1]
                .items
                .iter()
                .map(|item| item.game_id.as_deref())
                .collect::<Vec<_>>(),
            [Some("alpha"), Some("beta")]
        );
        assert_eq!(
            payload.sections[4]
                .items
                .iter()
                .map(|item| item.game_id.as_deref())
                .collect::<Vec<_>>(),
            [Some("alpha"), Some("beta")]
        );
    }
}
