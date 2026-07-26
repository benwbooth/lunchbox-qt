pub use lb_domain::{
    DiscoveryCatalog, DiscoveryCriterion, DiscoveryGame, DiscoveryList, DISCOVERY_LISTS_ENDPOINT,
    DISCOVERY_LISTS_MAX_ITEMS_PER_LIST,
};
use serde_json::{Map, Value};
use std::collections::BTreeSet;
use std::io::Read;
use std::time::Duration;
use thiserror::Error;

pub const DISCOVERY_LISTS_MAX_RESPONSE_BYTES: u64 = 2 * 1024 * 1024;
pub const DISCOVERY_LISTS_MAX_LISTS: usize = 256;
pub const DISCOVERY_LISTS_MAX_GAMES_PER_LIST: usize = 1_000;
pub const DISCOVERY_LISTS_MAX_CRITERIA_PER_LIST: usize = 64;
const DISCOVERY_LISTS_MAX_TEXT_BYTES: usize = 512;
const DISCOVERY_LISTS_MAX_VALUE_BYTES: usize = 4_096;
const DISCOVERY_LISTS_TIMEOUT: Duration = Duration::from_secs(8);

pub trait DiscoveryCatalogTransport: Send + Sync {
    fn fetch(&self, url: &str, maximum_bytes: u64) -> Result<Vec<u8>, DiscoveryProviderError>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LaunchBoxDiscoveryTransport;

impl DiscoveryCatalogTransport for LaunchBoxDiscoveryTransport {
    fn fetch(&self, url: &str, maximum_bytes: u64) -> Result<Vec<u8>, DiscoveryProviderError> {
        if url != DISCOVERY_LISTS_ENDPOINT {
            return Err(DiscoveryProviderError::UntrustedUrl {
                url: url.to_string(),
            });
        }
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(DISCOVERY_LISTS_TIMEOUT))
            .max_redirects(0)
            .build();
        let agent: ureq::Agent = config.into();
        let mut response = agent
            .get(url)
            .header("Accept", "application/json")
            .header("User-Agent", "lunchbox-qt")
            .call()
            .map_err(|error| DiscoveryProviderError::Transport {
                message: error.to_string(),
            })?;
        read_limited(
            response.body_mut().as_reader(),
            maximum_bytes,
            "Discovery-list response",
        )
    }
}

#[derive(Debug, Error)]
pub enum DiscoveryProviderError {
    #[error("untrusted Discovery-list URL: {url}")]
    UntrustedUrl { url: String },
    #[error("Discovery-list transport failed: {message}")]
    Transport { message: String },
    #[error("{label} exceeded the {maximum_bytes}-byte limit")]
    TooLarge {
        label: &'static str,
        maximum_bytes: u64,
    },
    #[error("could not read {label}: {message}")]
    Read {
        label: &'static str,
        message: String,
    },
    #[error("could not decode the Discovery-list response: {message}")]
    Json { message: String },
    #[error("invalid Discovery-list response at {path}: {message}")]
    Invalid { path: String, message: String },
}

pub fn fetch_discovery_catalog(
    transport: &dyn DiscoveryCatalogTransport,
) -> Result<DiscoveryCatalog, DiscoveryProviderError> {
    let bytes = transport.fetch(DISCOVERY_LISTS_ENDPOINT, DISCOVERY_LISTS_MAX_RESPONSE_BYTES)?;
    parse_discovery_catalog(&bytes)
}

