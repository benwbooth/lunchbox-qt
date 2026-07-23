use chrono::{Duration, NaiveDate, TimeZone, Utc};
use encoding_rs::SHIFT_JIS;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;
use thiserror::Error;

const MAGIC: &[u8; 28] = b"Sony PS2 Memory Card Format ";
const DIRENT_LENGTH: usize = 512;
const FAT_ALLOCATED: u32 = 0x8000_0000;
const FAT_CLUSTER_MASK: u32 = 0x7fff_ffff;
const FAT_CHAIN_END: u32 = u32::MAX;
const MAX_DIRECTORY_ENTRIES: u32 = 4096;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pcsx2MemoryCardSave {
    pub directory_name: String,
    pub title: String,
    pub has_icon_sys: bool,
    pub total_bytes: i64,
    pub modified: Option<SystemTime>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtractedPcsx2MemoryCardSave {
    pub save: Pcsx2MemoryCardSave,
    pub files: Vec<PathBuf>,
    /// LaunchBox 13.27's uppercase SHA-256 folder-manifest signature.
    pub signature: String,
}

#[derive(Clone, Debug)]
struct MemoryCardHeader {
    page_size: usize,
    pages_per_cluster: usize,
    clusters_per_card: usize,
    allocatable_cluster_offset: usize,
    allocatable_cluster_end: usize,
    cluster_size: usize,
    spare_size: usize,
    raw_page_size: usize,
    raw_cluster_size: usize,
    indirect_fat_clusters: [i32; 32],
    root_dir_cluster: u32,
}

#[derive(Clone, Debug)]
struct Dirent {
    mode: u16,
    length: u32,
    first_cluster: u32,
    name: String,
    modified: Option<SystemTime>,
}

impl Dirent {
    fn exists(&self) -> bool {
        self.mode & 0x8000 != 0
    }

    fn is_directory(&self) -> bool {
        self.exists() && self.mode & 0x20 != 0
    }

    fn is_file(&self) -> bool {
        self.exists() && self.mode & 0x10 != 0
    }

    fn is_directory_only(&self) -> bool {
        self.exists() && self.mode & 0x30 == 0x20
    }
}

struct RawMemoryCard {
    path: PathBuf,
    file: File,
    header: MemoryCardHeader,
    fat: Vec<u32>,
    use_raw_cluster_size: bool,
    root: Vec<Dirent>,
}

impl RawMemoryCard {
    fn open(path: &Path) -> Result<Self, Pcsx2CardError> {
        let path = canonical_regular_file(path, "PCSX2 raw memory card")?;
        let mut file = File::open(&path).map_err(|source| Pcsx2CardError::Read {
            path: path.clone(),
            source,
        })?;
        let header = read_header(&mut file, &path)?;
        let use_raw_cluster_size = header.spare_size > 0;
        let fat = read_fat(&mut file, &path, &header, use_raw_cluster_size)?;
        let root_count = root_entry_count(&mut file, &path, &header, use_raw_cluster_size)?;
        let root = read_directory(
            &mut file,
            &path,
            &header,
            &fat,
            header.root_dir_cluster,
            root_count,
            use_raw_cluster_size,
        )?;
        if root.len() < 2
            || root[0].name != "."
            || !root[0].is_directory()
            || root[1].name != ".."
            || !root[1].is_directory()
        {
            return Err(Pcsx2CardError::InvalidFilesystem {
                path,
                message: "root '.' and '..' entries are missing or invalid".into(),
            });
        }
        Ok(Self {
            path,
            file,
            header,
            fat,
            use_raw_cluster_size,
            root,
        })
    }

    fn list_saves(&mut self) -> Result<Vec<Pcsx2MemoryCardSave>, Pcsx2CardError> {
        let mut saves = Vec::new();
        let mut first_clusters = BTreeSet::new();
        for entry in self.root.clone() {
            if !entry.exists()
                || !entry.is_directory()
                || entry.name == "."
                || entry.name == ".."
                || !entry.is_directory_only()
                || entry.first_cluster == FAT_CHAIN_END
                || usize::try_from(entry.first_cluster)
                    .ok()
                    .is_none_or(|cluster| cluster >= self.fat.len())
                || entry.length == 0
                || !first_clusters.insert(entry.first_cluster)
            {
                continue;
            }
            let members = self.read_member_directory(&entry)?;
            saves.push(save_info(&entry, &members, |icon| self.read_file(icon))?);
        }
        Ok(saves)
    }

    fn member(&mut self, member_name: &str) -> Result<(Dirent, Vec<Dirent>), Pcsx2CardError> {
        validate_member_component(member_name)?;
        let matches = self
            .root
            .iter()
            .filter(|entry| {
                entry.exists()
                    && entry.is_directory_only()
                    && entry.name.eq_ignore_ascii_case(member_name)
            })
            .cloned()
            .collect::<Vec<_>>();
        let [entry] = matches.as_slice() else {
            return if matches.is_empty() {
                Err(Pcsx2CardError::MemberNotFound {
                    card: self.path.clone(),
                    member: member_name.to_string(),
                })
            } else {
                Err(Pcsx2CardError::AmbiguousMember {
                    card: self.path.clone(),
                    member: member_name.to_string(),
                })
            };
        };
        if entry.length == 0
            || usize::try_from(entry.first_cluster)
                .ok()
                .is_none_or(|cluster| cluster >= self.fat.len())
        {
            return Err(Pcsx2CardError::InvalidFilesystem {
                path: self.path.clone(),
                message: format!(
                    "save member {member_name} has an invalid directory cluster or length"
                ),
            });
        }
        let members = self.read_member_directory(entry)?;
        Ok((entry.clone(), members))
    }

    fn read_member_directory(&mut self, entry: &Dirent) -> Result<Vec<Dirent>, Pcsx2CardError> {
        read_directory(
            &mut self.file,
            &self.path,
            &self.header,
            &self.fat,
            entry.first_cluster,
            entry.length,
            self.use_raw_cluster_size,
        )
    }

    fn read_file(&mut self, entry: &Dirent) -> Result<Vec<u8>, Pcsx2CardError> {
        read_file(
            &mut self.file,
            &self.path,
            &self.header,
            &self.fat,
            entry.first_cluster,
            entry.length,
            self.use_raw_cluster_size,
        )
    }
}

pub fn list_pcsx2_memory_card_saves(
    card_path: &Path,
) -> Result<Vec<Pcsx2MemoryCardSave>, Pcsx2CardError> {
    let metadata = fs::symlink_metadata(card_path).map_err(|source| Pcsx2CardError::Read {
        path: card_path.to_path_buf(),
        source,
    })?;
    if metadata.file_type().is_symlink() {
        return Err(Pcsx2CardError::UnsupportedFileType {
            kind: "PCSX2 memory card",
            path: card_path.to_path_buf(),
        });
    }
    if metadata.file_type().is_dir() {
        list_folder_saves(card_path)
    } else if metadata.file_type().is_file() {
        RawMemoryCard::open(card_path)?.list_saves()
    } else {
        Err(Pcsx2CardError::UnsupportedFileType {
            kind: "PCSX2 memory card",
            path: card_path.to_path_buf(),
        })
    }
}

pub fn extract_pcsx2_memory_card_save(
    card_path: &Path,
    member_name: &str,
    destination: &Path,
) -> Result<ExtractedPcsx2MemoryCardSave, Pcsx2CardError> {
    validate_member_component(member_name)?;
    let destination_created = prepare_empty_destination(destination)?;
    let metadata = fs::symlink_metadata(card_path).map_err(|source| Pcsx2CardError::Read {
        path: card_path.to_path_buf(),
        source,
    })?;
    let result = if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
        extract_folder_member(card_path, member_name, destination)
    } else if metadata.file_type().is_file() && !metadata.file_type().is_symlink() {
        extract_raw_member(card_path, member_name, destination)
    } else {
        Err(Pcsx2CardError::UnsupportedFileType {
            kind: "PCSX2 memory card",
            path: card_path.to_path_buf(),
        })
    };
    if result.is_err() {
        let _ = clean_failed_destination(destination, destination_created);
    }
    result
}

