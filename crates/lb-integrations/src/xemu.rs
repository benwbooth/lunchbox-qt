use crate::emulator_lifecycle::{
    validate_asset_name, validate_sha256, validate_xemu_github_asset_url, DownloadReceipt,
    EmulatorLifecycleError, ReleaseTransport, MAX_ARTIFACT_BYTES, MAX_RELEASE_CATALOG_BYTES,
    XEMU_RELEASES_API,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const XEMU_PROVIDER: &str = "github:xemu-project/xemu";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum XemuArtifactKind {
    WindowsZipX64,
    WindowsZipArm64,
    LinuxAppImageX64,
    LinuxAppImageArm64,
    MacosUniversalZip,
}

impl XemuArtifactKind {
    pub const ALL: [Self; 5] = [
        Self::WindowsZipX64,
        Self::WindowsZipArm64,
        Self::LinuxAppImageX64,
        Self::LinuxAppImageArm64,
        Self::MacosUniversalZip,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::WindowsZipX64 => "windows_zip_x64",
            Self::WindowsZipArm64 => "windows_zip_arm64",
            Self::LinuxAppImageX64 => "linux_appimage_x64",
            Self::LinuxAppImageArm64 => "linux_appimage_arm64",
            Self::MacosUniversalZip => "macos_universal_zip",
        }
    }

    pub const fn executable_name(self) -> &'static str {
        match self {
            Self::WindowsZipX64 | Self::WindowsZipArm64 => "xemu.exe",
            Self::LinuxAppImageX64 | Self::LinuxAppImageArm64 => "xemu.AppImage",
            Self::MacosUniversalZip => "xemu.app/Contents/MacOS/xemu",
        }
    }

    pub const fn requires_extraction(self) -> bool {
        matches!(
            self,
            Self::WindowsZipX64 | Self::WindowsZipArm64 | Self::MacosUniversalZip
        )
    }

    pub fn expected_asset_name(self, version: &str) -> String {
        match self {
            Self::WindowsZipX64 => format!("xemu-{version}-windows-x86_64.zip"),
            Self::WindowsZipArm64 => format!("xemu-{version}-windows-arm64.zip"),
            Self::LinuxAppImageX64 => format!("xemu-{version}-x86_64.AppImage"),
            Self::LinuxAppImageArm64 => format!("xemu-{version}-aarch64.AppImage"),
            Self::MacosUniversalZip => format!("xemu-{version}-macos-universal.zip"),
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "windows_zip_x64" => Some(Self::WindowsZipX64),
            "windows_zip_arm64" => Some(Self::WindowsZipArm64),
            "linux_appimage_x64" => Some(Self::LinuxAppImageX64),
            "linux_appimage_arm64" => Some(Self::LinuxAppImageArm64),
            "macos_universal_zip" => Some(Self::MacosUniversalZip),
            _ => None,
        }
    }

    pub fn current_host() -> Result<Self, EmulatorLifecycleError> {
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            return Ok(Self::LinuxAppImageX64);
        }
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        {
            return Ok(Self::LinuxAppImageArm64);
        }
        #[cfg(all(windows, target_arch = "x86_64"))]
        {
            return Ok(Self::WindowsZipX64);
        }
        #[cfg(all(windows, target_arch = "aarch64"))]
        {
            return Ok(Self::WindowsZipArm64);
        }
        #[cfg(all(
            target_os = "macos",
            any(target_arch = "x86_64", target_arch = "aarch64")
        ))]
        {
            return Ok(Self::MacosUniversalZip);
        }
        #[allow(unreachable_code)]
        Err(EmulatorLifecycleError::UnsupportedHost {
            os: std::env::consts::OS,
            architecture: std::env::consts::ARCH,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct XemuReleaseOffer {
    pub version: String,
    pub tag: String,
    pub release_name: String,
    pub release_url: String,
    pub prerelease: bool,
    pub artifact_kind: XemuArtifactKind,
    pub asset_name: String,
    pub asset_url: String,
    pub asset_byte_len: u64,
    pub asset_sha256: String,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    name: Option<String>,
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

pub fn fetch_latest_xemu_release(
    transport: &dyn ReleaseTransport,
    artifact_kind: XemuArtifactKind,
) -> Result<XemuReleaseOffer, EmulatorLifecycleError> {
    let catalog = transport.fetch_catalog(XEMU_RELEASES_API, MAX_RELEASE_CATALOG_BYTES)?;
    select_xemu_release(&catalog, artifact_kind)
}

pub fn select_xemu_release(
    catalog: &[u8],
    artifact_kind: XemuArtifactKind,
) -> Result<XemuReleaseOffer, EmulatorLifecycleError> {
    let release: GithubRelease = serde_json::from_slice(catalog).map_err(|error| {
        EmulatorLifecycleError::InvalidCatalog {
            message: error.to_string(),
        }
    })?;
    if release.draft {
        return Err(EmulatorLifecycleError::InvalidCatalog {
            message: "the latest Xemu release is still a draft".into(),
        });
    }
    let tag = release.tag_name.trim();
    let version = tag.strip_prefix(['v', 'V']).unwrap_or(tag).trim();
    validate_release_token("tag", tag)?;
    validate_release_token("version", version)?;
    let expected_name = artifact_kind.expected_asset_name(version);
    validate_asset_name(&expected_name)?;
    let matches = release
        .assets
        .iter()
        .filter(|asset| asset.name == expected_name)
        .collect::<Vec<_>>();
    let asset = match matches.as_slice() {
        [asset] => *asset,
        [] => {
            return Err(EmulatorLifecycleError::NoCompatibleRelease {
                artifact_kind: artifact_kind.id(),
            })
        }
        _ => {
            return Err(EmulatorLifecycleError::InvalidCatalog {
                message: format!("Xemu release contains duplicate exact asset {expected_name}"),
            })
        }
    };
    validate_asset_name(&asset.name)?;
    validate_xemu_asset_url(&asset.browser_download_url, tag, &asset.name)?;
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
    let release_name = release
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty() && name.len() <= 256 && !name.chars().any(char::is_control))
        .unwrap_or(tag)
        .to_string();
    Ok(XemuReleaseOffer {
        version: version.to_string(),
        tag: tag.to_string(),
        release_name,
        release_url: format!("https://github.com/xemu-project/xemu/releases/tag/{tag}"),
        prerelease: release.prerelease,
        artifact_kind,
        asset_name: asset.name.clone(),
        asset_url: asset.browser_download_url.clone(),
        asset_byte_len: asset.size,
        asset_sha256: digest,
    })
}

pub fn download_xemu_release(
    transport: &dyn ReleaseTransport,
    offer: &XemuReleaseOffer,
    destination: &Path,
    progress: &mut dyn FnMut(u64, u64),
    should_cancel: &dyn Fn() -> bool,
) -> Result<DownloadReceipt, EmulatorLifecycleError> {
    validate_release_token("tag", &offer.tag)?;
    validate_release_token("version", &offer.version)?;
    let expected_name = offer.artifact_kind.expected_asset_name(&offer.version);
    if offer.asset_name != expected_name {
        return Err(EmulatorLifecycleError::InvalidCatalog {
            message: format!(
                "Xemu asset {} does not match expected host asset {expected_name}",
                offer.asset_name
            ),
        });
    }
    validate_asset_name(&offer.asset_name)?;
    validate_xemu_asset_url(&offer.asset_url, &offer.tag, &offer.asset_name)?;
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

fn validate_release_token(field: &str, value: &str) -> Result<(), EmulatorLifecycleError> {
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {
        return Err(EmulatorLifecycleError::InvalidCatalog {
            message: format!("Xemu release {field} is unsafe: {value:?}"),
        });
    }
    Ok(())
}

fn validate_xemu_asset_url(
    url: &str,
    tag: &str,
    asset_name: &str,
) -> Result<(), EmulatorLifecycleError> {
    validate_xemu_github_asset_url(url)?;
    let expected =
        format!("https://github.com/xemu-project/xemu/releases/download/{tag}/{asset_name}");
    if url != expected {
        return Err(EmulatorLifecycleError::UntrustedUrl {
            url: url.to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emulator_lifecycle::{file_receipt, FileReleaseTransport};
    use serde_json::json;
    use std::fs;

    const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    fn asset(kind: XemuArtifactKind, version: &str) -> serde_json::Value {
        let name = kind.expected_asset_name(version);
        json!({
            "name": name,
            "browser_download_url": format!(
                "https://github.com/xemu-project/xemu/releases/download/v{version}/{name}"
            ),
            "size": 1234,
            "digest": format!("sha256:{DIGEST}")
        })
    }

    fn catalog(assets: Vec<serde_json::Value>) -> Vec<u8> {
        serde_json::to_vec(&json!({
            "tag_name": "v0.8.136",
            "name": "v0.8.136",
            "html_url": "https://github.com/xemu-project/xemu/releases/tag/v0.8.136",
            "draft": false,
            "prerelease": false,
            "assets": assets
        }))
        .unwrap()
    }

    #[test]
    fn official_latest_release_selects_each_exact_host_artifact() {
        let assets = XemuArtifactKind::ALL
            .into_iter()
            .map(|kind| asset(kind, "0.8.136"))
            .chain([
                json!({
                    "name": "xemu-0.8.136-dbg-x86_64.AppImage",
                    "browser_download_url": "https://github.com/xemu-project/xemu/releases/download/v0.8.136/xemu-0.8.136-dbg-x86_64.AppImage",
                    "size": 100,
                    "digest": format!("sha256:{DIGEST}")
                }),
                json!({
                    "name": "xemu-0.8.136-macos-universal-unsigned.zip",
                    "browser_download_url": "https://github.com/xemu-project/xemu/releases/download/v0.8.136/xemu-0.8.136-macos-universal-unsigned.zip",
                    "size": 100,
                    "digest": format!("sha256:{DIGEST}")
                }),
                json!({
                    "name": "xemu-win-x86_64-release.zip",
                    "browser_download_url": "https://github.com/xemu-project/xemu/releases/download/v0.8.136/xemu-win-x86_64-release.zip",
                    "size": 100,
                    "digest": format!("sha256:{DIGEST}")
                }),
            ])
            .collect::<Vec<_>>();
        let catalog = catalog(assets);
        for kind in XemuArtifactKind::ALL {
            let offer = select_xemu_release(&catalog, kind).unwrap();
            assert_eq!(offer.version, "0.8.136");
            assert_eq!(offer.tag, "v0.8.136");
            assert_eq!(offer.artifact_kind, kind);
            assert_eq!(offer.asset_name, kind.expected_asset_name("0.8.136"));
            assert_eq!(offer.asset_sha256, DIGEST);
            assert!(!offer.prerelease);
        }
    }

    #[test]
    fn release_selection_rejects_debug_aliases_duplicates_and_untrusted_metadata() {
        let debug_only = catalog(vec![json!({
            "name": "xemu-0.8.136-dbg-x86_64.AppImage",
            "browser_download_url": "https://github.com/xemu-project/xemu/releases/download/v0.8.136/xemu-0.8.136-dbg-x86_64.AppImage",
            "size": 100,
            "digest": format!("sha256:{DIGEST}")
        })]);
        assert!(matches!(
            select_xemu_release(&debug_only, XemuArtifactKind::LinuxAppImageX64),
            Err(EmulatorLifecycleError::NoCompatibleRelease { .. })
        ));

        let exact = asset(XemuArtifactKind::WindowsZipX64, "0.8.136");
        assert!(matches!(
            select_xemu_release(
                &catalog(vec![exact.clone(), exact]),
                XemuArtifactKind::WindowsZipX64
            ),
            Err(EmulatorLifecycleError::InvalidCatalog { .. })
        ));

        let mut missing_digest = asset(XemuArtifactKind::LinuxAppImageX64, "0.8.136");
        missing_digest["digest"] = serde_json::Value::Null;
        assert!(matches!(
            select_xemu_release(
                &catalog(vec![missing_digest]),
                XemuArtifactKind::LinuxAppImageX64
            ),
            Err(EmulatorLifecycleError::MissingDigest { .. })
        ));

        let mut untrusted = asset(XemuArtifactKind::MacosUniversalZip, "0.8.136");
        untrusted["browser_download_url"] =
            json!("https://example.com/xemu-0.8.136-macos-universal.zip");
        assert!(matches!(
            select_xemu_release(
                &catalog(vec![untrusted]),
                XemuArtifactKind::MacosUniversalZip
            ),
            Err(EmulatorLifecycleError::UntrustedUrl { .. })
        ));

        let mut wrong_tag = asset(XemuArtifactKind::LinuxAppImageX64, "0.8.136");
        wrong_tag["browser_download_url"] = json!(
            "https://github.com/xemu-project/xemu/releases/download/v0.8.135/xemu-0.8.136-x86_64.AppImage"
        );
        assert!(matches!(
            select_xemu_release(
                &catalog(vec![wrong_tag]),
                XemuArtifactKind::LinuxAppImageX64
            ),
            Err(EmulatorLifecycleError::UntrustedUrl { .. })
        ));
    }

    #[test]
    fn file_transport_download_verifies_xemu_digest_and_cleans_mismatch() {
        let fixture = tempfile::tempdir().unwrap();
        let kind = XemuArtifactKind::LinuxAppImageX64;
        let name = kind.expected_asset_name("0.8.136");
        let source = fixture.path().join(&name);
        fs::write(&source, b"official xemu appimage fixture").unwrap();
        let receipt = file_receipt(&source).unwrap();
        let offer = XemuReleaseOffer {
            version: "0.8.136".into(),
            tag: "v0.8.136".into(),
            release_name: "v0.8.136".into(),
            release_url: "https://github.com/xemu-project/xemu/releases/tag/v0.8.136".into(),
            prerelease: false,
            artifact_kind: kind,
            asset_name: name.clone(),
            asset_url: format!(
                "https://github.com/xemu-project/xemu/releases/download/v0.8.136/{name}"
            ),
            asset_byte_len: receipt.byte_len,
            asset_sha256: receipt.sha256.clone(),
        };
        let transport = FileReleaseTransport::new(fixture.path());
        let destination = fixture.path().join("download.AppImage");
        let mut progress = Vec::new();
        let downloaded = download_xemu_release(
            &transport,
            &offer,
            &destination,
            &mut |current, total| progress.push((current, total)),
            &|| false,
        )
        .unwrap();
        assert_eq!(downloaded, receipt);
        assert_eq!(fs::read(&destination).unwrap(), fs::read(&source).unwrap());
        assert_eq!(progress.last(), Some(&(receipt.byte_len, receipt.byte_len)));

        fs::remove_file(&destination).unwrap();
        let mut wrong = offer;
        wrong.asset_sha256 = "f".repeat(64);
        assert!(matches!(
            download_xemu_release(&transport, &wrong, &destination, &mut |_, _| {}, &|| false),
            Err(EmulatorLifecycleError::DigestMismatch { .. })
        ));
        assert!(!destination.exists());
    }
}
