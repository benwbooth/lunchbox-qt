use lb_domain::PlatformFolder;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Resolves LaunchBox's persisted path syntax into a path meaningful on the
/// current host. Storage keeps the original string; only platform services
/// turn it into an operating-system path.
pub trait LaunchPathResolver {
    fn resolve(&self, launchbox_root: &Path, stored_path: &str)
        -> Result<PathBuf, LaunchPathError>;
}

/// Classifies only Windows-rooted persisted syntax without interpreting it as
/// a host path. Callers that merely census LaunchBox data should use this
/// helper instead of duplicating drive/UNC parsing outside the platform layer.
pub fn is_windows_absolute_path(path: &str) -> bool {
    windows_drive_path(path).is_some() || path.starts_with("\\\\") || path.starts_with("//")
}

/// Converts a LaunchBox platform display name into the portable filename stem
/// used for its `Data/Platforms/*.xml` document. This is deliberately separate
/// from persisted media paths: the result is a host filename component, while
/// values such as `Images\\Nintendo 64\\Box - Front` remain lexical LaunchBox
/// strings until a [`LaunchPathResolver`] consumes them.
pub fn navigation_document_file_name(display_name: &str) -> Result<String, PlatformPathError> {
    Ok(format!("{}.xml", portable_storage_name(display_name)?))
}

pub fn platform_document_file_name(platform_name: &str) -> Result<String, PlatformPathError> {
    navigation_document_file_name(platform_name)
}

/// Produces one portable platform directory component using the filename
/// restrictions shared by Windows, Linux, and macOS.
pub fn portable_storage_name(display_name: &str) -> Result<String, PlatformPathError> {
    let display_name = display_name.trim();
    if display_name.is_empty() {
        return Err(PlatformPathError::EmptyPlatformName);
    }

    let mut stem = display_name
        .chars()
        .map(|character| {
            if character.is_control()
                || matches!(
                    character,
                    '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
                )
            {
                '_'
            } else {
                character
            }
        })
        .collect::<String>();
    while stem.ends_with([' ', '.']) {
        stem.pop();
        stem.push('_');
    }

    let reserved = stem
        .split('.')
        .next()
        .is_some_and(is_windows_reserved_file_stem);
    if reserved {
        stem.push('_');
    }
    Ok(stem)
}

pub fn platform_storage_name(platform_name: &str) -> Result<String, PlatformPathError> {
    portable_storage_name(platform_name)
}

/// Reproduces the 51 default platform-folder records observed across every
/// platform in the supplied LaunchBox 13.24 installation. These values are
/// persisted LaunchBox syntax, not native host paths; they therefore retain
/// backslashes on every operating system.
pub fn default_platform_folders(
    platform_name: &str,
) -> Result<Vec<PlatformFolder>, PlatformPathError> {
    const IMAGE_MEDIA_TYPES: &[&str] = &[
        "Advertisement Flyer - Back",
        "Advertisement Flyer - Front",
        "Amazon Background",
        "Amazon Poster",
        "Amazon Screenshot",
        "Arcade - Cabinet",
        "Arcade - Circuit Board",
        "Arcade - Control Panel",
        "Arcade - Controls Information",
        "Arcade - Marquee",
        "Banner",
        "Box - 3D",
        "Box - Back",
        "Box - Back - Reconstructed",
        "Box - Front",
        "Box - Front - Reconstructed",
        "Box - Full",
        "Box - Spine",
        "Cart - 3D",
        "Cart - Back",
        "Cart - Front",
        "Clear Logo",
        "Disc",
        "Epic Games Background",
        "Epic Games Poster",
        "Epic Games Screenshot",
        "Fanart - Background",
        "Fanart - Box - Back",
        "Fanart - Box - Front",
        "Fanart - Cart - Back",
        "Fanart - Cart - Front",
        "Fanart - Disc",
        "GOG Poster",
        "GOG Screenshot",
        "Origin Background",
        "Origin Poster",
        "Origin Screenshot",
        "Screenshot - Game Over",
        "Screenshot - Game Select",
        "Screenshot - Game Title",
        "Screenshot - Gameplay",
        "Screenshot - High Scores",
        "Steam Banner",
        "Steam Poster",
        "Steam Screenshot",
        "Uplay Background",
        "Uplay Thumbnail",
    ];

    let storage_name = platform_storage_name(platform_name)?;
    let mut folders = IMAGE_MEDIA_TYPES
        .iter()
        .map(|media_type| PlatformFolder {
            platform: platform_name.trim().to_string(),
            media_type: (*media_type).to_string(),
            folder_path: format!(r"Images\{storage_name}\{media_type}"),
        })
        .collect::<Vec<_>>();
    for (media_type, folder_path) in [
        ("Manual", format!(r"Manuals\{storage_name}")),
        ("Music", format!(r"Music\{storage_name}")),
        ("Theme Video", format!(r"Videos\{storage_name}\Theme")),
        ("Video", format!(r"Videos\{storage_name}")),
    ] {
        folders.push(PlatformFolder {
            platform: platform_name.trim().to_string(),
            media_type: media_type.to_string(),
            folder_path,
        });
    }
    Ok(folders)
}

