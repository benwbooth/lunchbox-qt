use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::emulator_lifecycle::DownloadReceipt;

pub const BIGPEMU_DOWNLOAD_PAGE: &str =
    "https://www.richwhitehouse.com/jaguar/index.php?content=download";
pub const BIGPEMU_PROVIDER: &str = "richwhitehouse:bigpemu";
const BIGPEMU_ASSET_PREFIX: &str = "https://www.richwhitehouse.com/jaguar/builds/";
const MAX_RELEASE_PAGE_BYTES: u64 = 1024 * 1024;
const MAX_ARTIFACT_BYTES: u64 = 256 * 1024 * 1024;
const DOWNLOAD_BUFFER_BYTES: usize = 128 * 1024;
const MAX_README_BYTES: u64 = 1024 * 1024;
const FNV1A64_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV1A64_PRIME: u64 = 0x0000_0100_0000_01b3;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BigPEmuArtifactKind {
    WindowsZipX64,
    WindowsZipArm64,
    LinuxTarGzX64,
    LinuxTarGzArm64,
}

impl BigPEmuArtifactKind {
    pub const ALL: [Self; 4] = [
        Self::WindowsZipX64,
        Self::WindowsZipArm64,
        Self::LinuxTarGzX64,
        Self::LinuxTarGzArm64,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::WindowsZipX64 => "windows_zip_x64",
            Self::WindowsZipArm64 => "windows_zip_arm64",
            Self::LinuxTarGzX64 => "linux_tar_gz_x64",
            Self::LinuxTarGzArm64 => "linux_tar_gz_arm64",
        }
    }

    pub const fn platform_label(self) -> &'static str {
        match self {
            Self::WindowsZipX64 => "Windows (x64)",
            Self::WindowsZipArm64 => "Windows (ARM64)",
            Self::LinuxTarGzX64 => "Linux (x64)",
            Self::LinuxTarGzArm64 => "Linux (ARM64)",
        }
    }

    pub const fn executable_name(self) -> &'static str {
        match self {
            Self::WindowsZipX64 | Self::WindowsZipArm64 => "BigPEmu.exe",
            Self::LinuxTarGzX64 | Self::LinuxTarGzArm64 => "bigpemu",
        }
    }

    pub fn asset_name(self, version: &str) -> Result<String, BigPEmuLifecycleError> {
        validate_version(version)?;
        let compact = version.replace('.', "");
        Ok(match self {
            Self::WindowsZipX64 => format!("BigPEmu_v{compact}.zip"),
            Self::WindowsZipArm64 => format!("BigPEmu_WinARM64_v{compact}.zip"),
            Self::LinuxTarGzX64 => format!("BigPEmu_Linux64_v{compact}.tar.gz"),
            Self::LinuxTarGzArm64 => format!("BigPEmu_LinuxARM64_v{compact}.tar.gz"),
        })
    }

    pub fn current_host() -> Result<Self, BigPEmuLifecycleError> {
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        {
            return Ok(Self::WindowsZipX64);
        }
        #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
        {
            return Ok(Self::WindowsZipArm64);
        }
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            return Ok(Self::LinuxTarGzX64);
        }
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        {
            return Ok(Self::LinuxTarGzArm64);
        }
        #[allow(unreachable_code)]
        Err(BigPEmuLifecycleError::UnsupportedHost {
            os: std::env::consts::OS,
            architecture: std::env::consts::ARCH,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BigPEmuReleaseOffer {
    pub version: String,
    pub release_name: String,
    pub release_url: String,
    pub artifact_kind: BigPEmuArtifactKind,
    pub asset_name: String,
    pub asset_url: String,
    pub asset_byte_len: u64,
    pub asset_fnv1a64: String,
}

pub trait BigPEmuReleaseTransport: Send + Sync {
    fn fetch_release_page(
        &self,
        url: &str,
        max_bytes: u64,
    ) -> Result<Vec<u8>, BigPEmuLifecycleError>;

    fn download(
        &self,
        url: &str,
        destination: &Path,
        expected_byte_len: u64,
        progress: &mut dyn FnMut(u64, u64),
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<DownloadReceipt, BigPEmuLifecycleError>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RichWhitehouseReleaseTransport;

impl BigPEmuReleaseTransport for RichWhitehouseReleaseTransport {
    fn fetch_release_page(
        &self,
        url: &str,
        max_bytes: u64,
    ) -> Result<Vec<u8>, BigPEmuLifecycleError> {
        if url != BIGPEMU_DOWNLOAD_PAGE {
            return Err(BigPEmuLifecycleError::UntrustedUrl {
                url: url.to_string(),
            });
        }
        let mut response = ureq::get(url)
            .header("Accept", "text/html")
            .header("User-Agent", "lunchbox-qt")
            .call()
            .map_err(|error| BigPEmuLifecycleError::Transport {
                message: error.to_string(),
            })?;
        read_limited(
            response.body_mut().as_reader(),
            max_bytes,
            "BigPEmu release page",
        )
    }

    fn download(
        &self,
        url: &str,
        destination: &Path,
        expected_byte_len: u64,
        progress: &mut dyn FnMut(u64, u64),
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<DownloadReceipt, BigPEmuLifecycleError> {
        validate_asset_url(url, None)?;
        if should_cancel() {
            return Err(BigPEmuLifecycleError::Cancelled);
        }
        let mut response = ureq::get(url)
            .header("Accept", "application/octet-stream")
            .header("User-Agent", "lunchbox-qt")
            .call()
            .map_err(|error| BigPEmuLifecycleError::Transport {
                message: error.to_string(),
            })?;
        stream_download(
            response.body_mut().as_reader(),
            destination,
            expected_byte_len,
            progress,
            should_cancel,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileBigPEmuReleaseTransport {
    root: PathBuf,
}

impl FileBigPEmuReleaseTransport {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl BigPEmuReleaseTransport for FileBigPEmuReleaseTransport {
    fn fetch_release_page(
        &self,
        _url: &str,
        max_bytes: u64,
    ) -> Result<Vec<u8>, BigPEmuLifecycleError> {
        let path = self.root.join("download.html");
        let source = open_safe_fixture(&path)?;
        read_limited(source, max_bytes, "BigPEmu fixture release page")
    }

    fn download(
        &self,
        url: &str,
        destination: &Path,
        expected_byte_len: u64,
        progress: &mut dyn FnMut(u64, u64),
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<DownloadReceipt, BigPEmuLifecycleError> {
        let asset_name = url.rsplit('/').next().unwrap_or_default();
        validate_asset_name(asset_name)?;
        let source = open_safe_fixture(&self.root.join(asset_name))?;
        stream_download(
            source,
            destination,
            expected_byte_len,
            progress,
            should_cancel,
        )
    }
}

pub fn fetch_latest_bigpemu_release(
    transport: &dyn BigPEmuReleaseTransport,
    artifact_kind: BigPEmuArtifactKind,
) -> Result<BigPEmuReleaseOffer, BigPEmuLifecycleError> {
    let catalog = transport.fetch_release_page(BIGPEMU_DOWNLOAD_PAGE, MAX_RELEASE_PAGE_BYTES)?;
    select_bigpemu_release(&catalog, artifact_kind)
}

pub fn select_bigpemu_release(
    page: &[u8],
    artifact_kind: BigPEmuArtifactKind,
) -> Result<BigPEmuReleaseOffer, BigPEmuLifecycleError> {
    let page =
        std::str::from_utf8(page).map_err(|error| BigPEmuLifecycleError::InvalidCatalog {
            message: format!("release page is not valid UTF-8: {error}"),
        })?;
    let version = extract_exactly_one_between(
        page,
        "Current Version: <strong>",
        "</strong>",
        "current version",
    )?;
    validate_version(version)?;
    let label = format!("<strong>{}: </strong>", artifact_kind.platform_label());
    let section_start = exactly_one_index(page, &label, artifact_kind.platform_label())?
        .saturating_add(label.len());
    let section = &page[section_start..];
    let section_end =
        section
            .find("<br /><br />")
            .ok_or_else(|| BigPEmuLifecycleError::InvalidCatalog {
                message: format!(
                    "{} release section has no terminator",
                    artifact_kind.platform_label()
                ),
            })?;
    let section = &section[..section_end];
    let asset_url = extract_exactly_one_between(section, "<a href=\"", "\">", "asset URL")?;
    let expected_asset_name = artifact_kind.asset_name(version)?;
    validate_asset_url(asset_url, Some(&expected_asset_name))?;
    let size = extract_exactly_one_between(section, "Size: <i>", " bytes</i>", "asset size")?;
    let asset_byte_len = size.replace(',', "").parse::<u64>().map_err(|error| {
        BigPEmuLifecycleError::InvalidCatalog {
            message: format!("asset size is invalid: {error}"),
        }
    })?;
    if asset_byte_len == 0 || asset_byte_len > MAX_ARTIFACT_BYTES {
        return Err(BigPEmuLifecycleError::InvalidAssetSize {
            asset: expected_asset_name,
            byte_len: asset_byte_len,
        });
    }
    let asset_fnv1a64 =
        extract_exactly_one_between(section, "Hash: <i>", " (64-bit FNV-1a)</i>", "FNV hash")?
            .to_ascii_uppercase();
    validate_fnv1a64(&asset_fnv1a64)?;

    Ok(BigPEmuReleaseOffer {
        version: version.to_string(),
        release_name: format!("BigPEmu {version}"),
        release_url: BIGPEMU_DOWNLOAD_PAGE.to_string(),
        artifact_kind,
        asset_name: artifact_kind.asset_name(version)?,
        asset_url: asset_url.to_string(),
        asset_byte_len,
        asset_fnv1a64,
    })
}

pub fn download_bigpemu_release(
    transport: &dyn BigPEmuReleaseTransport,
    offer: &BigPEmuReleaseOffer,
    destination: &Path,
    progress: &mut dyn FnMut(u64, u64),
    should_cancel: &dyn Fn() -> bool,
) -> Result<DownloadReceipt, BigPEmuLifecycleError> {
    validate_version(&offer.version)?;
    let expected_name = offer.artifact_kind.asset_name(&offer.version)?;
    if offer.asset_name != expected_name {
        return Err(BigPEmuLifecycleError::InvalidCatalog {
            message: "asset name does not match the version and host kind".into(),
        });
    }
    validate_asset_url(&offer.asset_url, Some(&offer.asset_name))?;
    validate_fnv1a64(&offer.asset_fnv1a64)?;
    if offer.asset_byte_len == 0 || offer.asset_byte_len > MAX_ARTIFACT_BYTES {
        return Err(BigPEmuLifecycleError::InvalidAssetSize {
            asset: offer.asset_name.clone(),
            byte_len: offer.asset_byte_len,
        });
    }
    let receipt = transport.download(
        &offer.asset_url,
        destination,
        offer.asset_byte_len,
        progress,
        should_cancel,
    )?;
    if receipt.byte_len != offer.asset_byte_len {
        let _ = fs::remove_file(destination);
        return Err(BigPEmuLifecycleError::SizeMismatch {
            expected: offer.asset_byte_len,
            actual: receipt.byte_len,
        });
    }
    let actual_fnv = fnv1a64_file(destination)?;
    if !actual_fnv.eq_ignore_ascii_case(&offer.asset_fnv1a64) {
        let _ = fs::remove_file(destination);
        return Err(BigPEmuLifecycleError::FnvMismatch {
            expected: offer.asset_fnv1a64.clone(),
            actual: actual_fnv,
        });
    }
    Ok(receipt)
}

fn exactly_one_index(
    haystack: &str,
    needle: &str,
    label: &str,
) -> Result<usize, BigPEmuLifecycleError> {
    let mut matches = haystack.match_indices(needle);
    let first = matches.next().map(|(index, _)| index);
    if first.is_none() || matches.next().is_some() {
        return Err(BigPEmuLifecycleError::InvalidCatalog {
            message: format!("{label} does not occur exactly once"),
        });
    }
    Ok(first.expect("checked above"))
}

fn extract_exactly_one_between<'a>(
    text: &'a str,
    prefix: &str,
    suffix: &str,
    label: &str,
) -> Result<&'a str, BigPEmuLifecycleError> {
    let start = exactly_one_index(text, prefix, label)?.saturating_add(prefix.len());
    let remainder = &text[start..];
    let end = remainder
        .find(suffix)
        .ok_or_else(|| BigPEmuLifecycleError::InvalidCatalog {
            message: format!("{label} has no closing delimiter"),
        })?;
    let value = &remainder[..end];
    if value.is_empty() {
        return Err(BigPEmuLifecycleError::InvalidCatalog {
            message: format!("{label} is empty"),
        });
    }
    Ok(value)
}

fn validate_version(version: &str) -> Result<(), BigPEmuLifecycleError> {
    if version.is_empty()
        || version.len() > 32
        || version.starts_with('.')
        || version.ends_with('.')
        || version.split('.').any(|component| {
            component.is_empty() || !component.bytes().all(|byte| byte.is_ascii_digit())
        })
    {
        return Err(BigPEmuLifecycleError::InvalidCatalog {
            message: format!("BigPEmu version is unsafe: {version:?}"),
        });
    }
    Ok(())
}

fn validate_asset_url(url: &str, expected_name: Option<&str>) -> Result<(), BigPEmuLifecycleError> {
    if !url.starts_with(BIGPEMU_ASSET_PREFIX)
        || url.contains(['\r', '\n', '?', '#'])
        || url.rsplit('/').next().is_none_or(str::is_empty)
    {
        return Err(BigPEmuLifecycleError::UntrustedUrl {
            url: url.to_string(),
        });
    }
    let actual_name = url.rsplit('/').next().unwrap_or_default();
    validate_asset_name(actual_name)?;
    if expected_name.is_some_and(|expected| actual_name != expected) {
        return Err(BigPEmuLifecycleError::UntrustedUrl {
            url: url.to_string(),
        });
    }
    Ok(())
}

fn validate_asset_name(name: &str) -> Result<(), BigPEmuLifecycleError> {
    if name.is_empty()
        || name
            != Path::new(name)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("")
        || name.contains(['/', '\\'])
        || name == "."
        || name == ".."
    {
        return Err(BigPEmuLifecycleError::UnsafeAssetName {
            name: name.to_string(),
        });
    }
    Ok(())
}

fn validate_fnv1a64(hash: &str) -> Result<(), BigPEmuLifecycleError> {
    if hash.len() != 16
        || !hash
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_lowercase())
    {
        return Err(BigPEmuLifecycleError::InvalidFnv {
            hash: hash.to_string(),
        });
    }
    Ok(())
}

fn open_safe_fixture(path: &Path) -> Result<File, BigPEmuLifecycleError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| BigPEmuLifecycleError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(BigPEmuLifecycleError::UnsafeFixture {
            path: path.to_path_buf(),
        });
    }
    File::open(path).map_err(|source| BigPEmuLifecycleError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn stream_download(
    mut source: impl Read,
    destination: &Path,
    expected_byte_len: u64,
    progress: &mut dyn FnMut(u64, u64),
    should_cancel: &dyn Fn() -> bool,
) -> Result<DownloadReceipt, BigPEmuLifecycleError> {
    match fs::symlink_metadata(destination) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Ok(_) => {
            return Err(BigPEmuLifecycleError::DestinationExists {
                path: destination.to_path_buf(),
            });
        }
        Err(source) => {
            return Err(BigPEmuLifecycleError::Io {
                path: destination.to_path_buf(),
                source,
            });
        }
    }
    let mut destination_file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .map_err(|source| BigPEmuLifecycleError::Io {
            path: destination.to_path_buf(),
            source,
        })?;
    let result = (|| {
        let mut sha256 = Sha256::new();
        let mut byte_len = 0_u64;
        let mut buffer = vec![0_u8; DOWNLOAD_BUFFER_BYTES];
        progress(0, expected_byte_len);
        loop {
            if should_cancel() {
                return Err(BigPEmuLifecycleError::Cancelled);
            }
            let read = source
                .read(&mut buffer)
                .map_err(|source| BigPEmuLifecycleError::Io {
                    path: destination.to_path_buf(),
                    source,
                })?;
            if read == 0 {
                break;
            }
            destination_file
                .write_all(&buffer[..read])
                .map_err(|source| BigPEmuLifecycleError::Io {
                    path: destination.to_path_buf(),
                    source,
                })?;
            sha256.update(&buffer[..read]);
            byte_len = byte_len.saturating_add(u64::try_from(read).unwrap_or(u64::MAX));
            if byte_len > MAX_ARTIFACT_BYTES
                || (expected_byte_len > 0 && byte_len > expected_byte_len)
            {
                return Err(BigPEmuLifecycleError::DownloadTooLarge {
                    byte_len,
                    limit: expected_byte_len.min(MAX_ARTIFACT_BYTES),
                });
            }
            progress(byte_len, expected_byte_len);
        }
        destination_file
            .flush()
            .and_then(|()| destination_file.sync_all())
            .map_err(|source| BigPEmuLifecycleError::Io {
                path: destination.to_path_buf(),
                source,
            })?;
        let digest = sha256.finalize();
        let mut sha256 = String::with_capacity(digest.len() * 2);
        for byte in digest {
            write!(&mut sha256, "{byte:02x}")
                .expect("writing a SHA-256 digest to a String cannot fail");
        }
        Ok(DownloadReceipt { byte_len, sha256 })
    })();
    drop(destination_file);
    if result.is_err() {
        let _ = fs::remove_file(destination);
    }
    result
}

fn fnv1a64_file(path: &Path) -> Result<String, BigPEmuLifecycleError> {
    let mut source = File::open(path).map_err(|source| BigPEmuLifecycleError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut hash = FNV1A64_OFFSET_BASIS;
    let mut buffer = vec![0_u8; DOWNLOAD_BUFFER_BYTES];
    loop {
        let read = source
            .read(&mut buffer)
            .map_err(|source| BigPEmuLifecycleError::Io {
                path: path.to_path_buf(),
                source,
            })?;
        if read == 0 {
            break;
        }
        for byte in &buffer[..read] {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(FNV1A64_PRIME);
        }
    }
    Ok(format!("{hash:016X}"))
}

fn read_limited(
    source: impl Read,
    max_bytes: u64,
    label: &'static str,
) -> Result<Vec<u8>, BigPEmuLifecycleError> {
    let mut bytes = Vec::new();
    source
        .take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|source| BigPEmuLifecycleError::ReadResponse { label, source })?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > max_bytes {
        return Err(BigPEmuLifecycleError::ResponseTooLarge { label, max_bytes });
    }
    Ok(bytes)
}

/// Reads BigPEmu's installed version from the sibling `ReadMe.txt` used by
/// both the recovered 13.27 adapter and the current native Linux package.
///
/// This inspection is deliberately passive: the emulator is never started,
/// symlinks are rejected, and the text file has a fixed read limit.
pub fn installed_bigpemu_version(
    executable: &Path,
) -> Result<Option<String>, BigPEmuInspectionError> {
    let application_directory =
        executable
            .parent()
            .ok_or_else(|| BigPEmuInspectionError::MissingApplicationDirectory {
                executable: executable.to_path_buf(),
            })?;
    let readme = find_readme(application_directory)?;
    let Some(readme) = readme else {
        return Ok(None);
    };
    let metadata =
        fs::symlink_metadata(&readme).map_err(|source| BigPEmuInspectionError::Read {
            path: readme.clone(),
            source,
        })?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(BigPEmuInspectionError::UnsafeReadme { path: readme });
    }
    if metadata.len() > MAX_README_BYTES {
        return Err(BigPEmuInspectionError::ReadmeTooLarge {
            path: readme,
            byte_len: metadata.len(),
            max_bytes: MAX_README_BYTES,
        });
    }
    let mut bytes = Vec::with_capacity(usize::try_from(metadata.len()).unwrap_or_default());
    File::open(&readme)
        .and_then(|source| {
            source
                .take(MAX_README_BYTES.saturating_add(1))
                .read_to_end(&mut bytes)
        })
        .map_err(|source| BigPEmuInspectionError::Read {
            path: readme.clone(),
            source,
        })?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_README_BYTES {
        return Err(BigPEmuInspectionError::ReadmeTooLarge {
            path: readme,
            byte_len: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
            max_bytes: MAX_README_BYTES,
        });
    }
    let text =
        std::str::from_utf8(&bytes).map_err(|source| BigPEmuInspectionError::InvalidUtf8 {
            path: readme.clone(),
            source,
        })?;
    let Some(version) = text.lines().find_map(|line| {
        line.trim_end_matches('\r')
            .strip_prefix("Version:")
            .map(str::trim)
    }) else {
        return Ok(None);
    };
    if version.is_empty()
        || version.len() > 64
        || !version.is_ascii()
        || version
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace())
    {
        return Err(BigPEmuInspectionError::InvalidVersion {
            path: readme,
            version: version.to_string(),
        });
    }
    Ok(Some(version.to_string()))
}

fn find_readme(directory: &Path) -> Result<Option<PathBuf>, BigPEmuInspectionError> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(BigPEmuInspectionError::ReadDirectory {
                path: directory.to_path_buf(),
                source,
            });
        }
    };
    let mut matches = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| BigPEmuInspectionError::ReadDirectory {
            path: directory.to_path_buf(),
            source,
        })?;
        if entry
            .file_name()
            .to_str()
            .is_some_and(|name| name.eq_ignore_ascii_case("readme.txt"))
        {
            matches.push(entry.path());
        }
    }
    matches.sort();
    match matches.as_slice() {
        [] => Ok(None),
        [readme] => Ok(Some(readme.clone())),
        _ => Err(BigPEmuInspectionError::AmbiguousReadme {
            directory: directory.to_path_buf(),
        }),
    }
}

