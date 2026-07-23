mod archive;
mod dosbox;
mod m3u;
mod path;
mod path_settings;
mod scummvm;

pub use archive::{
    ArchiveCreationError, ArchiveExtractionError, ArchiveExtractor, LaunchResourceLease,
};
pub use dosbox::DosBoxPlanError;
pub use m3u::M3uPreparationError;
pub use path::{
    default_platform_folders, is_windows_absolute_path, navigation_document_file_name,
    platform_document_file_name, platform_storage_name, portable_storage_name,
    portable_stored_path, HostPathResolver, LaunchPathError, LaunchPathResolver, PlatformPathError,
};
pub use path_settings::{
    default_host_path_mappings_path, HostPathMappings, HostPathMappingsError, WindowsDriveMapping,
    WindowsUncMapping, HOST_PATH_MAPPINGS_VERSION,
};
pub use scummvm::ScummVmPlanError;

use lb_domain::{
    is_unassigned_emulator_id, AdditionalApplication, Emulator, EmulatorConfiguration,
    EmulatorPlatform, Game, Mount, UNASSIGNED_EMULATOR_ID,
};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant, SystemTime};
use thiserror::Error;

/// LaunchBox 3.1 documented a 30-second ceiling for an automatic before-app's
/// `WaitForExit` behavior. Keeping the timeout here makes it an orchestration
/// rule rather than a UI timer.
pub const AUTO_RUN_BEFORE_WAIT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaunchRequest {
    pub executable: PathBuf,
    pub arguments: Vec<OsString>,
    pub working_directory: Option<PathBuf>,
    pub hide_console: bool,
}

impl LaunchRequest {
    pub fn new(executable: impl Into<PathBuf>) -> Self {
        Self {
            executable: executable.into(),
            arguments: Vec::new(),
            working_directory: None,
            hide_console: false,
        }
    }