fn list_folder_saves(card_path: &Path) -> Result<Vec<Pcsx2MemoryCardSave>, Pcsx2CardError> {
    let card_path = canonical_directory(card_path, "PCSX2 folder memory card")?;
    let mut saves = Vec::new();
    for entry in sorted_entries(&card_path)? {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|source| Pcsx2CardError::Read {
            path: path.clone(),
            source,
        })?;
        if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
            continue;
        }
        let directory_name = unicode_file_name(&path)?.to_string();
        let files = folder_member_files(&path)?;
        let mut total_bytes = 0_i64;
        let mut modified = None;
        let mut icon_title = None;
        for file in files {
            let metadata = fs::symlink_metadata(&file).map_err(|source| Pcsx2CardError::Read {
                path: file.clone(),
                source,
            })?;
            let length = i64::try_from(metadata.len())
                .map_err(|_| Pcsx2CardError::TooLarge { path: path.clone() })?;
            total_bytes = total_bytes
                .checked_add(length)
                .ok_or_else(|| Pcsx2CardError::TooLarge { path: path.clone() })?;
            if let Ok(candidate) = metadata.modified() {
                modified =
                    Some(modified.map_or(candidate, |current: SystemTime| current.max(candidate)));
            }
            if unicode_file_name(&file)?.eq_ignore_ascii_case("icon.sys") {
                let bytes = fs::read(&file).map_err(|source| Pcsx2CardError::Read {
                    path: file.clone(),
                    source,
                })?;
                icon_title = parse_icon_title(&bytes);
            }
        }
        saves.push(Pcsx2MemoryCardSave {
            title: icon_title.unwrap_or_else(|| directory_name.clone()),
            directory_name,
            has_icon_sys: saves_icon_exists(&path)?,
            total_bytes,
            modified,
        });
    }
    Ok(saves)
}

fn saves_icon_exists(member: &Path) -> Result<bool, Pcsx2CardError> {
    Ok(folder_member_files(member)?.iter().any(|path| {
        unicode_file_name(path).is_ok_and(|name| name.eq_ignore_ascii_case("icon.sys"))
    }))
}

fn extract_folder_member(
    card_path: &Path,
    member_name: &str,
    destination: &Path,
) -> Result<ExtractedPcsx2MemoryCardSave, Pcsx2CardError> {
    let card = canonical_directory(card_path, "PCSX2 folder memory card")?;
    let member = find_folder_member(&card, member_name)?;
    let files = folder_member_files(&member)?;
    let save = list_folder_saves(&card)?
        .into_iter()
        .find(|save| save.directory_name.eq_ignore_ascii_case(member_name))
        .ok_or_else(|| Pcsx2CardError::MemberNotFound {
            card: card.clone(),
            member: member_name.to_string(),
        })?;
    let mut extracted = Vec::new();
    for source in files {
        let name = unicode_file_name(&source)?;
        validate_file_component(name, &card)?;
        let target = destination.join(name);
        copy_new_file(&source, &target)?;
        extracted.push(target);
    }
    finish_extraction(save, extracted, destination)
}

fn extract_raw_member(
    card_path: &Path,
    member_name: &str,
    destination: &Path,
) -> Result<ExtractedPcsx2MemoryCardSave, Pcsx2CardError> {
    let mut card = RawMemoryCard::open(card_path)?;
    let (directory, members) = card.member(member_name)?;
    let save = save_info(&directory, &members, |icon| card.read_file(icon))?;
    let mut extracted = Vec::new();
    let mut names = BTreeSet::new();
    for file in members.iter().filter(|entry| entry.is_file()) {
        validate_file_component(&file.name, &card.path)?;
        if !names.insert(file.name.to_ascii_lowercase()) {
            return Err(Pcsx2CardError::InvalidFilesystem {
                path: card.path.clone(),
                message: format!("save member contains duplicate filename {}", file.name),
            });
        }
        let bytes = card.read_file(file)?;
        let target = destination.join(&file.name);
        write_new_file(&target, &bytes)?;
        extracted.push(target);
    }
    finish_extraction(save, extracted, destination)
}