#[derive(Debug, Error)]
pub enum BigPEmuLifecycleError {
    #[error("BigPEmu installation is unsupported on {os}/{architecture}")]
    UnsupportedHost {
        os: &'static str,
        architecture: &'static str,
    },
    #[error("BigPEmu release page is invalid: {message}")]
    InvalidCatalog { message: String },
    #[error("BigPEmu release asset name is unsafe: {name}")]
    UnsafeAssetName { name: String },
    #[error("refusing an untrusted BigPEmu release URL: {url}")]
    UntrustedUrl { url: String },
    #[error("BigPEmu release asset {asset} has invalid size {byte_len}")]
    InvalidAssetSize { asset: String, byte_len: u64 },
    #[error("invalid uppercase 64-bit FNV-1a hash: {hash}")]
    InvalidFnv { hash: String },
    #[error("BigPEmu download transport failed: {message}")]
    Transport { message: String },
    #[error("could not read {label}: {source}")]
    ReadResponse {
        label: &'static str,
        source: io::Error,
    },
    #[error("{label} exceeded the {max_bytes}-byte limit")]
    ResponseTooLarge { label: &'static str, max_bytes: u64 },
    #[error("BigPEmu download was cancelled")]
    Cancelled,
    #[error("BigPEmu download destination already exists: {path}")]
    DestinationExists { path: PathBuf },
    #[error("BigPEmu download exceeded its {limit}-byte limit at {byte_len} bytes")]
    DownloadTooLarge { byte_len: u64, limit: u64 },
    #[error("BigPEmu download size mismatch: expected {expected}, received {actual}")]
    SizeMismatch { expected: u64, actual: u64 },
    #[error("BigPEmu FNV-1a mismatch: expected {expected}, received {actual}")]
    FnvMismatch { expected: String, actual: String },
    #[error("unsafe BigPEmu fixture file: {path}")]
    UnsafeFixture { path: PathBuf },
    #[error("BigPEmu I/O error at {path}: {source}")]
    Io { path: PathBuf, source: io::Error },
}

#[derive(Debug, Error)]
pub enum BigPEmuInspectionError {
    #[error("BigPEmu executable has no application directory: {executable}")]
    MissingApplicationDirectory { executable: PathBuf },
    #[error("could not inspect BigPEmu application directory {path}: {source}")]
    ReadDirectory { path: PathBuf, source: io::Error },
    #[error("BigPEmu application directory has ambiguous readme.txt entries: {directory}")]
    AmbiguousReadme { directory: PathBuf },
    #[error("BigPEmu readme is not a safe regular file: {path}")]
    UnsafeReadme { path: PathBuf },
    #[error("BigPEmu readme {path} is {byte_len} bytes, above the {max_bytes}-byte limit")]
    ReadmeTooLarge {
        path: PathBuf,
        byte_len: u64,
        max_bytes: u64,
    },
    #[error("could not read BigPEmu readme {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("BigPEmu readme {path} is not valid UTF-8: {source}")]
    InvalidUtf8 {
        path: PathBuf,
        source: std::str::Utf8Error,
    },
    #[error("BigPEmu readme {path} contains an invalid version value: {version:?}")]
    InvalidVersion { path: PathBuf, version: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn fnv1a64(bytes: &[u8]) -> String {
        let mut hash = FNV1A64_OFFSET_BASIS;
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(FNV1A64_PRIME);
        }
        format!("{hash:016X}")
    }

