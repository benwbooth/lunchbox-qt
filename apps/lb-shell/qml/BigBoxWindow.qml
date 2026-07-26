import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQml.Models
import QtMultimedia
import LaunchBoxPort

ApplicationWindow {
    id: window
    width: 1280
    height: 720
    visible: true
    visibility: Qt.application.arguments.indexOf("--windowed") >= 0
                || Qt.application.arguments.indexOf("--smoke-test") >= 0
                ? Window.Windowed : Window.FullScreen
    title: "BigBox Port"
    color: "#07090d"
    property bool smokeTest: Qt.application.arguments.indexOf("--smoke-test") >= 0
    property bool mediaSmokeTest: Qt.application.arguments.indexOf("--media-smoke-test") >= 0
    property bool mediaSmokeFinished: false
    property bool gameDetailsMediaSmokeTest:
        Qt.application.arguments.indexOf(
            "--bigbox-game-details-media-smoke-test") >= 0
    property int gameDetailsMediaSmokePhase: 0
    property bool gameDetailsMediaSmokeFinished: false
    property bool gameDetailsMediaScreenshotRequested: false
    property string gameDetailsMediaScreenshotPath:
        argumentValue("--bigbox-game-details-media-screenshot")
    property bool imageViewerSmokeTest:
        Qt.application.arguments.indexOf(
            "--bigbox-image-viewer-smoke-test") >= 0
    property int imageViewerSmokePhase: 0
    property bool imageViewerSmokeFinished: false
    property bool imageViewerScreenshotRequested: false
    property string imageViewerScreenshotPath:
        argumentValue("--bigbox-image-viewer-screenshot")
    property bool boxFlipSmokeTest:
        Qt.application.arguments.indexOf(
            "--bigbox-box-flip-smoke-test") >= 0
    property int boxFlipSmokePhase: 0
    property bool boxFlipSmokeFinished: false
    property bool boxFlipScreenshotRequested: false
    property string boxFlipScreenshotPath:
        argumentValue("--bigbox-box-flip-screenshot")
    property bool modelViewerSmokeTest:
        Qt.application.arguments.indexOf(
            "--bigbox-model-viewer-smoke-test") >= 0
    property int modelViewerSmokePhase: 0
    property bool modelViewerSmokeFinished: false
    property bool modelViewerScreenshotRequested: false
    property bool modelViewerRestoredHorizontalLock: false
    property string modelViewerScreenshotPath:
        argumentValue("--bigbox-model-viewer-screenshot")
    property bool navigationSmokeTest:
        Qt.application.arguments.indexOf("--navigation-smoke-test") >= 0
    property bool libraryFilterSmokeTest:
        Qt.application.arguments.indexOf("--library-filter-smoke-test") >= 0
    property int libraryFilterSmokePhase: 0
    property bool libraryFilterSmokeFinished: false
    property bool libraryOrderSmokeTest:
        Qt.application.arguments.indexOf("--library-order-smoke-test") >= 0
    property int libraryOrderSmokePhase: 0
    property int libraryOrderSmokeRandomRow: -1
    property bool libraryOrderSmokeFinished: false
    property bool launchSmokeTest: Qt.application.arguments.indexOf("--launch-smoke-test") >= 0
    property bool launchLifecycleSmokeTest:
        Qt.application.arguments.indexOf("--launch-lifecycle-smoke-test") >= 0
    property bool launchLifecycleShortProcess:
        Qt.application.arguments.indexOf("--launch-lifecycle-short-process") >= 0
    property int smokePhase: 0
    property int navigationSmokePhase: 0
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
    property string activeNavigationName: "All Games"
    property string selectedBigBoxGameId: ""
    property string selectedBigBoxGameTitle: ""
    property string selectedBigBoxGamePlatform: ""
    property string selectedBigBoxGameNotes: ""
    property string selectedBigBoxGameDeveloper: ""
    property string selectedBigBoxGamePublisher: ""
    property string selectedBigBoxGameGenre: ""
    property string selectedBigBoxGameVersion: ""
    property bool selectedBigBoxGameFavorite: false
    property bool selectedBigBoxGameCompleted: false
    property int selectedBigBoxGamePlayCount: 0
    property int selectedBigBoxGameStarRating: 0
    property real selectedBigBoxGamePlayTimeSeconds: 0
    property real selectedBigBoxGameCommunityRating: 0
    property url selectedBigBoxGameFrontImageUrl
    property bool selectedBigBoxGameBoxBackVisible: false
    readonly property url selectedBigBoxGameBackImageUrl: {
        const revision = controller.game_media_revision
        return selectedBigBoxGameId.length > 0
            ? controller.game_box_back_url_for_game(
                  selectedBigBoxGameId) : ""
    }
    readonly property int selectedBigBoxGameImageCount: {
        const revision = controller.game_media_revision
        return selectedBigBoxGameId.length > 0
            ? controller.game_image_count_for_game(
                  selectedBigBoxGameId) : 0
    }
    property string launchSmokeGameId: {
        const requested = argumentValue("--launch-game-id")
        return requested.length > 0 ? requested : "fixture-racer"
    }
    property string launchSmokeAdditionalApplicationId:
        argumentValue("--launch-additional-application-id")
    readonly property var gameStateFilterChoices: [
        { label: "Any state", key: "any" },
        { label: "Favorites", key: "favorite" },
        { label: "Not favorite", key: "not-favorite" },
        { label: "Completed", key: "completed" },
        { label: "Not completed", key: "not-completed" },
        { label: "Installed", key: "installed" },
        { label: "Not installed", key: "not-installed" },
        { label: "Installation unknown", key: "installation-unknown" },
        { label: "Played", key: "played" },
        { label: "Never played", key: "never-played" },
        { label: "Rated", key: "rated" },
        { label: "Unrated", key: "unrated" },
        { label: "Hidden only", key: "hidden" },
        { label: "Broken only", key: "broken" }
    ]
    readonly property var missingMediaFilterChoices: [
        { label: "Any media status", key: "none" },
        { label: "Missing any media", key: "any" },
        { label: "Missing background", key: "background-image" },
        { label: "Missing banner", key: "banner-image" },
        { label: "Missing 3D box", key: "box-3d-image" },
        { label: "Missing front box", key: "box-front-image" },
        { label: "Missing 3D cart", key: "cart-3d-image" },
        { label: "Missing cart", key: "cart-image" },
        { label: "Missing clear logo", key: "clear-logo-image" },
        { label: "Missing manual", key: "manual" },
        { label: "Missing marquee", key: "marquee-image" },
        { label: "Missing music", key: "music" },
        { label: "Missing screenshot", key: "screenshot-image" },
        { label: "Missing video", key: "video" }
    ]
    readonly property var gameSortChoices: [
        { label: "Title", key: "Title" },
        { label: "Sort title", key: "SortTitle" },
        { label: "Platform", key: "Platform" },
        { label: "Release date", key: "ReleaseDate" },
        { label: "Date added", key: "DateAdded" },
        { label: "Date modified", key: "DateModified" },
        { label: "Last played", key: "LastPlayed" },
        { label: "Play count", key: "PlayCount" },
        { label: "Play time", key: "PlayTime" },
        { label: "Star rating", key: "StarRating" },
        { label: "Community rating", key: "CommunityStarRating" },
        { label: "Developer", key: "Developer" },
        { label: "Publisher", key: "Publisher" },
        { label: "Genre", key: "Genre" },
        { label: "Series", key: "Series" },
        { label: "Status", key: "Status" },
        { label: "Favorite", key: "Favorite" }
    ]

    function launchSelection() {
        if (gameList.currentIndex >= 0 && !controller.loading && !controller.writing
                && !controller.launching && !controller.launch_session_active) {
            const gameId = controller.game_id_at(gameList.currentIndex)
            if (gameId.length > 0)
                controller.launch_game(gameList.currentIndex, gameId)
        }
    }

    function openGameDetails() {
        if (selectedBigBoxGameId.length === 0 || controller.loading
                || controller.writing)
            return false
        bigBoxGameDetails.open()
        return true
    }

    function openGameImages(preferredMediaIndex) {
        if (selectedBigBoxGameId.length === 0 || controller.loading
                || controller.writing
                || controller.game_image_count_for_game(
                    selectedBigBoxGameId) === 0)
            return false
        return bigBoxImageViewer.openForGame(
                    selectedBigBoxGameId, preferredMediaIndex)
    }

    function openGameModel(returnFocusItem) {
        if (!controller.big_box_show_game_menu_model
                || selectedBigBoxGameId.length === 0
                || controller.loading || controller.writing)
            return false
        bigBoxMediaPlayer.stop()
        return bigBoxModelViewer.openForGame(
                    selectedBigBoxGameId,
                    selectedBigBoxGameTitle,
                    returnFocusItem ? returnFocusItem : gameList)
    }

    function flipSelectedBox() {
        if (!controller.big_box_show_game_menu_flip_box
                || selectedBigBoxGameId.length === 0
                || selectedBigBoxGameBackImageUrl.toString().length === 0)
            return false
        selectedBigBoxGameBoxBackVisible =
            !selectedBigBoxGameBoxBackVisible
        return true
    }

    function formatPlayTime(seconds) {
        const totalMinutes = Math.floor(Math.max(0, seconds) / 60)
        const hours = Math.floor(totalMinutes / 60)
        const minutes = totalMinutes % 60
        if (hours > 0)
            return hours + "h " + minutes + "m"
        return minutes + "m"
    }

    function clearSelectedBigBoxGame() {
        selectedBigBoxGameId = ""
        selectedBigBoxGameTitle = ""
        selectedBigBoxGamePlatform = ""
        selectedBigBoxGameNotes = ""
        selectedBigBoxGameDeveloper = ""
        selectedBigBoxGamePublisher = ""
        selectedBigBoxGameGenre = ""
        selectedBigBoxGameVersion = ""
        selectedBigBoxGameFavorite = false
        selectedBigBoxGameCompleted = false
        selectedBigBoxGamePlayCount = 0
        selectedBigBoxGameStarRating = 0
        selectedBigBoxGamePlayTimeSeconds = 0
        selectedBigBoxGameCommunityRating = 0
        selectedBigBoxGameFrontImageUrl = ""
        selectedBigBoxGameBoxBackVisible = false
    }

    function showLaunchWithSelection() {
        if (gameList.currentIndex < 0 || controller.loading || controller.writing
                || controller.launching || controller.launch_session_active)
            return
        const gameId = controller.game_id_at(gameList.currentIndex)
        const count = controller.additional_application_count(
                          gameList.currentIndex, gameId)
        if (gameId.length > 0 && count > 0)
            launchWithDialog.prepare(gameList.currentIndex, gameId, count)
    }

    function filterChoiceIndex(choices, key) {
        for (let index = 0; index < choices.length; ++index) {
            if (choices[index].key === key)
                return index
        }
        return -1
    }

    function openAttributeFilters() {
        attributeFilterDrawer.open()
        bigBoxStateFilterCombo.forceActiveFocus()
    }

    function applyAttributeFilters() {
        const applied = controller.apply_game_attribute_filters(
                          bigBoxStateFilterCombo.currentValue,
                          bigBoxMissingMediaFilterCombo.currentValue,
                          bigBoxIncludeHiddenCheck.checked,
                          bigBoxIncludeBrokenCheck.checked)
        if (applied) {
            gameList.currentIndex = gameList.count > 0 ? 0 : -1
            gameList.positionViewAtBeginning()
        }
        return applied
    }

    function setAttributeFilters(stateKey, missingMediaKey,
                                 includeHidden, includeBroken) {
        bigBoxStateFilterCombo.currentIndex =
            filterChoiceIndex(gameStateFilterChoices, stateKey)
        bigBoxMissingMediaFilterCombo.currentIndex =
            filterChoiceIndex(missingMediaFilterChoices, missingMediaKey)
        bigBoxIncludeHiddenCheck.checked = includeHidden
        bigBoxIncludeBrokenCheck.checked = includeBroken
        return applyAttributeFilters()
    }

    function applyCurrentSort() {
        const selectedId = gameList.currentIndex >= 0
                           ? controller.game_id_at(gameList.currentIndex) : ""
        const applied = controller.apply_game_sort(
                            bigBoxSortCombo.currentValue,
                            bigBoxSortDescendingCheck.checked)
        if (applied) {
            const selectedRow = selectedId.length > 0
                              ? controller.row_for_game_id(selectedId) : -1
            gameList.currentIndex = selectedRow >= 0
                                  ? selectedRow
                                  : (gameList.count > 0 ? 0 : -1)
            if (gameList.currentIndex >= 0)
                gameList.positionViewAtIndex(
                            gameList.currentIndex, ListView.Contain)
        }
        return applied
    }

    function selectRandomGame() {
        const currentId = gameList.currentIndex >= 0
                        ? controller.game_id_at(gameList.currentIndex) : ""
        const row = controller.select_random_game(currentId)
        if (row >= 0) {
            gameList.currentIndex = row
            gameList.positionViewAtIndex(row, ListView.Center)
            gameList.forceActiveFocus()
        }
        return row
    }

    function filteredIdsMatch(expected) {
        if (controller.filtered_count !== expected.length)
            return false
        for (let index = 0; index < expected.length; ++index) {
            if (controller.game_id_at(index) !== expected[index])
                return false
        }
        return true
    }

    function bigBoxNavigationIndex(kind, key) {
        for (let index = 0; index < controller.big_box_navigation_entry_count; ++index) {
            if (controller.big_box_navigation_entry_kind_at(index) === kind
                    && controller.big_box_navigation_entry_key_at(index) === key)
                return index
        }
        return -1
    }

    function openNavigation() {
        navigationDrawer.open()
        navigationList.currentIndex = 0
        for (let index = 0; index < controller.big_box_navigation_entry_count; ++index) {
            const kind = controller.big_box_navigation_entry_kind_at(index)
            const key = controller.big_box_navigation_entry_key_at(index)
            if (kind === controller.navigation_filter_kind
                    && key === controller.navigation_filter_key) {
                navigationList.currentIndex = index + 1
                break
            }
        }
        navigationList.positionViewAtIndex(navigationList.currentIndex,
                                           ListView.Contain)
        navigationList.forceActiveFocus()
    }

    function activateNavigationRow(row) {
        if (row <= 0) {
            activeNavigationName = "All Games"
            controller.apply_filters("", "")
        } else {
            const index = row - 1
            const kind = controller.big_box_navigation_entry_kind_at(index)
            const key = controller.big_box_navigation_entry_key_at(index)
            activeNavigationName = controller.big_box_navigation_entry_name_at(index)
            if (kind === "category")
                controller.apply_category_filter("", key)
            else if (kind === "playlist")
                controller.apply_playlist_filter("", key)
            else if (kind === "platform")
                controller.apply_filters("", key)
            else
                return
        }
        navigationDrawer.close()
        gameList.currentIndex = gameList.count > 0 ? 0 : -1
        gameList.forceActiveFocus()
    }

    function verifyModelRoles(index, gameId, gameTitle, gamePlatform, gameFavorite,
                              gameCompleted, gamePlayCount, gameStarRating,
                              gameAdditionalApplicationCount, gamePlayTimeSeconds,
                              gameLastPlayedDate, gameDateAdded, gameDateModified,
                              gameCommunityStarRating,
                              gameCommunityStarRatingTotalVotes,
                              gameInstalledState, gameHidden, gameBroken,
                              gamePortable, gameVideoUrl, gameDatabaseId,
                              gameAlternateNames, gameFrontImageUrl,
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
        const statisticsMatch = smokePhase === 0
            ? gamePlayTimeSeconds === 5400
              && gameLastPlayedDate === "2026-07-22T10:00:00-07:00"
              && gameDateAdded === "2026-07-22T08:00:00-07:00"
              && gameDateModified === "2026-07-22T09:00:00-07:00"
              && gameCommunityStarRating === 4.25
              && gameCommunityStarRatingTotalVotes === 42
              && gameInstalledState === 1
            : gamePlayTimeSeconds === 14400 && gameLastPlayedDate === ""
              && gameDateAdded === "" && gameDateModified === ""
              && gameCommunityStarRating === 0
              && gameCommunityStarRatingTotalVotes === 0
              && gameInstalledState === -1
        const extendedListMatches = smokePhase === 0
            ? !gameHidden && !gameBroken && !gamePortable
              && gameVideoUrl
                 === "https://example.invalid/fixture-adventure.mp4"
              && gameDatabaseId === 1234
              && gameAlternateNames === "The Fixture Adventure"
            : !gameHidden && !gameBroken && !gamePortable
              && gameVideoUrl === "" && gameDatabaseId === 0
              && gameAlternateNames === ""
        if (gameId !== expectedId || gameTitle !== expectedTitle
                || gamePlatform !== "Fixture Console"
                || gameFavorite !== expectedFavorite || gameCompleted !== expectedCompleted
                || gamePlayCount !== expectedPlayCount || gameStarRating !== expectedStarRating
                || gameAdditionalApplicationCount !== expectedAdditionalApplicationCount
                || !statisticsMatch
                || !extendedListMatches
                || gameFrontImageUrl.toString() !== ""
                || rowCount !== expectedRows) {
            console.error("MODEL_ROLE_SMOKE_FAILED id=" + gameId
                          + " title=" + gameTitle
                          + " platform=" + gamePlatform
                          + " favorite=" + gameFavorite
                          + " completed=" + gameCompleted
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

    function argumentValue(flag) {
        const index = Qt.application.arguments.indexOf(flag)
        return index >= 0 && index + 1 < Qt.application.arguments.length
               ? Qt.application.arguments[index + 1] : ""
    }

    function isSmokeRun() {
        for (let index = 0; index < Qt.application.arguments.length; ++index) {
            const argument = Qt.application.arguments[index]
            if (argument === "--smoke-test"
                    || argument.endsWith("-smoke-test"))
                return true
        }
        return false
    }

    LibraryController {
        id: controller
    }

    Connections {
        target: controller

        function onGame_state_filterChanged() {
            const index = window.filterChoiceIndex(
                              window.gameStateFilterChoices,
                              controller.game_state_filter)
            if (index >= 0)
                bigBoxStateFilterCombo.currentIndex = index
        }

        function onMissing_media_filterChanged() {
            const index = window.filterChoiceIndex(
                              window.missingMediaFilterChoices,
                              controller.missing_media_filter)
            if (index >= 0)
                bigBoxMissingMediaFilterCombo.currentIndex = index
        }

        function onGame_sortChanged() {
            const index = window.filterChoiceIndex(
                              window.gameSortChoices, controller.game_sort)
            if (index >= 0)
                bigBoxSortCombo.currentIndex = index
        }

        function onGame_sort_descendingChanged() {
            bigBoxSortDescendingCheck.checked = controller.game_sort_descending
        }

        function onInclude_hidden_gamesChanged() {
            bigBoxIncludeHiddenCheck.checked = controller.include_hidden_games
        }

        function onInclude_broken_gamesChanged() {
            bigBoxIncludeBrokenCheck.checked = controller.include_broken_games
        }

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
        controller.configure_frontend(true)
        const usePersistedModelState = !window.isSmokeRun()
                                     || argumentValue(
                                         "--model-viewer-state-file")
                                        .length > 0
        if (usePersistedModelState
                && !controller.initialize_model_viewer_state()) {
            console.error(
                "MODEL_VIEWER_STATE_INITIALIZE_FAILED status="
                + controller.status_message)
            if (window.modelViewerSmokeTest)
                Qt.exit(544)
            return
        }
        if (!controller.initialize_host_path_mappings()) {
            console.error("HOST_PATH_MAPPING_INITIALIZE_FAILED status="
                          + controller.status_message)
            return
        }
        const library = argumentValue("--library")
        if (library.length > 0)
            controller.load_library(library)
        else
            controller.load_fixture()
        gameList.forceActiveFocus()
    }

    Timer {
        interval: 15000
        running: window.mediaSmokeTest && !window.mediaSmokeFinished
        onTriggered: {
            console.error("MEDIA_SMOKE_TIMEOUT images="
                          + controller.front_image_count)
            Qt.exit(45)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.boxFlipSmokeTest
                 && !window.boxFlipSmokeFinished
        onTriggered: {
            if (controller.loading || controller.library_path.length === 0)
                return
            if (window.boxFlipSmokePhase === 0) {
                const row = controller.row_for_game_id(
                                "fixture-adventure")
                if (row < 0)
                    return
                gameList.currentIndex = row
                gameList.positionViewAtIndex(row, ListView.Center)
                window.boxFlipSmokePhase = 1
                return
            }
            const card = gameList.currentItem
            if (!card)
                return
            // qmllint disable missing-property
            if (window.boxFlipSmokePhase === 1) {
                if (window.selectedBigBoxGameId
                        !== "fixture-adventure"
                        || window.selectedBigBoxGameBackImageUrl
                           .toString().length === 0
                        || window.selectedBigBoxGameBoxBackVisible
                        || card["displayedBoxSource"].toString()
                           !== window
                              .selectedBigBoxGameFrontImageUrl
                              .toString()
                        || card["displayedBoxStatus"]
                           !== Image.Ready)
                    return
                if (!bigBoxFlipBoxButton.activate()) {
                    console.error(
                        "BIGBOX_BOX_FLIP_CONTROL_MISSING")
                    Qt.exit(531)
                    return
                }
                window.boxFlipSmokePhase = 2
            } else if (window.boxFlipSmokePhase === 2) {
                if (!window.selectedBigBoxGameBoxBackVisible
                        || card["displayedBoxSource"].toString()
                           !== window.selectedBigBoxGameBackImageUrl
                              .toString()
                        || card["displayedBoxStatus"]
                           !== Image.Ready
                        || card["displayedBoxFlipAngle"] < 179)
                    return
                if (window.boxFlipScreenshotRequested)
                    return
                window.boxFlipScreenshotRequested = true
                const returnToFront = function() {
                    if (!bigBoxFlipBoxButton.activate()) {
                        console.error(
                            "BIGBOX_BOX_FLIP_RETURN_CONTROL_MISSING")
                        Qt.exit(532)
                        return
                    }
                    window.boxFlipSmokePhase = 3
                }
                if (window.boxFlipScreenshotPath.length === 0) {
                    returnToFront()
                    return
                }
                bigBoxContent.grabToImage(function(result) {
                    if (!result.saveToFile(
                            window.boxFlipScreenshotPath)) {
                        console.error(
                            "BIGBOX_BOX_FLIP_SCREENSHOT_SAVE_FAILED path="
                            + window.boxFlipScreenshotPath)
                        Qt.exit(533)
                        return
                    }
                    returnToFront()
                })
            } else if (window.boxFlipSmokePhase === 3) {
                if (window.selectedBigBoxGameBoxBackVisible
                        || card["displayedBoxSource"].toString()
                           !== window
                              .selectedBigBoxGameFrontImageUrl
                              .toString()
                        || card["displayedBoxStatus"]
                           !== Image.Ready
                        || card["displayedBoxFlipAngle"] > 1)
                    return
                if (!controller.report_big_box_box_flip_smoke_success(
                        window.selectedBigBoxGameId,
                        window.selectedBigBoxGameFrontImageUrl
                              .toString(),
                        window.selectedBigBoxGameBackImageUrl
                              .toString())) {
                    console.error(
                        "BIGBOX_BOX_FLIP_CONTROLLER_REJECTED")
                    Qt.exit(534)
                    return
                }
                window.boxFlipSmokeFinished = true
                Qt.quit()
            }
            // qmllint enable missing-property
        }
    }

    Timer {
        interval: 20000
        running: window.boxFlipSmokeTest
                 && !window.boxFlipSmokeFinished
        onTriggered: {
            console.error(
                "BIGBOX_BOX_FLIP_TIMEOUT phase="
                + window.boxFlipSmokePhase
                + " id=" + window.selectedBigBoxGameId
                + " status=" + controller.status_message)
            if (window.boxFlipSmokePhase === 2) {
                const card = gameList.currentItem
                if (!window.selectedBigBoxGameBoxBackVisible)
                    Qt.exit(551)
                else if (!card)
                    Qt.exit(552)
                // qmllint disable missing-property
                else if (card["displayedBoxSource"].toString()
                         !== window.selectedBigBoxGameBackImageUrl
                            .toString())
                    Qt.exit(553)
                else if (card["displayedBoxStatus"] !== Image.Ready)
                    Qt.exit(554)
                else if (card["displayedBoxFlipAngle"] < 179)
                    Qt.exit(555)
                // qmllint enable missing-property
                else
                    Qt.exit(556)
            } else {
                Qt.exit(535 + window.boxFlipSmokePhase)
            }
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.modelViewerSmokeTest
                 && !window.modelViewerSmokeFinished
        onTriggered: {
            if (controller.loading || controller.library_path.length === 0
                    || !controller.model_viewer_state_ready)
                return
            if (window.modelViewerSmokePhase === 0) {
                const row = controller.row_for_game_id(
                                "fixture-adventure")
                if (row < 0)
                    return
                gameList.currentIndex = row
                gameList.positionViewAtIndex(row, ListView.Center)
                window.modelViewerSmokePhase = 1
                return
            }
            if (window.modelViewerSmokePhase === 1) {
                if (window.selectedBigBoxGameId
                        !== "fixture-adventure"
                        || controller.model_rotation_lock
                           !== "horizontal")
                    return
                window.modelViewerRestoredHorizontalLock = true
                if (!bigBoxModelButton.activate()) {
                    console.error(
                        "BIGBOX_MODEL_VIEWER_CONTROL_MISSING")
                    Qt.exit(557)
                    return
                }
                window.modelViewerSmokePhase = 2
            } else if (window.modelViewerSmokePhase === 2) {
                if (!bigBoxModelViewer.opened
                        || !bigBoxModelViewer.sceneReady
                        || bigBoxModelViewer.frontImageStatus
                           !== Image.Ready
                        || bigBoxModelViewer.backImageStatus
                           !== Image.Ready
                        || bigBoxModelViewer.spineImageStatus
                           !== Image.Ready
                        || bigBoxModelViewer.fullImageStatus
                           !== Image.Ready
                        || !bigBoxModelViewer.hasBack
                        || !bigBoxModelViewer.hasSpine
                        || !bigBoxModelViewer.hasFull
                        || !bigBoxModelViewer.useFullScan
                        || Math.abs(
                            bigBoxModelViewer.fullSpineFraction - 0.143)
                           > 0.0001
                        || bigBoxModelViewer.rotationLock
                           !== "horizontal"
                        || bigBoxModelViewer.rotationX !== -8
                        || bigBoxModelViewer.rotationY !== -22)
                    return
                if (bigBoxModelViewer.modelType
                        !== "jewelCase"
                        || bigBoxModelViewer.modelTypeDisplay
                           !== "Jewel Case"
                        || bigBoxModelViewer.modelSettingsSource
                           !== "gameOverride"
                        || bigBoxModelViewer.modelWidth !== 260
                        || bigBoxModelViewer.modelHeight !== 230
                        || bigBoxModelViewer.modelDepth !== 20) {
                    console.error(
                        "BIGBOX_MODEL_SETTINGS_MISMATCH type="
                        + bigBoxModelViewer.modelType + " source="
                        + bigBoxModelViewer.modelSettingsSource + " size="
                        + bigBoxModelViewer.modelWidth + "x"
                        + bigBoxModelViewer.modelHeight + "x"
                        + bigBoxModelViewer.modelDepth)
                    Qt.exit(563)
                    return
                }
                bigBoxModelViewer.activateRotateUpControl()
                if (!bigBoxModelViewer
                        .activateRotateRightControl()) {
                    console.error(
                        "BIGBOX_MODEL_VIEWER_ROTATE_CONTROL_MISSING")
                    Qt.exit(558)
                    return
                }
                window.modelViewerSmokePhase = 3
            } else if (window.modelViewerSmokePhase === 3) {
                if (bigBoxModelViewer.rotationX !== -8
                        || bigBoxModelViewer.rotationY !== -12)
                    return
                if (!bigBoxModelViewer.activatePanRightControl()
                        || !bigBoxModelViewer
                           .activateZoomInControl()
                        || !bigBoxModelViewer
                           .activateVerticalLockControl()) {
                    console.error(
                        "BIGBOX_MODEL_VIEWER_NAVIGATION_CONTROL_MISSING")
                    Qt.exit(559)
                    return
                }
                window.modelViewerSmokePhase = 4
            } else if (window.modelViewerSmokePhase === 4) {
                if (controller.model_rotation_lock !== "vertical"
                        || bigBoxModelViewer.rotationLock
                           !== "vertical"
                        || bigBoxModelViewer.panX !== 10
                        || Math.abs(
                            bigBoxModelViewer.modelZoom - 1.15)
                           > 0.0001)
                    return
                bigBoxModelViewer.activateRotateRightControl()
                bigBoxModelViewer.activateRotateUpControl()
                window.modelViewerSmokePhase = 5
            } else if (window.modelViewerSmokePhase === 5) {
                if (bigBoxModelViewer.rotationX !== -18
                        || bigBoxModelViewer.rotationY !== -12)
                    return
                if (window.modelViewerScreenshotRequested)
                    return
                window.modelViewerScreenshotRequested = true
                const closeViewer = function() {
                    if (!bigBoxModelViewer
                            .activateBackControl()) {
                        console.error(
                            "BIGBOX_MODEL_VIEWER_BACK_CONTROL_MISSING")
                        Qt.exit(560)
                        return
                    }
                    window.modelViewerSmokePhase = 6
                }
                if (window.modelViewerScreenshotPath.length === 0) {
                    closeViewer()
                    return
                }
                bigBoxModelViewer.viewerContentItem.grabToImage(
                    function(result) {
                        if (!result.saveToFile(
                                window.modelViewerScreenshotPath)) {
                            console.error(
                                "BIGBOX_MODEL_VIEWER_SCREENSHOT_SAVE_FAILED path="
                                + window.modelViewerScreenshotPath)
                            Qt.exit(561)
                            return
                        }
                        closeViewer()
                    })
            } else if (window.modelViewerSmokePhase === 6) {
                if (bigBoxModelViewer.opened
                        || !bigBoxModelButton.activeFocus)
                    return
                if (!controller
                        .report_big_box_model_viewer_smoke_success(
                            "fixture-adventure",
                            bigBoxModelViewer.frontSource.toString(),
                            bigBoxModelViewer.backSource.toString(),
                            bigBoxModelViewer.spineSource.toString(),
                            bigBoxModelViewer.fullSource.toString(),
                            window
                            .modelViewerRestoredHorizontalLock)) {
                    console.error(
                        "BIGBOX_MODEL_VIEWER_CONTROLLER_REJECTED")
                    Qt.exit(562)
                    return
                }
                window.modelViewerSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 25000
        running: window.modelViewerSmokeTest
                 && !window.modelViewerSmokeFinished
        onTriggered: {
            console.error(
                "BIGBOX_MODEL_VIEWER_TIMEOUT phase="
                + window.modelViewerSmokePhase
                + " open=" + bigBoxModelViewer.opened
                + " ready=" + bigBoxModelViewer.sceneReady
                + " lock=" + controller.model_rotation_lock
                + " status=" + controller.status_message)
            Qt.exit(563 + window.modelViewerSmokePhase)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.gameDetailsMediaSmokeTest
                 && !window.gameDetailsMediaSmokeFinished
        onTriggered: {
            if (controller.loading || controller.library_path.length === 0
                    || window.selectedBigBoxGameId.length === 0)
                return
            if (window.gameDetailsMediaSmokePhase === 0) {
                const row = controller.row_for_game_id("fixture-adventure")
                if (row < 0)
                    return
                gameList.currentIndex = row
                gameList.positionViewAtIndex(row, ListView.Center)
                if (window.selectedBigBoxGameId !== "fixture-adventure")
                    return
                bigBoxGameDetailsButton.clicked()
                window.gameDetailsMediaSmokePhase = 1
            } else if (window.gameDetailsMediaSmokePhase === 1) {
                if (!bigBoxGameDetails.opened
                        || bigBoxGameDetails.mediaCount !== 7
                        || bigBoxGameDetails.selectedMediaIndex !== 6
                        || bigBoxGameDetails.selectedMediaKind !== "video"
                        || bigBoxGameDetails.selectedMediaType !== "Video Snap"
                        || bigBoxGameDetails.mediaDuration <= 0
                        || bigBoxGameDetails.mediaError !== MediaPlayer.NoError
                        || bigBoxGameDetails.mediaPlaybackState
                           !== MediaPlayer.PlayingState)
                    return
                bigBoxMediaPlayPauseButton.clicked()
                window.gameDetailsMediaSmokePhase = 2
            } else if (window.gameDetailsMediaSmokePhase === 2) {
                if (bigBoxGameDetails.mediaPlaybackState
                        !== MediaPlayer.PausedState)
                    return
                bigBoxPreviousMediaButton.clicked()
                window.gameDetailsMediaSmokePhase = 3
            } else if (window.gameDetailsMediaSmokePhase === 3) {
                if (bigBoxGameDetails.selectedMediaIndex !== 5
                        || bigBoxGameDetails.selectedMediaKind !== "image"
                        || bigBoxGameDetails.mediaImageStatus !== Image.Ready)
                    return
                if (!bigBoxGameDetails.clickMediaThumbnailForSmoke(0)) {
                    console.error(
                        "BIGBOX_GAME_DETAILS_MEDIA_IMAGE_THUMBNAIL_MISSING")
                    Qt.exit(507)
                    return
                }
                window.gameDetailsMediaSmokePhase = 4
            } else if (window.gameDetailsMediaSmokePhase === 4) {
                if (bigBoxGameDetails.selectedMediaIndex !== 0
                        || bigBoxGameDetails.selectedMediaKind !== "image"
                        || bigBoxGameDetails.selectedMediaType
                           !== "Box - Front"
                        || bigBoxGameDetails.mediaImageStatus !== Image.Ready)
                    return
                if (window.gameDetailsMediaScreenshotRequested)
                    return
                window.gameDetailsMediaScreenshotRequested = true
                const continueWithVideo = function() {
                    if (!bigBoxGameDetails.clickMediaThumbnailForSmoke(6)) {
                        console.error(
                            "BIGBOX_GAME_DETAILS_MEDIA_VIDEO_THUMBNAIL_MISSING")
                        Qt.exit(508)
                        return
                    }
                    window.gameDetailsMediaSmokePhase = 5
                }
                if (window.gameDetailsMediaScreenshotPath.length === 0) {
                    continueWithVideo()
                    return
                }
                bigBoxGameDetailsContent.grabToImage(function(result) {
                    if (!result.saveToFile(
                            window.gameDetailsMediaScreenshotPath)) {
                        console.error(
                            "BIGBOX_GAME_DETAILS_MEDIA_SCREENSHOT_SAVE_FAILED path="
                            + window.gameDetailsMediaScreenshotPath)
                        Qt.exit(509)
                        return
                    }
                    continueWithVideo()
                })
            } else if (window.gameDetailsMediaSmokePhase === 5) {
                if (bigBoxGameDetails.selectedMediaIndex !== 6
                        || bigBoxGameDetails.selectedMediaKind !== "video"
                        || bigBoxGameDetails.selectedMediaType !== "Video Snap"
                        || bigBoxGameDetails.mediaDuration <= 0
                        || bigBoxGameDetails.mediaError !== MediaPlayer.NoError
                        || bigBoxGameDetails.mediaPlaybackState
                           !== MediaPlayer.PlayingState)
                    return
                bigBoxGameDetailsBackButton.clicked()
                window.gameDetailsMediaSmokePhase = 6
            } else if (window.gameDetailsMediaSmokePhase === 6) {
                if (bigBoxGameDetails.opened
                        || bigBoxGameDetails.mediaPlaybackState
                           !== MediaPlayer.StoppedState)
                    return
                if (!controller
                        .report_big_box_game_details_media_smoke_success(
                        "fixture-adventure", 0, 6,
                        controller.game_media_url_at(
                            "fixture-adventure", 0).toString(),
                        controller.game_media_url_at(
                            "fixture-adventure", 6).toString())) {
                    console.error(
                        "BIGBOX_GAME_DETAILS_MEDIA_SMOKE_CONTROLLER_REJECTED")
                    Qt.exit(510)
                    return
                }
                window.gameDetailsMediaSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 20000
        running: window.gameDetailsMediaSmokeTest
                 && !window.gameDetailsMediaSmokeFinished
        onTriggered: {
            console.error(
                "BIGBOX_GAME_DETAILS_MEDIA_SMOKE_TIMEOUT phase="
                + window.gameDetailsMediaSmokePhase
                + " id=" + window.selectedBigBoxGameId
                + " open=" + bigBoxGameDetails.opened
                + " media=" + bigBoxGameDetails.mediaCount
                + " selected=" + bigBoxGameDetails.selectedMediaIndex
                + " playback=" + bigBoxGameDetails.mediaPlaybackState
                + " status=" + controller.status_message)
            Qt.exit(511)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.imageViewerSmokeTest
                 && !window.imageViewerSmokeFinished
        onTriggered: {
            if (controller.loading || controller.library_path.length === 0
                    || window.selectedBigBoxGameId.length === 0)
                return
            if (window.imageViewerSmokePhase === 0) {
                const row = controller.row_for_game_id("fixture-adventure")
                if (row < 0)
                    return
                gameList.currentIndex = row
                gameList.positionViewAtIndex(row, ListView.Center)
                if (window.selectedBigBoxGameId !== "fixture-adventure")
                    return
                bigBoxGameDetailsButton.clicked()
                window.imageViewerSmokePhase = 1
            } else if (window.imageViewerSmokePhase === 1) {
                if (!bigBoxGameDetails.opened
                        || bigBoxGameDetails.mediaCount !== 7)
                    return
                if (!bigBoxGameDetails.clickMediaThumbnailForSmoke(0)) {
                    console.error(
                        "BIGBOX_IMAGE_VIEWER_IMAGE_THUMBNAIL_MISSING")
                    Qt.exit(512)
                    return
                }
                window.imageViewerSmokePhase = 2
            } else if (window.imageViewerSmokePhase === 2) {
                if (bigBoxGameDetails.selectedMediaIndex !== 0
                        || bigBoxGameDetails.selectedMediaKind !== "image"
                        || bigBoxGameDetails.mediaImageStatus !== Image.Ready)
                    return
                bigBoxViewImageButton.clicked()
                window.imageViewerSmokePhase = 3
            } else if (window.imageViewerSmokePhase === 3) {
                if (!bigBoxImageViewer.opened
                        || bigBoxImageViewer.gameId
                           !== "fixture-adventure"
                        || bigBoxImageViewer.imageCount !== 6
                        || bigBoxImageViewer.selectedImageIndex !== 0
                        || bigBoxImageViewer.selectedMediaIndex !== 0
                        || bigBoxImageViewer.selectedMediaType
                           !== "Box - Front"
                        || bigBoxImageViewer.imageStatus !== Image.Ready
                        || bigBoxImageViewer.zoomFactor !== 1)
                    return
                bigBoxImageZoomInButton.clicked()
                bigBoxImageZoomInButton.clicked()
                window.imageViewerSmokePhase = 4
            } else if (window.imageViewerSmokePhase === 4) {
                if (bigBoxImageViewer.zoomFactor !== 1.5
                        || bigBoxImageViewer.panLimitY <= 0)
                    return
                bigBoxImagePanDownButton.clicked()
                window.imageViewerSmokePhase = 5
            } else if (window.imageViewerSmokePhase === 5) {
                if (bigBoxImageViewer.panY >= 0)
                    return
                if (window.imageViewerScreenshotRequested)
                    return
                window.imageViewerScreenshotRequested = true
                const continueWithNextImage = function() {
                    bigBoxImageNextButton.clicked()
                    window.imageViewerSmokePhase = 6
                }
                if (window.imageViewerScreenshotPath.length === 0) {
                    continueWithNextImage()
                    return
                }
                bigBoxImageViewerContent.grabToImage(function(result) {
                    if (!result.saveToFile(
                            window.imageViewerScreenshotPath)) {
                        console.error(
                            "BIGBOX_IMAGE_VIEWER_SCREENSHOT_SAVE_FAILED path="
                            + window.imageViewerScreenshotPath)
                        Qt.exit(513)
                        return
                    }
                    continueWithNextImage()
                })
            } else if (window.imageViewerSmokePhase === 6) {
                if (bigBoxImageViewer.selectedImageIndex !== 1
                        || bigBoxImageViewer.selectedMediaIndex !== 1
                        || bigBoxImageViewer.selectedMediaType
                           !== "Screenshot - Gameplay"
                        || bigBoxImageViewer.imageStatus !== Image.Ready
                        || bigBoxImageViewer.zoomFactor !== 1
                        || bigBoxImageViewer.panX !== 0
                        || bigBoxImageViewer.panY !== 0)
                    return
                bigBoxImageZoomInButton.clicked()
                window.imageViewerSmokePhase = 7
            } else if (window.imageViewerSmokePhase === 7) {
                if (bigBoxImageViewer.zoomFactor !== 1.25)
                    return
                bigBoxImageFitButton.clicked()
                window.imageViewerSmokePhase = 8
            } else if (window.imageViewerSmokePhase === 8) {
                if (bigBoxImageViewer.zoomFactor !== 1
                        || bigBoxImageViewer.panX !== 0
                        || bigBoxImageViewer.panY !== 0)
                    return
                bigBoxImageViewerBackButton.clicked()
                window.imageViewerSmokePhase = 9
            } else if (window.imageViewerSmokePhase === 9) {
                if (bigBoxImageViewer.opened
                        || !bigBoxGameDetails.opened)
                    return
                bigBoxGameDetailsBackButton.clicked()
                window.imageViewerSmokePhase = 10
            } else if (window.imageViewerSmokePhase === 10) {
                if (bigBoxImageViewer.opened
                        || bigBoxGameDetails.opened)
                    return
                if (!controller
                        .report_big_box_image_viewer_smoke_success(
                        "fixture-adventure", 0, 1,
                        controller.game_media_url_at(
                            "fixture-adventure", 0).toString(),
                        controller.game_media_url_at(
                            "fixture-adventure", 1).toString())) {
                    console.error(
                        "BIGBOX_IMAGE_VIEWER_SMOKE_CONTROLLER_REJECTED")
                    Qt.exit(514)
                    return
                }
                window.imageViewerSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 20000
        running: window.imageViewerSmokeTest
                 && !window.imageViewerSmokeFinished
        onTriggered: {
            console.error(
                "BIGBOX_IMAGE_VIEWER_SMOKE_TIMEOUT phase="
                + window.imageViewerSmokePhase
                + " id=" + window.selectedBigBoxGameId
                + " details=" + bigBoxGameDetails.opened
                + " viewer=" + bigBoxImageViewer.opened
                + " images=" + bigBoxImageViewer.imageCount
                + " selected=" + bigBoxImageViewer.selectedImageIndex
                + " media=" + bigBoxImageViewer.selectedMediaIndex
                + " imageStatus=" + bigBoxImageViewer.imageStatus
                + " zoom=" + bigBoxImageViewer.zoomFactor
                + " pan=" + bigBoxImageViewer.panX
                + "," + bigBoxImageViewer.panY
                + " status=" + controller.status_message)
            Qt.exit(515)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.libraryFilterSmokeTest
                 && !window.libraryFilterSmokeFinished
        onTriggered: {
            if (controller.loading || controller.library_path.length === 0)
                return
            if (window.libraryFilterSmokePhase === 0) {
                if (!window.filteredIdsMatch(["fixture-adventure"])) {
                    console.error("BIGBOX_LIBRARY_FILTER_SMOKE_BAD_DEFAULT")
                    Qt.exit(31)
                    return
                }
                window.libraryFilterSmokePhase = 1
                window.openAttributeFilters()
            } else if (window.libraryFilterSmokePhase === 1
                       && attributeFilterDrawer.opened) {
                window.libraryFilterSmokePhase = 2
                window.setAttributeFilters("any", "none", true, false)
            } else if (window.libraryFilterSmokePhase === 2) {
                if (!window.filteredIdsMatch(
                            ["fixture-adventure", "fixture-puzzle"])) {
                    console.error("BIGBOX_LIBRARY_FILTER_SMOKE_BAD_HIDDEN_INCLUDE")
                    Qt.exit(31)
                    return
                }
                window.libraryFilterSmokePhase = 3
                window.setAttributeFilters("any", "none", true, true)
            } else if (window.libraryFilterSmokePhase === 3) {
                if (!window.filteredIdsMatch(
                            ["fixture-adventure", "fixture-puzzle",
                             "fixture-racer"])) {
                    console.error("BIGBOX_LIBRARY_FILTER_SMOKE_BAD_ALL_INCLUDE")
                    Qt.exit(31)
                    return
                }
                window.libraryFilterSmokePhase = 4
                window.setAttributeFilters("completed", "video", true, true)
            } else if (window.libraryFilterSmokePhase === 4) {
                if (!window.filteredIdsMatch(["fixture-racer"])) {
                    console.error("BIGBOX_LIBRARY_FILTER_SMOKE_BAD_COMBINED")
                    Qt.exit(31)
                    return
                }
                window.libraryFilterSmokePhase = 5
                window.setAttributeFilters("any", "any", true, true)
            } else if (window.libraryFilterSmokePhase === 5) {
                if (!window.filteredIdsMatch(
                            ["fixture-adventure", "fixture-racer"])) {
                    console.error("BIGBOX_LIBRARY_FILTER_SMOKE_BAD_MISSING_ANY")
                    Qt.exit(31)
                    return
                }
                window.libraryFilterSmokePhase = 6
                window.setAttributeFilters("any", "banner-image", true, true)
            } else if (window.libraryFilterSmokePhase === 6) {
                if (!window.filteredIdsMatch(["fixture-adventure"])) {
                    console.error("BIGBOX_LIBRARY_FILTER_SMOKE_BAD_MISSING_TYPE")
                    Qt.exit(31)
                    return
                }
                window.libraryFilterSmokePhase = 7
                window.setAttributeFilters("hidden", "none", false, false)
            } else if (window.libraryFilterSmokePhase === 7) {
                if (!window.filteredIdsMatch(["fixture-puzzle"])) {
                    console.error("BIGBOX_LIBRARY_FILTER_SMOKE_BAD_HIDDEN_ONLY")
                    Qt.exit(31)
                    return
                }
                window.libraryFilterSmokePhase = 8
                window.setAttributeFilters("broken", "none", false, false)
            } else if (window.libraryFilterSmokePhase === 8) {
                if (!window.filteredIdsMatch(["fixture-racer"])) {
                    console.error("BIGBOX_LIBRARY_FILTER_SMOKE_BAD_BROKEN_ONLY")
                    Qt.exit(31)
                    return
                }
                if (controller.apply_game_attribute_filters(
                            "unknown-state", "none", false, false)) {
                    console.error("BIGBOX_LIBRARY_FILTER_SMOKE_ACCEPTED_UNKNOWN")
                    Qt.exit(31)
                    return
                }
                window.libraryFilterSmokePhase = 9
                window.setAttributeFilters("any", "none", false, false)
            } else if (window.libraryFilterSmokePhase === 9) {
                if (!window.filteredIdsMatch(["fixture-adventure"])
                        || !controller.report_library_filter_smoke_success()) {
                    console.error("BIGBOX_LIBRARY_FILTER_SMOKE_BAD_FINAL_STATE")
                    Qt.exit(31)
                    return
                }
                attributeFilterDrawer.close()
                window.libraryFilterSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 10000
        running: window.libraryFilterSmokeTest
                 && !window.libraryFilterSmokeFinished
        onTriggered: {
            console.error("BIGBOX_LIBRARY_FILTER_SMOKE_TIMEOUT phase="
                          + window.libraryFilterSmokePhase
                          + " drawer=" + attributeFilterDrawer.opened
                          + " filtered=" + controller.filtered_count
                          + " status=" + controller.status_message)
            Qt.exit(31)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.libraryOrderSmokeTest
                 && !window.libraryOrderSmokeFinished
        onTriggered: {
            if (controller.loading || controller.writing
                    || controller.library_path.length === 0)
                return
            if (window.libraryOrderSmokePhase === 0) {
                if (controller.game_sort !== "Title"
                        || controller.game_sort_descending
                        || !window.filteredIdsMatch(
                            ["fixture-adventure", "fixture-puzzle",
                             "fixture-racer"])) {
                    console.error(
                        "BIGBOX_LIBRARY_ORDER_SMOKE_BAD_PERSISTED_DEFAULT")
                    Qt.exit(32)
                    return
                }
                if (!controller.apply_game_sort("PlayCount", true)) {
                    console.error("BIGBOX_LIBRARY_ORDER_SMOKE_SORT_REJECTED")
                    Qt.exit(32)
                    return
                }
                window.libraryOrderSmokePhase = 1
            } else if (window.libraryOrderSmokePhase === 1) {
                if (!window.filteredIdsMatch(
                            ["fixture-racer", "fixture-adventure",
                             "fixture-puzzle"])
                        || controller.game_sort !== "PlayCount"
                        || !controller.game_sort_descending) {
                    console.error(
                        "BIGBOX_LIBRARY_ORDER_SMOKE_BAD_PLAY_COUNT_ORDER")
                    Qt.exit(32)
                    return
                }
                if (controller.apply_game_sort("UnknownSort", false)
                        || controller.game_sort !== "PlayCount"
                        || !controller.game_sort_descending) {
                    console.error(
                        "BIGBOX_LIBRARY_ORDER_SMOKE_ACCEPTED_UNKNOWN")
                    Qt.exit(32)
                    return
                }
                window.libraryOrderSmokeRandomRow =
                    controller.select_random_game("fixture-racer")
                if (window.libraryOrderSmokeRandomRow < 1
                        || controller.game_id_at(
                            window.libraryOrderSmokeRandomRow)
                           === "fixture-racer") {
                    console.error(
                        "BIGBOX_LIBRARY_ORDER_SMOKE_RANDOM_DID_NOT_AVOID")
                    Qt.exit(32)
                    return
                }
                controller.apply_filters("Fixture Racer", "")
                window.libraryOrderSmokePhase = 2
            } else if (window.libraryOrderSmokePhase === 2) {
                if (!window.filteredIdsMatch(["fixture-racer"])
                        || controller.select_random_game("fixture-racer") !== 0) {
                    console.error("BIGBOX_LIBRARY_ORDER_SMOKE_BAD_SINGLE_RANDOM")
                    Qt.exit(32)
                    return
                }
                controller.apply_filters("", "")
                window.libraryOrderSmokePhase = 3
            } else if (window.libraryOrderSmokePhase === 3) {
                if (!window.filteredIdsMatch(
                            ["fixture-racer", "fixture-adventure",
                             "fixture-puzzle"])
                        || !controller.report_library_order_smoke_success(
                            window.libraryOrderSmokeRandomRow,
                            "fixture-racer")) {
                    console.error("BIGBOX_LIBRARY_ORDER_SMOKE_BAD_FINAL_STATE")
                    Qt.exit(32)
                    return
                }
                window.libraryOrderSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 10000
        running: window.libraryOrderSmokeTest
                 && !window.libraryOrderSmokeFinished
        onTriggered: {
            console.error("BIGBOX_LIBRARY_ORDER_SMOKE_TIMEOUT phase="
                          + window.libraryOrderSmokePhase)
            Qt.exit(32)
        }
    }

    Timer {
        interval: 3000
        running: window.smokeTest
        onTriggered: {
            console.error("MODEL_ROLE_SMOKE_TIMEOUT rows=" + gameList.count)
            Qt.exit(4)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.navigationSmokeTest
        onTriggered: {
            if (window.navigationSmokePhase === 0 && !controller.loading
                    && controller.library_path.length > 0
                    && controller.big_box_navigation_entry_count === 3) {
                const category = window.bigBoxNavigationIndex(
                                   "category", "Fixture Category")
                const platform = window.bigBoxNavigationIndex(
                                   "platform", "Fixture Console")
                const playlist = window.bigBoxNavigationIndex(
                                   "playlist", "fixture-playlist")
                if (category < 0 || platform < 0 || playlist < 0
                        || navigationList.count !== 4
                        || controller.big_box_navigation_entry_depth_at(category) !== 0
                        || controller.big_box_navigation_entry_depth_at(platform) !== 1
                        || controller.big_box_navigation_entry_depth_at(playlist) !== 0
                        || controller.big_box_navigation_entry_game_count_at(category) !== 3
                        || controller.big_box_navigation_entry_game_count_at(platform) !== 3
                        || controller.big_box_navigation_entry_game_count_at(playlist) !== 1) {
                    console.error("BIGBOX_NAVIGATION_SMOKE_STRUCTURE_FAILED entries="
                                  + controller.big_box_navigation_entry_count
                                  + " category=" + category + " platform=" + platform
                                  + " playlist=" + playlist)
                    Qt.exit(8)
                    return
                }
                window.navigationSmokePhase = 1
                window.openNavigation()
            } else if (window.navigationSmokePhase === 1
                       && navigationDrawer.opened
                       && navigationList.activeFocus) {
                const playlist = window.bigBoxNavigationIndex(
                                   "playlist", "fixture-playlist")
                navigationList.currentIndex = playlist + 1
                window.navigationSmokePhase = 2
                window.activateNavigationRow(navigationList.currentIndex)
            } else if (window.navigationSmokePhase === 2
                       && controller.navigation_filter_kind === "playlist"
                       && controller.navigation_filter_key === "fixture-playlist"
                       && controller.filtered_count === 1) {
                if (controller.game_id_at(0) !== "fixture-adventure") {
                    console.error("BIGBOX_NAVIGATION_SMOKE_PLAYLIST_FAILED id="
                                  + controller.game_id_at(0))
                    Qt.exit(8)
                    return
                }
                window.navigationSmokePhase = 3
                window.activateNavigationRow(
                            window.bigBoxNavigationIndex(
                                "category", "Fixture Category") + 1)
            } else if (window.navigationSmokePhase === 3
                       && controller.navigation_filter_kind === "category"
                       && controller.navigation_filter_key === "Fixture Category"
                       && controller.filtered_count === 3) {
                window.navigationSmokePhase = 4
                window.activateNavigationRow(
                            window.bigBoxNavigationIndex(
                                "platform", "Fixture Console") + 1)
            } else if (window.navigationSmokePhase === 4
                       && controller.navigation_filter_kind === "platform"
                       && controller.navigation_filter_key === "Fixture Console"
                       && controller.filtered_count === 3) {
                window.navigationSmokePhase = 5
                window.activateNavigationRow(0)
            } else if (window.navigationSmokePhase === 5
                       && controller.navigation_filter_kind.length === 0
                       && controller.navigation_filter_key.length === 0
                       && controller.filtered_count === 3) {
                if (!controller.report_big_box_navigation_smoke_success()) {
                    console.error("BIGBOX_NAVIGATION_SMOKE_FINAL_STATE_FAILED")
                    Qt.exit(8)
                    return
                }
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 6000
        running: window.navigationSmokeTest
        onTriggered: {
            console.error("BIGBOX_NAVIGATION_SMOKE_TIMEOUT phase="
                          + window.navigationSmokePhase + " entries="
                          + controller.big_box_navigation_entry_count + " filtered="
                          + controller.filtered_count + " status="
                          + controller.status_message)
            Qt.exit(8)
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

    Rectangle {
        anchors.fill: parent
        gradient: Gradient {
            GradientStop { position: 0.0; color: "#17243b" }
            GradientStop { position: 0.62; color: "#090d14" }
            GradientStop { position: 1.0; color: "#050609" }
        }
    }

    Drawer {
        id: navigationDrawer
        width: Math.min(window.width * 0.42, 500)
        height: window.height
        edge: Qt.LeftEdge
        modal: true
        closePolicy: Popup.CloseOnPressOutside
        onOpened: navigationList.forceActiveFocus()
        onClosed: gameList.forceActiveFocus()

        background: Rectangle {
            color: "#101824"
            border.color: "#4679ad"
            border.width: 2
        }

        contentItem: ColumnLayout {
            spacing: 14
            anchors.fill: parent
            anchors.margins: 24

            Label {
                Layout.fillWidth: true
                text: "BROWSE LIBRARY"
                color: "#67b3ff"
                font.pixelSize: 26
                font.bold: true
                font.letterSpacing: 2
            }

            Label {
                Layout.fillWidth: true
                text: "Platforms, categories, and playlists"
                color: "#9badc4"
                font.pixelSize: 15
            }

            ListView {
                id: navigationList
                Layout.fillWidth: true
                Layout.fillHeight: true
                spacing: 6
                clip: true
                focus: true
                keyNavigationWraps: true
                model: controller.big_box_navigation_entry_count + 1
                Keys.onReturnPressed: function(event) {
                    window.activateNavigationRow(currentIndex)
                    event.accepted = true
                }
                Keys.onEnterPressed: function(event) {
                    window.activateNavigationRow(currentIndex)
                    event.accepted = true
                }
                Keys.onRightPressed: function(event) {
                    navigationDrawer.close()
                    gameList.forceActiveFocus()
                    event.accepted = true
                }

                delegate: ItemDelegate {
                    id: navigationDelegate
                    required property int index
                    readonly property int sourceIndex: index - 1
                    readonly property string entryKind: index === 0 ? "all"
                        : controller.big_box_navigation_entry_kind_at(sourceIndex)
                    readonly property string entryKey: index === 0 ? ""
                        : controller.big_box_navigation_entry_key_at(sourceIndex)
                    readonly property string entryName: index === 0 ? "All Games"
                        : controller.big_box_navigation_entry_name_at(sourceIndex)
                    readonly property int entryDepth: index === 0 ? 0
                        : controller.big_box_navigation_entry_depth_at(sourceIndex)
                    readonly property int entryCount: index === 0 ? controller.game_count
                        : controller.big_box_navigation_entry_game_count_at(sourceIndex)
                    readonly property bool activeEntry: index === 0
                        ? controller.navigation_filter_kind.length === 0
                        : controller.navigation_filter_kind === entryKind
                          && controller.navigation_filter_key === entryKey
                    width: ListView.view.width
                    height: 62
                    leftPadding: 18 + entryDepth * 22
                    rightPadding: 16
                    highlighted: navigationList.currentIndex === index
                    text: (entryKind === "category" ? "▾  "
                           : entryKind === "playlist" ? "≡  "
                           : entryKind === "platform" ? "▪  " : "◆  ")
                          + entryName + "    " + entryCount
                    font.pixelSize: 19
                    font.bold: activeEntry
                    Accessible.name: entryName + ", " + entryCount + " games"
                    onClicked: window.activateNavigationRow(index)

                    background: Rectangle {
                        radius: 7
                        color: navigationDelegate.highlighted
                               ? "#2f5680"
                               : navigationDelegate.activeEntry ? "#213d5c" : "transparent"
                        border.color: navigationDelegate.activeEntry
                                      ? "#67b3ff" : "transparent"
                        border.width: 2
                    }
                }
            }

            Label {
                Layout.fillWidth: true
                text: "↑ ↓  SELECT     ENTER  APPLY     →  GAMES"
                color: "#9badc4"
                font.pixelSize: 14
            }
        }
    }

    Drawer {
        id: attributeFilterDrawer
        width: Math.min(window.width * 0.42, 500)
        height: window.height
        edge: Qt.RightEdge
        modal: true
        closePolicy: Popup.CloseOnPressOutside
        onOpened: bigBoxStateFilterCombo.forceActiveFocus()
        onClosed: gameList.forceActiveFocus()

        background: Rectangle {
            color: "#101824"
            border.color: "#8b6fd6"
            border.width: 2
        }

        contentItem: ColumnLayout {
            spacing: 10
            anchors.fill: parent
            anchors.margins: 24

            Label {
                Layout.fillWidth: true
                text: "GAME FILTERS"
                color: "#b69cff"
                font.pixelSize: 26
                font.bold: true
                font.letterSpacing: 2
            }

            Label {
                Layout.fillWidth: true
                text: "State"
                color: "#b8c5d6"
                font.pixelSize: 16
                font.bold: true
            }
            ComboBox {
                id: bigBoxStateFilterCombo
                Layout.fillWidth: true
                Accessible.name: "BigBox game state filter"
                model: window.gameStateFilterChoices
                textRole: "label"
                valueRole: "key"
                currentIndex: 0
                font.pixelSize: 18
                KeyNavigation.down: bigBoxMissingMediaFilterCombo
            }

            Label {
                Layout.fillWidth: true
                text: "Missing media"
                color: "#b8c5d6"
                font.pixelSize: 16
                font.bold: true
            }
            ComboBox {
                id: bigBoxMissingMediaFilterCombo
                Layout.fillWidth: true
                Accessible.name: "BigBox missing media filter"
                model: window.missingMediaFilterChoices
                textRole: "label"
                valueRole: "key"
                currentIndex: 0
                font.pixelSize: 18
                KeyNavigation.up: bigBoxStateFilterCombo
                KeyNavigation.down: bigBoxSortCombo
            }

            Label {
                Layout.fillWidth: true
                text: "Arrange by"
                color: "#b8c5d6"
                font.pixelSize: 16
                font.bold: true
            }
            ComboBox {
                id: bigBoxSortCombo
                Layout.fillWidth: true
                Accessible.name: "BigBox arrange games by"
                model: window.gameSortChoices
                textRole: "label"
                valueRole: "key"
                currentIndex: 0
                font.pixelSize: 18
                KeyNavigation.up: bigBoxMissingMediaFilterCombo
                KeyNavigation.down: bigBoxSortDescendingCheck
            }
            CheckBox {
                id: bigBoxSortDescendingCheck
                Layout.fillWidth: true
                text: "Descending"
                Accessible.name: "BigBox sort games descending"
                font.pixelSize: 18
                KeyNavigation.up: bigBoxSortCombo
                KeyNavigation.down: bigBoxIncludeHiddenCheck
            }

            CheckBox {
                id: bigBoxIncludeHiddenCheck
                Layout.fillWidth: true
                text: "Show hidden games"
                Accessible.name: "Include hidden games"
                font.pixelSize: 18
                KeyNavigation.up: bigBoxSortDescendingCheck
                KeyNavigation.down: bigBoxIncludeBrokenCheck
            }
            CheckBox {
                id: bigBoxIncludeBrokenCheck
                Layout.fillWidth: true
                text: "Show broken games"
                Accessible.name: "Include broken games"
                font.pixelSize: 18
                KeyNavigation.up: bigBoxIncludeHiddenCheck
                KeyNavigation.down: applyBigBoxFiltersButton
            }

            Item { Layout.fillHeight: true }

            Button {
                id: applyBigBoxFiltersButton
                Layout.fillWidth: true
                text: "APPLY"
                Accessible.name: "Apply game filters"
                font.pixelSize: 18
                KeyNavigation.up: bigBoxIncludeBrokenCheck
                KeyNavigation.down: clearBigBoxFiltersButton
                onClicked: {
                    if (window.applyAttributeFilters()
                            && window.applyCurrentSort())
                        attributeFilterDrawer.close()
                }
            }
            Button {
                id: clearBigBoxFiltersButton
                Layout.fillWidth: true
                text: "CLEAR"
                Accessible.name: "Clear game filters"
                font.pixelSize: 18
                KeyNavigation.up: applyBigBoxFiltersButton
                KeyNavigation.down: closeBigBoxFiltersButton
                onClicked: {
                    window.setAttributeFilters("any", "none", false, false)
                    attributeFilterDrawer.close()
                }
            }
            Button {
                id: closeBigBoxFiltersButton
                Layout.fillWidth: true
                text: "CLOSE"
                font.pixelSize: 18
                KeyNavigation.up: clearBigBoxFiltersButton
                onClicked: attributeFilterDrawer.close()
            }

            Label {
                Layout.fillWidth: true
                text: "↑ ↓  SELECT     ENTER  APPLY     ESC  CLOSE"
                color: "#9badc4"
                font.pixelSize: 14
            }
        }
    }

    ColumnLayout {
        id: bigBoxContent
        anchors.fill: parent
        anchors.margins: 48
        spacing: 18

        RowLayout {
            Layout.fillWidth: true
            Label {
                Layout.fillWidth: true
                text: "BIGBOX"
                color: "#67b3ff"
                font.pixelSize: 34
                font.bold: true
                font.letterSpacing: 3
            }
            Label {
                text: controller.library_name
                color: "#b8c5d6"
                font.pixelSize: 20
            }
            Label {
                text: window.activeNavigationName.toUpperCase()
                      + "  •  " + controller.filtered_count
                color: "#67b3ff"
                font.pixelSize: 18
                font.bold: true
            }
        }

        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

            ListView {
                id: gameList
                anchors.fill: parent
                orientation: ListView.Horizontal
                spacing: 28
                clip: true
                focus: true
                keyNavigationWraps: true
                highlightRangeMode: ListView.StrictlyEnforceRange
                preferredHighlightBegin: width * 0.36
                preferredHighlightEnd: width * 0.64
                snapMode: ListView.SnapOneItem
                model: controller
                onCountChanged: {
                    if (count === 0) {
                        window.clearSelectedBigBoxGame()
                    } else if (currentIndex < 0) {
                        currentIndex = 0
                    }
                }
                onCurrentIndexChanged: {
                    if (currentIndex < 0)
                        window.clearSelectedBigBoxGame()
                }
                Keys.onReturnPressed: function(event) {
                    window.launchSelection()
                    event.accepted = true
                }
                Keys.onEnterPressed: function(event) {
                    window.launchSelection()
                    event.accepted = true
                }
                Keys.onUpPressed: function(event) {
                    window.openNavigation()
                    event.accepted = true
                }

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
                    required property string gameNotes
                    required property string gameDeveloper
                    required property string gamePublisher
                    required property string gameGenre
                    required property string gameVersion
                    required property real gamePlayTimeSeconds
                    required property string gameLastPlayedDate
                    required property string gameDateAdded
                    required property string gameDateModified
                    required property real gameCommunityStarRating
                    required property int gameCommunityStarRatingTotalVotes
                    required property int gameInstalledState
                    required property bool gameHidden
                    required property bool gameBroken
                    required property bool gamePortable
                    required property string gameVideoUrl
                    required property int gameDatabaseId
                    required property string gameAlternateNames
                    required property url gameFrontImageUrl
                    readonly property url gameBackImageUrl: {
                        const revision = controller.game_media_revision
                        return controller.game_box_back_url_for_game(gameId)
                    }
                    readonly property url displayedBoxSource:
                        coverImage.source
                    readonly property int displayedBoxStatus:
                        coverImage.status
                    readonly property real displayedBoxFlipAngle:
                        coverImage.flipAngle

                    function publishCurrentGame() {
                        if (!ListView.isCurrentItem)
                            return
                        if (window.selectedBigBoxGameId !== gameId)
                            window.selectedBigBoxGameBoxBackVisible = false
                        window.selectedBigBoxGameId = gameId
                        window.selectedBigBoxGameTitle = gameTitle
                        window.selectedBigBoxGamePlatform = gamePlatform
                        window.selectedBigBoxGameNotes = gameNotes
                        window.selectedBigBoxGameDeveloper = gameDeveloper
                        window.selectedBigBoxGamePublisher = gamePublisher
                        window.selectedBigBoxGameGenre = gameGenre
                        window.selectedBigBoxGameVersion = gameVersion
                        window.selectedBigBoxGameFavorite = gameFavorite
                        window.selectedBigBoxGameCompleted = gameCompleted
                        window.selectedBigBoxGamePlayCount = gamePlayCount
                        window.selectedBigBoxGameStarRating = gameStarRating
                        window.selectedBigBoxGamePlayTimeSeconds =
                            gamePlayTimeSeconds
                        window.selectedBigBoxGameCommunityRating =
                            gameCommunityStarRating
                        window.selectedBigBoxGameFrontImageUrl =
                            gameFrontImageUrl
                    }

                    Component.onCompleted: {
                        window.verifyModelRoles(
                            index, gameId, gameTitle, gamePlatform,
                            gameFavorite, gameCompleted, gamePlayCount,
                            gameStarRating,
                            gameAdditionalApplicationCount,
                            gamePlayTimeSeconds,
                            gameLastPlayedDate, gameDateAdded,
                            gameDateModified,
                            gameCommunityStarRating,
                            gameCommunityStarRatingTotalVotes,
                            gameInstalledState,
                            gameHidden, gameBroken,
                            gamePortable, gameVideoUrl,
                            gameDatabaseId,
                            gameAlternateNames,
                            gameFrontImageUrl,
                            gameList.count)
                        publishCurrentGame()
                    }
                    ListView.onIsCurrentItemChanged: publishCurrentGame()
                    width: 370
                    height: gameList.height - 20
                    radius: 14
                    scale: gameList.currentIndex === index ? 1.0 : 0.88
                    opacity: gameList.currentIndex === index ? 1.0 : 0.55
                    color: gameList.currentIndex === index ? "#253a59" : "#131a24"
                    border.color: gameFavorite ? "#f0c04a" : "#4775aa"
                    border.width: gameList.currentIndex === index ? 3 : 1

                    Behavior on scale { NumberAnimation { duration: 150 } }
                    Behavior on opacity { NumberAnimation { duration: 150 } }

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: 28
                        Label {
                            Layout.fillWidth: true
                            text: gamePlatform.toUpperCase()
                            color: "#7fbfff"
                            font.pixelSize: 17
                            font.bold: true
                            elide: Text.ElideRight
                        }
                        Item {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            Layout.minimumHeight: 260

                            Rectangle {
                                anchors.fill: parent
                                radius: 8
                                color: "#0c121b"
                                visible: coverImage.status !== Image.Ready
                                Label {
                                    anchors.centerIn: parent
                                    width: parent.width - 30
                                    text: gameTitle
                                    color: "#65758a"
                                    font.pixelSize: 26
                                    font.bold: true
                                    horizontalAlignment: Text.AlignHCenter
                                    wrapMode: Text.Wrap
                                }
                            }
                            BoxArtView {
                                id: coverImage
                                anchors.fill: parent
                                frontSource: gameFrontImageUrl
                                backSource: gameBackImageUrl
                                showingBack:
                                    gameList.currentIndex === index
                                    && window
                                       .selectedBigBoxGameBoxBackVisible
                                requestedSourceWidth: 600
                                requestedSourceHeight: 800
                                onStatusChanged: {
                                    if (!window.mediaSmokeTest
                                            || window.mediaSmokeFinished
                                            || index !== 0
                                            || status !== Image.Ready)
                                        return
                                    const sourceText = source.toString()
                                    const localPath =
                                        controller.local_path_from_url(
                                            sourceText)
                                    if (gameId !== "fixture-adventure"
                                            || controller.front_image_count !== 1
                                            || localPath.indexOf(
                                                "Fixture Adventure-01.svg") < 0) {
                                        console.error(
                                            "MEDIA_SMOKE_BAD_ART id=" + gameId
                                            + " images="
                                            + controller.front_image_count
                                            + " source=" + sourceText
                                            + " local=" + localPath)
                                        Qt.exit(46)
                                        return
                                    }
                                    if (!controller.report_media_smoke_success(
                                            index, sourceText)) {
                                        console.error(
                                            "MEDIA_SMOKE_CONTROLLER_REJECTED")
                                        Qt.exit(47)
                                        return
                                    }
                                    window.mediaSmokeFinished = true
                                    Qt.quit()
                                }
                            }
                        }
                        Label {
                            Layout.fillWidth: true
                            text: gameTitle
                            color: "white"
                            font.pixelSize: 28
                            font.bold: true
                            wrapMode: Text.Wrap
                        }
                        Label {
                            Layout.fillWidth: true
                            text: (gameFavorite ? "★ FAVORITE    " : "")
                                  + (gameCompleted ? "✓ COMPLETED" : "")
                            color: "#f0c04a"
                            font.pixelSize: 15
                            font.bold: true
                        }
                    }
                    MouseArea {
                        anchors.fill: parent
                        onClicked: gameList.currentIndex = index
                        onDoubleClicked: controller.launch_game(index, gameId)
                    }
                }
            }
        }

        RowLayout {
            Layout.fillWidth: true
            Button {
                text: "BROWSE: " + window.activeNavigationName.toUpperCase()
                enabled: !controller.loading && !controller.writing
                onClicked: window.openNavigation()
            }
            Button {
                text: "GAME FILTERS"
                enabled: !controller.loading && !controller.writing
                onClicked: window.openAttributeFilters()
            }
            Button {
                text: "RANDOM"
                enabled: controller.filtered_count > 0
                         && !controller.loading && !controller.writing
                onClicked: window.selectRandomGame()
            }
            Button {
                id: bigBoxGameDetailsButton
                text: "DETAILS"
                enabled: window.selectedBigBoxGameId.length > 0
                         && !controller.loading && !controller.writing
                onClicked: window.openGameDetails()
            }
            Button {
                id: bigBoxImagesButton
                text: "IMAGES"
                enabled: window.selectedBigBoxGameId.length > 0
                         && window.selectedBigBoxGameImageCount > 0
                         && !controller.loading && !controller.writing
                onClicked: window.openGameImages(-1)
            }
            Button {
                id: bigBoxModelButton
                text: "3D MODEL"
                visible: controller.big_box_show_game_menu_model
                enabled: visible
                         && window.selectedBigBoxGameId.length > 0
                         && window.selectedBigBoxGameFrontImageUrl
                            .toString().length > 0
                         && !controller.loading
                         && !controller.writing
                Accessible.name: "Show interactive 3D box model"

                function activate() {
                    if (!enabled)
                        return false
                    return window.openGameModel(bigBoxModelButton)
                }

                onClicked: activate()
            }
            Button {
                id: bigBoxFlipBoxButton
                text: window.selectedBigBoxGameBoxBackVisible
                      ? "SHOW FRONT" : "FLIP BOX"
                visible:
                    controller.big_box_show_game_menu_flip_box
                    && window.selectedBigBoxGameBackImageUrl
                    .toString().length > 0
                enabled: visible
                         && !controller.loading
                         && !controller.writing

                function activate() {
                    if (!enabled)
                        return false
                    return window.flipSelectedBox()
                }

                onClicked: activate()
            }
            Label {
                Layout.fillWidth: true
                text: gameList.count > 0
                      ? (gameList.currentIndex + 1) + " / " + gameList.count
                      : "No games"
                color: "#9badc4"
                font.pixelSize: 18
            }
            Button {
                text: "LAUNCH WITH…"
                visible: gameList.currentIndex >= 0
                         && controller.additional_application_count(
                             gameList.currentIndex,
                             controller.game_id_at(gameList.currentIndex)) > 0
                enabled: !controller.loading && !controller.writing
                         && !controller.launching
                         && !controller.launch_session_active
                onClicked: window.showLaunchWithSelection()
            }
            Label {
                text: controller.launching
                      ? "LAUNCHING…"
                      : "D  DETAILS     I  IMAGES     F  FLIP     ← →  GAMES     ENTER  PLAY"
                color: "#9badc4"
                font.pixelSize: 18
            }
        }
    }

    Popup {
        id: bigBoxGameDetails
        x: 0
        y: 0
        width: window.width
        height: window.height
        padding: 0
        modal: true
        dim: false
        focus: true
        closePolicy: Popup.NoAutoClose
        property string mediaGameId: window.selectedBigBoxGameId
        readonly property int mediaRevision: controller.game_media_revision
        readonly property int mediaCount: {
            const revision = mediaRevision
            return mediaGameId.length > 0
                ? controller.game_media_count_for_game(mediaGameId) : 0
        }
        property int selectedMediaIndex: -1
        readonly property string selectedMediaKind:
            selectedMediaIndex >= 0
            ? controller.game_media_kind_at(
                  mediaGameId, selectedMediaIndex) : ""
        readonly property string selectedMediaType:
            selectedMediaIndex >= 0
            ? controller.game_media_type_at(
                  mediaGameId, selectedMediaIndex) : ""
        readonly property url selectedMediaSource:
            selectedMediaIndex >= 0
            ? controller.game_media_url_at(
                  mediaGameId, selectedMediaIndex) : ""
        readonly property int mediaImageStatus:
            bigBoxSelectedMediaImage.status
        readonly property int mediaPlaybackState:
            bigBoxMediaPlayer.playbackState
        readonly property int mediaStatus:
            bigBoxMediaPlayer.mediaStatus
        readonly property var mediaError:
            bigBoxMediaPlayer.error
        readonly property real mediaDuration:
            bigBoxMediaPlayer.duration

        function resetMediaSelection() {
            bigBoxMediaPlayer.stop()
            selectedMediaIndex =
                mediaGameId.length > 0
                ? controller.game_media_default_index(mediaGameId) : -1
            if (selectedMediaIndex >= 0)
                bigBoxMediaThumbnailList.positionViewAtIndex(
                    selectedMediaIndex, ListView.Contain)
        }

        function selectMedia(index) {
            if (index < 0 || index >= mediaCount)
                return false
            bigBoxMediaPlayer.stop()
            selectedMediaIndex = index
            bigBoxMediaThumbnailList.positionViewAtIndex(
                index, ListView.Contain)
            return true
        }

        function selectPreviousMedia() {
            if (mediaCount === 0)
                return false
            return selectMedia(
                selectedMediaIndex <= 0
                ? mediaCount - 1 : selectedMediaIndex - 1)
        }

        function selectNextMedia() {
            if (mediaCount === 0)
                return false
            return selectMedia(
                selectedMediaIndex < 0
                || selectedMediaIndex + 1 >= mediaCount
                ? 0 : selectedMediaIndex + 1)
        }

        function togglePlayback() {
            if (selectedMediaKind !== "video")
                return false
            if (bigBoxMediaPlayer.playbackState
                    === MediaPlayer.PlayingState) {
                bigBoxMediaPlayer.pause()
            } else {
                if (bigBoxMediaPlayer.mediaStatus
                        === MediaPlayer.EndOfMedia)
                    bigBoxMediaPlayer.position = 0
                bigBoxMediaPlayer.play()
            }
            return true
        }

        function clickMediaThumbnailForSmoke(index) {
            const item = bigBoxMediaThumbnailList.itemAtIndex(index)
            if (!item)
                return false
            // qmllint disable missing-property
            item["clickForSmoke"]()
            // qmllint enable missing-property
            return true
        }

        onOpened: {
            resetMediaSelection()
            Qt.callLater(function() {
                bigBoxGameDetailsContent.forceActiveFocus()
            })
        }
        onClosed: {
            bigBoxMediaPlayer.stop()
            gameList.forceActiveFocus()
        }
        onMediaGameIdChanged: {
            if (opened)
                Qt.callLater(resetMediaSelection)
        }
        onMediaRevisionChanged: {
            if (opened)
                Qt.callLater(resetMediaSelection)
        }

        background: Rectangle {
            gradient: Gradient {
                GradientStop {
                    position: 0
                    color: "#17243b"
                }
                GradientStop {
                    position: 0.58
                    color: "#090d14"
                }
                GradientStop {
                    position: 1
                    color: "#050609"
                }
            }
        }

        contentItem: FocusScope {
            id: bigBoxGameDetailsContent
            focus: true

            Keys.onLeftPressed: function(event) {
                bigBoxGameDetails.selectPreviousMedia()
                event.accepted = true
            }
            Keys.onRightPressed: function(event) {
                bigBoxGameDetails.selectNextMedia()
                event.accepted = true
            }
            Keys.onSpacePressed: function(event) {
                bigBoxGameDetails.togglePlayback()
                event.accepted = true
            }
            Keys.onEscapePressed: function(event) {
                bigBoxGameDetails.close()
                event.accepted = true
            }
            Keys.onReturnPressed: function(event) {
                bigBoxGameDetails.close()
                Qt.callLater(window.launchSelection)
                event.accepted = true
            }
            Keys.onEnterPressed: function(event) {
                bigBoxGameDetails.close()
                Qt.callLater(window.launchSelection)
                event.accepted = true
            }
            Keys.onPressed: function(event) {
                if (event.key === Qt.Key_Back) {
                    bigBoxGameDetails.close()
                    event.accepted = true
                }
            }

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 38
                spacing: 16

                RowLayout {
                    Layout.fillWidth: true
                    spacing: 18

                    Button {
                        id: bigBoxGameDetailsBackButton
                        text: "‹  BACK"
                        onClicked: bigBoxGameDetails.close()
                    }
                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 2
                        Label {
                            Layout.fillWidth: true
                            text: window.selectedBigBoxGameTitle
                            color: "#ffffff"
                            font.pixelSize: 34
                            font.bold: true
                            elide: Text.ElideRight
                        }
                        Label {
                            Layout.fillWidth: true
                            text: window.selectedBigBoxGamePlatform.toUpperCase()
                            color: "#67b3ff"
                            font.pixelSize: 17
                            font.bold: true
                            font.letterSpacing: 2
                            elide: Text.ElideRight
                        }
                    }
                    Label {
                        text: bigBoxGameDetails.mediaCount > 0
                              ? (bigBoxGameDetails.selectedMediaIndex + 1)
                                + " / " + bigBoxGameDetails.mediaCount
                              : "NO MEDIA"
                        color: "#a9bdd6"
                        font.pixelSize: 18
                        font.bold: true
                    }
                }

                RowLayout {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    spacing: 24

                    ColumnLayout {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        Layout.preferredWidth: 780
                        spacing: 10

                        Rectangle {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            Layout.minimumHeight: 330
                            radius: 12
                            color: "#080c12"
                            border.color: "#30445d"
                            border.width: 2
                            clip: true

                            Image {
                                id: bigBoxSelectedMediaImage
                                anchors.fill: parent
                                anchors.margins: 8
                                visible:
                                    bigBoxGameDetails.selectedMediaKind
                                    === "image"
                                source: visible
                                        ? bigBoxGameDetails
                                          .selectedMediaSource : ""
                                asynchronous: true
                                cache: true
                                fillMode: Image.PreserveAspectFit
                                sourceSize.width: 1280
                                sourceSize.height: 960
                            }

                            AudioOutput {
                                id: bigBoxMediaAudio
                                muted: window.gameDetailsMediaSmokeTest
                                       || window.imageViewerSmokeTest
                            }

                            MediaPlayer {
                                id: bigBoxMediaPlayer
                                source:
                                    bigBoxGameDetails.selectedMediaKind
                                    === "video"
                                    ? bigBoxGameDetails
                                      .selectedMediaSource : ""
                                audioOutput: bigBoxMediaAudio
                                videoOutput: bigBoxVideoOutput
                                onSourceChanged: {
                                    if (source.toString().length > 0
                                            && bigBoxGameDetails.opened
                                            && controller
                                               .details_auto_play_video)
                                        Qt.callLater(play)
                                }
                                onErrorOccurred: function(error,
                                                          errorString) {
                                    console.warn(
                                        "BigBox selected-game media error "
                                        + error + ": " + errorString)
                                }
                            }

                            VideoOutput {
                                id: bigBoxVideoOutput
                                anchors.fill: parent
                                anchors.margins: 8
                                visible:
                                    bigBoxGameDetails.selectedMediaKind
                                    === "video"
                                fillMode: VideoOutput.PreserveAspectFit
                            }

                            Label {
                                anchors.centerIn: parent
                                width: parent.width - 40
                                visible:
                                    bigBoxGameDetails.mediaCount === 0
                                text: "No images or videos are available for this game."
                                color: "#6f829a"
                                font.pixelSize: 22
                                horizontalAlignment: Text.AlignHCenter
                                wrapMode: Text.Wrap
                            }

                            Rectangle {
                                anchors.left: parent.left
                                anchors.right: parent.right
                                anchors.bottom: parent.bottom
                                height: 44
                                color: "#d10b1118"
                                visible:
                                    bigBoxGameDetails.mediaCount > 0

                                RowLayout {
                                    anchors.fill: parent
                                    anchors.leftMargin: 14
                                    anchors.rightMargin: 14
                                    Label {
                                        Layout.fillWidth: true
                                        text:
                                            bigBoxGameDetails
                                            .selectedMediaType
                                        color: "#ffffff"
                                        font.pixelSize: 16
                                        font.bold: true
                                        elide: Text.ElideRight
                                    }
                                    Label {
                                        text:
                                            bigBoxGameDetails
                                            .selectedMediaKind
                                            .toUpperCase()
                                        color: "#7fbfff"
                                        font.pixelSize: 14
                                        font.bold: true
                                    }
                                }
                            }
                        }

                        ListView {
                            id: bigBoxMediaThumbnailList
                            Layout.fillWidth: true
                            Layout.preferredHeight: 94
                            orientation: ListView.Horizontal
                            spacing: 10
                            clip: true
                            model: bigBoxGameDetails.mediaCount
                            currentIndex:
                                bigBoxGameDetails.selectedMediaIndex

                            delegate: Rectangle {
                                id: bigBoxMediaThumbnail
                                required property int index
                                width: 122
                                height: 90
                                radius: 7
                                color:
                                    ListView.isCurrentItem
                                    ? "#244d73" : "#0e151f"
                                border.color:
                                    ListView.isCurrentItem
                                    ? "#67b7ff" : "#34475d"
                                border.width:
                                    ListView.isCurrentItem ? 3 : 1
                                readonly property string kind:
                                    controller.game_media_kind_at(
                                        bigBoxGameDetails.mediaGameId,
                                        index)
                                readonly property string mediaType:
                                    controller.game_media_type_at(
                                        bigBoxGameDetails.mediaGameId,
                                        index)
                                readonly property url mediaSource:
                                    controller.game_media_url_at(
                                        bigBoxGameDetails.mediaGameId,
                                        index)

                                function clickForSmoke() {
                                    bigBoxMediaThumbnailMouse.clicked(null)
                                }

                                Image {
                                    anchors.fill: parent
                                    anchors.margins: 5
                                    visible:
                                        bigBoxMediaThumbnail.kind
                                        === "image"
                                    source:
                                        visible
                                        ? bigBoxMediaThumbnail.mediaSource : ""
                                    asynchronous: true
                                    cache: true
                                    fillMode: Image.PreserveAspectCrop
                                    sourceSize.width: 244
                                    sourceSize.height: 180
                                }
                                Rectangle {
                                    anchors.fill: parent
                                    anchors.margins: 5
                                    visible:
                                        bigBoxMediaThumbnail.kind
                                        === "video"
                                    color: "#17283b"
                                    Label {
                                        anchors.centerIn: parent
                                        text: "▶"
                                        color: "#79c0ff"
                                        font.pixelSize: 30
                                    }
                                }
                                Rectangle {
                                    anchors.left: parent.left
                                    anchors.right: parent.right
                                    anchors.bottom: parent.bottom
                                    anchors.margins: 5
                                    height: 25
                                    color: "#d00b1118"
                                    Label {
                                        anchors.fill: parent
                                        anchors.leftMargin: 6
                                        anchors.rightMargin: 6
                                        text:
                                            bigBoxMediaThumbnail.mediaType
                                        color: "#ffffff"
                                        font.pixelSize: 10
                                        verticalAlignment:
                                            Text.AlignVCenter
                                        elide: Text.ElideRight
                                    }
                                }
                                MouseArea {
                                    id: bigBoxMediaThumbnailMouse
                                    anchors.fill: parent
                                    onClicked:
                                        bigBoxGameDetails.selectMedia(
                                            bigBoxMediaThumbnail.index)
                                }
                            }
                        }

                        RowLayout {
                            Layout.fillWidth: true
                            spacing: 10
                            Button {
                                id: bigBoxPreviousMediaButton
                                text: "‹  PREVIOUS"
                                enabled:
                                    bigBoxGameDetails.mediaCount > 1
                                onClicked:
                                    bigBoxGameDetails
                                    .selectPreviousMedia()
                            }
                            Button {
                                id: bigBoxViewImageButton
                                Layout.fillWidth: true
                                visible:
                                    bigBoxGameDetails.selectedMediaKind
                                    === "image"
                                text: "VIEW IMAGE"
                                onClicked:
                                    window.openGameImages(
                                        bigBoxGameDetails
                                        .selectedMediaIndex)
                            }
                            Button {
                                id: bigBoxMediaPlayPauseButton
                                Layout.fillWidth: true
                                visible:
                                    bigBoxGameDetails.selectedMediaKind
                                    === "video"
                                text:
                                    bigBoxMediaPlayer.playbackState
                                    === MediaPlayer.PlayingState
                                    ? "PAUSE VIDEO" : "PLAY VIDEO"
                                onClicked:
                                    bigBoxGameDetails.togglePlayback()
                            }
                            Button {
                                visible:
                                    bigBoxGameDetails.selectedMediaKind
                                    === "video"
                                text: bigBoxMediaAudio.muted
                                      ? "UNMUTE" : "MUTE"
                                onClicked:
                                    bigBoxMediaAudio.muted =
                                        !bigBoxMediaAudio.muted
                            }
                            Button {
                                id: bigBoxNextMediaButton
                                text: "NEXT  ›"
                                enabled:
                                    bigBoxGameDetails.mediaCount > 1
                                onClicked:
                                    bigBoxGameDetails.selectNextMedia()
                            }
                        }
                    }

                    Rectangle {
                        Layout.fillHeight: true
                        Layout.preferredWidth: 390
                        Layout.minimumWidth: 310
                        radius: 12
                        color: "#c40d1520"
                        border.color: "#30445d"
                        border.width: 1

                        ColumnLayout {
                            anchors.fill: parent
                            anchors.margins: 22
                            spacing: 12

                            Label {
                                Layout.fillWidth: true
                                text: "GAME DETAILS"
                                color: "#67b3ff"
                                font.pixelSize: 22
                                font.bold: true
                                font.letterSpacing: 2
                            }
                            Label {
                                Layout.fillWidth: true
                                text:
                                    (window.selectedBigBoxGameFavorite
                                     ? "★ FAVORITE    " : "")
                                    + (window.selectedBigBoxGameCompleted
                                       ? "✓ COMPLETED" : "")
                                visible: text.length > 0
                                color: "#f0c04a"
                                font.pixelSize: 15
                                font.bold: true
                            }
                            Label {
                                Layout.fillWidth: true
                                text:
                                    "Rating  "
                                    + window.selectedBigBoxGameStarRating
                                    + " / 5"
                                    + (window
                                       .selectedBigBoxGameCommunityRating > 0
                                       ? "    Community  "
                                         + window
                                           .selectedBigBoxGameCommunityRating
                                           .toFixed(2)
                                       : "")
                                color: "#dce7f5"
                                font.pixelSize: 16
                                wrapMode: Text.Wrap
                            }
                            Label {
                                Layout.fillWidth: true
                                text:
                                    "Played  "
                                    + window.selectedBigBoxGamePlayCount
                                    + " times  •  "
                                    + window.formatPlayTime(
                                        window
                                        .selectedBigBoxGamePlayTimeSeconds)
                                color: "#a9bdd6"
                                font.pixelSize: 15
                                wrapMode: Text.Wrap
                            }

                            Rectangle {
                                Layout.fillWidth: true
                                height: 1
                                color: "#30445d"
                            }

                            GridLayout {
                                Layout.fillWidth: true
                                columns: 2
                                columnSpacing: 10
                                rowSpacing: 8

                                Label {
                                    text: "Developer"
                                    color: "#71869f"
                                    font.pixelSize: 14
                                }
                                Label {
                                    Layout.fillWidth: true
                                    text:
                                        window.selectedBigBoxGameDeveloper
                                        .length > 0
                                        ? window
                                          .selectedBigBoxGameDeveloper : "—"
                                    color: "#dce7f5"
                                    font.pixelSize: 14
                                    elide: Text.ElideRight
                                }
                                Label {
                                    text: "Publisher"
                                    color: "#71869f"
                                    font.pixelSize: 14
                                }
                                Label {
                                    Layout.fillWidth: true
                                    text:
                                        window.selectedBigBoxGamePublisher
                                        .length > 0
                                        ? window
                                          .selectedBigBoxGamePublisher : "—"
                                    color: "#dce7f5"
                                    font.pixelSize: 14
                                    elide: Text.ElideRight
                                }
                                Label {
                                    text: "Genre"
                                    color: "#71869f"
                                    font.pixelSize: 14
                                }
                                Label {
                                    Layout.fillWidth: true
                                    text:
                                        window.selectedBigBoxGameGenre
                                        .length > 0
                                        ? window
                                          .selectedBigBoxGameGenre : "—"
                                    color: "#dce7f5"
                                    font.pixelSize: 14
                                    elide: Text.ElideRight
                                }
                                Label {
                                    text: "Version"
                                    color: "#71869f"
                                    font.pixelSize: 14
                                }
                                Label {
                                    Layout.fillWidth: true
                                    text:
                                        window.selectedBigBoxGameVersion
                                        .length > 0
                                        ? window
                                          .selectedBigBoxGameVersion : "—"
                                    color: "#dce7f5"
                                    font.pixelSize: 14
                                    elide: Text.ElideRight
                                }
                            }

                            Label {
                                Layout.fillWidth: true
                                text: "DESCRIPTION"
                                color: "#67b3ff"
                                font.pixelSize: 16
                                font.bold: true
                            }
                            ScrollView {
                                Layout.fillWidth: true
                                Layout.fillHeight: true
                                clip: true
                                contentWidth: availableWidth

                                Label {
                                    width: parent.width
                                    text:
                                        window.selectedBigBoxGameNotes
                                        .length > 0
                                        ? window.selectedBigBoxGameNotes
                                        : "No description is available."
                                    color: "#c7d5e5"
                                    font.pixelSize: 16
                                    wrapMode: Text.Wrap
                                }
                            }
                        }
                    }
                }

                RowLayout {
                    Layout.fillWidth: true
                    Label {
                        Layout.fillWidth: true
                        text:
                            "← →  MEDIA     SPACE  PLAY / PAUSE"
                            + "     ENTER  PLAY GAME     ESC  BACK"
                        color: "#9badc4"
                        font.pixelSize: 16
                    }
                }
            }
        }
    }

    Popup {
        id: bigBoxImageViewer
        x: 0
        y: 0
        width: window.width
        height: window.height
        padding: 0
        modal: true
        dim: false
        focus: true
        closePolicy: Popup.NoAutoClose
        property string gameId: ""
        readonly property int mediaRevision: controller.game_media_revision
        readonly property int imageCount: {
            const revision = mediaRevision
            return gameId.length > 0
                ? controller.game_image_count_for_game(gameId) : 0
        }
        property int selectedImageIndex: -1
        readonly property int selectedMediaIndex:
            selectedImageIndex >= 0
            ? controller.game_image_media_index_at(
                  gameId, selectedImageIndex) : -1
        readonly property string selectedMediaType:
            selectedMediaIndex >= 0
            ? controller.game_media_type_at(
                  gameId, selectedMediaIndex) : ""
        readonly property url selectedMediaSource:
            selectedMediaIndex >= 0
            ? controller.game_media_url_at(
                  gameId, selectedMediaIndex) : ""
        readonly property int imageStatus: bigBoxFullscreenImage.status
        readonly property real minimumZoom: 1.0
        readonly property real maximumZoom: 4.0
        readonly property real zoomStep: 0.25
        property real zoomFactor: minimumZoom
        property real panX: 0
        property real panY: 0
        readonly property real panLimitX:
            Math.max(0, (bigBoxFullscreenImage.paintedWidth
                         * zoomFactor - bigBoxImageViewport.width) / 2)
        readonly property real panLimitY:
            Math.max(0, (bigBoxFullscreenImage.paintedHeight
                         * zoomFactor - bigBoxImageViewport.height) / 2)

        function clamp(value, minimum, maximum) {
            return Math.min(maximum, Math.max(minimum, value))
        }

        function resetView() {
            zoomFactor = minimumZoom
            panX = 0
            panY = 0
        }

        function clampPan() {
            panX = clamp(panX, -panLimitX, panLimitX)
            panY = clamp(panY, -panLimitY, panLimitY)
        }

        function setZoom(value) {
            zoomFactor = clamp(value, minimumZoom, maximumZoom)
            if (zoomFactor <= minimumZoom) {
                panX = 0
                panY = 0
            } else {
                clampPan()
            }
            return zoomFactor
        }

        function panBy(horizontal, vertical) {
            if (zoomFactor <= minimumZoom)
                return false
            panX = clamp(panX + horizontal, -panLimitX, panLimitX)
            panY = clamp(panY + vertical, -panLimitY, panLimitY)
            return true
        }

        function selectImage(index) {
            if (index < 0 || index >= imageCount)
                return false
            selectedImageIndex = index
            resetView()
            return true
        }

        function selectPreviousImage() {
            if (imageCount === 0)
                return false
            return selectImage(
                selectedImageIndex <= 0
                ? imageCount - 1 : selectedImageIndex - 1)
        }

        function selectNextImage() {
            if (imageCount === 0)
                return false
            return selectImage(
                selectedImageIndex < 0
                || selectedImageIndex + 1 >= imageCount
                ? 0 : selectedImageIndex + 1)
        }

        function imageIndexForMedia(mediaIndex) {
            if (mediaIndex < 0)
                return -1
            for (let index = 0; index < imageCount; ++index) {
                if (controller.game_image_media_index_at(
                        gameId, index) === mediaIndex)
                    return index
            }
            return -1
        }

        function openForGame(requestedGameId, preferredMediaIndex) {
            if (requestedGameId.length === 0
                    || controller.game_image_count_for_game(
                        requestedGameId) === 0)
                return false
            gameId = requestedGameId
            const preferredImageIndex =
                imageIndexForMedia(preferredMediaIndex)
            selectedImageIndex =
                preferredImageIndex >= 0 ? preferredImageIndex : 0
            resetView()
            bigBoxMediaPlayer.stop()
            open()
            return true
        }

        onOpened: Qt.callLater(function() {
            bigBoxImageViewerContent.forceActiveFocus()
        })
        onClosed: {
            resetView()
            if (bigBoxGameDetails.opened)
                bigBoxGameDetailsContent.forceActiveFocus()
            else
                gameList.forceActiveFocus()
        }
        onImageCountChanged: {
            if (opened && (selectedImageIndex < 0
                           || selectedImageIndex >= imageCount)) {
                if (imageCount > 0)
                    selectImage(0)
                else
                    close()
            }
        }
        onPanLimitXChanged: clampPan()
        onPanLimitYChanged: clampPan()

        background: Rectangle {
            color: "#030508"
        }

        contentItem: FocusScope {
            id: bigBoxImageViewerContent
            focus: true
            Keys.priority: Keys.BeforeItem
            Keys.onPressed: function(event) {
                if (event.key === Qt.Key_Escape
                        || event.key === Qt.Key_Back) {
                    bigBoxImageViewer.close()
                } else if (event.key === Qt.Key_PageUp) {
                    bigBoxImageViewer.selectPreviousImage()
                } else if (event.key === Qt.Key_PageDown
                           || event.key === Qt.Key_Return
                           || event.key === Qt.Key_Enter) {
                    bigBoxImageViewer.selectNextImage()
                } else if (event.key === Qt.Key_Left) {
                    if (bigBoxImageViewer.zoomFactor
                            > bigBoxImageViewer.minimumZoom)
                        bigBoxImageViewer.panBy(64, 0)
                    else
                        bigBoxImageViewer.selectPreviousImage()
                } else if (event.key === Qt.Key_Right) {
                    if (bigBoxImageViewer.zoomFactor
                            > bigBoxImageViewer.minimumZoom)
                        bigBoxImageViewer.panBy(-64, 0)
                    else
                        bigBoxImageViewer.selectNextImage()
                } else if (event.key === Qt.Key_Up) {
                    bigBoxImageViewer.panBy(0, 64)
                } else if (event.key === Qt.Key_Down) {
                    bigBoxImageViewer.panBy(0, -64)
                } else if (event.key === Qt.Key_Plus
                           || event.key === Qt.Key_Equal) {
                    bigBoxImageViewer.setZoom(
                        bigBoxImageViewer.zoomFactor
                        + bigBoxImageViewer.zoomStep)
                } else if (event.key === Qt.Key_Minus) {
                    bigBoxImageViewer.setZoom(
                        bigBoxImageViewer.zoomFactor
                        - bigBoxImageViewer.zoomStep)
                } else if (event.key === Qt.Key_0) {
                    bigBoxImageViewer.resetView()
                } else {
                    return
                }
                event.accepted = true
            }

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 24
                spacing: 14

                RowLayout {
                    Layout.fillWidth: true
                    spacing: 14

                    Button {
                        id: bigBoxImageViewerBackButton
                        text: "‹  BACK"
                        onClicked: bigBoxImageViewer.close()
                    }
                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 1
                        Label {
                            Layout.fillWidth: true
                            text: window.selectedBigBoxGameTitle
                            color: "#ffffff"
                            font.pixelSize: 28
                            font.bold: true
                            elide: Text.ElideRight
                        }
                        Label {
                            Layout.fillWidth: true
                            text: bigBoxImageViewer.selectedMediaType
                            color: "#67b3ff"
                            font.pixelSize: 16
                            font.bold: true
                            font.letterSpacing: 1
                            elide: Text.ElideRight
                        }
                    }
                    Label {
                        text: bigBoxImageViewer.imageCount > 0
                              ? (bigBoxImageViewer.selectedImageIndex + 1)
                                + " / " + bigBoxImageViewer.imageCount
                              : "NO IMAGES"
                        color: "#c7d5e5"
                        font.pixelSize: 18
                        font.bold: true
                    }
                }

                Rectangle {
                    id: bigBoxImageViewport
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    color: "#080a0e"
                    border.color: "#26384d"
                    border.width: 1
                    clip: true

                    Item {
                        id: bigBoxImageLayer
                        x: bigBoxImageViewer.panX
                        y: bigBoxImageViewer.panY
                        width: bigBoxImageViewport.width
                        height: bigBoxImageViewport.height

                        Image {
                            id: bigBoxFullscreenImage
                            anchors.fill: parent
                            source: bigBoxImageViewer.opened
                                    ? bigBoxImageViewer
                                      .selectedMediaSource : ""
                            asynchronous: true
                            cache: true
                            smooth: true
                            mipmap: true
                            fillMode: Image.PreserveAspectFit
                            transformOrigin: Item.Center
                            scale: bigBoxImageViewer.zoomFactor
                        }
                    }

                    MouseArea {
                        id: bigBoxImageDragArea
                        anchors.fill: parent
                        acceptedButtons: Qt.LeftButton
                        property real lastX: 0
                        property real lastY: 0
                        onPressed: function(mouse) {
                            lastX = mouse.x
                            lastY = mouse.y
                        }
                        onPositionChanged: function(mouse) {
                            if (!pressed
                                    || bigBoxImageViewer.zoomFactor
                                       <= bigBoxImageViewer.minimumZoom)
                                return
                            bigBoxImageViewer.panBy(
                                mouse.x - lastX, mouse.y - lastY)
                            lastX = mouse.x
                            lastY = mouse.y
                        }
                        onDoubleClicked: {
                            if (bigBoxImageViewer.zoomFactor
                                    > bigBoxImageViewer.minimumZoom)
                                bigBoxImageViewer.resetView()
                            else
                                bigBoxImageViewer.setZoom(2)
                        }
                        onWheel: function(wheel) {
                            bigBoxImageViewer.setZoom(
                                bigBoxImageViewer.zoomFactor
                                + (wheel.angleDelta.y >= 0
                                   ? bigBoxImageViewer.zoomStep
                                   : -bigBoxImageViewer.zoomStep))
                            wheel.accepted = true
                        }
                    }
                }

                RowLayout {
                    Layout.fillWidth: true
                    spacing: 8

                    Button {
                        id: bigBoxImagePreviousButton
                        text: "‹  PREVIOUS IMAGE"
                        enabled: bigBoxImageViewer.imageCount > 1
                        onClicked:
                            bigBoxImageViewer.selectPreviousImage()
                    }
                    Button {
                        id: bigBoxImageNextButton
                        text: "NEXT IMAGE  ›"
                        enabled: bigBoxImageViewer.imageCount > 1
                        onClicked: bigBoxImageViewer.selectNextImage()
                    }
                    Label {
                        Layout.fillWidth: true
                        Layout.minimumWidth: 0
                        text: "ENTER / PAGE  SWITCH    DRAG / ARROWS  PAN"
                        color: "#7f93aa"
                        font.pixelSize: 12
                        horizontalAlignment: Text.AlignHCenter
                        elide: Text.ElideRight
                        clip: true
                    }
                    Button {
                        id: bigBoxImageZoomOutButton
                        Layout.minimumWidth: 44
                        Layout.preferredWidth: 44
                        Layout.maximumWidth: 44
                        text: "−"
                        enabled: bigBoxImageViewer.zoomFactor
                                 > bigBoxImageViewer.minimumZoom
                        onClicked:
                            bigBoxImageViewer.setZoom(
                                bigBoxImageViewer.zoomFactor
                                - bigBoxImageViewer.zoomStep)
                    }
                    Button {
                        id: bigBoxImageFitButton
                        Layout.minimumWidth: 96
                        Layout.preferredWidth: 96
                        Layout.maximumWidth: 96
                        text: Math.round(
                                  bigBoxImageViewer.zoomFactor * 100)
                              + "%  FIT"
                        enabled: bigBoxImageViewer.zoomFactor
                                 > bigBoxImageViewer.minimumZoom
                        onClicked: bigBoxImageViewer.resetView()
                    }
                    Button {
                        id: bigBoxImageZoomInButton
                        Layout.minimumWidth: 44
                        Layout.preferredWidth: 44
                        Layout.maximumWidth: 44
                        text: "+"
                        enabled: bigBoxImageViewer.zoomFactor
                                 < bigBoxImageViewer.maximumZoom
                        onClicked:
                            bigBoxImageViewer.setZoom(
                                bigBoxImageViewer.zoomFactor
                                + bigBoxImageViewer.zoomStep)
                    }
                    Button {
                        Layout.minimumWidth: 44
                        Layout.preferredWidth: 44
                        Layout.maximumWidth: 44
                        text: "←"
                        enabled: bigBoxImageViewer.zoomFactor
                                 > bigBoxImageViewer.minimumZoom
                        onClicked: bigBoxImageViewer.panBy(64, 0)
                    }
                    Button {
                        Layout.minimumWidth: 44
                        Layout.preferredWidth: 44
                        Layout.maximumWidth: 44
                        text: "↑"
                        enabled: bigBoxImageViewer.zoomFactor
                                 > bigBoxImageViewer.minimumZoom
                        onClicked: bigBoxImageViewer.panBy(0, 64)
                    }
                    Button {
                        id: bigBoxImagePanDownButton
                        Layout.minimumWidth: 44
                        Layout.preferredWidth: 44
                        Layout.maximumWidth: 44
                        text: "↓"
                        enabled: bigBoxImageViewer.zoomFactor
                                 > bigBoxImageViewer.minimumZoom
                        onClicked: bigBoxImageViewer.panBy(0, -64)
                    }
                    Button {
                        Layout.minimumWidth: 44
                        Layout.preferredWidth: 44
                        Layout.maximumWidth: 44
                        text: "→"
                        enabled: bigBoxImageViewer.zoomFactor
                                 > bigBoxImageViewer.minimumZoom
                        onClicked: bigBoxImageViewer.panBy(-64, 0)
                    }
                }
            }
        }
    }

    BoxModelViewer {
        id: bigBoxModelViewer
        parent: Overlay.overlay
        x: 0
        y: 0
        width: window.width
        height: window.height
        controller: controller
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
        anchors.margins: 22
        z: 9000
        visible: controller.pause_screen_available
                 && !controller.pause_screen_active
        text: "PAUSE GAME  CTRL+P"
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

    Dialog {
        id: launchWithDialog
        anchors.centerIn: parent
        modal: true
        title: "LAUNCH WITH"
        standardButtons: Dialog.Cancel
        property int modelRow: -1
        property string gameId: ""
        property int applicationCount: 0

        function prepare(row, id, count) {
            modelRow = row
            gameId = id
            applicationCount = count
            open()
            applicationList.forceActiveFocus()
        }

        contentItem: ListView {
            id: applicationList
            implicitWidth: 520
            implicitHeight: Math.min(420, contentHeight)
            spacing: 12
            clip: true
            focus: true
            keyNavigationWraps: true
            model: launchWithDialog.applicationCount
            delegate: Button {
                required property int index
                width: ListView.view.width
                text: controller.additional_application_name_at(
                          launchWithDialog.modelRow,
                          launchWithDialog.gameId, index)
                highlighted: applicationList.currentIndex === index
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

    Shortcut {
        sequence: "F"
        enabled: !bigBoxGameDetails.opened
                 && !bigBoxImageViewer.opened
                 && !bigBoxModelViewer.opened
                 && !attributeFilterDrawer.opened
                 && !navigationDrawer.opened
                 && controller.big_box_show_game_menu_flip_box
                 && window.selectedBigBoxGameBackImageUrl
                    .toString().length > 0
        onActivated: window.flipSelectedBox()
    }

    Shortcut {
        sequence: "L"
        enabled: !bigBoxGameDetails.opened
                 && !bigBoxImageViewer.opened
                 && !bigBoxModelViewer.opened
        onActivated: window.showLaunchWithSelection()
    }

    Shortcut {
        sequence: "D"
        enabled: !bigBoxGameDetails.opened
                 && !bigBoxImageViewer.opened
                 && !bigBoxModelViewer.opened
                 && !attributeFilterDrawer.opened
                 && !navigationDrawer.opened
                 && window.selectedBigBoxGameId.length > 0
        onActivated: window.openGameDetails()
    }

    Shortcut {
        sequence: "I"
        enabled: !bigBoxImageViewer.opened
                 && !bigBoxModelViewer.opened
                 && window.selectedBigBoxGameId.length > 0
                 && window.selectedBigBoxGameImageCount > 0
        onActivated:
            window.openGameImages(
                bigBoxGameDetails.opened
                ? bigBoxGameDetails.selectedMediaIndex : -1)
    }

    Shortcut {
        sequence: "M"
        enabled: !bigBoxImageViewer.opened
                 && !bigBoxModelViewer.opened
                 && controller.big_box_show_game_menu_model
                 && window.selectedBigBoxGameId.length > 0
        onActivated:
            window.openGameModel(
                bigBoxGameDetails.opened
                ? bigBoxGameDetailsContent : gameList)
    }

    Shortcut {
        sequence: "Tab"
        enabled: !bigBoxGameDetails.opened
                 && !bigBoxImageViewer.opened
                 && !bigBoxModelViewer.opened
        onActivated: {
            if (navigationDrawer.opened) {
                navigationDrawer.close()
                gameList.forceActiveFocus()
            } else {
                window.openNavigation()
            }
        }
    }

    Shortcut {
        sequence: "G"
        enabled: !bigBoxGameDetails.opened
                 && !bigBoxImageViewer.opened
                 && !bigBoxModelViewer.opened
        onActivated: window.openAttributeFilters()
    }

    Shortcut {
        sequence: "R"
        enabled: !bigBoxGameDetails.opened
                 && !bigBoxImageViewer.opened
                 && !bigBoxModelViewer.opened
                 && !attributeFilterDrawer.opened
                 && !navigationDrawer.opened
                 && controller.filtered_count > 0
        onActivated: window.selectRandomGame()
    }

    Shortcut {
        sequence: "Esc"
        onActivated: {
            if (bigBoxModelViewer.opened) {
                bigBoxModelViewer.close()
            } else if (bigBoxImageViewer.opened) {
                bigBoxImageViewer.close()
            } else if (bigBoxGameDetails.opened) {
                bigBoxGameDetails.close()
            } else if (attributeFilterDrawer.opened) {
                attributeFilterDrawer.close()
                gameList.forceActiveFocus()
            } else if (navigationDrawer.opened) {
                navigationDrawer.close()
                gameList.forceActiveFocus()
            } else {
                Qt.quit()
            }
        }
    }
}
