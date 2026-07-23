use lb_domain::{AdditionalApplication, Game, Mount};
use lb_platform::{
    build_additional_application_plan_with_mounts_context_and_resolver,
    build_launch_plan_with_mounts_context_and_resolver, is_windows_absolute_path, HostPathResolver,
    LaunchContext, LaunchKind, LaunchPlanError,
};
use lb_storage::LaunchBoxDataIndex;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::error::Error;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = std::env::args().skip(1);
    let input = arguments
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("fixtures/launchbox"));
    let mut path_resolver = HostPathResolver::default();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--map-windows-drive" => {
                let mapping = arguments
                    .next()
                    .ok_or("--map-windows-drive requires DRIVE=HOST_ROOT")?;
                let (drive, host_root) = mapping
                    .split_once('=')
                    .ok_or("--map-windows-drive requires DRIVE=HOST_ROOT")?;
                let mut characters = drive.chars();
                let drive = characters
                    .next()
                    .filter(|_| characters.next().is_none())
                    .ok_or("a drive mapping requires one drive letter")?;
                path_resolver =
                    path_resolver.with_windows_drive_mapping(drive, PathBuf::from(host_root))?;
            }
            "--map-windows-unc" => {
                let mapping = arguments
                    .next()
                    .ok_or("--map-windows-unc requires SERVER/SHARE=HOST_ROOT")?;
                let (windows_root, host_root) = mapping
                    .split_once('=')
                    .ok_or("--map-windows-unc requires SERVER/SHARE=HOST_ROOT")?;
                let (server, share) = windows_root
                    .split_once('/')
                    .ok_or("--map-windows-unc requires SERVER/SHARE=HOST_ROOT")?;
                path_resolver = path_resolver.with_windows_unc_mapping(
                    server,
                    share,
                    PathBuf::from(host_root),
                )?;
            }
            _ => return Err(format!("unknown argument {argument}").into()),
        }
    }
    let data = LaunchBoxDataIndex::load(&input)?;
    let launchbox_root = launchbox_root(&input);
    let configuration = data.emulator_configuration();
    let context = LaunchContext {
        frontend_executable: Some(launchbox_root.join("LaunchBox.exe")),
    };
    let mut counts = BTreeMap::<&'static str, usize>::new();
    counts.insert("plans.unresolved_known_variable", 0);
    counts.insert("additional_plans.unresolved_known_variable", 0);
    counts.insert("dosbox_mounts.total", 0);
    let mut mounts_by_game = HashMap::<&str, Vec<Mount>>::new();
    for mount in data.platforms().mounts() {
        increment(&mut counts, "dosbox_mounts.total");
        census_mount(mount, &mut counts);
        mounts_by_game
            .entry(mount.game_id.as_str())
            .or_default()
            .push(mount.clone());
    }

    if let Some(configuration) = configuration {
        for emulator in &configuration.emulators {
            census_command_line_variables(emulator.command_line.as_deref(), &mut counts);
        }
        for mapping in &configuration.platforms {
            census_command_line_variables(mapping.command_line.as_deref(), &mut counts);
            if mapping.m3u_disc_load_enabled {
                increment(&mut counts, "emulator_config.platforms.m3u_enabled");
            }
        }
    }

    for game in data.platforms().games() {
        increment(&mut counts, "games.total");
        census_game_features(game, &mut counts);
        let mounts = mounts_by_game
            .get(game.id.as_str())
            .map(Vec::as_slice)
            .unwrap_or_default();
        match build_launch_plan_with_mounts_context_and_resolver(
            &launchbox_root,
            game,
            mounts,
            configuration,
            &context,
            &path_resolver,
        ) {
            Ok(plan) => {
                if plan.request.arguments.iter().any(|argument| {
                    argument
                        .to_str()
                        .is_some_and(contains_known_launch_variable)
                }) {
                    increment(&mut counts, "plans.unresolved_known_variable");
                }
                match plan.kind {
                    LaunchKind::Direct => increment(&mut counts, "plans.direct"),
                    LaunchKind::DosBox => increment(&mut counts, "plans.dosbox"),
                    LaunchKind::ScummVm => increment(&mut counts, "plans.scummvm"),
                    LaunchKind::Emulator { .. } if game.emulator_id.is_some() => {
                        increment(&mut counts, "plans.emulator.explicit")
                    }
                    LaunchKind::Emulator { .. } => increment(&mut counts, "plans.emulator.default"),
                }
            }
            Err(error) => increment(&mut counts, launch_error_key(&error)),
        }
    }

    let games_by_id = data
        .platforms()
        .games()
        .map(|game| (game.id.as_str(), game))
        .collect::<HashMap<_, _>>();
    let mut games_with_disc_applications = HashSet::new();
    for application in data.platforms().additional_applications() {
        increment(&mut counts, "additional_apps.total");
        census_additional_application(application, &mut counts);
        if application.disc.is_some() {
            games_with_disc_applications.insert(application.game_id.as_str());
        }
        let Some(game) = games_by_id.get(application.game_id.as_str()) else {
            increment(&mut counts, "additional_plans.errors.missing_parent_game");
            continue;
        };
        let mounts = mounts_by_game
            .get(game.id.as_str())
            .map(Vec::as_slice)
            .unwrap_or_default();
        match build_additional_application_plan_with_mounts_context_and_resolver(
            &launchbox_root,
            game,
            application,
            mounts,
            configuration,
            &context,
            &path_resolver,
        ) {
            Ok(plan) => {
                if plan.request.arguments.iter().any(|argument| {
                    argument
                        .to_str()
                        .is_some_and(contains_known_launch_variable)
                }) {
                    increment(&mut counts, "additional_plans.unresolved_known_variable");
                }
                match plan.kind {
                    LaunchKind::Direct => increment(&mut counts, "additional_plans.direct"),
                    LaunchKind::DosBox => increment(&mut counts, "additional_plans.dosbox"),
                    LaunchKind::ScummVm => increment(&mut counts, "additional_plans.scummvm"),
                    LaunchKind::Emulator { .. } => {
                        increment(&mut counts, "additional_plans.emulator")
                    }
                }
            }
            Err(error) => increment(&mut counts, additional_launch_error_key(&error)),
        }
    }

    let supported = count(&counts, "plans.direct")
        + count(&counts, "plans.dosbox")
        + count(&counts, "plans.scummvm")
        + count(&counts, "plans.emulator.explicit")
        + count(&counts, "plans.emulator.default");
    counts.insert("plans.supported.total", supported);
    counts.insert(
        "plans.errors.total",
        count(&counts, "games.total").saturating_sub(supported),
    );
    let additional_supported = count(&counts, "additional_plans.direct")
        + count(&counts, "additional_plans.dosbox")
        + count(&counts, "additional_plans.scummvm")
        + count(&counts, "additional_plans.emulator");
    counts.insert("additional_plans.supported.total", additional_supported);
    counts.insert(
        "additional_plans.errors.total",
        count(&counts, "additional_apps.total").saturating_sub(additional_supported),
    );
    counts.insert(
        "games.with_disc_applications",
        games_with_disc_applications.len(),
    );

    println!("Launch plan compatibility audit (aggregate, value-free)");
    for (key, value) in counts {
        println!("{key}: {value}");
    }
    Ok(())
}

