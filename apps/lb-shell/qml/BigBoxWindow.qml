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
    onClosing: function(close) {
        if (!window.securitySmokeAborting
                && !window.guardSecurityAction("BigBoxExit")) {
            close.accepted = false
            return
        }
        bigBoxScreensaver.stopMode("frontend")
        bigBoxAttractMode.stopMode("frontend")
        startupPresentationOverlay.stopForFrontend()
        bigBoxMusicPlayer.stopPlayback(true)
        backgroundMusicPlayer.stopForFrontend()
        bigBoxMarquee.requestedVisible = false
        bigBoxMarquee.stopPlayback()
    }
    property bool smokeTest: Qt.application.arguments.indexOf("--smoke-test") >= 0
    property bool mediaSmokeTest: Qt.application.arguments.indexOf("--media-smoke-test") >= 0
    property bool mediaSmokeFinished: false
    property bool supplementalMediaSmokeTest:
        Qt.application.arguments.indexOf(
            "--supplemental-media-smoke-test") >= 0
    property int supplementalMediaSmokePhase: 0
    property bool supplementalMediaSmokeFinished: false
    property string supplementalMediaManualUrl: ""
    property string supplementalMediaFirstMusicUrl: ""
    property bool supplementalMediaScreenshotRequested: false
    property string supplementalMediaScreenshotPath:
        argumentValue("--supplemental-media-screenshot")
    property bool backgroundMusicSmokeTest:
        Qt.application.arguments.indexOf(
            "--bigbox-background-music-smoke-test") >= 0
    property int backgroundMusicSmokePhase: 0
    property bool backgroundMusicSmokeFinished: false
    property bool backgroundMusicScreenshotRequested: false
    property string backgroundMusicScreenshotPath:
        argumentValue("--bigbox-background-music-screenshot")
    property string backgroundMusicDefaultFirstUrl: ""
    property string backgroundMusicPlatformFirstUrl: ""
    property string backgroundMusicPlaylistFirstUrl: ""
    property string backgroundMusicCategoryFirstUrl: ""
    property bool startupVideoSmokeTest:
        Qt.application.arguments.indexOf(
            "--bigbox-startup-video-smoke-test") >= 0
    property bool startupVideoNaturalEnd:
        Qt.application.arguments.indexOf(
            "--bigbox-startup-video-natural-end") >= 0
    property int startupVideoRequestedIndex: {
        const argument = argumentValue("--bigbox-startup-video-index")
        if (argument.length === 0)
            return -1
        const value = Number(argument)
        return Number.isInteger(value) ? value : -1
    }
    property bool startupSplashSmokeTest:
        Qt.application.arguments.indexOf(
            "--bigbox-startup-splash-smoke-test") >= 0
    property bool startupSplashDisabledSmokeTest:
        Qt.application.arguments.indexOf(
            "--bigbox-startup-splash-disabled-smoke-test") >= 0
    property bool startupSplashAnySmokeTest:
        startupSplashSmokeTest || startupSplashDisabledSmokeTest
    property int startupSoundRequestedIndex: {
        const argument = argumentValue("--bigbox-startup-sound-index")
        if (argument.length === 0)
            return -1
        const value = Number(argument)
        return Number.isInteger(value) ? value : -1
    }
    property bool startupPresentationPending:
        argumentValue("--library").length > 0
    property bool startupPresentationDecisionMade: false
    property bool startupProbeReadyBeforeLoad: false
    property bool startupLibraryLoadSeen: false
    property bool startupVideoCompletionSeen: false
    property bool startupVideoFrameSeen: false
    property bool startupVideoScreenshotRequested: false
    property bool startupVideoScreenshotReady:
        startupVideoScreenshotPath.length === 0
    property bool startupVideoSmokeFinished: false
    property string startupVideoScreenshotPath:
        argumentValue("--bigbox-startup-video-screenshot")
    property bool startupSplashWasVisible: false
    property bool startupSoundPlaybackSeen: false
    property bool startupSplashScreenshotRequested: false
    property bool startupSplashScreenshotReady:
        startupSplashScreenshotPath.length === 0
    property bool startupSplashSmokeFinished: false
    property string startupSplashScreenshotPath:
        argumentValue("--bigbox-startup-splash-screenshot")
    property bool attractModeSmokeTest:
        Qt.application.arguments.indexOf(
            "--bigbox-attract-mode-smoke-test") >= 0
    property bool attractModeDisabledSmokeTest:
        Qt.application.arguments.indexOf(
            "--bigbox-attract-mode-disabled-smoke-test") >= 0
    property bool attractModeAnySmokeTest:
        attractModeSmokeTest || attractModeDisabledSmokeTest
    property int attractModeSmokePhase: 0
    property bool attractModeSmokeFinished: false
    property bool attractModeScreenshotRequested: false
    property string attractModeScreenshotPath:
        argumentValue("--bigbox-attract-mode-screenshot")
    property int attractModeAutoWheelSteps: 0
    property int attractModeAutoMovementCycles: 0
    property int attractModeAutoFilterSwitches: 0
    property int attractModeAutoDelayElapsedMs: 0
    property double attractModeDisabledWaitStartedAt: 0
    property int attractNavigationCursor: -1
    property bool screensaverSmokeTest:
        Qt.application.arguments.indexOf(
            "--bigbox-screensaver-smoke-test") >= 0
    property bool screensaverDisabledSmokeTest:
        Qt.application.arguments.indexOf(
            "--bigbox-screensaver-disabled-smoke-test") >= 0
    property bool screensaverAnySmokeTest:
        screensaverSmokeTest || screensaverDisabledSmokeTest
    property int screensaverSmokePhase: 0
    property bool screensaverSmokeFinished: false
    property double screensaverPhaseStartedAt: 0
    property double screensaverDisabledWaitStartedAt: 0
    property bool screensaverCapturePending: false
    property bool screensaverVideoReadySeen: false
    property string screensaverScreenshotPrefix:
        argumentValue("--bigbox-screensaver-screenshot-prefix")
    property bool inputSmokeTest:
        Qt.application.arguments.indexOf(
            "--bigbox-input-smoke-test") >= 0
    property int inputSmokePhase: 0
    property int inputSmokeSelectCount: 0
    property int inputSmokeBackCount: 0
    property int inputSmokeNavigationCount: 0
    property int inputSmokeImageOpenCount: 0
    property int inputSmokeImageBackCount: 0
    property bool inputSmokeFinished: false
    property bool inputEditorSmokeTest:
        Qt.application.arguments.indexOf(
            "--bigbox-input-editor-smoke-test") >= 0
    property int inputEditorSmokePhase: 0
    property int inputEditorSmokeStartRevision: -1
    property bool inputEditorSmokeFinished: false
    property bool inputEditorSmokeScreenshotRequested: false
    property bool inputEditorSmokeScreenshotReady: false
    property string inputEditorSmokeScreenshotPath:
        argumentValue("--bigbox-input-editor-screenshot")
    property bool marqueeSmokeTest:
        Qt.application.arguments.indexOf(
            "--bigbox-marquee-smoke-test") >= 0
    property int marqueeSmokePhase: 0
    property int marqueeSmokeStartRevision: -1
    property bool marqueeSmokeFinished: false
    property bool marqueeGameScreenshotRequested: false
    property bool marqueeGameScreenshotReady: false
    property bool marqueePlatformScreenshotRequested: false
    property bool marqueePlatformScreenshotReady: false
    property bool marqueeVideoReadySeen: false
    property string marqueeGameScreenshotPath:
        argumentValue("--bigbox-marquee-game-screenshot")
    property string marqueePlatformScreenshotPath:
        argumentValue("--bigbox-marquee-platform-screenshot")
    property string marqueeContextKind: "game"
    property string marqueeContextName: ""
    property bool securitySmokeTest:
        Qt.application.arguments.indexOf(
            "--bigbox-security-smoke-test") >= 0
    property int securitySmokePhase: 0
    property int securitySmokeStartRevision: -1
    property int securitySmokeBlockedActions: 0
    property int securitySmokeFailedUnlocks: 0
    property int securitySmokeSuccessfulUnlocks: 0
    property bool securitySmokeFinished: false
    property bool securitySmokeAborting: false
    property bool securityPinScreenshotRequested: false
    property bool securityEditorScreenshotRequested: false
    property string securityPinScreenshotPath:
        argumentValue("--bigbox-security-pin-screenshot")
    property string securityEditorScreenshotPath:
        argumentValue("--bigbox-security-editor-screenshot")
    property bool gameActionsSmokeTest:
        Qt.application.arguments.indexOf(
            "--bigbox-game-actions-smoke-test") >= 0
    property int gameActionsSmokePhase: 0
    property int gameActionsSmokeStartRevision: -1
    property bool gameActionsFavoriteFirstSeen: false
    property bool gameActionsPopupSeen: false
    property bool gameActionsBlockedFavoriteSeen: false
    property bool gameActionsScreenshotRequested: false
    property bool gameActionsSmokeFinished: false
    property string gameActionsScreenshotPath:
        argumentValue("--bigbox-game-actions-screenshot")
    property bool playlistActionsSmokeTest:
        Qt.application.arguments.indexOf(
            "--bigbox-playlist-actions-smoke-test") >= 0
    property int playlistActionsSmokePhase: 0
    property int playlistActionsSmokeStartRevision: -1
    property string playlistActionsSmokeGameId: ""
    property string playlistActionsSmokePlaylistId: ""
    property int playlistActionsSmokeAddTargetCount: 0
    property bool playlistActionsSmokePopupSeen: false
    property bool playlistActionsSmokeBlockedSeen: false
    property bool playlistActionsSmokeScreenshotRequested: false
    property bool playlistActionsSmokeFinished: false
    property string playlistActionsSmokeScreenshotPath:
        argumentValue("--bigbox-playlist-actions-screenshot")
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
    property string backgroundMusicContextKind: ""
    property string backgroundMusicContextName: "All Games"
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
    property real selectedBigBoxGameStarRatingFloat: 0
    property real selectedBigBoxGamePlayTimeSeconds: 0
    property real selectedBigBoxGameCommunityRating: 0
    property int selectedBigBoxGameCommunityVotes: 0
    property url selectedBigBoxGameFrontImageUrl
    property bool selectedBigBoxGameBoxBackVisible: false
    property var selectedBigBoxPlaylistAction: ({
        version: 1,
        gameId: "",
        addTargets: [],
        removeCurrent: null
    })
    readonly property int selectedBigBoxPlaylistAddTargetCount:
        selectedBigBoxPlaylistAction !== null
        && Array.isArray(selectedBigBoxPlaylistAction.addTargets)
        ? selectedBigBoxPlaylistAction.addTargets.length : 0
    readonly property var selectedBigBoxPlaylistRemoveCurrent:
        selectedBigBoxPlaylistAction !== null
        ? selectedBigBoxPlaylistAction.removeCurrent : null
    readonly property int playlistMembershipRevision:
        controller.big_box_playlist_membership_revision
    onPlaylistMembershipRevisionChanged: Qt.callLater(
        window.refreshSelectedBigBoxPlaylistAction)
    property int runtimeMasterVolumePercent: 100
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
            if (gameId.length > 0) {
                launchGame(gameList.currentIndex, gameId)
            }
        }
    }

    function moveMainSelection(offset) {
        if (gameList.count <= 0)
            return false
        const current = gameList.currentIndex < 0
                      ? 0 : gameList.currentIndex
        let row = (current + offset) % gameList.count
        if (row < 0)
            row += gameList.count
        gameList.currentIndex = row
        gameList.positionViewAtIndex(row, ListView.Center)
        gameList.forceActiveFocus()
        return true
    }

    function moveNavigationSelection(offset) {
        const count = controller.big_box_navigation_entry_count + 1
        if (count <= 0)
            return false
        let row = navigationList.currentIndex
        for (let attempt = 0; attempt < count; ++attempt) {
            row = (row + offset) % count
            if (row < 0)
                row += count
            if (navigationRowAllowed(row)) {
                navigationList.currentIndex = row
                navigationList.positionViewAtIndex(
                    row, ListView.Contain)
                return true
            }
        }
        return false
    }

    function moveFocusedControl(forward) {
        const current = window.activeFocusItem
        if (!current)
            return false
        const nextFunction = current["nextItemInFocusChain"]
        if (typeof nextFunction !== "function")
            return false
        const next = nextFunction.call(current, forward)
        if (!next || next === current)
            return false
        next.forceActiveFocus()
        return true
    }

    function activateFocusedControl() {
        const current = window.activeFocusItem
        if (!current)
            return false
        const click = current["clicked"]
        if (typeof click === "function") {
            click()
            return true
        }
        const toggle = current["toggle"]
        if (typeof toggle === "function") {
            toggle()
            return true
        }
        const popup = current["popup"]
        if (popup && typeof popup["open"] === "function") {
            popup.open()
            return true
        }
        return false
    }

    function securityActionAllowed(action) {
        return !controller.big_box_locked
               || controller.big_box_action_allowed_while_locked(action)
    }

    function guardSecurityAction(action) {
        if (securityActionAllowed(action))
            return true
        controller.note_big_box_locked_action(action)
        return false
    }

    function securityNavigationAction(kind) {
        if (kind === "" || kind === "all")
            return "BigBoxShowAllGames"
        if (kind === "platform")
            return "BigBoxShowPlatforms"
        if (kind === "category")
            return "BigBoxShowPlatformCategories"
        if (kind === "playlist")
            return "BigBoxShowPlaylists"
        return "BigBoxFilter"
    }

    function guardSecurityNavigation(kind) {
        if (!controller.big_box_locked
                || controller.big_box_navigation_allowed_while_locked(kind))
            return true
        controller.note_big_box_locked_action(
            securityNavigationAction(kind))
        return false
    }

    function navigationAccessAvailable() {
        return !controller.big_box_locked
               || controller.big_box_navigation_allowed_while_locked("all")
               || controller.big_box_navigation_allowed_while_locked(
                   "platform")
               || controller.big_box_navigation_allowed_while_locked(
                   "category")
               || controller.big_box_navigation_allowed_while_locked(
                   "playlist")
    }

    function navigationRowAllowed(row) {
        const kind = row <= 0 ? "all"
            : controller.big_box_navigation_entry_kind_at(row - 1)
        return !controller.big_box_locked
               || controller
                  .big_box_navigation_allowed_while_locked(kind)
    }

    function requestLockUnlock() {
        if (!controller.big_box_pin_configured) {
            return bigBoxSecuritySettings.openEditor()
        }
        if (controller.big_box_locked) {
            bigBoxUnlockPopup.openForPrompt(
                "Enter your PIN",
                "Unlock BigBox to use protected features.")
            return true
        }
        return controller.lock_big_box()
    }

    function failSecuritySmoke(message, exitCode) {
        console.error(message
                      + " phase=" + securitySmokePhase
                      + " configured="
                      + controller.big_box_pin_configured
                      + " locked=" + controller.big_box_locked
                      + " revision="
                      + controller.big_box_security_settings_revision
                      + " writing=" + controller.writing
                      + " status=" + controller.status_message)
        securitySmokeAborting = true
        Qt.exit(exitCode)
    }

    function failGameActionsSmoke(message, exitCode) {
        console.error(message
                      + " phase=" + gameActionsSmokePhase
                      + " game=" + selectedBigBoxGameId
                      + " favorite=" + selectedBigBoxGameFavorite
                      + " rating=" + selectedBigBoxGameStarRatingFloat
                      + " locked=" + controller.big_box_locked
                      + " revision="
                      + controller.big_box_game_state_revision
                      + " writing=" + controller.writing
                      + " status=" + controller.status_message)
        securitySmokeAborting = true
        Qt.exit(exitCode)
    }

    function failPlaylistActionsSmoke(message, exitCode) {
        console.error(message
                      + " phase=" + playlistActionsSmokePhase
                      + " game=" + selectedBigBoxGameId
                      + " filter=" + controller.navigation_filter_kind
                      + ":" + controller.navigation_filter_key
                      + " targets="
                      + selectedBigBoxPlaylistAddTargetCount
                      + " remove="
                      + (selectedBigBoxPlaylistRemoveCurrent === null
                         ? "none"
                         : selectedBigBoxPlaylistRemoveCurrent.playlistId)
                      + " locked=" + controller.big_box_locked
                      + " revision="
                      + controller.big_box_playlist_membership_revision
                      + " writing=" + controller.writing
                      + " status=" + controller.status_message)
        securitySmokeAborting = true
        Qt.exit(exitCode)
    }

    function closeBigBoxSurface() {
        if (bigBoxPlaylistPopup.opened) {
            bigBoxPlaylistPopup.cancelEntry()
        } else if (bigBoxStarRatingPopup.opened) {
            bigBoxStarRatingPopup.cancelEntry()
        } else if (bigBoxSecuritySettings.opened) {
            bigBoxSecuritySettings.close()
        } else if (bigBoxMarqueeSettings.opened) {
            bigBoxMarqueeSettings.close()
        } else if (bigBoxModelViewer.opened) {
            bigBoxModelViewer.close()
        } else if (bigBoxImageViewer.opened) {
            bigBoxImageViewer.close()
        } else if (bigBoxGameDetails.opened) {
            bigBoxGameDetails.close()
        } else if (launchWithDialog.opened) {
            launchWithDialog.close()
            gameList.forceActiveFocus()
        } else if (attributeFilterDrawer.opened) {
            attributeFilterDrawer.close()
            gameList.forceActiveFocus()
        } else if (navigationDrawer.opened) {
            navigationDrawer.close()
            gameList.forceActiveFocus()
        } else {
            return false
        }
        return true
    }

    function applyPrimaryMonitorScreen() {
        const index = controller.big_box_primary_monitor_index
        if (index >= 0 && index < controller.host_screen_count())
            controller.route_window_to_host_screen(window, index)
    }

    function resetMarqueeContext() {
        marqueeContextKind = "game"
        marqueeContextName = ""
    }

    function updateMarqueeNavigationPreview() {
        if (!navigationDrawer.opened
                || navigationList.currentIndex <= 0) {
            resetMarqueeContext()
            return
        }
        const index = navigationList.currentIndex - 1
        const kind =
            controller.big_box_navigation_entry_kind_at(index)
        if (kind === "platform") {
            marqueeContextKind = "platform"
            marqueeContextName =
                controller.big_box_navigation_entry_name_at(index)
        } else {
            resetMarqueeContext()
        }
    }

    function activateFirstNavigationKind(kind) {
        if (kind.length === 0) {
            activateNavigationRow(0)
            return true
        }
        for (let index = 0;
             index < controller.big_box_navigation_entry_count;
             ++index) {
            if (controller.big_box_navigation_entry_kind_at(index)
                    === kind) {
                activateNavigationRow(index + 1)
                return true
            }
        }
        return false
    }

    function adjustRuntimeVolume(delta) {
        const next = Math.max(
            0, Math.min(100, runtimeMasterVolumePercent + delta))
        if (next === runtimeMasterVolumePercent)
            return false
        runtimeMasterVolumePercent = next
        if (backgroundMusicPlayer.visible)
            backgroundMusicPlayer.showOnScreenDisplay()
        return true
    }

    function dispatchBigBoxInputCandidates(actions) {
        const values = actions.split("|")
        for (let index = 0; index < values.length; ++index) {
            if (dispatchBigBoxInputAction(values[index]))
                return true
        }
        return false
    }

    function dispatchBigBoxInputAction(action) {
        if (action.length === 0)
            return false
        if (bigBoxAttractMode.active) {
            bigBoxAttractMode.stopMode("input")
            return true
        }
        if (bigBoxScreensaver.active) {
            if (action === "BigBoxSelect"
                    || action === "BigBoxPlayGame")
                return bigBoxScreensaver.exploreCurrentGame()
            bigBoxScreensaver.stopMode("input")
            return true
        }
        if (startupPresentationPending)
            return false

        if (bigBoxUnlockPopup.opened)
            return bigBoxUnlockPopup.handleAction(action)
        if (bigBoxSecuritySettings.opened)
            return bigBoxSecuritySettings.handleAction(action)
        if (bigBoxPlaylistPopup.opened)
            return bigBoxPlaylistPopup.handleAction(action)
        if (bigBoxStarRatingPopup.opened)
            return bigBoxStarRatingPopup.handleAction(action)
        if (action === "BigBoxLockUnlock")
            return requestLockUnlock()
        if (action === "BigBoxBack") {
            if (closeBigBoxSurface())
                return true
            if (!guardSecurityAction("BigBoxExit"))
                return true
            Qt.quit()
            return true
        }
        if (action === "BigBoxExit") {
            if (!guardSecurityAction(action))
                return true
            Qt.quit()
            return true
        }
        if (!guardSecurityAction(action))
            return true
        if (action === "BigBoxVolumeUp")
            return adjustRuntimeVolume(5)
        if (action === "BigBoxVolumeDown")
            return adjustRuntimeVolume(-5)
        if (action === "BigBoxShowPauseScreen") {
            if (!controller.pause_screen_available)
                return false
            if (controller.pause_screen_active)
                controller.resume_launch_session()
            else
                controller.pause_launch_session()
            return true
        }
        if (action === "BigBoxFocusInterface") {
            window.requestActivate()
            return true
        }

        if (bigBoxModelViewer.opened) {
            if (action === "BigBoxRotateModelLeft"
                    || action === "BigBoxNavigateLeft")
                return bigBoxModelViewer.rotateBy(-8, 0)
            if (action === "BigBoxRotateModelRight"
                    || action === "BigBoxNavigateRight")
                return bigBoxModelViewer.rotateBy(8, 0)
            if (action === "BigBoxRotateModelUp"
                    || action === "BigBoxNavigateUp")
                return bigBoxModelViewer.rotateBy(0, -8)
            if (action === "BigBoxRotateModelDown"
                    || action === "BigBoxNavigateDown")
                return bigBoxModelViewer.rotateBy(0, 8)
            if (action === "BigBoxZoomIn")
                return bigBoxModelViewer.setZoom(
                    bigBoxModelViewer.modelZoom
                    + bigBoxModelViewer.zoomStep)
            if (action === "BigBoxZoomOut")
                return bigBoxModelViewer.setZoom(
                    bigBoxModelViewer.modelZoom
                    - bigBoxModelViewer.zoomStep)
            return false
        }

        if (bigBoxImageViewer.opened) {
            if (action === "BigBoxNavigateLeft"
                    || action === "BigBoxPageUp")
                return bigBoxImageViewer.zoomFactor
                       > bigBoxImageViewer.minimumZoom
                       ? bigBoxImageViewer.panBy(64, 0)
                       : bigBoxImageViewer.selectPreviousImage()
            if (action === "BigBoxNavigateRight"
                    || action === "BigBoxPageDown"
                    || action === "BigBoxSelect")
                return bigBoxImageViewer.zoomFactor
                       > bigBoxImageViewer.minimumZoom
                       ? bigBoxImageViewer.panBy(-64, 0)
                       : bigBoxImageViewer.selectNextImage()
            if (action === "BigBoxNavigateUp")
                return bigBoxImageViewer.panBy(0, 64)
            if (action === "BigBoxNavigateDown")
                return bigBoxImageViewer.panBy(0, -64)
            if (action === "BigBoxZoomIn")
                return bigBoxImageViewer.setZoom(
                    bigBoxImageViewer.zoomFactor
                    + bigBoxImageViewer.zoomStep) > 0
            if (action === "BigBoxZoomOut")
                return bigBoxImageViewer.setZoom(
                    bigBoxImageViewer.zoomFactor
                    - bigBoxImageViewer.zoomStep) > 0
            if (action === "BigBoxSwitchImageType")
                return bigBoxImageViewer.selectNextImage()
            return false
        }

        if (navigationDrawer.opened) {
            if (action === "BigBoxNavigateUp")
                return moveNavigationSelection(-1)
            if (action === "BigBoxNavigateDown")
                return moveNavigationSelection(1)
            if (action === "BigBoxPageUp")
                return moveNavigationSelection(-5)
            if (action === "BigBoxPageDown")
                return moveNavigationSelection(5)
            if (action === "BigBoxSelect") {
                activateNavigationRow(navigationList.currentIndex)
                return true
            }
            return false
        }

        if (attributeFilterDrawer.opened
                || launchWithDialog.opened) {
            if (action === "BigBoxNavigateUp"
                    || action === "BigBoxNavigateLeft")
                return moveFocusedControl(false)
            if (action === "BigBoxNavigateDown"
                    || action === "BigBoxNavigateRight")
                return moveFocusedControl(true)
            if (action === "BigBoxSelect")
                return activateFocusedControl()
            return false
        }

        if (bigBoxGameDetails.opened) {
            if (action === "BigBoxNavigateLeft"
                    || action === "BigBoxPageUp")
                return bigBoxGameDetails.selectPreviousMedia()
            if (action === "BigBoxNavigateRight"
                    || action === "BigBoxPageDown")
                return bigBoxGameDetails.selectNextMedia()
            if (action === "BigBoxSelect")
                return bigBoxGameDetails.togglePlayback()
            if (action === "BigBoxPlayGame") {
                bigBoxGameDetails.close()
                Qt.callLater(window.launchSelection)
                return true
            }
            if (action === "BigBoxShowImages")
                return openGameImages(
                    bigBoxGameDetails.selectedMediaIndex)
            if (action === "BigBoxShowModel")
                return openGameModel(bigBoxGameDetailsContent)
        }

        if (action === "BigBoxNavigateLeft")
            return moveMainSelection(-1)
        if (action === "BigBoxNavigateRight")
            return moveMainSelection(1)
        if (action === "BigBoxPageUp")
            return moveMainSelection(-5)
        if (action === "BigBoxPageDown")
            return moveMainSelection(5)
        if (action === "BigBoxNavigateUp") {
            openNavigation()
            return true
        }
        if (action === "BigBoxNavigateDown"
                || action === "BigBoxSelect"
                || action === "BigBoxShowGameDetails")
            return openGameDetails()
        if (action === "BigBoxPlayGame") {
            launchSelection()
            return true
        }
        if (action === "BigBoxShowImages")
            return openGameImages(-1)
        if (action === "BigBoxShowModel")
            return openGameModel(gameList)
        if (action === "BigBoxFlipBox")
            return flipSelectedBox()
        if (action === "BigBoxSetStarRating")
            return openSelectedStarRating()
        if (action === "BigBoxFilter") {
            openAttributeFilters()
            return true
        }
        if (action === "BigBoxShowAllGames")
            return activateFirstNavigationKind("")
        if (action === "BigBoxShowPlatforms")
            return activateFirstNavigationKind("platform")
        if (action === "BigBoxShowPlaylists")
            return activateFirstNavigationKind("playlist")
        if (action === "BigBoxShowPlatformCategories")
            return activateFirstNavigationKind("category")
        if (action === "BigBoxRandomGame"
                || action === "BigBoxWheelSpin")
            return selectRandomGame() >= 0
        if (action === "BigBoxStartAttractMode")
            return bigBoxAttractMode.startManual()
        if (action === "BigBoxStartScreensaver")
            return bigBoxScreensaver.startManual()
        if (action === "BigBoxPlayMusic") {
            if (bigBoxMusicPlayer.opened)
                return bigBoxMusicPlayer.togglePlayback()
            return playGameMusic(
                selectedBigBoxGameId,
                selectedBigBoxGameTitle, true)
        }
        if (action === "BigBoxNextMusicTrack")
            return bigBoxMusicPlayer.opened
                   ? bigBoxMusicPlayer.nextTrack()
                   : backgroundMusicPlayer.nextTrack()
        if (action === "BigBoxPreviousMusicTrack")
            return bigBoxMusicPlayer.opened
                   ? bigBoxMusicPlayer.previousTrack()
                   : backgroundMusicPlayer.previousTrack()
        return false
    }

    function beginApplicationStartupPresentation() {
        if (!startupPresentationPending
                || startupPresentationDecisionMade)
            return
        startupPresentationDecisionMade = true
        if (controller.startup_presentation_ready
                && controller.indexed_startup_video_count > 0
                && startupPresentationOverlay.beginVideo(
                    startupVideoRequestedIndex))
            return
        if (controller.startup_presentation_ready
                && controller.big_box_play_startup_sound
                && controller.indexed_startup_sound_count > 0)
            startupPresentationOverlay.beginStartupSound(
                startupSoundRequestedIndex)
        if (controller.startup_presentation_ready
                && controller.big_box_show_startup_splash_screen) {
            startupSplashWasVisible = true
            return
        }
        startupPresentationPending = false
        Qt.callLater(function() {
            gameList.forceActiveFocus()
        })
    }

    function finishApplicationStartupVideo() {
        startupVideoCompletionSeen = true
        if (controller.loading)
            return
        if (startupSplashSmokeTest
                && !startupSplashSmokeFinished)
            return
        startupPresentationPending = false
        Qt.callLater(function() {
            gameList.forceActiveFocus()
        })
    }

    function launchGame(row, gameId) {
        bigBoxMusicPlayer.stopPlayback(true)
        backgroundMusicPlayer.stopForFrontend()
        controller.launch_game(row, gameId)
    }

    function launchAdditionalApplication(row, gameId, applicationId) {
        bigBoxMusicPlayer.stopPlayback(true)
        backgroundMusicPlayer.stopForFrontend()
        controller.launch_additional_application(
                    row, gameId, applicationId)
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

    function openGameManual(gameId) {
        if (gameId.length === 0 || controller.loading
                || controller.writing
                || !controller.big_box_show_game_menu_view_manual)
            return false
        const manualUrl = controller.game_manual_url_for_game(gameId)
        if (manualUrl.toString().length === 0)
            return false
        if (supplementalMediaSmokeTest) {
            supplementalMediaManualUrl = manualUrl.toString()
            return true
        }
        return Qt.openUrlExternally(manualUrl)
    }

    function playGameMusic(gameId, gameTitle, playNow) {
        if (gameId.length === 0 || controller.loading
                || controller.writing || startupPresentationPending
                || controller.game_music_count_for_game(gameId) === 0)
            return false
        bigBoxMediaPlayer.stop()
        bigBoxMusicPlayer.shuffleEnabled =
            controller.big_box_shuffle_soundtrack_music
        bigBoxMusicPlayer.repeatEnabled =
            controller.big_box_repeat_game_music
        return bigBoxMusicPlayer.openForGame(gameId, gameTitle, playNow)
    }

    function autoPlaySelectedGameMusicFromList() {
        if (!controller.big_box_auto_play_music_games_list
                || startupPresentationPending
                || selectedBigBoxGameId.length === 0)
            return false
        if (controller.game_music_count_for_game(
                selectedBigBoxGameId) === 0) {
            if (bigBoxMusicPlayer.opened)
                bigBoxMusicPlayer.stopPlayback(true)
            return false
        }
        if (bigBoxMusicPlayer.opened
                && bigBoxMusicPlayer.gameId === selectedBigBoxGameId)
            return true
        return playGameMusic(
                    selectedBigBoxGameId,
                    selectedBigBoxGameTitle, true)
    }

    function finishSupplementalMediaSmoke() {
        function complete() {
            const firstMusicUrl =
                controller.game_music_url_at(
                    "fixture-adventure", 0).toString()
            const secondMusicUrl =
                controller.game_music_url_at(
                    "fixture-adventure", 1).toString()
            if (supplementalMediaFirstMusicUrl !== firstMusicUrl
                    || bigBoxMusicPlayer.trackSource.toString()
                       !== secondMusicUrl
                    || !controller
                        .report_supplemental_media_smoke_success(
                            "bigbox", "fixture-adventure",
                            supplementalMediaManualUrl,
                            firstMusicUrl, secondMusicUrl)) {
                console.error(
                    "BIGBOX_SUPPLEMENTAL_MEDIA_CONTROLLER_REJECTED"
                    + " manual=" + supplementalMediaManualUrl
                    + " first=" + supplementalMediaFirstMusicUrl
                    + " current="
                    + bigBoxMusicPlayer.trackSource.toString())
                Qt.exit(579)
                return
            }
            bigBoxMusicPlayer.clickStopForSmoke()
            supplementalMediaSmokeFinished = true
            Qt.quit()
        }
        if (supplementalMediaScreenshotPath.length === 0) {
            complete()
            return
        }
        if (supplementalMediaScreenshotRequested)
            return
        supplementalMediaScreenshotRequested = true
        bigBoxMusicPlayer.contentItem.grabToImage(function(result) {
            if (!result.saveToFile(
                    supplementalMediaScreenshotPath)) {
                console.error(
                    "BIGBOX_SUPPLEMENTAL_MEDIA_SCREENSHOT_SAVE_FAILED path="
                    + supplementalMediaScreenshotPath)
                Qt.exit(578)
                return
            }
            complete()
        })
    }

    function finishBackgroundMusicSmoke() {
        function complete() {
            if (!controller.report_background_music_smoke_success(
                    backgroundMusicDefaultFirstUrl,
                    backgroundMusicPlatformFirstUrl,
                    backgroundMusicPlaylistFirstUrl,
                    backgroundMusicCategoryFirstUrl)) {
                console.error(
                    "BIGBOX_BACKGROUND_MUSIC_CONTROLLER_REJECTED"
                    + " default=" + backgroundMusicDefaultFirstUrl
                    + " platform=" + backgroundMusicPlatformFirstUrl
                    + " playlist=" + backgroundMusicPlaylistFirstUrl
                    + " category=" + backgroundMusicCategoryFirstUrl)
                Qt.exit(629)
                return
            }
            backgroundMusicPlayer.stopForFrontend()
            backgroundMusicSmokeFinished = true
            Qt.quit()
        }
        if (backgroundMusicScreenshotPath.length === 0) {
            complete()
            return
        }
        if (backgroundMusicScreenshotRequested)
            return
        backgroundMusicScreenshotRequested = true
        backgroundMusicPlayer.contentItem.grabToImage(function(result) {
            if (!result.saveToFile(
                    backgroundMusicScreenshotPath)) {
                console.error(
                    "BIGBOX_BACKGROUND_MUSIC_SCREENSHOT_SAVE_FAILED path="
                    + backgroundMusicScreenshotPath)
                Qt.exit(628)
                return
            }
            complete()
        })
    }

    function flipSelectedBox() {
        if (!guardSecurityAction("BigBoxFlipBox")
                || !controller.big_box_show_game_menu_flip_box
                || selectedBigBoxGameId.length === 0
                || selectedBigBoxGameBackImageUrl.toString().length === 0)
            return false
        selectedBigBoxGameBoxBackVisible =
            !selectedBigBoxGameBoxBackVisible
        return true
    }

    function toggleSelectedFavorite() {
        if (!controller.big_box_show_game_menu_favorite
                || selectedBigBoxGameId.length === 0
                || controller.loading || controller.writing
                || !guardSecurityAction("BigBoxFavoriteGames"))
            return false
        return controller.update_big_box_game_state(
                    gameList.currentIndex,
                    selectedBigBoxGameId,
                    !selectedBigBoxGameFavorite,
                    selectedBigBoxGameStarRatingFloat)
    }

    function openSelectedStarRating() {
        if (!controller.big_box_show_game_menu_star_rating
                || selectedBigBoxGameId.length === 0
                || controller.loading || controller.writing
                || !guardSecurityAction("BigBoxSetStarRating"))
            return false
        bigBoxStarRatingPopup.openForGame(
                    gameList.currentIndex,
                    selectedBigBoxGameId,
                    selectedBigBoxGameTitle,
                    selectedBigBoxGameStarRatingFloat,
                    selectedBigBoxGameCommunityRating,
                    selectedBigBoxGameCommunityVotes)
        return true
    }

    function refreshSelectedBigBoxPlaylistAction() {
        const emptyPayload = {
            version: 1,
            gameId: selectedBigBoxGameId,
            addTargets: [],
            removeCurrent: null
        }
        if (selectedBigBoxGameId.length === 0
                || gameList.currentIndex < 0) {
            selectedBigBoxPlaylistAction = emptyPayload
            return false
        }
        let payload
        try {
            payload = JSON.parse(
                controller.big_box_playlist_action_json(
                    gameList.currentIndex,
                    selectedBigBoxGameId))
        } catch (error) {
            selectedBigBoxPlaylistAction = emptyPayload
            return false
        }
        if (payload === null
                || payload.version !== 1
                || payload.gameId !== selectedBigBoxGameId
                || !Array.isArray(payload.addTargets)) {
            selectedBigBoxPlaylistAction = emptyPayload
            return false
        }
        selectedBigBoxPlaylistAction = payload
        return true
    }

    function openSelectedPlaylistPopup() {
        if (!controller.big_box_show_game_menu_playlist_actions
                || selectedBigBoxPlaylistAddTargetCount <= 0
                || selectedBigBoxGameId.length === 0
                || controller.loading || controller.writing
                || !guardSecurityAction("BigBoxPlaylistActions"))
            return false
        const count = bigBoxPlaylistPopup.openForGame(
            gameList.currentIndex,
            selectedBigBoxGameId,
            selectedBigBoxGameTitle,
            JSON.stringify(selectedBigBoxPlaylistAction))
        return count > 0
    }

    function removeSelectedFromCurrentPlaylist() {
        if (!controller.big_box_show_game_menu_playlist_actions
                || selectedBigBoxPlaylistRemoveCurrent === null
                || selectedBigBoxGameId.length === 0
                || controller.loading || controller.writing
                || !guardSecurityAction("BigBoxPlaylistActions"))
            return false
        return controller.update_big_box_playlist_membership(
            gameList.currentIndex,
            selectedBigBoxGameId,
            selectedBigBoxPlaylistRemoveCurrent.playlistId,
            false)
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
        selectedBigBoxGameStarRatingFloat = 0
        selectedBigBoxGamePlayTimeSeconds = 0
        selectedBigBoxGameCommunityRating = 0
        selectedBigBoxGameCommunityVotes = 0
        selectedBigBoxGameFrontImageUrl = ""
        selectedBigBoxGameBoxBackVisible = false
        selectedBigBoxPlaylistAction = {
            version: 1,
            gameId: "",
            addTargets: [],
            removeCurrent: null
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
        if (!guardSecurityAction("BigBoxFilter"))
            return false
        attributeFilterDrawer.open()
        bigBoxStateFilterCombo.forceActiveFocus()
        return true
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

    function advanceAttractWheel() {
        if (gameList.count <= 0)
            return false
        const row = gameList.currentIndex < 0
                  ? 0 : (gameList.currentIndex + 1) % gameList.count
        gameList.currentIndex = row
        gameList.positionViewAtIndex(row, ListView.Center)
        return true
    }

    function currentBigBoxNavigationRow() {
        if (controller.navigation_filter_kind.length === 0)
            return 0
        for (let index = 0;
             index < controller.big_box_navigation_entry_count;
             ++index) {
            if (controller.big_box_navigation_entry_kind_at(index)
                    === controller.navigation_filter_kind
                    && controller.big_box_navigation_entry_key_at(index)
                       === controller.navigation_filter_key)
                return index + 1
        }
        return 0
    }

    function switchAttractFilter() {
        const rowCount = controller.big_box_navigation_entry_count + 1
        if (rowCount <= 1)
            return false
        if (attractNavigationCursor < 0
                || attractNavigationCursor >= rowCount)
            attractNavigationCursor = currentBigBoxNavigationRow()
        for (let offset = 1; offset < rowCount; ++offset) {
            const row = (attractNavigationCursor + offset) % rowCount
            const gameCount = row === 0
                            ? controller.game_count
                            : controller
                              .big_box_navigation_entry_game_count_at(
                                  row - 1)
            if (gameCount <= 0)
                continue
            attractNavigationCursor = row
            activateNavigationRow(row)
            return true
        }
        return false
    }

    function exploreScreensaverGame(gameId) {
        activateNavigationRow(0)
        setAttributeFilters("any", "none", false, false)
        controller.search_text = ""
        const row = controller.row_for_game_id(gameId)
        if (row < 0)
            return false
        gameList.currentIndex = row
        gameList.positionViewAtIndex(row, ListView.Center)
        gameList.forceActiveFocus()
        return true
    }

    function finishAttractModeSmoke(expectedEnabled) {
        const wheelSteps = expectedEnabled
                         ? attractModeAutoWheelSteps
                         : bigBoxAttractMode.totalWheelSteps
        const movementCycles = expectedEnabled
                             ? attractModeAutoMovementCycles
                             : bigBoxAttractMode.movementCycles
        const filterSwitches = expectedEnabled
                             ? attractModeAutoFilterSwitches
                             : bigBoxAttractMode.filterSwitches
        const automaticDelay = expectedEnabled
                             ? attractModeAutoDelayElapsedMs : 0
        if (!controller.report_big_box_attract_mode_smoke_success(
                expectedEnabled,
                wheelSteps,
                movementCycles,
                filterSwitches,
                automaticDelay,
                bigBoxAttractMode.manualStartCount > 0,
                bigBoxAttractMode.inputStopCount,
                bigBoxAttractMode.moveSoundReady)) {
            console.error(
                "BIGBOX_ATTRACT_MODE_CONTROLLER_REJECTED enabled="
                + expectedEnabled
                + " wheelSteps=" + wheelSteps
                + " movementCycles=" + movementCycles
                + " filterSwitches=" + filterSwitches
                + " autoDelay=" + automaticDelay
                + " manualStarts="
                + bigBoxAttractMode.manualStartCount
                + " inputStops=" + bigBoxAttractMode.inputStopCount
                + " soundStatus="
                + bigBoxAttractMode.moveSoundStatus)
            Qt.exit(638)
            return
        }
        attractModeSmokeFinished = true
        Qt.quit()
    }

    function captureAttractModeSmokeAndExit() {
        function exitMode() {
            bigBoxAttractMode.clickReturnForSmoke()
            attractModeSmokePhase = 3
        }
        if (attractModeScreenshotPath.length === 0) {
            exitMode()
            return
        }
        if (attractModeScreenshotRequested)
            return
        attractModeScreenshotRequested = true
        bigBoxAttractMode.grabToImage(function(result) {
            if (!result.saveToFile(attractModeScreenshotPath)) {
                console.error(
                    "BIGBOX_ATTRACT_MODE_SCREENSHOT_SAVE_FAILED path="
                    + attractModeScreenshotPath)
                Qt.exit(637)
                return
            }
            exitMode()
        })
    }

    function captureScreensaverView(viewOrdinal) {
        if (screensaverCapturePending)
            return
        const path = screensaverScreenshotPrefix.length > 0
                   ? screensaverScreenshotPrefix
                     + "-view" + viewOrdinal + ".png" : ""
        function advance() {
            screensaverCapturePending = false
            if (viewOrdinal < 4) {
                bigBoxScreensaver.setSmokeView(viewOrdinal + 1)
                screensaverSmokePhase += 1
                screensaverPhaseStartedAt = Date.now()
            } else {
                bigBoxScreensaver.clickReturnForSmoke()
                screensaverSmokePhase = 7
            }
        }
        if (path.length === 0) {
            advance()
            return
        }
        screensaverCapturePending = true
        bigBoxScreensaver.grabToImage(function(result) {
            if (!result.saveToFile(path)) {
                console.error(
                    "BIGBOX_SCREENSAVER_SCREENSHOT_SAVE_FAILED path="
                    + path)
                Qt.exit(648 + viewOrdinal)
                return
            }
            advance()
        })
    }

    function finishScreensaverSmoke(expectedEnabled) {
        const automaticDelay = expectedEnabled
                             ? bigBoxScreensaver
                               .lastAutomaticDelayElapsedMs : 0
        if (!controller.report_big_box_screensaver_smoke_success(
                expectedEnabled,
                bigBoxScreensaver.swapCount,
                bigBoxScreensaver.selectionCount,
                automaticDelay,
                bigBoxScreensaver.manualStartCount > 0,
                bigBoxScreensaver.inputStopCount,
                bigBoxScreensaver.exploreCount,
                bigBoxScreensaver.presentedViewsMask,
                bigBoxScreensaver.videoPlaybackSeen,
                screensaverVideoReadySeen)) {
            console.error(
                "BIGBOX_SCREENSAVER_CONTROLLER_REJECTED enabled="
                + expectedEnabled
                + " swaps=" + bigBoxScreensaver.swapCount
                + " selections=" + bigBoxScreensaver.selectionCount
                + " autoDelay=" + automaticDelay
                + " manualStarts="
                + bigBoxScreensaver.manualStartCount
                + " inputStops=" + bigBoxScreensaver.inputStopCount
                + " explore=" + bigBoxScreensaver.exploreCount
                + " views=" + bigBoxScreensaver.presentedViewsMask
                + " videoState="
                + bigBoxScreensaver.videoPlaybackState
                + " videoStatus="
                + bigBoxScreensaver.videoMediaStatus)
            Qt.exit(647)
            return
        }
        screensaverSmokeFinished = true
        Qt.quit()
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
        if (!navigationAccessAvailable()) {
            controller.note_big_box_locked_action(
                "BigBoxShowAllGames")
            return false
        }
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
        if (!navigationRowAllowed(navigationList.currentIndex)) {
            navigationList.currentIndex = -1
            moveNavigationSelection(1)
        }
        navigationList.forceActiveFocus()
        return true
    }

    function activateNavigationRow(row) {
        const requestedKind = row <= 0 ? "all"
            : controller.big_box_navigation_entry_kind_at(row - 1)
        if (!guardSecurityNavigation(requestedKind))
            return false
        if (row <= 0) {
            activeNavigationName = "All Games"
            backgroundMusicContextKind = ""
            backgroundMusicContextName = "All Games"
            controller.apply_filters("", "")
        } else {
            const index = row - 1
            const kind = controller.big_box_navigation_entry_kind_at(index)
            const key = controller.big_box_navigation_entry_key_at(index)
            activeNavigationName = controller.big_box_navigation_entry_name_at(index)
            backgroundMusicContextKind = kind
            backgroundMusicContextName = activeNavigationName
            if (kind === "category")
                controller.apply_category_filter("", key)
            else if (kind === "playlist")
                controller.apply_playlist_filter("", key)
            else if (kind === "platform")
                controller.apply_filters("", key)
            else
                return false
        }
        navigationDrawer.close()
        gameList.currentIndex = gameList.count > 0 ? 0 : -1
        gameList.forceActiveFocus()
        return true
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

        function onLoadingChanged() {
            if (controller.loading) {
                window.startupLibraryLoadSeen = true
                return
            }
            if (!window.startupPresentationPending
                    || !window.startupLibraryLoadSeen
                    || startupPresentationOverlay.active)
                return
            if (window.startupSplashSmokeTest
                    && !window.startupSplashSmokeFinished)
                return
            window.startupPresentationPending = false
            Qt.callLater(function() {
                gameList.forceActiveFocus()
            })
        }

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

        function onBig_box_primary_monitor_indexChanged() {
            window.applyPrimaryMonitorScreen()
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
        if (library.length > 0) {
            startupProbeReadyBeforeLoad =
                controller.prepare_big_box_startup_presentation(
                    library)
                && controller.startup_presentation_ready
            beginApplicationStartupPresentation()
            controller.load_library(library)
            startupLibraryLoadSeen = controller.loading
        } else {
            controller.load_fixture()
        }
        applyPrimaryMonitorScreen()
        gameList.forceActiveFocus()
    }

    Timer {
        interval: 25
        repeat: true
        running: window.startupVideoSmokeTest
                 && !window.startupVideoSmokeFinished
        onTriggered: {
            if (controller.loading || controller.writing)
                return
            if (!window.startupVideoFrameSeen
                    && startupPresentationOverlay.active
                    && startupPresentationOverlay.duration > 0
                    && startupPresentationOverlay.mediaError
                       === MediaPlayer.NoError
                    && startupPresentationOverlay.playbackState
                       === MediaPlayer.PlayingState) {
                const expectedUrl = controller.startup_video_url_at(
                                      startupPresentationOverlay
                                      .selectedIndex).toString()
                const expectedName =
                    controller.startup_video_file_name_at(
                        startupPresentationOverlay.selectedIndex)
                if (expectedUrl.length === 0
                        || startupPresentationOverlay
                           .selectedSource.toString() !== expectedUrl
                        || startupPresentationOverlay.selectedName
                           !== expectedName) {
                    console.error(
                        "BIGBOX_STARTUP_VIDEO_SOURCE_MISMATCH"
                        + " index="
                        + startupPresentationOverlay.selectedIndex
                        + " actual="
                        + startupPresentationOverlay
                          .selectedSource.toString()
                        + " expected=" + expectedUrl)
                    Qt.exit(650)
                    return
                }
                window.startupVideoFrameSeen = true
                window.startupVideoScreenshotRequested = true
                const continueAfterFrame = function() {
                    window.startupVideoScreenshotReady = true
                    if (!window.startupVideoNaturalEnd
                            && !startupPresentationOverlay
                                .triggerSkipForSmoke()) {
                        console.error(
                            "BIGBOX_STARTUP_VIDEO_SKIP_MISSING")
                        Qt.exit(651)
                    }
                }
                if (window.startupVideoScreenshotPath.length === 0) {
                    continueAfterFrame()
                } else {
                    startupPresentationOverlay.grabToImage(function(result) {
                        if (!result.saveToFile(
                                window
                                .startupVideoScreenshotPath)) {
                            console.error(
                                "BIGBOX_STARTUP_VIDEO_SCREENSHOT_SAVE_FAILED path="
                                + window
                                  .startupVideoScreenshotPath)
                            Qt.exit(652)
                            return
                        }
                        continueAfterFrame()
                    })
                }
                return
            }
            if (!window.startupVideoCompletionSeen
                    || !window.startupVideoFrameSeen
                    || !window.startupVideoScreenshotReady)
                return
            const completionMatches =
                window.startupVideoNaturalEnd
                ? startupPresentationOverlay.endedNaturally
                  && !startupPresentationOverlay.skipped
                : startupPresentationOverlay.skipped
                  && !startupPresentationOverlay.endedNaturally
            if (!completionMatches
                    || startupPresentationOverlay.failed
                    || !controller
                        .report_startup_video_smoke_success(
                            startupPresentationOverlay
                            .selectedSource.toString(),
                            startupPresentationOverlay.selectedIndex,
                            startupPresentationOverlay.skipped,
                            startupPresentationOverlay.endedNaturally,
                            window.startupProbeReadyBeforeLoad)) {
                console.error(
                    "BIGBOX_STARTUP_VIDEO_CONTROLLER_REJECTED"
                    + " index="
                    + startupPresentationOverlay.selectedIndex
                    + " skipped="
                    + startupPresentationOverlay.skipped
                    + " natural="
                    + startupPresentationOverlay.endedNaturally
                    + " failed="
                    + startupPresentationOverlay.failed)
                Qt.exit(653)
                return
            }
            window.startupVideoSmokeFinished = true
            Qt.quit()
        }
    }

    Timer {
        interval: 20000
        running: window.startupVideoSmokeTest
                 && !window.startupVideoSmokeFinished
        onTriggered: {
            console.error(
                "BIGBOX_STARTUP_VIDEO_TIMEOUT"
                + " pending="
                + window.startupPresentationPending
                + " decision="
                + window.startupPresentationDecisionMade
                + " active=" + startupPresentationOverlay.active
                + " count="
                + controller.indexed_startup_video_count
                + " index="
                + startupPresentationOverlay.selectedIndex
                + " name="
                + startupPresentationOverlay.selectedName
                + " state="
                + startupPresentationOverlay.playbackState
                + " status="
                + startupPresentationOverlay.mediaStatus
                + " error="
                + startupPresentationOverlay.mediaError
                + " duration="
                + startupPresentationOverlay.duration
                + " position="
                + startupPresentationOverlay.position
                + " frame=" + window.startupVideoFrameSeen
                + " complete="
                + window.startupVideoCompletionSeen
                + " controller="
                + controller.status_message)
            Qt.exit(654)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.startupSplashAnySmokeTest
                 && !window.startupSplashSmokeFinished
        onTriggered: {
            if (controller.loading || controller.writing)
                return
            const expectedEnabled =
                window.startupSplashSmokeTest
            if (expectedEnabled) {
                if (!window.startupSplashWasVisible
                        || !window.startupSoundPlaybackSeen
                        || startupPresentationOverlay.soundFailed)
                    return
                const expectedUrl =
                    controller.startup_sound_url_at(
                        startupPresentationOverlay
                        .selectedSoundIndex).toString()
                const expectedName =
                    controller.startup_sound_file_name_at(
                        startupPresentationOverlay
                        .selectedSoundIndex)
                if (expectedUrl.length === 0
                        || startupPresentationOverlay
                           .selectedSoundSource.toString()
                           !== expectedUrl
                        || startupPresentationOverlay
                           .selectedSoundName !== expectedName) {
                    console.error(
                        "BIGBOX_STARTUP_SOUND_SOURCE_MISMATCH"
                        + " index="
                        + startupPresentationOverlay
                          .selectedSoundIndex
                        + " actual="
                        + startupPresentationOverlay
                          .selectedSoundSource.toString()
                        + " expected=" + expectedUrl)
                    Qt.exit(655)
                    return
                }
                if (!window.startupSplashScreenshotReady) {
                    if (window.startupSplashScreenshotRequested)
                        return
                    window.startupSplashScreenshotRequested = true
                    startupPresentationOverlay.grabToImage(
                        function(result) {
                            if (!result.saveToFile(
                                    window
                                    .startupSplashScreenshotPath)) {
                                console.error(
                                    "BIGBOX_STARTUP_SPLASH_SCREENSHOT_SAVE_FAILED path="
                                    + window
                                      .startupSplashScreenshotPath)
                                Qt.exit(656)
                                return
                            }
                            window.startupSplashScreenshotReady = true
                        })
                    return
                }
            }
            const selectedSoundUrl = expectedEnabled
                ? startupPresentationOverlay
                  .selectedSoundSource.toString()
                : ""
            const selectedSoundIndex = expectedEnabled
                ? startupPresentationOverlay.selectedSoundIndex
                : -1
            if (!controller.report_startup_splash_smoke_success(
                    expectedEnabled,
                    selectedSoundUrl,
                    selectedSoundIndex,
                    window.startupProbeReadyBeforeLoad,
                    window.startupSplashWasVisible,
                    window.startupSoundPlaybackSeen)) {
                console.error(
                    "BIGBOX_STARTUP_SPLASH_CONTROLLER_REJECTED"
                    + " enabled=" + expectedEnabled
                    + " visible="
                    + window.startupSplashWasVisible
                    + " audio="
                    + window.startupSoundPlaybackSeen
                    + " sounds="
                    + controller.indexed_startup_sound_count
                    + " index="
                    + selectedSoundIndex
                    + " ready="
                    + window.startupProbeReadyBeforeLoad)
                Qt.exit(657)
                return
            }
            window.startupSplashSmokeFinished = true
            window.startupPresentationPending = false
            Qt.quit()
        }
    }

    Timer {
        interval: 20000
        running: window.startupSplashAnySmokeTest
                 && !window.startupSplashSmokeFinished
        onTriggered: {
            console.error(
                "BIGBOX_STARTUP_SPLASH_TIMEOUT"
                + " enabled="
                + window.startupSplashSmokeTest
                + " pending="
                + window.startupPresentationPending
                + " decision="
                + window.startupPresentationDecisionMade
                + " ready="
                + window.startupProbeReadyBeforeLoad
                + " loading=" + controller.loading
                + " visible="
                + window.startupSplashWasVisible
                + " sounds="
                + controller.indexed_startup_sound_count
                + " index="
                + startupPresentationOverlay
                  .selectedSoundIndex
                + " name="
                + startupPresentationOverlay
                  .selectedSoundName
                + " state="
                + startupPresentationOverlay
                  .soundPlaybackState
                + " status="
                + startupPresentationOverlay
                  .soundMediaStatus
                + " error="
                + startupPresentationOverlay
                  .soundMediaError
                + " duration="
                + startupPresentationOverlay
                  .soundDuration
                + " started="
                + startupPresentationOverlay.soundStarted
                + " failed="
                + startupPresentationOverlay.soundFailed
                + " controller="
                + controller.status_message)
            Qt.exit(658)
        }
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
                        || bigBoxGameDetails.mediaCount !== 10
                        || bigBoxGameDetails.selectedMediaIndex !== 8
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
                if (bigBoxGameDetails.selectedMediaIndex !== 7
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
                    if (!bigBoxGameDetails.clickMediaThumbnailForSmoke(8)) {
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
                if (bigBoxGameDetails.selectedMediaIndex !== 8
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
                        "fixture-adventure", 0, 8,
                        controller.game_media_url_at(
                            "fixture-adventure", 0).toString(),
                        controller.game_media_url_at(
                            "fixture-adventure", 8).toString())) {
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
                        || bigBoxGameDetails.mediaCount !== 10)
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
                        || bigBoxImageViewer.imageCount !== 8
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
                            ["fixture-adventure", "fixture-racer",
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
                if (window.libraryOrderSmokeRandomRow < 0
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
                            ["fixture-adventure", "fixture-racer",
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
        interval: 20
        repeat: true
        running: window.inputEditorSmokeTest
                 && !window.inputEditorSmokeFinished
        onTriggered: {
            if (controller.loading
                    || window.startupPresentationPending
                    || controller.library_path.length === 0)
                return
            if (window.inputEditorSmokePhase === 0) {
                if (controller.writing
                        || controller.big_box_input_revision < 1)
                    return
                window.inputEditorSmokeStartRevision =
                    controller.big_box_input_revision
                bigBoxInputSettings.openEditor()
                window.inputEditorSmokePhase = 1
            } else if (window.inputEditorSmokePhase === 1) {
                if (!bigBoxInputSettings.opened)
                    return
                if (window.inputEditorSmokeScreenshotPath.length > 0
                        && !window
                            .inputEditorSmokeScreenshotRequested) {
                    window.inputEditorSmokeScreenshotRequested = true
                    const captureStarted =
                        bigBoxInputSettings.smokeCaptureTarget.grabToImage(
                        function(result) {
                            if (!result.saveToFile(
                                    window
                                    .inputEditorSmokeScreenshotPath)) {
                                console.error(
                                    "BIGBOX_INPUT_EDITOR_SCREENSHOT_SAVE_FAILED path="
                                    + window
                                      .inputEditorSmokeScreenshotPath)
                                Qt.exit(662)
                                return
                            }
                            window.inputEditorSmokeScreenshotReady = true
                        })
                    if (!captureStarted)
                        window.inputEditorSmokeScreenshotRequested = false
                    return
                }
                if (window.inputEditorSmokeScreenshotRequested
                        && !window.inputEditorSmokeScreenshotReady)
                    return
                if (!bigBoxInputSettings.runSmokeExercise()) {
                    console.error(
                        "BIGBOX_INPUT_EDITOR_SMOKE_EXERCISE_FAILED")
                    Qt.exit(659)
                    return
                }
                window.inputEditorSmokePhase = 2
            } else if (window.inputEditorSmokePhase === 2) {
                if (controller.writing
                        || bigBoxInputSettings.opened
                        || controller.big_box_input_revision
                           === window.inputEditorSmokeStartRevision)
                    return
                if (!controller
                        .report_big_box_input_editor_smoke_success()) {
                    console.error(
                        "BIGBOX_INPUT_EDITOR_SMOKE_CONTROLLER_REJECTED"
                        + " status=" + controller.status_message)
                    Qt.exit(660)
                    return
                }
                window.inputEditorSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 8000
        repeat: false
        running: window.inputEditorSmokeTest
                 && !window.inputEditorSmokeFinished
        onTriggered: {
            controller.report_big_box_input_editor_smoke_success()
            console.error(
                "BIGBOX_INPUT_EDITOR_SMOKE_TIMEOUT phase="
                + window.inputEditorSmokePhase
                + " opened=" + bigBoxInputSettings.opened
                + " writing=" + controller.writing
                + " revision=" + controller.big_box_input_revision
                + " status=" + controller.status_message)
            Qt.exit(661)
        }
    }

    Timer {
        interval: 20
        repeat: true
        running: window.marqueeSmokeTest
                 && !window.marqueeSmokeFinished
        onTriggered: {
            if (controller.loading
                    || window.startupPresentationPending
                    || controller.library_path.length === 0)
                return
            if (window.marqueeSmokePhase === 0) {
                if (controller.writing
                        || controller.big_box_marquee_settings_revision < 1)
                    return
                if (window.selectedBigBoxGameId
                        !== "fixture-adventure") {
                    const row =
                        controller.row_for_game_id("fixture-adventure")
                    if (row < 0) {
                        console.error(
                            "BIGBOX_MARQUEE_SMOKE_GAME_MISSING")
                        Qt.exit(663)
                        return
                    }
                    gameList.currentIndex = row
                    gameList.positionViewAtIndex(
                        row, ListView.Center)
                    return
                }
                window.marqueeSmokeStartRevision =
                    controller.big_box_marquee_settings_revision
                bigBoxMarqueeSettings.openEditor()
                window.marqueeSmokePhase = 1
            } else if (window.marqueeSmokePhase === 1) {
                if (!bigBoxMarqueeSettings.opened)
                    return
                if (!bigBoxMarqueeSettings.runSmokeExercise()) {
                    console.error(
                        "BIGBOX_MARQUEE_SMOKE_SETTINGS_FAILED")
                    Qt.exit(664)
                    return
                }
                window.marqueeSmokePhase = 2
            } else if (window.marqueeSmokePhase === 2) {
                if (controller.writing
                        || bigBoxMarqueeSettings.opened
                        || controller.big_box_marquee_settings_revision
                           === window.marqueeSmokeStartRevision
                        || bigBoxMarquee.resolvedMonitorIndex !== 0
                        || !bigBoxMarquee.visible)
                    return
                if (!window.marqueeVideoReadySeen) {
                    if (!bigBoxMarquee.videoReady)
                        return
                    window.marqueeVideoReadySeen = true
                    // QQuickItem::grabToImage cannot read a software
                    // VideoOutput texture on all Qt backends. Decoder
                    // readiness is proven above; capture the indexed direct
                    // marquee image for deterministic rendered evidence.
                    bigBoxMarquee.suspendVideoForCapture = true
                    return
                }
                if (bigBoxMarquee.directImageStatus !== Image.Ready)
                    return
                if (!window.marqueeGameScreenshotRequested) {
                    if (window.marqueeGameScreenshotPath.length === 0) {
                        window.marqueeGameScreenshotRequested = true
                        window.marqueeGameScreenshotReady = true
                    } else {
                        window.marqueeGameScreenshotRequested = true
                        const started =
                            bigBoxMarquee.captureTarget.grabToImage(
                                function(result) {
                                    if (!result.saveToFile(
                                            window
                                            .marqueeGameScreenshotPath)) {
                                        console.error(
                                            "BIGBOX_MARQUEE_GAME_SCREENSHOT_SAVE_FAILED path="
                                            + window
                                              .marqueeGameScreenshotPath)
                                        Qt.exit(665)
                                        return
                                    }
                                    window.marqueeGameScreenshotReady = true
                                })
                        if (!started)
                            window.marqueeGameScreenshotRequested = false
                    }
                    return
                }
                if (!window.marqueeGameScreenshotReady)
                    return
                const platformIndex =
                    window.bigBoxNavigationIndex(
                        "platform", "Fixture Console")
                if (platformIndex < 0) {
                    console.error(
                        "BIGBOX_MARQUEE_SMOKE_PLATFORM_MISSING")
                    Qt.exit(666)
                    return
                }
                navigationDrawer.open()
                navigationList.currentIndex = platformIndex + 1
                navigationList.positionViewAtIndex(
                    navigationList.currentIndex, ListView.Contain)
                window.updateMarqueeNavigationPreview()
                window.marqueeSmokePhase = 3
            } else if (window.marqueeSmokePhase === 3) {
                if (window.marqueeContextKind !== "platform"
                        || window.marqueeContextName
                           !== "Fixture Console"
                        || bigBoxMarquee.platformBannerUrl
                           .toString().length === 0
                        || bigBoxMarquee.directImageStatus
                           !== Image.Ready)
                    return
                if (!window.marqueePlatformScreenshotRequested) {
                    if (window.marqueePlatformScreenshotPath.length === 0) {
                        window.marqueePlatformScreenshotRequested = true
                        window.marqueePlatformScreenshotReady = true
                    } else {
                        window.marqueePlatformScreenshotRequested = true
                        const started =
                            bigBoxMarquee.captureTarget.grabToImage(
                                function(result) {
                                    if (!result.saveToFile(
                                            window
                                            .marqueePlatformScreenshotPath)) {
                                        console.error(
                                            "BIGBOX_MARQUEE_PLATFORM_SCREENSHOT_SAVE_FAILED path="
                                            + window
                                              .marqueePlatformScreenshotPath)
                                        Qt.exit(667)
                                        return
                                    }
                                    window.marqueePlatformScreenshotReady = true
                                })
                        if (!started)
                            window.marqueePlatformScreenshotRequested = false
                    }
                    return
                }
                if (!window.marqueePlatformScreenshotReady)
                    return
                window.marqueeSmokePhase = 4
            } else if (window.marqueeSmokePhase === 4) {
                if (!controller.report_big_box_marquee_smoke_success(
                        bigBoxMarquee.screenCount,
                        bigBoxMarquee.resolvedMonitorIndex,
                        window.selectedBigBoxGameId,
                        window.marqueeContextName,
                        bigBoxMarquee.gameVideoUrl.toString(),
                        bigBoxMarquee.gameImageUrl.toString(),
                        bigBoxMarquee.platformBannerUrl.toString())) {
                    console.error(
                        "BIGBOX_MARQUEE_SMOKE_CONTROLLER_REJECTED"
                        + " screenCount=" + bigBoxMarquee.screenCount
                        + " monitor="
                        + bigBoxMarquee.resolvedMonitorIndex
                        + " videoReadySeen="
                        + window.marqueeVideoReadySeen
                        + " imageStatus="
                        + bigBoxMarquee.directImageStatus
                        + " revision="
                        + controller.big_box_marquee_settings_revision
                        + " status=" + controller.status_message)
                    Qt.exit(668)
                    return
                }
                window.marqueeSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 12000
        repeat: false
        running: window.marqueeSmokeTest
                 && !window.marqueeSmokeFinished
        onTriggered: {
            controller.report_big_box_marquee_smoke_success(
                bigBoxMarquee.screenCount,
                bigBoxMarquee.resolvedMonitorIndex,
                window.selectedBigBoxGameId,
                window.marqueeContextName,
                bigBoxMarquee.gameVideoUrl.toString(),
                bigBoxMarquee.gameImageUrl.toString(),
                bigBoxMarquee.platformBannerUrl.toString())
            console.error(
                "BIGBOX_MARQUEE_SMOKE_TIMEOUT phase="
                + window.marqueeSmokePhase
                + " visible=" + bigBoxMarquee.visible
                + " monitor=" + bigBoxMarquee.resolvedMonitorIndex
                + " videoReadySeen=" + window.marqueeVideoReadySeen
                + " videoStatus=" + bigBoxMarquee.videoMediaStatus
                + " videoState=" + bigBoxMarquee.videoPlaybackState
                + " imageStatus=" + bigBoxMarquee.directImageStatus
                + " writing=" + controller.writing
                + " revision="
                + controller.big_box_marquee_settings_revision
                + " status=" + controller.status_message)
            Qt.exit(669)
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

    Timer {
        interval: 25
        repeat: true
        running: window.securitySmokeTest
                 && !window.securitySmokeFinished
        onTriggered: {
            if (controller.loading
                    || window.startupPresentationPending
                    || controller.library_path.length === 0)
                return
            if (window.securitySmokePhase === 0) {
                if (!controller.big_box_pin_configured
                        || !controller.big_box_locked
                        || controller
                           .big_box_security_permission_count() !== 32
                        || !controller
                            .big_box_action_allowed_while_locked(
                                "BigBoxPlayGame")
                        || controller
                           .big_box_action_allowed_while_locked(
                               "BigBoxExit")
                        || controller
                           .big_box_navigation_allowed_while_locked(
                               "platform")
                        || !controller
                            .big_box_navigation_allowed_while_locked(
                                "all")) {
                    window.failSecuritySmoke(
                        "BIGBOX_SECURITY_INITIAL_POLICY_MISMATCH",
                        675)
                    return
                }
                window.securitySmokeStartRevision =
                    controller.big_box_security_settings_revision
                if (window.guardSecurityAction("BigBoxExit")
                        || window.guardSecurityNavigation("platform")) {
                    window.failSecuritySmoke(
                        "BIGBOX_SECURITY_BLOCKED_ACTION_ESCAPED",
                        676)
                    return
                }
                window.securitySmokeBlockedActions = 2
                if (!window.requestLockUnlock()) {
                    window.failSecuritySmoke(
                        "BIGBOX_SECURITY_UNLOCK_POPUP_MISSING",
                        677)
                    return
                }
                window.securitySmokePhase = 1
            } else if (window.securitySmokePhase === 1) {
                if (!bigBoxUnlockPopup.opened)
                    return
                if (window.securityPinScreenshotPath.length === 0) {
                    window.securitySmokePhase = 2
                    return
                }
                if (window.securityPinScreenshotRequested)
                    return
                window.securityPinScreenshotRequested = true
                bigBoxUnlockPopup.smokeCaptureTarget.grabToImage(
                    function(result) {
                        if (!result.saveToFile(
                                window.securityPinScreenshotPath)) {
                            window.failSecuritySmoke(
                                "BIGBOX_SECURITY_PIN_SCREENSHOT_FAILED",
                                678)
                            return
                        }
                        window.securitySmokePhase = 2
                    })
            } else if (window.securitySmokePhase === 2) {
                if (!bigBoxUnlockPopup.opened
                        || !bigBoxUnlockPopup.runSmokeEntry("0000")) {
                    window.failSecuritySmoke(
                        "BIGBOX_SECURITY_WRONG_PIN_ENTRY_FAILED",
                        679)
                    return
                }
                window.securitySmokeFailedUnlocks = 1
                window.securitySmokePhase = 3
            } else if (window.securitySmokePhase === 3) {
                if (!bigBoxUnlockPopup.opened)
                    return
                if (!bigBoxUnlockPopup.runSmokeEntry("2580")) {
                    window.failSecuritySmoke(
                        "BIGBOX_SECURITY_ORIGINAL_PIN_ENTRY_FAILED",
                        680)
                    return
                }
                window.securitySmokeSuccessfulUnlocks = 1
                window.securitySmokePhase = 4
            } else if (window.securitySmokePhase === 4) {
                if (controller.big_box_locked
                        || bigBoxUnlockPopup.opened)
                    return
                if (!bigBoxSecuritySettings.openEditor()) {
                    window.failSecuritySmoke(
                        "BIGBOX_SECURITY_EDITOR_OPEN_FAILED",
                        681)
                    return
                }
                window.securitySmokePhase = 5
            } else if (window.securitySmokePhase === 5) {
                if (!bigBoxSecuritySettings.opened
                        || controller
                           .big_box_security_permission_count() !== 32)
                    return
                if (window.securityEditorScreenshotPath.length === 0) {
                    window.securitySmokePhase = 6
                    return
                }
                if (window.securityEditorScreenshotRequested)
                    return
                window.securityEditorScreenshotRequested = true
                bigBoxSecuritySettings.smokeCaptureTarget.grabToImage(
                    function(result) {
                        if (!result.saveToFile(
                                window.securityEditorScreenshotPath)) {
                            window.failSecuritySmoke(
                                "BIGBOX_SECURITY_EDITOR_SCREENSHOT_FAILED",
                                682)
                            return
                        }
                        window.securitySmokePhase = 6
                    })
            } else if (window.securitySmokePhase === 6) {
                if (!bigBoxSecuritySettings.beginSetPin()) {
                    window.failSecuritySmoke(
                        "BIGBOX_SECURITY_SET_PIN_POPUP_FAILED",
                        683)
                    return
                }
                window.securitySmokePhase = 7
            } else if (window.securitySmokePhase === 7) {
                if (!bigBoxSecuritySettings.pinPopup.opened)
                    return
                if (!bigBoxSecuritySettings.pinPopup
                        .runSmokeEntry("8642")) {
                    window.failSecuritySmoke(
                        "BIGBOX_SECURITY_FIRST_NEW_PIN_FAILED",
                        684)
                    return
                }
                window.securitySmokePhase = 8
            } else if (window.securitySmokePhase === 8) {
                if (!bigBoxSecuritySettings.pinPopup.opened
                        || bigBoxSecuritySettings.pinPurpose
                           !== "repeat")
                    return
                if (!bigBoxSecuritySettings.pinPopup
                        .runSmokeEntry("8642")) {
                    window.failSecuritySmoke(
                        "BIGBOX_SECURITY_REPEAT_NEW_PIN_FAILED",
                        685)
                    return
                }
                window.securitySmokePhase = 9
            } else if (window.securitySmokePhase === 9) {
                if (bigBoxSecuritySettings.pinPopup.opened
                        || bigBoxSecuritySettings.pinChange !== "set")
                    return
                if (!bigBoxSecuritySettings.setPermission(
                        "AllowExitWhileUnlocked", true)
                        || !bigBoxSecuritySettings.setPermission(
                            "AllowChangeFilterPlatformsWhileLocked",
                            false)
                        || !bigBoxSecuritySettings.saveChanges()) {
                    window.failSecuritySmoke(
                        "BIGBOX_SECURITY_SAVE_START_FAILED",
                        686)
                    return
                }
                window.securitySmokePhase = 10
            } else if (window.securitySmokePhase === 10) {
                if (controller.writing
                        || bigBoxSecuritySettings.opened)
                    return
                if (controller.big_box_security_settings_revision
                        !== window.securitySmokeStartRevision + 1
                        || controller.big_box_locked
                        || !controller.big_box_pin_configured) {
                    window.failSecuritySmoke(
                        "BIGBOX_SECURITY_COMMITTED_POLICY_MISMATCH",
                        687)
                    return
                }
                if (!controller.lock_big_box()) {
                    window.failSecuritySmoke(
                        "BIGBOX_SECURITY_RELOCK_FAILED", 692)
                    return
                }
                if (!controller
                        .big_box_action_allowed_while_locked(
                            "BigBoxExit")
                        || controller
                           .big_box_navigation_allowed_while_locked(
                               "platform")) {
                    window.failSecuritySmoke(
                        "BIGBOX_SECURITY_RELOCKED_POLICY_MISMATCH",
                        694)
                    return
                }
                if (!window.requestLockUnlock()) {
                    window.failSecuritySmoke(
                        "BIGBOX_SECURITY_REPLACEMENT_POPUP_FAILED",
                        693)
                    return
                }
                window.securitySmokePhase = 11
            } else if (window.securitySmokePhase === 11) {
                if (!bigBoxUnlockPopup.opened)
                    return
                if (!bigBoxUnlockPopup.runSmokeEntry("2580")) {
                    window.failSecuritySmoke(
                        "BIGBOX_SECURITY_REPLACED_PIN_ACCEPTED",
                        688)
                    return
                }
                window.securitySmokeFailedUnlocks = 2
                window.securitySmokePhase = 12
            } else if (window.securitySmokePhase === 12) {
                if (!bigBoxUnlockPopup.opened)
                    return
                if (!bigBoxUnlockPopup.runSmokeEntry("8642")) {
                    window.failSecuritySmoke(
                        "BIGBOX_SECURITY_NEW_PIN_ENTRY_FAILED",
                        689)
                    return
                }
                window.securitySmokeSuccessfulUnlocks = 2
                window.securitySmokePhase = 13
            } else if (window.securitySmokePhase === 13) {
                if (controller.big_box_locked
                        || bigBoxUnlockPopup.opened)
                    return
                if (!controller.report_big_box_security_smoke_success(
                        window.securitySmokeStartRevision,
                        window.securitySmokeBlockedActions,
                        window.securitySmokeFailedUnlocks,
                        window.securitySmokeSuccessfulUnlocks)) {
                    window.failSecuritySmoke(
                        "BIGBOX_SECURITY_CONTROLLER_REJECTED",
                        690)
                    return
                }
                window.securitySmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 15000
        running: window.securitySmokeTest
                 && !window.securitySmokeFinished
        onTriggered: window.failSecuritySmoke(
            "BIGBOX_SECURITY_SMOKE_TIMEOUT",
            700 + window.securitySmokePhase)
    }

    Timer {
        interval: 25
        repeat: true
        running: window.gameActionsSmokeTest
                 && !window.gameActionsSmokeFinished
        onTriggered: {
            if (controller.loading
                    || window.startupPresentationPending
                    || controller.library_path.length === 0)
                return
            if (window.gameActionsSmokePhase === 0) {
                if (window.selectedBigBoxGameId.length === 0)
                    return
                if (window.selectedBigBoxGameId
                        !== "fixture-adventure"
                        || !window.selectedBigBoxGameFavorite
                        || window.selectedBigBoxGameStarRating !== 4
                        || Math.abs(
                            window.selectedBigBoxGameStarRatingFloat
                            - 4.5) > 0.000001
                        || !controller.big_box_locked
                        || !controller
                           .big_box_show_star_next_to_favorited_games
                        || !controller.big_box_show_favorited_games_first
                        || !controller.big_box_show_game_favorite
                        || !controller.big_box_show_game_menu_favorite
                        || !controller
                           .big_box_show_game_menu_star_rating
                        || !controller.big_box_show_game_star_rating
                        || !controller
                            .big_box_action_allowed_while_locked(
                                "BigBoxSetStarRating")
                        || controller
                           .big_box_action_allowed_while_locked(
                               "BigBoxFavoriteGames")) {
                    window.failGameActionsSmoke(
                        "BIGBOX_GAME_ACTIONS_INITIAL_MISMATCH",
                        730)
                    return
                }
                window.gameActionsSmokeStartRevision =
                    controller.big_box_game_state_revision
                window.gameActionsFavoriteFirstSeen =
                    controller.game_id_at(0)
                    === "fixture-adventure"
                if (window.guardSecurityAction(
                        "BigBoxFavoriteGames")) {
                    window.failGameActionsSmoke(
                        "BIGBOX_GAME_ACTIONS_LOCKED_FAVORITE_ESCAPED",
                        731)
                    return
                }
                window.gameActionsBlockedFavoriteSeen = true
                if (!window.dispatchBigBoxInputAction(
                        "BigBoxSetStarRating")) {
                    window.failGameActionsSmoke(
                        "BIGBOX_GAME_ACTIONS_POPUP_MISSING",
                        732)
                    return
                }
                window.gameActionsSmokePhase = 1
            } else if (window.gameActionsSmokePhase === 1) {
                if (!bigBoxStarRatingPopup.opened)
                    return
                window.gameActionsPopupSeen = true
                if (window.gameActionsScreenshotPath.length === 0) {
                    window.gameActionsSmokePhase = 2
                    return
                }
                if (window.gameActionsScreenshotRequested)
                    return
                window.gameActionsScreenshotRequested = true
                bigBoxStarRatingPopup.smokeCaptureTarget.grabToImage(
                    function(result) {
                        if (!result.saveToFile(
                                window.gameActionsScreenshotPath)) {
                            window.failGameActionsSmoke(
                                "BIGBOX_GAME_ACTIONS_SCREENSHOT_FAILED",
                                733)
                            return
                        }
                        window.gameActionsSmokePhase = 2
                    })
            } else if (window.gameActionsSmokePhase === 2) {
                if (!bigBoxStarRatingPopup.opened
                        || !bigBoxStarRatingPopup
                            .runSmokeSetRating(2.5)) {
                    window.failGameActionsSmoke(
                        "BIGBOX_GAME_ACTIONS_RATING_SAVE_FAILED",
                        734)
                    return
                }
                window.gameActionsSmokePhase = 3
            } else if (window.gameActionsSmokePhase === 3) {
                if (controller.writing
                        || controller.big_box_game_state_revision
                           !== window
                               .gameActionsSmokeStartRevision + 1)
                    return
                if (!window.selectedBigBoxGameFavorite
                        || Math.abs(
                            window.selectedBigBoxGameStarRatingFloat
                            - 2.5) > 0.000001
                        || !window.requestLockUnlock()) {
                    window.failGameActionsSmoke(
                        "BIGBOX_GAME_ACTIONS_RATING_COMMIT_MISMATCH",
                        735)
                    return
                }
                window.gameActionsSmokePhase = 4
            } else if (window.gameActionsSmokePhase === 4) {
                if (!bigBoxUnlockPopup.opened)
                    return
                if (!bigBoxUnlockPopup.runSmokeEntry("2580")) {
                    window.failGameActionsSmoke(
                        "BIGBOX_GAME_ACTIONS_UNLOCK_FAILED",
                        736)
                    return
                }
                window.gameActionsSmokePhase = 5
            } else if (window.gameActionsSmokePhase === 5) {
                if (controller.big_box_locked
                        || bigBoxUnlockPopup.opened)
                    return
                if (!window.toggleSelectedFavorite()) {
                    window.failGameActionsSmoke(
                        "BIGBOX_GAME_ACTIONS_FAVORITE_SAVE_FAILED",
                        737)
                    return
                }
                window.gameActionsSmokePhase = 6
            } else if (window.gameActionsSmokePhase === 6) {
                if (controller.writing
                        || controller.big_box_game_state_revision
                           !== window
                               .gameActionsSmokeStartRevision + 2)
                    return
                if (window.selectedBigBoxGameFavorite
                        || Math.abs(
                            window.selectedBigBoxGameStarRatingFloat
                            - 2.5) > 0.000001
                        || !controller
                            .report_big_box_game_actions_smoke_success(
                                window.gameActionsSmokeStartRevision,
                                window.selectedBigBoxGameId,
                                window.selectedBigBoxGameFavorite,
                                window
                                .selectedBigBoxGameStarRatingFloat,
                                window.gameActionsFavoriteFirstSeen,
                                window.gameActionsPopupSeen,
                                window.gameActionsBlockedFavoriteSeen)) {
                    window.failGameActionsSmoke(
                        "BIGBOX_GAME_ACTIONS_CONTROLLER_REJECTED",
                        738)
                    return
                }
                window.gameActionsSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 20000
        running: window.gameActionsSmokeTest
                 && !window.gameActionsSmokeFinished
        onTriggered: window.failGameActionsSmoke(
            "BIGBOX_GAME_ACTIONS_SMOKE_TIMEOUT",
            740 + window.gameActionsSmokePhase)
    }

    Timer {
        interval: 25
        repeat: true
        running: window.playlistActionsSmokeTest
                 && !window.playlistActionsSmokeFinished
        onTriggered: {
            if (controller.loading
                    || window.startupPresentationPending
                    || controller.library_path.length === 0)
                return
            if (window.playlistActionsSmokePhase === 0) {
                if (window.selectedBigBoxGameId.length === 0)
                    return
                window.refreshSelectedBigBoxPlaylistAction()
                const payload = window.selectedBigBoxPlaylistAction
                if (window.selectedBigBoxGameId
                        !== "fixture-adventure"
                        || !controller.big_box_locked
                        || !controller
                            .big_box_show_game_menu_playlist_actions
                        || window.selectedBigBoxPlaylistAddTargetCount
                           !== 1
                        || payload.addTargets[0].playlistId
                           !== "manual-playlist"
                        || payload.removeCurrent !== null
                        || window.securityActionAllowed(
                            "BigBoxPlaylistActions")) {
                    window.failPlaylistActionsSmoke(
                        "BIGBOX_PLAYLIST_ACTIONS_INITIAL_MISMATCH",
                        750)
                    return
                }
                window.playlistActionsSmokeStartRevision =
                    controller.big_box_playlist_membership_revision
                window.playlistActionsSmokeGameId =
                    window.selectedBigBoxGameId
                window.playlistActionsSmokePlaylistId =
                    payload.addTargets[0].playlistId
                window.playlistActionsSmokeAddTargetCount =
                    window.selectedBigBoxPlaylistAddTargetCount
                if (window.guardSecurityAction(
                        "BigBoxPlaylistActions")) {
                    window.failPlaylistActionsSmoke(
                        "BIGBOX_PLAYLIST_ACTIONS_LOCKED_ACTION_ESCAPED",
                        751)
                    return
                }
                window.playlistActionsSmokeBlockedSeen = true
                if (!window.requestLockUnlock()) {
                    window.failPlaylistActionsSmoke(
                        "BIGBOX_PLAYLIST_ACTIONS_UNLOCK_POPUP_MISSING",
                        752)
                    return
                }
                window.playlistActionsSmokePhase = 1
            } else if (window.playlistActionsSmokePhase === 1) {
                if (!bigBoxUnlockPopup.opened)
                    return
                if (!bigBoxUnlockPopup.runSmokeEntry("2580")) {
                    window.failPlaylistActionsSmoke(
                        "BIGBOX_PLAYLIST_ACTIONS_UNLOCK_FAILED",
                        753)
                    return
                }
                window.playlistActionsSmokePhase = 2
            } else if (window.playlistActionsSmokePhase === 2) {
                if (controller.big_box_locked
                        || bigBoxUnlockPopup.opened)
                    return
                window.refreshSelectedBigBoxPlaylistAction()
                if (!window.openSelectedPlaylistPopup()) {
                    window.failPlaylistActionsSmoke(
                        "BIGBOX_PLAYLIST_ACTIONS_POPUP_MISSING",
                        754)
                    return
                }
                window.playlistActionsSmokePhase = 3
            } else if (window.playlistActionsSmokePhase === 3) {
                if (!bigBoxPlaylistPopup.opened)
                    return
                window.playlistActionsSmokePopupSeen = true
                if (window.playlistActionsSmokeScreenshotPath.length
                        === 0) {
                    window.playlistActionsSmokePhase = 4
                    return
                }
                if (window.playlistActionsSmokeScreenshotRequested)
                    return
                window.playlistActionsSmokeScreenshotRequested = true
                bigBoxPlaylistPopup.smokeCaptureTarget.grabToImage(
                    function(result) {
                        if (!result.saveToFile(
                                window
                                .playlistActionsSmokeScreenshotPath)) {
                            window.failPlaylistActionsSmoke(
                                "BIGBOX_PLAYLIST_ACTIONS_SCREENSHOT_FAILED",
                                755)
                            return
                        }
                        window.playlistActionsSmokePhase = 4
                    })
            } else if (window.playlistActionsSmokePhase === 4) {
                if (!bigBoxPlaylistPopup.opened
                        || !bigBoxPlaylistPopup.runSmokeSelect(0)) {
                    window.failPlaylistActionsSmoke(
                        "BIGBOX_PLAYLIST_ACTIONS_ADD_FAILED",
                        756)
                    return
                }
                window.playlistActionsSmokePhase = 5
            } else if (window.playlistActionsSmokePhase === 5) {
                if (controller.writing
                        || controller
                           .big_box_playlist_membership_revision
                           !== window
                              .playlistActionsSmokeStartRevision + 1)
                    return
                window.refreshSelectedBigBoxPlaylistAction()
                if (window.selectedBigBoxPlaylistAddTargetCount
                        !== 0) {
                    window.failPlaylistActionsSmoke(
                        "BIGBOX_PLAYLIST_ACTIONS_ADD_COMMIT_MISMATCH",
                        757)
                    return
                }
                const index = window.bigBoxNavigationIndex(
                    "playlist",
                    window.playlistActionsSmokePlaylistId)
                if (index < 0
                        || !window.activateNavigationRow(index + 1)) {
                    window.failPlaylistActionsSmoke(
                        "BIGBOX_PLAYLIST_ACTIONS_NAVIGATION_MISSING",
                        758)
                    return
                }
                window.playlistActionsSmokePhase = 6
            } else if (window.playlistActionsSmokePhase === 6) {
                if (controller.navigation_filter_kind
                        !== "playlist"
                        || controller.navigation_filter_key
                           !== window
                              .playlistActionsSmokePlaylistId
                        || window.selectedBigBoxGameId
                           !== window.playlistActionsSmokeGameId)
                    return
                window.refreshSelectedBigBoxPlaylistAction()
                if (window.selectedBigBoxPlaylistRemoveCurrent
                        === null
                        || window
                           .selectedBigBoxPlaylistRemoveCurrent
                           .playlistId
                           !== window
                              .playlistActionsSmokePlaylistId
                        || !window
                            .removeSelectedFromCurrentPlaylist()) {
                    window.failPlaylistActionsSmoke(
                        "BIGBOX_PLAYLIST_ACTIONS_REMOVE_FAILED",
                        759)
                    return
                }
                window.playlistActionsSmokePhase = 7
            } else if (window.playlistActionsSmokePhase === 7) {
                if (controller.writing
                        || controller
                           .big_box_playlist_membership_revision
                           !== window
                              .playlistActionsSmokeStartRevision + 2
                        || controller.filtered_count !== 1
                        || window.selectedBigBoxGameId
                           !== "fixture-puzzle")
                    return
                if (!controller
                        .report_big_box_playlist_actions_smoke_success(
                            window
                            .playlistActionsSmokeStartRevision,
                            window.playlistActionsSmokeGameId,
                            window.playlistActionsSmokePlaylistId,
                            window.playlistActionsSmokePopupSeen,
                            window.playlistActionsSmokeBlockedSeen,
                            window
                            .playlistActionsSmokeAddTargetCount)) {
                    window.failPlaylistActionsSmoke(
                        "BIGBOX_PLAYLIST_ACTIONS_CONTROLLER_REJECTED",
                        760)
                    return
                }
                window.playlistActionsSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 20000
        running: window.playlistActionsSmokeTest
                 && !window.playlistActionsSmokeFinished
        onTriggered: window.failPlaylistActionsSmoke(
            "BIGBOX_PLAYLIST_ACTIONS_SMOKE_TIMEOUT",
            761 + window.playlistActionsSmokePhase)
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
        onOpened: {
            window.updateMarqueeNavigationPreview()
            navigationList.forceActiveFocus()
        }
        onClosed: {
            window.resetMarqueeContext()
            gameList.forceActiveFocus()
        }

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
                onCurrentIndexChanged:
                    window.updateMarqueeNavigationPreview()
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
                    readonly property bool securityAllowed:
                        !controller.big_box_locked
                        || controller
                           .big_box_navigation_allowed_while_locked(
                               entryKind)
                    width: ListView.view.width
                    height: 62
                    enabled: securityAllowed
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
                    Accessible.description:
                        securityAllowed
                        ? "Available"
                        : "Unlock BigBox to use this library filter"
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
                text: "NAVIGATE     SELECT  APPLY     BACK  GAMES"
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
                text: "NAVIGATE     SELECT  APPLY     BACK  CLOSE"
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
                visible: controller.big_box_locked
                text: "LOCKED"
                color: "#f0c04a"
                font.pixelSize: 16
                font.bold: true
                font.letterSpacing: 2
                Accessible.name: "BigBox locked mode active"
            }
            Label {
                visible:
                    controller.big_box_gamepad_enabled
                    && (controller.big_box_gamepad_connected_count > 0
                        || controller.big_box_gamepad_status
                           .startsWith("Unavailable"))
                text:
                    controller.big_box_gamepad_connected_count > 0
                    ? "GAMEPAD  "
                      + controller.big_box_gamepad_connected_count
                    : "GAMEPAD UNAVAILABLE"
                color:
                    controller.big_box_gamepad_connected_count > 0
                    ? "#8bd49c" : "#ffab70"
                font.pixelSize: 14
                font.bold: true
                Accessible.name: controller.big_box_gamepad_status
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
                Keys.onPressed: function(event) {
                    if (!bigBoxAttractMode.active)
                        bigBoxAttractMode.noteActivity()
                    if (!bigBoxScreensaver.active)
                        bigBoxScreensaver.noteActivity()
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
                        window.selectedBigBoxGameStarRatingFloat =
                            controller.big_box_star_rating_at(index, gameId)
                        window.selectedBigBoxGamePlayTimeSeconds =
                            gamePlayTimeSeconds
                        window.selectedBigBoxGameCommunityRating =
                            gameCommunityStarRating
                        window.selectedBigBoxGameCommunityVotes =
                            gameCommunityStarRatingTotalVotes
                        window.selectedBigBoxGameFrontImageUrl =
                            gameFrontImageUrl
                        window.refreshSelectedBigBoxPlaylistAction()
                        if (controller
                                .big_box_auto_play_music_games_list)
                            Qt.callLater(
                                window
                                .autoPlaySelectedGameMusicFromList)
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
                    border.color:
                        gameFavorite
                        && controller
                           .big_box_show_star_next_to_favorited_games
                        ? "#f0c04a" : "#4775aa"
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
                            text:
                                  (gameFavorite
                                   && controller
                                      .big_box_show_star_next_to_favorited_games
                                   ? "★ FAVORITE    " : "")
                                  + (gameCompleted ? "✓ COMPLETED" : "")
                            color: "#f0c04a"
                            font.pixelSize: 15
                            font.bold: true
                        }
                    }
                    MouseArea {
                        anchors.fill: parent
                        onClicked: {
                            bigBoxScreensaver.noteActivity()
                            gameList.currentIndex = index
                        }
                        onDoubleClicked: window.launchGame(index, gameId)
                    }
                }
            }
        }

        RowLayout {
            Layout.fillWidth: true
            Button {
                text: "BROWSE: " + window.activeNavigationName.toUpperCase()
                enabled: !controller.loading && !controller.writing
                         && window.navigationAccessAvailable()
                onClicked: window.openNavigation()
            }
            Button {
                text: "GAME FILTERS"
                enabled: !controller.loading && !controller.writing
                         && window.securityActionAllowed("BigBoxFilter")
                onClicked: window.openAttributeFilters()
            }
            Button {
                text: "INPUT"
                enabled: !controller.loading && !controller.writing
                         && !controller.big_box_locked
                Accessible.name: "Edit BigBox input settings"
                onClicked: bigBoxInputSettings.openEditor()
            }
            Button {
                text: "DISPLAYS"
                enabled: !controller.loading && !controller.writing
                         && !controller.big_box_locked
                Accessible.name: "Edit BigBox display and marquee settings"
                onClicked: bigBoxMarqueeSettings.openEditor()
            }
            Button {
                text: "SECURITY"
                visible: !controller.big_box_locked
                enabled: visible
                         && !controller.loading && !controller.writing
                Accessible.name: "Edit BigBox PIN and locked mode permissions"
                onClicked: bigBoxSecuritySettings.openEditor()
            }
            Button {
                text: controller.big_box_locked ? "UNLOCK" : "LOCK"
                visible:
                    controller.big_box_pin_configured
                    && controller.big_box_show_game_lock_unlock
                enabled: visible
                         && !controller.loading && !controller.writing
                Accessible.name:
                    controller.big_box_locked
                    ? "Unlock BigBox" : "Lock BigBox"
                onClicked: window.requestLockUnlock()
            }
            Button {
                text: "RANDOM"
                enabled: controller.filtered_count > 0
                         && !controller.loading && !controller.writing
                onClicked: window.selectRandomGame()
            }
            Button {
                id: startAttractButton
                text: "START ATTRACT"
                enabled:
                    controller.filtered_count > 0
                    && !bigBoxAttractMode.blocked
                    && !bigBoxAttractMode.active
                Accessible.name: "Start Attract Mode"
                onClicked: bigBoxAttractMode.startManual()
            }
            Button {
                id: startScreensaverButton
                text: "START SCREENSAVER"
                enabled:
                    controller.big_box_screensaver_candidate_count > 0
                    && !bigBoxScreensaver.blocked
                    && !bigBoxScreensaver.active
                Accessible.name: "Start screensaver"
                onClicked: bigBoxScreensaver.startManual()
            }
            Button {
                id: bigBoxGameDetailsButton
                text: "DETAILS"
                enabled: window.selectedBigBoxGameId.length > 0
                         && !controller.loading && !controller.writing
                onClicked: window.openGameDetails()
            }
            Button {
                id: bigBoxFavoriteButton
                text: window.selectedBigBoxGameFavorite
                      ? "UNFAVORITE" : "FAVORITE"
                visible:
                    controller.big_box_show_game_menu_favorite
                    && window.selectedBigBoxGameId.length > 0
                enabled: visible
                         && !controller.loading
                         && !controller.writing
                         && window.securityActionAllowed(
                             "BigBoxFavoriteGames")
                Accessible.name:
                    window.selectedBigBoxGameFavorite
                    ? "Remove selected game from favorites"
                    : "Add selected game to favorites"
                onClicked: window.toggleSelectedFavorite()
            }
            Button {
                id: bigBoxStarRatingButton
                text: "RATE "
                      + window.selectedBigBoxGameStarRatingFloat
                        .toFixed(1)
                visible:
                    controller.big_box_show_game_menu_star_rating
                    && window.selectedBigBoxGameId.length > 0
                enabled: visible
                         && !controller.loading
                         && !controller.writing
                         && window.securityActionAllowed(
                             "BigBoxSetStarRating")
                Accessible.name: "Set selected game star rating"
                onClicked: window.openSelectedStarRating()
            }
            Button {
                id: bigBoxAddToPlaylistButton
                text: "ADD TO PLAYLIST"
                visible:
                    controller.big_box_show_game_menu_playlist_actions
                    && window.selectedBigBoxGameId.length > 0
                    && window.selectedBigBoxPlaylistAddTargetCount > 0
                enabled: visible
                         && !controller.loading
                         && !controller.writing
                         && window.securityActionAllowed(
                             "BigBoxPlaylistActions")
                Accessible.name: "Add selected game to a playlist"
                onClicked: window.openSelectedPlaylistPopup()
            }
            Button {
                id: bigBoxRemoveFromPlaylistButton
                text: window.selectedBigBoxPlaylistRemoveCurrent === null
                      ? "REMOVE FROM PLAYLIST"
                      : "REMOVE FROM "
                        + window
                          .selectedBigBoxPlaylistRemoveCurrent.name
                          .toUpperCase()
                visible:
                    controller.big_box_show_game_menu_playlist_actions
                    && window.selectedBigBoxGameId.length > 0
                    && window.selectedBigBoxPlaylistRemoveCurrent
                       !== null
                enabled: visible
                         && !controller.loading
                         && !controller.writing
                         && window.securityActionAllowed(
                             "BigBoxPlaylistActions")
                Accessible.name:
                    window.selectedBigBoxPlaylistRemoveCurrent === null
                    ? "Remove selected game from playlist"
                    : "Remove selected game from "
                      + window
                        .selectedBigBoxPlaylistRemoveCurrent.name
                onClicked:
                    window.removeSelectedFromCurrentPlaylist()
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
                id: bigBoxManualButton
                text: "MANUAL"
                visible:
                    controller.big_box_show_game_menu_view_manual
                    && window.selectedBigBoxGameId.length > 0
                    && controller.game_manual_url_for_game(
                        window.selectedBigBoxGameId)
                       .toString().length > 0
                enabled: visible
                         && !controller.loading
                         && !controller.writing
                Accessible.name: "Open the selected game manual"
                onClicked:
                    window.openGameManual(
                        window.selectedBigBoxGameId)
            }
            Button {
                id: bigBoxMusicButton
                text: bigBoxMusicPlayer.opened
                      && bigBoxMusicPlayer.gameId
                         === window.selectedBigBoxGameId
                      ? "MUSIC…" : "MUSIC"
                visible:
                    controller.big_box_show_game_menu_play_music
                    && window.selectedBigBoxGameId.length > 0
                    && controller.game_music_count_for_game(
                        window.selectedBigBoxGameId) > 0
                enabled: visible
                         && !controller.loading
                         && !controller.writing
                Accessible.name: "Play the selected game music"
                onClicked:
                    window.playGameMusic(
                        window.selectedBigBoxGameId,
                        window.selectedBigBoxGameTitle, true)
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
                         && window.securityActionAllowed(
                             "BigBoxFlipBox")

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
                visible: window.width >= 1500
                text: controller.launching
                      ? "LAUNCHING…"
                      : "SELECT  DETAILS     IMAGES     FLIP     NAVIGATE     PLAY GAME"
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
            if (controller.game_media_kind_at(
                    mediaGameId, index) === "video")
                bigBoxMusicPlayer.stopPlayback(true)
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

        function hasVideoMedia() {
            for (let index = 0; index < mediaCount; ++index) {
                if (controller.game_media_kind_at(
                        mediaGameId, index) === "video")
                    return true
            }
            return false
        }

        function autoPlayMusicIfAllowed() {
            if (!controller.big_box_auto_play_music_game_details
                    || mediaGameId.length === 0
                    || (hasVideoMedia()
                        && controller.details_auto_play_video
                        && !controller
                            .big_box_prioritize_music_over_video_audio)) {
                if (bigBoxMusicPlayer.opened
                        && bigBoxMusicPlayer.gameId === mediaGameId)
                    bigBoxMusicPlayer.stopPlayback(true)
                return false
            }
            return window.playGameMusic(
                        mediaGameId,
                        window.selectedBigBoxGameTitle, true)
        }

        function clickMediaThumbnailForSmoke(index) {
            if (index < 0 || index >= mediaCount)
                return false
            bigBoxMediaThumbnailList.positionViewAtIndex(
                index, ListView.Contain)
            bigBoxMediaThumbnailList.forceLayout()
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
                bigBoxGameDetails.autoPlayMusicIfAllowed()
            })
        }
        onClosed: {
            bigBoxMediaPlayer.stop()
            gameList.forceActiveFocus()
            Qt.callLater(window.autoPlaySelectedGameMusicFromList)
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
                                volume:
                                    window.runtimeMasterVolumePercent / 100
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
                                    (controller.big_box_show_game_favorite
                                     && window.selectedBigBoxGameFavorite
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
                                    + window
                                      .selectedBigBoxGameStarRatingFloat
                                      .toFixed(1)
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
                                visible:
                                    controller.big_box_show_game_star_rating
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

                            RowLayout {
                                Layout.fillWidth: true
                                Button {
                                    Layout.fillWidth: true
                                    text: "VIEW MANUAL"
                                    visible:
                                        controller
                                        .big_box_show_game_menu_view_manual
                                        && controller
                                           .game_manual_url_for_game(
                                               bigBoxGameDetails
                                               .mediaGameId)
                                           .toString().length > 0
                                    enabled: visible
                                             && !controller.loading
                                             && !controller.writing
                                    onClicked:
                                        window.openGameManual(
                                            bigBoxGameDetails
                                            .mediaGameId)
                                }
                                Button {
                                    Layout.fillWidth: true
                                    text:
                                        bigBoxMusicPlayer.opened
                                        && bigBoxMusicPlayer.gameId
                                           === bigBoxGameDetails
                                              .mediaGameId
                                        ? "MUSIC CONTROLS"
                                        : "PLAY MUSIC"
                                    visible:
                                        controller
                                        .big_box_show_game_menu_play_music
                                        && controller
                                           .game_music_count_for_game(
                                               bigBoxGameDetails
                                               .mediaGameId) > 0
                                    enabled: visible
                                             && !controller.loading
                                             && !controller.writing
                                    onClicked:
                                        window.playGameMusic(
                                            bigBoxGameDetails
                                            .mediaGameId,
                                            window
                                            .selectedBigBoxGameTitle,
                                            true)
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
                            "NAVIGATE  MEDIA     SELECT  PLAY / PAUSE"
                            + "     PLAY GAME     BACK"
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
            if (!window.guardSecurityAction(
                    "BigBoxSwitchImageType")
                    || imageCount === 0)
                return false
            return selectImage(
                selectedImageIndex <= 0
                ? imageCount - 1 : selectedImageIndex - 1)
        }

        function selectNextImage() {
            if (!window.guardSecurityAction(
                    "BigBoxSwitchImageType")
                    || imageCount === 0)
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
                        text: "SELECT / PAGE  SWITCH    DRAG / NAVIGATE  PAN"
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

    BigBoxScreensaver {
        id: bigBoxScreensaver
        anchors.fill: parent
        z: 9500
        controller: controller
        mutedForSmoke: window.screensaverAnySmokeTest
        runtimeMasterVolumeScale:
            window.runtimeMasterVolumePercent / 100
        blocked:
            controller.loading
            || controller.writing
            || controller.launching
            || controller.launch_session_active
            || controller.startup_screen_active
            || controller.shutdown_screen_active
            || controller.pause_screen_active
            || window.startupPresentationPending
            || bigBoxGameDetails.opened
            || bigBoxImageViewer.opened
            || bigBoxModelViewer.opened
            || attributeFilterDrawer.opened
            || navigationDrawer.opened
            || launchWithDialog.opened
            || bigBoxUnlockPopup.opened
            || bigBoxPlaylistPopup.opened
            || bigBoxStarRatingPopup.opened
            || bigBoxSecuritySettings.opened
            || bigBoxAttractMode.active
        activationCallback: function() {
            bigBoxAttractMode.stopMode("screensaver")
            bigBoxMediaPlayer.stop()
            bigBoxMusicPlayer.stopPlayback(true)
        }
        exploreGameCallback: window.exploreScreensaverGame
        focusReturnCallback: function() {
            gameList.forceActiveFocus()
        }
    }

    BigBoxAttractMode {
        id: bigBoxAttractMode
        anchors.fill: parent
        z: 9000
        controller: controller
        mutedForSmoke: window.attractModeAnySmokeTest
        runtimeMasterVolumeScale:
            window.runtimeMasterVolumePercent / 100
        blocked:
            controller.loading
            || controller.writing
            || controller.launching
            || controller.launch_session_active
            || controller.startup_screen_active
            || controller.shutdown_screen_active
            || controller.pause_screen_active
            || window.startupPresentationPending
            || bigBoxGameDetails.opened
            || bigBoxImageViewer.opened
            || bigBoxModelViewer.opened
            || attributeFilterDrawer.opened
            || navigationDrawer.opened
            || launchWithDialog.opened
            || bigBoxUnlockPopup.opened
            || bigBoxPlaylistPopup.opened
            || bigBoxStarRatingPopup.opened
            || bigBoxSecuritySettings.opened
            || bigBoxScreensaver.active
        advanceWheelCallback: window.advanceAttractWheel
        switchFilterCallback: window.switchAttractFilter
        focusReturnCallback: function() {
            gameList.forceActiveFocus()
        }
        onActiveChanged: {
            if (active)
                window.attractNavigationCursor =
                    window.currentBigBoxNavigationRow()
        }
    }

    BigBoxStartupPresentation {
        id: startupPresentationOverlay
        anchors.fill: parent
        z: 10000
        controller: controller
        coverVisible:
            window.startupPresentationPending
            && window.startupPresentationDecisionMade
            && !active
        showSplashBranding:
            controller.big_box_show_startup_splash_screen
        outputVolume:
            Math.max(0, Math.min(
                1,
                controller.big_box_startup_video_volume_percent
                * window.runtimeMasterVolumePercent
                / 10000))
        soundOutputVolume:
            Math.max(0, Math.min(
                1,
                controller.big_box_startup_sound_volume_percent
                * controller.big_box_master_volume_percent
                * window.runtimeMasterVolumePercent
                / 1000000))
        mutedForSmoke:
            window.startupVideoSmokeTest
            || window.startupSplashAnySmokeTest
        onCoverVisibleChanged: {
            if (coverVisible && showSplashBranding)
                window.startupSplashWasVisible = true
        }
        onSoundStartedChanged: {
            if (soundStarted)
                window.startupSoundPlaybackSeen = true
        }
        onVideoCompleted:
            window.finishApplicationStartupVideo()
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

    BackgroundMusicPlayer {
        id: backgroundMusicPlayer
        parent: Overlay.overlay
        x: Math.round((window.width - width) / 2)
        y: 76
        controller: controller
        contextKind: window.backgroundMusicContextKind
        contextName: window.backgroundMusicContextName
        backgroundMusicEnabled:
            controller.big_box_background_music_enabled
        shuffleEnabled: controller.big_box_shuffle_background_music
        onScreenDisplayEnabled:
            controller.big_box_music_on_screen_display_enabled
        outputVolume:
            Math.max(0, Math.min(
                1,
                controller.big_box_background_music_volume_percent
                * (bigBoxAttractMode.active
                   ? controller.big_box_attract_mode_master_volume_percent
                   : 100)
                * window.runtimeMasterVolumePercent
                / 1000000))
        blocked:
            controller.loading
            || controller.writing
            || controller.launching
            || controller.launch_session_active
            || window.startupPresentationPending
            || bigBoxScreensaver.active
            || bigBoxMusicPlayer.opened
            || (!controller
                .big_box_play_video_audio_with_background_music
                && bigBoxMediaPlayer.playbackState
                   === MediaPlayer.PlayingState)
        mutedForSmoke: window.backgroundMusicSmokeTest
        pinnedForSmoke: window.backgroundMusicSmokeTest
    }

    GameMusicPlayer {
        id: bigBoxMusicPlayer
        parent: Overlay.overlay
        x: Math.round((window.width - width) / 2)
        y: Math.max(18, window.height - height - 24)
        controller: controller
        shuffleEnabled:
            controller.big_box_shuffle_soundtrack_music
        repeatEnabled: controller.big_box_repeat_game_music
        outputVolume:
            Math.max(0, Math.min(
                1,
                controller.big_box_music_volume_percent
                * (bigBoxAttractMode.active
                   ? controller.big_box_attract_mode_master_volume_percent
                   : 100)
                * window.runtimeMasterVolumePercent
                / 1000000))
        mutedForSmoke: window.supplementalMediaSmokeTest
                       || window.backgroundMusicSmokeTest
    }

    Timer {
        interval: 20
        repeat: true
        running: window.inputSmokeTest
                 && !window.inputSmokeFinished
        onTriggered: {
            if (controller.loading || controller.writing
                    || window.startupPresentationPending
                    || controller.library_path.length === 0)
                return
            if (window.inputSmokePhase === 0) {
                const row =
                    controller.row_for_game_id("fixture-adventure")
                if (row < 0
                        || controller.game_image_count_for_game(
                            "fixture-adventure") === 0)
                    return
                gameList.currentIndex = row
                gameList.positionViewAtIndex(row, ListView.Center)
                if (!controller.submit_big_box_controller_event(
                        7, "Button1", true)
                        || !controller.submit_big_box_controller_event(
                            7, "Button1", false)) {
                    console.error(
                        "BIGBOX_INPUT_SMOKE_SELECT_SUBMIT_FAILED")
                    Qt.exit(651)
                    return
                }
                window.inputSmokeSelectCount += 1
                window.inputSmokePhase = 1
            } else if (window.inputSmokePhase === 1) {
                if (!bigBoxGameDetails.opened)
                    return
                if (!controller.submit_big_box_controller_event(
                        7, "Button2", true)
                        || !controller.submit_big_box_controller_event(
                            7, "Button2", false)) {
                    console.error(
                        "BIGBOX_INPUT_SMOKE_BACK_SUBMIT_FAILED")
                    Qt.exit(652)
                    return
                }
                window.inputSmokeBackCount += 1
                window.inputSmokePhase = 2
            } else if (window.inputSmokePhase === 2) {
                if (bigBoxGameDetails.opened)
                    return
                if (!controller.submit_big_box_controller_event(
                        7, "DPadRight", true)
                        || !controller.submit_big_box_controller_event(
                            7, "DPadRight", false)) {
                    console.error(
                        "BIGBOX_INPUT_SMOKE_RIGHT_SUBMIT_FAILED")
                    Qt.exit(653)
                    return
                }
                window.inputSmokeNavigationCount += 1
                window.inputSmokePhase = 3
            } else if (window.inputSmokePhase === 3) {
                if (window.selectedBigBoxGameId
                        !== "fixture-puzzle")
                    return
                if (!controller.submit_big_box_controller_event(
                        7, "DPadLeft", true)
                        || !controller.submit_big_box_controller_event(
                            7, "DPadLeft", false)) {
                    console.error(
                        "BIGBOX_INPUT_SMOKE_LEFT_SUBMIT_FAILED")
                    Qt.exit(654)
                    return
                }
                window.inputSmokeNavigationCount += 1
                window.inputSmokePhase = 4
            } else if (window.inputSmokePhase === 4) {
                if (window.selectedBigBoxGameId
                        !== "fixture-adventure")
                    return
                if (!controller.submit_big_box_controller_event(
                        7, "Button4", true)
                        || !controller.submit_big_box_controller_event(
                            7, "Button4", false)) {
                    console.error(
                        "BIGBOX_INPUT_SMOKE_IMAGES_SUBMIT_FAILED")
                    Qt.exit(655)
                    return
                }
                window.inputSmokePhase = 5
            } else if (window.inputSmokePhase === 5) {
                if (!bigBoxImageViewer.opened)
                    return
                window.inputSmokeImageOpenCount += 1
                if (!controller.submit_big_box_controller_event(
                        7, "Button2", true)
                        || !controller.submit_big_box_controller_event(
                            7, "Button2", false)) {
                    console.error(
                        "BIGBOX_INPUT_SMOKE_IMAGE_BACK_SUBMIT_FAILED")
                    Qt.exit(656)
                    return
                }
                window.inputSmokePhase = 6
            } else if (window.inputSmokePhase === 6) {
                if (bigBoxImageViewer.opened)
                    return
                window.inputSmokeImageBackCount += 1
                if (!controller.report_big_box_input_smoke_success(
                        window.inputSmokeSelectCount,
                        window.inputSmokeBackCount,
                        window.inputSmokeNavigationCount,
                        window.inputSmokeImageOpenCount,
                        window.inputSmokeImageBackCount,
                        window.selectedBigBoxGameId)) {
                    console.error(
                        "BIGBOX_INPUT_SMOKE_CONTROLLER_REJECTED"
                        + " phase=" + window.inputSmokePhase
                        + " selected="
                        + window.selectedBigBoxGameId
                        + " status="
                        + controller.big_box_gamepad_status)
                    Qt.exit(657)
                    return
                }
                window.inputSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 6000
        repeat: false
        running: window.inputSmokeTest
                 && !window.inputSmokeFinished
        onTriggered: {
            controller.report_big_box_input_smoke_success(
                window.inputSmokeSelectCount,
                window.inputSmokeBackCount,
                window.inputSmokeNavigationCount,
                window.inputSmokeImageOpenCount,
                window.inputSmokeImageBackCount,
                window.selectedBigBoxGameId)
            console.error(
                "BIGBOX_INPUT_SMOKE_TIMEOUT phase="
                + window.inputSmokePhase
                + " selected=" + window.selectedBigBoxGameId
                + " details=" + bigBoxGameDetails.opened
                + " images=" + bigBoxImageViewer.opened
                + " status=" + controller.big_box_gamepad_status)
            Qt.exit(658)
        }
    }

    Timer {
        interval: 20
        repeat: true
        running: window.attractModeAnySmokeTest
                 && !window.attractModeSmokeFinished
        onTriggered: {
            if (controller.loading || controller.writing
                    || controller.library_path.length === 0)
                return

            if (window.attractModeDisabledSmokeTest) {
                if (window.attractModeSmokePhase === 0) {
                    window.attractModeDisabledWaitStartedAt = Date.now()
                    window.attractModeSmokePhase = 1
                } else if (window.attractModeSmokePhase === 1) {
                    if (Date.now()
                            - window.attractModeDisabledWaitStartedAt
                            < 1300)
                        return
                    if (bigBoxAttractMode.active
                            || bigBoxAttractMode.lastStartSource
                               === "automatic") {
                        console.error(
                            "BIGBOX_ATTRACT_MODE_DISABLED_AUTO_STARTED")
                        Qt.exit(639)
                        return
                    }
                    startAttractButton.clicked()
                    window.attractModeSmokePhase = 2
                } else if (window.attractModeSmokePhase === 2) {
                    if (!bigBoxAttractMode.active
                            || bigBoxAttractMode.lastStartSource
                               !== "manual"
                            || bigBoxAttractMode.totalWheelSteps < 1
                            || !bigBoxAttractMode.moveSoundReady)
                        return
                    bigBoxAttractMode.clickReturnForSmoke()
                    window.attractModeSmokePhase = 3
                } else if (window.attractModeSmokePhase === 3) {
                    if (bigBoxAttractMode.active)
                        return
                    window.finishAttractModeSmoke(false)
                }
                return
            }

            if (window.attractModeSmokePhase === 0) {
                if (!controller.big_box_attract_mode_enabled
                        || bigBoxAttractMode.idleCountdownStartedAt <= 0)
                    return
                window.attractModeSmokePhase = 1
            } else if (window.attractModeSmokePhase === 1) {
                if (!bigBoxAttractMode.active
                        || bigBoxAttractMode.lastStartSource
                           !== "automatic")
                    return
                window.attractModeSmokePhase = 2
            } else if (window.attractModeSmokePhase === 2) {
                if (!bigBoxAttractMode.active
                        || bigBoxAttractMode.movementCycles < 1
                        || bigBoxAttractMode.filterSwitches < 1
                        || !bigBoxAttractMode.moveSoundReady)
                    return
                window.attractModeAutoWheelSteps =
                    bigBoxAttractMode.totalWheelSteps
                window.attractModeAutoMovementCycles =
                    bigBoxAttractMode.movementCycles
                window.attractModeAutoFilterSwitches =
                    bigBoxAttractMode.filterSwitches
                window.attractModeAutoDelayElapsedMs =
                    bigBoxAttractMode.lastAutomaticDelayElapsedMs
                window.captureAttractModeSmokeAndExit()
            } else if (window.attractModeSmokePhase === 3) {
                if (bigBoxAttractMode.active)
                    return
                startAttractButton.clicked()
                window.attractModeSmokePhase = 4
            } else if (window.attractModeSmokePhase === 4) {
                if (!bigBoxAttractMode.active
                        || bigBoxAttractMode.lastStartSource !== "manual"
                        || bigBoxAttractMode.totalWheelSteps
                           <= window.attractModeAutoWheelSteps)
                    return
                bigBoxAttractMode.clickReturnForSmoke()
                window.attractModeSmokePhase = 5
            } else if (window.attractModeSmokePhase === 5) {
                if (bigBoxAttractMode.active)
                    return
                window.finishAttractModeSmoke(true)
            }
        }
    }

    Timer {
        interval: 15000
        running: window.attractModeAnySmokeTest
                 && !window.attractModeSmokeFinished
        onTriggered: {
            console.error(
                "BIGBOX_ATTRACT_MODE_TIMEOUT phase="
                + window.attractModeSmokePhase
                + " active=" + bigBoxAttractMode.active
                + " source=" + bigBoxAttractMode.lastStartSource
                + " stop=" + bigBoxAttractMode.lastStopReason
                + " wheelSteps=" + bigBoxAttractMode.totalWheelSteps
                + " movements=" + bigBoxAttractMode.movementCycles
                + " filters=" + bigBoxAttractMode.filterSwitches
                + " soundStatus=" + bigBoxAttractMode.moveSoundStatus
                + " controller=" + controller.status_message)
            Qt.exit(640 + window.attractModeSmokePhase)
        }
    }

    Timer {
        interval: 20
        repeat: true
        running: window.screensaverAnySmokeTest
                 && !window.screensaverSmokeFinished
        onTriggered: {
            if (controller.loading || controller.writing
                    || controller.library_path.length === 0)
                return
            if (bigBoxScreensaver.videoReady)
                window.screensaverVideoReadySeen = true

            if (window.screensaverDisabledSmokeTest) {
                if (window.screensaverSmokePhase === 0) {
                    window.screensaverDisabledWaitStartedAt = Date.now()
                    window.screensaverSmokePhase = 1
                } else if (window.screensaverSmokePhase === 1) {
                    if (Date.now()
                            - window.screensaverDisabledWaitStartedAt
                            < 1300)
                        return
                    if (bigBoxScreensaver.active
                            || bigBoxScreensaver.lastStartSource
                               === "automatic") {
                        console.error(
                            "BIGBOX_SCREENSAVER_DISABLED_AUTO_STARTED")
                        Qt.exit(646)
                        return
                    }
                    startScreensaverButton.clicked()
                    window.screensaverSmokePhase = 2
                } else if (window.screensaverSmokePhase === 2) {
                    if (!bigBoxScreensaver.active
                            || bigBoxScreensaver.lastStartSource
                               !== "manual"
                            || !bigBoxScreensaver.videoPlaybackSeen
                            || !window.screensaverVideoReadySeen)
                        return
                    bigBoxScreensaver.setSmokeView(1)
                    bigBoxScreensaver.setSmokeView(2)
                    bigBoxScreensaver.setSmokeView(3)
                    bigBoxScreensaver.setSmokeView(4)
                    bigBoxScreensaver.clickReturnForSmoke()
                    window.screensaverSmokePhase = 3
                } else if (window.screensaverSmokePhase === 3) {
                    if (bigBoxScreensaver.active)
                        return
                    window.finishScreensaverSmoke(false)
                }
                return
            }

            if (window.screensaverSmokePhase === 0) {
                if (!controller.big_box_screensaver_enabled
                        || bigBoxScreensaver.idleCountdownStartedAt <= 0)
                    return
                window.screensaverSmokePhase = 1
            } else if (window.screensaverSmokePhase === 1) {
                if (!bigBoxScreensaver.active
                        || bigBoxScreensaver.lastStartSource
                           !== "automatic")
                    return
                window.screensaverSmokePhase = 2
            } else if (window.screensaverSmokePhase === 2) {
                if (!bigBoxScreensaver.active
                        || bigBoxScreensaver.swapCount < 1
                        || !bigBoxScreensaver.videoPlaybackSeen
                        || !window.screensaverVideoReadySeen)
                    return
                bigBoxScreensaver.setSmokeView(1)
                window.screensaverPhaseStartedAt = Date.now()
                window.screensaverSmokePhase = 3
            } else if (window.screensaverSmokePhase >= 3
                       && window.screensaverSmokePhase <= 6) {
                if (Date.now() - window.screensaverPhaseStartedAt < 1400)
                    return
                window.captureScreensaverView(
                    window.screensaverSmokePhase - 2)
            } else if (window.screensaverSmokePhase === 7) {
                if (bigBoxScreensaver.active)
                    return
                startScreensaverButton.clicked()
                window.screensaverSmokePhase = 8
            } else if (window.screensaverSmokePhase === 8) {
                if (!bigBoxScreensaver.active
                        || bigBoxScreensaver.lastStartSource !== "manual")
                    return
                bigBoxScreensaver.clickExploreForSmoke()
                window.screensaverSmokePhase = 9
            } else if (window.screensaverSmokePhase === 9) {
                if (bigBoxScreensaver.active
                        || window.selectedBigBoxGameId
                           !== "fixture-adventure")
                    return
                window.finishScreensaverSmoke(true)
            }
        }
    }

    Timer {
        interval: 20000
        running: window.screensaverAnySmokeTest
                 && !window.screensaverSmokeFinished
        onTriggered: {
            console.error(
                "BIGBOX_SCREENSAVER_TIMEOUT phase="
                + window.screensaverSmokePhase
                + " active=" + bigBoxScreensaver.active
                + " source=" + bigBoxScreensaver.lastStartSource
                + " stop=" + bigBoxScreensaver.lastStopReason
                + " swaps=" + bigBoxScreensaver.swapCount
                + " selections=" + bigBoxScreensaver.selectionCount
                + " views=" + bigBoxScreensaver.presentedViewsMask
                + " videoState="
                + bigBoxScreensaver.videoPlaybackState
                + " videoStatus="
                + bigBoxScreensaver.videoMediaStatus
                + " controller=" + controller.status_message)
            Qt.exit(645 + window.screensaverSmokePhase)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.supplementalMediaSmokeTest
                 && !window.supplementalMediaSmokeFinished
        onTriggered: {
            if (controller.loading || controller.writing
                    || controller.library_path.length === 0)
                return
            if (window.supplementalMediaSmokePhase === 0) {
                if (controller.indexed_manual_count !== 1
                        || controller.indexed_music_track_count !== 2
                        || controller.big_box_auto_play_music_games_list
                        || controller.big_box_auto_play_music_game_details
                        || controller
                           .big_box_prioritize_music_over_video_audio
                        || controller.big_box_repeat_game_music
                        || controller.big_box_shuffle_soundtrack_music
                        || !controller
                            .big_box_show_game_menu_play_music
                        || !controller
                            .big_box_show_game_menu_view_manual
                        || controller.big_box_music_volume_percent
                           !== 75)
                    return
                const row = controller.row_for_game_id(
                                "fixture-adventure")
                if (row < 0)
                    return
                gameList.currentIndex = row
                gameList.positionViewAtIndex(
                            row, ListView.Center)
                window.supplementalMediaSmokePhase = 1
                return
            }
            if (window.supplementalMediaSmokePhase === 1) {
                if (window.selectedBigBoxGameId
                        !== "fixture-adventure"
                        || !bigBoxManualButton.visible
                        || !bigBoxManualButton.enabled
                        || !bigBoxMusicButton.visible
                        || !bigBoxMusicButton.enabled)
                    return
                bigBoxManualButton.clicked()
                bigBoxMusicButton.clicked()
                window.supplementalMediaSmokePhase = 2
            } else if (window.supplementalMediaSmokePhase === 2) {
                if (window.supplementalMediaManualUrl.length === 0
                        || !bigBoxMusicPlayer.opened
                        || bigBoxMusicPlayer.gameId
                           !== "fixture-adventure"
                        || bigBoxMusicPlayer.trackCount !== 2
                        || bigBoxMusicPlayer.currentTrackIndex !== 0
                        || bigBoxMusicPlayer.trackName
                           !== "Fixture Adventure-01.mp3"
                        || bigBoxMusicPlayer.duration <= 0
                        || bigBoxMusicPlayer.mediaError
                           !== MediaPlayer.NoError
                        || bigBoxMusicPlayer.playbackState
                           !== MediaPlayer.PlayingState)
                    return
                window.supplementalMediaFirstMusicUrl =
                    bigBoxMusicPlayer.trackSource.toString()
                if (!bigBoxMusicPlayer
                        .clickPlayPauseForSmoke()) {
                    console.error(
                        "BIGBOX_SUPPLEMENTAL_MEDIA_PAUSE_MISSING")
                    Qt.exit(576)
                    return
                }
                window.supplementalMediaSmokePhase = 3
            } else if (window.supplementalMediaSmokePhase === 3) {
                if (bigBoxMusicPlayer.playbackState
                        !== MediaPlayer.PausedState)
                    return
                if (!bigBoxMusicPlayer.clickNextForSmoke()) {
                    console.error(
                        "BIGBOX_SUPPLEMENTAL_MEDIA_NEXT_MISSING")
                    Qt.exit(577)
                    return
                }
                window.supplementalMediaSmokePhase = 4
            } else if (window.supplementalMediaSmokePhase === 4) {
                if (bigBoxMusicPlayer.currentTrackIndex !== 1
                        || bigBoxMusicPlayer.trackName
                           !== "Fixture Adventure-02.mp3")
                    return
                window.finishSupplementalMediaSmoke()
            }
        }
    }

    Timer {
        interval: 20000
        running: window.supplementalMediaSmokeTest
                 && !window.supplementalMediaSmokeFinished
        onTriggered: {
            console.error(
                "BIGBOX_SUPPLEMENTAL_MEDIA_TIMEOUT phase="
                + window.supplementalMediaSmokePhase
                + " id=" + window.selectedBigBoxGameId
                + " track=" + bigBoxMusicPlayer.currentTrackIndex
                + " state=" + bigBoxMusicPlayer.playbackState
                + " status=" + bigBoxMusicPlayer.mediaStatus
                + " error=" + bigBoxMusicPlayer.mediaError
                + " duration=" + bigBoxMusicPlayer.duration
                + " controller=" + controller.status_message)
            Qt.exit(window.supplementalMediaScreenshotRequested
                    ? 599
                    : 580 + window.supplementalMediaSmokePhase)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.backgroundMusicSmokeTest
                 && !window.backgroundMusicSmokeFinished
        onTriggered: {
            if (controller.loading || controller.writing
                    || controller.library_path.length === 0)
                return
            if (window.backgroundMusicSmokePhase === 0) {
                if (controller.indexed_background_music_track_count
                        !== 8
                        || !controller.big_box_background_music_enabled
                        || controller
                           .big_box_background_music_volume_percent
                           !== 63
                        || !controller
                            .big_box_music_on_screen_display_enabled
                        || controller.big_box_shuffle_background_music
                        || !controller
                            .big_box_context_specific_background_music
                        || backgroundMusicPlayer.currentCollectionKey
                           !== "default"
                        || backgroundMusicPlayer.currentTrackIndex !== 0
                        || backgroundMusicPlayer.trackName
                           !== "Default-01.mp3"
                        || backgroundMusicPlayer.duration <= 0
                        || backgroundMusicPlayer.mediaError
                           !== MediaPlayer.NoError
                        || backgroundMusicPlayer.playbackState
                           !== MediaPlayer.PlayingState)
                    return
                window.backgroundMusicDefaultFirstUrl =
                    backgroundMusicPlayer.trackSource.toString()
                if (!backgroundMusicPlayer
                        .clickPlayPauseForSmoke()) {
                    console.error(
                        "BIGBOX_BACKGROUND_MUSIC_PAUSE_MISSING")
                    Qt.exit(616)
                    return
                }
                window.backgroundMusicSmokePhase = 1
            } else if (window.backgroundMusicSmokePhase === 1) {
                if (backgroundMusicPlayer.playbackState
                        !== MediaPlayer.PausedState)
                    return
                if (!backgroundMusicPlayer.clickNextForSmoke()) {
                    console.error(
                        "BIGBOX_BACKGROUND_MUSIC_NEXT_MISSING")
                    Qt.exit(617)
                    return
                }
                window.backgroundMusicSmokePhase = 2
            } else if (window.backgroundMusicSmokePhase === 2) {
                if (backgroundMusicPlayer.currentTrackIndex !== 1
                        || backgroundMusicPlayer.trackName
                           !== "Default-02.mp3"
                        || backgroundMusicPlayer.playbackState
                           !== MediaPlayer.PlayingState)
                    return
                if (!backgroundMusicPlayer
                        .clickPreviousForSmoke()) {
                    console.error(
                        "BIGBOX_BACKGROUND_MUSIC_PREVIOUS_MISSING")
                    Qt.exit(618)
                    return
                }
                window.backgroundMusicSmokePhase = 3
            } else if (window.backgroundMusicSmokePhase === 3) {
                if (backgroundMusicPlayer.currentTrackIndex !== 0
                        || backgroundMusicPlayer.trackName
                           !== "Default-01.mp3"
                        || backgroundMusicPlayer.playbackState
                           !== MediaPlayer.PlayingState)
                    return
                const index = window.bigBoxNavigationIndex(
                                  "platform", "Fixture Console")
                if (index < 0) {
                    console.error(
                        "BIGBOX_BACKGROUND_MUSIC_PLATFORM_MISSING")
                    Qt.exit(619)
                    return
                }
                window.activateNavigationRow(index + 1)
                window.backgroundMusicSmokePhase = 4
            } else if (window.backgroundMusicSmokePhase === 4) {
                if (controller.navigation_filter_kind !== "platform"
                        || window.activeNavigationName
                           !== "Fixture Console"
                        || backgroundMusicPlayer.trackName
                           !== "Platform-01.mp3"
                        || backgroundMusicPlayer.playbackState
                           !== MediaPlayer.PlayingState)
                    return
                window.backgroundMusicPlatformFirstUrl =
                    backgroundMusicPlayer.trackSource.toString()
                const index = window.bigBoxNavigationIndex(
                                  "playlist", "fixture-playlist")
                if (index < 0) {
                    console.error(
                        "BIGBOX_BACKGROUND_MUSIC_PLAYLIST_MISSING")
                    Qt.exit(620)
                    return
                }
                window.activateNavigationRow(index + 1)
                window.backgroundMusicSmokePhase = 5
            } else if (window.backgroundMusicSmokePhase === 5) {
                if (controller.navigation_filter_kind !== "playlist"
                        || window.activeNavigationName
                           !== "Fixture Favorites"
                        || backgroundMusicPlayer.trackName
                           !== "Playlist-01.mp3"
                        || backgroundMusicPlayer.playbackState
                           !== MediaPlayer.PlayingState)
                    return
                window.backgroundMusicPlaylistFirstUrl =
                    backgroundMusicPlayer.trackSource.toString()
                const index = window.bigBoxNavigationIndex(
                                  "category", "Fixture Category")
                if (index < 0) {
                    console.error(
                        "BIGBOX_BACKGROUND_MUSIC_CATEGORY_MISSING")
                    Qt.exit(621)
                    return
                }
                window.activateNavigationRow(index + 1)
                window.backgroundMusicSmokePhase = 6
            } else if (window.backgroundMusicSmokePhase === 6) {
                if (controller.navigation_filter_kind !== "category"
                        || window.activeNavigationName
                           !== "Fixture Category"
                        || backgroundMusicPlayer.trackName
                           !== "Category-01.mp3"
                        || backgroundMusicPlayer.playbackState
                           !== MediaPlayer.PlayingState)
                    return
                window.backgroundMusicCategoryFirstUrl =
                    backgroundMusicPlayer.trackSource.toString()
                const index = window.bigBoxNavigationIndex(
                                  "platform", "Fixture Console")
                if (index < 0) {
                    console.error(
                        "BIGBOX_BACKGROUND_MUSIC_RETURN_PLATFORM_MISSING")
                    Qt.exit(626)
                    return
                }
                window.activateNavigationRow(index + 1)
                window.backgroundMusicSmokePhase = 7
            } else if (window.backgroundMusicSmokePhase === 7) {
                const row = controller.row_for_game_id(
                                "fixture-adventure")
                if (controller.navigation_filter_kind !== "platform"
                        || backgroundMusicPlayer.trackName
                           !== "Platform-01.mp3"
                        || backgroundMusicPlayer.playbackState
                           !== MediaPlayer.PlayingState
                        || row < 0)
                    return
                gameList.currentIndex = row
                if (window.selectedBigBoxGameId
                        !== "fixture-adventure")
                    return
                if (!window.openGameDetails()) {
                    console.error(
                        "BIGBOX_BACKGROUND_MUSIC_DETAILS_MISSING")
                    Qt.exit(622)
                    return
                }
                window.backgroundMusicSmokePhase = 8
            } else if (window.backgroundMusicSmokePhase === 8) {
                const videoAudioOverlap = controller
                    .big_box_play_video_audio_with_background_music
                if (!bigBoxGameDetails.opened
                        || bigBoxMediaPlayer.playbackState
                           !== MediaPlayer.PlayingState
                        || (videoAudioOverlap
                            && (backgroundMusicPlayer.playbackState
                                !== MediaPlayer.PlayingState
                                || backgroundMusicPlayer.pausedForBlock))
                        || (!videoAudioOverlap
                            && (backgroundMusicPlayer.playbackState
                                !== MediaPlayer.PausedState
                                || !backgroundMusicPlayer.pausedForBlock)))
                    return
                bigBoxGameDetails.close()
                window.backgroundMusicSmokePhase = 9
            } else if (window.backgroundMusicSmokePhase === 9) {
                if (bigBoxGameDetails.opened
                        || backgroundMusicPlayer.playbackState
                           !== MediaPlayer.PlayingState)
                    return
                if (!window.playGameMusic(
                        "fixture-adventure",
                        "Fixture Adventure", true)) {
                    console.error(
                        "BIGBOX_BACKGROUND_MUSIC_GAME_TRACK_MISSING")
                    Qt.exit(623)
                    return
                }
                window.backgroundMusicSmokePhase = 10
            } else if (window.backgroundMusicSmokePhase === 10) {
                if (!bigBoxMusicPlayer.opened
                        || bigBoxMusicPlayer.playbackState
                           !== MediaPlayer.PlayingState
                        || backgroundMusicPlayer.playbackState
                           !== MediaPlayer.PausedState)
                    return
                bigBoxMusicPlayer.stopPlayback(true)
                window.backgroundMusicSmokePhase = 11
            } else if (window.backgroundMusicSmokePhase === 11) {
                if (bigBoxMusicPlayer.opened
                        || backgroundMusicPlayer.playbackState
                           !== MediaPlayer.PlayingState)
                    return
                if (!backgroundMusicControlsButton.visible
                        || !backgroundMusicPlayer
                            .clickStopForSmoke()) {
                    console.error(
                        "BIGBOX_BACKGROUND_MUSIC_STOP_MISSING")
                    Qt.exit(624)
                    return
                }
                window.backgroundMusicSmokePhase = 12
            } else if (window.backgroundMusicSmokePhase === 12) {
                if (!backgroundMusicPlayer.stoppedByUser
                        || backgroundMusicPlayer.playbackState
                           !== MediaPlayer.StoppedState)
                    return
                backgroundMusicControlsButton.clicked()
                window.backgroundMusicSmokePhase = 13
            } else if (window.backgroundMusicSmokePhase === 13) {
                if (backgroundMusicPlayer.stoppedByUser
                        || backgroundMusicPlayer.playbackState
                           !== MediaPlayer.PlayingState)
                    return
                const index = window.bigBoxNavigationIndex(
                                  "category", "Fixture Category")
                if (index < 0) {
                    console.error(
                        "BIGBOX_BACKGROUND_MUSIC_FINAL_CATEGORY_MISSING")
                    Qt.exit(625)
                    return
                }
                window.activateNavigationRow(index + 1)
                window.backgroundMusicSmokePhase = 14
            } else if (window.backgroundMusicSmokePhase === 14) {
                if (backgroundMusicPlayer.trackName
                        !== "Category-01.mp3"
                        || backgroundMusicPlayer.playbackState
                           !== MediaPlayer.PlayingState)
                    return
                window.finishBackgroundMusicSmoke()
            }
        }
    }

    Timer {
        interval: 35000
        running: window.backgroundMusicSmokeTest
                 && !window.backgroundMusicSmokeFinished
        onTriggered: {
            console.error(
                "BIGBOX_BACKGROUND_MUSIC_TIMEOUT phase="
                + window.backgroundMusicSmokePhase
                + " collection="
                + backgroundMusicPlayer.currentCollectionKey
                + " track=" + backgroundMusicPlayer.trackName
                + " index="
                + backgroundMusicPlayer.currentTrackIndex
                + " state="
                + backgroundMusicPlayer.playbackState
                + " status="
                + backgroundMusicPlayer.mediaStatus
                + " error=" + backgroundMusicPlayer.mediaError
                + " duration=" + backgroundMusicPlayer.duration
                + " blocked=" + backgroundMusicPlayer.blocked
                + " controller=" + controller.status_message)
            Qt.exit(window.backgroundMusicScreenshotRequested
                    ? 649
                    : 630 + window.backgroundMusicSmokePhase)
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
        id: backgroundMusicControlsButton
        anchors.top: parent.top
        anchors.right: parent.right
        anchors.topMargin: 22
        anchors.rightMargin:
            controller.pause_screen_available
            && !controller.pause_screen_active
            ? 240 : 22
        z: 8999
        visible: controller.big_box_background_music_enabled
                 && controller
                    .indexed_background_music_track_count > 0
        text: "♫  BACKGROUND MUSIC"
        Accessible.name: "Show background music controls"
        onClicked: {
            if (backgroundMusicPlayer.stoppedByUser
                    || backgroundMusicPlayer.playbackState
                       === MediaPlayer.StoppedState)
                backgroundMusicPlayer.togglePlayback()
            backgroundMusicPlayer.showOnScreenDisplay()
        }
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
                 && !bigBoxInputSettings.opened
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
                    window.launchAdditionalApplication(
                                row, gameId, applicationId)
                }
            }
        }
    }

    BigBoxInputSettings {
        id: bigBoxInputSettings
        controller: controller
    }

    BigBoxMarqueeSettings {
        id: bigBoxMarqueeSettings
        controller: controller
    }

    BigBoxSecuritySettings {
        id: bigBoxSecuritySettings
        controller: controller
    }

    BigBoxPinPopup {
        id: bigBoxUnlockPopup

        onSubmitted: function(pin) {
            if (controller.unlock_big_box(pin)) {
                gameList.forceActiveFocus()
                return
            }
            Qt.callLater(function() {
                bigBoxUnlockPopup.openForPrompt(
                    "Enter your PIN",
                    "Incorrect PIN. Try again.")
            })
        }

        onCancelled: gameList.forceActiveFocus()
    }

    BigBoxStarRatingPopup {
        id: bigBoxStarRatingPopup
        busy: controller.writing

        onSubmitted: function(row, gameId, rating) {
            controller.update_big_box_game_state(
                        row, gameId,
                        window.selectedBigBoxGameFavorite,
                        rating)
            gameList.forceActiveFocus()
        }

        onCancelled: gameList.forceActiveFocus()
    }

    BigBoxPlaylistPopup {
        id: bigBoxPlaylistPopup
        busy: controller.writing

        onSelected: function(row, gameId, playlistId) {
            controller.update_big_box_playlist_membership(
                        row, gameId, playlistId, true)
            gameList.forceActiveFocus()
        }

        onCancelled: gameList.forceActiveFocus()
    }

    BigBoxMarqueeWindow {
        id: bigBoxMarquee
        controller: controller
        requestedVisible:
            window.visible
            && controller.library_path.length > 0
            && controller.big_box_marquee_monitor_index >= 0
        windowedForSmoke: window.marqueeSmokeTest
        gameId: window.selectedBigBoxGameId
        gameTitle: window.selectedBigBoxGameTitle
        contextKind: window.marqueeContextKind
        contextName: window.marqueeContextName
    }

    BigBoxInputRouter {
        id: bigBoxInputRouter
        controller: controller
        enabled: window.visible
                 && controller.library_path.length > 0
                 && !bigBoxInputSettings.opened
                 && !bigBoxMarqueeSettings.opened
        onActionsTriggered: function(actions) {
            if (!bigBoxAttractMode.active)
                bigBoxAttractMode.noteActivity()
            if (!bigBoxScreensaver.active)
                bigBoxScreensaver.noteActivity()
            window.dispatchBigBoxInputCandidates(actions)
        }
    }

    Shortcut {
        sequence: "L"
        enabled: !bigBoxGameDetails.opened
                 && !bigBoxImageViewer.opened
                 && !bigBoxModelViewer.opened
                 && !bigBoxInputSettings.opened
                 && !bigBoxMarqueeSettings.opened
                 && !bigBoxSecuritySettings.opened
                 && !bigBoxUnlockPopup.opened
                 && !bigBoxPlaylistPopup.opened
                 && !bigBoxStarRatingPopup.opened
        onActivated: window.showLaunchWithSelection()
    }

    Shortcut {
        sequence: "Tab"
        enabled: !bigBoxGameDetails.opened
                 && !bigBoxImageViewer.opened
                 && !bigBoxModelViewer.opened
                 && !bigBoxInputSettings.opened
                 && !bigBoxMarqueeSettings.opened
                 && !bigBoxSecuritySettings.opened
                 && !bigBoxUnlockPopup.opened
                 && !bigBoxPlaylistPopup.opened
                 && !bigBoxStarRatingPopup.opened
                 && window.navigationAccessAvailable()
        onActivated: {
            if (navigationDrawer.opened) {
                navigationDrawer.close()
                gameList.forceActiveFocus()
            } else {
                window.openNavigation()
            }
        }
    }

}
