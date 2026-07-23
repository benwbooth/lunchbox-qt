use chd::{read::ChdReader, Chd};
use flate2::read::{DeflateDecoder, GzDecoder};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

const GZIP_DECOMPRESSED_LIMIT: u64 = 96 * 1024 * 1024;
const FILE_PREFIX_LIMIT: u64 = 1024 * 1024;
const LAYOUT_SCAN_LIMIT: u64 = 64 * 1024 * 1024;
const DIRECTORY_LIMIT: usize = 2 * 1024 * 1024;
const SYSTEM_CNF_LIMIT: usize = 128 * 1024;
const SIDECAR_LIMIT: u64 = 16 * 1024 * 1024;
const NRG_CHUNK_LIMIT: u64 = 4 * 1024 * 1024;
const CSO_INDEX_LIMIT: usize = 64 * 1024 * 1024;
const CSO_BLOCK_LIMIT: usize = 16 * 1024 * 1024;
const CSO_CACHE_LIMIT: usize = 64 * 1024 * 1024;
const CHD_HUNK_LIMIT: u32 = 64 * 1024 * 1024;
const CHD_MAP_LIMIT: u64 = 128 * 1024 * 1024;
const SECTOR_SIZES: [usize; 4] = [2048, 2352, 2336, 2448];
const DATA_OFFSETS: [usize; 4] = [0, 16, 24, 32];
const ISO_PRIMARY_VOLUME_DESCRIPTOR: [u8; 7] = [1, b'C', b'D', b'0', b'0', b'1', 1];

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct CacheKey {
    path: PathBuf,
    byte_len: u64,
    modified: Option<SystemTime>,
}

static SERIAL_CACHE: OnceLock<Mutex<HashMap<CacheKey, Option<String>>>> = OnceLock::new();

trait RomReader {
    fn len(&self) -> u64;
    fn supports_fast_scan(&self) -> bool;
    fn read_at(&mut self, offset: u64, buffer: &mut [u8]) -> Option<usize>;
}

struct FileRomReader {
    file: File,
    len: u64,
}

impl FileRomReader {
    fn open(path: &Path) -> Option<Self> {
        let file = File::open(path).ok()?;
        let len = file.metadata().ok()?.len();
        Some(Self { file, len })
    }
}

impl RomReader for FileRomReader {
    fn len(&self) -> u64 {
        self.len
    }

    fn supports_fast_scan(&self) -> bool {
        true
    }

    fn read_at(&mut self, offset: u64, buffer: &mut [u8]) -> Option<usize> {
        if offset >= self.len {
            return Some(0);
        }
        self.file.seek(SeekFrom::Start(offset)).ok()?;
        let count = usize::try_from((self.len - offset).min(buffer.len() as u64)).ok()?;
        self.file.read_exact(&mut buffer[..count]).ok()?;
        Some(count)
    }
}

struct MemoryRomReader {
    bytes: Vec<u8>,
}

impl RomReader for MemoryRomReader {
    fn len(&self) -> u64 {
        self.bytes.len() as u64
    }

    fn supports_fast_scan(&self) -> bool {
        false
    }

    fn read_at(&mut self, offset: u64, buffer: &mut [u8]) -> Option<usize> {
        let start = usize::try_from(offset).ok()?;
        let source = self.bytes.get(start..)?;
        let count = source.len().min(buffer.len());
        buffer[..count].copy_from_slice(&source[..count]);
        Some(count)
    }
}

struct CsoRomReader {
    file: File,
    file_len: u64,
    data_offset: u64,
    index: Vec<u32>,
    block_size: usize,
    align: u8,
    len: u64,
    block_cache: HashMap<usize, Vec<u8>>,
    block_cache_bytes: usize,
}

impl CsoRomReader {
    fn open(path: &Path) -> Option<Self> {
        let mut file = File::open(path).ok()?;
        let file_len = file.metadata().ok()?.len();
        let mut header = [0_u8; 24];
        file.read_exact(&mut header).ok()?;
        if &header[..4] != b"CISO" {
            return None;
        }
        let header_size = u32::from_le_bytes(header[4..8].try_into().ok()?) as u64;
        let len = u64::from_le_bytes(header[8..16].try_into().ok()?);
        let block_size = u32::from_le_bytes(header[16..20].try_into().ok()?) as usize;
        let align = header[21];
        if header_size < header.len() as u64
            || header_size > file_len
            || len == 0
            || block_size == 0
            || block_size > CSO_BLOCK_LIMIT
            || align > 31
        {
            return None;
        }
        let block_count = len
            .checked_add(block_size as u64 - 1)?
            .checked_div(block_size as u64)?
            .checked_add(1)?;
        let index_bytes = usize::try_from(block_count).ok()?.checked_mul(4)?;
        if index_bytes > CSO_INDEX_LIMIT {
            return None;
        }
        let data_offset = header_size.checked_add(index_bytes as u64)?;
        if data_offset > file_len {
            return None;
        }
        file.seek(SeekFrom::Start(header_size)).ok()?;
        let mut raw_index = vec![0_u8; index_bytes];
        file.read_exact(&mut raw_index).ok()?;
        let index = raw_index
            .chunks_exact(4)
            .map(|bytes| u32::from_le_bytes(bytes.try_into().expect("four-byte CSO index")))
            .collect();
        Some(Self {
            file,
            file_len,
            data_offset,
            index,
            block_size,
            align,
            len,
            block_cache: HashMap::new(),
            block_cache_bytes: 0,
        })
    }

