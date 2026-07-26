mod archive;
mod attract;
mod dosbox;
mod input;
mod m3u;
mod media;
mod model_viewer_state;
mod path;
mod path_settings;
mod screensaver;
mod scummvm;
mod ui_state;

pub use archive::{
    ArchiveCreationError, ArchiveExtractionError, ArchiveExtractor, LaunchResourceLease,
};
pub use attract::{BigBoxAttractModePolicy, BIG_BOX_ATTRACT_MODE_WHEEL_STEPS};
pub use dosbox::DosBoxPlanError;
pub use input::{
    qt_key_to_wpf_key, qt_key_to_wpf_key_with_modifiers, wpf_key_to_qt_portable_text,
    BigBoxInputAction, BigBoxInputEngine, BigBoxInputPolicy, ControllerBinding,
    GamepadBackendStatus, GamepadInputEvent, BIG_BOX_INPUT_ACTIONS,
};
pub use m3u::M3uPreparationError;
pub use media::{
    background_music_context_key, front_image_type_priorities,
    index_big_box_platform_marquee_media, index_big_box_startup_presentation, index_front_images,
    index_game_media, index_game_supplemental_media, launchbox_media_stem, region_priorities,
    BigBoxBackgroundMusicPolicy, BigBoxGameMarqueeMedia, BigBoxMarqueeCompatibilityMode,
    BigBoxMarqueePolicy, BigBoxMusicPolicy, BigBoxPlatformMarqueeIndex, BigBoxPlatformMarqueeMedia,
    BigBoxStartupPresentationIndex, BigBoxStartupPresentationPolicy, FrontImageIndex,
    FrontImageIndexReport, GameDetailsMediaPolicy, GameMediaIndex, GameMediaIndexReport,
    GameMediaItem, GameMediaKind, GameSupplementalMediaIndex, GameSupplementalMediaIndexReport,
    LaunchBoxMusicPolicy,
};
pub use model_viewer_state::{
    default_model_viewer_state_path, ModelRotationLock, ModelViewerState, ModelViewerStateError,
    MODEL_VIEWER_STATE_VERSION,
};
pub use path::{
    default_platform_folders, is_windows_absolute_path, navigation_document_file_name,
    platform_document_file_name, platform_storage_name, portable_storage_name,
    portable_stored_path, HostPathResolver, LaunchPathError, LaunchPathResolver, PlatformPathError,
};
pub use path_settings::{
    default_host_path_mappings_path, HostPathMappings, HostPathMappingsError, WindowsDriveMapping,
    WindowsUncMapping, HOST_PATH_MAPPINGS_VERSION,
};
pub use screensaver::{
    project_big_box_screensaver_candidates, select_big_box_screensaver_candidate,
    BigBoxScreensaverCandidate, BigBoxScreensaverMedia, BigBoxScreensaverPolicy,
    BigBoxScreensaverView,
};
pub use scummvm::ScummVmPlanError;
pub use ui_state::{
    default_launchbox_ui_state_path, default_list_view_column_widths, GameDetailsWindowState,
    LaunchBoxUiState, LaunchBoxUiStateError, LAUNCHBOX_UI_STATE_VERSION,
    MAX_GAME_DETAILS_PANE_WIDTH, MAX_GAME_DETAILS_WINDOW_HEIGHT, MAX_GAME_DETAILS_WINDOW_WIDTH,
    MAX_LIST_VIEW_COLUMN_WIDTH, MIN_GAME_DETAILS_PANE_WIDTH, MIN_GAME_DETAILS_WINDOW_HEIGHT,
    MIN_GAME_DETAILS_WINDOW_WIDTH, MIN_LIST_VIEW_COLUMN_WIDTH,
};

use lb_domain::{
    is_unassigned_emulator_id, AdditionalApplication, Emulator, EmulatorConfiguration,
    EmulatorPlatform, FrontendSettings, Game, Mount, UNASSIGNED_EMULATOR_ID,
};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::{Duration, Instant, SystemTime};
use thiserror::Error;

/// LaunchBox 3.1 documented a 30-second ceiling for an automatic before-app's
/// `WaitForExit` behavior. Keeping the timeout here makes it an orchestration
/// rule rather than a UI timer.
pub const AUTO_RUN_BEFORE_WAIT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LaunchStartupSettingsSource {
    GameOverride,
    EmulatorDefault,
    DirectGame,
}

