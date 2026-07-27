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
    property bool migrateMedia: true
    property var platforms: []
    property var emulators: []
    property var controllers: []
    property var currentControllers: []
    property var controllerSupportLevels: []
    property var controllerIdsToAdd: []
    property var controllerIdsToRemove: []
    property int controllerSupportLevel: 0
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
        const platformResult = []
        const emulatorResult = []
        const controllerResult = []
        const currentControllerResult = []
        const supportLevelResult = []
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
            for (let index = 0;
                 index < controller.platform_entry_count; ++index)
                platformResult.push(controller.platform_name_at(index))
            for (let index = 2;
                 index < controller.emulator_entry_count(); ++index) {
                const id = controller.emulator_id_at(index)
                const title = controller.emulator_title_at(index)
                emulatorResult.push({
                    id: id,
                    title: title,
                    label: title + " (" + id + ")"
                })
            }
            for (let index = 0;
                 index < controller.game_controller_count(); ++index) {
                const id = controller.game_controller_id_at(index)
                const name = controller.game_controller_name_at(index)
                const category = controller.game_controller_category_at(index)
                const currentCount =
                    controller.bulk_edit_controller_current_game_count(id)
                const entry = {
                    id: id,
                    name: name,
                    category: category,
                    currentCount: currentCount,
                    label: name + " — " + category
                           + " (" + id + ")"
                }
                controllerResult.push(entry)
                if (currentCount > 0)
                    currentControllerResult.push(entry)
            }
            for (let index = 0;
                 index < controller.game_controller_support_level_count();
                 ++index) {
                supportLevelResult.push(
                    controller.game_controller_support_level_name_at(index))
            }
        }
        fields = result
        platforms = platformResult
        emulators = emulatorResult
        controllers = controllerResult
        currentControllers = currentControllerResult
        controllerSupportLevels = supportLevelResult
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
        migrateMedia = true
        controllerIdsToAdd = []
        controllerIdsToRemove = []
        controllerSupportLevel = 0
        resetModelSettingsEditor()
    }

    function resetModelSettingsEditor() {
        if (controller === null)
            return
        const encoded =
            controller.model_settings_defaults_json_for_type("box")
        if (encoded.length === 0)
            return
        const defaults = JSON.parse(encoded)
        bulkModelSettingsEditor.load(null, defaults, "boxFallback")
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
        if (editor === "controllerSupport" || editor === "modelSettings")
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
            version: 3,
            field: selectedField.key,
            operation: operation
        }
        if (editor === "controllerSupport") {
            request.addControllerIds = controllerIdsToAdd
            request.removeControllerIds = controllerIdsToRemove
            if (controllerIdsToAdd.length > 0)
                request.supportLevel = controllerSupportLevel
        } else if (editor === "modelSettings") {
            request.overrideDefaultModelSettings =
                bulkModelSettingsEditor.overrideEnabled
            request.modelSettings = bulkModelSettingsEditor.editPayload()
        } else if (editor === "boolean")
            request.boolean = booleanValue
        else if (editor === "rating")
            request.number = ratingValue
        else if (editor === "unsignedInteger")
            request.number = Number(textValue)
        else if (operation !== "clear")
            request.text = textValue
        if (editor === "customField")
            request.customFieldName = customFieldName
        if (editor === "platform")
            request.migrateMedia = migrateMedia
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
        if (editor === "unsignedInteger")
            return "set to " + Number(textValue).toFixed(0)
        if (editor === "controllerSupport") {
            const descriptions = []
            if (controllerIdsToRemove.length > 0)
                descriptions.push("remove "
                                  + controllerIdsToRemove.length
                                  + " controller"
                                  + (controllerIdsToRemove.length === 1
                                     ? "" : "s"))
            if (controllerIdsToAdd.length > 0)
                descriptions.push("add or update "
                                  + controllerIdsToAdd.length
                                  + " controller"
                                  + (controllerIdsToAdd.length === 1
                                     ? "" : "s")
                                  + " at "
                                  + controllerSupportLevels[
                                      controllerSupportLevel])
            return descriptions.join(" and ")
        }
        if (editor === "modelSettings") {
            if (!bulkModelSettingsEditor.overrideEnabled)
                return "remove game overrides and inherit platform or built-in settings"
            return "set one complete "
                   + bulkModelSettingsEditor.modelTypeKey
                   + " override on every selected game"
        }
        if (editor === "platform")
            return "move to \"" + textValue + "\""
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
        if (editor === "unsignedInteger")
            return /^(0|[1-9][0-9]*)$/.test(textValue.trim())
                   && Number(textValue) <= 4294967295
        if (editor === "controllerSupport")
            return controllerIdsToAdd.length
                   + controllerIdsToRemove.length > 0
                   && (controllerIdsToAdd.length === 0
                       || (controllerSupportLevel >= 0
                           && controllerSupportLevel
                              < controllerSupportLevels.length))
        if (editor === "modelSettings")
            return bulkModelSettingsEditor.isValid()
        if (editor === "platform")
            return platforms.length > 0 && textValue.trim().length > 0
        if (editor === "customField"
                && customFieldName.trim().length === 0)
            return false
        if (selectedField.key === "maxPlayers")
            return /^[1-9][0-9]*$/.test(textValue.trim())
        return textValue.trim().length > 0
    }

    function setControllerAdd(controllerId, selected) {
        let additions = controllerIdsToAdd.slice()
        let removals = controllerIdsToRemove.slice()
        const addIndex = additions.indexOf(controllerId)
        if (selected && addIndex < 0)
            additions.push(controllerId)
        else if (!selected && addIndex >= 0)
            additions.splice(addIndex, 1)
        if (selected) {
            const removeIndex = removals.indexOf(controllerId)
            if (removeIndex >= 0)
                removals.splice(removeIndex, 1)
        }
        controllerIdsToAdd = additions
        controllerIdsToRemove = removals
    }

    function setControllerRemove(controllerId, selected) {
        let additions = controllerIdsToAdd.slice()
        let removals = controllerIdsToRemove.slice()
        const removeIndex = removals.indexOf(controllerId)
        if (selected && removeIndex < 0)
            removals.push(controllerId)
        else if (!selected && removeIndex >= 0)
            removals.splice(removeIndex, 1)
        if (selected) {
            const addIndex = additions.indexOf(controllerId)
            if (addIndex >= 0)
                additions.splice(addIndex, 1)
        }
        controllerIdsToAdd = additions
        controllerIdsToRemove = removals
    }

    function applyRequest() {
        if (!editorIsValid() || controller === null)
            return false
        if (!controller.apply_bulk_edit(
                    JSON.stringify(requestObject())))
            return false
        page = 4
        return true
    }

    function smokeSelectPublisher(value) {
        for (let index = 0; index < fields.length; ++index) {
            if (fields[index].key === "publisher") {
                selectedFieldIndex = index
                operation = "set"
                textValue = value
                page = 3
                return true
            }
        }
        return false
    }

    function smokeSelectCustomDosBoxVersion(value) {
        for (let index = 0; index < fields.length; ++index) {
            if (fields[index].key === "customDosBoxVersion") {
                selectedFieldIndex = index
                operation = "set"
                textValue = value
                page = 3
                return true
            }
        }
        return false
    }

    function smokeSelectEmulator(value) {
        let emulatorIndex = -1
        for (let index = 0; index < emulators.length; ++index) {
            if (emulators[index].id === value) {
                emulatorIndex = index
                break
            }
        }
        if (emulatorIndex < 0)
            return false
        for (let index = 0; index < fields.length; ++index) {
            if (fields[index].key === "emulator") {
                selectedFieldIndex = index
                emulatorSelector.currentIndex = emulatorIndex
                operation = "set"
                textValue = value
                page = 3
                return true
            }
        }
        return false
    }

    function smokeSelectControllerSupport(controllerId, supportLevel) {
        let controllerIndex = -1
        for (let index = 0; index < controllers.length; ++index) {
            if (controllers[index].id === controllerId) {
                controllerIndex = index
                break
            }
        }
        if (controllerIndex < 0
                || controllers[controllerIndex].currentCount !== 1
                || supportLevel < 0
                || supportLevel >= controllerSupportLevels.length)
            return false
        for (let index = 0; index < fields.length; ++index) {
            if (fields[index].key === "controllerSupport") {
                selectedFieldIndex = index
                operation = "set"
                controllerIdsToAdd = [controllerId]
                controllerIdsToRemove = []
                controllerSupportLevel = supportLevel
                page = 1
                return true
            }
        }
        return false
    }

    function smokeConfirmControllerSupport() {
        if (editor !== "controllerSupport" || !editorIsValid())
            return false
        page = 3
        return true
    }

    function smokeSelectModelSettings() {
        for (let index = 0; index < fields.length; ++index) {
            if (fields[index].key === "modelSettings") {
                selectedFieldIndex = index
                operation = "set"
                resetModelSettingsEditor()
                bulkModelSettingsEditor.setSmokeValues(
                    "longJewelCase", false, 0.143,
                    "#ff123456", "#ffabcdef", [5, 7, 1])
                page = 1
                return editorIsValid()
            }
        }
        return false
    }

    function smokeConfirmModelSettings() {
        if (editor !== "modelSettings" || !editorIsValid())
            return false
        page = 3
        return true
    }

    function smokeSelectPlatform(value, migrate) {
        for (let index = 0; index < fields.length; ++index) {
            if (fields[index].key === "platform"
                    && platforms.indexOf(value) >= 0) {
                selectedFieldIndex = index
                operation = "set"
                textValue = value
                migrateMedia = migrate
                platformSelector.currentIndex = platforms.indexOf(value)
                page = 2
                return true
            }
        }
        return false
    }

    function smokeSelectPublisherAndApply(value) {
        return smokeSelectPublisher(value) && applyRequest()
    }

    function smokeSelectStartupLoadDelay(value) {
        for (let index = 0; index < fields.length; ++index) {
            if (fields[index].key === "startupScreenLoadDelay") {
                selectedFieldIndex = index
                operation = "set"
                textValue = String(value)
                page = 1
                return editor === "unsignedInteger" && editorIsValid()
            }
        }
        return false
    }

    function smokeSelectPauseScript(value) {
        for (let index = 0; index < fields.length; ++index) {
            if (fields[index].key
                    === "pauseScreenPauseGameAutoHotkeyScript") {
                selectedFieldIndex = index
                operation = "set"
                textValue = value
                page = 1
                return editor === "multilineText" && editorIsValid()
            }
        }
        return false
    }

    function smokeConfirmScalarField() {
        if (!editorIsValid()
                || (editor !== "unsignedInteger"
                    && editor !== "multilineText"))
            return false
        page = 3
        return true
    }

    onClosed: {
        if (controller !== null && controller.bulk_edit_visible
                && !controller.writing)
            controller.close_bulk_edit()
    }

    Connections {
        target: root.controller

        function onBulkEditRevisionChanged() {
            if (root.page === 4 && !root.controller.writing)
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
                          : root.page === 2 ? "Migrate game media"
                          : root.page === 3 ? "Confirm changes"
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
                            root.controllerIdsToAdd = []
                            root.controllerIdsToRemove = []
                            root.controllerSupportLevel = 0
                            if (root.editor === "platform"
                                    && root.platforms.length > 0) {
                                platformSelector.currentIndex = 0
                                root.textValue = root.platforms[0]
                            } else if (root.editor === "emulator"
                                       && root.emulators.length > 0) {
                                emulatorSelector.currentIndex = 0
                                root.textValue = root.emulators[0].id
                            } else if (root.editor === "modelSettings") {
                                root.resetModelSettingsEditor()
                            }
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
                                 && root.editor !== "controllerSupport"
                                 && root.editor !== "modelSettings"
                                 && root.editor !== "platform"
                        text: "Operation"
                        color: "#c7d2dc"
                    }

                    ComboBox {
                        id: operationSelector
                        objectName: "bulkEditOperationSelector"
                        visible: root.editor !== "boolean"
                                 && root.editor !== "rating"
                                 && root.editor !== "controllerSupport"
                                 && root.editor !== "modelSettings"
                                 && root.editor !== "platform"
                        Layout.fillWidth: true
                        model: root.operationModel()
                        onActivated: function(index) {
                            root.selectOperation(index)
                        }
                    }

                    Label {
                        visible: root.operation !== "clear"
                                 && root.editor !== "controllerSupport"
                                 && root.editor !== "modelSettings"
                        text: root.editor === "boolean" ? "Value"
                              : root.editor === "rating" ? "Rating"
                              : root.editor === "unsignedInteger"
                                ? "Milliseconds"
                              : root.editor === "lexicalPath"
                                ? "Stored path"
                              : root.editor === "emulator"
                                  ? "Emulator ID"
                                  : root.editor === "platform"
                                    ? "Destination platform"
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
                                 && root.editor !== "emulator"
                                 && root.editor !== "controllerSupport"
                                 && root.editor !== "modelSettings"
                                 && root.editor !== "platform"
                        Layout.fillWidth: true
                        placeholderText:
                            root.editor === "lexicalPath"
                            ? "LaunchBox path spelling is preserved"
                            : root.editor === "date" ? "YYYY-MM-DD"
                            : root.editor === "unsignedInteger"
                              ? "Whole milliseconds (0–4294967295)"
                            : "Value"
                        inputMethodHints:
                            root.editor === "unsignedInteger"
                            ? Qt.ImhDigitsOnly : Qt.ImhNone
                        text: root.textValue
                        onTextEdited: root.textValue = text
                    }

                    ComboBox {
                        id: emulatorSelector
                        objectName: "bulkEditEmulatorValue"
                        visible: root.operation !== "clear"
                                 && root.editor === "emulator"
                        Layout.fillWidth: true
                        model: root.emulators
                        textRole: "label"
                        onActivated: function(index) {
                            root.textValue = root.emulators[index].id
                        }
                    }

                    ComboBox {
                        id: platformSelector
                        objectName: "bulkEditPlatformValue"
                        visible: root.editor === "platform"
                        Layout.fillWidth: true
                        model: root.platforms
                        onActivated: function(index) {
                            root.textValue = root.platforms[index]
                        }
                    }

                    GridLayout {
                        visible: root.editor === "controllerSupport"
                        Layout.fillWidth: true
                        columns: 2
                        columnSpacing: 12
                        rowSpacing: 6

                        Label {
                            text: "Remove current controllers"
                            color: "#c7d2dc"
                            Layout.fillWidth: true
                        }

                        Label {
                            text: "Add or update controllers"
                            color: "#c7d2dc"
                            Layout.fillWidth: true
                        }

                        Frame {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 120

                            ListView {
                                objectName: "bulkEditControllerRemoveList"
                                anchors.fill: parent
                                clip: true
                                model: root.currentControllers
                                delegate: CheckBox {
                                    required property var modelData
                                    width: ListView.view.width
                                    text: modelData.label + " — on "
                                          + modelData.currentCount + " of "
                                          + root.controller.bulk_edit_target_count
                                    checked:
                                        root.controllerIdsToRemove.indexOf(
                                            modelData.id) >= 0
                                    onClicked:
                                        root.setControllerRemove(modelData.id,
                                                                 checked)
                                }
                            }
                        }

                        Frame {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 120

                            ListView {
                                objectName: "bulkEditControllerAddList"
                                anchors.fill: parent
                                clip: true
                                model: root.controllers
                                delegate: CheckBox {
                                    required property var modelData
                                    width: ListView.view.width
                                    text: modelData.label
                                    checked:
                                        root.controllerIdsToAdd.indexOf(
                                            modelData.id) >= 0
                                    onClicked:
                                        root.setControllerAdd(modelData.id,
                                                              checked)
                                }
                            }
                        }

                        Label {
                            visible: root.currentControllers.length === 0
                            text: "None of the selected games currently has "
                                  + "controller-support rows."
                            color: "#91a4b5"
                            wrapMode: Text.WordWrap
                            Layout.fillWidth: true
                            Layout.columnSpan: 2
                        }

                        Label {
                            visible: root.controllerIdsToAdd.length > 0
                            text: "Which support level would you like to set "
                                  + "the added controllers at?"
                            color: "#c7d2dc"
                            wrapMode: Text.WordWrap
                            Layout.fillWidth: true
                            Layout.columnSpan: 2
                        }

                        ComboBox {
                            objectName: "bulkEditControllerSupportLevel"
                            visible: root.controllerIdsToAdd.length > 0
                            Layout.fillWidth: true
                            Layout.columnSpan: 2
                            model: root.controllerSupportLevels
                            currentIndex: root.controllerSupportLevel
                            onActivated: function(index) {
                                root.controllerSupportLevel = index
                            }
                        }
                    }

                    ScrollView {
                        id: bulkModelSettingsScroll
                        visible: root.editor === "modelSettings"
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        contentWidth: availableWidth
                        clip: true

                        ModelSettingsEditor {
                            id: bulkModelSettingsEditor
                            objectName: "bulkEditModelSettings"
                            width: bulkModelSettingsScroll.availableWidth
                            scopeLabel: "selected games"
                            overrideLabel: "Override Default Model Settings"
                        }
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

                    Item {
                        visible: root.editor !== "modelSettings"
                        Layout.fillHeight: true
                    }
                }

                ColumnLayout {
                    spacing: 18

                    Label {
                        Layout.fillWidth: true
                        text: "The games being edited may have images and "
                              + "videos associated with them. Would you like "
                              + "to migrate that media to the new platform "
                              + "folders? Otherwise, it will no longer be "
                              + "associated with the games."
                        color: "#dfe8ef"
                        wrapMode: Text.WordWrap
                        font.pixelSize: 17
                    }

                    RadioButton {
                        objectName: "bulkEditMigrateMediaYes"
                        text: "Yes, I would like to migrate my media."
                        checked: root.migrateMedia
                        onToggled: {
                            if (checked)
                                root.migrateMedia = true
                        }
                    }

                    RadioButton {
                        objectName: "bulkEditMigrateMediaNo"
                        text: "No, I would not like to migrate my media."
                        checked: !root.migrateMedia
                        onToggled: {
                            if (checked)
                                root.migrateMedia = false
                        }
                    }

                    Label {
                        Layout.fillWidth: true
                        text: "Migration is limited to safely indexed image "
                              + "and video files. Shared-title ambiguity or "
                              + "an existing destination file stops the whole "
                              + "transaction before any XML is replaced."
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
                                    + (root.editor === "platform"
                                       ? "\n\nImages and videos: "
                                         + (root.migrateMedia
                                            ? "migrate to the destination folders"
                                            : "leave in the current folders")
                                       : "")
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
                    visible: root.page < 4
                    onClicked: root.closeWizard()
                }

                Button {
                    text: "Back"
                    visible: root.page === 1 || root.page === 2
                             || root.page === 3
                    onClicked: {
                        if (root.page === 3 && root.editor !== "platform")
                            root.page = 1
                        else
                            --root.page
                    }
                }

                Item { Layout.fillWidth: true }

                Button {
                    objectName: "bulkEditStartOverButton"
                    text: "Make Another Change"
                    visible: root.page === 4 && root.complete
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
                    visible: root.page === 4
                             && root.controller !== null
                             && !root.controller.writing
                    onClicked: root.closeWizard()
                }

                Button {
                    objectName: "bulkEditContinueButton"
                    text: root.page === 3 ? "Apply Changes" : "Continue"
                    visible: root.page < 4
                    enabled: root.page !== 1 || root.editorIsValid()
                    onClicked: {
                        if (root.page === 3)
                            root.applyRequest()
                        else if (root.page === 1
                                 && root.editor !== "platform")
                            root.page = 3
                        else
                            ++root.page
                    }
                }
            }
        }
    }
}
