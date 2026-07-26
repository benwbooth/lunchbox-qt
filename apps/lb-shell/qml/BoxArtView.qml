import QtQuick

Item {
    id: root

    property url frontSource
    property url backSource
    property bool showingBack: false
    property int requestedSourceWidth: 600
    property int requestedSourceHeight: 800

    readonly property bool canFlip: backSource.toString().length > 0
    readonly property url source:
        showingBack && canFlip ? backSource : frontSource
    readonly property int status:
        showingBack && canFlip ? backImage.status : frontImage.status
    property real flipAngle: showingBack && canFlip ? 180 : 0

    Behavior on flipAngle {
        NumberAnimation {
            duration: 220
            easing.type: Easing.InOutCubic
        }
    }

    Image {
        id: frontImage
        anchors.fill: parent
        source: root.frontSource
        asynchronous: true
        cache: true
        fillMode: Image.PreserveAspectFit
        sourceSize.width: root.requestedSourceWidth
        sourceSize.height: root.requestedSourceHeight
        visible: root.flipAngle < 90
        transform: Rotation {
            origin.x: frontImage.width / 2
            origin.y: frontImage.height / 2
            axis.x: 0
            axis.y: 1
            axis.z: 0
            angle: root.flipAngle
        }
    }

    Image {
        id: backImage
        anchors.fill: parent
        source: root.canFlip ? root.backSource : ""
        asynchronous: true
        cache: true
        fillMode: Image.PreserveAspectFit
        sourceSize.width: root.requestedSourceWidth
        sourceSize.height: root.requestedSourceHeight
        visible: root.flipAngle >= 90
        transform: Rotation {
            origin.x: backImage.width / 2
            origin.y: backImage.height / 2
            axis.x: 0
            axis.y: 1
            axis.z: 0
            angle: root.flipAngle - 180
        }
    }
}