fn finish_extraction(
    save: Pcsx2MemoryCardSave,
    mut files: Vec<PathBuf>,
    destination: &Path,
) -> Result<ExtractedPcsx2MemoryCardSave, Pcsx2CardError> {
    files.sort_by_key(|path| {
        path.file_name()
            .map(|name| name.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default()
    });
    if files.is_empty() {
        return Err(Pcsx2CardError::EmptyMember {
            card: destination.to_path_buf(),
            member: save.directory_name,
        });
    }
    let signature = folder_manifest_signature(destination)?;
    Ok(ExtractedPcsx2MemoryCardSave {
        save,
        files,
        signature,
    })
}

pub fn folder_manifest_signature(folder: &Path) -> Result<String, Pcsx2CardError> {
    let folder = canonical_directory(folder, "PCSX2 extracted save")?;
    let mut files = Vec::new();
    collect_manifest_files(&folder, &folder, &mut files)?;
    files.sort_by(|left, right| {
        left.0
            .to_ascii_lowercase()
            .cmp(&right.0.to_ascii_lowercase())
            .then_with(|| left.0.cmp(&right.0))
    });
    if files.is_empty() {
        return Err(Pcsx2CardError::EmptyManifest { path: folder });
    }
    let mut manifest = Vec::new();
    for (relative, path) in files {
        let mut file = File::open(&path).map_err(|source| Pcsx2CardError::Read {
            path: path.clone(),
            source,
        })?;
        let mut digest = Sha256::new();
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let count = file
                .read(&mut buffer)
                .map_err(|source| Pcsx2CardError::Read {
                    path: path.clone(),
                    source,
                })?;
            if count == 0 {
                break;
            }
            digest.update(&buffer[..count]);
        }
        writeln!(
            manifest,
            "{}|{:X}",
            relative.replace('\\', "/").to_lowercase(),
            digest.finalize()
        )
        .expect("writing to a Vec cannot fail");
    }
    Ok(format!("{:X}", Sha256::digest(manifest)))
}

fn collect_manifest_files(
    root: &Path,
    directory: &Path,
    files: &mut Vec<(String, PathBuf)>,
) -> Result<(), Pcsx2CardError> {
    for entry in sorted_entries(directory)? {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|source| Pcsx2CardError::Read {
            path: path.clone(),
            source,
        })?;
        if metadata.file_type().is_symlink() {
            return Err(Pcsx2CardError::UnsupportedFileType {
                kind: "PCSX2 manifest member",
                path,
            });
        }
        if metadata.file_type().is_dir() {
            collect_manifest_files(root, &path, files)?;
            continue;
        }
        if !metadata.file_type().is_file() {
            return Err(Pcsx2CardError::UnsupportedFileType {
                kind: "PCSX2 manifest member",
                path,
            });
        }
        let name = unicode_file_name(&path)?;
        if ["manifest.sha256", "desktop.ini", "thumbs.db"]
            .iter()
            .any(|excluded| name.eq_ignore_ascii_case(excluded))
        {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|_| Pcsx2CardError::UnsafeMemberName {
                name: path.display().to_string(),
            })?;
        if relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err(Pcsx2CardError::UnsafeMemberName {
                name: relative.display().to_string(),
            });
        }
        let relative = relative
            .to_str()
            .ok_or_else(|| Pcsx2CardError::NonUnicodePath { path: path.clone() })?
            .to_string();
        files.push((relative, path));
    }
    Ok(())
}

fn save_info<F>(
    directory: &Dirent,
    members: &[Dirent],
    mut read_icon: F,
) -> Result<Pcsx2MemoryCardSave, Pcsx2CardError>
where
    F: FnMut(&Dirent) -> Result<Vec<u8>, Pcsx2CardError>,
{
    let icon = members
        .iter()
        .find(|entry| entry.is_file() && entry.name.eq_ignore_ascii_case("icon.sys"));
    let icon_title = icon
        .map(&mut read_icon)
        .transpose()?
        .and_then(|bytes| parse_icon_title(&bytes));
    let total_bytes = members
        .iter()
        .filter(|entry| entry.is_file())
        .try_fold(0_i64, |total, entry| {
            total.checked_add(i64::from(entry.length))
        })
        .ok_or_else(|| Pcsx2CardError::TooLarge {
            path: PathBuf::from(&directory.name),
        })?;
    let modified = members
        .iter()
        .filter(|entry| entry.exists() && (entry.is_file() || entry.is_directory()))
        .filter_map(|entry| entry.modified)
        .max();
    Ok(Pcsx2MemoryCardSave {
        title: icon_title.unwrap_or_else(|| {
            if directory.name.trim().is_empty() {
                "Save File".into()
            } else {
                directory.name.clone()
            }
        }),
        directory_name: directory.name.clone(),
        has_icon_sys: icon.is_some(),
        total_bytes,
        modified,
    })
}

pub(crate) fn parse_icon_title(data: &[u8]) -> Option<String> {
    if data.len() < 148 || data.get(..4)? != b"PS2D" {
        return None;
    }
    let line_break = usize::from(u16::from_le_bytes([data[6], data[7]]));
    let bytes = &data[80..148];
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    let (decoded, had_errors) = SHIFT_JIS.decode_without_bom_handling(&bytes[..end]);
    if had_errors {
        return None;
    }
    let mut characters = decoded.chars().collect::<Vec<_>>();
    if line_break > 0 && line_break < characters.len() {
        characters.insert(line_break, ' ');
    }
    let title = characters.into_iter().collect::<String>();
    let title = title.trim();
    (!title.is_empty()
        && title
            .chars()
            .all(|character| (' '..='~').contains(&character) && character != '?'))
    .then(|| title.to_string())
}