    fn release_page(version: &str, assets: &[(BigPEmuArtifactKind, &[u8])]) -> Vec<u8> {
        let mut page = format!(
            "<html><font size=\"5\">Current Version: <strong>{version}</strong></font><br /><br />"
        );
        for (kind, bytes) in assets {
            let name = kind.asset_name(version).expect("fixture asset name");
            let url = format!("{BIGPEMU_ASSET_PREFIX}{name}");
            write!(
                &mut page,
                "<strong>{}: </strong><a href=\"{url}\">{url}</a><br />Size: <i>{} bytes</i><br />Hash: <i>{} (64-bit FNV-1a)</i><br /><br />",
                kind.platform_label(),
                bytes.len(),
                fnv1a64(bytes)
            )
            .expect("fixture HTML");
        }
        page.push_str("</html>");
        page.into_bytes()
    }

    #[test]
    fn official_page_shape_selects_each_exact_host_artifact() {
        let assets = BigPEmuArtifactKind::ALL.map(|kind| {
            let bytes = match kind {
                BigPEmuArtifactKind::WindowsZipX64 => b"windows-x64".as_slice(),
                BigPEmuArtifactKind::WindowsZipArm64 => b"windows-arm64".as_slice(),
                BigPEmuArtifactKind::LinuxTarGzX64 => b"linux-x64".as_slice(),
                BigPEmuArtifactKind::LinuxTarGzArm64 => b"linux-arm64".as_slice(),
            };
            (kind, bytes)
        });
        let page = release_page("1.221", &assets);
        for (kind, bytes) in assets {
            let offer = select_bigpemu_release(&page, kind).expect("release offer");
            assert_eq!(offer.version, "1.221");
            assert_eq!(offer.artifact_kind, kind);
            assert_eq!(offer.asset_name, kind.asset_name("1.221").unwrap());
            assert_eq!(offer.asset_byte_len, bytes.len() as u64);
            assert_eq!(offer.asset_fnv1a64, fnv1a64(bytes));
            assert_eq!(
                offer.asset_url,
                format!("{BIGPEMU_ASSET_PREFIX}{}", offer.asset_name)
            );
        }
    }

