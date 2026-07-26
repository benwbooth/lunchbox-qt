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
const MAX_VIDEO_BYTES: u64 = 8 * 1024 * 1024 * 1024;
const MAX_MEDIA_ITEMS: usize = 1_000_000;
const MAX_MEDIA_ITEMS_PER_GAME: usize = 512;

const FALLBACK_FRONT_IMAGE_TYPES: &[&str] = &[
    "Box - Front",
    "Box - Front - Reconstructed",
    "Fanart - Box - Front",
];

const FALLBACK_BACK_IMAGE_TYPES: &[&str] = &[
    "Box - Back",
    "Box - Back - Reconstructed",
    "Advertisement Flyer - Back",
    "Fanart - Box - Back",
];

const SUPPORTED_IMAGE_EXTENSIONS: &[&str] = &[
    "bmp", "gif", "jpeg", "jpg", "png", "svg", "tif", "tiff", "webp",
];

const FALLBACK_VIDEO_TYPES: &[&str] = &[
    "Theme Video",
    "Video Snap",
    "Recording",
    "Trailer",
    "Marquee",
];

const SUPPORTED_VIDEO_EXTENSIONS: &[&str] = &[
    "3g2", "3gp", "3gp2", "3gpp", "asf", "avi", "ifo", "m1v", "m2t", "m2ts", "m2v", "m4v", "mod",
    "mov", "mp4", "mp4v", "mpa", "mpe", "mpeg", "mpg", "mts", "ts", "tts", "vob", "webm", "wm",
    "wmd", "wmv",
];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum GameMediaKind {
    Image,
    Video,
}