fn read_header(file: &mut File, path: &Path) -> Result<MemoryCardHeader, Pcsx2CardError> {
    let mut bytes = [0_u8; 340];
    read_exact_at(file, path, 0, &mut bytes)?;
    if bytes.get(..MAGIC.len()) != Some(MAGIC) {
        return Err(Pcsx2CardError::InvalidHeader {
            path: path.to_path_buf(),
            message: "Sony PS2 memory-card magic is missing".into(),
        });
    }
    let page_size = usize::from(read_u16(&bytes, 40));
    let pages_per_cluster = usize::from(read_u16(&bytes, 42));
    let clusters_per_card = positive_i32(&bytes, 48, path, "clusters per card")?;
    let allocatable_cluster_offset =
        nonnegative_i32(&bytes, 52, path, "allocatable cluster offset")?;
    let allocatable_cluster_end = positive_i32(&bytes, 56, path, "allocatable cluster end")?;
    let cluster_size =
        page_size
            .checked_mul(pages_per_cluster)
            .ok_or_else(|| Pcsx2CardError::InvalidHeader {
                path: path.to_path_buf(),
                message: "cluster size overflows".into(),
            })?;
    if page_size < DIRENT_LENGTH
        || pages_per_cluster == 0
        || cluster_size % DIRENT_LENGTH != 0
        || cluster_size > 1024 * 1024
        || clusters_per_card > 16 * 1024 * 1024
        || allocatable_cluster_end > clusters_per_card
        || allocatable_cluster_offset
            .checked_add(allocatable_cluster_end)
            .is_none_or(|end| end > clusters_per_card)
    {
        return Err(Pcsx2CardError::InvalidHeader {
            path: path.to_path_buf(),
            message: "page, cluster, or allocatable range is unsupported".into(),
        });
    }
    let length = usize::try_from(
        file.metadata()
            .map_err(|source| Pcsx2CardError::Read {
                path: path.to_path_buf(),
                source,
            })?
            .len(),
    )
    .map_err(|_| Pcsx2CardError::TooLarge {
        path: path.to_path_buf(),
    })?;
    let logical_length = cluster_size.checked_mul(clusters_per_card).ok_or_else(|| {
        Pcsx2CardError::InvalidHeader {
            path: path.to_path_buf(),
            message: "logical card size overflows".into(),
        }
    })?;
    if length < logical_length {
        return Err(Pcsx2CardError::InvalidHeader {
            path: path.to_path_buf(),
            message: format!(
                "card is truncated: expected at least {logical_length} bytes, found {length}"
            ),
        });
    }
    let mut raw_cluster_size = cluster_size;
    let mut raw_page_size = page_size;
    let mut spare_size = 0;
    if length > logical_length && length % clusters_per_card == 0 {
        let candidate_cluster_size = length / clusters_per_card;
        if candidate_cluster_size > cluster_size && candidate_cluster_size % pages_per_cluster == 0
        {
            let candidate_page_size = candidate_cluster_size / pages_per_cluster;
            if candidate_page_size > page_size {
                raw_cluster_size = candidate_cluster_size;
                raw_page_size = candidate_page_size;
                spare_size = candidate_page_size - page_size;
            }
        }
    }
    if length
        < raw_cluster_size
            .checked_mul(clusters_per_card)
            .ok_or_else(|| Pcsx2CardError::InvalidHeader {
                path: path.to_path_buf(),
                message: "physical card size overflows".into(),
            })?
    {
        return Err(Pcsx2CardError::InvalidHeader {
            path: path.to_path_buf(),
            message: "physical card image is truncated".into(),
        });
    }
    let mut indirect_fat_clusters = [0_i32; 32];
    for (index, entry) in indirect_fat_clusters.iter_mut().enumerate() {
        *entry = read_i32(&bytes, 80 + index * 4);
    }
    Ok(MemoryCardHeader {
        page_size,
        pages_per_cluster,
        clusters_per_card,
        allocatable_cluster_offset,
        allocatable_cluster_end,
        cluster_size,
        spare_size,
        raw_page_size,
        raw_cluster_size,
        indirect_fat_clusters,
        root_dir_cluster: 0,
    })
}

fn read_fat(
    file: &mut File,
    path: &Path,
    header: &MemoryCardHeader,
    use_raw: bool,
) -> Result<Vec<u32>, Pcsx2CardError> {
    let mut fat = Vec::with_capacity(header.allocatable_cluster_end);
    let entries_per_cluster = header.cluster_size / 4;
    for indirect_cluster in header.indirect_fat_clusters {
        if indirect_cluster <= 0 {
            break;
        }
        let indirect_cluster =
            usize::try_from(indirect_cluster).map_err(|_| Pcsx2CardError::InvalidFilesystem {
                path: path.to_path_buf(),
                message: "negative indirect FAT cluster".into(),
            })?;
        let indirect = read_cluster(file, path, header, indirect_cluster, use_raw)?;
        for index in 0..entries_per_cluster {
            if fat.len() >= header.allocatable_cluster_end {
                break;
            }
            let direct_cluster = read_i32(&indirect, index * 4);
            if direct_cluster <= 0 {
                continue;
            }
            let direct_cluster =
                usize::try_from(direct_cluster).map_err(|_| Pcsx2CardError::InvalidFilesystem {
                    path: path.to_path_buf(),
                    message: "negative direct FAT cluster".into(),
                })?;
            let direct = read_cluster(file, path, header, direct_cluster, use_raw)?;
            let count = entries_per_cluster.min(header.allocatable_cluster_end - fat.len());
            fat.extend((0..count).map(|entry| read_u32(&direct, entry * 4)));
        }
        if fat.len() >= header.allocatable_cluster_end {
            break;
        }
    }
    if fat.len() != header.allocatable_cluster_end {
        return Err(Pcsx2CardError::InvalidFilesystem {
            path: path.to_path_buf(),
            message: format!(
                "FAT contains {} of {} required entries",
                fat.len(),
                header.allocatable_cluster_end
            ),
        });
    }
    Ok(fat)
}

fn root_entry_count(
    file: &mut File,
    path: &Path,
    header: &MemoryCardHeader,
    use_raw: bool,
) -> Result<u32, Pcsx2CardError> {
    let cluster = read_alloc_cluster(file, path, header, header.root_dir_cluster, use_raw)?;
    let entry = parse_dirent(&cluster[..DIRENT_LENGTH]);
    let count = if (2..=MAX_DIRECTORY_ENTRIES).contains(&entry.length) {
        entry.length
    } else {
        u32::try_from(header.cluster_size / DIRENT_LENGTH).expect("bounded cluster size")
    };
    Ok(count)
}

fn read_directory(
    file: &mut File,
    path: &Path,
    header: &MemoryCardHeader,
    fat: &[u32],
    first_cluster: u32,
    entry_count: u32,
    use_raw: bool,
) -> Result<Vec<Dirent>, Pcsx2CardError> {
    if entry_count == 0 || entry_count > MAX_DIRECTORY_ENTRIES {
        return Err(Pcsx2CardError::InvalidFilesystem {
            path: path.to_path_buf(),
            message: format!("unsupported directory entry count {entry_count}"),
        });
    }
    let needed = usize::try_from(entry_count)
        .expect("bounded entry count")
        .checked_mul(DIRENT_LENGTH)
        .expect("bounded directory size");
    let chain = enumerate_chain(fat, first_cluster, path)?;
    let mut bytes = Vec::with_capacity(needed);
    for cluster in chain {
        let data = read_alloc_cluster(
            file,
            path,
            header,
            u32::try_from(cluster).expect("FAT index fits u32"),
            use_raw,
        )?;
        let remaining = needed - bytes.len();
        bytes.extend_from_slice(&data[..remaining.min(data.len())]);
        if bytes.len() == needed {
            break;
        }
    }
    if bytes.len() != needed {
        return Err(Pcsx2CardError::InvalidFilesystem {
            path: path.to_path_buf(),
            message: format!(
                "directory chain ended after {} of {needed} bytes",
                bytes.len()
            ),
        });
    }
    Ok(bytes
        .chunks_exact(DIRENT_LENGTH)
        .map(parse_dirent)
        .collect())
}

