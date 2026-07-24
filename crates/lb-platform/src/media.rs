use crate::path::LaunchPathResolver;
use lb_domain::{FrontendSettings, Game, PlatformFolder};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use unicode_normalization::UnicodeNormalization;

const MAX_IMAGE_TYPE_PRIORITIES: usize = 64;
const MAX_REGION_PRIORITIES: usize = 64;
const MAX_PLATFORM_FOLDERS: usize = 4_096;
const MAX_FILES_PER_FOLDER: usize = 100_000;
const MAX_IMAGE_BYTES: u64 = 64 * 1024 * 1024;

const FALLBACK_FRONT_IMAGE_TYPES: &[&str] = &[
    "Box - Front",
    "Box - Front - Reconstructed",
    "Fanart - Box - Front",
];

const SUPPORTED_IMAGE_EXTENSIONS: &[&str] = &[
    "bmp", "gif", "jpeg", "jpg", "png", "svg", "tif", "tiff", "webp",
];

/// One immutable, read-only index of the front artwork selected for each game.
///
/// Paths are native host paths resolved at the platform boundary. They are
/// keyed by stable game ID so filtering, sorting, and row insertion never make
/// artwork point at a different record.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FrontImageIndex {
    pub paths_by_game_id: BTreeMap<String, PathBuf>,
    pub report: FrontImageIndexReport,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FrontImageIndexReport {
    pub configured_folders: usize,
    pub truncated_configured_folders: usize,
    pub scanned_folders: usize,
    pub scanned_files: usize,
    pub matched_games: usize,
    pub unresolved_folders: usize,
    pub unsafe_entries: usize,
    pub oversized_files: usize,
    pub truncated_folders: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ImageCandidate {
    path: PathBuf,
    region: Option<String>,
    ordinal: u32,
}

/// Reproduces the filename-safe title used by LaunchBox game media.
///
/// The apostrophe replacement is intentional even though apostrophes are
/// legal Windows filename characters. It is present in the observed 13.24
/// media library alongside the normal Windows-invalid character replacements.
pub fn launchbox_media_stem(title: &str) -> String {
    title
        .chars()
        .map(|character| {
            if character.is_control()
                || matches!(
                    character,
                    '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' | '\''
                )
            {
                '_'
            } else {
                character
            }
        })
        .collect()
}

pub fn front_image_type_priorities(settings: Option<&FrontendSettings>) -> Vec<String> {
    let configured = settings
        .and_then(|settings| settings.get("FrontImageTypePriorities"))
        .map(split_priorities)
        .unwrap_or_default();
    if !configured.is_empty() {
        return configured;
    }

    if settings.is_some_and(|settings| {
        settings.image_type_settings.iter().any(|setting| {
            setting.is_default
                && setting
                    .image_type
                    .trim()
                    .eq_ignore_ascii_case("Box - Front")
        })
    }) {
        return vec!["Box - Front".to_string()];
    }

    FALLBACK_FRONT_IMAGE_TYPES
        .iter()
        .map(|value| (*value).to_string())
        .collect()
}

pub fn region_priorities(settings: Option<&FrontendSettings>) -> Vec<String> {
    settings
        .and_then(|settings| settings.get("RegionPriorities"))
        .map(split_priorities)
        .unwrap_or_default()
        .into_iter()
        .take(MAX_REGION_PRIORITIES)
        .collect()
}

/// Builds a bounded front-art index without creating, changing, or deleting
/// any media. Unmapped, unreadable, unsafe, or oversized folders/files are
/// isolated and reported rather than preventing the library from loading.
pub fn index_front_images(
    launchbox_root: &Path,
    games: &[Game],
    folders: &[PlatformFolder],
    settings: Option<&FrontendSettings>,
    path_resolver: &dyn LaunchPathResolver,
) -> FrontImageIndex {
    let image_types = front_image_type_priorities(settings);
    let region_priorities = region_priorities(settings);
    let mut report = FrontImageIndexReport::default();
    let folder_index = index_platform_folders(folders, &mut report);
    let games_by_platform = games_by_platform(games);
    let mut paths_by_game_id = BTreeMap::new();

    for (platform_key, platform_games) in games_by_platform {
        let mut remaining = platform_games
            .iter()
            .map(|game| game.id.as_str())
            .collect::<BTreeSet<_>>();
        for image_type in &image_types {
            if remaining.is_empty() {
                break;
            }
            let key = (platform_key.clone(), normalized_key(image_type));
            let Some(stored_path) = folder_index.get(&key) else {
                continue;
            };
            let Ok(folder) = path_resolver.resolve(launchbox_root, stored_path) else {
                report.unresolved_folders = report.unresolved_folders.saturating_add(1);
                continue;
            };
            let candidates = scan_image_folder(&folder, &mut report);
            if candidates.is_empty() {
                continue;
            }
            for game in &platform_games {
                if !remaining.contains(game.id.as_str()) {
                    continue;
                }
                let title_key = normalized_key(&launchbox_media_stem(&game.title));
                let Some(matches) = candidates.get(&title_key) else {
                    continue;
                };
                if let Some(selected) =
                    select_candidate(matches, game.region.as_deref(), &region_priorities)
                {
                    paths_by_game_id.insert(game.id.clone(), selected.path.clone());
                    remaining.remove(game.id.as_str());
                }
            }
        }
    }

    report.matched_games = paths_by_game_id.len();
    FrontImageIndex {
        paths_by_game_id,
        report,
    }
}

fn split_priorities(value: &str) -> Vec<String> {
    deduplicate_priorities(
        value
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .collect(),
        MAX_IMAGE_TYPE_PRIORITIES,
    )
}

fn deduplicate_priorities(values: Vec<String>, limit: usize) -> Vec<String> {
    let mut seen = BTreeSet::new();
    values
        .into_iter()
        .filter(|value| seen.insert(normalized_key(value)))
        .take(limit)
        .collect()
}

fn index_platform_folders(
    folders: &[PlatformFolder],
    report: &mut FrontImageIndexReport,
) -> BTreeMap<(String, String), String> {
    let mut index = BTreeMap::new();
    if folders.len() > MAX_PLATFORM_FOLDERS {
        report.truncated_configured_folders = folders.len() - MAX_PLATFORM_FOLDERS;
    }
    for folder in folders.iter().take(MAX_PLATFORM_FOLDERS) {
        if folder.platform.trim().is_empty()
            || folder.media_type.trim().is_empty()
            || folder.folder_path.trim().is_empty()
        {
            continue;
        }
        report.configured_folders = report.configured_folders.saturating_add(1);
        index
            .entry((
                normalized_key(&folder.platform),
                normalized_key(&folder.media_type),
            ))
            .or_insert_with(|| folder.folder_path.clone());
    }
    index
}

fn games_by_platform(games: &[Game]) -> BTreeMap<String, Vec<&Game>> {
    let mut by_platform = BTreeMap::<String, Vec<&Game>>::new();
    for game in games {
        by_platform
            .entry(normalized_key(&game.platform))
            .or_default()
            .push(game);
    }
    by_platform
}

fn scan_image_folder(
    folder: &Path,
    report: &mut FrontImageIndexReport,
) -> BTreeMap<String, Vec<ImageCandidate>> {
    let Ok(metadata) = fs::symlink_metadata(folder) else {
        report.unresolved_folders = report.unresolved_folders.saturating_add(1);
        return BTreeMap::new();
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        report.unsafe_entries = report.unsafe_entries.saturating_add(1);
        return BTreeMap::new();
    }
    report.scanned_folders = report.scanned_folders.saturating_add(1);

    let mut candidates = BTreeMap::<String, Vec<ImageCandidate>>::new();
    let Ok((first_level, first_level_truncated)) =
        sorted_directory_entries(folder, MAX_FILES_PER_FOLDER)
    else {
        report.unresolved_folders = report.unresolved_folders.saturating_add(1);
        return candidates;
    };
    let mut remaining_entries = MAX_FILES_PER_FOLDER;
    let mut truncated = first_level_truncated;
    for entry in first_level {
        if remaining_entries == 0 {
            truncated = true;
            break;
        }
        remaining_entries -= 1;
        let Ok(file_type) = entry.file_type() else {
            report.unsafe_entries = report.unsafe_entries.saturating_add(1);
            continue;
        };
        if file_type.is_symlink() {
            report.unsafe_entries = report.unsafe_entries.saturating_add(1);
            continue;
        }
        if file_type.is_file() {
            add_candidate(&mut candidates, entry.path(), None, report);
            continue;
        }
        if !file_type.is_dir() {
            report.unsafe_entries = report.unsafe_entries.saturating_add(1);
            continue;
        }
        let Some(region) = entry.file_name().to_str().map(str::to_string) else {
            report.unsafe_entries = report.unsafe_entries.saturating_add(1);
            continue;
        };
        let Ok((region_entries, region_truncated)) =
            sorted_directory_entries(&entry.path(), remaining_entries)
        else {
            report.unresolved_folders = report.unresolved_folders.saturating_add(1);
            continue;
        };
        truncated |= region_truncated;
        for region_entry in region_entries {
            if remaining_entries == 0 {
                truncated = true;
                break;
            }
            remaining_entries -= 1;
            let Ok(region_file_type) = region_entry.file_type() else {
                report.unsafe_entries = report.unsafe_entries.saturating_add(1);
                continue;
            };
            if !region_file_type.is_file() || region_file_type.is_symlink() {
                report.unsafe_entries = report.unsafe_entries.saturating_add(1);
                continue;
            }
            add_candidate(
                &mut candidates,
                region_entry.path(),
                Some(region.clone()),
                report,
            );
        }
    }
    if truncated {
        report.truncated_folders = report.truncated_folders.saturating_add(1);
    }
    candidates
}

fn sorted_directory_entries(
    folder: &Path,
    limit: usize,
) -> std::io::Result<(Vec<fs::DirEntry>, bool)> {
    let mut entries = fs::read_dir(folder)?
        .take(limit.saturating_add(1))
        .collect::<Result<Vec<_>, _>>()?;
    let truncated = entries.len() > limit;
    entries.truncate(limit);
    entries.sort_by_key(fs::DirEntry::file_name);
    Ok((entries, truncated))
}

fn add_candidate(
    candidates: &mut BTreeMap<String, Vec<ImageCandidate>>,
    path: PathBuf,
    region: Option<String>,
    report: &mut FrontImageIndexReport,
) {
    let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
        return;
    };
    if !SUPPORTED_IMAGE_EXTENSIONS
        .iter()
        .any(|supported| extension.eq_ignore_ascii_case(supported))
    {
        return;
    }
    let Ok(metadata) = fs::symlink_metadata(&path) else {
        report.unsafe_entries = report.unsafe_entries.saturating_add(1);
        return;
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        report.unsafe_entries = report.unsafe_entries.saturating_add(1);
        return;
    }
    if metadata.len() > MAX_IMAGE_BYTES {
        report.oversized_files = report.oversized_files.saturating_add(1);
        return;
    }
    let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
        report.unsafe_entries = report.unsafe_entries.saturating_add(1);
        return;
    };
    let (base, ordinal) = split_image_ordinal(stem);
    if base.is_empty() {
        return;
    }
    report.scanned_files = report.scanned_files.saturating_add(1);
    candidates
        .entry(normalized_key(base))
        .or_default()
        .push(ImageCandidate {
            path,
            region,
            ordinal,
        });
}

