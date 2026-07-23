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
    property bool categoryCrudSmokeTest:
        Qt.application.arguments.indexOf("--category-crud-smoke-test") >= 0
    property bool playlistCrudSmokeTest:
        Qt.application.arguments.indexOf("--playlist-crud-smoke-test") >= 0
    property bool importSmokeTest:
        Qt.application.arguments.indexOf("--import-smoke-test") >= 0
    property bool launchSmokeTest: Qt.application.arguments.indexOf("--launch-smoke-test") >= 0
    property bool pathMappingSmokeTest:
        Qt.application.arguments.indexOf("--path-mapping-smoke-test") >= 0
    property int loadHeartbeat: 0
    property int smokePhase: 0
    property int editSmokePhase: 0
    property bool editSmokeFinished: false
    property int crudSmokePhase: 0
    property int crudBlockedReferences: 0
    property string crudAddedGameId: ""
    property bool crudSmokeFinished: false
    property int platformCrudSmokePhase: 0
    property int platformCrudBlockedReferences: 0
    property string platformCrudAddedGameId: ""
    property bool platformCrudSmokeFinished: false
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
    property int pathMappingSmokePhase: 0
    property bool pathMappingSmokeFinished: false
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

    Component.onCompleted: {
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
                if (first.length === 0 || second.length === 0) {
                    console.error("IMPORT_SMOKE_MISSING_SOURCE_ARGUMENTS")
                    Qt.exit(12)
                    return
                }
                window.importSmokePhase = 1
                romImportDialog.smokePrepare([first, second], "Fixture Console")
            } else if (window.importSmokePhase === 1
                       && !controller.import_scanning
                       && controller.import_preview_json.length > 0) {
                window.importSmokePhase = 2
                romImportDialog.smokeSubmitPreview()
            } else if (window.importSmokePhase === 2
                       && !controller.writing
                       && controller.last_import_count === 1) {
                if (!controller.report_import_smoke_success(1, 2, 0)) {
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
                        model: controller.emulator_entry_count()
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
                Button {
                    text: "Delete Game…"
                    enabled: !controller.writing
                    onClicked: {
                        gameEditor.close()
                        deleteConfirmation.prepare(gameEditor.modelRow, gameEditor.gameId,
                                                   gameEditor.gameTitle)
                    }
                }
            }
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
            combineDiscSets.checked = true
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
                "combine_disc_sets": combineDiscSets.checked,
                "emulator_id": emulatorId.length === 0 ? null : emulatorId
            }
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
                    "fileCount": 1 + row.additional_discs.length
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
                    "included": row.included
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
                        model: controller.emulator_entry_count()
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
                        id: combineDiscSets
                        text: "Combine complete (Disc N) sets into one game"
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
                                required property int fileCount
                                width: ListView.view.width
                                height: 92
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
                                        Label {
                                            Layout.fillWidth: true
                                            text: importPreviewDelegate.sourcePath
                                                  + (importPreviewDelegate.fileCount > 1
                                                     ? "  (+" + (importPreviewDelegate.fileCount - 1)
                                                       + " disc file(s))"
                                                     : "")
                                            elide: Text.ElideMiddle
                                            color: "#8b949e"
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