    fn block(&mut self, block_index: usize) -> Option<Vec<u8>> {
        if let Some(block) = self.block_cache.get(&block_index) {
            return Some(block.clone());
        }
        let current = *self.index.get(block_index)?;
        let next = *self.index.get(block_index + 1)?;
        let plain = current & 0x8000_0000 != 0;
        let current = u64::from(current & 0x7fff_ffff);
        let next = u64::from(next & 0x7fff_ffff);
        let shifted_current = current.checked_shl(u32::from(self.align))?;
        let shifted_next = next.checked_shl(u32::from(self.align))?;

        // Standard CISO indexes are absolute file offsets shifted by `align`.
        // LaunchBox 13.27's recovered reader instead adds the end of the index
        // table and ignores `align`. Accept that historical relative shape as
        // a fallback without weakening bounds validation for either form.
        let bounds = [
            (shifted_current, shifted_next),
            (
                self.data_offset.checked_add(current)?,
                self.data_offset.checked_add(next)?,
            ),
        ]
        .into_iter()
        .find(|(start, end)| {
            *start >= self.data_offset
                && *end > *start
                && *start < self.file_len
                && *end <= self.file_len
        })?;
        let stored_len = bounds.1.checked_sub(bounds.0)?;
        let logical_start = block_index.checked_mul(self.block_size)?;
        let remaining = usize::try_from(self.len.saturating_sub(logical_start as u64)).ok()?;
        let output_len = self.block_size.min(remaining);
        if output_len == 0 {
            return None;
        }
        self.file.seek(SeekFrom::Start(bounds.0)).ok()?;
        let mut block = if plain {
            if stored_len < output_len as u64 {
                return None;
            }
            let mut block = vec![0_u8; output_len];
            self.file.read_exact(&mut block).ok()?;
            block
        } else {
            let compressed = self.file.by_ref().take(stored_len);
            let mut decoder = DeflateDecoder::new(compressed);
            let mut block = Vec::with_capacity(output_len);
            decoder
                .by_ref()
                .take(output_len as u64 + 1)
                .read_to_end(&mut block)
                .ok()?;
            if block.len() != output_len {
                return None;
            }
            block
        };
        block.truncate(output_len);
        if self.block_cache_bytes.saturating_add(block.len()) > CSO_CACHE_LIMIT {
            self.block_cache.clear();
            self.block_cache_bytes = 0;
        }
        self.block_cache_bytes += block.len();
        self.block_cache.insert(block_index, block.clone());
        Some(block)
    }
}

impl RomReader for CsoRomReader {
    fn len(&self) -> u64 {
        self.len
    }

    fn supports_fast_scan(&self) -> bool {
        false
    }

    fn read_at(&mut self, mut offset: u64, buffer: &mut [u8]) -> Option<usize> {
        if offset >= self.len {
            return Some(0);
        }
        let mut written = 0;
        while written < buffer.len() && offset < self.len {
            let block_index = usize::try_from(offset / self.block_size as u64).ok()?;
            let block_offset = usize::try_from(offset % self.block_size as u64).ok()?;
            let block = self.block(block_index)?;
            let source = block.get(block_offset..)?;
            let count = source.len().min(buffer.len() - written);
            if count == 0 {
                break;
            }
            buffer[written..written + count].copy_from_slice(&source[..count]);
            written += count;
            offset += count as u64;
        }
        Some(written)
    }
}

struct ChdRomReader {
    reader: ChdReader<BufReader<File>>,
    len: u64,
}

impl ChdRomReader {
    fn open(path: &Path) -> Option<Self> {
        validate_chd_v5_header(path)?;
        let file = File::open(path).ok()?;
        let chd = Chd::open(BufReader::new(file), None).ok()?;
        let len = chd.header().logical_bytes();
        if len == 0 || chd.header().hunk_size() > CHD_HUNK_LIMIT {
            return None;
        }
        Some(Self {
            reader: ChdReader::new(chd),
            len,
        })
    }
}

impl RomReader for ChdRomReader {
    fn len(&self) -> u64 {
        self.len
    }

    fn supports_fast_scan(&self) -> bool {
        false
    }

    fn read_at(&mut self, offset: u64, buffer: &mut [u8]) -> Option<usize> {
        if offset >= self.len {
            return Some(0);
        }
        self.reader.seek(SeekFrom::Start(offset)).ok()?;
        let count = usize::try_from((self.len - offset).min(buffer.len() as u64)).ok()?;
        self.reader.read_exact(&mut buffer[..count]).ok()?;
        Some(count)
    }
}