    pub fn arg(mut self, argument: impl AsRef<OsStr>) -> Self {
        self.arguments.push(argument.as_ref().to_os_string());
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LaunchKind {
    Direct,
    DosBox,
    ScummVm,
    Emulator { id: String, title: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LaunchTarget {
    MainGame,
    AdditionalApplication {
        application_id: String,
        application_name: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaunchPlan {
    pub game_id: String,
    pub game_title: String,
    pub target: LaunchTarget,
    pub kind: LaunchKind,
    pub request: LaunchRequest,
    /// Temporary inputs retained until the process consuming this plan exits.
    /// Ordinary plans that need neither extraction nor a generated playlist
    /// have no resource leases.
    pub resource_leases: Vec<LaunchResourceLease>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LaunchStepRole {
    AutomaticBefore,
    MainGame,
    SelectedAdditionalApplication,
    AutomaticAfter,
}

impl LaunchStepRole {
    pub fn is_primary(self) -> bool {
        matches!(self, Self::MainGame | Self::SelectedAdditionalApplication)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaunchStep {
    pub role: LaunchStepRole,
    pub wait_for_exit: bool,
    pub plan: LaunchPlan,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaunchSequence {
    pub game_id: String,
    pub game_title: String,
    pub steps: Vec<LaunchStep>,
}

/// Runtime values that are not stored in LaunchBox's XML but may be referenced
/// by a persisted command line.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LaunchContext {
    pub frontend_executable: Option<PathBuf>,
}

/// Resolves the executable and semantic argv without invoking a command shell.
/// Per-platform emulator parameters override emulator-wide parameters, while
/// game parameters are appended before the game path.
pub fn build_launch_plan(
    launchbox_root: &Path,
    game: &Game,
    configuration: Option<&EmulatorConfiguration>,
) -> Result<LaunchPlan, LaunchPlanError> {
    build_launch_plan_with_context(
        launchbox_root,
        game,
        configuration,
        &LaunchContext::default(),
    )
}

/// Builds a launch plan with values supplied by the running frontend.
pub fn build_launch_plan_with_context(
    launchbox_root: &Path,
    game: &Game,
    configuration: Option<&EmulatorConfiguration>,
    context: &LaunchContext,
) -> Result<LaunchPlan, LaunchPlanError> {
    build_launch_plan_with_context_and_resolver(
        launchbox_root,
        game,
        configuration,
        context,
        &HostPathResolver::default(),
    )
}

/// Builds a launch plan through an explicit host path service. Linux and macOS
/// callers can supply reviewed mappings for Windows drive or UNC paths without
/// putting Windows parsing rules in the UI or process launcher.
pub fn build_launch_plan_with_context_and_resolver(
    launchbox_root: &Path,
    game: &Game,
    configuration: Option<&EmulatorConfiguration>,
    context: &LaunchContext,
    path_resolver: &dyn LaunchPathResolver,
) -> Result<LaunchPlan, LaunchPlanError> {
    build_launch_plan_with_mounts_context_and_resolver(
        launchbox_root,
        game,
        &[],
        configuration,
        context,
        path_resolver,
    )
}

/// Builds a game launch plan with the game's typed DOSBox mount records.
/// Mount paths are resolved by the same host-path service as every other
/// persisted LaunchBox path.
pub fn build_launch_plan_with_mounts_context_and_resolver(
    launchbox_root: &Path,
    game: &Game,
    mounts: &[Mount],
    configuration: Option<&EmulatorConfiguration>,
    context: &LaunchContext,
    path_resolver: &dyn LaunchPathResolver,
) -> Result<LaunchPlan, LaunchPlanError> {
    validate_game_launch_mode(game)?;

    if game.use_dos_box {
        let plan =
            dosbox::build_plan(launchbox_root, game, mounts, path_resolver).map_err(|source| {
                LaunchPlanError::DosBoxPlanning {
                    game_id: game.id.clone(),
                    source: Box::new(source),
                }
            })?;
        return Ok(LaunchPlan {
            game_id: game.id.clone(),
            game_title: game.title.clone(),
            target: LaunchTarget::MainGame,
            kind: LaunchKind::DosBox,
            request: plan.request,
            resource_leases: plan.resource_leases,
        });
    }
    if game.use_scumm_vm {
        let request =
            scummvm::build_request(launchbox_root, game, path_resolver).map_err(|source| {
                LaunchPlanError::ScummVmPlanning {
                    game_id: game.id.clone(),
                    source: Box::new(source),
                }
            })?;
        return Ok(LaunchPlan {
            game_id: game.id.clone(),
            game_title: game.title.clone(),
            target: LaunchTarget::MainGame,
            kind: LaunchKind::ScummVm,
            request,
            resource_leases: Vec::new(),
        });
    }

    let game_path = resolve_game_path(launchbox_root, game, path_resolver)?;
    build_launch_plan_for_resolved_path(
        launchbox_root,
        game,
        configuration,
        context,
        path_resolver,
        game_path,
        false,
        Vec::new(),
    )
}

fn validate_game_launch_mode(game: &Game) -> Result<(), LaunchPlanError> {
    if game.use_dos_box && game.use_scumm_vm {
        return Err(LaunchPlanError::ConflictingModes {
            game_id: game.id.clone(),
        });
    }
    if !game.use_dos_box && !game.use_scumm_vm && game.application_path.trim().is_empty() {
        return Err(LaunchPlanError::MissingGameApplicationPath {
            game_id: game.id.clone(),
        });
    }
    Ok(())
}

fn resolve_game_path(
    launchbox_root: &Path,
    game: &Game,
    path_resolver: &dyn LaunchPathResolver,
) -> Result<PathBuf, LaunchPlanError> {
    path_resolver
        .resolve(launchbox_root, &game.application_path)
        .map_err(|source| LaunchPlanError::GamePath {
            game_id: game.id.clone(),
            source,
        })
}

#[allow(clippy::too_many_arguments)]
fn build_launch_plan_for_resolved_path(
    launchbox_root: &Path,
    game: &Game,
    configuration: Option<&EmulatorConfiguration>,
    context: &LaunchContext,
    path_resolver: &dyn LaunchPathResolver,
    game_path: PathBuf,
    archive_was_prepared: bool,
    resource_leases: Vec<LaunchResourceLease>,
) -> Result<LaunchPlan, LaunchPlanError> {
    let selected = select_emulator(game, configuration)?;
    let (kind, mut request) = if let Some((emulator, mapping)) = selected {
        if emulator.application_path.trim().is_empty() {
            return Err(LaunchPlanError::MissingEmulatorApplicationPath {
                emulator_id: emulator.id.clone(),
            });
        }
        let auto_extract = mapping
            .and_then(|mapping| mapping.auto_extract)
            .unwrap_or(emulator.auto_extract);
        if auto_extract && is_archive_path(&game_path) && !archive_was_prepared {
            return Err(LaunchPlanError::UnsupportedArchiveExtraction {
                game_id: game.id.clone(),
                path: game_path,
            });
        }
        let executable = path_resolver
            .resolve(launchbox_root, &emulator.application_path)
            .map_err(|source| LaunchPlanError::EmulatorPath {
                emulator_id: emulator.id.clone(),
                source,
            })?;
        let mut request = LaunchRequest::new(&executable);
        request.working_directory = executable.parent().map(Path::to_path_buf);
        request.hide_console = emulator.hide_console;
        let game_argument = if emulator.file_name_without_extension_and_path {
            game_path
                .file_stem()
                .map(OsStr::to_os_string)
                .ok_or_else(|| LaunchPlanError::MissingGameFileName {
                    game_id: game.id.clone(),
                    path: game_path.clone(),
                })?
        } else {
            game_path.as_os_str().to_os_string()
        };
        let variables = LaunchVariables {
            rom_file: &game_argument,
            rom_location: game_path.parent().map(Path::to_path_buf).ok_or_else(|| {
                LaunchPlanError::MissingRomLocation {
                    game_id: game.id.clone(),
                    path: game_path.clone(),
                }
            })?,
            platform: &game.platform,
            game_id: &game.id,
            frontend_executable: context.frontend_executable.as_deref(),
        };
        let effective_parameters = mapping
            .and_then(|mapping| mapping.command_line.as_deref())
            .or(emulator.command_line.as_deref());
        let rom_file_was_explicit =
            append_command_line(&mut request.arguments, effective_parameters, &variables)?
                | append_command_line(
                    &mut request.arguments,
                    game.command_line.as_deref(),
                    &variables,
                )?;

        if !rom_file_was_explicit {
            if emulator.no_space {
                if let Some(last) = request.arguments.last_mut() {
                    last.push(&game_argument);
                } else {
                    request.arguments.push(game_argument);
                }
            } else {
                request.arguments.push(game_argument);
            }
        }
        (
            LaunchKind::Emulator {
                id: emulator.id.clone(),
                title: emulator.title.clone(),
            },
            request,
        )
    } else {
        let mut request = LaunchRequest::new(&game_path);
        request.working_directory = game_path.parent().map(Path::to_path_buf);
        let variables = LaunchVariables {
            rom_file: game_path.as_os_str(),
            rom_location: game_path.parent().map(Path::to_path_buf).ok_or_else(|| {
                LaunchPlanError::MissingRomLocation {
                    game_id: game.id.clone(),
                    path: game_path.clone(),
                }
            })?,
            platform: &game.platform,
            game_id: &game.id,
            frontend_executable: context.frontend_executable.as_deref(),
        };
        append_command_line(
            &mut request.arguments,
            game.command_line.as_deref(),
            &variables,
        )?;
        (LaunchKind::Direct, request)
    };
    request.arguments.shrink_to_fit();
    Ok(LaunchPlan {
        game_id: game.id.clone(),
        game_title: game.title.clone(),
        target: LaunchTarget::MainGame,
        kind,
        request,
        resource_leases,
    })
}

/// Resolves and, when the selected emulator requests it, safely extracts an
/// archived ROM before building its semantic process request. The returned
/// plan owns the temporary extraction until every clone of the plan's leases
/// has been dropped.
pub fn prepare_launch_plan_with_context_and_resolver(
    launchbox_root: &Path,
    game: &Game,
    configuration: Option<&EmulatorConfiguration>,
    context: &LaunchContext,
    path_resolver: &dyn LaunchPathResolver,
    archive_extractor: &ArchiveExtractor,
) -> Result<LaunchPlan, LaunchPlanError> {
    prepare_launch_plan_with_mounts_context_and_resolver(
        launchbox_root,
        game,
        &[],
        configuration,
        context,
        path_resolver,
        archive_extractor,
    )
}

pub fn prepare_launch_plan_with_mounts_context_and_resolver(
    launchbox_root: &Path,
    game: &Game,
    mounts: &[Mount],
    configuration: Option<&EmulatorConfiguration>,
    context: &LaunchContext,
    path_resolver: &dyn LaunchPathResolver,
    archive_extractor: &ArchiveExtractor,
) -> Result<LaunchPlan, LaunchPlanError> {
    validate_game_launch_mode(game)?;
    if game.use_dos_box || game.use_scumm_vm {
        return build_launch_plan_with_mounts_context_and_resolver(
            launchbox_root,
            game,
            mounts,
            configuration,
            context,
            path_resolver,
        );
    }
    let game_path = resolve_game_path(launchbox_root, game, path_resolver)?;
    let auto_extract = select_emulator(game, configuration)?
        .map(|(emulator, mapping)| {
            mapping
                .and_then(|mapping| mapping.auto_extract)
                .unwrap_or(emulator.auto_extract)
        })
        .unwrap_or(false);

    if auto_extract && is_archive_path(&game_path) {
        let prepared = archive_extractor.extract(&game_path).map_err(|source| {
            LaunchPlanError::ArchiveExtraction {
                game_id: game.id.clone(),
                path: game_path.clone(),
                source: Box::new(source),
            }
        })?;
        build_launch_plan_for_resolved_path(
            launchbox_root,
            game,
            configuration,
            context,
            path_resolver,
            prepared.launch_path,
            true,
            vec![prepared.lease],
        )
    } else {
        build_launch_plan_for_resolved_path(
            launchbox_root,
            game,
            configuration,
            context,
            path_resolver,
            game_path,
            false,
            Vec::new(),
        )
    }
}

/// Builds a plan for a user-selected additional application. The parent game
/// supplies platform and variable context, while the additional application's
/// executable, arguments, DOSBox flag, and emulator selection remain distinct.
pub fn build_additional_application_plan_with_context_and_resolver(
    launchbox_root: &Path,
    game: &Game,
    application: &AdditionalApplication,
    configuration: Option<&EmulatorConfiguration>,
    context: &LaunchContext,
    path_resolver: &dyn LaunchPathResolver,
) -> Result<LaunchPlan, LaunchPlanError> {
    build_additional_application_plan_with_mounts_context_and_resolver(
        launchbox_root,
        game,
        application,
        &[],
        configuration,
        context,
        path_resolver,
    )
}

pub fn build_additional_application_plan_with_mounts_context_and_resolver(
    launchbox_root: &Path,
    game: &Game,
    application: &AdditionalApplication,
    mounts: &[Mount],
    configuration: Option<&EmulatorConfiguration>,
    context: &LaunchContext,
    path_resolver: &dyn LaunchPathResolver,
) -> Result<LaunchPlan, LaunchPlanError> {
    if application.game_id != game.id {
        return Err(LaunchPlanError::AdditionalApplicationGameMismatch {
            application_id: application.id.clone(),
            expected_game_id: game.id.clone(),
            actual_game_id: application.game_id.clone(),
        });
    }
    if application.application_path.trim().is_empty() && !application.use_dos_box {
        return Err(LaunchPlanError::MissingAdditionalApplicationPath {
            application_id: application.id.clone(),
        });
    }

    let mut target = game.clone();
    target.application_path = application.application_path.clone();
    target.command_line = application.command_line.clone();
    target.emulator_id = if application.use_emulator {
        application.emulator_id.clone()
    } else {
        Some(UNASSIGNED_EMULATOR_ID.to_string())
    };
    target.use_dos_box = application.use_dos_box;
    target.use_scumm_vm = false;

    let mut plan = build_launch_plan_with_mounts_context_and_resolver(
        launchbox_root,
        &target,
        mounts,
        configuration,
        context,
        path_resolver,
    )?;
    plan.game_title = game.title.clone();
    plan.target = LaunchTarget::AdditionalApplication {
        application_id: application.id.clone(),
        application_name: application.name.clone(),
    };
    Ok(plan)
}

#[allow(clippy::too_many_arguments)]
fn prepare_additional_application_plan_with_mounts_context_and_resolver(
    launchbox_root: &Path,
    game: &Game,
    application: &AdditionalApplication,
    mounts: &[Mount],
    configuration: Option<&EmulatorConfiguration>,
    context: &LaunchContext,
    path_resolver: &dyn LaunchPathResolver,
    archive_extractor: &ArchiveExtractor,
) -> Result<LaunchPlan, LaunchPlanError> {
    if application.game_id != game.id {
        return Err(LaunchPlanError::AdditionalApplicationGameMismatch {
            application_id: application.id.clone(),
            expected_game_id: game.id.clone(),
            actual_game_id: application.game_id.clone(),
        });
    }
    if application.application_path.trim().is_empty() && !application.use_dos_box {
        return Err(LaunchPlanError::MissingAdditionalApplicationPath {
            application_id: application.id.clone(),
        });
    }

    let mut target = game.clone();
    target.application_path = application.application_path.clone();
    target.command_line = application.command_line.clone();
    target.emulator_id = if application.use_emulator {
        application.emulator_id.clone()
    } else {
        Some(UNASSIGNED_EMULATOR_ID.to_string())
    };
    target.use_dos_box = application.use_dos_box;
    target.use_scumm_vm = false;

    let mut plan = prepare_launch_plan_with_mounts_context_and_resolver(
        launchbox_root,
        &target,
        mounts,
        configuration,
        context,
        path_resolver,
        archive_extractor,
    )?;
    plan.game_title = game.title.clone();
    plan.target = LaunchTarget::AdditionalApplication {
        application_id: application.id.clone(),
        application_name: application.name.clone(),
    };
    Ok(plan)
}

/// Builds and validates the complete automatic launch lifecycle before any
/// process is spawned. Automatic applications are ordered by LaunchBox's
/// integer priority and then ID for deterministic ties.
pub fn build_game_launch_sequence_with_context_and_resolver<'a>(
    launchbox_root: &Path,
    game: &Game,
    additional_applications: impl IntoIterator<Item = &'a AdditionalApplication>,
    configuration: Option<&EmulatorConfiguration>,
    context: &LaunchContext,
    path_resolver: &dyn LaunchPathResolver,
) -> Result<LaunchSequence, LaunchPlanError> {
    build_game_launch_sequence_with_mounts_context_and_resolver(
        launchbox_root,
        game,
        additional_applications,
        &[],
        configuration,
        context,
        path_resolver,
    )
}

pub fn build_game_launch_sequence_with_mounts_context_and_resolver<'a>(
    launchbox_root: &Path,
    game: &Game,
    additional_applications: impl IntoIterator<Item = &'a AdditionalApplication>,
    mounts: &[Mount],
    configuration: Option<&EmulatorConfiguration>,
    context: &LaunchContext,
    path_resolver: &dyn LaunchPathResolver,
) -> Result<LaunchSequence, LaunchPlanError> {
    build_game_launch_sequence_internal(
        launchbox_root,
        game,
        additional_applications,
        mounts,
        configuration,
        context,
        path_resolver,
        None,
    )
}

/// Prepares every automatic and primary step before spawning any process, so
/// an extraction failure cannot leave a partially started launch sequence.
pub fn prepare_game_launch_sequence_with_context_and_resolver<'a>(
    launchbox_root: &Path,
    game: &Game,
    additional_applications: impl IntoIterator<Item = &'a AdditionalApplication>,
    configuration: Option<&EmulatorConfiguration>,
    context: &LaunchContext,
    path_resolver: &dyn LaunchPathResolver,
    archive_extractor: &ArchiveExtractor,
) -> Result<LaunchSequence, LaunchPlanError> {
    prepare_game_launch_sequence_with_mounts_context_and_resolver(
        launchbox_root,
        game,
        additional_applications,
        &[],
        configuration,
        context,
        path_resolver,
        archive_extractor,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn prepare_game_launch_sequence_with_mounts_context_and_resolver<'a>(
    launchbox_root: &Path,
    game: &Game,
    additional_applications: impl IntoIterator<Item = &'a AdditionalApplication>,
    mounts: &[Mount],
    configuration: Option<&EmulatorConfiguration>,
    context: &LaunchContext,
    path_resolver: &dyn LaunchPathResolver,
    archive_extractor: &ArchiveExtractor,
) -> Result<LaunchSequence, LaunchPlanError> {
    build_game_launch_sequence_internal(
        launchbox_root,
        game,
        additional_applications,
        mounts,
        configuration,
        context,
        path_resolver,
        Some(archive_extractor),
    )
}

#[allow(clippy::too_many_arguments)]
fn prepare_main_game_plan_for_sequence(
    launchbox_root: &Path,
    game: &Game,
    additional_applications: &[&AdditionalApplication],
    mounts: &[Mount],
    configuration: Option<&EmulatorConfiguration>,
    context: &LaunchContext,
    path_resolver: &dyn LaunchPathResolver,
    archive_extractor: &ArchiveExtractor,
) -> Result<LaunchPlan, LaunchPlanError> {
    validate_game_launch_mode(game)?;
    if game.use_dos_box || game.use_scumm_vm {
        return prepare_launch_plan_with_mounts_context_and_resolver(
            launchbox_root,
            game,
            mounts,
            configuration,
            context,
            path_resolver,
            archive_extractor,
        );
    }
    let game_path = resolve_game_path(launchbox_root, game, path_resolver)?;
    let selected = select_emulator(game, configuration)?;
    let m3u_enabled = selected
        .and_then(|(_, mapping)| mapping)
        .is_some_and(|mapping| mapping.m3u_disc_load_enabled);
    if !m3u_enabled {
        return prepare_launch_plan_with_context_and_resolver(
            launchbox_root,
            game,
            configuration,
            context,
            path_resolver,
            archive_extractor,
        );
    }

    let auto_extract = selected
        .map(|(emulator, mapping)| {
            mapping
                .and_then(|mapping| mapping.auto_extract)
                .unwrap_or(emulator.auto_extract)
        })
        .unwrap_or(false);
    let Some(prepared) = m3u::prepare_m3u(
        launchbox_root,
        &game.id,
        &game_path,
        additional_applications,
        path_resolver,
        auto_extract,
        archive_extractor,
    )
    .map_err(|source| LaunchPlanError::M3uPreparation {
        game_id: game.id.clone(),
        source: Box::new(source),
    })?
    else {
        return prepare_launch_plan_with_context_and_resolver(
            launchbox_root,
            game,
            configuration,
            context,
            path_resolver,
            archive_extractor,
        );
    };

    build_launch_plan_for_resolved_path(
        launchbox_root,
        game,
        configuration,
        context,
        path_resolver,
        prepared.launch_path,
        true,
        prepared.resource_leases,
    )
}

#[allow(clippy::too_many_arguments)]
fn build_game_launch_sequence_internal<'a>(
    launchbox_root: &Path,
    game: &Game,
    additional_applications: impl IntoIterator<Item = &'a AdditionalApplication>,
    mounts: &[Mount],
    configuration: Option<&EmulatorConfiguration>,
    context: &LaunchContext,
    path_resolver: &dyn LaunchPathResolver,
    archive_extractor: Option<&ArchiveExtractor>,
) -> Result<LaunchSequence, LaunchPlanError> {
    let mut applications = additional_applications
        .into_iter()
        .filter(|application| application.game_id == game.id)
        .collect::<Vec<_>>();
    applications.sort_by(|left, right| {
        left.priority
            .cmp(&right.priority)
            .then_with(|| left.id.cmp(&right.id))
    });
    if !game.use_dos_box
        && !game.use_scumm_vm
        && archive_extractor.is_none()
        && applications
            .iter()
            .any(|application| application.disc.is_some())
        && select_emulator(game, configuration)?
            .and_then(|(_, mapping)| mapping)
            .is_some_and(|mapping| mapping.m3u_disc_load_enabled)
    {
        return Err(LaunchPlanError::UnsupportedM3uPreparation {
            game_id: game.id.clone(),
        });
    }

    let mut steps = Vec::new();
    for application in applications
        .iter()
        .copied()
        .filter(|application| application.auto_run_before)
    {
        steps.push(LaunchStep {
            role: LaunchStepRole::AutomaticBefore,
            wait_for_exit: application.wait_for_exit,
            plan: if let Some(archive_extractor) = archive_extractor {
                prepare_additional_application_plan_with_mounts_context_and_resolver(
                    launchbox_root,
                    game,
                    application,
                    mounts,
                    configuration,
                    context,
                    path_resolver,
                    archive_extractor,
                )?
            } else {
                build_additional_application_plan_with_mounts_context_and_resolver(
                    launchbox_root,
                    game,
                    application,
                    mounts,
                    configuration,
                    context,
                    path_resolver,
                )?
            },
        });
    }

    steps.push(LaunchStep {
        role: LaunchStepRole::MainGame,
        wait_for_exit: applications
            .iter()
            .any(|application| application.auto_run_after),
        plan: if let Some(archive_extractor) = archive_extractor {
            prepare_main_game_plan_for_sequence(
                launchbox_root,
                game,
                &applications,
                mounts,
                configuration,
                context,
                path_resolver,
                archive_extractor,
            )?
        } else {
            build_launch_plan_with_mounts_context_and_resolver(
                launchbox_root,
                game,
                mounts,
                configuration,
                context,
                path_resolver,
            )?
        },
    });

    for application in applications
        .iter()
        .copied()
        .filter(|application| application.auto_run_after)
    {
        steps.push(LaunchStep {
            role: LaunchStepRole::AutomaticAfter,
            wait_for_exit: false,
            plan: if let Some(archive_extractor) = archive_extractor {
                prepare_additional_application_plan_with_mounts_context_and_resolver(
                    launchbox_root,
                    game,
                    application,
                    mounts,
                    configuration,
                    context,
                    path_resolver,
                    archive_extractor,
                )?
            } else {
                build_additional_application_plan_with_mounts_context_and_resolver(
                    launchbox_root,
                    game,
                    application,
                    mounts,
                    configuration,
                    context,
                    path_resolver,
                )?
            },
        });
    }

    Ok(LaunchSequence {
        game_id: game.id.clone(),
        game_title: game.title.clone(),
        steps,
    })
}

pub fn build_selected_additional_application_sequence_with_context_and_resolver(
    launchbox_root: &Path,
    game: &Game,
    application: &AdditionalApplication,
    configuration: Option<&EmulatorConfiguration>,
    context: &LaunchContext,
    path_resolver: &dyn LaunchPathResolver,
) -> Result<LaunchSequence, LaunchPlanError> {
    build_selected_additional_application_sequence_with_mounts_context_and_resolver(
        launchbox_root,
        game,
        application,
        &[],
        configuration,
        context,
        path_resolver,
    )
}

pub fn build_selected_additional_application_sequence_with_mounts_context_and_resolver(
    launchbox_root: &Path,
    game: &Game,
    application: &AdditionalApplication,
    mounts: &[Mount],
    configuration: Option<&EmulatorConfiguration>,
    context: &LaunchContext,
    path_resolver: &dyn LaunchPathResolver,
) -> Result<LaunchSequence, LaunchPlanError> {
    Ok(LaunchSequence {
        game_id: game.id.clone(),
        game_title: game.title.clone(),
        steps: vec![LaunchStep {
            role: LaunchStepRole::SelectedAdditionalApplication,
            wait_for_exit: false,
            plan: build_additional_application_plan_with_mounts_context_and_resolver(
                launchbox_root,
                game,
                application,
                mounts,
                configuration,
                context,
                path_resolver,
            )?,
        }],
    })
}

pub fn prepare_selected_additional_application_sequence_with_context_and_resolver(
    launchbox_root: &Path,
    game: &Game,
    application: &AdditionalApplication,
    configuration: Option<&EmulatorConfiguration>,
    context: &LaunchContext,
    path_resolver: &dyn LaunchPathResolver,
    archive_extractor: &ArchiveExtractor,
) -> Result<LaunchSequence, LaunchPlanError> {
    prepare_selected_additional_application_sequence_with_mounts_context_and_resolver(
        launchbox_root,
        game,
        application,
        &[],
        configuration,
        context,
        path_resolver,
        archive_extractor,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn prepare_selected_additional_application_sequence_with_mounts_context_and_resolver(
    launchbox_root: &Path,
    game: &Game,
    application: &AdditionalApplication,
    mounts: &[Mount],
    configuration: Option<&EmulatorConfiguration>,
    context: &LaunchContext,
    path_resolver: &dyn LaunchPathResolver,
    archive_extractor: &ArchiveExtractor,
) -> Result<LaunchSequence, LaunchPlanError> {
    Ok(LaunchSequence {
        game_id: game.id.clone(),
        game_title: game.title.clone(),
        steps: vec![LaunchStep {
            role: LaunchStepRole::SelectedAdditionalApplication,
            wait_for_exit: false,
            plan: prepare_additional_application_plan_with_mounts_context_and_resolver(
                launchbox_root,
                game,
                application,
                mounts,
                configuration,
                context,
                path_resolver,
                archive_extractor,
            )?,
        }],
    })
}

fn select_emulator<'a>(
    game: &Game,
    configuration: Option<&'a EmulatorConfiguration>,
) -> Result<Option<(&'a Emulator, Option<&'a EmulatorPlatform>)>, LaunchPlanError> {
    if game
        .emulator_id
        .as_deref()
        .is_some_and(is_unassigned_emulator_id)
    {
        return Ok(None);
    }
    let Some(configuration) = configuration else {
        return if let Some(emulator_id) = &game.emulator_id {
            Err(LaunchPlanError::MissingEmulatorConfiguration {
                game_id: game.id.clone(),
                emulator_id: emulator_id.clone(),
            })
        } else {
            Ok(None)
        };
    };

    let emulator_id = if let Some(emulator_id) = game.emulator_id.as_deref() {
        Some(emulator_id)
    } else {
        let defaults = configuration
            .platforms
            .iter()
            .filter(|mapping| mapping.platform == game.platform && mapping.default)
            .collect::<Vec<_>>();
        match defaults.as_slice() {
            [] => None,
            [mapping] => Some(mapping.emulator_id.as_str()),
            _ => {
                return Err(LaunchPlanError::AmbiguousDefaultEmulator {
                    platform: game.platform.clone(),
                    count: defaults.len(),
                });
            }
        }
    };
    let Some(emulator_id) = emulator_id else {
        return Ok(None);
    };
    let emulator = configuration
        .emulators
        .iter()
        .find(|emulator| emulator.id == emulator_id)
        .ok_or_else(|| LaunchPlanError::EmulatorNotFound {
            emulator_id: emulator_id.to_string(),
            game_id: game.id.clone(),
        })?;
    let mappings = configuration
        .platforms
        .iter()
        .filter(|mapping| mapping.emulator_id == emulator_id && mapping.platform == game.platform)
        .collect::<Vec<_>>();
    let mapping = match mappings.as_slice() {
        [] => None,
        [mapping] => Some(*mapping),
        _ => {
            return Err(LaunchPlanError::AmbiguousEmulatorPlatform {
                emulator_id: emulator_id.to_string(),
                platform: game.platform.clone(),
                count: mappings.len(),
            });
        }
    };
    Ok(Some((emulator, mapping)))
}

/// Resolves the emulator and optional platform mapping that own a game's
/// runtime integration.
///
/// Save discovery and other emulator adapters use the exact same explicit,
/// default, unassigned, missing, and ambiguity rules as launch planning. This
/// keeps adapter ownership out of the Qt and storage layers.
pub fn select_emulator_for_game<'a>(
    game: &Game,
    configuration: Option<&'a EmulatorConfiguration>,
) -> Result<Option<(&'a Emulator, Option<&'a EmulatorPlatform>)>, LaunchPlanError> {
    select_emulator(game, configuration)
}

fn is_archive_path(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "7z" | "rar" | "zip"
            )
        })
}

struct LaunchVariables<'a> {
    rom_file: &'a OsStr,
    rom_location: PathBuf,
    platform: &'a str,
    game_id: &'a str,
    frontend_executable: Option<&'a Path>,
}

fn append_command_line(
    arguments: &mut Vec<OsString>,
    command_line: Option<&str>,
    variables: &LaunchVariables<'_>,
) -> Result<bool, LaunchPlanError> {
    let mut rom_file_was_explicit = false;
    if let Some(command_line) = command_line.filter(|value| !value.trim().is_empty()) {
        for argument in split_windows_command_line(command_line) {
            let (argument, used_rom_file) = expand_launch_variables(&argument, variables)?;
            rom_file_was_explicit |= used_rom_file;
            arguments.push(OsString::from(argument));
        }
    }
    Ok(rom_file_was_explicit)
}

fn expand_launch_variables(
    argument: &str,
    variables: &LaunchVariables<'_>,
) -> Result<(String, bool), LaunchPlanError> {
    let mut expanded = argument.to_string();
    let mut used_rom_file = false;

    if contains_ascii_token(&expanded, "%romfile%") {
        let value = unicode_path_variable("%romfile%", variables.rom_file)?;
        (expanded, used_rom_file) = (replace_ascii_token(&expanded, "%romfile%", value), true);
    }
    if contains_ascii_token(&expanded, "%romlocation%") {
        let value = unicode_path_variable("%romlocation%", variables.rom_location.as_os_str())?;
        expanded = replace_ascii_token(&expanded, "%romlocation%", value);
    }
    expanded = replace_ascii_token(&expanded, "%platform%", variables.platform);
    expanded = replace_ascii_token(&expanded, "%gameid%", variables.game_id);
    if contains_ascii_token(&expanded, "%launchboxorbigboxexepath%") {
        let executable = variables
            .frontend_executable
            .ok_or(LaunchPlanError::MissingFrontendExecutableForVariable)?;
        let value = unicode_path_variable("%launchboxorbigboxexepath%", executable.as_os_str())?;
        expanded = replace_ascii_token(&expanded, "%launchboxorbigboxexepath%", value);
    }
    Ok((expanded, used_rom_file))
}

fn unicode_path_variable<'a>(
    variable: &'static str,
    value: &'a OsStr,
) -> Result<&'a str, LaunchPlanError> {
    value
        .to_str()
        .ok_or(LaunchPlanError::NonUnicodeLaunchVariable { variable })
}

