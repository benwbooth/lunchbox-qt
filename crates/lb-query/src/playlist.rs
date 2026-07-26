use super::parse_timestamp;
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use lb_domain::{Game, PlaylistFilter};
use std::collections::BTreeMap;

/// LaunchBox combines criteria for the same field with OR, then combines the
/// distinct field groups with AND.
pub fn playlist_filters_match(game: &Game, filters: &[PlaylistFilter], now: DateTime<Utc>) -> bool {
    let mut grouped = BTreeMap::<String, Vec<&PlaylistFilter>>::new();
    for filter in filters {
        grouped
            .entry(filter.field_key.to_ascii_lowercase())
            .or_default()
            .push(filter);
    }
    grouped.values().all(|group| {
        group
            .iter()
            .any(|filter| playlist_filter_matches(game, filter, now))
    })
}

pub fn playlist_filter_is_supported(filter: &PlaylistFilter) -> bool {
    let field = filter.field_key.trim().to_ascii_lowercase();
    let comparison = filter.comparison_type_key.trim().to_ascii_lowercase();
    let value = filter.value.trim();
    match field.as_str() {
        "favorite" | "completed" | "broken" | "hidden" | "hide" | "installed" | "portable"
        | "usedosbox" => {
            matches!(
                comparison.as_str(),
                "istrue" | "isfalse" | "equalto" | "isequalto" | "notequalto" | "isnotequalto"
            ) && (!matches!(
                comparison.as_str(),
                "equalto" | "isequalto" | "notequalto" | "isnotequalto"
            ) || value.parse::<bool>().is_ok())
        }
        "playcount" | "maxplayers" | "starrating" | "launchboxid" => {
            numeric_comparison_supported(&comparison, value)
        }
        "lastplayed" | "dateadded" | "datemodified" | "releasedate" => {
            date_comparison_supported(&comparison, value)
        }
        "title" | "sorttitle" | "platform" | "genre" | "publisher" | "series" | "source"
        | "playmode" | "developer" | "status" | "region" | "rating" | "releasetype" | "version"
        | "progress" | "notes" | "applicationrompath" => text_comparison_supported(&comparison),
        _ => false,
    }
}

pub fn playlist_filter_matches(game: &Game, filter: &PlaylistFilter, now: DateTime<Utc>) -> bool {
    if !playlist_filter_is_supported(filter) {
        return false;
    }
    let field = filter.field_key.trim().to_ascii_lowercase();
    let comparison = filter.comparison_type_key.trim().to_ascii_lowercase();
    let expected = filter.value.trim();

    if let Some(actual) = boolean_value(game, &field) {
        return match comparison.as_str() {
            "istrue" => actual,
            "isfalse" => !actual,
            "equalto" | "isequalto" => expected
                .parse::<bool>()
                .is_ok_and(|expected| actual == expected),
            "notequalto" | "isnotequalto" => expected
                .parse::<bool>()
                .is_ok_and(|expected| actual != expected),
            _ => false,
        };
    }
    if let Some(actual) = numeric_value(game, &field) {
        return numeric_matches(actual, &comparison, expected);
    }
    if let Some(actual) = date_value(game, &field) {
        return date_matches(actual, &comparison, expected, now);
    }
    text_matches(text_value(game, &field), &comparison, expected)
}

fn boolean_value(game: &Game, field: &str) -> Option<bool> {
    match field {
        "favorite" => Some(game.favorite),
        "completed" => Some(game.completed),
        "broken" => Some(game.broken),
        "hidden" | "hide" => Some(game.hidden),
        "installed" => game.installed,
        "portable" => Some(game.portable),
        "usedosbox" => Some(game.use_dos_box),
        _ => None,
    }
}

fn numeric_value(game: &Game, field: &str) -> Option<f64> {
    match field {
        "playcount" => Some(f64::from(game.play_count)),
        "maxplayers" => game.max_players.map(f64::from),
        "starrating" => Some(if game.star_rating_float > 0.0 {
            game.star_rating_float
        } else {
            f64::from(game.star_rating)
        }),
        "launchboxid" => game.database_id.map(f64::from),
        _ => None,
    }
}