fn validate_chd_v5_header(path: &Path) -> Option<()> {
    let mut file = File::open(path).ok()?;
    let mut header = [0_u8; 64];
    file.read_exact(&mut header).ok()?;
    if &header[..8] != b"MComprHD"
        || u32::from_be_bytes(header[8..12].try_into().ok()?) != 124
        || u32::from_be_bytes(header[12..16].try_into().ok()?) != 5
    {
        return None;
    }
    let logical_bytes = u64::from_be_bytes(header[32..40].try_into().ok()?);
    let hunk_bytes = u32::from_be_bytes(header[56..60].try_into().ok()?);
    let unit_bytes = u32::from_be_bytes(header[60..64].try_into().ok()?);
    if logical_bytes == 0 || hunk_bytes == 0 || hunk_bytes > CHD_HUNK_LIMIT || unit_bytes == 0 {
        return None;
    }
    let hunk_count = logical_bytes
        .checked_add(u64::from(hunk_bytes) - 1)?
        .checked_div(u64::from(hunk_bytes))?;
    let maximum_map_bytes = hunk_count.checked_mul(12)?;
    (maximum_map_bytes <= CHD_MAP_LIMIT).then_some(())
}

/// Extracts the PS2 boot serial from a disc image using LaunchBox 13.27's
/// filesystem-first format and layout probes.
///
/// The reader is native and shell-free. It recognizes ISO, BIN/IMG/DUMP,
/// MDF/MDS, NRG, GZip, CSO, and self-contained CHD content. Known PS2 image
/// formats never accept a loose prefix match when their ISO9660 filesystem
/// cannot be validated.
pub fn extract_pcsx2_disc_serial(path: &Path) -> Option<String> {
    let metadata = fs::symlink_metadata(path).ok()?;
    if !metadata.file_type().is_file() {
        return None;
    }
    let path = fs::canonicalize(path).ok()?;
    let metadata = fs::metadata(&path).ok()?;
    let key = CacheKey {
        path: path.clone(),
        byte_len: metadata.len(),
        modified: metadata.modified().ok(),
    };
    let cache = SERIAL_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(cache) = cache.lock() {
        if let Some(serial) = cache.get(&key) {
            return serial.clone();
        }
    }
    let serial = extract_uncached(&path);
    if let Ok(mut cache) = cache.lock() {
        cache.insert(key, serial.clone());
    }
    serial
}

fn extract_uncached(path: &Path) -> Option<String> {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase);
    let mut reader = open_reader(path, extension.as_deref())?;
    let mut base_offsets = vec![0];
    base_offsets.extend(container_base_offsets(
        path,
        extension.as_deref(),
        reader.len(),
    ));
    base_offsets.sort_unstable();
    base_offsets.dedup();
    for base_offset in base_offsets {
        if let Some(serial) = extract_known_layouts(reader.as_mut(), base_offset) {
            return Some(serial);
        }
    }
    if reader.supports_fast_scan() {
        for layout in discover_layouts(reader.as_mut()) {
            if let Some(serial) = extract_layout(reader.as_mut(), layout) {
                return Some(serial);
            }
        }
    }
    if is_known_ps2_disc_format(extension.as_deref()) {
        return None;
    }
    let prefix_len = reader.len().min(FILE_PREFIX_LIMIT) as usize;
    let mut prefix = vec![0_u8; prefix_len];
    let read = reader.read_at(0, &mut prefix)?;
    prefix.truncate(read);
    parse_serial_from_bytes(&prefix)
}