const KNOWN_LAUNCH_VARIABLES: [(&str, &str); 5] = [
    ("%romfile%", "emulator_config.command_lines.romfile"),
    ("%romlocation%", "emulator_config.command_lines.romlocation"),
    ("%platform%", "emulator_config.command_lines.platform"),
    ("%gameid%", "emulator_config.command_lines.gameid"),
    (
        "%launchboxorbigboxexepath%",
        "emulator_config.command_lines.frontend_executable",
    ),
];

fn census_command_line_variables(
    command_line: Option<&str>,
    counts: &mut BTreeMap<&'static str, usize>,
) {
    let Some(command_line) = command_line else {
        return;
    };
    for (variable, key) in KNOWN_LAUNCH_VARIABLES {
        if command_line
            .to_ascii_lowercase()
            .contains(&variable.to_ascii_lowercase())
        {
            increment(counts, key);
        }
    }
}

fn contains_known_launch_variable(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    KNOWN_LAUNCH_VARIABLES
        .iter()
        .any(|(variable, _)| value.contains(variable))
}

fn launchbox_root(input: &Path) -> PathBuf {
    if input.join("Data").is_dir() {
        input.to_path_buf()
    } else if input.file_name().is_some_and(|name| name == "Data") {
        input.parent().unwrap_or(input).to_path_buf()
    } else {
        input.to_path_buf()
    }
}

