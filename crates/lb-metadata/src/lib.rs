use rusqlite::{Connection, OpenFlags, Row};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq)]
pub struct MetadataGame {
    pub database_id: i64,
    pub name: String,
    pub compare_name: String,
    pub release_date: Option<String>,
    pub release_year: Option<i32>,
    pub overview: Option<String>,
    pub max_players: Option<i32>,
    pub release_type: Option<String>,
    pub cooperative: bool,
    pub video_url: Option<String>,
    pub community_rating: Option<f64>,
    pub wikipedia_url: Option<String>,
    pub platform: String,
    pub esrb: Option<String>,
    pub genres: String,
    pub developer: Option<String>,
    pub publisher: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataMatchKind {
    Exact,
    Partial,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MetadataSearchResult {
    pub kind: Option<MetadataMatchKind>,
    pub games: Vec<MetadataGame>,
}

#[derive(Debug)]
pub struct MetadataDatabase {
    path: PathBuf,
    connection: Connection,
}

impl MetadataDatabase {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, MetadataError> {
        let path = path.as_ref().to_path_buf();
        let connection = Connection::open_with_flags(
            &path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|source| MetadataError::Open {
            path: path.clone(),
            source,
        })?;
        validate_schema(&connection, &path)?;
        Ok(Self { path, connection })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn canonical_platform_name(&self, platform: &str) -> Result<Option<String>, MetadataError> {
        let platform = platform.trim();
        if platform.is_empty() {
            return Ok(None);
        }
        let mut official = self
            .connection
            .prepare(
                "SELECT Name FROM Platforms
                 WHERE Name = ?1 COLLATE NOCASE
                 ORDER BY Name
                 LIMIT 2",
            )
            .map_err(|source| self.query_error(source))?;
        let matches = official
            .query_map([platform], |row| row.get::<_, String>(0))
            .map_err(|source| self.query_error(source))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|source| self.query_error(source))?;
        if let Some(name) = exactly_one(matches) {
            return Ok(Some(name));
        }

        let mut alternate = self
            .connection
            .prepare(
                "SELECT Name FROM PlatformAlternateNames
                 WHERE Alternate = ?1 COLLATE NOCASE
                 ORDER BY Name
                 LIMIT 2",
            )
            .map_err(|source| self.query_error(source))?;
        let matches = alternate
            .query_map([platform], |row| row.get::<_, String>(0))
            .map_err(|source| self.query_error(source))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|source| self.query_error(source))?;
        Ok(exactly_one(matches))
    }

    pub fn search_exact(
        &self,
        platform: &str,
        query: &str,
        application_path: Option<&Path>,
    ) -> Result<Vec<MetadataGame>, MetadataError> {
        let Some(platform) = self.canonical_platform_name(platform)? else {
            return Ok(Vec::new());
        };
        let compare_value = comparison_value(query);
        if compare_value.is_empty() {
            return Ok(Vec::new());
        }

        let mut statement = self
            .connection
            .prepare(EXACT_SEARCH_SQL)
            .map_err(|source| self.query_error(source))?;
        let matches = statement
            .query_map((&platform, &compare_value), matched_game_from_row)
            .map_err(|source| self.query_error(source))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|source| self.query_error(source))?;
        Ok(prefer_qualifier_matches(matches, query, application_path))
    }

    pub fn search(
        &self,
        platform: &str,
        query: &str,
        application_path: Option<&Path>,
    ) -> Result<MetadataSearchResult, MetadataError> {
        let exact = self.search_exact(platform, query, application_path)?;
        if !exact.is_empty() {
            return Ok(MetadataSearchResult {
                kind: Some(MetadataMatchKind::Exact),
                games: exact,
            });
        }
        let partial = self.search_partial(platform, query, application_path)?;
        Ok(MetadataSearchResult {
            kind: (!partial.is_empty()).then_some(MetadataMatchKind::Partial),
            games: partial,
        })
    }

