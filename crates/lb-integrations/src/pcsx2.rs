use crate::{DiscoveredContainerSave, DiscoveredEmulatorSave, EmulatorSaveKind};
#[path = "pcsx2_card.rs"]
mod card;
pub use card::{
    extract_pcsx2_memory_card_save, folder_manifest_signature, list_pcsx2_memory_card_saves,
    prepare_pcsx2_memory_card_deletion, prepare_pcsx2_memory_card_restore,
    ExtractedPcsx2MemoryCardSave, Pcsx2CardError, Pcsx2MemoryCardSave,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pcsx2Content {
    pub game_id: String,
    pub additional_application_id: Option<String>,
    pub content_path: PathBuf,
    pub title: String,
    pub alternate_titles: Vec<String>,
}

#[derive(Clone, Debug)]
struct PreparedContent<'a> {
    target: &'a Pcsx2Content,
    content_stem: String,
    serial: Option<String>,
}

#[derive(Clone, Debug)]
struct CardRecord<'a> {
    context: &'a PreparedContent<'a>,
    card_path: PathBuf,
    group_id: String,
    save: Pcsx2MemoryCardSave,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StateName {
    serial: String,
    crc: String,
    slot: i32,
}

pub fn is_pcsx2_emulator(title: &str, application_path: &Path) -> bool {
    title.trim().eq_ignore_ascii_case("pcsx2")
        || application_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .is_some_and(|stem| {
                stem.eq_ignore_ascii_case("pcsx2") || stem.eq_ignore_ascii_case("pcsx2-qt")
            })
}

/// Returns existing PCSX2 data roots using LaunchBox 13.27's portable probes
/// plus the host-native PCSX2 root.
///
/// Each returned root may contain `memcards` and `sstates`. The executable's
/// `inis` directory is retained as a compatibility probe for older PCSX2
/// layouts even though current PCSX2 keeps those folders directly below its
/// data root.
pub fn default_pcsx2_data_directories(emulator_application_path: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(parent) = emulator_application_path.parent() {
        candidates.push(parent.to_path_buf());
        candidates.push(parent.join("inis"));
    }

    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    {
        let mut found_absolute_config_home = false;
        if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
            let config_home = PathBuf::from(config_home);
            if config_home.is_absolute() {
                candidates.push(config_home.join("PCSX2"));
                found_absolute_config_home = true;
            }
        }
        if !found_absolute_config_home {
            if let Some(home) = std::env::var_os("HOME") {
                candidates.push(PathBuf::from(home).join(".config").join("PCSX2"));
            }
        }
    }
    #[cfg(target_os = "macos")]
    if let Some(home) = std::env::var_os("HOME") {
        candidates.push(
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("PCSX2"),
        );
    }
    #[cfg(windows)]
    if let Some(profile) = std::env::var_os("USERPROFILE") {
        candidates.push(PathBuf::from(profile).join("Documents").join("PCSX2"));
    }

    existing_unique_directories(candidates)
}

/// Discovers PCSX2 save states and folder/raw-memory-card members.
///
/// Save states are ordinary files and can use the application's regular save
/// transactions. Memory-card members deliberately identify their containing
/// card separately so the complete shared card can never fall through to an
/// ordinary-file transaction.
pub fn discover_pcsx2_saves(
    emulator_application_path: &Path,
    targets: &[Pcsx2Content],
    data_directories: &[PathBuf],
) -> Result<Vec<DiscoveredEmulatorSave>, Pcsx2Error> {
    let emulator_application_path =
        canonical_regular_file(emulator_application_path, "PCSX2 executable")?;
    let emulator_file_name = unicode_file_name(&emulator_application_path)?.to_string();
    let data_directories = existing_unique_directories(data_directories.iter().cloned());
    let game_index = load_game_index_titles(&emulator_application_path)?;

    let mut prepared = Vec::new();
    for target in targets {
        let content_path = canonical_regular_file(&target.content_path, "PCSX2 content")?;
        let content_stem = content_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .filter(|stem| !stem.is_empty())
            .ok_or_else(|| Pcsx2Error::NonUnicodePath {
                path: content_path.clone(),
            })?
            .to_string();
        let serial = extract_serial_from_content(&content_path)
            .or_else(|| extract_serial_from_text(&content_stem))
            .or_else(|| extract_serial_from_text(&target.title));
        prepared.push(PreparedContent {
            target,
            content_stem,
            serial,
        });
    }

    let mut discovered = discover_folder_card_saves(
        &emulator_file_name,
        &prepared,
        &data_directories,
        &game_index,
    )?;
    discovered.extend(discover_state_files(
        &emulator_file_name,
        &prepared,
        &data_directories,
    )?);
    discovered.sort_by(|left, right| {
        left.game_id
            .cmp(&right.game_id)
            .then_with(|| {
                left.additional_application_id
                    .cmp(&right.additional_application_id)
            })
            .then_with(|| {
                path_sort_key(&left.primary_path).cmp(&path_sort_key(&right.primary_path))
            })
            .then_with(|| left.save_group_id.cmp(&right.save_group_id))
    });
    Ok(discovered)
}

