use crate::Game;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuditColumnKind {
    Text,
    Number,
    Boolean,
    DateTime,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuditColumn {
    pub key: &'static str,
    pub label: &'static str,
    pub kind: AuditColumnKind,
    pub width: u16,
}

macro_rules! text {
    ($key:literal, $label:literal, $width:literal) => {
        AuditColumn {
            key: $key,
            label: $label,
            kind: AuditColumnKind::Text,
            width: $width,
        }
    };
}

macro_rules! number {
    ($key:literal, $label:literal, $width:literal) => {
        AuditColumn {
            key: $key,
            label: $label,
            kind: AuditColumnKind::Number,
            width: $width,
        }
    };
}

macro_rules! boolean {
    ($key:literal, $label:literal) => {
        AuditColumn {
            key: $key,
            label: $label,
            kind: AuditColumnKind::Boolean,
            width: 92,
        }
    };
}

macro_rules! date {
    ($key:literal, $label:literal) => {
        AuditColumn {
            key: $key,
            label: $label,
            kind: AuditColumnKind::DateTime,
            width: 170,
        }
    };
}

/// The 76 visible `AuditEntry` properties structurally recovered from
/// LaunchBox 13.27. The internal `Game` reference and four `*SortValue`
/// properties are deliberately excluded.
pub const LAUNCHBOX_AUDIT_COLUMNS: [AuditColumn; 76] = [
    number!("AdditionalApps", "Additional Applications", 150),
    boolean!("Alternate", "Alternate"),
    text!("AlternateNames", "Alternate Names", 220),
    text!("ApplicationPath", "Application Path", 300),
    boolean!("BadDump", "Bad Dump"),
    boolean!("Broken", "Broken"),
    text!("Progress", "Progress", 120),
    date!("DateAdded", "Date Added"),
    date!("DateModified", "Date Modified"),
    text!("Developer", "Developer", 180),
    number!("ArcadeCabinetImageCount", "Arcade Cabinet Images", 170),
    number!(
        "ArcadeCircuitBoardImageCount",
        "Arcade Circuit Board Images",
        190
    ),
    number!(
        "ArcadeControlPanelImageCount",
        "Arcade Control Panel Images",
        190
    ),
    number!(
        "ArcadeControlsInformationImageCount",
        "Arcade Controls Information",
        190
    ),
    number!("BannerImageCount", "Banner Images", 120),
    number!("BackgroundImageCount", "Background Images", 140),
    number!("Box3dImageCount", "3D Box Images", 120),
    number!("BoxBackImageCount", "Box Back Images", 130),
    number!("BoxFrontImageCount", "Box Front Images", 130),
    number!("BoxSpineImageCount", "Box Spine Images", 130),
    number!("Cart3dImageCount", "3D Cart Images", 120),
    number!("CartBackImageCount", "Cart Back Images", 130),
    number!("CartFrontImageCount", "Cart Front Images", 130),
    number!("ClearLogoImageCount", "Clear Logo Images", 140),
    text!("CloneOf", "Clone Of", 160),
    boolean!("Duplicate", "Duplicate"),
    boolean!("Fixed", "Fixed"),
    text!("Genres", "Genres", 180),
    boolean!("Hidden", "Hidden"),
    text!("Id", "ID", 290),
    boolean!("Installed", "Installed"),
    boolean!("IsBootleg", "Bootleg"),
    boolean!("IsCasino", "Casino"),
    text!("IsEpicGames", "Epic Games", 140),
    boolean!("IsFruit", "Fruit Machine"),
    text!("IsGog", "GOG", 140),
    boolean!("IsHack", "Hack"),
    boolean!("IsMahjong", "Mahjong"),
    boolean!("IsMature", "Mature"),
    boolean!("IsMechanical", "Mechanical"),
    boolean!("IsNonArcade", "Non-Arcade"),
    text!("IsOrigin", "Origin", 140),
    boolean!("IsPlayChoice", "PlayChoice"),
    boolean!("IsPrototype", "Prototype"),
    boolean!("IsQuiz", "Quiz"),
    boolean!("IsRunnable", "Runnable"),
    boolean!("IsRythm", "Rhythm"),
    text!("IsSteam", "Steam", 140),
    boolean!("IsTabletop", "Tabletop"),
    text!("IsUplay", "Ubisoft Connect", 150),
    date!("LastPlayed", "Last Played"),
    text!("LaunchboxDatabaseId", "LaunchBox Database ID", 170),
    text!("ManualPath", "Manual Path", 280),
    number!("MarqueeImageCount", "Marquee Images", 140),
    number!("MaxPlayers", "Max Players", 110),
    text!("MusicPath", "Music Path", 280),
    text!("Notes", "Notes", 320),
    boolean!("Overdump", "Overdump"),
    text!("Platform", "Platform", 180),
    text!("PlayMode", "Play Mode", 130),
    text!("Publisher", "Publisher", 180),
    text!("Region", "Region", 120),
    date!("ReleaseDate", "Release Date"),
    text!("ReleaseType", "Release Type", 130),
    number!("ScreenshotImageCount", "Screenshot Images", 150),
    text!("Series", "Series", 180),
    text!("Status", "Status", 130),
    text!("Title", "Title", 260),
    boolean!("Trainer", "Trainer"),
    boolean!("Translation", "Translation"),
    boolean!("Unlicensed", "Unlicensed"),
    boolean!("Verified", "Verified"),
    text!("Version", "Version", 120),
    number!("VideoCount", "Videos", 90),
    text!("VideoUrl", "Video URL", 280),
    text!("WikipediaUrl", "Wikipedia URL", 280),
];

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AuditMediaCounts {
    pub arcade_cabinet: usize,
    pub arcade_circuit_board: usize,
    pub arcade_control_panel: usize,
    pub arcade_controls_information: usize,
    pub banner: usize,
    pub background: usize,
    pub box_3d: usize,
    pub box_back: usize,
    pub box_front: usize,
    pub box_spine: usize,
    pub cart_3d: usize,
    pub cart_back: usize,
    pub cart_front: usize,
    pub clear_logo: usize,
    pub marquee: usize,
    pub screenshot: usize,
    pub video: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AuditSupplement {
    pub additional_application_count: usize,
    pub alternate_names: Vec<String>,
    pub media: AuditMediaCounts,
}

pub fn audit_column_index(key: &str) -> Option<usize> {
    LAUNCHBOX_AUDIT_COLUMNS
        .iter()
        .position(|column| column.key == key)
}

/// LaunchBox 13.27's embedded release notes identify repeated, non-zero
/// LaunchBox Games Database IDs as the audit duplicate criterion. Missing and
/// zero IDs are not identities and therefore never form a duplicate group.
pub fn duplicate_game_ids(games: &[Game]) -> BTreeSet<String> {
    let mut grouped: BTreeMap<u32, Vec<&str>> = BTreeMap::new();
    for game in games {
        if let Some(database_id) = game.database_id.filter(|database_id| *database_id != 0) {
            grouped.entry(database_id).or_default().push(&game.id);
        }
    }
    grouped
        .into_values()
        .filter(|ids| ids.len() > 1)
        .flatten()
        .map(str::to_string)
        .collect()
}

pub fn audit_cell(
    game: &Game,
    supplement: &AuditSupplement,
    duplicate: bool,
    column_key: &str,
) -> String {
    match column_key {
        "AdditionalApps" => supplement.additional_application_count.to_string(),
        // These MAME-derived flags are not persisted on a LaunchBox Game
        // record. Empty means unavailable, never false.
        "Alternate" | "BadDump" | "Fixed" | "IsBootleg" | "IsCasino" | "IsFruit" | "IsHack"
        | "IsMahjong" | "IsMature" | "IsMechanical" | "IsNonArcade" | "IsPlayChoice"
        | "IsPrototype" | "IsQuiz" | "IsRythm" | "IsTabletop" | "Overdump" | "Trainer"
        | "Translation" | "Unlicensed" | "Verified" => String::new(),
        "AlternateNames" => supplement.alternate_names.join("; "),
        "ApplicationPath" => game.application_path.clone(),
        "Broken" => bool_text(game.broken),
        "Progress" => option_text(&game.progress),
        "DateAdded" => game.date_added.clone(),
        "DateModified" => game.date_modified.clone(),
        "Developer" => option_text(&game.developer),
        "ArcadeCabinetImageCount" => supplement.media.arcade_cabinet.to_string(),
        "ArcadeCircuitBoardImageCount" => supplement.media.arcade_circuit_board.to_string(),
        "ArcadeControlPanelImageCount" => supplement.media.arcade_control_panel.to_string(),
        "ArcadeControlsInformationImageCount" => {
            supplement.media.arcade_controls_information.to_string()
        }
        "BannerImageCount" => supplement.media.banner.to_string(),
        "BackgroundImageCount" => supplement.media.background.to_string(),
        "Box3dImageCount" => supplement.media.box_3d.to_string(),
        "BoxBackImageCount" => supplement.media.box_back.to_string(),
        "BoxFrontImageCount" => supplement.media.box_front.to_string(),
        "BoxSpineImageCount" => supplement.media.box_spine.to_string(),
        "Cart3dImageCount" => supplement.media.cart_3d.to_string(),
        "CartBackImageCount" => supplement.media.cart_back.to_string(),
        "CartFrontImageCount" => supplement.media.cart_front.to_string(),
        "ClearLogoImageCount" => supplement.media.clear_logo.to_string(),
        "CloneOf" => option_text(&game.clone_of),
        "Duplicate" => bool_text(duplicate),
        "Genres" => option_text(&game.genre),
        "Hidden" => bool_text(game.hidden),
        "Id" => game.id.clone(),
        "Installed" => game.installed.map(bool_text).unwrap_or_default(),
        "IsEpicGames" | "IsSteam" | "IsUplay" => String::new(),
        "IsGog" => option_text(&game.gog_app_id),
        "IsOrigin" => option_text(&game.origin_app_id),
        "IsRunnable" => bool_text(
            !game.application_path.trim().is_empty() || game.use_dos_box || game.use_scumm_vm,
        ),
        "LastPlayed" => option_text(&game.last_played_date),
        "LaunchboxDatabaseId" => game
            .database_id
            .map(|value| value.to_string())
            .unwrap_or_default(),
        "ManualPath" => option_text(&game.manual_path),
        "MarqueeImageCount" => supplement.media.marquee.to_string(),
        "MaxPlayers" => game
            .max_players
            .map(|value| value.to_string())
            .unwrap_or_default(),
        "MusicPath" => option_text(&game.music_path),
        "Notes" => option_text(&game.notes),
        "Platform" => game.platform.clone(),
        "PlayMode" => option_text(&game.play_mode),
        "Publisher" => option_text(&game.publisher),
        "Region" => option_text(&game.region),
        "ReleaseDate" => option_text(&game.release_date),
        "ReleaseType" => option_text(&game.release_type),
        "ScreenshotImageCount" => supplement.media.screenshot.to_string(),
        "Series" => option_text(&game.series),
        "Status" => option_text(&game.status),
        "Title" => game.title.clone(),
        "Version" => option_text(&game.version),
        "VideoCount" => supplement.media.video.to_string(),
        "VideoUrl" => option_text(&game.video_url),
        "WikipediaUrl" => option_text(&game.wikipedia_url),
        _ => String::new(),
    }
}

pub fn audit_tsv(
    games: &[Game],
    supplements: &BTreeMap<String, AuditSupplement>,
    selected_ids: Option<&BTreeSet<String>>,
) -> String {
    let duplicate_ids = duplicate_game_ids(games);
    let selected = games
        .iter()
        .filter(|game| selected_ids.is_none_or(|selected_ids| selected_ids.contains(&game.id)));
    let mut lines = Vec::new();
    lines.push(
        LAUNCHBOX_AUDIT_COLUMNS
            .iter()
            .map(|column| column.label)
            .collect::<Vec<_>>()
            .join("\t"),
    );
    for game in selected {
        let empty = AuditSupplement::default();
        let supplement = supplements.get(&game.id).unwrap_or(&empty);
        lines.push(
            LAUNCHBOX_AUDIT_COLUMNS
                .iter()
                .map(|column| {
                    sanitize_tsv_cell(&audit_cell(
                        game,
                        supplement,
                        duplicate_ids.contains(&game.id),
                        column.key,
                    ))
                })
                .collect::<Vec<_>>()
                .join("\t"),
        );
    }
    lines.join("\n")
}

fn bool_text(value: bool) -> String {
    if value { "Yes" } else { "No" }.to_string()
}

fn option_text(value: &Option<String>) -> String {
    value.clone().unwrap_or_default()
}

fn sanitize_tsv_cell(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            '\t' | '\r' | '\n' => ' ',
            other => other,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game(id: &str, title: &str, platform: &str) -> Game {
        Game {
            id: id.into(),
            title: title.into(),
            platform: platform.into(),
            application_path: format!("Games/{title}.rom"),
            notes: Some("line one\nline two".into()),
            ..Game::default()
        }
    }

    #[test]
    fn frozen_contract_has_all_recovered_visible_properties() {
        assert_eq!(LAUNCHBOX_AUDIT_COLUMNS.len(), 76);
        assert_eq!(LAUNCHBOX_AUDIT_COLUMNS[0].key, "AdditionalApps");
        assert_eq!(audit_column_index("Duplicate"), Some(25));
        assert_eq!(LAUNCHBOX_AUDIT_COLUMNS[75].key, "WikipediaUrl");
        assert!(
            LAUNCHBOX_AUDIT_COLUMNS
                .iter()
                .map(|column| column.key)
                .collect::<BTreeSet<_>>()
                .len()
                == 76
        );
    }

    #[test]
    fn duplicate_policy_requires_a_repeated_nonzero_database_id() {
        let mut first = game("a", "First", "Fixture Console");
        first.database_id = Some(4242);
        let mut second = game("b", "Different Title", "Other Console");
        second.database_id = Some(4242);
        let mut zero = game("c", "First", "Fixture Console");
        zero.database_id = Some(0);
        let missing = game("d", "First", "Fixture Console");
        let games = vec![first, second, zero, missing];
        assert_eq!(
            duplicate_game_ids(&games),
            BTreeSet::from(["a".to_string(), "b".to_string()])
        );
    }

    #[test]
    fn projection_distinguishes_unavailable_flags_from_false_values() {
        let game = Game {
            id: "fixture".into(),
            title: "Fixture".into(),
            platform: "Arcade".into(),
            broken: false,
            installed: Some(false),
            application_path: "Games/fixture.zip".into(),
            ..Game::default()
        };
        let supplement = AuditSupplement {
            additional_application_count: 2,
            alternate_names: vec!["Fixture Alternate".into()],
            media: AuditMediaCounts {
                box_front: 3,
                video: 1,
                ..AuditMediaCounts::default()
            },
        };
        assert_eq!(audit_cell(&game, &supplement, false, "Broken"), "No");
        assert_eq!(audit_cell(&game, &supplement, false, "Installed"), "No");
        assert_eq!(audit_cell(&game, &supplement, false, "BadDump"), "");
        assert_eq!(audit_cell(&game, &supplement, false, "AdditionalApps"), "2");
        assert_eq!(
            audit_cell(&game, &supplement, false, "BoxFrontImageCount"),
            "3"
        );
    }

    #[test]
    fn tsv_export_has_headers_selected_rows_and_single_line_cells() {
        let games = vec![
            game("a", "First", "Fixture"),
            game("b", "Second", "Fixture"),
        ];
        let selected = BTreeSet::from(["b".to_string()]);
        let output = audit_tsv(&games, &BTreeMap::new(), Some(&selected));
        let lines = output.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("Additional Applications\tAlternate\t"));
        assert!(lines[1].contains("line one line two"));
        assert!(!lines[1].contains("First"));
        assert!(lines[1].contains("Second"));
    }
}
