pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Dialog {
    id: root

    required property var controller
    property bool pendingSave: false
    property int saveStartRevision: -1
    property string editorMessage: ""
    property string pinChange: "keep"
    property string pendingPin: ""
    property string firstPin: ""
    property string pinPurpose: ""
    property alias smokeCaptureTarget: editorLayout
    property alias pinPopup: setPinPopup

    title: "BIG BOX SECURITY"
    modal: true
    focus: true
    popupType: Popup.Item
    closePolicy: Popup.NoAutoClose
    anchors.centerIn: parent
    width: Math.min(880, parent ? parent.width - 80 : 880)
    height: Math.min(760, parent ? parent.height - 80 : 760)
    standardButtons: Dialog.NoButton

    function reloadFromController() {
        permissionModel.clear()
        for (let index = 0;
             index < controller.big_box_security_permission_count();
             ++index) {
            permissionModel.append({
                key: controller.big_box_security_permission_key_at(index),
                label:
                    controller.big_box_security_permission_label_at(index),
                allowed:
                    controller.big_box_security_permission_allowed_at(index)
            })
        }
        showLockUnlockCheck.checked =
            controller.big_box_show_game_lock_unlock
        pinChange = "keep"
        pendingPin = ""
        firstPin = ""
        pinPurpose = ""
        pendingSave = false
        editorMessage = ""
        permissionList.currentIndex = permissionModel.count > 0 ? 0 : -1
        permissionList.forceActiveFocus()
    }

    function openEditor() {
        if (controller.big_box_locked) {
            controller.note_big_box_locked_action("BigBoxLockUnlock")
            return false
        }
        open()
        return true
    }

    function beginSetPin() {
        if (pendingSave)
            return false
        firstPin = ""
        pinPurpose = "first"
        setPinPopup.openForPrompt(
            controller.big_box_pin_configured
                ? "Enter your new PIN" : "Enter your PIN",
            "Use the keypad, keyboard digits, or a controller.")
        return true
    }

    function clearPin() {
        if (pendingSave)
            return false
        pinChange = "clear"
        pendingPin = ""
        firstPin = ""
        editorMessage =
            "The PIN will be cleared when these settings are saved."
        return true
    }

    function permissionIndex(settingKey) {
        for (let index = 0; index < permissionModel.count; ++index) {
            if (permissionModel.get(index).key === settingKey)
                return index
        }
        return -1
    }

    function setPermission(settingKey, allowed) {
        const index = permissionIndex(settingKey)
        if (index < 0)
            return false
        permissionModel.setProperty(index, "allowed", allowed)
        permissionList.currentIndex = index
        return true
    }

    function saveChanges() {
        if (permissionModel.count
                !== controller.big_box_security_permission_count()) {
            editorMessage =
                "The recovered permission catalog is incomplete."
            return false
        }
        const permissions = []
        for (let index = 0; index < permissionModel.count; ++index) {
            const permission = permissionModel.get(index)
            permissions.push({
                key: permission.key,
                allowed: permission.allowed
            })
        }
        const payload = {
            version: 1,
            pinChange: pinChange,
            pin: pinChange === "set" ? pendingPin : "",
            showGameLockUnlock: showLockUnlockCheck.checked,
            permissions: permissions
        }
        saveStartRevision =
            controller.big_box_security_settings_revision
        if (!controller.save_big_box_security_settings(
                JSON.stringify(payload))) {
            editorMessage = controller.status_message
            return false
        }
        pendingSave = true
        editorMessage =
            "Saving one atomic BigBox settings transaction…"
        return true
    }

    function movePermission(offset) {
        if (permissionModel.count === 0)
            return false
        let index = (permissionList.currentIndex + offset)
                    % permissionModel.count
        if (index < 0)
            index += permissionModel.count
        permissionList.currentIndex = index
        permissionList.positionViewAtIndex(index, ListView.Contain)
        return true
    }

    function handleAction(action) {
        if (setPinPopup.opened)
            return setPinPopup.handleAction(action)
        if (!opened)
            return false
        if (action === "BigBoxBack") {
            if (!pendingSave)
                close()
            return true
        }
        if (action === "BigBoxNavigateUp")
            return movePermission(-1)
        if (action === "BigBoxNavigateDown")
            return movePermission(1)
        if (action === "BigBoxPageUp")
            return movePermission(-7)
        if (action === "BigBoxPageDown")
            return movePermission(7)
        if (action === "BigBoxSelect"
                && permissionList.currentIndex >= 0) {
            const index = permissionList.currentIndex
            permissionModel.setProperty(
                index, "allowed",
                !permissionModel.get(index).allowed)
            return true
        }
        return true
    }

    onOpened: reloadFromController()
    onClosed: {
        pendingPin = ""
        firstPin = ""
        pinPurpose = ""
    }

    Connections {
        target: root.controller

        function onWritingChanged() {
            if (!root.pendingSave || root.controller.writing)
                return
            Qt.callLater(function() {
                if (!root.pendingSave)
                    return
                root.pendingSave = false
                root.pendingPin = ""
                root.firstPin = ""
                if (root.controller.big_box_security_settings_revision
                        !== root.saveStartRevision) {
                    root.close()
                } else {
                    root.editorMessage =
                        root.controller.status_message
                }
            })
        }
    }

    ListModel {
        id: permissionModel
    }

    BigBoxPinPopup {
        id: setPinPopup

        onSubmitted: function(pin) {
            if (root.pinPurpose === "first") {
                root.firstPin = pin
                root.pinPurpose = "repeat"
                Qt.callLater(function() {
                    setPinPopup.openForPrompt(
                        "Repeat your PIN",
                        "Enter the same digits again.")
                })
                return
            }
            if (root.pinPurpose !== "repeat") {
                root.firstPin = ""
                root.pinPurpose = ""
                return
            }
            if (pin !== root.firstPin) {
                root.firstPin = ""
                root.pinPurpose = ""
                root.editorMessage =
                    "The two PINs did not match. The PIN was not changed."
                return
            }
            root.pendingPin = pin
            root.firstPin = ""
            root.pinPurpose = ""
            root.pinChange = "set"
            root.editorMessage =
                "The new PIN will take effect when these settings are saved."
        }

        onCancelled: {
            root.firstPin = ""
            root.pinPurpose = ""
        }
    }

    contentItem: ColumnLayout {
        id: editorLayout
        spacing: 12

        Label {
            Layout.fillWidth: true
            text: "Locked mode starts automatically when BigBox opens "
                  + "with a PIN. Game launching and navigation remain "
                  + "available according to the LaunchBox 13.27 policy below."
            color: "#b9c8da"
            wrapMode: Text.Wrap
            font.pixelSize: 15
        }

        RowLayout {
            Layout.fillWidth: true

            Label {
                Layout.fillWidth: true
                text: root.pinChange === "set"
                      ? "PIN change pending"
                      : root.pinChange === "clear"
                        ? "PIN removal pending"
                        : root.controller.big_box_pin_configured
                          ? "PIN configured"
                          : "No PIN configured"
                color: root.controller.big_box_pin_configured
                       || root.pinChange === "set"
                       ? "#7fd48d" : "#f0c04a"
                font.bold: true
            }
            Button {
                text: root.controller.big_box_pin_configured
                      || root.pinChange === "set"
                      ? "REPLACE PIN" : "SET PIN"
                enabled: !root.pendingSave
                onClicked: root.beginSetPin()
            }
            Button {
                text: "CLEAR PIN"
                enabled: !root.pendingSave
                         && (root.controller.big_box_pin_configured
                             || root.pinChange === "set")
                onClicked: root.clearPin()
            }
        }

        CheckBox {
            id: showLockUnlockCheck
            text: "Show Lock / Unlock in the BigBox game menu"
        }

        Label {
            Layout.fillWidth: true
            text: "ALLOW WHILE LOCKED"
            color: "#67b3ff"
            font.pixelSize: 18
            font.bold: true
            font.letterSpacing: 1
        }

        ListView {
            id: permissionList
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            spacing: 4
            focus: true
            keyNavigationWraps: true
            model: permissionModel

            delegate: CheckDelegate {
                id: permissionDelegate
                required property int index
                required property string key
                required property string label
                required property bool allowed
                width: ListView.view.width
                text: label
                checked: allowed
                highlighted: permissionList.currentIndex === index
                Accessible.description: key
                onClicked: {
                    permissionList.currentIndex = index
                    permissionModel.setProperty(
                        index, "allowed", checked)
                }
            }
        }

        Label {
            Layout.fillWidth: true
            text: root.editorMessage
            color: "#f0c04a"
            wrapMode: Text.Wrap
        }

        RowLayout {
            Layout.fillWidth: true

            Label {
                Layout.fillWidth: true
                text: "32 recovered LaunchBox 13.27 permissions"
                color: "#91a4ba"
            }
            Button {
                text: "CANCEL"
                enabled: !root.pendingSave
                onClicked: root.close()
            }
            Button {
                text: root.pendingSave ? "SAVING…" : "SAVE"
                highlighted: true
                enabled: !root.pendingSave
                         && !root.controller.writing
                onClicked: root.saveChanges()
            }
        }
    }
}
