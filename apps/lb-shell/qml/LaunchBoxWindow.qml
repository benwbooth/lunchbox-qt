import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Layouts
import QtQml.Models
import LaunchBoxPort

ApplicationWindow {
    id: window
    width: 1280
    height: 800
    minimumWidth: 900
    minimumHeight: 600
    visible: true
    title: controller.library_name.length > 0
           ? controller.library_name + " — LaunchBox Port"
           : "LaunchBox Port"
    color: "#14171c"

    property string selectedPlatform: ""
    property string selectedNavigationKind: "all"
    property string selectedNavigationKey: ""
    property string selectedNavigationName: "All Games"
    property bool smokeTest: Qt.application.arguments.indexOf("--smoke-test") >= 0
    property bool loadSmokeTest: Qt.application.arguments.indexOf("--load-smoke-test") >= 0
    property bool editSmokeTest: Qt.application.arguments.indexOf("--edit-smoke-test") >= 0
    property bool crudSmokeTest: Qt.application.arguments.indexOf("--crud-smoke-test") >= 0
    property bool platformCrudSmokeTest:
        Qt.application.arguments.indexOf("--platform-crud-smoke-test") >= 0
    property bool emulatorCrudSmokeTest:
        Qt.application.arguments.indexOf("--emulator-crud-smoke-test") >= 0
    property bool categoryCrudSmokeTest:
        Qt.application.arguments.indexOf("--category-crud-smoke-test") >= 0
    property bool playlistCrudSmokeTest:
        Qt.application.arguments.indexOf("--playlist-crud-smoke-test") >= 0
    property bool importSmokeTest:
        Qt.application.arguments.indexOf("--import-smoke-test") >= 0
    property bool launchSmokeTest: Qt.application.arguments.indexOf("--launch-smoke-test") >= 0
    property bool launchLifecycleSmokeTest:
        Qt.application.arguments.indexOf("--launch-lifecycle-smoke-test") >= 0
    property bool launchLifecycleShortProcess:
        Qt.application.arguments.indexOf("--launch-lifecycle-short-process") >= 0
    property bool pathMappingSmokeTest:
        Qt.application.arguments.indexOf("--path-mapping-smoke-test") >= 0
    property bool gameGroupingSmokeTest:
        Qt.application.arguments.indexOf("--game-grouping-smoke-test") >= 0
    property int loadHeartbeat: 0
    property int smokePhase: 0
    property int editSmokePhase: 0
    property bool editSmokeFinished: false
    property int crudSmokePhase: 0
    property int crudBlockedReferences: 0
    property string crudAddedGameId: ""
    property bool crudSmokeFinished: false
    property bool additionalApplicationCrudSmokeTest:
        Qt.application.arguments.indexOf(
            "--additional-application-crud-smoke-test") >= 0
    property int additionalApplicationCrudSmokePhase: 0
    property bool additionalApplicationCrudSmokeFinished: false
    property string additionalApplicationCrudSmokeAddedId: ""
    property bool additionalApplicationDefaultSmokeTest:
        Qt.application.arguments.indexOf(
            "--additional-application-default-smoke-test") >= 0
    property int additionalApplicationDefaultSmokePhase: 0
    property bool additionalApplicationDefaultSmokeFinished: false
    property bool gameSaveMetadataSmokeTest:
        Qt.application.arguments.indexOf(
            "--game-save-metadata-smoke-test") >= 0
    property int gameSaveMetadataSmokePhase: 0
    property bool gameSaveMetadataSmokeFinished: false
    property bool gameSaveBackupSmokeTest:
        Qt.application.arguments.indexOf(
            "--game-save-backup-smoke-test") >= 0
    property int gameSaveBackupSmokePhase: 0
    property bool gameSaveBackupSmokeFinished: false
    property bool pcsx2SaveBackupSmokeTest:
        Qt.application.arguments.indexOf(
            "--pcsx2-save-backup-smoke-test") >= 0
    property int pcsx2SaveBackupSmokePhase: 0
    property bool pcsx2SaveBackupSmokeFinished: false
    property bool pcsx2SaveLifecycleSmokeTest:
        Qt.application.arguments.indexOf(
            "--pcsx2-save-lifecycle-smoke-test") >= 0
    property int pcsx2SaveLifecycleSmokePhase: 0
    property bool pcsx2SaveLifecycleSmokeFinished: false
    property bool dolphinWiiSaveLifecycleSmokeTest:
        Qt.application.arguments.indexOf(
            "--dolphin-wii-save-lifecycle-smoke-test") >= 0
    property int dolphinWiiSaveLifecycleSmokePhase: 0
    property bool dolphinWiiSaveLifecycleSmokeFinished: false
    property bool gameSaveDeleteSmokeTest:
        Qt.application.arguments.indexOf(
            "--game-save-delete-smoke-test") >= 0
    property int gameSaveDeleteSmokePhase: 0
    property bool gameSaveDeleteSmokeFinished: false
    property bool gameSaveActiveDeleteSmokeTest:
        Qt.application.arguments.indexOf(
            "--game-save-active-delete-smoke-test") >= 0
    property int gameSaveActiveDeleteSmokePhase: 0
    property bool gameSaveActiveDeleteSmokeFinished: false
    property bool gameSaveRestoreSmokeTest:
        Qt.application.arguments.indexOf(
            "--game-save-restore-smoke-test") >= 0
    property int gameSaveRestoreSmokePhase: 0
    property bool gameSaveRestoreSmokeFinished: false
    property bool gameSaveSaturnRestoreSmokeTest:
        Qt.application.arguments.indexOf(
            "--game-save-saturn-restore-smoke-test") >= 0
    property int gameSaveSaturnRestoreSmokePhase: 0
    property bool gameSaveSaturnRestoreSmokeFinished: false
    property bool retroarchSaveScanSmokeTest:
        Qt.application.arguments.indexOf(
            "--retroarch-save-scan-smoke-test") >= 0
    property int retroarchSaveScanSmokePhase: 0
    property bool retroarchSaveScanSmokeFinished: false
    property bool dolphinSaveScanSmokeTest:
        Qt.application.arguments.indexOf(
            "--dolphin-save-scan-smoke-test") >= 0
    property int dolphinSaveScanSmokePhase: 0
    property bool dolphinSaveScanSmokeFinished: false
    property bool pcsx2SaveScanSmokeTest:
        Qt.application.arguments.indexOf(
            "--pcsx2-save-scan-smoke-test") >= 0
    property int pcsx2SaveScanSmokePhase: 0
    property bool pcsx2SaveScanSmokeFinished: false
    property int platformCrudSmokePhase: 0
    property int platformCrudBlockedReferences: 0
    property string platformCrudAddedGameId: ""
    property bool platformCrudSmokeFinished: false
    property int emulatorCrudSmokePhase: 0
    property int emulatorCrudBlockedReferences: 0
    property string emulatorCrudAddedId: ""
    property int emulatorCrudInitialRevision: -1
    property bool emulatorCrudSmokeFinished: false
    property bool emulatorDiscoverySmokeTest:
        Qt.application.arguments.indexOf("--emulator-discovery-smoke-test") >= 0
    property int emulatorDiscoverySmokePhase: 0
    property int emulatorDiscoveryCandidateIndex: -1
    property int emulatorDiscoveryInitialEmulatorRevision: -1
    property int emulatorDiscoveryInitialRevision: -1
    property bool emulatorDiscoverySmokeFinished: false
    property bool emulatorBiosSmokeTest:
        Qt.application.arguments.indexOf("--emulator-bios-smoke-test") >= 0
    property int emulatorBiosSmokePhase: 0
    property int emulatorBiosInitialRevision: -1
    property bool emulatorBiosSmokeFinished: false
    property bool emulatorInstallSmokeTest:
        Qt.application.arguments.indexOf("--emulator-install-smoke-test") >= 0
    property int emulatorInstallSmokePhase: 0
    property int emulatorInstallInitialRevision: -1
    property bool emulatorInstallSmokeFinished: false
    property bool emulatorRemoveSmokeTest:
        Qt.application.arguments.indexOf("--emulator-remove-smoke-test") >= 0
    property int emulatorRemoveSmokePhase: 0
    property int emulatorRemoveInitialRevision: -1
    property bool emulatorRemoveSmokeFinished: false
    property int categoryCrudSmokePhase: 0
    property bool categoryCrudSmokeFinished: false
    property int playlistCrudSmokePhase: 0
    property bool playlistCrudSmokeFinished: false
    property int importSmokePhase: 0
    property bool importSmokeFinished: false
    property string playlistCrudParentId: ""
    property string playlistCrudChildId: ""
    property int launchSmokePhase: 0
    property bool launchSmokeFinished: false
    property int launchLifecycleSmokePhase: 0
    property bool launchLifecycleSmokeFinished: false
    property bool launchLifecycleStartupVisibleSeen: false
    property bool launchLifecyclePrimaryStartedSeen: false
    property bool launchLifecycleDismissedBeforeExit: false
    property bool launchLifecycleScreenshotRequested: false
    property bool launchLifecycleShutdownVisibleSeen: false
    property bool launchLifecycleShutdownScreenshotRequested: false
    property double launchLifecycleStartupPresentedAt: 0
    property double launchLifecyclePrimaryStartedAt: 0
    property double launchLifecycleStartupDismissedAt: 0
    property double launchLifecycleShutdownPresentedAt: 0
    property double launchLifecycleShutdownDismissedAt: 0
    property string launchLifecycleScreenshotPath:
        argumentValue("--launch-lifecycle-screenshot")
    property string launchLifecycleShutdownScreenshotPath:
        argumentValue("--launch-lifecycle-shutdown-screenshot")
    property bool launchPauseSmokeTest:
        Qt.application.arguments.indexOf("--launch-pause-smoke-test") >= 0
    property int launchPauseSmokePhase: 0
    property bool launchPauseSmokeFinished: false
    property bool launchPauseVisibleSeen: false
    property bool launchPauseProcessSuspendedSeen: false
    property bool launchPauseResumeSeen: false
    property bool launchPauseScreenshotRequested: false
    property string launchPauseScreenshotPath:
        argumentValue("--launch-pause-screenshot")
    property int pathMappingSmokePhase: 0
    property bool pathMappingSmokeFinished: false
    property int gameGroupingSmokePhase: 0
    property bool gameGroupingSmokeFinished: false
    property string launchSmokeGameId: {
        const requested = argumentValue("--launch-game-id")
        return requested.length > 0 ? requested : "fixture-racer"
    }
    property string launchSmokeAdditionalApplicationId:
        argumentValue("--launch-additional-application-id")

    function platformName(row) { return controller.platform_name_at(row) }
    function platformGameCount(row) { return controller.platform_game_count_at(row) }
    function platformIndex(name) {
        for (let index = 0; index < controller.platform_entry_count; ++index) {
            if (platformName(index) === name)
                return index
        }
        return -1
    }

    function navigationIndex(kind, key) {
        for (let index = 0; index < controller.navigation_entry_count; ++index) {
            if (controller.navigation_entry_kind_at(index) === kind
                    && controller.navigation_entry_key_at(index) === key)
                return index
        }
        return -1
    }

    function applyCurrentFilter() {
        if (selectedNavigationKind === "category")
            controller.apply_category_filter(searchField.text, selectedNavigationKey)
        else if (selectedNavigationKind === "playlist")
            controller.apply_playlist_filter(searchField.text, selectedNavigationKey)
        else
            controller.apply_filters(searchField.text, selectedPlatform)
    }

    function verifyModelRoles(index, gameId, gameTitle, gamePlatform, gameFavorite,
                              gameCompleted, gamePlayCount, gameStarRating,
                              gameAdditionalApplicationCount, gameSortTitle,
                              gameNotes, gameDeveloper, gameGenre, gameMaxPlayers,
                              gamePlayMode, gameProgress, gamePublisher, gameRating,
                              gameRegion, gameReleaseDate, gameReleaseType, gameSeries,
                              gameSource, gameStatus, gameVersion, gameWikipediaUrl,
                              gameApplicationPath, gameCommandLine, gameEmulatorId,
                              gameUseDosBox, gameCustomDosBoxVersionPath,
                              gameDosBoxConfigurationPath, gameUseScummVm,
                              gameScummVmAspectCorrection, gameScummVmFullscreen,
                              gameScummVmGameDataFolderPath, gameScummVmGameType,
                              rowCount) {
        if (!smokeTest || index !== 0)
            return
        const expectedTitle = smokePhase === 0 ? "Fixture Adventure" : "Fixture Racer"
        const expectedFavorite = smokePhase === 0
        const expectedCompleted = smokePhase === 1
        const expectedId = smokePhase === 0 ? "fixture-adventure" : "fixture-racer"
        const expectedPlayCount = smokePhase === 0 ? 3 : 8
        const expectedStarRating = smokePhase === 0 ? 4 : 5
        const expectedRows = smokePhase === 0 ? 3 : 1
        const expectedAdditionalApplicationCount = smokePhase === 0 ? 1 : 0
        const metadataMatches = smokePhase === 0
            ? gameSortTitle === "Adventure, Fixture"
              && gameNotes === "A synthetic adventure used to verify LaunchBox XML compatibility."
              && gameDeveloper === "Fixture Labs" && gameGenre === "Adventure"
              && gameMaxPlayers === 4 && gamePlayMode === "Cooperative"
              && gameProgress === "25%" && gamePublisher === "Fixture Publishing"
              && gameRating === "E" && gameRegion === "North America"
              && gameReleaseDate === "1999-03-04" && gameReleaseType === "Released"
              && gameSeries === "Fixture Saga" && gameSource === "Fixture Store"
              && gameStatus === "Ready" && gameVersion === "1.2"
              && gameWikipediaUrl === "https://example.invalid/wiki/fixture-adventure"
            : gameSortTitle === "" && gameNotes === "" && gameDeveloper === ""
              && gameGenre === "" && gameMaxPlayers === 0 && gamePlayMode === ""
              && gameProgress === "" && gamePublisher === "" && gameRating === ""
              && gameRegion === "" && gameReleaseDate === "" && gameReleaseType === ""
              && gameSeries === "" && gameSource === "" && gameStatus === ""
              && gameVersion === "" && gameWikipediaUrl === ""
        const launchConfigurationMatches = smokePhase === 0
            ? gameApplicationPath === "Games\\Fixture Adventure\\adventure.rom"
              && gameCommandLine === "--region auto"
              && gameEmulatorId === "fixture-emulator" && gameUseDosBox
              && gameCustomDosBoxVersionPath === "ThirdParty\\DOSBox\\fixture"
              && gameDosBoxConfigurationPath === "Config\\dosbox-fixture.conf"
              && gameUseScummVm && gameScummVmAspectCorrection
              && gameScummVmFullscreen
              && gameScummVmGameDataFolderPath
                    === "Games\\Fixture Adventure\\ScummVM"
              && gameScummVmGameType === "fixture-scumm-id"
            : gameApplicationPath === "Games\\Fixture Racer\\racer.rom"
              && gameCommandLine === "" && gameEmulatorId === ""
              && !gameUseDosBox && gameCustomDosBoxVersionPath === ""
              && gameDosBoxConfigurationPath === "" && !gameUseScummVm
              && !gameScummVmAspectCorrection && !gameScummVmFullscreen
              && gameScummVmGameDataFolderPath === ""
              && gameScummVmGameType === ""
        if (gameId !== expectedId || gameTitle !== expectedTitle
                || gamePlatform !== "Fixture Console"
                || gameFavorite !== expectedFavorite || gameCompleted !== expectedCompleted
                || gamePlayCount !== expectedPlayCount || gameStarRating !== expectedStarRating
                || gameAdditionalApplicationCount !== expectedAdditionalApplicationCount
                || !metadataMatches
                || !launchConfigurationMatches
                || rowCount !== expectedRows) {
            console.error("MODEL_ROLE_SMOKE_FAILED id=" + gameId
                          + " title=" + gameTitle
                          + " platform=" + gamePlatform
                          + " favorite=" + gameFavorite
                          + " completed=" + gameCompleted
                          + " developer=" + gameDeveloper
                          + " genre=" + gameGenre
                          + " applicationPath=" + gameApplicationPath
                          + " emulator=" + gameEmulatorId
                          + " additionalApps=" + gameAdditionalApplicationCount
                          + " rows=" + rowCount)
            Qt.exit(4)
        } else if (smokePhase === 0) {
            smokePhase = 1
            Qt.callLater(function() { controller.apply_filters("Racer", "") })
        } else {
            controller.report_model_smoke_success(rowCount)
            Qt.quit()
        }
    }

    function verifyEditState(index, gameId, gameTitle, gameSortTitle, gameNotes,
                             gameDeveloper, gameGenre, gameMaxPlayers, gamePlayMode,
                             gameProgress, gamePublisher, gameRating, gameRegion,
                             gameReleaseDate, gameReleaseType, gameSeries, gameSource,
                             gameStatus, gameVersion, gameWikipediaUrl, gameFavorite,
                             gameCompleted, gameStarRating, gameApplicationPath,
                             gameCommandLine, gameEmulatorId, gameUseDosBox,
                             gameCustomDosBoxVersionPath, gameDosBoxConfigurationPath,
                             gameUseScummVm, gameScummVmAspectCorrection,
                             gameScummVmFullscreen, gameScummVmGameDataFolderPath,
                             gameScummVmGameType) {
        if (!editSmokeTest || editSmokeFinished || index !== 0)
            return
        if (editSmokePhase === 0) {
            if (gameId !== "fixture-adventure" || gameTitle !== "Fixture Adventure"
                    || !gameFavorite || gameCompleted || gameStarRating !== 4) {
                console.error("EDIT_SMOKE_BAD_INITIAL_STATE id=" + gameId
                              + " title=" + gameTitle
                              + " favorite=" + gameFavorite
                              + " completed=" + gameCompleted
                              + " rating=" + gameStarRating)
                Qt.exit(5)
                return
            }
            editSmokePhase = 1
            Qt.callLater(function() {
                gameEditor.smokeSaveState(
                    index, gameId, gameTitle, gameSortTitle, gameNotes,
                    gameDeveloper, gameGenre, gameMaxPlayers, gamePlayMode,
                    gameProgress, gamePublisher, gameRating, gameRegion,
                    gameReleaseDate, gameReleaseType, gameSeries, gameSource,
                    gameStatus, gameVersion, gameWikipediaUrl, gameApplicationPath,
                    gameCommandLine, gameEmulatorId, gameUseDosBox,
                    gameCustomDosBoxVersionPath, gameDosBoxConfigurationPath,
                    gameUseScummVm, gameScummVmAspectCorrection,
                    gameScummVmFullscreen, gameScummVmGameDataFolderPath,
                    gameScummVmGameType)
            })
        } else if (editSmokePhase === 1 && !controller.writing && !gameFavorite
                   && gameCompleted && gameStarRating === 2) {
            if (!controller.report_state_edit_smoke_success(gameId)) {
                console.error("EDIT_SMOKE_NOTIFICATION_FAILED")
                Qt.exit(5)
                return
            }
            editSmokePhase = 2
            Qt.callLater(function() {
                controller.apply_filters("Fixture Adventure", "")
            })
        }
    }

    function argumentValue(flag) {
        const index = Qt.application.arguments.indexOf(flag)
        return index >= 0 && index + 1 < Qt.application.arguments.length
               ? Qt.application.arguments[index + 1] : ""
    }

    LibraryController {
        id: controller
    }

    Connections {
        target: controller

        function onStartup_screen_activeChanged() {
            if (window.launchLifecycleSmokeTest
                    && controller.startup_screen_active
                    && !controller.startup_screen_primary_started) {
                window.launchLifecycleStartupVisibleSeen = true
                if (window.launchLifecycleStartupPresentedAt === 0)
                    window.launchLifecycleStartupPresentedAt = Date.now()
                if (window.launchLifecycleScreenshotPath.length > 0
                        && !window.launchLifecycleScreenshotRequested) {
                    window.launchLifecycleScreenshotRequested = true
                    Qt.callLater(function() {
                        launchStartupOverlay.grabToImage(function(result) {
                            if (!result.saveToFile(
                                    window.launchLifecycleScreenshotPath))
                                console.error(
                                    "LAUNCH_LIFECYCLE_SCREENSHOT_SAVE_FAILED path="
                                    + window.launchLifecycleScreenshotPath)
                        })
                    })
                }
            } else if (window.launchLifecycleSmokeTest
                       && !controller.startup_screen_active
                       && window.launchLifecycleStartupPresentedAt > 0
                       && window.launchLifecycleStartupDismissedAt === 0) {
                window.launchLifecycleStartupDismissedAt = Date.now()
            }
        }

        function onStartup_screen_primary_startedChanged() {
            if (window.launchLifecycleSmokeTest
                    && controller.startup_screen_active
                    && controller.startup_screen_primary_started) {
                window.launchLifecyclePrimaryStartedSeen = true
                if (window.launchLifecyclePrimaryStartedAt === 0)
                    window.launchLifecyclePrimaryStartedAt = Date.now()
            }
        }

        function onShutdown_screen_activeChanged() {
            if (window.launchLifecycleSmokeTest
                    && controller.shutdown_screen_active) {
                window.launchLifecycleShutdownVisibleSeen = true
                if (window.launchLifecycleShutdownPresentedAt === 0)
                    window.launchLifecycleShutdownPresentedAt = Date.now()
                if (window.launchLifecycleShutdownScreenshotPath.length > 0
                        && !window.launchLifecycleShutdownScreenshotRequested) {
                    window.launchLifecycleShutdownScreenshotRequested = true
                    Qt.callLater(function() {
                        launchShutdownOverlay.grabToImage(function(result) {
                            if (!result.saveToFile(
                                    window.launchLifecycleShutdownScreenshotPath))
                                console.error(
                                    "LAUNCH_LIFECYCLE_SHUTDOWN_SCREENSHOT_SAVE_FAILED path="
                                    + window.launchLifecycleShutdownScreenshotPath)
                        })
                    })
                }
            } else if (window.launchLifecycleSmokeTest
                       && window.launchLifecycleShutdownPresentedAt > 0
                       && window.launchLifecycleShutdownDismissedAt === 0) {
                window.launchLifecycleShutdownDismissedAt = Date.now()
            }
        }

        function onPause_screen_activeChanged() {
            if (!window.launchPauseSmokeTest)
                return
            if (controller.pause_screen_active) {
                window.launchPauseVisibleSeen = true
                window.launchPauseProcessSuspendedSeen =
                    controller.pause_screen_process_suspended
                if (window.launchPauseScreenshotPath.length > 0
                        && !window.launchPauseScreenshotRequested) {
                    window.launchPauseScreenshotRequested = true
                    Qt.callLater(function() {
                        launchPauseOverlay.grabToImage(function(result) {
                            if (!result.saveToFile(
                                    window.launchPauseScreenshotPath))
                                console.error(
                                    "LAUNCH_PAUSE_SCREENSHOT_SAVE_FAILED path="
                                    + window.launchPauseScreenshotPath)
                        })
                    })
                }
            } else if (window.launchPauseVisibleSeen) {
                window.launchPauseResumeSeen = true
            }
        }
    }

    Component.onCompleted: {
        controller.configure_frontend(false)
        if (!controller.initialize_host_path_mappings()) {
            if (pathMappingSmokeTest) {
                console.error("PATH_MAPPING_SMOKE_INITIALIZE_FAILED status="
                              + controller.status_message)
                Qt.exit(8)
            }
            return
        }
        const library = argumentValue("--library")
        if (library.length > 0)
            controller.load_library(library)
        else
            controller.load_fixture()
    }

    Timer {
        interval: 25
        repeat: true
        running: window.pathMappingSmokeTest && !window.pathMappingSmokeFinished
        onTriggered: {
            const hostRoot = window.argumentValue("--path-mapping-host-root")
            if (hostRoot.length === 0) {
                console.error("PATH_MAPPING_SMOKE_MISSING_HOST_ROOT")
                Qt.exit(8)
                return
            }
            if (window.pathMappingSmokePhase === 0) {
                if (controller.path_mapping_count !== 0) {
                    console.error("PATH_MAPPING_SMOKE_NOT_ISOLATED count="
                                  + controller.path_mapping_count)
                    Qt.exit(8)
                    return
                }
                window.pathMappingSmokePhase = 1
                if (!controller.save_windows_drive_mapping("Z", hostRoot)) {
                    console.error("PATH_MAPPING_SMOKE_DRIVE_SAVE_FAILED status="
                                  + controller.status_message)
                    Qt.exit(8)
                }
            } else if (window.pathMappingSmokePhase === 1
                       && controller.path_mapping_count === 1) {
                if (controller.path_mapping_kind_at(0) !== "Drive"
                        || controller.path_mapping_windows_root_at(0) !== "Z:\\"
                        || controller.path_mapping_host_root_at(0) !== hostRoot) {
                    console.error("PATH_MAPPING_SMOKE_BAD_DRIVE_ENTRY")
                    Qt.exit(8)
                    return
                }
                window.pathMappingSmokePhase = 2
                if (!controller.save_windows_unc_mapping(
                        "fixture-server", "roms", hostRoot)) {
                    console.error("PATH_MAPPING_SMOKE_UNC_SAVE_FAILED status="
                                  + controller.status_message)
                    Qt.exit(8)
                }
            } else if (window.pathMappingSmokePhase === 2
                       && controller.path_mapping_count === 2) {
                if (controller.path_mapping_kind_at(1) !== "UNC"
                        || controller.path_mapping_windows_root_at(1)
                           !== "\\\\fixture-server\\roms"
                        || controller.path_mapping_host_root_at(1) !== hostRoot) {
                    console.error("PATH_MAPPING_SMOKE_BAD_UNC_ENTRY")
                    Qt.exit(8)
                    return
                }
                window.pathMappingSmokePhase = 3
                if (!controller.remove_path_mapping(1)) {
                    console.error("PATH_MAPPING_SMOKE_REMOVE_FAILED status="
                                  + controller.status_message)
                    Qt.exit(8)
                }
            } else if (window.pathMappingSmokePhase === 3
                       && controller.path_mapping_count === 1) {
                if (!controller.report_path_mapping_smoke_success(1)) {
                    console.error("PATH_MAPPING_SMOKE_CONTRACT_FAILED")
                    Qt.exit(8)
                    return
                }
                window.pathMappingSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 10000
        running: window.pathMappingSmokeTest && !window.pathMappingSmokeFinished
        onTriggered: {
            console.error("PATH_MAPPING_SMOKE_TIMEOUT phase="
                          + window.pathMappingSmokePhase
                          + " status=" + controller.status_message)
            Qt.exit(8)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.gameGroupingSmokeTest
                 && !window.gameGroupingSmokeFinished
        onTriggered: {
            const rootId = "fixture-adventure"
            if (window.gameGroupingSmokePhase === 0
                    && !controller.loading && !controller.writing
                    && controller.library_path.length > 0
                    && controller.game_count === 3) {
                const row = controller.row_for_game_id(rootId)
                if (row < 0) {
                    console.error("GAME_GROUPING_SMOKE_MISSING_ROOT")
                    Qt.exit(28)
                    return
                }
                window.gameGroupingSmokePhase = 1
                gameCombineDialog.smokeCombine(
                    row, rootId, "Fixture Adventure", "fixture-racer")
            } else if (window.gameGroupingSmokePhase === 1
                       && !controller.loading && !controller.writing
                       && controller.game_grouping_revision === 1) {
                const row = controller.row_for_game_id(rootId)
                if (controller.last_game_grouping_operation !== "combine"
                        || controller.last_game_grouping_root_id !== rootId
                        || controller.last_game_grouping_removed_count !== 1
                        || controller.last_game_grouping_created_count !== 0
                        || controller.game_count !== 2 || row < 0
                        || controller.additional_application_count(row, rootId) !== 3) {
                    console.error("GAME_GROUPING_SMOKE_BAD_COMBINE_STATE status="
                                  + controller.status_message)
                    Qt.exit(28)
                    return
                }
                window.gameGroupingSmokePhase = 2
                gameExpandConfirmation.smokeExpand(
                    row, rootId, "Fixture Adventure")
            } else if (window.gameGroupingSmokePhase === 2
                       && !controller.loading && !controller.writing
                       && controller.game_grouping_revision === 2) {
                if (!controller.report_game_grouping_smoke_success(rootId)) {
                    console.error("GAME_GROUPING_SMOKE_MODEL_CONTRACT_FAILED")
                    Qt.exit(28)
                    return
                }
                window.gameGroupingSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 30000
        running: window.gameGroupingSmokeTest
                 && !window.gameGroupingSmokeFinished
        onTriggered: {
            console.error("GAME_GROUPING_SMOKE_TIMEOUT phase="
                          + window.gameGroupingSmokePhase
                          + " revision=" + controller.game_grouping_revision
                          + " status=" + controller.status_message)
            Qt.exit(28)
        }
    }

    Timer {
        interval: 3000
        running: window.smokeTest
        onTriggered: {
            console.error("MODEL_ROLE_SMOKE_TIMEOUT rows=" + gameGrid.count)
            Qt.exit(4)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.importSmokeTest && !window.importSmokeFinished
        onTriggered: {
            if (window.importSmokePhase === 0 && !controller.loading
                    && controller.library_path.length > 0
                    && controller.game_count === 3) {
                const first = window.argumentValue("--import-rom-1")
                const second = window.argumentValue("--import-rom-2")
                const third = window.argumentValue("--import-rom-3")
                const fourth = window.argumentValue("--import-rom-4")
                if (first.length === 0 || second.length === 0
                        || third.length === 0 || fourth.length === 0) {
                    console.error("IMPORT_SMOKE_MISSING_SOURCE_ARGUMENTS")
                    Qt.exit(12)
                    return
                }
                window.importSmokePhase = 1
                romImportDialog.smokePrepare(
                    [first, second, third, fourth], "Fixture Console")
            } else if (window.importSmokePhase === 1
                       && !controller.import_scanning
                       && controller.import_preview_json.length > 0) {
                const preview = JSON.parse(controller.import_preview_json)
                let partialRow = null
                let versionRow = null
                for (let index = 0; index < preview.rows.length; ++index) {
                    const row = preview.rows[index]
                    if (row.metadata_match_kind === "partial")
                        partialRow = row
                    else if (row.metadata_match_kind === "exact")
                        versionRow = row
                }
                if (preview.rows.length !== 2 || partialRow === null
                        || partialRow.metadata_candidate_count !== 2
                        || partialRow.metadata_candidates.length !== 2
                        || partialRow.metadata_candidates[0].database_id !== 4242
                        || partialRow.manual_candidate_count !== 1
                        || partialRow.manual === null
                        || partialRow.manual.stored_path
                           !== "Games\\Fixture Console\\Fixture Sag\\Fixture Sag (USA) - (Disc 1 of 2).pdf"
                        || partialRow.additional_roms.length !== 1
                        || versionRow === null
                        || versionRow.metadata_candidate_count !== 1
                        || versionRow.metadata.database_id !== 4242
                        || versionRow.version !== "(USA)"
                        || versionRow.region !== "North America"
                        || versionRow.additional_roms.length !== 1
                        || versionRow.additional_roms[0].version
                           !== "(World) (Rev 1)"
                        || versionRow.additional_roms[0].region !== "World") {
                    console.error("IMPORT_SMOKE_MANUAL_PREVIEW_CONTRACT_FAILED")
                    Qt.exit(12)
                    return
                }
                window.importSmokePhase = 2
                romImportDialog.smokeSubmitPreview()
            } else if (window.importSmokePhase === 2
                       && !controller.writing
                       && controller.last_import_count === 2) {
                if (!controller.report_import_smoke_success(2, 7, 0)) {
                    console.error("IMPORT_SMOKE_MODEL_CONTRACT_FAILED")
                    Qt.exit(12)
                    return
                }
                window.importSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 20000
        running: window.importSmokeTest && !window.importSmokeFinished
        onTriggered: {
            console.error("IMPORT_SMOKE_TIMEOUT phase=" + window.importSmokePhase
                          + " status=" + controller.status_message)
            Qt.exit(12)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.crudSmokeTest && !window.crudSmokeFinished
        onTriggered: {
            if (window.crudSmokePhase === 0 && !controller.loading
                    && controller.library_path.length > 0
                    && controller.game_count === 3) {
                const row = controller.row_for_game_id("fixture-adventure")
                if (row < 0) {
                    console.error("CRUD_SMOKE_MISSING_REFERENCED_GAME")
                    Qt.exit(6)
                    return
                }
                window.crudSmokePhase = 1
                controller.delete_game(row, "fixture-adventure")
            } else if (window.crudSmokePhase === 1 && !controller.writing
                       && controller.delete_blocker_count > 0) {
                window.crudBlockedReferences = controller.delete_blocker_count
                window.crudSmokePhase = 2
                controller.add_game("Added Fixture", "Games\\Added\\added.rom",
                                    "Fixture Console")
            } else if (window.crudSmokePhase === 2 && !controller.writing
                       && controller.game_count === 4
                       && controller.last_added_game_id.length > 0) {
                window.crudAddedGameId = controller.last_added_game_id
                const addedRow = controller.row_for_game_id(window.crudAddedGameId)
                if (addedRow < 0) {
                    console.error("CRUD_SMOKE_INSERT_NOT_VISIBLE")
                    Qt.exit(6)
                    return
                }
                window.crudSmokePhase = 3
                controller.delete_game(addedRow, window.crudAddedGameId)
            } else if (window.crudSmokePhase === 3 && !controller.writing
                       && controller.game_count === 3) {
                if (!controller.report_crud_smoke_success(
                        window.crudAddedGameId, window.crudBlockedReferences)) {
                    console.error("CRUD_SMOKE_MODEL_CONTRACT_FAILED")
                    Qt.exit(6)
                    return
                }
                window.crudSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 15000
        running: window.crudSmokeTest && !window.crudSmokeFinished
        onTriggered: {
            console.error("CRUD_SMOKE_TIMEOUT phase=" + window.crudSmokePhase
                          + " status=" + controller.status_message)
            Qt.exit(6)
        }
    }

    Timer {
        interval: 10
        repeat: true
        running: window.launchPauseSmokeTest
                 && !window.launchPauseSmokeFinished
        onTriggered: {
            if (window.launchPauseSmokePhase === 0 && !controller.loading
                    && controller.library_path.length > 0
                    && controller.game_count > 0) {
                const row = controller.row_for_game_id("fixture-racer")
                if (row < 0) {
                    console.error("LAUNCH_PAUSE_SMOKE_MISSING_GAME")
                    Qt.exit(7)
                    return
                }
                window.launchPauseSmokePhase = 1
                controller.launch_game(row, "fixture-racer")
            } else if (window.launchPauseSmokePhase === 1
                       && controller.pause_screen_available
                       && controller.last_launch_succeeded
                       && !controller.startup_screen_active
                       && controller.launch_session_active) {
                window.launchPauseSmokePhase = 2
                controller.pause_launch_session()
            } else if (window.launchPauseSmokePhase === 2
                       && controller.pause_screen_active) {
                window.launchPauseSmokePhase = 3
                launchPauseHoldTimer.restart()
            } else if (window.launchPauseSmokePhase >= 3
                       && !controller.launching
                       && !controller.launch_session_active
                       && !controller.pause_screen_active
                       && !controller.startup_screen_active
                       && !controller.shutdown_screen_active) {
                const presentationFlags =
                    (window.launchPauseVisibleSeen ? 1 : 0)
                    | (window.launchPauseProcessSuspendedSeen ? 2 : 0)
                    | (window.launchPauseResumeSeen ? 4 : 0)
                if (!controller.report_launch_pause_smoke_success(
                            "fixture-racer", presentationFlags)) {
                    console.error(
                        "LAUNCH_PAUSE_SMOKE_FAILED visible="
                        + window.launchPauseVisibleSeen
                        + " suspended="
                        + window.launchPauseProcessSuspendedSeen
                        + " resumed=" + window.launchPauseResumeSeen
                        + " status=" + controller.status_message)
                    Qt.exit(7)
                    return
                }
                window.launchPauseSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        id: launchPauseHoldTimer
        interval: 300
        repeat: false
        onTriggered: controller.resume_launch_session()
    }

    Timer {
        interval: 15000
        running: window.launchPauseSmokeTest
                 && !window.launchPauseSmokeFinished
        onTriggered: {
            console.error(
                "LAUNCH_PAUSE_SMOKE_TIMEOUT phase="
                + window.launchPauseSmokePhase
                + " available=" + controller.pause_screen_available
                + " active=" + controller.pause_screen_active
                + " suspended="
                + controller.pause_screen_process_suspended
                + " session=" + controller.launch_session_active
                + " status=" + controller.status_message)
            Qt.exit(7)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.additionalApplicationCrudSmokeTest
                 && !window.additionalApplicationCrudSmokeFinished
        onTriggered: {
            const gameId = "fixture-adventure"
            const applicationId = "fixture-adventure-manual"
            if (window.additionalApplicationCrudSmokePhase === 0
                    && !controller.loading && !controller.writing
                    && controller.library_path.length > 0
                    && controller.game_count === 3) {
                const row = controller.row_for_game_id(gameId)
                if (row < 0
                        || controller.additional_application_count(row, gameId) !== 1) {
                    console.error("ADDITIONAL_APPLICATION_CRUD_SMOKE_MISSING_FIXTURE")
                    Qt.exit(14)
                    return
                }
                additionalApplicationManager.prepare(
                    row, gameId, "Fixture Adventure")
                window.additionalApplicationCrudSmokePhase = 1
                additionalApplicationEditor.smokeEdit(row, gameId, applicationId)
            } else if (window.additionalApplicationCrudSmokePhase === 1
                       && !controller.writing
                       && controller.additional_application_revision === 1) {
                const editRow = controller.row_for_game_id(gameId)
                window.additionalApplicationCrudSmokePhase = 2
                additionalApplicationEditor.smokeCreate(editRow, gameId)
            } else if (window.additionalApplicationCrudSmokePhase === 2
                       && !controller.writing
                       && controller.additional_application_revision === 2
                       && controller.last_added_additional_application_id.length > 0) {
                window.additionalApplicationCrudSmokeAddedId =
                    controller.last_added_additional_application_id
                const deleteRow = controller.row_for_game_id(gameId)
                window.additionalApplicationCrudSmokePhase = 3
                additionalApplicationDeleteDialog.smokeDelete(
                    deleteRow, gameId,
                    window.additionalApplicationCrudSmokeAddedId,
                    "Temporary Fixture Application")
            } else if (window.additionalApplicationCrudSmokePhase === 3
                       && !controller.writing
                       && controller.additional_application_revision === 3) {
                if (!controller.report_additional_application_crud_smoke_success(
                        gameId)) {
                    console.error(
                        "ADDITIONAL_APPLICATION_CRUD_SMOKE_MODEL_CONTRACT_FAILED")
                    Qt.exit(14)
                    return
                }
                window.additionalApplicationCrudSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 15000
        running: window.additionalApplicationCrudSmokeTest
                 && !window.additionalApplicationCrudSmokeFinished
        onTriggered: {
            console.error("ADDITIONAL_APPLICATION_CRUD_SMOKE_TIMEOUT phase="
                          + window.additionalApplicationCrudSmokePhase
                          + " status=" + controller.status_message)
            Qt.exit(14)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.additionalApplicationDefaultSmokeTest
                 && !window.additionalApplicationDefaultSmokeFinished
        onTriggered: {
            const gameId = "fixture-adventure"
            const applicationId = "fixture-adventure-manual"
            if (window.additionalApplicationDefaultSmokePhase === 0
                    && !controller.loading && !controller.writing
                    && controller.library_path.length > 0
                    && controller.game_count === 3) {
                const row = controller.row_for_game_id(gameId)
                if (row < 0
                        || controller.additional_application_count(row, gameId) !== 1) {
                    console.error(
                        "ADDITIONAL_APPLICATION_DEFAULT_SMOKE_MISSING_FIXTURE")
                    Qt.exit(15)
                    return
                }
                additionalApplicationManager.prepare(
                    row, gameId, "Fixture Adventure")
                window.additionalApplicationDefaultSmokePhase = 1
                additionalApplicationEditor.smokeEdit(row, gameId, applicationId)
            } else if (window.additionalApplicationDefaultSmokePhase === 1
                       && !controller.writing
                       && controller.additional_application_revision === 1) {
                const defaultRow = controller.row_for_game_id(gameId)
                window.additionalApplicationDefaultSmokePhase = 2
                additionalApplicationDefaultDialog.smokeMakeDefault(
                    defaultRow, gameId, applicationId, "Edited Fixture Manual")
            } else if (window.additionalApplicationDefaultSmokePhase === 2
                       && !controller.writing
                       && controller.last_default_additional_application_id
                          === applicationId) {
                if (!controller.report_additional_application_default_smoke_success(
                        gameId, applicationId)) {
                    console.error(
                        "ADDITIONAL_APPLICATION_DEFAULT_SMOKE_MODEL_CONTRACT_FAILED")
                    Qt.exit(15)
                    return
                }
                window.additionalApplicationDefaultSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 15000
        running: window.additionalApplicationDefaultSmokeTest
                 && !window.additionalApplicationDefaultSmokeFinished
        onTriggered: {
            console.error("ADDITIONAL_APPLICATION_DEFAULT_SMOKE_TIMEOUT phase="
                          + window.additionalApplicationDefaultSmokePhase
                          + " status=" + controller.status_message)
            Qt.exit(15)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.gameSaveMetadataSmokeTest
                 && !window.gameSaveMetadataSmokeFinished
        onTriggered: {
            const gameId = "fixture-adventure"
            if (window.gameSaveMetadataSmokePhase === 0
                    && !controller.loading && !controller.writing
                    && controller.library_path.length > 0
                    && controller.game_count === 3) {
                const row = controller.row_for_game_id(gameId)
                if (row < 0 || controller.game_save_count(row, gameId) !== 3) {
                    console.error("GAME_SAVE_METADATA_SMOKE_MISSING_FIXTURE")
                    Qt.exit(16)
                    return
                }
                gameSaveManager.prepare(row, gameId, "Fixture Adventure")
                window.gameSaveMetadataSmokePhase = 1
                gameSaveTextDialog.smoke("version", "Renamed Active")
            } else if (window.gameSaveMetadataSmokePhase === 1
                       && !controller.writing
                       && controller.game_save_revision === 1) {
                window.gameSaveMetadataSmokePhase = 2
                gameSaveManager.selectedGroupIndex = 0
                gameSaveManager.selectedVersionIndex = 0
                gameSaveTextDialog.smoke("group", "Renamed Run")
            } else if (window.gameSaveMetadataSmokePhase === 2
                       && !controller.writing
                       && controller.game_save_revision === 2) {
                window.gameSaveMetadataSmokePhase = 3
                gameSaveManager.selectedGroupIndex = 1
                gameSaveManager.selectedVersionIndex = 0
                gameSaveCombineDialog.smoke()
            } else if (window.gameSaveMetadataSmokePhase === 3
                       && !controller.writing
                       && controller.game_save_revision === 3) {
                window.gameSaveMetadataSmokePhase = 4
                gameSaveManager.selectedGroupIndex = 0
                gameSaveManager.selectedVersionIndex = 2
                gameSaveTextDialog.smoke("split", "Split History")
            } else if (window.gameSaveMetadataSmokePhase === 4
                       && !controller.writing
                       && controller.game_save_revision === 4) {
                if (!controller.report_game_save_metadata_smoke_success(gameId)) {
                    console.error(
                        "GAME_SAVE_METADATA_SMOKE_MODEL_CONTRACT_FAILED")
                    Qt.exit(16)
                    return
                }
                window.gameSaveMetadataSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 15000
        running: window.gameSaveMetadataSmokeTest
                 && !window.gameSaveMetadataSmokeFinished
        onTriggered: {
            console.error("GAME_SAVE_METADATA_SMOKE_TIMEOUT phase="
                          + window.gameSaveMetadataSmokePhase
                          + " status=" + controller.status_message)
            Qt.exit(16)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.gameSaveBackupSmokeTest
                 && !window.gameSaveBackupSmokeFinished
        onTriggered: {
            const gameId = "fixture-adventure"
            if (window.gameSaveBackupSmokePhase === 0
                    && !controller.loading && !controller.writing
                    && controller.library_path.length > 0
                    && controller.game_count === 3) {
                const row = controller.row_for_game_id(gameId)
                if (row < 0 || controller.game_save_count(row, gameId) !== 1) {
                    console.error("GAME_SAVE_BACKUP_SMOKE_MISSING_FIXTURE")
                    Qt.exit(17)
                    return
                }
                gameSaveManager.prepare(row, gameId, "Fixture Adventure")
                const version = gameSaveManager.selectedVersion()
                if (version === null || version.location_kind !== "active") {
                    console.error(
                        "GAME_SAVE_BACKUP_SMOKE_ACTIVE_NOT_RESOLVED")
                    Qt.exit(17)
                    return
                }
                window.gameSaveBackupSmokePhase = 1
                controller.backup_game_save(
                    row, gameId, version.source_index)
            } else if (window.gameSaveBackupSmokePhase === 1
                       && !controller.writing
                       && controller.game_save_revision === 1) {
                if (!controller.report_game_save_backup_smoke_success(gameId)) {
                    console.error(
                        "GAME_SAVE_BACKUP_SMOKE_MODEL_CONTRACT_FAILED")
                    Qt.exit(17)
                    return
                }
                window.gameSaveBackupSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 15000
        running: window.gameSaveBackupSmokeTest
                 && !window.gameSaveBackupSmokeFinished
        onTriggered: {
            console.error("GAME_SAVE_BACKUP_SMOKE_TIMEOUT phase="
                          + window.gameSaveBackupSmokePhase
                          + " status=" + controller.status_message)
            Qt.exit(17)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.pcsx2SaveBackupSmokeTest
                 && !window.pcsx2SaveBackupSmokeFinished
        onTriggered: {
            const gameId = "fixture-adventure"
            if (window.pcsx2SaveBackupSmokePhase === 0
                    && !controller.loading && !controller.writing
                    && controller.library_path.length > 0
                    && controller.game_count === 3) {
                const row = controller.row_for_game_id(gameId)
                if (row < 0 || controller.game_save_count(row, gameId) !== 1) {
                    console.error(
                        "PCSX2_SAVE_BACKUP_SMOKE_MISSING_FIXTURE")
                    Qt.exit(25)
                    return
                }
                gameSaveManager.prepare(row, gameId, "Fixture Adventure")
                const version = gameSaveManager.selectedVersion()
                if (version === null || version.location_kind !== "active") {
                    console.error(
                        "PCSX2_SAVE_BACKUP_SMOKE_ACTIVE_NOT_RESOLVED")
                    Qt.exit(25)
                    return
                }
                window.pcsx2SaveBackupSmokePhase = 1
                controller.backup_game_save(
                    row, gameId, version.source_index)
            } else if (window.pcsx2SaveBackupSmokePhase === 1
                       && !controller.writing
                       && controller.game_save_revision === 1) {
                if (!controller.report_pcsx2_save_backup_smoke_success(
                        gameId)) {
                    console.error(
                        "PCSX2_SAVE_BACKUP_SMOKE_MODEL_CONTRACT_FAILED")
                    Qt.exit(25)
                    return
                }
                window.pcsx2SaveBackupSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 15000
        running: window.pcsx2SaveBackupSmokeTest
                 && !window.pcsx2SaveBackupSmokeFinished
        onTriggered: {
            console.error("PCSX2_SAVE_BACKUP_SMOKE_TIMEOUT phase="
                          + window.pcsx2SaveBackupSmokePhase
                          + " status=" + controller.status_message)
            Qt.exit(25)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.pcsx2SaveLifecycleSmokeTest
                 && !window.pcsx2SaveLifecycleSmokeFinished
        onTriggered: {
            const gameId = "fixture-adventure"
            if (window.pcsx2SaveLifecycleSmokePhase === 0
                    && !controller.loading && !controller.writing
                    && controller.library_path.length > 0
                    && controller.game_count === 3) {
                const row = controller.row_for_game_id(gameId)
                if (row < 0 || controller.game_save_count(row, gameId) !== 2) {
                    console.error(
                        "PCSX2_SAVE_LIFECYCLE_SMOKE_MISSING_FIXTURE")
                    Qt.exit(27)
                    return
                }
                gameSaveManager.prepare(row, gameId, "Fixture Adventure")
                gameSaveManager.selectedVersionIndex = 1
                const version = gameSaveManager.selectedVersion()
                if (version === null || version.location_kind !== "vault") {
                    console.error(
                        "PCSX2_SAVE_LIFECYCLE_SMOKE_VAULT_NOT_RESOLVED")
                    Qt.exit(27)
                    return
                }
                window.pcsx2SaveLifecycleSmokePhase = 1
                gameSaveRestoreDialog.smoke()
            } else if (window.pcsx2SaveLifecycleSmokePhase === 1
                       && !controller.writing
                       && controller.game_save_revision === 1) {
                if (controller.status_message.indexOf(
                        "Recovered 28 orphaned memory-card cluster(s) "
                        + "in the private working copy before replacement.")
                        < 0) {
                    console.error(
                        "PCSX2_SAVE_LIFECYCLE_SMOKE_REPAIR_STATUS_FAILED "
                        + controller.status_message)
                    Qt.exit(27)
                    return
                }
                const row = controller.row_for_game_id(gameId)
                if (row < 0 || controller.game_save_count(row, gameId) !== 3) {
                    console.error(
                        "PCSX2_SAVE_LIFECYCLE_SMOKE_RESTORE_MODEL_FAILED")
                    Qt.exit(27)
                    return
                }
                gameSaveManager.prepare(row, gameId, "Fixture Adventure")
                gameSaveManager.selectedVersionIndex = 0
                const version = gameSaveManager.selectedVersion()
                if (version === null || version.location_kind !== "active") {
                    console.error(
                        "PCSX2_SAVE_LIFECYCLE_SMOKE_ACTIVE_NOT_RESOLVED")
                    Qt.exit(27)
                    return
                }
                window.pcsx2SaveLifecycleSmokePhase = 2
                gameSaveActiveDeleteDialog.smoke()
            } else if (window.pcsx2SaveLifecycleSmokePhase === 2
                       && !controller.writing
                       && controller.game_save_revision === 2) {
                if (!controller.report_pcsx2_save_lifecycle_smoke_success(
                        gameId)) {
                    console.error(
                        "PCSX2_SAVE_LIFECYCLE_SMOKE_MODEL_CONTRACT_FAILED")
                    Qt.exit(27)
                    return
                }
                window.pcsx2SaveLifecycleSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 30000
        running: window.pcsx2SaveLifecycleSmokeTest
                 && !window.pcsx2SaveLifecycleSmokeFinished
        onTriggered: {
            console.error("PCSX2_SAVE_LIFECYCLE_SMOKE_TIMEOUT phase="
                          + window.pcsx2SaveLifecycleSmokePhase
                          + " status=" + controller.status_message)
            Qt.exit(27)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.dolphinWiiSaveLifecycleSmokeTest
                 && !window.dolphinWiiSaveLifecycleSmokeFinished
        onTriggered: {
            const gameId = "fixture-adventure"
            if (window.dolphinWiiSaveLifecycleSmokePhase === 0
                    && !controller.loading && !controller.writing
                    && controller.library_path.length > 0
                    && controller.game_count === 3) {
                const row = controller.row_for_game_id(gameId)
                if (row < 0 || controller.game_save_count(row, gameId) !== 2) {
                    console.error(
                        "DOLPHIN_WII_SAVE_LIFECYCLE_SMOKE_MISSING_FIXTURE")
                    Qt.exit(42)
                    return
                }
                gameSaveManager.prepare(row, gameId, "Fixture Adventure")
                gameSaveManager.selectedVersionIndex = 1
                const version = gameSaveManager.selectedVersion()
                if (version === null || version.location_kind !== "vault") {
                    console.error(
                        "DOLPHIN_WII_SAVE_LIFECYCLE_SMOKE_VAULT_NOT_RESOLVED")
                    Qt.exit(42)
                    return
                }
                window.dolphinWiiSaveLifecycleSmokePhase = 1
                gameSaveRestoreDialog.smoke()
            } else if (window.dolphinWiiSaveLifecycleSmokePhase === 1
                       && !controller.writing
                       && controller.game_save_revision === 1) {
                const row = controller.row_for_game_id(gameId)
                if (row < 0 || controller.game_save_count(row, gameId) !== 3) {
                    console.error(
                        "DOLPHIN_WII_SAVE_LIFECYCLE_SMOKE_RESTORE_MODEL_FAILED")
                    Qt.exit(42)
                    return
                }
                gameSaveManager.prepare(row, gameId, "Fixture Adventure")
                gameSaveManager.selectedVersionIndex = 0
                const version = gameSaveManager.selectedVersion()
                if (version === null || version.location_kind !== "active") {
                    console.error(
                        "DOLPHIN_WII_SAVE_LIFECYCLE_SMOKE_ACTIVE_NOT_RESOLVED")
                    Qt.exit(42)
                    return
                }
                window.dolphinWiiSaveLifecycleSmokePhase = 2
                gameSaveActiveDeleteDialog.smoke()
            } else if (window.dolphinWiiSaveLifecycleSmokePhase === 2
                       && !controller.writing
                       && controller.game_save_revision === 2) {
                if (!controller.report_dolphin_wii_save_lifecycle_smoke_success(
                        gameId)) {
                    console.error(
                        "DOLPHIN_WII_SAVE_LIFECYCLE_SMOKE_MODEL_CONTRACT_FAILED")
                    Qt.exit(42)
                    return
                }
                window.dolphinWiiSaveLifecycleSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 30000
        running: window.dolphinWiiSaveLifecycleSmokeTest
                 && !window.dolphinWiiSaveLifecycleSmokeFinished
        onTriggered: {
            console.error("DOLPHIN_WII_SAVE_LIFECYCLE_SMOKE_TIMEOUT phase="
                          + window.dolphinWiiSaveLifecycleSmokePhase
                          + " status=" + controller.status_message)
            Qt.exit(42)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.gameSaveDeleteSmokeTest
                 && !window.gameSaveDeleteSmokeFinished
        onTriggered: {
            const gameId = "fixture-adventure"
            if (window.gameSaveDeleteSmokePhase === 0
                    && !controller.loading && !controller.writing
                    && controller.library_path.length > 0
                    && controller.game_count === 3) {
                const row = controller.row_for_game_id(gameId)
                if (row < 0 || controller.game_save_count(row, gameId) !== 2) {
                    console.error("GAME_SAVE_DELETE_SMOKE_MISSING_FIXTURE")
                    Qt.exit(18)
                    return
                }
                gameSaveManager.prepare(row, gameId, "Fixture Adventure")
                gameSaveManager.selectedVersionIndex = 1
                const version = gameSaveManager.selectedVersion()
                if (version === null || version.location_kind !== "vault") {
                    console.error(
                        "GAME_SAVE_DELETE_SMOKE_VAULT_NOT_RESOLVED")
                    Qt.exit(18)
                    return
                }
                window.gameSaveDeleteSmokePhase = 1
                gameSaveDeleteDialog.smoke()
            } else if (window.gameSaveDeleteSmokePhase === 1
                       && !controller.writing
                       && controller.game_save_revision === 1) {
                if (!controller.report_game_save_delete_smoke_success(gameId)) {
                    console.error(
                        "GAME_SAVE_DELETE_SMOKE_MODEL_CONTRACT_FAILED")
                    Qt.exit(18)
                    return
                }
                window.gameSaveDeleteSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 15000
        running: window.gameSaveDeleteSmokeTest
                 && !window.gameSaveDeleteSmokeFinished
        onTriggered: {
            console.error("GAME_SAVE_DELETE_SMOKE_TIMEOUT phase="
                          + window.gameSaveDeleteSmokePhase
                          + " status=" + controller.status_message)
            Qt.exit(18)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.gameSaveActiveDeleteSmokeTest
                 && !window.gameSaveActiveDeleteSmokeFinished
        onTriggered: {
            const gameId = "fixture-adventure"
            if (window.gameSaveActiveDeleteSmokePhase === 0
                    && !controller.loading && !controller.writing
                    && controller.library_path.length > 0
                    && controller.game_count === 3) {
                const row = controller.row_for_game_id(gameId)
                if (row < 0 || controller.game_save_count(row, gameId) !== 1) {
                    console.error(
                        "GAME_SAVE_ACTIVE_DELETE_SMOKE_MISSING_FIXTURE")
                    Qt.exit(22)
                    return
                }
                gameSaveManager.prepare(row, gameId, "Fixture Adventure")
                const version = gameSaveManager.selectedVersion()
                if (version === null || version.location_kind !== "active") {
                    console.error(
                        "GAME_SAVE_ACTIVE_DELETE_SMOKE_ACTIVE_NOT_RESOLVED")
                    Qt.exit(22)
                    return
                }
                window.gameSaveActiveDeleteSmokePhase = 1
                gameSaveActiveDeleteDialog.smoke()
            } else if (window.gameSaveActiveDeleteSmokePhase === 1
                       && !controller.writing
                       && controller.game_save_revision === 1) {
                if (!controller.report_game_save_active_delete_smoke_success(
                        gameId)) {
                    console.error(
                        "GAME_SAVE_ACTIVE_DELETE_SMOKE_MODEL_CONTRACT_FAILED")
                    Qt.exit(22)
                    return
                }
                window.gameSaveActiveDeleteSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 15000
        running: window.gameSaveActiveDeleteSmokeTest
                 && !window.gameSaveActiveDeleteSmokeFinished
        onTriggered: {
            console.error("GAME_SAVE_ACTIVE_DELETE_SMOKE_TIMEOUT phase="
                          + window.gameSaveActiveDeleteSmokePhase
                          + " status=" + controller.status_message)
            Qt.exit(22)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.gameSaveRestoreSmokeTest
                 && !window.gameSaveRestoreSmokeFinished
        onTriggered: {
            const gameId = "fixture-adventure"
            if (window.gameSaveRestoreSmokePhase === 0
                    && !controller.loading && !controller.writing
                    && controller.library_path.length > 0
                    && controller.game_count === 3) {
                const row = controller.row_for_game_id(gameId)
                if (row < 0 || controller.game_save_count(row, gameId) !== 2) {
                    console.error("GAME_SAVE_RESTORE_SMOKE_MISSING_FIXTURE")
                    Qt.exit(19)
                    return
                }
                gameSaveManager.prepare(row, gameId, "Fixture Adventure")
                gameSaveManager.selectedVersionIndex = 1
                const version = gameSaveManager.selectedVersion()
                if (version === null || version.location_kind !== "vault") {
                    console.error(
                        "GAME_SAVE_RESTORE_SMOKE_VAULT_NOT_RESOLVED")
                    Qt.exit(19)
                    return
                }
                window.gameSaveRestoreSmokePhase = 1
                gameSaveRestoreDialog.smoke()
            } else if (window.gameSaveRestoreSmokePhase === 1
                       && !controller.writing
                       && controller.game_save_revision === 1) {
                if (!controller.report_game_save_restore_smoke_success(gameId)) {
                    console.error(
                        "GAME_SAVE_RESTORE_SMOKE_MODEL_CONTRACT_FAILED")
                    Qt.exit(19)
                    return
                }
                window.gameSaveRestoreSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 15000
        running: window.gameSaveRestoreSmokeTest
                 && !window.gameSaveRestoreSmokeFinished
        onTriggered: {
            console.error("GAME_SAVE_RESTORE_SMOKE_TIMEOUT phase="
                          + window.gameSaveRestoreSmokePhase
                          + " status=" + controller.status_message)
            Qt.exit(19)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.gameSaveSaturnRestoreSmokeTest
                 && !window.gameSaveSaturnRestoreSmokeFinished
        onTriggered: {
            const gameId = "fixture-adventure"
            if (window.gameSaveSaturnRestoreSmokePhase === 0
                    && !controller.loading && !controller.writing
                    && controller.library_path.length > 0
                    && controller.game_count === 3) {
                const row = controller.row_for_game_id(gameId)
                if (row < 0 || controller.game_save_count(row, gameId) !== 2) {
                    console.error(
                        "GAME_SAVE_SATURN_RESTORE_SMOKE_MISSING_FIXTURE")
                    Qt.exit(21)
                    return
                }
                gameSaveManager.prepare(row, gameId, "Fixture Adventure")
                gameSaveManager.selectedVersionIndex = 1
                const version = gameSaveManager.selectedVersion()
                if (version === null || version.location_kind !== "vault") {
                    console.error(
                        "GAME_SAVE_SATURN_RESTORE_SMOKE_VAULT_NOT_RESOLVED")
                    Qt.exit(21)
                    return
                }
                window.gameSaveSaturnRestoreSmokePhase = 1
                gameSaveRestoreDialog.smoke()
            } else if (window.gameSaveSaturnRestoreSmokePhase === 1
                       && !controller.writing
                       && controller.game_save_revision === 1) {
                if (!controller.report_game_save_saturn_restore_smoke_success(
                        gameId)) {
                    console.error(
                        "GAME_SAVE_SATURN_RESTORE_SMOKE_MODEL_CONTRACT_FAILED")
                    Qt.exit(21)
                    return
                }
                window.gameSaveSaturnRestoreSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 15000
        running: window.gameSaveSaturnRestoreSmokeTest
                 && !window.gameSaveSaturnRestoreSmokeFinished
        onTriggered: {
            console.error("GAME_SAVE_SATURN_RESTORE_SMOKE_TIMEOUT phase="
                          + window.gameSaveSaturnRestoreSmokePhase
                          + " status=" + controller.status_message)
            Qt.exit(21)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.retroarchSaveScanSmokeTest
                 && !window.retroarchSaveScanSmokeFinished
        onTriggered: {
            const gameId = "fixture-racer"
            if (window.retroarchSaveScanSmokePhase === 0
                    && !controller.loading && !controller.writing
                    && controller.library_path.length > 0
                    && controller.game_count === 3) {
                const row = controller.row_for_game_id(gameId)
                if (row < 0 || controller.game_save_count(row, gameId) !== 0) {
                    console.error("RETROARCH_SAVE_SCAN_SMOKE_BAD_FIXTURE")
                    Qt.exit(20)
                    return
                }
                gameSaveManager.prepare(row, gameId, "Fixture Racer")
                window.retroarchSaveScanSmokePhase = 1
                gameSaveManager.smokeScan()
            } else if (window.retroarchSaveScanSmokePhase === 1
                       && !controller.writing
                       && controller.game_save_revision === 1) {
                if (!controller.report_retroarch_save_scan_smoke_success(
                        gameId)) {
                    console.error(
                        "RETROARCH_SAVE_SCAN_SMOKE_MODEL_CONTRACT_FAILED")
                    Qt.exit(20)
                    return
                }
                window.retroarchSaveScanSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 15000
        running: window.retroarchSaveScanSmokeTest
                 && !window.retroarchSaveScanSmokeFinished
        onTriggered: {
            console.error("RETROARCH_SAVE_SCAN_SMOKE_TIMEOUT phase="
                          + window.retroarchSaveScanSmokePhase
                          + " status=" + controller.status_message)
            Qt.exit(20)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.dolphinSaveScanSmokeTest
                 && !window.dolphinSaveScanSmokeFinished
        onTriggered: {
            const gameId = "fixture-racer"
            if (window.dolphinSaveScanSmokePhase === 0
                    && !controller.loading && !controller.writing
                    && controller.library_path.length > 0
                    && controller.game_count === 3) {
                const row = controller.row_for_game_id(gameId)
                if (row < 0 || controller.game_save_count(row, gameId) !== 0) {
                    console.error("DOLPHIN_SAVE_SCAN_SMOKE_BAD_FIXTURE")
                    Qt.exit(23)
                    return
                }
                gameSaveManager.prepare(row, gameId, "Fixture Racer")
                window.dolphinSaveScanSmokePhase = 1
                gameSaveManager.smokeScan()
            } else if (window.dolphinSaveScanSmokePhase === 1
                       && !controller.writing
                       && controller.game_save_revision === 1) {
                if (!controller.report_dolphin_save_scan_smoke_success(
                        gameId)) {
                    console.error(
                        "DOLPHIN_SAVE_SCAN_SMOKE_MODEL_CONTRACT_FAILED")
                    Qt.exit(23)
                    return
                }
                window.dolphinSaveScanSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 15000
        running: window.dolphinSaveScanSmokeTest
                 && !window.dolphinSaveScanSmokeFinished
        onTriggered: {
            console.error("DOLPHIN_SAVE_SCAN_SMOKE_TIMEOUT phase="
                          + window.dolphinSaveScanSmokePhase
                          + " status=" + controller.status_message)
            Qt.exit(23)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.pcsx2SaveScanSmokeTest
                 && !window.pcsx2SaveScanSmokeFinished
        onTriggered: {
            const gameId = "fixture-racer"
            if (window.pcsx2SaveScanSmokePhase === 0
                    && !controller.loading && !controller.writing
                    && controller.library_path.length > 0
                    && controller.game_count === 3) {
                const row = controller.row_for_game_id(gameId)
                if (row < 0 || controller.game_save_count(row, gameId) !== 0) {
                    console.error("PCSX2_SAVE_SCAN_SMOKE_BAD_FIXTURE")
                    Qt.exit(24)
                    return
                }
                gameSaveManager.prepare(row, gameId, "Fixture Racer")
                window.pcsx2SaveScanSmokePhase = 1
                gameSaveManager.smokeScan()
            } else if (window.pcsx2SaveScanSmokePhase === 1
                       && !controller.writing
                       && controller.game_save_revision === 1) {
                if (!controller.report_pcsx2_save_scan_smoke_success(
                        gameId)) {
                    console.error(
                        "PCSX2_SAVE_SCAN_SMOKE_MODEL_CONTRACT_FAILED")
                    Qt.exit(24)
                    return
                }
                window.pcsx2SaveScanSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 15000
        running: window.pcsx2SaveScanSmokeTest
                 && !window.pcsx2SaveScanSmokeFinished
        onTriggered: {
            console.error("PCSX2_SAVE_SCAN_SMOKE_TIMEOUT phase="
                          + window.pcsx2SaveScanSmokePhase
                          + " status=" + controller.status_message)
            Qt.exit(24)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.platformCrudSmokeTest && !window.platformCrudSmokeFinished
        onTriggered: {
            const platformName = "Dragon 32/64"
            if (window.platformCrudSmokePhase === 0 && !controller.loading
                    && controller.library_path.length > 0
                    && controller.game_count === 3
                    && controller.platform_entry_count === 1) {
                window.platformCrudSmokePhase = 1
                addPlatformDialog.smokeCreate(platformName, platformName)
            } else if (window.platformCrudSmokePhase === 1 && !controller.writing
                       && controller.platform_entry_count === 2) {
                const platformIndex = window.platformIndex(platformName)
                if (platformIndex < 0 || controller.platform_game_count_at(platformIndex) !== 0) {
                    console.error("PLATFORM_CRUD_SMOKE_EMPTY_PLATFORM_MISSING")
                    Qt.exit(9)
                    return
                }
                window.platformCrudSmokePhase = 2
                platformEditor.smokeSave(platformName)
            } else if (window.platformCrudSmokePhase === 2 && !controller.writing) {
                const saved = controller.platform_edit_payload(platformName)
                if (saved.length === 0) {
                    console.error("PLATFORM_CRUD_SMOKE_EDIT_PAYLOAD_MISSING")
                    Qt.exit(9)
                    return
                }
                const payload = JSON.parse(saved)
                if (payload.platform.metadata.sort_title !== "Dragon, 32/64"
                        || payload.platform.metadata.developer !== "Qt Forge"
                        || payload.platform.metadata.cpu !== "6809"
                        || payload.platform.metadata.notes
                           !== "Edited through the real platform dialog."
                        || !payload.platform.metadata.hide_in_big_box
                        || !payload.platform.disable_auto_import
                        || payload.folders.length !== 52
                        || payload.folders[0].folder_path
                           !== "Images\\Dragon 32_64\\Edited"
                        || payload.folders[51].media_type !== "Test Media"
                        || payload.folders[51].folder_path
                           !== "Portable\\Dragon 32_64") {
                    console.error("PLATFORM_CRUD_SMOKE_EDIT_NOT_PERSISTED payload="
                                  + saved)
                    Qt.exit(9)
                    return
                }
                window.platformCrudSmokePhase = 3
                addGameDialog.smokeAdd("Dragon Test",
                                       "Games\\Dragon 32_64\\test.vdk",
                                       platformName)
            } else if (window.platformCrudSmokePhase === 3 && !controller.writing
                       && controller.game_count === 4
                       && controller.last_added_game_id.length > 0) {
                window.platformCrudAddedGameId = controller.last_added_game_id
                window.platformCrudSmokePhase = 4
                deletePlatformConfirmation.smokeDelete(platformName)
            } else if (window.platformCrudSmokePhase === 4 && !controller.writing
                       && controller.delete_blocker_count > 0) {
                window.platformCrudBlockedReferences = controller.delete_blocker_count
                const row = controller.row_for_game_id(window.platformCrudAddedGameId)
                if (row < 0) {
                    console.error("PLATFORM_CRUD_SMOKE_ADDED_GAME_MISSING")
                    Qt.exit(9)
                    return
                }
                window.platformCrudSmokePhase = 5
                deleteConfirmation.smokeDelete(
                    row, window.platformCrudAddedGameId, "Dragon Test")
            } else if (window.platformCrudSmokePhase === 5 && !controller.writing
                       && controller.game_count === 3) {
                window.platformCrudSmokePhase = 6
                deletePlatformConfirmation.smokeDelete(platformName)
            } else if (window.platformCrudSmokePhase === 6 && !controller.writing
                       && controller.platform_entry_count === 1) {
                if (!controller.report_platform_crud_smoke_success(
                        platformName, window.platformCrudBlockedReferences)) {
                    console.error("PLATFORM_CRUD_SMOKE_MODEL_CONTRACT_FAILED")
                    Qt.exit(9)
                    return
                }
                window.platformCrudSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 20000
        running: window.platformCrudSmokeTest && !window.platformCrudSmokeFinished
        onTriggered: {
            console.error("PLATFORM_CRUD_SMOKE_TIMEOUT phase="
                          + window.platformCrudSmokePhase
                          + " status=" + controller.status_message)
            Qt.exit(9)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.emulatorCrudSmokeTest && !window.emulatorCrudSmokeFinished
        onTriggered: {
            const fixtureId = "fixture-emulator"
            if (window.emulatorCrudSmokePhase === 0 && !controller.loading
                    && !controller.writing && controller.library_path.length > 0
                    && controller.emulator_entry_count() === 3) {
                window.emulatorCrudInitialRevision = controller.emulator_revision
                window.emulatorCrudSmokePhase = 1
                emulatorEditor.smokeEdit(fixtureId)
            } else if (window.emulatorCrudSmokePhase === 1
                       && !controller.writing
                       && controller.emulator_revision
                          === window.emulatorCrudInitialRevision + 1) {
                const serialized = controller.emulator_edit_payload(fixtureId)
                const payload = serialized.length > 0 ? JSON.parse(serialized) : null
                if (payload === null
                        || payload.emulator.title !== "Edited Fixture Emulator"
                        || payload.emulator.application_path
                           !== "Emulators\\Edited Fixture\\fixture.exe"
                        || payload.emulator.auto_hotkey_script
                           !== "Smoke launch script"
                        || payload.platforms.length !== 1
                        || payload.platforms[0].command_line !== "--edited-mapping"
                        || !payload.platforms[0].default
                        || payload.platforms[0].auto_extract !== false
                        || !payload.platforms[0].m3u_disc_load_enabled) {
                    console.error("EMULATOR_CRUD_SMOKE_EDIT_NOT_PERSISTED payload="
                                  + serialized)
                    Qt.exit(35)
                    return
                }
                window.emulatorCrudSmokePhase = 2
                emulatorEditor.smokeCreate()
            } else if (window.emulatorCrudSmokePhase === 2
                       && !controller.writing
                       && controller.emulator_revision
                          === window.emulatorCrudInitialRevision + 2
                       && controller.last_added_emulator_id.length > 0) {
                window.emulatorCrudAddedId = controller.last_added_emulator_id
                window.emulatorCrudSmokePhase = 3
                deleteEmulatorConfirmation.smokeDelete(
                            fixtureId, "Edited Fixture Emulator")
            } else if (window.emulatorCrudSmokePhase === 3
                       && !controller.writing
                       && controller.delete_blocker_count > 0) {
                window.emulatorCrudBlockedReferences =
                    controller.delete_blocker_count
                window.emulatorCrudSmokePhase = 4
                deleteEmulatorConfirmation.smokeDelete(
                            window.emulatorCrudAddedId, "Temporary Qt Emulator")
            } else if (window.emulatorCrudSmokePhase === 4
                       && !controller.writing
                       && controller.emulator_revision
                          === window.emulatorCrudInitialRevision + 3) {
                if (!controller.report_emulator_crud_smoke_success(
                        window.emulatorCrudAddedId,
                        window.emulatorCrudBlockedReferences,
                        window.emulatorCrudInitialRevision)) {
                    console.error("EMULATOR_CRUD_SMOKE_MODEL_CONTRACT_FAILED")
                    Qt.exit(35)
                    return
                }
                window.emulatorCrudSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 20000
        running: window.emulatorCrudSmokeTest && !window.emulatorCrudSmokeFinished
        onTriggered: {
            console.error("EMULATOR_CRUD_SMOKE_TIMEOUT phase="
                          + window.emulatorCrudSmokePhase
                          + " status=" + controller.status_message)
            Qt.exit(35)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.emulatorDiscoverySmokeTest
                 && !window.emulatorDiscoverySmokeFinished
        onTriggered: {
            if (window.emulatorDiscoverySmokePhase === 0 && !controller.loading
                    && !controller.writing && controller.library_path.length > 0) {
                window.emulatorDiscoveryInitialEmulatorRevision =
                    controller.emulator_revision
                window.emulatorDiscoveryInitialRevision =
                    controller.emulator_discovery_revision
                window.emulatorDiscoverySmokePhase = 1
                controller.scan_installed_emulators()
            } else if (window.emulatorDiscoverySmokePhase === 1
                       && !controller.emulator_discovery_scanning
                       && controller.emulator_discovery_revision
                          === window.emulatorDiscoveryInitialRevision + 1) {
                let candidateIndex = -1
                for (let index = 0;
                        index < controller.discovered_emulator_count(); ++index) {
                    if (controller.discovered_emulator_profile_id_at(index) === "pcsx2"
                            && controller.discovered_emulator_source_at(index)
                               === "LaunchBox Emulators folder"
                            && controller.discovered_emulator_path_at(index)
                               .endsWith("/Emulators/PCSX2/pcsx2-qt")) {
                        candidateIndex = index
                        break
                    }
                }
                if (candidateIndex < 0
                        || controller.discovered_emulator_registered_at(candidateIndex)) {
                    console.error("EMULATOR_DISCOVERY_SMOKE_CANDIDATE_MISSING")
                    Qt.exit(48)
                    return
                }
                window.emulatorDiscoveryCandidateIndex = candidateIndex
                window.emulatorDiscoverySmokePhase = 2
                emulatorEditor.smokeDiscovered(candidateIndex)
            } else if (window.emulatorDiscoverySmokePhase === 2
                       && !controller.writing
                       && controller.emulator_revision
                          === window.emulatorDiscoveryInitialEmulatorRevision + 1
                       && controller.emulator_discovery_revision
                          === window.emulatorDiscoveryInitialRevision + 2) {
                if (!controller.discovered_emulator_registered_at(
                        window.emulatorDiscoveryCandidateIndex)) {
                    console.error("EMULATOR_DISCOVERY_SMOKE_NOT_REGISTERED")
                    Qt.exit(48)
                    return
                }
                window.emulatorDiscoverySmokePhase = 3
                controller.scan_installed_emulators()
            } else if (window.emulatorDiscoverySmokePhase === 3
                       && !controller.emulator_discovery_scanning
                       && controller.emulator_discovery_revision
                          === window.emulatorDiscoveryInitialRevision + 3) {
                let candidateIndex = -1
                for (let index = 0;
                        index < controller.discovered_emulator_count(); ++index) {
                    if (controller.discovered_emulator_profile_id_at(index) === "pcsx2"
                            && controller.discovered_emulator_source_at(index)
                               === "LaunchBox Emulators folder") {
                        candidateIndex = index
                        break
                    }
                }
                if (candidateIndex < 0
                        || !controller.report_emulator_discovery_smoke_success(
                            candidateIndex,
                            window.emulatorDiscoveryInitialEmulatorRevision,
                            window.emulatorDiscoveryInitialRevision)) {
                    console.error("EMULATOR_DISCOVERY_SMOKE_MODEL_CONTRACT_FAILED")
                    Qt.exit(48)
                    return
                }
                window.emulatorDiscoverySmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 20000
        running: window.emulatorDiscoverySmokeTest
                 && !window.emulatorDiscoverySmokeFinished
        onTriggered: {
            console.error("EMULATOR_DISCOVERY_SMOKE_TIMEOUT phase="
                          + window.emulatorDiscoverySmokePhase
                          + " status=" + controller.status_message)
            Qt.exit(48)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.emulatorBiosSmokeTest && !window.emulatorBiosSmokeFinished
        onTriggered: {
            const emulatorId = "pcsx2-bios-fixture"
            if (window.emulatorBiosSmokePhase === 0 && !controller.loading
                    && controller.library_path.length > 0
                    && controller.emulator_entry_count() === 3) {
                if (!controller.emulator_bios_supported(emulatorId)) {
                    console.error("EMULATOR_BIOS_SMOKE_ADAPTER_NOT_SUPPORTED")
                    Qt.exit(49)
                    return
                }
                window.emulatorBiosInitialRevision = controller.emulator_bios_revision
                window.emulatorBiosSmokePhase = 1
                biosManager.smokeAudit(emulatorId, "PCSX2")
            } else if (window.emulatorBiosSmokePhase === 1
                       && !controller.emulator_bios_scanning
                       && controller.emulator_bios_revision
                          === window.emulatorBiosInitialRevision + 1) {
                const serialized = controller.emulator_bios_audit_json
                const payload = serialized.length > 0
                              ? JSON.parse(serialized) : null
                let mismatchFound = false
                let unsafeFound = false
                if (payload !== null) {
                    for (let index = 0; index < payload.files.length; ++index) {
                        const file = payload.files[index]
                        if (file.file_name === "ps2-0100jd-20000117.bin"
                                && file.state === "hash_mismatch"
                                && file.actual_md5 !== null)
                            mismatchFound = true
                        if (file.file_name === "ps2-0100j-20000117.bin"
                                && file.state === "unsafe_entry"
                                && file.actual_md5 === null)
                            unsafeFound = true
                    }
                }
                const group = payload !== null && payload.groups.length > 0
                            ? payload.groups[0] : null
                if (payload === null || payload.version !== 3
                        || payload.emulator_id !== emulatorId
                        || payload.adapter !== "pcsx2"
                        || payload.targets.length !== 0
                        || !payload.search_root.endsWith(
                            "/Emulators/PCSX2/custom-bios")
                        || payload.configuration_path === null
                        || !payload.configuration_path.endsWith(
                            "/Emulators/PCSX2/inis/PCSX2.ini")
                        || payload.location_source
                           !== "portable PCSX2 configuration"
                        || group === null
                        || group.id !== "ps2 bios"
                        || !group.required
                        || group.rule !== "any"
                        || group.all_items_required
                        || group.satisfied
                        || group.valid_count !== 0
                        || group.mismatch_count !== 1
                        || group.unsafe_count !== 1
                        || group.unreadable_count !== 0
                        || group.missing_count !== 71
                        || payload.files.length !== 73
                        || !mismatchFound || !unsafeFound
                        || !controller.report_emulator_bios_smoke_success(
                            emulatorId, window.emulatorBiosInitialRevision)) {
                    console.error("EMULATOR_BIOS_SMOKE_MODEL_CONTRACT_FAILED payload="
                                  + serialized)
                    Qt.exit(49)
                    return
                }
                window.emulatorBiosSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 20000
        running: window.emulatorBiosSmokeTest && !window.emulatorBiosSmokeFinished
        onTriggered: {
            console.error("EMULATOR_BIOS_SMOKE_TIMEOUT phase="
                          + window.emulatorBiosSmokePhase
                          + " status=" + controller.status_message)
            Qt.exit(49)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.emulatorInstallSmokeTest
                 && !window.emulatorInstallSmokeFinished
        onTriggered: {
            if (window.emulatorInstallSmokePhase === 0 && !controller.loading
                    && controller.library_path.length > 0) {
                window.emulatorInstallInitialRevision =
                    controller.emulator_install_revision
                window.emulatorInstallSmokePhase = 1
                pcsx2InstallManager.smokeCheck()
            } else if (window.emulatorInstallSmokePhase === 1
                       && !controller.emulator_release_checking
                       && controller.emulator_release_json.length > 0) {
                const payload = JSON.parse(controller.emulator_release_json)
                if (payload.version !== 1 || payload.profile_id !== "pcsx2"
                        || payload.release.version !== "2.7.492"
                        || payload.release.tag !== "v2.7.492"
                        || !payload.release.prerelease
                        || payload.release.artifact_kind
                           !== "linux_app_image_x64"
                        || payload.release.asset_sha256.length !== 64
                        || payload.action !== "install" || !payload.can_install
                        || payload.managed_install !== null
                        || !payload.read_only_check) {
                    console.error(
                        "EMULATOR_INSTALL_SMOKE_RELEASE_CONTRACT_FAILED payload="
                        + controller.emulator_release_json)
                    Qt.exit(50)
                    return
                }
                window.emulatorInstallSmokePhase = 2
                controller.install_pcsx2_release()
            } else if (window.emulatorInstallSmokePhase === 2
                       && !controller.emulator_installing
                       && controller.emulator_install_revision
                          === window.emulatorInstallInitialRevision + 2) {
                if (!controller.report_emulator_install_smoke_success(
                        window.emulatorInstallInitialRevision)) {
                    console.error(
                        "EMULATOR_INSTALL_SMOKE_MODEL_CONTRACT_FAILED status="
                        + controller.status_message)
                    Qt.exit(50)
                    return
                }
                window.emulatorInstallSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 30000
        running: window.emulatorInstallSmokeTest
                 && !window.emulatorInstallSmokeFinished
        onTriggered: {
            console.error("EMULATOR_INSTALL_SMOKE_TIMEOUT phase="
                          + window.emulatorInstallSmokePhase
                          + " revision=" + controller.emulator_install_revision
                          + " status=" + controller.status_message)
            Qt.exit(50)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.emulatorRemoveSmokeTest
                 && !window.emulatorRemoveSmokeFinished
        onTriggered: {
            if (window.emulatorRemoveSmokePhase === 0 && !controller.loading
                    && controller.library_path.length > 0) {
                window.emulatorRemoveInitialRevision =
                    controller.emulator_install_revision
                window.emulatorRemoveSmokePhase = 1
                controller.review_managed_pcsx2()
            } else if (window.emulatorRemoveSmokePhase === 1
                       && !controller.emulator_managed_checking
                       && controller.emulator_managed_json.length > 0) {
                const review =
                    JSON.parse(controller.emulator_managed_json)
                if (review.version !== 1 || review.profile_id !== "pcsx2"
                        || review.managed_install === null
                        || review.managed_install.manifest.schema_version !== 2
                        || review.managed_install.manifest.installed_files.length !== 2
                        || review.owned_file_count !== 3
                        || review.reference_count !== 0
                        || !review.can_remove
                        || review.blocked_reason !== null
                        || !review.read_only_check) {
                    console.error(
                        "EMULATOR_REMOVE_SMOKE_REVIEW_CONTRACT_FAILED payload="
                        + controller.emulator_managed_json)
                    Qt.exit(51)
                    return
                }
                window.emulatorRemoveSmokePhase = 2
                controller.remove_managed_pcsx2()
            } else if (window.emulatorRemoveSmokePhase === 2
                       && !controller.writing
                       && controller.emulator_install_revision
                          === window.emulatorRemoveInitialRevision + 2) {
                if (!controller.report_emulator_remove_smoke_success(
                        window.emulatorRemoveInitialRevision)) {
                    console.error(
                        "EMULATOR_REMOVE_SMOKE_MODEL_CONTRACT_FAILED status="
                        + controller.status_message)
                    Qt.exit(51)
                    return
                }
                window.emulatorRemoveSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 20000
        running: window.emulatorRemoveSmokeTest
                 && !window.emulatorRemoveSmokeFinished
        onTriggered: {
            console.error("EMULATOR_REMOVE_SMOKE_TIMEOUT phase="
                          + window.emulatorRemoveSmokePhase
                          + " revision=" + controller.emulator_install_revision
                          + " status=" + controller.status_message)
            Qt.exit(51)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.categoryCrudSmokeTest && !window.categoryCrudSmokeFinished
        onTriggered: {
            const categoryName = "Portable Collections"
            if (window.categoryCrudSmokePhase === 0 && !controller.loading
                    && controller.library_path.length > 0
                    && controller.navigation_entry_count === 3) {
                window.categoryCrudSmokePhase = 1
                categoryEditor.smokeCreate(categoryName, "platform_category",
                                           "Fixture Category")
            } else if (window.categoryCrudSmokePhase === 1 && !controller.writing) {
                const categoryIndex = window.navigationIndex("category", categoryName)
                const platformIndex = window.navigationIndex("platform", "Fixture Console")
                if (categoryIndex < 0 || platformIndex < 0
                        || controller.navigation_entry_depth_at(categoryIndex) !== 1
                        || controller.navigation_entry_depth_at(platformIndex) !== 2
                        || controller.navigation_entry_game_count_at(categoryIndex) !== 3) {
                    console.error("CATEGORY_CRUD_SMOKE_BAD_NESTING categoryIndex="
                                  + categoryIndex + " platformIndex=" + platformIndex)
                    Qt.exit(10)
                    return
                }
                window.categoryCrudSmokePhase = 2
                categoryEditor.smokeSave(categoryName)
            } else if (window.categoryCrudSmokePhase === 2 && !controller.writing) {
                const serialized = controller.category_edit_payload(categoryName)
                if (serialized.length === 0) {
                    console.error("CATEGORY_CRUD_SMOKE_EDIT_PAYLOAD_MISSING")
                    Qt.exit(10)
                    return
                }
                const payload = JSON.parse(serialized)
                let hasRoot = false
                for (let index = 0; index < payload.parents.length; ++index) {
                    if (payload.parents[index].target_kind === "root")
                        hasRoot = true
                }
                if (payload.category.nested_name !== "Portable"
                        || payload.category.sort_title !== "Collections, Portable"
                        || payload.category.notes
                           !== "Edited through the real category dialog."
                        || payload.category.video_path
                           !== "Videos\\Portable Collections\\theme.mp4"
                        || payload.category.image_type !== "Clear Logo"
                        || !payload.category.hide_in_big_box
                        || payload.parents.length !== 2 || !hasRoot) {
                    console.error("CATEGORY_CRUD_SMOKE_EDIT_NOT_PERSISTED payload="
                                  + serialized)
                    Qt.exit(10)
                    return
                }
                window.categoryCrudSmokePhase = 3
                deleteCategoryConfirmation.smokeDelete(categoryName)
            } else if (window.categoryCrudSmokePhase === 3 && !controller.writing
                       && window.navigationIndex("category", categoryName) < 0) {
                const platformIndex = window.navigationIndex("platform", "Fixture Console")
                if (platformIndex < 0
                        || controller.navigation_entry_depth_at(platformIndex) !== 0
                        || !controller.report_category_crud_smoke_success(categoryName, 1)) {
                    console.error("CATEGORY_CRUD_SMOKE_MODEL_CONTRACT_FAILED")
                    Qt.exit(10)
                    return
                }
                window.categoryCrudSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 20000
        running: window.categoryCrudSmokeTest && !window.categoryCrudSmokeFinished
        onTriggered: {
            console.error("CATEGORY_CRUD_SMOKE_TIMEOUT phase="
                          + window.categoryCrudSmokePhase
                          + " status=" + controller.status_message)
            Qt.exit(10)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.playlistCrudSmokeTest && !window.playlistCrudSmokeFinished
        onTriggered: {
            const parentName = "Portable/Queue"
            const childName = "Portable Child"
            if (window.playlistCrudSmokePhase === 0 && !controller.loading
                    && controller.library_path.length > 0
                    && controller.navigation_entry_count === 3) {
                window.playlistCrudSmokePhase = 1
                playlistEditor.smokeCreateParent(parentName)
            } else if (window.playlistCrudSmokePhase === 1 && !controller.writing) {
                const parentIndex = window.navigationIndex(
                            "playlist", window.playlistCrudParentId)
                if (parentIndex < 0
                        || controller.navigation_entry_depth_at(parentIndex) !== 0
                        || controller.navigation_entry_game_count_at(parentIndex) !== 1) {
                    console.error("PLAYLIST_CRUD_SMOKE_CREATE_NOT_VISIBLE index="
                                  + parentIndex)
                    Qt.exit(11)
                    return
                }
                controller.apply_playlist_filter("", window.playlistCrudParentId)
                if (controller.filtered_count !== 1) {
                    console.error("PLAYLIST_CRUD_SMOKE_MANUAL_FILTER_FAILED count="
                                  + controller.filtered_count)
                    Qt.exit(11)
                    return
                }
                window.playlistCrudSmokePhase = 2
                playlistEditor.smokeSaveParent(window.playlistCrudParentId)
            } else if (window.playlistCrudSmokePhase === 2 && !controller.writing) {
                const serialized = controller.playlist_edit_payload(
                            window.playlistCrudParentId)
                if (serialized.length === 0) {
                    console.error("PLAYLIST_CRUD_SMOKE_EDIT_PAYLOAD_MISSING")
                    Qt.exit(11)
                    return
                }
                const payload = JSON.parse(serialized)
                const parentIndex = window.navigationIndex(
                            "playlist", window.playlistCrudParentId)
                if (payload.playlist.name !== parentName
                        || payload.playlist.nested_name !== "Portable Favorites"
                        || payload.playlist.sort_title !== "Favorites, Portable"
                        || payload.playlist.notes
                           !== "Edited through the real playlist dialog."
                        || payload.playlist.video_path
                           !== "Videos\\Portable Favorites\\theme.mp4"
                        || payload.playlist.image_type !== "Clear Logo"
                        || payload.playlist.category !== "Arcade"
                        || payload.playlist.last_game_id !== "fixture-adventure"
                        || payload.playlist.big_box_view !== "TextGamesView"
                        || payload.playlist.big_box_theme !== "Default"
                        || !payload.playlist.hide_in_big_box
                        || !payload.playlist.include_with_platforms
                        || !payload.playlist.auto_populate
                        || payload.filters.length !== 1
                        || payload.filters[0].field_key !== "Favorite"
                        || payload.filters[0].comparison_type_key !== "IsTrue"
                        || parentIndex < 0
                        || controller.navigation_entry_name_at(parentIndex)
                           !== "Portable Favorites"
                        || controller.navigation_entry_game_count_at(parentIndex) !== 1
                        || controller.filtered_count !== 1) {
                    console.error("PLAYLIST_CRUD_SMOKE_EDIT_NOT_PERSISTED payload="
                                  + serialized + " filtered=" + controller.filtered_count)
                    Qt.exit(11)
                    return
                }
                window.playlistCrudSmokePhase = 3
                playlistEditor.smokeCreateChild(childName,
                                                window.playlistCrudParentId)
            } else if (window.playlistCrudSmokePhase === 3 && !controller.writing) {
                const parentIndex = window.navigationIndex(
                            "playlist", window.playlistCrudParentId)
                const childIndex = window.navigationIndex(
                            "playlist", window.playlistCrudChildId)
                if (parentIndex < 0 || childIndex < 0
                        || controller.navigation_entry_depth_at(childIndex)
                           !== controller.navigation_entry_depth_at(parentIndex) + 1) {
                    console.error("PLAYLIST_CRUD_SMOKE_CHILD_NOT_NESTED parent="
                                  + parentIndex + " child=" + childIndex)
                    Qt.exit(11)
                    return
                }
                window.playlistCrudSmokePhase = 4
                deletePlaylistConfirmation.smokeDelete(
                            window.playlistCrudParentId, parentName)
            } else if (window.playlistCrudSmokePhase === 4 && !controller.writing
                       && window.navigationIndex(
                           "playlist", window.playlistCrudParentId) < 0) {
                const childIndex = window.navigationIndex(
                            "playlist", window.playlistCrudChildId)
                if (childIndex < 0
                        || controller.navigation_entry_depth_at(childIndex) !== 0) {
                    console.error("PLAYLIST_CRUD_SMOKE_CHILD_NOT_DETACHED child="
                                  + childIndex)
                    Qt.exit(11)
                    return
                }
                window.playlistCrudSmokePhase = 5
                deletePlaylistConfirmation.smokeDelete(
                            window.playlistCrudChildId, childName)
            } else if (window.playlistCrudSmokePhase === 5 && !controller.writing
                       && window.navigationIndex(
                           "playlist", window.playlistCrudChildId) < 0) {
                if (!controller.report_playlist_crud_smoke_success(
                        window.playlistCrudParentId, 1, 0)) {
                    console.error("PLAYLIST_CRUD_SMOKE_MODEL_CONTRACT_FAILED")
                    Qt.exit(11)
                    return
                }
                window.playlistCrudSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 25000
        running: window.playlistCrudSmokeTest && !window.playlistCrudSmokeFinished
        onTriggered: {
            console.error("PLAYLIST_CRUD_SMOKE_TIMEOUT phase="
                          + window.playlistCrudSmokePhase
                          + " status=" + controller.status_message)
            Qt.exit(11)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.launchSmokeTest && !window.launchSmokeFinished
        onTriggered: {
            if (window.launchSmokePhase === 0 && !controller.loading
                    && controller.library_path.length > 0
                    && controller.game_count > 0) {
                const row = controller.row_for_game_id(window.launchSmokeGameId)
                if (row < 0) {
                    console.error("LAUNCH_SMOKE_MISSING_GAME id="
                                  + window.launchSmokeGameId)
                    Qt.exit(7)
                    return
                }
                window.launchSmokePhase = 1
                if (window.launchSmokeAdditionalApplicationId.length > 0)
                    controller.launch_additional_application(
                                row, window.launchSmokeGameId,
                                window.launchSmokeAdditionalApplicationId)
                else
                    controller.launch_game(row, window.launchSmokeGameId)
            } else if (window.launchSmokePhase === 1 && !controller.launching
                       && !controller.launch_session_active) {
                const contractOk = window.launchSmokeAdditionalApplicationId.length > 0
                    ? controller.report_additional_application_launch_smoke_success(
                          window.launchSmokeGameId,
                          window.launchSmokeAdditionalApplicationId)
                    : controller.report_launch_smoke_success(window.launchSmokeGameId)
                if (!controller.last_launch_succeeded || !contractOk) {
                    console.error("LAUNCH_SMOKE_FAILED id="
                                  + window.launchSmokeGameId
                                  + " status=" + controller.status_message)
                    Qt.exit(7)
                    return
                }
                window.launchSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 15000
        running: window.launchSmokeTest && !window.launchSmokeFinished
        onTriggered: {
            console.error("LAUNCH_SMOKE_TIMEOUT phase=" + window.launchSmokePhase
                          + " status=" + controller.status_message)
            Qt.exit(7)
        }
    }

    Timer {
        interval: 10
        repeat: true
        running: window.launchLifecycleSmokeTest
                 && !window.launchLifecycleSmokeFinished
        onTriggered: {
            if (window.launchLifecycleSmokePhase === 0 && !controller.loading
                    && controller.library_path.length > 0
                    && controller.game_count > 0) {
                const row = controller.row_for_game_id("fixture-racer")
                if (row < 0) {
                    console.error("LAUNCH_LIFECYCLE_SMOKE_MISSING_GAME")
                    Qt.exit(7)
                    return
                }
                window.launchLifecycleSmokePhase = 1
                controller.launch_game(row, "fixture-racer")
            } else if (window.launchLifecycleSmokePhase >= 1) {
                if (controller.startup_screen_active
                        && !controller.startup_screen_primary_started)
                    window.launchLifecycleStartupVisibleSeen = true
                if (controller.startup_screen_active
                        && controller.startup_screen_primary_started)
                    window.launchLifecyclePrimaryStartedSeen = true
                if (window.launchLifecyclePrimaryStartedSeen
                        && !controller.startup_screen_active
                        && controller.launch_session_active) {
                    window.launchLifecycleDismissedBeforeExit = true
                    window.launchLifecycleSmokePhase = 2
                }
                if (!controller.launching && !controller.launch_session_active
                        && !controller.startup_screen_active
                        && !controller.shutdown_screen_active) {
                    const startupPrelaunchMs =
                        window.launchLifecycleStartupPresentedAt > 0
                        && window.launchLifecyclePrimaryStartedAt > 0
                        ? Math.round(
                              window.launchLifecyclePrimaryStartedAt
                              - window.launchLifecycleStartupPresentedAt)
                        : 0
                    const startupVisibleMs =
                        window.launchLifecycleStartupPresentedAt > 0
                        && window.launchLifecycleStartupDismissedAt > 0
                        ? Math.round(
                              window.launchLifecycleStartupDismissedAt
                              - window.launchLifecycleStartupPresentedAt)
                        : 0
                    const shutdownVisibleMs =
                        window.launchLifecycleShutdownPresentedAt > 0
                        && window.launchLifecycleShutdownDismissedAt > 0
                        ? Math.round(
                              window.launchLifecycleShutdownDismissedAt
                              - window.launchLifecycleShutdownPresentedAt)
                        : 0
                    const presentationFlags =
                        (window.launchLifecycleStartupVisibleSeen ? 1 : 0)
                        | (window.launchLifecyclePrimaryStartedSeen ? 2 : 0)
                        | (window.launchLifecycleDismissedBeforeExit ? 4 : 0)
                        | (window.launchLifecycleShutdownVisibleSeen ? 8 : 0)
                        | (window.launchLifecycleShortProcess ? 16 : 0)
                    const contractOk =
                        controller.report_launch_lifecycle_smoke_success(
                            "fixture-racer",
                            presentationFlags,
                            startupPrelaunchMs,
                            startupVisibleMs,
                            shutdownVisibleMs)
                    if (!contractOk) {
                        console.error(
                            "LAUNCH_LIFECYCLE_SMOKE_FAILED visible="
                            + window.launchLifecycleStartupVisibleSeen
                            + " primary="
                            + window.launchLifecyclePrimaryStartedSeen
                            + " dismissed="
                            + window.launchLifecycleDismissedBeforeExit
                            + " prelaunchMs=" + startupPrelaunchMs
                            + " startupMs=" + startupVisibleMs
                            + " shutdown="
                            + window.launchLifecycleShutdownVisibleSeen
                            + " shutdownMs=" + shutdownVisibleMs
                            + " status=" + controller.status_message)
                        Qt.exit(7)
                        return
                    }
                    window.launchLifecycleSmokeFinished = true
                    Qt.quit()
                }
            }
        }
    }

    Timer {
        interval: 15000
        running: window.launchLifecycleSmokeTest
                 && !window.launchLifecycleSmokeFinished
        onTriggered: {
            console.error("LAUNCH_LIFECYCLE_SMOKE_TIMEOUT phase="
                          + window.launchLifecycleSmokePhase
                          + " startup=" + controller.startup_screen_active
                          + " primary="
                          + controller.startup_screen_primary_started
                          + " shutdown=" + controller.shutdown_screen_active
                          + " session=" + controller.launch_session_active
                          + " status=" + controller.status_message)
            Qt.exit(7)
        }
    }

    Timer {
        interval: 50
        repeat: true
        running: window.loadSmokeTest && controller.loading
        onTriggered: window.loadHeartbeat += 1
    }

    Timer {
        interval: 50
        repeat: true
        running: window.loadSmokeTest
        onTriggered: {
            if (!controller.loading && controller.library_path.length > 0) {
                controller.report_load_smoke_success(controller.game_count,
                                                     controller.platform_entry_count,
                                                     window.loadHeartbeat)
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 45000
        running: window.loadSmokeTest
        onTriggered: {
            console.error("LOAD_SMOKE_TIMEOUT status=" + controller.status_message)
            Qt.exit(3)
        }
    }

    Timer {
        interval: 10000
        running: window.editSmokeTest && !window.editSmokeFinished
        onTriggered: {
            console.error("EDIT_SMOKE_TIMEOUT phase=" + window.editSmokePhase
                          + " filtered=" + controller.filtered_count
                          + " writing=" + controller.writing
                          + " status=" + controller.status_message)
            Qt.exit(5)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.editSmokeTest && window.editSmokePhase >= 2
                 && !window.editSmokeFinished
        onTriggered: {
            if (window.editSmokePhase === 2 && !controller.writing
                    && controller.filtered_count === 1) {
                window.editSmokePhase = 3
                gameEditor.smokeSaveMetadata(0)
            } else if (window.editSmokePhase === 3 && !controller.writing
                       && controller.filtered_count === 0) {
                if (!controller.report_title_edit_smoke_success(
                        "fixture-adventure", "Renamed Adventure")) {
                    console.error("EDIT_SMOKE_TITLE_FILTER_FAILED")
                    Qt.exit(5)
                    return
                }
                window.editSmokeFinished = true
                Qt.quit()
            }
        }
    }

    header: ToolBar {
        background: Rectangle { color: "#20252d" }
        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: 14
            anchors.rightMargin: 14
            spacing: 10

            Label {
                text: "LAUNCHBOX"
                color: "#58a6ff"
                font.pixelSize: 20
                font.bold: true
            }
            TextField {
                id: searchField
                Layout.fillWidth: true
                placeholderText: "Search title and descriptive metadata"
                onTextChanged: window.applyCurrentFilter()
            }
            TextField {
                id: pathField
                Layout.preferredWidth: 350
                placeholderText: "/path/to/LaunchBox"
                text: controller.library_path
                onAccepted: controller.load_library(text)
            }
            Button {
                text: controller.loading ? "Loading…" : "Load Library"
                enabled: !controller.loading && !controller.writing
                         && !controller.launching
                onClicked: controller.load_library(pathField.text)
            }
            Button {
                text: "Host Paths…"
                onClicked: pathMappingsDialog.open()
            }
            BusyIndicator {
                running: controller.loading || controller.writing || controller.launching
                visible: running
                Layout.preferredWidth: 32
                Layout.preferredHeight: 32
            }
        }
    }

    SplitView {
        anchors.fill: parent

        Rectangle {
            SplitView.preferredWidth: 235
            color: "#191d24"
            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 12
                spacing: 8

                RowLayout {
                    Layout.fillWidth: true
                    Label {
                        Layout.fillWidth: true
                        text: "Library"
                        color: "#aeb8c5"
                        font.bold: true
                        font.pixelSize: 16
                    }
                    ToolButton {
                        text: "+ List"
                        Accessible.name: "Add playlist"
                        enabled: controller.library_path.length > 0
                                 && !controller.loading && !controller.writing
                                 && !controller.launching
                                 && !controller.write_conflict
                                 && controller.pending_recovery_count === 0
                        onClicked: playlistEditor.prepareCreate()
                    }
                    ToolButton {
                        text: "+ Cat"
                        Accessible.name: "Add platform category"
                        enabled: controller.library_path.length > 0
                                 && !controller.loading && !controller.writing
                                 && !controller.launching
                                 && !controller.write_conflict
                                 && controller.pending_recovery_count === 0
                        onClicked: categoryEditor.prepareCreate()
                    }
                    ToolButton {
                        text: "+ Plat"
                        Accessible.name: "Add platform"
                        enabled: controller.library_path.length > 0
                                 && !controller.loading && !controller.writing
                                 && !controller.launching
                                 && !controller.write_conflict
                                 && controller.pending_recovery_count === 0
                        onClicked: addPlatformDialog.prepare()
                    }
                }
                RowLayout {
                    Layout.fillWidth: true
                    Button {
                        Layout.fillWidth: true
                        text: "Edit"
                        Accessible.name: "Edit selected playlist, category, or platform"
                        enabled: (window.selectedNavigationKind === "platform"
                                  || window.selectedNavigationKind === "category"
                                  || window.selectedNavigationKind === "playlist")
                                 && !controller.loading && !controller.writing
                                 && !controller.launching
                                 && !controller.write_conflict
                                 && controller.pending_recovery_count === 0
                        onClicked: {
                            if (window.selectedNavigationKind === "category")
                                categoryEditor.prepareEdit(window.selectedNavigationKey)
                            else if (window.selectedNavigationKind === "playlist")
                                playlistEditor.prepareEdit(window.selectedNavigationKey)
                            else
                                platformEditor.prepare(window.selectedNavigationKey)
                        }
                    }
                    Button {
                        Layout.fillWidth: true
                        text: "Delete"
                        Accessible.name: "Delete selected playlist, category, or platform"
                        enabled: (window.selectedNavigationKind === "platform"
                                  || window.selectedNavigationKind === "category"
                                  || window.selectedNavigationKind === "playlist")
                                 && !controller.loading && !controller.writing
                                 && !controller.launching
                                 && !controller.write_conflict
                                 && controller.pending_recovery_count === 0
                        onClicked: {
                            if (window.selectedNavigationKind === "category")
                                deleteCategoryConfirmation.prepare(
                                            window.selectedNavigationKey)
                            else if (window.selectedNavigationKind === "playlist")
                                deletePlaylistConfirmation.prepare(
                                            window.selectedNavigationKey,
                                            window.selectedNavigationName)
                            else
                                deletePlatformConfirmation.prepare(
                                            window.selectedNavigationKey)
                        }
                    }
                }
                ListView {
                    id: platformList
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    model: {
                        const revision = controller.platform_revision
                        return controller.navigation_entry_count + 1
                    }
                    delegate: ItemDelegate {
                        id: platformDelegate
                        required property int index
                        property string entryKind: {
                            const revision = controller.platform_revision
                            return index === 0 ? "all"
                                               : controller.navigation_entry_kind_at(index - 1)
                        }
                        property string entryKey: {
                            const revision = controller.platform_revision
                            return index === 0 ? ""
                                               : controller.navigation_entry_key_at(index - 1)
                        }
                        property string entryName: {
                            const revision = controller.platform_revision
                            return index === 0 ? "All Games"
                                               : controller.navigation_entry_name_at(index - 1)
                        }
                        property int entryDepth: {
                            const revision = controller.platform_revision
                            return index === 0 ? 0
                                               : controller.navigation_entry_depth_at(index - 1)
                        }
                        property int entryCount: {
                            const revision = controller.platform_revision
                            return index === 0 ? controller.game_count
                                               : controller.navigation_entry_game_count_at(index - 1)
                        }
                        width: platformList.width
                        highlighted: entryKind === window.selectedNavigationKind
                                     && entryKey === window.selectedNavigationKey
                        contentItem: RowLayout {
                            spacing: 6
                            Label {
                                Layout.leftMargin: platformDelegate.entryDepth * 16
                                text: platformDelegate.entryKind === "category" ? "▾"
                                      : platformDelegate.entryKind === "playlist" ? "≡" : "▪"
                                color: platformDelegate.entryKind === "category"
                                       ? "#7fbfff"
                                       : platformDelegate.entryKind === "playlist"
                                         ? "#a78bfa" : "#7d8590"
                            }
                            Label {
                                Layout.fillWidth: true
                                text: entryName
                                color: platformDelegate.highlighted ? "#ffffff" : "#c9d1d9"
                                elide: Text.ElideRight
                            }
                            Label {
                                text: entryCount
                                color: "#7d8590"
                            }
                        }
                        onClicked: {
                            window.selectedNavigationKind = entryKind
                            window.selectedNavigationKey = entryKey
                            window.selectedNavigationName = entryName
                            window.selectedPlatform = entryKind === "platform" ? entryKey : ""
                            window.applyCurrentFilter()
                        }
                    }
                }
            }
        }

        Rectangle {
            color: "#101318"
            SplitView.fillWidth: true
            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 14
                spacing: 8

                RowLayout {
                    Layout.fillWidth: true
                    Label {
                        Layout.fillWidth: true
                        text: window.selectedNavigationName
                        color: "white"
                        font.pixelSize: 24
                        font.bold: true
                    }
                    Label {
                        text: controller.filtered_count + " shown / " + controller.game_count + " total"
                        color: "#8b949e"
                    }
                    Button {
                        text: "Emulators…"
                        enabled: controller.library_path.length > 0
                                 && !controller.loading && !controller.writing
                                 && !controller.launching
                                 && !controller.write_conflict
                                 && controller.pending_recovery_count === 0
                        onClicked: emulatorManager.openManager()
                    }
                    Button {
                        text: "Import ROMs"
                        enabled: controller.library_path.length > 0
                                 && controller.platform_entry_count > 0
                                 && !controller.loading && !controller.import_scanning
                                 && !controller.writing && !controller.launching
                                 && !controller.write_conflict
                                 && controller.pending_recovery_count === 0
                        onClicked: romImportDialog.prepare()
                    }
                    Button {
                        text: "Add Game"
                        enabled: controller.library_path.length > 0
                                 && controller.platform_entry_count > 0
                                 && !controller.loading && !controller.import_scanning
                                 && !controller.writing
                                 && !controller.launching
                                 && !controller.write_conflict
                                 && controller.pending_recovery_count === 0
                        onClicked: addGameDialog.prepare()
                    }
                }

                Label {
                    Layout.fillWidth: true
                    text: controller.library_path.length > 0
                          ? "Double-click a game to edit its descriptive metadata and library state."
                          : "Load a library directory to enable safe transactional editing."
                    color: "#7d8590"
                    font.pixelSize: 12
                }

                GridView {
                    id: gameGrid
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    cellWidth: 210
                    cellHeight: 150
                    model: controller
                    delegate: Rectangle {
                        required property int index
                        required property string gameTitle
                        required property string gameId
                        required property string gamePlatform
                        required property bool gameFavorite
                        required property bool gameCompleted
                        required property int gamePlayCount
                        required property int gameStarRating
                        required property int gameAdditionalApplicationCount
                        required property int gameSaveCount
                        required property string gameSortTitle
                        required property string gameNotes
                        required property string gameDeveloper
                        required property string gameGenre
                        required property int gameMaxPlayers
                        required property string gamePlayMode
                        required property string gameProgress
                        required property string gamePublisher
                        required property string gameRating
                        required property string gameRegion
                        required property string gameReleaseDate
                        required property string gameReleaseType
                        required property string gameSeries
                        required property string gameSource
                        required property string gameStatus
                        required property string gameVersion
                        required property string gameWikipediaUrl
                        required property string gameApplicationPath
                        required property string gameCommandLine
                        required property string gameEmulatorId
                        required property bool gameUseDosBox
                        required property string gameCustomDosBoxVersionPath
                        required property string gameDosBoxConfigurationPath
                        required property bool gameUseScummVm
                        required property bool gameScummVmAspectCorrection
                        required property bool gameScummVmFullscreen
                        required property string gameScummVmGameDataFolderPath
                        required property string gameScummVmGameType

                        function verifyEdit() {
                            window.verifyEditState(index, gameId, gameTitle, gameSortTitle,
                                                   gameNotes, gameDeveloper, gameGenre,
                                                   gameMaxPlayers, gamePlayMode, gameProgress,
                                                   gamePublisher, gameRating, gameRegion,
                                                   gameReleaseDate, gameReleaseType, gameSeries,
                                                   gameSource, gameStatus, gameVersion,
                                                   gameWikipediaUrl, gameFavorite,
                                                   gameCompleted, gameStarRating,
                                                   gameApplicationPath, gameCommandLine,
                                                   gameEmulatorId, gameUseDosBox,
                                                   gameCustomDosBoxVersionPath,
                                                   gameDosBoxConfigurationPath,
                                                   gameUseScummVm,
                                                   gameScummVmAspectCorrection,
                                                   gameScummVmFullscreen,
                                                   gameScummVmGameDataFolderPath,
                                                   gameScummVmGameType)
                        }

                        Component.onCompleted: {
                            window.verifyModelRoles(index, gameId, gameTitle, gamePlatform,
                                                    gameFavorite, gameCompleted, gamePlayCount,
                                                    gameStarRating,
                                                    gameAdditionalApplicationCount,
                                                    gameSortTitle, gameNotes, gameDeveloper,
                                                    gameGenre, gameMaxPlayers, gamePlayMode,
                                                    gameProgress, gamePublisher, gameRating,
                                                    gameRegion, gameReleaseDate, gameReleaseType,
                                                    gameSeries, gameSource, gameStatus, gameVersion,
                                                    gameWikipediaUrl, gameApplicationPath,
                                                    gameCommandLine, gameEmulatorId,
                                                    gameUseDosBox,
                                                    gameCustomDosBoxVersionPath,
                                                    gameDosBoxConfigurationPath,
                                                    gameUseScummVm,
                                                    gameScummVmAspectCorrection,
                                                    gameScummVmFullscreen,
                                                    gameScummVmGameDataFolderPath,
                                                    gameScummVmGameType,
                                                    gameGrid.count)
                            verifyEdit()
                        }
                        onGameTitleChanged: {
                            if (window.editSmokePhase > 0)
                                verifyEdit()
                        }
                        onGameFavoriteChanged: {
                            if (window.editSmokePhase > 0)
                                verifyEdit()
                        }
                        onGameCompletedChanged: {
                            if (window.editSmokePhase > 0)
                                verifyEdit()
                        }
                        onGameStarRatingChanged: {
                            if (window.editSmokePhase > 0)
                                verifyEdit()
                        }
                        width: gameGrid.cellWidth - 12
                        height: gameGrid.cellHeight - 12
                        radius: 6
                        color: gameMouse.containsMouse ? "#29313c" : "#20262e"
                        border.color: gameFavorite ? "#e3b341" : "#30363d"
                        border.width: gameFavorite ? 2 : 1

                        ColumnLayout {
                            anchors.fill: parent
                            anchors.margins: 12
                            Label {
                                Layout.fillWidth: true
                                Layout.fillHeight: true
                                text: gameTitle
                                color: "white"
                                font.pixelSize: 18
                                font.bold: true
                                wrapMode: Text.Wrap
                                verticalAlignment: Text.AlignVCenter
                            }
                            Label {
                                Layout.fillWidth: true
                                text: gamePlatform
                                      + (gameFavorite ? "  ★" : "")
                                      + (gameCompleted ? "  ✓" : "")
                                color: "#8b949e"
                                elide: Text.ElideRight
                            }
                        }
                        MouseArea {
                            id: gameMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            enabled: controller.library_path.length > 0
                                     && !controller.loading && !controller.writing
                                     && !controller.launching
                                     && !controller.write_conflict
                                     && controller.pending_recovery_count === 0
                            onDoubleClicked: gameEditor.edit(
                                index, gameId, gameTitle, gameSortTitle, gameNotes,
                                gameDeveloper, gameGenre, gameMaxPlayers, gamePlayMode,
                                gameProgress, gamePublisher, gameRating, gameRegion,
                                gameReleaseDate, gameReleaseType, gameSeries, gameSource,
                                gameStatus, gameVersion, gameWikipediaUrl, gameFavorite,
                                gameCompleted, gameStarRating, gameApplicationPath,
                                gameCommandLine, gameEmulatorId, gameUseDosBox,
                                gameCustomDosBoxVersionPath,
                                gameDosBoxConfigurationPath, gameUseScummVm,
                                gameScummVmAspectCorrection, gameScummVmFullscreen,
                                gameScummVmGameDataFolderPath, gameScummVmGameType)
                        }
                        Button {
                            anchors.right: parent.right
                            anchors.bottom: parent.bottom
                            anchors.margins: 8
                            z: 2
                            text: controller.launching ? "Launching…" : "Play"
                            enabled: controller.library_path.length > 0
                                     && !controller.loading && !controller.writing
                                     && !controller.launching
                                     && !controller.launch_session_active
                                     && controller.pending_recovery_count === 0
                            onClicked: controller.launch_game(index, gameId)
                        }
                        Button {
                            anchors.left: parent.left
                            anchors.bottom: parent.bottom
                            anchors.margins: 8
                            z: 2
                            text: "Launch With…"
                            visible: gameAdditionalApplicationCount > 0
                            enabled: controller.library_path.length > 0
                                     && !controller.loading && !controller.writing
                                     && !controller.launching
                                     && !controller.launch_session_active
                                     && controller.pending_recovery_count === 0
                            onClicked: launchWithDialog.prepare(
                                           index, gameId, gameTitle,
                                           gameAdditionalApplicationCount)
                        }
                        Button {
                            anchors.right: parent.right
                            anchors.top: parent.top
                            anchors.margins: 8
                            z: 2
                            text: "Apps (" + gameAdditionalApplicationCount + ")"
                            enabled: controller.library_path.length > 0
                                     && !controller.loading && !controller.writing
                                     && !controller.launching
                                     && controller.pending_recovery_count === 0
                                     && !controller.write_conflict
                            onClicked: additionalApplicationManager.prepare(
                                           index, gameId, gameTitle)
                        }
                        Button {
                            anchors.left: parent.left
                            anchors.top: parent.top
                            anchors.margins: 8
                            z: 2
                            text: "Saves (" + gameSaveCount + ")"
                            visible: gameSaveCount > 0
                            enabled: controller.library_path.length > 0
                                     && !controller.loading && !controller.writing
                                     && !controller.launching
                                     && controller.pending_recovery_count === 0
                                     && !controller.write_conflict
                            onClicked: gameSaveManager.prepare(
                                           index, gameId, gameTitle)
                        }
                    }
                }
            }
        }
    }

    Dialog {
        id: pathMappingsDialog
        anchors.centerIn: parent
        modal: true
        title: "Host Path Mappings"
        standardButtons: Dialog.Close

        contentItem: ColumnLayout {
            spacing: 10
            Label {
                Layout.preferredWidth: 720
                text: "Translate absolute Windows library paths on this host. Portable LaunchBox-relative paths need no mapping, and original XML paths are never rewritten."
                wrapMode: Text.Wrap
                color: "#aeb8c5"
            }
            Label {
                Layout.preferredWidth: 720
                text: "Settings: " + controller.path_mapping_settings_path
                elide: Text.ElideMiddle
                color: "#7d8590"
            }
            Label {
                text: controller.path_mapping_count > 0
                      ? "Saved mappings"
                      : "No saved mappings"
                font.bold: true
            }
            ListView {
                id: pathMappingList
                Layout.fillWidth: true
                Layout.preferredHeight: controller.path_mapping_count > 0
                                        ? Math.min(180, contentHeight) : 0
                clip: true
                spacing: 4
                model: controller.path_mapping_count
                delegate: RowLayout {
                    required property int index
                    width: ListView.view.width
                    Label {
                        Layout.preferredWidth: 60
                        text: controller.path_mapping_kind_at(index)
                        color: "#7fbfff"
                        font.bold: true
                    }
                    Label {
                        Layout.preferredWidth: 190
                        text: controller.path_mapping_windows_root_at(index)
                        elide: Text.ElideRight
                    }
                    Label {
                        Layout.fillWidth: true
                        text: "→  " + controller.path_mapping_host_root_at(index)
                        elide: Text.ElideMiddle
                    }
                    Button {
                        text: "Remove"
                        onClicked: controller.remove_path_mapping(index)
                    }
                }
            }
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 1
                color: "#30363d"
            }
            Label { text: "Windows drive"; font.bold: true }
            RowLayout {
                Layout.fillWidth: true
                TextField {
                    id: mappingDrive
                    Layout.preferredWidth: 70
                    placeholderText: "D"
                    maximumLength: 1
                }
                TextField {
                    id: mappingDriveRoot
                    Layout.fillWidth: true
                    placeholderText: "/mnt/games"
                }
                Button {
                    text: "Save Drive"
                    onClicked: controller.save_windows_drive_mapping(
                                   mappingDrive.text, mappingDriveRoot.text)
                }
            }
            Label { text: "Windows network share"; font.bold: true }
            RowLayout {
                Layout.fillWidth: true
                TextField {
                    id: mappingServer
                    Layout.preferredWidth: 150
                    placeholderText: "server"
                }
                TextField {
                    id: mappingShare
                    Layout.preferredWidth: 150
                    placeholderText: "share"
                }
                TextField {
                    id: mappingUncRoot
                    Layout.fillWidth: true
                    placeholderText: "/mnt/network-games"
                }
                Button {
                    text: "Save Share"
                    onClicked: controller.save_windows_unc_mapping(
                                   mappingServer.text, mappingShare.text,
                                   mappingUncRoot.text)
                }
            }
            Label {
                Layout.preferredWidth: 720
                text: "Command-line --map-windows-drive and --map-windows-unc values remain temporary overrides and are not written here. Host roots must be absolute paths."
                wrapMode: Text.Wrap
                color: "#7d8590"
            }
        }
    }

    Dialog {
        id: launchWithDialog
        anchors.centerIn: parent
        modal: true
        title: "Launch " + gameTitle + " With"
        standardButtons: Dialog.Cancel
        property int modelRow: -1
        property string gameId: ""
        property string gameTitle: ""

        property int applicationCount: 0

        function prepare(row, id, title, count) {
            modelRow = row
            gameId = id
            gameTitle = title
            applicationCount = count
            open()
        }

        contentItem: ListView {
            implicitWidth: 420
            implicitHeight: Math.min(360, contentHeight)
            spacing: 8
            clip: true
            model: launchWithDialog.applicationCount
            delegate: Button {
                required property int index
                width: ListView.view.width
                text: controller.additional_application_name_at(
                          launchWithDialog.modelRow,
                          launchWithDialog.gameId, index)
                onClicked: {
                    const applicationId = controller.additional_application_id_at(
                                              launchWithDialog.modelRow,
                                              launchWithDialog.gameId, index)
                    const row = launchWithDialog.modelRow
                    const gameId = launchWithDialog.gameId
                    launchWithDialog.close()
                    controller.launch_additional_application(row, gameId, applicationId)
                }
            }
        }
    }

    Dialog {
        id: additionalApplicationManager
        anchors.centerIn: parent
        modal: true
        title: "Additional Applications — " + gameTitle
        standardButtons: Dialog.Close
        property int modelRow: -1
        property string gameId: ""
        property string gameTitle: ""
        property int applicationCount: 0
        property int selectedIndex: -1
        property int observedRevision: controller.additional_application_revision

        onObservedRevisionChanged: {
            if (visible)
                refresh()
        }

        function refresh() {
            applicationCount = controller.additional_application_count(modelRow, gameId)
            if (applicationCount === 0)
                selectedIndex = -1
            else if (selectedIndex < 0 || selectedIndex >= applicationCount)
                selectedIndex = 0
        }

        function prepare(row, id, title) {
            modelRow = row
            gameId = id
            gameTitle = title
            selectedIndex = -1
            refresh()
            open()
        }

        function selectedApplicationId() {
            return selectedIndex >= 0
                   ? controller.additional_application_id_at(
                         modelRow, gameId, selectedIndex) : ""
        }

        contentItem: ColumnLayout {
            spacing: 10
            Label {
                Layout.preferredWidth: 620
                text: "Applications are stored in LaunchBox priority order. Editing or deleting a record never deletes its target file."
                wrapMode: Text.Wrap
                color: "#aeb8c5"
            }
            ListView {
                id: additionalApplicationList
                Layout.fillWidth: true
                Layout.preferredHeight: Math.max(
                                            80, Math.min(320, contentHeight))
                clip: true
                spacing: 4
                model: additionalApplicationManager.applicationCount
                delegate: ItemDelegate {
                    required property int index
                    width: ListView.view.width
                    highlighted: index === additionalApplicationManager.selectedIndex
                    text: controller.additional_application_name_at(
                              additionalApplicationManager.modelRow,
                              additionalApplicationManager.gameId, index)
                    onClicked: additionalApplicationManager.selectedIndex = index
                    onDoubleClicked: {
                        const applicationId =
                            additionalApplicationManager.selectedApplicationId()
                        if (applicationId.length > 0)
                            additionalApplicationEditor.prepareEdit(
                                additionalApplicationManager.modelRow,
                                additionalApplicationManager.gameId,
                                applicationId)
                    }
                }
            }
            RowLayout {
                Layout.fillWidth: true
                Button {
                    text: "Add"
                    enabled: !controller.writing
                    onClicked: additionalApplicationEditor.prepareCreate(
                                   additionalApplicationManager.modelRow,
                                   additionalApplicationManager.gameId)
                }
                Button {
                    text: "Edit"
                    enabled: additionalApplicationManager.selectedIndex >= 0
                             && !controller.writing
                    onClicked: additionalApplicationEditor.prepareEdit(
                                   additionalApplicationManager.modelRow,
                                   additionalApplicationManager.gameId,
                                   additionalApplicationManager.selectedApplicationId())
                }
                Button {
                    text: "Make Default"
                    enabled: additionalApplicationManager.selectedIndex >= 0
                             && !controller.writing
                    onClicked: additionalApplicationDefaultDialog.prepare(
                                   additionalApplicationManager.modelRow,
                                   additionalApplicationManager.gameId,
                                   additionalApplicationManager.selectedApplicationId(),
                                   controller.additional_application_name_at(
                                       additionalApplicationManager.modelRow,
                                       additionalApplicationManager.gameId,
                                       additionalApplicationManager.selectedIndex))
                }
                Button {
                    text: "Delete"
                    enabled: additionalApplicationManager.selectedIndex >= 0
                             && !controller.writing
                    onClicked: additionalApplicationDeleteDialog.prepare(
                                   additionalApplicationManager.modelRow,
                                   additionalApplicationManager.gameId,
                                   additionalApplicationManager.selectedApplicationId(),
                                   controller.additional_application_name_at(
                                       additionalApplicationManager.modelRow,
                                       additionalApplicationManager.gameId,
                                       additionalApplicationManager.selectedIndex))
                }
                Item { Layout.fillWidth: true }
                Label {
                    text: additionalApplicationManager.applicationCount
                          + " application(s)"
                    color: "#7d8590"
                }
            }
        }
    }

    Dialog {
        id: gameSaveManager
        anchors.centerIn: parent
        modal: true
        title: "Game Saves — " + gameTitle
        standardButtons: Dialog.Close
        property int modelRow: -1
        property string gameId: ""
        property string gameTitle: ""
        property var groups: []
        property int selectedGroupIndex: -1
        property int selectedVersionIndex: -1
        property int observedRevision: controller.game_save_revision

        onObservedRevisionChanged: {
            if (visible)
                refresh()
        }

        function selectedGroup() {
            return selectedGroupIndex >= 0
                   && selectedGroupIndex < groups.length
                   ? groups[selectedGroupIndex] : null
        }

        function selectedVersion() {
            const group = selectedGroup()
            return group !== null && selectedVersionIndex >= 0
                   && selectedVersionIndex < group.versions.length
                   ? group.versions[selectedVersionIndex] : null
        }

        function refresh() {
            const payloadText =
                controller.game_save_manager_payload(modelRow, gameId)
            if (payloadText.length === 0) {
                groups = []
                selectedGroupIndex = -1
                selectedVersionIndex = -1
                return
            }
            const payload = JSON.parse(payloadText)
            groups = payload.groups
            if (groups.length === 0) {
                selectedGroupIndex = -1
                selectedVersionIndex = -1
            } else {
                selectedGroupIndex = Math.max(
                    0, Math.min(selectedGroupIndex, groups.length - 1))
                const group = groups[selectedGroupIndex]
                selectedVersionIndex = Math.max(
                    0, Math.min(selectedVersionIndex,
                                group.versions.length - 1))
            }
        }

        function prepare(row, id, title) {
            modelRow = row
            gameId = id
            gameTitle = title
            selectedGroupIndex = 0
            selectedVersionIndex = 0
            refresh()
            open()
        }

        function smokeScan() {
            gameSaveScanButton.clicked()
        }

        contentItem: ColumnLayout {
            spacing: 10
            Label {
                Layout.preferredWidth: 820
                text: "Active identifies a resolved emulator save; Vault identifies a resolved path under LaunchBox/Saves. Unresolved Windows paths need a host mapping. Find Active Saves reads configured RetroArch, Dolphin, and PCSX2 launch targets and records newly discovered saves without deleting existing history. RetroArch discovery covers regular saves, states, and grouped Saturn companion sets. Dolphin discovery covers GameCube memory-card files, Wii title data directories, and save states. Wii directory changes use verified nested 7z archives and retain the previous complete tree as a sibling recovery copy. PCSX2 reads the SYSTEM.CNF serial natively from ISO, raw-sector, GZip, CSO, CHD, MDF/MDS, and NRG disc images, then discovers ordinary save states plus folder-format and raw memory-card members. Card-member backup, restore, and deletion use a validated complete-card working copy and retain the previous card as a sibling recovery copy. Active deletion always archives the exact current save in the vault before removing it."
                wrapMode: Text.Wrap
                color: "#aeb8c5"
            }
            RowLayout {
                Layout.fillWidth: true
                Layout.preferredHeight: 360
                spacing: 12
                ColumnLayout {
                    Layout.preferredWidth: 260
                    Layout.fillHeight: true
                    Label {
                        text: "Save groups"
                        font.bold: true
                    }
                    ListView {
                        id: gameSaveGroupList
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        clip: true
                        spacing: 4
                        model: gameSaveManager.groups
                        delegate: ItemDelegate {
                            required property int index
                            required property var modelData
                            width: ListView.view.width
                            highlighted:
                                index === gameSaveManager.selectedGroupIndex
                            text: modelData.name + "  ("
                                  + modelData.versions.length + ")"
                            onClicked: {
                                gameSaveManager.selectedGroupIndex = index
                                gameSaveManager.selectedVersionIndex = 0
                            }
                        }
                    }
                    RowLayout {
                        Button {
                            text: "Rename"
                            enabled: gameSaveManager.selectedGroup() !== null
                                     && !controller.writing
                            onClicked: gameSaveTextDialog.prepare(
                                           "group",
                                           gameSaveManager.selectedGroup().name)
                        }
                        Button {
                            text: "Combine…"
                            enabled: gameSaveManager.groups.length > 1
                                     && gameSaveManager.selectedGroup() !== null
                                     && !controller.writing
                            onClicked: gameSaveCombineDialog.prepare()
                        }
                    }
                }
                Rectangle {
                    Layout.preferredWidth: 1
                    Layout.fillHeight: true
                    color: "#30363d"
                }
                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    Label {
                        text: "Version history"
                        font.bold: true
                    }
                    ListView {
                        id: gameSaveVersionList
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        clip: true
                        spacing: 6
                        model: {
                            const group = gameSaveManager.selectedGroup()
                            return group !== null ? group.versions : []
                        }
                        delegate: ItemDelegate {
                            required property int index
                            required property var modelData
                            width: ListView.view.width
                            highlighted:
                                index === gameSaveManager.selectedVersionIndex
                            onClicked:
                                gameSaveManager.selectedVersionIndex = index
                            contentItem: Column {
                                spacing: 3
                                Label {
                                    width: parent.width
                                    text: modelData.title + "  ·  "
                                          + modelData.location_kind.toUpperCase()
                                          + (modelData.display_chip_text
                                             ? "  ·  "
                                               + modelData.display_chip_text : "")
                                    font.bold: true
                                    elide: Text.ElideRight
                                }
                                Label {
                                    width: parent.width
                                    text: modelData.file_path
                                    color: "#8b949e"
                                    elide: Text.ElideMiddle
                                }
                                Label {
                                    width: parent.width
                                    visible: modelData.reported_last_modified_utc
                                             || modelData.reported_file_size_bytes
                                    text: (modelData.reported_last_modified_utc || "")
                                          + (modelData.reported_file_size_bytes
                                             ? "  ·  "
                                               + modelData.reported_file_size_bytes
                                               + " bytes" : "")
                                    color: "#7d8590"
                                    elide: Text.ElideRight
                                }
                            }
                        }
                    }
                    RowLayout {
                        Button {
                            id: gameSaveScanButton
                            text: "Find Active Saves"
                            enabled: !controller.writing
                            onClicked: controller.scan_game_saves(
                                           gameSaveManager.modelRow,
                                           gameSaveManager.gameId)
                        }
                        Button {
                            text: "Backup Save"
                            enabled: gameSaveManager.selectedVersion() !== null
                                     && gameSaveManager.selectedVersion().location_kind
                                        === "active"
                                     && !controller.writing
                            onClicked: controller.backup_game_save(
                                           gameSaveManager.modelRow,
                                           gameSaveManager.gameId,
                                           gameSaveManager.selectedVersion().source_index)
                        }
                        Button {
                            text: "Restore…"
                            enabled: gameSaveManager.selectedVersion() !== null
                                     && gameSaveManager.selectedVersion().location_kind
                                        === "vault"
                                     && !controller.writing
                            onClicked: gameSaveRestoreDialog.prepare()
                        }
                        Button {
                            text: "Rename Version"
                            enabled: gameSaveManager.selectedVersion() !== null
                                     && !controller.writing
                            onClicked: gameSaveTextDialog.prepare(
                                           "version",
                                           gameSaveManager.selectedVersion().title)
                        }
                        Button {
                            text: "Make New Save"
                            enabled: gameSaveManager.selectedVersion() !== null
                                     && gameSaveManager.selectedGroup() !== null
                                     && gameSaveManager.selectedGroup().versions.length > 1
                                     && !controller.writing
                            onClicked: gameSaveTextDialog.prepare(
                                           "split",
                                           gameSaveManager.selectedVersion().title
                                           + " (New Save)")
                        }
                        Button {
                            text: "Delete Backup…"
                            enabled: gameSaveManager.selectedVersion() !== null
                                     && gameSaveManager.selectedVersion().location_kind
                                        === "vault"
                                     && !controller.writing
                            onClicked: gameSaveDeleteDialog.prepare()
                        }
                        Button {
                            text: "Delete Active…"
                            enabled: gameSaveManager.selectedVersion() !== null
                                     && gameSaveManager.selectedVersion().location_kind
                                        === "active"
                                     && !controller.writing
                            onClicked: gameSaveActiveDeleteDialog.prepare()
                        }
                        Item { Layout.fillWidth: true }
                        Label {
                            text: gameSaveManager.selectedGroup() !== null
                                  ? gameSaveManager.selectedGroup().versions.length
                                    + " version(s)" : "0 versions"
                            color: "#7d8590"
                        }
                    }
                }
            }
        }
    }

    Dialog {
        id: gameSaveTextDialog
        anchors.centerIn: parent
        modal: true
        standardButtons: Dialog.Save | Dialog.Cancel
        property string operation: ""
        title: operation === "group" ? "Rename Save Group"
               : operation === "split" ? "Make New Save"
               : "Rename Save Version"

        function prepare(requestedOperation, initialText) {
            operation = requestedOperation
            gameSaveTextField.text = initialText
            open()
            gameSaveTextField.forceActiveFocus()
            gameSaveTextField.selectAll()
        }

        function smoke(requestedOperation, value) {
            prepare(requestedOperation, value)
            Qt.callLater(function() { gameSaveTextDialog.accept() })
        }

        onAccepted: {
            const value = gameSaveTextField.text
            const group = gameSaveManager.selectedGroup()
            const version = gameSaveManager.selectedVersion()
            if (operation === "group" && group !== null)
                controller.rename_game_save_group(
                    gameSaveManager.modelRow, gameSaveManager.gameId,
                    group.key, value)
            else if (operation === "split" && version !== null)
                controller.split_game_save_version(
                    gameSaveManager.modelRow, gameSaveManager.gameId,
                    version.source_index, value)
            else if (version !== null)
                controller.rename_game_save_version(
                    gameSaveManager.modelRow, gameSaveManager.gameId,
                    version.source_index, value)
        }

        contentItem: ColumnLayout {
            Label {
                text: gameSaveTextDialog.operation === "split"
                      ? "Move this version into a new save group. The save file stays in place."
                      : "This changes LaunchBox metadata only. The save file stays in place."
                wrapMode: Text.Wrap
                Layout.preferredWidth: 460
            }
            TextField {
                id: gameSaveTextField
                Layout.fillWidth: true
                placeholderText: "Name"
            }
        }
    }

    Dialog {
        id: gameSaveCombineDialog
        anchors.centerIn: parent
        modal: true
        title: "Combine Save Groups"
        standardButtons: Dialog.Ok | Dialog.Cancel

        function prepare() {
            gameSaveCombineTarget.currentIndex =
                gameSaveManager.selectedGroupIndex === 0 ? 1 : 0
            open()
        }

        function smoke() {
            prepare()
            Qt.callLater(function() { gameSaveCombineDialog.accept() })
        }

        onAccepted: {
            const source = gameSaveManager.selectedGroup()
            const target = gameSaveManager.groups[
                               gameSaveCombineTarget.currentIndex]
            if (source !== null && target !== undefined)
                controller.combine_game_save_groups(
                    gameSaveManager.modelRow, gameSaveManager.gameId,
                    source.key, target.key)
        }

        contentItem: ColumnLayout {
            Label {
                Layout.preferredWidth: 460
                text: "Move every version in “"
                      + (gameSaveManager.selectedGroup() !== null
                         ? gameSaveManager.selectedGroup().name : "")
                      + "” into this existing group. Only LaunchBox grouping metadata changes."
                wrapMode: Text.Wrap
            }
            ComboBox {
                id: gameSaveCombineTarget
                Layout.fillWidth: true
                model: gameSaveManager.groups
                textRole: "name"
            }
        }
    }

    Dialog {
        id: gameSaveDeleteDialog
        anchors.centerIn: parent
        modal: true
        title: "Delete Save Backup"
        standardButtons: Dialog.Yes | Dialog.No

        function prepare() {
            const version = gameSaveManager.selectedVersion()
            if (version !== null && version.location_kind === "vault")
                open()
        }

        function smoke() {
            prepare()
            Qt.callLater(function() { gameSaveDeleteDialog.accept() })
        }

        onAccepted: {
            const version = gameSaveManager.selectedVersion()
            if (version !== null && version.location_kind === "vault")
                controller.delete_game_save_backup(
                    gameSaveManager.modelRow, gameSaveManager.gameId,
                    version.source_index)
        }

        contentItem: Label {
            Layout.preferredWidth: 460
            text: {
                const version = gameSaveManager.selectedVersion()
                return "Delete the vault file and its LaunchBox save-history row for “"
                       + (version !== null ? version.title : "")
                       + "”? The active emulator save is not touched. The transaction retains exact recovery copies."
            }
            wrapMode: Text.Wrap
        }
    }

    Dialog {
        id: gameSaveRestoreDialog
        anchors.centerIn: parent
        modal: true
        title: "Restore Save Backup"
        standardButtons: Dialog.Yes | Dialog.No

        function prepare() {
            const version = gameSaveManager.selectedVersion()
            if (version !== null && version.location_kind === "vault")
                open()
        }

        function smoke() {
            prepare()
            Qt.callLater(function() { gameSaveRestoreDialog.accept() })
        }

        onAccepted: {
            const version = gameSaveManager.selectedVersion()
            if (version !== null && version.location_kind === "vault")
                controller.restore_game_save_backup(
                    gameSaveManager.modelRow, gameSaveManager.gameId,
                    version.source_index)
        }

        contentItem: Label {
            width: 460
            text: {
                const version = gameSaveManager.selectedVersion()
                return "Restore “"
                       + (version !== null ? version.title : "")
                       + "” over the active save? The current active file, RetroArch Saturn companion set, Dolphin Wii title directory, or PCSX2 memory-card member is first committed as a new vault version. Dolphin Wii restoration verifies a nested archive and swaps the complete directory; PCSX2 restoration validates and swaps a complete card working copy."
            }
            wrapMode: Text.Wrap
        }
    }

    Dialog {
        id: gameSaveActiveDeleteDialog
        anchors.centerIn: parent
        modal: true
        title: "Delete Active Save"
        standardButtons: Dialog.Yes | Dialog.No

        function prepare() {
            const version = gameSaveManager.selectedVersion()
            if (version !== null && version.location_kind === "active")
                open()
        }

        function smoke() {
            prepare()
            Qt.callLater(function() {
                gameSaveActiveDeleteDialog.accept()
            })
        }

        onAccepted: {
            const version = gameSaveManager.selectedVersion()
            if (version !== null && version.location_kind === "active")
                controller.delete_game_save_active(
                    gameSaveManager.modelRow, gameSaveManager.gameId,
                    version.source_index)
        }

        contentItem: Label {
            width: 460
            text: {
                const version = gameSaveManager.selectedVersion()
                return "Archive “"
                       + (version !== null ? version.title : "")
                       + "” in the portable vault, then delete its active emulator file, complete RetroArch Saturn companion set, Dolphin Wii title directory, or PCSX2 memory-card member? Exact sibling recovery copies are retained; Dolphin Wii deletion retains the complete directory tree and PCSX2 deletion swaps a validated complete card working copy."
            }
            wrapMode: Text.Wrap
        }
    }

    Dialog {
        id: additionalApplicationDefaultDialog
        anchors.centerIn: parent
        modal: true
        title: "Make Additional Application Default"
        standardButtons: Dialog.Yes | Dialog.No
        property int modelRow: -1
        property string gameId: ""
        property string applicationId: ""
        property string applicationName: ""

        function prepare(row, id, appId, appName) {
            modelRow = row
            gameId = id
            applicationId = appId
            applicationName = appName
            open()
        }

        function smokeMakeDefault(row, id, appId, appName) {
            prepare(row, id, appId, appName)
            Qt.callLater(function() {
                additionalApplicationDefaultDialog.accept()
            })
        }

        onAccepted: controller.make_additional_application_default(
                        modelRow, gameId, applicationId)

        contentItem: Label {
            width: 500
            text: "Make “"
                  + additionalApplicationDefaultDialog.applicationName
                  + "” this game’s default launch? Its launch, emulator, version "
                  + "metadata, and play statistics will replace the game’s current "
                  + "defaults. The additional-application record and target files "
                  + "will remain in place."
            wrapMode: Text.Wrap
        }
    }

    Dialog {
        id: additionalApplicationDeleteDialog
        anchors.centerIn: parent
        modal: true
        title: "Delete Additional Application"
        standardButtons: Dialog.Yes | Dialog.No
        property int modelRow: -1
        property string gameId: ""
        property string applicationId: ""
        property string applicationName: ""

        function prepare(row, id, appId, appName) {
            modelRow = row
            gameId = id
            applicationId = appId
            applicationName = appName
            open()
        }

        function smokeDelete(row, id, appId, appName) {
            prepare(row, id, appId, appName)
            Qt.callLater(function() {
                additionalApplicationDeleteDialog.accept()
            })
        }

        onAccepted: controller.delete_additional_application(
                        modelRow, gameId, applicationId)

        contentItem: Label {
            width: 460
            text: "Delete “" + additionalApplicationDeleteDialog.applicationName
                  + "” from this game? Its target file and media will not be deleted. "
                  + "Deletion is refused while a game-save record references it."
            wrapMode: Text.Wrap
        }
    }

    Dialog {
        id: additionalApplicationEditor
        anchors.centerIn: parent
        modal: true
        title: creating ? "Add Additional Application"
                        : "Edit Additional Application"
        standardButtons: Dialog.Save | Dialog.Cancel
        property int modelRow: -1
        property string gameId: ""
        property string applicationId: ""
        property bool creating: false
        property bool useEmulator: false
        property string selectedEmulatorId: ""

        function optionalText(value) {
            return value.trim().length > 0 ? value : null
        }

        function selectEmulator(useEmulatorValue, emulatorId) {
            useEmulator = useEmulatorValue
            selectedEmulatorId = emulatorId || ""
            additionalApplicationEmulatorChoice.currentIndex = -1
            if (!useEmulatorValue) {
                additionalApplicationEmulatorChoice.currentIndex = 1
                return
            }
            if (selectedEmulatorId.length === 0) {
                additionalApplicationEmulatorChoice.currentIndex = 0
                return
            }
            for (let index = 2; index < controller.emulator_entry_count(); ++index) {
                if (controller.emulator_id_at(index) === selectedEmulatorId) {
                    additionalApplicationEmulatorChoice.currentIndex = index
                    return
                }
            }
        }

        function loadPayload(payloadText) {
            if (payloadText.length === 0)
                return false
            const payload = JSON.parse(payloadText)
            const app = payload.application
            applicationNameField.text = app.name
            additionalApplicationPathField.text = app.application_path
            additionalApplicationCommandLineField.text = app.command_line || ""
            runBeforeCheck.checked = app.auto_run_before
            runAfterCheck.checked = app.auto_run_after
            waitForExitCheck.checked = app.wait_for_exit
            selectEmulator(app.use_emulator, app.emulator_id)
            additionalApplicationUseDosBoxCheck.checked = app.use_dos_box
            priorityField.value = app.priority
            playCountField.value = app.play_count
            playTimeField.value = app.play_time_seconds
            hasDiscCheck.checked = app.disc !== null
            discField.value = app.disc !== null ? app.disc : 0
            sideACheck.checked = app.side_a
            sideBCheck.checked = app.side_b
            additionalApplicationDeveloperField.text = app.developer || ""
            additionalApplicationPublisherField.text = app.publisher || ""
            additionalApplicationRegionField.text = app.region || ""
            additionalApplicationReleaseDateField.text = app.release_date || ""
            additionalApplicationVersionField.text = app.version || ""
            additionalApplicationStatusField.text = app.status || ""
            installedChoice.currentIndex = app.installed === null
                                           ? 0 : (app.installed ? 1 : 2)
            lastPlayedField.text = app.last_played || ""
            return true
        }

        function prepareCreate(row, id) {
            modelRow = row
            gameId = id
            applicationId = ""
            creating = true
            if (loadPayload(controller.new_additional_application_edit_payload(row, id)))
                open()
        }

        function prepareEdit(row, id, appId) {
            modelRow = row
            gameId = id
            applicationId = appId
            creating = false
            if (loadPayload(controller.additional_application_edit_payload(
                                row, id, appId)))
                open()
        }

        function smokeEdit(row, id, appId) {
            prepareEdit(row, id, appId)
            applicationNameField.text = "Edited Fixture Manual"
            additionalApplicationPathField.text =
                "Games\\Fixture Adventure\\edited-manual.pdf"
            additionalApplicationCommandLineField.text = "--page 3"
            runBeforeCheck.checked = true
            runAfterCheck.checked = false
            waitForExitCheck.checked = true
            selectEmulator(false, "")
            additionalApplicationUseDosBoxCheck.checked = false
            priorityField.value = 4
            playCountField.value = 5
            playTimeField.value = 321
            hasDiscCheck.checked = true
            discField.value = 2
            sideACheck.checked = true
            sideBCheck.checked = false
            additionalApplicationDeveloperField.text = "Qt Docs"
            additionalApplicationPublisherField.text = "Port Press"
            additionalApplicationRegionField.text = "Europe"
            additionalApplicationReleaseDateField.text = "2005-06-07"
            additionalApplicationVersionField.text = "Rev 3"
            additionalApplicationStatusField.text = "Installed"
            installedChoice.currentIndex = 1
            lastPlayedField.text = "2026-07-22T13:14:15.0000000-07:00"
            Qt.callLater(function() { additionalApplicationEditor.accept() })
        }

        function smokeCreate(row, id) {
            prepareCreate(row, id)
            applicationNameField.text = "Temporary Fixture Application"
            additionalApplicationPathField.text =
                "Games\\Fixture Adventure\\temporary-tool.exe"
            additionalApplicationCommandLineField.text = "--temporary"
            priorityField.value = 9
            Qt.callLater(function() { additionalApplicationEditor.accept() })
        }

        function editPayload() {
            return JSON.stringify({
                version: 1,
                application: {
                    name: applicationNameField.text,
                    application_path: additionalApplicationPathField.text,
                    command_line: optionalText(
                                      additionalApplicationCommandLineField.text),
                    auto_run_before: runBeforeCheck.checked,
                    auto_run_after: runAfterCheck.checked,
                    wait_for_exit: waitForExitCheck.checked,
                    use_emulator: useEmulator,
                    emulator_id: useEmulator
                                 ? optionalText(selectedEmulatorId) : null,
                    use_dos_box: additionalApplicationUseDosBoxCheck.checked,
                    priority: priorityField.value,
                    play_count: playCountField.value,
                    play_time_seconds: playTimeField.value,
                    disc: hasDiscCheck.checked ? discField.value : null,
                    side_a: sideACheck.checked,
                    side_b: sideBCheck.checked,
                    developer: optionalText(
                                   additionalApplicationDeveloperField.text),
                    publisher: optionalText(
                                   additionalApplicationPublisherField.text),
                    region: optionalText(additionalApplicationRegionField.text),
                    release_date: optionalText(
                                      additionalApplicationReleaseDateField.text),
                    version: optionalText(additionalApplicationVersionField.text),
                    status: optionalText(additionalApplicationStatusField.text),
                    installed: installedChoice.currentIndex === 0
                               ? null : installedChoice.currentIndex === 1,
                    last_played: optionalText(lastPlayedField.text)
                }
            })
        }

        onAccepted: {
            if (creating)
                controller.add_additional_application(
                    modelRow, gameId, editPayload())
            else
                controller.save_additional_application(
                    modelRow, gameId, applicationId, editPayload())
        }

        contentItem: ScrollView {
            id: additionalApplicationEditorScroll
            implicitWidth: 720
            implicitHeight: Math.min(700, window.height - 120)
            contentWidth: availableWidth
            clip: true

            ColumnLayout {
                width: additionalApplicationEditorScroll.availableWidth
                spacing: 12
                Label {
                    Layout.fillWidth: true
                    text: "Stored paths remain lexical LaunchBox data. Windows drive, UNC, and separator syntax is interpreted only by the cross-platform launch service."
                    wrapMode: Text.Wrap
                    color: "#7fbfff"
                }
                GridLayout {
                    Layout.fillWidth: true
                    columns: 2
                    columnSpacing: 12
                    rowSpacing: 8

                    Label { text: "Application name" }
                    TextField {
                        id: applicationNameField
                        Layout.fillWidth: true
                        placeholderText: "Required display name"
                    }
                    Label { text: "Application path" }
                    TextField {
                        id: additionalApplicationPathField
                        Layout.fillWidth: true
                        placeholderText: "LaunchBox path or URL"
                    }
                    Label { text: "Command line" }
                    TextField {
                        id: additionalApplicationCommandLineField
                        Layout.fillWidth: true
                        placeholderText: "Optional arguments"
                    }
                    Label { text: "Emulator choice" }
                    ComboBox {
                        id: additionalApplicationEmulatorChoice
                        Layout.fillWidth: true
                        model: {
                            const revision = controller.emulator_revision
                            return controller.emulator_entry_count()
                        }
                        displayText: currentIndex >= 0
                                     ? controller.emulator_title_at(currentIndex)
                                     : "Unavailable stored emulator"
                        delegate: ItemDelegate {
                            required property int index
                            width: ListView.view ? ListView.view.width : implicitWidth
                            text: controller.emulator_title_at(index)
                        }
                        onActivated: function(index) {
                            if (index === 1) {
                                additionalApplicationEditor.useEmulator = false
                                additionalApplicationEditor.selectedEmulatorId = ""
                            } else {
                                additionalApplicationEditor.useEmulator = true
                                additionalApplicationEditor.selectedEmulatorId =
                                    controller.emulator_id_at(index)
                            }
                        }
                    }
                    Label { text: "Priority" }
                    SpinBox {
                        id: priorityField
                        from: 0
                        to: 2147483647
                        editable: true
                    }
                    Label { text: "Disc" }
                    RowLayout {
                        CheckBox {
                            id: hasDiscCheck
                            text: "Set"
                        }
                        SpinBox {
                            id: discField
                            from: 0
                            to: 999
                            editable: true
                            enabled: hasDiscCheck.checked
                        }
                    }
                    Label { text: "Developer" }
                    TextField {
                        id: additionalApplicationDeveloperField
                        Layout.fillWidth: true
                    }
                    Label { text: "Publisher" }
                    TextField {
                        id: additionalApplicationPublisherField
                        Layout.fillWidth: true
                    }
                    Label { text: "Region" }
                    TextField {
                        id: additionalApplicationRegionField
                        Layout.fillWidth: true
                    }
                    Label { text: "Release date" }
                    TextField {
                        id: additionalApplicationReleaseDateField
                        Layout.fillWidth: true
                        placeholderText: "Stored LaunchBox date value"
                    }
                    Label { text: "Version" }
                    TextField {
                        id: additionalApplicationVersionField
                        Layout.fillWidth: true
                    }
                    Label { text: "Status" }
                    TextField {
                        id: additionalApplicationStatusField
                        Layout.fillWidth: true
                    }
                    Label { text: "Installed" }
                    ComboBox {
                        id: installedChoice
                        Layout.fillWidth: true
                        model: ["Unknown", "Installed", "Not installed"]
                    }
                    Label { text: "Play count" }
                    SpinBox {
                        id: playCountField
                        from: 0
                        to: 2147483647
                        editable: true
                    }
                    Label { text: "Play time (seconds)" }
                    SpinBox {
                        id: playTimeField
                        from: 0
                        to: 2147483647
                        editable: true
                    }
                    Label { text: "Last played" }
                    TextField {
                        id: lastPlayedField
                        Layout.fillWidth: true
                        placeholderText: "Stored LaunchBox timestamp"
                    }
                }
                Flow {
                    Layout.fillWidth: true
                    spacing: 10
                    CheckBox {
                        id: runBeforeCheck
                        text: "Run before main application"
                    }
                    CheckBox {
                        id: runAfterCheck
                        text: "Run after main application"
                    }
                    CheckBox {
                        id: waitForExitCheck
                        text: "Wait for exit"
                        enabled: runBeforeCheck.checked
                    }
                    CheckBox {
                        id: additionalApplicationUseDosBoxCheck
                        text: "Use DOSBox"
                    }
                    CheckBox {
                        id: sideACheck
                        text: "Side A"
                    }
                    CheckBox {
                        id: sideBCheck
                        text: "Side B"
                    }
                }
            }
        }
    }

    Dialog {
        id: gameEditor
        anchors.centerIn: parent
        modal: true
        title: "Edit " + gameTitle
        standardButtons: Dialog.Save | Dialog.Cancel
        property int modelRow: -1
        property string gameId: ""
        property string gameTitle: ""

        ListModel { id: alternateNameEditorModel }
        ListModel { id: customFieldEditorModel }

        function loadRepeatedMetadata(row, id) {
            alternateNameEditorModel.clear()
            const alternateNameCount = controller.alternate_name_count(row, id)
            for (let index = 0; index < alternateNameCount; ++index) {
                alternateNameEditorModel.append({
                    sourceIndex: index,
                    entryName: controller.alternate_name_name_at(row, id, index),
                    entryRegion: controller.alternate_name_region_at(row, id, index)
                })
            }
            customFieldEditorModel.clear()
            const customFieldCount = controller.custom_field_count(row, id)
            for (let index = 0; index < customFieldCount; ++index) {
                customFieldEditorModel.append({
                    sourceIndex: index,
                    fieldName: controller.custom_field_name_at(row, id, index),
                    fieldValue: controller.custom_field_value_at(row, id, index)
                })
            }
        }

        function selectEmulatorId(emulatorId) {
            emulatorIdField.text = emulatorId
            emulatorChoice.currentIndex = -1
            for (let index = 0; index < controller.emulator_entry_count(); ++index) {
                if (controller.emulator_id_at(index) === emulatorId) {
                    emulatorChoice.currentIndex = index
                    break
                }
            }
        }

        function edit(row, id, title, sortTitle, notes, developer, genre,
                      maxPlayers, playMode, progress, publisher, rating, region,
                      releaseDate, releaseType, series, source, status, version,
                      wikipediaUrl, favorite, completed, starRatingValue,
                      applicationPath, commandLine, emulatorId, useDosBox,
                      customDosBoxVersionPath, dosBoxConfigurationPath,
                      useScummVm, scummVmAspectCorrection, scummVmFullscreen,
                      scummVmGameDataFolderPath, scummVmGameType) {
            modelRow = row
            gameId = id
            gameTitle = title
            loadRepeatedMetadata(row, id)
            titleField.text = title
            sortTitleField.text = sortTitle
            notesField.text = notes
            developerField.text = developer
            genreField.text = genre
            maxPlayersField.value = maxPlayers
            playModeField.text = playMode
            progressField.text = progress
            publisherField.text = publisher
            ratingField.text = rating
            regionField.text = region
            releaseDateField.text = releaseDate
            releaseTypeField.text = releaseType
            seriesField.text = series
            sourceField.text = source
            statusField.text = status
            versionField.text = version
            wikipediaUrlField.text = wikipediaUrl
            favoriteCheck.checked = favorite
            completedCheck.checked = completed
            starRating.value = starRatingValue
            applicationPathField.text = applicationPath
            commandLineField.text = commandLine
            selectEmulatorId(emulatorId)
            useDosBoxCheck.checked = useDosBox
            customDosBoxVersionPathField.text = customDosBoxVersionPath
            dosBoxConfigurationPathField.text = dosBoxConfigurationPath
            useScummVmCheck.checked = useScummVm
            scummVmAspectCorrectionCheck.checked = scummVmAspectCorrection
            scummVmFullscreenCheck.checked = scummVmFullscreen
            scummVmGameDataFolderPathField.text = scummVmGameDataFolderPath
            scummVmGameTypeField.text = scummVmGameType
            open()
        }

        function smokeSaveState(row, id, title, sortTitle, notes, developer, genre,
                                maxPlayers, playMode, progress, publisher, rating,
                                region, releaseDate, releaseType, series, source,
                                status, version, wikipediaUrl, applicationPath,
                                commandLine, emulatorId, useDosBox,
                                customDosBoxVersionPath, dosBoxConfigurationPath,
                                useScummVm, scummVmAspectCorrection,
                                scummVmFullscreen, scummVmGameDataFolderPath,
                                scummVmGameType) {
            edit(row, id, title, sortTitle, notes, developer, genre, maxPlayers,
                 playMode, progress, publisher, rating, region, releaseDate,
                 releaseType, series, source, status, version, wikipediaUrl,
                 false, true, 2, applicationPath, commandLine, emulatorId,
                 useDosBox, customDosBoxVersionPath, dosBoxConfigurationPath,
                 useScummVm, scummVmAspectCorrection, scummVmFullscreen,
                 scummVmGameDataFolderPath, scummVmGameType)
            Qt.callLater(function() { gameEditor.accept() })
        }

        function smokeSaveMetadata(row) {
            modelRow = row
            gameId = controller.game_id_at(row)
            loadRepeatedMetadata(row, gameId)
            gameTitle = "Renamed Adventure"
            titleField.text = gameTitle
            sortTitleField.text = "Adventure, Renamed"
            notesField.text = "Edited notes from Qt."
            developerField.text = "Qt Forge"
            genreField.text = "Action Adventure"
            maxPlayersField.value = 6
            playModeField.text = "Local Cooperative"
            progressField.text = "75%"
            publisherField.text = "Port Press"
            ratingField.text = "T"
            regionField.text = "Europe"
            releaseDateField.text = "2001-02-03"
            releaseTypeField.text = "Homebrew"
            seriesField.text = ""
            sourceField.text = "Physical Media"
            statusField.text = "Imported"
            versionField.text = "2.0"
            wikipediaUrlField.text = ""
            applicationPathField.text = "Runtime\\edited-recorder"
            commandLineField.text = "--edited \"%gameid%\" \"two words\""
            selectEmulatorId(controller.emulator_id_at(1))
            useDosBoxCheck.checked = false
            customDosBoxVersionPathField.text = ""
            dosBoxConfigurationPathField.text = ""
            useScummVmCheck.checked = false
            scummVmAspectCorrectionCheck.checked = false
            scummVmFullscreenCheck.checked = false
            scummVmGameDataFolderPathField.text = ""
            scummVmGameTypeField.text = ""
            alternateNameEditorModel.setProperty(0, "entryName",
                                                 "Adventure, Renamed Alias")
            alternateNameEditorModel.setProperty(0, "entryRegion", "Europe")
            alternateNameEditorModel.append({
                sourceIndex: -1,
                entryName: "Aventure Qt",
                entryRegion: "France"
            })
            customFieldEditorModel.setProperty(0, "fieldName", "Cabinet Style")
            customFieldEditorModel.setProperty(0, "fieldValue", "Cocktail")
            customFieldEditorModel.append({
                sourceIndex: -1,
                fieldName: "Port Status",
                fieldValue: "Native Qt"
            })
            open()
            Qt.callLater(function() { gameEditor.accept() })
        }

        function optionalText(value) {
            return value.trim().length > 0 ? value : null
        }

        function editPayload() {
            const alternateNames = []
            for (let index = 0; index < alternateNameEditorModel.count; ++index) {
                const entry = alternateNameEditorModel.get(index)
                if (entry.entryName.trim().length > 0
                        || entry.entryRegion.trim().length > 0) {
                    alternateNames.push({
                        source_index: entry.sourceIndex >= 0 ? entry.sourceIndex : null,
                        name: entry.entryName,
                        region: optionalText(entry.entryRegion)
                    })
                }
            }
            const customFields = []
            for (let index = 0; index < customFieldEditorModel.count; ++index) {
                const field = customFieldEditorModel.get(index)
                if (field.fieldName.trim().length > 0
                        || field.fieldValue.trim().length > 0) {
                    customFields.push({
                        source_index: field.sourceIndex >= 0 ? field.sourceIndex : null,
                        name: field.fieldName,
                        value: field.fieldValue
                    })
                }
            }
            return JSON.stringify({
                version: 3,
                metadata: {
                    title: titleField.text,
                    sort_title: optionalText(sortTitleField.text),
                    notes: optionalText(notesField.text),
                    developer: optionalText(developerField.text),
                    genre: optionalText(genreField.text),
                    max_players: maxPlayersField.value > 0 ? maxPlayersField.value : null,
                    play_mode: optionalText(playModeField.text),
                    progress: optionalText(progressField.text),
                    publisher: optionalText(publisherField.text),
                    rating: optionalText(ratingField.text),
                    region: optionalText(regionField.text),
                    release_date: optionalText(releaseDateField.text),
                    release_type: optionalText(releaseTypeField.text),
                    series: optionalText(seriesField.text),
                    source: optionalText(sourceField.text),
                    status: optionalText(statusField.text),
                    version: optionalText(versionField.text),
                    wikipedia_url: optionalText(wikipediaUrlField.text)
                },
                launch_configuration: {
                    application_path: applicationPathField.text,
                    command_line: optionalText(commandLineField.text),
                    emulator_id: optionalText(emulatorIdField.text),
                    use_dos_box: useDosBoxCheck.checked,
                    custom_dos_box_version_path:
                        optionalText(customDosBoxVersionPathField.text),
                    dos_box_configuration_path:
                        optionalText(dosBoxConfigurationPathField.text),
                    use_scumm_vm: useScummVmCheck.checked,
                    scumm_vm_aspect_correction: scummVmAspectCorrectionCheck.checked,
                    scumm_vm_fullscreen: scummVmFullscreenCheck.checked,
                    scumm_vm_game_data_folder_path:
                        optionalText(scummVmGameDataFolderPathField.text),
                    scumm_vm_game_type: optionalText(scummVmGameTypeField.text)
                },
                alternate_names: alternateNames,
                custom_fields: customFields,
                favorite: favoriteCheck.checked,
                completed: completedCheck.checked,
                star_rating: starRating.value
            })
        }

        onAccepted: controller.save_game(modelRow, gameId, editPayload())

        contentItem: ScrollView {
            id: gameEditorScroll
            implicitWidth: 680
            implicitHeight: Math.min(680, window.height - 160)
            contentWidth: availableWidth
            clip: true

            ColumnLayout {
                width: gameEditorScroll.availableWidth
                spacing: 12

                GridLayout {
                    Layout.fillWidth: true
                    columns: 2
                    columnSpacing: 12
                    rowSpacing: 8

                    Label { text: "Title" }
                    TextField {
                        id: titleField
                        Layout.fillWidth: true
                        placeholderText: "Game title"
                    }
                    Label { text: "Sort title" }
                    TextField {
                        id: sortTitleField
                        Layout.fillWidth: true
                        placeholderText: "Optional alternate sorting title"
                    }
                    Label { text: "Developer" }
                    TextField { id: developerField; Layout.fillWidth: true }
                    Label { text: "Publisher" }
                    TextField { id: publisherField; Layout.fillWidth: true }
                    Label { text: "Genre" }
                    TextField { id: genreField; Layout.fillWidth: true }
                    Label { text: "Series" }
                    TextField { id: seriesField; Layout.fillWidth: true }
                    Label { text: "Region" }
                    TextField { id: regionField; Layout.fillWidth: true }
                    Label { text: "Release date" }
                    TextField {
                        id: releaseDateField
                        Layout.fillWidth: true
                        placeholderText: "YYYY-MM-DD or stored LaunchBox value"
                    }
                    Label { text: "Release type" }
                    TextField { id: releaseTypeField; Layout.fillWidth: true }
                    Label { text: "Version" }
                    TextField { id: versionField; Layout.fillWidth: true }
                    Label { text: "Source" }
                    TextField { id: sourceField; Layout.fillWidth: true }
                    Label { text: "Status" }
                    TextField { id: statusField; Layout.fillWidth: true }
                    Label { text: "Content rating" }
                    TextField { id: ratingField; Layout.fillWidth: true }
                    Label { text: "Play mode" }
                    TextField { id: playModeField; Layout.fillWidth: true }
                    Label { text: "Progress" }
                    TextField { id: progressField; Layout.fillWidth: true }
                    Label { text: "Maximum players" }
                    SpinBox {
                        id: maxPlayersField
                        from: 0
                        to: 999
                        editable: true
                        Accessible.description: "Zero means unknown"
                    }
                    Label { text: "Wikipedia URL" }
                    TextField {
                        id: wikipediaUrlField
                        Layout.fillWidth: true
                        placeholderText: "Optional URL"
                    }
                }

                Label { text: "Notes" }
                TextArea {
                    id: notesField
                    Layout.fillWidth: true
                    Layout.preferredHeight: 110
                    wrapMode: TextEdit.Wrap
                    placeholderText: "Game description or notes"
                }

                RowLayout {
                    Layout.fillWidth: true
                    CheckBox {
                        id: favoriteCheck
                        text: "Favorite"
                    }
                    CheckBox {
                        id: completedCheck
                        text: "Completed"
                    }
                    Item { Layout.fillWidth: true }
                    Label { text: "Star rating" }
                    SpinBox {
                        id: starRating
                        from: 0
                        to: 5
                        editable: true
                    }
                }
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 1
                    color: "#30363d"
                }
                Label {
                    text: "Alternate names"
                    font.pixelSize: 18
                    font.bold: true
                }
                Label {
                    Layout.fillWidth: true
                    text: "Aliases remain in LaunchBox source order. Region is optional."
                    wrapMode: Text.Wrap
                    color: "#7d8590"
                }
                Repeater {
                    model: alternateNameEditorModel
                    delegate: RowLayout {
                        required property int index
                        required property int sourceIndex
                        required property string entryName
                        required property string entryRegion
                        Layout.fillWidth: true
                        spacing: 8

                        TextField {
                            Layout.fillWidth: true
                            text: entryName
                            placeholderText: "Alternate name"
                            onTextEdited: alternateNameEditorModel.setProperty(
                                              index, "entryName", text)
                        }
                        TextField {
                            Layout.fillWidth: true
                            text: entryRegion
                            placeholderText: "Optional region"
                            onTextEdited: alternateNameEditorModel.setProperty(
                                              index, "entryRegion", text)
                        }
                        Button {
                            text: "Remove"
                            onClicked: alternateNameEditorModel.remove(index)
                        }
                    }
                }
                Button {
                    text: "Add Alternate Name"
                    onClicked: alternateNameEditorModel.append({
                        sourceIndex: -1,
                        entryName: "",
                        entryRegion: ""
                    })
                }
                Label {
                    text: "Custom fields"
                    font.pixelSize: 18
                    font.bold: true
                }
                Label {
                    Layout.fillWidth: true
                    text: "Custom field values may be empty; every saved field needs a name."
                    wrapMode: Text.Wrap
                    color: "#7d8590"
                }
                Repeater {
                    model: customFieldEditorModel
                    delegate: RowLayout {
                        required property int index
                        required property int sourceIndex
                        required property string fieldName
                        required property string fieldValue
                        Layout.fillWidth: true
                        spacing: 8

                        TextField {
                            Layout.fillWidth: true
                            text: fieldName
                            placeholderText: "Field name"
                            onTextEdited: customFieldEditorModel.setProperty(
                                              index, "fieldName", text)
                        }
                        TextField {
                            Layout.fillWidth: true
                            text: fieldValue
                            placeholderText: "Value"
                            onTextEdited: customFieldEditorModel.setProperty(
                                              index, "fieldValue", text)
                        }
                        Button {
                            text: "Remove"
                            onClicked: customFieldEditorModel.remove(index)
                        }
                    }
                }
                Button {
                    text: "Add Custom Field"
                    onClicked: customFieldEditorModel.append({
                        sourceIndex: -1,
                        fieldName: "",
                        fieldValue: ""
                    })
                }
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 1
                    color: "#30363d"
                }
                Label {
                    text: "Launch configuration"
                    font.pixelSize: 18
                    font.bold: true
                }
                Label {
                    Layout.fillWidth: true
                    text: "Paths are stored exactly as LaunchBox data. Windows separators and roots are translated by the cross-platform path service only when a game launches."
                    wrapMode: Text.Wrap
                    color: "#7fbfff"
                }
                GridLayout {
                    Layout.fillWidth: true
                    columns: 2
                    columnSpacing: 12
                    rowSpacing: 8

                    Label { text: "Application path" }
                    TextField {
                        id: applicationPathField
                        Layout.fillWidth: true
                        placeholderText: "LaunchBox executable or game path"
                    }
                    Label { text: "Command line" }
                    TextField {
                        id: commandLineField
                        Layout.fillWidth: true
                        placeholderText: "Optional LaunchBox arguments"
                    }
                    Label { text: "Emulator choice" }
                    ComboBox {
                        id: emulatorChoice
                        Layout.fillWidth: true
                        model: {
                            const revision = controller.emulator_revision
                            return controller.emulator_entry_count()
                        }
                        displayText: currentIndex >= 0
                                     ? controller.emulator_title_at(currentIndex)
                                     : "Custom or unavailable emulator ID"
                        delegate: ItemDelegate {
                            required property int index
                            width: ListView.view ? ListView.view.width : implicitWidth
                            text: controller.emulator_title_at(index)
                        }
                        onActivated: function(index) {
                            emulatorIdField.text = controller.emulator_id_at(index)
                        }
                    }
                    Label { text: "Stored emulator ID" }
                    TextField {
                        id: emulatorIdField
                        Layout.fillWidth: true
                        placeholderText: "Empty uses the platform default"
                        onTextEdited: emulatorChoice.currentIndex = -1
                    }
                    Label { text: "Legacy launch mode" }
                    RowLayout {
                        Layout.fillWidth: true
                        CheckBox {
                            id: useDosBoxCheck
                            text: "Use DOSBox"
                        }
                        CheckBox {
                            id: useScummVmCheck
                            text: "Use ScummVM"
                        }
                    }
                    Label { text: "DOSBox executable" }
                    TextField {
                        id: customDosBoxVersionPathField
                        Layout.fillWidth: true
                        placeholderText: "Optional custom DOSBox path"
                    }
                    Label { text: "DOSBox configuration" }
                    TextField {
                        id: dosBoxConfigurationPathField
                        Layout.fillWidth: true
                        placeholderText: "Optional .conf path"
                    }
                    Label { text: "ScummVM game-data folder" }
                    TextField {
                        id: scummVmGameDataFolderPathField
                        Layout.fillWidth: true
                    }
                    Label { text: "ScummVM target" }
                    TextField {
                        id: scummVmGameTypeField
                        Layout.fillWidth: true
                        placeholderText: "ScummVM game ID"
                    }
                    Label { text: "ScummVM display" }
                    RowLayout {
                        Layout.fillWidth: true
                        CheckBox {
                            id: scummVmFullscreenCheck
                            text: "Fullscreen"
                        }
                        CheckBox {
                            id: scummVmAspectCorrectionCheck
                            text: "Aspect correction"
                        }
                    }
                }
                Label {
                    Layout.fillWidth: true
                    text: "Empty optional fields are removed. DOSBox and ScummVM cannot both be selected. Save creates an exact sibling backup and refuses external file changes."
                    wrapMode: Text.Wrap
                    color: "#7d8590"
                }
                RowLayout {
                    Layout.fillWidth: true
                    Button {
                        text: "Combine Games…"
                        enabled: !controller.writing
                                 && controller.pending_recovery_count === 0
                                 && !controller.write_conflict
                        onClicked: {
                            gameEditor.close()
                            gameCombineDialog.prepare(
                                gameEditor.modelRow, gameEditor.gameId,
                                gameEditor.gameTitle)
                        }
                    }
                    Button {
                        text: "Expand Versions…"
                        enabled: !controller.writing
                                 && controller.pending_recovery_count === 0
                                 && !controller.write_conflict
                                 && controller.additional_application_count(
                                     gameEditor.modelRow, gameEditor.gameId) > 0
                        onClicked: {
                            gameEditor.close()
                            gameExpandConfirmation.prepare(
                                gameEditor.modelRow, gameEditor.gameId,
                                gameEditor.gameTitle)
                        }
                    }
                    Item { Layout.fillWidth: true }
                    Button {
                        text: "Delete Game…"
                        enabled: !controller.writing
                        onClicked: {
                            gameEditor.close()
                            deleteConfirmation.prepare(
                                gameEditor.modelRow, gameEditor.gameId,
                                gameEditor.gameTitle)
                        }
                    }
                }
            }
        }
    }

    Dialog {
        id: gameCombineDialog
        anchors.centerIn: parent
        modal: true
        title: "Combine Games into " + rootGameTitle
        standardButtons: Dialog.Ok | Dialog.Cancel
        property int modelRow: -1
        property string rootGameId: ""
        property string rootGameTitle: ""

        ListModel { id: gameCombineCandidateModel }

        function selectedGameIds() {
            const ids = []
            for (let index = 0; index < gameCombineCandidateModel.count; ++index) {
                const candidate = gameCombineCandidateModel.get(index)
                if (candidate.chosen)
                    ids.push(candidate.candidateId)
            }
            return ids
        }

        function updateAcceptance() {
            const button = standardButton(Dialog.Ok)
            if (button)
                button.enabled = selectedGameIds().length > 0
        }

        function prepare(row, id, title) {
            modelRow = row
            rootGameId = id
            rootGameTitle = title
            gameCombineCandidateModel.clear()
            const serialized = controller.game_combine_candidates(row, id)
            if (serialized.length > 0) {
                const candidates = JSON.parse(serialized)
                for (let index = 0; index < candidates.length; ++index) {
                    const candidate = candidates[index]
                    gameCombineCandidateModel.append({
                        candidateId: candidate.id,
                        candidateTitle: candidate.title,
                        candidatePlatform: candidate.platform,
                        candidatePath: candidate.application_path,
                        chosen: false
                    })
                }
            }
            open()
            Qt.callLater(updateAcceptance)
        }

        function smokeCombine(row, id, title, candidateId) {
            prepare(row, id, title)
            for (let index = 0; index < gameCombineCandidateModel.count; ++index) {
                if (gameCombineCandidateModel.get(index).candidateId === candidateId) {
                    gameCombineCandidateModel.setProperty(index, "chosen", true)
                    break
                }
            }
            updateAcceptance()
            Qt.callLater(function() { gameCombineDialog.accept() })
        }

        onAccepted: controller.combine_games(
                        modelRow, rootGameId,
                        JSON.stringify(selectedGameIds()))

        contentItem: ColumnLayout {
            spacing: 10
            Label {
                Layout.preferredWidth: 700
                text: "Select same-platform games to make launchable versions of the retained root. The root keeps its title, metadata, and media associations; all modeled references are migrated in one transaction."
                wrapMode: Text.Wrap
                color: "#aeb8c5"
            }
            Label {
                Layout.preferredWidth: 700
                visible: gameCombineCandidateModel.count === 0
                text: "No other games are available in this platform document."
                wrapMode: Text.Wrap
                color: "#f0b35a"
            }
            ListView {
                Layout.fillWidth: true
                Layout.preferredHeight: gameCombineCandidateModel.count > 0
                                        ? Math.min(360, contentHeight) : 0
                clip: true
                spacing: 4
                model: gameCombineCandidateModel
                delegate: CheckDelegate {
                    required property int index
                    required property string candidateId
                    required property string candidateTitle
                    required property string candidatePlatform
                    required property string candidatePath
                    required property bool chosen
                    width: ListView.view.width
                    checked: chosen
                    text: candidateTitle + " — " + candidatePlatform
                          + "\n" + candidatePath
                    onToggled: {
                        gameCombineCandidateModel.setProperty(
                            index, "chosen", checked)
                        gameCombineDialog.updateAcceptance()
                    }
                }
            }
            Label {
                Layout.preferredWidth: 700
                text: "Stored paths remain lexical LaunchBox values. Exact backups are created for every changed XML file; no ROM or media files are moved or deleted."
                wrapMode: Text.Wrap
                color: "#7fbfff"
            }
        }
    }

    Dialog {
        id: gameExpandConfirmation
        anchors.centerIn: parent
        modal: true
        title: "Expand Versions from " + gameTitle + "?"
        standardButtons: Dialog.Yes | Dialog.No
        property int modelRow: -1
        property string gameId: ""
        property string gameTitle: ""

        function prepare(row, id, title) {
            modelRow = row
            gameId = id
            gameTitle = title
            open()
        }

        function smokeExpand(row, id, title) {
            prepare(row, id, title)
            Qt.callLater(function() { gameExpandConfirmation.accept() })
        }

        onAccepted: controller.expand_game_versions(modelRow, gameId)

        contentItem: Label {
            width: 620
            text: "Each launchable version becomes a standalone game. The default-version representative is consumed without duplicating the retained game; documents and automatic helper applications stay attached. Exact XML backup is created, and no ROM or media files are moved or deleted."
            wrapMode: Text.Wrap
        }
    }

    Dialog {
        id: emulatorManager
        anchors.centerIn: parent
        modal: true
        title: "Manage Emulators"
        standardButtons: Dialog.Close
        property int selectedIndex: -1
        property int selectedDiscoveryIndex: -1

        function selectedId() {
            return selectedIndex >= 0
                   ? controller.emulator_id_at(selectedIndex + 2) : ""
        }

        function selectedTitle() {
            return selectedIndex >= 0
                   ? controller.emulator_title_at(selectedIndex + 2) : ""
        }

        function openManager() {
            selectedIndex = controller.emulator_entry_count() > 2 ? 0 : -1
            selectedDiscoveryIndex =
                controller.discovered_emulator_count() > 0 ? 0 : -1
            open()
        }

        Connections {
            target: controller
            function onEmulatorRevisionChanged() {
                const count = Math.max(
                                0, controller.emulator_entry_count() - 2)
                if (count === 0)
                    emulatorManager.selectedIndex = -1
                else if (emulatorManager.selectedIndex < 0)
                    emulatorManager.selectedIndex = 0
                else if (emulatorManager.selectedIndex >= count)
                    emulatorManager.selectedIndex = count - 1
            }
            function onEmulatorDiscoveryRevisionChanged() {
                const count = controller.discovered_emulator_count()
                if (count === 0)
                    emulatorManager.selectedDiscoveryIndex = -1
                else if (emulatorManager.selectedDiscoveryIndex < 0)
                    emulatorManager.selectedDiscoveryIndex = 0
                else if (emulatorManager.selectedDiscoveryIndex >= count)
                    emulatorManager.selectedDiscoveryIndex = count - 1
            }
        }

        contentItem: ColumnLayout {
            implicitWidth: 760
            implicitHeight: 620
            spacing: 10

            Label {
                Layout.fillWidth: true
                text: "Emulator definitions and per-platform mappings are stored in Data/Emulators.xml. Manual edits keep application paths as lexical LaunchBox values. The reviewed PCSX2 workflow below can separately install or update a verified official portable build."
                wrapMode: Text.Wrap
                color: "#7fbfff"
            }
            Label {
                text: "Configured"
                color: "white"
                font.bold: true
            }
            ListView {
                id: emulatorList
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.minimumHeight: 130
                clip: true
                model: {
                    const revision = controller.emulator_revision
                    return Math.max(0, controller.emulator_entry_count() - 2)
                }
                delegate: ItemDelegate {
                    id: emulatorDelegate
                    required property int index
                    width: emulatorList.width
                    property string emulatorId: controller.emulator_id_at(index + 2)
                    property string emulatorTitle: controller.emulator_title_at(index + 2)
                    highlighted: emulatorManager.selectedIndex === index
                    contentItem: ColumnLayout {
                        Label {
                            Layout.fillWidth: true
                            text: emulatorDelegate.emulatorTitle
                            color: "white"
                            font.bold: true
                            elide: Text.ElideRight
                        }
                        Label {
                            Layout.fillWidth: true
                            text: emulatorDelegate.emulatorId
                            color: "#7d8590"
                            font.pixelSize: 11
                            elide: Text.ElideMiddle
                        }
                    }
                    onClicked: emulatorManager.selectedIndex = index
                    onDoubleClicked: emulatorEditor.prepareEdit(emulatorId)
                }
            }
            RowLayout {
                Layout.fillWidth: true
                Button {
                    text: "Add"
                    enabled: !controller.writing && !controller.emulator_installing
                             && !controller.emulator_release_checking
                             && !controller.write_conflict
                             && controller.pending_recovery_count === 0
                    onClicked: emulatorEditor.prepareCreate()
                }
                Button {
                    text: controller.emulator_installing
                          ? "Installing PCSX2…"
                          : controller.emulator_release_checking
                            ? "Checking PCSX2…" : "Install / Update PCSX2"
                    enabled: !controller.loading && !controller.import_scanning
                             && !controller.emulator_discovery_scanning
                             && !controller.emulator_bios_scanning
                             && !controller.emulator_release_checking
                             && !controller.emulator_installing
                             && !controller.writing && !controller.launching
                             && !controller.write_conflict
                             && controller.pending_recovery_count === 0
                    onClicked: pcsx2InstallManager.prepare()
                }
                Button {
                    text: "Edit"
                    enabled: emulatorManager.selectedIndex >= 0
                             && !controller.writing
                             && !controller.emulator_installing
                             && !controller.emulator_release_checking
                             && !controller.write_conflict
                             && controller.pending_recovery_count === 0
                    onClicked: emulatorEditor.prepareEdit(emulatorManager.selectedId())
                }
                Button {
                    text: controller.emulator_bios_scanning
                          ? "Checking BIOS…" : "BIOS"
                    enabled: emulatorManager.selectedIndex >= 0
                             && controller.emulator_bios_supported(
                                 emulatorManager.selectedId())
                             && !controller.emulator_bios_scanning
                             && !controller.emulator_discovery_scanning
                             && !controller.emulator_release_checking
                             && !controller.emulator_installing
                             && !controller.writing && !controller.launching
                    onClicked: biosManager.prepare(
                                   emulatorManager.selectedId(),
                                   emulatorManager.selectedTitle())
                }
                Button {
                    text: "Delete"
                    enabled: emulatorManager.selectedIndex >= 0
                             && !controller.writing
                             && !controller.emulator_installing
                             && !controller.emulator_release_checking
                             && !controller.write_conflict
                             && controller.pending_recovery_count === 0
                    onClicked: deleteEmulatorConfirmation.prepare(
                                   emulatorManager.selectedId(),
                                   emulatorManager.selectedTitle())
                }
                Item { Layout.fillWidth: true }
                Label {
                    text: Math.max(0, controller.emulator_entry_count() - 2)
                          + " configured"
                    color: "#7d8590"
                }
            }
            Rectangle {
                Layout.fillWidth: true
                implicitHeight: 1
                color: "#30363d"
            }
            RowLayout {
                Layout.fillWidth: true
                Label {
                    text: "Installed candidates"
                    color: "white"
                    font.bold: true
                }
                Item { Layout.fillWidth: true }
                Button {
                    text: controller.emulator_discovery_scanning
                          ? "Scanning…" : "Scan Installed"
                    enabled: !controller.loading && !controller.import_scanning
                             && !controller.emulator_discovery_scanning
                             && !controller.emulator_bios_scanning
                             && !controller.emulator_release_checking
                             && !controller.emulator_installing
                             && !controller.writing && !controller.launching
                    onClicked: controller.scan_installed_emulators()
                }
            }
            Label {
                Layout.fillWidth: true
                text: "The scan checks the portable Emulators folder, native application locations, and PATH for reviewed executable names. It never starts or modifies a candidate. Review & Add opens the complete editor before any XML is written."
                wrapMode: Text.Wrap
                color: "#7d8590"
                font.pixelSize: 11
            }
            ListView {
                id: discoveredEmulatorList
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.minimumHeight: 130
                clip: true
                model: {
                    const revision = controller.emulator_discovery_revision
                    return controller.discovered_emulator_count()
                }
                delegate: ItemDelegate {
                    id: discoveredEmulatorDelegate
                    required property int index
                    width: discoveredEmulatorList.width
                    property string emulatorTitle:
                        controller.discovered_emulator_title_at(index)
                    property string executablePath:
                        controller.discovered_emulator_path_at(index)
                    property string discoverySource:
                        controller.discovered_emulator_source_at(index)
                    property bool registered:
                        controller.discovered_emulator_registered_at(index)
                    highlighted:
                        emulatorManager.selectedDiscoveryIndex === index
                    contentItem: RowLayout {
                        ColumnLayout {
                            Layout.fillWidth: true
                            Label {
                                Layout.fillWidth: true
                                text: discoveredEmulatorDelegate.emulatorTitle
                                color: "white"
                                font.bold: true
                                elide: Text.ElideRight
                            }
                            Label {
                                Layout.fillWidth: true
                                text: discoveredEmulatorDelegate.executablePath
                                      + " · "
                                      + discoveredEmulatorDelegate.discoverySource
                                color: "#7d8590"
                                font.pixelSize: 11
                                elide: Text.ElideMiddle
                            }
                        }
                        Label {
                            text: discoveredEmulatorDelegate.registered
                                  ? "Configured" : "Review required"
                            color: discoveredEmulatorDelegate.registered
                                   ? "#3fb950" : "#d29922"
                        }
                    }
                    onClicked:
                        emulatorManager.selectedDiscoveryIndex = index
                    onDoubleClicked: {
                        if (!registered)
                            emulatorEditor.prepareDiscovered(index)
                    }
                }
            }
            RowLayout {
                Layout.fillWidth: true
                Button {
                    text: "Review & Add"
                    enabled: emulatorManager.selectedDiscoveryIndex >= 0
                             && !controller.discovered_emulator_registered_at(
                                 emulatorManager.selectedDiscoveryIndex)
                             && !controller.writing
                             && !controller.emulator_installing
                             && !controller.emulator_release_checking
                             && !controller.write_conflict
                             && controller.pending_recovery_count === 0
                    onClicked: emulatorEditor.prepareDiscovered(
                                   emulatorManager.selectedDiscoveryIndex)
                }
                Item { Layout.fillWidth: true }
                Label {
                    text: controller.discovered_emulator_count()
                          + " candidate(s)"
                    color: "#7d8590"
                }
            }
        }
    }

    Dialog {
        id: pcsx2InstallManager
        anchors.centerIn: parent
        modal: true
        title: "Managed PCSX2"
        standardButtons: Dialog.Close
        property var review: null
        property var managedReview: null
        property bool releaseCheckPending: false

        function loadReview() {
            review = controller.emulator_release_json.length > 0
                     ? JSON.parse(controller.emulator_release_json) : null
            managedReview = controller.emulator_managed_json.length > 0
                            ? JSON.parse(controller.emulator_managed_json)
                            : null
        }

        function prepare() {
            review = null
            managedReview = null
            releaseCheckPending = true
            open()
            controller.review_managed_pcsx2()
        }

        function smokeCheck() {
            review = null
            open()
            controller.check_pcsx2_release()
        }

        function actionLabel() {
            if (review === null)
                return "Install"
            if (review.action === "update")
                return "Update"
            if (review.action === "repair")
                return "Repair"
            if (review.action === "current")
                return "Current"
            if (review.action === "blocked")
                return "Blocked"
            return "Install"
        }

        Connections {
            target: controller
            function onEmulatorInstallRevisionChanged() {
                pcsx2InstallManager.loadReview()
                if (pcsx2InstallManager.releaseCheckPending
                        && !controller.emulator_managed_checking
                        && pcsx2InstallManager.managedReview !== null) {
                    pcsx2InstallManager.releaseCheckPending = false
                    Qt.callLater(function() {
                        controller.check_pcsx2_release()
                    })
                }
            }
        }

        contentItem: ColumnLayout {
            implicitWidth: 760
            implicitHeight: 560
            spacing: 10

            Label {
                Layout.fillWidth: true
                text: "This provider checks PCSX2/pcsx2 on GitHub, selects the exact native artifact, verifies GitHub's SHA-256 digest and byte count, then commits every portable artifact path, portable.ini, an ownership manifest, and Data/Emulators.xml together. It never runs the downloaded artifact during installation."
                wrapMode: Text.Wrap
                color: "#7fbfff"
            }
            RowLayout {
                Layout.fillWidth: true
                BusyIndicator {
                    running: controller.emulator_release_checking
                             || controller.emulator_managed_checking
                    visible: running
                }
                Label {
                    Layout.fillWidth: true
                    text: {
                        if (controller.emulator_managed_checking)
                            return "Auditing local ownership…"
                        if (controller.emulator_release_checking)
                            return "Checking official releases…"
                        if (pcsx2InstallManager.review === null)
                            return "No reviewed release"
                        const release = pcsx2InstallManager.review.release
                        return "PCSX2 " + release.version
                               + (release.prerelease ? " · nightly" : " · stable")
                               + " · " + pcsx2InstallManager.actionLabel()
                    }
                    color: pcsx2InstallManager.review !== null
                           && pcsx2InstallManager.review.action === "blocked"
                           ? "#f85149" : "#ffffff"
                    font.bold: true
                }
                Button {
                    text: "Check Again"
                    enabled: !controller.emulator_release_checking
                             && !controller.emulator_managed_checking
                             && !controller.emulator_installing
                             && !controller.writing
                    onClicked: pcsx2InstallManager.prepare()
                }
            }
            Label {
                Layout.fillWidth: true
                visible: pcsx2InstallManager.review !== null
                text: pcsx2InstallManager.review === null ? ""
                      : "Install target: "
                        + pcsx2InstallManager.review.install_directory
                color: "white"
                elide: Text.ElideMiddle
            }
            Label {
                Layout.fillWidth: true
                visible: pcsx2InstallManager.review !== null
                text: pcsx2InstallManager.review === null ? ""
                      : "Artifact: "
                        + pcsx2InstallManager.review.release.asset_name
                        + " · "
                        + pcsx2InstallManager.review.release.asset_byte_len
                        + " bytes"
                color: "#c9d1d9"
                elide: Text.ElideMiddle
            }
            Label {
                Layout.fillWidth: true
                visible: pcsx2InstallManager.review !== null
                text: pcsx2InstallManager.review === null ? ""
                      : "SHA-256: "
                        + pcsx2InstallManager.review.release.asset_sha256
                color: "#7d8590"
                font.family: "monospace"
                font.pixelSize: 11
                elide: Text.ElideMiddle
            }
            Label {
                Layout.fillWidth: true
                visible: pcsx2InstallManager.review !== null
                text: pcsx2InstallManager.review === null ? ""
                      : "Official source: "
                        + pcsx2InstallManager.review.release.asset_url
                color: "#7d8590"
                font.pixelSize: 11
                elide: Text.ElideMiddle
            }
            Label {
                Layout.fillWidth: true
                visible: pcsx2InstallManager.review !== null
                         && pcsx2InstallManager.review.release.artifact_kind
                            === "macos_qt_tar_xz"
                text: "The official macOS bundle is x86-64. Intel Macs run it natively; Apple Silicon requires Rosetta 2. Native macOS execution still needs a real-host verification gate."
                wrapMode: Text.Wrap
                color: "#d29922"
            }
            Label {
                Layout.fillWidth: true
                visible: pcsx2InstallManager.managedReview !== null
                         && pcsx2InstallManager.managedReview.managed_install
                            !== null
                text: {
                    if (pcsx2InstallManager.managedReview === null
                            || pcsx2InstallManager.managedReview.managed_install
                               === null)
                        return ""
                    const installed =
                        pcsx2InstallManager.managedReview.managed_install
                    return "Managed version "
                           + installed.manifest.version
                           + " · executable "
                           + installed.executable_state
                           + " · "
                           + pcsx2InstallManager.managedReview.owned_file_count
                           + " recoverable owned files"
                }
                color: "#c9d1d9"
            }
            Label {
                Layout.fillWidth: true
                visible: pcsx2InstallManager.managedReview !== null
                         && pcsx2InstallManager.managedReview.blocked_reason
                            !== null
                text: pcsx2InstallManager.managedReview === null ? ""
                      : pcsx2InstallManager.managedReview.blocked_reason || ""
                wrapMode: Text.Wrap
                color: "#f85149"
            }
            Label {
                Layout.fillWidth: true
                visible: pcsx2InstallManager.review !== null
                         && pcsx2InstallManager.review.blocked_reason !== null
                text: pcsx2InstallManager.review === null ? ""
                      : pcsx2InstallManager.review.blocked_reason || ""
                wrapMode: Text.Wrap
                color: "#f85149"
            }
            Label {
                Layout.fillWidth: true
                text: "Updates replace only manifest-owned paths. Removal requires every owned file to match its recorded digest, refuses pinned emulator references, retains exact recovery copies, and leaves user settings, unrelated files, and directories in place."
                wrapMode: Text.Wrap
                color: "#d29922"
            }
            ProgressBar {
                Layout.fillWidth: true
                visible: controller.emulator_installing
                from: 0
                to: 1
                value: controller.emulator_install_progress
            }
            RowLayout {
                Layout.fillWidth: true
                Button {
                    text: pcsx2InstallManager.actionLabel()
                    enabled: pcsx2InstallManager.review !== null
                             && pcsx2InstallManager.review.can_install
                             && !controller.emulator_release_checking
                             && !controller.emulator_managed_checking
                             && !controller.emulator_installing
                             && !controller.writing
                             && !controller.write_conflict
                             && controller.pending_recovery_count === 0
                    onClicked: controller.install_pcsx2_release()
                }
                Button {
                    text: "Cancel Download"
                    visible: controller.emulator_installing
                    enabled: controller.emulator_installing
                    onClicked: controller.cancel_emulator_install()
                }
                Button {
                    text: "Remove Managed Install"
                    visible: pcsx2InstallManager.managedReview !== null
                             && pcsx2InstallManager.managedReview.managed_install
                                !== null
                    enabled: visible
                             && pcsx2InstallManager.managedReview.can_remove
                             && !controller.emulator_release_checking
                             && !controller.emulator_managed_checking
                             && !controller.emulator_installing
                             && !controller.writing
                             && !controller.write_conflict
                             && controller.pending_recovery_count === 0
                    onClicked: removeManagedPcsx2Confirmation.open()
                }
                Item { Layout.fillWidth: true }
                Label {
                    visible: controller.emulator_installing
                    text: Math.round(
                              controller.emulator_install_progress * 100)
                          + "%"
                    color: "#7d8590"
                }
            }
        }
    }

    Dialog {
        id: removeManagedPcsx2Confirmation
        anchors.centerIn: parent
        modal: true
        title: "Remove managed PCSX2?"
        standardButtons: Dialog.Yes | Dialog.No

        onAccepted: controller.remove_managed_pcsx2()

        contentItem: Label {
            width: 650
            text: pcsx2InstallManager.managedReview === null ? ""
                  : "Remove "
                    + pcsx2InstallManager.managedReview.owned_file_count
                    + " verified port-owned file(s) and the managed emulator definition? Exact recovery copies are retained. User settings, unrelated files, ROMs, media, and directories are not deleted."
            wrapMode: Text.Wrap
        }
    }

    Dialog {
        id: biosManager
        anchors.centerIn: parent
        modal: true
        title: emulatorTitle.length > 0
               ? emulatorTitle + " BIOS Status" : "Emulator BIOS Status"
        standardButtons: Dialog.Close
        property string emulatorId: ""
        property string emulatorTitle: ""
        property var audit: null

        function loadAudit() {
            const serialized = controller.emulator_bios_audit_json
            if (serialized.length === 0) {
                audit = null
                return
            }
            const parsed = JSON.parse(serialized)
            audit = parsed.emulator_id === emulatorId ? parsed : null
        }

        function prepare(id, title) {
            emulatorId = id
            emulatorTitle = title
            audit = null
            open()
            controller.scan_emulator_bios(id)
        }

        function smokeAudit(id, title) {
            prepare(id, title)
        }

        function stateLabel(state) {
            if (state === "valid")
                return "VALID"
            if (state === "hash_mismatch")
                return "HASH MISMATCH"
            if (state === "unsafe_entry")
                return "UNSAFE ENTRY"
            if (state === "unreadable")
                return "UNREADABLE"
            return "MISSING"
        }

        function stateColor(state) {
            if (state === "valid")
                return "#3fb950"
            if (state === "missing")
                return "#7d8590"
            return "#f85149"
        }

        function groupSummary() {
            if (audit === null)
                return ""
            let required = 0
            let satisfied = 0
            for (let index = 0; index < audit.groups.length; ++index) {
                const group = audit.groups[index]
                if (group.required) {
                    ++required
                    if (group.satisfied)
                        ++satisfied
                }
            }
            return audit.groups.length + " requirement group(s) · "
                   + satisfied + "/" + required
                   + " required group(s) ready"
        }

        function targetSummary() {
            if (audit === null || audit.targets.length === 0)
                return ""
            const summaries = []
            for (let index = 0; index < audit.targets.length; ++index) {
                const target = audit.targets[index]
                summaries.push(target.platform + " (" + target.core + ", "
                               + target.requirement_count + " file(s))")
            }
            return summaries.join(" · ")
        }

        function groupDescription(groupId) {
            if (audit === null)
                return groupId
            for (let index = 0; index < audit.groups.length; ++index) {
                if (audit.groups[index].id === groupId)
                    return audit.groups[index].description
            }
            return groupId
        }

        Connections {
            target: controller
            function onEmulatorBiosRevisionChanged() {
                biosManager.loadAudit()
            }
        }

        contentItem: ColumnLayout {
            implicitWidth: 820
            implicitHeight: 620
            spacing: 10

            Label {
                Layout.fillWidth: true
                text: {
                    const name = biosManager.audit === null
                               ? biosManager.emulatorTitle
                               : biosManager.audit.emulator_title
                    return "This is a read-only validation of the complete "
                           + "LaunchBox 13.27 " + name
                           + " BIOS contract. Every required group must be "
                           + "satisfied. The audit does not run the emulator, "
                           + "follow firmware symlinks, download firmware, or "
                           + "change files and configuration."
                }
                wrapMode: Text.Wrap
                color: "#7fbfff"
            }
            RowLayout {
                Layout.fillWidth: true
                BusyIndicator {
                    running: controller.emulator_bios_scanning
                    visible: running
                }
                Label {
                    Layout.fillWidth: true
                    text: {
                        if (controller.emulator_bios_scanning)
                            return "Checking configured BIOS location…"
                        if (biosManager.audit === null)
                            return "No audit result"
                        return biosManager.audit.ready
                               ? "READY — "
                                 + biosManager.audit.valid_count
                                 + " recognized required file(s)"
                               : "NEEDS FIRMWARE — one or more required groups are unsatisfied"
                    }
                    color: biosManager.audit !== null
                           && biosManager.audit.ready
                           ? "#3fb950" : "#d29922"
                    font.bold: true
                }
                Button {
                    text: "Check Again"
                    enabled: biosManager.emulatorId.length > 0
                             && !controller.emulator_bios_scanning
                    onClicked: controller.scan_emulator_bios(
                                   biosManager.emulatorId)
                }
            }
            Label {
                Layout.fillWidth: true
                visible: biosManager.audit !== null
                text: biosManager.audit === null ? ""
                      : "Search root: " + biosManager.audit.search_root
                color: "white"
                elide: Text.ElideMiddle
            }
            Label {
                Layout.fillWidth: true
                visible: biosManager.audit !== null
                         && biosManager.audit.targets.length > 0
                text: biosManager.targetSummary()
                color: "#7fbfff"
                font.pixelSize: 11
                elide: Text.ElideRight
            }
            Label {
                Layout.fillWidth: true
                visible: biosManager.audit !== null
                text: {
                    if (biosManager.audit === null)
                        return ""
                    const config = biosManager.audit.configuration_path === null
                                 ? "no readable emulator configuration"
                                 : biosManager.audit.configuration_path
                    return "Source: " + biosManager.audit.location_source
                           + " · " + config
                }
                color: "#7d8590"
                font.pixelSize: 11
                elide: Text.ElideMiddle
            }
            Label {
                Layout.fillWidth: true
                visible: biosManager.audit !== null
                text: {
                    if (biosManager.audit === null)
                        return ""
                    return biosManager.groupSummary()
                }
                color: "white"
            }
            ListView {
                id: biosGroupList
                Layout.fillWidth: true
                Layout.preferredHeight: biosManager.audit === null ? 0
                    : Math.min(150, biosManager.audit.groups.length * 34)
                visible: biosManager.audit !== null
                         && biosManager.audit.groups.length > 0
                clip: true
                model: biosManager.audit === null ? [] : biosManager.audit.groups
                delegate: ItemDelegate {
                    required property var modelData
                    width: biosGroupList.width
                    contentItem: RowLayout {
                        Label {
                            Layout.fillWidth: true
                            text: modelData.description
                                  + (modelData.required ? " · REQUIRED" : " · OPTIONAL")
                                  + " · " + modelData.rule.toUpperCase()
                            color: "white"
                            elide: Text.ElideRight
                        }
                        Label {
                            text: modelData.satisfied ? "READY" : "MISSING"
                            color: modelData.satisfied ? "#3fb950" : "#d29922"
                            font.bold: true
                        }
                    }
                }
            }
            ListView {
                id: biosFileList
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                model: biosManager.audit === null ? [] : biosManager.audit.files
                delegate: ItemDelegate {
                    required property var modelData
                    width: biosFileList.width
                    contentItem: RowLayout {
                        ColumnLayout {
                            Layout.fillWidth: true
                            Label {
                                Layout.fillWidth: true
                                text: biosManager.groupDescription(modelData.group_id) + " · "
                                      + modelData.file_name + " · "
                                      + modelData.description
                                color: "white"
                                font.bold: true
                                elide: Text.ElideRight
                            }
                            Label {
                                Layout.fillWidth: true
                                text: modelData.path
                                color: "#7d8590"
                                font.pixelSize: 11
                                elide: Text.ElideMiddle
                            }
                            Label {
                                Layout.fillWidth: true
                                visible: modelData.state === "hash_mismatch"
                                         && modelData.expected_md5 !== null
                                text: "Expected " + modelData.expected_md5
                                      + " · actual "
                                      + (modelData.actual_md5 || "unavailable")
                                color: "#f85149"
                                font.pixelSize: 11
                                elide: Text.ElideRight
                            }
                        }
                        Label {
                            text: biosManager.stateLabel(modelData.state)
                            color: biosManager.stateColor(modelData.state)
                            font.bold: true
                        }
                    }
                }
            }
        }
    }

    Dialog {
        id: emulatorEditor
        anchors.centerIn: parent
        modal: true
        title: createMode ? "Add Emulator" : "Edit " + emulatorTitleField.text
        standardButtons: Dialog.Save | Dialog.Cancel
        property bool createMode: false
        property string originalId: ""
        property var draft: null

        ListModel { id: emulatorPlatformEditorModel }

        function storedText(value) {
            return value === null || value === undefined ? "" : value
        }

        function optionalText(value) {
            return value.trim().length > 0 ? value : null
        }

        function platformIndex(name) {
            if (draft === null)
                return -1
            for (let index = 0; index < draft.available_platforms.length; ++index) {
                if (draft.available_platforms[index].toLowerCase()
                        === name.toLowerCase())
                    return index
            }
            return -1
        }

        function defaultPlatformOptions() {
            if (draft === null)
                return ["Platform default"]
            return ["Platform default"].concat(draft.available_platforms)
        }

        function loadPayload(serialized, creating) {
            if (serialized.length === 0)
                return
            createMode = creating
            draft = JSON.parse(serialized)
            originalId = creating ? "" : draft.emulator.id
            const emulator = draft.emulator
            emulatorDefinitionIdField.text = emulator.id
            emulatorTitleField.text = emulator.title
            emulatorApplicationPathField.text = emulator.application_path
            emulatorCommandLineField.text = storedText(emulator.command_line)
            const defaultIndex = emulator.default_platform === null
                                 ? -1 : platformIndex(emulator.default_platform)
            emulatorDefaultPlatform.currentIndex = defaultIndex + 1
            emulatorStartupLoadDelayField.text = String(emulator.startup_load_delay)
            emulatorAutoExtractCheck.checked = emulator.auto_extract
            emulatorAggressiveWindowHidingCheck.checked =
                emulator.aggressive_window_hiding
            emulatorDefaultPauseSettingsPushedCheck.checked =
                emulator.default_pause_settings_pushed
            emulatorDisableShutdownScreenCheck.checked =
                emulator.disable_shutdown_screen
            emulatorHardcoreAchievementsCheck.checked =
                emulator.enable_hardcore_achievements
            emulatorFileNameOnlyCheck.checked =
                emulator.file_name_without_extension_and_path
            emulatorForcefulPauseCheck.checked =
                emulator.forceful_pause_screen_activation
            emulatorHideFullscreenWindowsCheck.checked =
                emulator.hide_all_non_exclusive_fullscreen_windows
            emulatorHideConsoleCheck.checked = emulator.hide_console
            emulatorHideMouseCheck.checked = emulator.hide_mouse_cursor_in_game
            emulatorCheevoLoginCheck.checked =
                emulator.login_to_cheevo_on_game_launch
            emulatorNoQuotesCheck.checked = emulator.no_quotes
            emulatorNoSpaceCheck.checked = emulator.no_space
            emulatorSkipVersionCheck.checked = emulator.skip_version_check
            emulatorSuspendOnPauseCheck.checked = emulator.suspend_process_on_pause
            emulatorUsePauseScreenCheck.checked = emulator.use_pause_screen
            emulatorUseStartupScreenCheck.checked = emulator.use_startup_screen
            emulatorAutoHotkeyScript.text = storedText(emulator.auto_hotkey_script)
            emulatorExitAutoHotkeyScript.text =
                storedText(emulator.exit_auto_hotkey_script)
            emulatorLoadStateAutoHotkeyScript.text =
                storedText(emulator.load_state_auto_hotkey_script)
            emulatorPauseAutoHotkeyScript.text =
                storedText(emulator.pause_auto_hotkey_script)
            emulatorResetAutoHotkeyScript.text =
                storedText(emulator.reset_auto_hotkey_script)
            emulatorResumeAutoHotkeyScript.text =
                storedText(emulator.resume_auto_hotkey_script)
            emulatorSaveStateAutoHotkeyScript.text =
                storedText(emulator.save_state_auto_hotkey_script)
            emulatorSwapDiscsAutoHotkeyScript.text =
                storedText(emulator.swap_discs_auto_hotkey_script)
            emulatorPlatformEditorModel.clear()
            for (let index = 0; index < draft.platforms.length; ++index) {
                const mapping = draft.platforms[index]
                emulatorPlatformEditorModel.append({
                    sourceIndex: mapping.source_index === null
                                 ? -1 : mapping.source_index,
                    platformName: mapping.platform,
                    commandLine: storedText(mapping.command_line),
                    isDefault: mapping.default,
                    autoExtractMode: mapping.auto_extract === null
                                     ? 0 : (mapping.auto_extract ? 1 : 2),
                    m3uEnabled: mapping.m3u_disc_load_enabled
                })
            }
            open()
        }

        function prepareCreate() {
            loadPayload(controller.new_emulator_edit_payload(), true)
        }

        function prepareEdit(emulatorId) {
            loadPayload(controller.emulator_edit_payload(emulatorId), false)
        }

        function prepareDiscovered(candidateIndex) {
            loadPayload(
                controller.discovered_emulator_edit_payload(candidateIndex), true)
        }

        function editPayload() {
            const emulator = draft.emulator
            emulator.title = emulatorTitleField.text
            emulator.application_path = emulatorApplicationPathField.text
            emulator.command_line = optionalText(emulatorCommandLineField.text)
            emulator.default_platform = emulatorDefaultPlatform.currentIndex <= 0
                                        ? null
                                        : draft.available_platforms[
                                              emulatorDefaultPlatform.currentIndex - 1]
            emulator.startup_load_delay =
                Number(emulatorStartupLoadDelayField.text)
            emulator.auto_extract = emulatorAutoExtractCheck.checked
            emulator.aggressive_window_hiding =
                emulatorAggressiveWindowHidingCheck.checked
            emulator.default_pause_settings_pushed =
                emulatorDefaultPauseSettingsPushedCheck.checked
            emulator.disable_shutdown_screen =
                emulatorDisableShutdownScreenCheck.checked
            emulator.enable_hardcore_achievements =
                emulatorHardcoreAchievementsCheck.checked
            emulator.file_name_without_extension_and_path =
                emulatorFileNameOnlyCheck.checked
            emulator.forceful_pause_screen_activation =
                emulatorForcefulPauseCheck.checked
            emulator.hide_all_non_exclusive_fullscreen_windows =
                emulatorHideFullscreenWindowsCheck.checked
            emulator.hide_console = emulatorHideConsoleCheck.checked
            emulator.hide_mouse_cursor_in_game = emulatorHideMouseCheck.checked
            emulator.login_to_cheevo_on_game_launch =
                emulatorCheevoLoginCheck.checked
            emulator.no_quotes = emulatorNoQuotesCheck.checked
            emulator.no_space = emulatorNoSpaceCheck.checked
            emulator.skip_version_check = emulatorSkipVersionCheck.checked
            emulator.suspend_process_on_pause =
                emulatorSuspendOnPauseCheck.checked
            emulator.use_pause_screen = emulatorUsePauseScreenCheck.checked
            emulator.use_startup_screen = emulatorUseStartupScreenCheck.checked
            emulator.auto_hotkey_script =
                optionalText(emulatorAutoHotkeyScript.text)
            emulator.exit_auto_hotkey_script =
                optionalText(emulatorExitAutoHotkeyScript.text)
            emulator.load_state_auto_hotkey_script =
                optionalText(emulatorLoadStateAutoHotkeyScript.text)
            emulator.pause_auto_hotkey_script =
                optionalText(emulatorPauseAutoHotkeyScript.text)
            emulator.reset_auto_hotkey_script =
                optionalText(emulatorResetAutoHotkeyScript.text)
            emulator.resume_auto_hotkey_script =
                optionalText(emulatorResumeAutoHotkeyScript.text)
            emulator.save_state_auto_hotkey_script =
                optionalText(emulatorSaveStateAutoHotkeyScript.text)
            emulator.swap_discs_auto_hotkey_script =
                optionalText(emulatorSwapDiscsAutoHotkeyScript.text)
            const mappings = []
            for (let index = 0; index < emulatorPlatformEditorModel.count; ++index) {
                const mapping = emulatorPlatformEditorModel.get(index)
                mappings.push({
                    source_index: mapping.sourceIndex < 0
                                  ? null : mapping.sourceIndex,
                    platform: mapping.platformName,
                    command_line: optionalText(mapping.commandLine),
                    default: mapping.isDefault,
                    auto_extract: mapping.autoExtractMode === 0
                                  ? null : mapping.autoExtractMode === 1,
                    m3u_disc_load_enabled: mapping.m3uEnabled
                })
            }
            draft.platforms = mappings
            return JSON.stringify(draft)
        }

        function addMapping() {
            if (draft === null || draft.available_platforms.length === 0)
                return
            let platform = draft.available_platforms[0]
            for (let available = 0;
                    available < draft.available_platforms.length; ++available) {
                let used = false
                for (let index = 0; index < emulatorPlatformEditorModel.count; ++index) {
                    if (emulatorPlatformEditorModel.get(index).platformName
                            === draft.available_platforms[available]) {
                        used = true
                        break
                    }
                }
                if (!used) {
                    platform = draft.available_platforms[available]
                    break
                }
            }
            emulatorPlatformEditorModel.append({
                sourceIndex: -1,
                platformName: platform,
                commandLine: "",
                isDefault: false,
                autoExtractMode: 0,
                m3uEnabled: false
            })
        }

        function smokeEdit(emulatorId) {
            prepareEdit(emulatorId)
            emulatorTitleField.text = "Edited Fixture Emulator"
            emulatorApplicationPathField.text =
                "Emulators\\Edited Fixture\\fixture.exe"
            emulatorAutoHotkeyScript.text = "Smoke launch script"
            emulatorUseStartupScreenCheck.checked = true
            emulatorUsePauseScreenCheck.checked = true
            if (emulatorPlatformEditorModel.count > 0) {
                emulatorPlatformEditorModel.setProperty(
                            0, "commandLine", "--edited-mapping")
                emulatorPlatformEditorModel.setProperty(0, "isDefault", true)
                emulatorPlatformEditorModel.setProperty(0, "autoExtractMode", 2)
                emulatorPlatformEditorModel.setProperty(0, "m3uEnabled", true)
            }
            Qt.callLater(function() { emulatorEditor.accept() })
        }

        function smokeCreate() {
            prepareCreate()
            emulatorTitleField.text = "Temporary Qt Emulator"
            emulatorApplicationPathField.text =
                "C:\\Portable\\Temporary Qt\\temp.exe"
            emulatorUseStartupScreenCheck.checked = true
            emulatorUsePauseScreenCheck.checked = true
            addMapping()
            Qt.callLater(function() { emulatorEditor.accept() })
        }

        function smokeDiscovered(candidateIndex) {
            prepareDiscovered(candidateIndex)
            Qt.callLater(function() { emulatorEditor.accept() })
        }

        onAccepted: {
            const payload = editPayload()
            if (createMode)
                controller.add_emulator(payload)
            else
                controller.save_emulator(originalId, payload)
        }

        contentItem: ScrollView {
            id: emulatorEditorScroll
            implicitWidth: 850
            implicitHeight: Math.min(700, window.height - 120)
            contentWidth: availableWidth
            clip: true

            ColumnLayout {
                width: emulatorEditorScroll.availableWidth
                spacing: 12

                Label {
                    Layout.fillWidth: true
                    text: "The generated emulator ID is immutable. Paths and scripts are persisted exactly as LaunchBox data; host path translation happens only when a game is launched."
                    wrapMode: Text.Wrap
                    color: "#d29922"
                }
                GridLayout {
                    Layout.fillWidth: true
                    columns: 2
                    columnSpacing: 12
                    rowSpacing: 8
                    Label { text: "ID" }
                    TextField {
                        id: emulatorDefinitionIdField
                        Layout.fillWidth: true
                        readOnly: true
                    }
                    Label { text: "Title" }
                    TextField { id: emulatorTitleField; Layout.fillWidth: true }
                    Label { text: "Application path" }
                    TextField {
                        id: emulatorApplicationPathField
                        Layout.fillWidth: true
                        placeholderText: "Stored LaunchBox executable path"
                    }
                    Label { text: "Command line" }
                    TextField {
                        id: emulatorCommandLineField
                        Layout.fillWidth: true
                    }
                    Label { text: "Default platform" }
                    ComboBox {
                        id: emulatorDefaultPlatform
                        Layout.fillWidth: true
                        model: emulatorEditor.defaultPlatformOptions()
                    }
                    Label { text: "Startup delay (ms)" }
                    TextField {
                        id: emulatorStartupLoadDelayField
                        Layout.fillWidth: true
                        validator: IntValidator { bottom: 0 }
                    }
                }
                Flow {
                    Layout.fillWidth: true
                    spacing: 8
                    CheckBox { id: emulatorAutoExtractCheck; text: "Extract archives" }
                    CheckBox {
                        id: emulatorAggressiveWindowHidingCheck
                        text: "Aggressive window hiding"
                    }
                    CheckBox {
                        id: emulatorDefaultPauseSettingsPushedCheck
                        text: "Default pause settings pushed"
                    }
                    CheckBox {
                        id: emulatorDisableShutdownScreenCheck
                        text: "Disable shutdown screen"
                    }
                    CheckBox {
                        id: emulatorHardcoreAchievementsCheck
                        text: "Hardcore achievements"
                    }
                    CheckBox {
                        id: emulatorFileNameOnlyCheck
                        text: "ROM filename only"
                    }
                    CheckBox {
                        id: emulatorForcefulPauseCheck
                        text: "Forceful pause activation"
                    }
                    CheckBox {
                        id: emulatorHideFullscreenWindowsCheck
                        text: "Hide non-exclusive fullscreen windows"
                    }
                    CheckBox { id: emulatorHideConsoleCheck; text: "Hide console" }
                    CheckBox {
                        id: emulatorHideMouseCheck
                        text: "Hide mouse cursor in game"
                    }
                    CheckBox {
                        id: emulatorCheevoLoginCheck
                        text: "Log in to achievements on launch"
                    }
                    CheckBox { id: emulatorNoQuotesCheck; text: "Do not quote ROM path" }
                    CheckBox { id: emulatorNoSpaceCheck; text: "No space before ROM path" }
                    CheckBox {
                        id: emulatorSkipVersionCheck
                        text: "Skip version check"
                    }
                    CheckBox {
                        id: emulatorSuspendOnPauseCheck
                        text: "Suspend process on pause"
                    }
                    CheckBox {
                        id: emulatorUsePauseScreenCheck
                        text: "Use pause screen"
                    }
                    CheckBox {
                        id: emulatorUseStartupScreenCheck
                        text: "Use startup screen"
                    }
                }
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 1
                    color: "#30363d"
                }
                Label { text: "AutoHotkey scripts"; font.pixelSize: 18; font.bold: true }
                GridLayout {
                    Layout.fillWidth: true
                    columns: 2
                    columnSpacing: 12
                    rowSpacing: 8
                    Label { text: "Launch" }
                    TextArea {
                        id: emulatorAutoHotkeyScript
                        Layout.fillWidth: true
                        Layout.preferredHeight: 60
                        wrapMode: TextEdit.Wrap
                    }
                    Label { text: "Exit" }
                    TextArea {
                        id: emulatorExitAutoHotkeyScript
                        Layout.fillWidth: true
                        Layout.preferredHeight: 60
                        wrapMode: TextEdit.Wrap
                    }
                    Label { text: "Load state" }
                    TextArea {
                        id: emulatorLoadStateAutoHotkeyScript
                        Layout.fillWidth: true
                        Layout.preferredHeight: 60
                        wrapMode: TextEdit.Wrap
                    }
                    Label { text: "Pause" }
                    TextArea {
                        id: emulatorPauseAutoHotkeyScript
                        Layout.fillWidth: true
                        Layout.preferredHeight: 60
                        wrapMode: TextEdit.Wrap
                    }
                    Label { text: "Reset" }
                    TextArea {
                        id: emulatorResetAutoHotkeyScript
                        Layout.fillWidth: true
                        Layout.preferredHeight: 60
                        wrapMode: TextEdit.Wrap
                    }
                    Label { text: "Resume" }
                    TextArea {
                        id: emulatorResumeAutoHotkeyScript
                        Layout.fillWidth: true
                        Layout.preferredHeight: 60
                        wrapMode: TextEdit.Wrap
                    }
                    Label { text: "Save state" }
                    TextArea {
                        id: emulatorSaveStateAutoHotkeyScript
                        Layout.fillWidth: true
                        Layout.preferredHeight: 60
                        wrapMode: TextEdit.Wrap
                    }
                    Label { text: "Swap discs" }
                    TextArea {
                        id: emulatorSwapDiscsAutoHotkeyScript
                        Layout.fillWidth: true
                        Layout.preferredHeight: 60
                        wrapMode: TextEdit.Wrap
                    }
                }
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 1
                    color: "#30363d"
                }
                Label { text: "Platform mappings"; font.pixelSize: 18; font.bold: true }
                Label {
                    Layout.fillWidth: true
                    text: "A platform can have one mapping per emulator and one default emulator. Selecting Default here atomically clears the former default mapping for that platform."
                    wrapMode: Text.Wrap
                    color: "#7d8590"
                }
                Repeater {
                    model: emulatorPlatformEditorModel
                    delegate: ColumnLayout {
                        required property int index
                        required property int sourceIndex
                        required property string platformName
                        required property string commandLine
                        required property bool isDefault
                        required property int autoExtractMode
                        required property bool m3uEnabled
                        Layout.fillWidth: true

                        RowLayout {
                            Layout.fillWidth: true
                            ComboBox {
                                Layout.preferredWidth: 190
                                model: emulatorEditor.draft === null
                                       ? [] : emulatorEditor.draft.available_platforms
                                currentIndex: emulatorEditor.platformIndex(platformName)
                                onActivated: emulatorPlatformEditorModel.setProperty(
                                                 index, "platformName",
                                                 emulatorEditor.draft
                                                 .available_platforms[currentIndex])
                            }
                            TextField {
                                Layout.fillWidth: true
                                text: commandLine
                                placeholderText: "Per-platform command line"
                                onTextEdited: emulatorPlatformEditorModel.setProperty(
                                                  index, "commandLine", text)
                            }
                            ComboBox {
                                Layout.preferredWidth: 135
                                model: ["Inherit extract", "Extract", "Do not extract"]
                                currentIndex: autoExtractMode
                                onActivated: emulatorPlatformEditorModel.setProperty(
                                                 index, "autoExtractMode", currentIndex)
                            }
                            Button {
                                text: "Remove"
                                onClicked: emulatorPlatformEditorModel.remove(index)
                            }
                        }
                        RowLayout {
                            CheckBox {
                                text: "Default emulator"
                                checked: isDefault
                                onToggled: emulatorPlatformEditorModel.setProperty(
                                               index, "isDefault", checked)
                            }
                            CheckBox {
                                text: "M3U multi-disc loading"
                                checked: m3uEnabled
                                onToggled: emulatorPlatformEditorModel.setProperty(
                                               index, "m3uEnabled", checked)
                            }
                            Item { Layout.fillWidth: true }
                        }
                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 1
                            color: "#30363d"
                        }
                    }
                }
                Button {
                    text: "Add Platform Mapping"
                    enabled: emulatorEditor.draft !== null
                             && emulatorEditor.draft.available_platforms.length > 0
                    onClicked: emulatorEditor.addMapping()
                }
            }
        }
    }

    Dialog {
        id: deleteEmulatorConfirmation
        anchors.centerIn: parent
        modal: true
        title: "Delete " + emulatorTitle + "?"
        standardButtons: Dialog.Yes | Dialog.No
        property string emulatorId: ""
        property string emulatorTitle: ""

        function prepare(id, title) {
            emulatorId = id
            emulatorTitle = title
            open()
        }

        function smokeDelete(id, title) {
            prepare(id, title)
            Qt.callLater(function() { deleteEmulatorConfirmation.accept() })
        }

        onAccepted: controller.delete_emulator(emulatorId)

        contentItem: Label {
            width: 600
            text: "The emulator definition and mappings it owns will be removed only if no game or additional application pins this emulator ID. An exact Emulators.xml backup is created. Emulator binaries, directories, ROMs, and media are never deleted."
            wrapMode: Text.Wrap
        }
    }

    Dialog {
        id: platformEditor
        anchors.centerIn: parent
        modal: true
        title: "Edit " + originalName
        standardButtons: Dialog.Save | Dialog.Cancel
        property string originalName: ""
        property var draft: null

        ListModel { id: platformFolderEditorModel }

        function storedText(value) {
            return value === null || value === undefined ? "" : value
        }

        function optionalText(value) {
            return value.trim().length > 0 ? value : null
        }

        function prepare(name) {
            const serialized = controller.platform_edit_payload(name)
            if (serialized.length === 0)
                return
            originalName = name
            draft = JSON.parse(serialized)
            const metadata = draft.platform.metadata
            platformNameField.text = metadata.name
            platformNestedNameField.text = storedText(metadata.nested_name)
            platformSortTitleField.text = storedText(metadata.sort_title)
            platformScrapeAsField.text = storedText(metadata.scrape_as)
            platformReleaseDateField.text = storedText(draft.platform.release_date)
            platformCategoryField.text = storedText(metadata.category)
            platformImageTypeField.text = storedText(metadata.image_type)
            platformGameFolderField.text = storedText(metadata.folder)
            platformDeveloperField.text = storedText(metadata.developer)
            platformManufacturerField.text = storedText(metadata.manufacturer)
            platformCpuField.text = storedText(metadata.cpu)
            platformMemoryField.text = storedText(metadata.memory)
            platformGraphicsField.text = storedText(metadata.graphics)
            platformSoundField.text = storedText(metadata.sound)
            platformDisplayField.text = storedText(metadata.display)
            platformMediaField.text = storedText(metadata.media)
            platformMaxControllersField.text = storedText(metadata.max_controllers)
            platformBigBoxThemeField.text = storedText(metadata.big_box_theme)
            platformBigBoxViewField.text = storedText(metadata.big_box_view)
            platformVideoPathField.text = storedText(metadata.video_path)
            platformVideosFolderField.text = storedText(metadata.videos_folder)
            platformFrontImagesFolderField.text = storedText(metadata.front_images_folder)
            platformBackImagesFolderField.text = storedText(metadata.back_images_folder)
            platformClearLogoImagesFolderField.text = storedText(metadata.clear_logo_images_folder)
            platformFanartImagesFolderField.text = storedText(metadata.fanart_images_folder)
            platformScreenshotImagesFolderField.text = storedText(metadata.screenshot_images_folder)
            platformBannerImagesFolderField.text = storedText(metadata.banner_images_folder)
            platformSteamBannerImagesFolderField.text = storedText(metadata.steam_banner_images_folder)
            platformManualsFolderField.text = storedText(metadata.manuals_folder)
            platformMusicFolderField.text = storedText(metadata.music_folder)
            platformAndroidThemeVideoPathField.text = storedText(metadata.android_theme_video_path)
            platformNotesField.text = storedText(metadata.notes)
            platformHideBigBoxCheck.checked = metadata.hide_in_big_box
            platformDisableAutoImportCheck.checked = draft.platform.disable_auto_import
            platformFolderEditorModel.clear()
            for (let index = 0; index < draft.folders.length; ++index) {
                const folder = draft.folders[index]
                platformFolderEditorModel.append({
                    sourceIndex: folder.source_index === null ? -1 : folder.source_index,
                    mediaType: folder.media_type,
                    folderPath: folder.folder_path
                })
            }
            open()
        }

        function editPayload() {
            const metadata = draft.platform.metadata
            metadata.nested_name = optionalText(platformNestedNameField.text)
            metadata.sort_title = optionalText(platformSortTitleField.text)
            metadata.scrape_as = optionalText(platformScrapeAsField.text)
            draft.platform.release_date = optionalText(platformReleaseDateField.text)
            metadata.category = optionalText(platformCategoryField.text)
            metadata.image_type = optionalText(platformImageTypeField.text)
            metadata.folder = optionalText(platformGameFolderField.text)
            metadata.developer = optionalText(platformDeveloperField.text)
            metadata.manufacturer = optionalText(platformManufacturerField.text)
            metadata.cpu = optionalText(platformCpuField.text)
            metadata.memory = optionalText(platformMemoryField.text)
            metadata.graphics = optionalText(platformGraphicsField.text)
            metadata.sound = optionalText(platformSoundField.text)
            metadata.display = optionalText(platformDisplayField.text)
            metadata.media = optionalText(platformMediaField.text)
            metadata.max_controllers = optionalText(platformMaxControllersField.text)
            metadata.big_box_theme = optionalText(platformBigBoxThemeField.text)
            metadata.big_box_view = optionalText(platformBigBoxViewField.text)
            metadata.video_path = optionalText(platformVideoPathField.text)
            metadata.videos_folder = optionalText(platformVideosFolderField.text)
            metadata.front_images_folder = optionalText(platformFrontImagesFolderField.text)
            metadata.back_images_folder = optionalText(platformBackImagesFolderField.text)
            metadata.clear_logo_images_folder = optionalText(platformClearLogoImagesFolderField.text)
            metadata.fanart_images_folder = optionalText(platformFanartImagesFolderField.text)
            metadata.screenshot_images_folder = optionalText(platformScreenshotImagesFolderField.text)
            metadata.banner_images_folder = optionalText(platformBannerImagesFolderField.text)
            metadata.steam_banner_images_folder = optionalText(platformSteamBannerImagesFolderField.text)
            metadata.manuals_folder = optionalText(platformManualsFolderField.text)
            metadata.music_folder = optionalText(platformMusicFolderField.text)
            metadata.android_theme_video_path = optionalText(platformAndroidThemeVideoPathField.text)
            metadata.notes = optionalText(platformNotesField.text)
            metadata.hide_in_big_box = platformHideBigBoxCheck.checked
            draft.platform.disable_auto_import = platformDisableAutoImportCheck.checked
            const folders = []
            for (let index = 0; index < platformFolderEditorModel.count; ++index) {
                const folder = platformFolderEditorModel.get(index)
                folders.push({
                    source_index: folder.sourceIndex < 0 ? null : folder.sourceIndex,
                    media_type: folder.mediaType,
                    folder_path: folder.folderPath
                })
            }
            draft.folders = folders
            return JSON.stringify(draft)
        }

        function smokeSave(name) {
            prepare(name)
            platformSortTitleField.text = "Dragon, 32/64"
            platformDeveloperField.text = "Qt Forge"
            platformCpuField.text = "6809"
            platformNotesField.text = "Edited through the real platform dialog."
            platformHideBigBoxCheck.checked = true
            platformDisableAutoImportCheck.checked = true
            if (platformFolderEditorModel.count > 0)
                platformFolderEditorModel.setProperty(0, "folderPath",
                                                      "Images\\Dragon 32_64\\Edited")
            platformFolderEditorModel.append({
                sourceIndex: -1,
                mediaType: "Test Media",
                folderPath: "Portable\\Dragon 32_64"
            })
            Qt.callLater(function() { platformEditor.accept() })
        }

        onAccepted: controller.save_platform(originalName, editPayload())

        contentItem: ScrollView {
            id: platformEditorScroll
            implicitWidth: 760
            implicitHeight: Math.min(680, window.height - 160)
            contentWidth: availableWidth
            clip: true

            ColumnLayout {
                width: platformEditorScroll.availableWidth
                spacing: 12

                Label {
                    Layout.fillWidth: true
                    text: "LaunchBox 13.27 exposes platform identity as getter-only. Name stays fixed until a runtime oracle establishes safe rename behavior across games, emulators, playlists, parents, controllers, settings, and the platform filename."
                    wrapMode: Text.Wrap
                    color: "#d29922"
                }
                GridLayout {
                    Layout.fillWidth: true
                    columns: 2
                    columnSpacing: 12
                    rowSpacing: 8

                    Label { text: "Name" }
                    TextField { id: platformNameField; Layout.fillWidth: true; readOnly: true }
                    Label { text: "Nested name" }
                    TextField { id: platformNestedNameField; Layout.fillWidth: true }
                    Label { text: "Sort title" }
                    TextField { id: platformSortTitleField; Layout.fillWidth: true }
                    Label { text: "Scrape as" }
                    TextField { id: platformScrapeAsField; Layout.fillWidth: true }
                    Label { text: "Release date" }
                    TextField { id: platformReleaseDateField; Layout.fillWidth: true }
                    Label { text: "Category" }
                    TextField { id: platformCategoryField; Layout.fillWidth: true }
                    Label { text: "Image type" }
                    TextField { id: platformImageTypeField; Layout.fillWidth: true }
                    Label { text: "Games folder" }
                    TextField { id: platformGameFolderField; Layout.fillWidth: true }
                    Label { text: "Developer" }
                    TextField { id: platformDeveloperField; Layout.fillWidth: true }
                    Label { text: "Manufacturer" }
                    TextField { id: platformManufacturerField; Layout.fillWidth: true }
                    Label { text: "CPU" }
                    TextField { id: platformCpuField; Layout.fillWidth: true }
                    Label { text: "Memory" }
                    TextField { id: platformMemoryField; Layout.fillWidth: true }
                    Label { text: "Graphics" }
                    TextField { id: platformGraphicsField; Layout.fillWidth: true }
                    Label { text: "Sound" }
                    TextField { id: platformSoundField; Layout.fillWidth: true }
                    Label { text: "Display" }
                    TextField { id: platformDisplayField; Layout.fillWidth: true }
                    Label { text: "Media" }
                    TextField { id: platformMediaField; Layout.fillWidth: true }
                    Label { text: "Maximum controllers" }
                    TextField { id: platformMaxControllersField; Layout.fillWidth: true }
                    Label { text: "BigBox theme" }
                    TextField { id: platformBigBoxThemeField; Layout.fillWidth: true }
                    Label { text: "BigBox view" }
                    TextField { id: platformBigBoxViewField; Layout.fillWidth: true }
                    Label { text: "Video path" }
                    TextField { id: platformVideoPathField; Layout.fillWidth: true }
                    Label { text: "Videos folder" }
                    TextField { id: platformVideosFolderField; Layout.fillWidth: true }
                    Label { text: "Front images folder" }
                    TextField { id: platformFrontImagesFolderField; Layout.fillWidth: true }
                    Label { text: "Back images folder" }
                    TextField { id: platformBackImagesFolderField; Layout.fillWidth: true }
                    Label { text: "Clear logos folder" }
                    TextField { id: platformClearLogoImagesFolderField; Layout.fillWidth: true }
                    Label { text: "Fanart folder" }
                    TextField { id: platformFanartImagesFolderField; Layout.fillWidth: true }
                    Label { text: "Screenshots folder" }
                    TextField { id: platformScreenshotImagesFolderField; Layout.fillWidth: true }
                    Label { text: "Banners folder" }
                    TextField { id: platformBannerImagesFolderField; Layout.fillWidth: true }
                    Label { text: "Steam banners folder" }
                    TextField { id: platformSteamBannerImagesFolderField; Layout.fillWidth: true }
                    Label { text: "Manuals folder" }
                    TextField { id: platformManualsFolderField; Layout.fillWidth: true }
                    Label { text: "Music folder" }
                    TextField { id: platformMusicFolderField; Layout.fillWidth: true }
                    Label { text: "Android theme video" }
                    TextField { id: platformAndroidThemeVideoPathField; Layout.fillWidth: true }
                }
                Label { text: "Notes" }
                TextArea {
                    id: platformNotesField
                    Layout.fillWidth: true
                    Layout.preferredHeight: 110
                    wrapMode: TextEdit.Wrap
                }
                RowLayout {
                    CheckBox { id: platformHideBigBoxCheck; text: "Hide in BigBox" }
                    CheckBox { id: platformDisableAutoImportCheck; text: "Disable auto-import" }
                    Item { Layout.fillWidth: true }
                }
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 1
                    color: "#30363d"
                }
                Label { text: "Platform folders"; font.pixelSize: 18; font.bold: true }
                Label {
                    Layout.fillWidth: true
                    text: "These are lexical LaunchBox paths. Editing them does not create, move, or delete directories on this host."
                    wrapMode: Text.Wrap
                    color: "#7d8590"
                }
                Repeater {
                    model: platformFolderEditorModel
                    delegate: RowLayout {
                        required property int index
                        required property int sourceIndex
                        required property string mediaType
                        required property string folderPath
                        Layout.fillWidth: true
                        spacing: 8
                        TextField {
                            Layout.preferredWidth: 210
                            text: mediaType
                            placeholderText: "Media type"
                            onTextEdited: platformFolderEditorModel.setProperty(
                                              index, "mediaType", text)
                        }
                        TextField {
                            Layout.fillWidth: true
                            text: folderPath
                            placeholderText: "Stored LaunchBox folder path"
                            onTextEdited: platformFolderEditorModel.setProperty(
                                              index, "folderPath", text)
                        }
                        Button {
                            text: "Remove"
                            onClicked: platformFolderEditorModel.remove(index)
                        }
                    }
                }
                Button {
                    text: "Add Platform Folder"
                    onClicked: platformFolderEditorModel.append({
                        sourceIndex: -1,
                        mediaType: "",
                        folderPath: ""
                    })
                }
            }
        }
    }

    Dialog {
        id: addPlatformDialog
        anchors.centerIn: parent
        modal: true
        title: "Add Platform"
        standardButtons: Dialog.Save | Dialog.Cancel

        function prepare() {
            addPlatformName.text = ""
            addPlatformScrapeAs.text = ""
            open()
        }

        function smokeCreate(name, scrapeAs) {
            prepare()
            addPlatformName.text = name
            addPlatformScrapeAs.text = scrapeAs
            Qt.callLater(function() { addPlatformDialog.accept() })
        }

        onAccepted: controller.add_platform(addPlatformName.text,
                                            addPlatformScrapeAs.text)

        contentItem: ColumnLayout {
            spacing: 10
            Label { text: "Name" }
            TextField {
                id: addPlatformName
                Layout.preferredWidth: 440
                placeholderText: "Platform name"
            }
            Label { text: "Scrape as (optional)" }
            TextField {
                id: addPlatformScrapeAs
                Layout.fillWidth: true
                placeholderText: "Metadata platform name"
            }
            Label {
                Layout.preferredWidth: 440
                text: "Creates a portable platform XML document and LaunchBox-compatible media-folder records in one recoverable transaction. Stored LaunchBox paths keep their native backslash syntax; media directories are not created."
                wrapMode: Text.Wrap
                color: "#7d8590"
            }
        }
    }

    Dialog {
        id: deletePlatformConfirmation
        anchors.centerIn: parent
        modal: true
        title: "Delete " + platformName + "?"
        standardButtons: Dialog.Yes | Dialog.No
        property string platformName: ""

        function prepare(name) {
            platformName = name
            open()
        }

        function smokeDelete(name) {
            prepare(name)
            Qt.callLater(function() { deletePlatformConfirmation.accept() })
        }

        onAccepted: {
            const deleting = platformName
            if (window.selectedPlatform === deleting) {
                window.selectedPlatform = ""
                window.selectedNavigationKind = "all"
                window.selectedNavigationKey = ""
                window.selectedNavigationName = "All Games"
                controller.apply_filters(searchField.text, "")
            }
            controller.delete_platform(deleting)
        }

        contentItem: Label {
            width: 440
            text: "Deletion is refused while any game, emulator, playlist, navigation, controller, or frontend setting refers to this platform. Only its catalog records and empty platform XML are removed; media files and directories are never deleted."
            wrapMode: Text.Wrap
        }
    }

    Dialog {
        id: categoryEditor
        anchors.centerIn: parent
        modal: true
        title: createMode ? "Add Platform Category" : "Edit " + originalName
        standardButtons: Dialog.Save | Dialog.Cancel
        property bool createMode: true
        property string originalName: ""
        property var draft: null

        ListModel { id: categoryParentEditorModel }

        function storedText(value) {
            return value === null || value === undefined ? "" : value
        }

        function optionalText(value) {
            return value.trim().length > 0 ? value : null
        }

        function targetIndex(kind, key) {
            if (draft === null)
                return -1
            for (let index = 0; index < draft.available_parent_targets.length; ++index) {
                const target = draft.available_parent_targets[index]
                if (target.target_kind === kind && target.target_key === key)
                    return index
            }
            return -1
        }

        function loadDraft(serialized, creating) {
            if (serialized.length === 0)
                return false
            createMode = creating
            draft = JSON.parse(serialized)
            originalName = creating ? "" : draft.category.name
            categoryNameField.text = draft.category.name
            categoryNestedNameField.text = storedText(draft.category.nested_name)
            categorySortTitleField.text = storedText(draft.category.sort_title)
            categoryImageTypeField.text = storedText(draft.category.image_type)
            categoryVideoPathField.text = storedText(draft.category.video_path)
            categoryNotesField.text = storedText(draft.category.notes)
            categoryHideBigBoxCheck.checked = draft.category.hide_in_big_box
            categoryParentEditorModel.clear()
            for (let index = 0; index < draft.parents.length; ++index) {
                const parent = draft.parents[index]
                categoryParentEditorModel.append({
                    sourceIndex: parent.source_index === null ? -1 : parent.source_index,
                    targetKind: parent.target_kind,
                    targetKey: parent.target_key
                })
            }
            return true
        }

        function prepareCreate() {
            if (loadDraft(controller.new_category_edit_payload(), true))
                open()
        }

        function prepareEdit(name) {
            if (loadDraft(controller.category_edit_payload(name), false))
                open()
        }

        function selectParent(row, kind, key) {
            const index = targetIndex(kind, key)
            if (index < 0)
                return false
            const target = draft.available_parent_targets[index]
            categoryParentEditorModel.setProperty(row, "targetKind", target.target_kind)
            categoryParentEditorModel.setProperty(row, "targetKey", target.target_key)
            return true
        }

        function addRootPlacement() {
            categoryParentEditorModel.append({
                sourceIndex: -1,
                targetKind: "root",
                targetKey: ""
            })
        }

        function editPayload() {
            draft.category.name = categoryNameField.text
            draft.category.nested_name = optionalText(categoryNestedNameField.text)
            draft.category.sort_title = optionalText(categorySortTitleField.text)
            draft.category.image_type = optionalText(categoryImageTypeField.text)
            draft.category.video_path = optionalText(categoryVideoPathField.text)
            draft.category.notes = optionalText(categoryNotesField.text)
            draft.category.hide_in_big_box = categoryHideBigBoxCheck.checked
            const parents = []
            for (let index = 0; index < categoryParentEditorModel.count; ++index) {
                const parent = categoryParentEditorModel.get(index)
                parents.push({
                    source_index: parent.sourceIndex < 0 ? null : parent.sourceIndex,
                    target_kind: parent.targetKind,
                    target_key: parent.targetKey
                })
            }
            draft.parents = parents
            return JSON.stringify(draft)
        }

        function smokeCreate(name, parentKind, parentKey) {
            prepareCreate()
            categoryNameField.text = name
            if (!selectParent(0, parentKind, parentKey)) {
                console.error("CATEGORY_CRUD_SMOKE_PARENT_TARGET_MISSING")
                Qt.exit(10)
                return
            }
            Qt.callLater(function() { categoryEditor.accept() })
        }

        function smokeSave(name) {
            prepareEdit(name)
            categoryNestedNameField.text = "Portable"
            categorySortTitleField.text = "Collections, Portable"
            categoryImageTypeField.text = "Clear Logo"
            categoryVideoPathField.text = "Videos\\Portable Collections\\theme.mp4"
            categoryNotesField.text = "Edited through the real category dialog."
            categoryHideBigBoxCheck.checked = true
            addRootPlacement()
            Qt.callLater(function() { categoryEditor.accept() })
        }

        onAccepted: {
            const serialized = editPayload()
            if (createMode)
                controller.add_category(serialized)
            else
                controller.save_category(originalName, serialized)
        }

        contentItem: ScrollView {
            id: categoryEditorScroll
            implicitWidth: 680
            implicitHeight: Math.min(650, window.height - 160)
            contentWidth: availableWidth
            clip: true

            ColumnLayout {
                width: categoryEditorScroll.availableWidth
                spacing: 12

                Label {
                    Layout.fillWidth: true
                    text: "LaunchBox 13.27 exposes category identity as getter-only. Existing names stay fixed; hierarchy placements and the metadata below are editable."
                    wrapMode: Text.Wrap
                    color: "#d29922"
                }
                GridLayout {
                    Layout.fillWidth: true
                    columns: 2
                    columnSpacing: 12
                    rowSpacing: 8

                    Label { text: "Name" }
                    TextField {
                        id: categoryNameField
                        Layout.fillWidth: true
                        readOnly: !categoryEditor.createMode
                    }
                    Label { text: "Nested name" }
                    TextField { id: categoryNestedNameField; Layout.fillWidth: true }
                    Label { text: "Sort title" }
                    TextField { id: categorySortTitleField; Layout.fillWidth: true }
                    Label { text: "Image type" }
                    TextField { id: categoryImageTypeField; Layout.fillWidth: true }
                    Label { text: "Video path" }
                    TextField {
                        id: categoryVideoPathField
                        Layout.fillWidth: true
                        placeholderText: "Stored LaunchBox path"
                    }
                }
                Label { text: "Notes" }
                TextArea {
                    id: categoryNotesField
                    Layout.fillWidth: true
                    Layout.preferredHeight: 100
                    wrapMode: TextEdit.Wrap
                }
                CheckBox {
                    id: categoryHideBigBoxCheck
                    text: "Hide in BigBox"
                }
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 1
                    color: "#30363d"
                }
                Label { text: "Hierarchy placements"; font.pixelSize: 18; font.bold: true }
                Label {
                    Layout.fillWidth: true
                    text: "A category can appear at root or below any category, platform, or playlist. Cycles and duplicate placements are rejected."
                    wrapMode: Text.Wrap
                    color: "#7d8590"
                }
                Repeater {
                    model: categoryParentEditorModel
                    delegate: RowLayout {
                        required property int index
                        required property int sourceIndex
                        required property string targetKind
                        required property string targetKey
                        Layout.fillWidth: true
                        spacing: 8

                        ComboBox {
                            Layout.fillWidth: true
                            model: categoryEditor.draft === null
                                   ? [] : categoryEditor.draft.available_parent_targets
                            textRole: "label"
                            currentIndex: categoryEditor.targetIndex(targetKind, targetKey)
                            onActivated: function(selectedIndex) {
                                const target = model[selectedIndex]
                                categoryParentEditorModel.setProperty(
                                            index, "targetKind", target.target_kind)
                                categoryParentEditorModel.setProperty(
                                            index, "targetKey", target.target_key)
                            }
                        }
                        Button {
                            text: "Remove"
                            enabled: categoryParentEditorModel.count > 1
                            onClicked: categoryParentEditorModel.remove(index)
                        }
                    }
                }
                Button {
                    text: "Add Placement"
                    onClicked: categoryEditor.addRootPlacement()
                }
                Label {
                    Layout.fillWidth: true
                    text: "Save updates Data/Platforms.xml and Data/Parents.xml in one recoverable transaction with an exact backup of each file. Video paths remain lexical; no media is created, moved, or deleted."
                    wrapMode: Text.Wrap
                    color: "#7d8590"
                }
            }
        }
    }

    Dialog {
        id: deleteCategoryConfirmation
        anchors.centerIn: parent
        modal: true
        title: "Delete " + categoryName + "?"
        standardButtons: Dialog.Yes | Dialog.No
        property string categoryName: ""

        function prepare(name) {
            categoryName = name
            open()
        }

        function smokeDelete(name) {
            prepare(name)
            Qt.callLater(function() { deleteCategoryConfirmation.accept() })
        }

        onAccepted: {
            const deleting = categoryName
            if (window.selectedNavigationKind === "category"
                    && window.selectedNavigationKey === deleting) {
                window.selectedPlatform = ""
                window.selectedNavigationKind = "all"
                window.selectedNavigationKey = ""
                window.selectedNavigationName = "All Games"
                controller.apply_filters(searchField.text, "")
            }
            controller.delete_category(deleting)
        }

        contentItem: Label {
            width: 500
            text: "The category and all of its placements will be removed. Direct child categories, platforms, and playlists are detached to the root. No platforms, playlists, games, media files, or directories are deleted."
            wrapMode: Text.Wrap
        }
    }

    Dialog {
        id: playlistEditor
        anchors.centerIn: parent
        modal: true
        title: createMode ? "Add Playlist" : "Edit " + originalName
        standardButtons: Dialog.Save | Dialog.Cancel
        property bool createMode: true
        property string originalId: ""
        property string originalName: ""
        property var draft: null

        ListModel { id: playlistParentEditorModel }
        ListModel { id: playlistFilterEditorModel }
        ListModel { id: playlistGameEditorModel }

        function storedText(value) {
            return value === null || value === undefined ? "" : value
        }

        function optionalText(value) {
            return value.trim().length > 0 ? value : null
        }

        function targetIndex(kind, key) {
            if (draft === null)
                return -1
            for (let index = 0; index < draft.available_parent_targets.length; ++index) {
                const target = draft.available_parent_targets[index]
                if (target.target_kind === kind && target.target_key === key)
                    return index
            }
            return -1
        }

        function gameIndex(gameId) {
            if (draft === null)
                return -1
            for (let index = 0; index < draft.available_games.length; ++index) {
                if (draft.available_games[index].game_id === gameId)
                    return index
            }
            return -1
        }

        function loadDraft(serialized, creating) {
            if (serialized.length === 0)
                return false
            createMode = creating
            draft = JSON.parse(serialized)
            originalId = creating ? "" : draft.playlist.id
            originalName = creating ? "" : draft.playlist.name
            playlistIdField.text = draft.playlist.id
            playlistNameField.text = draft.playlist.name
            playlistNestedNameField.text = storedText(draft.playlist.nested_name)
            playlistSortTitleField.text = storedText(draft.playlist.sort_title)
            playlistSortByField.text = storedText(draft.playlist.sort_by)
            playlistImageTypeField.text = storedText(draft.playlist.image_type)
            playlistCategoryField.text = storedText(draft.playlist.category)
            playlistLastGameField.text = storedText(draft.playlist.last_game_id)
            playlistVideoPathField.text = storedText(draft.playlist.video_path)
            playlistBigBoxViewField.text = storedText(draft.playlist.big_box_view)
            playlistBigBoxThemeField.text = storedText(draft.playlist.big_box_theme)
            playlistNotesField.text = storedText(draft.playlist.notes)
            playlistHideBigBoxCheck.checked = draft.playlist.hide_in_big_box
            playlistIncludePlatformsCheck.checked = draft.playlist.include_with_platforms
            playlistAutoPopulateCheck.checked = draft.playlist.auto_populate
            playlistAutogeneratedCheck.checked = draft.playlist.is_autogenerated
            playlistParentEditorModel.clear()
            for (let index = 0; index < draft.parents.length; ++index) {
                const parent = draft.parents[index]
                playlistParentEditorModel.append({
                    sourceIndex: parent.source_index === null ? -1 : parent.source_index,
                    targetKind: parent.target_kind,
                    targetKey: parent.target_key
                })
            }
            playlistFilterEditorModel.clear()
            for (let index = 0; index < draft.filters.length; ++index) {
                const filter = draft.filters[index]
                playlistFilterEditorModel.append({
                    sourceIndex: filter.source_index === null ? -1 : filter.source_index,
                    fieldKey: filter.field_key,
                    comparisonTypeKey: filter.comparison_type_key,
                    filterValue: filter.value
                })
            }
            playlistGameEditorModel.clear()
            for (let index = 0; index < draft.games.length; ++index) {
                const game = draft.games[index]
                playlistGameEditorModel.append({
                    sourceIndex: game.source_index === null ? -1 : game.source_index,
                    gameId: game.game_id,
                    gameTitle: game.game_title,
                    gamePlatform: game.game_platform,
                    gameFileName: game.game_file_name,
                    launchboxDbId: game.launchbox_db_id === null ? -1 : game.launchbox_db_id,
                    manualOrder: game.manual_order
                })
            }
            return true
        }

        function prepareCreate() {
            if (loadDraft(controller.new_playlist_edit_payload(), true))
                open()
        }

        function prepareEdit(playlistId) {
            if (loadDraft(controller.playlist_edit_payload(playlistId), false))
                open()
        }

        function addRootPlacement() {
            playlistParentEditorModel.append({
                sourceIndex: -1,
                targetKind: "root",
                targetKey: ""
            })
        }

        function addFilter() {
            playlistFilterEditorModel.append({
                sourceIndex: -1,
                fieldKey: "Platform",
                comparisonTypeKey: "EqualTo",
                filterValue: ""
            })
        }

        function addGame() {
            if (draft === null)
                return
            for (let index = 0; index < draft.available_games.length; ++index) {
                const available = draft.available_games[index]
                let duplicate = false
                for (let row = 0; row < playlistGameEditorModel.count; ++row) {
                    if (playlistGameEditorModel.get(row).gameId === available.game_id) {
                        duplicate = true
                        break
                    }
                }
                if (!duplicate) {
                    playlistGameEditorModel.append({
                        sourceIndex: -1,
                        gameId: available.game_id,
                        gameTitle: available.title,
                        gamePlatform: available.platform,
                        gameFileName: "",
                        launchboxDbId: -1,
                        manualOrder: playlistGameEditorModel.count + 1
                    })
                    return
                }
            }
        }

        function editPayload() {
            draft.playlist.id = playlistIdField.text
            draft.playlist.name = playlistNameField.text
            draft.playlist.nested_name = optionalText(playlistNestedNameField.text)
            draft.playlist.sort_title = optionalText(playlistSortTitleField.text)
            draft.playlist.sort_by = optionalText(playlistSortByField.text)
            draft.playlist.image_type = optionalText(playlistImageTypeField.text)
            draft.playlist.category = optionalText(playlistCategoryField.text)
            draft.playlist.last_game_id = optionalText(playlistLastGameField.text)
            draft.playlist.video_path = optionalText(playlistVideoPathField.text)
            draft.playlist.big_box_view = optionalText(playlistBigBoxViewField.text)
            draft.playlist.big_box_theme = optionalText(playlistBigBoxThemeField.text)
            draft.playlist.notes = optionalText(playlistNotesField.text)
            draft.playlist.hide_in_big_box = playlistHideBigBoxCheck.checked
            draft.playlist.include_with_platforms = playlistIncludePlatformsCheck.checked
            draft.playlist.auto_populate = playlistAutoPopulateCheck.checked
            draft.playlist.is_autogenerated = playlistAutogeneratedCheck.checked
            const parents = []
            for (let index = 0; index < playlistParentEditorModel.count; ++index) {
                const parent = playlistParentEditorModel.get(index)
                parents.push({
                    source_index: parent.sourceIndex < 0 ? null : parent.sourceIndex,
                    target_kind: parent.targetKind,
                    target_key: parent.targetKey
                })
            }
            draft.parents = parents
            const filters = []
            for (let index = 0; index < playlistFilterEditorModel.count; ++index) {
                const filter = playlistFilterEditorModel.get(index)
                filters.push({
                    source_index: filter.sourceIndex < 0 ? null : filter.sourceIndex,
                    field_key: filter.fieldKey,
                    comparison_type_key: filter.comparisonTypeKey,
                    value: filter.filterValue
                })
            }
            draft.filters = filters
            const games = []
            for (let index = 0; index < playlistGameEditorModel.count; ++index) {
                const game = playlistGameEditorModel.get(index)
                games.push({
                    source_index: game.sourceIndex < 0 ? null : game.sourceIndex,
                    game_id: game.gameId,
                    game_title: game.gameTitle,
                    game_platform: game.gamePlatform,
                    game_file_name: game.gameFileName,
                    launchbox_db_id: game.launchboxDbId < 0 ? null : game.launchboxDbId,
                    manual_order: game.manualOrder
                })
            }
            draft.games = games
            return JSON.stringify(draft)
        }

        function selectParent(row, kind, key) {
            const index = targetIndex(kind, key)
            if (index < 0)
                return false
            const target = draft.available_parent_targets[index]
            playlistParentEditorModel.setProperty(row, "targetKind", target.target_kind)
            playlistParentEditorModel.setProperty(row, "targetKey", target.target_key)
            return true
        }

        function addGameById(gameId) {
            const index = gameIndex(gameId)
            if (index < 0)
                return false
            const game = draft.available_games[index]
            playlistGameEditorModel.append({
                sourceIndex: -1,
                gameId: game.game_id,
                gameTitle: game.title,
                gamePlatform: game.platform,
                gameFileName: "",
                launchboxDbId: -1,
                manualOrder: playlistGameEditorModel.count + 1
            })
            return true
        }

        function smokeCreateParent(name) {
            prepareCreate()
            window.playlistCrudParentId = draft.playlist.id
            playlistNameField.text = name
            playlistNestedNameField.text = "Portable Queue"
            playlistVideoPathField.text = "Videos\\Playlists\\portable.mp4"
            if (!addGameById("fixture-racer")) {
                console.error("PLAYLIST_CRUD_SMOKE_GAME_TARGET_MISSING")
                Qt.exit(11)
                return
            }
            Qt.callLater(function() { playlistEditor.accept() })
        }

        function smokeSaveParent(playlistId) {
            prepareEdit(playlistId)
            playlistNestedNameField.text = "Portable Favorites"
            playlistSortTitleField.text = "Favorites, Portable"
            playlistSortByField.text = "Title"
            playlistImageTypeField.text = "Clear Logo"
            playlistCategoryField.text = "Arcade"
            playlistLastGameField.text = "fixture-adventure"
            playlistVideoPathField.text = "Videos\\Portable Favorites\\theme.mp4"
            playlistBigBoxViewField.text = "TextGamesView"
            playlistBigBoxThemeField.text = "Default"
            playlistNotesField.text = "Edited through the real playlist dialog."
            playlistHideBigBoxCheck.checked = true
            playlistIncludePlatformsCheck.checked = true
            playlistAutoPopulateCheck.checked = true
            playlistFilterEditorModel.clear()
            playlistFilterEditorModel.append({
                sourceIndex: -1,
                fieldKey: "Favorite",
                comparisonTypeKey: "IsTrue",
                filterValue: ""
            })
            Qt.callLater(function() { playlistEditor.accept() })
        }

        function smokeCreateChild(name, parentId) {
            prepareCreate()
            window.playlistCrudChildId = draft.playlist.id
            playlistNameField.text = name
            playlistNestedNameField.text = "Nested Queue"
            if (!selectParent(0, "playlist", parentId)) {
                console.error("PLAYLIST_CRUD_SMOKE_PARENT_TARGET_MISSING")
                Qt.exit(11)
                return
            }
            Qt.callLater(function() { playlistEditor.accept() })
        }

        onAccepted: {
            const serialized = editPayload()
            if (createMode)
                controller.add_playlist(serialized)
            else
                controller.save_playlist(originalId, serialized)
        }

        contentItem: ScrollView {
            id: playlistEditorScroll
            implicitWidth: 760
            implicitHeight: Math.min(680, window.height - 120)
            contentWidth: availableWidth
            clip: true

            ColumnLayout {
                width: playlistEditorScroll.availableWidth
                spacing: 12

                Label {
                    Layout.fillWidth: true
                    text: "LaunchBox 13.27 treats PlaylistId and the unique Name as identity. Existing values stay fixed; Nested Name is the editable display label."
                    wrapMode: Text.Wrap
                    color: "#d29922"
                }
                GridLayout {
                    Layout.fillWidth: true
                    columns: 2
                    columnSpacing: 12
                    rowSpacing: 8

                    Label { text: "Playlist ID" }
                    TextField { id: playlistIdField; Layout.fillWidth: true; readOnly: true }
                    Label { text: "Unique name" }
                    TextField {
                        id: playlistNameField
                        Layout.fillWidth: true
                        readOnly: !playlistEditor.createMode
                    }
                    Label { text: "Nested name" }
                    TextField { id: playlistNestedNameField; Layout.fillWidth: true }
                    Label { text: "Sort title" }
                    TextField { id: playlistSortTitleField; Layout.fillWidth: true }
                    Label { text: "Sort games by" }
                    TextField { id: playlistSortByField; Layout.fillWidth: true }
                    Label { text: "Image type" }
                    TextField { id: playlistImageTypeField; Layout.fillWidth: true }
                    Label { text: "Category metadata" }
                    TextField { id: playlistCategoryField; Layout.fillWidth: true }
                    Label { text: "Last game ID" }
                    TextField { id: playlistLastGameField; Layout.fillWidth: true }
                    Label { text: "Video path" }
                    TextField {
                        id: playlistVideoPathField
                        Layout.fillWidth: true
                        placeholderText: "Stored LaunchBox path"
                    }
                    Label { text: "BigBox view" }
                    TextField { id: playlistBigBoxViewField; Layout.fillWidth: true }
                    Label { text: "BigBox theme" }
                    TextField { id: playlistBigBoxThemeField; Layout.fillWidth: true }
                }
                Label { text: "Notes" }
                TextArea {
                    id: playlistNotesField
                    Layout.fillWidth: true
                    Layout.preferredHeight: 90
                    wrapMode: TextEdit.Wrap
                }
                RowLayout {
                    CheckBox { id: playlistHideBigBoxCheck; text: "Hide in BigBox" }
                    CheckBox {
                        id: playlistIncludePlatformsCheck
                        text: "Include with platforms"
                    }
                    CheckBox {
                        id: playlistAutogeneratedCheck
                        text: "LaunchBox-generated"
                        enabled: playlistEditor.createMode
                    }
                }
                CheckBox {
                    id: playlistAutoPopulateCheck
                    text: "Auto-populate from filter rules"
                }
                Rectangle { Layout.fillWidth: true; Layout.preferredHeight: 1; color: "#30363d" }
                Label { text: "Hierarchy placements"; font.pixelSize: 18; font.bold: true }
                Label {
                    Layout.fillWidth: true
                    text: "A playlist needs at least one location and can appear at root or below a category, platform, or another playlist. Duplicate placements and cycles are rejected."
                    wrapMode: Text.Wrap
                    color: "#7d8590"
                }
                Repeater {
                    model: playlistParentEditorModel
                    delegate: RowLayout {
                        required property int index
                        required property int sourceIndex
                        required property string targetKind
                        required property string targetKey
                        Layout.fillWidth: true

                        ComboBox {
                            Layout.fillWidth: true
                            model: playlistEditor.draft === null
                                   ? [] : playlistEditor.draft.available_parent_targets
                            textRole: "label"
                            currentIndex: playlistEditor.targetIndex(targetKind, targetKey)
                            onActivated: function(selectedIndex) {
                                const target = model[selectedIndex]
                                playlistParentEditorModel.setProperty(
                                            index, "targetKind", target.target_kind)
                                playlistParentEditorModel.setProperty(
                                            index, "targetKey", target.target_key)
                            }
                        }
                        Button {
                            text: "Remove"
                            enabled: playlistParentEditorModel.count > 1
                            onClicked: playlistParentEditorModel.remove(index)
                        }
                    }
                }
                Button { text: "Add Placement"; onClicked: playlistEditor.addRootPlacement() }
                Rectangle { Layout.fillWidth: true; Layout.preferredHeight: 1; color: "#30363d" }
                Label {
                    text: "Auto-populate filters"
                    font.pixelSize: 18
                    font.bold: true
                    visible: playlistAutoPopulateCheck.checked
                }
                Label {
                    Layout.fillWidth: true
                    visible: playlistAutoPopulateCheck.checked
                    text: "Rules sharing a FieldKey are ORed; different fields are ANDed, matching the recovered LaunchBox playlist behavior."
                    wrapMode: Text.Wrap
                    color: "#7d8590"
                }
                Repeater {
                    model: playlistFilterEditorModel
                    delegate: RowLayout {
                        required property int index
                        required property int sourceIndex
                        required property string fieldKey
                        required property string comparisonTypeKey
                        required property string filterValue
                        Layout.fillWidth: true
                        visible: playlistAutoPopulateCheck.checked

                        TextField {
                            Layout.preferredWidth: 170
                            text: fieldKey
                            placeholderText: "FieldKey"
                            onTextEdited: playlistFilterEditorModel.setProperty(index, "fieldKey", text)
                        }
                        TextField {
                            Layout.preferredWidth: 170
                            text: comparisonTypeKey
                            placeholderText: "Comparison"
                            onTextEdited: playlistFilterEditorModel.setProperty(
                                              index, "comparisonTypeKey", text)
                        }
                        TextField {
                            Layout.fillWidth: true
                            text: filterValue
                            placeholderText: "Value"
                            onTextEdited: playlistFilterEditorModel.setProperty(
                                              index, "filterValue", text)
                        }
                        Button { text: "Remove"; onClicked: playlistFilterEditorModel.remove(index) }
                    }
                }
                Button {
                    text: "Add Filter"
                    visible: playlistAutoPopulateCheck.checked
                    onClicked: playlistEditor.addFilter()
                }
                Label {
                    text: "Manual games"
                    font.pixelSize: 18
                    font.bold: true
                    visible: !playlistAutoPopulateCheck.checked
                }
                Repeater {
                    model: playlistGameEditorModel
                    delegate: RowLayout {
                        required property int index
                        required property int sourceIndex
                        required property string gameId
                        required property string gameTitle
                        required property string gamePlatform
                        required property int manualOrder
                        Layout.fillWidth: true
                        visible: !playlistAutoPopulateCheck.checked

                        ComboBox {
                            Layout.fillWidth: true
                            model: playlistEditor.draft === null
                                   ? [] : playlistEditor.draft.available_games
                            textRole: "title"
                            currentIndex: playlistEditor.gameIndex(gameId)
                            displayText: currentIndex < 0
                                         ? gameTitle + " — " + gamePlatform
                                         : model[currentIndex].title + " — "
                                           + model[currentIndex].platform
                            onActivated: function(selectedIndex) {
                                const game = model[selectedIndex]
                                playlistGameEditorModel.setProperty(index, "gameId", game.game_id)
                                playlistGameEditorModel.setProperty(index, "gameTitle", game.title)
                                playlistGameEditorModel.setProperty(
                                            index, "gamePlatform", game.platform)
                            }
                        }
                        SpinBox {
                            from: -1000000
                            to: 1000000
                            value: manualOrder
                            editable: true
                            onValueModified: playlistGameEditorModel.setProperty(
                                                 index, "manualOrder", value)
                        }
                        Button { text: "Remove"; onClicked: playlistGameEditorModel.remove(index) }
                    }
                }
                Button {
                    text: "Add Game"
                    visible: !playlistAutoPopulateCheck.checked
                    enabled: playlistEditor.draft !== null
                             && playlistGameEditorModel.count
                                < playlistEditor.draft.available_games.length
                    onClicked: playlistEditor.addGame()
                }
                Label {
                    Layout.fillWidth: true
                    text: "Save writes the playlist XML and Data/Parents.xml as one recoverable transaction. Stored paths remain lexical LaunchBox strings. Editing membership never edits game records or media."
                    wrapMode: Text.Wrap
                    color: "#7d8590"
                }
            }
        }
    }

    Dialog {
        id: deletePlaylistConfirmation
        anchors.centerIn: parent
        modal: true
        title: "Delete " + playlistName + "?"
        standardButtons: Dialog.Yes | Dialog.No
        property string playlistId: ""
        property string playlistName: ""

        function prepare(id, name) {
            playlistId = id
            playlistName = name
            open()
        }

        function smokeDelete(id, name) {
            prepare(id, name)
            Qt.callLater(function() { deletePlaylistConfirmation.accept() })
        }

        onAccepted: {
            const deleting = playlistId
            if (window.selectedNavigationKind === "playlist"
                    && window.selectedNavigationKey === deleting) {
                window.selectedPlatform = ""
                window.selectedNavigationKind = "all"
                window.selectedNavigationKey = ""
                window.selectedNavigationName = "All Games"
                controller.apply_filters(searchField.text, "")
            }
            controller.delete_playlist(deleting)
        }

        contentItem: Label {
            width: 540
            text: "This permanently deletes ALL INSTANCES of the playlist. To remove only one location, edit its hierarchy placements instead. Direct children are detached to root and matching list-cache rows are removed. No games, game XML, media files, or media directories are deleted."
            wrapMode: Text.Wrap
        }
    }

    Dialog {
        id: romImportDialog
        anchors.centerIn: parent
        modal: true
        closePolicy: Popup.CloseOnEscape
        title: "Import ROM Files"
        standardButtons: Dialog.NoButton

        property int page: 0
        property bool awaitingPreview: false
        property var previewRequest: null

        function addLocation(path, kind) {
            if (path.length === 0)
                return
            for (let index = 0; index < importLocations.count; ++index) {
                if (importLocations.get(index).path === path)
                    return
            }
            importLocations.append({ "path": path, "kind": kind })
        }

        function selectPlatform(platformName) {
            let selectedIndex = 0
            for (let index = 0; index < controller.platform_entry_count; ++index) {
                if (window.platformName(index) === platformName) {
                    selectedIndex = index
                    break
                }
            }
            importPlatform.currentIndex = selectedIndex
        }

        function selectEmulator(emulatorId) {
            let selectedIndex = 0
            for (let index = 0; index < controller.emulator_entry_count(); ++index) {
                if (controller.emulator_id_at(index) === emulatorId) {
                    selectedIndex = index
                    break
                }
            }
            importEmulator.currentIndex = selectedIndex
        }

        function prepare() {
            page = 0
            awaitingPreview = false
            previewRequest = null
            importLocations.clear()
            importPreviewRows.clear()
            recursiveFolders.checked = true
            folderTitles.checked = false
            duplicateFiles.checked = false
            copySameNameFiles.checked = true
            copyToSubfolders.checked = false
            combineDiscSets.checked = true
            combineMatchingTitles.checked = true
            searchLocalMetadata.checked = true
            lookForPdfManuals.checked = true
            importFilePolicy.currentIndex = 1
            extensionFilter.text = ""
            selectPlatform(window.selectedPlatform)
            selectEmulator("")
            controller.clear_rom_import_preview()
            open()
        }

        function smokePrepare(paths, platformName) {
            prepare()
            for (let index = 0; index < paths.length; ++index)
                addLocation(paths[index], "file")
            selectPlatform(platformName)
            selectEmulator("fixture-emulator")
            importFilePolicy.currentIndex = 1
            copyToSubfolders.checked = true
            page = 1
            requestPreview()
        }

        function filePolicy() {
            if (importFilePolicy.currentIndex === 1)
                return "copy"
            if (importFilePolicy.currentIndex === 2)
                return "move"
            return "leave"
        }

        function requestObject() {
            const locations = []
            for (let index = 0; index < importLocations.count; ++index) {
                const location = importLocations.get(index)
                locations.push({ "path": location.path, "kind": location.kind })
            }
            const extensions = extensionFilter.text.split(",")
                  .map(function(value) { return value.trim() })
                  .filter(function(value) { return value.length > 0 })
            const emulatorId = controller.emulator_id_at(importEmulator.currentIndex)
            return {
                "platform": window.platformName(importPlatform.currentIndex),
                "locations": locations,
                "recursive": recursiveFolders.checked,
                "use_folder_names": folderTitles.checked,
                "file_policy": filePolicy(),
                "duplicate_policy": duplicateFiles.checked ? "import" : "skip",
                "extensions": extensions,
                "copy_files_with_same_name": copySameNameFiles.checked,
                "copy_to_subfolders": copyToSubfolders.checked,
                "combine_disc_sets": combineDiscSets.checked,
                "combine_matching_titles": combineMatchingTitles.checked,
                "search_local_metadata": searchLocalMetadata.checked,
                "look_for_pdf_manuals": lookForPdfManuals.checked,
                "emulator_id": emulatorId.length === 0 ? null : emulatorId
            }
        }

        function companionFileCount(row) {
            let count = row.same_name_files.length
            for (let index = 0; index < row.additional_roms.length; ++index)
                count += row.additional_roms[index].same_name_files.length
            return count
        }

        function metadataChoiceCount(candidates) {
            return candidates.count === undefined ? candidates.length : candidates.count
        }

        function metadataChoiceAt(candidates, index) {
            return candidates.get === undefined ? candidates[index] : candidates.get(index)
        }

        function metadataChoiceLabels(candidates) {
            const labels = ["Do not apply metadata"]
            for (let index = 0; index < metadataChoiceCount(candidates); ++index) {
                const candidate = metadataChoiceAt(candidates, index)
                let details = "Database ID " + candidate.database_id
                if (candidate.release_year !== null)
                    details += ", " + candidate.release_year
                if (candidate.developer !== null)
                    details += ", " + candidate.developer
                labels.push(candidate.title + " — " + details)
            }
            return labels
        }

        function metadataChoiceIndex(candidates, databaseId) {
            for (let index = 0; index < metadataChoiceCount(candidates); ++index) {
                if (metadataChoiceAt(candidates, index).database_id === databaseId)
                    return index + 1
            }
            return 0
        }

        function selectMetadataChoice(rowIndex, candidates, choiceIndex) {
            if (choiceIndex === 0) {
                importPreviewRows.setProperty(rowIndex,
                                              "selectedMetadataDatabaseId", 0)
                return
            }
            const candidate = metadataChoiceAt(candidates, choiceIndex - 1)
            importPreviewRows.setProperty(rowIndex,
                                          "selectedMetadataDatabaseId",
                                          candidate.database_id)
            importPreviewRows.setProperty(rowIndex, "title", candidate.title)
        }

        function requestPreview() {
            if (importLocations.count === 0 || importPlatform.currentIndex < 0)
                return
            awaitingPreview = true
            importPreviewRows.clear()
            controller.preview_rom_import(JSON.stringify(requestObject()))
        }

        function loadPreview() {
            if (controller.import_preview_json.length === 0)
                return false
            const preview = JSON.parse(controller.import_preview_json)
            previewRequest = preview.request
            importPreviewRows.clear()
            for (let index = 0; index < preview.rows.length; ++index) {
                const row = preview.rows[index]
                importPreviewRows.append({
                    "sourcePath": row.source_path,
                    "title": row.title,
                    "extension": row.extension,
                    "destinationPath": row.destination_path === null
                                       ? "" : row.destination_path,
                    "applicationPath": row.application_path,
                    "rowState": row.state,
                    "included": row.included,
                    "message": row.message,
                    "romFileCount": 1 + row.additional_roms.length,
                    "companionFileCount": companionFileCount(row),
                    "metadataCandidateCount": row.metadata_candidate_count,
                    "metadataMatchKind": row.metadata_match_kind === null
                                         ? "" : row.metadata_match_kind,
                    "metadataCandidates": row.metadata_candidates,
                    "selectedMetadataDatabaseId": row.metadata === null
                                                  ? 0 : row.metadata.database_id,
                    "manualCandidateCount": row.manual_candidate_count,
                    "manualSourcePath": row.manual === null
                                        ? "" : row.manual.source_path,
                    "manualStoredPath": row.manual === null
                                        ? "" : row.manual.stored_path
                })
            }
            awaitingPreview = false
            page = 2
            return true
        }

        function submitPreview() {
            if (previewRequest === null || importPreviewRows.count === 0)
                return
            const rows = []
            for (let index = 0; index < importPreviewRows.count; ++index) {
                const row = importPreviewRows.get(index)
                rows.push({
                    "source_path": row.sourcePath,
                    "title": row.title,
                    "included": row.included,
                    "metadata_database_id": row.selectedMetadataDatabaseId === 0
                                            ? null
                                            : row.selectedMetadataDatabaseId
                })
            }
            controller.import_roms(JSON.stringify({
                "request": previewRequest,
                "rows": rows
            }))
            close()
        }

        function smokeSubmitPreview() {
            if (previewRequest === null && !loadPreview())
                return
            let selectedRecoveredCandidate = false
            for (let rowIndex = 0; rowIndex < importPreviewRows.count; ++rowIndex) {
                const row = importPreviewRows.get(rowIndex)
                if (metadataChoiceCount(row.metadataCandidates) < 2)
                    continue
                for (let candidateIndex = 0;
                     candidateIndex < metadataChoiceCount(row.metadataCandidates);
                     ++candidateIndex) {
                    const candidate = metadataChoiceAt(row.metadataCandidates,
                                                       candidateIndex)
                    if (candidate.database_id !== 4242)
                        continue
                    selectMetadataChoice(rowIndex, row.metadataCandidates,
                                         candidateIndex + 1)
                    selectedRecoveredCandidate = true
                    break
                }
            }
            if (!selectedRecoveredCandidate) {
                console.error("Import smoke did not expose the expected ambiguous metadata choices")
                Qt.exit(1)
                return
            }
            submitPreview()
        }

        onClosed: {
            awaitingPreview = false
            controller.clear_rom_import_preview()
        }

        Timer {
            interval: 25
            repeat: true
            running: romImportDialog.visible && romImportDialog.awaitingPreview
                     && !controller.import_scanning
            onTriggered: {
                if (controller.import_preview_json.length > 0)
                    romImportDialog.loadPreview()
                else
                    romImportDialog.awaitingPreview = false
            }
        }

        ListModel { id: importLocations }
        ListModel { id: importPreviewRows }

        contentItem: ColumnLayout {
            implicitWidth: 820
            implicitHeight: 610
            spacing: 10

            Label {
                Layout.fillWidth: true
                text: romImportDialog.page === 0
                      ? "1 of 3 — Choose files and folders"
                      : romImportDialog.page === 1
                        ? "2 of 3 — Platform and file handling"
                        : "3 of 3 — Review and edit"
                color: "#7fbfff"
                font.bold: true
            }

            StackLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                currentIndex: romImportDialog.page

                ColumnLayout {
                    spacing: 8
                    RowLayout {
                        Button {
                            text: "Add Files…"
                            onClicked: importFileDialog.open()
                        }
                        Button {
                            text: "Add Folder…"
                            onClicked: importFolderDialog.open()
                        }
                        Button {
                            text: "Clear"
                            enabled: importLocations.count > 0
                            onClicked: importLocations.clear()
                        }
                        Item { Layout.fillWidth: true }
                        CheckBox {
                            id: recursiveFolders
                            text: "Include subfolders"
                        }
                    }
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        color: "#101318"
                        border.color: "#30363d"
                        ListView {
                            anchors.fill: parent
                            anchors.margins: 4
                            clip: true
                            model: importLocations
                            delegate: RowLayout {
                                required property int index
                                required property string path
                                required property string kind
                                width: ListView.view.width
                                Label {
                                    text: kind === "folder" ? "Folder" : "File"
                                    color: "#7fbfff"
                                    Layout.preferredWidth: 62
                                }
                                Label {
                                    Layout.fillWidth: true
                                    text: path
                                    elide: Text.ElideMiddle
                                }
                                ToolButton {
                                    text: "Remove"
                                    onClicked: importLocations.remove(index)
                                }
                            }
                        }
                    }
                    Label {
                        Layout.fillWidth: true
                        text: "Native host paths stay native in the picker. The importer stores library-relative LaunchBox paths, or reverses an explicit Windows drive/UNC mapping when applicable."
                        wrapMode: Text.Wrap
                        color: "#7d8590"
                    }
                }

                ColumnLayout {
                    spacing: 10
                    Label { text: "Platform" }
                    ComboBox {
                        id: importPlatform
                        Layout.fillWidth: true
                        model: controller.platform_entry_count
                        displayText: currentIndex >= 0
                                     ? window.platformName(currentIndex) : ""
                        delegate: ItemDelegate {
                            required property int index
                            width: importPlatform.width
                            text: window.platformName(index)
                        }
                    }
                    Label { text: "Emulator" }
                    ComboBox {
                        id: importEmulator
                        Layout.fillWidth: true
                        model: {
                            const revision = controller.emulator_revision
                            return controller.emulator_entry_count()
                        }
                        displayText: currentIndex >= 0
                                     ? controller.emulator_title_at(currentIndex) : ""
                        delegate: ItemDelegate {
                            required property int index
                            width: importEmulator.width
                            text: controller.emulator_title_at(index)
                        }
                    }
                    Label {
                        Layout.fillWidth: true
                        text: "Platform default leaves the Emulator field absent. Direct launch writes LaunchBox's explicit no-emulator sentinel; a named emulator pins its configured ID."
                        wrapMode: Text.Wrap
                        color: "#7d8590"
                    }
                    Label { text: "What should happen to the selected files?" }
                    ComboBox {
                        id: importFilePolicy
                        Layout.fillWidth: true
                        model: [
                            "Use files in their current locations",
                            "Copy files into the LaunchBox Games folder",
                            "Move files into the LaunchBox Games folder"
                        ]
                    }
                    CheckBox {
                        id: folderTitles
                        text: "Use the containing folder name as the game title"
                    }
                    CheckBox {
                        id: duplicateFiles
                        text: "Import files already referenced by another game"
                    }
                    CheckBox {
                        id: copySameNameFiles
                        text: "Also copy/move all files with the same name but different file extensions (recommended)"
                        visible: importFilePolicy.currentIndex !== 0
                    }
                    CheckBox {
                        id: copyToSubfolders
                        text: "Copy/move files into subfolders named with the game's title and year"
                        visible: importFilePolicy.currentIndex !== 0
                    }
                    CheckBox {
                        id: combineDiscSets
                        text: "Combine complete (Disc N) sets into one game"
                    }
                    CheckBox {
                        id: combineMatchingTitles
                        text: "Combine ROMs with matching titles into one game"
                    }
                    CheckBox {
                        id: searchLocalMetadata
                        text: "Search for game information in the local metadata database (recommended)"
                    }
                    CheckBox {
                        id: lookForPdfManuals
                        text: "Look for PDF files for use as the game manual"
                    }
                    Label { text: "File extensions (optional, comma-separated)" }
                    TextField {
                        id: extensionFilter
                        Layout.fillWidth: true
                        placeholderText: "zip, 7z, cue, chd, iso, rom"
                    }
                    Label {
                        Layout.fillWidth: true
                        text: importFilePolicy.currentIndex === 2
                              ? "Move first copies every file into the recoverable library transaction. Original files are removed only after the XML commit and a byte-for-byte revision check."
                              : importFilePolicy.currentIndex === 1
                                ? "ROM copies and platform XML are committed under one durable recovery manifest. Existing destination files are never overwritten."
                                : "Files outside the library remain host-specific unless a configured Windows path mapping can be reversed."
                        wrapMode: Text.Wrap
                        color: "#7d8590"
                    }
                    Item { Layout.fillHeight: true }
                    ProgressBar {
                        Layout.fillWidth: true
                        indeterminate: true
                        visible: controller.import_scanning
                    }
                }

                ColumnLayout {
                    spacing: 8
                    RowLayout {
                        Layout.fillWidth: true
                        Label {
                            Layout.fillWidth: true
                            text: importPreviewRows.count + " planned game(s)"
                            font.bold: true
                        }
                        Label {
                            text: "Titles are editable before import"
                            color: "#7d8590"
                        }
                    }
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        color: "#101318"
                        border.color: "#30363d"
                        ListView {
                            anchors.fill: parent
                            anchors.margins: 4
                            spacing: 4
                            clip: true
                            model: importPreviewRows
                            delegate: Rectangle {
                                id: importPreviewDelegate
                                required property int index
                                required property string sourcePath
                                required property string title
                                required property string extension
                                required property string destinationPath
                                required property string applicationPath
                                required property string rowState
                                required property bool included
                                required property string message
                                required property int romFileCount
                                required property int companionFileCount
                                required property int metadataCandidateCount
                                required property string metadataMatchKind
                                required property var metadataCandidates
                                required property double selectedMetadataDatabaseId
                                required property int manualCandidateCount
                                required property string manualSourcePath
                                required property string manualStoredPath
                                width: ListView.view.width
                                height: 92
                                        + (manualSourcePath.length > 0 ? 18 : 0)
                                        + (metadataCandidateCount > 1 ? 44 : 0)
                                color: index % 2 === 0 ? "#171b22" : "#14181e"
                                RowLayout {
                                    anchors.fill: parent
                                    anchors.margins: 6
                                    CheckBox {
                                        checked: importPreviewDelegate.included
                                        enabled: importPreviewDelegate.rowState === "ready"
                                                 || importPreviewDelegate.rowState === "duplicate"
                                        onToggled: importPreviewRows.setProperty(
                                                       importPreviewDelegate.index,
                                                       "included", checked)
                                    }
                                    ColumnLayout {
                                        Layout.fillWidth: true
                                        spacing: 2
                                        TextField {
                                            Layout.fillWidth: true
                                            text: importPreviewDelegate.title
                                            enabled: importPreviewDelegate.rowState === "ready"
                                                     || importPreviewDelegate.rowState === "duplicate"
                                            onEditingFinished: importPreviewRows.setProperty(
                                                                   importPreviewDelegate.index,
                                                                   "title", text)
                                        }
                                        ComboBox {
                                            Layout.fillWidth: true
                                            visible: importPreviewDelegate.metadataCandidateCount > 1
                                            model: romImportDialog.metadataChoiceLabels(
                                                       importPreviewDelegate.metadataCandidates)
                                            currentIndex: romImportDialog.metadataChoiceIndex(
                                                              importPreviewDelegate.metadataCandidates,
                                                              importPreviewDelegate.selectedMetadataDatabaseId)
                                            Accessible.name: "Metadata match for "
                                                             + importPreviewDelegate.title
                                            onActivated: function(choiceIndex) {
                                                romImportDialog.selectMetadataChoice(
                                                    importPreviewDelegate.index,
                                                    importPreviewDelegate.metadataCandidates,
                                                    choiceIndex)
                                            }
                                        }
                                        Label {
                                            Layout.fillWidth: true
                                            text: importPreviewDelegate.sourcePath
                                                  + (importPreviewDelegate.romFileCount > 1
                                                     ? "  (+" + (importPreviewDelegate.romFileCount - 1)
                                                       + " combined ROM file(s))"
                                                     : "")
                                                  + (importPreviewDelegate.companionFileCount > 0
                                                     ? "  (+" + importPreviewDelegate.companionFileCount
                                                       + " same-name file(s))"
                                                     : "")
                                                  + (importPreviewDelegate.metadataCandidateCount > 0
                                                     ? "  (" + importPreviewDelegate.metadataCandidateCount
                                                       + " " + importPreviewDelegate.metadataMatchKind
                                                       + " metadata match(es))"
                                                     : "")
                                                  + (importPreviewDelegate.manualCandidateCount > 0
                                                     ? "  (" + importPreviewDelegate.manualCandidateCount
                                                       + " PDF manual candidate(s))"
                                                     : "")
                                            elide: Text.ElideMiddle
                                            color: "#8b949e"
                                        }
                                        Label {
                                            Layout.fillWidth: true
                                            visible: importPreviewDelegate.manualSourcePath.length > 0
                                            text: "Manual: " + importPreviewDelegate.manualSourcePath
                                                  + " — stored as "
                                                  + importPreviewDelegate.manualStoredPath
                                            elide: Text.ElideMiddle
                                            color: "#79c0ff"
                                        }
                                        Label {
                                            Layout.fillWidth: true
                                            text: importPreviewDelegate.message
                                                  + " — stored as "
                                                  + importPreviewDelegate.applicationPath
                                            elide: Text.ElideRight
                                            color: importPreviewDelegate.rowState === "ready"
                                                   ? "#7ee787"
                                                   : importPreviewDelegate.rowState === "duplicate"
                                                     ? "#d2a8ff"
                                                   : "#f2cc60"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            RowLayout {
                Layout.fillWidth: true
                Button {
                    text: "Cancel"
                    onClicked: romImportDialog.close()
                }
                Item { Layout.fillWidth: true }
                Button {
                    text: "Back"
                    visible: romImportDialog.page > 0
                    enabled: !controller.import_scanning
                    onClicked: romImportDialog.page -= 1
                }
                Button {
                    text: romImportDialog.page === 0 ? "Next"
                          : romImportDialog.page === 1 ? "Preview"
                          : "Import Selected"
                    enabled: !controller.import_scanning
                             && (romImportDialog.page !== 0
                                 || importLocations.count > 0)
                    onClicked: {
                        if (romImportDialog.page === 0)
                            romImportDialog.page = 1
                        else if (romImportDialog.page === 1)
                            romImportDialog.requestPreview()
                        else
                            romImportDialog.submitPreview()
                    }
                }
            }
        }
    }

    FileDialog {
        id: importFileDialog
        title: "Choose ROM files"
        fileMode: FileDialog.OpenFiles
        nameFilters: ["All files (*)"]
        onAccepted: {
            for (let index = 0; index < selectedFiles.length; ++index) {
                const path = controller.local_path_from_url(
                               selectedFiles[index].toString())
                romImportDialog.addLocation(path, "file")
            }
        }
    }

    FolderDialog {
        id: importFolderDialog
        title: "Choose a ROM folder"
        onAccepted: {
            const path = controller.local_path_from_url(selectedFolder.toString())
            romImportDialog.addLocation(path, "folder")
        }
    }

    Dialog {
        id: addGameDialog
        anchors.centerIn: parent
        modal: true
        title: "Add Game"
        standardButtons: Dialog.Save | Dialog.Cancel

        function prepare() {
            addTitle.text = ""
            addApplicationPath.text = ""
            let selectedIndex = 0
            for (let index = 0; index < controller.platform_entry_count; ++index) {
                if (window.platformName(index) === window.selectedPlatform) {
                    selectedIndex = index
                    break
                }
            }
            addPlatform.currentIndex = selectedIndex
            open()
        }

        function smokeAdd(title, applicationPath, platformName) {
            prepare()
            addTitle.text = title
            addApplicationPath.text = applicationPath
            for (let index = 0; index < controller.platform_entry_count; ++index) {
                if (window.platformName(index) === platformName) {
                    addPlatform.currentIndex = index
                    break
                }
            }
            Qt.callLater(function() { addGameDialog.accept() })
        }

        onAccepted: controller.add_game(addTitle.text, addApplicationPath.text,
                                        window.platformName(addPlatform.currentIndex))

        contentItem: ColumnLayout {
            spacing: 10
            Label { text: "Title" }
            TextField {
                id: addTitle
                Layout.preferredWidth: 420
                placeholderText: "Game title"
            }
            Label { text: "Application path" }
            TextField {
                id: addApplicationPath
                Layout.fillWidth: true
                placeholderText: "Games/Platform/game.rom"
            }
            Label { text: "Platform" }
            ComboBox {
                id: addPlatform
                Layout.fillWidth: true
                model: controller.platform_entry_count
                displayText: currentIndex >= 0 ? window.platformName(currentIndex) : ""
                delegate: ItemDelegate {
                    required property int index
                    width: addPlatform.width
                    text: window.platformName(index)
                }
            }
            Label {
                Layout.preferredWidth: 420
                text: "A UUID is generated automatically. Save uses the same conflict-checked transaction and exact-backup path as edits."
                wrapMode: Text.Wrap
                color: "#7d8590"
            }
        }
    }

    Dialog {
        id: deleteConfirmation
        anchors.centerIn: parent
        modal: true
        title: "Delete " + gameTitle + "?"
        standardButtons: Dialog.Yes | Dialog.No
        property int modelRow: -1
        property string gameId: ""
        property string gameTitle: ""

        function prepare(row, id, title) {
            modelRow = row
            gameId = id
            gameTitle = title
            open()
        }

        function smokeDelete(row, id, title) {
            prepare(row, id, title)
            Qt.callLater(function() { deleteConfirmation.accept() })
        }

        onAccepted: controller.delete_game(modelRow, gameId)

        contentItem: Label {
            width: 420
            text: "Deletion is refused if any modeled platform, playlist, navigation, clone, save, controller, or blacklist record still references this game. Media files are not deleted."
            wrapMode: Text.Wrap
        }
    }

    LaunchStartupOverlay {
        id: launchStartupOverlay
        anchors.fill: parent
        controller: controller
    }

    LaunchShutdownOverlay {
        id: launchShutdownOverlay
        anchors.fill: parent
        controller: controller
    }

    Button {
        anchors.top: parent.top
        anchors.right: parent.right
        anchors.margins: 18
        z: 9000
        visible: controller.pause_screen_available
                 && !controller.pause_screen_active
        text: "Pause Game  Ctrl+P"
        onClicked: controller.pause_launch_session()
    }

    Shortcut {
        sequence: "Ctrl+P"
        enabled: controller.pause_screen_available
        onActivated: {
            if (controller.pause_screen_active)
                controller.resume_launch_session()
            else
                controller.pause_launch_session()
        }
    }

    LaunchPauseOverlay {
        id: launchPauseOverlay
        anchors.fill: parent
        controller: controller
    }

    footer: Column {
        width: parent.width

        Rectangle {
            width: parent.width
            height: visible ? 48 : 0
            visible: controller.delete_blocker_count > 0
            color: "#5c3b12"
            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 14
                anchors.rightMargin: 14
                Label {
                    Layout.fillWidth: true
                    text: "Delete refused: " + controller.delete_blocker_summary
                    color: "white"
                    font.bold: true
                    elide: Text.ElideRight
                }
                Button {
                    text: "Dismiss"
                    onClicked: controller.dismiss_delete_blocker()
                }
            }
        }

        Rectangle {
            width: parent.width
            height: visible ? 48 : 0
            visible: controller.pending_recovery_count > 0 || controller.write_conflict
            color: controller.pending_recovery_count > 0 ? "#5c3b12" : "#5a1f24"
            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 14
                anchors.rightMargin: 14
                Label {
                    Layout.fillWidth: true
                    text: controller.pending_recovery_count > 0
                          ? controller.pending_recovery_count
                            + " interrupted transaction(s) require safe rollback before editing."
                          : "The library changed during a write. No data was overwritten; reload before retrying."
                    color: "white"
                    font.bold: true
                    elide: Text.ElideRight
                }
                Button {
                    text: controller.pending_recovery_count > 0 ? "Recover" : "Reload"
                    enabled: !controller.loading && !controller.writing
                             && !controller.launching
                    onClicked: {
                        if (controller.pending_recovery_count > 0)
                            controller.recover_pending_changes()
                        else
                            controller.reload_library()
                    }
                }
            }
        }

        ToolBar {
            width: parent.width
            background: Rectangle { color: "#20252d" }
            Label {
                anchors.fill: parent
                anchors.leftMargin: 14
                verticalAlignment: Text.AlignVCenter
                text: controller.status_message
                color: "#aeb8c5"
                elide: Text.ElideRight
            }
        }
    }
}