fn split_image_ordinal(stem: &str) -> (&str, u32) {
    let Some((base, suffix)) = stem.rsplit_once('-') else {
        return (stem, 0);
    };
    if suffix.is_empty() || !suffix.bytes().all(|byte| byte.is_ascii_digit()) {
        return (stem, 0);
    }
    suffix
        .parse()
        .ok()
        .map(|ordinal| (base, ordinal))
        .unwrap_or((stem, 0))
}

fn select_candidate<'a>(
    candidates: &'a [ImageCandidate],
    game_region: Option<&str>,
    region_priorities: &[String],
) -> Option<&'a ImageCandidate> {
    candidates.iter().min_by(|left, right| {
        candidate_region_rank(left, game_region, region_priorities)
            .cmp(&candidate_region_rank(
                right,
                game_region,
                region_priorities,
            ))
            .then_with(|| left.ordinal.cmp(&right.ordinal))
            .then_with(|| left.path.cmp(&right.path))
    })
}

fn candidate_region_rank(
    candidate: &ImageCandidate,
    game_region: Option<&str>,
    region_priorities: &[String],
) -> (u8, usize, String) {
    let candidate_regions = candidate
        .region
        .as_deref()
        .map(region_parts)
        .unwrap_or_default();
    if let Some(priority) = region_priorities.iter().position(|priority| {
        let priority = normalized_key(priority);
        candidate_regions.contains(&priority)
    }) {
        return (0, priority, String::new());
    }
    if game_region.is_some_and(|game_region| {
        let game_regions = region_parts(game_region);
        candidate_regions
            .iter()
            .any(|candidate| game_regions.contains(candidate))
    }) {
        return (1, 0, String::new());
    }
    if candidate.region.is_none() {
        return (2, 0, String::new());
    }
    if candidate_regions.contains(&normalized_key("World")) {
        return (3, 0, String::new());
    }
    (
        4,
        0,
        candidate
            .region
            .as_deref()
            .map(normalized_key)
            .unwrap_or_default(),
    )
}