fn contains_ascii_token(value: &str, token: &str) -> bool {
    let token = token.as_bytes();
    value
        .as_bytes()
        .windows(token.len())
        .any(|candidate| candidate.eq_ignore_ascii_case(token))
}

fn replace_ascii_token(value: &str, token: &str, replacement: &str) -> String {
    let mut output = String::with_capacity(value.len() + replacement.len());
    let mut cursor = 0;
    while cursor < value.len() {
        let remainder = &value[cursor..];
        if remainder.len() >= token.len()
            && remainder
                .as_bytes()
                .get(..token.len())
                .is_some_and(|candidate| candidate.eq_ignore_ascii_case(token.as_bytes()))
        {
            output.push_str(replacement);
            cursor += token.len();
        } else {
            let character = remainder
                .chars()
                .next()
                .expect("cursor remains on a character boundary");
            output.push(character);
            cursor += character.len_utf8();
        }
    }
    output
}

/// Parses persisted Windows command-line parameters into semantic arguments.
/// Backslashes immediately before quotes follow the Microsoft CRT rules.
pub fn split_windows_command_line(command_line: &str) -> Vec<String> {
    let characters = command_line.chars().collect::<Vec<_>>();
    let mut arguments = Vec::new();
    let mut cursor = 0;
    while cursor < characters.len() {
        while cursor < characters.len() && characters[cursor].is_whitespace() {
            cursor += 1;
        }
        if cursor == characters.len() {
            break;
        }
        let mut argument = String::new();
        let mut in_quotes = false;
        while cursor < characters.len() {
            if characters[cursor] == '\\' {
                let start = cursor;
                while cursor < characters.len() && characters[cursor] == '\\' {
                    cursor += 1;
                }
                let count = cursor - start;
                if cursor < characters.len() && characters[cursor] == '"' {
                    argument.extend(std::iter::repeat_n('\\', count / 2));
                    if count % 2 == 0 {
                        in_quotes = !in_quotes;
                    } else {
                        argument.push('"');
                    }
                    cursor += 1;
                } else {
                    argument.extend(std::iter::repeat_n('\\', count));
                }
            } else if characters[cursor] == '"' {
                if in_quotes && cursor + 1 < characters.len() && characters[cursor + 1] == '"' {
                    argument.push('"');
                    cursor += 2;
                } else {
                    in_quotes = !in_quotes;
                    cursor += 1;
                }
            } else if characters[cursor].is_whitespace() && !in_quotes {
                break;
            } else {
                argument.push(characters[cursor]);
                cursor += 1;
            }
        }
        arguments.push(argument);
        while cursor < characters.len() && characters[cursor].is_whitespace() {
            cursor += 1;
        }
    }
    arguments
}

