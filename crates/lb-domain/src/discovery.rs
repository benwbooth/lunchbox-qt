/// LaunchBox 13.27's first-party endpoint for the Discovery playlist catalog.
///
/// The endpoint is part of the recovered catalog contract rather than the
/// transport implementation, so native projections can identify their source
/// without depending on an HTTP client or TLS provider.
pub const DISCOVERY_LISTS_ENDPOINT: &str =
    "https://api.gamesdb.launchbox-app.com/api/discovery-lists";

/// Maximum number of items accepted for one provider-defined Discovery list.
pub const DISCOVERY_LISTS_MAX_ITEMS_PER_LIST: usize = 1_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveryCatalog {
    pub lists: Vec<DiscoveryList>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveryList {
    pub id: i32,
    pub title: String,
    pub subtitle: Option<String>,
    pub list_type: Option<String>,
    pub sort_by: Option<String>,
    pub sort_ascending: Option<bool>,
    pub priority_rank: Option<i32>,
    pub minimum_items: Option<usize>,
    pub maximum_items: Option<usize>,
    pub games: Option<Vec<DiscoveryGame>>,
    pub criteria: Option<Vec<DiscoveryCriterion>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveryGame {
    pub database_id: i32,
    pub platform: String,
    pub title: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveryCriterion {
    pub field: String,
    pub comparison: String,
    pub value: String,
}