fn census_game_features(game: &Game, counts: &mut BTreeMap<&'static str, usize>) {
    if game.emulator_id.is_some() {
        increment(counts, "games.explicit_emulator");
    }
    if non_empty(game.command_line.as_deref()) {
        increment(counts, "games.command_line");
    }
    if game.use_dos_box {
        increment(counts, "games.dosbox");
    }
    if game.use_scumm_vm {
        increment(counts, "games.scummvm");
    }
    if archive_path(&game.application_path) {
        increment(counts, "games.archive_path");
    }
    if game.use_startup_screen {
        increment(counts, "games.startup_screen");
    }
    if game.use_pause_screen {
        increment(counts, "games.pause_screen");
    }
    if game_has_script(game) {
        increment(counts, "games.one_or_more_scripts");
    }
    if is_windows_absolute_path(&game.application_path) {
        increment(counts, "games.windows_absolute_path");
    }
}

fn census_additional_application(
    application: &AdditionalApplication,
    counts: &mut BTreeMap<&'static str, usize>,
) {
    if application.auto_run_before {
        increment(counts, "additional_apps.auto_run_before");
    }
    if application.auto_run_after {
        increment(counts, "additional_apps.auto_run_after");
    }
    if application.wait_for_exit {
        increment(counts, "additional_apps.wait_for_exit");
    }
    if application.use_emulator {
        increment(counts, "additional_apps.use_emulator");
    }
    if application.use_dos_box {
        increment(counts, "additional_apps.use_dosbox");
    }
    if non_empty(application.command_line.as_deref()) {
        increment(counts, "additional_apps.command_line");
    }
    if archive_path(&application.application_path) {
        increment(counts, "additional_apps.archive_path");
    }
    if application.disc.is_some() {
        increment(counts, "additional_apps.disc");
        if archive_path(&application.application_path) {
            increment(counts, "additional_apps.disc_archive_path");
        }
    }
}

fn census_mount(mount: &Mount, counts: &mut BTreeMap<&'static str, usize>) {
    if mount.mount_type.eq_ignore_ascii_case("Folder") {
        increment(counts, "dosbox_mounts.folder");
    } else if mount.mount_type.eq_ignore_ascii_case("File") {
        increment(counts, "dosbox_mounts.file");
    } else {
        increment(counts, "dosbox_mounts.unknown_mount_type");
    }
    if mount.media_type.eq_ignore_ascii_case("Floppy") {
        increment(counts, "dosbox_mounts.media.floppy");
    } else if mount.media_type.eq_ignore_ascii_case("CD-ROM/ISO") {
        increment(counts, "dosbox_mounts.media.cdrom_iso");
    } else if mount.media_type.eq_ignore_ascii_case("Hard Disk") {
        increment(counts, "dosbox_mounts.media.hard_disk");
    } else if mount.media_type.trim().is_empty() {
        increment(counts, "dosbox_mounts.media.unspecified");
    } else {
        increment(counts, "dosbox_mounts.media.unknown");
    }
}

fn game_has_script(game: &Game) -> bool {
    [
        game.load_state_auto_hotkey_script.as_deref(),
        game.pause_auto_hotkey_script.as_deref(),
        game.reset_auto_hotkey_script.as_deref(),
        game.resume_auto_hotkey_script.as_deref(),
        game.save_state_auto_hotkey_script.as_deref(),
        game.swap_discs_auto_hotkey_script.as_deref(),
    ]
    .into_iter()
    .any(non_empty)
}

