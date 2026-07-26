import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick3D
import LaunchBoxPort

Popup {
    id: viewer

    required property LibraryController controller
    property string gameId: ""
    property string gameTitle: ""
    property Item returnFocusItem: null
    readonly property int mediaRevision: controller.game_media_revision
    readonly property url frontSource: {
        const revision = mediaRevision
        return gameId.length > 0
            ? controller.game_box_front_url_for_game(gameId) : ""
    }
    readonly property url backSource: {
        const revision = mediaRevision
        return gameId.length > 0
            ? controller.game_box_back_url_for_game(gameId) : ""
    }
    readonly property url spineSource: {
        const revision = mediaRevision
        return gameId.length > 0
            ? controller.game_box_spine_url_for_game(gameId) : ""
    }
    readonly property bool hasFront:
        frontSource.toString().length > 0
    readonly property bool hasBack:
        backSource.toString().length > 0
    readonly property bool hasSpine:
        spineSource.toString().length > 0
    readonly property int frontImageStatus: frontImageProbe.status
    readonly property int backImageStatus: backImageProbe.status
    readonly property int spineImageStatus: spineImageProbe.status
    readonly property bool sceneReady:
        opened && hasFront && frontImageStatus === Image.Ready
        && (!hasBack || backImageStatus === Image.Ready)
        && (!hasSpine || spineImageStatus === Image.Ready)
    readonly property string rotationLock:
        controller.model_rotation_lock
    readonly property real minimumZoom: 0.65
    readonly property real maximumZoom: 1.8
    readonly property real zoomStep: 0.15
    property real rotationX: -8
    property real rotationY: -22
    property real modelZoom: 1
    property real panX: 0
    property real panY: 0
    property alias viewerContentItem: modelViewerContent

    padding: 0
    modal: true
    dim: false
    focus: true
    closePolicy: Popup.NoAutoClose

    function clamp(value, minimum, maximum) {
        return Math.min(maximum, Math.max(minimum, value))
    }

    function normalizedAngle(value) {
        let angle = value % 360
        if (angle > 180)
            angle -= 360
        if (angle < -180)
            angle += 360
        return angle
    }

    function resetView() {
        rotationX = -8
        rotationY = -22
        modelZoom = 1
        panX = 0
        panY = 0
    }

    function rotateBy(horizontal, vertical) {
        let changed = false
        if (rotationLock !== "vertical" && horizontal !== 0) {
            rotationY = normalizedAngle(rotationY + horizontal)
            changed = true
        }
        if (rotationLock !== "horizontal" && vertical !== 0) {
            rotationX = clamp(rotationX + vertical, -80, 80)
            changed = true
        }
        return changed
    }

    function panBy(horizontal, vertical) {
        const nextX = clamp(panX + horizontal, -80, 80)
        const nextY = clamp(panY + vertical, -80, 80)
        const changed = nextX !== panX || nextY !== panY
        panX = nextX
        panY = nextY
        return changed
    }

    function setZoom(value) {
        const next = clamp(value, minimumZoom, maximumZoom)
        const changed = next !== modelZoom
        modelZoom = next
        return changed
    }

    function setRotationLock(value) {
        if (value !== "free"
                && value !== "horizontal"
                && value !== "vertical")
            return false
        return controller.save_model_rotation_lock(value)
    }

    function openForGame(requestedGameId, requestedGameTitle, focusItem) {
        if (requestedGameId.length === 0
                || controller.game_box_front_url_for_game(
                    requestedGameId).toString().length === 0)
            return false
        gameId = requestedGameId
        gameTitle = requestedGameTitle
        returnFocusItem = focusItem
        resetView()
        open()
        return true
    }

    // Runtime smokes invoke the actual visible controls through these methods.
    function activateRotateRightControl() {
        return rotateRightButton.activate()
    }

    function activateRotateUpControl() {
        return rotateUpButton.activate()
    }

    function activatePanRightControl() {
        return panRightButton.activate()
    }

    function activateZoomInControl() {
        return zoomInButton.activate()
    }

    function activateHorizontalLockControl() {
        return horizontalLockButton.activate()
    }

    function activateVerticalLockControl() {
        return verticalLockButton.activate()
    }

    function activateResetControl() {
        return resetButton.activate()
    }

    function activateBackControl() {
        return backButton.activate()
    }

    onOpened: Qt.callLater(function() {
        modelViewerContent.forceActiveFocus()
    })
    onClosed: {
        resetView()
        if (returnFocusItem)
            returnFocusItem.forceActiveFocus()
    }
    onHasFrontChanged: {
        if (opened && !hasFront)
            close()
    }

    background: Rectangle {
        color: "#030508"
    }

    contentItem: FocusScope {
        id: modelViewerContent
        focus: true
        Keys.priority: Keys.BeforeItem
        Keys.onPressed: function(event) {
            const movement = 7
            const translating = event.modifiers & Qt.ShiftModifier
            if (event.key === Qt.Key_Escape
                    || event.key === Qt.Key_Back) {
                viewer.close()
            } else if (event.key === Qt.Key_Left) {
                if (translating)
                    viewer.panBy(-8, 0)
                else
                    viewer.rotateBy(-movement, 0)
            } else if (event.key === Qt.Key_Right) {
                if (translating)
                    viewer.panBy(8, 0)
                else
                    viewer.rotateBy(movement, 0)
            } else if (event.key === Qt.Key_Up) {
                if (translating)
                    viewer.panBy(0, 8)
                else
                    viewer.rotateBy(0, -movement)
            } else if (event.key === Qt.Key_Down) {
                if (translating)
                    viewer.panBy(0, -8)
                else
                    viewer.rotateBy(0, movement)
            } else if (event.key === Qt.Key_Plus
                       || event.key === Qt.Key_Equal
                       || event.key === Qt.Key_PageUp) {
                viewer.setZoom(viewer.modelZoom + viewer.zoomStep)
            } else if (event.key === Qt.Key_Minus
                       || event.key === Qt.Key_PageDown) {
                viewer.setZoom(viewer.modelZoom - viewer.zoomStep)
            } else if (event.key === Qt.Key_0
                       || event.key === Qt.Key_Home) {
                viewer.resetView()
            } else {
                return
            }
            event.accepted = true
        }

        Image {
            id: frontImageProbe
            width: 1
            height: 1
            visible: false
            source: viewer.opened ? viewer.frontSource : ""
            asynchronous: true
            cache: true
        }
        Image {
            id: backImageProbe
            width: 1
            height: 1
            visible: false
            source: viewer.opened && viewer.hasBack
                    ? viewer.backSource : ""
            asynchronous: true
            cache: true
        }
        Image {
            id: spineImageProbe
            width: 1
            height: 1
            visible: false
            source: viewer.opened && viewer.hasSpine
                    ? viewer.spineSource : ""
            asynchronous: true
            cache: true
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
                        text: "INTERACTIVE 3D BOX MODEL"
                        color: "#67b3ff"
                        font.pixelSize: 15
                        font.bold: true
                        font.letterSpacing: 2
                        elide: Text.ElideRight
                    }
                }
                Label {
                    text: viewer.hasSpine
                          ? "FRONT  •  BACK  •  SPINE"
                          : viewer.hasBack
                            ? "FRONT  •  BACK"
                            : "FRONT"
                    color: "#9eb0c5"
                    font.pixelSize: 13
                    font.bold: true
                }
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.fillHeight: true
                color: "#080a0e"
                border.color: "#26384d"
                border.width: 1
                clip: true

                View3D {
                    id: modelView
                    anchors.fill: parent
                    camera: modelCamera
                    environment: SceneEnvironment {
                        backgroundMode: SceneEnvironment.Color
                        clearColor: "#080a0e"
                        antialiasingMode: SceneEnvironment.MSAA
                        antialiasingQuality: SceneEnvironment.High
                    }

                    PerspectiveCamera {
                        id: modelCamera
                        z: 650
                        fieldOfView: 38
                        clipNear: 10
                        clipFar: 2000
                    }

                    DirectionalLight {
                        eulerRotation.x: -28
                        eulerRotation.y: -34
                        brightness: 1.15
                        castsShadow: true
                    }
                    DirectionalLight {
                        eulerRotation.x: 34
                        eulerRotation.y: 150
                        brightness: 0.55
                    }

                    Node {
                        id: boxModel
                        position: Qt.vector3d(
                                      viewer.panX, viewer.panY, 0)
                        eulerRotation.x: viewer.rotationX
                        eulerRotation.y: viewer.rotationY
                        scale: Qt.vector3d(
                                   viewer.modelZoom,
                                   viewer.modelZoom,
                                   viewer.modelZoom)

                        Texture {
                            id: frontTexture
                            source: viewer.frontSource
                            generateMipmaps: true
                        }
                        Texture {
                            id: backTexture
                            source: viewer.backSource
                            generateMipmaps: true
                        }
                        Texture {
                            id: spineTexture
                            source: viewer.spineSource
                            generateMipmaps: true
                        }

                        Model {
                            source: "#Rectangle"
                            z: 21
                            scale: Qt.vector3d(1.8, 2.6, 1)
                            castsShadows: true
                            receivesShadows: true
                            materials: PrincipledMaterial {
                                baseColor: "#ffffff"
                                baseColorMap: frontTexture
                                roughness: 0.48
                            }
                        }
                        Model {
                            source: "#Rectangle"
                            z: -21
                            eulerRotation.y: 180
                            scale: Qt.vector3d(1.8, 2.6, 1)
                            castsShadows: true
                            receivesShadows: true
                            materials: PrincipledMaterial {
                                baseColor: viewer.hasBack
                                           ? "#ffffff" : "#13243a"
                                baseColorMap: viewer.hasBack
                                              ? backTexture : null
                                roughness: 0.52
                            }
                        }
                        Model {
                            source: "#Rectangle"
                            x: -90
                            eulerRotation.y: -90
                            scale: Qt.vector3d(0.42, 2.6, 1)
                            castsShadows: true
                            receivesShadows: true
                            materials: PrincipledMaterial {
                                baseColor: viewer.hasSpine
                                           ? "#ffffff" : "#244e78"
                                baseColorMap: viewer.hasSpine
                                              ? spineTexture : null
                                roughness: 0.5
                            }
                        }
                        Model {
                            source: "#Rectangle"
                            x: 90
                            eulerRotation.y: 90
                            scale: Qt.vector3d(0.42, 2.6, 1)
                            castsShadows: true
                            receivesShadows: true
                            materials: PrincipledMaterial {
                                baseColor: viewer.hasSpine
                                           ? "#ffffff" : "#244e78"
                                baseColorMap: viewer.hasSpine
                                              ? spineTexture : null
                                roughness: 0.5
                            }
                        }
                        Model {
                            source: "#Rectangle"
                            y: 130
                            eulerRotation.x: -90
                            scale: Qt.vector3d(1.8, 0.42, 1)
                            castsShadows: true
                            receivesShadows: true
                            materials: PrincipledMaterial {
                                baseColor: "#1b3552"
                                roughness: 0.56
                            }
                        }
                        Model {
                            source: "#Rectangle"
                            y: -130
                            eulerRotation.x: 90
                            scale: Qt.vector3d(1.8, 0.42, 1)
                            castsShadows: true
                            receivesShadows: true
                            materials: PrincipledMaterial {
                                baseColor: "#102238"
                                roughness: 0.56
                            }
                        }
                    }
                }

                MouseArea {
                    id: modelPointerArea
                    anchors.fill: parent
                    acceptedButtons: Qt.LeftButton | Qt.RightButton
                    cursorShape: pressed
                                 ? Qt.ClosedHandCursor
                                 : Qt.OpenHandCursor
                    property real lastX: 0
                    property real lastY: 0
                    onPressed: function(mouse) {
                        lastX = mouse.x
                        lastY = mouse.y
                    }
                    onPositionChanged: function(mouse) {
                        if (!pressed)
                            return
                        const dx = mouse.x - lastX
                        const dy = mouse.y - lastY
                        if (mouse.buttons & Qt.RightButton)
                            viewer.panBy(dx * 0.35, -dy * 0.35)
                        else
                            viewer.rotateBy(dx * 0.45, dy * 0.45)
                        lastX = mouse.x
                        lastY = mouse.y
                    }
                    onDoubleClicked: viewer.resetView()
                    onWheel: function(wheel) {
                        viewer.setZoom(
                            viewer.modelZoom
                            + (wheel.angleDelta.y >= 0
                               ? viewer.zoomStep : -viewer.zoomStep))
                        wheel.accepted = true
                    }
                }

                Label {
                    anchors.centerIn: parent
                    visible: viewer.opened && !viewer.sceneReady
                    text: viewer.hasFront
                          ? "Loading 3D box textures…"
                          : "No front box image is available."
                    color: "#9eb0c5"
                    font.pixelSize: 18
                }
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: 7

                Button {
                    id: rotateLeftButton
                    text: "↶"
                    Accessible.name: "Rotate model left"
                    onClicked: viewer.rotateBy(-10, 0)
                }
                Button {
                    id: rotateUpButton
                    text: "↑"
                    Accessible.name: "Rotate model up"
                    function activate() {
                        return viewer.rotateBy(0, -10)
                    }
                    onClicked: activate()
                }
                Button {
                    id: rotateDownButton
                    text: "↓"
                    Accessible.name: "Rotate model down"
                    onClicked: viewer.rotateBy(0, 10)
                }
                Button {
                    id: rotateRightButton
                    text: "↷"
                    Accessible.name: "Rotate model right"
                    function activate() {
                        return viewer.rotateBy(10, 0)
                    }
                    onClicked: activate()
                }
                Button {
                    id: panRightButton
                    text: "PAN →"
                    Accessible.name: "Translate model right"
                    function activate() {
                        return viewer.panBy(10, 0)
                    }
                    onClicked: activate()
                }
                Button {
                    id: zoomOutButton
                    text: "−"
                    enabled: viewer.modelZoom > viewer.minimumZoom
                    Accessible.name: "Zoom model out"
                    onClicked:
                        viewer.setZoom(
                            viewer.modelZoom - viewer.zoomStep)
                }
                Button {
                    id: zoomInButton
                    text: "+"
                    enabled: viewer.modelZoom < viewer.maximumZoom
                    Accessible.name: "Zoom model in"
                    function activate() {
                        if (!enabled)
                            return false
                        return viewer.setZoom(
                            viewer.modelZoom + viewer.zoomStep)
                    }
                    onClicked: activate()
                }
                Button {
                    id: resetButton
                    text: "RESET"
                    Accessible.name: "Reset model view"
                    function activate() {
                        viewer.resetView()
                        return true
                    }
                    onClicked: activate()
                }

                Item {
                    Layout.fillWidth: true
                }

                Label {
                    text: "ROTATION"
                    color: "#8297ae"
                    font.pixelSize: 11
                    font.bold: true
                }
                Button {
                    id: freeLockButton
                    text: "FREE"
                    checked: viewer.rotationLock === "free"
                    checkable: true
                    autoExclusive: true
                    Accessible.name: "Free model rotation"
                    onClicked: viewer.setRotationLock("free")
                }
                Button {
                    id: horizontalLockButton
                    text: "HORIZONTAL"
                    checked: viewer.rotationLock === "horizontal"
                    checkable: true
                    autoExclusive: true
                    Accessible.name: "Lock model rotation horizontal"
                    function activate() {
                        return viewer.setRotationLock("horizontal")
                    }
                    onClicked: activate()
                }
                Button {
                    id: verticalLockButton
                    text: "VERTICAL"
                    checked: viewer.rotationLock === "vertical"
                    checkable: true
                    autoExclusive: true
                    Accessible.name: "Lock model rotation vertical"
                    function activate() {
                        return viewer.setRotationLock("vertical")
                    }
                    onClicked: activate()
                }
            }

            Label {
                Layout.fillWidth: true
                text: "DRAG  ROTATE    RIGHT-DRAG / SHIFT+ARROWS  PAN    WHEEL / +/-  ZOOM    DOUBLE-CLICK / 0  RESET"
                color: "#75899f"
                font.pixelSize: 12
                horizontalAlignment: Text.AlignHCenter
                elide: Text.ElideRight
            }
        }
    }
}
