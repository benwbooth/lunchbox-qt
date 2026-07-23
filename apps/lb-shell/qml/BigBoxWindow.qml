import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQml.Models
import LaunchBoxPort

ApplicationWindow {
    id: window
    width: 1280
    height: 720
    visible: true
    visibility: Qt.application.arguments.indexOf("--windowed") >= 0
                || Qt.application.arguments.indexOf("--smoke-test") >= 0
                ? Window.Windowed : Window.FullScreen
    title: "BigBox Port"
    color: "#07090d"
    property bool smokeTest: Qt.application.arguments.indexOf("--smoke-test") >= 0
    property bool launchSmokeTest: Qt.application.arguments.indexOf("--launch-smoke-test") >= 0
    property int smokePhase: 0
    property int launchSmokePhase: 0
    property bool launchSmokeFinished: false
    property string launchSmokeGameId: {
        const requested = argumentValue("--launch-game-id")
        return requested.length > 0 ? requested : "fixture-racer"
    }
    property string launchSmokeAdditionalApplicationId:
        argumentValue("--launch-additional-application-id")

    function launchSelection() {
        if (gameList.currentIndex >= 0 && !controller.loading && !controller.writing
                && !controller.launching && !controller.launch_session_active) {
            const gameId = controller.game_id_at(gameList.currentIndex)
            if (gameId.length > 0)
                controller.launch_game(gameList.currentIndex, gameId)
        }
    }

    function showLaunchWithSelection() {
        if (gameList.currentIndex < 0 || controller.loading || controller.writing
                || controller.launching || controller.launch_session_active)
            return
        const gameId = controller.game_id_at(gameList.currentIndex)
        const count = controller.additional_application_count(
                          gameList.currentIndex, gameId)
        if (gameId.length > 0 && count > 0)
            launchWithDialog.prepare(gameList.currentIndex, gameId, count)
    }

    function verifyModelRoles(index, gameId, gameTitle, gamePlatform, gameFavorite,
                              gameCompleted, gamePlayCount, gameStarRating,
                              gameAdditionalApplicationCount, rowCount) {
        if (!smokeTest || index !== 0)
            return
        const expectedTitle = smokePhase === 0 ? "Fixture Adventure" : "Fixture Racer"
        const expectedFavorite = smokePhase === 0
        const expectedCompleted = smokePhase === 1
        const expectedId = smokePhase === 0 ? "fixture-adventure" : "fixture-racer"
        const expectedPlayCount = smokePhase === 0 ? 3 : 8
        const expectedStarRating = smokePhase === 0 ? 4 : 5
        const expectedRows = smokePhase === 0 ? 3 : 1
        const expectedAdditionalApplicationCount = smokePhase === 0 ? 1 : 0
        if (gameId !== expectedId || gameTitle !== expectedTitle
                || gamePlatform !== "Fixture Console"
                || gameFavorite !== expectedFavorite || gameCompleted !== expectedCompleted
                || gamePlayCount !== expectedPlayCount || gameStarRating !== expectedStarRating
                || gameAdditionalApplicationCount !== expectedAdditionalApplicationCount
                || rowCount !== expectedRows) {
            console.error("MODEL_ROLE_SMOKE_FAILED id=" + gameId
                          + " title=" + gameTitle
                          + " platform=" + gamePlatform
                          + " favorite=" + gameFavorite
                          + " completed=" + gameCompleted
                          + " additionalApps=" + gameAdditionalApplicationCount
                          + " rows=" + rowCount)
            Qt.exit(4)
        } else if (smokePhase === 0) {
            smokePhase = 1
            Qt.callLater(function() { controller.apply_filters("Racer", "") })
        } else {
            controller.report_model_smoke_success(rowCount)
            Qt.quit()
        }
    }

    function argumentValue(flag) {
        const index = Qt.application.arguments.indexOf(flag)
        return index >= 0 && index + 1 < Qt.application.arguments.length
               ? Qt.application.arguments[index + 1] : ""
    }

    LibraryController {
        id: controller
    }

    Component.onCompleted: {
        if (!controller.initialize_host_path_mappings()) {
            console.error("HOST_PATH_MAPPING_INITIALIZE_FAILED status="
                          + controller.status_message)
            return
        }
        const library = argumentValue("--library")
        if (library.length > 0)
            controller.load_library(library)
        else
            controller.load_fixture()
        gameList.forceActiveFocus()
    }

    Timer {
        interval: 3000
        running: window.smokeTest
        onTriggered: {
            console.error("MODEL_ROLE_SMOKE_TIMEOUT rows=" + gameList.count)
            Qt.exit(4)
        }
    }

    Timer {
        interval: 25
        repeat: true
        running: window.launchSmokeTest && !window.launchSmokeFinished
        onTriggered: {
            if (window.launchSmokePhase === 0 && !controller.loading
                    && controller.library_path.length > 0
                    && controller.game_count > 0) {
                const row = controller.row_for_game_id(window.launchSmokeGameId)
                if (row < 0) {
                    console.error("LAUNCH_SMOKE_MISSING_GAME id="
                                  + window.launchSmokeGameId)
                    Qt.exit(7)
                    return
                }
                window.launchSmokePhase = 1
                if (window.launchSmokeAdditionalApplicationId.length > 0)
                    controller.launch_additional_application(
                                row, window.launchSmokeGameId,
                                window.launchSmokeAdditionalApplicationId)
                else
                    controller.launch_game(row, window.launchSmokeGameId)
            } else if (window.launchSmokePhase === 1 && !controller.launching
                       && !controller.launch_session_active) {
                const contractOk = window.launchSmokeAdditionalApplicationId.length > 0
                    ? controller.report_additional_application_launch_smoke_success(
                          window.launchSmokeGameId,
                          window.launchSmokeAdditionalApplicationId)
                    : controller.report_launch_smoke_success(window.launchSmokeGameId)
                if (!controller.last_launch_succeeded || !contractOk) {
                    console.error("LAUNCH_SMOKE_FAILED id="
                                  + window.launchSmokeGameId
                                  + " status=" + controller.status_message)
                    Qt.exit(7)
                    return
                }
                window.launchSmokeFinished = true
                Qt.quit()
            }
        }
    }

    Timer {
        interval: 15000
        running: window.launchSmokeTest && !window.launchSmokeFinished
        onTriggered: {
            console.error("LAUNCH_SMOKE_TIMEOUT phase=" + window.launchSmokePhase
                          + " status=" + controller.status_message)
            Qt.exit(7)
        }
    }

    Rectangle {
        anchors.fill: parent
        gradient: Gradient {
            GradientStop { position: 0.0; color: "#17243b" }
            GradientStop { position: 0.62; color: "#090d14" }
            GradientStop { position: 1.0; color: "#050609" }
        }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 48
        spacing: 18

        RowLayout {
            Layout.fillWidth: true
            Label {
                Layout.fillWidth: true
                text: "BIGBOX"
                color: "#67b3ff"
                font.pixelSize: 34
                font.bold: true
                font.letterSpacing: 3
            }
            Label {
                text: controller.library_name
                color: "#b8c5d6"
                font.pixelSize: 20
            }
        }

        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

            ListView {
                id: gameList
                anchors.fill: parent
                orientation: ListView.Horizontal
                spacing: 28
                clip: true
                focus: true
                keyNavigationWraps: true
                highlightRangeMode: ListView.StrictlyEnforceRange
                preferredHighlightBegin: width * 0.36
                preferredHighlightEnd: width * 0.64
                snapMode: ListView.SnapOneItem
                model: controller
                Keys.onReturnPressed: function(event) {
                    window.launchSelection()
                    event.accepted = true
                }
                Keys.onEnterPressed: function(event) {
                    window.launchSelection()
                    event.accepted = true
                }

                delegate: Rectangle {
                    required property int index
                    required property string gameTitle
                    required property string gameId
                    required property string gamePlatform
                    required property bool gameFavorite
                    required property bool gameCompleted
                    required property int gamePlayCount
                    required property int gameStarRating
                    required property int gameAdditionalApplicationCount
                    Component.onCompleted: window.verifyModelRoles(
                                               index, gameId, gameTitle, gamePlatform,
                                               gameFavorite, gameCompleted, gamePlayCount,
                                               gameStarRating,
                                               gameAdditionalApplicationCount,
                                               gameList.count)
                    width: 370
                    height: gameList.height - 20
                    radius: 14
                    scale: gameList.currentIndex === index ? 1.0 : 0.88
                    opacity: gameList.currentIndex === index ? 1.0 : 0.55
                    color: gameList.currentIndex === index ? "#253a59" : "#131a24"
                    border.color: gameFavorite ? "#f0c04a" : "#4775aa"
                    border.width: gameList.currentIndex === index ? 3 : 1

                    Behavior on scale { NumberAnimation { duration: 150 } }
                    Behavior on opacity { NumberAnimation { duration: 150 } }

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: 28
                        Label {
                            Layout.fillWidth: true
                            text: gamePlatform.toUpperCase()
                            color: "#7fbfff"
                            font.pixelSize: 17
                            font.bold: true
                            elide: Text.ElideRight
                        }
                        Item { Layout.fillHeight: true }
                        Label {
                            Layout.fillWidth: true
                            text: gameTitle
                            color: "white"
                            font.pixelSize: 35
                            font.bold: true
                            wrapMode: Text.Wrap
                        }
                        Label {
                            Layout.fillWidth: true
                            text: (gameFavorite ? "★ FAVORITE    " : "")
                                  + (gameCompleted ? "✓ COMPLETED" : "")
                            color: "#f0c04a"
                            font.pixelSize: 15
                            font.bold: true
                        }
                    }
                    MouseArea {
                        anchors.fill: parent
                        onClicked: gameList.currentIndex = index
                        onDoubleClicked: controller.launch_game(index, gameId)
                    }
                }
            }
        }

        RowLayout {
            Layout.fillWidth: true
            Label {
                Layout.fillWidth: true
                text: gameList.count > 0
                      ? (gameList.currentIndex + 1) + " / " + gameList.count
                      : "No games"
                color: "#9badc4"
                font.pixelSize: 18
            }
            Button {
                text: "LAUNCH WITH…"
                visible: gameList.currentIndex >= 0
                         && controller.additional_application_count(
                             gameList.currentIndex,
                             controller.game_id_at(gameList.currentIndex)) > 0
                enabled: !controller.loading && !controller.writing
                         && !controller.launching
                         && !controller.launch_session_active
                onClicked: window.showLaunchWithSelection()
            }
            Label {
                text: controller.launching
                      ? "LAUNCHING…"
                      : "← →  BROWSE     ENTER  PLAY     ESC  EXIT"
                color: "#9badc4"
                font.pixelSize: 18
            }
        }
    }

    Dialog {
        id: launchWithDialog
        anchors.centerIn: parent
        modal: true
        title: "LAUNCH WITH"
        standardButtons: Dialog.Cancel
        property int modelRow: -1
        property string gameId: ""
        property int applicationCount: 0

        function prepare(row, id, count) {
            modelRow = row
            gameId = id
            applicationCount = count
            open()
            applicationList.forceActiveFocus()
        }

        contentItem: ListView {
            id: applicationList
            implicitWidth: 520
            implicitHeight: Math.min(420, contentHeight)
            spacing: 12
            clip: true
            focus: true
            keyNavigationWraps: true
            model: launchWithDialog.applicationCount
            delegate: Button {
                required property int index
                width: ListView.view.width
                text: controller.additional_application_name_at(
                          launchWithDialog.modelRow,
                          launchWithDialog.gameId, index)
                highlighted: applicationList.currentIndex === index
                onClicked: {
                    const applicationId = controller.additional_application_id_at(
                                              launchWithDialog.modelRow,
                                              launchWithDialog.gameId, index)
                    const row = launchWithDialog.modelRow
                    const gameId = launchWithDialog.gameId
                    launchWithDialog.close()
                    controller.launch_additional_application(row, gameId, applicationId)
                }
            }
        }
    }

    Shortcut {
        sequence: "L"
        onActivated: window.showLaunchWithSelection()
    }

    Shortcut {
        sequence: "Esc"
        onActivated: Qt.quit()
    }
}
