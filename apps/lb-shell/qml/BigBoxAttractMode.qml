import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtMultimedia

Item {
    id: root

    required property var controller
    property var advanceWheelCallback
    property var switchFilterCallback
    property var focusReturnCallback
    property bool blocked: false
    property bool mutedForSmoke: false
    property real runtimeMasterVolumeScale: 1.0
    property bool active: false
    property string phase: "idle"
    property string lastStartSource: ""
    property string lastStopReason: ""
    property int wheelStep: 0
    property int totalWheelSteps: 0
    property int movementCycles: 0
    property int filterSwitches: 0
    property int inputStopCount: 0
    property int manualStartCount: 0
    property double idleCountdownStartedAt: 0
    property int lastAutomaticDelayElapsedMs: 0
    property bool moveSoundPending: false
    readonly property bool automaticEligible:
        controller.big_box_attract_mode_enabled
        && controller.library_path.length > 0
        && controller.filtered_count > 0
        && !blocked
        && !active
    readonly property int moveSoundStatus: moveSoundPlayer.mediaStatus
    readonly property bool moveSoundReady:
        controller.indexed_attract_move_sound_count > 0
        && (moveSoundPlayer.mediaStatus === MediaPlayer.LoadedMedia
            || moveSoundPlayer.mediaStatus === MediaPlayer.BufferedMedia)

    function startAutomatic() {
        return startMode("automatic")
    }

    function startManual() {
        return startMode("manual")
    }

    function startMode(source) {
        if (blocked || active || controller.filtered_count <= 0)
            return false
        idleTimer.stop()
        wheelTimer.stop()
        movementPauseTimer.stop()
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
        active = true
        phase = "spinning"
        wheelStep = 0
        prepareMoveSound()
        inputLayer.forceActiveFocus()
        wheelTimer.interval =
            controller.big_box_attract_mode_wheel_interval_ms(0)
        wheelTimer.start()
        return true
    }

    function stopMode(reason) {
        if (!active)
            return false
        wheelTimer.stop()
        movementPauseTimer.stop()
        moveSoundPlayer.stop()
        moveSoundPending = false
        active = false
        phase = "idle"
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

    function prepareMoveSound() {
        moveSoundPending = false
        if (!controller.big_box_play_move_in_attract_mode
                || controller.indexed_attract_move_sound_count <= 0) {
            moveSoundPlayer.source = ""
            return
        }
        const index = movementCycles
                    % controller.indexed_attract_move_sound_count
        const source = controller.attract_move_sound_url_at(index)
        if (moveSoundPlayer.source.toString() !== source.toString()) {
            moveSoundPlayer.stop()
            moveSoundPlayer.source = source
        }
    }

    function playMoveSound() {
        if (!controller.big_box_play_move_in_attract_mode
                || moveSoundPlayer.source.toString().length === 0)
            return
        if (moveSoundReady) {
            moveSoundPlayer.stop()
            moveSoundPlayer.position = 0
            moveSoundPlayer.play()
        } else {
            moveSoundPending = true
        }
    }

    function advanceWheel() {
        if (!active || blocked) {
            stopMode("blocked")
            return
        }
        let moved = true
        if (advanceWheelCallback)
            moved = advanceWheelCallback()
        if (moved) {
            totalWheelSteps += 1
            playMoveSound()
        }
        wheelStep += 1
        if (wheelStep >= controller.big_box_attract_mode_wheel_step_count()) {
            wheelTimer.stop()
            movementCycles += 1
            if (controller.big_box_attract_mode_switch_filters
                    && switchFilterCallback
                    && switchFilterCallback())
                filterSwitches += 1
            phase = "resting"
            prepareMoveSound()
            movementPauseTimer.restart()
            return
        }
        wheelTimer.interval =
            controller.big_box_attract_mode_wheel_interval_ms(wheelStep)
    }

    function clickReturnForSmoke() {
        returnButton.clicked()
    }

    onBlockedChanged: {
        if (blocked && active)
            stopMode("blocked")
        else if (!blocked && automaticEligible)
            idleTimer.restart()
    }

    Timer {
        id: idleTimer
        interval:
            Math.max(1,
                root.controller.big_box_attract_mode_delay_seconds) * 1000
        repeat: false
        running: root.automaticEligible
        onRunningChanged: {
            if (running)
                root.idleCountdownStartedAt = Date.now()
        }
        onTriggered: root.startAutomatic()
    }

    Timer {
        id: wheelTimer
        repeat: true
        onTriggered: root.advanceWheel()
    }

    Timer {
        id: movementPauseTimer
        interval:
            Math.max(1,
                root.controller
                .big_box_attract_mode_time_per_movement_seconds) * 1000
        repeat: false
        onTriggered: {
            if (!root.active || root.blocked) {
                root.stopMode("blocked")
                return
            }
            root.phase = "spinning"
            root.wheelStep = 0
            wheelTimer.interval =
                root.controller.big_box_attract_mode_wheel_interval_ms(0)
            wheelTimer.start()
        }
    }

    AudioOutput {
        id: moveSoundAudio
        muted: root.mutedForSmoke
        volume:
            Math.max(0, Math.min(
                1,
                root.controller
                    .big_box_attract_mode_navigation_sound_volume_percent
                * root.controller
                  .big_box_attract_mode_master_volume_percent
                * root.runtimeMasterVolumeScale
                / 10000))
    }

    MediaPlayer {
        id: moveSoundPlayer
        audioOutput: moveSoundAudio

        onMediaStatusChanged: {
            if ((mediaStatus === MediaPlayer.LoadedMedia
                    || mediaStatus === MediaPlayer.BufferedMedia)
                    && root.moveSoundPending
                    && root.active) {
                root.moveSoundPending = false
                play()
            } else if (mediaStatus === MediaPlayer.InvalidMedia) {
                root.moveSoundPending = false
            }
        }

        onErrorOccurred: function(error, errorString) {
            console.warn("BigBox Attract Mode move sound error "
                         + error + ": " + errorString)
            root.moveSoundPending = false
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
            root.stopMode("input")
            event.accepted = true
        }

        MouseArea {
            anchors.fill: parent
            hoverEnabled: false
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

        Rectangle {
            anchors.top: parent.top
            anchors.right: parent.right
            anchors.topMargin: 32
            anchors.rightMargin: 34
            width: Math.min(430, parent.width - 68)
            height: attractContent.implicitHeight + 34
            radius: 12
            color: "#e6111823"
            border.color: "#67b3ff"
            border.width: 2

            ColumnLayout {
                id: attractContent
                anchors.fill: parent
                anchors.margins: 17
                spacing: 6

                Label {
                    Layout.fillWidth: true
                    text: "ATTRACT MODE"
                    color: "#67b3ff"
                    font.pixelSize: 23
                    font.bold: true
                    font.letterSpacing: 2
                }

                Label {
                    Layout.fillWidth: true
                    text:
                        (root.phase === "spinning"
                         ? "SPINNING THE WHEEL"
                         : "NEXT MOVEMENT IN "
                           + root.controller
                             .big_box_attract_mode_time_per_movement_seconds
                           + "S")
                        + "  •  " + root.movementCycles
                        + " MOVEMENTS"
                    color: "#c4d3e7"
                    font.pixelSize: 14
                    font.bold: true
                }

                Label {
                    Layout.fillWidth: true
                    text: "PRESS ANY KEY OR BUTTON TO RETURN"
                    color: "#8fa4bf"
                    font.pixelSize: 13
                }

                Button {
                    id: returnButton
                    Layout.alignment: Qt.AlignRight
                    text: "RETURN TO BIGBOX"
                    Accessible.name: "Exit Attract Mode"
                    onClicked: root.stopMode("input")
                }
            }
        }
    }
}
