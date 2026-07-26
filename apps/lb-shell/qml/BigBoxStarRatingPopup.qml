pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Dialog {
    id: root

    property int modelRow: -1
    property string gameId: ""
    property string gameTitle: ""
    property real ratingValue: 0
    property real communityRating: 0
    property int communityVotes: 0
    property bool busy: false
    property alias smokeCaptureTarget: popupLayout

    signal submitted(int row, string gameId, real rating)
    signal cancelled()

    title: "SET STAR RATING"
    modal: true
    focus: true
    popupType: Popup.Item
    closePolicy: Popup.NoAutoClose
    anchors.centerIn: parent
    width: Math.min(900, parent ? parent.width - 80 : 900)
    height: Math.min(480, parent ? parent.height - 80 : 480)
    standardButtons: Dialog.NoButton

    function normalizedRating(value) {
        return Math.max(0, Math.min(5, Math.round(value * 2) / 2))
    }

    function openForGame(row, id, title, rating, community, votes) {
        modelRow = row
        gameId = id
        gameTitle = title
        ratingValue = normalizedRating(rating)
        communityRating = Math.max(0, Math.min(5, community))
        communityVotes = Math.max(0, votes)
        open()
        ratingScope.forceActiveFocus()
    }

    function adjustRating(delta) {
        ratingValue = normalizedRating(ratingValue + delta)
        return true
    }

    function saveRating() {
        if (busy || modelRow < 0 || gameId.length === 0)
            return false
        const row = modelRow
        const id = gameId
        const rating = normalizedRating(ratingValue)
        close()
        submitted(row, id, rating)
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
        if (action === "BigBoxNavigateLeft"
                || action === "BigBoxPageUp")
            return adjustRating(-0.5)
        if (action === "BigBoxNavigateRight"
                || action === "BigBoxPageDown")
            return adjustRating(0.5)
        if (action === "BigBoxSelect"
                || action === "BigBoxPlayGame"
                || action === "BigBoxSetStarRating")
            return saveRating()
        if (action === "BigBoxBack")
            return cancelEntry()
        return true
    }

    function runSmokeSetRating(value) {
        if (!opened)
            return false
        ratingValue = normalizedRating(value)
        return saveRating()
    }

    onOpened: ratingScope.forceActiveFocus()

    background: Rectangle {
        radius: 14
        color: "#f2111822"
        border.color: "#5c7696"
        border.width: 2
    }

    contentItem: FocusScope {
        id: ratingScope
        focus: true

        Keys.onPressed: function(event) {
            if (event.key === Qt.Key_Left)
                event.accepted = root.adjustRating(-0.5)
            else if (event.key === Qt.Key_Right)
                event.accepted = root.adjustRating(0.5)
            else if (event.key === Qt.Key_PageUp)
                event.accepted = root.adjustRating(-1)
            else if (event.key === Qt.Key_PageDown)
                event.accepted = root.adjustRating(1)
            else if (event.key === Qt.Key_Return
                     || event.key === Qt.Key_Enter)
                event.accepted = root.saveRating()
            else if (event.key === Qt.Key_Escape)
                event.accepted = root.cancelEntry()
            else if (event.key === Qt.Key_0) {
                root.ratingValue = 0
                event.accepted = true
            }
        }

        ColumnLayout {
            id: popupLayout
            anchors.fill: parent
            anchors.margins: 28
            spacing: 18

            Label {
                Layout.fillWidth: true
                text: root.gameTitle
                color: "white"
                font.pixelSize: 27
                font.bold: true
                horizontalAlignment: Text.AlignHCenter
                elide: Text.ElideRight
            }

            Item {
                id: starStrip
                Layout.alignment: Qt.AlignHCenter
                Layout.preferredWidth: emptyStars.implicitWidth
                Layout.preferredHeight: emptyStars.implicitHeight

                Label {
                    id: emptyStars
                    text: "★★★★★"
                    color: "#3c4654"
                    font.pixelSize: 74
                }

                Item {
                    width: emptyStars.implicitWidth
                           * root.ratingValue / 5
                    height: emptyStars.implicitHeight
                    clip: true

                    Label {
                        width: emptyStars.implicitWidth
                        text: "★★★★★"
                        color: "#f0c04a"
                        font.pixelSize: 74
                    }
                }

                MouseArea {
                    anchors.fill: parent
                    onPressed: function(mouse) {
                        root.ratingValue = root.normalizedRating(
                            mouse.x / width * 5)
                    }
                }
            }

            Label {
                Layout.fillWidth: true
                text: root.ratingValue === 0
                      ? "Not rated"
                      : root.ratingValue.toFixed(1) + " / 5"
                color: "#f0c04a"
                font.pixelSize: 25
                font.bold: true
                horizontalAlignment: Text.AlignHCenter
            }

            Label {
                Layout.fillWidth: true
                text: root.communityRating > 0
                      ? "Community  "
                        + root.communityRating.toFixed(2)
                        + " / 5  •  "
                        + root.communityVotes
                        + (root.communityVotes === 1 ? " vote" : " votes")
                      : "No community rating"
                color: "#a9bdd6"
                font.pixelSize: 17
                horizontalAlignment: Text.AlignHCenter
            }

            Item {
                Layout.fillHeight: true
            }

            RowLayout {
                Layout.alignment: Qt.AlignHCenter
                spacing: 14

                Button {
                    text: "CLEAR"
                    enabled: !root.busy
                    onClicked: root.ratingValue = 0
                }
                Button {
                    text: "SAVE"
                    highlighted: true
                    enabled: !root.busy
                    onClicked: root.saveRating()
                }
                Button {
                    text: "CANCEL"
                    enabled: !root.busy
                    onClicked: root.cancelEntry()
                }
            }

            Label {
                Layout.fillWidth: true
                text: "LEFT / RIGHT  HALF STAR     SELECT  SAVE     BACK  CANCEL"
                color: "#70849d"
                font.pixelSize: 14
                horizontalAlignment: Text.AlignHCenter
            }
        }
    }
}
