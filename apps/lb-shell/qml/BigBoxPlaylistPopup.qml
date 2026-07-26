pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Dialog {
    id: root

    property int modelRow: -1
    property string gameId: ""
    property string gameTitle: ""
    property var targets: []
    property int currentIndex: -1
    property bool busy: false
    property alias smokeCaptureTarget: popupLayout

    signal selected(int row, string gameId, string playlistId)
    signal cancelled()

    title: "ADD TO PLAYLIST"
    modal: true
    focus: true
    popupType: Popup.Item
    closePolicy: Popup.NoAutoClose
    anchors.centerIn: parent
    width: Math.min(820, parent ? parent.width - 80 : 820)
    height: Math.min(680, parent ? parent.height - 80 : 680)
    standardButtons: Dialog.NoButton

    function openForGame(row, id, title, payloadJson) {
        let payload
        try {
            payload = JSON.parse(payloadJson)
        } catch (error) {
            return 0
        }
        if (payload === null
                || payload.version !== 1
                || payload.gameId !== id
                || !Array.isArray(payload.addTargets))
            return 0
        const validated = []
        for (let index = 0; index < payload.addTargets.length; ++index) {
            const target = payload.addTargets[index]
            if (target === null
                    || typeof target.playlistId !== "string"
                    || target.playlistId.length === 0
                    || typeof target.name !== "string"
                    || target.name.length === 0)
                return 0
            validated.push({
                playlistId: target.playlistId,
                name: target.name
            })
        }
        if (validated.length === 0)
            return 0
        modelRow = row
        gameId = id
        gameTitle = title
        targets = validated
        currentIndex = 0
        open()
        listScope.forceActiveFocus()
        playlistList.positionViewAtIndex(0, ListView.Beginning)
        return validated.length
    }

    function moveSelection(delta) {
        if (targets.length === 0)
            return false
        let next = (currentIndex + delta) % targets.length
        if (next < 0)
            next += targets.length
        currentIndex = next
        playlistList.positionViewAtIndex(currentIndex, ListView.Contain)
        return true
    }

    function movePage(delta) {
        if (targets.length === 0)
            return false
        currentIndex = Math.max(
            0, Math.min(targets.length - 1, currentIndex + delta * 6))
        playlistList.positionViewAtIndex(currentIndex, ListView.Contain)
        return true
    }

    function chooseIndex(index) {
        if (busy || modelRow < 0 || gameId.length === 0
                || index < 0 || index >= targets.length)
            return false
        const row = modelRow
        const id = gameId
        const playlistId = targets[index].playlistId
        close()
        selected(row, id, playlistId)
        return true
    }

    function cancelEntry() {
        close()
        cancelled()
        return true
    }

    function handleAction(action) {
        if (!opened)
            return false
        if (action === "BigBoxNavigateUp"
                || action === "BigBoxNavigateLeft")
            return moveSelection(-1)
        if (action === "BigBoxNavigateDown"
                || action === "BigBoxNavigateRight")
            return moveSelection(1)
        if (action === "BigBoxPageUp")
            return movePage(-1)
        if (action === "BigBoxPageDown")
            return movePage(1)
        if (action === "BigBoxSelect"
                || action === "BigBoxPlayGame")
            return chooseIndex(currentIndex)
        if (action === "BigBoxBack")
            return cancelEntry()
        return true
    }

    function runSmokeSelect(index) {
        if (!opened)
            return false
        currentIndex = index
        return chooseIndex(index)
    }

    onOpened: listScope.forceActiveFocus()
    onClosed: {
        targets = []
        currentIndex = -1
    }

    background: Rectangle {
        radius: 12
        color: "#d9000000"
        border.color: "#547ba8"
        border.width: 2
    }

    contentItem: FocusScope {
        id: listScope
        focus: true

        Keys.onPressed: function(event) {
            if (event.key === Qt.Key_Up
                    || event.key === Qt.Key_Left)
                event.accepted = root.moveSelection(-1)
            else if (event.key === Qt.Key_Down
                     || event.key === Qt.Key_Right)
                event.accepted = root.moveSelection(1)
            else if (event.key === Qt.Key_PageUp)
                event.accepted = root.movePage(-1)
            else if (event.key === Qt.Key_PageDown)
                event.accepted = root.movePage(1)
            else if (event.key === Qt.Key_Return
                     || event.key === Qt.Key_Enter)
                event.accepted = root.chooseIndex(root.currentIndex)
            else if (event.key === Qt.Key_Escape)
                event.accepted = root.cancelEntry()
        }

        ColumnLayout {
            id: popupLayout
            anchors.fill: parent
            anchors.margins: 26
            spacing: 16

            Label {
                Layout.fillWidth: true
                text: root.title
                color: "white"
                font.pixelSize: 30
                font.bold: true
                horizontalAlignment: Text.AlignHCenter
            }

            Label {
                Layout.fillWidth: true
                text: root.gameTitle
                color: "#a9bdd6"
                font.pixelSize: 20
                font.bold: true
                horizontalAlignment: Text.AlignHCenter
                elide: Text.ElideRight
            }

            ListView {
                id: playlistList
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                spacing: 6
                focus: true
                keyNavigationWraps: true
                model: root.targets
                currentIndex: root.currentIndex

                delegate: Rectangle {
                    id: targetDelegate
                    required property int index
                    required property var modelData
                    width: ListView.view.width
                    height: 62
                    radius: 5
                    color: root.currentIndex === index
                           ? "#245f9f" : "#151c26"
                    border.color: root.currentIndex === index
                                  ? "#83b9ef" : "#2c3b4d"
                    border.width: root.currentIndex === index ? 2 : 1

                    Label {
                        anchors.fill: parent
                        anchors.leftMargin: 20
                        anchors.rightMargin: 20
                        text: targetDelegate.modelData.name
                        color: "white"
                        font.pixelSize: 22
                        font.bold: root.currentIndex
                                   === targetDelegate.index
                        verticalAlignment: Text.AlignVCenter
                        elide: Text.ElideRight
                    }

                    MouseArea {
                        anchors.fill: parent
                        onClicked: {
                            root.currentIndex = targetDelegate.index
                            listScope.forceActiveFocus()
                        }
                        onDoubleClicked:
                            root.chooseIndex(targetDelegate.index)
                    }
                }

                ScrollBar.vertical: ScrollBar {}
            }

            Label {
                Layout.fillWidth: true
                text: "UP / DOWN  CHOOSE     SELECT  ADD     BACK  CANCEL"
                color: "#7188a3"
                font.pixelSize: 14
                horizontalAlignment: Text.AlignHCenter
            }
        }
    }
}
