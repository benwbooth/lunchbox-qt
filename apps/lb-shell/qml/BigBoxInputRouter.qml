import QtQuick

Item {
    id: root

    required property var controller
    property bool enabled: true
    signal actionsTriggered(string actions)

    Instantiator {
        model: {
            const revision = root.controller.big_box_input_revision
            return root.controller.big_box_keyboard_binding_count()
        }

        delegate: Shortcut {
            required property int index
            sequence: {
                const revision = root.controller.big_box_input_revision
                return root.controller.big_box_keyboard_sequence_at(index)
            }
            context: Qt.ApplicationShortcut
            enabled: root.enabled && sequence.toString().length > 0
            onActivated:
                root.actionsTriggered(
                    root.controller.big_box_keyboard_actions_at(index))
        }
    }

    Timer {
        interval: 8
        repeat: true
        running: root.enabled
                 && root.controller.big_box_gamepad_enabled
        onTriggered: {
            // Keep one busy controller from starving rendering, while still
            // draining short bursts in a single frame.
            for (let count = 0; count < 16; ++count) {
                const action =
                    root.controller.poll_big_box_gamepad_action()
                if (action.length === 0)
                    break
                root.actionsTriggered(action)
            }
        }
    }
}
