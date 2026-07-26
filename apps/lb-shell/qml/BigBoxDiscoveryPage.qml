pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Popup {
    id: root

    property var controller
    property var sections: []
    property int sectionIndex: -1
    property int itemIndex: -1
    property string loadError: ""
    property alias smokeCaptureTarget: pageLayout

    signal gameSelected(string gameId)
    signal platformSelected(string platformKey)
    signal cancelled()

    modal: true
    focus: true
    dim: false
    popupType: Popup.Item
    closePolicy: Popup.NoAutoClose
    anchors.centerIn: parent
    width: parent ? parent.width : 1280
    height: parent ? parent.height : 720

    readonly property var activeSection:
        sectionIndex >= 0 && sectionIndex < sections.length
        ? sections[sectionIndex] : null
    readonly property var activeItems:
        activeSection !== null && Array.isArray(activeSection.items)
        ? activeSection.items : []
    readonly property var activeItem:
        itemIndex >= 0 && itemIndex < activeItems.length
        ? activeItems[itemIndex] : null

    function parsePayload(payloadJson) {
        let payload
        try {
            payload = JSON.parse(payloadJson)
        } catch (error) {
            return false
        }
        if (payload === null || payload.version !== 1
                || payload.contractSource
                   !== "launchBox13.27EmbeddedDefaultView"
                || !Array.isArray(payload.sections)
                || payload.sections.length !== 6)
            return false
        const expectedSections = [
            { key: "highlyRated", minimumItems: 1,
              maximumItems: 25 },
            { key: "recentlyPlayed", minimumItems: 1,
              maximumItems: 25 },
            { key: "recentlyAdded", minimumItems: 5,
              maximumItems: 25 },
            { key: "platforms", minimumItems: 1,
              maximumItems: null },
            { key: "favorites", minimumItems: 1,
              maximumItems: 25 },
            { key: "mameHighScores", minimumItems: 1,
              maximumItems: 25 }
        ]
        const validated = []
        for (let sectionIndex = 0;
             sectionIndex < payload.sections.length; ++sectionIndex) {
            const section = payload.sections[sectionIndex]
            const expected = expectedSections[sectionIndex]
            if (section === null
                    || section.key !== expected.key
                    || typeof section.title !== "string"
                    || typeof section.listType !== "string"
                    || typeof section.source !== "string"
                    || typeof section.available !== "boolean"
                    || typeof section.displayable !== "boolean"
                    || section.minimumItems
                       !== expected.minimumItems
                    || section.maximumItems
                       !== expected.maximumItems
                    || !Array.isArray(section.items)
                    || section.items.length > 1000
                    || (expected.maximumItems !== null
                        && section.items.length
                           > expected.maximumItems))
                return false
            const items = []
            for (let itemIndex = 0;
                 itemIndex < section.items.length; ++itemIndex) {
                const item = section.items[itemIndex]
                if (item === null
                        || (item.kind !== "game"
                            && item.kind !== "platform")
                        || typeof item.title !== "string"
                        || item.title.length === 0
                        || typeof item.subtitle !== "string"
                        || typeof item.platform !== "string"
                        || typeof item.rating !== "number"
                        || typeof item.favorite !== "boolean"
                        || typeof item.gameCount !== "number"
                        || (item.kind === "game"
                            && (typeof item.gameId !== "string"
                                || item.gameId.length === 0))
                        || (item.kind === "platform"
                            && (typeof item.platformKey !== "string"
                                || item.platformKey.length === 0)))
                    return false
                items.push(item)
            }
            if (section.displayable
                    && items.length < section.minimumItems)
                return false
            if (section.displayable && section.available
                    && items.length > 0) {
                validated.push({
                    key: section.key,
                    title: section.title,
                    listType: section.listType,
                    source: section.source,
                    items: items
                })
            }
        }
        sections = validated
        sectionIndex = sections.length > 0 ? 0 : -1
        itemIndex = activeItems.length > 0 ? 0 : -1
        loadError = sections.length > 0
                  ? "" : "No local discovery lists are available."
        return sections.length > 0
    }

    function openCenter() {
        sections = []
        sectionIndex = -1
        itemIndex = -1
        loadError = ""
        if (controller === null
                || !controller.load_big_box_discovery_center()) {
            loadError = controller === null
                      ? "The library controller is unavailable."
                      : controller.status_message
            return false
        }
        if (!parsePayload(controller.big_box_discovery_json)) {
            loadError = "The Discovery Center payload was invalid."
            return false
        }
        open()
        pageFocus.forceActiveFocus()
        return true
    }

    function moveSection(delta) {
        if (sections.length === 0)
            return false
        let next = (sectionIndex + delta) % sections.length
        if (next < 0)
            next += sections.length
        sectionIndex = next
        itemIndex = activeItems.length > 0
                  ? Math.min(Math.max(itemIndex, 0),
                             activeItems.length - 1)
                  : -1
        sectionList.currentIndex = sectionIndex
        sectionList.positionViewAtIndex(sectionIndex, ListView.Contain)
        return true
    }

    function moveItem(delta) {
        if (activeItems.length === 0)
            return false
        let next = (itemIndex + delta) % activeItems.length
        if (next < 0)
            next += activeItems.length
        itemIndex = next
        return true
    }

    function chooseCurrent() {
        if (activeItem === null)
            return false
        const chosen = activeItem
        close()
        if (chosen.kind === "game") {
            gameSelected(chosen.gameId)
            return true
        }
        if (chosen.kind === "platform") {
            platformSelected(chosen.platformKey)
            return true
        }
        return false
    }

    function cancelPage() {
        close()
        cancelled()
        return true
    }

    function handleAction(action) {
        if (!opened)
            return false
        if (action === "BigBoxNavigateLeft")
            return moveItem(-1)
        if (action === "BigBoxNavigateRight")
            return moveItem(1)
        if (action === "BigBoxNavigateUp")
            return moveSection(-1)
        if (action === "BigBoxNavigateDown")
            return moveSection(1)
        if (action === "BigBoxPageUp")
            return moveSection(-3)
        if (action === "BigBoxPageDown")
            return moveSection(3)
        if (action === "BigBoxSelect"
                || action === "BigBoxPlayGame")
            return chooseCurrent()
        if (action === "BigBoxBack")
            return cancelPage()
        return true
    }

    onOpened: pageFocus.forceActiveFocus()
    onClosed: {
        sections = []
        sectionIndex = -1
        itemIndex = -1
        loadError = ""
    }

    background: Rectangle {
        color: "#11151d"

        gradient: Gradient {
            GradientStop { position: 0.0; color: "#19283d" }
            GradientStop { position: 0.48; color: "#11151d" }
            GradientStop { position: 1.0; color: "#07090e" }
        }
    }

    contentItem: FocusScope {
        id: pageFocus
        focus: true

        Keys.onPressed: function(event) {
            if (event.key === Qt.Key_Left)
                event.accepted = root.moveItem(-1)
            else if (event.key === Qt.Key_Right)
                event.accepted = root.moveItem(1)
            else if (event.key === Qt.Key_Up)
                event.accepted = root.moveSection(-1)
            else if (event.key === Qt.Key_Down)
                event.accepted = root.moveSection(1)
            else if (event.key === Qt.Key_PageUp)
                event.accepted = root.moveSection(-3)
            else if (event.key === Qt.Key_PageDown)
                event.accepted = root.moveSection(3)
            else if (event.key === Qt.Key_Return
                     || event.key === Qt.Key_Enter)
                event.accepted = root.chooseCurrent()
            else if (event.key === Qt.Key_Escape
                     || event.key === Qt.Key_Back)
                event.accepted = root.cancelPage()
        }

        ColumnLayout {
            id: pageLayout
            anchors.fill: parent
            anchors.margins: 32
            spacing: 14

            RowLayout {
                Layout.fillWidth: true
                Layout.preferredHeight: 66
                spacing: 20

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 0

                    Label {
                        text: "GAME DISCOVERY CENTER"
                        color: "white"
                        font.pixelSize: 32
                        font.bold: true
                        font.letterSpacing: 1.5
                    }
                    Label {
                        text: "Explore your library by rating, history, recency, platform, and favorites"
                        color: "#a9bad0"
                        font.pixelSize: 15
                    }
                }

                Label {
                    text: root.sectionIndex >= 0
                          ? (root.sectionIndex + 1) + " / "
                            + root.sections.length
                          : ""
                    color: "#8ba8c8"
                    font.pixelSize: 18
                    font.bold: true
                }

                Button {
                    text: "BACK"
                    Accessible.name: "Close Game Discovery Center"
                    onClicked: root.cancelPage()
                }
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 1
                color: "#39516f"
            }

            Label {
                Layout.fillWidth: true
                Layout.fillHeight: true
                visible: root.loadError.length > 0
                text: root.loadError
                color: "#ffb8b8"
                font.pixelSize: 22
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
            }

            ListView {
                id: sectionList
                Layout.fillWidth: true
                Layout.fillHeight: true
                visible: root.loadError.length === 0
                clip: true
                spacing: 14
                model: root.sections
                currentIndex: root.sectionIndex
                boundsBehavior: Flickable.StopAtBounds

                delegate: Item {
                    id: sectionDelegate
                    required property int index
                    required property var modelData

                    width: ListView.view.width
                    height: 174
                    opacity: root.sectionIndex === index ? 1.0 : 0.58

                    Column {
                        anchors.fill: parent
                        spacing: 8

                        Row {
                            width: parent.width
                            spacing: 12

                            Rectangle {
                                width: 5
                                height: 24
                                radius: 2
                                color: root.sectionIndex
                                       === sectionDelegate.index
                                       ? "#54a8ff" : "#3b4d62"
                            }
                            Label {
                                text: sectionDelegate.modelData.title
                                      .toUpperCase()
                                color: "white"
                                font.pixelSize: 21
                                font.bold: true
                                font.letterSpacing: 1
                            }
                            Label {
                                text: sectionDelegate.modelData.items.length
                                      + " ITEMS"
                                color: "#7890ac"
                                font.pixelSize: 13
                                anchors.baseline: parent.children[1].baseline
                            }
                        }

                        ListView {
                            id: itemList
                            width: parent.width
                            height: 138
                            orientation: ListView.Horizontal
                            spacing: 12
                            clip: true
                            interactive: true
                            model: sectionDelegate.modelData.items
                            currentIndex: root.sectionIndex
                                          === sectionDelegate.index
                                          ? root.itemIndex : -1

                            delegate: Rectangle {
                                id: card
                                required property int index
                                required property var modelData

                                readonly property bool selected:
                                    root.sectionIndex
                                    === sectionDelegate.index
                                    && root.itemIndex === index
                                readonly property string artGameId:
                                    modelData.kind === "game"
                                    ? modelData.gameId
                                    : (typeof modelData
                                              .representativeGameId
                                       === "string"
                                       ? modelData.representativeGameId : "")

                                width: 224
                                height: 132
                                radius: 8
                                color: modelData.kind === "platform"
                                       ? "#18324e" : "#242a34"
                                border.width: selected ? 4 : 1
                                border.color: selected
                                              ? "#65b5ff" : "#405066"
                                scale: selected ? 1.025 : 1.0

                                Behavior on scale {
                                    NumberAnimation { duration: 120 }
                                }

                                Row {
                                    anchors.fill: parent
                                    anchors.margins: 8
                                    spacing: 10

                                    Rectangle {
                                        width: 78
                                        height: 116
                                        radius: 4
                                        color: "#0d121a"
                                        clip: true

                                        Image {
                                            id: artImage
                                            anchors.fill: parent
                                            source: root.controller !== null
                                                    && card.artGameId
                                                       .length > 0
                                                    ? root.controller
                                                      .game_box_front_url_for_game(
                                                          card.artGameId)
                                                    : ""
                                            fillMode: Image.PreserveAspectCrop
                                            asynchronous: true
                                        }

                                        Label {
                                            anchors.centerIn: parent
                                            visible: artImage.status
                                                     !== Image.Ready
                                            text: card.modelData.kind
                                                  === "platform" ? "PLATFORM"
                                                                : "NO ART"
                                            color: "#66809c"
                                            font.pixelSize: 10
                                            font.bold: true
                                        }
                                    }

                                    Column {
                                        width: parent.width - 88
                                        anchors.verticalCenter: parent.verticalCenter
                                        spacing: 7

                                        Label {
                                            width: parent.width
                                            text: card.modelData.title
                                            color: "white"
                                            font.pixelSize: 17
                                            font.bold: true
                                            wrapMode: Text.Wrap
                                            maximumLineCount: 3
                                            elide: Text.ElideRight
                                        }
                                        Label {
                                            width: parent.width
                                            text: card.modelData.subtitle
                                            color: "#9cb0c6"
                                            font.pixelSize: 12
                                            wrapMode: Text.Wrap
                                            maximumLineCount: 3
                                            elide: Text.ElideRight
                                        }
                                        Label {
                                            visible: card.modelData.favorite
                                            text: "★ FAVORITE"
                                            color: "#f4c95d"
                                            font.pixelSize: 11
                                            font.bold: true
                                        }
                                    }
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: {
                                        root.sectionIndex =
                                            sectionDelegate.index
                                        root.itemIndex = card.index
                                    }
                                    onDoubleClicked: {
                                        root.sectionIndex =
                                            sectionDelegate.index
                                        root.itemIndex = card.index
                                        root.chooseCurrent()
                                    }
                                }
                            }
                        }
                    }
                }
            }

            RowLayout {
                Layout.fillWidth: true
                Layout.preferredHeight: 28

                Label {
                    Layout.fillWidth: true
                    text: "↑ ↓ LISTS     ← → GAMES     ENTER SELECT     ESC BACK"
                    color: "#8da2b9"
                    font.pixelSize: 13
                    font.bold: true
                }
                Label {
                    text: root.activeItem !== null
                          ? root.activeItem.title : ""
                    color: "#c8d6e6"
                    font.pixelSize: 13
                    font.bold: true
                    elide: Text.ElideRight
                    Layout.maximumWidth: 420
                }
            }
        }
    }
}