    #[test]
    fn release_page_rejects_duplicates_untrusted_urls_and_inconsistent_names() {
        let bytes = b"linux";
        let page = release_page("1.221", &[(BigPEmuArtifactKind::LinuxTarGzX64, bytes)]);
        let duplicate = [page.as_slice(), page.as_slice()].concat();
        assert!(matches!(
            select_bigpemu_release(&duplicate, BigPEmuArtifactKind::LinuxTarGzX64),
            Err(BigPEmuLifecycleError::InvalidCatalog { .. })
        ));

        let untrusted = String::from_utf8(page.clone())
            .unwrap()
            .replace(BIGPEMU_ASSET_PREFIX, "https://example.com/");
        assert!(matches!(
            select_bigpemu_release(untrusted.as_bytes(), BigPEmuArtifactKind::LinuxTarGzX64),
            Err(BigPEmuLifecycleError::UntrustedUrl { .. })
        ));

        let wrong_name = String::from_utf8(page)
            .unwrap()
            .replace("BigPEmu_Linux64_v1221.tar.gz", "BigPEmu_v1221.zip");
        assert!(matches!(
            select_bigpemu_release(wrong_name.as_bytes(), BigPEmuArtifactKind::LinuxTarGzX64),
            Err(BigPEmuLifecycleError::UntrustedUrl { .. })
        ));
    }

