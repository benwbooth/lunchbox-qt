import QtQuick
import QtQuick.Controls
import QtMultimedia

Item {
    id: root

    required property var controller
    property bool coverVisible: false
    property bool showSplashBranding: true
    property bool active: false
    property bool pendingPlay: false
    property bool mutedForSmoke: false
    property real outputVolume: 0.75
    property real soundOutputVolume: 1
    property int selectedIndex: -1
    property url selectedSource
    property string selectedName: ""
    property bool soundPendingPlay: false
    property bool soundStarted: false
    property bool soundFailed: false
    property int selectedSoundIndex: -1
    property url selectedSoundSource
    property string selectedSoundName: ""
    property bool skipped: false
    property bool endedNaturally: false
    property bool failed: false
    readonly property int playbackState: startupPlayer.playbackState
    readonly property int mediaStatus: startupPlayer.mediaStatus
    readonly property var mediaError: startupPlayer.error
    readonly property real duration: startupPlayer.duration
    readonly property real position: startupPlayer.position
    readonly property int soundPlaybackState:
        startupSoundPlayer.playbackState
    readonly property int soundMediaStatus:
        startupSoundPlayer.mediaStatus
    readonly property var soundMediaError: startupSoundPlayer.error
    readonly property real soundDuration: startupSoundPlayer.duration

    signal videoCompleted()

    visible: coverVisible || active
    focus: active

    function chooseVideoIndex(requestedIndex) {
        const count = controller.indexed_startup_video_count
        if (count <= 0)
            return -1
        if (requestedIndex >= 0 && requestedIndex < count)
            return requestedIndex
        return Math.floor(Math.random() * count)
    }

    function chooseSoundIndex(requestedIndex) {
        const count = controller.indexed_startup_sound_count
        if (count <= 0)
            return -1
        if (requestedIndex >= 0 && requestedIndex < count)
            return requestedIndex
        return Math.floor(Math.random() * count)
    }

    function beginVideo(requestedIndex) {
        const index = chooseVideoIndex(requestedIndex)
        if (index < 0)
            return false
        const source = controller.startup_video_url_at(index)
        const name = controller.startup_video_file_name_at(index)
        if (source.toString().length === 0 || name.length === 0)
            return false

        startupPlayer.stop()
        selectedIndex = index
        selectedSource = source
        selectedName = name
        skipped = false
        endedNaturally = false
        failed = false
        pendingPlay = true
        active = true
        root.forceActiveFocus()
        return true
    }

    function beginStartupSound(requestedIndex) {
        const index = chooseSoundIndex(requestedIndex)
        if (index < 0)
            return false
        const source = controller.startup_sound_url_at(index)
        const name = controller.startup_sound_file_name_at(index)
        if (source.toString().length === 0 || name.length === 0)
            return false

        startupSoundPlayer.stop()
        selectedSoundIndex = index
        selectedSoundSource = source
        selectedSoundName = name
        soundPendingPlay = true
        soundStarted = false
        soundFailed = false
        return true
    }

    function finishPlayback(wasSkipped, wasNatural, didFail) {
        if (!active)
            return false
        pendingPlay = false
        startupPlayer.stop()
        skipped = wasSkipped
        endedNaturally = wasNatural
        failed = didFail
        active = false
        videoCompleted()
        return true
    }

    function skipPlayback() {
        return finishPlayback(true, false, false)
    }

    function stopForFrontend() {
        pendingPlay = false
        startupPlayer.stop()
        soundPendingPlay = false
        startupSoundPlayer.stop()
        active = false
        return true
    }

    function triggerSkipForSmoke() {
        if (!active)
            return false
        skipAction.trigger()
        return true
    }

    onActiveChanged: {
        if (active)
            Qt.callLater(function() {
                root.forceActiveFocus()
            })
    }

    Keys.onPressed: function(event) {
        if (!active)
            return
        skipAction.trigger()
        event.accepted = true
    }

    Action {
        id: skipAction
        enabled: root.active
        onTriggered: root.skipPlayback()
    }

    TapHandler {
        enabled: root.active
        onTapped: skipAction.trigger()
    }

    Rectangle {
        anchors.fill: parent
        color: "#000000"
    }

    Rectangle {
        anchors.fill: parent
        visible: root.coverVisible && !root.active
        gradient: Gradient {
            GradientStop {
                position: 0
                color: "#05070b"
            }
            GradientStop {
                position: 0.56
                color: "#101a28"
            }
            GradientStop {
                position: 1
                color: "#05070b"
            }
        }
    }

    VideoOutput {
        id: startupVideoOutput
        anchors.fill: parent
        visible: root.active
        fillMode: VideoOutput.PreserveAspectFit
    }

    Column {
        anchors.centerIn: parent
        visible: root.coverVisible && !root.active
                 && root.showSplashBranding
        spacing: 20

        Row {
            anchors.horizontalCenter: parent.horizontalCenter
            spacing: 16

            Label {
                anchors.verticalCenter: parent.verticalCenter
                text: "BIGBOX"
                color: "#f5f8fc"
                font.pixelSize: 58
                font.bold: true
                font.letterSpacing: 3
            }

            Column {
                anchors.verticalCenter: parent.verticalCenter
                spacing: 5

                Repeater {
                    model: ["#58c7ff", "#248dde", "#12559a"]

                    Rectangle {
                        required property string modelData
                        width: 42
                        height: 8
                        radius: 4
                        color: modelData
                    }
                }
            }
        }

        Label {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "CROSS-PLATFORM PORT"
            color: "#67b3ff"
            font.pixelSize: 15
            font.bold: true
            font.letterSpacing: 4
        }

        BusyIndicator {
            anchors.horizontalCenter: parent.horizontalCenter
            width: 42
            height: 42
            running: visible
        }

        Label {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "LOADING LIBRARY"
            color: "#a9b8ca"
            font.pixelSize: 14
            font.letterSpacing: 3
        }
    }

    Rectangle {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        height: 64
        visible: root.active
        color: "#b0000000"

        Label {
            anchors.centerIn: parent
            text: "PRESS ANY KEY OR BUTTON TO SKIP"
            color: "#ffffff"
            font.pixelSize: 18
            font.bold: true
            font.letterSpacing: 1
            Accessible.name: "Skip startup video"
        }
    }

    AudioOutput {
        id: startupAudio
        muted: root.mutedForSmoke
        volume: Math.max(0, Math.min(1, root.outputVolume))
    }

    MediaPlayer {
        id: startupPlayer
        source: root.selectedSource
        audioOutput: startupAudio
        videoOutput: startupVideoOutput

        onMediaStatusChanged: {
            if ((startupPlayer.mediaStatus === MediaPlayer.LoadedMedia
                    || startupPlayer.mediaStatus === MediaPlayer.BufferedMedia)
                    && root.active && root.pendingPlay) {
                root.pendingPlay = false
                Qt.callLater(function() {
                    startupPlayer.play()
                })
            } else if (startupPlayer.mediaStatus === MediaPlayer.EndOfMedia
                       && root.active) {
                root.finishPlayback(false, true, false)
            } else if (startupPlayer.mediaStatus === MediaPlayer.InvalidMedia
                       && root.active) {
                root.finishPlayback(false, false, true)
            }
        }

        onErrorOccurred: function(error, errorString) {
            console.warn("BigBox startup video error "
                         + error + ": " + errorString)
            if (root.active)
                root.finishPlayback(false, false, true)
        }
    }

    AudioOutput {
        id: startupSoundAudio
        muted: root.mutedForSmoke
        volume: Math.max(0, Math.min(1, root.soundOutputVolume))
    }

    MediaPlayer {
        id: startupSoundPlayer
        source: root.selectedSoundSource
        audioOutput: startupSoundAudio

        onMediaStatusChanged: {
            if ((startupSoundPlayer.mediaStatus
                    === MediaPlayer.LoadedMedia
                    || startupSoundPlayer.mediaStatus
                    === MediaPlayer.BufferedMedia)
                    && root.soundPendingPlay) {
                root.soundPendingPlay = false
                Qt.callLater(function() {
                    startupSoundPlayer.play()
                })
            } else if (startupSoundPlayer.mediaStatus
                       === MediaPlayer.InvalidMedia) {
                root.soundPendingPlay = false
                root.soundFailed = true
            }
        }

        onPlaybackStateChanged: {
            if (startupSoundPlayer.playbackState
                    === MediaPlayer.PlayingState)
                root.soundStarted = true
        }

        onErrorOccurred: function(error, errorString) {
            console.warn("BigBox startup sound error "
                         + error + ": " + errorString)
            root.soundPendingPlay = false
            root.soundFailed = true
        }
    }
}
