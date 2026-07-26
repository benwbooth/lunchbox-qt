pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Dialog {
    id: root

    property string promptText: "Enter your PIN"
    property string editorMessage: ""
    property string pin: ""
    property int selectedIndex: 0
    readonly property int maximumDigits: 32
    property alias smokeCaptureTarget: popupLayout

    signal submitted(string pin)
    signal cancelled()

    title: "BIG BOX PIN"
    modal: true
    focus: true
    popupType: Popup.Item
    closePolicy: Popup.NoAutoClose
    anchors.centerIn: parent
    width: Math.min(500, parent ? parent.width - 80 : 500)
    height: Math.min(650, parent ? parent.height - 80 : 650)
    standardButtons: Dialog.NoButton

    function openForPrompt(prompt, message) {
        promptText = prompt
        editorMessage = message === undefined ? "" : message
        pin = ""
        selectedIndex = 0
        open()
        keypadScope.forceActiveFocus()
    }

    function digitIndex(digit) {
        return ["7", "8", "9", "4", "5", "6",
                "1", "2", "3", "0"].indexOf(digit)
    }

    function appendDigit(digit) {
        if (pin.length >= maximumDigits) {
            editorMessage =
                "A BigBox PIN can contain at most 32 digits."
            return false
        }
        pin += digit
        editorMessage = ""
        return true
    }

    function deleteDigit() {
        if (pin.length === 0)
            return false
        pin = pin.slice(0, -1)
        editorMessage = ""
        return true
    }

    function submitPin() {
        if (pin.length === 0) {
            editorMessage = "Enter at least one digit."
            return false
        }
        const submittedPin = pin
        pin = ""
        close()
        submitted(submittedPin)
        return true
    }

    function cancelEntry() {
        pin = ""
        close()
        cancelled()
        return true
    }

    function activateIndex(index) {
        if (index >= 0 && index <= 9)
            return appendDigit(keypadModel[index])
        if (index === 10)
            return deleteDigit()
        if (index === 11)
            return submitPin()
        return false
    }

    function moveSelection(horizontal, vertical) {
        const column = selectedIndex % 3
        const row = Math.floor(selectedIndex / 3)
        let nextColumn = (column + horizontal) % 3
        let nextRow = (row + vertical) % 4
        if (nextColumn < 0)
            nextColumn += 3
        if (nextRow < 0)
            nextRow += 4
        selectedIndex = nextRow * 3 + nextColumn
        return true
    }

    function handleAction(action) {
        if (!opened)
            return false
        if (action === "BigBoxNavigateLeft")
            return moveSelection(-1, 0)
        if (action === "BigBoxNavigateRight")
            return moveSelection(1, 0)
        if (action === "BigBoxNavigateUp")
            return moveSelection(0, -1)
        if (action === "BigBoxNavigateDown")
            return moveSelection(0, 1)
        if (action === "BigBoxSelect"
                || action === "BigBoxPlayGame")
            return activateIndex(selectedIndex)
        if (action === "BigBoxBack")
            return cancelEntry()
        return true
    }

    function runSmokeEntry(value) {
        if (!opened || value.length === 0)
            return false
        for (let index = 0; index < value.length; ++index) {
            const keypadIndex = digitIndex(value[index])
            if (keypadIndex < 0)
                return false
            selectedIndex = keypadIndex
            if (!activateIndex(selectedIndex))
                return false
        }
        selectedIndex = 11
        return activateIndex(selectedIndex)
    }

    readonly property var keypadModel: [
        "7", "8", "9",
        "4", "5", "6",
        "1", "2", "3",
        "0", "DELETE", "DONE"
    ]

    onOpened: keypadScope.forceActiveFocus()
    onClosed: pin = ""

    contentItem: FocusScope {
        id: keypadScope
        focus: true

        Keys.onPressed: function(event) {
            if (event.key >= Qt.Key_0 && event.key <= Qt.Key_9) {
                root.appendDigit(String(event.key - Qt.Key_0))
                event.accepted = true
                return
            }
            if (event.key === Qt.Key_Backspace
                    || event.key === Qt.Key_Delete) {
                root.deleteDigit()
                event.accepted = true
                return
            }
            if (event.key === Qt.Key_Return
                    || event.key === Qt.Key_Enter) {
                root.submitPin()
                event.accepted = true
                return
            }
            if (event.key === Qt.Key_Escape) {
                root.cancelEntry()
                event.accepted = true
                return
            }
            if (event.key === Qt.Key_Left)
                event.accepted = root.moveSelection(-1, 0)
            else if (event.key === Qt.Key_Right)
                event.accepted = root.moveSelection(1, 0)
            else if (event.key === Qt.Key_Up)
                event.accepted = root.moveSelection(0, -1)
            else if (event.key === Qt.Key_Down)
                event.accepted = root.moveSelection(0, 1)
        }

        ColumnLayout {
            id: popupLayout
            anchors.fill: parent
            spacing: 16

            Label {
                Layout.fillWidth: true
                text: root.promptText
                horizontalAlignment: Text.AlignHCenter
                color: "white"
                font.pixelSize: 24
                font.bold: true
                wrapMode: Text.Wrap
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 64
                radius: 7
                color: "#0a1018"
                border.color: "#4679ad"
                border.width: 2

                Label {
                    anchors.centerIn: parent
                    text: "\u2022".repeat(root.pin.length)
                    color: "#f0c04a"
                    font.pixelSize: 30
                    font.letterSpacing: 5
                    Accessible.name:
                        root.pin.length + " PIN digits entered"
                }
            }

            GridLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                columns: 3
                columnSpacing: 10
                rowSpacing: 10

                Repeater {
                    model: root.keypadModel

                    delegate: Button {
                        id: keypadButton
                        required property int index
                        required property string modelData
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        Layout.minimumHeight: 68
                        text: modelData
                        highlighted: root.selectedIndex === index
                        font.pixelSize: modelData.length === 1 ? 25 : 16
                        Accessible.name:
                            modelData === "DELETE" ? "Delete PIN digit"
                            : modelData === "DONE" ? "Submit PIN"
                            : "PIN digit " + modelData
                        onHoveredChanged: {
                            if (hovered)
                                root.selectedIndex = index
                        }
                        onClicked: {
                            root.selectedIndex = index
                            root.activateIndex(index)
                        }
                    }
                }
            }

            Label {
                Layout.fillWidth: true
                text: root.editorMessage
                color: "#f0c04a"
                horizontalAlignment: Text.AlignHCenter
                wrapMode: Text.Wrap
                font.pixelSize: 15
            }

            Label {
                Layout.fillWidth: true
                text: "NAVIGATE     SELECT     BACK  CANCEL"
                color: "#91a4ba"
                horizontalAlignment: Text.AlignHCenter
                font.pixelSize: 13
            }
        }
    }
}