    fn search_partial(
        &self,
        platform: &str,
        query: &str,
        application_path: Option<&Path>,
    ) -> Result<Vec<MetadataGame>, MetadataError> {
        let Some(platform) = self.canonical_platform_name(platform)? else {
            return Ok(Vec::new());
        };
        let compare_value = comparison_value(query);
        if compare_value.is_empty() {
            return Ok(Vec::new());
        }

        let mut title_statement = self
            .connection
            .prepare(PARTIAL_SEARCH_TITLES_SQL)
            .map_err(|source| self.query_error(source))?;
        let title_matches = title_statement
            .query_map([&platform], |row| {
                Ok(MatchedTitle {
                    database_id: row.get(0)?,
                    matched_title: row.get(1)?,
                    compare_value: row.get(2)?,
                })
            })
            .map_err(|source| self.query_error(source))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|source| self.query_error(source))?
            .into_iter()
            .filter(|candidate| {
                candidate.compare_value != compare_value
                    && recovered_partial_match(&candidate.compare_value, &compare_value)
            })
            .collect::<Vec<_>>();
        let candidate_titles = title_matches
            .iter()
            .map(|candidate| (candidate.database_id, candidate.matched_title.as_str()))
            .collect::<Vec<_>>();
        let retained_ids = preferred_candidate_ids(&candidate_titles, query, application_path);

        let mut game_statement = self
            .connection
            .prepare(GAME_BY_ID_SQL)
            .map_err(|source| self.query_error(source))?;
        retained_ids
            .into_iter()
            .map(|database_id| {
                game_statement
                    .query_row([database_id], metadata_game_from_row)
                    .map_err(|source| self.query_error(source))
            })
            .collect()
    }

    fn query_error(&self, source: rusqlite::Error) -> MetadataError {
        MetadataError::Query {
            path: self.path.clone(),
            source,
        }
    }
}

#[derive(Clone, Debug)]
struct MatchedGame {
    game: MetadataGame,
    matched_title: String,
}

#[derive(Clone, Debug)]
struct MatchedTitle {
    database_id: i64,
    matched_title: String,
    compare_value: String,
}

const EXACT_SEARCH_SQL: &str = "
    SELECT
        g.DatabaseID, g.Name, g.CompareName, g.ReleaseDate, g.ReleaseYear,
        g.Overview, g.MaxPlayers, g.ReleaseType, g.Cooperative, g.VideoURL,
        g.CommunityRating, g.WikipediaURL, g.Platform, g.ESRB, g.Genres,
        g.Developer, g.Publisher, g.Name AS MatchedTitle
    FROM Games g
    WHERE g.Platform = ?1 COLLATE NOCASE
      AND g.CompareName = ?2
    UNION ALL
    SELECT
        g.DatabaseID, g.Name, g.CompareName, g.ReleaseDate, g.ReleaseYear,
        g.Overview, g.MaxPlayers, g.ReleaseType, g.Cooperative, g.VideoURL,
        g.CommunityRating, g.WikipediaURL, g.Platform, g.ESRB, g.Genres,
        g.Developer, g.Publisher, a.AlternateName AS MatchedTitle
    FROM GameAlternateTitles a
    JOIN Games g ON g.DatabaseID = a.DatabaseID
    WHERE g.Platform = ?1 COLLATE NOCASE
      AND a.AltNameCompareValue = ?2
    ORDER BY DatabaseID, MatchedTitle";

const PARTIAL_SEARCH_TITLES_SQL: &str = "
    SELECT g.DatabaseID, g.Name AS MatchedTitle, g.CompareName
    FROM Games g
    WHERE g.Platform = ?1 COLLATE NOCASE
    UNION ALL
    SELECT g.DatabaseID, a.AlternateName AS MatchedTitle, a.AltNameCompareValue
    FROM GameAlternateTitles a
    JOIN Games g ON g.DatabaseID = a.DatabaseID
    WHERE g.Platform = ?1 COLLATE NOCASE
    ORDER BY DatabaseID, MatchedTitle";

const GAME_BY_ID_SQL: &str = "
    SELECT
        g.DatabaseID, g.Name, g.CompareName, g.ReleaseDate, g.ReleaseYear,
        g.Overview, g.MaxPlayers, g.ReleaseType, g.Cooperative, g.VideoURL,
        g.CommunityRating, g.WikipediaURL, g.Platform, g.ESRB, g.Genres,
        g.Developer, g.Publisher
    FROM Games g
    WHERE g.DatabaseID = ?1";

fn matched_game_from_row(row: &Row<'_>) -> rusqlite::Result<MatchedGame> {
    Ok(MatchedGame {
        game: metadata_game_from_row(row)?,
        matched_title: row.get(17)?,
    })
}