fn launch_error_key(error: &LaunchPlanError) -> &'static str {
    match error {
        LaunchPlanError::InvalidFrontendLaunchSetting { .. } => {
            "errors.invalid_frontend_launch_setting"
        }
        LaunchPlanError::AdditionalApplicationGameMismatch { .. } => {
            "errors.additional_app.game_mismatch"
        }
        LaunchPlanError::MissingAdditionalApplicationPath { .. } => {
            "errors.additional_app.missing_application_path"
        }
        LaunchPlanError::MissingGameApplicationPath { .. } => {
            "errors.missing_game_application_path"
        }
        LaunchPlanError::GamePath { source, .. } => path_error_key("game", source),
        LaunchPlanError::MissingEmulatorConfiguration { .. } => {
            "errors.missing_emulator_configuration"
        }
        LaunchPlanError::EmulatorNotFound { .. } => "errors.emulator_not_found",
        LaunchPlanError::AmbiguousDefaultEmulator { .. } => "errors.ambiguous_default_emulator",
        LaunchPlanError::AmbiguousEmulatorPlatform { .. } => "errors.ambiguous_emulator_platform",
        LaunchPlanError::MissingEmulatorApplicationPath { .. } => {
            "errors.missing_emulator_application_path"
        }
        LaunchPlanError::EmulatorPath { source, .. } => path_error_key("emulator", source),
        LaunchPlanError::MissingGameFileName { .. } => "errors.missing_game_file_name",
        LaunchPlanError::MissingRomLocation { .. } => "errors.missing_rom_location",
        LaunchPlanError::MissingFrontendExecutableForVariable => {
            "errors.missing_frontend_executable_for_variable"
        }
        LaunchPlanError::NonUnicodeLaunchVariable { .. } => "errors.non_unicode_launch_variable",
        LaunchPlanError::UnsupportedMode { mode: "DOSBox", .. } => "errors.unsupported_dosbox",
        LaunchPlanError::UnsupportedMode {
            mode: "ScummVM", ..
        } => "errors.unsupported_scummvm",
        LaunchPlanError::UnsupportedMode { .. } => "errors.unsupported_other_mode",
        LaunchPlanError::ConflictingModes { .. } => "errors.conflicting_modes",
        LaunchPlanError::DosBoxPlanning { .. } => "errors.dosbox_planning",
        LaunchPlanError::ScummVmPlanning { .. } => "errors.scummvm_planning",
        LaunchPlanError::UnsupportedArchiveExtraction { .. } => {
            "errors.unsupported_archive_extraction"
        }
        LaunchPlanError::UnsupportedM3uPreparation { .. } => "errors.unsupported_m3u_preparation",
        LaunchPlanError::ArchiveExtraction { .. } => "errors.archive_extraction",
        LaunchPlanError::M3uPreparation { .. } => "errors.m3u_preparation",
    }
}

fn additional_launch_error_key(error: &LaunchPlanError) -> &'static str {
    match error {
        LaunchPlanError::InvalidFrontendLaunchSetting { .. } => {
            "additional_plans.errors.invalid_frontend_launch_setting"
        }
        LaunchPlanError::AdditionalApplicationGameMismatch { .. } => {
            "additional_plans.errors.game_mismatch"
        }
        LaunchPlanError::MissingAdditionalApplicationPath { .. }
        | LaunchPlanError::MissingGameApplicationPath { .. } => {
            "additional_plans.errors.missing_application_path"
        }
        LaunchPlanError::GamePath { source, .. } => additional_path_error_key("target", source),
        LaunchPlanError::MissingEmulatorConfiguration { .. } => {
            "additional_plans.errors.missing_emulator_configuration"
        }
        LaunchPlanError::EmulatorNotFound { .. } => "additional_plans.errors.emulator_not_found",
        LaunchPlanError::AmbiguousDefaultEmulator { .. } => {
            "additional_plans.errors.ambiguous_default_emulator"
        }
        LaunchPlanError::AmbiguousEmulatorPlatform { .. } => {
            "additional_plans.errors.ambiguous_emulator_platform"
        }
        LaunchPlanError::MissingEmulatorApplicationPath { .. } => {
            "additional_plans.errors.missing_emulator_application_path"
        }
        LaunchPlanError::EmulatorPath { source, .. } => {
            additional_path_error_key("emulator", source)
        }
        LaunchPlanError::MissingGameFileName { .. } => "additional_plans.errors.missing_file_name",
        LaunchPlanError::MissingRomLocation { .. } => {
            "additional_plans.errors.missing_rom_location"
        }
        LaunchPlanError::MissingFrontendExecutableForVariable => {
            "additional_plans.errors.missing_frontend_executable_for_variable"
        }
        LaunchPlanError::NonUnicodeLaunchVariable { .. } => {
            "additional_plans.errors.non_unicode_launch_variable"
        }
        LaunchPlanError::UnsupportedMode { mode: "DOSBox", .. } => {
            "additional_plans.errors.unsupported_dosbox"
        }
        LaunchPlanError::UnsupportedMode {
            mode: "ScummVM", ..
        } => "additional_plans.errors.unsupported_scummvm",
        LaunchPlanError::UnsupportedMode { .. } => "additional_plans.errors.unsupported_other_mode",
        LaunchPlanError::ConflictingModes { .. } => "additional_plans.errors.conflicting_modes",
        LaunchPlanError::DosBoxPlanning { .. } => "additional_plans.errors.dosbox_planning",
        LaunchPlanError::ScummVmPlanning { .. } => "additional_plans.errors.scummvm_planning",
        LaunchPlanError::UnsupportedArchiveExtraction { .. } => {
            "additional_plans.errors.unsupported_archive_extraction"
        }
        LaunchPlanError::UnsupportedM3uPreparation { .. } => {
            "additional_plans.errors.unsupported_m3u_preparation"
        }
        LaunchPlanError::ArchiveExtraction { .. } => "additional_plans.errors.archive_extraction",
        LaunchPlanError::M3uPreparation { .. } => "additional_plans.errors.m3u_preparation",
    }
}

