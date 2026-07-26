import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtMultimedia

Popup {
    id: root

    required property var controller
    property string contextKind: ""
    property string contextName: ""
    property bool backgroundMusicEnabled: false
    property bool blocked: false
    property bool shuffleEnabled: true
    property bool onScreenDisplayEnabled: true
    property bool mutedForSmoke: false
    property bool pinnedForSmoke: false
    property real outputVolume: 0.75
    property int currentTrackIndex: -1
    property string currentCollectionKey: ""
    property string currentContextKind: ""
    property string currentContextName: ""
    property bool pendingPlay: false
    property bool userPaused: false
    property bool pausedForBlock: false
    property bool stoppedByUser: false
    property int trackCount: 0
    property url trackSource
    property string trackName: ""
    readonly property int mediaRevision: controller.game_media_revision
    readonly property string resolvedCollectionKey: {
        const revision = mediaRevision
        return backgroundMusicEnabled
            ? controller.background_music_collection_key(
                  contextKind, contextName) : ""
    }
    readonly property int resolvedTrackCount: {
        const revision = mediaRevision
        return backgroundMusicEnabled
            ? controller.background_music_count(
                  contextKind, contextName) : 0
    }
    readonly property int playbackState: backgroundPlayer.playbackState
    readonly property int mediaStatus: backgroundPlayer.mediaStatus
    readonly property var mediaError: backgroundPlayer.error
    readonly property real duration: backgroundPlayer.duration
    readonly property real position: backgroundPlayer.position

    modal: false
    dim: false
    focus: false
    closePolicy: Popup.CloseOnEscape
    padding: 10
    implicitWidth: 800
    implicitHeight: 84

    function chooseShuffledIndex() {
        if (trackCount <= 1)
            return 0
        let nextIndex = Math.floor(Math.random() * trackCount)
        if (nextIndex === currentTrackIndex)
            nextIndex = (nextIndex + 1) % trackCount
        return nextIndex
    }

    function showOnScreenDisplay() {
        if (onScreenDisplayEnabled || pinnedForSmoke) {
            if (!opened)
                open()
            displayTimer.restart()
        }
    }

    function scheduleCollectionRefresh() {
        collectionRefreshTimer.restart()
    }

    function updateCurrentTrack() {
        if (currentTrackIndex < 0
                || currentTrackIndex >= trackCount) {
            trackSource = ""
            trackName = ""
            return false
        }
        trackSource = controller.background_music_url_at(
                    currentContextKind, currentContextName,
                    currentTrackIndex)
        trackName = controller.background_music_file_name_at(
                    currentContextKind, currentContextName,
                    currentTrackIndex)
        return trackSource.toString().length > 0
    }

    function schedulePlay() {
        if (!backgroundMusicEnabled || blocked || userPaused
                || stoppedByUser || trackCount === 0) {
            pendingPlay = false
            return false
        }
        pendingPlay = true
        Qt.callLater(function() {
            if (root.pendingPlay && !root.blocked
                    && root.trackSource.toString().length > 0) {
                root.pendingPlay = false
                backgroundPlayer.play()
                root.showOnScreenDisplay()
            }
        })
        return true
    }

    function refreshCollection() {
        if (!backgroundMusicEnabled || resolvedCollectionKey.length === 0
                || resolvedTrackCount === 0) {
            pendingPlay = false
            pausedForBlock = false
            backgroundPlayer.stop()
            currentTrackIndex = -1
            currentCollectionKey = ""
            trackCount = 0
            trackSource = ""
            trackName = ""
            close()
            return false
        }
        if (currentCollectionKey === resolvedCollectionKey
                && currentTrackIndex >= 0
                && currentTrackIndex < resolvedTrackCount) {
            trackCount = resolvedTrackCount
            updateCurrentTrack()
            if (backgroundPlayer.playbackState
                    === MediaPlayer.StoppedState
                    && !stoppedByUser)
                schedulePlay()
            return true
        }
        backgroundPlayer.stop()
        currentContextKind = contextKind
        currentContextName = contextName
        currentCollectionKey = resolvedCollectionKey
        trackCount = resolvedTrackCount
        currentTrackIndex = shuffleEnabled
                          ? chooseShuffledIndex() : 0
        updateCurrentTrack()
        userPaused = false
        pausedForBlock = false
        stoppedByUser = false
        return schedulePlay()
    }

    function switchTrack(index, playNow) {
        if (index < 0 || index >= trackCount)
            return false
        backgroundPlayer.stop()
        pendingPlay = playNow
        userPaused = false
        pausedForBlock = false
        stoppedByUser = false
        if (currentTrackIndex === index) {
            pendingPlay = false
            backgroundPlayer.position = 0
            if (playNow)
                schedulePlay()
        } else {
            currentTrackIndex = index
            updateCurrentTrack()
        }
        showOnScreenDisplay()
        return true
    }

    function previousTrack() {
        if (trackCount === 0)
            return false
        return switchTrack(
            currentTrackIndex <= 0
            ? trackCount - 1 : currentTrackIndex - 1, true)
    }

    function nextTrack() {
        if (trackCount === 0)
            return false
        const nextIndex = shuffleEnabled
                        ? chooseShuffledIndex()
                        : (currentTrackIndex + 1) % trackCount
        return switchTrack(nextIndex, true)
    }

    function togglePlayback() {
        if (trackCount === 0)
            return false
        stoppedByUser = false
        if (backgroundPlayer.playbackState
                === MediaPlayer.PlayingState) {
            userPaused = true
            pausedForBlock = false
            backgroundPlayer.pause()
        } else {
            userPaused = false
            if (!blocked) {
                if (backgroundPlayer.mediaStatus
                        === MediaPlayer.EndOfMedia)
                    backgroundPlayer.position = 0
                backgroundPlayer.play()
            }
        }
        showOnScreenDisplay()
        return true
    }

    function stopByUser() {
        pendingPlay = false
        userPaused = false
        pausedForBlock = false
        stoppedByUser = true
        backgroundPlayer.stop()
        close()
        return true
    }

    function stopForFrontend() {
        pendingPlay = false
        userPaused = false
        pausedForBlock = false
        stoppedByUser = false
        backgroundPlayer.stop()
        close()
        return true
    }

    function clickPlayPauseForSmoke() {
        if (!playPauseButton.enabled)
            return false
        playPauseButton.clicked()
        return true
    }

    function clickNextForSmoke() {
        if (!nextButton.enabled)
            return false
        nextButton.clicked()
        return true
    }

    function clickPreviousForSmoke() {
        if (!previousButton.enabled)
            return false
        previousButton.clicked()
        return true
    }

    function clickStopForSmoke() {
        if (!stopButton.enabled)
            return false
        stopButton.clicked()
        return true
    }

    onResolvedCollectionKeyChanged: scheduleCollectionRefresh()
    onResolvedTrackCountChanged: scheduleCollectionRefresh()
    onBackgroundMusicEnabledChanged: scheduleCollectionRefresh()
    onMediaRevisionChanged: scheduleCollectionRefresh()

    onTrackSourceChanged: {
        if (pendingPlay && trackSource.toString().length > 0) {
            pendingPlay = false
            Qt.callLater(function() {
                if (!root.blocked && !root.userPaused
                        && !root.stoppedByUser) {
                    backgroundPlayer.play()
                    root.showOnScreenDisplay()
                }
            })
        }
    }

    onBlockedChanged: {
        if (blocked) {
            pendingPlay = false
            if (backgroundPlayer.playbackState
                    === MediaPlayer.PlayingState) {
                pausedForBlock = true
                backgroundPlayer.pause()
            }
        } else if (pausedForBlock && !userPaused
                   && !stoppedByUser && backgroundMusicEnabled) {
            pausedForBlock = false
            backgroundPlayer.play()
            showOnScreenDisplay()
        } else if (!userPaused && !stoppedByUser
                   && backgroundMusicEnabled
                   && backgroundPlayer.playbackState
                      === MediaPlayer.StoppedState) {
            schedulePlay()
        }
    }

    background: Rectangle {
        radius: 9
        color: "#f3161d27"
        border.color: "#6599cf"
        border.width: 1
    }

    contentItem: RowLayout {
        spacing: 9

        Label {
            text: "♫"
            color: "#70b7ff"
            font.pixelSize: 24
            Accessible.name: "Background music"
        }
        Button {
            id: previousButton
            text: "◀"
            enabled: root.trackCount > 1
            Accessible.name: "Previous background music track"
            onClicked: root.previousTrack()
        }
        Button {
            id: playPauseButton
            text: root.playbackState === MediaPlayer.PlayingState
                  ? "PAUSE" : "PLAY"
            enabled: root.trackCount > 0 && !root.blocked
            Accessible.name: "Play or pause background music"
            onClicked: root.togglePlayback()
        }
        Button {
            id: nextButton
            text: "▶"
            enabled: root.trackCount > 1 && !root.blocked
            Accessible.name: "Next background music track"
            onClicked: root.nextTrack()
        }
        Button {
            id: stopButton
            text: "STOP"
            enabled: root.trackCount > 0
            Accessible.name: "Stop background music"
            onClicked: root.stopByUser()
        }
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 1
            Label {
                Layout.fillWidth: true
                text: root.currentCollectionKey === "default"
                      ? "BACKGROUND MUSIC"
                      : "BACKGROUND MUSIC · "
                        + root.currentContextKind.toUpperCase()
                        + " — " + root.currentContextName
                color: "#ffffff"
                font.bold: true
                elide: Text.ElideRight
            }
            Label {
                Layout.fillWidth: true
                text: root.trackName
                      + (root.trackCount > 1
                         ? "  ·  " + (root.currentTrackIndex + 1)
                           + " / " + root.trackCount : "")
                color: "#a9bdd6"
                elide: Text.ElideMiddle
            }
        }
        Label {
            text: "VOL"
            color: "#a9bdd6"
        }
        Slider {
            from: 0
            to: 1
            stepSize: 0.01
            value: root.outputVolume
            Layout.preferredWidth: 100
            Accessible.name: "Background music volume"
            onMoved: root.outputVolume = value
        }
    }

    Timer {
        id: collectionRefreshTimer
        interval: 40
        onTriggered: root.refreshCollection()
    }

    Timer {
        id: displayTimer
        interval: 4000
        onTriggered: {
            if (!root.pinnedForSmoke)
                root.close()
        }
    }

    AudioOutput {
        id: backgroundAudio
        muted: root.mutedForSmoke
        volume: Math.max(0, Math.min(1, root.outputVolume))
    }

    MediaPlayer {
        id: backgroundPlayer
        source: root.trackSource
        audioOutput: backgroundAudio

        onMediaStatusChanged: {
            if (mediaStatus === MediaPlayer.EndOfMedia)
                root.nextTrack()
        }

        onErrorOccurred: function(error, errorString) {
            console.warn("Background music playback error "
                         + error + ": " + errorString)
        }
    }
}
