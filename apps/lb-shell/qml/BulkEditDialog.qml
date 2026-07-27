import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Dialog {
    id: root

    property var controller
    property int page: 0
    property var fields: []
    property int selectedFieldIndex: 0
    property string operation: "set"
    property string textValue: ""
    property bool booleanValue: true
    property real ratingValue: 0
    property string customFieldName: ""
    property alias smokeCaptureTarget: surface

    readonly property var selectedField:
        fields.length === 0 || selectedFieldIndex < 0
        || selectedFieldIndex >= fields.length
        ? null : fields[selectedFieldIndex]
    readonly property string editor:
        selectedField === null ? "" : selectedField.editor
    readonly property bool complete:
        controller !== null
        && controller.bulk_edit_completed_count
           === controller.bulk_edit_target_count
        && controller.bulk_edit_completed_count > 0

    title: "Bulk Edit"
    modal: true
    focus: true
    popupType: Popup.Item
    closePolicy: Popup.NoAutoClose
    standardButtons: Dialog.NoButton
    anchors.centerIn: parent
    width: Math.min(760, parent ? parent.width * 0.9 : 760)
    height: Math.min(620, parent ? parent.height * 0.9 : 620)

    function rebuildFields() {
        const result = []
        if (controller !== null) {
            for (let index = 0;
                 index < controller.bulk_edit_field_count(); ++index) {
                result.push({
                    key: controller.bulk_edit_field_key_at(index),
                    label: controller.bulk_edit_field_label_at(index),
                    editor: controller.bulk_edit_field_editor_at(index),
                    clearable:
                        controller.bulk_edit_field_clearable_at(index)
                })
            }
        }
        fields = result
    }

    function resetEditor() {
        page = 0
        selectedFieldIndex = 0
        operationSelector.currentIndex = 0
        operation = "set"
        textValue = ""
        booleanValue = true
        ratingValue = 0
        customFieldName = ""
    }

    function openWizard() {
        if (controller === null || !controller.open_audit_bulk_edit())
            return false
        rebuildFields()
        resetEditor()
        open()
        return true
    }

    function closeWizard() {
        if (controller !== null)
            controller.close_bulk_edit()
        if (controller === null || !controller.writing)
            close()
    }

    function operationModel() {
        if (selectedField === null)
            return ["Set"]
        if (editor === "multiValue" || editor === "customField")
            return selectedField.clearable
                    ? ["Set", "Add", "Remove", "Clear"]
                    : ["Set", "Add", "Remove"]
        return selectedField.clearable ? ["Set", "Clear"] : ["Set"]
    }

    function selectOperation(index) {
        const keys = ["set", "clear"]
        if (editor === "multiValue" || editor === "customField")
            operation = ["set", "add", "remove", "clear"][index]
        else
            operation = keys[index]
    }

    function requestObject() {
        const request = {
            version: 1,
            field: selectedField.key,
            operation: operation
        }
        if (editor === "boolean")
            request.boolean = booleanValue
        else if (editor === "rating")
            request.number = ratingValue
        else if (operation !== "clear")
            request.text = textValue
        if (editor === "customField")
            request.customFieldName = customFieldName
        return request
    }

    function valueDescription() {
        if (selectedField === null)
            return ""
        if (operation === "clear")
            return "clear the stored value"
        if (editor === "boolean")
            return booleanValue ? "set to Yes" : "set to No"
        if (editor === "rating")
            return "set to " + ratingValue.toFixed(1) + " stars"
        const prefix = operation === "add" ? "add "
                     : operation === "remove" ? "remove "
                     : "set to "
        if (editor === "customField")
            return prefix + "\"" + textValue + "\" in custom field \""
                   + customFieldName + "\""
        return prefix + "\"" + textValue + "\""
    }

    function editorIsValid() {
        if (selectedField === null)
            return false
        if (operation === "clear")
            return editor !== "customField"
                   || customFieldName.trim().length > 0
        if (editor === "boolean" || editor === "rating")
            return true
        if (editor === "customField"
                && customFieldName.trim().length === 0)
            return false
        if (selectedField.key === "maxPlayers")
            return /^[1-9][0-9]*$/.test(textValue.trim())
        return textValue.trim().length > 0
    }

    function applyRequest() {
        if (!editorIsValid() || controller === null)
            return false
        if (!controller.apply_bulk_edit(
                    JSON.stringify(requestObject())))
            return false
        page = 3
        return true
    }

    function smokeSelectPublisher(value) {
        for (let index = 0; index < fields.length; ++index) {
            if (fields[index].key === "publisher") {
                selectedFieldIndex = index
                operation = "set"
                textValue = value
                page = 2
                return true
            }
        }
        return false
    }

    function smokeSelectPublisherAndApply(value) {
        return smokeSelectPublisher(value) && applyRequest()
    }

    onClosed: {
        if (controller !== null && controller.bulk_edit_visible
                && !controller.writing)
            controller.close_bulk_edit()
    }

    Connections {
        target: root.controller

        function onBulkEditRevisionChanged() {
            if (root.page === 3 && !root.controller.writing)
                resultLabel.forceActiveFocus()
        }
    }

    contentItem: Rectangle {
        id: surface
        color: "#111820"
        border.color: "#526170"
        radius: 6

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 20
            spacing: 16

            RowLayout {
                Layout.fillWidth: true

                Label {
                    Layout.fillWidth: true
                    text: root.page === 0 ? "Welcome"
                          : root.page === 1 ? "Choose a field and value"
                          : root.page === 2 ? "Confirm changes"
                          : root.complete ? "Changes applied"
                          : root.controller !== null
                            && root.controller.writing
                            ? "Applying changes"
                            : "Could not apply changes"
                    color: "#f5f7fa"
                    font.pixelSize: 24
                    font.bold: true
                }

                Label {
                    text: root.controller === null ? ""
                          : root.controller.bulk_edit_target_count + " games"
                    color: "#9fc8ee"
                    font.pixelSize: 15
                    font.bold: true
                }
            }

            Rectangle {
                Layout.fillWidth: true
                height: 1
                color: "#34414d"
            }

            StackLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                currentIndex: root.page

                ColumnLayout {
                    spacing: 18

                    Label {
                        Layout.fillWidth: true
                        text: "This wizard changes one field on every selected "
                              + "game. The complete selection is validated "
                              + "again immediately before writing."
                        color: "#dfe8ef"
                        wrapMode: Text.WordWrap
                        font.pixelSize: 17
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 120
                        color: "#17232d"
                        border.color: "#3b4a57"
                        radius: 5

                        Label {
                            anchors.fill: parent
                            anchors.margins: 18
                            text: root.controller === null ? ""
                                  : root.controller.bulk_edit_target_count
                                    + " stable game IDs will be updated in one "
                                    + "recoverable transaction. Each affected "
                                    + "platform XML receives an exact backup."
                            color: "#b9c8d4"
                            wrapMode: Text.WordWrap
                            verticalAlignment: Text.AlignVCenter
                            font.pixelSize: 15
                        }
                    }

                    Item { Layout.fillHeight: true }
                }

                ColumnLayout {
                    spacing: 12

                    Label {
                        text: "Field"
                        color: "#c7d2dc"
                    }

                    ComboBox {
                        id: fieldSelector
                        objectName: "bulkEditFieldSelector"
                        Layout.fillWidth: true
                        model: root.fields
                        textRole: "label"
                        currentIndex: root.selectedFieldIndex
                        onActivated: function(index) {
                            root.selectedFieldIndex = index
                            operationSelector.currentIndex = 0
                            root.operation = "set"
                            root.textValue = ""
                            root.customFieldName = ""
                        }
                    }

                    Label {
                        visible: root.editor === "customField"
                        text: "Custom field name"
                        color: "#c7d2dc"
                    }

                    TextField {
                        objectName: "bulkEditCustomFieldName"
                        visible: root.editor === "customField"
                        Layout.fillWidth: true
                        placeholderText: "Field name"
                        text: root.customFieldName
                        onTextEdited: root.customFieldName = text
                    }

                    Label {
                        visible: root.editor !== "boolean"
                                 && root.editor !== "rating"
                        text: "Operation"
                        color: "#c7d2dc"
                    }

                    ComboBox {
                        id: operationSelector
                        objectName: "bulkEditOperationSelector"
                        visible: root.editor !== "boolean"
                                 && root.editor !== "rating"
                        Layout.fillWidth: true
                        model: root.operationModel()
                        onActivated: function(index) {
                            root.selectOperation(index)
                        }
                    }

                    Label {
                        visible: root.operation !== "clear"
                        text: root.editor === "boolean" ? "Value"
                              : root.editor === "rating" ? "Rating"
                              : root.editor === "lexicalPath"
                                ? "Stored path"
                                : root.editor === "emulator"
                                  ? "Emulator ID"
                                  : "Value"
                        color: "#c7d2dc"
                    }

                    ComboBox {
                        objectName: "bulkEditBooleanValue"
                        visible: root.editor === "boolean"
                        Layout.fillWidth: true
                        model: ["Yes", "No"]
                        onActivated: function(index) {
                            root.booleanValue = index === 0
                        }
                    }

                    RowLayout {
                        visible: root.editor === "rating"
                        Layout.fillWidth: true

                        Slider {
                            objectName: "bulkEditRatingValue"
                            Layout.fillWidth: true
                            from: 0
                            to: 5
                            stepSize: 0.5
                            value: root.ratingValue
                            onMoved: root.ratingValue = value
                        }

                        Label {
                            text: root.ratingValue.toFixed(1)
                            color: "#f5f7fa"
                            font.pixelSize: 17
                        }
                    }

                    TextArea {
                        objectName: "bulkEditMultilineValue"
                        visible: root.operation !== "clear"
                                 && root.editor === "multilineText"
                        Layout.fillWidth: true
                        Layout.preferredHeight: 130
                        wrapMode: TextEdit.Wrap
                        placeholderText: "Stored text"
                        text: root.textValue
                        onTextChanged: root.textValue = text
                    }

                    TextField {
                        objectName: "bulkEditTextValue"
                        visible: root.operation !== "clear"
                                 && root.editor !== "boolean"
                                 && root.editor !== "rating"
                                 && root.editor !== "multilineText"
                        Layout.fillWidth: true
                        placeholderText:
                            root.editor === "lexicalPath"
                            ? "LaunchBox path spelling is preserved"
                            : root.editor === "date" ? "YYYY-MM-DD"
                            : "Value"
                        text: root.textValue
                        onTextEdited: root.textValue = text
                    }

                    Label {
                        Layout.fillWidth: true
                        visible: root.editor === "lexicalPath"
                        text: "Paths are persisted as LaunchBox lexical data. "
                              + "They are not interpreted as Linux, Windows, "
                              + "or macOS host paths by this editor."
                        color: "#91a4b5"
                        wrapMode: Text.WordWrap
                    }

                    Item { Layout.fillHeight: true }
                }

                ColumnLayout {
                    spacing: 18

                    Label {
                        Layout.fillWidth: true
                        text: root.controller === null ? ""
                              : "Apply this change to "
                                + root.controller.bulk_edit_target_count
                                + " games?"
                        color: "#f5f7fa"
                        font.pixelSize: 19
                        font.bold: true
                        wrapMode: Text.WordWrap
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 150
                        color: "#17232d"
                        border.color: "#3b4a57"
                        radius: 5

                        Label {
                            anchors.fill: parent
                            anchors.margins: 18
                            text: root.selectedField === null ? ""
                                  : root.selectedField.label + ": "
                                    + root.valueDescription()
                                    + "\n\nAll affected platform documents "
                                    + "commit together or none do."
                            color: "#dfe8ef"
                            wrapMode: Text.WordWrap
                            verticalAlignment: Text.AlignVCenter
                            font.pixelSize: 16
                        }
                    }

                    Item { Layout.fillHeight: true }
                }

                ColumnLayout {
                    spacing: 18

                    BusyIndicator {
                        Layout.alignment: Qt.AlignHCenter
                        running: root.controller !== null
                                 && root.controller.writing
                        visible: running
                    }

                    Label {
                        id: resultLabel
                        objectName: "bulkEditResultLabel"
                        Layout.fillWidth: true
                        text: root.controller === null ? ""
                              : root.controller.bulk_edit_result_message
                        color: root.complete ? "#91d7a4" : "#dfe8ef"
                        wrapMode: Text.WordWrap
                        horizontalAlignment: Text.AlignHCenter
                        font.pixelSize: 17
                    }

                    ProgressBar {
                        Layout.fillWidth: true
                        from: 0
                        to: Math.max(1, root.controller === null
                                     ? 1
                                     : root.controller.bulk_edit_target_count)
                        value: root.controller === null ? 0
                               : root.controller.writing ? 0
                               : root.controller.bulk_edit_completed_count
                        indeterminate: root.controller !== null
                                       && root.controller.writing
                    }

                    Item { Layout.fillHeight: true }
                }
            }

            RowLayout {
                Layout.fillWidth: true

                Button {
                    text: "Cancel"
                    visible: root.page < 3
                    onClicked: root.closeWizard()
                }

                Button {
                    text: "Back"
                    visible: root.page === 1 || root.page === 2
                    onClicked: --root.page
                }

                Item { Layout.fillWidth: true }

                Button {
                    objectName: "bulkEditStartOverButton"
                    text: "Make Another Change"
                    visible: root.page === 3 && root.complete
                    onClicked: {
                        root.page = 1
                        operationSelector.currentIndex = 0
                        root.operation = "set"
                        root.textValue = ""
                        root.customFieldName = ""
                    }
                }

                Button {
                    objectName: "bulkEditCloseButton"
                    text: "Close"
                    visible: root.page === 3
                             && root.controller !== null
                             && !root.controller.writing
                    onClicked: root.closeWizard()
                }

                Button {
                    objectName: "bulkEditContinueButton"
                    text: root.page === 2 ? "Apply Changes" : "Continue"
                    visible: root.page < 3
                    enabled: root.page !== 1 || root.editorIsValid()
                    onClicked: {
                        if (root.page === 2)
                            root.applyRequest()
                        else
                            ++root.page
                    }
                }
            }
        }
    }
}