fn discover_folder_card_saves<'a>(
    emulator_file_name: &str,
    targets: &'a [PreparedContent<'a>],
    data_directories: &[PathBuf],
    game_index: &BTreeMap<String, String>,
) -> Result<Vec<DiscoveredEmulatorSave>, Pcsx2Error> {
    let mut card_paths = Vec::new();
    for data_directory in data_directories {
        let memory_cards = data_directory.join("memcards");
        if !memory_cards.is_dir() {
            continue;
        }
        for entry in sorted_directory_entries(&memory_cards)? {
            let file_type = entry.file_type().map_err(|source| Pcsx2Error::Read {
                path: entry.path(),
                source,
            })?;
            if (!file_type.is_dir() && !file_type.is_file())
                || file_type.is_symlink()
                || !has_extension(&entry.path(), "ps2")
            {
                continue;
            }
            card_paths.push(if file_type.is_dir() {
                canonical_directory(&entry.path(), "PCSX2 folder memory card")?
            } else {
                canonical_regular_file(&entry.path(), "PCSX2 raw memory card")?
            });
        }
    }
    card_paths.sort_by_key(|path| path_sort_key(path));
    card_paths.dedup();

    let mut records = Vec::new();
    let mut identities = BTreeSet::new();
    for card_path in card_paths {
        let Ok(saves) = list_pcsx2_memory_card_saves(&card_path) else {
            // LaunchBox 13.27 treats an unreadable/corrupt card as an isolated
            // probe failure so other configured cards and states still scan.
            continue;
        };
        let save_records = saves
            .into_iter()
            .map(|save| {
                let serial = extract_serial_from_text(&save.directory_name);
                (save, serial)
            })
            .collect::<Vec<_>>();
        let mut card_matched = false;
        let mut unknown = 0_usize;
        for context in targets {
            for (save, save_serial) in &save_records {
                if !folder_save_matches(context, save, save_serial.as_deref(), game_index) {
                    continue;
                }
                let member = nonempty_member_name(&save.directory_name, &mut unknown);
                let group_id = folder_card_group_id(&card_path, &member)?;
                if identities.insert(group_id.to_ascii_lowercase()) {
                    records.push(CardRecord {
                        context,
                        card_path: card_path.clone(),
                        group_id,
                        save: save.clone(),
                    });
                    card_matched = true;
                }
            }
        }

        // LaunchBox 13.27's final fallback is intentionally narrow: one
        // serial-less context may claim serial-less members on an otherwise
        // unmatched card.
        if card_matched || targets.len() != 1 || targets[0].serial.is_some() {
            continue;
        }
        for (save, save_serial) in &save_records {
            if save_serial.is_some() {
                continue;
            }
            let member = nonempty_member_name(&save.directory_name, &mut unknown);
            let group_id = folder_card_group_id(&card_path, &member)?;
            if identities.insert(group_id.to_ascii_lowercase()) {
                records.push(CardRecord {
                    context: &targets[0],
                    card_path: card_path.clone(),
                    group_id,
                    save: save.clone(),
                });
            }
        }
    }

    let mut group_counts = BTreeMap::<String, usize>::new();
    for record in &records {
        *group_counts.entry(context_key(record.context)).or_default() += 1;
    }
    Ok(records
        .into_iter()
        .map(|record| {
            let member = record
                .group_id
                .rsplit_once(':')
                .map(|(_, member)| member)
                .unwrap_or(&record.save.directory_name)
                .to_string();
            let include_member = group_counts
                .get(&context_key(record.context))
                .copied()
                .unwrap_or_default()
                > 1;
            DiscoveredEmulatorSave {
                game_id: record.context.target.game_id.clone(),
                additional_application_id: record.context.target.additional_application_id.clone(),
                emulator_file_name: emulator_file_name.to_string(),
                emulator_core: String::new(),
                kind: EmulatorSaveKind::Game,
                primary_path: record.card_path,
                companion_paths: Vec::new(),
                save_group_id: Some(record.group_id),
                save_group_name: if include_member {
                    format!("My Save File - {member}")
                } else {
                    "My Save File".to_string()
                },
                display_chip_text: None,
                container_save: Some(DiscoveredContainerSave {
                    original_file_name: member,
                    reported_file_size_bytes: (record.save.total_bytes > 0)
                        .then_some(record.save.total_bytes),
                    reported_last_modified: record.save.modified,
                }),
            }
        })
        .collect())
}

