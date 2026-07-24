import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQml.Models
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
                              gameAdditionalApplicationCount, rowCount) {
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
        if (gameId !== expectedId || gameTitle !== expectedTitle
                || gamePlatform !== "Fixture Console"
                || gameFavorite !== expectedFavorite || gameCompleted !== expectedCompleted
                || gamePlayCount !== expectedPlayCount || gameStarRating !== expectedStarRating
                || gameAdditionalApplicationCount !== expectedAdditionalApplicationCount
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
                    Component.onCompleted: window.verifyModelRoles(
                                               index, gameId, gameTitle, gamePlatform,
                                               gameFavorite, gameCompleted, gamePlayCount,
                                               gameStarRating,
                                               gameAdditionalApplicationCount,
                                               gameList.count)
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
                        Item { Layout.fillHeight: true }
                        Label {
                            Layout.fillWidth: true
                            text: gameTitle
                            color: "white"
                            font.pixelSize: 35
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
                      : "↑ / TAB  BROWSE     G  FILTERS     R  RANDOM     ← →  GAMES     ENTER  PLAY"
                color: "#9badc4"
                font.pixelSize: 18
            }
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
        sequence: "L"
        onActivated: window.showLaunchWithSelection()
    }

    Shortcut {
        sequence: "Tab"
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
        sequence: "F"
        onActivated: window.openNavigation()
    }

    Shortcut {
        sequence: "G"
        onActivated: window.openAttributeFilters()
    }

    Shortcut {
        sequence: "R"
        enabled: !attributeFilterDrawer.opened && !navigationDrawer.opened
                 && controller.filtered_count > 0
        onActivated: window.selectRandomGame()
    }

    Shortcut {
        sequence: "Esc"
        onActivated: {
            if (attributeFilterDrawer.opened) {
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
