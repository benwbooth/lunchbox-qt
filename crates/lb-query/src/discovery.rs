use super::{
    parse_timestamp, playlist_filter_is_supported, playlist_filters_match, GameFilter, GameSort,
};
use chrono::{DateTime, Duration, Utc};
use lb_domain::{
    DiscoveryCatalog, DiscoveryList, Game, PlaylistFilter, DISCOVERY_LISTS_ENDPOINT,
    DISCOVERY_LISTS_MAX_ITEMS_PER_LIST,
};
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

pub const BIG_BOX_DISCOVERY_PAYLOAD_VERSION: u8 = 2;
pub const BIG_BOX_DISCOVERY_MAXIMUM_GAME_ITEMS: usize = 25;
pub const BIG_BOX_DISCOVERY_RECENTLY_ADDED_DAYS: i64 = 360;
pub const BIG_BOX_DISCOVERY_RECENTLY_ADDED_MINIMUM_ITEMS: usize = 5;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BigBoxDiscoveryPayload {
    pub version: u8,
    pub contract_source: String,
    pub provider: BigBoxDiscoveryProviderStatus,
    pub sections: Vec<BigBoxDiscoverySection>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BigBoxDiscoveryProviderStatus {
    pub state: String,
    pub endpoint: String,
    pub fetched_lists: usize,
    pub rendered_lists: usize,
    pub rejected_lists: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BigBoxDiscoveryProviderState {
    NotLoaded,
    Loading,
    Ready,
    Unavailable,
}

impl BigBoxDiscoveryProviderState {
    const fn key(self) -> &'static str {
        match self {
            Self::NotLoaded => "notLoaded",
            Self::Loading => "loading",
            Self::Ready => "ready",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BigBoxDiscoverySection {
    pub key: String,
    pub title: String,
    pub subtitle: String,
    pub list_type: String,
    pub source: String,
    pub provider_id: Option<i32>,
    pub priority_rank: Option<i32>,
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
    project_big_box_discovery_with_provider(
        games,
        now,
        None,
        BigBoxDiscoveryProviderState::NotLoaded,
        0,
    )
}

pub fn project_big_box_discovery_with_provider(
    games: &[Game],
    now: DateTime<Utc>,
    provider_catalog: Option<&DiscoveryCatalog>,
    provider_state: BigBoxDiscoveryProviderState,
    entropy: u64,
) -> BigBoxDiscoveryPayload {
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

    let mut sections = vec![
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
            subtitle: String::new(),
            list_type: "Platforms".to_string(),
            source: "recoveredViewModelPortProjection".to_string(),
            provider_id: None,
            priority_rank: None,
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
            subtitle: String::new(),
            list_type: "MameHighScores".to_string(),
            source: "recoveredViewModelAdapterPending".to_string(),
            provider_id: None,
            priority_rank: None,
            available: false,
            displayable: false,
            minimum_items: 1,
            maximum_items: Some(BIG_BOX_DISCOVERY_MAXIMUM_GAME_ITEMS),
            items: Vec::new(),
        },
    ];

    let fetched_lists = provider_catalog.map_or(0, |catalog| catalog.lists.len());
    let mut rejected_lists = 0;
    if let Some(catalog) = provider_catalog {
        let mut prioritized = catalog
            .lists
            .iter()
            .filter(|list| list.priority_rank.is_some())
            .collect::<Vec<_>>();
        prioritized.sort_by(|left, right| {
            left.priority_rank
                .unwrap_or(i32::MAX)
                .cmp(&right.priority_rank.unwrap_or(i32::MAX))
                .then_with(|| {
                    randomized_list_key(left.id, entropy)
                        .cmp(&randomized_list_key(right.id, entropy))
                })
                .then_with(|| left.id.cmp(&right.id))
        });
        let mut random = catalog
            .lists
            .iter()
            .filter(|list| list.priority_rank.is_none())
            .collect::<Vec<_>>();
        random.sort_by(|left, right| {
            randomized_list_key(left.id, entropy)
                .cmp(&randomized_list_key(right.id, entropy))
                .then_with(|| left.id.cmp(&right.id))
        });
        for list in prioritized.into_iter().chain(random) {
            match provider_section(list, &visible, now) {
                Some(section) => sections.push(section),
                None => rejected_lists += 1,
            }
        }
    }
    let rendered_lists = sections
        .iter()
        .skip(6)
        .filter(|section| section.displayable)
        .count();
    BigBoxDiscoveryPayload {
        version: BIG_BOX_DISCOVERY_PAYLOAD_VERSION,
        contract_source: "launchBox13.27EmbeddedDefaultView".to_string(),
        provider: BigBoxDiscoveryProviderStatus {
            state: provider_state.key().to_string(),
            endpoint: DISCOVERY_LISTS_ENDPOINT.to_string(),
            fetched_lists,
            rendered_lists,
            rejected_lists,
        },
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
        subtitle: String::new(),
        list_type: list_type.to_string(),
        source: source.to_string(),
        provider_id: None,
        priority_rank: None,
        available: true,
        displayable: items.len() >= minimum_items,
        minimum_items,
        maximum_items: Some(BIG_BOX_DISCOVERY_MAXIMUM_GAME_ITEMS),
        items,
    }
}

fn provider_section(
    list: &DiscoveryList,
    visible_games: &[&Game],
    now: DateTime<Utc>,
) -> Option<BigBoxDiscoverySection> {
    let minimum_items = list.minimum_items.unwrap_or(1);
    let maximum_items = list
        .maximum_items
        .unwrap_or(BIG_BOX_DISCOVERY_MAXIMUM_GAME_ITEMS)
        .min(DISCOVERY_LISTS_MAX_ITEMS_PER_LIST);
    if minimum_items > maximum_items {
        return None;
    }
    let sort = match list
        .sort_by
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        Some(key) => GameSort::from_key(key)?,
        None => GameSort::Title,
    };
    let mut matches = if let Some(criteria) = list
        .criteria
        .as_ref()
        .filter(|criteria| !criteria.is_empty())
    {
        let filters = criteria
            .iter()
            .map(|criterion| PlaylistFilter {
                field_key: criterion.field.clone(),
                comparison_type_key: criterion.comparison.clone(),
                value: criterion.value.clone(),
            })
            .collect::<Vec<_>>();
        if filters
            .iter()
            .any(|filter| !playlist_filter_is_supported(filter))
        {
            return None;
        }
        visible_games
            .iter()
            .copied()
            .filter(|game| playlist_filters_match(game, &filters, now))
            .collect::<Vec<_>>()
    } else {
        let requested = list.games.as_ref()?;
        let mut matched_ids = BTreeSet::new();
        let mut matches = Vec::new();
        for item in requested {
            let matched = visible_games
                .iter()
                .copied()
                .find(|game| {
                    u32::try_from(item.database_id)
                        .ok()
                        .is_some_and(|database_id| game.database_id == Some(database_id))
                })
                .or_else(|| {
                    visible_games.iter().copied().find(|game| {
                        game.title.eq_ignore_ascii_case(item.title.trim())
                            && game.platform.eq_ignore_ascii_case(item.platform.trim())
                    })
                });
            if let Some(game) = matched.filter(|game| matched_ids.insert(game.id.clone())) {
                matches.push(game);
            }
        }
        matches
    };
    let filter = GameFilter {
        sort,
        sort_descending: list.sort_ascending.is_some_and(|ascending| !ascending),
        ..GameFilter::default()
    };
    matches.sort_by(|left, right| super::compare_games(left, right, &filter));
    matches.truncate(maximum_items);
    let items = matches
        .into_iter()
        .map(|game| {
            game_item(
                game,
                format!("{}  •  {}", game.platform, list.title),
                effective_rating(game).0,
                effective_rating(game).1,
            )
        })
        .collect::<Vec<_>>();
    Some(BigBoxDiscoverySection {
        key: format!("provider:{}", list.id),
        title: list.title.clone(),
        subtitle: list.subtitle.clone().unwrap_or_default(),
        list_type: list
            .list_type
            .clone()
            .unwrap_or_else(|| "Games".to_string()),
        source: "launchBox13.27PlaylistProvider".to_string(),
        provider_id: Some(list.id),
        priority_rank: list.priority_rank,
        available: true,
        displayable: items.len() >= minimum_items,
        minimum_items,
        maximum_items: Some(maximum_items),
        items,
    })
}

fn randomized_list_key(id: i32, entropy: u64) -> u64 {
    let mut value = entropy ^ u64::from(id as u32).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
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

        assert_eq!(payload.version, 2);
        assert_eq!(payload.provider.state, "notLoaded");
        assert_eq!(payload.provider.fetched_lists, 0);
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

    #[test]
    fn provider_lists_follow_priority_then_random_contract_and_resolve_exact_games() {
        use lb_domain::{DiscoveryCatalog, DiscoveryCriterion, DiscoveryGame, DiscoveryList};

        let mut arcade = game("arcade", "Robotron", "Arcade");
        arcade.database_id = Some(42);
        arcade.star_rating_float = 4.5;
        let mut fallback = game("fallback", "Fallback", "Console");
        fallback.star_rating = 4;
        let catalog = DiscoveryCatalog {
            lists: vec![
                DiscoveryList {
                    id: 30,
                    title: "Random Automatic".into(),
                    subtitle: Some("Automatic provider criteria".into()),
                    list_type: Some("Games".into()),
                    sort_by: Some("StarRating".into()),
                    sort_ascending: Some(false),
                    priority_rank: None,
                    minimum_items: Some(1),
                    maximum_items: Some(25),
                    games: Some(Vec::new()),
                    criteria: Some(vec![DiscoveryCriterion {
                        field: "StarRating".into(),
                        comparison: "GreaterThan".into(),
                        value: "3".into(),
                    }]),
                },
                DiscoveryList {
                    id: 20,
                    title: "Second Priority".into(),
                    subtitle: None,
                    list_type: None,
                    sort_by: Some("Title".into()),
                    sort_ascending: Some(true),
                    priority_rank: Some(2),
                    minimum_items: Some(1),
                    maximum_items: Some(1),
                    games: Some(vec![DiscoveryGame {
                        database_id: 999,
                        platform: "Console".into(),
                        title: "Fallback".into(),
                    }]),
                    criteria: Some(Vec::new()),
                },
                DiscoveryList {
                    id: 10,
                    title: "First Priority".into(),
                    subtitle: None,
                    list_type: Some("Games".into()),
                    sort_by: Some("Title".into()),
                    sort_ascending: Some(true),
                    priority_rank: Some(1),
                    minimum_items: Some(1),
                    maximum_items: Some(25),
                    games: Some(vec![DiscoveryGame {
                        database_id: 42,
                        platform: "Wrong platform is ignored when the ID matches".into(),
                        title: "Wrong title is ignored when the ID matches".into(),
                    }]),
                    criteria: Some(Vec::new()),
                },
            ],
        };
        let payload = project_big_box_discovery_with_provider(
            &[fallback, arcade],
            Utc.with_ymd_and_hms(2026, 7, 26, 0, 0, 0).unwrap(),
            Some(&catalog),
            BigBoxDiscoveryProviderState::Ready,
            17,
        );

        assert_eq!(payload.provider.state, "ready");
        assert_eq!(payload.provider.fetched_lists, 3);
        assert_eq!(payload.provider.rendered_lists, 3);
        assert_eq!(payload.provider.rejected_lists, 0);
        assert_eq!(
            payload
                .sections
                .iter()
                .skip(6)
                .map(|section| section.key.as_str())
                .collect::<Vec<_>>(),
            ["provider:10", "provider:20", "provider:30"]
        );
        assert_eq!(
            payload.sections[6].items[0].game_id.as_deref(),
            Some("arcade")
        );
        assert_eq!(
            payload.sections[7].items[0].game_id.as_deref(),
            Some("fallback")
        );
        assert_eq!(
            payload.sections[8]
                .items
                .iter()
                .map(|item| item.game_id.as_deref())
                .collect::<Vec<_>>(),
            [Some("arcade"), Some("fallback")]
        );
        assert_eq!(payload.sections[8].subtitle, "Automatic provider criteria");
    }

    #[test]
    fn provider_lists_reject_unsupported_semantics_and_enforce_bounds() {
        use lb_domain::{DiscoveryCatalog, DiscoveryCriterion, DiscoveryList};

        let mut rated = game("rated", "Rated", "Console");
        rated.star_rating = 5;
        let catalog = DiscoveryCatalog {
            lists: vec![
                DiscoveryList {
                    id: 1,
                    title: "Unsupported".into(),
                    subtitle: None,
                    list_type: None,
                    sort_by: None,
                    sort_ascending: None,
                    priority_rank: Some(1),
                    minimum_items: Some(1),
                    maximum_items: Some(25),
                    games: None,
                    criteria: Some(vec![DiscoveryCriterion {
                        field: "HighScoreSupport".into(),
                        comparison: "IsTrue".into(),
                        value: String::new(),
                    }]),
                },
                DiscoveryList {
                    id: 2,
                    title: "Needs Two".into(),
                    subtitle: None,
                    list_type: None,
                    sort_by: None,
                    sort_ascending: None,
                    priority_rank: None,
                    minimum_items: Some(2),
                    maximum_items: Some(25),
                    games: None,
                    criteria: Some(vec![DiscoveryCriterion {
                        field: "StarRating".into(),
                        comparison: "GreaterThan".into(),
                        value: "4".into(),
                    }]),
                },
            ],
        };
        let payload = project_big_box_discovery_with_provider(
            &[rated],
            Utc.with_ymd_and_hms(2026, 7, 26, 0, 0, 0).unwrap(),
            Some(&catalog),
            BigBoxDiscoveryProviderState::Ready,
            0,
        );

        assert_eq!(payload.provider.rejected_lists, 1);
        assert_eq!(payload.provider.rendered_lists, 0);
        assert_eq!(payload.sections.len(), 7);
        assert_eq!(payload.sections[6].key, "provider:2");
        assert!(!payload.sections[6].displayable);
        assert_eq!(payload.sections[6].items.len(), 1);
    }
}
