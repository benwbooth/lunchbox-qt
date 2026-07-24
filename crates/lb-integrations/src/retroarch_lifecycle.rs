use crate::emulator_lifecycle::{
    stream_download, DownloadReceipt, EmulatorLifecycleError, MAX_ARTIFACT_BYTES,
    MAX_RELEASE_CATALOG_BYTES,
};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

pub const RETROARCH_PROVIDER: &str = "buildbot.libretro.com/stable";
pub const RETROARCH_STABLE_INDEX: &str = "https://buildbot.libretro.com/stable/";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetroArchArtifactKind {
    Linux7zX64,
    Windows7zX64,
    Windows7zX86,
    MacosMetalDmgUniversal,
}

impl RetroArchArtifactKind {
    pub const ALL: [Self; 4] = [
        Self::Linux7zX64,
        Self::Windows7zX64,
        Self::Windows7zX86,
        Self::MacosMetalDmgUniversal,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Linux7zX64 => "linux_7z_x64",
            Self::Windows7zX64 => "windows_7z_x64",
            Self::Windows7zX86 => "windows_7z_x86",
            Self::MacosMetalDmgUniversal => "macos_metal_dmg_universal",
        }
    }

    pub const fn buildbot_path(self) -> &'static str {
        match self {
            Self::Linux7zX64 => "linux/x86_64",
            Self::Windows7zX64 => "windows/x86_64",
            Self::Windows7zX86 => "windows/x86",
            Self::MacosMetalDmgUniversal => "apple/osx/universal",
        }
    }

    pub const fn frontend_asset_name(self) -> &'static str {
        match self {
            Self::MacosMetalDmgUniversal => "RetroArch_Metal.dmg",
            _ => "RetroArch.7z",
        }
    }

    pub const fn cores_asset_name(self) -> Option<&'static str> {
        match self {
            Self::MacosMetalDmgUniversal => None,
            _ => Some("RetroArch_cores.7z"),
        }
    }

    pub const fn archive_root(self) -> &'static str {
        match self {
            Self::Linux7zX64 => "RetroArch-Linux-x86_64",
            Self::Windows7zX64 => "RetroArch-Win64",
            Self::Windows7zX86 => "RetroArch-Win32",
            Self::MacosMetalDmgUniversal => "RetroArch/RetroArch.app",
        }
    }

    pub const fn executable_name(self) -> &'static str {
        match self {
            Self::Linux7zX64 => "RetroArch-Linux-x86_64.AppImage",
            Self::Windows7zX64 | Self::Windows7zX86 => "retroarch.exe",
            Self::MacosMetalDmgUniversal => "RetroArch.app/Contents/MacOS/RetroArch",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.id() == id)
    }

    pub fn current_host() -> Result<Self, EmulatorLifecycleError> {
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            return Ok(Self::Linux7zX64);
        }
        #[cfg(all(windows, target_arch = "x86_64"))]
        {
            return Ok(Self::Windows7zX64);
        }
        #[cfg(all(windows, target_arch = "x86"))]
        {
            return Ok(Self::Windows7zX86);
        }
        #[cfg(all(
            target_os = "macos",
            any(target_arch = "x86_64", target_arch = "aarch64")
        ))]
        {
            return Ok(Self::MacosMetalDmgUniversal);
        }
        #[allow(unreachable_code)]
        Err(EmulatorLifecycleError::UnsupportedHost {
            os: std::env::consts::OS,
            architecture: std::env::consts::ARCH,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RetroArchReleaseArtifact {
    pub role: &'static str,
    pub asset_name: String,
    pub asset_url: String,
    pub asset_byte_len: u64,
    pub published_sha256: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RetroArchReleaseOffer {
    pub version: String,
    pub tag: String,
    pub release_name: String,
    pub release_url: String,
    pub prerelease: bool,
    pub artifact_kind: RetroArchArtifactKind,
    pub asset_name: String,
    pub asset_url: String,
    pub asset_byte_len: u64,
    pub asset_sha256: Option<String>,
    pub cores: Option<RetroArchReleaseArtifact>,
}

pub trait RetroArchReleaseTransport: Send + Sync {
    fn fetch_stable_index(
        &self,
        url: &str,
        max_bytes: u64,
    ) -> Result<Vec<u8>, EmulatorLifecycleError>;

    fn artifact_byte_len(&self, url: &str) -> Result<u64, EmulatorLifecycleError>;

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
pub struct BuildbotRetroArchReleaseTransport;

impl RetroArchReleaseTransport for BuildbotRetroArchReleaseTransport {
    fn fetch_stable_index(
        &self,
        url: &str,
        max_bytes: u64,
    ) -> Result<Vec<u8>, EmulatorLifecycleError> {
        if url != RETROARCH_STABLE_INDEX {
            return Err(EmulatorLifecycleError::UntrustedUrl {
                url: url.to_string(),
            });
        }
        let mut response = ureq::get(url)
            .header("Accept", "text/html")
            .header("User-Agent", "lunchbox-qt")
            .call()
            .map_err(|error| EmulatorLifecycleError::Transport {
                message: error.to_string(),
            })?;
        read_limited(
            response.body_mut().as_reader(),
            max_bytes,
            "RetroArch stable buildbot index",
        )
    }

    fn artifact_byte_len(&self, url: &str) -> Result<u64, EmulatorLifecycleError> {
        validate_buildbot_artifact_url(url)?;
        let response = ureq::head(url)
            .header("Accept", "application/octet-stream")
            .header("User-Agent", "lunchbox-qt")
            .call()
            .map_err(|error| EmulatorLifecycleError::Transport {
                message: error.to_string(),
            })?;
        let value = response
            .headers()
            .get("content-length")
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| EmulatorLifecycleError::InvalidCatalog {
                message: format!("RetroArch buildbot response has no Content-Length for {url}"),
            })?;
        let byte_len =
            value
                .parse::<u64>()
                .map_err(|error| EmulatorLifecycleError::InvalidCatalog {
                    message: format!(
                        "RetroArch buildbot Content-Length is invalid for {url}: {error}"
                    ),
                })?;
        validate_artifact_size(url, byte_len)?;
        Ok(byte_len)
    }

    fn download(
        &self,
        url: &str,
        destination: &Path,
        expected_byte_len: u64,
        progress: &mut dyn FnMut(u64, u64),
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<DownloadReceipt, EmulatorLifecycleError> {
        validate_buildbot_artifact_url(url)?;
        validate_artifact_size(url, expected_byte_len)?;
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
pub struct FileRetroArchReleaseTransport {
    root: PathBuf,
}

impl FileRetroArchReleaseTransport {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl RetroArchReleaseTransport for FileRetroArchReleaseTransport {
    fn fetch_stable_index(
        &self,
        _url: &str,
        max_bytes: u64,
    ) -> Result<Vec<u8>, EmulatorLifecycleError> {
        let path = self.root.join("stable.html");
        let source = checked_fixture_file(&path)?;
        read_limited(source, max_bytes, "RetroArch fixture stable index")
    }

    fn artifact_byte_len(&self, url: &str) -> Result<u64, EmulatorLifecycleError> {
        validate_buildbot_artifact_url(url)?;
        let path = self.root.join(asset_name_from_url(url)?);
        let metadata =
            fs::symlink_metadata(&path).map_err(|source| EmulatorLifecycleError::Io {
                path: path.clone(),
                source,
            })?;
        if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
            return Err(EmulatorLifecycleError::UnsafeFixture { path });
        }
        validate_artifact_size(url, metadata.len())?;
        Ok(metadata.len())
    }

    fn download(
        &self,
        url: &str,
        destination: &Path,
        expected_byte_len: u64,
        progress: &mut dyn FnMut(u64, u64),
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<DownloadReceipt, EmulatorLifecycleError> {
        validate_buildbot_artifact_url(url)?;
        let path = self.root.join(asset_name_from_url(url)?);
        let source = checked_fixture_file(&path)?;
        stream_download(
            source,
            destination,
            expected_byte_len,
            progress,
            should_cancel,
        )
    }
}

pub fn fetch_latest_retroarch_release(
    transport: &dyn RetroArchReleaseTransport,
    artifact_kind: RetroArchArtifactKind,
) -> Result<RetroArchReleaseOffer, EmulatorLifecycleError> {
    let index = transport.fetch_stable_index(RETROARCH_STABLE_INDEX, MAX_RELEASE_CATALOG_BYTES)?;
    let version = select_latest_retroarch_version(&index)?;
    let asset_name = artifact_kind.frontend_asset_name().to_string();
    let asset_url = buildbot_artifact_url(&version, artifact_kind, &asset_name)?;
    let asset_byte_len = transport.artifact_byte_len(&asset_url)?;
    let cores = artifact_kind
        .cores_asset_name()
        .map(|name| {
            let url = buildbot_artifact_url(&version, artifact_kind, name)?;
            let byte_len = transport.artifact_byte_len(&url)?;
            Ok(RetroArchReleaseArtifact {
                role: "cores",
                asset_name: name.to_string(),
                asset_url: url,
                asset_byte_len: byte_len,
                published_sha256: None,
            })
        })
        .transpose()?;
    Ok(RetroArchReleaseOffer {
        tag: version.clone(),
        release_name: format!("RetroArch {version}"),
        release_url: format!("{RETROARCH_STABLE_INDEX}{version}/"),
        version,
        prerelease: false,
        artifact_kind,
        asset_name,
        asset_url,
        asset_byte_len,
        asset_sha256: None,
        cores,
    })
}

pub fn select_latest_retroarch_version(catalog: &[u8]) -> Result<String, EmulatorLifecycleError> {
    let catalog =
        std::str::from_utf8(catalog).map_err(|error| EmulatorLifecycleError::InvalidCatalog {
            message: format!("RetroArch stable index is not UTF-8: {error}"),
        })?;
    let mut versions = Vec::new();
    for suffix in catalog.split("href=").skip(1) {
        let suffix = suffix.trim_start();
        let Some(quote) = suffix
            .chars()
            .next()
            .filter(|quote| matches!(quote, '"' | '\''))
        else {
            continue;
        };
        let remainder = &suffix[quote.len_utf8()..];
        let Some(end) = remainder.find(quote) else {
            continue;
        };
        let candidate = remainder[..end].trim_end_matches('/');
        if let Some(components) = parse_stable_version(candidate) {
            versions.push((components, candidate.to_string()));
        }
    }
    versions.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    versions.dedup_by(|left, right| left.1 == right.1);
    versions.pop().map(|(_, version)| version).ok_or_else(|| {
        EmulatorLifecycleError::InvalidCatalog {
            message: "RetroArch stable index contains no semantic release directory".into(),
        }
    })
}

pub fn download_retroarch_artifact(
    transport: &dyn RetroArchReleaseTransport,
    artifact: &RetroArchReleaseArtifact,
    destination: &Path,
    progress: &mut dyn FnMut(u64, u64),
    should_cancel: &dyn Fn() -> bool,
) -> Result<DownloadReceipt, EmulatorLifecycleError> {
    validate_buildbot_artifact_url(&artifact.asset_url)?;
    if asset_name_from_url(&artifact.asset_url)? != artifact.asset_name {
        return Err(EmulatorLifecycleError::InvalidCatalog {
            message: format!(
                "RetroArch artifact URL does not end in {}",
                artifact.asset_name
            ),
        });
    }
    validate_artifact_size(&artifact.asset_name, artifact.asset_byte_len)?;
    let receipt = transport.download(
        &artifact.asset_url,
        destination,
        artifact.asset_byte_len,
        progress,
        should_cancel,
    )?;
    if receipt.byte_len != artifact.asset_byte_len {
        let _ = fs::remove_file(destination);
        return Err(EmulatorLifecycleError::SizeMismatch {
            expected: artifact.asset_byte_len,
            actual: receipt.byte_len,
        });
    }
    Ok(receipt)
}

pub fn frontend_artifact(offer: &RetroArchReleaseOffer) -> RetroArchReleaseArtifact {
    RetroArchReleaseArtifact {
        role: "frontend",
        asset_name: offer.asset_name.clone(),
        asset_url: offer.asset_url.clone(),
        asset_byte_len: offer.asset_byte_len,
        published_sha256: offer.asset_sha256.clone(),
    }
}

fn parse_stable_version(value: &str) -> Option<(u32, u32, u32)> {
    if value.is_empty()
        || value
            .bytes()
            .any(|byte| !byte.is_ascii_digit() && byte != b'.')
    {
        return None;
    }
    let components = value
        .split('.')
        .map(|component| {
            if component.is_empty()
                || (component.len() > 1 && component.starts_with('0'))
                || component.len() > 10
            {
                return None;
            }
            component.parse::<u32>().ok()
        })
        .collect::<Option<Vec<_>>>()?;
    let [major, minor, patch] = components.as_slice() else {
        return None;
    };
    Some((*major, *minor, *patch))
}

fn buildbot_artifact_url(
    version: &str,
    artifact_kind: RetroArchArtifactKind,
    asset_name: &str,
) -> Result<String, EmulatorLifecycleError> {
    parse_stable_version(version).ok_or_else(|| EmulatorLifecycleError::InvalidCatalog {
        message: format!("RetroArch stable version is invalid: {version}"),
    })?;
    if !matches!(
        asset_name,
        "RetroArch.7z" | "RetroArch_cores.7z" | "RetroArch_Metal.dmg"
    ) {
        return Err(EmulatorLifecycleError::UnsafeAssetName {
            name: asset_name.to_string(),
        });
    }
    let expected_name = if artifact_kind == RetroArchArtifactKind::MacosMetalDmgUniversal {
        "RetroArch_Metal.dmg"
    } else if asset_name == "RetroArch_Metal.dmg" {
        return Err(EmulatorLifecycleError::InvalidCatalog {
            message: "RetroArch Metal DMG is only valid for the universal macOS target".into(),
        });
    } else {
        asset_name
    };
    Ok(format!(
        "{RETROARCH_STABLE_INDEX}{version}/{}/{expected_name}",
        artifact_kind.buildbot_path()
    ))
}

fn validate_buildbot_artifact_url(url: &str) -> Result<(), EmulatorLifecycleError> {
    if url.contains(['\r', '\n', '?', '#']) || !url.starts_with(RETROARCH_STABLE_INDEX) {
        return Err(EmulatorLifecycleError::UntrustedUrl {
            url: url.to_string(),
        });
    }
    let suffix = &url[RETROARCH_STABLE_INDEX.len()..];
    let mut parts = suffix.split('/');
    let Some(version) = parts.next() else {
        return Err(EmulatorLifecycleError::UntrustedUrl {
            url: url.to_string(),
        });
    };
    if parse_stable_version(version).is_none() {
        return Err(EmulatorLifecycleError::UntrustedUrl {
            url: url.to_string(),
        });
    }
    let remaining = parts.collect::<Vec<_>>();
    let valid = RetroArchArtifactKind::ALL.into_iter().any(|kind| {
        let mut expected = kind
            .buildbot_path()
            .split('/')
            .map(str::to_string)
            .collect::<Vec<_>>();
        expected.push(kind.frontend_asset_name().to_string());
        remaining == expected
            || kind.cores_asset_name().is_some_and(|cores| {
                let mut expected = kind
                    .buildbot_path()
                    .split('/')
                    .map(str::to_string)
                    .collect::<Vec<_>>();
                expected.push(cores.to_string());
                remaining == expected
            })
    });
    if !valid {
        return Err(EmulatorLifecycleError::UntrustedUrl {
            url: url.to_string(),
        });
    }
    Ok(())
}

fn asset_name_from_url(url: &str) -> Result<&str, EmulatorLifecycleError> {
    url.rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .ok_or_else(|| EmulatorLifecycleError::UntrustedUrl {
            url: url.to_string(),
        })
}

fn validate_artifact_size(label: &str, byte_len: u64) -> Result<(), EmulatorLifecycleError> {
    if byte_len == 0 || byte_len > MAX_ARTIFACT_BYTES {
        return Err(EmulatorLifecycleError::InvalidAssetSize {
            asset: label.to_string(),
            byte_len,
        });
    }
    Ok(())
}

fn checked_fixture_file(path: &Path) -> Result<File, EmulatorLifecycleError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| EmulatorLifecycleError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(EmulatorLifecycleError::UnsafeFixture {
            path: path.to_path_buf(),
        });
    }
    File::open(path).map_err(|source| EmulatorLifecycleError::Io {
        path: path.to_path_buf(),
        source,
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn stable_index_selects_semantic_latest_and_ignores_non_release_links() {
        let html = br#"
            <a href="../">parent</a>
            <a href="1.9.14/">old</a>
            <a href="1.22.2/">latest</a>
            <a href="1.22.10/">newer patch</a>
            <a href="nightly/">moving</a>
            <a href="01.23.0/">non-canonical</a>
        "#;
        assert_eq!(select_latest_retroarch_version(html).unwrap(), "1.22.10");
    }

    #[test]
    fn exact_cross_platform_buildbot_contract_includes_universal_macos() {
        let version = "1.22.2";
        let expected = [
            (
                RetroArchArtifactKind::Linux7zX64,
                "linux/x86_64/RetroArch.7z",
                Some("RetroArch_cores.7z"),
                "RetroArch-Linux-x86_64.AppImage",
            ),
            (
                RetroArchArtifactKind::Windows7zX64,
                "windows/x86_64/RetroArch.7z",
                Some("RetroArch_cores.7z"),
                "retroarch.exe",
            ),
            (
                RetroArchArtifactKind::Windows7zX86,
                "windows/x86/RetroArch.7z",
                Some("RetroArch_cores.7z"),
                "retroarch.exe",
            ),
            (
                RetroArchArtifactKind::MacosMetalDmgUniversal,
                "apple/osx/universal/RetroArch_Metal.dmg",
                None,
                "RetroArch.app/Contents/MacOS/RetroArch",
            ),
        ];
        for (kind, suffix, cores, executable) in expected {
            let url = buildbot_artifact_url(version, kind, kind.frontend_asset_name()).unwrap();
            assert_eq!(
                url,
                format!("https://buildbot.libretro.com/stable/{version}/{suffix}")
            );
            validate_buildbot_artifact_url(&url).unwrap();
            assert_eq!(kind.cores_asset_name(), cores);
            assert_eq!(kind.executable_name(), executable);
        }
    }

    #[test]
    fn fixture_provider_fetches_both_artifacts_and_records_local_digests() {
        let fixture = tempfile::tempdir().unwrap();
        let destination = tempfile::tempdir().unwrap();
        fs::write(
            fixture.path().join("stable.html"),
            br#"<a href="1.22.2/">1.22.2</a>"#,
        )
        .unwrap();
        fs::write(fixture.path().join("RetroArch.7z"), b"frontend").unwrap();
        fs::write(fixture.path().join("RetroArch_cores.7z"), b"cores").unwrap();
        let transport = FileRetroArchReleaseTransport::new(fixture.path());
        let offer =
            fetch_latest_retroarch_release(&transport, RetroArchArtifactKind::Linux7zX64).unwrap();
        assert_eq!(offer.version, "1.22.2");
        assert_eq!(offer.asset_byte_len, 8);
        assert!(offer.asset_sha256.is_none());
        assert_eq!(offer.cores.as_ref().unwrap().asset_byte_len, 5);

        let calls = AtomicUsize::new(0);
        let receipt = download_retroarch_artifact(
            &transport,
            &frontend_artifact(&offer),
            &destination.path().join("frontend.7z"),
            &mut |current, total| {
                assert!(current <= total);
                calls.fetch_add(1, Ordering::Relaxed);
            },
            &|| false,
        )
        .unwrap();
        assert_eq!(receipt.byte_len, 8);
        assert_eq!(receipt.sha256.len(), 64);
        assert!(calls.load(Ordering::Relaxed) >= 2);
    }

    #[test]
    fn macos_offer_has_one_dmg_and_no_fictitious_stable_cores_archive() {
        let fixture = tempfile::tempdir().unwrap();
        fs::write(
            fixture.path().join("stable.html"),
            br#"<a href="1.22.2/">1.22.2</a>"#,
        )
        .unwrap();
        fs::write(fixture.path().join("RetroArch_Metal.dmg"), b"universal dmg").unwrap();
        let offer = fetch_latest_retroarch_release(
            &FileRetroArchReleaseTransport::new(fixture.path()),
            RetroArchArtifactKind::MacosMetalDmgUniversal,
        )
        .unwrap();
        assert_eq!(offer.asset_name, "RetroArch_Metal.dmg");
        assert!(offer.cores.is_none());
        assert_eq!(
            offer.artifact_kind.archive_root(),
            "RetroArch/RetroArch.app"
        );
    }

    #[test]
    fn moving_aliases_queries_and_unexpected_assets_are_rejected() {
        for url in [
            "https://buildbot.libretro.com/nightly/linux/x86_64/RetroArch.7z",
            "https://buildbot.libretro.com/stable/1.22.2/linux/x86_64/RetroArch.7z?x=1",
            "https://buildbot.libretro.com/stable/1.22.2/apple/osx/universal/RetroArch_cores.7z",
            "https://example.com/stable/1.22.2/linux/x86_64/RetroArch.7z",
        ] {
            assert!(matches!(
                validate_buildbot_artifact_url(url),
                Err(EmulatorLifecycleError::UntrustedUrl { .. })
            ));
        }
    }

    #[test]
    fn cancelled_download_removes_partial_artifact() {
        let fixture = tempfile::tempdir().unwrap();
        let destination = tempfile::tempdir().unwrap();
        fs::write(
            fixture.path().join("stable.html"),
            br#"<a href="1.22.2/">1.22.2</a>"#,
        )
        .unwrap();
        fs::write(fixture.path().join("RetroArch.7z"), vec![0x5a; 300_000]).unwrap();
        fs::write(fixture.path().join("RetroArch_cores.7z"), b"cores").unwrap();
        let transport = FileRetroArchReleaseTransport::new(fixture.path());
        let offer =
            fetch_latest_retroarch_release(&transport, RetroArchArtifactKind::Linux7zX64).unwrap();
        let target = destination.path().join("cancelled.7z");
        let calls = AtomicUsize::new(0);
        assert!(matches!(
            download_retroarch_artifact(
                &transport,
                &frontend_artifact(&offer),
                &target,
                &mut |_, _| {},
                &|| calls.fetch_add(1, Ordering::Relaxed) > 0,
            ),
            Err(EmulatorLifecycleError::Cancelled)
        ));
        assert!(!target.exists());
    }
}