fn open_reader(path: &Path, extension: Option<&str>) -> Option<Box<dyn RomReader>> {
    match extension {
        Some("gz") => {
            let file = File::open(path).ok()?;
            let mut decoder = GzDecoder::new(BufReader::new(file));
            let mut bytes = Vec::new();
            decoder
                .by_ref()
                .take(GZIP_DECOMPRESSED_LIMIT)
                .read_to_end(&mut bytes)
                .ok()?;
            (!bytes.is_empty()).then_some(Box::new(MemoryRomReader { bytes }) as Box<dyn RomReader>)
        }
        Some("cso") => {
            CsoRomReader::open(path).map(|reader| Box::new(reader) as Box<dyn RomReader>)
        }
        Some("chd") => {
            ChdRomReader::open(path).map(|reader| Box::new(reader) as Box<dyn RomReader>)
        }
        _ => FileRomReader::open(path).map(|reader| Box::new(reader) as Box<dyn RomReader>),
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct DiscLayout {
    base_offset: u64,
    sector_size: usize,
    data_offset: usize,
}

fn extract_known_layouts(reader: &mut dyn RomReader, base_offset: u64) -> Option<String> {
    for sector_size in SECTOR_SIZES {
        for data_offset in DATA_OFFSETS {
            if data_offset + 2048 <= sector_size {
                if let Some(serial) = extract_layout(
                    reader,
                    DiscLayout {
                        base_offset,
                        sector_size,
                        data_offset,
                    },
                ) {
                    return Some(serial);
                }
            }
        }
    }
    None
}

fn extract_layout(reader: &mut dyn RomReader, layout: DiscLayout) -> Option<String> {
    let primary = read_iso_payload(reader, layout, 16, 2048)?;
    if primary.get(..ISO_PRIMARY_VOLUME_DESCRIPTOR.len())
        != Some(ISO_PRIMARY_VOLUME_DESCRIPTOR.as_slice())
    {
        return None;
    }
    let root = parse_directory_record(&primary, 156)?;
    let directory_len = root.byte_len.min(DIRECTORY_LIMIT);
    let directory = read_iso_payload(reader, layout, root.extent, directory_len)?;
    let system_cnf = find_directory_file(&directory, b"SYSTEM.CNF")?;
    let system_cnf_len = system_cnf.byte_len.min(SYSTEM_CNF_LIMIT);
    let bytes = read_iso_payload(reader, layout, system_cnf.extent, system_cnf_len)?;
    parse_serial_from_bytes(&bytes)
}

fn read_iso_payload(
    reader: &mut dyn RomReader,
    layout: DiscLayout,
    start_lba: usize,
    byte_count: usize,
) -> Option<Vec<u8>> {
    if byte_count == 0 || layout.sector_size == 0 || layout.data_offset + 2048 > layout.sector_size
    {
        return None;
    }
    let sectors = byte_count.checked_add(2047)?.checked_div(2048)?;
    let capacity = sectors.checked_mul(2048)?;
    let mut payload = Vec::with_capacity(capacity);
    let mut sector = vec![0_u8; layout.sector_size];
    for relative in 0..sectors {
        let lba = start_lba.checked_add(relative)?;
        let offset = (lba as u64)
            .checked_mul(layout.sector_size as u64)?
            .checked_add(layout.base_offset)?;
        let end = offset.checked_add(layout.sector_size as u64)?;
        if end > reader.len() || reader.read_at(offset, &mut sector)? != sector.len() {
            return None;
        }
        payload.extend_from_slice(&sector[layout.data_offset..layout.data_offset + 2048]);
    }
    payload.truncate(byte_count);
    Some(payload)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DirectoryRecord {
    extent: usize,
    byte_len: usize,
    name: Vec<u8>,
}

fn parse_directory_record(bytes: &[u8], offset: usize) -> Option<DirectoryRecord> {
    let record_len = *bytes.get(offset)? as usize;
    if record_len < 34 || offset.checked_add(record_len)? > bytes.len() {
        return None;
    }
    let extent = u32::from_le_bytes(bytes.get(offset + 2..offset + 6)?.try_into().ok()?);
    let byte_len = u32::from_le_bytes(bytes.get(offset + 10..offset + 14)?.try_into().ok()?);
    let name_len = *bytes.get(offset + 32)? as usize;
    let name_start = offset.checked_add(33)?;
    let name_end = name_start.checked_add(name_len)?;
    if name_end > offset + record_len {
        return None;
    }
    let mut name = bytes[name_start..name_end].to_vec();
    if let Some(version) = name.iter().position(|byte| *byte == b';') {
        name.truncate(version);
    }
    if name.last() == Some(&b'.') {
        name.pop();
    }
    Some(DirectoryRecord {
        extent: extent as usize,
        byte_len: byte_len as usize,
        name,
    })
}

fn find_directory_file(bytes: &[u8], target: &[u8]) -> Option<DirectoryRecord> {
    let mut offset = 0;
    while offset < bytes.len() {
        let record_len = bytes[offset] as usize;
        if record_len == 0 {
            offset = offset
                .checked_div(2048)?
                .checked_add(1)?
                .checked_mul(2048)?;
            continue;
        }
        let record = parse_directory_record(bytes, offset)?;
        if record.name.eq_ignore_ascii_case(target) {
            return Some(record);
        }
        offset = offset.checked_add(record_len)?;
    }
    None
}

fn parse_serial_from_bytes(bytes: &[u8]) -> Option<String> {
    for start in 0..bytes.len() {
        let candidate = bytes.get(start..start + 11)?;
        if candidate[..4].iter().all(u8::is_ascii_alphabetic)
            && candidate[4] == b'_'
            && candidate[5..8].iter().all(u8::is_ascii_digit)
            && candidate[8] == b'.'
            && candidate[9..11].iter().all(u8::is_ascii_digit)
        {
            let mut serial = String::with_capacity(10);
            serial.extend(
                candidate[..4]
                    .iter()
                    .map(|byte| char::from(*byte).to_ascii_uppercase()),
            );
            serial.push('-');
            serial.extend(candidate[5..8].iter().map(|byte| char::from(*byte)));
            serial.extend(candidate[9..11].iter().map(|byte| char::from(*byte)));
            return Some(serial);
        }
    }
    None
}

fn discover_layouts(reader: &mut dyn RomReader) -> Vec<DiscLayout> {
    let scan_limit = reader.len().min(LAYOUT_SCAN_LIMIT);
    let mut layouts = Vec::new();
    let mut seen = HashSet::new();
    let mut position = 0_u64;
    let mut chunk = vec![0_u8; FILE_PREFIX_LIMIT as usize];
    while position < scan_limit {
        let requested =
            usize::try_from((scan_limit - position).min(FILE_PREFIX_LIMIT)).unwrap_or(chunk.len());
        let Some(read) = reader.read_at(position, &mut chunk[..requested]) else {
            break;
        };
        if read == 0 {
            break;
        }
        for index in 0..=read.saturating_sub(ISO_PRIMARY_VOLUME_DESCRIPTOR.len()) {
            if chunk[index..index + ISO_PRIMARY_VOLUME_DESCRIPTOR.len()]
                != ISO_PRIMARY_VOLUME_DESCRIPTOR
            {
                continue;
            }
            let absolute = position + index as u64;
            for sector_size in SECTOR_SIZES {
                for data_offset in DATA_OFFSETS {
                    let Some(pvd_offset) = (16_u64)
                        .checked_mul(sector_size as u64)
                        .and_then(|offset| offset.checked_add(data_offset as u64))
                    else {
                        continue;
                    };
                    let Some(base_offset) = absolute.checked_sub(pvd_offset) else {
                        continue;
                    };
                    let layout = DiscLayout {
                        base_offset,
                        sector_size,
                        data_offset,
                    };
                    if seen.insert(layout) {
                        layouts.push(layout);
                    }
                }
            }
        }
        if read < requested {
            break;
        }
        let step = read.saturating_sub(ISO_PRIMARY_VOLUME_DESCRIPTOR.len() - 1);
        if step == 0 {
            break;
        }
        position += step as u64;
    }
    layouts
}

fn container_base_offsets(path: &Path, extension: Option<&str>, content_len: u64) -> Vec<u64> {
    match extension {
        Some("mdf") => read_mds_base_offsets(path, content_len),
        Some("nrg") => read_nrg_base_offsets(path, content_len),
        _ => Vec::new(),
    }
}

fn read_mds_base_offsets(image_path: &Path, image_len: u64) -> Vec<u64> {
    let Some(sidecar) = case_insensitive_sidecar(image_path, "mds") else {
        return Vec::new();
    };
    match fs::symlink_metadata(&sidecar) {
        Ok(metadata) if metadata.file_type().is_file() && metadata.len() <= SIDECAR_LIMIT => {}
        _ => return Vec::new(),
    }
    let bytes = match read_bounded_file(&sidecar, SIDECAR_LIMIT) {
        Some(bytes) => bytes,
        None => return Vec::new(),
    };
    let mut offsets = HashSet::new();
    for chunk in bytes.chunks_exact(4) {
        let value = u64::from(u32::from_le_bytes(
            chunk.try_into().expect("four-byte MDS field"),
        ));
        if value != 0 && value < image_len && likely_sector_aligned(value) {
            offsets.insert(value);
        }
        if let Some(bytes) = value.checked_mul(2048) {
            if bytes != 0 && bytes < image_len {
                offsets.insert(bytes);
            }
        }
        if offsets.len() >= 48 {
            break;
        }
    }
    let mut offsets = offsets.into_iter().collect::<Vec<_>>();
    offsets.sort_unstable();
    offsets
}

fn read_bounded_file(path: &Path, limit: u64) -> Option<Vec<u8>> {
    let file = File::open(path).ok()?;
    let mut bytes = Vec::new();
    file.take(limit.checked_add(1)?)
        .read_to_end(&mut bytes)
        .ok()?;
    (bytes.len() as u64 <= limit).then_some(bytes)
}

fn case_insensitive_sidecar(image_path: &Path, extension: &str) -> Option<PathBuf> {
    let parent = image_path.parent()?;
    let stem = image_path.file_stem()?.to_str()?;
    let mut matches = fs::read_dir(parent)
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_type()
                .ok()
                .is_some_and(|file_type| file_type.is_file() && !file_type.is_symlink())
        })
        .filter_map(|entry| {
            let path = entry.path();
            let matching = path
                .file_stem()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case(stem))
                && path
                    .extension()
                    .and_then(|value| value.to_str())
                    .is_some_and(|value| value.eq_ignore_ascii_case(extension));
            matching.then_some(path)
        })
        .collect::<Vec<_>>();
    matches.sort_by_key(|path| path.to_string_lossy().to_ascii_lowercase());
    matches.into_iter().next()
}

