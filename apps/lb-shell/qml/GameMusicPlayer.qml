import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtMultimedia

Popup {
    id: root

    required property var controller
    property string gameId: ""
    property string gameTitle: ""
    property int currentTrackIndex: -1
    property bool shuffleEnabled: false
    property bool repeatEnabled: false
    property bool mutedForSmoke: false
    property real outputVolume: 0.75
    property bool pendingPlay: false
    readonly property int mediaRevision: controller.game_media_revision
    readonly property int trackCount: {
        const revision = mediaRevision
        return gameId.length > 0
            ? controller.game_music_count_for_game(gameId) : 0
    }
    readonly property url trackSource: {
        const revision = mediaRevision
        return currentTrackIndex >= 0 && currentTrackIndex < trackCount
            ? controller.game_music_url_at(gameId, currentTrackIndex) : ""
    }
    readonly property string trackName: {
        const revision = mediaRevision
        return currentTrackIndex >= 0 && currentTrackIndex < trackCount
            ? controller.game_music_file_name_at(gameId, currentTrackIndex) : ""
    }
    readonly property int playbackState: musicPlayer.playbackState
    readonly property int mediaStatus: musicPlayer.mediaStatus
    readonly property int mediaError: musicPlayer.error
    readonly property real duration: musicPlayer.duration
    readonly property real position: musicPlayer.position

    modal: false
    dim: false
    focus: false
    closePolicy: Popup.NoAutoClose
    padding: 10
    implicitWidth: 760
    implicitHeight: 82

    function openForGame(requestedGameId, requestedTitle, playNow) {
        if (requestedGameId.length === 0
                || controller.game_music_count_for_game(
                    requestedGameId) === 0)
            return false
        const changed = gameId !== requestedGameId
        if (changed) {
            musicPlayer.stop()
            gameId = requestedGameId
            gameTitle = requestedTitle
            currentTrackIndex = 0
        } else if (currentTrackIndex < 0
                   || currentTrackIndex >= trackCount) {
            currentTrackIndex = 0
        }
        if (!opened)
            open()
        if (playNow) {
            pendingPlay = true
            Qt.callLater(function() {
                if (root.pendingPlay
                        && root.trackSource.toString().length > 0) {
                    root.pendingPlay = false
                    musicPlayer.play()
                }
            })
        }
        return true
    }

    function togglePlayback() {
        if (trackCount === 0)
            return false
        if (musicPlayer.playbackState === MediaPlayer.PlayingState) {
            musicPlayer.pause()
        } else {
            if (musicPlayer.mediaStatus === MediaPlayer.EndOfMedia)
                musicPlayer.position = 0
            musicPlayer.play()
        }
        return true
    }

    function chooseShuffledIndex() {
        if (trackCount <= 1)
            return 0
        let nextIndex = Math.floor(Math.random() * trackCount)
        if (nextIndex === currentTrackIndex)
            nextIndex = (nextIndex + 1) % trackCount
        return nextIndex
    }

    function switchTrack(index, playNow) {
        if (index < 0 || index >= trackCount)
            return false
        musicPlayer.stop()
        pendingPlay = playNow
        if (currentTrackIndex === index) {
            if (playNow) {
                pendingPlay = false
                musicPlayer.position = 0
                musicPlayer.play()
            }
        } else {
            currentTrackIndex = index
        }
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

    function stopPlayback(closePlayer) {
        pendingPlay = false
        musicPlayer.stop()
        if (closePlayer)
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
        if (!nextTrackButton.enabled)
            return false
        nextTrackButton.clicked()
        return true
    }

    function clickStopForSmoke() {
        if (!stopButton.enabled)
            return false
        stopButton.clicked()
        return true
    }

    onTrackSourceChanged: {
        if (pendingPlay && trackSource.toString().length > 0) {
            pendingPlay = false
            Qt.callLater(function() {
                musicPlayer.play()
            })
        }
    }

    onMediaRevisionChanged: {
        if (gameId.length === 0)
            return
        if (controller.game_music_count_for_game(gameId) === 0) {
            stopPlayback(true)
            gameId = ""
            gameTitle = ""
            currentTrackIndex = -1
        } else if (currentTrackIndex >= trackCount) {
            currentTrackIndex = 0
        }
    }

    onClosed: musicPlayer.stop()

    background: Rectangle {
        radius: 9
        color: "#f3161d27"
        border.color: "#5b7698"
        border.width: 1
    }

    contentItem: RowLayout {
        spacing: 9

        Button {
            text: "◀"
            enabled: root.trackCount > 1
            Accessible.name: "Previous music track"
            onClicked: root.previousTrack()
        }
        Button {
            id: playPauseButton
            text: root.playbackState === MediaPlayer.PlayingState
                  ? "PAUSE" : "PLAY"
            enabled: root.trackCount > 0
            Accessible.name: "Play or pause game music"
            onClicked: root.togglePlayback()
        }
        Button {
            id: nextTrackButton
            text: "▶"
            enabled: root.trackCount > 1
            Accessible.name: "Next music track"
            onClicked: root.nextTrack()
        }
        Button {
            id: stopButton
            text: "STOP"
            enabled: root.trackCount > 0
            Accessible.name: "Stop game music"
            onClicked: root.stopPlayback(true)
        }
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 1
            Label {
                Layout.fillWidth: true
                text: root.gameTitle.length > 0
                      ? root.gameTitle : "Game music"
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
            Layout.preferredWidth: 110
            Accessible.name: "Game music volume"
            onMoved: root.outputVolume = value
        }
    }

    AudioOutput {
        id: musicAudio
        muted: root.mutedForSmoke
        volume: Math.max(0, Math.min(1, root.outputVolume))
    }

    MediaPlayer {
        id: musicPlayer
        source: root.trackSource
        audioOutput: musicAudio

        onMediaStatusChanged: {
            if (mediaStatus !== MediaPlayer.EndOfMedia)
                return
            if (root.trackCount > 1
                    && (root.repeatEnabled
                        || root.currentTrackIndex + 1 < root.trackCount)) {
                const nextIndex = root.shuffleEnabled
                                ? root.chooseShuffledIndex()
                                : (root.currentTrackIndex + 1)
                                  % root.trackCount
                root.switchTrack(nextIndex, true)
            } else if (root.repeatEnabled) {
                position = 0
                play()
            }
        }

        onErrorOccurred: function(error, errorString) {
            console.warn("Game music playback error "
                         + error + ": " + errorString)
        }
    }
}
