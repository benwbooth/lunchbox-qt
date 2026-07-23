import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: overlay

    required property var controller

    visible: controller.shutdown_screen_active
    enabled: visible
    focus: visible
    z: 10001

    Timer {
        id: minimumDisplayTimer
        interval: Math.max(
                      1, controller.frontend_minimum_shutdown_screen_ms)
        repeat: false
        onTriggered: controller.dismiss_shutdown_screen()
    }

    onVisibleChanged: {
        if (visible)
            minimumDisplayTimer.restart()
        else
            minimumDisplayTimer.stop()
    }

    Rectangle {
        anchors.fill: parent
        color: "#f207090d"
    }

    Rectangle {
        anchors.centerIn: parent
        width: Math.min(parent.width - 48, 720)
        height: Math.min(parent.height - 48, 340)
        radius: 24
        color: "#f21b2638"
        border.width: 2
        border.color: "#6ca782"

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 42
            spacing: 20

            Item { Layout.fillHeight: true }

            Label {
                Layout.alignment: Qt.AlignHCenter
                text: "SESSION COMPLETE"
                color: "#8dd8aa"
                font.pixelSize: 20
                font.bold: true
                font.letterSpacing: 4
            }

            Label {
                Layout.fillWidth: true
                text: controller.shutdown_screen_game_title
                color: "white"
                font.pixelSize: 38
                font.bold: true
                horizontalAlignment: Text.AlignHCenter
                elide: Text.ElideRight
            }

            Label {
                Layout.alignment: Qt.AlignHCenter
                text: controller.frontend_is_big_box
                      ? "Returning to BigBox"
                      : "Returning to LaunchBox"
                color: "#c3cedd"
                font.pixelSize: 16
            }

            Label {
                Layout.alignment: Qt.AlignHCenter
                text: "Settings: "
                      + controller.shutdown_screen_settings_source
                color: "#7f91aa"
                font.pixelSize: 13
            }

            Item { Layout.fillHeight: true }
        }
    }

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.AllButtons
        cursorShape: controller.frontend_hide_mouse_on_startup_screens
                     ? Qt.BlankCursor : Qt.ArrowCursor
        onPressed: function(mouse) {
            mouse.accepted = true
        }
    }

    Keys.onPressed: function(event) {
        event.accepted = true
    }
}