/// Native host resolver with explicit mappings for foreign Windows paths.
///
/// Portable LaunchBox-relative paths accept either slash style. On Windows,
/// drive and UNC paths remain native. On other hosts they must be mapped; this
/// prevents a string such as `D:\Games\game.rom` from becoming a misleading
/// relative Linux filename.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HostPathResolver {
    windows_drives: BTreeMap<char, PathBuf>,
    windows_unc_roots: BTreeMap<(String, String), PathBuf>,
}

impl HostPathResolver {
    pub fn with_windows_drive_mapping(
        mut self,
        drive: char,
        host_root: impl Into<PathBuf>,
    ) -> Result<Self, LaunchPathError> {
        let drive = normalize_drive(drive)?;
        let host_root = host_root.into();
        validate_host_root(&host_root)?;
        self.windows_drives.insert(drive, host_root);
        Ok(self)
    }

    pub fn with_windows_unc_mapping(
        mut self,
        server: impl Into<String>,
        share: impl Into<String>,
        host_root: impl Into<PathBuf>,
    ) -> Result<Self, LaunchPathError> {
        let server = server.into();
        let share = share.into();
        if server.trim().is_empty() || share.trim().is_empty() {
            return Err(LaunchPathError::InvalidWindowsUncMapping);
        }
        let host_root = host_root.into();
        validate_host_root(&host_root)?;
        self.windows_unc_roots.insert(
            (server.to_ascii_lowercase(), share.to_ascii_lowercase()),
            host_root,
        );
        Ok(self)
    }

    /// Converts one absolute native host path back into persisted LaunchBox
    /// syntax without leaking native separators into portable paths.
    ///
    /// Paths under the LaunchBox root are stored as relative backslash-delimited
    /// values. Paths under a configured Windows mapping recover their original
    /// drive or UNC prefix. Other absolute paths remain native host paths.
    pub fn stored_path_for_host_path(
        &self,
        launchbox_root: &Path,
        host_path: &Path,
    ) -> Result<String, LaunchPathError> {
        if !launchbox_root.is_absolute() {
            return Err(LaunchPathError::LaunchBoxRootNotAbsolute {
                root: launchbox_root.to_path_buf(),
            });
        }
        if !host_path.is_absolute() {
            return Err(LaunchPathError::HostPathNotAbsolute {
                path: host_path.to_path_buf(),
            });
        }
        if let Ok(relative) = host_path.strip_prefix(launchbox_root) {
            return portable_stored_path(relative);
        }

        let mut mapped = Vec::new();
        for (drive, root) in &self.windows_drives {
            if let Ok(relative) = host_path.strip_prefix(root) {
                mapped.push((
                    root.components().count(),
                    format!("{drive}:\\{}", portable_stored_path(relative)?),
                ));
            }
        }
        for ((server, share), root) in &self.windows_unc_roots {
            if let Ok(relative) = host_path.strip_prefix(root) {
                mapped.push((
                    root.components().count(),
                    format!("\\\\{server}\\{share}\\{}", portable_stored_path(relative)?),
                ));
            }
        }
        if let Some((_, path)) = mapped
            .into_iter()
            .max_by(|left, right| left.0.cmp(&right.0).then_with(|| right.1.cmp(&left.1)))
        {
            return Ok(path);
        }

        host_path
            .to_str()
            .map(str::to_string)
            .ok_or_else(|| LaunchPathError::NonUnicodeHostPath {
                path: host_path.to_path_buf(),
            })
    }
}

