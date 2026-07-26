pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Dialog {
    id: root

    required property var controller
    property var actionKeys: []
    property var actionLabels: []
    property var bindingKeys: []
    property var holdKeys: []
    property bool originalGamepadEnabled: true
    property bool originalUseAllControllers: false
    property bool controllerRulesDirty: false
    property bool pendingSave: false
    property int saveStartRevision: -1
    property int captureActionRow: -1
    property int captureSlot: -1
    property string editorMessage: ""
    property alias smokeCaptureTarget: editorLayout

    title: "BIG BOX INPUT SETTINGS"
    modal: true
    focus: true
    popupType: Popup.Item
    closePolicy: Popup.NoAutoClose
    anchors.centerIn: parent
    width: Math.min(1120, parent ? parent.width - 80 : 1120)
    height: Math.min(760, parent ? parent.height - 80 : 760)
    standardButtons: Dialog.NoButton

    function openEditor() {
        open()
    }

    function actionIndex(key) {
        return actionKeys.indexOf(key)
    }

    function bindingIndex(key) {
        return bindingKeys.indexOf(key)
    }

    function holdIndex(key) {
        return holdKeys.indexOf(key)
    }

    function keyboardProperty(slot) {
        return "key" + slot
    }

    function originalKeyboardProperty(slot) {
        return "originalKey" + slot
    }

    function reloadFromController() {
        const nextActionKeys = []
        const nextActionLabels = []
        keyboardModel.clear()
        for (let actionIndex = 0;
             actionIndex < controller.big_box_input_action_count();
             ++actionIndex) {
            const actionKey =
                controller.big_box_input_action_key_at(actionIndex)
            const actionLabel =
                controller.big_box_input_action_label_at(actionIndex)
            nextActionKeys.push(actionKey)
            nextActionLabels.push(actionLabel)
            const slotCount =
                controller.big_box_input_action_keyboard_slot_count_at(
                    actionIndex)
            const row = {
                actionIndex: actionIndex,
                actionKey: actionKey,
                actionLabel: actionLabel,
                slotCount: slotCount,
                key0: -1,
                key1: -1,
                key2: -1,
                key3: -1,
                originalKey0: -1,
                originalKey1: -1,
                originalKey2: -1,
                originalKey3: -1
            }
            for (let slot = 0; slot < slotCount; ++slot) {
                const key =
                    controller.big_box_input_action_keyboard_wpf_key_at(
                        actionIndex, slot)
                row[keyboardProperty(slot)] = key
                row[originalKeyboardProperty(slot)] = key
            }
            keyboardModel.append(row)
        }
        actionKeys = nextActionKeys
        actionLabels = nextActionLabels

        const nextBindingKeys = []
        for (let bindingIndex = 0;
             bindingIndex
             < controller.big_box_controller_binding_option_count();
             ++bindingIndex) {
            nextBindingKeys.push(
                controller.big_box_controller_binding_option_key_at(
                    bindingIndex))
        }
        bindingKeys = nextBindingKeys
        holdKeys = ["None"].concat(nextBindingKeys)

        controllerRuleModel.clear()
        for (let ruleIndex = 0;
             ruleIndex < controller.big_box_controller_rule_count;
             ++ruleIndex) {
            controllerRuleModel.append({
                actionKey:
                    controller.big_box_controller_rule_action_at(ruleIndex),
                bindingKey:
                    controller.big_box_controller_rule_binding_at(ruleIndex),
                holdKey:
                    controller.big_box_controller_rule_hold_at(ruleIndex)
            })
        }

        originalGamepadEnabled = controller.big_box_gamepad_enabled
        originalUseAllControllers =
            controller.big_box_use_all_controllers
        gamepadEnabledCheck.checked = originalGamepadEnabled
        useAllControllersCheck.checked = originalUseAllControllers
        controllerRulesDirty = false
        pendingSave = false
        captureActionRow = -1
        captureSlot = -1
        editorMessage = ""
        tabBar.currentIndex = 0
        keyboardList.currentIndex = 0
        keyboardList.forceActiveFocus()
    }

    function beginCapture(actionRow, slot) {
        captureActionRow = actionRow
        captureSlot = slot
        editorMessage = "Press a key. Modifiers are stored as their own "
                        + "LaunchBox key, matching the recovered format."
        keyScope.forceActiveFocus()
    }

    function cancelCapture() {
        captureActionRow = -1
        captureSlot = -1
        editorMessage = ""
    }

    function captureKey(qtKey, modifiers) {
        const wpfKey = controller.big_box_wpf_key_for_qt_key(
            qtKey, modifiers === undefined ? 0 : modifiers)
        if (wpfKey < 0) {
            editorMessage =
                "That logical key is not representable in LaunchBox's "
                + "persisted keyboard format."
            return
        }
        keyboardModel.setProperty(
            captureActionRow, keyboardProperty(captureSlot), wpfKey)
        captureActionRow = -1
        captureSlot = -1
        editorMessage = ""
    }

    function clearKeyboardSlot(actionRow, slot) {
        keyboardModel.setProperty(
            actionRow, keyboardProperty(slot), 0)
        cancelCapture()
    }

    function hasKeyboardChanges() {
        for (let row = 0; row < keyboardModel.count; ++row) {
            const entry = keyboardModel.get(row)
            for (let slot = 0; slot < entry.slotCount; ++slot) {
                if (entry[keyboardProperty(slot)]
                        !== entry[originalKeyboardProperty(slot)])
                    return true
            }
        }
        return false
    }

    function controllerRulesAreValid() {
        const seen = {}
        for (let row = 0; row < controllerRuleModel.count; ++row) {
            const rule = controllerRuleModel.get(row)
            if (actionIndex(rule.actionKey) < 0
                    || bindingIndex(rule.bindingKey) < 0
                    || holdIndex(rule.holdKey) < 0) {
                editorMessage =
                    "Every controller row needs a known action, binding, "
                    + "and optional hold."
                return false
            }
            const identity = rule.actionKey + "\u001f"
                             + rule.bindingKey + "\u001f" + rule.holdKey
            if (seen[identity] === true) {
                editorMessage =
                    "Duplicate controller rows are not allowed."
                return false
            }
            seen[identity] = true
        }
        return true
    }

    function saveChanges() {
        cancelCapture()
        if (!controllerRulesAreValid())
            return
        const keyboardChanges = []
        for (let row = 0; row < keyboardModel.count; ++row) {
            const entry = keyboardModel.get(row)
            for (let slot = 0; slot < entry.slotCount; ++slot) {
                const key = entry[keyboardProperty(slot)]
                if (key !== entry[originalKeyboardProperty(slot)]) {
                    keyboardChanges.push({
                        action: entry.actionKey,
                        slot: slot,
                        wpfKey: key
                    })
                }
            }
        }

        const payload = {
            version: 1,
            keyboardChanges: keyboardChanges
        }
        if (gamepadEnabledCheck.checked
                !== originalGamepadEnabled) {
            payload.gamepadEnabled = gamepadEnabledCheck.checked
        }
        if (useAllControllersCheck.checked
                !== originalUseAllControllers) {
            payload.useAllControllers =
                useAllControllersCheck.checked
        }
        if (controllerRulesDirty) {
            const rules = []
            for (let row = 0; row < controllerRuleModel.count; ++row) {
                const rule = controllerRuleModel.get(row)
                rules.push({
                    action: rule.actionKey,
                    binding: rule.bindingKey,
                    hold: rule.holdKey
                })
            }
            payload.controllerRules = rules
        }

        if (keyboardChanges.length === 0
                && payload.gamepadEnabled === undefined
                && payload.useAllControllers === undefined
                && payload.controllerRules === undefined) {
            close()
            return
        }
        saveStartRevision = controller.big_box_input_revision
        if (!controller.save_big_box_input_settings(
                JSON.stringify(payload))) {
            editorMessage = controller.status_message
            return
        }
        pendingSave = true
        editorMessage = "Saving transaction…"
    }

    function runSmokeExercise() {
        if (!opened
                || keyboardModel.count !== 59
                || controllerRuleModel.count !== 18
                || actionKeys.length !== 59
                || bindingKeys.length !== 46)
            return false
        const selectRow = actionIndex("BigBoxSelect")
        if (selectRow < 0)
            return false
        beginCapture(selectRow, 1)
        captureKey(0x5a, 0)
        useAllControllersCheck.checked = true
        controllerRuleModel.setProperty(
            0, "actionKey", "BigBoxExit")
        controllerRuleModel.setProperty(
            0, "bindingKey", "Button8")
        controllerRuleModel.setProperty(
            0, "holdKey", "Button7")
        controllerRulesDirty = true
        saveChanges()
        return pendingSave
    }

    onOpened: reloadFromController()
    onClosed: cancelCapture()

    Connections {
        target: root.controller

        function onWritingChanged() {
            if (!root.pendingSave || root.controller.writing)
                return
            // The Rust completion handler publishes the committed policy and
            // revision immediately after it clears the shared writing flag.
            // Defer one event-loop turn so this observes that complete state.
            Qt.callLater(function() {
                if (!root.pendingSave)
                    return
                root.pendingSave = false
                if (root.controller.big_box_input_revision
                        !== root.saveStartRevision) {
                    root.close()
                } else {
                    root.editorMessage =
                        root.controller.status_message
                }
            })
        }
    }

    ListModel {
        id: keyboardModel
    }

    ListModel {
        id: controllerRuleModel
    }

    contentItem: FocusScope {
        id: keyScope
        focus: true

        Keys.onPressed: function(event) {
            if (root.captureActionRow < 0)
                return
            if (event.isAutoRepeat) {
                event.accepted = true
                return
            }
            root.captureKey(event.key, event.modifiers)
            event.accepted = true
        }

        ColumnLayout {
            id: editorLayout
            anchors.fill: parent
            spacing: 12

            Label {
                Layout.fillWidth: true
                text: "Edit LaunchBox-compatible bindings. Keyboard keys "
                      + "are logical and portable; controller mappings use "
                      + "the recovered semantic vocabulary."
                wrapMode: Text.Wrap
                color: "#b9c8da"
                font.pixelSize: 15
            }

            TabBar {
                id: tabBar
                Layout.fillWidth: true

                TabButton {
                    text: "KEYBOARD"
                }
                TabButton {
                    text: "CONTROLLERS"
                }
            }

            StackLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                currentIndex: tabBar.currentIndex

                Frame {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    padding: 0

                    ListView {
                        id: keyboardList
                        anchors.fill: parent
                        anchors.margins: 8
                        clip: true
                        spacing: 6
                        model: keyboardModel
                        keyNavigationWraps: true

                        delegate: Rectangle {
                            id: keyboardRow
                            required property int index
                            required property string actionLabel
                            required property int slotCount
                            required property int key0
                            required property int key1
                            required property int key2
                            required property int key3

                            width: ListView.view.width
                            height: slotCount > 0 ? 62 : 50
                            radius: 7
                            color: keyboardList.currentIndex === index
                                   ? "#20334a" : "#121c29"

                            function keyAt(slot) {
                                if (slot === 0)
                                    return key0
                                if (slot === 1)
                                    return key1
                                if (slot === 2)
                                    return key2
                                return key3
                            }

                            RowLayout {
                                anchors.fill: parent
                                anchors.margins: 8
                                spacing: 8

                                Label {
                                    Layout.preferredWidth: 235
                                    Layout.fillHeight: true
                                    text: keyboardRow.actionLabel
                                    verticalAlignment: Text.AlignVCenter
                                    elide: Text.ElideRight
                                    color: "white"
                                    font.bold: true
                                }

                                Label {
                                    visible: keyboardRow.slotCount === 0
                                    Layout.fillWidth: true
                                    text: "Controller only"
                                    color: "#7f90a5"
                                    font.italic: true
                                }

                                Repeater {
                                    model: 4

                                    RowLayout {
                                        id: slotEditor
                                        required property int index
                                        visible:
                                            index < keyboardRow.slotCount
                                        spacing: 3

                                        Button {
                                            id: keyButton
                                            Layout.preferredWidth: 98
                                            text:
                                                root.captureActionRow
                                                === keyboardRow.index
                                                && root.captureSlot
                                                === slotEditor.index
                                                ? "PRESS KEY…"
                                                : root.controller
                                                  .big_box_wpf_key_label(
                                                      keyboardRow.keyAt(
                                                          slotEditor.index))
                                            highlighted:
                                                root.captureActionRow
                                                === keyboardRow.index
                                                && root.captureSlot
                                                === slotEditor.index
                                            onClicked: {
                                                keyboardList.currentIndex =
                                                    keyboardRow.index
                                                root.beginCapture(
                                                    keyboardRow.index,
                                                    slotEditor.index)
                                            }
                                        }
                                        ToolButton {
                                            text: "×"
                                            Accessible.name:
                                                "Clear keyboard slot "
                                                + (slotEditor.index + 1)
                                                + " for "
                                                + keyboardRow.actionLabel
                                            onClicked:
                                                root.clearKeyboardSlot(
                                                    keyboardRow.index,
                                                    slotEditor.index)
                                        }
                                    }
                                }
                            }
                        }

                        ScrollBar.vertical: ScrollBar {}
                    }
                }

                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    spacing: 10

                    RowLayout {
                        Layout.fillWidth: true

                        CheckBox {
                            id: gamepadEnabledCheck
                            text: "Enable gamepads"
                        }
                        CheckBox {
                            id: useAllControllersCheck
                            text: "Use all controllers"
                            enabled: gamepadEnabledCheck.checked
                        }
                        Label {
                            Layout.fillWidth: true
                            horizontalAlignment: Text.AlignRight
                            text: root.controller.big_box_gamepad_status
                                  + " · "
                                  + root.controller
                                        .big_box_gamepad_connected_count
                                  + " connected"
                            color: "#91a4ba"
                        }
                    }

                    Label {
                        Layout.fillWidth: true
                        visible:
                            root.controller
                            .big_box_unsupported_controller_rule_count > 0
                        text:
                            root.controller
                            .big_box_unsupported_controller_rule_count
                            + " future or unsupported BigBox mapping(s) "
                            + "are preserved but cannot be edited here."
                        color: "#f0c04a"
                        wrapMode: Text.Wrap
                    }

                    Frame {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        padding: 0

                        ListView {
                            id: controllerList
                            anchors.fill: parent
                            anchors.margins: 8
                            clip: true
                            spacing: 6
                            model: controllerRuleModel

                            delegate: Rectangle {
                                id: controllerRow
                                required property int index
                                required property string actionKey
                                required property string bindingKey
                                required property string holdKey

                                width: ListView.view.width
                                height: 58
                                radius: 7
                                color: "#121c29"

                                RowLayout {
                                    anchors.fill: parent
                                    anchors.margins: 8
                                    spacing: 8

                                    ComboBox {
                                        Layout.fillWidth: true
                                        Layout.preferredWidth: 310
                                        model: root.actionLabels
                                        currentIndex:
                                            root.actionIndex(
                                                controllerRow.actionKey)
                                        onActivated:
                                            function(selectedIndex) {
                                                controllerRuleModel.setProperty(
                                                    controllerRow.index,
                                                    "actionKey",
                                                    root.actionKeys[
                                                        selectedIndex])
                                                root.controllerRulesDirty =
                                                    true
                                            }
                                    }
                                    ComboBox {
                                        Layout.preferredWidth: 180
                                        model: root.bindingKeys
                                        currentIndex:
                                            root.bindingIndex(
                                                controllerRow.bindingKey)
                                        onActivated:
                                            function(selectedIndex) {
                                                controllerRuleModel.setProperty(
                                                    controllerRow.index,
                                                    "bindingKey",
                                                    root.bindingKeys[
                                                        selectedIndex])
                                                root.controllerRulesDirty =
                                                    true
                                            }
                                    }
                                    Label {
                                        text: "while holding"
                                        color: "#91a4ba"
                                    }
                                    ComboBox {
                                        Layout.preferredWidth: 180
                                        model: root.holdKeys
                                        currentIndex:
                                            root.holdIndex(
                                                controllerRow.holdKey)
                                        onActivated:
                                            function(selectedIndex) {
                                                controllerRuleModel.setProperty(
                                                    controllerRow.index,
                                                    "holdKey",
                                                    root.holdKeys[
                                                        selectedIndex])
                                                root.controllerRulesDirty =
                                                    true
                                            }
                                    }
                                    ToolButton {
                                        text: "REMOVE"
                                        Accessible.name:
                                            "Remove controller mapping"
                                        onClicked: {
                                            controllerRuleModel.remove(
                                                controllerRow.index)
                                            root.controllerRulesDirty = true
                                        }
                                    }
                                }
                            }

                            ScrollBar.vertical: ScrollBar {}
                        }
                    }

                    Button {
                        text: "ADD CONTROLLER MAPPING"
                        enabled: root.actionKeys.length > 0
                                 && root.bindingKeys.length > 0
                        onClicked: {
                            controllerRuleModel.append({
                                actionKey: "BigBoxSelect",
                                bindingKey: "Button1",
                                holdKey: "None"
                            })
                            root.controllerRulesDirty = true
                            controllerList.positionViewAtEnd()
                        }
                    }
                }
            }

            RowLayout {
                Layout.fillWidth: true

                Label {
                    Layout.fillWidth: true
                    text: root.editorMessage.length > 0
                          ? root.editorMessage
                          : (root.hasKeyboardChanges()
                             || root.controllerRulesDirty
                             || gamepadEnabledCheck.checked
                                !== root.originalGamepadEnabled
                             || useAllControllersCheck.checked
                                !== root.originalUseAllControllers
                             ? "Unsaved changes"
                             : "No changes")
                    color: root.editorMessage.length > 0
                           ? "#f0c04a" : "#91a4ba"
                    wrapMode: Text.Wrap
                }
                Button {
                    visible: root.captureActionRow >= 0
                    text: "CANCEL CAPTURE"
                    onClicked: root.cancelCapture()
                }
                Button {
                    text: "CANCEL"
                    enabled: !root.pendingSave
                    onClicked: root.close()
                }
                Button {
                    text: root.pendingSave ? "SAVING…" : "SAVE"
                    highlighted: true
                    enabled: !root.pendingSave
                             && !root.controller.writing
                    onClicked: root.saveChanges()
                }
            }
        }
    }
}
