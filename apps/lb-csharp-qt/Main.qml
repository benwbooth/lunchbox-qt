import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ApplicationWindow {
    id: window
    visible: true
    width: 900
    height: 620
    title: "LaunchBox C# / Qt"

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 14

        Label {
            text: "LaunchBox library"
            font.pixelSize: 28
            Layout.fillWidth: true
        }

        Label {
            text: Library.status + "  (" + Library.gameCount + " games)"
            color: "#666666"
            Layout.fillWidth: true
            elide: Text.ElideMiddle
        }

        ListView {
            id: games
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: Library.games
            spacing: 6

            delegate: Frame {
                required property var modelData
                width: games.width

                RowLayout {
                    anchors.fill: parent
                    spacing: 12

                    Label {
                        text: modelData.title
                        Layout.fillWidth: true
                        elide: Text.ElideRight
                    }

                    Label {
                        text: modelData.favorite ? "Favorite" : ""
                        color: "#b06a00"
                    }

                    Button {
                        text: modelData.favorite ? "Unfavorite" : "Favorite"
                        onClicked: Library.setFavorite(modelData.id, !modelData.favorite)
                    }
                }
            }
        }
    }

    Timer {
        interval: 250
        running: Library.smokeMode
        repeat: false
        onTriggered: Qt.quit()
    }
}