pub fn parse_discovery_catalog(bytes: &[u8]) -> Result<DiscoveryCatalog, DiscoveryProviderError> {
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > DISCOVERY_LISTS_MAX_RESPONSE_BYTES {
        return Err(DiscoveryProviderError::TooLarge {
            label: "Discovery-list response",
            maximum_bytes: DISCOVERY_LISTS_MAX_RESPONSE_BYTES,
        });
    }
    let root: Value =
        serde_json::from_slice(bytes).map_err(|error| DiscoveryProviderError::Json {
            message: error.to_string(),
        })?;
    let root = object_at(&root, "$")?;
    let Some(data) = property_ci(root, "data", "$")? else {
        return Ok(DiscoveryCatalog { lists: Vec::new() });
    };
    if data.is_null() {
        return Ok(DiscoveryCatalog { lists: Vec::new() });
    }
    let data = array_at(data, "$.data")?;
    if data.len() > DISCOVERY_LISTS_MAX_LISTS {
        return Err(invalid(
            "$.data",
            format!(
                "contains {} lists, above the {}-list limit",
                data.len(),
                DISCOVERY_LISTS_MAX_LISTS
            ),
        ));
    }

    let mut ids = BTreeSet::new();
    let mut lists = Vec::with_capacity(data.len());
    for (index, value) in data.iter().enumerate() {
        if value.is_null() {
            continue;
        }
        let path = format!("$.data[{index}]");
        let object = object_at(value, &path)?;
        let id = required_i32(object, "id", &path)?;
        if !ids.insert(id) {
            return Err(invalid(format!("{path}.id"), format!("duplicate ID {id}")));
        }
        let title = required_text(object, "title", &path, DISCOVERY_LISTS_MAX_TEXT_BYTES)?;
        let subtitle = optional_text(object, "subtitle", &path, DISCOVERY_LISTS_MAX_TEXT_BYTES)?;
        let list_type = optional_text(object, "listType", &path, DISCOVERY_LISTS_MAX_TEXT_BYTES)?;
        let sort_by = optional_text(object, "sortBy", &path, DISCOVERY_LISTS_MAX_TEXT_BYTES)?;
        let sort_ascending = optional_bool(object, "sortAsc", &path)?;
        let priority_rank = optional_i32(object, "priorityRank", &path)?;
        let minimum_items = optional_item_count(object, "minimumItems", &path)?;
        let maximum_items = optional_item_count(object, "maximumItems", &path)?;
        if minimum_items
            .zip(maximum_items)
            .is_some_and(|(minimum, maximum)| minimum > maximum)
        {
            return Err(invalid(
                format!("{path}.minimumItems"),
                "minimumItems exceeds maximumItems",
            ));
        }
        let games = optional_array(object, "games", &path)?
            .map(|values| parse_games(values, &path))
            .transpose()?;
        let criteria = optional_array(object, "criteria", &path)?
            .map(|values| parse_criteria(values, &path))
            .transpose()?;
        lists.push(DiscoveryList {
            id,
            title,
            subtitle,
            list_type,
            sort_by,
            sort_ascending,
            priority_rank,
            minimum_items,
            maximum_items,
            games,
            criteria,
        });
    }
    Ok(DiscoveryCatalog { lists })
}

fn parse_games(
    values: &[Value],
    parent: &str,
) -> Result<Vec<DiscoveryGame>, DiscoveryProviderError> {
    if values.len() > DISCOVERY_LISTS_MAX_GAMES_PER_LIST {
        return Err(invalid(
            format!("{parent}.games"),
            format!(
                "contains {} games, above the {}-game limit",
                values.len(),
                DISCOVERY_LISTS_MAX_GAMES_PER_LIST
            ),
        ));
    }
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let path = format!("{parent}.games[{index}]");
            let object = object_at(value, &path)?;
            let database_id = required_i32(object, "id", &path)?;
            let platform =
                required_text(object, "platform", &path, DISCOVERY_LISTS_MAX_TEXT_BYTES)?;
            let title = required_text(object, "title", &path, DISCOVERY_LISTS_MAX_TEXT_BYTES)?;
            Ok(DiscoveryGame {
                database_id,
                platform,
                title,
            })
        })
        .collect()
}

fn parse_criteria(
    values: &[Value],
    parent: &str,
) -> Result<Vec<DiscoveryCriterion>, DiscoveryProviderError> {
    if values.len() > DISCOVERY_LISTS_MAX_CRITERIA_PER_LIST {
        return Err(invalid(
            format!("{parent}.criteria"),
            format!(
                "contains {} criteria, above the {}-criterion limit",
                values.len(),
                DISCOVERY_LISTS_MAX_CRITERIA_PER_LIST
            ),
        ));
    }
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let path = format!("{parent}.criteria[{index}]");
            let object = object_at(value, &path)?;
            Ok(DiscoveryCriterion {
                field: required_text(object, "field", &path, DISCOVERY_LISTS_MAX_TEXT_BYTES)?,
                comparison: required_text(
                    object,
                    "comparison",
                    &path,
                    DISCOVERY_LISTS_MAX_TEXT_BYTES,
                )?,
                value: optional_text_allow_empty(
                    object,
                    "value",
                    &path,
                    DISCOVERY_LISTS_MAX_VALUE_BYTES,
                )?
                .unwrap_or_default(),
            })
        })
        .collect()
}

