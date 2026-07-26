import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import LaunchBoxPort

Popup {
    id: viewer
    required property LibraryController controller
    property string gameId: ""
    property string gameTitle: ""
    property Item returnFocusItem: null
    readonly property int mediaRevision: controller.game_media_revision
    readonly property int imageCount: {
        const revision = mediaRevision
        return gameId.length > 0
            ? controller.game_image_count_for_game(gameId) : 0
    }
    property int selectedImageIndex: -1
    readonly property int selectedMediaIndex:
        selectedImageIndex >= 0
        ? controller.game_image_media_index_at(
              gameId, selectedImageIndex) : -1
    readonly property string selectedMediaType:
        selectedMediaIndex >= 0
        ? controller.game_media_type_at(
              gameId, selectedMediaIndex) : ""
    readonly property url selectedMediaSource:
        selectedMediaIndex >= 0
        ? controller.game_media_url_at(
              gameId, selectedMediaIndex) : ""
    readonly property int imageStatus: fullscreenImage.status
    readonly property real minimumZoom: 1.0
    readonly property real maximumZoom: 4.0
    readonly property real zoomStep: 0.25
    property real zoomFactor: minimumZoom
    property real panX: 0
    property real panY: 0
    readonly property real panLimitX:
        Math.max(0, (fullscreenImage.paintedWidth
                     * zoomFactor - imageViewport.width) / 2)
    readonly property real panLimitY:
        Math.max(0, (fullscreenImage.paintedHeight
                     * zoomFactor - imageViewport.height) / 2)
    property alias viewerContentItem: imageViewerContent

    padding: 0
    modal: true
    dim: false
    focus: true
    closePolicy: Popup.NoAutoClose

    function clamp(value, minimum, maximum) {
        return Math.min(maximum, Math.max(minimum, value))
    }

    function resetView() {
        zoomFactor = minimumZoom
        panX = 0
        panY = 0
    }

    function clampPan() {
        panX = clamp(panX, -panLimitX, panLimitX)
        panY = clamp(panY, -panLimitY, panLimitY)
    }

    function setZoom(value) {
        zoomFactor = clamp(value, minimumZoom, maximumZoom)
        if (zoomFactor <= minimumZoom) {
            panX = 0
            panY = 0
        } else {
            clampPan()
        }
        return zoomFactor
    }

    function panBy(horizontal, vertical) {
        if (zoomFactor <= minimumZoom)
            return false
        panX = clamp(panX + horizontal, -panLimitX, panLimitX)
        panY = clamp(panY + vertical, -panLimitY, panLimitY)
        return true
    }

    function selectImage(index) {
        if (index < 0 || index >= imageCount)
            return false
        selectedImageIndex = index
        resetView()
        return true
    }

    function selectPreviousImage() {
        if (imageCount === 0)
            return false
        return selectImage(
            selectedImageIndex <= 0
            ? imageCount - 1 : selectedImageIndex - 1)
    }

    function selectNextImage() {
        if (imageCount === 0)
            return false
        return selectImage(
            selectedImageIndex < 0
            || selectedImageIndex + 1 >= imageCount
            ? 0 : selectedImageIndex + 1)
    }

    function imageIndexForMedia(mediaIndex) {
        if (mediaIndex < 0)
            return -1
        for (let index = 0; index < imageCount; ++index) {
            if (controller.game_image_media_index_at(
                    gameId, index) === mediaIndex)
                return index
        }
        return -1
    }

    function openForGame(requestedGameId, requestedGameTitle,
                         preferredMediaIndex, focusItem) {
        if (requestedGameId.length === 0) {
            return false
        }
        if (controller.game_image_count_for_game(
                requestedGameId) === 0) {
            return false
        }
        gameId = requestedGameId
        gameTitle = requestedGameTitle
        returnFocusItem = focusItem
        const preferredImageIndex =
            imageIndexForMedia(preferredMediaIndex)
        selectedImageIndex =
            preferredImageIndex >= 0 ? preferredImageIndex : 0
        resetView()
        viewer.open()
        return true
    }

    // Runtime smokes call these methods so the actual visible controls, their
    // enablement, and their signal wiring remain part of the tested path.
    function activateZoomInControl() {
        if (!zoomInButton.enabled)
            return false
        return zoomInButton.activate()
    }

    function activatePanDownControl() {
        if (!panDownButton.enabled)
            return false
        return panDownButton.activate()
    }

    function activateNextControl() {
        if (!nextButton.enabled)
            return false
        return nextButton.activate()
    }

    function activateFitControl() {
        if (!fitButton.enabled)
            return false
        return fitButton.activate()
    }

    function activateBackControl() {
        return backButton.activate()
    }

    onOpened: Qt.callLater(function() {
        imageViewerContent.forceActiveFocus()
    })
    onClosed: {
        resetView()
        if (returnFocusItem)
            returnFocusItem.forceActiveFocus()
    }
    onImageCountChanged: {
        if (opened && (selectedImageIndex < 0
                       || selectedImageIndex >= imageCount)) {
            if (imageCount > 0)
                selectImage(0)
            else
                viewer.close()
        }
    }
    onPanLimitXChanged: clampPan()
    onPanLimitYChanged: clampPan()

    background: Rectangle {
        color: "#030508"
    }

    contentItem: FocusScope {
        id: imageViewerContent
        focus: true
        Keys.priority: Keys.BeforeItem
        Keys.onPressed: function(event) {
            if (event.key === Qt.Key_Escape
                    || event.key === Qt.Key_Back) {
                viewer.close()
            } else if (event.key === Qt.Key_PageUp) {
                viewer.selectPreviousImage()
            } else if (event.key === Qt.Key_PageDown
                       || event.key === Qt.Key_Return
                       || event.key === Qt.Key_Enter) {
                viewer.selectNextImage()
            } else if (event.key === Qt.Key_Left) {
                if (viewer.zoomFactor > viewer.minimumZoom)
                    viewer.panBy(64, 0)
                else
                    viewer.selectPreviousImage()
            } else if (event.key === Qt.Key_Right) {
                if (viewer.zoomFactor > viewer.minimumZoom)
                    viewer.panBy(-64, 0)
                else
                    viewer.selectNextImage()
            } else if (event.key === Qt.Key_Up) {
                viewer.panBy(0, 64)
            } else if (event.key === Qt.Key_Down) {
                viewer.panBy(0, -64)
            } else if (event.key === Qt.Key_Plus
                       || event.key === Qt.Key_Equal) {
                viewer.setZoom(viewer.zoomFactor + viewer.zoomStep)
            } else if (event.key === Qt.Key_Minus) {
                viewer.setZoom(viewer.zoomFactor - viewer.zoomStep)
            } else if (event.key === Qt.Key_0) {
                viewer.resetView()
            } else {
                return
            }
            event.accepted = true
        }

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 24
            spacing: 14

            RowLayout {
                Layout.fillWidth: true
                spacing: 14

                Button {
                    id: backButton
                    text: "‹  BACK"
                    function activate() {
                        viewer.close()
                        return true
                    }
                    onClicked: activate()
                }
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 1
                    Label {
                        Layout.fillWidth: true
                        text: viewer.gameTitle
                        color: "#ffffff"
                        font.pixelSize: 28
                        font.bold: true
                        elide: Text.ElideRight
                    }
                    Label {
                        Layout.fillWidth: true
                        text: viewer.selectedMediaType
                        color: "#67b3ff"
                        font.pixelSize: 16
                        font.bold: true
                        font.letterSpacing: 1
                        elide: Text.ElideRight
                    }
                }
                Label {
                    text: viewer.imageCount > 0
                          ? (viewer.selectedImageIndex + 1)
                            + " / " + viewer.imageCount
                          : "NO IMAGES"
                    color: "#c7d5e5"
                    font.pixelSize: 18
                    font.bold: true
                }
            }

            Rectangle {
                id: imageViewport
                Layout.fillWidth: true
                Layout.fillHeight: true
                color: "#080a0e"
                border.color: "#26384d"
                border.width: 1
                clip: true

                Item {
                    x: viewer.panX
                    y: viewer.panY
                    width: imageViewport.width
                    height: imageViewport.height

                    Image {
                        id: fullscreenImage
                        anchors.fill: parent
                        source: viewer.opened
                                ? viewer.selectedMediaSource : ""
                        asynchronous: true
                        cache: true
                        smooth: true
                        mipmap: true
                        fillMode: Image.PreserveAspectFit
                        transformOrigin: Item.Center
                        scale: viewer.zoomFactor
                    }
                }

                MouseArea {
                    anchors.fill: parent
                    acceptedButtons: Qt.LeftButton
                    property real lastX: 0
                    property real lastY: 0
                    onPressed: function(mouse) {
                        lastX = mouse.x
                        lastY = mouse.y
                    }
                    onPositionChanged: function(mouse) {
                        if (!pressed
                                || viewer.zoomFactor
                                   <= viewer.minimumZoom)
                            return
                        viewer.panBy(
                            mouse.x - lastX, mouse.y - lastY)
                        lastX = mouse.x
                        lastY = mouse.y
                    }
                    onDoubleClicked: {
                        if (viewer.zoomFactor > viewer.minimumZoom)
                            viewer.resetView()
                        else
                            viewer.setZoom(2)
                    }
                    onWheel: function(wheel) {
                        viewer.setZoom(
                            viewer.zoomFactor
                            + (wheel.angleDelta.y >= 0
                               ? viewer.zoomStep
                               : -viewer.zoomStep))
                        wheel.accepted = true
                    }
                }
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: 8

                Button {
                    text: "‹  PREVIOUS IMAGE"
                    enabled: viewer.imageCount > 1
                    onClicked: viewer.selectPreviousImage()
                }
                Button {
                    id: nextButton
                    text: "NEXT IMAGE  ›"
                    enabled: viewer.imageCount > 1
                    function activate() {
                        return viewer.selectNextImage()
                    }
                    onClicked: activate()
                }
                Label {
                    Layout.fillWidth: true
                    Layout.minimumWidth: 0
                    text: "ENTER / PAGE  SWITCH    DRAG / ARROWS  PAN"
                    color: "#7f93aa"
                    font.pixelSize: 12
                    horizontalAlignment: Text.AlignHCenter
                    elide: Text.ElideRight
                    clip: true
                }
                Button {
                    Layout.minimumWidth: 44
                    Layout.preferredWidth: 44
                    Layout.maximumWidth: 44
                    text: "−"
                    enabled: viewer.zoomFactor > viewer.minimumZoom
                    onClicked:
                        viewer.setZoom(
                            viewer.zoomFactor - viewer.zoomStep)
                }
                Button {
                    id: fitButton
                    Layout.minimumWidth: 96
                    Layout.preferredWidth: 96
                    Layout.maximumWidth: 96
                    text: Math.round(viewer.zoomFactor * 100)
                          + "%  FIT"
                    enabled: viewer.zoomFactor > viewer.minimumZoom
                    function activate() {
                        viewer.resetView()
                        return true
                    }
                    onClicked: activate()
                }
                Button {
                    id: zoomInButton
                    Layout.minimumWidth: 44
                    Layout.preferredWidth: 44
                    Layout.maximumWidth: 44
                    text: "+"
                    enabled: viewer.zoomFactor < viewer.maximumZoom
                    function activate() {
                        viewer.setZoom(
                            viewer.zoomFactor + viewer.zoomStep)
                        return true
                    }
                    onClicked: activate()
                }
                Button {
                    Layout.minimumWidth: 44
                    Layout.preferredWidth: 44
                    Layout.maximumWidth: 44
                    text: "←"
                    enabled: viewer.zoomFactor > viewer.minimumZoom
                    onClicked: viewer.panBy(64, 0)
                }
                Button {
                    Layout.minimumWidth: 44
                    Layout.preferredWidth: 44
                    Layout.maximumWidth: 44
                    text: "↑"
                    enabled: viewer.zoomFactor > viewer.minimumZoom
                    onClicked: viewer.panBy(0, 64)
                }
                Button {
                    id: panDownButton
                    Layout.minimumWidth: 44
                    Layout.preferredWidth: 44
                    Layout.maximumWidth: 44
                    text: "↓"
                    enabled: viewer.zoomFactor > viewer.minimumZoom
                    function activate() {
                        return viewer.panBy(0, -64)
                    }
                    onClicked: activate()
                }
                Button {
                    Layout.minimumWidth: 44
                    Layout.preferredWidth: 44
                    Layout.maximumWidth: 44
                    text: "→"
                    enabled: viewer.zoomFactor > viewer.minimumZoom
                    onClicked: viewer.panBy(-64, 0)
                }
            }
        }
    }
}