fn read_nrg_base_offsets(path: &Path, image_len: u64) -> Vec<u64> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return Vec::new(),
    };
    let file_len = match file.metadata() {
        Ok(metadata) => metadata.len(),
        Err(_) => return Vec::new(),
    };
    if file_len < 12 || file.seek(SeekFrom::End(-12)).is_err() {
        return Vec::new();
    }
    let mut trailer = [0_u8; 12];
    if file.read_exact(&mut trailer).is_err() {
        return Vec::new();
    }
    let chunk_offset = match &trailer[..4] {
        b"NER5" => u64::from_be_bytes(trailer[4..12].try_into().expect("eight-byte NRG offset")),
        b"NERO" => u64::from(u32::from_be_bytes(
            trailer[8..12].try_into().expect("four-byte NRG offset"),
        )),
        _ => return Vec::new(),
    };
    if chunk_offset == 0 || chunk_offset >= file_len {
        return Vec::new();
    }
    let mut offsets = HashSet::new();
    add_container_offset(&mut offsets, chunk_offset, image_len);
    let chunk_len = (file_len - chunk_offset).min(NRG_CHUNK_LIMIT);
    if chunk_len <= 16 || file.seek(SeekFrom::Start(chunk_offset)).is_err() {
        return sorted_limited_offsets(offsets, 64);
    }
    let mut bytes = vec![0_u8; chunk_len as usize];
    if file.read_exact(&mut bytes).is_err() {
        return sorted_limited_offsets(offsets, 64);
    }
    for index in (0..=bytes.len().saturating_sub(8)).step_by(4) {
        let value = u64::from(u32::from_be_bytes(
            bytes[index..index + 4]
                .try_into()
                .expect("four-byte NRG candidate"),
        ));
        add_container_offset(&mut offsets, value, image_len);
        if index + 8 <= bytes.len() {
            let value = u64::from_be_bytes(
                bytes[index..index + 8]
                    .try_into()
                    .expect("eight-byte NRG candidate"),
            );
            add_container_offset(&mut offsets, value, image_len);
        }
        if offsets.len() >= 96 {
            break;
        }
    }
    sorted_limited_offsets(offsets, 64)
}

