import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: overlay

    required property var controller

    visible: controller.startup_screen_active
    enabled: visible
    focus: visible
    z: 10000

    Timer {
        interval: Math.max(1, controller.startup_screen_delay_ms)
        running: overlay.visible && controller.startup_screen_primary_started
        repeat: false
        onTriggered: controller.dismiss_startup_screen()
    }

    Rectangle {
        anchors.fill: parent
        color: "#ed07090d"
    }

    Rectangle {
        anchors.centerIn: parent
        width: Math.min(parent.width - 48, 720)
        height: Math.min(parent.height - 48, 360)
        radius: 24
        color: "#f21b2638"
        border.width: 2
        border.color: "#5f86bd"

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 42
            spacing: 20

            Item { Layout.fillHeight: true }

            Label {
                Layout.alignment: Qt.AlignHCenter
                text: "NOW LOADING"
                color: "#8fb9ef"
                font.pixelSize: 20
                font.bold: true
                font.letterSpacing: 4
            }

            Label {
                Layout.fillWidth: true
                text: controller.startup_screen_game_title
                color: "white"
                font.pixelSize: 38
                font.bold: true
                horizontalAlignment: Text.AlignHCenter
                elide: Text.ElideRight
            }

            BusyIndicator {
                Layout.alignment: Qt.AlignHCenter
                running: overlay.visible
            }

            Label {
                Layout.alignment: Qt.AlignHCenter
                text: controller.startup_screen_primary_started
                      ? "Game process started"
                      : "Preparing launch"
                color: "#c3cedd"
                font.pixelSize: 16
            }

            Label {
                Layout.alignment: Qt.AlignHCenter
                text: "Settings: " + controller.startup_screen_settings_source
                color: "#7f91aa"
                font.pixelSize: 13
            }

            Item { Layout.fillHeight: true }
        }
    }

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.AllButtons
        onPressed: function(mouse) {
            mouse.accepted = true
        }
    }

    Keys.onPressed: function(event) {
        event.accepted = true
    }
}
