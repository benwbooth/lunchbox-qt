pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Dialog {
    id: root

    property var controller
    property string gameId: ""
    property string gameTitle: ""
    property var sections: []
    property int tabIndex: 0
    property int currentIndex: -1
    property bool busy: false
    property string loadError: ""
    property alias smokeCaptureTarget: popupLayout

    signal selected(string gameId)
    signal cancelled()

    title: "RELATED GAMES"
    modal: true
    focus: true
    popupType: Popup.Item
    closePolicy: Popup.NoAutoClose
    anchors.centerIn: parent
    width: Math.min(1120, parent ? parent.width * 0.9 : 1120)
    height: Math.min(780, parent ? parent.height * 0.9 : 780)
    standardButtons: Dialog.NoButton

    readonly property var activeItems:
        sections.length > tabIndex && sections[tabIndex] !== null
        && Array.isArray(sections[tabIndex].items)
        ? sections[tabIndex].items : []

    function emptySections() {
        return [
            { key: "recommended", label: "Recommended Games", items: [] },
            { key: "similar", label: "Similar Games", items: [] },
            { key: "possiblePorts", label: "Possible Ports", items: [] }
        ]
    }

    function openForGame(id, title) {
        if (id.length === 0)
            return false
        gameId = id
        gameTitle = title
        sections = emptySections()
        tabIndex = 0
        currentIndex = -1
        loadError = ""
        open()
        popupFocus.forceActiveFocus()
        return true
    }

    function applyPayload(payloadJson) {
        if (!opened || payloadJson.length === 0)
            return false
        let payload
        try {
            payload = JSON.parse(payloadJson)
        } catch (error) {
            return false
        }
        if (payload === null || payload.version !== 1
                || payload.gameId !== gameId
                || !Array.isArray(payload.sections)
                || payload.sections.length !== 3)
            return false
        const expectedKeys = ["recommended", "similar", "possiblePorts"]
        const profileSources = [
            "persistedLaunchBoxSettings",
            "recoveredLaunchBoxDefault",
            "portReconstruction"
        ]
        const validated = []
        for (let index = 0; index < expectedKeys.length; ++index) {
            const section = payload.sections[index]
            if (section === null || section.key !== expectedKeys[index]
                    || typeof section.label !== "string"
                    || profileSources.indexOf(section.profileSource) < 0
                    || !Array.isArray(section.items)
                    || section.items.length > 10)
                return false
            const items = []
            for (let itemIndex = 0;
                 itemIndex < section.items.length; ++itemIndex) {
                const item = section.items[itemIndex]
                if (item === null || typeof item.title !== "string"
                        || item.title.length === 0
                        || typeof item.platform !== "string"
                        || typeof item.scorePercent !== "number"
                        || (item.source !== "local"
                            && item.source !== "database")
                        || (item.source === "local"
                            && (typeof item.localGameId !== "string"
                                || item.localGameId.length === 0))
                        || (item.source === "database"
                            && item.localGameId !== null))
                    return false
                items.push(item)
            }
            validated.push({
                key: section.key,
                label: section.label,
                profileSource: section.profileSource,
                items: items
            })
        }
        sections = validated
        currentIndex = activeItems.length > 0 ? 0 : -1
        loadError = ""
        relatedList.positionViewAtBeginning()
        return true
    }

    function applyControllerPayload() {
        if (!opened || controller === null
                || controller.big_box_related_games_loading)
            return false
        if (controller.big_box_related_games_json.length === 0
                || !applyPayload(
                    controller.big_box_related_games_json)) {
            failLoad(controller.status_message)
            return false
        }
        return true
    }

    function failLoad(message) {
        if (!opened)
            return false
        loadError = message
        currentIndex = -1
        return true
    }

    function switchTab(delta) {
        if (busy)
            return false
        let next = (tabIndex + delta) % sections.length
        if (next < 0)
            next += sections.length
        tabIndex = next
        currentIndex = activeItems.length > 0 ? 0 : -1
        relatedList.positionViewAtBeginning()
        return true
    }

    function moveSelection(delta) {
        if (busy || activeItems.length === 0)
            return false
        let next = (currentIndex + delta) % activeItems.length
        if (next < 0)
            next += activeItems.length
        currentIndex = next
        relatedList.positionViewAtIndex(currentIndex, ListView.Contain)
        return true
    }

    function movePage(delta) {
        if (busy || activeItems.length === 0)
            return false
        currentIndex = Math.max(
            0, Math.min(activeItems.length - 1,
                        currentIndex + delta * 3))
        relatedList.positionViewAtIndex(currentIndex, ListView.Contain)
        return true
    }

    function chooseIndex(index) {
        if (busy || index < 0 || index >= activeItems.length)
            return false
        const item = activeItems[index]
        if (item.source !== "local"
                || typeof item.localGameId !== "string"
                || item.localGameId.length === 0)
            return false
        const selectedId = item.localGameId
        close()
        selected(selectedId)
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
        if (action === "BigBoxNavigateLeft")
            return switchTab(-1)
        if (action === "BigBoxNavigateRight")
            return switchTab(1)
        if (action === "BigBoxNavigateUp")
            return moveSelection(-1)
        if (action === "BigBoxNavigateDown")
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

    onOpened: popupFocus.forceActiveFocus()
    onBusyChanged: {
        if (!busy)
            applyControllerPayload()
    }
    onClosed: {
        sections = []
        currentIndex = -1
        loadError = ""
    }

    Timer {
        // CXX-Qt publishes the payload and ready flag independently. Keep the
        // modal's pending state self-healing if those queued notifications
        // are coalesced while the render thread is busy.
        interval: 50
        repeat: true
        running: root.opened && root.loadError.length === 0
                 && root.sections.length === 3
                 && typeof root.sections[0].profileSource !== "string"
        onTriggered: root.applyControllerPayload()
    }

    background: Rectangle {
        color: "#d9000000"
        border.color: "white"
        border.width: 3
    }

    contentItem: FocusScope {
        id: popupFocus
        focus: true

        Keys.onPressed: function(event) {
            if (event.key === Qt.Key_Left)
                event.accepted = root.switchTab(-1)
            else if (event.key === Qt.Key_Right)
                event.accepted = root.switchTab(1)
            else if (event.key === Qt.Key_Up)
                event.accepted = root.moveSelection(-1)
            else if (event.key === Qt.Key_Down)
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
            spacing: 0

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 70
                color: "white"

                Label {
                    anchors.fill: parent
                    anchors.leftMargin: 24
                    anchors.rightMargin: 24
                    text: root.title + " — " + root.gameTitle
                    color: "black"
                    font.pixelSize: 28
                    font.bold: true
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                    elide: Text.ElideRight
                }
            }

            RowLayout {
                Layout.fillWidth: true
                Layout.preferredHeight: 70
                Layout.leftMargin: 20
                Layout.rightMargin: 20
                spacing: 8

                Repeater {
                    model: root.sections

                    Button {
                        required property int index
                        required property var modelData
                        Layout.fillWidth: true
                        text: modelData.label
                              + (Array.isArray(modelData.items)
                                 ? " (" + modelData.items.length + ")" : "")
                        highlighted: root.tabIndex === index
                        onClicked: {
                            root.tabIndex = index
                            root.currentIndex =
                                root.activeItems.length > 0 ? 0 : -1
                            relatedList.positionViewAtBeginning()
                            popupFocus.forceActiveFocus()
                        }
                    }
                }
            }

            Item {
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.leftMargin: 20
                Layout.rightMargin: 20
                Layout.bottomMargin: 20

                BusyIndicator {
                    anchors.centerIn: parent
                    running: root.busy
                    visible: running
                }

                Label {
                    anchors.centerIn: parent
                    width: Math.min(parent.width - 80, 720)
                    visible: !root.busy
                             && (root.loadError.length > 0
                                 || root.activeItems.length === 0)
                    text: root.loadError.length > 0
                          ? root.loadError
                          : "No games found for this section."
                    color: root.loadError.length > 0 ? "#ff9b9b" : "white"
                    font.pixelSize: 24
                    horizontalAlignment: Text.AlignHCenter
                    wrapMode: Text.Wrap
                }

                ListView {
                    id: relatedList
                    anchors.fill: parent
                    visible: !root.busy && root.activeItems.length > 0
                    clip: true
                    spacing: 8
                    focus: true
                    keyNavigationWraps: true
                    model: root.activeItems
                    currentIndex: root.currentIndex

                    delegate: Rectangle {
                        id: relatedDelegate
                        required property int index
                        required property var modelData
                        width: ListView.view.width
                        height: 170
                        opacity: modelData.source === "local" ? 1.0 : 0.58
                        color: root.currentIndex === index
                               ? "#ff3399ff" : "#b3121212"
                        border.color: root.currentIndex === index
                                      ? "white" : "#555555"
                        border.width: root.currentIndex === index ? 2 : 1

                        RowLayout {
                            anchors.fill: parent
                            anchors.margins: 10
                            spacing: 18

                            Rectangle {
                                Layout.preferredWidth: 130
                                Layout.fillHeight: true
                                color: "#1c1c1c"

                                Image {
                                    anchors.fill: parent
                                    anchors.margins: 4
                                    source:
                                        relatedDelegate.modelData.source
                                        === "local"
                                        && root.controller
                                        ? root.controller
                                          .game_box_front_url_for_game(
                                              relatedDelegate
                                              .modelData.localGameId)
                                        : ""
                                    fillMode: Image.PreserveAspectFit
                                    asynchronous: true
                                    cache: true
                                }

                                Label {
                                    anchors.centerIn: parent
                                    visible:
                                        relatedDelegate.modelData.source
                                        !== "local"
                                    text: "☁"
                                    color: "white"
                                    font.pixelSize: 54
                                    Accessible.name:
                                        "Available from the LaunchBox metadata database"
                                }
                            }

                            ColumnLayout {
                                Layout.fillWidth: true
                                Layout.fillHeight: true
                                spacing: 6

                                Label {
                                    Layout.fillWidth: true
                                    text: relatedDelegate.modelData.title
                                    color: "white"
                                    font.pixelSize: 26
                                    font.bold: true
                                    elide: Text.ElideRight
                                }

                                Label {
                                    Layout.fillWidth: true
                                    text:
                                        (relatedDelegate.modelData.releaseYear
                                         ? relatedDelegate
                                           .modelData.releaseYear + " • " : "")
                                        + relatedDelegate.modelData.platform
                                        + (relatedDelegate.modelData.source
                                           === "local"
                                           ? " • INSTALLED"
                                           : " • DATABASE")
                                    color: "#d5e1ef"
                                    font.pixelSize: 18
                                    font.bold: true
                                    elide: Text.ElideRight
                                }

                                Label {
                                    Layout.fillWidth: true
                                    Layout.fillHeight: true
                                    text: relatedDelegate.modelData.notes || ""
                                    color: "#e3e3e3"
                                    font.pixelSize: 16
                                    wrapMode: Text.Wrap
                                    elide: Text.ElideRight
                                    maximumLineCount: 3
                                }
                            }

                            Label {
                                Layout.preferredWidth: 110
                                text: relatedDelegate.modelData.scorePercent
                                      + "%"
                                color: "white"
                                font.pixelSize: 30
                                font.bold: true
                                horizontalAlignment: Text.AlignHCenter
                            }
                        }

                        MouseArea {
                            anchors.fill: parent
                            onClicked: {
                                root.currentIndex = relatedDelegate.index
                                popupFocus.forceActiveFocus()
                            }
                            onDoubleClicked:
                                root.chooseIndex(relatedDelegate.index)
                        }
                    }
                }
            }
        }
    }
}
