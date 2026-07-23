use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const PCSX2_RELEASES_API: &str =
    "https://api.github.com/repos/PCSX2/pcsx2/releases?per_page=20";
pub const MANAGED_INSTALL_MANIFEST_NAME: &str = ".launchbox-port-install.json";
pub const MANAGED_INSTALL_MANIFEST_VERSION: u32 = 2;
const LEGACY_MANAGED_INSTALL_MANIFEST_VERSION: u32 = 1;

const GITHUB_ASSET_PREFIX: &str = "https://github.com/PCSX2/pcsx2/releases/download/";
const MAX_RELEASE_CATALOG_BYTES: u64 = 8 * 1024 * 1024;
const MAX_ARTIFACT_BYTES: u64 = 512 * 1024 * 1024;
const MAX_MANIFEST_BYTES: u64 = 4 * 1024 * 1024;
const MAX_MANAGED_FILES: usize = 16 * 1024;
const DOWNLOAD_BUFFER_BYTES: usize = 128 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Pcsx2ArtifactKind {
    LinuxAppImageX64,
    WindowsQt7zX64,
    MacosQtTarXz,
}

impl Pcsx2ArtifactKind {
    pub const fn id(self) -> &'static str {
        match self {
            Self::LinuxAppImageX64 => "linux_appimage_x64",
            Self::WindowsQt7zX64 => "windows_qt_7z_x64",
            Self::MacosQtTarXz => "macos_qt_tar_xz",
        }
    }

    pub const fn asset_suffix(self) -> &'static str {
        match self {
            Self::LinuxAppImageX64 => "-linux-appimage-x64-Qt.AppImage",
            Self::WindowsQt7zX64 => "-windows-x64-Qt.7z",
            Self::MacosQtTarXz => "-macos-Qt.tar.xz",
        }
    }

    pub const fn executable_name(self) -> &'static str {
        match self {
            Self::LinuxAppImageX64 => "pcsx2-qt.AppImage",
            Self::WindowsQt7zX64 => "pcsx2-qt.exe",
            Self::MacosQtTarXz => "pcsx2-qt",
        }
    }

    pub const fn requires_extraction(self) -> bool {
        !matches!(self, Self::LinuxAppImageX64)
    }

    pub fn current_host() -> Result<Self, EmulatorLifecycleError> {
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            return Ok(Self::LinuxAppImageX64);
        }
        #[cfg(all(windows, target_arch = "x86_64"))]
        {
            return Ok(Self::WindowsQt7zX64);
        }
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        {
            return Ok(Self::MacosQtTarXz);
        }
        #[allow(unreachable_code)]
        Err(EmulatorLifecycleError::UnsupportedHost {
            os: std::env::consts::OS,
            architecture: std::env::consts::ARCH,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Pcsx2ReleaseOffer {
    pub version: String,
    pub tag: String,
    pub release_name: String,
    pub release_url: String,
    pub prerelease: bool,
    pub artifact_kind: Pcsx2ArtifactKind,
    pub asset_name: String,
    pub asset_url: String,
    pub asset_byte_len: u64,
    pub asset_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DownloadReceipt {
    pub byte_len: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ManagedInstalledFile {
    pub relative_path: String,
    pub byte_len: u64,
    pub sha256: String,
}

impl ManagedInstalledFile {
    pub fn new(
        relative_path: &Path,
        receipt: &DownloadReceipt,
    ) -> Result<Self, EmulatorLifecycleError> {
        let relative_path = portable_relative_path(relative_path)?;
        let file = Self {
            relative_path,
            byte_len: receipt.byte_len,
            sha256: receipt.sha256.clone(),
        };
        file.validate()?;
        Ok(file)
    }

    pub fn host_relative_path(&self) -> Result<PathBuf, EmulatorLifecycleError> {
        validate_portable_relative_path(&self.relative_path)?;
        Ok(self.relative_path.split('/').collect())
    }

    fn validate(&self) -> Result<(), EmulatorLifecycleError> {
        validate_portable_relative_path(&self.relative_path)?;
        validate_sha256(&self.sha256)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ManagedEmulatorInstall {
    pub schema_version: u32,
    pub profile_id: String,
    pub provider: String,
    #[serde(default)]
    pub emulator_id: Option<String>,
    pub version: String,
    pub tag: String,
    pub prerelease: bool,
    pub artifact_kind: Pcsx2ArtifactKind,
    pub asset_name: String,
    pub asset_url: String,
    pub asset_byte_len: u64,
    pub asset_sha256: String,
    pub executable_name: String,
    pub executable_byte_len: u64,
    pub executable_sha256: String,
    #[serde(default)]
    pub installed_files: Vec<ManagedInstalledFile>,
}

impl ManagedEmulatorInstall {
    pub fn from_offer(
        offer: &Pcsx2ReleaseOffer,
        executable: &DownloadReceipt,
        emulator_id: String,
        installed_files: Vec<ManagedInstalledFile>,
    ) -> Result<Self, EmulatorLifecycleError> {
        let manifest = Self {
            schema_version: MANAGED_INSTALL_MANIFEST_VERSION,
            profile_id: "pcsx2".into(),
            provider: "github:PCSX2/pcsx2".into(),
            emulator_id: Some(emulator_id),
            version: offer.version.clone(),
            tag: offer.tag.clone(),
            prerelease: offer.prerelease,
            artifact_kind: offer.artifact_kind,
            asset_name: offer.asset_name.clone(),
            asset_url: offer.asset_url.clone(),
            asset_byte_len: offer.asset_byte_len,
            asset_sha256: offer.asset_sha256.clone(),
            executable_name: offer.artifact_kind.executable_name().into(),
            executable_byte_len: executable.byte_len,
            executable_sha256: executable.sha256.clone(),
            installed_files,
        };
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), EmulatorLifecycleError> {
        if !matches!(
            self.schema_version,
            LEGACY_MANAGED_INSTALL_MANIFEST_VERSION | MANAGED_INSTALL_MANIFEST_VERSION
        ) {
            return Err(EmulatorLifecycleError::UnsupportedManifestVersion {
                version: self.schema_version,
            });
        }
        if self.profile_id != "pcsx2" || self.provider != "github:PCSX2/pcsx2" {
            return Err(EmulatorLifecycleError::InvalidManifest {
                message: "managed install profile or provider is not PCSX2".into(),
            });
        }
        for (field, value) in [
            ("version", self.version.as_str()),
            ("tag", self.tag.as_str()),
            ("asset_name", self.asset_name.as_str()),
            ("asset_url", self.asset_url.as_str()),
            ("executable_name", self.executable_name.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(EmulatorLifecycleError::InvalidManifest {
                    message: format!("{field} is empty"),
                });
            }
        }
        validate_asset_name(&self.asset_name)?;
        validate_github_asset_url(&self.asset_url)?;
        if self.executable_name != self.artifact_kind.executable_name()
            || self.asset_byte_len == 0
            || self.asset_byte_len > MAX_ARTIFACT_BYTES
            || self.executable_byte_len == 0
        {
            return Err(EmulatorLifecycleError::InvalidManifest {
                message: "artifact or executable metadata is inconsistent".into(),
            });
        }
        validate_sha256(&self.asset_sha256)?;
        validate_sha256(&self.executable_sha256)?;

        if self.schema_version == LEGACY_MANAGED_INSTALL_MANIFEST_VERSION {
            if self.emulator_id.is_some() || !self.installed_files.is_empty() {
                return Err(EmulatorLifecycleError::InvalidManifest {
                    message: "legacy manifest unexpectedly contains version 2 ownership data"
                        .into(),
                });
            }
            return Ok(());
        }

        if self.emulator_id.as_deref().is_none_or(|id| {
            id.trim().is_empty() || id.len() > 1024 || id.chars().any(char::is_control)
        }) {
            return Err(EmulatorLifecycleError::InvalidManifest {
                message: "managed emulator ID is empty".into(),
            });
        }
        if self.installed_files.is_empty() || self.installed_files.len() > MAX_MANAGED_FILES {
            return Err(EmulatorLifecycleError::InvalidManifest {
                message: format!(
                    "managed install owns invalid file count {}",
                    self.installed_files.len()
                ),
            });
        }
        let mut normalized_paths = std::collections::BTreeSet::new();
        let mut executable_matches = 0;
        for file in &self.installed_files {
            file.validate()?;
            if !normalized_paths.insert(file.relative_path.to_ascii_lowercase()) {
                return Err(EmulatorLifecycleError::InvalidManifest {
                    message: format!("managed install owns duplicate path {}", file.relative_path),
                });
            }
            if file.relative_path == self.executable_name {
                executable_matches += 1;
                if file.byte_len != self.executable_byte_len
                    || file.sha256 != self.executable_sha256
                {
                    return Err(EmulatorLifecycleError::InvalidManifest {
                        message: "managed executable ownership metadata is inconsistent".into(),
                    });
                }
            }
        }
        if executable_matches != 1 {
            return Err(EmulatorLifecycleError::InvalidManifest {
                message: "managed executable is not owned exactly once".into(),
            });
        }
        Ok(())
    }

    pub fn to_json_bytes(&self) -> Result<Vec<u8>, EmulatorLifecycleError> {
        self.validate()?;
        let mut bytes = serde_json::to_vec_pretty(self).map_err(|error| {
            EmulatorLifecycleError::InvalidManifest {
                message: error.to_string(),
            }
        })?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ManagedExecutableState {
    Valid,
    Missing,
    Modified,
    Unsafe,
    Unreadable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ManagedInstallAudit {
    pub manifest: ManagedEmulatorInstall,
    pub manifest_path: PathBuf,
    pub executable_path: PathBuf,
    pub executable_state: ManagedExecutableState,
    pub actual_executable: Option<DownloadReceipt>,
    pub installed_files: Vec<ManagedInstalledFileAudit>,
}

impl ManagedInstallAudit {
    pub fn update_available(&self, offer: &Pcsx2ReleaseOffer) -> bool {
        self.manifest.tag != offer.tag
            || self.manifest.asset_sha256 != offer.asset_sha256
            || self.manifest.artifact_kind != offer.artifact_kind
    }

    pub fn safe_to_update(&self) -> bool {
        self.executable_state == ManagedExecutableState::Valid
    }

    pub fn ownership_manifest_current(&self) -> bool {
        self.manifest.schema_version == MANAGED_INSTALL_MANIFEST_VERSION
    }

    pub fn safe_to_remove(&self) -> bool {
        self.ownership_manifest_current()
            && !self.installed_files.is_empty()
            && self
                .installed_files
                .iter()
                .all(|file| file.state == ManagedExecutableState::Valid)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ManagedInstalledFileAudit {
    pub relative_path: String,
    pub path: PathBuf,
    pub state: ManagedExecutableState,
    pub actual: Option<DownloadReceipt>,
}

pub trait ReleaseTransport: Send + Sync {
    fn fetch_catalog(&self, url: &str, max_bytes: u64) -> Result<Vec<u8>, EmulatorLifecycleError>;

    fn download(
        &self,
        url: &str,
        destination: &Path,
        expected_byte_len: u64,
        progress: &mut dyn FnMut(u64, u64),
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<DownloadReceipt, EmulatorLifecycleError>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GithubReleaseTransport;

impl ReleaseTransport for GithubReleaseTransport {
    fn fetch_catalog(&self, url: &str, max_bytes: u64) -> Result<Vec<u8>, EmulatorLifecycleError> {
        if url != PCSX2_RELEASES_API {
            return Err(EmulatorLifecycleError::UntrustedUrl {
                url: url.to_string(),
            });
        }
        let mut response = ureq::get(url)
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "lunchbox-qt")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .call()
            .map_err(|error| EmulatorLifecycleError::Transport {
                message: error.to_string(),
            })?;
        read_limited(
            response.body_mut().as_reader(),
            max_bytes,
            "PCSX2 release catalog",
        )
    }

    fn download(
        &self,
        url: &str,
        destination: &Path,
        expected_byte_len: u64,
        progress: &mut dyn FnMut(u64, u64),
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<DownloadReceipt, EmulatorLifecycleError> {
        validate_github_asset_url(url)?;
        if should_cancel() {
            return Err(EmulatorLifecycleError::Cancelled);
        }
        let mut response = ureq::get(url)
            .header("Accept", "application/octet-stream")
            .header("User-Agent", "lunchbox-qt")
            .call()
            .map_err(|error| EmulatorLifecycleError::Transport {
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
pub struct FileReleaseTransport {
    root: PathBuf,
}

impl FileReleaseTransport {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

impl ReleaseTransport for FileReleaseTransport {
    fn fetch_catalog(&self, _url: &str, max_bytes: u64) -> Result<Vec<u8>, EmulatorLifecycleError> {
        let path = self.root.join("releases.json");
        let metadata =
            fs::symlink_metadata(&path).map_err(|source| EmulatorLifecycleError::Io {
                path: path.clone(),
                source,
            })?;
        if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
            return Err(EmulatorLifecycleError::UnsafeFixture { path });
        }
        let source = File::open(&path).map_err(|source| EmulatorLifecycleError::Io {
            path: path.clone(),
            source,
        })?;
        read_limited(source, max_bytes, "PCSX2 fixture release catalog")
    }

    fn download(
        &self,
        url: &str,
        destination: &Path,
        expected_byte_len: u64,
        progress: &mut dyn FnMut(u64, u64),
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<DownloadReceipt, EmulatorLifecycleError> {
        let asset_name = url.rsplit('/').next().unwrap_or_default();
        validate_asset_name(asset_name)?;
        let source_path = self.root.join(asset_name);
        let metadata =
            fs::symlink_metadata(&source_path).map_err(|source| EmulatorLifecycleError::Io {
                path: source_path.clone(),
                source,
            })?;
        if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
            return Err(EmulatorLifecycleError::UnsafeFixture { path: source_path });
        }
        let source = File::open(&source_path).map_err(|source| EmulatorLifecycleError::Io {
            path: source_path.clone(),
            source,
        })?;
        stream_download(
            source,
            destination,
            expected_byte_len,
            progress,
            should_cancel,
        )
    }
}

pub fn fetch_latest_pcsx2_release(
    transport: &dyn ReleaseTransport,
    artifact_kind: Pcsx2ArtifactKind,
) -> Result<Pcsx2ReleaseOffer, EmulatorLifecycleError> {
    let catalog = transport.fetch_catalog(PCSX2_RELEASES_API, MAX_RELEASE_CATALOG_BYTES)?;
    select_pcsx2_release(&catalog, artifact_kind)
}

pub fn select_pcsx2_release(
    catalog: &[u8],
    artifact_kind: Pcsx2ArtifactKind,
) -> Result<Pcsx2ReleaseOffer, EmulatorLifecycleError> {
    let releases: Vec<GithubRelease> = serde_json::from_slice(catalog).map_err(|error| {
        EmulatorLifecycleError::InvalidCatalog {
            message: error.to_string(),
        }
    })?;
    for release in releases.into_iter().filter(|release| !release.draft) {
        let Some(asset) = release
            .assets
            .iter()
            .find(|asset| asset.name.ends_with(artifact_kind.asset_suffix()))
        else {
            continue;
        };
        validate_asset_name(&asset.name)?;
        validate_github_asset_url(&asset.browser_download_url)?;
        let digest = asset
            .digest
            .as_deref()
            .and_then(|digest| digest.strip_prefix("sha256:"))
            .ok_or_else(|| EmulatorLifecycleError::MissingDigest {
                asset: asset.name.clone(),
            })?
            .to_ascii_lowercase();
        validate_sha256(&digest)?;
        if asset.size == 0 || asset.size > MAX_ARTIFACT_BYTES {
            return Err(EmulatorLifecycleError::InvalidAssetSize {
                asset: asset.name.clone(),
                byte_len: asset.size,
            });
        }
        let tag = release.tag_name.trim();
        let version = tag.strip_prefix(['v', 'V']).unwrap_or(tag).trim();
        if tag.is_empty() || version.is_empty() {
            return Err(EmulatorLifecycleError::InvalidCatalog {
                message: "release tag or version is empty".into(),
            });
        }
        return Ok(Pcsx2ReleaseOffer {
            version: version.to_string(),
            tag: tag.to_string(),
            release_name: release.name.unwrap_or_else(|| tag.to_string()),
            release_url: release
                .html_url
                .unwrap_or_else(|| format!("https://github.com/PCSX2/pcsx2/releases/tag/{tag}")),
            prerelease: release.prerelease,
            artifact_kind,
            asset_name: asset.name.clone(),
            asset_url: asset.browser_download_url.clone(),
            asset_byte_len: asset.size,
            asset_sha256: digest,
        });
    }
    Err(EmulatorLifecycleError::NoCompatibleRelease {
        artifact_kind: artifact_kind.id(),
    })
}

pub fn download_pcsx2_release(
    transport: &dyn ReleaseTransport,
    offer: &Pcsx2ReleaseOffer,
    destination: &Path,
    progress: &mut dyn FnMut(u64, u64),
    should_cancel: &dyn Fn() -> bool,
) -> Result<DownloadReceipt, EmulatorLifecycleError> {
    validate_github_asset_url(&offer.asset_url)?;
    validate_asset_name(&offer.asset_name)?;
    validate_sha256(&offer.asset_sha256)?;
    if offer.asset_byte_len == 0 || offer.asset_byte_len > MAX_ARTIFACT_BYTES {
        return Err(EmulatorLifecycleError::InvalidAssetSize {
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
        return Err(EmulatorLifecycleError::SizeMismatch {
            expected: offer.asset_byte_len,
            actual: receipt.byte_len,
        });
    }
    if !receipt.sha256.eq_ignore_ascii_case(&offer.asset_sha256) {
        let _ = fs::remove_file(destination);
        return Err(EmulatorLifecycleError::DigestMismatch {
            expected: offer.asset_sha256.clone(),
            actual: receipt.sha256,
        });
    }
    Ok(receipt)
}

pub fn read_managed_pcsx2_install(
    install_directory: &Path,
) -> Result<Option<ManagedInstallAudit>, EmulatorLifecycleError> {
    let manifest_path = install_directory.join(MANAGED_INSTALL_MANIFEST_NAME);
    let metadata = match fs::symlink_metadata(&manifest_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(EmulatorLifecycleError::Io {
                path: manifest_path,
                source,
            })
        }
    };
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(EmulatorLifecycleError::UnsafeManifest {
            path: manifest_path,
        });
    }
    let source = File::open(&manifest_path).map_err(|source| EmulatorLifecycleError::Io {
        path: manifest_path.clone(),
        source,
    })?;
    let bytes = read_limited(source, MAX_MANIFEST_BYTES, "managed emulator manifest")?;
    let manifest: ManagedEmulatorInstall = serde_json::from_slice(&bytes).map_err(|error| {
        EmulatorLifecycleError::InvalidManifest {
            message: error.to_string(),
        }
    })?;
    manifest.validate()?;
    let executable_path = install_directory.join(&manifest.executable_name);
    let (executable_state, actual_executable) = audit_managed_executable(
        &executable_path,
        manifest.executable_byte_len,
        &manifest.executable_sha256,
    );
    let installed_files = manifest
        .installed_files
        .iter()
        .map(|file| {
            let relative = file.host_relative_path()?;
            let path = install_directory.join(relative);
            let (state, actual) = audit_managed_executable(&path, file.byte_len, &file.sha256);
            Ok(ManagedInstalledFileAudit {
                relative_path: file.relative_path.clone(),
                path,
                state,
                actual,
            })
        })
        .collect::<Result<Vec<_>, EmulatorLifecycleError>>()?;
    Ok(Some(ManagedInstallAudit {
        manifest,
        manifest_path,
        executable_path,
        executable_state,
        actual_executable,
        installed_files,
    }))
}

pub fn file_receipt(path: &Path) -> Result<DownloadReceipt, EmulatorLifecycleError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| EmulatorLifecycleError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(EmulatorLifecycleError::UnsafeExecutable {
            path: path.to_path_buf(),
        });
    }
    let source = File::open(path).map_err(|source| EmulatorLifecycleError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    hash_reader(source).map_err(|source| EmulatorLifecycleError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn audit_managed_executable(
    path: &Path,
    expected_byte_len: u64,
    expected_sha256: &str,
) -> (ManagedExecutableState, Option<DownloadReceipt>) {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return (ManagedExecutableState::Missing, None);
        }
        Err(_) => return (ManagedExecutableState::Unreadable, None),
    };
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return (ManagedExecutableState::Unsafe, None);
    }
    let receipt = match file_receipt(path) {
        Ok(receipt) => receipt,
        Err(_) => return (ManagedExecutableState::Unreadable, None),
    };
    let state = if receipt.byte_len == expected_byte_len && receipt.sha256 == expected_sha256 {
        ManagedExecutableState::Valid
    } else {
        ManagedExecutableState::Modified
    };
    (state, Some(receipt))
}

fn portable_relative_path(path: &Path) -> Result<String, EmulatorLifecycleError> {
    use std::path::Component;

    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => {
                let part =
                    part.to_str()
                        .ok_or_else(|| EmulatorLifecycleError::InvalidManifest {
                            message: format!(
                                "managed path is not valid UTF-8: {}",
                                path.to_string_lossy()
                            ),
                        })?;
                parts.push(part);
            }
            _ => {
                return Err(EmulatorLifecycleError::InvalidManifest {
                    message: format!(
                        "managed path is not relative and normalized: {}",
                        path.display()
                    ),
                });
            }
        }
    }
    let portable = parts.join("/");
    validate_portable_relative_path(&portable)?;
    Ok(portable)
}

fn validate_portable_relative_path(path: &str) -> Result<(), EmulatorLifecycleError> {
    if path.is_empty()
        || path.len() > 4096
        || path.starts_with('/')
        || path.ends_with('/')
        || path.contains('\\')
        || path
            .split('/')
            .any(|part| !portable_component_is_safe(part))
        || path.eq_ignore_ascii_case(MANAGED_INSTALL_MANIFEST_NAME)
    {
        return Err(EmulatorLifecycleError::InvalidManifest {
            message: format!("managed relative path is unsafe or reserved: {path}"),
        });
    }
    Ok(())
}

fn portable_component_is_safe(component: &str) -> bool {
    if component.is_empty()
        || component.len() > 255
        || matches!(component, "." | "..")
        || component.ends_with('.')
        || component.ends_with(' ')
        || component
            .chars()
            .any(|character| character.is_control() || r#"<>:"|?*"#.contains(character))
    {
        return false;
    }
    let stem = component
        .split_once('.')
        .map_or(component, |(stem, _extension)| stem)
        .to_ascii_lowercase();
    !matches!(stem.as_str(), "con" | "prn" | "aux" | "nul")
        && !(stem.len() == 4
            && (stem.starts_with("com") || stem.starts_with("lpt"))
            && stem.as_bytes()[3].is_ascii_digit()
            && stem.as_bytes()[3] != b'0')
}

fn stream_download(
    mut source: impl Read,
    destination: &Path,
    expected_byte_len: u64,
    progress: &mut dyn FnMut(u64, u64),
    should_cancel: &dyn Fn() -> bool,
) -> Result<DownloadReceipt, EmulatorLifecycleError> {
    match fs::symlink_metadata(destination) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Ok(_) => {
            return Err(EmulatorLifecycleError::DestinationExists {
                path: destination.to_path_buf(),
            })
        }
        Err(source) => {
            return Err(EmulatorLifecycleError::Io {
                path: destination.to_path_buf(),
                source,
            })
        }
    }
    let mut destination_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .map_err(|source| EmulatorLifecycleError::Io {
            path: destination.to_path_buf(),
            source,
        })?;
    let result = (|| {
        let mut digest = Sha256::new();
        let mut byte_len = 0_u64;
        let mut buffer = vec![0_u8; DOWNLOAD_BUFFER_BYTES];
        progress(0, expected_byte_len);
        loop {
            if should_cancel() {
                return Err(EmulatorLifecycleError::Cancelled);
            }
            let read = source
                .read(&mut buffer)
                .map_err(|source| EmulatorLifecycleError::Io {
                    path: destination.to_path_buf(),
                    source,
                })?;
            if read == 0 {
                break;
            }
            destination_file
                .write_all(&buffer[..read])
                .map_err(|source| EmulatorLifecycleError::Io {
                    path: destination.to_path_buf(),
                    source,
                })?;
            digest.update(&buffer[..read]);
            byte_len = byte_len.saturating_add(u64::try_from(read).unwrap_or(u64::MAX));
            if byte_len > MAX_ARTIFACT_BYTES
                || (expected_byte_len > 0 && byte_len > expected_byte_len)
            {
                return Err(EmulatorLifecycleError::DownloadTooLarge {
                    byte_len,
                    limit: expected_byte_len.min(MAX_ARTIFACT_BYTES),
                });
            }
            progress(byte_len, expected_byte_len);
        }
        destination_file
            .flush()
            .and_then(|()| destination_file.sync_all())
            .map_err(|source| EmulatorLifecycleError::Io {
                path: destination.to_path_buf(),
                source,
            })?;
        let digest = digest.finalize();
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

fn hash_reader(mut source: impl Read) -> io::Result<DownloadReceipt> {
    let mut digest = Sha256::new();
    let mut byte_len = 0_u64;
    let mut buffer = vec![0_u8; DOWNLOAD_BUFFER_BYTES];
    loop {
        let read = source.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
        byte_len = byte_len.saturating_add(u64::try_from(read).unwrap_or(u64::MAX));
    }
    let digest = digest.finalize();
    let mut sha256 = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut sha256, "{byte:02x}")
            .expect("writing a SHA-256 digest to a String cannot fail");
    }
    Ok(DownloadReceipt { byte_len, sha256 })
}

fn read_limited(
    source: impl Read,
    max_bytes: u64,
    label: &'static str,
) -> Result<Vec<u8>, EmulatorLifecycleError> {
    let mut bytes = Vec::new();
    source
        .take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|source| EmulatorLifecycleError::ReadResponse { label, source })?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > max_bytes {
        return Err(EmulatorLifecycleError::ResponseTooLarge { label, max_bytes });
    }
    Ok(bytes)
}

fn validate_asset_name(name: &str) -> Result<(), EmulatorLifecycleError> {
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
        return Err(EmulatorLifecycleError::UnsafeAssetName {
            name: name.to_string(),
        });
    }
    Ok(())
}

fn validate_github_asset_url(url: &str) -> Result<(), EmulatorLifecycleError> {
    if !url.starts_with(GITHUB_ASSET_PREFIX)
        || url.contains(['\r', '\n'])
        || url
            .split('?')
            .next()
            .unwrap_or(url)
            .rsplit('/')
            .next()
            .is_none()
    {
        return Err(EmulatorLifecycleError::UntrustedUrl {
            url: url.to_string(),
        });
    }
    Ok(())
}

fn validate_sha256(digest: &str) -> Result<(), EmulatorLifecycleError> {
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(EmulatorLifecycleError::InvalidDigest {
            digest: digest.to_string(),
        });
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    name: Option<String>,
    html_url: Option<String>,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
    digest: Option<String>,
}

#[derive(Debug, Error)]
pub enum EmulatorLifecycleError {
    #[error("emulator installation is unsupported on {os}/{architecture}")]
    UnsupportedHost {
        os: &'static str,
        architecture: &'static str,
    },
    #[error("could not find a compatible PCSX2 release for {artifact_kind}")]
    NoCompatibleRelease { artifact_kind: &'static str },
    #[error("PCSX2 release catalog is invalid: {message}")]
    InvalidCatalog { message: String },
    #[error("PCSX2 release asset {asset} has no GitHub SHA-256 digest")]
    MissingDigest { asset: String },
    #[error("PCSX2 release asset name is unsafe: {name}")]
    UnsafeAssetName { name: String },
    #[error("refusing an untrusted emulator release URL: {url}")]
    UntrustedUrl { url: String },
    #[error("PCSX2 release asset {asset} has invalid size {byte_len}")]
    InvalidAssetSize { asset: String, byte_len: u64 },
    #[error("invalid lowercase SHA-256 digest: {digest}")]
    InvalidDigest { digest: String },
    #[error("emulator download transport failed: {message}")]
    Transport { message: String },
    #[error("could not read {label}: {source}")]
    ReadResponse {
        label: &'static str,
        source: io::Error,
    },
    #[error("{label} exceeded the {max_bytes}-byte limit")]
    ResponseTooLarge { label: &'static str, max_bytes: u64 },
    #[error("download was cancelled")]
    Cancelled,
    #[error("download destination already exists: {path}")]
    DestinationExists { path: PathBuf },
    #[error("download exceeded its {limit}-byte limit at {byte_len} bytes")]
    DownloadTooLarge { byte_len: u64, limit: u64 },
    #[error("download size mismatch: expected {expected}, received {actual}")]
    SizeMismatch { expected: u64, actual: u64 },
    #[error("download SHA-256 mismatch: expected {expected}, received {actual}")]
    DigestMismatch { expected: String, actual: String },
    #[error("unsafe fixture file: {path}")]
    UnsafeFixture { path: PathBuf },
    #[error("managed install manifest is unsafe: {path}")]
    UnsafeManifest { path: PathBuf },
    #[error("managed emulator executable is unsafe: {path}")]
    UnsafeExecutable { path: PathBuf },
    #[error("managed install manifest version {version} is unsupported")]
    UnsupportedManifestVersion { version: u32 },
    #[error("managed install manifest is invalid: {message}")]
    InvalidManifest { message: String },
    #[error("I/O error at {path}: {source}")]
    Io { path: PathBuf, source: io::Error },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn release_catalog_selects_exact_current_host_artifacts_and_official_digest() {
        let linux_bytes = b"linux appimage fixture";
        let windows_bytes = b"windows archive fixture";
        let catalog = catalog_json(linux_bytes, windows_bytes);

        let linux = select_pcsx2_release(&catalog, Pcsx2ArtifactKind::LinuxAppImageX64).unwrap();
        assert_eq!(linux.version, "2.7.492");
        assert!(linux.prerelease);
        assert_eq!(
            linux.asset_sha256,
            format!("{:x}", Sha256::digest(linux_bytes))
        );
        assert!(linux
            .asset_name
            .ends_with("-linux-appimage-x64-Qt.AppImage"));

        let windows = select_pcsx2_release(&catalog, Pcsx2ArtifactKind::WindowsQt7zX64).unwrap();
        assert!(windows.asset_name.ends_with("-windows-x64-Qt.7z"));
        assert!(!windows.asset_name.contains("symbols"));
        assert_eq!(
            windows.asset_sha256,
            format!("{:x}", Sha256::digest(windows_bytes))
        );
    }

    #[test]
    fn release_catalog_refuses_missing_digest_untrusted_url_and_oversize_asset() {
        let valid = catalog_json(b"linux", b"windows");
        let missing = String::from_utf8(valid.clone())
            .unwrap()
            .replace("\"digest\":\"sha256:", "\"other_digest\":\"sha256:");
        assert!(matches!(
            select_pcsx2_release(missing.as_bytes(), Pcsx2ArtifactKind::LinuxAppImageX64),
            Err(EmulatorLifecycleError::MissingDigest { .. })
        ));

        let untrusted = String::from_utf8(valid.clone()).unwrap().replace(
            "https://github.com/PCSX2/pcsx2/releases/download/",
            "https://example.com/",
        );
        assert!(matches!(
            select_pcsx2_release(untrusted.as_bytes(), Pcsx2ArtifactKind::LinuxAppImageX64),
            Err(EmulatorLifecycleError::UntrustedUrl { .. })
        ));

        let oversize = String::from_utf8(valid).unwrap().replace(
            "\"size\":5",
            &format!("\"size\":{}", MAX_ARTIFACT_BYTES + 1),
        );
        assert!(matches!(
            select_pcsx2_release(oversize.as_bytes(), Pcsx2ArtifactKind::LinuxAppImageX64),
            Err(EmulatorLifecycleError::InvalidAssetSize { .. })
        ));
    }

    #[test]
    fn file_transport_streams_and_verifies_exact_bytes_with_progress() {
        let fixture = tempfile::tempdir().unwrap();
        let destination_root = tempfile::tempdir().unwrap();
        let bytes = vec![0x5a; DOWNLOAD_BUFFER_BYTES * 2 + 17];
        let catalog = catalog_json(&bytes, b"windows");
        fs::write(fixture.path().join("releases.json"), catalog).unwrap();
        let name = "pcsx2-v2.7.492-linux-appimage-x64-Qt.AppImage";
        fs::write(fixture.path().join(name), &bytes).unwrap();
        let transport = FileReleaseTransport::new(fixture.path());
        let offer =
            fetch_latest_pcsx2_release(&transport, Pcsx2ArtifactKind::LinuxAppImageX64).unwrap();
        let destination = destination_root.path().join("download");
        let progress_calls = AtomicUsize::new(0);
        let receipt = download_pcsx2_release(
            &transport,
            &offer,
            &destination,
            &mut |current, total| {
                assert!(current <= total);
                progress_calls.fetch_add(1, Ordering::Relaxed);
            },
            &|| false,
        )
        .unwrap();
        assert_eq!(receipt.byte_len, bytes.len() as u64);
        assert_eq!(receipt.sha256, offer.asset_sha256);
        assert_eq!(fs::read(destination).unwrap(), bytes);
        assert!(progress_calls.load(Ordering::Relaxed) >= 3);
    }

    #[test]
    fn cancelled_or_mismatched_download_leaves_no_partial_file() {
        let fixture = tempfile::tempdir().unwrap();
        let destination_root = tempfile::tempdir().unwrap();
        let bytes = vec![0x3c; DOWNLOAD_BUFFER_BYTES * 2];
        fs::write(
            fixture.path().join("releases.json"),
            catalog_json(&bytes, b"windows"),
        )
        .unwrap();
        let name = "pcsx2-v2.7.492-linux-appimage-x64-Qt.AppImage";
        fs::write(fixture.path().join(name), &bytes).unwrap();
        let transport = FileReleaseTransport::new(fixture.path());
        let offer =
            fetch_latest_pcsx2_release(&transport, Pcsx2ArtifactKind::LinuxAppImageX64).unwrap();
        let destination = destination_root.path().join("cancelled");
        let calls = AtomicUsize::new(0);
        assert!(matches!(
            download_pcsx2_release(&transport, &offer, &destination, &mut |_, _| {}, &|| calls
                .fetch_add(1, Ordering::Relaxed)
                > 0,),
            Err(EmulatorLifecycleError::Cancelled)
        ));
        assert!(!destination.exists());

        let mut wrong = offer;
        wrong.asset_sha256 = "00".repeat(32);
        let destination = destination_root.path().join("mismatch");
        assert!(matches!(
            download_pcsx2_release(&transport, &wrong, &destination, &mut |_, _| {}, &|| false,),
            Err(EmulatorLifecycleError::DigestMismatch { .. })
        ));
        assert!(!destination.exists());
    }

    #[test]
    fn managed_manifest_detects_valid_modified_missing_and_unsafe_executables() {
        let directory = tempfile::tempdir().unwrap();
        let bytes = b"managed executable";
        let executable = directory.path().join("pcsx2-qt.AppImage");
        fs::write(&executable, bytes).unwrap();
        let receipt = file_receipt(&executable).unwrap();
        let offer = select_pcsx2_release(
            &catalog_json(bytes, b"windows"),
            Pcsx2ArtifactKind::LinuxAppImageX64,
        )
        .unwrap();
        let manifest = ManagedEmulatorInstall::from_offer(
            &offer,
            &receipt,
            "pcsx2-fixture".into(),
            vec![ManagedInstalledFile::new(Path::new("pcsx2-qt.AppImage"), &receipt).unwrap()],
        )
        .unwrap();
        fs::write(
            directory.path().join(MANAGED_INSTALL_MANIFEST_NAME),
            manifest.to_json_bytes().unwrap(),
        )
        .unwrap();
        let audit = read_managed_pcsx2_install(directory.path())
            .unwrap()
            .unwrap();
        assert_eq!(audit.executable_state, ManagedExecutableState::Valid);
        assert!(!audit.update_available(&offer));
        assert!(audit.safe_to_update());

        fs::write(&executable, b"locally modified").unwrap();
        let audit = read_managed_pcsx2_install(directory.path())
            .unwrap()
            .unwrap();
        assert_eq!(audit.executable_state, ManagedExecutableState::Modified);
        assert!(!audit.safe_to_update());

        fs::remove_file(&executable).unwrap();
        let audit = read_managed_pcsx2_install(directory.path())
            .unwrap()
            .unwrap();
        assert_eq!(audit.executable_state, ManagedExecutableState::Missing);

        fs::create_dir(&executable).unwrap();
        let audit = read_managed_pcsx2_install(directory.path())
            .unwrap()
            .unwrap();
        assert_eq!(audit.executable_state, ManagedExecutableState::Unsafe);
    }

    #[test]
    fn managed_manifest_tracks_portable_owned_paths_and_reads_legacy_version() {
        let directory = tempfile::tempdir().unwrap();
        let executable = directory.path().join("pcsx2-qt.AppImage");
        fs::write(&executable, b"managed executable").unwrap();
        let receipt = file_receipt(&executable).unwrap();
        let offer = select_pcsx2_release(
            &catalog_json(b"managed executable", b"windows"),
            Pcsx2ArtifactKind::LinuxAppImageX64,
        )
        .unwrap();
        let manifest = ManagedEmulatorInstall::from_offer(
            &offer,
            &receipt,
            "managed-pcsx2".into(),
            vec![
                ManagedInstalledFile::new(Path::new("pcsx2-qt.AppImage"), &receipt).unwrap(),
                ManagedInstalledFile::new(
                    Path::new("portable.ini"),
                    &DownloadReceipt {
                        byte_len: 0,
                        sha256: format!("{:x}", Sha256::digest([])),
                    },
                )
                .unwrap(),
            ],
        )
        .unwrap();
        fs::write(directory.path().join("portable.ini"), []).unwrap();
        fs::write(
            directory.path().join(MANAGED_INSTALL_MANIFEST_NAME),
            manifest.to_json_bytes().unwrap(),
        )
        .unwrap();
        let audit = read_managed_pcsx2_install(directory.path())
            .unwrap()
            .unwrap();
        assert!(audit.ownership_manifest_current());
        assert!(audit.safe_to_remove());
        assert_eq!(audit.installed_files.len(), 2);
        assert!(audit
            .installed_files
            .iter()
            .any(|file| file.relative_path == "portable.ini"));

        let mut legacy = serde_json::to_value(&manifest).unwrap();
        let object = legacy.as_object_mut().unwrap();
        object.insert("schema_version".into(), serde_json::json!(1));
        object.remove("emulator_id");
        object.remove("installed_files");
        fs::write(
            directory.path().join(MANAGED_INSTALL_MANIFEST_NAME),
            serde_json::to_vec_pretty(&legacy).unwrap(),
        )
        .unwrap();
        let legacy_audit = read_managed_pcsx2_install(directory.path())
            .unwrap()
            .unwrap();
        assert!(!legacy_audit.ownership_manifest_current());
        assert!(!legacy_audit.safe_to_remove());
        assert!(legacy_audit.installed_files.is_empty());

        assert!(ManagedInstalledFile::new(Path::new("../outside"), &receipt).is_err());
        assert!(
            ManagedInstalledFile::new(Path::new(MANAGED_INSTALL_MANIFEST_NAME), &receipt).is_err()
        );
        assert!(ManagedInstalledFile::new(Path::new("C:drive-relative"), &receipt).is_err());
        assert!(ManagedInstalledFile::new(Path::new("assets/CON.txt"), &receipt).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn fixture_manifest_and_executable_symlinks_are_refused() {
        use std::os::unix::fs::symlink;

        let directory = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let outside_manifest = outside.path().join("manifest.json");
        fs::write(&outside_manifest, b"{}").unwrap();
        symlink(
            &outside_manifest,
            directory.path().join(MANAGED_INSTALL_MANIFEST_NAME),
        )
        .unwrap();
        assert!(matches!(
            read_managed_pcsx2_install(directory.path()),
            Err(EmulatorLifecycleError::UnsafeManifest { .. })
        ));

        fs::remove_file(directory.path().join(MANAGED_INSTALL_MANIFEST_NAME)).unwrap();
        let bytes = b"managed executable";
        let receipt = DownloadReceipt {
            byte_len: bytes.len() as u64,
            sha256: format!("{:x}", Sha256::digest(bytes)),
        };
        let offer = select_pcsx2_release(
            &catalog_json(bytes, b"windows"),
            Pcsx2ArtifactKind::LinuxAppImageX64,
        )
        .unwrap();
        let manifest = ManagedEmulatorInstall::from_offer(
            &offer,
            &receipt,
            "pcsx2-fixture".into(),
            vec![ManagedInstalledFile::new(Path::new("pcsx2-qt.AppImage"), &receipt).unwrap()],
        )
        .unwrap();
        fs::write(
            directory.path().join(MANAGED_INSTALL_MANIFEST_NAME),
            manifest.to_json_bytes().unwrap(),
        )
        .unwrap();
        let target = outside.path().join("pcsx2");
        fs::write(&target, bytes).unwrap();
        symlink(&target, directory.path().join("pcsx2-qt.AppImage")).unwrap();
        let audit = read_managed_pcsx2_install(directory.path())
            .unwrap()
            .unwrap();
        assert_eq!(audit.executable_state, ManagedExecutableState::Unsafe);
        assert_eq!(fs::read(target).unwrap(), bytes);
    }

    fn catalog_json(linux_bytes: &[u8], windows_bytes: &[u8]) -> Vec<u8> {
        let linux_name = "pcsx2-v2.7.492-linux-appimage-x64-Qt.AppImage";
        let windows_symbols = "pcsx2-v2.7.492-windows-x64-Qt-symbols.7z";
        let windows_name = "pcsx2-v2.7.492-windows-x64-Qt.7z";
        serde_json::to_vec(&serde_json::json!([
            {
                "tag_name": "v2.7.492",
                "name": "v2.7.492",
                "html_url": "https://github.com/PCSX2/pcsx2/releases/tag/v2.7.492",
                "draft": false,
                "prerelease": true,
                "assets": [
                    {
                        "name": linux_name,
                        "browser_download_url": format!(
                            "{GITHUB_ASSET_PREFIX}v2.7.492/{linux_name}"
                        ),
                        "size": linux_bytes.len(),
                        "digest": format!("sha256:{:x}", Sha256::digest(linux_bytes))
                    },
                    {
                        "name": windows_symbols,
                        "browser_download_url": format!(
                            "{GITHUB_ASSET_PREFIX}v2.7.492/{windows_symbols}"
                        ),
                        "size": 7,
                        "digest": format!("sha256:{:x}", Sha256::digest(b"symbols"))
                    },
                    {
                        "name": windows_name,
                        "browser_download_url": format!(
                            "{GITHUB_ASSET_PREFIX}v2.7.492/{windows_name}"
                        ),
                        "size": windows_bytes.len(),
                        "digest": format!("sha256:{:x}", Sha256::digest(windows_bytes))
                    }
                ]
            }
        ]))
        .unwrap()
    }
}