impl LaunchStartupSettingsSource {
    pub const fn label(self) -> &'static str {
        match self {
            Self::GameOverride => "game override",
            Self::EmulatorDefault => "emulator default",
            Self::DirectGame => "game settings",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LaunchStartupPolicy {
    pub enabled: bool,
    pub load_delay: Duration,
    pub source: LaunchStartupSettingsSource,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LaunchShutdownPolicy {
    pub enabled: bool,
    pub source: LaunchStartupSettingsSource,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LaunchPausePolicy {
    pub enabled: bool,
    pub suspend_process: bool,
    pub forceful_activation: bool,
    pub source: LaunchStartupSettingsSource,
}

impl LaunchPausePolicy {
    pub const fn disabled() -> Self {
        Self {
            enabled: false,
            suspend_process: false,
            forceful_activation: false,
            source: LaunchStartupSettingsSource::DirectGame,
        }
    }
}

impl Default for LaunchPausePolicy {
    fn default() -> Self {
        Self::disabled()
    }
}

impl LaunchShutdownPolicy {
    pub const fn disabled() -> Self {
        Self {
            enabled: false,
            source: LaunchStartupSettingsSource::DirectGame,
        }
    }
}

impl Default for LaunchShutdownPolicy {
    fn default() -> Self {
        Self::disabled()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrontendLaunchScreenPolicy {
    pub enabled: bool,
    pub theme: String,
    pub minimum_startup_display: Duration,
    pub minimum_shutdown_display: Duration,
    pub hide_mouse_cursor: bool,
}

impl FrontendLaunchScreenPolicy {
    pub fn from_settings(settings: Option<&FrontendSettings>) -> Result<Self, LaunchPlanError> {
        let Some(settings) = settings else {
            return Ok(Self::default());
        };
        Ok(Self {
            enabled: frontend_bool_setting(settings, "UseStartupScreen", true)?,
            theme: settings
                .get("StartupTheme")
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("Default")
                .to_string(),
            minimum_startup_display: frontend_duration_setting(
                settings,
                "MinimumStartupScreenDisplayTime",
            )?,
            minimum_shutdown_display: frontend_duration_setting(
                settings,
                "MinimumShutdownScreenDisplayTime",
            )?,
            hide_mouse_cursor: frontend_bool_setting(
                settings,
                "HideMouseCursorOnStartupScreens",
                false,
            )?,
        })
    }
}

impl Default for FrontendLaunchScreenPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            theme: "Default".to_string(),
            minimum_startup_display: Duration::ZERO,
            minimum_shutdown_display: Duration::ZERO,
            hide_mouse_cursor: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrontendPauseScreenPolicy {
    pub enabled: bool,
    pub theme: String,
    pub mute_frontend_audio: bool,
    pub fade_frontend: bool,
}

impl FrontendPauseScreenPolicy {
    pub fn from_settings(settings: Option<&FrontendSettings>) -> Result<Self, LaunchPlanError> {
        let Some(settings) = settings else {
            return Ok(Self::default());
        };
        Ok(Self {
            enabled: frontend_bool_setting(settings, "UsePauseScreen", true)?,
            theme: settings
                .get("PauseTheme")
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("Default")
                .to_string(),
            mute_frontend_audio: frontend_bool_setting(settings, "PauseScreenMuting", true)?,
            fade_frontend: frontend_bool_setting(settings, "PauseScreenFading", true)?,
        })
    }
}

impl Default for FrontendPauseScreenPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            theme: "Default".to_string(),
            mute_frontend_audio: true,
            fade_frontend: true,
        }
    }
}

fn frontend_bool_setting(
    settings: &FrontendSettings,
    key: &'static str,
    default: bool,
) -> Result<bool, LaunchPlanError> {
    let Some(value) = settings.get(key) else {
        return Ok(default);
    };
    match value.trim() {
        value if value.eq_ignore_ascii_case("true") => Ok(true),
        value if value.eq_ignore_ascii_case("false") => Ok(false),
        _ => Err(LaunchPlanError::InvalidFrontendLaunchSetting {
            record: settings.record_name.clone(),
            key,
            value: value.to_string(),
            expected: "true or false",
        }),
    }
}

fn frontend_duration_setting(
    settings: &FrontendSettings,
    key: &'static str,
) -> Result<Duration, LaunchPlanError> {
    let Some(value) = settings.get(key) else {
        return Ok(Duration::ZERO);
    };
    let milliseconds =
        value
            .trim()
            .parse::<u64>()
            .map_err(|_| LaunchPlanError::InvalidFrontendLaunchSetting {
                record: settings.record_name.clone(),
                key,
                value: value.to_string(),
                expected: "a non-negative millisecond count",
            })?;
    Ok(Duration::from_millis(milliseconds))
}

impl LaunchStartupPolicy {
    pub const fn disabled() -> Self {
        Self {
            enabled: false,
            load_delay: Duration::ZERO,
            source: LaunchStartupSettingsSource::DirectGame,
        }
    }
}

impl Default for LaunchStartupPolicy {
    fn default() -> Self {
        Self::disabled()
    }
}

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
    pub startup: LaunchStartupPolicy,
    pub shutdown: LaunchShutdownPolicy,
    pub pause: LaunchPausePolicy,
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

    let main_plan = if let Some(archive_extractor) = archive_extractor {
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
    };
    let (startup, shutdown, pause) = launch_screen_policies(game, &main_plan, configuration)?;
    steps.push(LaunchStep {
        role: LaunchStepRole::MainGame,
        wait_for_exit: applications
            .iter()
            .any(|application| application.auto_run_after),
        plan: main_plan,
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
        startup,
        shutdown,
        pause,
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
    let plan = build_additional_application_plan_with_mounts_context_and_resolver(
        launchbox_root,
        game,
        application,
        mounts,
        configuration,
        context,
        path_resolver,
    )?;
    let (startup, shutdown, pause) = launch_screen_policies(game, &plan, configuration)?;
    Ok(LaunchSequence {
        game_id: game.id.clone(),
        game_title: game.title.clone(),
        startup,
        shutdown,
        pause,
        steps: vec![LaunchStep {
            role: LaunchStepRole::SelectedAdditionalApplication,
            wait_for_exit: false,
            plan,
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
    let plan = prepare_additional_application_plan_with_mounts_context_and_resolver(
        launchbox_root,
        game,
        application,
        mounts,
        configuration,
        context,
        path_resolver,
        archive_extractor,
    )?;
    let (startup, shutdown, pause) = launch_screen_policies(game, &plan, configuration)?;
    Ok(LaunchSequence {
        game_id: game.id.clone(),
        game_title: game.title.clone(),
        startup,
        shutdown,
        pause,
        steps: vec![LaunchStep {
            role: LaunchStepRole::SelectedAdditionalApplication,
            wait_for_exit: false,
            plan,
        }],
    })
}

fn launch_screen_policies(
    game: &Game,
    primary_plan: &LaunchPlan,
    configuration: Option<&EmulatorConfiguration>,
) -> Result<(LaunchStartupPolicy, LaunchShutdownPolicy, LaunchPausePolicy), LaunchPlanError> {
    let emulator = if let LaunchKind::Emulator { id, .. } = &primary_plan.kind {
        Some(
            configuration
                .and_then(|configuration| {
                    configuration
                        .emulators
                        .iter()
                        .find(|emulator| emulator.id.eq_ignore_ascii_case(id))
                })
                .ok_or_else(|| LaunchPlanError::EmulatorNotFound {
                    game_id: game.id.clone(),
                    emulator_id: id.clone(),
                })?,
        )
    } else {
        None
    };

    let (startup, shutdown) = if game.override_default_startup_screen_settings {
        let startup = LaunchStartupPolicy {
            enabled: game.use_startup_screen,
            load_delay: Duration::from_millis(u64::from(game.startup_load_delay)),
            source: LaunchStartupSettingsSource::GameOverride,
        };
        (
            startup,
            LaunchShutdownPolicy {
                enabled: startup.enabled && !game.disable_shutdown_screen,
                source: startup.source,
            },
        )
    } else if let Some(emulator) = emulator {
        let startup = LaunchStartupPolicy {
            enabled: emulator.use_startup_screen,
            load_delay: Duration::from_millis(emulator.startup_load_delay),
            source: LaunchStartupSettingsSource::EmulatorDefault,
        };
        (
            startup,
            LaunchShutdownPolicy {
                enabled: startup.enabled && !emulator.disable_shutdown_screen,
                source: startup.source,
            },
        )
    } else {
        let startup = LaunchStartupPolicy {
            enabled: game.use_startup_screen,
            load_delay: Duration::from_millis(u64::from(game.startup_load_delay)),
            source: LaunchStartupSettingsSource::DirectGame,
        };
        (
            startup,
            LaunchShutdownPolicy {
                enabled: startup.enabled && !game.disable_shutdown_screen,
                source: startup.source,
            },
        )
    };

    let pause = if game.override_default_pause_screen_settings {
        LaunchPausePolicy {
            enabled: game.use_pause_screen,
            suspend_process: game.suspend_process_on_pause,
            forceful_activation: game.forceful_pause_screen_activation,
            source: LaunchStartupSettingsSource::GameOverride,
        }
    } else if let Some(emulator) = emulator {
        LaunchPausePolicy {
            enabled: emulator.use_pause_screen,
            suspend_process: emulator.suspend_process_on_pause,
            forceful_activation: emulator.forceful_pause_screen_activation,
            source: LaunchStartupSettingsSource::EmulatorDefault,
        }
    } else {
        LaunchPausePolicy {
            enabled: game.use_pause_screen,
            suspend_process: game.suspend_process_on_pause,
            forceful_activation: game.forceful_pause_screen_activation,
            source: LaunchStartupSettingsSource::DirectGame,
        }
    };

    Ok((startup, shutdown, pause))
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
    fn suspend(&mut self) -> std::io::Result<()>;
    fn resume(&mut self) -> std::io::Result<()>;

    /// Reports whether a process in this launch session remained alive after
    /// the directly spawned primary process exited.
    fn delegated_descendant_observed(&self) -> bool {
        false
    }
}

pub struct SystemLaunchProcess {
    child: Child,
    primary_status: Option<ExitStatus>,
    delegated_descendant_observed: bool,
    #[cfg(unix)]
    process_group: libc::pid_t,
    #[cfg(unix)]
    suspended: bool,
    #[cfg(windows)]
    job: std::os::windows::io::OwnedHandle,
    #[cfg(windows)]
    suspended_threads: Vec<(u32, u32)>,
}

impl SystemLaunchProcess {
    fn session_has_active_processes(&self) -> std::io::Result<bool> {
        #[cfg(unix)]
        {
            process_group_has_members(self.process_group)
        }
        #[cfg(windows)]
        {
            windows_job_active_processes(&self.job).map(|count| count > 0)
        }
        #[cfg(not(any(unix, windows)))]
        {
            Ok(false)
        }
    }
}

impl LaunchProcess for SystemLaunchProcess {
    fn id(&self) -> u32 {
        self.child.id()
    }

    fn wait(&mut self) -> std::io::Result<ExitStatus> {
        loop {
            if let Some(status) = self.try_wait()? {
                return Ok(status);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn try_wait(&mut self) -> std::io::Result<Option<ExitStatus>> {
        if self.primary_status.is_none() {
            self.primary_status = self.child.try_wait()?;
        }
        let Some(status) = self.primary_status else {
            return Ok(None);
        };
        if self.session_has_active_processes()? {
            self.delegated_descendant_observed = true;
            return Ok(None);
        }
        Ok(Some(status))
    }

    #[cfg(unix)]
    fn suspend(&mut self) -> std::io::Result<()> {
        if self.suspended {
            return Ok(());
        }
        signal_process_group(self.process_group, libc::SIGSTOP)?;
        self.suspended = true;
        Ok(())
    }

    #[cfg(unix)]
    fn resume(&mut self) -> std::io::Result<()> {
        if !self.suspended {
            return Ok(());
        }
        signal_process_group(self.process_group, libc::SIGCONT)?;
        self.suspended = false;
        Ok(())
    }

    #[cfg(windows)]
    fn suspend(&mut self) -> std::io::Result<()> {
        if !self.suspended_threads.is_empty() {
            return Ok(());
        }
        let threads = windows_job_thread_ids(&self.job)?;
        let mut suspended = Vec::with_capacity(threads.len());
        for (process_id, thread_id) in threads {
            if let Err(error) = windows_adjust_thread(thread_id, true) {
                self.suspended_threads = suspended;
                let _ = self.resume();
                return Err(error);
            }
            suspended.push((process_id, thread_id));
        }
        if suspended.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "launch session for process {} has no controllable threads",
                    self.child.id()
                ),
            ));
        }
        self.suspended_threads = suspended;
        Ok(())
    }

    #[cfg(windows)]
    fn resume(&mut self) -> std::io::Result<()> {
        if self.suspended_threads.is_empty() {
            return Ok(());
        }
        let live_threads = windows_job_thread_ids(&self.job)?;
        let mut remaining = Vec::new();
        let mut first_error = None;
        for thread in std::mem::take(&mut self.suspended_threads) {
            if !live_threads.contains(&thread) {
                continue;
            }
            let (_, thread_id) = thread;
            if let Err(error) = windows_adjust_thread(thread_id, false) {
                remaining.push(thread);
                if first_error.is_none() {
                    first_error = Some(error);
                }
            }
        }
        self.suspended_threads = remaining;
        first_error.map_or(Ok(()), Err)
    }

    #[cfg(not(any(unix, windows)))]
    fn suspend(&mut self) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "pausing a launched process is not supported on this host",
        ))
    }

    #[cfg(not(any(unix, windows)))]
    fn resume(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn delegated_descendant_observed(&self) -> bool {
        self.delegated_descendant_observed
    }
}

impl Drop for SystemLaunchProcess {
    fn drop(&mut self) {
        let _ = <Self as LaunchProcess>::resume(self);
    }
}

#[cfg(unix)]
fn process_group_has_members(process_group: libc::pid_t) -> std::io::Result<bool> {
    // SAFETY: the negated process-group identifier was created specifically
    // for this launch. Signal zero performs existence/permission checking
    // without changing any process state.
    if unsafe { libc::kill(-process_group, 0) } == 0 {
        return Ok(true);
    }
    let error = std::io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ESRCH) {
        Ok(false)
    } else {
        Err(error)
    }
}

#[cfg(unix)]
fn signal_process_group(process_group: libc::pid_t, signal: libc::c_int) -> std::io::Result<()> {
    // SAFETY: `process_group` is a positive, isolated group created for this
    // launch, so its negation targets only that group. Callers restrict
    // `signal` to SIGSTOP or SIGCONT.
    if unsafe { libc::kill(-process_group, signal) } == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

#[cfg(windows)]
fn windows_create_job() -> std::io::Result<std::os::windows::io::OwnedHandle> {
    use std::os::windows::io::FromRawHandle;
    use windows_sys::Win32::System::JobObjects::CreateJobObjectW;

    // SAFETY: null security attributes and name request a private job. The
    // checked handle is transferred into OwnedHandle exactly once.
    unsafe {
        let handle = CreateJobObjectW(std::ptr::null(), std::ptr::null());
        if handle.is_null() {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(std::os::windows::io::OwnedHandle::from_raw_handle(handle))
        }
    }
}

#[cfg(windows)]
fn windows_assign_process_to_job(
    job: &std::os::windows::io::OwnedHandle,
    child: &Child,
) -> std::io::Result<()> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::System::JobObjects::AssignProcessToJobObject;

    // SAFETY: both raw handles are borrowed from live owned objects for the
    // duration of the call.
    if unsafe { AssignProcessToJobObject(job.as_raw_handle(), child.as_raw_handle()) } != 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

#[cfg(windows)]
fn windows_job_active_processes(job: &std::os::windows::io::OwnedHandle) -> std::io::Result<u32> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::System::JobObjects::{
        JobObjectBasicAccountingInformation, QueryInformationJobObject,
        JOBOBJECT_BASIC_ACCOUNTING_INFORMATION,
    };

    let mut information = JOBOBJECT_BASIC_ACCOUNTING_INFORMATION::default();
    // SAFETY: the output pointer and byte length describe the initialized
    // accounting structure, and the job handle remains live.
    if unsafe {
        QueryInformationJobObject(
            job.as_raw_handle(),
            JobObjectBasicAccountingInformation,
            std::ptr::from_mut(&mut information).cast(),
            std::mem::size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
            std::ptr::null_mut(),
        )
    } != 0
    {
        Ok(information.ActiveProcesses)
    } else {
        Err(std::io::Error::last_os_error())
    }
}

#[cfg(windows)]
fn windows_job_process_ids(job: &std::os::windows::io::OwnedHandle) -> std::io::Result<Vec<u32>> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::System::JobObjects::{
        JobObjectBasicProcessIdList, QueryInformationJobObject, JOBOBJECT_BASIC_PROCESS_ID_LIST,
    };

    const ERROR_MORE_DATA: i32 = 234;
    const MAX_SESSION_PROCESSES: usize = 16_384;
    let mut capacity = 16usize;
    loop {
        let byte_length = std::mem::size_of::<u32>() * 2 + std::mem::size_of::<usize>() * capacity;
        let words = byte_length.div_ceil(std::mem::size_of::<usize>());
        let mut storage = vec![0usize; words];
        let information = storage
            .as_mut_ptr()
            .cast::<JOBOBJECT_BASIC_PROCESS_ID_LIST>();
        // SAFETY: `storage` is usize-aligned and large enough for the fixed
        // header plus `capacity` process identifiers. The returned count is
        // checked against that capacity before the flexible array is read.
        let success = unsafe {
            QueryInformationJobObject(
                job.as_raw_handle(),
                JobObjectBasicProcessIdList,
                information.cast(),
                u32::try_from(byte_length).expect("bounded job buffer fits u32"),
                std::ptr::null_mut(),
            )
        };
        if success != 0 {
            // SAFETY: successful query initialized the header.
            let count = unsafe { (*information).NumberOfProcessIdsInList as usize };
            if count > capacity {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Windows returned more job process identifiers than fit the query buffer",
                ));
            }
            // SAFETY: the successful query initialized exactly `count`
            // entries in the flexible ProcessIdList array.
            let identifiers =
                unsafe { std::slice::from_raw_parts((*information).ProcessIdList.as_ptr(), count) };
            return identifiers
                .iter()
                .copied()
                .map(|identifier| {
                    u32::try_from(identifier).map_err(|_| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Windows process identifier {identifier} does not fit u32"),
                        )
                    })
                })
                .collect();
        }

