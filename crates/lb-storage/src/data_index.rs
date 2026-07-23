use super::{LibraryIndex, StorageError};
use lb_domain::{
    Emulator, EmulatorConfiguration, EmulatorPlatform, FrontendSettings, GameController,
    ImageTypeSetting, InputBinding, ListCacheItem, NavigationMetadata, ParentRelationship,
    PlatformCatalog, PlatformCategory, PlatformDefinition, PlatformFolder, Playlist,
    PlaylistDocument, PlaylistFilter, PlaylistGame, SettingEntry,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use xmltree::{Element, XMLNode};

/// The non-platform XML document families present in a LaunchBox data
/// directory. The lossless editor uses this to run the same semantic parser as
/// the read index before replacing a user file.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuxiliaryDocumentKind {
    Playlist,
    Emulators,
    Platforms,
    Parents,
    GameControllers,
    InputBindings,
    ImportBlacklist,
    ListCache,
    Settings,
    BigBoxSettings,
}

impl AuxiliaryDocumentKind {
    pub fn infer(path: &Path) -> Result<Self, StorageError> {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        let kind = match file_name {
            "Emulators.xml" => Self::Emulators,
            "Platforms.xml" => Self::Platforms,
            "Parents.xml" => Self::Parents,
            "GameControllers.xml" => Self::GameControllers,
            "InputBindings.xml" => Self::InputBindings,
            "ImportBlacklist.xml" => Self::ImportBlacklist,
            "ListCache.xml" => Self::ListCache,
            "Settings.xml" => Self::Settings,
            "BigBoxSettings.xml" => Self::BigBoxSettings,
            _ if path
                .parent()
                .and_then(Path::file_name)
                .and_then(|name| name.to_str())
                == Some("Playlists") =>
            {
                Self::Playlist
            }
            _ => {
                return Err(StorageError::UnsupportedAuxiliaryDocument {
                    path: path.to_path_buf(),
                });
            }
        };
        Ok(kind)
    }
}

/// Complete read model for the XML data files in a LaunchBox installation.
/// Large per-platform game files retain their streaming index; the much
/// smaller catalog/configuration files are parsed into typed records.
#[derive(Clone, Debug)]
pub struct LaunchBoxDataIndex {
    root: PathBuf,
    data_root: PathBuf,
    platforms: LibraryIndex,
    playlists: Vec<PlaylistDocument>,
    emulator_configuration: Option<EmulatorConfiguration>,
    platform_catalog: Option<PlatformCatalog>,
    parents: Vec<ParentRelationship>,
    game_controllers: Vec<GameController>,
    input_bindings: Vec<InputBinding>,
    ignored_game_ids: Vec<String>,
    list_cache: Vec<ListCacheItem>,
    settings: Option<FrontendSettings>,
    big_box_settings: Option<FrontendSettings>,
}

impl LaunchBoxDataIndex {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let path = path.as_ref();
        let data_root = find_data_directory(path)?;
        let platforms = LibraryIndex::load(&data_root)?;
        let playlists = load_playlists(&data_root.join("Playlists"))?;
        let emulator_configuration = load_emulators(&data_root.join("Emulators.xml"))?;
        let platform_catalog = load_platform_catalog(&data_root.join("Platforms.xml"))?;
        let parents = load_parents(&data_root.join("Parents.xml"))?;
        let game_controllers = load_game_controllers(&data_root.join("GameControllers.xml"))?;
        let input_bindings = load_input_bindings(&data_root.join("InputBindings.xml"))?;
        let ignored_game_ids = load_ignored_games(&data_root.join("ImportBlacklist.xml"))?;
        let list_cache = load_list_cache(&data_root.join("ListCache.xml"))?;
        let settings = load_settings(&data_root.join("Settings.xml"), "Settings")?;
        let big_box_settings =
            load_settings(&data_root.join("BigBoxSettings.xml"), "BigBoxSettings")?;

