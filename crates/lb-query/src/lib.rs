use lb_domain::Game;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GameFilter {
    pub text: String,
    pub platform: Option<String>,
    pub include_hidden: bool,
    pub include_broken: bool,
}

pub fn filter_games<'a>(games: &'a [Game], filter: &GameFilter) -> Vec<&'a Game> {
    filter_game_indices(games, filter)
        .into_iter()
        .map(|index| &games[index])
        .collect()
}

/// Returns stable indices into the caller-owned game slice. This is the form
/// used by the Qt model so filtering does not clone records or serialize a
/// whole-library JSON snapshot.
pub fn filter_game_indices(games: &[Game], filter: &GameFilter) -> Vec<usize> {
    let needle = filter.text.trim().to_lowercase();
    let platform = filter.platform.as_deref();

    let mut matches: Vec<_> = games
        .iter()
        .enumerate()
        .filter(|(_, game)| filter.include_hidden || !game.hidden)
        .filter(|(_, game)| filter.include_broken || !game.broken)
        .filter(|(_, game)| platform.is_none_or(|expected| game.platform == expected))
        .filter(|(_, game)| needle.is_empty() || searchable_metadata(game, &needle))
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
}