fn add_container_offset(offsets: &mut HashSet<u64>, value: u64, image_len: u64) {
    if value > 0 && value < image_len && likely_sector_aligned(value) {
        offsets.insert(value);
    }
}

fn sorted_limited_offsets(offsets: HashSet<u64>, limit: usize) -> Vec<u64> {
    let mut offsets = offsets.into_iter().collect::<Vec<_>>();
    offsets.sort_unstable();
    offsets.truncate(limit);
    offsets
}

fn likely_sector_aligned(value: u64) -> bool {
    [2048_u64, 2352, 2336, 2448]
        .into_iter()
        .any(|sector_size| value.is_multiple_of(sector_size))
}

fn is_known_ps2_disc_format(extension: Option<&str>) -> bool {
    extension.is_some_and(|extension| {
        [
            "iso", "mdf", "nrg", "bin", "img", "dump", "gz", "cso", "chd",
        ]
        .contains(&extension)
    })
}

#[cfg(any(test, feature = "test-fixtures"))]
pub(crate) mod fixture {
    use flate2::{write::DeflateEncoder, write::GzEncoder, Compression};
    use std::io::Write;

    const SECTOR_SIZE: usize = 2048;
    const SECTORS: usize = 24;
    const ROOT_EXTENT: usize = 20;
    const SYSTEM_CNF_EXTENT: usize = 21;

    pub(crate) fn iso(serial: &str) -> Vec<u8> {
        let mut image = vec![0_u8; SECTOR_SIZE * SECTORS];
        let pvd = &mut image[16 * SECTOR_SIZE..17 * SECTOR_SIZE];
        pvd[..7].copy_from_slice(&super::ISO_PRIMARY_VOLUME_DESCRIPTOR);
        write_directory_record(&mut pvd[156..], ROOT_EXTENT, SECTOR_SIZE, &[0]);

        let root = &mut image[ROOT_EXTENT * SECTOR_SIZE..(ROOT_EXTENT + 1) * SECTOR_SIZE];
        write_directory_record(root, SYSTEM_CNF_EXTENT, 128, b"SYSTEM.CNF;1");
        let system_cnf =
            &mut image[SYSTEM_CNF_EXTENT * SECTOR_SIZE..(SYSTEM_CNF_EXTENT + 1) * SECTOR_SIZE];
        let text = format!("BOOT2 = cdrom0:\\\\{};1\r\nVER = 1.00\r\n", serial);
        system_cnf[..text.len()].copy_from_slice(text.as_bytes());
        image
    }

    #[cfg(test)]
    pub(crate) fn raw_sector_image(
        iso: &[u8],
        base_offset: usize,
        sector_size: usize,
        data_offset: usize,
    ) -> Vec<u8> {
        assert!(data_offset + SECTOR_SIZE <= sector_size);
        let sectors = iso.len() / SECTOR_SIZE;
        let mut image = vec![0xa5_u8; base_offset + sectors * sector_size];
        for sector in 0..sectors {
            let source = &iso[sector * SECTOR_SIZE..(sector + 1) * SECTOR_SIZE];
            let start = base_offset + sector * sector_size + data_offset;
            image[start..start + SECTOR_SIZE].copy_from_slice(source);
        }
        image
    }