fn metadata_game_from_row(row: &Row<'_>) -> rusqlite::Result<MetadataGame> {
    Ok(MetadataGame {
        database_id: row.get(0)?,
        name: row.get(1)?,
        compare_name: row.get(2)?,
        release_date: row.get(3)?,
        release_year: row.get(4)?,
        overview: row.get(5)?,
        max_players: row.get(6)?,
        release_type: row.get(7)?,
        cooperative: row.get(8)?,
        video_url: row.get(9)?,
        community_rating: row.get(10)?,
        wikipedia_url: row.get(11)?,
        platform: row.get(12)?,
        esrb: row.get(13)?,
        genres: row.get(14)?,
        developer: row.get(15)?,
        publisher: row.get(16)?,
    })
}

fn prefer_qualifier_matches(
    matches: Vec<MatchedGame>,
    query: &str,
    application_path: Option<&Path>,
) -> Vec<MetadataGame> {
    if matches.is_empty() {
        return Vec::new();
    }
    let candidate_titles = matches
        .iter()
        .map(|candidate| (candidate.game.database_id, candidate.matched_title.as_str()))
        .collect::<Vec<_>>();
    let retained_ids = preferred_candidate_ids(&candidate_titles, query, application_path);
    unique_games(matches, &retained_ids)
}

fn preferred_candidate_ids(
    candidates: &[(i64, &str)],
    query: &str,
    application_path: Option<&Path>,
) -> BTreeSet<i64> {
    let mut supplied_qualifiers = parenthetical_values(query);
    if let Some(application_title) = application_path
        .and_then(Path::file_stem)
        .and_then(|value| value.to_str())
    {
        supplied_qualifiers.extend(parenthetical_values(application_title));
    }

    let qualified_ids = candidates
        .iter()
        .filter(|(_, matched_title)| {
            parenthetical_values(matched_title)
                .iter()
                .any(|candidate_value| {
                    supplied_qualifiers
                        .iter()
                        .any(|supplied| candidate_value.eq_ignore_ascii_case(supplied))
                })
        })
        .map(|(database_id, _)| *database_id)
        .collect::<BTreeSet<_>>();
    if !qualified_ids.is_empty() {
        return qualified_ids;
    }

    let unqualified_ids = candidates
        .iter()
        .filter(|(_, matched_title)| parenthetical_values(matched_title).is_empty())
        .map(|(database_id, _)| *database_id)
        .collect::<BTreeSet<_>>();
    if !unqualified_ids.is_empty() {
        return unqualified_ids;
    }
    candidates
        .iter()
        .map(|(database_id, _)| *database_id)
        .collect()
}

fn unique_games(matches: Vec<MatchedGame>, retained_ids: &BTreeSet<i64>) -> Vec<MetadataGame> {
    matches
        .into_iter()
        .filter(|candidate| retained_ids.contains(&candidate.game.database_id))
        .map(|candidate| (candidate.game.database_id, candidate.game))
        .collect::<BTreeMap<_, _>>()
        .into_values()
        .collect()
}

fn recovered_partial_match(candidate: &str, query: &str) -> bool {
    if candidate.contains(query) && !is_bare_numbered_suffix(candidate, query) {
        return true;
    }
    let candidate_words = candidate.split(' ').collect::<BTreeSet<_>>();
    query
        .split(' ')
        .all(|query_word| candidate_words.contains(query_word))
}

fn is_bare_numbered_suffix(candidate: &str, query: &str) -> bool {
    candidate
        .strip_prefix(query)
        .is_some_and(|suffix| !suffix.is_empty() && suffix.chars().all(char::is_numeric))
}