fn region_parts(region: &str) -> BTreeSet<String> {
    region
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(normalized_key)
        .collect()
}

fn normalized_key(value: &str) -> String {
    value
        .nfkc()
        .flat_map(char::to_lowercase)
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HostPathResolver;
    use lb_domain::{ImageTypeSetting, SettingEntry};

    fn game(id: &str, title: &str, region: Option<&str>) -> Game {
        Game {
            id: id.into(),
            title: title.into(),
            platform: "Fixture Console".into(),
            region: region.map(str::to_string),
            application_path: "Games/game.rom".into(),
            ..Game::default()
        }
    }

    fn settings(front_types: &str, regions: &str) -> FrontendSettings {
        FrontendSettings {
            entries: vec![
                SettingEntry {
                    key: "FrontImageTypePriorities".into(),
                    value: front_types.into(),
                },
                SettingEntry {
                    key: "RegionPriorities".into(),
                    value: regions.into(),
                },
            ],
            ..FrontendSettings::default()
        }
    }

    #[test]
    fn media_stems_match_observed_launchbox_punctuation() {
        assert_eq!(
            launchbox_media_stem("Wonder Boy III: The Dragon's Trap"),
            "Wonder Boy III_ The Dragon_s Trap"
        );
        assert_eq!(launchbox_media_stem("Quest / \"CON\"?"), "Quest _ _CON__");
    }

    #[test]
    fn priorities_are_typed_bounded_and_deduplicated() {
        let settings = settings(
            "Steam Poster, Box - Front, steam poster, Fanart - Box - Front",
            "North America, United States",
        );
        assert_eq!(
            front_image_type_priorities(Some(&settings)),
            ["Steam Poster", "Box - Front", "Fanart - Box - Front"]
        );
        assert_eq!(
            region_priorities(Some(&settings)),
            ["North America", "United States"]
        );

        let default = FrontendSettings {
            image_type_settings: vec![ImageTypeSetting {
                image_type: "Box - Front".into(),
                is_default: true,
                use_in_auto_imports: true,
            }],
            ..FrontendSettings::default()
        };
        assert_eq!(front_image_type_priorities(Some(&default)), ["Box - Front"]);

        let unrelated_default = FrontendSettings {
            image_type_settings: vec![ImageTypeSetting {
                image_type: "Steam Poster".into(),
                is_default: true,
                use_in_auto_imports: true,
            }],
            ..FrontendSettings::default()
        };
        assert_eq!(
            front_image_type_priorities(Some(&unrelated_default)),
            FALLBACK_FRONT_IMAGE_TYPES
        );
    }

    #[test]
    fn indexes_native_art_by_type_region_ordinal_and_stable_game_id() {
        let directory = tempfile::tempdir().expect("temporary LaunchBox root");
        let steam = directory.path().join("Images/Fixture Console/Steam Poster");
        let boxes = directory.path().join("Images/Fixture Console/Box - Front");
        fs::create_dir_all(steam.join("Europe")).expect("Steam region");
        fs::create_dir_all(boxes.join("Europe")).expect("European boxes");
        fs::create_dir_all(boxes.join("North America")).expect("North American boxes");
        fs::write(
            steam.join("Europe/Wonder Boy III_ The Dragon_s Trap-02.png"),
            b"steam",
        )
        .expect("Steam art");
        fs::write(
            boxes.join("Europe/Wonder Boy III_ The Dragon_s Trap-01.jpg"),
            b"europe",
        )
        .expect("European box");
        fs::write(
            boxes.join("North America/Wonder Boy III_ The Dragon_s Trap-02.jpg"),
            b"north-america-two",
        )
        .expect("North American second box");
        fs::write(
            boxes.join("North America/Wonder Boy III_ The Dragon_s Trap-01.jpg"),
            b"north-america-one",
        )
        .expect("North American first box");

        let folders = [
            PlatformFolder {
                platform: "Fixture Console".into(),
                media_type: "Steam Poster".into(),
                folder_path: r"Images\Fixture Console\Steam Poster".into(),
            },
            PlatformFolder {
                platform: "Fixture Console".into(),
                media_type: "Box - Front".into(),
                folder_path: r"Images\Fixture Console\Box - Front".into(),
            },
        ];
        let games = [game(
            "stable-id",
            "Wonder Boy III: The Dragon's Trap",
            Some("North America"),
        )];
        let configured_settings = settings("Steam Poster,Box - Front", "North America,Europe");
        let index = index_front_images(
            directory.path(),
            &games,
            &folders,
            Some(&configured_settings),
            &HostPathResolver::default(),
        );
        assert_eq!(
            index.paths_by_game_id["stable-id"],
            steam.join("Europe/Wonder Boy III_ The Dragon_s Trap-02.png"),
            "image-type priority wins before region and ordinal priority"
        );
        assert_eq!(index.report.matched_games, 1);
        assert_eq!(index.report.scanned_folders, 1);
        assert_eq!(index.report.scanned_files, 1);

        let box_settings = settings("Box - Front", "North America,Europe");
        let boxes_only = index_front_images(
            directory.path(),
            &games,
            &folders,
            Some(&box_settings),
            &HostPathResolver::default(),
        );
        assert_eq!(
            boxes_only.paths_by_game_id["stable-id"],
            boxes.join("North America/Wonder Boy III_ The Dragon_s Trap-01.jpg")
        );
    }

    #[cfg(unix)]
    #[test]
    fn ignores_symlinks_deep_nesting_and_oversized_files() {
        use std::os::unix::fs::symlink;

        let directory = tempfile::tempdir().expect("temporary LaunchBox root");
        let boxes = directory.path().join("Images/Fixture Console/Box - Front");
        fs::create_dir_all(boxes.join("North America/Too Deep")).expect("nested region");
        fs::write(
            boxes.join("North America/Too Deep/Fixture-01.png"),
            b"nested",
        )
        .expect("nested art");
        fs::write(boxes.join("outside.png"), b"outside").expect("outside art");
        symlink(boxes.join("outside.png"), boxes.join("Fixture-01.png")).expect("file symlink");
        let oversized = fs::File::create(boxes.join("Fixture-02.png")).expect("oversized file");
        oversized
            .set_len(MAX_IMAGE_BYTES + 1)
            .expect("sparse oversized file");

        let folders = [PlatformFolder {
            platform: "Fixture Console".into(),
            media_type: "Box - Front".into(),
            folder_path: r"Images\Fixture Console\Box - Front".into(),
        }];
        let games = [game("fixture", "Fixture", Some("North America"))];
        let index = index_front_images(
            directory.path(),
            &games,
            &folders,
            None,
            &HostPathResolver::default(),
        );
        assert!(index.paths_by_game_id.is_empty());
        assert!(index.report.unsafe_entries >= 2);
        assert_eq!(index.report.oversized_files, 1);
    }

    #[cfg(not(windows))]
    #[test]
    fn indexes_front_art_through_an_explicit_windows_drive_mapping() {
        let mapped_drive = tempfile::tempdir().expect("mapped drive");
        let boxes = mapped_drive
            .path()
            .join("Images/Fixture Console/Box - Front");
        fs::create_dir_all(&boxes).expect("box folder");
        fs::write(boxes.join("Fixture-01.png"), b"fixture").expect("box art");
        let launchbox_root = tempfile::tempdir().expect("LaunchBox root");
        let resolver = HostPathResolver::default()
            .with_windows_drive_mapping('D', mapped_drive.path())
            .expect("drive mapping");
        let folders = [PlatformFolder {
            platform: "Fixture Console".into(),
            media_type: "Box - Front".into(),
            folder_path: r"D:\Images\Fixture Console\Box - Front".into(),
        }];

        let index = index_front_images(
            launchbox_root.path(),
            &[game("fixture", "Fixture", None)],
            &folders,
            None,
            &resolver,
        );
        assert_eq!(
            index.paths_by_game_id["fixture"],
            boxes.join("Fixture-01.png")
        );
    }
}