fn read_file(
    file: &mut File,
    path: &Path,
    header: &MemoryCardHeader,
    fat: &[u32],
    first_cluster: u32,
    length: u32,
    use_raw: bool,
) -> Result<Vec<u8>, Pcsx2CardError> {
    let needed = usize::try_from(length).map_err(|_| Pcsx2CardError::TooLarge {
        path: path.to_path_buf(),
    })?;
    if needed == 0 {
        return Ok(Vec::new());
    }
    let chain = enumerate_chain(fat, first_cluster, path)?;
    let mut bytes = Vec::with_capacity(needed);
    for cluster in chain {
        let data = read_alloc_cluster(
            file,
            path,
            header,
            u32::try_from(cluster).expect("FAT index fits u32"),
            use_raw,
        )?;
        let remaining = needed - bytes.len();
        bytes.extend_from_slice(&data[..remaining.min(data.len())]);
        if bytes.len() == needed {
            break;
        }
    }
    if bytes.len() != needed {
        return Err(Pcsx2CardError::InvalidFilesystem {
            path: path.to_path_buf(),
            message: format!("file chain ended after {} of {needed} bytes", bytes.len()),
        });
    }
    Ok(bytes)
}

fn enumerate_chain(fat: &[u32], start: u32, path: &Path) -> Result<Vec<usize>, Pcsx2CardError> {
    let mut current = start;
    let mut seen = BTreeSet::new();
    let mut chain = Vec::new();
    while current != FAT_CHAIN_END {
        let index = usize::try_from(current).map_err(|_| Pcsx2CardError::InvalidFilesystem {
            path: path.to_path_buf(),
            message: format!("cluster {current} cannot be represented on this host"),
        })?;
        if index >= fat.len() {
            return Err(Pcsx2CardError::InvalidFilesystem {
                path: path.to_path_buf(),
                message: format!("cluster {current} is outside the FAT"),
            });
        }
        if !seen.insert(current) {
            return Err(Pcsx2CardError::InvalidFilesystem {
                path: path.to_path_buf(),
                message: format!("FAT chain contains a cycle at cluster {current}"),
            });
        }
        chain.push(index);
        let next = fat[index];
        if next == FAT_CHAIN_END || next & FAT_ALLOCATED == 0 {
            break;
        }
        current = next & FAT_CLUSTER_MASK;
    }
    Ok(chain)
}

fn read_alloc_cluster(
    file: &mut File,
    path: &Path,
    header: &MemoryCardHeader,
    alloc_cluster: u32,
    use_raw: bool,
) -> Result<Vec<u8>, Pcsx2CardError> {
    let alloc_cluster =
        usize::try_from(alloc_cluster).map_err(|_| Pcsx2CardError::InvalidFilesystem {
            path: path.to_path_buf(),
            message: "allocatable cluster index is too large".into(),
        })?;
    if alloc_cluster >= header.allocatable_cluster_end {
        return Err(Pcsx2CardError::InvalidFilesystem {
            path: path.to_path_buf(),
            message: format!("allocatable cluster {alloc_cluster} is outside the FAT"),
        });
    }
    let physical = header
        .allocatable_cluster_offset
        .checked_add(alloc_cluster)
        .filter(|cluster| *cluster < header.clusters_per_card)
        .ok_or_else(|| Pcsx2CardError::InvalidFilesystem {
            path: path.to_path_buf(),
            message: "allocatable cluster maps outside the card".into(),
        })?;
    read_cluster(file, path, header, physical, use_raw)
}

fn read_cluster(
    file: &mut File,
    path: &Path,
    header: &MemoryCardHeader,
    cluster: usize,
    use_raw: bool,
) -> Result<Vec<u8>, Pcsx2CardError> {
    if cluster >= header.clusters_per_card {
        return Err(Pcsx2CardError::InvalidFilesystem {
            path: path.to_path_buf(),
            message: format!("physical cluster {cluster} is outside the card"),
        });
    }
    let physical_size = if use_raw {
        header.raw_cluster_size
    } else {
        header.cluster_size
    };
    let offset = cluster
        .checked_mul(physical_size)
        .and_then(|offset| u64::try_from(offset).ok())
        .ok_or_else(|| Pcsx2CardError::TooLarge {
            path: path.to_path_buf(),
        })?;
    if !use_raw {
        let mut data = vec![0_u8; header.cluster_size];
        read_exact_at(file, path, offset, &mut data)?;
        return Ok(data);
    }
    let mut raw = vec![0_u8; header.raw_cluster_size];
    read_exact_at(file, path, offset, &mut raw)?;
    let mut data = vec![0_u8; header.cluster_size];
    for page in 0..header.pages_per_cluster {
        let raw_start = page * header.raw_page_size;
        let logical_start = page * header.page_size;
        data[logical_start..logical_start + header.page_size]
            .copy_from_slice(&raw[raw_start..raw_start + header.page_size]);
    }
    Ok(data)
}

fn parse_dirent(bytes: &[u8]) -> Dirent {
    let name_bytes = &bytes[64..DIRENT_LENGTH];
    let end = name_bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(name_bytes.len());
    Dirent {
        mode: read_u16(bytes, 0),
        length: read_u32(bytes, 4),
        first_cluster: read_u32(bytes, 16),
        name: String::from_utf8_lossy(&name_bytes[..end]).into_owned(),
        modified: parse_tod(bytes, 24),
    }
}

fn parse_tod(bytes: &[u8], offset: usize) -> Option<SystemTime> {
    let second = u32::from(*bytes.get(offset + 1)?);
    let minute = u32::from(*bytes.get(offset + 2)?);
    let hour = u32::from(*bytes.get(offset + 3)?);
    let day = u32::from(*bytes.get(offset + 4)?).max(1);
    let month = u32::from(*bytes.get(offset + 5)?).max(1);
    let year = i32::from(read_u16(bytes, offset + 6));
    if !(1970..=9999).contains(&year) {
        return None;
    }
    let naive = NaiveDate::from_ymd_opt(year, month, day)?.and_hms_opt(hour, minute, second)?;
    let utc = Utc.from_utc_datetime(&naive) - Duration::hours(9);
    Some(utc.into())
}

