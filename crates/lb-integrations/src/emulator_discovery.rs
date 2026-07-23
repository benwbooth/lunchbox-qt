use std::collections::BTreeSet;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

/// Emulator identities for which the port has an evidence-backed native
/// executable name and registration template. This is intentionally smaller
/// than LaunchBox's full plugin catalog: a filename is not treated as an
/// emulator unless its adapter has been reviewed.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EmulatorDiscoveryProfile {
    RetroArch,
    Dolphin,
    Pcsx2,
    ScummVm,
    Xemu,
}

impl EmulatorDiscoveryProfile {
    pub const ALL: [Self; 5] = [
        Self::RetroArch,
        Self::Dolphin,
        Self::Pcsx2,
        Self::ScummVm,
        Self::Xemu,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::RetroArch => "retroarch",
            Self::Dolphin => "dolphin",
            Self::Pcsx2 => "pcsx2",
            Self::ScummVm => "scummvm",
            Self::Xemu => "xemu",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::RetroArch => "RetroArch",
            Self::Dolphin => "Dolphin",
            Self::Pcsx2 => "PCSX2",
            Self::ScummVm => "ScummVM",
            Self::Xemu => "Xemu",
        }
    }

    /// Default command lines recovered from the concrete LaunchBox 13.27
    /// adapters. RetroArch requires a per-platform core and therefore has no
    /// safe emulator-wide default.
    pub const fn command_line(self) -> Option<&'static str> {
        match self {
            Self::RetroArch => None,
            Self::Dolphin => Some(
                "-b -C Dolphin.Interface.ConfirmStop=False -C Dolphin.Display.Fullscreen=True -e",
            ),
            Self::Pcsx2 => Some("-fullscreen -nogui"),
            Self::ScummVm => Some("-p %romfile% --auto-detect --fullscreen"),
            Self::Xemu => Some("-full-screen -dvd_path"),
        }
    }

    pub const fn auto_extract(self) -> bool {
        matches!(self, Self::RetroArch)
    }

    pub const fn hide_console(self) -> bool {
        matches!(self, Self::RetroArch | Self::Dolphin | Self::ScummVm)
    }

    pub const fn use_pause_screen(self) -> bool {
        !matches!(self, Self::Xemu)
    }

    pub const fn suspend_process_on_pause(self) -> bool {
        !matches!(self, Self::Xemu)
    }

    pub const fn forceful_pause_screen_activation(self) -> bool {
        !matches!(self, Self::Xemu)
    }

    /// Platforms whose support is explicit in the concrete 13.27 adapter and
    /// does not depend on the online LaunchBox Games DB.
    pub const fn supported_platforms(self) -> &'static [&'static str] {
        match self {
            Self::RetroArch => &[],
            Self::Dolphin => &["Nintendo GameCube", "Nintendo Wii"],
            Self::Pcsx2 => &["Sony PlayStation 2"],
            Self::ScummVm => &["ScummVM"],
            Self::Xemu => &["Microsoft Xbox"],
        }
    }

    #[cfg(target_os = "windows")]
    const fn executable_names(self) -> &'static [&'static str] {
        match self {
            Self::RetroArch => &["retroarch.exe"],
            Self::Dolphin => &["Dolphin.exe"],
            Self::Pcsx2 => &["pcsx2-qt.exe", "pcsx2.exe"],
            Self::ScummVm => &["scummvm.exe"],
            Self::Xemu => &["xemu.exe"],
        }
    }

    #[cfg(target_os = "macos")]
    const fn executable_names(self) -> &'static [&'static str] {
        match self {
            Self::RetroArch => &["retroarch"],
            Self::Dolphin => &["dolphin-emu"],
            Self::Pcsx2 => &["pcsx2-qt", "PCSX2"],
            Self::ScummVm => &["scummvm"],
            Self::Xemu => &["xemu"],
        }
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    const fn executable_names(self) -> &'static [&'static str] {
        match self {
            Self::RetroArch => &["retroarch"],
            // `dolphin` is deliberately excluded on Unix because it is also
            // KDE's file manager. The emulator installs `dolphin-emu`.
            Self::Dolphin => &["dolphin-emu"],
            Self::Pcsx2 => &["pcsx2-qt", "pcsx2"],
            Self::ScummVm => &["scummvm"],
            Self::Xemu => &["xemu"],
        }
    }

    #[cfg(not(any(unix, target_os = "windows")))]
    const fn executable_names(self) -> &'static [&'static str] {
        &[]
    }

    fn matches_file_name(self, file_name: &str) -> bool {
        self.executable_names().iter().any(|candidate| {
            if cfg!(target_os = "windows") {
                candidate.eq_ignore_ascii_case(file_name)
            } else {
                candidate == &file_name
            }
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EmulatorDiscoverySource {
    PortableLibrary,
    NativeApplication,
    SearchPath,
}

impl EmulatorDiscoverySource {
    pub const fn label(self) -> &'static str {
        match self {
            Self::PortableLibrary => "LaunchBox Emulators folder",
            Self::NativeApplication => "native application location",
            Self::SearchPath => "PATH",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveredEmulatorExecutable {
    pub profile: EmulatorDiscoveryProfile,
    pub executable: PathBuf,
    pub source: EmulatorDiscoverySource,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EmulatorDiscoveryRequest {
    pub path_directories: Vec<PathBuf>,
    pub portable_emulators_root: Option<PathBuf>,
    pub native_candidates: Vec<(EmulatorDiscoveryProfile, PathBuf)>,
}

impl EmulatorDiscoveryRequest {
    /// Builds a request from the running host without executing any discovered
    /// program. Search paths are native paths; persisted LaunchBox paths are
    /// produced later by the shared host-path service.
    pub fn for_current_host(launchbox_root: &Path) -> Self {
        let current_directory = env::current_dir().ok();
        let path_directories = env::var_os("PATH")
            .map(|value| {
                env::split_paths(&value)
                    .filter(|path| !path.as_os_str().is_empty())
                    .map(|path| absolute_search_path(path, current_directory.as_deref()))
                    .collect()
            })
            .unwrap_or_default();
        Self {
            path_directories,
            portable_emulators_root: Some(launchbox_root.join("Emulators")),
            native_candidates: native_application_candidates(),
        }
    }

    pub fn with_search_path(mut self, value: impl Into<OsString>) -> Self {
        let current_directory = env::current_dir().ok();
        let value = value.into();
        self.path_directories = env::split_paths(&value)
            .filter(|path| !path.as_os_str().is_empty())
            .map(|path| absolute_search_path(path, current_directory.as_deref()))
            .collect();
        self
    }
}

/// Discovers runnable files only. No candidate is executed, no directory is
/// created, and unreadable search locations are skipped. Portable candidates
/// take precedence over native application locations and PATH aliases.
pub fn discover_emulator_executables(
    request: &EmulatorDiscoveryRequest,
) -> Vec<DiscoveredEmulatorExecutable> {
    let mut candidates = Vec::new();
    if let Some(root) = request
        .portable_emulators_root
        .as_deref()
        .filter(|root| root.is_dir())
    {
        scan_portable_root(root, 0, &mut candidates);
    }
    for (profile, path) in &request.native_candidates {
        push_candidate(
            &mut candidates,
            *profile,
            path.clone(),
            EmulatorDiscoverySource::NativeApplication,
        );
    }
    for directory in &request.path_directories {
        for profile in EmulatorDiscoveryProfile::ALL {
            for name in profile.executable_names() {
                push_candidate(
                    &mut candidates,
                    profile,
                    directory.join(name),
                    EmulatorDiscoverySource::SearchPath,
                );
            }
        }
    }

    candidates.sort_by(|left, right| {
        left.profile
            .cmp(&right.profile)
            .then_with(|| left.source.cmp(&right.source))
            .then_with(|| left.executable.cmp(&right.executable))
    });
    let mut canonical_paths = BTreeSet::new();
    candidates.retain(|candidate| {
        let canonical = fs::canonicalize(&candidate.executable)
            .unwrap_or_else(|_| candidate.executable.clone());
        canonical_paths.insert((candidate.profile, canonical))
    });
    candidates
}

fn scan_portable_root(
    directory: &Path,
    depth: usize,
    candidates: &mut Vec<DiscoveredEmulatorExecutable>,
) {
    const MAX_DEPTH: usize = 3;
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    let mut entries = entries.filter_map(Result::ok).collect::<Vec<_>>();
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() && !file_type.is_symlink() && depth < MAX_DEPTH {
            scan_portable_root(&path, depth + 1, candidates);
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        for profile in EmulatorDiscoveryProfile::ALL {
            if profile.matches_file_name(file_name) {
                push_candidate(
                    candidates,
                    profile,
                    path.clone(),
                    EmulatorDiscoverySource::PortableLibrary,
                );
            }
        }
    }
}

fn push_candidate(
    candidates: &mut Vec<DiscoveredEmulatorExecutable>,
    profile: EmulatorDiscoveryProfile,
    executable: PathBuf,
    source: EmulatorDiscoverySource,
) {
    if is_runnable_file(&executable) {
        candidates.push(DiscoveredEmulatorExecutable {
            profile,
            executable,
            source,
        });
    }
}

fn is_runnable_file(path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

fn absolute_search_path(path: PathBuf, current_directory: Option<&Path>) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        current_directory
            .map(|current| current.join(&path))
            .unwrap_or(path)
    }
}

#[cfg(target_os = "windows")]
fn native_application_candidates() -> Vec<(EmulatorDiscoveryProfile, PathBuf)> {
    let mut candidates = Vec::new();
    for variable in ["ProgramFiles", "ProgramFiles(x86)", "LOCALAPPDATA"] {
        let Some(root) = env::var_os(variable).map(PathBuf::from) else {
            continue;
        };
        candidates.extend([
            (
                EmulatorDiscoveryProfile::RetroArch,
                root.join("RetroArch-Win64/retroarch.exe"),
            ),
            (
                EmulatorDiscoveryProfile::Dolphin,
                root.join("Dolphin/Dolphin.exe"),
            ),
            (
                EmulatorDiscoveryProfile::Pcsx2,
                root.join("PCSX2/pcsx2-qt.exe"),
            ),
            (
                EmulatorDiscoveryProfile::ScummVm,
                root.join("ScummVM/scummvm.exe"),
            ),
            (EmulatorDiscoveryProfile::Xemu, root.join("xemu/xemu.exe")),
        ]);
    }
    candidates
}

#[cfg(target_os = "macos")]
fn native_application_candidates() -> Vec<(EmulatorDiscoveryProfile, PathBuf)> {
    vec![
        (
            EmulatorDiscoveryProfile::RetroArch,
            PathBuf::from("/Applications/RetroArch.app/Contents/MacOS/RetroArch"),
        ),
        (
            EmulatorDiscoveryProfile::Dolphin,
            PathBuf::from("/Applications/Dolphin.app/Contents/MacOS/Dolphin"),
        ),
        (
            EmulatorDiscoveryProfile::Pcsx2,
            PathBuf::from("/Applications/PCSX2.app/Contents/MacOS/PCSX2"),
        ),
        (
            EmulatorDiscoveryProfile::ScummVm,
            PathBuf::from("/Applications/ScummVM.app/Contents/MacOS/scummvm"),
        ),
        (
            EmulatorDiscoveryProfile::Xemu,
            PathBuf::from("/Applications/xemu.app/Contents/MacOS/xemu"),
        ),
    ]
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn native_application_candidates() -> Vec<(EmulatorDiscoveryProfile, PathBuf)> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[cfg(unix)]
    fn make_executable(path: &Path) {
        use std::os::unix::fs::PermissionsExt;
        File::create(path).expect("create executable");
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("mark executable");
    }

    #[cfg(not(unix))]
    fn make_executable(path: &Path) {
        File::create(path).expect("create executable");
    }

    #[test]
    fn discovery_is_read_only_deterministic_and_rejects_name_collisions() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let portable = temporary.path().join("Emulators");
        let path_directory = temporary.path().join("bin");
        fs::create_dir_all(portable.join("PCSX2/bin")).expect("portable tree");
        fs::create_dir_all(portable.join("Xemu")).expect("portable Xemu tree");
        fs::create_dir_all(&path_directory).expect("PATH tree");

        let portable_pcsx2 = if cfg!(target_os = "windows") {
            portable.join("PCSX2/bin/pcsx2-qt.exe")
        } else {
            portable.join("PCSX2/bin/pcsx2-qt")
        };
        make_executable(&portable_pcsx2);
        let portable_xemu = if cfg!(target_os = "windows") {
            portable.join("Xemu/xemu.exe")
        } else {
            portable.join("Xemu/xemu")
        };
        make_executable(&portable_xemu);
        let scummvm = if cfg!(target_os = "windows") {
            path_directory.join("scummvm.exe")
        } else {
            path_directory.join("scummvm")
        };
        make_executable(&scummvm);
        if !cfg!(target_os = "windows") {
            // KDE's file manager must never be mistaken for the emulator.
            make_executable(&path_directory.join("dolphin"));
        }
        let non_executable_retroarch = if cfg!(target_os = "windows") {
            path_directory.join("retroarch.exe")
        } else {
            path_directory.join("retroarch")
        };
        File::create(&non_executable_retroarch).expect("non-executable candidate");

        let request = EmulatorDiscoveryRequest {
            path_directories: vec![path_directory],
            portable_emulators_root: Some(portable.clone()),
            native_candidates: Vec::new(),
        };
        let first = discover_emulator_executables(&request);
        let second = discover_emulator_executables(&request);
        assert_eq!(first, second);
        assert!(first.iter().any(|candidate| {
            candidate.profile == EmulatorDiscoveryProfile::Pcsx2
                && candidate.executable == portable_pcsx2
                && candidate.source == EmulatorDiscoverySource::PortableLibrary
        }));
        assert!(first.iter().any(|candidate| {
            candidate.profile == EmulatorDiscoveryProfile::ScummVm
                && candidate.executable == scummvm
                && candidate.source == EmulatorDiscoverySource::SearchPath
        }));
        assert!(first.iter().any(|candidate| {
            candidate.profile == EmulatorDiscoveryProfile::Xemu
                && candidate.executable == portable_xemu
                && candidate.source == EmulatorDiscoverySource::PortableLibrary
        }));
        if !cfg!(target_os = "windows") {
            assert!(!first
                .iter()
                .any(|candidate| candidate.profile == EmulatorDiscoveryProfile::Dolphin));
            assert!(!first
                .iter()
                .any(|candidate| candidate.profile == EmulatorDiscoveryProfile::RetroArch));
        }
        assert_eq!(fs::read_dir(&portable).expect("portable root").count(), 2);
    }

    #[cfg(unix)]
    #[test]
    fn canonical_aliases_are_deduplicated_with_portable_precedence() {
        use std::os::unix::fs::symlink;

        let temporary = tempfile::tempdir().expect("temporary directory");
        let portable = temporary.path().join("Emulators/RetroArch");
        let path_directory = temporary.path().join("bin");
        fs::create_dir_all(&portable).expect("portable tree");
        fs::create_dir_all(&path_directory).expect("PATH tree");
        let executable = portable.join("retroarch");
        make_executable(&executable);
        symlink(&executable, path_directory.join("retroarch")).expect("PATH alias");

        let candidates = discover_emulator_executables(&EmulatorDiscoveryRequest {
            path_directories: vec![path_directory],
            portable_emulators_root: Some(temporary.path().join("Emulators")),
            native_candidates: Vec::new(),
        });
        assert_eq!(candidates.len(), 1);
        assert_eq!(
            candidates[0].source,
            EmulatorDiscoverySource::PortableLibrary
        );
        assert_eq!(candidates[0].executable, executable);
    }

    #[test]
    fn reviewed_profiles_keep_online_database_support_out_of_static_mappings() {
        assert!(EmulatorDiscoveryProfile::RetroArch
            .supported_platforms()
            .is_empty());
        assert_eq!(
            EmulatorDiscoveryProfile::Dolphin.supported_platforms(),
            ["Nintendo GameCube", "Nintendo Wii"]
        );
        assert_eq!(
            EmulatorDiscoveryProfile::Pcsx2.supported_platforms(),
            ["Sony PlayStation 2"]
        );
        assert_eq!(
            EmulatorDiscoveryProfile::ScummVm.command_line(),
            Some("-p %romfile% --auto-detect --fullscreen")
        );
        assert_eq!(
            EmulatorDiscoveryProfile::Xemu.supported_platforms(),
            ["Microsoft Xbox"]
        );
        assert_eq!(
            EmulatorDiscoveryProfile::Xemu.command_line(),
            Some("-full-screen -dvd_path")
        );
        assert!(!EmulatorDiscoveryProfile::Xemu.use_pause_screen());
    }
}