        Ok(Self {
            root: path.to_path_buf(),
            data_root,
            platforms,
            playlists,
            emulator_configuration,
            platform_catalog,
            parents,
            game_controllers,
            input_bindings,
            ignored_game_ids,
            list_cache,
            settings,
            big_box_settings,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn data_root(&self) -> &Path {
        &self.data_root
    }

    pub fn platforms(&self) -> &LibraryIndex {
        &self.platforms
    }

    pub fn playlists(&self) -> &[PlaylistDocument] {
        &self.playlists
    }

    pub fn playlist_filters(&self) -> impl Iterator<Item = &PlaylistFilter> {
        self.playlists
            .iter()
            .flat_map(|document| document.filters.iter())
    }

    pub fn playlist_games(&self) -> impl Iterator<Item = &PlaylistGame> {
        self.playlists
            .iter()
            .flat_map(|document| document.games.iter())
    }

    pub fn emulator_configuration(&self) -> Option<&EmulatorConfiguration> {
        self.emulator_configuration.as_ref()
    }

    pub fn platform_catalog(&self) -> Option<&PlatformCatalog> {
        self.platform_catalog.as_ref()
    }

    pub fn parents(&self) -> &[ParentRelationship] {
        &self.parents
    }

    pub fn game_controllers(&self) -> &[GameController] {
        &self.game_controllers
    }

    pub fn input_bindings(&self) -> &[InputBinding] {
        &self.input_bindings
    }

    pub fn ignored_game_ids(&self) -> &[String] {
        &self.ignored_game_ids
    }

    pub fn list_cache(&self) -> &[ListCacheItem] {
        &self.list_cache
    }

    pub fn settings(&self) -> Option<&FrontendSettings> {
        self.settings.as_ref()
    }

    pub fn big_box_settings(&self) -> Option<&FrontendSettings> {
        self.big_box_settings.as_ref()
    }
}

fn find_data_directory(path: &Path) -> Result<PathBuf, StorageError> {
    [path.join("Data"), path.to_path_buf()]
        .into_iter()
        .find(|candidate| candidate.join("Platforms").is_dir())
        .ok_or_else(|| StorageError::NoDataDirectory {
            path: path.to_path_buf(),
        })
}

fn load_playlists(directory: &Path) -> Result<Vec<PlaylistDocument>, StorageError> {
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    let mut files = xml_files(directory)?;
    files.sort();
    files.into_iter().map(|path| load_playlist(&path)).collect()
}

fn load_playlist(path: &Path) -> Result<PlaylistDocument, StorageError> {
    let root = load_root(path)?;
    parse_playlist(path, &root)
}

pub(crate) fn parse_playlist(
    path: &Path,
    root: &Element,
) -> Result<PlaylistDocument, StorageError> {
    let playlist_elements = elements_named(root, "Playlist").collect::<Vec<_>>();
    let playlist_element = exactly_one(path, "Playlist", &playlist_elements)?;
    let playlist = Playlist {
        id: required_text(playlist_element, "Playlist", "PlaylistId")?,
        metadata: parse_navigation_metadata(playlist_element, "Playlist")?,
        auto_populate: bool_field(playlist_element, "Playlist", "AutoPopulate")?,
        include_with_platforms: bool_field(playlist_element, "Playlist", "IncludeWithPlatforms")?,
        is_autogenerated: bool_field(playlist_element, "Playlist", "IsAutogenerated")?,
        sort_by: optional_text(playlist_element, "SortBy"),
    };
    let filters = elements_named(root, "PlaylistFilter")
        .map(|element| {
            Ok(PlaylistFilter {
                field_key: required_text(element, "PlaylistFilter", "FieldKey")?,
                comparison_type_key: required_text(element, "PlaylistFilter", "ComparisonTypeKey")?,
                value: text(element, "Value").unwrap_or_default(),
            })
        })
        .collect::<Result<Vec<_>, StorageError>>()?;
    let games = elements_named(root, "PlaylistGame")
        .map(|element| {
            Ok(PlaylistGame {
                game_id: required_text(element, "PlaylistGame", "GameId")?,
                game_title: required_text(element, "PlaylistGame", "GameTitle")?,
                game_platform: required_text(element, "PlaylistGame", "GamePlatform")?,
                game_file_name: text(element, "GameFileName").unwrap_or_default(),
                launchbox_db_id: optional_number(
                    element,
                    "PlaylistGame",
                    "LaunchBoxDbId",
                    "unsigned integer",
                )?,
                manual_order: number_or_default(element, "PlaylistGame", "ManualOrder", "integer")?,
            })
        })
        .collect::<Result<Vec<_>, StorageError>>()?;
    let document = PlaylistDocument {
        source_path: path.to_path_buf(),
        playlist,
        filters,
        games,
    };
    document.validate()?;
    Ok(document)
}

fn load_emulators(path: &Path) -> Result<Option<EmulatorConfiguration>, StorageError> {
    let Some(root) = load_optional_root(path)? else {
        return Ok(None);
    };
    parse_emulators(path, &root).map(Some)
}

fn parse_emulators(path: &Path, root: &Element) -> Result<EmulatorConfiguration, StorageError> {
    let emulators = elements_named(root, "Emulator")
        .map(parse_emulator)
        .collect::<Result<Vec<_>, _>>()?;
    let platforms = elements_named(root, "EmulatorPlatform")
        .map(parse_emulator_platform)
        .collect::<Result<Vec<_>, _>>()?;
    let configuration = EmulatorConfiguration {
        source_path: path.to_path_buf(),
        emulators,
        platforms,
    };
    configuration.validate()?;
    Ok(configuration)
}

fn parse_emulator(element: &Element) -> Result<Emulator, StorageError> {
    Ok(Emulator {
        id: required_text(element, "Emulator", "ID")?,
        title: required_text(element, "Emulator", "Title")?,
        application_path: text(element, "ApplicationPath").unwrap_or_default(),
        command_line: optional_text(element, "CommandLine"),
        default_platform: optional_text(element, "DefaultPlatform"),
        auto_extract: bool_field(element, "Emulator", "AutoExtract")?,
        aggressive_window_hiding: bool_field(element, "Emulator", "AggressiveWindowHiding")?,
        default_pause_settings_pushed: bool_field(
            element,
            "Emulator",
            "DefaultPauseSettingsPushed",
        )?,
        disable_shutdown_screen: bool_field(element, "Emulator", "DisableShutdownScreen")?,
        enable_hardcore_achievements: bool_field(
            element,
            "Emulator",
            "EnableHardcoreAchievements",
        )?,
        file_name_without_extension_and_path: bool_field(
            element,
            "Emulator",
            "FileNameWithoutExtensionAndPath",
        )?,
        forceful_pause_screen_activation: bool_field(
            element,
            "Emulator",
            "ForcefulPauseScreenActivation",
        )?,
        hide_all_non_exclusive_fullscreen_windows: bool_field(
            element,
            "Emulator",
            "HideAllNonExclusiveFullscreenWindows",
        )?,
        hide_console: bool_field(element, "Emulator", "HideConsole")?,
        hide_mouse_cursor_in_game: bool_field(element, "Emulator", "HideMouseCursorInGame")?,
        login_to_cheevo_on_game_launch: bool_field(
            element,
            "Emulator",
            "LoginToCheevoOnGameLaunch",
        )?,
        no_quotes: bool_field(element, "Emulator", "NoQuotes")?,
        no_space: bool_field(element, "Emulator", "NoSpace")?,
        skip_version_check: bool_field(element, "Emulator", "SkipVersionCheck")?,
        startup_load_delay: number_or_default(
            element,
            "Emulator",
            "StartupLoadDelay",
            "unsigned integer",
        )?,
        suspend_process_on_pause: bool_field(element, "Emulator", "SuspendProcessOnPause")?,
        use_pause_screen: bool_field(element, "Emulator", "UsePauseScreen")?,
        use_startup_screen: bool_field(element, "Emulator", "UseStartupScreen")?,
        auto_hotkey_script: optional_text(element, "AutoHotkeyScript"),
        exit_auto_hotkey_script: optional_text(element, "ExitAutoHotkeyScript"),
        load_state_auto_hotkey_script: optional_text(element, "LoadStateAutoHotkeyScript"),
        pause_auto_hotkey_script: optional_text(element, "PauseAutoHotkeyScript"),
        reset_auto_hotkey_script: optional_text(element, "ResetAutoHotkeyScript"),
        resume_auto_hotkey_script: optional_text(element, "ResumeAutoHotkeyScript"),
        save_state_auto_hotkey_script: optional_text(element, "SaveStateAutoHotkeyScript"),
        swap_discs_auto_hotkey_script: optional_text(element, "SwapDiscsAutoHotkeyScript"),
    })
}

fn parse_emulator_platform(element: &Element) -> Result<EmulatorPlatform, StorageError> {
    Ok(EmulatorPlatform {
        emulator_id: required_text(element, "EmulatorPlatform", "Emulator")?,
        platform: required_text(element, "EmulatorPlatform", "Platform")?,
        command_line: optional_text(element, "CommandLine"),
        default: bool_field(element, "EmulatorPlatform", "Default")?,
        auto_extract: optional_bool_field(element, "EmulatorPlatform", "AutoExtract")?,
        m3u_disc_load_enabled: bool_field(element, "EmulatorPlatform", "M3uDiscLoadEnabled")?,
    })
}

fn load_platform_catalog(path: &Path) -> Result<Option<PlatformCatalog>, StorageError> {
    let Some(root) = load_optional_root(path)? else {
        return Ok(None);
    };
    parse_platform_catalog(path, &root).map(Some)
}

pub(crate) fn parse_platform_catalog(
    path: &Path,
    root: &Element,
) -> Result<PlatformCatalog, StorageError> {
    let platforms = elements_named(root, "Platform")
        .map(|element| {
            Ok(PlatformDefinition {
                metadata: parse_navigation_metadata(element, "Platform")?,
                release_date: optional_text(element, "ReleaseDate"),
                disable_auto_import: bool_field(element, "Platform", "DisableAutoImport")?,
            })
        })
        .collect::<Result<Vec<_>, StorageError>>()?;
    let categories = elements_named(root, "PlatformCategory")
        .map(|element| {
            Ok(PlatformCategory {
                metadata: parse_navigation_metadata(element, "PlatformCategory")?,
                is_autogenerated: bool_field(element, "PlatformCategory", "IsAutogenerated")?,
                disable_auto_import: bool_field(element, "PlatformCategory", "DisableAutoImport")?,
            })
        })
        .collect::<Result<Vec<_>, StorageError>>()?;
    let folders = elements_named(root, "PlatformFolder")
        .map(|element| {
            Ok(PlatformFolder {
                platform: required_text(element, "PlatformFolder", "Platform")?,
                media_type: required_text(element, "PlatformFolder", "MediaType")?,
                folder_path: required_text(element, "PlatformFolder", "FolderPath")?,
            })
        })
        .collect::<Result<Vec<_>, StorageError>>()?;
    let catalog = PlatformCatalog {
        source_path: path.to_path_buf(),
        platforms,
        categories,
        folders,
    };
    catalog.validate()?;
    Ok(catalog)
}

fn parse_navigation_metadata(
    element: &Element,
    record: &'static str,
) -> Result<NavigationMetadata, StorageError> {
    Ok(NavigationMetadata {
        name: required_text(element, record, "Name")?,
        nested_name: optional_text(element, "NestedName"),
        sort_title: optional_text(element, "SortTitle"),
        notes: optional_text(element, "Notes"),
        folder: optional_text(element, "Folder"),
        category: optional_text(element, "Category"),
        image_type: optional_text(element, "ImageType"),
        scrape_as: optional_text(element, "ScrapeAs"),
        hide_in_big_box: bool_field(element, record, "HideInBigBox")?,
        local_db_parsed: bool_field(element, record, "LocalDbParsed")?,
        last_game_id: optional_text(element, "LastGameId"),
        last_selected_child: optional_text(element, "LastSelectedChild"),
        cpu: optional_text(element, "Cpu"),
        developer: optional_text(element, "Developer"),
        display: optional_text(element, "Display"),
        graphics: optional_text(element, "Graphics"),
        manufacturer: optional_text(element, "Manufacturer"),
        max_controllers: optional_text(element, "MaxControllers"),
        media: optional_text(element, "Media"),
        memory: optional_text(element, "Memory"),
        sound: optional_text(element, "Sound"),
        android_theme_video_path: optional_text(element, "AndroidThemeVideoPath"),
        back_images_folder: optional_text(element, "BackImagesFolder"),
        banner_images_folder: optional_text(element, "BannerImagesFolder"),
        big_box_theme: optional_text(element, "BigBoxTheme"),
        big_box_view: optional_text(element, "BigBoxView"),
        clear_logo_images_folder: optional_text(element, "ClearLogoImagesFolder"),
        fanart_images_folder: optional_text(element, "FanartImagesFolder"),
        front_images_folder: optional_text(element, "FrontImagesFolder"),
        manuals_folder: optional_text(element, "ManualsFolder"),
        music_folder: optional_text(element, "MusicFolder"),
        screenshot_images_folder: optional_text(element, "ScreenshotImagesFolder"),
        steam_banner_images_folder: optional_text(element, "SteamBannerImagesFolder"),
        video_path: optional_text(element, "VideoPath"),
        videos_folder: optional_text(element, "VideosFolder"),
    })
}

fn load_parents(path: &Path) -> Result<Vec<ParentRelationship>, StorageError> {
    let Some(root) = load_optional_root(path)? else {
        return Ok(Vec::new());
    };
    parse_parents(&root)
}

pub(crate) fn parse_parents(root: &Element) -> Result<Vec<ParentRelationship>, StorageError> {
    elements_named(root, "Parent")
        .map(|element| {
            let relationship = ParentRelationship {
                parent_platform_category_name: optional_text(element, "ParentPlatformCategoryName"),
                parent_platform_name: optional_text(element, "ParentPlatformName"),
                parent_playlist_id: optional_text(element, "ParentPlaylistId"),
                platform_category_name: optional_text(element, "PlatformCategoryName"),
                platform_name: optional_text(element, "PlatformName"),
                playlist_id: optional_text(element, "PlaylistId"),
            };
            relationship.validate()?;
            Ok(relationship)
        })
        .collect()
}

fn load_game_controllers(path: &Path) -> Result<Vec<GameController>, StorageError> {
    let Some(root) = load_optional_root(path)? else {
        return Ok(Vec::new());
    };
    parse_game_controllers(&root)
}

fn parse_game_controllers(root: &Element) -> Result<Vec<GameController>, StorageError> {
    elements_named(root, "GameController")
        .map(|element| {
            let controller = GameController {
                id: required_text(element, "GameController", "Id")?,
                name: required_text(element, "GameController", "Name")?,
                category: text(element, "Category").unwrap_or_default(),
                associated_platforms: optional_text(element, "AssociatedPlatforms"),
            };
            controller.validate()?;
            Ok(controller)
        })
        .collect()
}

fn load_input_bindings(path: &Path) -> Result<Vec<InputBinding>, StorageError> {
    let Some(root) = load_optional_root(path)? else {
        return Ok(Vec::new());
    };
    parse_input_bindings(&root)
}

fn parse_input_bindings(root: &Element) -> Result<Vec<InputBinding>, StorageError> {
    elements_named(root, "InputBinding")
        .map(|element| {
            let binding = InputBinding {
                input_action: required_text(element, "InputBinding", "InputAction")?,
                controller_binding: text(element, "ControllerBinding").unwrap_or_default(),
                controller_hold_binding: text(element, "ControllerHoldBinding").unwrap_or_default(),
            };
            binding.validate()?;
            Ok(binding)
        })
        .collect()
}

fn load_ignored_games(path: &Path) -> Result<Vec<String>, StorageError> {
    let Some(root) = load_optional_root(path)? else {
        return Ok(Vec::new());
    };
    parse_ignored_games(&root)
}

fn parse_ignored_games(root: &Element) -> Result<Vec<String>, StorageError> {
    elements_named(root, "IgnoredGameId")
        .map(|element| required_text(element, "IgnoredGameId", "GameId"))
        .collect()
}

fn load_list_cache(path: &Path) -> Result<Vec<ListCacheItem>, StorageError> {
    let Some(root) = load_optional_root(path)? else {
        return Ok(Vec::new());
    };
    parse_list_cache(&root)
}

fn parse_list_cache(root: &Element) -> Result<Vec<ListCacheItem>, StorageError> {
    elements_named(root, "ListCacheItem")
        .map(|element| {
            let item = ListCacheItem {
                playlist_id: required_text(element, "ListCacheItem", "PlaylistId")?,
                launchbox_count: number_or_default(
                    element,
                    "ListCacheItem",
                    "LaunchBoxCount",
                    "unsigned integer",
                )?,
                launchbox_include_broken: bool_field(
                    element,
                    "ListCacheItem",
                    "LaunchBoxIncludeBroken",
                )?,
                launchbox_include_hidden: bool_field(
                    element,
                    "ListCacheItem",
                    "LaunchBoxIncludeHidden",
                )?,
                launchbox_exclude_games_missing_background_image: bool_field(
                    element,
                    "ListCacheItem",
                    "LaunchBoxExcludeGamesMissingBackgroundImage",
                )?,
                launchbox_exclude_games_missing_box_front_image: bool_field(
                    element,
                    "ListCacheItem",
                    "LaunchBoxExcludeGamesMissingBoxFrontImage",
                )?,
                launchbox_exclude_games_missing_clear_logo_image: bool_field(
                    element,
                    "ListCacheItem",
                    "LaunchBoxExcludeGamesMissingClearLogoImage",
                )?,
                launchbox_exclude_games_missing_screenshot_image: bool_field(
                    element,
                    "ListCacheItem",
                    "LaunchBoxExcludeGamesMissingScreenshotImage",
                )?,
                launchbox_exclude_games_missing_videos: bool_field(
                    element,
                    "ListCacheItem",
                    "LaunchBoxExcludeGamesMissingVideos",
                )?,
                big_box_count: number_or_default(
                    element,
                    "ListCacheItem",
                    "BigBoxCount",
                    "unsigned integer",
                )?,
                big_box_include_broken: bool_field(
                    element,
                    "ListCacheItem",
                    "BigBoxIncludeBroken",
                )?,
                big_box_include_hidden: bool_field(
                    element,
                    "ListCacheItem",
                    "BigBoxIncludeHidden",
                )?,
                big_box_exclude_games_missing_background_image: bool_field(
                    element,
                    "ListCacheItem",
                    "BigBoxExcludeGamesMissingBackgroundImage",
                )?,
                big_box_exclude_games_missing_box_front_image: bool_field(
                    element,
                    "ListCacheItem",
                    "BigBoxExcludeGamesMissingBoxFrontImage",
                )?,
                big_box_exclude_games_missing_clear_logo_image: bool_field(
                    element,
                    "ListCacheItem",
                    "BigBoxExcludeGamesMissingClearLogoImage",
                )?,
                big_box_exclude_games_missing_screenshot_image: bool_field(
                    element,
                    "ListCacheItem",
                    "BigBoxExcludeGamesMissingScreenshotImage",
                )?,
                big_box_exclude_games_missing_videos: bool_field(
                    element,
                    "ListCacheItem",
                    "BigBoxExcludeGamesMissingVideos",
                )?,
            };
            item.validate()?;
            Ok(item)
        })
        .collect()
}

fn load_settings(
    path: &Path,
    record_name: &'static str,
) -> Result<Option<FrontendSettings>, StorageError> {
    let Some(root) = load_optional_root(path)? else {
        return Ok(None);
    };
    parse_settings(path, &root, record_name).map(Some)
}

fn parse_settings(
    path: &Path,
    root: &Element,
    record_name: &'static str,
) -> Result<FrontendSettings, StorageError> {
    let records = elements_named(root, record_name).collect::<Vec<_>>();
    let record = exactly_one(path, record_name, &records)?;
    let mut entries = Vec::new();
    for field in record.children.iter().filter_map(XMLNode::as_element) {
        if field
            .children
            .iter()
            .any(|child| child.as_element().is_some())
        {
            return Err(StorageError::NestedSettingField {
                record: record_name.to_string(),
                field: field.name.clone(),
            });
        }
        entries.push(SettingEntry {
            key: field.name.clone(),
            value: field
                .get_text()
                .map(|value| value.into_owned())
                .unwrap_or_default(),
        });
    }
    let image_type_settings = elements_named(root, "ImageTypeSettings")
        .map(|element| {
            Ok(ImageTypeSetting {
                image_type: required_text(element, "ImageTypeSettings", "ImageType")?,
                is_default: bool_field(element, "ImageTypeSettings", "IsDefault")?,
                use_in_auto_imports: bool_field(element, "ImageTypeSettings", "UseInAutoImports")?,
            })
        })
        .collect::<Result<Vec<_>, StorageError>>()?;
    Ok(FrontendSettings {
        source_path: path.to_path_buf(),
        record_name: record_name.to_string(),
        entries,
        image_type_settings,
    })
}

pub(crate) fn validate_auxiliary_root(
    kind: AuxiliaryDocumentKind,
    path: &Path,
    root: &Element,
) -> Result<(), StorageError> {
    if root.name != "LaunchBox" {
        return Err(StorageError::InvalidRoot {
            path: path.to_path_buf(),
            actual: root.name.clone(),
        });
    }

    match kind {
        AuxiliaryDocumentKind::Playlist => {
            parse_playlist(path, root)?;
        }
        AuxiliaryDocumentKind::Emulators => {
            parse_emulators(path, root)?;
        }
        AuxiliaryDocumentKind::Platforms => {
            parse_platform_catalog(path, root)?;
        }
        AuxiliaryDocumentKind::Parents => {
            parse_parents(root)?;
        }
        AuxiliaryDocumentKind::GameControllers => {
            parse_game_controllers(root)?;
        }
        AuxiliaryDocumentKind::InputBindings => {
            parse_input_bindings(root)?;
        }
        AuxiliaryDocumentKind::ImportBlacklist => {
            parse_ignored_games(root)?;
        }
        AuxiliaryDocumentKind::ListCache => {
            parse_list_cache(root)?;
        }
        AuxiliaryDocumentKind::Settings => {
            parse_settings(path, root, "Settings")?;
        }
        AuxiliaryDocumentKind::BigBoxSettings => {
            parse_settings(path, root, "BigBoxSettings")?;
        }
    }
    Ok(())
}

fn xml_files(directory: &Path) -> Result<Vec<PathBuf>, StorageError> {
    Ok(fs::read_dir(directory)
        .map_err(|source| StorageError::Read {
            path: directory.to_path_buf(),
            source,
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("xml"))
        })
        .collect())
}

fn load_optional_root(path: &Path) -> Result<Option<Element>, StorageError> {
    path.is_file().then(|| load_root(path)).transpose()
}

fn load_root(path: &Path) -> Result<Element, StorageError> {
    let file = fs::File::open(path).map_err(|source| StorageError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let root = Element::parse(file).map_err(|source| StorageError::Parse {
        path: path.to_path_buf(),
        source,
    })?;
    if root.name != "LaunchBox" {
        return Err(StorageError::InvalidRoot {
            path: path.to_path_buf(),
            actual: root.name,
        });
    }
    Ok(root)
}

fn exactly_one<'a>(
    path: &Path,
    record: &'static str,
    elements: &[&'a Element],
) -> Result<&'a Element, StorageError> {
    match elements {
        [] => Err(StorageError::MissingDocumentRecord {
            path: path.to_path_buf(),
            record,
        }),
        [element] => Ok(element),
        _ => Err(StorageError::DuplicateDocumentRecord {
            path: path.to_path_buf(),
            record,
        }),
    }
}

fn elements_named<'a>(root: &'a Element, name: &'a str) -> impl Iterator<Item = &'a Element> {
    root.children
        .iter()
        .filter_map(XMLNode::as_element)
        .filter(move |element| element.name == name)
}

fn text(element: &Element, field: &str) -> Option<String> {
    element
        .get_child(field)
        .and_then(Element::get_text)
        .map(|value| value.into_owned())
}

fn optional_text(element: &Element, field: &str) -> Option<String> {
    text(element, field).filter(|value| !value.is_empty())
}

fn required_text(
    element: &Element,
    record: &'static str,
    field: &'static str,
) -> Result<String, StorageError> {
    optional_text(element, field).ok_or(StorageError::MissingRecordField { record, field })
}

fn bool_field(
    element: &Element,
    record: &'static str,
    field: &'static str,
) -> Result<bool, StorageError> {
    optional_bool_field(element, record, field).map(|value| value.unwrap_or(false))
}

fn optional_bool_field(
    element: &Element,
    record: &'static str,
    field: &'static str,
) -> Result<Option<bool>, StorageError> {
    let Some(value) = optional_text(element, field) else {
        return Ok(None);
    };
    if value.eq_ignore_ascii_case("true") {
        Ok(Some(true))
    } else if value.eq_ignore_ascii_case("false") {
        Ok(Some(false))
    } else {
        Err(StorageError::InvalidRecordField {
            record,
            field,
            expected: "boolean",
        })
    }
}

fn number_or_default<T>(
    element: &Element,
    record: &'static str,
    field: &'static str,
    expected: &'static str,
) -> Result<T, StorageError>
where
    T: FromStr + Default,
{
    optional_number(element, record, field, expected).map(|value| value.unwrap_or_default())
}

fn optional_number<T>(
    element: &Element,
    record: &'static str,
    field: &'static str,
    expected: &'static str,
) -> Result<Option<T>, StorageError>
where
    T: FromStr,
{
    let Some(value) = optional_text(element, field) else {
        return Ok(None);
    };
    value
        .parse()
        .map(Some)
        .map_err(|_| StorageError::InvalidRecordField {
            record,
            field,
            expected,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE_FILES: &[(&str, &str)] = &[
        (
            "Data/Platforms/Fixture Console.xml",
            include_str!("../../../fixtures/launchbox/Data/Platforms/Fixture Console.xml"),
        ),
        (
            "Data/Playlists/Fixture Playlist.xml",
            include_str!("../../../fixtures/launchbox/Data/Playlists/Fixture Playlist.xml"),
        ),
        (
            "Data/Emulators.xml",
            include_str!("../../../fixtures/launchbox/Data/Emulators.xml"),
        ),
        (
            "Data/Platforms.xml",
            include_str!("../../../fixtures/launchbox/Data/Platforms.xml"),
        ),
        (
            "Data/Parents.xml",
            include_str!("../../../fixtures/launchbox/Data/Parents.xml"),
        ),
        (
            "Data/GameControllers.xml",
            include_str!("../../../fixtures/launchbox/Data/GameControllers.xml"),
        ),
        (
            "Data/InputBindings.xml",
            include_str!("../../../fixtures/launchbox/Data/InputBindings.xml"),
        ),
        (
            "Data/ImportBlacklist.xml",
            include_str!("../../../fixtures/launchbox/Data/ImportBlacklist.xml"),
        ),
        (
            "Data/ListCache.xml",
            include_str!("../../../fixtures/launchbox/Data/ListCache.xml"),
        ),
        (
            "Data/Settings.xml",
            include_str!("../../../fixtures/launchbox/Data/Settings.xml"),
        ),
        (
            "Data/BigBoxSettings.xml",
            include_str!("../../../fixtures/launchbox/Data/BigBoxSettings.xml"),
        ),
    ];

    fn write_complete_fixture(root: &Path) {
        for (relative_path, contents) in FIXTURE_FILES {
            let path = root.join(relative_path);
            fs::create_dir_all(path.parent().expect("fixture parent"))
                .expect("create fixture directory");
            fs::write(path, contents).expect("write fixture document");
        }
    }

    #[test]
    fn loads_every_supported_data_file_family() {
        let directory = tempfile::tempdir().expect("temporary directory");
        write_complete_fixture(directory.path());

        let index = LaunchBoxDataIndex::load(directory.path()).expect("load complete fixture");
        assert_eq!(index.platforms().platforms().len(), 1);
        assert_eq!(index.platforms().games().count(), 3);
        assert_eq!(index.playlists().len(), 1);
        assert_eq!(index.playlist_filters().count(), 1);
        assert_eq!(index.playlist_games().count(), 1);

        let emulators = index
            .emulator_configuration()
            .expect("emulator configuration");
        assert_eq!(emulators.emulators.len(), 1);
        assert_eq!(emulators.platforms.len(), 1);
        assert_eq!(emulators.platforms[0].auto_extract, None);
        assert_eq!(emulators.emulators[0].startup_load_delay, 250);

        let catalog = index.platform_catalog().expect("platform catalog");
        assert_eq!(catalog.platforms.len(), 1);
        assert_eq!(catalog.categories.len(), 1);
        assert_eq!(catalog.folders.len(), 1);
        assert_eq!(catalog.platforms[0].release_date.as_deref(), Some("1999"));

        assert_eq!(index.parents().len(), 2);
        assert_eq!(index.game_controllers().len(), 1);
        assert_eq!(index.input_bindings().len(), 1);
        assert_eq!(index.ignored_game_ids(), &["fixture-ignored-game"]);
        assert_eq!(index.list_cache().len(), 1);
    }

    #[test]
    fn settings_keep_all_scalar_entries_including_empty_values() {
        let directory = tempfile::tempdir().expect("temporary directory");
        write_complete_fixture(directory.path());

        let index = LaunchBoxDataIndex::load(directory.path()).expect("load complete fixture");
        let settings = index.settings().expect("LaunchBox settings");
        assert_eq!(settings.entries.len(), 3);
        assert_eq!(settings.get("Theme"), Some("Fixture Theme"));
        assert_eq!(settings.get_bool("DebugLog"), Some(false));
        assert_eq!(settings.get("EmptyValue"), Some(""));
        assert_eq!(settings.image_type_settings.len(), 1);

        let big_box = index.big_box_settings().expect("BigBox settings");
        assert_eq!(big_box.entries.len(), 2);
        assert_eq!(big_box.get_bool("EnableAttractMode"), Some(true));
    }
}