fn read_limited(
    mut reader: impl Read,
    maximum_bytes: u64,
    label: &'static str,
) -> Result<Vec<u8>, DiscoveryProviderError> {
    let mut bytes = Vec::new();
    reader
        .by_ref()
        .take(maximum_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| DiscoveryProviderError::Read {
            label,
            message: error.to_string(),
        })?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > maximum_bytes {
        return Err(DiscoveryProviderError::TooLarge {
            label,
            maximum_bytes,
        });
    }
    Ok(bytes)
}

fn object_at<'a>(
    value: &'a Value,
    path: &str,
) -> Result<&'a Map<String, Value>, DiscoveryProviderError> {
    value
        .as_object()
        .ok_or_else(|| invalid(path, "expected an object"))
}

fn array_at<'a>(value: &'a Value, path: &str) -> Result<&'a [Value], DiscoveryProviderError> {
    value
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| invalid(path, "expected an array"))
}

fn property_ci<'a>(
    object: &'a Map<String, Value>,
    name: &str,
    path: &str,
) -> Result<Option<&'a Value>, DiscoveryProviderError> {
    let matches = object
        .iter()
        .filter(|(key, _)| key.eq_ignore_ascii_case(name))
        .collect::<Vec<_>>();
    if matches.len() > 1 {
        return Err(invalid(
            format!("{path}.{name}"),
            "the property is repeated with different casing",
        ));
    }
    Ok(matches.first().map(|(_, value)| *value))
}

fn required_text(
    object: &Map<String, Value>,
    name: &str,
    path: &str,
    maximum_bytes: usize,
) -> Result<String, DiscoveryProviderError> {
    let text = required_text_allow_empty(object, name, path, maximum_bytes)?;
    if text.trim().is_empty() {
        return Err(invalid(format!("{path}.{name}"), "the string is empty"));
    }
    Ok(text)
}

fn required_text_allow_empty(
    object: &Map<String, Value>,
    name: &str,
    path: &str,
    maximum_bytes: usize,
) -> Result<String, DiscoveryProviderError> {
    let value = property_ci(object, name, path)?
        .ok_or_else(|| invalid(format!("{path}.{name}"), "the property is missing"))?;
    let text = value
        .as_str()
        .ok_or_else(|| invalid(format!("{path}.{name}"), "expected a string"))?;
    if text.len() > maximum_bytes {
        return Err(invalid(
            format!("{path}.{name}"),
            format!("the string exceeds the {maximum_bytes}-byte limit"),
        ));
    }
    Ok(text.to_string())
}

fn optional_text(
    object: &Map<String, Value>,
    name: &str,
    path: &str,
    maximum_bytes: usize,
) -> Result<Option<String>, DiscoveryProviderError> {
    let Some(value) = property_ci(object, name, path)? else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    required_text_allow_empty(object, name, path, maximum_bytes).map(Some)
}

fn optional_text_allow_empty(
    object: &Map<String, Value>,
    name: &str,
    path: &str,
    maximum_bytes: usize,
) -> Result<Option<String>, DiscoveryProviderError> {
    let Some(value) = property_ci(object, name, path)? else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    required_text_allow_empty(object, name, path, maximum_bytes).map(Some)
}

fn required_i32(
    object: &Map<String, Value>,
    name: &str,
    path: &str,
) -> Result<i32, DiscoveryProviderError> {
    let value = property_ci(object, name, path)?
        .ok_or_else(|| invalid(format!("{path}.{name}"), "the property is missing"))?;
    let value = value
        .as_i64()
        .and_then(|value| i32::try_from(value).ok())
        .ok_or_else(|| invalid(format!("{path}.{name}"), "expected a 32-bit integer"))?;
    Ok(value)
}