fn date_value(game: &Game, field: &str) -> Option<i64> {
    match field {
        "lastplayed" => game.last_played_date.as_deref(),
        "dateadded" => Some(game.date_added.as_str()),
        "datemodified" => Some(game.date_modified.as_str()),
        "releasedate" => game.release_date.as_deref(),
        _ => None,
    }
    .and_then(parse_timestamp)
}

fn text_value<'a>(game: &'a Game, field: &str) -> &'a str {
    match field {
        "title" => Some(game.title.as_str()),
        "sorttitle" => Some(game.display_sort_title()),
        "platform" => Some(game.platform.as_str()),
        "genre" => game.genre.as_deref(),
        "publisher" => game.publisher.as_deref(),
        "series" => game.series.as_deref(),
        "source" => game.source.as_deref(),
        "playmode" => game.play_mode.as_deref(),
        "developer" => game.developer.as_deref(),
        "status" => game.status.as_deref(),
        "region" => game.region.as_deref(),
        "rating" => game.rating.as_deref(),
        "releasetype" => game.release_type.as_deref(),
        "version" => game.version.as_deref(),
        "progress" => game.progress.as_deref(),
        "notes" => game.notes.as_deref(),
        "applicationrompath" => Some(game.application_path.as_str()),
        _ => None,
    }
    .unwrap_or_default()
}

fn text_comparison_supported(comparison: &str) -> bool {
    matches!(
        comparison,
        "contains"
            | "doesntcontain"
            | "notcontains"
            | "equalto"
            | "isequalto"
            | "notequalto"
            | "isnotequalto"
            | "startswith"
            | "endswith"
            | "isempty"
            | "isnotempty"
            | "containsany"
            | "doesntcontainany"
            | "hasatleastoneof"
            | "hasallvalues"
            | "hasnoneofthevalues"
    )
}

fn text_matches(actual: &str, comparison: &str, expected: &str) -> bool {
    let actual = actual.trim().to_lowercase();
    let expected = expected.trim().to_lowercase();
    let values = split_values(&expected);
    match comparison {
        "contains" => actual.contains(&expected),
        "doesntcontain" | "notcontains" => !actual.contains(&expected),
        "equalto" | "isequalto" => actual == expected,
        "notequalto" | "isnotequalto" => actual != expected,
        "startswith" => actual.starts_with(&expected),
        "endswith" => actual.ends_with(&expected),
        "isempty" => actual.is_empty(),
        "isnotempty" => !actual.is_empty(),
        "containsany" | "hasatleastoneof" => {
            !values.is_empty() && values.iter().any(|value| actual.contains(value))
        }
        "doesntcontainany" | "hasnoneofthevalues" => {
            values.iter().all(|value| !actual.contains(value))
        }
        "hasallvalues" => !values.is_empty() && values.iter().all(|value| actual.contains(value)),
        _ => false,
    }
}

fn split_values(value: &str) -> Vec<&str> {
    value
        .split([',', ';', '|'])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect()
}

fn numeric_comparison_supported(comparison: &str, value: &str) -> bool {
    match comparison {
        "equalto" | "isequalto" | "notequalto" | "isnotequalto" | "greaterthan" | "lessthan" => {
            value.parse::<f64>().is_ok()
        }
        "isbetweennumeric" => parse_between_numeric(value).is_some(),
        _ => false,
    }
}

fn numeric_matches(actual: f64, comparison: &str, expected: &str) -> bool {
    match comparison {
        "isbetweennumeric" => parse_between_numeric(expected)
            .is_some_and(|(start, end)| actual >= start && actual <= end),
        _ => {
            let Ok(expected) = expected.parse::<f64>() else {
                return false;
            };
            match comparison {
                "equalto" | "isequalto" => actual == expected,
                "notequalto" | "isnotequalto" => actual != expected,
                "greaterthan" => actual > expected,
                "lessthan" => actual < expected,
                _ => false,
            }
        }
    }
}

fn parse_between_numeric(value: &str) -> Option<(f64, f64)> {
    let (start, end) = split_between(value)?;
    Some((start.parse().ok()?, end.parse().ok()?))
}

fn date_comparison_supported(comparison: &str, value: &str) -> bool {
    match comparison {
        "recentdays" => value.parse::<i64>().is_ok_and(|days| days >= 0),
        "onorafter" | "onorbefore" => parse_expected_date(value).is_some(),
        "isbetweendates" => parse_between_dates(value).is_some(),
        _ => false,
    }
}

