import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Dialog {
    id: root

    property var controller
    property real auditTableWidth: 0
    property alias smokeCaptureTarget: auditSurface

    signal editRequested(string gameId)

    title: controller === null ? "AUDIT"
                               : "AUDIT — " + controller.audit_scope
    modal: true
    focus: true
    popupType: Popup.Item
    closePolicy: Popup.NoAutoClose
    anchors.centerIn: parent
    width: Math.max(760, parent ? parent.width * 0.96 : 1280)
    height: Math.max(520, parent ? parent.height * 0.94 : 820)
    standardButtons: Dialog.NoButton

    function refreshWidth() {
        if (controller === null) {
            auditTableWidth = 0
            return
        }
        let total = 0
        for (let column = 0;
             column < controller.audit_column_count; ++column)
            total += controller.audit_column_width_at(column)
        auditTableWidth = total
    }

    function openForPlatform(platform) {
        if (controller === null
                || !controller.open_game_audit(platform))
            return false
        refreshWidth()
        open()
        auditRows.currentIndex = controller.audit_row_count > 0 ? 0 : -1
        auditRows.positionViewAtBeginning()
        horizontalTable.contentX = 0
        auditFocus.forceActiveFocus()
        return true
    }

    function closeAudit() {
        if (controller !== null)
            controller.close_game_audit()
        root.close()
    }

    function copySelected() {
        if (controller === null)
            return false
        const text = controller.selected_game_audit_tsv()
        if (text.length === 0)
            return false
        clipboardProxy.text = text
        clipboardProxy.selectAll()
        clipboardProxy.copy()
        clipboardProxy.deselect()
        return true
    }

    function editCurrent() {
        if (controller === null || auditRows.currentIndex < 0)
            return false
        const gameId = controller.audit_game_id_at(
            auditRows.currentIndex)
        if (gameId.length === 0)
            return false
        editRequested(gameId)
        closeAudit()
        return true
    }

    onClosed: {
        if (controller !== null && controller.audit_visible)
            controller.close_game_audit()
    }

    Connections {
        target: root.controller

        function onAuditRevisionChanged() {
            auditRows.forceLayout()
            headerRepeater.model = 0
            headerRepeater.model = root.controller === null
                    ? 0 : root.controller.audit_column_count
        }
    }

    contentItem: Rectangle {
        id: auditSurface
        color: "#111820"
        border.color: "#526170"
        radius: 6

        FocusScope {
            id: auditFocus
            anchors.fill: parent
            focus: true

            Keys.onEscapePressed: root.closeAudit()
            Keys.onReturnPressed: root.editCurrent()
            Keys.onEnterPressed: root.editCurrent()
            Keys.onPressed: function(event) {
                if ((event.modifiers & Qt.ControlModifier)
                        && event.key === Qt.Key_A) {
                    root.controller.select_all_audit_rows()
                    event.accepted = true
                } else if ((event.modifiers & Qt.ControlModifier)
                           && event.key === Qt.Key_C) {
                    root.copySelected()
                    event.accepted = true
                }
            }

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 14
                spacing: 10

                RowLayout {
                    Layout.fillWidth: true
                    spacing: 10

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 2

                        Label {
                            text: root.controller === null
                                  ? "Game Audit"
                                  : "Game Audit — "
                                    + root.controller.audit_scope
                            color: "#f5f7fa"
                            font.pixelSize: 23
                            font.bold: true
                        }

                        Label {
                            text: root.controller === null ? ""
                                  : root.controller.audit_row_count
                                    + " games · "
                                    + root.controller.audit_selected_count
                                    + " selected · "
                                    + root.controller.audit_column_count
                                    + " recovered columns"
                            color: "#aebdca"
                            font.pixelSize: 13
                        }
                    }

                    Button {
                        objectName: "auditSelectAllButton"
                        text: "Select All"
                        enabled: root.controller !== null
                                 && root.controller.audit_row_count > 0
                        onClicked: root.controller.select_all_audit_rows()
                    }

                    Button {
                        objectName: "auditClearSelectionButton"
                        text: "Clear"
                        enabled: root.controller !== null
                                 && root.controller.audit_selected_count > 0
                        onClicked: root.controller.clear_audit_selection()
                    }

                    Button {
                        objectName: "auditCopyButton"
                        text: "Copy Selected"
                        enabled: root.controller !== null
                                 && root.controller.audit_selected_count > 0
                        onClicked: root.copySelected()
                    }

                    Button {
                        objectName: "auditEditButton"
                        text: "Edit"
                        enabled: auditRows.currentIndex >= 0
                        onClicked: root.editCurrent()
                    }

                    Button {
                        objectName: "auditCloseButton"
                        text: "Close"
                        onClicked: root.closeAudit()
                    }
                }

                Label {
                    Layout.fillWidth: true
                    text: "Click a row to select it; double-click to edit. "
                          + "Click a heading to sort. Duplicate games are "
                          + "highlighted in red. Blank MAME classification "
                          + "cells mean the source metadata is unavailable, "
                          + "not false. Copy produces tab-separated data for "
                          + "spreadsheet applications."
                    color: "#91a4b5"
                    wrapMode: Text.WordWrap
                    font.pixelSize: 12
                }

                Rectangle {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    color: "#0c1218"
                    border.color: "#34414d"
                    clip: true

                    Flickable {
                        id: horizontalTable
                        anchors.fill: parent
                        contentWidth: Math.max(width, root.auditTableWidth)
                        contentHeight: height
                        boundsBehavior: Flickable.StopAtBounds
                        flickableDirection: Flickable.HorizontalFlick
                        clip: true
                        ScrollBar.horizontal: ScrollBar {
                            policy: ScrollBar.AsNeeded
                        }

                        Column {
                            width: Math.max(horizontalTable.width,
                                            root.auditTableWidth)
                            height: horizontalTable.height

                            Row {
                                id: auditHeader
                                height: 40

                                Repeater {
                                    id: headerRepeater
                                    model: root.controller === null
                                           ? 0
                                           : root.controller.audit_column_count

                                    delegate: Button {
                                        required property int index
                                        width: root.controller
                                               .audit_column_width_at(index)
                                        height: auditHeader.height
                                        text: {
                                            const label = root.controller
                                                .audit_column_label_at(index)
                                            const key = root.controller
                                                .audit_column_key_at(index)
                                            if (root.controller.audit_sort_key
                                                    !== key)
                                                return label
                                            return label
                                                   + (root.controller
                                                      .audit_sort_descending
                                                      ? " ▼" : " ▲")
                                        }
                                        font.pixelSize: 11
                                        font.bold: true
                                        onClicked: {
                                            root.controller
                                                .sort_game_audit(index)
                                            auditRows.positionViewAtBeginning()
                                        }
                                    }
                                }
                            }

                            ListView {
                                id: auditRows
                                objectName: "gameAuditRows"
                                width: parent.width
                                height: parent.height - auditHeader.height
                                clip: true
                                model: root.controller === null
                                       ? 0 : root.controller.audit_row_count
                                currentIndex: -1
                                boundsBehavior: Flickable.StopAtBounds
                                ScrollBar.vertical: ScrollBar {
                                    policy: ScrollBar.AsNeeded
                                }

                                delegate: Item {
                                    id: auditRow
                                    required property int index
                                    width: auditRows.width
                                    height: 32

                                    readonly property bool selected:
                                        root.controller
                                            .audit_row_is_selected(index)
                                    readonly property bool duplicate:
                                        root.controller
                                            .audit_row_is_duplicate(index)

                                    Rectangle {
                                        anchors.fill: parent
                                        color: auditRow.selected
                                               ? "#285f91"
                                               : auditRow.duplicate
                                                 ? (auditRow.index % 2 === 0
                                                    ? "#6e2830"
                                                    : "#5b2028")
                                                 : (auditRow.index % 2 === 0
                                                    ? "#17212a"
                                                    : "#111920")
                                        border.color:
                                            auditRows.currentIndex
                                            === auditRow.index
                                            ? "#78b6ee" : "#24313c"
                                    }

                                    Row {
                                        anchors.fill: parent

                                        Repeater {
                                            model: root.controller === null
                                                   ? 0
                                                   : root.controller
                                                     .audit_column_count

                                            delegate: Item {
                                                required property int index
                                                width: root.controller
                                                       .audit_column_width_at(
                                                           index)
                                                height: auditRow.height

                                                Text {
                                                    anchors.fill: parent
                                                    anchors.leftMargin: 7
                                                    anchors.rightMargin: 7
                                                    verticalAlignment:
                                                        Text.AlignVCenter
                                                    text: root.controller
                                                        .audit_cell_at(
                                                            auditRow.index,
                                                            index)
                                                    color: "#e5ebf0"
                                                    elide: Text.ElideRight
                                                    font.pixelSize: 11
                                                }

                                                Rectangle {
                                                    anchors.right:
                                                        parent.right
                                                    width: 1
                                                    height: parent.height
                                                    color: "#2b3945"
                                                }
                                            }
                                        }
                                    }

                                    MouseArea {
                                        anchors.fill: parent
                                        acceptedButtons: Qt.LeftButton
                                        onClicked: {
                                            auditRows.currentIndex =
                                                auditRow.index
                                            root.controller
                                                .toggle_audit_row_selected(
                                                    auditRow.index)
                                        }
                                        onDoubleClicked: {
                                            auditRows.currentIndex =
                                                auditRow.index
                                            root.editCurrent()
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                TextEdit {
                    id: clipboardProxy
                    visible: false
                    readOnly: true
                }
            }
        }
    }
}