    pub(crate) fn gzip(iso: &[u8]) -> Vec<u8> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(iso).unwrap();
        encoder.finish().unwrap()
    }

    pub(crate) fn cso(iso: &[u8]) -> Vec<u8> {
        let blocks = iso.len().div_ceil(SECTOR_SIZE);
        let header_size = 24_usize;
        let index_size = (blocks + 1) * 4;
        let data_offset = header_size + index_size;
        let mut bytes = vec![0_u8; data_offset];
        bytes[..4].copy_from_slice(b"CISO");
        bytes[4..8].copy_from_slice(&(header_size as u32).to_le_bytes());
        bytes[8..16].copy_from_slice(&(iso.len() as u64).to_le_bytes());
        bytes[16..20].copy_from_slice(&(SECTOR_SIZE as u32).to_le_bytes());
        bytes[20] = 1;
        bytes[21] = 0;
        let mut payload = Vec::new();
        for (index, block) in iso.chunks(SECTOR_SIZE).enumerate() {
            let offset = data_offset + payload.len();
            let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(block).unwrap();
            let compressed = encoder.finish().unwrap();
            let (value, stored) = if compressed.len() < block.len() {
                (offset as u32, compressed.as_slice())
            } else {
                ((offset as u32) | 0x8000_0000, block)
            };
            let start = header_size + index * 4;
            bytes[start..start + 4].copy_from_slice(&value.to_le_bytes());
            payload.extend_from_slice(stored);
        }
        let final_offset = (data_offset + payload.len()) as u32;
        let final_index = header_size + blocks * 4;
        bytes[final_index..final_index + 4].copy_from_slice(&final_offset.to_le_bytes());
        bytes.extend_from_slice(&payload);
        bytes
    }

    fn write_directory_record(destination: &mut [u8], extent: usize, byte_len: usize, name: &[u8]) {
        let record_len = 33 + name.len() + usize::from(name.len().is_multiple_of(2));
        destination[0] = record_len as u8;
        destination[2..6].copy_from_slice(&(extent as u32).to_le_bytes());
        destination[6..10].copy_from_slice(&(extent as u32).to_be_bytes());
        destination[10..14].copy_from_slice(&(byte_len as u32).to_le_bytes());
        destination[14..18].copy_from_slice(&(byte_len as u32).to_be_bytes());
        destination[25] = 0;
        destination[28..30].copy_from_slice(&1_u16.to_le_bytes());
        destination[30..32].copy_from_slice(&1_u16.to_be_bytes());
        destination[32] = name.len() as u8;
        destination[33..33 + name.len()].copy_from_slice(name);
    }
}

#[cfg(feature = "test-fixtures")]
#[doc(hidden)]
pub mod test_fixtures {
    pub fn iso(serial: &str) -> Vec<u8> {
        super::fixture::iso(serial)
    }

    pub fn gzip(serial: &str) -> Vec<u8> {
        super::fixture::gzip(&super::fixture::iso(serial))
    }

    pub fn cso(serial: &str) -> Vec<u8> {
        super::fixture::cso(&super::fixture::iso(serial))
    }

    pub fn chd_cd() -> &'static [u8] {
        include_bytes!("../../../fixtures/pcsx2/synthetic-cd.chd")
    }

    pub fn chd_dvd() -> &'static [u8] {
        include_bytes!("../../../fixtures/pcsx2/synthetic-lzma.chd")
    }
}

#[cfg(test)]
mod tests {
    use super::fixture::*;
    use super::*;

    #[test]
    fn extracts_serial_from_iso9660_instead_of_loose_prefix_bytes() {
        let directory = tempfile::tempdir().unwrap();
        let iso_path = directory.path().join("generic.iso");
        fs::write(&iso_path, iso("SLUS_203.12")).unwrap();
        assert_eq!(
            extract_pcsx2_disc_serial(&iso_path),
            Some("SLUS-20312".to_string())
        );

        let false_positive = directory.path().join("false-positive.iso");
        fs::write(
            &false_positive,
            b"BOOT2 = cdrom0:\\\\SLES_999.99;1 but no ISO filesystem",
        )
        .unwrap();
        assert_eq!(extract_pcsx2_disc_serial(&false_positive), None);
    }

    #[test]
    fn extracts_serial_from_scanned_raw_sector_layout() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("generic.bin");
        fs::write(&path, raw_sector_image(&iso("SCES_512.34"), 4096, 2352, 16)).unwrap();
        assert_eq!(
            extract_pcsx2_disc_serial(&path),
            Some("SCES-51234".to_string())
        );