    #[test]
    fn file_transport_streams_and_verifies_official_fnv_with_cleanup() {
        let fixture = tempfile::tempdir().expect("release fixture");
        let destination_root = tempfile::tempdir().expect("destination");
        let bytes = vec![0x5a; DOWNLOAD_BUFFER_BYTES * 2 + 17];
        let kind = BigPEmuArtifactKind::LinuxTarGzX64;
        let page = release_page("1.221", &[(kind, &bytes)]);
        fs::write(fixture.path().join("download.html"), page).expect("release page");
        let name = kind.asset_name("1.221").unwrap();
        fs::write(fixture.path().join(&name), &bytes).expect("release asset");
        let transport = FileBigPEmuReleaseTransport::new(fixture.path());
        let offer = fetch_latest_bigpemu_release(&transport, kind).expect("offer");
        let progress_calls = AtomicUsize::new(0);
        let destination = destination_root.path().join("download");
        let receipt = download_bigpemu_release(
            &transport,
            &offer,
            &destination,
            &mut |current, total| {
                assert!(current <= total);
                progress_calls.fetch_add(1, Ordering::Relaxed);
            },
            &|| false,
        )
        .expect("verified download");
        assert_eq!(receipt.byte_len, bytes.len() as u64);
        assert_eq!(receipt.sha256, format!("{:x}", Sha256::digest(&bytes)));
        assert_eq!(fs::read(&destination).unwrap(), bytes);
        assert!(progress_calls.load(Ordering::Relaxed) >= 3);

        let mut wrong = offer;
        wrong.asset_fnv1a64 = "0000000000000000".into();
        let mismatch = destination_root.path().join("mismatch");
        assert!(matches!(
            download_bigpemu_release(&transport, &wrong, &mismatch, &mut |_, _| {}, &|| false),
            Err(BigPEmuLifecycleError::FnvMismatch { .. })
        ));
        assert!(!mismatch.exists());
    }