fn discover_state_files(
    emulator_file_name: &str,
    targets: &[PreparedContent<'_>],
    data_directories: &[PathBuf],
) -> Result<Vec<DiscoveredEmulatorSave>, Pcsx2Error> {
    let mut paths = Vec::new();
    for data_directory in data_directories {
        let states = data_directory.join("sstates");
        if !states.is_dir() {
            continue;
        }
        for entry in sorted_directory_entries(&states)? {
            let file_type = entry.file_type().map_err(|source| Pcsx2Error::Read {
                path: entry.path(),
                source,
            })?;
            if file_type.is_file()
                && (has_extension(&entry.path(), "p2s") || has_extension(&entry.path(), "p2z"))
            {
                paths.push(canonical_regular_file(&entry.path(), "PCSX2 save state")?);
            }
        }
    }
    paths.sort_by_key(|path| path_sort_key(path));
    paths.dedup();

    let mut discovered = Vec::new();
    let mut identities = BTreeSet::new();
    for path in paths {
        let Some(state) = unicode_file_name(&path).ok().and_then(parse_state_name) else {
            continue;
        };
        for context in targets {
            if !context
                .serial
                .as_deref()
                .is_some_and(|serial| serials_equal(serial, &state.serial))
            {
                continue;
            }
            let group_id = format!("pcsx2-state:{}:{:02}", state.serial, state.slot);
            let identity = format!("{}:{}", group_id, context_key(context)).to_ascii_lowercase();
            if !identities.insert(identity) {
                continue;
            }
            discovered.push(DiscoveredEmulatorSave {
                game_id: context.target.game_id.clone(),
                additional_application_id: context.target.additional_application_id.clone(),
                emulator_file_name: emulator_file_name.to_string(),
                emulator_core: String::new(),
                kind: EmulatorSaveKind::State { slot: state.slot },
                primary_path: path.clone(),
                companion_paths: Vec::new(),
                save_group_id: Some(group_id),
                save_group_name: "My Save State".to_string(),
                display_chip_text: None,
                container_save: None,
            });
        }
    }
    Ok(discovered)
}

fn folder_save_matches(
    context: &PreparedContent<'_>,
    save: &Pcsx2MemoryCardSave,
    save_serial: Option<&str>,
    game_index: &BTreeMap<String, String>,
) -> bool {
    if let (Some(save_serial), Some(context_serial)) = (save_serial, context.serial.as_deref()) {
        return serials_equal(context_serial, save_serial);
    }
    if let Some(save_serial) = save_serial {
        if let Some(game_index_title) = game_index.get(&normalize_serial(save_serial)) {
            if context_titles(context)
                .into_iter()
                .any(|title| titles_similar(game_index_title, &title))
            {
                return true;
            }
        }
    }
    [save.directory_name.as_str(), save.title.as_str()]
        .into_iter()
        .any(|save_title| {
            context_titles(context)
                .into_iter()
                .any(|context_title| titles_similar(save_title, &context_title))
        })
}

fn context_titles(context: &PreparedContent<'_>) -> Vec<String> {
    let mut titles = Vec::new();
    if !context.target.title.trim().is_empty() {
        titles.push(context.target.title.clone());
    }
    titles.extend(
        context
            .target
            .alternate_titles
            .iter()
            .filter(|title| !title.trim().is_empty())
            .cloned(),
    );
    titles.push(context.content_stem.clone());
    let cleaned = clean_rom_name(&context.content_stem);
    if !cleaned.eq_ignore_ascii_case(&context.content_stem) && !cleaned.is_empty() {
        titles.push(cleaned);
    }
    titles
}

fn parse_state_name(file_name: &str) -> Option<StateName> {
    let (body, extension) = file_name.rsplit_once('.')?;
    if !extension.eq_ignore_ascii_case("p2s") && !extension.eq_ignore_ascii_case("p2z") {
        return None;
    }
    let (prefix, slot) = body.rsplit_once('.')?;
    if slot.len() != 2 || !slot.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let close = prefix.strip_suffix(')')?;
    let open = close.rfind('(')?;
    let serial = close[..open].trim();
    let crc = &close[open + 1..];
    if serial.is_empty()
        || !serial
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
        || crc.len() != 8
        || !crc.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return None;
    }
    Some(StateName {
        serial: serial
            .bytes()
            .filter(|byte| *byte != b'-')
            .map(|byte| char::from(byte).to_ascii_uppercase())
            .collect(),
        crc: crc.to_ascii_uppercase(),
        slot: slot.parse().ok()?,
    })
}