fn find_folder_member(card: &Path, member_name: &str) -> Result<PathBuf, Pcsx2CardError> {
    let matches = sorted_entries(card)?
        .into_iter()
        .filter_map(|entry| {
            let path = entry.path();
            let name = entry.file_name();
            name.to_str()
                .is_some_and(|name| name.eq_ignore_ascii_case(member_name))
                .then_some(path)
        })
        .filter(|path| {
            fs::symlink_metadata(path).ok().is_some_and(|metadata| {
                metadata.file_type().is_dir() && !metadata.file_type().is_symlink()
            })
        })
        .collect::<Vec<_>>();
    let [member] = matches.as_slice() else {
        return if matches.is_empty() {
            Err(Pcsx2CardError::MemberNotFound {
                card: card.to_path_buf(),
                member: member_name.to_string(),
            })
        } else {
            Err(Pcsx2CardError::AmbiguousMember {
                card: card.to_path_buf(),
                member: member_name.to_string(),
            })
        };
    };
    let member = fs::canonicalize(member).map_err(|source| Pcsx2CardError::Read {
        path: member.clone(),
        source,
    })?;
    if member.parent() != Some(card) {
        return Err(Pcsx2CardError::UnsafeMemberName {
            name: member_name.to_string(),
        });
    }
    Ok(member)
}

fn folder_member_files(member: &Path) -> Result<Vec<PathBuf>, Pcsx2CardError> {
    let mut files = Vec::new();
    for entry in sorted_entries(member)? {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|source| Pcsx2CardError::Read {
            path: path.clone(),
            source,
        })?;
        if metadata.file_type().is_file() && !metadata.file_type().is_symlink() {
            files.push(path);
        } else if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
            return Err(Pcsx2CardError::UnsupportedFileType {
                kind: "PCSX2 folder-card member",
                path,
            });
        }
    }
    Ok(files)
}

fn prepare_empty_destination(destination: &Path) -> Result<bool, Pcsx2CardError> {
    match fs::symlink_metadata(destination) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => fs::create_dir(destination)
            .map_err(|source| Pcsx2CardError::Write {
                path: destination.to_path_buf(),
                source,
            })
            .map(|()| true),
        Ok(metadata) if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() => {
            let mut entries = fs::read_dir(destination).map_err(|source| Pcsx2CardError::Read {
                path: destination.to_path_buf(),
                source,
            })?;
            if entries.next().is_none() {
                Ok(false)
            } else {
                Err(Pcsx2CardError::DestinationNotEmpty {
                    path: destination.to_path_buf(),
                })
            }
        }
        Ok(_) => Err(Pcsx2CardError::UnsupportedFileType {
            kind: "PCSX2 extraction destination",
            path: destination.to_path_buf(),
        }),
        Err(source) => Err(Pcsx2CardError::Read {
            path: destination.to_path_buf(),
            source,
        }),
    }
}

fn clean_failed_destination(
    destination: &Path,
    destination_created: bool,
) -> Result<(), std::io::Error> {
    if destination_created {
        return fs::remove_dir_all(destination);
    }
    if destination.is_dir() {
        for entry in fs::read_dir(destination)? {
            let entry = entry?;
            let metadata = fs::symlink_metadata(entry.path())?;
            if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
                fs::remove_dir_all(entry.path())?;
            } else {
                fs::remove_file(entry.path())?;
            }
        }
    }
    Ok(())
}

fn copy_new_file(source: &Path, target: &Path) -> Result<(), Pcsx2CardError> {
    let mut input = File::open(source).map_err(|source_error| Pcsx2CardError::Read {
        path: source.to_path_buf(),
        source: source_error,
    })?;
    let mut output = open_new_file(target)?;
    std::io::copy(&mut input, &mut output).map_err(|source| Pcsx2CardError::Write {
        path: target.to_path_buf(),
        source,
    })?;
    output.sync_all().map_err(|source| Pcsx2CardError::Write {
        path: target.to_path_buf(),
        source,
    })
}

fn write_new_file(target: &Path, bytes: &[u8]) -> Result<(), Pcsx2CardError> {
    let mut file = open_new_file(target)?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|source| Pcsx2CardError::Write {
            path: target.to_path_buf(),
            source,
        })
}

fn open_new_file(path: &Path) -> Result<File, Pcsx2CardError> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|source| Pcsx2CardError::Write {
            path: path.to_path_buf(),
            source,
        })
}

fn validate_member_component(name: &str) -> Result<(), Pcsx2CardError> {
    if name.trim().is_empty()
        || name == "."
        || name == ".."
        || name.contains(['/', '\\'])
        || Path::new(name).components().count() != 1
    {
        return Err(Pcsx2CardError::UnsafeMemberName {
            name: name.to_string(),
        });
    }
    Ok(())
}

fn validate_file_component(name: &str, card: &Path) -> Result<(), Pcsx2CardError> {
    validate_member_component(name).map_err(|_| Pcsx2CardError::InvalidFilesystem {
        path: card.to_path_buf(),
        message: format!("save member contains unsafe filename {name:?}"),
    })
}

fn sorted_entries(path: &Path) -> Result<Vec<fs::DirEntry>, Pcsx2CardError> {
    let mut entries = fs::read_dir(path)
        .map_err(|source| Pcsx2CardError::Read {
            path: path.to_path_buf(),
            source,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| Pcsx2CardError::Read {
            path: path.to_path_buf(),
            source,
        })?;
    entries.sort_by_key(|entry| entry.file_name().to_string_lossy().to_ascii_lowercase());
    Ok(entries)
}

fn canonical_regular_file(path: &Path, kind: &'static str) -> Result<PathBuf, Pcsx2CardError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| Pcsx2CardError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(Pcsx2CardError::UnsupportedFileType {
            kind,
            path: path.to_path_buf(),
        });
    }
    fs::canonicalize(path).map_err(|source| Pcsx2CardError::Read {
        path: path.to_path_buf(),
        source,
    })
}

fn canonical_directory(path: &Path, kind: &'static str) -> Result<PathBuf, Pcsx2CardError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| Pcsx2CardError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(Pcsx2CardError::UnsupportedFileType {
            kind,
            path: path.to_path_buf(),
        });
    }
    fs::canonicalize(path).map_err(|source| Pcsx2CardError::Read {
        path: path.to_path_buf(),
        source,
    })
}

fn unicode_file_name(path: &Path) -> Result<&str, Pcsx2CardError> {
    path.file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| Pcsx2CardError::NonUnicodePath {
            path: path.to_path_buf(),
        })
}