pub trait ProcessLauncher {
    type Handle: LaunchProcess;

    fn launch(&self, request: &LaunchRequest) -> Result<Self::Handle, LaunchError>;
}

pub trait LaunchProcess: Send + 'static {
    fn id(&self) -> u32;
    fn wait(&mut self) -> std::io::Result<ExitStatus>;
    fn try_wait(&mut self) -> std::io::Result<Option<ExitStatus>>;
}

impl LaunchProcess for Child {
    fn id(&self) -> u32 {
        Child::id(self)
    }

    fn wait(&mut self) -> std::io::Result<ExitStatus> {
        Child::wait(self)
    }

    fn try_wait(&mut self) -> std::io::Result<Option<ExitStatus>> {
        Child::try_wait(self)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemProcessLauncher;

fn system_process_command(request: &LaunchRequest) -> (PathBuf, Vec<OsString>) {
    #[cfg(target_os = "linux")]
    if request
        .executable
        .file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|name| name.to_ascii_lowercase().ends_with(".appimage"))
    {
        let arguments = std::iter::once(request.executable.as_os_str().to_os_string())
            .chain(request.arguments.iter().cloned())
            .collect();
        return (PathBuf::from("appimage-run"), arguments);
    }
    (request.executable.clone(), request.arguments.clone())
}

impl ProcessLauncher for SystemProcessLauncher {
    type Handle = Child;