fn extract_serial_from_content(path: &Path) -> Option<String> {
    let mut file = fs::File::open(path).ok()?;
    let mut bytes = Vec::new();
    use std::io::Read;
    file.by_ref()
        .take(1024 * 1024)
        .read_to_end(&mut bytes)
        .ok()?;
    extract_system_cnf_serial(&bytes)
}

fn extract_system_cnf_serial(bytes: &[u8]) -> Option<String> {
    for window in bytes.windows(11) {
        if window[..4].iter().all(u8::is_ascii_alphabetic)
            && window[4] == b'_'
            && window[5..8].iter().all(u8::is_ascii_digit)
            && window[8] == b'.'
            && window[9..11].iter().all(u8::is_ascii_digit)
        {
            let mut serial = String::with_capacity(9);
            serial.extend(
                window[..4]
                    .iter()
                    .map(|byte| char::from(*byte).to_ascii_uppercase()),
            );
            serial.extend(window[5..8].iter().map(|byte| char::from(*byte)));
            serial.extend(window[9..11].iter().map(|byte| char::from(*byte)));
            return Some(serial);
        }
    }
    None
}

fn extract_serial_from_text(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    for start in 0..bytes.len() {
        for strip_ba in [true, false] {
            let mut cursor = start;
            if strip_ba {
                if !bytes
                    .get(cursor..cursor + 2)
                    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"BA"))
                {
                    continue;
                }
                cursor += 2;
            }
            for letter_count in (4..=6).rev() {
                let Some(letters) = bytes.get(cursor..cursor + letter_count) else {
                    continue;
                };
                if !letters.iter().all(u8::is_ascii_alphabetic) {
                    continue;
                }
                let mut digits_at = cursor + letter_count;
                if bytes.get(digits_at) == Some(&b'-') {
                    digits_at += 1;
                }
                let Some(digits) = bytes.get(digits_at..digits_at + 5) else {
                    continue;
                };
                if !digits.iter().all(u8::is_ascii_digit) {
                    continue;
                }
                let mut serial = String::with_capacity(letter_count + 5);
                serial.extend(
                    letters
                        .iter()
                        .map(|byte| char::from(*byte).to_ascii_uppercase()),
                );
                serial.extend(digits.iter().map(|byte| char::from(*byte)));
                return Some(serial);
            }
        }
    }
    None
}

fn serials_equal(left: &str, right: &str) -> bool {
    normalize_serial(left).eq_ignore_ascii_case(&normalize_serial(right))
}

fn normalize_serial(value: &str) -> String {
    let mut normalized = value
        .bytes()
        .filter(|byte| *byte != b'-')
        .map(|byte| char::from(byte).to_ascii_uppercase())
        .collect::<String>();
    if let Some(remainder) = normalized.strip_prefix("BA") {
        if (9..=10).contains(&remainder.len())
            && remainder
                .bytes()
                .take(remainder.len() - 5)
                .all(|byte| byte.is_ascii_uppercase())
            && remainder
                .bytes()
                .skip(remainder.len() - 5)
                .all(|byte| byte.is_ascii_digit())
        {
            normalized = remainder.to_string();
        }
    }
    normalized
}