fn additional_path_error_key(scope: &str, error: &lb_platform::LaunchPathError) -> &'static str {
    use lb_platform::LaunchPathError;
    match (scope, error) {
        ("target", LaunchPathError::UnmappedWindowsDrive { .. }) => {
            "additional_plans.errors.target_path.unmapped_windows_drive"
        }
        ("target", LaunchPathError::UnmappedWindowsUnc { .. }) => {
            "additional_plans.errors.target_path.unmapped_windows_unc"
        }
        ("target", LaunchPathError::InvalidWindowsUncPath) => {
            "additional_plans.errors.target_path.invalid_windows_unc"
        }
        ("emulator", LaunchPathError::UnmappedWindowsDrive { .. }) => {
            "additional_plans.errors.emulator_path.unmapped_windows_drive"
        }
        ("emulator", LaunchPathError::UnmappedWindowsUnc { .. }) => {
            "additional_plans.errors.emulator_path.unmapped_windows_unc"
        }
        ("emulator", LaunchPathError::InvalidWindowsUncPath) => {
            "additional_plans.errors.emulator_path.invalid_windows_unc"
        }
        (_, LaunchPathError::UnmappedWindowsDrive { .. }) => {
            "additional_plans.errors.path.unmapped_windows_drive"
        }
        (_, LaunchPathError::UnmappedWindowsUnc { .. }) => {
            "additional_plans.errors.path.unmapped_windows_unc"
        }
        (_, LaunchPathError::InvalidWindowsUncPath) => {
            "additional_plans.errors.path.invalid_windows_unc"
        }
        (_, LaunchPathError::InvalidWindowsDriveMapping { .. }) => {
            "additional_plans.errors.invalid_windows_drive_mapping"
        }
        (_, LaunchPathError::InvalidWindowsUncMapping) => {
            "additional_plans.errors.invalid_windows_unc_mapping"
        }
        (_, LaunchPathError::HostMappingRootNotAbsolute { .. }) => {
            "additional_plans.errors.host_mapping_root_not_absolute"
        }
        _ => "additional_plans.errors.path_other",
    }
}

fn path_error_key(scope: &str, error: &lb_platform::LaunchPathError) -> &'static str {
    use lb_platform::LaunchPathError;
    match (scope, error) {
        ("game", LaunchPathError::UnmappedWindowsDrive { .. }) => {
            "errors.game_path.unmapped_windows_drive"
        }
        ("game", LaunchPathError::UnmappedWindowsUnc { .. }) => {
            "errors.game_path.unmapped_windows_unc"
        }
        ("game", LaunchPathError::InvalidWindowsUncPath) => "errors.game_path.invalid_windows_unc",
        ("emulator", LaunchPathError::UnmappedWindowsDrive { .. }) => {
            "errors.emulator_path.unmapped_windows_drive"
        }
        ("emulator", LaunchPathError::UnmappedWindowsUnc { .. }) => {
            "errors.emulator_path.unmapped_windows_unc"
        }
        ("emulator", LaunchPathError::InvalidWindowsUncPath) => {
            "errors.emulator_path.invalid_windows_unc"
        }
        (_, LaunchPathError::InvalidWindowsDriveMapping { .. }) => {
            "errors.invalid_windows_drive_mapping"
        }
        (_, LaunchPathError::InvalidWindowsUncMapping) => "errors.invalid_windows_unc_mapping",
        (_, LaunchPathError::HostMappingRootNotAbsolute { .. }) => {
            "errors.host_mapping_root_not_absolute"
        }
        _ => "errors.path_other",
    }
}

fn increment(counts: &mut BTreeMap<&'static str, usize>, key: &'static str) {
    *counts.entry(key).or_default() += 1;
}

fn count(counts: &BTreeMap<&'static str, usize>, key: &'static str) -> usize {
    counts.get(key).copied().unwrap_or_default()
}

fn non_empty(value: Option<&str>) -> bool {
    value.is_some_and(|value| !value.trim().is_empty())
}

fn archive_path(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "7z" | "rar" | "zip"
            )
        })
}