impl GameMediaKind {
    pub const fn key(self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Video => "video",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameMediaItem {
    pub kind: GameMediaKind,
    pub media_type: String,
    pub path: PathBuf,
    pub region: Option<String>,
    pub ordinal: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameDetailsMediaPolicy {
    pub show_video: bool,
    pub auto_play_video: bool,
    pub video_type_priorities: Vec<String>,
}

impl Default for GameDetailsMediaPolicy {
    fn default() -> Self {
        Self {
            show_video: true,
            auto_play_video: true,
            video_type_priorities: FALLBACK_VIDEO_TYPES
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
        }
    }
}

impl GameDetailsMediaPolicy {
    pub fn from_settings(settings: Option<&FrontendSettings>) -> Self {
        let fallback = Self::default();
        let Some(settings) = settings else {
            return fallback;
        };
        Self {
            show_video: settings.get_bool("ShowDetailsVideo").unwrap_or(true),
            auto_play_video: settings.get_bool("AutoPlayDetailsVideo").unwrap_or(true),
            video_type_priorities: settings
                .get("VideoTypePriorities")
                .map(split_priorities)
                .filter(|values| !values.is_empty())
                .unwrap_or(fallback.video_type_priorities),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GameMediaIndex {
    pub items_by_game_id: BTreeMap<String, Vec<GameMediaItem>>,
    pub front_paths_by_game_id: BTreeMap<String, PathBuf>,
    pub back_paths_by_game_id: BTreeMap<String, PathBuf>,
    pub spine_paths_by_game_id: BTreeMap<String, PathBuf>,
    pub full_paths_by_game_id: BTreeMap<String, PathBuf>,
    pub policy: GameDetailsMediaPolicy,
    pub report: GameMediaIndexReport,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GameMediaIndexReport {
    pub configured_folders: usize,
    pub truncated_configured_folders: usize,
    pub scanned_folders: usize,
    pub scanned_files: usize,
    pub indexed_images: usize,
    pub indexed_videos: usize,
    pub matched_games: usize,
    pub unresolved_folders: usize,
    pub unresolved_files: usize,
    pub unsafe_entries: usize,
    pub oversized_files: usize,
    pub truncated_folders: usize,
    pub truncated_items: usize,
}

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

pub fn back_image_type_priorities(settings: Option<&FrontendSettings>) -> Vec<String> {
    settings
        .and_then(|settings| settings.get("BackImageTypePriorities"))
        .map(split_priorities)
        .filter(|values| !values.is_empty())
        .unwrap_or_else(|| {
            FALLBACK_BACK_IMAGE_TYPES
                .iter()
                .map(|value| (*value).to_string())
                .collect()
        })
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

/// Builds the read-only image/video collection consumed by LaunchBox's
/// selected-game media list.
///
/// The index retains native host paths only at this platform boundary. It
/// reads configured platform folders, explicit game video paths, region
/// folders, and LaunchBox's numbered filename convention without following
/// links or creating media. Images and videos stay keyed by stable game ID so
/// sorting and filtering cannot retarget a preview.
pub fn index_game_media(
    launchbox_root: &Path,
    games: &[Game],
    folders: &[PlatformFolder],
    settings: Option<&FrontendSettings>,
    path_resolver: &dyn LaunchPathResolver,
) -> GameMediaIndex {
    let policy = GameDetailsMediaPolicy::from_settings(settings);
    let front_priorities = front_image_type_priorities(settings);
    let back_priorities = back_image_type_priorities(settings);
    let region_priorities = region_priorities(settings);
    let games_by_platform = games_by_platform(games);
    let mut scan_report = FrontImageIndexReport::default();
    let mut report = GameMediaIndexReport::default();
    let mut items_by_game_id = BTreeMap::<String, Vec<GameMediaItem>>::new();
    let mut seen_by_game_id = BTreeMap::<String, BTreeSet<PathBuf>>::new();
    let mut scanned_paths = BTreeSet::<(GameMediaKind, PathBuf)>::new();
    let mut indexed_items = 0_usize;

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
        let media_kind = match normalized_key(&folder.media_type).as_str() {
            "manual" | "music" => continue,
            "video" | "theme video" => GameMediaKind::Video,
            _ => GameMediaKind::Image,
        };
        if media_kind == GameMediaKind::Video && !policy.show_video {
            continue;
        }
        report.configured_folders = report.configured_folders.saturating_add(1);
        let Ok(native_folder) = path_resolver.resolve(launchbox_root, &folder.folder_path) else {
            scan_report.unresolved_folders = scan_report.unresolved_folders.saturating_add(1);
            continue;
        };
        if media_kind == GameMediaKind::Image
            && !scanned_paths.insert((media_kind, native_folder.clone()))
        {
            continue;
        }
        if media_kind == GameMediaKind::Video
            && scanned_paths.contains(&(media_kind, native_folder.clone()))
        {
            continue;
        }
        let Some(platform_games) = games_by_platform.get(&normalized_key(&folder.platform)) else {
            continue;
        };
        match media_kind {
            GameMediaKind::Image => {
                let candidates = scan_image_folder(&native_folder, &mut scan_report);
                for game in platform_games {
                    let title_key = normalized_key(&launchbox_media_stem(&game.title));
                    let Some(matches) = candidates.get(&title_key) else {
                        continue;
                    };
                    let mut matches = matches.iter().collect::<Vec<_>>();
                    matches.sort_by(|left, right| {
                        candidate_region_rank(left, game.region.as_deref(), &region_priorities)
                            .cmp(&candidate_region_rank(
                                right,
                                game.region.as_deref(),
                                &region_priorities,
                            ))
                            .then_with(|| left.ordinal.cmp(&right.ordinal))
                            .then_with(|| left.path.cmp(&right.path))
                    });
                    for candidate in matches {
                        push_game_media_item(
                            game,
                            GameMediaItem {
                                kind: GameMediaKind::Image,
                                media_type: folder.media_type.clone(),
                                path: candidate.path.clone(),
                                region: candidate.region.clone(),
                                ordinal: candidate.ordinal,
                            },
                            &mut items_by_game_id,
                            &mut seen_by_game_id,
                            &mut indexed_items,
                            &mut report,
                        );
                    }
                }
            }
            GameMediaKind::Video => {
                let candidates = scan_video_folder(
                    &native_folder,
                    &folder.media_type,
                    &policy.video_type_priorities,
                    &mut scanned_paths,
                    &mut scan_report,
                );
                for game in platform_games {
                    let title_key = normalized_key(&launchbox_media_stem(&game.title));
                    let Some(matches) = candidates.get(&title_key) else {
                        continue;
                    };
                    let mut matches = matches.iter().collect::<Vec<_>>();
                    matches.sort_by(|left, right| {
                        video_type_rank(&left.media_type, &policy.video_type_priorities)
                            .cmp(&video_type_rank(
                                &right.media_type,
                                &policy.video_type_priorities,
                            ))
                            .then_with(|| {
                                media_region_rank(
                                    left.region.as_deref(),
                                    game.region.as_deref(),
                                    &region_priorities,
                                )
                                .cmp(&media_region_rank(
                                    right.region.as_deref(),
                                    game.region.as_deref(),
                                    &region_priorities,
                                ))
                            })
                            .then_with(|| left.ordinal.cmp(&right.ordinal))
                            .then_with(|| left.path.cmp(&right.path))
                    });
                    for candidate in matches {
                        push_game_media_item(
                            game,
                            GameMediaItem {
                                kind: GameMediaKind::Video,
                                media_type: candidate.media_type.clone(),
                                path: candidate.path.clone(),
                                region: candidate.region.clone(),
                                ordinal: candidate.ordinal,
                            },
                            &mut items_by_game_id,
                            &mut seen_by_game_id,
                            &mut indexed_items,
                            &mut report,
                        );
                    }
                }
            }
        }
    }

    if policy.show_video {
        for game in games {
            for (stored_path, media_type) in [
                (game.theme_video_path.as_deref(), "Theme Video"),
                (game.video_path.as_deref(), "Video Snap"),
            ] {
                let Some(stored_path) = stored_path.filter(|value| !value.trim().is_empty()) else {
                    continue;
                };
                let Ok(path) = path_resolver.resolve(launchbox_root, stored_path) else {
                    report.unresolved_files = report.unresolved_files.saturating_add(1);
                    continue;
                };
                if !validate_media_file(
                    &path,
                    SUPPORTED_VIDEO_EXTENSIONS,
                    MAX_VIDEO_BYTES,
                    &mut scan_report,
                ) {
                    report.unresolved_files = report.unresolved_files.saturating_add(1);
                    continue;
                }
                push_game_media_item(
                    game,
                    GameMediaItem {
                        kind: GameMediaKind::Video,
                        media_type: media_type.to_string(),
                        path,
                        region: game.region.clone(),
                        ordinal: 0,
                    },
                    &mut items_by_game_id,
                    &mut seen_by_game_id,
                    &mut indexed_items,
                    &mut report,
                );
            }
        }
    }

    for items in items_by_game_id.values_mut() {
        items.sort_by(|left, right| {
            let left_kind = match left.kind {
                GameMediaKind::Image => 0,
                GameMediaKind::Video => 1,
            };
            let right_kind = match right.kind {
                GameMediaKind::Image => 0,
                GameMediaKind::Video => 1,
            };
            let left_type = if left.kind == GameMediaKind::Image {
                media_type_rank(&left.media_type, &front_priorities)
            } else {
                media_type_rank(&left.media_type, &policy.video_type_priorities)
            };
            let right_type = if right.kind == GameMediaKind::Image {
                media_type_rank(&right.media_type, &front_priorities)
            } else {
                media_type_rank(&right.media_type, &policy.video_type_priorities)
            };
            left_kind
                .cmp(&right_kind)
                .then_with(|| left_type.cmp(&right_type))
                .then_with(|| left.ordinal.cmp(&right.ordinal))
                .then_with(|| left.path.cmp(&right.path))
        });
    }

    let games_by_id = games
        .iter()
        .map(|game| (game.id.as_str(), game))
        .collect::<BTreeMap<_, _>>();
    let mut front_paths_by_game_id = BTreeMap::new();
    let mut back_paths_by_game_id = BTreeMap::new();
    let mut spine_paths_by_game_id = BTreeMap::new();
    let mut full_paths_by_game_id = BTreeMap::new();
    let spine_priorities = ["Box - Spine".to_string()];
    let full_priorities = ["Box - Full".to_string()];
    for (game_id, items) in &items_by_game_id {
        let Some(game) = games_by_id.get(game_id.as_str()) else {
            continue;
        };
        if let Some(selected) =
            select_game_image(items, game, &front_priorities, &region_priorities)
        {
            front_paths_by_game_id.insert(game_id.clone(), selected.path.clone());
        }
        if let Some(selected) = select_game_image(items, game, &back_priorities, &region_priorities)
        {
            back_paths_by_game_id.insert(game_id.clone(), selected.path.clone());
        }
        if let Some(selected) =
            select_game_image(items, game, &spine_priorities, &region_priorities)
        {
            spine_paths_by_game_id.insert(game_id.clone(), selected.path.clone());
        }
        if let Some(selected) = select_game_image(items, game, &full_priorities, &region_priorities)
        {
            full_paths_by_game_id.insert(game_id.clone(), selected.path.clone());
        }
    }

    report.scanned_folders = scan_report.scanned_folders;
    report.scanned_files = scan_report.scanned_files;
    report.unresolved_folders = scan_report.unresolved_folders;
    report.unsafe_entries = scan_report.unsafe_entries;
    report.oversized_files = scan_report.oversized_files;
    report.truncated_folders = scan_report.truncated_folders;
    report.matched_games = items_by_game_id.len();
    GameMediaIndex {
        items_by_game_id,
        front_paths_by_game_id,
        back_paths_by_game_id,
        spine_paths_by_game_id,
        full_paths_by_game_id,
        policy,
        report,
    }
}

fn select_game_image<'a>(
    items: &'a [GameMediaItem],
    game: &Game,
    priorities: &[String],
    region_priorities: &[String],
) -> Option<&'a GameMediaItem> {
    priorities.iter().find_map(|priority| {
        items
            .iter()
            .filter(|item| {
                item.kind == GameMediaKind::Image
                    && item.media_type.trim().eq_ignore_ascii_case(priority)
            })
            .min_by(|left, right| {
                media_region_rank(
                    left.region.as_deref(),
                    game.region.as_deref(),
                    region_priorities,
                )
                .cmp(&media_region_rank(
                    right.region.as_deref(),
                    game.region.as_deref(),
                    region_priorities,
                ))
                .then_with(|| left.ordinal.cmp(&right.ordinal))
                .then_with(|| left.path.cmp(&right.path))
            })
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct VideoCandidate {
    path: PathBuf,
    media_type: String,
    region: Option<String>,
    ordinal: u32,
}

fn scan_video_folder(
    folder: &Path,
    configured_media_type: &str,
    video_type_priorities: &[String],
    scanned_paths: &mut BTreeSet<(GameMediaKind, PathBuf)>,
    report: &mut FrontImageIndexReport,
) -> BTreeMap<String, Vec<VideoCandidate>> {
    let mut candidates = BTreeMap::<String, Vec<VideoCandidate>>::new();
    let mut remaining = MAX_FILES_PER_FOLDER;
    if configured_media_type
        .trim()
        .eq_ignore_ascii_case("Theme Video")
    {
        scan_video_level(
            folder,
            "Theme Video",
            true,
            &mut remaining,
            scanned_paths,
            report,
            &mut candidates,
        );
    } else {
        if !scanned_paths.insert((GameMediaKind::Video, folder.to_path_buf())) {
            return candidates;
        }
        if !safe_media_directory(folder, report) {
            return candidates;
        }
        report.scanned_folders = report.scanned_folders.saturating_add(1);
        let Ok((entries, truncated)) = sorted_directory_entries(folder, remaining) else {
            report.unresolved_folders = report.unresolved_folders.saturating_add(1);
            return candidates;
        };
        if truncated {
            report.truncated_folders = report.truncated_folders.saturating_add(1);
        }
        for entry in entries {
            if remaining == 0 {
                report.truncated_folders = report.truncated_folders.saturating_add(1);
                break;
            }
            remaining -= 1;
            let Ok(file_type) = entry.file_type() else {
                report.unsafe_entries = report.unsafe_entries.saturating_add(1);
                continue;
            };
            if file_type.is_symlink() {
                report.unsafe_entries = report.unsafe_entries.saturating_add(1);
                continue;
            }
            if file_type.is_file() {
                add_video_candidate(&mut candidates, entry.path(), "Video Snap", None, report);
                continue;
            }
            if !file_type.is_dir() {
                report.unsafe_entries = report.unsafe_entries.saturating_add(1);
                continue;
            }
            let Some(name) = entry.file_name().to_str().map(str::to_string) else {
                report.unsafe_entries = report.unsafe_entries.saturating_add(1);
                continue;
            };
            let Some(media_type) = video_subfolder_type(&name) else {
                continue;
            };
            scan_video_level(
                &entry.path(),
                media_type,
                true,
                &mut remaining,
                scanned_paths,
                report,
                &mut candidates,
            );
        }
    }
    for values in candidates.values_mut() {
        values.sort_by(|left, right| {
            video_type_rank(&left.media_type, video_type_priorities)
                .cmp(&video_type_rank(&right.media_type, video_type_priorities))
                .then_with(|| left.ordinal.cmp(&right.ordinal))
                .then_with(|| left.path.cmp(&right.path))
        });
    }
    candidates
}

#[allow(clippy::too_many_arguments)]
fn scan_video_level(
    folder: &Path,
    media_type: &str,
    allow_regions: bool,
    remaining: &mut usize,
    scanned_paths: &mut BTreeSet<(GameMediaKind, PathBuf)>,
    report: &mut FrontImageIndexReport,
    candidates: &mut BTreeMap<String, Vec<VideoCandidate>>,
) {
    if !scanned_paths.insert((GameMediaKind::Video, folder.to_path_buf())) {
        return;
    }
    if !safe_media_directory(folder, report) {
        return;
    }
    report.scanned_folders = report.scanned_folders.saturating_add(1);
    let Ok((entries, truncated)) = sorted_directory_entries(folder, *remaining) else {
        report.unresolved_folders = report.unresolved_folders.saturating_add(1);
        return;
    };
    if truncated {
        report.truncated_folders = report.truncated_folders.saturating_add(1);
    }
    for entry in entries {
        if *remaining == 0 {
            report.truncated_folders = report.truncated_folders.saturating_add(1);
            break;
        }
        *remaining -= 1;
        let Ok(file_type) = entry.file_type() else {
            report.unsafe_entries = report.unsafe_entries.saturating_add(1);
            continue;
        };
        if file_type.is_symlink() {
            report.unsafe_entries = report.unsafe_entries.saturating_add(1);
            continue;
        }
        if file_type.is_file() {
            add_video_candidate(candidates, entry.path(), media_type, None, report);
            continue;
        }
        if !allow_regions || !file_type.is_dir() {
            if !file_type.is_dir() {
                report.unsafe_entries = report.unsafe_entries.saturating_add(1);
            }
            continue;
        }
        let Some(region) = entry.file_name().to_str().map(str::to_string) else {
            report.unsafe_entries = report.unsafe_entries.saturating_add(1);
            continue;
        };
        if !scanned_paths.insert((GameMediaKind::Video, entry.path())) {
            continue;
        }
        if !safe_media_directory(&entry.path(), report) {
            continue;
        }
        report.scanned_folders = report.scanned_folders.saturating_add(1);
        let Ok((region_entries, region_truncated)) =
            sorted_directory_entries(&entry.path(), *remaining)
        else {
            report.unresolved_folders = report.unresolved_folders.saturating_add(1);
            continue;
        };
        if region_truncated {
            report.truncated_folders = report.truncated_folders.saturating_add(1);
        }
        for region_entry in region_entries {
            if *remaining == 0 {
                report.truncated_folders = report.truncated_folders.saturating_add(1);
                break;
            }
            *remaining -= 1;
            let Ok(region_type) = region_entry.file_type() else {
                report.unsafe_entries = report.unsafe_entries.saturating_add(1);
                continue;
            };
            if !region_type.is_file() || region_type.is_symlink() {
                report.unsafe_entries = report.unsafe_entries.saturating_add(1);
                continue;
            }
            add_video_candidate(
                candidates,
                region_entry.path(),
                media_type,
                Some(region.clone()),
                report,
            );
        }
    }
}

fn safe_media_directory(folder: &Path, report: &mut FrontImageIndexReport) -> bool {
    let Ok(metadata) = fs::symlink_metadata(folder) else {
        report.unresolved_folders = report.unresolved_folders.saturating_add(1);
        return false;
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        report.unsafe_entries = report.unsafe_entries.saturating_add(1);
        return false;
    }
    true
}

fn add_video_candidate(
    candidates: &mut BTreeMap<String, Vec<VideoCandidate>>,
    path: PathBuf,
    media_type: &str,
    region: Option<String>,
    report: &mut FrontImageIndexReport,
) {
    if !validate_media_file(&path, SUPPORTED_VIDEO_EXTENSIONS, MAX_VIDEO_BYTES, report) {
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
        .push(VideoCandidate {
            path,
            media_type: media_type.to_string(),
            region,
            ordinal,
        });
}

fn validate_media_file(
    path: &Path,
    extensions: &[&str],
    maximum_bytes: u64,
    report: &mut FrontImageIndexReport,
) -> bool {
    let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
        return false;
    };
    if !extensions
        .iter()
        .any(|supported| extension.eq_ignore_ascii_case(supported))
    {
        return false;
    }
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return false;
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        report.unsafe_entries = report.unsafe_entries.saturating_add(1);
        return false;
    }
    if metadata.len() > maximum_bytes {
        report.oversized_files = report.oversized_files.saturating_add(1);
        return false;
    }
    true
}

fn push_game_media_item(
    game: &Game,
    item: GameMediaItem,
    items_by_game_id: &mut BTreeMap<String, Vec<GameMediaItem>>,
    seen_by_game_id: &mut BTreeMap<String, BTreeSet<PathBuf>>,
    indexed_items: &mut usize,
    report: &mut GameMediaIndexReport,
) {
    if *indexed_items >= MAX_MEDIA_ITEMS {
        report.truncated_items = report.truncated_items.saturating_add(1);
        return;
    }
    let game_items = items_by_game_id.entry(game.id.clone()).or_default();
    if game_items.len() >= MAX_MEDIA_ITEMS_PER_GAME {
        report.truncated_items = report.truncated_items.saturating_add(1);
        return;
    }
    if !seen_by_game_id
        .entry(game.id.clone())
        .or_default()
        .insert(item.path.clone())
    {
        return;
    }
    match item.kind {
        GameMediaKind::Image => {
            report.indexed_images = report.indexed_images.saturating_add(1);
        }
        GameMediaKind::Video => {
            report.indexed_videos = report.indexed_videos.saturating_add(1);
        }
    }
    game_items.push(item);
    *indexed_items = (*indexed_items).saturating_add(1);
}

fn video_subfolder_type(folder_name: &str) -> Option<&'static str> {
    if folder_name.eq_ignore_ascii_case("Theme") {
        Some("Theme Video")
    } else if folder_name.eq_ignore_ascii_case("Trailer") {
        Some("Trailer")
    } else if folder_name.eq_ignore_ascii_case("Recordings") {
        Some("Recording")
    } else if folder_name.eq_ignore_ascii_case("Marquee") {
        Some("Marquee")
    } else {
        None
    }
}

fn video_type_rank(media_type: &str, priorities: &[String]) -> usize {
    media_type_rank(media_type, priorities)
}

fn media_type_rank(media_type: &str, priorities: &[String]) -> usize {
    priorities
        .iter()
        .position(|priority| priority.trim().eq_ignore_ascii_case(media_type.trim()))
        .unwrap_or(priorities.len())
}

fn media_region_rank(
    region: Option<&str>,
    game_region: Option<&str>,
    region_priorities: &[String],
) -> (u8, usize, String) {
    let candidate_regions = region.map(region_parts).unwrap_or_default();
    if let Some(priority) = region_priorities
        .iter()
        .position(|priority| candidate_regions.contains(&normalized_key(priority)))
    {
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
    if region.is_none() {
        return (2, 0, String::new());
    }
    if candidate_regions.contains(&normalized_key("World")) {
        return (3, 0, String::new());
    }
    (4, 0, region.map(normalized_key).unwrap_or_default())
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
            back_image_type_priorities(Some(&settings)),
            FALLBACK_BACK_IMAGE_TYPES
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

        let configured_back = FrontendSettings {
            entries: vec![SettingEntry {
                key: "BackImageTypePriorities".into(),
                value: "Fanart - Box - Back, Box - Back, fanart - box - back".into(),
            }],
            ..FrontendSettings::default()
        };
        assert_eq!(
            back_image_type_priorities(Some(&configured_back)),
            ["Fanart - Box - Back", "Box - Back"]
        );
    }

    #[test]
    fn selected_game_media_policy_uses_typed_launchbox_settings_and_safe_fallbacks() {
        let configured = FrontendSettings {
            entries: vec![
                SettingEntry {
                    key: "ShowDetailsVideo".into(),
                    value: "false".into(),
                },
                SettingEntry {
                    key: "AutoPlayDetailsVideo".into(),
                    value: "true".into(),
                },
                SettingEntry {
                    key: "VideoTypePriorities".into(),
                    value: "Trailer, Theme Video, trailer, Video Snap".into(),
                },
            ],
            ..FrontendSettings::default()
        };
        assert_eq!(
            GameDetailsMediaPolicy::from_settings(Some(&configured)),
            GameDetailsMediaPolicy {
                show_video: false,
                auto_play_video: true,
                video_type_priorities: vec![
                    "Trailer".into(),
                    "Theme Video".into(),
                    "Video Snap".into(),
                ],
            }
        );

        let malformed = FrontendSettings {
            entries: vec![
                SettingEntry {
                    key: "ShowDetailsVideo".into(),
                    value: "sometimes".into(),
                },
                SettingEntry {
                    key: "AutoPlayDetailsVideo".into(),
                    value: String::new(),
                },
                SettingEntry {
                    key: "VideoTypePriorities".into(),
                    value: " , , ".into(),
                },
            ],
            ..FrontendSettings::default()
        };
        assert_eq!(
            GameDetailsMediaPolicy::from_settings(Some(&malformed)),
            GameDetailsMediaPolicy::default()
        );
    }

    #[test]
    fn indexes_all_selected_game_media_with_native_paths_and_launchbox_ordering() {
        let directory = tempfile::tempdir().expect("temporary LaunchBox root");
        let boxes = directory.path().join("Images/Fixture Console/Box - Front");
        let box_backs = directory.path().join("Images/Fixture Console/Box - Back");
        let box_spines = directory.path().join("Images/Fixture Console/Box - Spine");
        let box_full = directory.path().join("Images/Fixture Console/Box - Full");
        let screenshots = directory
            .path()
            .join("Images/Fixture Console/Screenshot - Gameplay");
        let fanart = directory
            .path()
            .join("Images/Fixture Console/Fanart - Background");
        let videos = directory.path().join("Videos/Fixture Console");
        for folder in [
            &boxes,
            &box_backs,
            &box_spines,
            &box_full,
            &screenshots,
            &fanart,
        ] {
            fs::create_dir_all(folder.join("North America")).expect("image region");
        }
        fs::create_dir_all(videos.join("Theme/North America")).expect("theme video region");
        fs::create_dir_all(videos.join("Trailer")).expect("trailer video folder");
        fs::write(boxes.join("North America/Fixture Adventure-02.png"), b"box").expect("box");
        fs::write(
            box_backs.join("North America/Fixture Adventure-01.png"),
            b"box back",
        )
        .expect("box back");
        fs::write(
            box_spines.join("North America/Fixture Adventure-01.png"),
            b"box spine",
        )
        .expect("box spine");
        fs::write(
            box_full.join("North America/Fixture Adventure-01.png"),
            b"box full",
        )
        .expect("box full");
        fs::write(
            screenshots.join("North America/Fixture Adventure-01.jpg"),
            b"screenshot",
        )
        .expect("screenshot");
        fs::write(
            fanart.join("North America/Fixture Adventure-01.webp"),
            b"fanart",
        )
        .expect("fanart");
        fs::write(videos.join("Fixture Adventure-03.mp4"), b"video snap").expect("video snap");
        fs::write(
            videos.join("Theme/North America/Fixture Adventure-01.mp4"),
            b"theme",
        )
        .expect("theme");
        fs::write(videos.join("Trailer/Fixture Adventure-02.webm"), b"trailer").expect("trailer");
        fs::write(videos.join("explicit-file-name.mov"), b"explicit").expect("explicit video");

        let folders = [
            PlatformFolder {
                platform: "Fixture Console".into(),
                media_type: "Video".into(),
                folder_path: r"Videos\Fixture Console".into(),
            },
            PlatformFolder {
                platform: "Fixture Console".into(),
                media_type: "Theme Video".into(),
                folder_path: r"Videos\Fixture Console\Theme".into(),
            },
            PlatformFolder {
                platform: "Fixture Console".into(),
                media_type: "Screenshot - Gameplay".into(),
                folder_path: r"Images\Fixture Console\Screenshot - Gameplay".into(),
            },
            PlatformFolder {
                platform: "Fixture Console".into(),
                media_type: "Box - Front".into(),
                folder_path: r"Images\Fixture Console\Box - Front".into(),
            },
            PlatformFolder {
                platform: "Fixture Console".into(),
                media_type: "Box - Back".into(),
                folder_path: r"Images\Fixture Console\Box - Back".into(),
            },
            PlatformFolder {
                platform: "Fixture Console".into(),
                media_type: "Box - Spine".into(),
                folder_path: r"Images\Fixture Console\Box - Spine".into(),
            },
            PlatformFolder {
                platform: "Fixture Console".into(),
                media_type: "Box - Full".into(),
                folder_path: r"Images\Fixture Console\Box - Full".into(),
            },
            PlatformFolder {
                platform: "Fixture Console".into(),
                media_type: "Fanart - Background".into(),
                folder_path: r"Images\Fixture Console\Fanart - Background".into(),
            },
        ];
        let mut fixture_game = game(
            "fixture-adventure",
            "Fixture Adventure",
            Some("North America"),
        );
        fixture_game.video_path = Some(r"Videos\Fixture Console\explicit-file-name.mov".into());
        fixture_game.theme_video_path =
            Some(r"Videos\Fixture Console\Theme\North America\Fixture Adventure-01.mp4".into());
        let configured = FrontendSettings {
            entries: vec![
                SettingEntry {
                    key: "FrontImageTypePriorities".into(),
                    value: "Box - Front,Screenshot - Gameplay,Fanart - Background".into(),
                },
                SettingEntry {
                    key: "BackImageTypePriorities".into(),
                    value: "Box - Back,Box - Back - Reconstructed,Advertisement Flyer - Back,Fanart - Box - Back".into(),
                },
                SettingEntry {
                    key: "RegionPriorities".into(),
                    value: "North America,World".into(),
                },
                SettingEntry {
                    key: "ShowDetailsVideo".into(),
                    value: "true".into(),
                },
                SettingEntry {
                    key: "AutoPlayDetailsVideo".into(),
                    value: "true".into(),
                },
                SettingEntry {
                    key: "VideoTypePriorities".into(),
                    value: "Theme Video,Trailer,Video Snap".into(),
                },
            ],
            ..FrontendSettings::default()
        };

        let index = index_game_media(
            directory.path(),
            &[fixture_game],
            &folders,
            Some(&configured),
            &HostPathResolver::default(),
        );
        let items = &index.items_by_game_id["fixture-adventure"];
        assert_eq!(items.len(), 10);
        assert_eq!(
            items
                .iter()
                .map(|item| (item.kind, item.media_type.as_str(), item.ordinal))
                .collect::<Vec<_>>(),
            [
                (GameMediaKind::Image, "Box - Front", 2),
                (GameMediaKind::Image, "Screenshot - Gameplay", 1),
                (GameMediaKind::Image, "Fanart - Background", 1),
                (GameMediaKind::Image, "Box - Back", 1),
                (GameMediaKind::Image, "Box - Full", 1),
                (GameMediaKind::Image, "Box - Spine", 1),
                (GameMediaKind::Video, "Theme Video", 1),
                (GameMediaKind::Video, "Trailer", 2),
                (GameMediaKind::Video, "Video Snap", 0),
                (GameMediaKind::Video, "Video Snap", 3),
            ]
        );
        assert_eq!(
            index.front_paths_by_game_id["fixture-adventure"],
            boxes.join("North America/Fixture Adventure-02.png")
        );
        assert_eq!(
            index.back_paths_by_game_id["fixture-adventure"],
            box_backs.join("North America/Fixture Adventure-01.png")
        );
        assert_eq!(
            index.spine_paths_by_game_id["fixture-adventure"],
            box_spines.join("North America/Fixture Adventure-01.png")
        );
        assert_eq!(
            index.full_paths_by_game_id["fixture-adventure"],
            box_full.join("North America/Fixture Adventure-01.png")
        );
        assert_eq!(index.report.indexed_images, 6);
        assert_eq!(index.report.indexed_videos, 4);
        assert_eq!(index.report.matched_games, 1);
        assert!(
            items
                .iter()
                .all(|item| item.path.starts_with(directory.path())),
            "only native paths leave the platform boundary"
        );
        assert_eq!(
            items
                .iter()
                .filter(|item| item.media_type == "Theme Video")
                .count(),
            1,
            "an explicitly configured Theme folder must not duplicate the Video-root scan"
        );
    }

    #[test]
    fn selected_game_media_hides_all_videos_when_launchbox_disables_them() {
        let directory = tempfile::tempdir().expect("temporary LaunchBox root");
        let videos = directory.path().join("Videos/Fixture Console");
        fs::create_dir_all(&videos).expect("video folder");
        fs::write(videos.join("Fixture-01.mp4"), b"video").expect("video");
        let mut fixture_game = game("fixture", "Fixture", None);
        fixture_game.video_path = Some(r"Videos\Fixture Console\Fixture-01.mp4".into());
        let configured = FrontendSettings {
            entries: vec![SettingEntry {
                key: "ShowDetailsVideo".into(),
                value: "false".into(),
            }],
            ..FrontendSettings::default()
        };
        let index = index_game_media(
            directory.path(),
            &[fixture_game],
            &[PlatformFolder {
                platform: "Fixture Console".into(),
                media_type: "Video".into(),
                folder_path: r"Videos\Fixture Console".into(),
            }],
            Some(&configured),
            &HostPathResolver::default(),
        );
        assert!(index.items_by_game_id.is_empty());
        assert_eq!(index.report.indexed_videos, 0);
        assert_eq!(index.report.configured_folders, 0);
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