fn parenthetical_values(value: &str) -> Vec<String> {
    delimited_values(value, '(', ')')
        .into_iter()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn delimited_values(value: &str, open: char, close: char) -> Vec<&str> {
    let mut values = Vec::new();
    let mut start = None;
    for (index, character) in value.char_indices() {
        if character == open {
            start = Some(index + character.len_utf8());
        } else if character == close {
            if let Some(start) = start.take() {
                values.push(&value[start..index]);
            }
        }
    }
    values
}

pub fn comparison_value(title: &str) -> String {
    if title.is_empty() {
        return String::new();
    }
    let mut title = remove_delimited(title, '(', ')');
    title = remove_delimited(&title, '[', ']');
    title = remove_delimited(&title, '{', '}');
    let trimmed = title.trim();
    if ends_with_ignore_ascii_case(trimmed, ", The") {
        title = format!("The {}", &trimmed[..trimmed.len() - 5]);
    } else if ends_with_ignore_ascii_case(trimmed, ", A") {
        title = format!("A {}", &trimmed[..trimmed.len() - 3]);
    } else if ends_with_ignore_ascii_case(trimmed, ", An") {
        title = format!("An {}", &trimmed[..trimmed.len() - 4]);
    }

    title = format!(" {title} ");
    for (from, to) in [
        (":", " "),
        (",", " "),
        (".", ""),
        ("-", " "),
        ("/", " "),
        ("\\", " "),
        ("'", ""),
        ("\"", " "),
        ("&", " "),
        ("!", " "),
        ("?", " "),
        (" II ", " 2 "),
        (" III ", " 3 "),
        (" IV ", " 4 "),
        (" V ", " 5 "),
        (" VI ", " 6 "),
        (" VII ", " 7 "),
        (" VIII ", " 8 "),
    ] {
        title = title.replace(from, to);
    }
    title = title.to_uppercase();
    for article in [" THE ", " AN ", " A ", " AND "] {
        title = title.replace(article, " ");
    }
    title.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn remove_delimited(value: &str, open: char, close: char) -> String {
    let mut output = String::with_capacity(value.len());
    let mut inside = false;
    for character in value.chars() {
        if character == open {
            inside = true;
        } else if character == close && inside {
            inside = false;
        } else if !inside {
            output.push(character);
        }
    }
    output
}

fn ends_with_ignore_ascii_case(value: &str, suffix: &str) -> bool {
    value
        .get(value.len().saturating_sub(suffix.len())..)
        .is_some_and(|ending| ending.eq_ignore_ascii_case(suffix))
}

fn exactly_one(mut values: Vec<String>) -> Option<String> {
    (values.len() == 1).then(|| values.remove(0))
}

fn validate_schema(connection: &Connection, path: &Path) -> Result<(), MetadataError> {
    for table in [
        "Games",
        "GameAlternateTitles",
        "Platforms",
        "PlatformAlternateNames",
    ] {
        let count = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [table],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|source| MetadataError::Query {
                path: path.to_path_buf(),
                source,
            })?;
        if count != 1 {
            return Err(MetadataError::MissingTable {
                path: path.to_path_buf(),
                table,
            });
        }
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum MetadataError {
    #[error("could not open LaunchBox metadata database {path}: {source}")]
    Open {
        path: PathBuf,
        #[source]
        source: rusqlite::Error,
    },
    #[error("LaunchBox metadata database {path} is missing table {table}")]
    MissingTable { path: PathBuf, table: &'static str },
    #[error("could not query LaunchBox metadata database {path}: {source}")]
    Query {
        path: PathBuf,
        #[source]
        source: rusqlite::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparison_normalization_matches_the_recovered_pipeline() {
        assert_eq!(
            comparison_value("Legend of Zelda II, The (USA) [!]"),
            "LEGEND OF ZELDA 2"
        );
        assert_eq!(
            comparison_value("Dungeons & Dragons III"),
            "DUNGEONS DRAGONS 3"
        );
        assert_eq!(comparison_value("Game, An"), "GAME");
    }

    #[test]
    fn exact_search_uses_platform_aliases_alternate_titles_and_region_qualifiers() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("LaunchBox.Metadata.db");
        create_fixture(&path);
        let database = MetadataDatabase::open(&path).unwrap();

        assert_eq!(
            database
                .canonical_platform_name("Fixture Alias")
                .unwrap()
                .as_deref(),
            Some("Fixture Console")
        );
        let result = database
            .search_exact(
                "Fixture Alias",
                "Fixture Quest (USA)",
                Some(Path::new("Fixture Quest (USA).rom")),
            )
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].database_id, 2);
        assert_eq!(result[0].release_year, Some(2002));

        let alternate = database
            .search_exact("Fixture Console", "Quest Alternate", None)
            .unwrap();
        assert_eq!(alternate.len(), 1);
        assert_eq!(alternate[0].database_id, 1);
    }

    #[test]
    fn ambiguous_exact_search_stays_explicit() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("LaunchBox.Metadata.db");
        create_fixture(&path);
        let database = MetadataDatabase::open(&path).unwrap();

        let result = database
            .search_exact("Fixture Console", "Fixture Quest", None)
            .unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(
            result
                .iter()
                .map(|game| game.database_id)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
    }

    #[test]
    fn search_uses_recovered_partial_fallback_only_after_exact_results() {
        assert!(!recovered_partial_match("VALOR2", "VALOR"));
        assert!(recovered_partial_match("VALOR 2", "VALOR"));

        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("LaunchBox.Metadata.db");
        create_fixture(&path);
        let database = MetadataDatabase::open(&path).unwrap();

        let exact = database
            .search("Fixture Console", "Fixture Quest", None)
            .unwrap();
        assert_eq!(exact.kind, Some(MetadataMatchKind::Exact));
        assert_eq!(
            exact
                .games
                .iter()
                .map(|game| game.database_id)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );

        let reordered_words = database
            .search("Fixture Console", "Valor Legends", None)
            .unwrap();
        assert_eq!(reordered_words.kind, Some(MetadataMatchKind::Partial));
        assert_eq!(reordered_words.games.len(), 1);
        assert_eq!(reordered_words.games[0].database_id, 3);

        let alternate_substring = database
            .search("Fixture Console", "Grand Chronicle", None)
            .unwrap();
        assert_eq!(alternate_substring.kind, Some(MetadataMatchKind::Partial));
        assert_eq!(alternate_substring.games.len(), 1);
        assert_eq!(alternate_substring.games[0].database_id, 5);

        let numbered_suffix = database
            .search(
                "Fixture Console",
                "Valor (USA)",
                Some(Path::new("Valor (USA).rom")),
            )
            .unwrap();
        assert_eq!(numbered_suffix.kind, Some(MetadataMatchKind::Partial));
        assert_eq!(
            numbered_suffix
                .games
                .iter()
                .map(|game| game.database_id)
                .collect::<Vec<_>>(),
            vec![3]
        );
    }

    fn create_fixture(path: &Path) {
        let connection = Connection::open(path).unwrap();
        connection
            .execute_batch(
                "
                CREATE TABLE Platforms (Name TEXT NOT NULL);
                CREATE TABLE PlatformAlternateNames (
                    Name TEXT NOT NULL,
                    Alternate TEXT NOT NULL
                );
                CREATE TABLE Games (
                    DatabaseID INTEGER PRIMARY KEY,
                    Name TEXT NOT NULL,
                    CompareName TEXT NOT NULL,
                    ReleaseDate TEXT,
                    ReleaseYear INTEGER,
                    Overview TEXT,
                    MaxPlayers INTEGER,
                    ReleaseType TEXT,
                    Cooperative INTEGER NOT NULL,
                    VideoURL TEXT,
                    CommunityRating REAL,
                    WikipediaURL TEXT,
                    Platform TEXT NOT NULL,
                    ESRB TEXT,
                    Genres TEXT NOT NULL,
                    Developer TEXT,
                    Publisher TEXT
                );
                CREATE TABLE GameAlternateTitles (
                    AlternateName TEXT NOT NULL,
                    DatabaseID INTEGER NOT NULL,
                    Region TEXT NOT NULL,
                    AltNameCompareValue TEXT NOT NULL
                );
                INSERT INTO Platforms VALUES ('Fixture Console');
                INSERT INTO PlatformAlternateNames VALUES
                    ('Fixture Console', 'Fixture Alias');
                INSERT INTO Games VALUES
                    (1, 'Fixture Quest (Japan)', 'FIXTURE QUEST', '2001-04-05',
                     2001, 'Japan overview', 1, 'Released', 0, NULL, 4.1,
                     NULL, 'Fixture Console', 'E', 'Role-Playing', 'Studio A',
                     'Publisher A'),
                    (2, 'Fixture Quest (USA)', 'FIXTURE QUEST', NULL,
                     2002, 'USA overview', 2, 'Released', 1, NULL, 4.2,
                     NULL, 'Fixture Console', 'E10+', 'Role-Playing', 'Studio B',
                     'Publisher B'),
                    (3, 'Legends of Valor (USA)', 'LEGENDS OF VALOR', NULL,
                     2003, 'Valor overview', 1, 'Released', 0, NULL, 4.3,
                     NULL, 'Fixture Console', 'E', 'Adventure', 'Studio C',
                     'Publisher C'),
                    (4, 'Valor2', 'VALOR2', NULL, 2004, 'Sequel overview', 1,
                     'Released', 0, NULL, 4.4, NULL, 'Fixture Console', 'E',
                     'Adventure', 'Studio D', 'Publisher D'),
                    (5, 'Chronicles of Courage', 'CHRONICLES OF COURAGE', NULL,
                     2005, 'Courage overview', 1, 'Released', 0, NULL, 4.5,
                     NULL, 'Fixture Console', 'E', 'Adventure', 'Studio E',
                     'Publisher E');
                INSERT INTO GameAlternateTitles VALUES
                    ('Quest Alternate', 1, 'Japan', 'QUEST ALTERNATE'),
                    ('Valor Grand Chronicle', 5, 'USA',
                     'VALOR GRAND CHRONICLE');
                ",
            )
            .unwrap();
    }
}