    #[test]
    fn cancelled_download_removes_partial_destination() {
        let fixture = tempfile::tempdir().expect("release fixture");
        let destination_root = tempfile::tempdir().expect("destination");
        let bytes = vec![0x3c; DOWNLOAD_BUFFER_BYTES * 2];
        let kind = BigPEmuArtifactKind::LinuxTarGzX64;
        fs::write(
            fixture.path().join("download.html"),
            release_page("1.221", &[(kind, &bytes)]),
        )
        .expect("release page");
        let name = kind.asset_name("1.221").unwrap();
        fs::write(fixture.path().join(&name), &bytes).expect("release asset");
        let transport = FileBigPEmuReleaseTransport::new(fixture.path());
        let offer = fetch_latest_bigpemu_release(&transport, kind).expect("offer");
        let calls = AtomicUsize::new(0);
        let destination = destination_root.path().join("cancelled");
        assert!(matches!(
            download_bigpemu_release(&transport, &offer, &destination, &mut |_, _| {}, &|| calls
                .fetch_add(1, Ordering::Relaxed)
                > 0),
            Err(BigPEmuLifecycleError::Cancelled)
        ));
        assert!(!destination.exists());
    }

    #[test]
    fn reads_current_linux_and_legacy_windows_readme_casing_without_execution() {
        for name in ["ReadMe.txt", "readme.txt"] {
            let directory = tempfile::tempdir().expect("temporary BigPEmu");
            let executable = directory.path().join("bigpemu");
            fs::write(&executable, b"not an executable image").expect("fixture executable");
            fs::write(
                directory.path().join(name),
                b"Title: BigPEmu\r\nVersion: 1.221\r\nAuthor: Rich Whitehouse\r\n",
            )
            .expect("fixture readme");
            assert_eq!(
                installed_bigpemu_version(&executable).expect("version inspection"),
                Some("1.221".into())
            );
            assert_eq!(
                fs::read(&executable).expect("unchanged executable"),
                b"not an executable image"
            );
        }
    }