fn titles_similar(left: &str, right: &str) -> bool {
    let left = normalize_title(left);
    let right = normalize_title(right);
    if left.is_empty() || right.is_empty() {
        return false;
    }
    if left.eq_ignore_ascii_case(&right) {
        return true;
    }
    let left_numbers = numeric_tokens(&left);
    let right_numbers = numeric_tokens(&right);
    if (!left_numbers.is_empty() || !right_numbers.is_empty()) && left_numbers != right_numbers {
        return false;
    }
    let left_tokens = strict_tokens(&left);
    let right_tokens = strict_tokens(&right);
    if left_tokens.is_empty() || right_tokens.is_empty() {
        return false;
    }
    if left_tokens.intersection(&right_tokens).count() < 2 {
        return false;
    }
    if left_tokens.len().abs_diff(right_tokens.len()) > 1 {
        return false;
    }
    left_tokens.is_subset(&right_tokens) || right_tokens.is_subset(&left_tokens)
}

fn normalize_title(value: &str) -> String {
    let mut normalized = String::new();
    let mut space = true;
    for character in value
        .nfd()
        .filter(|character| !is_combining_mark(*character))
    {
        if character.is_alphanumeric() {
            normalized.extend(character.to_uppercase());
            space = false;
        } else if !space {
            normalized.push(' ');
            space = true;
        }
    }
    normalized.trim().to_string()
}

fn numeric_tokens(value: &str) -> Vec<String> {
    let mut seen = BTreeSet::new();
    value
        .split_whitespace()
        .filter(|token| token.chars().all(|character| character.is_ascii_digit()))
        .filter(|token| seen.insert((*token).to_string()))
        .map(str::to_string)
        .collect()
}

fn strict_tokens(value: &str) -> BTreeSet<String> {
    value
        .split_whitespace()
        .filter(|token| {
            token.len() >= 3
                || (token.len() >= 2 && token.chars().all(|character| character.is_ascii_digit()))
        })
        .map(str::to_string)
        .collect()
}

fn clean_rom_name(value: &str) -> String {
    let mut cleaned = String::new();
    let mut skipping = false;
    for character in value.chars() {
        if !skipping && matches!(character, '(' | '[') {
            skipping = true;
        } else if skipping && matches!(character, ')' | ']') {
            skipping = false;
        } else if !skipping {
            cleaned.push(character);
        }
    }
    cleaned.trim().to_string()
}

fn load_game_index_titles(
    emulator_application_path: &Path,
) -> Result<BTreeMap<String, String>, Pcsx2Error> {
    let Some(root) = emulator_application_path.parent() else {
        return Ok(BTreeMap::new());
    };
    let path = root.join("resources").join("GameIndex.yaml");
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(source) => return Err(Pcsx2Error::Read { path, source }),
    };
    let mut current_serial = None;
    let mut titles = BTreeMap::new();
    for raw in contents.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(header) = line.strip_suffix(':') {
            if !header.is_empty()
                && header
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            {
                current_serial = Some(normalize_serial(header));
                continue;
            }
        }
        let Some(serial) = current_serial.as_ref() else {
            continue;
        };
        let Some(name) = line.strip_prefix("name:") else {
            continue;
        };
        let name = name.trim();
        let name = name
            .strip_prefix('"')
            .and_then(|name| name.strip_suffix('"'))
            .unwrap_or(name);
        titles
            .entry(serial.clone())
            .or_insert_with(|| name.to_string());
    }
    Ok(titles)
}

fn folder_card_group_id(card_path: &Path, member: &str) -> Result<String, Pcsx2Error> {
    let card_name = card_path
        .file_stem()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .ok_or_else(|| Pcsx2Error::NonUnicodePath {
            path: card_path.to_path_buf(),
        })?;
    Ok(format!("pcsx2:{card_name}:{member}"))
}

fn nonempty_member_name(directory_name: &str, unknown: &mut usize) -> String {
    if !directory_name.trim().is_empty() {
        return directory_name.to_string();
    }
    let member = format!("unknown{unknown}");
    *unknown += 1;
    member
}

fn context_key(context: &PreparedContent<'_>) -> String {
    context
        .target
        .additional_application_id
        .as_ref()
        .unwrap_or(&context.target.game_id)
        .to_ascii_lowercase()
}