fn optional_i32(
    object: &Map<String, Value>,
    name: &str,
    path: &str,
) -> Result<Option<i32>, DiscoveryProviderError> {
    let Some(value) = property_ci(object, name, path)? else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    required_i32(object, name, path).map(Some)
}

fn optional_bool(
    object: &Map<String, Value>,
    name: &str,
    path: &str,
) -> Result<Option<bool>, DiscoveryProviderError> {
    let Some(value) = property_ci(object, name, path)? else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    value
        .as_bool()
        .map(Some)
        .ok_or_else(|| invalid(format!("{path}.{name}"), "expected a boolean"))
}

fn optional_item_count(
    object: &Map<String, Value>,
    name: &str,
    path: &str,
) -> Result<Option<usize>, DiscoveryProviderError> {
    let Some(value) = optional_i32(object, name, path)? else {
        return Ok(None);
    };
    let value = usize::try_from(value)
        .ok()
        .filter(|value| *value <= DISCOVERY_LISTS_MAX_ITEMS_PER_LIST)
        .ok_or_else(|| {
            invalid(
                format!("{path}.{name}"),
                format!(
                    "expected a value from 0 through {}",
                    DISCOVERY_LISTS_MAX_ITEMS_PER_LIST
                ),
            )
        })?;
    Ok(Some(value))
}

fn optional_array<'a>(
    object: &'a Map<String, Value>,
    name: &str,
    path: &str,
) -> Result<Option<&'a [Value]>, DiscoveryProviderError> {
    let Some(value) = property_ci(object, name, path)? else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    array_at(value, &format!("{path}.{name}")).map(Some)
}

fn invalid(path: impl Into<String>, message: impl Into<String>) -> DiscoveryProviderError {
    DiscoveryProviderError::Invalid {
        path: path.into(),
        message: message.into(),
    }
}

