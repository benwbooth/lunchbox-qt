use lb_domain::{FrontendSettings, Game};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::io::Cursor;
use xmltree::Element;

const MAX_PROFILE_XML_BYTES: usize = 256 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RelatedGamesSection {
    Recommended,
    Similar,
    PossiblePorts,
}

impl RelatedGamesSection {
    pub const ALL: [Self; 3] = [Self::Recommended, Self::Similar, Self::PossiblePorts];

    pub const fn key(self) -> &'static str {
        match self {
            Self::Recommended => "recommended",
            Self::Similar => "similar",
            Self::PossiblePorts => "possiblePorts",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Recommended => "Recommended Games",
            Self::Similar => "Similar Games",
            Self::PossiblePorts => "Possible Ports",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RelatedProfileSource {
    PersistedLaunchBoxSettings,
    RecoveredLaunchBoxDefault,
    PortReconstruction,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RelatedCandidateSource {
    #[default]
    Local,
    Database,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelatedFilterType {
    AllGames,
    LocalGamesOnly,
    DatabaseGamesOnly,
}

impl RelatedFilterType {
    fn applies_to(self, source: RelatedCandidateSource) -> bool {
        match self {
            Self::AllGames => true,
            Self::LocalGamesOnly => source == RelatedCandidateSource::Local,
            Self::DatabaseGamesOnly => source == RelatedCandidateSource::Database,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RelatedField {
    Notes,
    ReleaseType,
    Title,
    AlternateName,
    Series,
    Genre,
    PlayMode,
    MaxPlayers,
    Platform,
    Rating,
    Developer,
    Publisher,
    StarRating,
}

impl RelatedField {
    pub const fn key(self) -> &'static str {
        match self {
            Self::Notes => "Notes",
            Self::ReleaseType => "ReleaseType",
            Self::Title => "Title",
            Self::AlternateName => "AlternateName",
            Self::Series => "Series",
            Self::Genre => "Genre",
            Self::PlayMode => "PlayMode",
            Self::MaxPlayers => "MaxPlayers",
            Self::Platform => "Platform",
            Self::Rating => "Rating",
            Self::Developer => "Developer",
            Self::Publisher => "Publisher",
            Self::StarRating => "StarRating",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelatedComparison {
    IsNotEmpty,
    IsEmpty,
    EqualTo,
    NotEqualTo,
    IsSimilarTo,
    IsNotSimilarTo,
    GreaterThan,
    LessThan,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelatedCriterion {
    pub field: RelatedField,
    pub comparison: RelatedComparison,
    pub comparison_value: Option<String>,
    pub use_game_value: bool,
    /// `None` is a required criterion. `Some(weight)` contributes to scoring.
    pub weight: Option<u16>,
    pub filter: RelatedFilterType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelatedGamesProfile {
    pub allow_database_games: bool,
    pub minimum_score: u32,
    pub criteria: Vec<RelatedCriterion>,
    pub source: RelatedProfileSource,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelatedGamesPolicy {
    pub show_game_menu_related_games: bool,
    pub recommended: RelatedGamesProfile,
    pub similar: RelatedGamesProfile,
    pub possible_ports: RelatedGamesProfile,
}

impl Default for RelatedGamesPolicy {
    fn default() -> Self {
        Self::from_settings(None, None)
    }
}

impl RelatedGamesPolicy {
    pub fn from_settings(
        launchbox_settings: Option<&FrontendSettings>,
        big_box_settings: Option<&FrontendSettings>,
    ) -> Self {
        Self {
            show_game_menu_related_games: big_box_settings
                .and_then(|settings| settings.get_bool("ShowGameMenuViewRelatedGames"))
                .unwrap_or(true),
            recommended: profile_or_default(
                launchbox_settings.and_then(|settings| settings.get("RecommendedGamesXmlString")),
                recovered_recommended_profile,
            ),
            similar: profile_or_default(
                launchbox_settings.and_then(|settings| settings.get("SimilarGamesXmlString")),
                recovered_similar_profile,
            ),
            possible_ports: profile_or_default(
                launchbox_settings.and_then(|settings| settings.get("PossiblePortsXmlString")),
                reconstructed_possible_ports_profile,
            ),
        }
    }

    pub fn profile(&self, section: RelatedGamesSection) -> &RelatedGamesProfile {
        match section {
            RelatedGamesSection::Recommended => &self.recommended,
            RelatedGamesSection::Similar => &self.similar,
            RelatedGamesSection::PossiblePorts => &self.possible_ports,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedGameCandidate {
    pub source: RelatedCandidateSource,
    pub local_game_id: Option<String>,
    pub database_id: Option<i64>,
    pub title: String,
    pub alternate_names: Vec<String>,
    pub platform: String,
    pub notes: Option<String>,
    pub release_date: Option<String>,
    pub release_year: Option<i32>,
    pub release_type: Option<String>,
    pub series: Option<String>,
    pub genres: Vec<String>,
    pub play_modes: Vec<String>,
    pub max_players: Option<u32>,
    pub rating: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub star_rating: f64,
    pub star_rating_votes: u32,
}

impl RelatedGameCandidate {
    pub fn from_local_game(game: &Game, alternate_names: Vec<String>) -> Self {
        Self {
            source: RelatedCandidateSource::Local,
            local_game_id: Some(game.id.clone()),
            database_id: game.database_id.map(i64::from),
            title: game.title.clone(),
            alternate_names,
            platform: game.platform.clone(),
            notes: game.notes.clone(),
            release_date: game.release_date.clone(),
            release_year: game
                .release_date
                .as_deref()
                .and_then(|value| value.get(..4))
                .and_then(|year| year.parse().ok()),
            release_type: game.release_type.clone(),
            series: game.series.clone(),
            genres: split_multi(game.genre.as_deref().unwrap_or_default()),
            play_modes: split_multi(game.play_mode.as_deref().unwrap_or_default()),
            max_players: game.max_players,
            rating: game.rating.clone(),
            developer: game.developer.clone(),
            publisher: game.publisher.clone(),
            star_rating: if game.star_rating_float > 0.0 {
                game.star_rating_float
            } else {
                f64::from(game.star_rating)
            },
            star_rating_votes: 0,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedCriterionScore {
    pub field: RelatedField,
    pub earned: u32,
    pub possible: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedGameSuggestion {
    #[serde(flatten)]
    pub candidate: RelatedGameCandidate,
    pub actual_score: u32,
    pub total_score: u32,
    pub score_percent: u32,
    pub score_breakdown: Vec<RelatedCriterionScore>,
}

/// Evaluate one recovered LaunchBox profile.
///
/// Criteria, filters and weights are recovered from persisted 13.27 settings.
/// LaunchBox's protected similarity implementation is unavailable, so this
/// port uses [`port_similarity`] and deterministic nearest-integer percentages.
pub fn related_game_suggestions(
    profile: &RelatedGamesProfile,
    seed: &RelatedGameCandidate,
    candidates: &[RelatedGameCandidate],
    maximum: usize,
) -> Vec<RelatedGameSuggestion> {
    let mut suggestions = candidates
        .iter()
        .filter(|candidate| {
            profile.allow_database_games || candidate.source != RelatedCandidateSource::Database
        })
        .filter(|candidate| !same_game(seed, candidate))
        .filter_map(|candidate| score_candidate(profile, seed, candidate))
        .collect::<Vec<_>>();
    suggestions.sort_by(|left, right| {
        right
            .score_percent
            .cmp(&left.score_percent)
            .then_with(|| right.actual_score.cmp(&left.actual_score))
            .then_with(|| {
                left.candidate
                    .title
                    .to_lowercase()
                    .cmp(&right.candidate.title.to_lowercase())
            })
            .then_with(|| {
                left.candidate
                    .platform
                    .to_lowercase()
                    .cmp(&right.candidate.platform.to_lowercase())
            })
            .then_with(|| left.candidate.database_id.cmp(&right.candidate.database_id))
            .then_with(|| {
                left.candidate
                    .local_game_id
                    .cmp(&right.candidate.local_game_id)
            })
    });
    suggestions.truncate(maximum);
    suggestions
}

fn score_candidate(
    profile: &RelatedGamesProfile,
    seed: &RelatedGameCandidate,
    candidate: &RelatedGameCandidate,
) -> Option<RelatedGameSuggestion> {
    for criterion in profile
        .criteria
        .iter()
        .filter(|criterion| criterion.weight.is_none())
        .filter(|criterion| criterion.filter.applies_to(candidate.source))
    {
        if !criterion_matches(criterion, seed, candidate) {
            return None;
        }
    }

    let mut actual_score = 0_u32;
    let mut total_score = 0_u32;
    let mut score_breakdown = Vec::new();
    for criterion in profile
        .criteria
        .iter()
        .filter(|criterion| criterion.filter.applies_to(candidate.source))
    {
        let Some(weight) = criterion.weight else {
            continue;
        };
        let possible = u32::from(weight);
        let earned = if criterion_matches(criterion, seed, candidate) {
            possible
        } else {
            0
        };
        actual_score = actual_score.saturating_add(earned);
        total_score = total_score.saturating_add(possible);
        score_breakdown.push(RelatedCriterionScore {
            field: criterion.field,
            earned,
            possible,
        });
    }
    if actual_score < profile.minimum_score {
        return None;
    }
    let score_percent = actual_score
        .saturating_mul(100)
        .saturating_add(total_score / 2)
        .checked_div(total_score)
        // A profile containing only required criteria is a complete match
        // after those criteria pass; showing it as 0% would be misleading.
        .unwrap_or(100);
    Some(RelatedGameSuggestion {
        candidate: candidate.clone(),
        actual_score,
        total_score,
        score_percent,
        score_breakdown,
    })
}

fn criterion_matches(
    criterion: &RelatedCriterion,
    seed: &RelatedGameCandidate,
    candidate: &RelatedGameCandidate,
) -> bool {
    let candidate_values = field_values(candidate, criterion.field);
    let comparison_values = if criterion.use_game_value {
        field_values(seed, criterion.field)
    } else {
        criterion
            .comparison_value
            .as_deref()
            .map(split_multi)
            .unwrap_or_default()
    };
    match criterion.comparison {
        RelatedComparison::IsNotEmpty => candidate_values
            .iter()
            .any(|value| !value.trim().is_empty()),
        RelatedComparison::IsEmpty => candidate_values.iter().all(|value| value.trim().is_empty()),
        RelatedComparison::EqualTo => any_equal(&candidate_values, &comparison_values),
        RelatedComparison::NotEqualTo => !any_equal(&candidate_values, &comparison_values),
        RelatedComparison::IsSimilarTo => any_similar(&candidate_values, &comparison_values),
        RelatedComparison::IsNotSimilarTo => !any_similar(&candidate_values, &comparison_values),
        RelatedComparison::GreaterThan => {
            compare_numbers(&candidate_values, &comparison_values, |a, b| a > b)
        }
        RelatedComparison::LessThan => {
            compare_numbers(&candidate_values, &comparison_values, |a, b| a < b)
        }
    }
}

fn field_values(candidate: &RelatedGameCandidate, field: RelatedField) -> Vec<String> {
    match field {
        RelatedField::Notes => option_values(candidate.notes.as_deref()),
        RelatedField::ReleaseType => option_values(candidate.release_type.as_deref()),
        RelatedField::Title => vec![candidate.title.clone()],
        RelatedField::AlternateName => candidate.alternate_names.clone(),
        RelatedField::Series => option_values(candidate.series.as_deref()),
        RelatedField::Genre => candidate.genres.clone(),
        RelatedField::PlayMode => candidate.play_modes.clone(),
        RelatedField::MaxPlayers => candidate
            .max_players
            .map(|value| vec![value.to_string()])
            .unwrap_or_default(),
        RelatedField::Platform => vec![candidate.platform.clone()],
        RelatedField::Rating => option_values(candidate.rating.as_deref()),
        RelatedField::Developer => option_values(candidate.developer.as_deref()),
        RelatedField::Publisher => option_values(candidate.publisher.as_deref()),
        RelatedField::StarRating => vec![candidate.star_rating.to_string()],
    }
}

fn option_values(value: Option<&str>) -> Vec<String> {
    value.map(split_multi).unwrap_or_default()
}

fn split_multi(value: &str) -> Vec<String> {
    value
        .split([';', ','])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn any_equal(left: &[String], right: &[String]) -> bool {
    left.iter().any(|left| {
        right
            .iter()
            .any(|right| left.trim().eq_ignore_ascii_case(right.trim()))
    })
}

fn any_similar(left: &[String], right: &[String]) -> bool {
    left.iter()
        .any(|left| right.iter().any(|right| port_similarity(left, right)))
}

/// Port-owned replacement for the protected 13.27 similarity predicate.
///
/// Exact normalized values match. Otherwise at least half of the distinct
/// normalized tokens must overlap in both directions.
pub fn port_similarity(left: &str, right: &str) -> bool {
    let left = normalized_tokens(left);
    let right = normalized_tokens(right);
    if left.is_empty() || right.is_empty() {
        return false;
    }
    if left == right {
        return true;
    }
    let overlap = left.intersection(&right).count();
    overlap >= 2
        && overlap.saturating_mul(2) >= left.len()
        && overlap.saturating_mul(2) >= right.len()
}

fn normalized_tokens(value: &str) -> BTreeSet<String> {
    value
        .chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .filter(|token| !matches!(*token, "a" | "an" | "the" | "and"))
        .map(ToOwned::to_owned)
        .collect()
}

fn compare_numbers(
    left: &[String],
    right: &[String],
    predicate: impl Fn(f64, f64) -> bool,
) -> bool {
    left.iter().any(|left| {
        left.parse::<f64>().ok().is_some_and(|left| {
            right.iter().any(|right| {
                right
                    .parse::<f64>()
                    .ok()
                    .is_some_and(|right| predicate(left, right))
            })
        })
    })
}

fn same_game(left: &RelatedGameCandidate, right: &RelatedGameCandidate) -> bool {
    match (&left.local_game_id, &right.local_game_id) {
        (Some(left), Some(right)) if left.eq_ignore_ascii_case(right) => return true,
        _ => {}
    }
    matches!(
        (left.database_id, right.database_id),
        (Some(left), Some(right)) if left == right
    )
}

fn profile_or_default(
    serialized: Option<&str>,
    default: fn() -> RelatedGamesProfile,
) -> RelatedGamesProfile {
    let Some(serialized) = serialized.map(str::trim).filter(|value| !value.is_empty()) else {
        return default();
    };
    parse_profile(serialized).unwrap_or_else(default)
}

fn parse_profile(serialized: &str) -> Option<RelatedGamesProfile> {
    if serialized.len() > MAX_PROFILE_XML_BYTES {
        return None;
    }
    let serialized = strip_xml_declaration(serialized);
    let root = Element::parse(Cursor::new(serialized.as_bytes())).ok()?;
    if root.name != "GameSuggesterSaveData" {
        return None;
    }
    let allow_database_games = child_text(&root, "AllowDbGames")?.parse::<bool>().ok()?;
    let minimum_score = child_text(&root, "MinimumScore")?.parse::<u32>().ok()?;
    let criteria_element = root.get_child("Criteria")?;
    let criteria = criteria_element
        .children
        .iter()
        .filter_map(|node| node.as_element())
        .map(parse_criterion)
        .collect::<Option<Vec<_>>>()?;
    if criteria.is_empty() || criteria.len() > 128 {
        return None;
    }
    Some(RelatedGamesProfile {
        allow_database_games,
        minimum_score,
        criteria,
        source: RelatedProfileSource::PersistedLaunchBoxSettings,
    })
}

fn parse_criterion(element: &Element) -> Option<RelatedCriterion> {
    if element.name != "CriteriaRecord" {
        return None;
    }
    let field = match child_text(element, "FieldKey")?.as_str() {
        "Notes" => RelatedField::Notes,
        "ReleaseType" => RelatedField::ReleaseType,
        "Title" => RelatedField::Title,
        "AlternateName" => RelatedField::AlternateName,
        "Series" => RelatedField::Series,
        "Genre" => RelatedField::Genre,
        "PlayMode" => RelatedField::PlayMode,
        "MaxPlayers" => RelatedField::MaxPlayers,
        "Platform" => RelatedField::Platform,
        "Rating" => RelatedField::Rating,
        "Developer" => RelatedField::Developer,
        "Publisher" => RelatedField::Publisher,
        "StarRating" => RelatedField::StarRating,
        _ => return None,
    };
    let comparison = match child_text(element, "ComparisonTypeKey")?.as_str() {
        "IsNotEmpty" => RelatedComparison::IsNotEmpty,
        "IsEmpty" => RelatedComparison::IsEmpty,
        "EqualTo" => RelatedComparison::EqualTo,
        "NotEqualTo" => RelatedComparison::NotEqualTo,
        "IsSimilarTo" => RelatedComparison::IsSimilarTo,
        "IsNotSimilarTo" => RelatedComparison::IsNotSimilarTo,
        "GreaterThan" => RelatedComparison::GreaterThan,
        "LessThan" => RelatedComparison::LessThan,
        _ => return None,
    };
    let filter = match child_text(element, "FilterType")?.as_str() {
        "AllGames" => RelatedFilterType::AllGames,
        "LocalGamesOnly" => RelatedFilterType::LocalGamesOnly,
        "DatabaseGamesOnly" => RelatedFilterType::DatabaseGamesOnly,
        _ => return None,
    };
    let use_game_value = child_text(element, "UseGameValue")?.parse::<bool>().ok()?;
    let comparison_value = element
        .get_child("ComparisonValue")
        .and_then(Element::get_text)
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
    let weight = element
        .get_child("Weight")
        .and_then(Element::get_text)
        .map(|value| value.trim().parse::<u16>())
        .transpose()
        .ok()?
        .filter(|weight| *weight > 0);
    Some(RelatedCriterion {
        field,
        comparison,
        comparison_value,
        use_game_value,
        weight,
        filter,
    })
}

fn strip_xml_declaration(value: &str) -> &str {
    let value = value.trim_start_matches('\u{feff}').trim_start();
    value
        .strip_prefix("<?xml")
        .and_then(|rest| rest.find("?>").map(|end| &rest[end + 2..]))
        .unwrap_or(value)
}

fn child_text(element: &Element, name: &str) -> Option<String> {
    element
        .get_child(name)
        .and_then(Element::get_text)
        .map(|value| value.trim().to_owned())
}

fn criterion(
    field: RelatedField,
    comparison: RelatedComparison,
    filter: RelatedFilterType,
    use_game_value: bool,
    comparison_value: Option<&str>,
    weight: Option<u16>,
) -> RelatedCriterion {
    RelatedCriterion {
        field,
        comparison,
        comparison_value: comparison_value.map(ToOwned::to_owned),
        use_game_value,
        weight,
        filter,
    }
}

pub fn recovered_similar_profile() -> RelatedGamesProfile {
    use RelatedComparison::{EqualTo, IsNotEmpty, IsSimilarTo, NotEqualTo};
    use RelatedField::*;
    use RelatedFilterType::{AllGames, DatabaseGamesOnly};
    RelatedGamesProfile {
        allow_database_games: true,
        minimum_score: 0,
        source: RelatedProfileSource::RecoveredLaunchBoxDefault,
        criteria: vec![
            criterion(Notes, IsNotEmpty, DatabaseGamesOnly, false, None, None),
            criterion(
                ReleaseType,
                EqualTo,
                DatabaseGamesOnly,
                false,
                Some("Released"),
                None,
            ),
            criterion(Title, NotEqualTo, AllGames, true, None, None),
            criterion(Title, IsSimilarTo, AllGames, true, None, Some(2)),
            criterion(AlternateName, IsSimilarTo, AllGames, true, None, Some(2)),
            criterion(Series, IsSimilarTo, AllGames, true, None, Some(2)),
            criterion(Genre, EqualTo, AllGames, true, None, Some(3)),
            criterion(PlayMode, EqualTo, AllGames, true, None, Some(2)),
            criterion(MaxPlayers, EqualTo, AllGames, true, None, Some(1)),
            criterion(Platform, EqualTo, AllGames, true, None, Some(2)),
            criterion(Rating, EqualTo, AllGames, true, None, Some(2)),
            criterion(Developer, EqualTo, AllGames, true, None, Some(1)),
            criterion(Publisher, EqualTo, AllGames, true, None, Some(1)),
        ],
    }
}

pub fn recovered_recommended_profile() -> RelatedGamesProfile {
    use RelatedComparison::{EqualTo, GreaterThan, IsNotSimilarTo, NotEqualTo};
    use RelatedField::*;
    use RelatedFilterType::{AllGames, DatabaseGamesOnly, LocalGamesOnly};
    RelatedGamesProfile {
        allow_database_games: true,
        minimum_score: 0,
        source: RelatedProfileSource::RecoveredLaunchBoxDefault,
        criteria: vec![
            criterion(
                ReleaseType,
                EqualTo,
                DatabaseGamesOnly,
                false,
                Some("Released"),
                None,
            ),
            criterion(Title, NotEqualTo, AllGames, true, None, None),
            criterion(StarRating, GreaterThan, AllGames, false, Some("3.5"), None),
            criterion(Series, IsNotSimilarTo, LocalGamesOnly, true, None, None),
            criterion(Genre, EqualTo, AllGames, true, None, Some(3)),
            criterion(PlayMode, EqualTo, AllGames, true, None, Some(2)),
            criterion(MaxPlayers, EqualTo, AllGames, true, None, Some(1)),
            criterion(Platform, EqualTo, AllGames, true, None, Some(1)),
            criterion(
                StarRating,
                GreaterThan,
                AllGames,
                false,
                Some("4.1"),
                Some(3),
            ),
            criterion(Rating, EqualTo, AllGames, true, None, Some(2)),
            criterion(Developer, EqualTo, AllGames, true, None, Some(1)),
            criterion(Publisher, EqualTo, AllGames, true, None, Some(1)),
        ],
    }
}

/// No serialized default was present in the inspected installations. This
/// profile implements the publicly documented exact-title/different-platform
/// behavior and is deliberately marked as a port reconstruction.
pub fn reconstructed_possible_ports_profile() -> RelatedGamesProfile {
    use RelatedComparison::{EqualTo, NotEqualTo};
    use RelatedField::{Platform, ReleaseType, Title};
    use RelatedFilterType::{AllGames, DatabaseGamesOnly};
    RelatedGamesProfile {
        allow_database_games: true,
        minimum_score: 0,
        source: RelatedProfileSource::PortReconstruction,
        criteria: vec![
            criterion(Title, EqualTo, AllGames, true, None, None),
            criterion(Platform, NotEqualTo, AllGames, true, None, None),
            criterion(
                ReleaseType,
                EqualTo,
                DatabaseGamesOnly,
                false,
                Some("Released"),
                None,
            ),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(
        id: Option<&str>,
        database_id: Option<i64>,
        title: &str,
        platform: &str,
    ) -> RelatedGameCandidate {
        RelatedGameCandidate {
            source: if id.is_some() {
                RelatedCandidateSource::Local
            } else {
                RelatedCandidateSource::Database
            },
            local_game_id: id.map(ToOwned::to_owned),
            database_id,
            title: title.to_owned(),
            platform: platform.to_owned(),
            notes: Some("A game".to_owned()),
            release_type: Some("Released".to_owned()),
            genres: vec!["Adventure".to_owned()],
            play_modes: vec!["Single Player".to_owned()],
            max_players: Some(1),
            rating: Some("E".to_owned()),
            developer: Some("Studio".to_owned()),
            publisher: Some("Publisher".to_owned()),
            star_rating: 4.5,
            ..RelatedGameCandidate::default()
        }
    }

    #[test]
    fn recovered_defaults_keep_explicit_parity_boundaries() {
        let policy = RelatedGamesPolicy::from_settings(None, None);
        assert!(policy.show_game_menu_related_games);
        assert_eq!(
            policy.recommended.source,
            RelatedProfileSource::RecoveredLaunchBoxDefault
        );
        assert_eq!(policy.recommended.criteria.len(), 12);
        assert_eq!(policy.similar.criteria.len(), 13);
        assert_eq!(
            policy.possible_ports.source,
            RelatedProfileSource::PortReconstruction
        );
        assert_eq!(policy.possible_ports.criteria.len(), 3);
    }

    #[test]
    fn serialized_profile_is_bounded_and_parsed_without_trusting_xml_encoding_label() {
        let xml = r#"<?xml version="1.0" encoding="utf-16"?>
            <GameSuggesterSaveData xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
              <AllowDbGames>false</AllowDbGames>
              <Criteria>
                <CriteriaRecord>
                  <ComparisonTypeKey>EqualTo</ComparisonTypeKey>
                  <ComparisonValue>Adventure</ComparisonValue>
                  <FieldKey>Genre</FieldKey>
                  <FilterType>AllGames</FilterType>
                  <UseGameValue>false</UseGameValue>
                  <Weight xsi:nil="true" />
                </CriteriaRecord>
                <CriteriaRecord>
                  <ComparisonTypeKey>GreaterThan</ComparisonTypeKey>
                  <ComparisonValue>4.1</ComparisonValue>
                  <FieldKey>StarRating</FieldKey>
                  <FilterType>LocalGamesOnly</FilterType>
                  <UseGameValue>false</UseGameValue>
                  <Weight>3</Weight>
                </CriteriaRecord>
              </Criteria>
              <MinimumScore>2</MinimumScore>
            </GameSuggesterSaveData>"#;
        let profile = parse_profile(xml).unwrap();
        assert!(!profile.allow_database_games);
        assert_eq!(profile.minimum_score, 2);
        assert_eq!(profile.criteria.len(), 2);
        assert_eq!(profile.criteria[0].weight, None);
        assert_eq!(profile.criteria[1].weight, Some(3));
        assert_eq!(
            profile.source,
            RelatedProfileSource::PersistedLaunchBoxSettings
        );
        assert!(parse_profile(&"x".repeat(MAX_PROFILE_XML_BYTES + 1)).is_none());
    }

    #[test]
    fn malformed_or_unknown_profile_falls_back_as_a_whole() {
        let settings = FrontendSettings {
            entries: vec![lb_domain::SettingEntry {
                key: "SimilarGamesXmlString".to_owned(),
                value: "<GameSuggesterSaveData><unknown/></GameSuggesterSaveData>".to_owned(),
            }],
            ..FrontendSettings::default()
        };
        let policy = RelatedGamesPolicy::from_settings(Some(&settings), None);
        assert_eq!(
            policy.similar.source,
            RelatedProfileSource::RecoveredLaunchBoxDefault
        );
        assert_eq!(policy.similar.criteria.len(), 13);
    }

    #[test]
    fn possible_ports_requires_exact_title_on_another_platform() {
        let seed = candidate(Some("seed"), Some(1), "Fixture Quest", "Console A");
        let candidates = vec![
            candidate(Some("same-platform"), Some(2), "Fixture Quest", "Console A"),
            candidate(Some("port"), Some(3), "Fixture Quest", "Console B"),
            candidate(None, Some(4), "Fixture Quest", "Console C"),
            candidate(Some("sequel"), Some(5), "Fixture Quest II", "Console B"),
        ];
        let suggestions = related_game_suggestions(
            &reconstructed_possible_ports_profile(),
            &seed,
            &candidates,
            10,
        );
        assert_eq!(suggestions.len(), 2);
        assert_eq!(
            suggestions
                .iter()
                .map(|suggestion| suggestion.candidate.platform.as_str())
                .collect::<Vec<_>>(),
            vec!["Console B", "Console C"]
        );
        assert!(suggestions
            .iter()
            .all(|suggestion| suggestion.score_percent == 100));
    }

    #[test]
    fn recommendations_apply_source_filters_score_and_stable_ties() {
        let mut seed = candidate(Some("seed"), Some(1), "Seed", "Console");
        seed.series = Some("Seed Series".to_owned());
        let mut alpha = candidate(Some("alpha"), Some(2), "Alpha", "Console");
        alpha.series = Some("Other Series".to_owned());
        let mut beta = candidate(None, Some(3), "Beta", "Console");
        beta.genres = vec!["Puzzle".to_owned()];
        let mut same_series = candidate(Some("series"), Some(4), "Series Game", "Console");
        same_series.series = seed.series.clone();
        let mut unrated = candidate(Some("unrated"), Some(5), "Unrated", "Console");
        unrated.star_rating = 3.5;

        let candidates = vec![beta, same_series, unrated, alpha];
        let suggestions =
            related_game_suggestions(&recovered_recommended_profile(), &seed, &candidates, 10);
        assert_eq!(
            suggestions
                .iter()
                .map(|suggestion| suggestion.candidate.title.as_str())
                .collect::<Vec<_>>(),
            vec!["Alpha", "Beta"]
        );
        assert!(suggestions[0].score_percent > suggestions[1].score_percent);
        assert_eq!(suggestions[0].total_score, 14);
    }

    #[test]
    fn database_candidates_can_be_disabled_and_seed_identity_is_excluded() {
        let seed = candidate(Some("seed"), Some(42), "Seed", "Console");
        let mut profile = reconstructed_possible_ports_profile();
        profile.allow_database_games = false;
        let candidates = vec![
            candidate(Some("same-db"), Some(42), "Seed", "Other"),
            candidate(None, Some(50), "Seed", "Cloud"),
            candidate(Some("local"), Some(51), "Seed", "Local"),
        ];
        let suggestions = related_game_suggestions(&profile, &seed, &candidates, 10);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(
            suggestions[0].candidate.local_game_id.as_deref(),
            Some("local")
        );
    }

    #[test]
    fn port_similarity_is_conservative_and_platform_neutral() {
        assert!(port_similarity("The Legend of Fixture", "Legend Fixture"));
        assert!(!port_similarity("Fixture Quest", "Completely Different"));
        assert!(!port_similarity("", ""));
    }
}
