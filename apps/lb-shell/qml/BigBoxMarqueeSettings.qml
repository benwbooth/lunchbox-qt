pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Dialog {
    id: root

    required property var controller
    property var screenLabels: []
    property var marqueeScreenLabels: []
    property var compatibilityKeys: []
    property var compatibilityLabels: []
    property bool pendingSave: false
    property int saveStartRevision: -1
    property string editorMessage: ""
    property alias smokeCaptureTarget: settingsLayout

    title: "BIG BOX DISPLAY SETTINGS"
    modal: true
    focus: true
    popupType: Popup.Item
    closePolicy: Popup.NoAutoClose
    anchors.centerIn: parent
    width: Math.min(760, parent ? parent.width - 80 : 760)
    height: Math.min(600, parent ? parent.height - 80 : 600)
    standardButtons: Dialog.NoButton

    function monitorLabel(index) {
        if (index < 0 || index >= controller.host_screen_count())
            return "Display " + (index + 1)
        const nativeName = controller.host_screen_name_at(index)
        const name = nativeName.length > 0
                   ? nativeName : "Display " + (index + 1)
        return (index + 1) + " — " + name + " ("
               + controller.host_screen_width_at(index) + "×"
               + controller.host_screen_height_at(index) + ")"
    }

    function modeIndex(key) {
        const index = compatibilityKeys.indexOf(key)
        return index >= 0 ? index : 0
    }

    function reloadFromController() {
        const nextScreens = []
        const screenCount = controller.host_screen_count()
        for (let index = 0; index < screenCount; ++index)
            nextScreens.push(monitorLabel(index))
        screenLabels = nextScreens
        marqueeScreenLabels = ["Disabled"].concat(nextScreens)

        const nextModeKeys = []
        const nextModeLabels = []
        for (let index = 0;
             index < controller.big_box_marquee_compatibility_mode_count();
             ++index) {
            nextModeKeys.push(
                controller.big_box_marquee_compatibility_mode_key_at(index))
            nextModeLabels.push(
                controller.big_box_marquee_compatibility_mode_label_at(index))
        }
        compatibilityKeys = nextModeKeys
        compatibilityLabels = nextModeLabels

        const primary = controller.big_box_primary_monitor_index
        primaryMonitorCombo.currentIndex =
            primary >= 0 && primary < nextScreens.length ? primary : 0
        const marquee = controller.big_box_marquee_monitor_index
        marqueeMonitorCombo.currentIndex =
            marquee >= 0 && marquee < nextScreens.length ? marquee + 1 : 0
        ignoreThemeViewsCheck.checked =
            controller.big_box_marquee_ignore_theme_views
        stretchImagesCheck.checked =
            controller.big_box_marquee_stretch_images
        compatibilityModeCombo.currentIndex =
            modeIndex(controller.big_box_marquee_compatibility_mode)
        pendingSave = false
        editorMessage = ""
        primaryMonitorCombo.forceActiveFocus()
    }

    function openEditor() {
        open()
    }

    function saveChanges() {
        if (screenLabels.length === 0
                || primaryMonitorCombo.currentIndex < 0
                || compatibilityModeCombo.currentIndex < 0
                || compatibilityModeCombo.currentIndex
                   >= compatibilityKeys.length) {
            editorMessage = "No valid display configuration is available."
            return false
        }
        const payload = {
            version: 1,
            primaryMonitorIndex: primaryMonitorCombo.currentIndex,
            marqueeMonitorIndex: marqueeMonitorCombo.currentIndex - 1,
            ignoreThemeViews: ignoreThemeViewsCheck.checked,
            stretchImages: stretchImagesCheck.checked,
            compatibilityMode:
                compatibilityKeys[compatibilityModeCombo.currentIndex]
        }
        saveStartRevision =
            controller.big_box_marquee_settings_revision
        if (!controller.save_big_box_marquee_settings(
                JSON.stringify(payload))) {
            editorMessage = controller.status_message
            return false
        }
        pendingSave = true
        editorMessage = "Saving one atomic BigBox settings transaction…"
        return true
    }

    function runSmokeExercise() {
        if (!opened || screenLabels.length < 1
                || compatibilityKeys.length !== 8)
            return false
        primaryMonitorCombo.currentIndex = 0
        marqueeMonitorCombo.currentIndex = 1
        ignoreThemeViewsCheck.checked = true
        stretchImagesCheck.checked = true
        compatibilityModeCombo.currentIndex =
            modeIndex("TopHalfCutOff")
        if (compatibilityModeCombo.currentIndex < 0)
            return false
        return saveChanges()
    }

    onOpened: reloadFromController()

    Connections {
        target: root.controller

        function onWritingChanged() {
            if (!root.pendingSave || root.controller.writing)
                return
            Qt.callLater(function() {
                if (!root.pendingSave)
                    return
                root.pendingSave = false
                if (root.controller.big_box_marquee_settings_revision
                        !== root.saveStartRevision) {
                    root.close()
                } else {
                    root.editorMessage =
                        root.controller.status_message
                }
            })
        }
    }

    contentItem: ColumnLayout {
        id: settingsLayout
        spacing: 16

        Label {
            Layout.fillWidth: true
            text: "Route BigBox and its independent marquee window using "
                  + "Qt's native screen objects. Display names come from "
                  + "the current Windows, Linux, or macOS host."
            wrapMode: Text.Wrap
            color: "#b9c8da"
            font.pixelSize: 15
        }

        GridLayout {
            Layout.fillWidth: true
            columns: 2
            columnSpacing: 18
            rowSpacing: 14

            Label {
                text: "Primary BigBox display"
                color: "white"
                font.bold: true
            }
            ComboBox {
                id: primaryMonitorCombo
                Layout.fillWidth: true
                model: root.screenLabels
                Accessible.name: "Primary BigBox display"
            }

            Label {
                text: "Marquee display"
                color: "white"
                font.bold: true
            }
            ComboBox {
                id: marqueeMonitorCombo
                Layout.fillWidth: true
                model: root.marqueeScreenLabels
                Accessible.name: "Marquee display"
            }

            Label {
                text: "Screen compatibility"
                color: "white"
                font.bold: true
            }
            ComboBox {
                id: compatibilityModeCombo
                Layout.fillWidth: true
                model: root.compatibilityLabels
                Accessible.name: "Marquee screen compatibility mode"
            }
        }

        CheckBox {
            id: ignoreThemeViewsCheck
            text: "Ignore theme marquee views"
            Accessible.description:
                "Use only direct marquee video or image media"
        }

        CheckBox {
            id: stretchImagesCheck
            text: "Stretch marquee images to fill the display"
        }

        Frame {
            Layout.fillWidth: true

            Label {
                anchors.fill: parent
                text: "The marquee is a separate native Qt window. It is "
                      + "frameless, never accepts focus, follows the selected "
                      + "game or platform, and plays marquee video silently."
                wrapMode: Text.Wrap
                color: "#91a4ba"
            }
        }

        Item {
            Layout.fillHeight: true
        }

        RowLayout {
            Layout.fillWidth: true

            Label {
                Layout.fillWidth: true
                text: root.editorMessage
                color: "#f0c04a"
                wrapMode: Text.Wrap
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