pub fn embedded_discovery_catalog_fixture() -> DiscoveryCatalog {
    parse_discovery_catalog(
        br#"{
          "DATA": [
            {
              "Id": 7301,
              "Title": "Arcade Favorites",
              "Subtitle": "Provider manual list",
              "ListType": "Games",
              "SortBy": "Title",
              "SortAsc": true,
              "PriorityRank": 2,
              "MinimumItems": 1,
              "MaximumItems": 25,
              "Games": [
                { "Id": 1234, "Platform": "Fixture Console", "Title": "Fixture Adventure" }
              ],
              "Criteria": []
            },
            {
              "Id": 7302,
              "Title": "Top Rated Fixture Games",
              "Subtitle": "Provider automatic list",
              "ListType": "Games",
              "SortBy": "StarRating",
              "SortAsc": false,
              "PriorityRank": null,
              "MinimumItems": 1,
              "MaximumItems": 25,
              "Games": [],
              "Criteria": [
                { "Field": "Platform", "Comparison": "EqualTo", "Value": "Fixture Console" },
                { "Field": "StarRating", "Comparison": "GreaterThan", "Value": "3" }
              ]
            }
          ]
        }"#,
    )
    .expect("the compiled Discovery catalog fixture must remain valid")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    struct MemoryTransport {
        bytes: Vec<u8>,
    }

    impl DiscoveryCatalogTransport for MemoryTransport {
        fn fetch(&self, url: &str, maximum_bytes: u64) -> Result<Vec<u8>, DiscoveryProviderError> {
            assert_eq!(url, DISCOVERY_LISTS_ENDPOINT);
            assert_eq!(maximum_bytes, DISCOVERY_LISTS_MAX_RESPONSE_BYTES);
            Ok(self.bytes.clone())
        }
    }

    #[test]
    fn parses_the_recovered_case_insensitive_response_contract() {
        let bytes = br#"{
          "DATA": [{
            "id": 17,
            "TITLE": "Recently Added",
            "subtitle": null,
            "listTYPE": "Games",
            "sortby": "DateAdded",
            "SORTASC": false,
            "priorityrank": 1,
            "minimumitems": 5,
            "maximumitems": 25,
            "GAMES": [{"ID": 42, "PLATFORM": "Arcade", "TITLE": "Robotron"}],
            "CRITERIA": [{"FIELD": "DateAdded", "COMPARISON": "RecentDays", "VALUE": "360"}]
          }]
        }"#;
        let catalog = fetch_discovery_catalog(&MemoryTransport {
            bytes: bytes.into(),
        })
        .unwrap();
        assert_eq!(
            catalog,
            DiscoveryCatalog {
                lists: vec![DiscoveryList {
                    id: 17,
                    title: "Recently Added".into(),
                    subtitle: None,
                    list_type: Some("Games".into()),
                    sort_by: Some("DateAdded".into()),
                    sort_ascending: Some(false),
                    priority_rank: Some(1),
                    minimum_items: Some(5),
                    maximum_items: Some(25),
                    games: Some(vec![DiscoveryGame {
                        database_id: 42,
                        platform: "Arcade".into(),
                        title: "Robotron".into(),
                    }]),
                    criteria: Some(vec![DiscoveryCriterion {
                        field: "DateAdded".into(),
                        comparison: "RecentDays".into(),
                        value: "360".into(),
                    }]),
                }],
            }
        );
    }

    #[test]
    fn rejects_duplicate_ids_ambiguous_case_and_invalid_bounds() {
        let duplicate_ids = br#"{"data":[
          {"id":1,"title":"One","games":[],"criteria":[]},
          {"id":1,"title":"Two","games":[],"criteria":[]}
        ]}"#;
        assert!(parse_discovery_catalog(duplicate_ids)
            .unwrap_err()
            .to_string()
            .contains("duplicate ID"));

        let ambiguous_case = br#"{"data":[],"DATA":[]}"#;
        assert!(parse_discovery_catalog(ambiguous_case)
            .unwrap_err()
            .to_string()
            .contains("different casing"));

        let invalid_bounds = br#"{"data":[{
          "id":1,"title":"Bounds","minimumItems":26,"maximumItems":25,
          "games":[],"criteria":[]
        }]}"#;
        assert!(parse_discovery_catalog(invalid_bounds)
            .unwrap_err()
            .to_string()
            .contains("exceeds maximumItems"));
    }

    #[test]
    fn accepts_the_recovered_nullable_root_and_criterion_value_contract() {
        assert_eq!(
            parse_discovery_catalog(br#"{"data":null}"#).unwrap(),
            DiscoveryCatalog { lists: Vec::new() }
        );
        assert_eq!(
            parse_discovery_catalog(br#"{}"#).unwrap(),
            DiscoveryCatalog { lists: Vec::new() }
        );
        let catalog = parse_discovery_catalog(
            br#"{"data":[{
              "id":7,"title":"Boolean","games":[{"id":-1,"platform":"A","title":"B"}],
              "criteria":[{"field":"Favorite","comparison":"IsTrue","value":null}]
            }]}"#,
        )
        .unwrap();
        assert_eq!(catalog.lists[0].games.as_ref().unwrap()[0].database_id, -1);
        assert_eq!(catalog.lists[0].criteria.as_ref().unwrap()[0].value, "");
    }

    #[test]
    fn rejects_oversized_documents_and_nested_collections() {
        assert!(matches!(
            parse_discovery_catalog(&vec![
                b' ';
                usize::try_from(DISCOVERY_LISTS_MAX_RESPONSE_BYTES)
                    .unwrap()
                    + 1
            ]),
            Err(DiscoveryProviderError::TooLarge { .. })
        ));

        let games = (0..=DISCOVERY_LISTS_MAX_GAMES_PER_LIST)
            .map(|index| {
                format!(
                    r#"{{"id":{},"platform":"Arcade","title":"Game"}}"#,
                    index + 1
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let document = format!(
            r#"{{"data":[{{"id":1,"title":"Too Many","games":[{games}],"criteria":[]}}]}}"#
        );
        assert!(parse_discovery_catalog(document.as_bytes())
            .unwrap_err()
            .to_string()
            .contains("above the 1000-game limit"));
    }

    #[test]
    fn official_transport_refuses_every_other_url_before_network_io() {
        assert!(matches!(
            LaunchBoxDiscoveryTransport.fetch("https://example.com/data", 10),
            Err(DiscoveryProviderError::UntrustedUrl { .. })
        ));
    }
}