fn sorted_directory_entries(path: &Path) -> Result<Vec<fs::DirEntry>, Pcsx2Error> {
    let mut entries = fs::read_dir(path)
        .map_err(|source| Pcsx2Error::Read {
            path: path.to_path_buf(),
            source,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| Pcsx2Error::Read {
            path: path.to_path_buf(),
            source,
        })?;
    entries.sort_by_key(|entry| entry.file_name().to_string_lossy().to_ascii_lowercase());
    Ok(entries)
}

fn existing_unique_directories(candidates: impl IntoIterator<Item = PathBuf>) -> Vec<PathBuf> {
    let mut seen = BTreeSet::new();
    candidates
        .into_iter()
        .filter_map(|path| fs::canonicalize(path).ok())
        .filter(|path| path.is_dir() && seen.insert(path.clone()))
        .collect()
}

fn canonical_regular_file(path: &Path, kind: &'static str) -> Result<PathBuf, Pcsx2Error> {
    let metadata = fs::symlink_metadata(path).map_err(|source| Pcsx2Error::Read {
        path: path.to_path_buf(),
        source,
    })?;
    if !metadata.file_type().is_file() {
        return Err(Pcsx2Error::UnexpectedFileType {
            kind,
            path: path.to_path_buf(),
        });
    }
    fs::canonicalize(path).map_err(|source| Pcsx2Error::Read {
        path: path.to_path_buf(),
        source,
    })
}

fn canonical_directory(path: &Path, kind: &'static str) -> Result<PathBuf, Pcsx2Error> {
    let metadata = fs::symlink_metadata(path).map_err(|source| Pcsx2Error::Read {
        path: path.to_path_buf(),
        source,
    })?;
    if !metadata.file_type().is_dir() {
        return Err(Pcsx2Error::UnexpectedFileType {
            kind,
            path: path.to_path_buf(),
        });
    }
    fs::canonicalize(path).map_err(|source| Pcsx2Error::Read {
        path: path.to_path_buf(),
        source,
    })
}

fn unicode_file_name(path: &Path) -> Result<&str, Pcsx2Error> {
    path.file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| Pcsx2Error::NonUnicodePath {
            path: path.to_path_buf(),
        })
}

fn has_extension(path: &Path, expected: &str) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case(expected))
}

fn path_sort_key(path: &Path) -> String {
    path.to_string_lossy().to_ascii_lowercase()
}