        let error = std::io::Error::last_os_error();
        // SAFETY: even ERROR_MORE_DATA initializes the fixed header with the
        // assigned-process count documented by QueryInformationJobObject.
        let assigned = unsafe { (*information).NumberOfAssignedProcesses as usize };
        if error.raw_os_error() == Some(ERROR_MORE_DATA)
            && assigned > capacity
            && assigned <= MAX_SESSION_PROCESSES
        {
            capacity = assigned;
            continue;
        }
        return Err(error);
    }
}

#[cfg(windows)]
fn windows_process_thread_ids(pid: u32) -> std::io::Result<Vec<u32>> {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
    };

    // SAFETY: the snapshot handle is checked and closed on every path; the
    // initialized THREADENTRY32 is kept alive for each Win32 call.
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return Err(std::io::Error::last_os_error());
        }
        let mut entry = THREADENTRY32 {
            dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
            ..THREADENTRY32::default()
        };
        let mut thread_ids = Vec::new();
        if Thread32First(snapshot, &mut entry) != 0 {
            loop {
                if entry.th32OwnerProcessID == pid {
                    thread_ids.push(entry.th32ThreadID);
                }
                if Thread32Next(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snapshot);
        Ok(thread_ids)
    }
}

#[cfg(windows)]
fn windows_job_thread_ids(
    job: &std::os::windows::io::OwnedHandle,
) -> std::io::Result<Vec<(u32, u32)>> {
    let process_ids = windows_job_process_ids(job)?;
    let mut threads = Vec::new();
    for process_id in process_ids {
        threads.extend(
            windows_process_thread_ids(process_id)?
                .into_iter()
                .map(|thread_id| (process_id, thread_id)),
        );
    }
    Ok(threads)
}