fn read_exact_at(
    file: &mut File,
    path: &Path,
    offset: u64,
    bytes: &mut [u8],
) -> Result<(), Pcsx2CardError> {
    file.seek(SeekFrom::Start(offset))
        .and_then(|_| file.read_exact(bytes))
        .map_err(|source| Pcsx2CardError::Read {
            path: path.to_path_buf(),
            source,
        })
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn read_i32(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn positive_i32(
    bytes: &[u8],
    offset: usize,
    path: &Path,
    name: &str,
) -> Result<usize, Pcsx2CardError> {
    let value = read_i32(bytes, offset);
    usize::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| Pcsx2CardError::InvalidHeader {
            path: path.to_path_buf(),
            message: format!("{name} is not positive"),
        })
}

fn nonnegative_i32(
    bytes: &[u8],
    offset: usize,
    path: &Path,
    name: &str,
) -> Result<usize, Pcsx2CardError> {
    usize::try_from(read_i32(bytes, offset)).map_err(|_| Pcsx2CardError::InvalidHeader {
        path: path.to_path_buf(),
        message: format!("{name} is negative"),
    })
}

#[derive(Debug, Error)]
pub enum Pcsx2CardError {
    #[error("could not read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not write {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{kind} has an unsupported file type: {path}")]
    UnsupportedFileType { kind: &'static str, path: PathBuf },
    #[error("path is not valid Unicode: {path}")]
    NonUnicodePath { path: PathBuf },
    #[error("memory-card metadata is too large: {path}")]
    TooLarge { path: PathBuf },
    #[error("invalid PCSX2 memory-card header at {path}: {message}")]
    InvalidHeader { path: PathBuf, message: String },
    #[error("invalid PCSX2 memory-card filesystem at {path}: {message}")]
    InvalidFilesystem { path: PathBuf, message: String },
    #[error("unsafe PCSX2 memory-card member name: {name:?}")]
    UnsafeMemberName { name: String },
    #[error("PCSX2 memory card {card} has no member named {member}")]
    MemberNotFound { card: PathBuf, member: String },
    #[error("PCSX2 memory card {card} has multiple members named {member}")]
    AmbiguousMember { card: PathBuf, member: String },
    #[error("PCSX2 memory-card member {member} at {card} contains no files")]
    EmptyMember { card: PathBuf, member: String },
    #[error("PCSX2 extraction destination is not empty: {path}")]
    DestinationNotEmpty { path: PathBuf },
    #[error("PCSX2 save manifest contains no eligible files: {path}")]
    EmptyManifest { path: PathBuf },
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAGE_SIZE: usize = 512;
    const PAGES_PER_CLUSTER: usize = 4;
    const CLUSTER_SIZE: usize = PAGE_SIZE * PAGES_PER_CLUSTER;
    const SPARE_SIZE: usize = 16;
    const RAW_PAGE_SIZE: usize = PAGE_SIZE + SPARE_SIZE;
    const RAW_CLUSTER_SIZE: usize = RAW_PAGE_SIZE * PAGES_PER_CLUSTER;
    const CLUSTERS_PER_CARD: usize = 64;
    const ALLOCATABLE_OFFSET: usize = 4;
    const ALLOCATABLE_END: usize = 32;

    fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
        bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn write_i32(bytes: &mut [u8], offset: usize, value: i32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn dirent(mode: u16, length: u32, first_cluster: u32, name: &str) -> [u8; 512] {
        let mut bytes = [0_u8; 512];
        write_u16(&mut bytes, 0, mode);
        write_u32(&mut bytes, 4, length);
        write_u32(&mut bytes, 16, first_cluster);
        bytes[25] = 1;
        bytes[26] = 2;
        bytes[27] = 3;
        bytes[28] = 4;
        bytes[29] = 5;
        write_u16(&mut bytes, 30, 2024);
        bytes[64..64 + name.len()].copy_from_slice(name.as_bytes());
        bytes
    }

    fn icon_sys(title: &str) -> Vec<u8> {
        let mut bytes = vec![0_u8; 148];
        bytes[..4].copy_from_slice(b"PS2D");
        bytes[6..8].copy_from_slice(&7_u16.to_le_bytes());
        bytes[80..80 + title.len()].copy_from_slice(title.as_bytes());
        bytes
    }

    fn put_cluster(card: &mut [u8], cluster: usize, data: &[u8], raw: bool) {
        if raw {
            let base = cluster * RAW_CLUSTER_SIZE;
            for page in 0..PAGES_PER_CLUSTER {
                let source = page * PAGE_SIZE;
                let target = base + page * RAW_PAGE_SIZE;
                card[target..target + PAGE_SIZE].copy_from_slice(&data[source..source + PAGE_SIZE]);
                card[target + PAGE_SIZE..target + RAW_PAGE_SIZE].fill(0xff);
            }
        } else {
            let base = cluster * CLUSTER_SIZE;
            card[base..base + CLUSTER_SIZE].copy_from_slice(data);
        }
    }

    fn card_fixture(raw: bool) -> Vec<u8> {
        let physical_cluster_size = if raw { RAW_CLUSTER_SIZE } else { CLUSTER_SIZE };
        let mut card = vec![0xff_u8; CLUSTERS_PER_CARD * physical_cluster_size];
        let mut header_cluster = vec![0xff_u8; CLUSTER_SIZE];
        header_cluster[..MAGIC.len()].copy_from_slice(MAGIC);
        write_u16(&mut header_cluster, 40, PAGE_SIZE as u16);
        write_u16(&mut header_cluster, 42, PAGES_PER_CLUSTER as u16);
        write_u16(&mut header_cluster, 44, 16);
        write_i32(&mut header_cluster, 48, CLUSTERS_PER_CARD as i32);
        write_i32(&mut header_cluster, 52, ALLOCATABLE_OFFSET as i32);
        write_i32(&mut header_cluster, 56, ALLOCATABLE_END as i32);
        write_i32(&mut header_cluster, 60, 0);
        write_i32(&mut header_cluster, 64, 62);
        write_i32(&mut header_cluster, 68, 61);
        write_i32(&mut header_cluster, 80, 1);
        put_cluster(&mut card, 0, &header_cluster, raw);

        let mut indirect = vec![0_u8; CLUSTER_SIZE];
        write_i32(&mut indirect, 0, 2);
        put_cluster(&mut card, 1, &indirect, raw);

        let mut fat = vec![0_u8; CLUSTER_SIZE];
        for cluster in 0..=3 {
            write_u32(&mut fat, cluster * 4, FAT_CHAIN_END);
        }
        put_cluster(&mut card, 2, &fat, raw);

        let mut root = vec![0_u8; CLUSTER_SIZE];
        root[..512].copy_from_slice(&dirent(0x8427, 3, 0, "."));
        root[512..1024].copy_from_slice(&dirent(0x8427, 0, 0, ".."));
        root[1024..1536].copy_from_slice(&dirent(0x8427, 4, 1, "BASLUS-12345SAVE"));
        put_cluster(&mut card, ALLOCATABLE_OFFSET, &root, raw);

        let mut member = vec![0_u8; CLUSTER_SIZE];
        member[..512].copy_from_slice(&dirent(0x8427, 4, 1, "."));
        member[512..1024].copy_from_slice(&dirent(0x8427, 0, 0, ".."));
        member[1024..1536].copy_from_slice(&dirent(0x8417, 148, 2, "icon.sys"));
        member[1536..2048].copy_from_slice(&dirent(0x8417, 10, 3, "save.bin"));
        put_cluster(&mut card, ALLOCATABLE_OFFSET + 1, &member, raw);

        let mut icon = vec![0_u8; CLUSTER_SIZE];
        icon[..148].copy_from_slice(&icon_sys("FixtureRacer"));
        put_cluster(&mut card, ALLOCATABLE_OFFSET + 2, &icon, raw);
        let mut save = vec![0_u8; CLUSTER_SIZE];
        save[..10].copy_from_slice(b"save bytes");
        put_cluster(&mut card, ALLOCATABLE_OFFSET + 3, &save, raw);
        card
    }

    #[test]
    fn lists_and_extracts_logical_and_raw_ps2_card_members() {
        for raw in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let card = directory
                .path()
                .join(if raw { "raw.ps2" } else { "logical.ps2" });
            fs::write(&card, card_fixture(raw)).unwrap();
            let saves = list_pcsx2_memory_card_saves(&card).unwrap();
            assert_eq!(saves.len(), 1);
            assert_eq!(saves[0].directory_name, "BASLUS-12345SAVE");
            assert_eq!(saves[0].title, "Fixture Racer");
            assert!(saves[0].has_icon_sys);
            assert_eq!(saves[0].total_bytes, 158);
            assert!(saves[0].modified.is_some());

            let destination = directory.path().join("extracted");
            let extracted =
                extract_pcsx2_memory_card_save(&card, "baslus-12345save", &destination).unwrap();
            assert_eq!(extracted.save, saves[0]);
            assert_eq!(
                fs::read(destination.join("save.bin")).unwrap(),
                b"save bytes"
            );
            assert_eq!(extracted.files.len(), 2);
            assert_eq!(extracted.signature.len(), 64);
            assert!(extracted
                .signature
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_lowercase()));
        }
    }

    #[test]
    fn folder_and_raw_members_share_the_same_manifest_contract() {
        let directory = tempfile::tempdir().unwrap();
        let card = directory.path().join("raw.ps2");
        fs::write(&card, card_fixture(true)).unwrap();
        let extracted = directory.path().join("raw-member");
        let raw = extract_pcsx2_memory_card_save(&card, "BASLUS-12345SAVE", &extracted).unwrap();

        let folder_card = directory.path().join("Folder.ps2/BASLUS-12345SAVE");
        fs::create_dir_all(&folder_card).unwrap();
        fs::copy(extracted.join("icon.sys"), folder_card.join("icon.sys")).unwrap();
        fs::copy(extracted.join("save.bin"), folder_card.join("save.bin")).unwrap();
        let folder_destination = directory.path().join("folder-member");
        let folder = extract_pcsx2_memory_card_save(
            folder_card.parent().unwrap(),
            "BASLUS-12345SAVE",
            &folder_destination,
        )
        .unwrap();

        assert_eq!(raw.signature, folder.signature);
        assert_eq!(raw.save.total_bytes, folder.save.total_bytes);
        assert_eq!(raw.save.title, folder.save.title);
    }

    #[test]
    fn raw_card_member_flows_through_pcsx2_discovery_with_container_identity() {
        let directory = tempfile::tempdir().unwrap();
        let emulator_root = directory.path().join("PCSX2");
        let memcards = emulator_root.join("memcards");
        fs::create_dir_all(&memcards).unwrap();
        let card = memcards.join("Mcd001.ps2");
        fs::write(&card, card_fixture(true)).unwrap();
        let emulator = emulator_root.join("pcsx2-qt");
        fs::write(&emulator, b"pcsx2").unwrap();
        let content = directory.path().join("Fixture Racer (SLUS-12345).iso");
        fs::write(&content, b"disc bytes").unwrap();

        let saves = super::super::discover_pcsx2_saves(
            &emulator,
            &[super::super::Pcsx2Content {
                game_id: "game".into(),
                additional_application_id: None,
                content_path: content,
                title: "Fixture Racer".into(),
                alternate_titles: Vec::new(),
            }],
            &[emulator_root],
        )
        .unwrap();

        assert_eq!(saves.len(), 1);
        assert_eq!(saves[0].primary_path, fs::canonicalize(card).unwrap());
        assert_eq!(
            saves[0].save_group_id.as_deref(),
            Some("pcsx2:Mcd001:BASLUS-12345SAVE")
        );
        assert_eq!(
            saves[0]
                .container_save
                .as_ref()
                .map(|member| member.original_file_name.as_str()),
            Some("BASLUS-12345SAVE")
        );
    }

    #[test]
    fn extraction_rejects_traversal_and_symlink_members() {
        let directory = tempfile::tempdir().unwrap();
        let card = directory.path().join("Folder.ps2");
        let member = card.join("SAVE");
        fs::create_dir_all(&member).unwrap();
        fs::write(member.join("icon.sys"), icon_sys("Fixture")).unwrap();
        assert!(matches!(
            extract_pcsx2_memory_card_save(&card, "../SAVE", &directory.path().join("out")),
            Err(Pcsx2CardError::UnsafeMemberName { .. })
        ));
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink("/etc/passwd", member.join("escape")).unwrap();
            assert!(matches!(
                extract_pcsx2_memory_card_save(&card, "SAVE", &directory.path().join("out")),
                Err(Pcsx2CardError::UnsupportedFileType { .. })
            ));
        }
    }
}
