import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtMultimedia

Item {
    id: root

    required property var controller
    property var exploreGameCallback
    property var focusReturnCallback
    property var activationCallback
    property bool blocked: false
    property bool mutedForSmoke: false
    property bool active: false
    property string lastStartSource: ""
    property string lastStopReason: ""
    property int candidateIndex: -1
    property string currentGameId: ""
    property string currentTitle: ""
    property string currentPlatform: ""
    property string currentGenre: ""
    property string currentReleaseDate: ""
    property double currentPlayTimeSeconds: 0
    property double currentStarRating: 0
    property url currentBackgroundUrl: ""
    property url currentBoxArtUrl: ""
    property url currentScreenshotUrl: ""
    property url currentVideoUrl: ""
    property int swapCount: 0
    property int selectionCount: 0
    property int inputStopCount: 0
    property int exploreCount: 0
    property int manualStartCount: 0
    property double idleCountdownStartedAt: 0
    property int lastAutomaticDelayElapsedMs: 0
    property bool videoPlaybackSeen: false
    property bool mediaRevealed: false
    property int smokeViewOverride: 0
    property int presentedViewsMask: 0
    readonly property string presentationView:
        smokeViewOverride >= 1 && smokeViewOverride <= 4
        ? "Screensaver" + smokeViewOverride + "View"
        : controller.big_box_screensaver_view
    readonly property int presentationOrdinal:
        presentationView === "Screensaver2View" ? 2
        : presentationView === "Screensaver3View" ? 3
        : presentationView === "Screensaver4View" ? 4 : 1
    readonly property bool automaticEligible:
        controller.big_box_screensaver_enabled
        && controller.library_path.length > 0
        && controller.big_box_screensaver_candidate_count > 0
        && !blocked
        && !active
    readonly property bool videoPresentation:
        presentationOrdinal >= 2
    readonly property bool videoReady:
        currentVideoUrl.toString().length > 0
        && (screensaverPlayer.mediaStatus === MediaPlayer.LoadedMedia
            || screensaverPlayer.mediaStatus === MediaPlayer.BufferedMedia)
    readonly property int videoMediaStatus: screensaverPlayer.mediaStatus
    readonly property int videoPlaybackState: screensaverPlayer.playbackState

    function startAutomatic() {
        return startMode("automatic")
    }

    function startManual() {
        return startMode("manual")
    }

    function startMode(source) {
        if (blocked || active
                || controller.big_box_screensaver_candidate_count <= 0)
            return false
        idleTimer.stop()
        swapTimer.stop()
        lastStartSource = source
        lastStopReason = ""
        if (source === "automatic") {
            lastAutomaticDelayElapsedMs =
                idleCountdownStartedAt > 0
                ? Math.max(0, Math.round(
                    Date.now() - idleCountdownStartedAt)) : 0
        } else {
            manualStartCount += 1
        }
        if (!selectCandidate(""))
            return false
        active = true
        configureVideo()
        if (activationCallback)
            activationCallback()
        inputLayer.forceActiveFocus()
        scheduleSwap()
        return true
    }

    function stopMode(reason) {
        if (!active)
            return false
        swapTimer.stop()
        screensaverPlayer.stop()
        active = false
        lastStopReason = reason
        if (reason === "input")
            inputStopCount += 1
        if (focusReturnCallback)
            Qt.callLater(focusReturnCallback)
        if (automaticEligible)
            idleTimer.restart()
        return true
    }

    function noteActivity() {
        if (active)
            return stopMode("input")
        if (automaticEligible)
            idleTimer.restart()
        return false
    }

    function selectCandidate(avoidGameId) {
        const selected =
            controller.select_big_box_screensaver_candidate(avoidGameId)
        if (selected < 0)
            return false
        candidateIndex = selected
        currentGameId =
            controller.big_box_screensaver_game_id_at(selected)
        currentTitle =
            controller.big_box_screensaver_title_at(selected)
        currentPlatform =
            controller.big_box_screensaver_platform_at(selected)
        currentGenre =
            controller.big_box_screensaver_genre_at(selected)
        currentReleaseDate =
            controller.big_box_screensaver_release_date_at(selected)
        currentPlayTimeSeconds =
            controller.big_box_screensaver_play_time_seconds_at(selected)
        currentStarRating =
            controller.big_box_screensaver_star_rating_at(selected)
        currentBackgroundUrl =
            controller.big_box_screensaver_background_url_at(selected)
        currentBoxArtUrl =
            controller.big_box_screensaver_box_art_url_at(selected)
        currentScreenshotUrl =
            controller.big_box_screensaver_screenshot_url_at(selected)
        currentVideoUrl =
            controller.big_box_screensaver_video_url_at(selected)
        selectionCount += 1
        markPresentedView()
        configureVideo()
        presentationFade.restart()
        return currentGameId.length > 0
    }

    function configureVideo() {
        mediaRevealTimer.stop()
        mediaRevealed = presentationOrdinal === 1
                        || presentationOrdinal === 3
        screensaverPlayer.stop()
        screensaverPlayer.source =
            videoPresentation ? currentVideoUrl : ""
        if (active && videoPresentation
                && currentVideoUrl.toString().length > 0)
            screensaverPlayer.play()
        if (active
                && (presentationOrdinal === 2
                    || presentationOrdinal === 4)
                && (currentVideoUrl.toString().length > 0
                    || currentScreenshotUrl.toString().length > 0))
            mediaRevealTimer.restart()
    }

    function scheduleSwap() {
        if (!active)
            return
        swapTimer.interval =
            controller.big_box_screensaver_swap_time_ms(
                selectionCount + swapCount)
        swapTimer.restart()
    }

    function swapCandidate() {
        if (!active || blocked) {
            stopMode("blocked")
            return
        }
        if (selectCandidate(currentGameId))
            swapCount += 1
        scheduleSwap()
    }

    function exploreCurrentGame() {
        if (!active || currentGameId.length === 0)
            return false
        const gameId = currentGameId
        exploreCount += 1
        stopMode("explore")
        if (exploreGameCallback)
            Qt.callLater(function() {
                exploreGameCallback(gameId)
            })
        return true
    }

    function markPresentedView() {
        presentedViewsMask |= 1 << (presentationOrdinal - 1)
    }

    function setSmokeView(viewOrdinal) {
        smokeViewOverride = viewOrdinal
        if (active) {
            markPresentedView()
            configureVideo()
            presentationFade.restart()
        }
    }

    function clickReturnForSmoke() {
        returnButton.clicked()
    }

    function clickExploreForSmoke() {
        viewGameButton.clicked()
    }

    function releaseYear() {
        return currentReleaseDate.length >= 4
            ? currentReleaseDate.substring(0, 4) : ""
    }

    function formatPlayTime() {
        const hours = Math.floor(
            Math.max(0, currentPlayTimeSeconds) / 3600)
        return hours > 0 ? "PLAYED " + hours + "H" : ""
    }

    function metadataLine() {
        const values = []
        const year = releaseYear()
        if (year.length > 0)
            values.push(year)
        if (currentGenre.length > 0)
            values.push(currentGenre.toUpperCase())
        if (currentStarRating > 0)
            values.push("★ " + currentStarRating.toFixed(1))
        return values.join("  •  ")
    }

    onBlockedChanged: {
        if (blocked && active)
            stopMode("blocked")
        else if (!blocked && automaticEligible)
            idleTimer.restart()
    }

    onPresentationViewChanged: {
        if (active) {
            markPresentedView()
            configureVideo()
        }
    }

    Timer {
        id: idleTimer
        interval:
            Math.max(1,
                root.controller.big_box_screensaver_delay_seconds) * 1000
        repeat: false
        running: root.automaticEligible
        onRunningChanged: {
            if (running)
                root.idleCountdownStartedAt = Date.now()
        }
        onTriggered: root.startAutomatic()
    }

    Timer {
        id: swapTimer
        repeat: false
        onTriggered: root.swapCandidate()
    }

    Timer {
        id: mediaRevealTimer
        interval: 1200
        repeat: false
        onTriggered: root.mediaRevealed = true
    }

    AudioOutput {
        id: screensaverAudio
        muted: root.mutedForSmoke
        volume:
            Math.max(0, Math.min(
                1,
                root.controller.big_box_screensaver_video_volume_percent
                * root.controller.big_box_screensaver_master_volume_percent
                / 10000))
    }

    MediaPlayer {
        id: screensaverPlayer
        audioOutput: screensaverAudio
        videoOutput: screensaverVideo

        onPlaybackStateChanged: {
            if (playbackState === MediaPlayer.PlayingState)
                root.videoPlaybackSeen = true
        }

        onMediaStatusChanged: {
            if ((mediaStatus === MediaPlayer.LoadedMedia
                    || mediaStatus === MediaPlayer.BufferedMedia)
                    && root.active && root.videoPresentation)
                play()
        }

        onErrorOccurred: function(error, errorString) {
            console.warn("BigBox screensaver video error "
                         + error + ": " + errorString)
        }
    }

    HoverHandler {
        enabled: !root.active
        onPointChanged: root.noteActivity()
    }

    FocusScope {
        id: inputLayer
        anchors.fill: parent
        visible: root.active
        focus: visible

        Keys.priority: Keys.BeforeItem
        Keys.onPressed: function(event) {
            if (event.key === Qt.Key_Return
                    || event.key === Qt.Key_Enter
                    || event.key === Qt.Key_Space)
                root.exploreCurrentGame()
            else
                root.stopMode("input")
            event.accepted = true
        }

        Rectangle {
            anchors.fill: parent
            color: "#05070b"
        }

        Image {
            id: backgroundImage
            anchors.fill: parent
            source: root.currentBackgroundUrl
            fillMode: Image.PreserveAspectCrop
            asynchronous: true
            cache: false
            opacity: root.presentationOrdinal === 2
                     && root.videoReady ? 0.16 : 0.82
        }

        Rectangle {
            anchors.fill: parent
            color: "#58000000"
        }

        VideoOutput {
            id: screensaverVideo
            visible:
                root.videoPresentation
                && root.currentVideoUrl.toString().length > 0
                && (root.presentationOrdinal === 3
                    || root.mediaRevealed)
            x: root.presentationOrdinal === 2 ? 0
               : root.presentationOrdinal === 3
                 ? Math.round(parent.width * 0.50)
                 : Math.round(parent.width * 0.27)
            y: root.presentationOrdinal === 2 ? 0
               : root.presentationOrdinal === 3
                 ? Math.round(parent.height * 0.16)
                 : Math.round(parent.height * 0.17)
            width: root.presentationOrdinal === 2
                   ? parent.width
                   : root.presentationOrdinal === 3
                     ? Math.round(parent.width * 0.45)
                     : Math.round(parent.width * 0.46)
            height: root.presentationOrdinal === 2
                    ? parent.height
                    : root.presentationOrdinal === 3
                      ? Math.round(parent.height * 0.55)
                      : Math.round(parent.height * 0.43)
            fillMode: VideoOutput.PreserveAspectFit
        }

        Image {
            id: screenshotImage
            visible:
                (root.presentationOrdinal === 3
                 || root.presentationOrdinal === 4)
                && (root.presentationOrdinal === 3
                    || root.mediaRevealed)
                && !screensaverVideo.visible
            x: screensaverVideo.x
            y: screensaverVideo.y
            width: screensaverVideo.width
            height: screensaverVideo.height
            source: root.currentScreenshotUrl
            fillMode: Image.PreserveAspectFit
            asynchronous: true
            cache: false
        }

        Rectangle {
            visible:
                root.presentationOrdinal >= 3
                && (screensaverVideo.visible
                    || screenshotImage.visible)
            x: screensaverVideo.x - 2
            y: screensaverVideo.y - 2
            width: screensaverVideo.width + 4
            height: screensaverVideo.height + 4
            color: "transparent"
            border.color: "#88c5dcff"
            border.width: 2
            radius: 8
        }

        Image {
            id: boxArtImage
            visible:
                root.presentationOrdinal === 3
                || (root.presentationOrdinal === 4
                    && !root.mediaRevealed)
            x: root.presentationOrdinal === 3
               ? Math.round(parent.width * 0.09)
               : Math.round(parent.width * 0.34)
            y: Math.round(parent.height * 0.15)
            width: root.presentationOrdinal === 3
                   ? Math.round(parent.width * 0.31)
                   : Math.round(parent.width * 0.32)
            height: Math.round(parent.height * 0.57)
            source: root.currentBoxArtUrl
            fillMode: Image.PreserveAspectFit
            asynchronous: true
            cache: false
        }

        Rectangle {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            height:
                root.presentationOrdinal === 4
                ? Math.round(parent.height * 0.30)
                : Math.round(parent.height * 0.34)
            gradient: Gradient {
                GradientStop { position: 0; color: "#00101622" }
                GradientStop { position: 0.28; color: "#cc0b111b" }
                GradientStop { position: 1; color: "#fa070a10" }
            }
        }

        Column {
            id: identityColumn
            x: root.presentationOrdinal === 4
               ? Math.round((parent.width - width) / 2)
               : Math.round(parent.width * 0.065)
            y: root.presentationOrdinal === 4
               ? Math.round(parent.height * 0.72)
               : Math.round(parent.height * 0.69)
            width: root.presentationOrdinal === 4
                   ? Math.round(parent.width * 0.76)
                   : Math.round(parent.width * 0.70)
            spacing: 8

            Label {
                width: parent.width
                text: root.currentTitle
                color: "white"
                font.pixelSize:
                    root.presentationOrdinal === 4 ? 44 : 50
                font.bold: true
                elide: Text.ElideRight
                horizontalAlignment:
                    root.presentationOrdinal === 4
                    ? Text.AlignHCenter : Text.AlignLeft
            }

            Label {
                width: parent.width
                text: root.metadataLine()
                color: "#b8c8db"
                font.pixelSize: 20
                font.bold: true
                elide: Text.ElideRight
                horizontalAlignment:
                    root.presentationOrdinal === 4
                    ? Text.AlignHCenter : Text.AlignLeft
            }

            Label {
                width: parent.width
                text:
                    (root.currentPlatform.length > 0
                     ? root.currentPlatform.toUpperCase() : "")
                    + (root.formatPlayTime().length > 0
                       ? "  •  " + root.formatPlayTime() : "")
                color: "#69b7ff"
                font.pixelSize: 16
                font.bold: true
                elide: Text.ElideRight
                horizontalAlignment:
                    root.presentationOrdinal === 4
                    ? Text.AlignHCenter : Text.AlignLeft
            }
        }

        Rectangle {
            anchors.top: parent.top
            anchors.right: parent.right
            anchors.margins: 28
            width: promptRow.implicitWidth + 34
            height: promptRow.implicitHeight + 22
            radius: 10
            color: "#d70d1520"
            border.color: "#506c87a6"

            RowLayout {
                id: promptRow
                anchors.centerIn: parent
                spacing: 12

                Label {
                    text: "PRESS ENTER TO EXPLORE"
                    color: "#d8e5f3"
                    font.pixelSize: 14
                    font.bold: true
                }

                Label {
                    text: "•"
                    color: "#6f87a2"
                }

                Label {
                    text:
                        "VIEW " + root.presentationOrdinal
                        + "  •  NEXT "
                        + Math.round(swapTimer.interval / 1000) + "S"
                    color: "#75bfff"
                    font.pixelSize: 13
                    font.bold: true
                }
            }
        }

        RowLayout {
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.rightMargin: 28
            anchors.bottomMargin: 24
            spacing: 10

            Button {
                id: viewGameButton
                text: "VIEW GAME"
                Accessible.name: "View screensaver game"
                onClicked: root.exploreCurrentGame()
            }

            Button {
                id: returnButton
                text: "RETURN TO BIGBOX"
                Accessible.name: "Exit screensaver"
                onClicked: root.stopMode("input")
            }
        }

        MouseArea {
            anchors.fill: parent
            z: -1
            acceptedButtons: Qt.AllButtons
            onPressed: function(mouse) {
                root.stopMode("input")
                mouse.accepted = true
            }
            onWheel: function(wheel) {
                root.stopMode("input")
                wheel.accepted = true
            }
        }

        SequentialAnimation {
            id: presentationFade
            NumberAnimation {
                target: inputLayer
                property: "opacity"
                to: 0.25
                duration: 80
            }
            NumberAnimation {
                target: inputLayer
                property: "opacity"
                to: 1
                duration: 520
                easing.type: Easing.OutCubic
            }
        }
    }
}
