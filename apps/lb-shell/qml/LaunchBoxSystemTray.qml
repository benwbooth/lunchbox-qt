import QtQuick
import Qt.labs.platform as Platform

Platform.SystemTrayIcon {
    id: tray

    required property var controller
    property string applicationTitle: "LaunchBox Port"

    signal restoreRequested()
    signal exitRequested()
    signal notificationCenterRequested()
    signal messageBoxRequested(string title, string message)

    visible: controller.desktop_tray_enabled
    tooltip: applicationTitle
    icon.source:
        "qrc:/qt/qml/LaunchBoxPort/qml/launchbox-port-tray.svg"

    menu: Platform.Menu {
        Platform.MenuItem {
            text: "Show LaunchBox"
            onTriggered: tray.restoreRequested()
        }
        Platform.MenuItem {
            text: tray.controller.desktop_unread_notification_count > 0
                  ? "Notifications ("
                    + tray.controller.desktop_unread_notification_count + ")"
                  : "Notifications"
            onTriggered: {
                tray.restoreRequested()
                tray.notificationCenterRequested()
            }
        }
        Platform.MenuSeparator {}
        Platform.MenuItem {
            text: "Exit"
            onTriggered: tray.exitRequested()
        }
    }

    onActivated: function(reason) {
        if (reason === Platform.SystemTrayIcon.Trigger
                || reason === Platform.SystemTrayIcon.DoubleClick)
            restoreRequested()
    }

    function sendWindowToTray(reason) {
        if (!visible || !available)
            return false

        if (controller.desktop_show_sent_to_tray_notification) {
            const message = "LaunchBox has been sent to the system tray"
            controller.raise_desktop_notification(message, false)
            if (controller.desktop_notification_type
                    === "windowsNotifications" && supportsMessages) {
                showMessage(applicationTitle, message,
                            Platform.SystemTrayIcon.Information, 5000)
            } else if (controller.desktop_notification_type
                       === "messageBoxes") {
                messageBoxRequested(applicationTitle, message)
            }
        }
        return true
    }
}