    #[test]
    fn missing_version_is_not_invented_and_malformed_values_are_rejected() {
        let directory = tempfile::tempdir().expect("temporary BigPEmu");
        let executable = directory.path().join("bigpemu");
        fs::write(directory.path().join("ReadMe.txt"), b"Title: BigPEmu\n")
            .expect("fixture readme");
        assert_eq!(
            installed_bigpemu_version(&executable).expect("missing version"),
            None
        );

        fs::write(
            directory.path().join("ReadMe.txt"),
            b"Version: unsafe value\n",
        )
        .expect("malformed readme");
        assert!(matches!(
            installed_bigpemu_version(&executable),
            Err(BigPEmuInspectionError::InvalidVersion { .. })
        ));
    }

    #[test]
    fn oversized_and_non_utf8_readmes_are_rejected_before_version_parsing() {
        let directory = tempfile::tempdir().expect("temporary BigPEmu");
        let executable = directory.path().join("bigpemu");
        let readme = directory.path().join("ReadMe.txt");
        let oversized = File::create(&readme).expect("oversized readme");
        oversized
            .set_len(MAX_README_BYTES + 1)
            .expect("extend readme");
        assert!(matches!(
            installed_bigpemu_version(&executable),
            Err(BigPEmuInspectionError::ReadmeTooLarge { .. })
        ));

        fs::write(&readme, [0xff, 0xfe]).expect("non-UTF-8 readme");
        assert!(matches!(
            installed_bigpemu_version(&executable),
            Err(BigPEmuInspectionError::InvalidUtf8 { .. })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn readme_symlinks_and_case_ambiguous_files_are_rejected() {
        use std::os::unix::fs::symlink;

        let directory = tempfile::tempdir().expect("temporary BigPEmu");
        let executable = directory.path().join("bigpemu");
        let external = directory.path().join("external.txt");
        fs::write(&external, b"Version: 1.221\n").expect("external readme");
        symlink(&external, directory.path().join("ReadMe.txt")).expect("readme link");
        assert!(matches!(
            installed_bigpemu_version(&executable),
            Err(BigPEmuInspectionError::UnsafeReadme { .. })
        ));

        fs::remove_file(directory.path().join("ReadMe.txt")).expect("remove link");
        fs::write(directory.path().join("ReadMe.txt"), b"Version: 1.221\n").expect("first readme");
        fs::write(directory.path().join("readme.txt"), b"Version: 1.22\n").expect("second readme");
        assert!(matches!(
            installed_bigpemu_version(&executable),
            Err(BigPEmuInspectionError::AmbiguousReadme { .. })
        ));
    }
}