/// Encodes a safe relative native path as LaunchBox's portable lexical path.
/// This is the only separator conversion needed when persisting paths created
/// below the library root.
pub fn portable_stored_path(relative: &Path) -> Result<String, LaunchPathError> {
    if relative.is_absolute() {
        return Err(LaunchPathError::UnsafePortableRelativePath {
            path: relative.to_path_buf(),
        });
    }
    let mut parts = Vec::new();
    for component in relative.components() {
        match component {
            std::path::Component::Normal(part) => {
                parts.push(
                    part.to_str()
                        .ok_or_else(|| LaunchPathError::NonUnicodeHostPath {
                            path: relative.to_path_buf(),
                        })?,
                );
            }
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir
            | std::path::Component::RootDir
            | std::path::Component::Prefix(_) => {
                return Err(LaunchPathError::UnsafePortableRelativePath {
                    path: relative.to_path_buf(),
                });
            }
        }
    }
    if parts.is_empty() {
        return Err(LaunchPathError::UnsafePortableRelativePath {
            path: relative.to_path_buf(),
        });
    }
    Ok(parts.join("\\"))
}

impl LaunchPathResolver for HostPathResolver {
    fn resolve(
        &self,
        launchbox_root: &Path,
        stored_path: &str,
    ) -> Result<PathBuf, LaunchPathError> {
        if let Some((drive, remainder)) = windows_drive_path(stored_path) {
            #[cfg(windows)]
            {
                let _ = (drive, remainder);
                return Ok(PathBuf::from(stored_path));
            }
            #[cfg(not(windows))]
            {
                let root = self
                    .windows_drives
                    .get(&drive)
                    .ok_or(LaunchPathError::UnmappedWindowsDrive { drive })?;
                return Ok(join_windows_absolute_components(root.clone(), remainder));
            }
        }

        if let Some(unc) = windows_unc_path(stored_path)? {
            #[cfg(windows)]
            {
                let _ = (unc.server, unc.share, unc.remainder);
                return Ok(PathBuf::from(stored_path));
            }
            #[cfg(not(windows))]
            {
                let key = (
                    unc.server.to_ascii_lowercase(),
                    unc.share.to_ascii_lowercase(),
                );
                let root = self.windows_unc_roots.get(&key).ok_or_else(|| {
                    LaunchPathError::UnmappedWindowsUnc {
                        server: unc.server.to_string(),
                        share: unc.share.to_string(),
                    }
                })?;
                return Ok(join_windows_absolute_components(
                    root.clone(),
                    unc.remainder,
                ));
            }
        }

        let native = Path::new(stored_path);
        if native.is_absolute() {
            Ok(native.to_path_buf())
        } else {
            Ok(join_portable_components(
                launchbox_root.to_path_buf(),
                stored_path,
            ))
        }
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum LaunchPathError {
    #[error("Windows drive mapping must use an ASCII letter, not {drive:?}")]
    InvalidWindowsDriveMapping { drive: char },
    #[error("Windows UNC mapping requires a non-empty server and share")]
    InvalidWindowsUncMapping,
    #[error("host path mapping root must be absolute: {root}")]
    HostMappingRootNotAbsolute { root: PathBuf },
    #[error("Windows drive {drive}: has no mapping on this host")]
    UnmappedWindowsDrive { drive: char },
    #[error("Windows UNC share \\\\{server}\\{share} has no mapping on this host")]
    UnmappedWindowsUnc { server: String, share: String },
    #[error("Windows UNC path has no server and share")]
    InvalidWindowsUncPath,
    #[error("LaunchBox root must be absolute when persisting a path: {root}")]
    LaunchBoxRootNotAbsolute { root: PathBuf },
    #[error("host path must be absolute when persisting it: {path}")]
    HostPathNotAbsolute { path: PathBuf },
    #[error("host path cannot be represented as LaunchBox Unicode path data: {path}")]
    NonUnicodeHostPath { path: PathBuf },
    #[error("portable LaunchBox path must be a non-empty safe relative path: {path}")]
    UnsafePortableRelativePath { path: PathBuf },
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum PlatformPathError {
    #[error("platform name cannot be empty")]
    EmptyPlatformName,
}

fn is_windows_reserved_file_stem(stem: &str) -> bool {
    let stem = stem.to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || stem
            .strip_prefix("COM")
            .or_else(|| stem.strip_prefix("LPT"))
            .is_some_and(|number| {
                matches!(number, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
            })
}

fn normalize_drive(drive: char) -> Result<char, LaunchPathError> {
    if drive.is_ascii_alphabetic() {
        Ok(drive.to_ascii_uppercase())
    } else {
        Err(LaunchPathError::InvalidWindowsDriveMapping { drive })
    }
}

fn validate_host_root(root: &Path) -> Result<(), LaunchPathError> {
    if root.is_absolute() {
        Ok(())
    } else {
        Err(LaunchPathError::HostMappingRootNotAbsolute {
            root: root.to_path_buf(),
        })
    }
}

fn windows_drive_path(path: &str) -> Option<(char, &str)> {
    let bytes = path.as_bytes();
    if bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'/' | b'\\')
    {
        Some((char::from(bytes[0]).to_ascii_uppercase(), &path[3..]))
    } else {
        None
    }
}

struct WindowsUncPath<'a> {
    server: &'a str,
    share: &'a str,
    remainder: &'a str,
}

fn windows_unc_path(path: &str) -> Result<Option<WindowsUncPath<'_>>, LaunchPathError> {
    if !(path.starts_with("\\\\") || path.starts_with("//")) {
        return Ok(None);
    }
    let without_prefix = &path[2..];
    let mut parts = without_prefix.splitn(3, ['/', '\\']);
    let server = parts.next().unwrap_or_default();
    let share = parts.next().unwrap_or_default();
    if server.is_empty() || share.is_empty() {
        return Err(LaunchPathError::InvalidWindowsUncPath);
    }
    Ok(Some(WindowsUncPath {
        server,
        share,
        remainder: parts.next().unwrap_or_default(),
    }))
}

fn join_portable_components(mut root: PathBuf, path: &str) -> PathBuf {
    for component in path.split(['/', '\\']).filter(|part| !part.is_empty()) {
        root.push(component);
    }
    root
}

#[cfg(not(windows))]
fn join_windows_absolute_components(mut root: PathBuf, path: &str) -> PathBuf {
    let mut appended_components = 0usize;
    for component in path.split(['/', '\\']).filter(|part| !part.is_empty()) {
        match component {
            "." => {}
            ".." if appended_components > 0 => {
                root.pop();
                appended_components -= 1;
            }
            ".." => {}
            _ => {
                root.push(component);
                appended_components += 1;
            }
        }
    }
    root
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_paths_accept_both_separator_styles() {
        let resolver = HostPathResolver::default();
        assert_eq!(
            resolver
                .resolve(Path::new("/library"), r"Games\Console/game.rom")
                .expect("resolve portable path"),
            PathBuf::from("/library/Games/Console/game.rom")
        );
    }

    #[test]
    fn windows_absolute_classification_is_host_independent() {
        assert!(is_windows_absolute_path(r"D:\Games\game.rom"));
        assert!(is_windows_absolute_path("e:/Games/game.rom"));
        assert!(is_windows_absolute_path(r"\\server\share\game.rom"));
        assert!(is_windows_absolute_path("//server/share/game.rom"));
        assert!(!is_windows_absolute_path(r"Games\game.rom"));
        assert!(!is_windows_absolute_path("Games/game.rom"));
        assert!(!is_windows_absolute_path("/native/host/game.rom"));
    }

    #[test]
    fn platform_document_names_are_valid_on_every_supported_host() {
        assert_eq!(
            navigation_document_file_name("Playlist: DOS/Windows").unwrap(),
            "Playlist_ DOS_Windows.xml"
        );
        assert_eq!(
            platform_document_file_name("Dragon 32/64").unwrap(),
            "Dragon 32_64.xml"
        );
        assert_eq!(platform_document_file_name("CON").unwrap(), "CON_.xml");
        assert_eq!(
            platform_document_file_name("Arcade. ").unwrap(),
            "Arcade_.xml"
        );
        assert_eq!(
            platform_document_file_name("PC: DOS? Classics").unwrap(),
            "PC_ DOS_ Classics.xml"
        );
        assert_eq!(
            platform_document_file_name("   "),
            Err(PlatformPathError::EmptyPlatformName)
        );
    }

    #[test]
    fn default_platform_folders_keep_launchbox_syntax_out_of_native_paths() {
        let folders = default_platform_folders("Dragon 32/64").unwrap();
        assert_eq!(folders.len(), 51);
        assert_eq!(
            folders
                .iter()
                .find(|folder| folder.media_type == "Box - Front")
                .unwrap()
                .folder_path,
            r"Images\Dragon 32_64\Box - Front"
        );
        assert_eq!(
            folders
                .iter()
                .find(|folder| folder.media_type == "Theme Video")
                .unwrap()
                .folder_path,
            r"Videos\Dragon 32_64\Theme"
        );
        assert!(folders
            .iter()
            .all(|folder| folder.platform == "Dragon 32/64"));
    }

    #[cfg(not(windows))]
    #[test]
    fn foreign_windows_drives_are_explicitly_mapped_or_rejected() {
        let unmapped = HostPathResolver::default();
        assert_eq!(
            unmapped.resolve(Path::new("/library"), r"d:\Games\game.rom"),
            Err(LaunchPathError::UnmappedWindowsDrive { drive: 'D' })
        );

        let mapped = HostPathResolver::default()
            .with_windows_drive_mapping('d', "/mnt/windows-games")
            .expect("valid drive mapping");
        assert_eq!(
            mapped
                .resolve(Path::new("/library"), r"D:\Games\game.rom")
                .expect("resolve mapped drive"),
            PathBuf::from("/mnt/windows-games/Games/game.rom")
        );
        assert_eq!(
            mapped
                .resolve(Path::new("/library"), r"D:\..\outside.rom")
                .expect("drive root clamps parent traversal"),
            PathBuf::from("/mnt/windows-games/outside.rom")
        );
    }

    #[test]
    fn mapping_roots_must_be_unambiguous_host_absolute_paths() {
        assert_eq!(
            HostPathResolver::default().with_windows_drive_mapping('D', "relative/root"),
            Err(LaunchPathError::HostMappingRootNotAbsolute {
                root: PathBuf::from("relative/root"),
            })
        );
    }

    #[test]
    fn persisted_paths_are_portable_below_the_library_root() {
        let resolver = HostPathResolver::default();
        assert_eq!(
            resolver
                .stored_path_for_host_path(
                    Path::new("/library"),
                    Path::new("/library/Games/Fixture Console/game.rom"),
                )
                .unwrap(),
            r"Games\Fixture Console\game.rom"
        );
        assert_eq!(
            portable_stored_path(Path::new("Games/Fixture Console/game.rom")).unwrap(),
            r"Games\Fixture Console\game.rom"
        );
        assert!(matches!(
            portable_stored_path(Path::new("../outside.rom")),
            Err(LaunchPathError::UnsafePortableRelativePath { .. })
        ));
    }

    #[cfg(not(windows))]
    #[test]
    fn persisted_paths_recover_the_most_specific_windows_mapping() {
        let resolver = HostPathResolver::default()
            .with_windows_drive_mapping('D', "/mnt/windows")
            .unwrap()
            .with_windows_drive_mapping('E', "/mnt/windows/roms")
            .unwrap()
            .with_windows_unc_mapping("SERVER", "Archive", "/net/archive")
            .unwrap();
        assert_eq!(
            resolver
                .stored_path_for_host_path(
                    Path::new("/library"),
                    Path::new("/mnt/windows/roms/Arcade/game.zip"),
                )
                .unwrap(),
            r"E:\Arcade\game.zip"
        );
        assert_eq!(
            resolver
                .stored_path_for_host_path(
                    Path::new("/library"),
                    Path::new("/net/archive/Console/game.chd"),
                )
                .unwrap(),
            r"\\server\archive\Console\game.chd"
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn foreign_unc_shares_are_explicitly_mapped() {
        let resolver = HostPathResolver::default()
            .with_windows_unc_mapping("SERVER", "Roms", "/net/roms")
            .expect("valid UNC mapping");
        assert_eq!(
            resolver
                .resolve(Path::new("/library"), r"\\server\roms\Arcade\game.zip")
                .expect("resolve mapped UNC path"),
            PathBuf::from("/net/roms/Arcade/game.zip")
        );
    }
}