    fn launch(&self, request: &LaunchRequest) -> Result<Self::Handle, LaunchError> {
        let (executable, arguments) = system_process_command(request);
        let mut command = Command::new(executable);
        command.args(arguments);
        if let Some(directory) = &request.working_directory {
            command.current_dir(directory);
        }
        if request.hide_console {
            command.stdin(Stdio::null());
            command.stdout(Stdio::null());
            command.stderr(Stdio::null());
        }
        #[cfg(windows)]
        if request.hide_console {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            command.creation_flags(CREATE_NO_WINDOW);
        }
        command.spawn().map_err(|source| LaunchError::Spawn {
            executable: request.executable.clone(),
            source,
        })
    }
}

#[derive(Debug, Error)]
pub enum LaunchError {
    #[error("failed to launch {executable}: {source}")]
    Spawn {
        executable: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Clone, Debug)]
pub enum LaunchSequenceEvent {
    StepStarted {
        role: LaunchStepRole,
        target: LaunchTarget,
        executable: PathBuf,
        pid: u32,
        started_at: SystemTime,
    },
    StepExited {
        role: LaunchStepRole,
        target: LaunchTarget,
        status: ExitStatus,
    },
    BeforeWaitTimedOut {
        target: LaunchTarget,
        timeout: Duration,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaunchSequenceReport {
    pub game_id: String,
    pub game_title: String,
    pub primary_target: LaunchTarget,
    pub primary_executable: PathBuf,
    pub primary_pid: u32,
    pub primary_started_at: SystemTime,
    pub primary_runtime: Duration,
    pub primary_exit_success: bool,
    pub automatic_before_started: usize,
    pub automatic_after_started: usize,
    pub before_wait_timeouts: usize,
}

/// Executes a previously validated sequence without a command shell. A main
/// primary process is always observed through exit so callers can persist real
/// play-session duration. Non-gating automatic applications are handed to
/// small background reapers so this function does not leave zombies.
pub fn execute_launch_sequence(
    sequence: &LaunchSequence,
    mut on_event: impl FnMut(LaunchSequenceEvent),
) -> Result<LaunchSequenceReport, LaunchSequenceError> {
    execute_launch_sequence_with(
        sequence,
        &SystemProcessLauncher,
        AUTO_RUN_BEFORE_WAIT_TIMEOUT,
        &mut on_event,
    )
}

pub fn execute_launch_sequence_with<L>(
    sequence: &LaunchSequence,
    launcher: &L,
    before_wait_timeout: Duration,
    mut on_event: impl FnMut(LaunchSequenceEvent),
) -> Result<LaunchSequenceReport, LaunchSequenceError>
where
    L: ProcessLauncher,
{
    let primary_count = sequence
        .steps
        .iter()
        .filter(|step| step.role.is_primary())
        .count();
    if primary_count != 1 {
        return Err(LaunchSequenceError::InvalidPrimaryStepCount {
            count: primary_count,
        });
    }

    let mut primary = None;
    let mut automatic_before_started = 0usize;
    let mut automatic_after_started = 0usize;
    let mut before_wait_timeouts = 0usize;

    for step in &sequence.steps {
        let mut process = launcher.launch(&step.plan.request).map_err(|source| {
            LaunchSequenceError::StartStep {
                role: step.role,
                target: step.plan.target.clone(),
                source,
            }
        })?;
        let pid = process.id();
        let started_at = SystemTime::now();
        let runtime_started = Instant::now();
        on_event(LaunchSequenceEvent::StepStarted {
            role: step.role,
            target: step.plan.target.clone(),
            executable: step.plan.request.executable.clone(),
            pid,
            started_at,
        });

        match step.role {
            LaunchStepRole::AutomaticBefore => automatic_before_started += 1,
            LaunchStepRole::AutomaticAfter => automatic_after_started += 1,
            LaunchStepRole::MainGame | LaunchStepRole::SelectedAdditionalApplication => {}
        }

        if step.role == LaunchStepRole::AutomaticBefore && step.wait_for_exit {
            match wait_for_exit_until(&mut process, before_wait_timeout).map_err(|source| {
                LaunchSequenceError::WaitForStep {
                    role: step.role,
                    target: step.plan.target.clone(),
                    source,
                }
            })? {
                Some(status) => on_event(LaunchSequenceEvent::StepExited {
                    role: step.role,
                    target: step.plan.target.clone(),
                    status,
                }),
                None => {
                    before_wait_timeouts += 1;
                    on_event(LaunchSequenceEvent::BeforeWaitTimedOut {
                        target: step.plan.target.clone(),
                        timeout: before_wait_timeout,
                    });
                    reap_in_background(
                        process,
                        step.role,
                        step.plan.target.clone(),
                        step.plan.resource_leases.clone(),
                    );
                }
            }
        } else if step.wait_for_exit || step.role.is_primary() {
            let status = process
                .wait()
                .map_err(|source| LaunchSequenceError::WaitForStep {
                    role: step.role,
                    target: step.plan.target.clone(),
                    source,
                })?;
            on_event(LaunchSequenceEvent::StepExited {
                role: step.role,
                target: step.plan.target.clone(),
                status,
            });
            if step.role.is_primary() {
                primary = Some((
                    step.plan.target.clone(),
                    step.plan.request.executable.clone(),
                    pid,
                    started_at,
                    runtime_started.elapsed(),
                    status.success(),
                ));
            }
        } else {
            reap_in_background(
                process,
                step.role,
                step.plan.target.clone(),
                step.plan.resource_leases.clone(),
            );
        }
    }

    let (
        primary_target,
        primary_executable,
        primary_pid,
        primary_started_at,
        primary_runtime,
        primary_exit_success,
    ) = primary.expect("primary count was validated before spawning");
    Ok(LaunchSequenceReport {
        game_id: sequence.game_id.clone(),
        game_title: sequence.game_title.clone(),
        primary_target,
        primary_executable,
        primary_pid,
        primary_started_at,
        primary_runtime,
        primary_exit_success,
        automatic_before_started,
        automatic_after_started,
        before_wait_timeouts,
    })
}

fn wait_for_exit_until<P: LaunchProcess>(
    process: &mut P,
    timeout: Duration,
) -> std::io::Result<Option<ExitStatus>> {
    let started = Instant::now();
    loop {
        if let Some(status) = process.try_wait()? {
            return Ok(Some(status));
        }
        if started.elapsed() >= timeout {
            return Ok(None);
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn reap_in_background<P: LaunchProcess>(
    mut process: P,
    role: LaunchStepRole,
    target: LaunchTarget,
    resource_leases: Vec<LaunchResourceLease>,
) {
    let _ = std::thread::Builder::new()
        .name("launchbox-process-reaper".to_string())
        .spawn(move || {
            let _resource_leases = resource_leases;
            if let Err(error) = process.wait() {
                eprintln!("Could not reap {role:?} process for {target:?}: {error}");
            }
        });
}

#[derive(Debug, Error)]
pub enum LaunchSequenceError {
    #[error("launch sequence has {count} primary steps; exactly one is required")]
    InvalidPrimaryStepCount { count: usize },
    #[error("could not start {role:?} target {target:?}: {source}")]
    StartStep {
        role: LaunchStepRole,
        target: LaunchTarget,
        #[source]
        source: LaunchError,
    },
    #[error("could not wait for {role:?} target {target:?}: {source}")]
    WaitForStep {
        role: LaunchStepRole,
        target: LaunchTarget,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum LaunchPlanError {
    #[error(
        "additional application {application_id} belongs to game {actual_game_id}, not {expected_game_id}"
    )]
    AdditionalApplicationGameMismatch {
        application_id: String,
        expected_game_id: String,
        actual_game_id: String,
    },
    #[error("additional application {application_id} has no application path")]
    MissingAdditionalApplicationPath { application_id: String },
    #[error("game {game_id} has no application path")]
    MissingGameApplicationPath { game_id: String },
    #[error("game {game_id} application path cannot be used on this host: {source}")]
    GamePath {
        game_id: String,
        #[source]
        source: LaunchPathError,
    },
    #[error(
        "game {game_id} selects emulator {emulator_id}, but no emulator configuration is loaded"
    )]
    MissingEmulatorConfiguration {
        game_id: String,
        emulator_id: String,
    },
    #[error("game {game_id} selects missing emulator {emulator_id}")]
    EmulatorNotFound {
        game_id: String,
        emulator_id: String,
    },
    #[error("platform {platform} has {count} default emulator mappings")]
    AmbiguousDefaultEmulator { platform: String, count: usize },
    #[error("emulator {emulator_id} has {count} mappings for platform {platform}")]
    AmbiguousEmulatorPlatform {
        emulator_id: String,
        platform: String,
        count: usize,
    },
    #[error("emulator {emulator_id} has no application path")]
    MissingEmulatorApplicationPath { emulator_id: String },
    #[error("emulator {emulator_id} application path cannot be used on this host: {source}")]
    EmulatorPath {
        emulator_id: String,
        #[source]
        source: LaunchPathError,
    },
    #[error("game {game_id} path {path} has no file name")]
    MissingGameFileName { game_id: String, path: PathBuf },
    #[error("game {game_id} path {path} has no ROM directory")]
    MissingRomLocation { game_id: String, path: PathBuf },
    #[error("%launchboxorbigboxexepath% requires a running frontend executable")]
    MissingFrontendExecutableForVariable,
    #[error("{variable} cannot represent a non-Unicode host path")]
    NonUnicodeLaunchVariable { variable: &'static str },
    #[error("{mode} launch is not implemented for game {game_id}")]
    UnsupportedMode { game_id: String, mode: &'static str },
    #[error("game {game_id} enables both DOSBox and ScummVM")]
    ConflictingModes { game_id: String },
    #[error("could not build DOSBox launch for game {game_id}: {source}")]
    DosBoxPlanning {
        game_id: String,
        #[source]
        source: Box<DosBoxPlanError>,
    },
    #[error("could not build ScummVM launch for game {game_id}: {source}")]
    ScummVmPlanning {
        game_id: String,
        #[source]
        source: Box<ScummVmPlanError>,
    },
    #[error("archive extraction is required but not implemented for game {game_id}: {path}")]
    UnsupportedArchiveExtraction { game_id: String, path: PathBuf },
    #[error("M3U playlist preparation requires the prepared launch API for game {game_id}")]
    UnsupportedM3uPreparation { game_id: String },
    #[error("could not prepare archive for game {game_id} at {path}: {source}")]
    ArchiveExtraction {
        game_id: String,
        path: PathBuf,
        #[source]
        source: Box<ArchiveExtractionError>,
    },
    #[error("could not prepare an M3U playlist for game {game_id}: {source}")]
    M3uPreparation {
        game_id: String,
        #[source]
        source: Box<M3uPreparationError>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game() -> Game {
        Game {
            id: "game-id".into(),
            title: "Game Title".into(),
            platform: "Console".into(),
            application_path: r"Games\Game Title\game.rom".into(),
            command_line: Some(r#"--region "North America""#.into()),
            emulator_id: Some("emulator-id".into()),
            ..Game::default()
        }
    }

    fn configuration() -> EmulatorConfiguration {
        EmulatorConfiguration {
            source_path: PathBuf::from("Data/Emulators.xml"),
            emulators: vec![Emulator {
                id: "emulator-id".into(),
                title: "Fixture Emulator".into(),
                application_path: r"Emulators\Fixture\emulator.exe".into(),
                command_line: Some("--global ignored".into()),
                ..Emulator::default()
            }],
            platforms: vec![EmulatorPlatform {
                emulator_id: "emulator-id".into(),
                platform: "Console".into(),
                command_line: Some(r#"--platform "Fixture Console""#.into()),
                default: true,
                ..EmulatorPlatform::default()
            }],
        }
    }

    fn additional_application(id: &str, priority: i32) -> AdditionalApplication {
        AdditionalApplication {
            id: id.into(),
            game_id: "game-id".into(),
            name: format!("Additional {id}"),
            application_path: format!(r"Extras\{id}.exe"),
            command_line: Some(format!(r#"--app "{id} with spaces""#)),
            priority,
            ..AdditionalApplication::default()
        }
    }

    #[test]
    fn request_keeps_arguments_separate_from_shell_quoting() {
        let request = LaunchRequest::new("emulator")
            .arg("--fullscreen")
            .arg("Games/A title with spaces.rom");
        assert_eq!(
            request.arguments,
            ["--fullscreen", "Games/A title with spaces.rom"].map(OsString::from)
        );
    }

    #[test]
    fn resolves_emulator_mapping_game_parameters_and_portable_paths() {
        let plan = build_launch_plan(Path::new("/launchbox"), &game(), Some(&configuration()))
            .expect("build emulator plan");
        assert_eq!(
            plan.kind,
            LaunchKind::Emulator {
                id: "emulator-id".into(),
                title: "Fixture Emulator".into()
            }
        );
        assert_eq!(
            plan.request.executable,
            PathBuf::from("/launchbox/Emulators/Fixture/emulator.exe")
        );
        assert_eq!(
            plan.request.arguments,
            [
                "--platform",
                "Fixture Console",
                "--region",
                "North America",
                "/launchbox/Games/Game Title/game.rom",
            ]
            .map(OsString::from)
        );
        assert_eq!(
            plan.request.working_directory,
            Some(PathBuf::from("/launchbox/Emulators/Fixture"))
        );
    }

    #[test]
    fn falls_back_to_default_emulator_or_direct_process() {
        let mut default_game = game();
        default_game.emulator_id = None;
        let emulated = build_launch_plan(
            Path::new("/launchbox"),
            &default_game,
            Some(&configuration()),
        )
        .expect("use default emulator");
        assert!(matches!(emulated.kind, LaunchKind::Emulator { .. }));

        let direct = build_launch_plan(Path::new("/launchbox"), &default_game, None)
            .expect("build direct plan");
        assert_eq!(direct.kind, LaunchKind::Direct);
        assert_eq!(
            direct.request.executable,
            PathBuf::from("/launchbox/Games/Game Title/game.rom")
        );
        assert_eq!(
            direct.request.arguments,
            ["--region", "North America"].map(OsString::from)
        );
    }

    #[test]
    fn explicit_unassigned_emulator_bypasses_the_platform_default() {
        let mut direct_game = game();
        direct_game.emulator_id = Some(lb_domain::UNASSIGNED_EMULATOR_ID.into());
        let plan = build_launch_plan(
            Path::new("/launchbox"),
            &direct_game,
            Some(&configuration()),
        )
        .expect("explicitly unassigned game should launch directly");
        assert_eq!(plan.kind, LaunchKind::Direct);
        assert_eq!(
            plan.request.executable,
            PathBuf::from("/launchbox/Games/Game Title/game.rom")
        );
        assert_eq!(
            plan.request.arguments,
            ["--region", "North America"].map(OsString::from)
        );
    }

    #[test]
    fn additional_application_keeps_parent_context_but_uses_its_own_target() {
        let application = additional_application("manual", 0);
        let plan = build_additional_application_plan_with_context_and_resolver(
            Path::new("/launchbox"),
            &game(),
            &application,
            Some(&configuration()),
            &LaunchContext::default(),
            &HostPathResolver::default(),
        )
        .expect("build direct additional application");
        assert_eq!(plan.game_id, "game-id");
        assert_eq!(plan.game_title, "Game Title");
        assert_eq!(
            plan.target,
            LaunchTarget::AdditionalApplication {
                application_id: "manual".into(),
                application_name: "Additional manual".into(),
            }
        );
        assert_eq!(plan.kind, LaunchKind::Direct);
        assert_eq!(
            plan.request.executable,
            PathBuf::from("/launchbox/Extras/manual.exe")
        );
        assert_eq!(
            plan.request.arguments,
            ["--app", "manual with spaces"].map(OsString::from)
        );
    }

    #[test]
    fn additional_application_can_select_its_own_emulator() {
        let mut application = additional_application("disc-two", 0);
        application.use_emulator = true;
        application.emulator_id = Some("emulator-id".into());
        let plan = build_additional_application_plan_with_context_and_resolver(
            Path::new("/launchbox"),
            &game(),
            &application,
            Some(&configuration()),
            &LaunchContext::default(),
            &HostPathResolver::default(),
        )
        .expect("build emulated additional application");
        assert!(matches!(plan.kind, LaunchKind::Emulator { .. }));
        assert_eq!(
            plan.request.arguments,
            [
                "--platform",
                "Fixture Console",
                "--app",
                "disc-two with spaces",
                "/launchbox/Extras/disc-two.exe",
            ]
            .map(OsString::from)
        );
    }

    #[test]
    fn automatic_launch_sequence_is_priority_ordered_and_waits_for_after_apps() {
        let mut before_later = additional_application("before-later", 20);
        before_later.auto_run_before = true;
        let mut before_first = additional_application("before-first", 10);
        before_first.auto_run_before = true;
        before_first.wait_for_exit = true;
        let mut after = additional_application("after", 5);
        after.auto_run_after = true;
        let unrelated = AdditionalApplication {
            game_id: "other-game".into(),
            ..additional_application("unrelated", 0)
        };
        let applications = [before_later, after, unrelated, before_first];

        let sequence = build_game_launch_sequence_with_context_and_resolver(
            Path::new("/launchbox"),
            &game(),
            &applications,
            Some(&configuration()),
            &LaunchContext::default(),
            &HostPathResolver::default(),
        )
        .expect("build complete sequence");

        assert_eq!(
            sequence
                .steps
                .iter()
                .map(|step| step.role)
                .collect::<Vec<_>>(),
            [
                LaunchStepRole::AutomaticBefore,
                LaunchStepRole::AutomaticBefore,
                LaunchStepRole::MainGame,
                LaunchStepRole::AutomaticAfter,
            ]
        );
        assert_eq!(
            sequence
                .steps
                .iter()
                .filter_map(|step| match &step.plan.target {
                    LaunchTarget::AdditionalApplication { application_id, .. } => {
                        Some(application_id.as_str())
                    }
                    LaunchTarget::MainGame => None,
                })
                .collect::<Vec<_>>(),
            ["before-first", "before-later", "after"]
        );
        assert!(sequence.steps[0].wait_for_exit);
        assert!(!sequence.steps[1].wait_for_exit);
        assert!(sequence.steps[2].wait_for_exit);
        assert!(!sequence.steps[3].wait_for_exit);
    }

    #[test]
    fn prepared_sequence_replaces_the_rom_with_a_leased_m3u_playlist() {
        let root = tempfile::tempdir().expect("create library root");
        let mut game = game();
        game.application_path = r"Games\Multi Disc (Disc 1).chd".into();
        game.command_line = None;
        let mut configuration = configuration();
        configuration.platforms[0].m3u_disc_load_enabled = true;
        configuration.platforms[0].command_line = Some("--playlist %romfile%".into());

        let mut first = additional_application("disc-1", 1);
        first.application_path = game.application_path.clone();
        first.use_emulator = true;
        first.emulator_id = Some("emulator-id".into());
        first.disc = Some(1);
        let mut second = additional_application("disc-2", 2);
        second.application_path = r"Games\Multi Disc (Disc 2).chd".into();
        second.use_emulator = true;
        second.emulator_id = Some("emulator-id".into());
        second.disc = Some(2);
        let manual = additional_application("manual", 0);

        let sequence = prepare_game_launch_sequence_with_context_and_resolver(
            root.path(),
            &game,
            [&second, &manual, &first],
            Some(&configuration),
            &LaunchContext::default(),
            &HostPathResolver::default(),
            &ArchiveExtractor::default(),
        )
        .expect("prepare multi-disc sequence");
        let main = sequence
            .steps
            .iter()
            .find(|step| step.role == LaunchStepRole::MainGame)
            .expect("main step");
        assert_eq!(main.plan.request.arguments[0], "--playlist");
        let playlist_path = PathBuf::from(&main.plan.request.arguments[1]);
        assert_eq!(
            std::fs::read_to_string(&playlist_path).expect("read generated playlist"),
            format!(
                "{}\n{}\n",
                root.path().join("Games/Multi Disc (Disc 1).chd").display(),
                root.path().join("Games/Multi Disc (Disc 2).chd").display(),
            )
        );
        assert_eq!(main.plan.resource_leases.len(), 1);

        drop(sequence);
        assert!(!playlist_path.exists());
    }

    #[test]
    fn pure_sequence_planner_refuses_to_ignore_an_enabled_m3u_mapping() {
        let mut game = game();
        game.application_path = r"Games\Multi Disc (Disc 1).chd".into();
        let mut configuration = configuration();
        configuration.platforms[0].m3u_disc_load_enabled = true;
        let mut first = additional_application("disc-1", 1);
        first.application_path = game.application_path.clone();
        first.disc = Some(1);

        assert_eq!(
            build_game_launch_sequence_with_context_and_resolver(
                Path::new("/launchbox"),
                &game,
                [&first],
                Some(&configuration),
                &LaunchContext::default(),
                &HostPathResolver::default(),
            ),
            Err(LaunchPlanError::UnsupportedM3uPreparation {
                game_id: "game-id".into(),
            })
        );
    }

    #[test]
    fn invalid_automatic_application_prevents_the_whole_sequence_plan() {
        let mut application = additional_application("broken-before", 0);
        application.auto_run_before = true;
        application.application_path.clear();
        assert_eq!(
            build_game_launch_sequence_with_context_and_resolver(
                Path::new("/launchbox"),
                &game(),
                [&application],
                Some(&configuration()),
                &LaunchContext::default(),
                &HostPathResolver::default(),
            ),
            Err(LaunchPlanError::MissingAdditionalApplicationPath {
                application_id: "broken-before".into(),
            })
        );
    }

    #[cfg(unix)]
    #[test]
    fn sequence_executor_runs_waited_before_main_and_after_in_order() {
        let log = std::env::temp_dir().join(format!(
            "launchbox-sequence-order-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        let _ = std::fs::remove_file(&log);
        let append = |value: &str| {
            LaunchRequest::new("/bin/sh")
                .arg("-c")
                .arg(format!("printf '%s\\n' {value} >> {}", log.display()))
        };
        let step = |role, wait_for_exit, name: &str| LaunchStep {
            role,
            wait_for_exit,
            plan: LaunchPlan {
                game_id: "game-id".into(),
                game_title: "Game Title".into(),
                target: if role == LaunchStepRole::MainGame {
                    LaunchTarget::MainGame
                } else {
                    LaunchTarget::AdditionalApplication {
                        application_id: name.into(),
                        application_name: name.into(),
                    }
                },
                kind: LaunchKind::Direct,
                request: append(name),
                resource_leases: Vec::new(),
            },
        };
        let sequence = LaunchSequence {
            game_id: "game-id".into(),
            game_title: "Game Title".into(),
            steps: vec![
                step(LaunchStepRole::AutomaticBefore, true, "before"),
                step(LaunchStepRole::MainGame, true, "main"),
                step(LaunchStepRole::AutomaticAfter, false, "after"),
            ],
        };
        let mut events = Vec::new();
        let report = execute_launch_sequence_with(
            &sequence,
            &SystemProcessLauncher,
            Duration::from_millis(100),
            |event| events.push(event),
        )
        .expect("execute sequence");
        assert_eq!(report.automatic_before_started, 1);
        assert_eq!(report.automatic_after_started, 1);
        assert_eq!(report.before_wait_timeouts, 0);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, LaunchSequenceEvent::StepStarted { .. }))
                .count(),
            3
        );

        let started = Instant::now();
        let contents = loop {
            let contents = std::fs::read_to_string(&log).unwrap_or_default();
            if contents.lines().count() == 3 {
                break contents;
            }
            assert!(
                started.elapsed() < Duration::from_secs(1),
                "after-app did not finish: {contents:?}"
            );
            std::thread::sleep(Duration::from_millis(10));
        };
        assert_eq!(
            contents.lines().collect::<Vec<_>>(),
            ["before", "main", "after"]
        );
        std::fs::remove_file(log).expect("remove sequence log");
    }

    #[cfg(unix)]
    #[test]
    fn before_wait_timeout_does_not_prevent_the_main_process() {
        let before = LaunchStep {
            role: LaunchStepRole::AutomaticBefore,
            wait_for_exit: true,
            plan: LaunchPlan {
                game_id: "game-id".into(),
                game_title: "Game Title".into(),
                target: LaunchTarget::AdditionalApplication {
                    application_id: "slow-before".into(),
                    application_name: "Slow Before".into(),
                },
                kind: LaunchKind::Direct,
                request: LaunchRequest::new("/bin/sh").arg("-c").arg("sleep 0.05"),
                resource_leases: Vec::new(),
            },
        };
        let main = LaunchStep {
            role: LaunchStepRole::MainGame,
            wait_for_exit: false,
            plan: LaunchPlan {
                game_id: "game-id".into(),
                game_title: "Game Title".into(),
                target: LaunchTarget::MainGame,
                kind: LaunchKind::Direct,
                request: LaunchRequest::new("/bin/sh").arg("-c").arg("exit 0"),
                resource_leases: Vec::new(),
            },
        };
        let sequence = LaunchSequence {
            game_id: "game-id".into(),
            game_title: "Game Title".into(),
            steps: vec![before, main],
        };
        let mut timed_out = false;
        let report = execute_launch_sequence_with(
            &sequence,
            &SystemProcessLauncher,
            Duration::from_millis(1),
            |event| timed_out |= matches!(event, LaunchSequenceEvent::BeforeWaitTimedOut { .. }),
        )
        .expect("timeout is non-fatal");
        assert!(timed_out);
        assert_eq!(report.before_wait_timeouts, 1);
        assert_ne!(report.primary_pid, 0);
    }

    #[cfg(unix)]
    #[test]
    fn primary_exit_is_observed_before_temporary_resources_are_released() {
        let lease = crate::archive::temporary_launch_resource_for_test();
        let resource_path = lease.path().to_path_buf();
        std::fs::write(resource_path.join("rom.bin"), b"fixture")
            .expect("write temporary ROM fixture");
        let sequence = LaunchSequence {
            game_id: "game-id".into(),
            game_title: "Game Title".into(),
            steps: vec![LaunchStep {
                role: LaunchStepRole::MainGame,
                wait_for_exit: false,
                plan: LaunchPlan {
                    game_id: "game-id".into(),
                    game_title: "Game Title".into(),
                    target: LaunchTarget::MainGame,
                    kind: LaunchKind::Direct,
                    request: LaunchRequest::new("/bin/sh").arg("-c").arg("sleep 0.15"),
                    resource_leases: vec![lease],
                },
            }],
        };

        let started = Instant::now();
        let report =
            execute_launch_sequence(&sequence, |_| {}).expect("launch observed primary process");
        assert!(report.primary_exit_success);
        assert!(report.primary_runtime >= Duration::from_millis(100));
        assert!(started.elapsed() >= Duration::from_millis(100));
        drop(sequence);
        assert!(
            !resource_path.exists(),
            "the completed primary session retained its temporary resource"
        );
    }

    #[test]
    fn filename_only_and_no_space_modes_are_preserved_semantically() {
        let mut configuration = configuration();
        configuration.emulators[0].file_name_without_extension_and_path = true;
        configuration.emulators[0].no_space = true;
        configuration.platforms[0].command_line = Some("--load=".into());
        let mut game = game();
        game.command_line = None;
        let plan = build_launch_plan(Path::new("/launchbox"), &game, Some(&configuration))
            .expect("build no-space plan");
        assert_eq!(plan.request.arguments, ["--load=game"].map(OsString::from));
    }

    #[test]
    fn expands_launchbox_variables_without_losing_argument_boundaries() {
        let mut configuration = configuration();
        configuration.platforms[0].command_line = Some(
            r#"--rom %ROMFILE% --dir=%romlocation% --platform "%platform%" --id=%gameid% --frontend %launchboxorbigboxexepath%"#
                .into(),
        );
        let mut game = game();
        game.command_line = None;
        let context = LaunchContext {
            frontend_executable: Some(PathBuf::from("/opt/Launch Box/launchbox")),
        };
        let plan = build_launch_plan_with_context(
            Path::new("/launchbox"),
            &game,
            Some(&configuration),
            &context,
        )
        .expect("expand command-line variables");
        assert_eq!(
            plan.request.arguments,
            [
                "--rom",
                "/launchbox/Games/Game Title/game.rom",
                "--dir=/launchbox/Games/Game Title",
                "--platform",
                "Console",
                "--id=game-id",
                "--frontend",
                "/opt/Launch Box/launchbox",
            ]
            .map(OsString::from)
        );
    }

    #[test]
    fn embedded_rom_variable_uses_filename_only_and_is_not_appended_twice() {
        let mut configuration = configuration();
        configuration.emulators[0].file_name_without_extension_and_path = true;
        configuration.platforms[0].command_line = Some("--mount=%romfile%:cass3".into());
        let mut game = game();
        game.application_path = r"Games\Arcade\game.rom".into();
        game.command_line = None;
        let plan = build_launch_plan(Path::new("/launchbox"), &game, Some(&configuration))
            .expect("expand embedded ROM variable");
        assert_eq!(
            plan.request.arguments,
            ["--mount=game:cass3"].map(OsString::from)
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn rom_location_uses_the_same_explicit_windows_drive_mapping_as_the_rom() {
        let mut configuration = configuration();
        configuration.emulators[0].file_name_without_extension_and_path = true;
        configuration.platforms[0].command_line = Some("-rompath %romlocation%".into());
        let mut game = game();
        game.application_path = r"C:\Games\Arcade\game.zip".into();
        game.command_line = None;
        let resolver = HostPathResolver::default()
            .with_windows_drive_mapping('C', "/mnt/windows")
            .expect("valid drive mapping");
        let plan = build_launch_plan_with_context_and_resolver(
            Path::new("/launchbox"),
            &game,
            Some(&configuration),
            &LaunchContext::default(),
            &resolver,
        )
        .expect("expand mapped Windows ROM location");
        assert_eq!(
            plan.request.arguments,
            ["-rompath", "/mnt/windows/Games/Arcade", "game"].map(OsString::from)
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn launch_plan_rejects_an_unmapped_foreign_windows_path() {
        let mut game = game();
        game.application_path = r"D:\Games\game.rom".into();
        assert_eq!(
            build_launch_plan(Path::new("/launchbox"), &game, Some(&configuration())),
            Err(LaunchPlanError::GamePath {
                game_id: "game-id".into(),
                source: LaunchPathError::UnmappedWindowsDrive { drive: 'D' },
            })
        );
    }

    #[test]
    fn frontend_variable_requires_explicit_runtime_context() {
        let mut configuration = configuration();
        configuration.platforms[0].command_line =
            Some("--frontend=%launchboxorbigboxexepath%".into());
        assert_eq!(
            build_launch_plan(Path::new("/launchbox"), &game(), Some(&configuration)),
            Err(LaunchPlanError::MissingFrontendExecutableForVariable)
        );
    }

    #[test]
    fn parses_windows_quotes_backslashes_and_empty_arguments() {
        assert_eq!(
            split_windows_command_line(r#"--one "two words" "" C:\Games\file.rom"#),
            ["--one", "two words", "", r"C:\Games\file.rom"]
        );
        assert_eq!(
            split_windows_command_line(r#"--name "say ""hello""""#),
            ["--name", "say \"hello\""]
        );
    }

    #[test]
    fn plans_dosbox_but_defers_archive_extraction_to_the_prepared_api() {
        let mut dos_game = game();
        dos_game.use_dos_box = true;
        dos_game.root_folder = Some("Games".into());
        assert_eq!(
            build_launch_plan(Path::new("/launchbox"), &dos_game, Some(&configuration()))
                .expect("DOSBox lifecycle is implemented")
                .kind,
            LaunchKind::DosBox
        );

        let mut archive_game = game();
        archive_game.application_path = "Games/game.zip".into();
        let mut configuration = configuration();
        configuration.emulators[0].auto_extract = true;
        assert!(matches!(
            build_launch_plan(Path::new("/launchbox"), &archive_game, Some(&configuration)),
            Err(LaunchPlanError::UnsupportedArchiveExtraction { .. })
        ));
    }

    #[test]
    fn refuses_conflicting_legacy_launch_modes() {
        let mut conflicting = game();
        conflicting.use_dos_box = true;
        conflicting.use_scumm_vm = true;
        assert_eq!(
            build_launch_plan(
                Path::new("/launchbox"),
                &conflicting,
                Some(&configuration())
            ),
            Err(LaunchPlanError::ConflictingModes {
                game_id: "game-id".into(),
            })
        );
    }

    #[cfg(unix)]
    #[test]
    fn system_launcher_runs_without_a_shell() {
        let mut child = SystemProcessLauncher
            .launch(&LaunchRequest::new("/bin/sh").arg("-c").arg("exit 0"))
            .expect("launch test process");
        assert!(child.wait().expect("wait for test process").success());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn system_launcher_routes_appimages_through_packaged_appimage_run_without_a_shell() {
        let request = LaunchRequest::new("/launchbox/Emulators/PCSX2/pcsx2-qt.AppImage")
            .arg("-fullscreen")
            .arg("/launchbox/Games/game.iso");
        let (executable, arguments) = system_process_command(&request);
        assert_eq!(executable, Path::new("appimage-run"));
        assert_eq!(
            arguments,
            [
                OsString::from("/launchbox/Emulators/PCSX2/pcsx2-qt.AppImage"),
                OsString::from("-fullscreen"),
                OsString::from("/launchbox/Games/game.iso"),
            ]
        );
    }
}
