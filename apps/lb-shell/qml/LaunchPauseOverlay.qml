import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: overlay

    required property var controller

    visible: controller.pause_screen_active
    enabled: visible
    focus: visible
    z: 10002

    onVisibleChanged: {
        if (visible)
            forceActiveFocus()
    }

    Rectangle {
        anchors.fill: parent
        color: "#f00a0d14"

        MouseArea {
            anchors.fill: parent
            acceptedButtons: Qt.AllButtons
            onPressed: function(mouse) {
                mouse.accepted = true
            }
        }
    }

    Rectangle {
        anchors.centerIn: parent
        width: Math.min(parent.width - 48, 760)
        height: Math.min(parent.height - 48, 430)
        radius: 24
        color: "#fa18243a"
        border.width: 2
        border.color: "#d5aa68"

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 42
            spacing: 18

            Item { Layout.fillHeight: true }

            Label {
                Layout.alignment: Qt.AlignHCenter
                text: "GAME PAUSED"
                color: "#f2c57c"
                font.pixelSize: 20
                font.bold: true
                font.letterSpacing: 4
            }

            Label {
                Layout.fillWidth: true
                text: controller.pause_screen_game_title
                color: "white"
                font.pixelSize: 38
                font.bold: true
                horizontalAlignment: Text.AlignHCenter
                elide: Text.ElideRight
            }

            Label {
                Layout.alignment: Qt.AlignHCenter
                text: controller.pause_screen_process_suspended
                      ? "Game process suspended"
                      : "Pause screen active"
                color: "#d1d9e7"
                font.pixelSize: 16
            }

            Button {
                Layout.alignment: Qt.AlignHCenter
                text: "RESUME GAME"
                highlighted: true
                onClicked: controller.resume_launch_session()
            }

            Label {
                Layout.alignment: Qt.AlignHCenter
                text: "Theme: " + controller.frontend_pause_theme
                      + "  •  Settings: "
                      + controller.pause_screen_settings_source
                color: "#899bb5"
                font.pixelSize: 13
            }

            Item { Layout.fillHeight: true }
        }
    }

    Keys.onPressed: function(event) {
        if (event.key === Qt.Key_Escape
                || event.key === Qt.Key_Return
                || event.key === Qt.Key_Enter
                || event.key === Qt.Key_Space) {
            controller.resume_launch_session()
        }
        event.accepted = true
    }
}
