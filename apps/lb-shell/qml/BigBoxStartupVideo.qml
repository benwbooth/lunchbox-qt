import QtQuick
import QtQuick.Controls
import QtMultimedia

Item {
    id: root

    required property var controller
    property bool awaitingDecision: false
    property bool active: false
    property bool pendingPlay: false
    property bool mutedForSmoke: false
    property real outputVolume: 0.75
    property int selectedIndex: -1
    property url selectedSource
    property string selectedName: ""
    property bool skipped: false
    property bool endedNaturally: false
    property bool failed: false
    readonly property int playbackState: startupPlayer.playbackState
    readonly property int mediaStatus: startupPlayer.mediaStatus
    readonly property var mediaError: startupPlayer.error
    readonly property real duration: startupPlayer.duration
    readonly property real position: startupPlayer.position

    signal completed()

    visible: awaitingDecision || active
    focus: active

    function chooseIndex(requestedIndex) {
        const count = controller.indexed_startup_video_count
        if (count <= 0)
            return -1
        if (requestedIndex >= 0 && requestedIndex < count)
            return requestedIndex
        return Math.floor(Math.random() * count)
    }

    function begin(requestedIndex) {
        const index = chooseIndex(requestedIndex)
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

    function finishPlayback(wasSkipped, wasNatural, didFail) {
        if (!active)
            return false
        pendingPlay = false
        startupPlayer.stop()
        skipped = wasSkipped
        endedNaturally = wasNatural
        failed = didFail
        active = false
        completed()
        return true
    }

    function skipPlayback() {
        return finishPlayback(true, false, false)
    }

    function stopForFrontend() {
        pendingPlay = false
        startupPlayer.stop()
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

    VideoOutput {
        id: startupVideoOutput
        anchors.fill: parent
        visible: root.active
        fillMode: VideoOutput.PreserveAspectFit
    }

    Column {
        anchors.centerIn: parent
        visible: root.awaitingDecision && !root.active
        spacing: 12

        Label {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "BIGBOX"
            color: "#67b3ff"
            font.pixelSize: 52
            font.bold: true
            font.letterSpacing: 5
        }

        Label {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "LOADING LIBRARY"
            color: "#91a5bd"
            font.pixelSize: 16
            font.letterSpacing: 2
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
}
