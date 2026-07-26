import QtQuick
import QtQuick.Controls
import QtMultimedia

Window {
    id: root

    required property var controller
    property bool requestedVisible: false
    property bool windowedForSmoke: false
    property string gameId: ""
    property string gameTitle: ""
    property string contextKind: "game"
    property string contextName: ""
    property bool videoFailed: false
    property bool suspendVideoForCapture: false

    property int screenCount: 0
    readonly property int configuredMonitorIndex:
        controller.big_box_marquee_monitor_index
    readonly property bool monitorIsValid:
        configuredMonitorIndex >= 0
        && configuredMonitorIndex < screenCount
    readonly property int resolvedMonitorIndex:
        monitorIsValid ? configuredMonitorIndex : -1
    readonly property string compatibilityMode:
        controller.big_box_marquee_compatibility_mode
    readonly property bool platformPresentation:
        contextKind === "platform" && contextName.length > 0
    readonly property bool themeFallbackEnabled:
        !controller.big_box_marquee_ignore_theme_views

    readonly property url gameVideoUrl: {
        const revision = controller.game_media_revision
        return gameId.length > 0
            ? controller.big_box_game_marquee_video_url(gameId) : ""
    }
    readonly property url gameImageUrl: {
        const revision = controller.game_media_revision
        return gameId.length > 0
            ? controller.big_box_game_marquee_image_url(gameId) : ""
    }
    readonly property url gameClearLogoUrl: {
        const revision = controller.game_media_revision
        return gameId.length > 0
            ? controller.big_box_game_marquee_clear_logo_url(gameId) : ""
    }
    readonly property url gameBoxArtUrl: {
        const revision = controller.game_media_revision
        return gameId.length > 0
            ? controller.big_box_game_marquee_box_art_url(gameId) : ""
    }
    readonly property url gameBackgroundUrl: {
        const revision = controller.game_media_revision
        return gameId.length > 0
            ? controller.big_box_game_marquee_background_url(gameId) : ""
    }
    readonly property url platformBannerUrl:
        contextName.length > 0
        ? controller.big_box_platform_marquee_banner_url(contextName) : ""
    readonly property url platformClearLogoUrl:
        contextName.length > 0
        ? controller.big_box_platform_marquee_clear_logo_url(contextName) : ""
    readonly property url platformBackgroundUrl:
        contextName.length > 0
        ? controller.big_box_platform_marquee_background_url(contextName) : ""
    readonly property url selectedVideoUrl:
        !platformPresentation ? gameVideoUrl : ""
    readonly property bool directVideoVisible:
        selectedVideoUrl.toString().length > 0
        && !videoFailed && !suspendVideoForCapture
    readonly property bool directGameImageVisible:
        !platformPresentation && !directVideoVisible
        && gameImageUrl.toString().length > 0
    readonly property bool directPlatformImageVisible:
        platformPresentation
        && platformBannerUrl.toString().length > 0
    readonly property bool videoReady:
        directVideoVisible
        && (marqueePlayer.mediaStatus === MediaPlayer.LoadedMedia
            || marqueePlayer.mediaStatus === MediaPlayer.BufferedMedia)
    readonly property int videoMediaStatus: marqueePlayer.mediaStatus
    readonly property int videoPlaybackState: marqueePlayer.playbackState
    readonly property int directImageStatus: directImage.status
    readonly property alias captureTarget: marqueeViewport

    // These fractions implement the named 13.27 compatibility choices in a
    // deterministic, host-independent way: content is compressed into the
    // portion of a display that remains visible.
    readonly property real compatibilityTopFraction:
        compatibilityMode === "TopHalfCutOff" ? 0.5
        : compatibilityMode === "TopTwoThirdsCutOff" ? 2 / 3
        : compatibilityMode === "TopAndBottomOneThirdCutOff" ? 1 / 3
        : 0
    readonly property real compatibilityHeightFraction:
        compatibilityMode === "HalfSizeStretched"
        || compatibilityMode === "BottomHalfCutOff"
        || compatibilityMode === "TopHalfCutOff" ? 0.5
        : compatibilityMode === "ThirdSizeStretched"
          || compatibilityMode === "BottomTwoThirdsCutOff"
          || compatibilityMode === "TopTwoThirdsCutOff"
          || compatibilityMode === "TopAndBottomOneThirdCutOff" ? 1 / 3
        : 1

    title: platformPresentation
           ? "BigBox Marquee — " + contextName
           : "BigBox Marquee — " + gameTitle
    color: "black"
    flags: Qt.FramelessWindowHint
           | Qt.WindowDoesNotAcceptFocus
           | Qt.WindowStaysOnTopHint
    transientParent: null
    visibility: requestedVisible && monitorIsValid
                ? (windowedForSmoke ? Window.Windowed : Window.FullScreen)
                : Window.Hidden
    width: 1000
    height: 300

    function refreshHostScreens() {
        const nextCount = controller.host_screen_count()
        if (nextCount !== screenCount)
            screenCount = nextCount
        if (monitorIsValid)
            controller.route_window_to_host_screen(
                root, configuredMonitorIndex)
    }

    function stopPlayback() {
        marqueePlayer.stop()
    }

    function configurePlayback() {
        marqueePlayer.stop()
        videoFailed = false
        marqueePlayer.source = selectedVideoUrl
        if (requestedVisible && monitorIsValid
                && selectedVideoUrl.toString().length > 0)
            marqueePlayer.play()
    }

    onSelectedVideoUrlChanged: configurePlayback()
    onRequestedVisibleChanged: configurePlayback()
    onMonitorIsValidChanged: {
        refreshHostScreens()
        configurePlayback()
    }
    onConfiguredMonitorIndexChanged: refreshHostScreens()
    onPlatformPresentationChanged: configurePlayback()
    Component.onCompleted: refreshHostScreens()

    Timer {
        interval: 1000
        repeat: true
        running: true
        onTriggered: root.refreshHostScreens()
    }

    Rectangle {
        anchors.fill: parent
        color: "black"
    }

    Item {
        id: marqueeViewport
        x: 0
        y: root.height * root.compatibilityTopFraction
        width: root.width
        height: root.height * root.compatibilityHeightFraction
        clip: true

        Image {
            id: directImage
            anchors.fill: parent
            source: root.platformPresentation
                    ? root.platformBackgroundUrl
                    : root.gameBackgroundUrl
            visible: root.themeFallbackEnabled
                     && !root.directVideoVisible
                     && !root.directGameImageVisible
                     && !root.directPlatformImageVisible
                     && source.toString().length > 0
            asynchronous: true
            cache: false
            fillMode: Image.PreserveAspectCrop
        }

        Rectangle {
            anchors.fill: parent
            visible: root.themeFallbackEnabled
                     && !root.directVideoVisible
                     && !root.directGameImageVisible
                     && !root.directPlatformImageVisible
            gradient: Gradient {
                GradientStop { position: 0; color: "#15000000" }
                GradientStop { position: 1; color: "#b0000000" }
            }
        }

        Image {
            anchors.centerIn: parent
            width: parent.width * 0.38
            height: parent.height * 0.82
            source: root.gameBoxArtUrl
            visible: !root.platformPresentation
                     && root.themeFallbackEnabled
                     && !root.directVideoVisible
                     && !root.directGameImageVisible
                     && source.toString().length > 0
            asynchronous: true
            cache: false
            fillMode: Image.PreserveAspectFit
        }

        Image {
            anchors.centerIn: parent
            width: parent.width * 0.82
            height: parent.height * 0.72
            source: root.platformPresentation
                    ? root.platformClearLogoUrl
                    : root.gameClearLogoUrl
            visible: root.themeFallbackEnabled
                     && !root.directVideoVisible
                     && !root.directGameImageVisible
                     && !root.directPlatformImageVisible
                     && source.toString().length > 0
            asynchronous: true
            cache: false
            fillMode: Image.PreserveAspectFit
        }

        Image {
            anchors.fill: parent
            source: root.platformPresentation
                    ? root.platformBannerUrl : root.gameImageUrl
            visible: root.directPlatformImageVisible
                     || root.directGameImageVisible
            asynchronous: true
            cache: false
            fillMode: root.controller.big_box_marquee_stretch_images
                      ? Image.Stretch : Image.PreserveAspectFit
        }

        VideoOutput {
            id: marqueeVideoOutput
            anchors.fill: parent
            visible: root.directVideoVisible
            fillMode: root.controller.big_box_marquee_stretch_images
                      ? VideoOutput.Stretch
                      : VideoOutput.PreserveAspectFit
        }

        Label {
            anchors.centerIn: parent
            visible: root.themeFallbackEnabled
                     && !root.directVideoVisible
                     && !root.directGameImageVisible
                     && !root.directPlatformImageVisible
                     && root.gameBackgroundUrl.toString().length === 0
                     && root.platformBackgroundUrl.toString().length === 0
                     && root.gameClearLogoUrl.toString().length === 0
                     && root.platformClearLogoUrl.toString().length === 0
            text: root.platformPresentation
                  ? root.contextName.toUpperCase()
                  : root.gameTitle.toUpperCase()
            color: "#e8f3ff"
            font.pixelSize: Math.max(22, parent.height * 0.18)
            font.bold: true
            font.letterSpacing: 2
        }
    }

    AudioOutput {
        id: silentAudio
        muted: true
        volume: 0
    }

    MediaPlayer {
        id: marqueePlayer
        audioOutput: silentAudio
        videoOutput: marqueeVideoOutput

        onMediaStatusChanged: {
            if ((mediaStatus === MediaPlayer.LoadedMedia
                    || mediaStatus === MediaPlayer.BufferedMedia)
                    && root.requestedVisible && root.monitorIsValid)
                play()
            else if (mediaStatus === MediaPlayer.EndOfMedia
                     && root.requestedVisible && root.monitorIsValid) {
                position = 0
                play()
            } else if (mediaStatus === MediaPlayer.InvalidMedia) {
                root.videoFailed = true
            }
        }
    }
}