#[cfg(windows)]
fn windows_adjust_thread(thread_id: u32, suspend: bool) -> std::io::Result<()> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{
        OpenThread, ResumeThread, SuspendThread, THREAD_SUSPEND_RESUME,
    };

    // SAFETY: OpenThread returns a checked owned handle, which is closed after
    // the single suspend-count adjustment.
    unsafe {
        let thread = OpenThread(THREAD_SUSPEND_RESUME, 0, thread_id);
        if thread.is_null() {
            return Err(std::io::Error::last_os_error());
        }
        let previous_count = if suspend {
            SuspendThread(thread)
        } else {
            ResumeThread(thread)
        };
        let result = if previous_count == u32::MAX {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        };
        CloseHandle(thread);
        result
    }
}

#[cfg(windows)]
fn windows_resume_created_process(pid: u32) -> std::io::Result<()> {
    let threads = windows_process_thread_ids(pid)?;
    if threads.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("new process {pid} has no resumable primary thread"),
        ));
    }
    for thread_id in threads {
        windows_adjust_thread(thread_id, false)?;
    }
    Ok(())
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
    type Handle = SystemLaunchProcess;

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
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        #[cfg(windows)]
        let job = windows_create_job().map_err(|source| LaunchError::Supervise {
            executable: request.executable.clone(),
            source,
        })?;
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            const CREATE_SUSPENDED: u32 = 0x0000_0004;
            let mut flags = CREATE_SUSPENDED;
            if request.hide_console {
                flags |= CREATE_NO_WINDOW;
            }
            command.creation_flags(flags);
        }
        let mut child = command.spawn().map_err(|source| LaunchError::Spawn {
            executable: request.executable.clone(),
            source,
        })?;
        #[cfg(unix)]
        let process_group = match libc::pid_t::try_from(child.id()) {
            Ok(process_group) if process_group > 0 => process_group,
            _ => {
                let source = std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("process identifier {} does not fit pid_t", child.id()),
                );
                let _ = child.kill();
                let _ = child.wait();
                return Err(LaunchError::Supervise {
                    executable: request.executable.clone(),
                    source,
                });
            }
        };
        #[cfg(windows)]
        {
            let supervision = windows_assign_process_to_job(&job, &child)
                .and_then(|()| windows_resume_created_process(child.id()));
            if let Err(source) = supervision {
                let _ = child.kill();
                let _ = child.wait();
                return Err(LaunchError::Supervise {
                    executable: request.executable.clone(),
                    source,
                });
            }
        }
        Ok(SystemLaunchProcess {
            child,
            primary_status: None,
            delegated_descendant_observed: false,
            #[cfg(unix)]
            process_group,
            #[cfg(unix)]
            suspended: false,
            #[cfg(windows)]
            job,
            #[cfg(windows)]
            suspended_threads: Vec::new(),
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
    #[error("failed to establish supervision for {executable}: {source}")]
    Supervise {
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
    PrimaryPaused {
        process_suspended: bool,
    },
    PrimaryResumed {
        process_resumed: bool,
    },
    PrimaryControlFailed {
        action: LaunchControlCommand,
        message: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LaunchControlCommand {
    Pause,
    Resume,
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
    pub delegated_descendant_observed: bool,
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
    execute_launch_sequence_with_control(
        sequence,
        &SystemProcessLauncher,
        AUTO_RUN_BEFORE_WAIT_TIMEOUT,
        None,
        &mut on_event,
    )
}

/// Executes a launch sequence while accepting pause/resume commands for its
/// one primary child. Dropping the sender while paused causes the worker to
/// resume the child before it continues supervising the session.
pub fn execute_launch_sequence_controlled(
    sequence: &LaunchSequence,
    control: &Receiver<LaunchControlCommand>,
    mut on_event: impl FnMut(LaunchSequenceEvent),
) -> Result<LaunchSequenceReport, LaunchSequenceError> {
    execute_launch_sequence_with_control(
        sequence,
        &SystemProcessLauncher,
        AUTO_RUN_BEFORE_WAIT_TIMEOUT,
        Some(control),
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
    execute_launch_sequence_with_control(
        sequence,
        launcher,
        before_wait_timeout,
        None,
        &mut on_event,
    )
}

pub fn execute_launch_sequence_with_control<L>(
    sequence: &LaunchSequence,
    launcher: &L,
    before_wait_timeout: Duration,
    control: Option<&Receiver<LaunchControlCommand>>,
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
            let status = if step.role.is_primary() {
                if let Some(control) = control {
                    wait_for_primary_exit(&mut process, sequence.pause, control, &mut on_event)
                } else {
                    process.wait()
                }
            } else {
                process.wait()
            }
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
                let delegated_descendant_observed = process.delegated_descendant_observed();
                primary = Some((
                    step.plan.target.clone(),
                    step.plan.request.executable.clone(),
                    pid,
                    started_at,
                    runtime_started.elapsed(),
                    status.success(),
                    delegated_descendant_observed,
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
        delegated_descendant_observed,
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
        delegated_descendant_observed,
        automatic_before_started,
        automatic_after_started,
        before_wait_timeouts,
    })
}

fn wait_for_primary_exit<P: LaunchProcess>(
    process: &mut P,
    pause_policy: LaunchPausePolicy,
    control: &Receiver<LaunchControlCommand>,
    on_event: &mut impl FnMut(LaunchSequenceEvent),
) -> std::io::Result<ExitStatus> {
    let mut paused = false;
    let mut process_suspended = false;
    let mut control_connected = true;
    loop {
        if let Some(status) = process.try_wait()? {
            return Ok(status);
        }

        if control_connected {
            loop {
                match control.try_recv() {
                    Ok(LaunchControlCommand::Pause) if pause_policy.enabled && !paused => {
                        if pause_policy.suspend_process {
                            match process.suspend() {
                                Ok(()) => process_suspended = true,
                                Err(error) => {
                                    let _ = process.resume();
                                    on_event(LaunchSequenceEvent::PrimaryControlFailed {
                                        action: LaunchControlCommand::Pause,
                                        message: error.to_string(),
                                    });
                                    continue;
                                }
                            }
                        }
                        paused = true;
                        on_event(LaunchSequenceEvent::PrimaryPaused { process_suspended });
                    }
                    Ok(LaunchControlCommand::Resume) if paused => {
                        if process_suspended {
                            if let Err(error) = process.resume() {
                                on_event(LaunchSequenceEvent::PrimaryControlFailed {
                                    action: LaunchControlCommand::Resume,
                                    message: error.to_string(),
                                });
                                continue;
                            }
                        }
                        let was_suspended = process_suspended;
                        paused = false;
                        process_suspended = false;
                        on_event(LaunchSequenceEvent::PrimaryResumed {
                            process_resumed: was_suspended,
                        });
                    }
                    Ok(_) => {}
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        control_connected = false;
                        break;
                    }
                }
            }
        }

        if !control_connected && paused && (!process_suspended || process.resume().is_ok()) {
            paused = false;
            process_suspended = false;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
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
    #[error("{record}.{key} has invalid value {value:?}; expected {expected}")]
    InvalidFrontendLaunchSetting {
        record: String,
        key: &'static str,
        value: String,
        expected: &'static str,
    },
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
    fn startup_policy_uses_game_override_emulator_default_or_direct_game_settings() {
        let mut configuration = configuration();
        configuration.emulators[0].use_startup_screen = true;
        configuration.emulators[0].startup_load_delay = 1_250;

        let mut emulated_game = game();
        emulated_game.use_startup_screen = false;
        emulated_game.startup_load_delay = 25;
        let sequence = build_game_launch_sequence_with_context_and_resolver(
            Path::new("/launchbox"),
            &emulated_game,
            [],
            Some(&configuration),
            &LaunchContext::default(),
            &HostPathResolver::default(),
        )
        .expect("build emulator-default startup sequence");
        assert_eq!(
            sequence.startup,
            LaunchStartupPolicy {
                enabled: true,
                load_delay: Duration::from_millis(1_250),
                source: LaunchStartupSettingsSource::EmulatorDefault,
            }
        );
        assert_eq!(
            sequence.shutdown,
            LaunchShutdownPolicy {
                enabled: true,
                source: LaunchStartupSettingsSource::EmulatorDefault,
            }
        );

        emulated_game.override_default_startup_screen_settings = true;
        emulated_game.use_startup_screen = true;
        emulated_game.startup_load_delay = 375;
        emulated_game.disable_shutdown_screen = true;
        let sequence = build_game_launch_sequence_with_context_and_resolver(
            Path::new("/launchbox"),
            &emulated_game,
            [],
            Some(&configuration),
            &LaunchContext::default(),
            &HostPathResolver::default(),
        )
        .expect("build game-override startup sequence");
        assert_eq!(
            sequence.startup,
            LaunchStartupPolicy {
                enabled: true,
                load_delay: Duration::from_millis(375),
                source: LaunchStartupSettingsSource::GameOverride,
            }
        );
        assert_eq!(
            sequence.shutdown,
            LaunchShutdownPolicy {
                enabled: false,
                source: LaunchStartupSettingsSource::GameOverride,
            }
        );

        let mut direct_game = emulated_game;
        direct_game.override_default_startup_screen_settings = false;
        direct_game.emulator_id = Some(UNASSIGNED_EMULATOR_ID.into());
        direct_game.startup_load_delay = 90;
        direct_game.disable_shutdown_screen = false;
        let sequence = build_game_launch_sequence_with_context_and_resolver(
            Path::new("/launchbox"),
            &direct_game,
            [],
            Some(&configuration),
            &LaunchContext::default(),
            &HostPathResolver::default(),
        )
        .expect("build direct-game startup sequence");
        assert_eq!(
            sequence.startup,
            LaunchStartupPolicy {
                enabled: true,
                load_delay: Duration::from_millis(90),
                source: LaunchStartupSettingsSource::DirectGame,
            }
        );
        assert_eq!(
            sequence.shutdown,
            LaunchShutdownPolicy {
                enabled: true,
                source: LaunchStartupSettingsSource::DirectGame,
            }
        );
    }

    #[test]
    fn pause_policy_uses_game_override_emulator_default_or_direct_game_settings() {
        let mut configuration = configuration();
        configuration.emulators[0].use_pause_screen = true;
        configuration.emulators[0].suspend_process_on_pause = true;
        configuration.emulators[0].forceful_pause_screen_activation = true;

        let mut emulated_game = game();
        emulated_game.use_pause_screen = false;
        let sequence = build_game_launch_sequence_with_context_and_resolver(
            Path::new("/launchbox"),
            &emulated_game,
            [],
            Some(&configuration),
            &LaunchContext::default(),
            &HostPathResolver::default(),
        )
        .expect("build emulator-default pause sequence");
        assert_eq!(
            sequence.pause,
            LaunchPausePolicy {
                enabled: true,
                suspend_process: true,
                forceful_activation: true,
                source: LaunchStartupSettingsSource::EmulatorDefault,
            }
        );

        emulated_game.override_default_pause_screen_settings = true;
        emulated_game.use_pause_screen = true;
        emulated_game.suspend_process_on_pause = false;
        emulated_game.forceful_pause_screen_activation = false;
        let sequence = build_game_launch_sequence_with_context_and_resolver(
            Path::new("/launchbox"),
            &emulated_game,
            [],
            Some(&configuration),
            &LaunchContext::default(),
            &HostPathResolver::default(),
        )
        .expect("build game-override pause sequence");
        assert_eq!(
            sequence.pause,
            LaunchPausePolicy {
                enabled: true,
                suspend_process: false,
                forceful_activation: false,
                source: LaunchStartupSettingsSource::GameOverride,
            }
        );

        let mut direct_game = emulated_game;
        direct_game.override_default_pause_screen_settings = false;
        direct_game.emulator_id = Some(UNASSIGNED_EMULATOR_ID.into());
        direct_game.use_pause_screen = true;
        direct_game.suspend_process_on_pause = true;
        direct_game.forceful_pause_screen_activation = true;
        let sequence = build_game_launch_sequence_with_context_and_resolver(
            Path::new("/launchbox"),
            &direct_game,
            [],
            Some(&configuration),
            &LaunchContext::default(),
            &HostPathResolver::default(),
        )
        .expect("build direct-game pause sequence");
        assert_eq!(
            sequence.pause,
            LaunchPausePolicy {
                enabled: true,
                suspend_process: true,
                forceful_activation: true,
                source: LaunchStartupSettingsSource::DirectGame,
            }
        );
    }

    #[test]
    fn selected_additional_application_uses_its_effective_emulator_screen_defaults() {
        let mut configuration = configuration();
        configuration.emulators[0].use_startup_screen = true;
        configuration.emulators[0].startup_load_delay = 640;
        configuration.emulators[0].disable_shutdown_screen = true;
        configuration.emulators[0].use_pause_screen = true;
        configuration.emulators[0].suspend_process_on_pause = true;
        configuration.emulators[0].forceful_pause_screen_activation = true;
        let mut application = additional_application("version", 0);
        application.use_emulator = true;
        application.emulator_id = Some("emulator-id".into());

        let sequence = build_selected_additional_application_sequence_with_context_and_resolver(
            Path::new("/launchbox"),
            &game(),
            &application,
            Some(&configuration),
            &LaunchContext::default(),
            &HostPathResolver::default(),
        )
        .expect("build selected application sequence");

        assert_eq!(
            sequence.startup,
            LaunchStartupPolicy {
                enabled: true,
                load_delay: Duration::from_millis(640),
                source: LaunchStartupSettingsSource::EmulatorDefault,
            }
        );
        assert_eq!(
            sequence.shutdown,
            LaunchShutdownPolicy {
                enabled: false,
                source: LaunchStartupSettingsSource::EmulatorDefault,
            }
        );
        assert_eq!(
            sequence.pause,
            LaunchPausePolicy {
                enabled: true,
                suspend_process: true,
                forceful_activation: true,
                source: LaunchStartupSettingsSource::EmulatorDefault,
            }
        );
    }

    #[test]
    fn frontend_launch_policy_parses_milliseconds_and_rejects_invalid_values() {
        let settings = FrontendSettings {
            record_name: "BigBoxSettings".into(),
            entries: vec![
                lb_domain::SettingEntry {
                    key: "UseStartupScreen".into(),
                    value: "false".into(),
                },
                lb_domain::SettingEntry {
                    key: "StartupTheme".into(),
                    value: "Fixture Theme".into(),
                },
                lb_domain::SettingEntry {
                    key: "MinimumStartupScreenDisplayTime".into(),
                    value: "1250".into(),
                },
                lb_domain::SettingEntry {
                    key: "MinimumShutdownScreenDisplayTime".into(),
                    value: "750".into(),
                },
                lb_domain::SettingEntry {
                    key: "HideMouseCursorOnStartupScreens".into(),
                    value: "true".into(),
                },
            ],
            ..FrontendSettings::default()
        };
        assert_eq!(
            FrontendLaunchScreenPolicy::from_settings(Some(&settings))
                .expect("parse frontend launch-screen settings"),
            FrontendLaunchScreenPolicy {
                enabled: false,
                theme: "Fixture Theme".into(),
                minimum_startup_display: Duration::from_millis(1_250),
                minimum_shutdown_display: Duration::from_millis(750),
                hide_mouse_cursor: true,
            }
        );

        let invalid = FrontendSettings {
            record_name: "Settings".into(),
            entries: vec![lb_domain::SettingEntry {
                key: "MinimumStartupScreenDisplayTime".into(),
                value: "-1".into(),
            }],
            ..FrontendSettings::default()
        };
        assert!(matches!(
            FrontendLaunchScreenPolicy::from_settings(Some(&invalid)),
            Err(LaunchPlanError::InvalidFrontendLaunchSetting {
                key: "MinimumStartupScreenDisplayTime",
                ..
            })
        ));
    }

    #[test]
    fn frontend_pause_policy_parses_independent_launchbox_and_bigbox_settings() {
        let settings = FrontendSettings {
            record_name: "BigBoxSettings".into(),
            entries: vec![
                lb_domain::SettingEntry {
                    key: "UsePauseScreen".into(),
                    value: "false".into(),
                },
                lb_domain::SettingEntry {
                    key: "PauseTheme".into(),
                    value: "Fixture Pause".into(),
                },
                lb_domain::SettingEntry {
                    key: "PauseScreenMuting".into(),
                    value: "false".into(),
                },
                lb_domain::SettingEntry {
                    key: "PauseScreenFading".into(),
                    value: "true".into(),
                },
            ],
            ..FrontendSettings::default()
        };
        assert_eq!(
            FrontendPauseScreenPolicy::from_settings(Some(&settings))
                .expect("parse frontend pause-screen settings"),
            FrontendPauseScreenPolicy {
                enabled: false,
                theme: "Fixture Pause".into(),
                mute_frontend_audio: false,
                fade_frontend: true,
            }
        );

        let invalid = FrontendSettings {
            record_name: "Settings".into(),
            entries: vec![lb_domain::SettingEntry {
                key: "PauseScreenFading".into(),
                value: "sometimes".into(),
            }],
            ..FrontendSettings::default()
        };
        assert!(matches!(
            FrontendPauseScreenPolicy::from_settings(Some(&invalid)),
            Err(LaunchPlanError::InvalidFrontendLaunchSetting {
                key: "PauseScreenFading",
                ..
            })
        ));
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

    #[test]
    fn bigpemu_template_passes_the_native_rom_path_once_and_enables_local_data() {
        let mut configuration = configuration();
        configuration.emulators[0].title = "BigPEmu".into();
        configuration.emulators[0].application_path = r"Emulators\BigPEmu\bigpemu".into();
        configuration.emulators[0].command_line = Some("%romfile% -localdata".into());
        configuration.platforms[0].command_line = None;
        let mut game = game();
        game.platform = "Atari Jaguar".into();
        game.application_path = r"Games\Atari Jaguar\Tempest 2000.j64".into();
        game.command_line = None;

        let plan = build_launch_plan(Path::new("/launchbox"), &game, Some(&configuration))
            .expect("build BigPEmu plan");

        assert_eq!(
            plan.request.executable,
            PathBuf::from("/launchbox/Emulators/BigPEmu/bigpemu")
        );
        assert_eq!(
            plan.request.arguments,
            [
                "/launchbox/Games/Atari Jaguar/Tempest 2000.j64",
                "-localdata",
            ]
            .map(OsString::from)
        );
        assert_eq!(
            plan.request.working_directory,
            Some(PathBuf::from("/launchbox/Emulators/BigPEmu"))
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

        let xemu = LaunchRequest::new("/launchbox/Emulators/Xemu/xemu.AppImage")
            .arg("-full-screen")
            .arg("-dvd_path")
            .arg("/launchbox/Games/xbox.iso");
        let (executable, arguments) = system_process_command(&xemu);
        assert_eq!(executable, Path::new("appimage-run"));
        assert_eq!(
            arguments,
            [
                OsString::from("/launchbox/Emulators/Xemu/xemu.AppImage"),
                OsString::from("-full-screen"),
                OsString::from("-dvd_path"),
                OsString::from("/launchbox/Games/xbox.iso"),
            ]
        );
    }
}