#[derive(Debug, Error)]
pub enum Pcsx2Error {
    #[error("could not read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{kind} has an unsupported file type: {path}")]
    UnexpectedFileType { kind: &'static str, path: PathBuf },
    #[error("path is not valid Unicode: {path}")]
    NonUnicodePath { path: PathBuf },
    #[error("save metadata is too large: {path}")]
    TooLarge { path: PathBuf },
    #[error(transparent)]
    MemoryCard(#[from] Pcsx2CardError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use encoding_rs::SHIFT_JIS;

    fn icon_sys(title: &str, line_break: usize) -> Vec<u8> {
        let mut bytes = vec![0_u8; 148];
        bytes[..4].copy_from_slice(b"PS2D");
        bytes[6..8].copy_from_slice(&(line_break as u16).to_le_bytes());
        let (encoded, _, had_errors) = SHIFT_JIS.encode(title);
        assert!(!had_errors);
        bytes[80..80 + encoded.len()].copy_from_slice(&encoded);
        bytes
    }

    #[test]
    fn discovers_portable_folder_card_member_and_regular_state() {
        let directory = tempfile::tempdir().unwrap();
        let emulator_root = directory.path().join("PCSX2");
        let card = emulator_root.join("memcards/Mcd001.ps2");
        let member = card.join("BASLUS-12345SAVE");
        let states = emulator_root.join("sstates");
        fs::create_dir_all(&member).unwrap();
        fs::create_dir_all(&states).unwrap();
        let emulator = emulator_root.join("pcsx2-qt");
        fs::write(&emulator, b"pcsx2").unwrap();
        fs::write(
            emulator_root.join("memcards/Mcd002.ps2"),
            b"raw card remains gated",
        )
        .unwrap();
        fs::write(member.join("icon.sys"), icon_sys("FixtureRacer", 7)).unwrap();
        fs::write(member.join("save.bin"), b"save bytes").unwrap();
        fs::write(states.join("SLUS-12345 (DeAdBeEf).03.p2s"), b"state bytes").unwrap();
        fs::write(states.join("SLES-99999 (11111111).04.p2z"), b"other state").unwrap();
        let content = directory.path().join("Fixture Racer (SLUS-12345).iso");
        fs::write(&content, b"disc bytes").unwrap();

        let roots = default_pcsx2_data_directories(&emulator);
        assert_eq!(roots, vec![fs::canonicalize(&emulator_root).unwrap()]);
        let saves = discover_pcsx2_saves(
            &emulator,
            &[Pcsx2Content {
                game_id: "game".into(),
                additional_application_id: None,
                content_path: content,
                title: "Fixture Racer".into(),
                alternate_titles: Vec::new(),
            }],
            &roots,
        )
        .unwrap();

        assert_eq!(saves.len(), 2);
        let card_save = saves
            .iter()
            .find(|save| save.container_save.is_some())
            .unwrap();
        assert_eq!(
            card_save.save_group_id.as_deref(),
            Some("pcsx2:Mcd001:BASLUS-12345SAVE")
        );
        assert_eq!(card_save.save_group_name, "My Save File");
        let metadata = card_save.container_save.as_ref().unwrap();
        assert_eq!(metadata.original_file_name, "BASLUS-12345SAVE");
        assert!(metadata.reported_file_size_bytes.unwrap() > 0);
        assert!(metadata.reported_last_modified.is_some());

        let state = saves.iter().find(|save| save.slot() == Some(3)).unwrap();
        assert!(state.primary_path.ends_with("SLUS-12345 (DeAdBeEf).03.p2s"));
        assert_eq!(
            state.save_group_id.as_deref(),
            Some("pcsx2-state:SLUS12345:03")
        );
        assert!(state.container_save.is_none());
    }

    #[test]
    fn uses_game_index_and_title_matching_without_cross_assigning_serials() {
        let directory = tempfile::tempdir().unwrap();
        let emulator_root = directory.path().join("PCSX2");
        let card = emulator_root.join("memcards/Shared.ps2");
        fs::create_dir_all(card.join("SLUS-11111SAVE")).unwrap();
        fs::create_dir_all(card.join("MYSTERY-RACER-DATA")).unwrap();
        fs::create_dir_all(emulator_root.join("resources")).unwrap();
        let emulator = emulator_root.join("pcsx2");
        fs::write(&emulator, b"pcsx2").unwrap();
        fs::write(
            emulator_root.join("resources/GameIndex.yaml"),
            "SLUS-11111:\n  name: \"First Adventure\"\n",
        )
        .unwrap();
        fs::write(card.join("SLUS-11111SAVE/data"), b"first").unwrap();
        fs::write(card.join("MYSTERY-RACER-DATA/data"), b"second").unwrap();
        let first = directory.path().join("unknown-one.iso");
        let second = directory.path().join("unknown-two.iso");
        fs::write(&first, b"one").unwrap();
        fs::write(&second, b"two").unwrap();

        let saves = discover_pcsx2_saves(
            &emulator,
            &[
                Pcsx2Content {
                    game_id: "first".into(),
                    additional_application_id: None,
                    content_path: first,
                    title: "First Adventure".into(),
                    alternate_titles: Vec::new(),
                },
                Pcsx2Content {
                    game_id: "second".into(),
                    additional_application_id: Some("disc-two".into()),
                    content_path: second,
                    title: "Mystery Racer".into(),
                    alternate_titles: vec!["Mystery Racer Data".into()],
                },
            ],
            &[emulator_root],
        )
        .unwrap();

        assert_eq!(saves.len(), 2);
        assert!(saves.iter().any(|save| {
            save.game_id == "first"
                && save
                    .container_save
                    .as_ref()
                    .is_some_and(|member| member.original_file_name == "SLUS-11111SAVE")
        }));
        assert!(saves.iter().any(|save| {
            save.game_id == "second"
                && save.additional_application_id.as_deref() == Some("disc-two")
                && save
                    .container_save
                    .as_ref()
                    .is_some_and(|member| member.original_file_name == "MYSTERY-RACER-DATA")
        }));
    }

    #[test]
    fn parses_1327_state_names_and_strict_title_similarity() {
        assert_eq!(
            parse_state_name("baSLUS-20312 (00ab12CD).09.P2Z"),
            Some(StateName {
                serial: "BASLUS20312".into(),
                crc: "00AB12CD".into(),
                slot: 9,
            })
        );
        assert!(parse_state_name("SLUS-20312 (00AB12CD).9.p2s").is_none());
        assert!(parse_state_name("../SLUS-20312 (00AB12CD).09.p2s").is_none());
        assert!(titles_similar("Gran Turismo 4", "Gran Turismo 4 (Europe)"));
        assert!(!titles_similar("Gran Turismo 3", "Gran Turismo 4"));
        assert_eq!(
            extract_system_cnf_serial(b"BOOT2 = cdrom0:\\\\SLUS_203.12;1"),
            Some("SLUS20312".into())
        );
    }
}