fn date_matches(actual: i64, comparison: &str, expected: &str, now: DateTime<Utc>) -> bool {
    match comparison {
        "recentdays" => expected.parse::<i64>().is_ok_and(|days| {
            days >= 0
                && actual <= now.timestamp()
                && now.timestamp().saturating_sub(actual) <= days.saturating_mul(86_400)
        }),
        "onorafter" => parse_expected_date(expected).is_some_and(|expected| actual >= expected),
        "onorbefore" => parse_expected_date(expected).is_some_and(|expected| actual <= expected),
        "isbetweendates" => parse_between_dates(expected)
            .is_some_and(|(start, end)| actual >= start && actual <= end),
        _ => false,
    }
}

fn parse_between_dates(value: &str) -> Option<(i64, i64)> {
    let (start, end) = split_between(value)?;
    Some((parse_expected_date(start)?, parse_expected_date(end)?))
}

fn split_between(value: &str) -> Option<(&str, &str)> {
    let (start, end) = value.split_once(" AND ")?;
    Some((start.trim(), end.trim()))
}

fn parse_expected_date(value: &str) -> Option<i64> {
    parse_timestamp(value).or_else(|| {
        ["%Y-%m-%d", "%m/%d/%Y"]
            .into_iter()
            .find_map(|format| NaiveDate::parse_from_str(value, format).ok())
            .and_then(|date| date.and_hms_opt(0, 0, 0))
            .map(|date| Utc.from_utc_datetime(&date).timestamp())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn filter(field: &str, comparison: &str, value: &str) -> PlaylistFilter {
        PlaylistFilter {
            field_key: field.into(),
            comparison_type_key: comparison.into(),
            value: value.into(),
        }
    }

    fn game() -> Game {
        Game {
            id: "one".into(),
            title: "Robotron 2084".into(),
            platform: "Arcade".into(),
            application_path: "Games/robotron.zip".into(),
            database_id: Some(42),
            date_added: "2026-07-20T00:00:00Z".into(),
            date_modified: "2026-07-21T00:00:00Z".into(),
            release_date: Some("1982-01-01".into()),
            genre: Some("Action; Shooter".into()),
            favorite: true,
            installed: Some(true),
            play_count: 9,
            star_rating_float: 4.5,
            ..Game::default()
        }
    }

    #[test]
    fn uses_or_within_a_field_and_and_across_fields() {
        let filters = [
            filter("Genre", "Contains", "Puzzle"),
            filter("Genre", "Contains", "Shooter"),
            filter("Platform", "EqualTo", "Arcade"),
        ];
        assert!(playlist_filters_match(
            &game(),
            &filters,
            Utc.with_ymd_and_hms(2026, 7, 26, 0, 0, 0).unwrap()
        ));
        let mut wrong_platform = filters;
        wrong_platform[2].value = "Console".into();
        assert!(!playlist_filters_match(
            &game(),
            &wrong_platform,
            Utc.with_ymd_and_hms(2026, 7, 26, 0, 0, 0).unwrap()
        ));
    }

    #[test]
    fn handles_recovered_boolean_numeric_date_and_multivalue_comparisons() {
        let now = Utc.with_ymd_and_hms(2026, 7, 26, 0, 0, 0).unwrap();
        for criterion in [
            filter("Favorite", "IsTrue", ""),
            filter("StarRating", "GreaterThan", "4"),
            filter("PlayCount", "IsBetweenNumeric", "5 AND 10"),
            filter("DateAdded", "RecentDays", "7"),
            filter("ReleaseDate", "OnOrBefore", "1982-01-01"),
            filter("Genre", "HasAtLeastOneOf", "Racing, Shooter"),
        ] {
            assert!(playlist_filter_is_supported(&criterion));
            assert!(
                playlist_filter_matches(&game(), &criterion, now),
                "{criterion:?}"
            );
        }
    }

    #[test]
    fn unsupported_contracts_are_never_silently_treated_as_matches() {
        let unsupported = filter("HighScoreSupport", "IsTrue", "");
        assert!(!playlist_filter_is_supported(&unsupported));
        assert!(!playlist_filter_matches(&game(), &unsupported, Utc::now()));
        assert!(!playlist_filter_is_supported(&filter(
            "PlayCount",
            "GreaterThan",
            "many"
        )));
    }
}