        let boundary_path = directory.path().join("chunk-boundary.bin");
        let signature_offset = FILE_PREFIX_LIMIT as usize - 3;
        let base_offset = signature_offset - 16 * 2352 - 16;
        fs::write(
            &boundary_path,
            raw_sector_image(&iso("SCES_512.35"), base_offset, 2352, 16),
        )
        .unwrap();
        assert_eq!(
            extract_pcsx2_disc_serial(&boundary_path),
            Some("SCES-51235".to_string())
        );
    }

    #[test]
    fn extracts_serial_from_mdf_offset_named_by_case_insensitive_mds_sidecar() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("generic.mdf");
        let base_offset = 4096_usize;
        let mut image = vec![0x5a; base_offset];
        image.extend_from_slice(&iso("SLES_543.21"));
        fs::write(&path, image).unwrap();
        fs::write(
            directory.path().join("GENERIC.MDS"),
            (base_offset as u32).to_le_bytes(),
        )
        .unwrap();
        assert_eq!(
            extract_pcsx2_disc_serial(&path),
            Some("SLES-54321".to_string())
        );
    }

    #[test]
    fn extracts_serial_from_nrg_chunk_offset_beyond_the_fast_scan_limit() {
        use std::io::Write;

        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("generic.nrg");
        let base_offset = 68_u64 * 1024 * 1024;
        let image = iso("SCUS_777.01");
        let mut file = File::create(&path).unwrap();
        file.seek(SeekFrom::Start(base_offset)).unwrap();
        file.write_all(&image).unwrap();
        let chunk_offset = base_offset + image.len() as u64;
        file.write_all(&base_offset.to_be_bytes()).unwrap();
        file.write_all(b"NER5").unwrap();
        file.write_all(&chunk_offset.to_be_bytes()).unwrap();
        file.sync_all().unwrap();

        assert_eq!(
            extract_pcsx2_disc_serial(&path),
            Some("SCUS-77701".to_string())
        );
    }

    #[test]
    fn extracts_serial_from_bounded_gzip_and_standard_cso() {
        let directory = tempfile::tempdir().unwrap();
        let image = iso("SLPM_654.32");
        let gzip_path = directory.path().join("generic.gz");
        let cso_path = directory.path().join("generic.cso");
        fs::write(&gzip_path, gzip(&image)).unwrap();
        fs::write(&cso_path, cso(&image)).unwrap();
        assert_eq!(
            extract_pcsx2_disc_serial(&gzip_path),
            Some("SLPM-65432".to_string())
        );
        assert_eq!(
            extract_pcsx2_disc_serial(&cso_path),
            Some("SLPM-65432".to_string())
        );
    }

    #[test]
    fn extracts_serial_from_real_lzma_dvd_and_cd_chd_v5_fixtures() {
        let directory = tempfile::tempdir().unwrap();
        for (name, bytes) in [
            (
                "dvd.chd",
                include_bytes!("../../../fixtures/pcsx2/synthetic-lzma.chd").as_slice(),
            ),
            (
                "cd.chd",
                include_bytes!("../../../fixtures/pcsx2/synthetic-cd.chd").as_slice(),
            ),
        ] {
            let path = directory.path().join(name);
            fs::write(&path, bytes).unwrap();
            assert_eq!(
                extract_pcsx2_disc_serial(&path),
                Some("SLUS-20312".to_string())
            );
        }
    }

    #[test]
    fn unknown_extensions_keep_the_recovered_prefix_fallback() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("header.rom");
        fs::write(&path, b"metadata SCPS_123.45 trailing").unwrap();
        assert_eq!(
            extract_pcsx2_disc_serial(&path),
            Some("SCPS-12345".to_string())
        );
    }

    #[test]
    fn rejects_symlinks_and_invalid_compressed_images() {
        let directory = tempfile::tempdir().unwrap();
        let target = directory.path().join("target.iso");
        fs::write(&target, iso("SLUS_111.11")).unwrap();
        #[cfg(unix)]
        {
            let link = directory.path().join("link.iso");
            std::os::unix::fs::symlink(&target, &link).unwrap();
            assert_eq!(extract_pcsx2_disc_serial(&link), None);
        }
        let invalid = directory.path().join("invalid.cso");
        fs::write(&invalid, b"CISO").unwrap();
        assert_eq!(extract_pcsx2_disc_serial(&invalid), None);

        let oversized = directory.path().join("oversized-block.cso");
        let mut header = [0_u8; 24];
        header[..4].copy_from_slice(b"CISO");
        header[4..8].copy_from_slice(&24_u32.to_le_bytes());
        header[8..16].copy_from_slice(&2048_u64.to_le_bytes());
        header[16..20].copy_from_slice(&((CSO_BLOCK_LIMIT + 1) as u32).to_le_bytes());
        header[20] = 1;
        fs::write(&oversized, header).unwrap();
        assert_eq!(extract_pcsx2_disc_serial(&oversized), None);

        let oversized_chd = directory.path().join("oversized-hunk.chd");
        let mut chd = include_bytes!("../../../fixtures/pcsx2/synthetic-lzma.chd").to_vec();
        chd[56..60].copy_from_slice(&(CHD_HUNK_LIMIT + 1).to_be_bytes());
        fs::write(&oversized_chd, chd).unwrap();
        assert_eq!(extract_pcsx2_disc_serial(&oversized_chd), None);
    }
}
