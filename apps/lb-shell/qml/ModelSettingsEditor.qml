import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ColumnLayout {
    id: editor

    property string scopeLabel: "game"
    property string overrideLabel: ""
    property color labelColor: "#c7d2dc"
    property alias overrideEnabled: overrideCheck.checked
    property string inheritedSource: "boxFallback"
    readonly property string modelTypeKey: modelTypeKeyField.text
    readonly property bool fullScanEnabled: fullScanCheck.checked
    readonly property real fullScanSpineWidth:
        Number(fullScanSpineWidthField.text)
    readonly property bool forcedSizeEnabled: forceSizeCheck.checked
    readonly property string caseColorText:
        caseColorCheck.checked ? caseColorField.text : ""
    readonly property string coverColorText:
        coverColorCheck.checked ? coverColorField.text : ""

    spacing: 10

    function storedText(value, fallback) {
        return value === null || value === undefined
               ? (fallback === undefined ? "" : fallback) : String(value)
    }

    function settingValue(settings, camelName, snakeName, fallback) {
        if (settings !== null && settings !== undefined) {
            if (settings[camelName] !== undefined)
                return settings[camelName]
            if (settings[snakeName] !== undefined)
                return settings[snakeName]
        }
        return fallback
    }

    function selectKnownType(key) {
        modelTypeChoice.currentIndex = -1
        for (let index = 0; index < modelTypeChoice.model.length; ++index) {
            if (modelTypeChoice.model[index].key === key) {
                modelTypeChoice.currentIndex = index
                break
            }
        }
    }

    function load(overrideSettings, effectiveSettings, source) {
        overrideEnabled = overrideSettings !== null
                          && overrideSettings !== undefined
        inheritedSource = source
        const settings = overrideEnabled
                         ? overrideSettings : effectiveSettings
        const typeKey = storedText(
            settingValue(settings, "modelType", "model_type", "box"),
            "box")
        modelTypeKeyField.text = typeKey
        selectKnownType(typeKey)

        const caseColor = settingValue(
            settings, "caseColor", "case_color", null)
        caseColorCheck.checked = caseColor !== null
                                 && caseColor !== undefined
        caseColorField.text = storedText(caseColor, "#ff000000")
        const coverColor = settingValue(
            settings, "coverColor", "cover_color", null)
        coverColorCheck.checked = coverColor !== null
                                  && coverColor !== undefined
        coverColorField.text = storedText(coverColor, "#ffffffff")
        frontSpineImageField.text = storedText(
            settingValue(settings, "frontSpineImage",
                         "front_spine_image", null))
        frontSpineClearCheck.checked = Boolean(
            settingValue(settings, "frontSpineIsClear",
                         "front_spine_is_clear", false))
        fullScanSpineWidthField.text = String(
            settingValue(settings, "fullImageSpineWidth",
                         "full_image_spine_width", 0.143))
        fullScanLandscapeCheck.checked = Boolean(
            settingValue(settings, "fullScanIsLandscape",
                         "full_scan_is_landscape", false))
        logoFontField.text = storedText(
            settingValue(settings, "logoFont", "logo_font", null))
        logoRotationField.text = storedText(
            settingValue(settings, "logoRotation",
                         "logo_rotation", "0,0,0,"))
        spineRotationField.text = storedText(
            settingValue(settings, "spineRotation",
                         "spine_rotation", "0,,0,"))
        fullScanCheck.checked = Boolean(
            settingValue(settings, "useFullScanImages",
                         "use_full_scan_images", false))

        const size = settingValue(settings, "modelSize",
                                  "model_size", null)
        forceSizeCheck.checked = size !== null && size !== undefined
        modelSizeXField.text = size ? String(size[0]) : "1"
        modelSizeYField.text = size ? String(size[1]) : "1"
        modelSizeZField.text = size ? String(size[2]) : "0.1"
    }

    function optionalText(value) {
        return value.trim().length > 0 ? value : null
    }

    function editPayload() {
        if (!overrideEnabled)
            return null
        return {
            model_type: modelTypeKeyField.text.trim(),
            case_color: caseColorCheck.checked
                        ? optionalText(caseColorField.text) : null,
            cover_color: coverColorCheck.checked
                         ? optionalText(coverColorField.text) : null,
            front_spine_image: optionalText(frontSpineImageField.text),
            front_spine_is_clear: frontSpineClearCheck.checked,
            full_image_spine_width:
                Number(fullScanSpineWidthField.text),
            full_scan_is_landscape: fullScanLandscapeCheck.checked,
            logo_font: optionalText(logoFontField.text),
            logo_rotation: logoRotationField.text,
            model_size: forceSizeCheck.checked
                        ? [Number(modelSizeXField.text),
                           Number(modelSizeYField.text),
                           Number(modelSizeZField.text)]
                        : null,
            spine_rotation: spineRotationField.text,
            use_full_scan_images: fullScanCheck.checked
        }
    }

    function isValid() {
        if (!overrideEnabled)
            return true
        if (modelTypeKeyField.text.trim().length === 0)
            return false
        const spineWidth = Number(fullScanSpineWidthField.text)
        if (!Number.isFinite(spineWidth)
                || spineWidth < 0 || spineWidth > 1)
            return false
        const colorPattern = /^#(?:[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$/
        if (caseColorCheck.checked
                && !colorPattern.test(caseColorField.text.trim()))
            return false
        if (coverColorCheck.checked
                && !colorPattern.test(coverColorField.text.trim()))
            return false
        if (forceSizeCheck.checked) {
            const x = Number(modelSizeXField.text)
            const y = Number(modelSizeYField.text)
            const z = Number(modelSizeZField.text)
            if (!Number.isFinite(x) || !Number.isFinite(y)
                    || !Number.isFinite(z) || x <= 0 || y <= 0 || z <= 0)
                return false
        }
        return true
    }

    function setSmokeValues(typeKey, fullScan, spineWidth,
                            caseColor, coverColor, size) {
        overrideEnabled = true
        modelTypeKeyField.text = typeKey
        selectKnownType(typeKey)
        fullScanCheck.checked = fullScan
        fullScanSpineWidthField.text = String(spineWidth)
        caseColorCheck.checked = true
        caseColorField.text = caseColor
        coverColorCheck.checked = true
        coverColorField.text = coverColor
        forceSizeCheck.checked = size !== null
        if (size !== null) {
            modelSizeXField.text = String(size[0])
            modelSizeYField.text = String(size[1])
            modelSizeZField.text = String(size[2])
        }
    }

    Label {
        Layout.fillWidth: true
        text: "LaunchBox stores one whole-record override. Disable it to inherit the platform or built-in model without copying those values into XML."
        wrapMode: Text.Wrap
        color: "#7fbfff"
    }

    CheckBox {
        id: overrideCheck
        palette.windowText: editor.labelColor
        palette.buttonText: editor.labelColor
        palette.text: editor.labelColor
        contentItem: Text {
            text: overrideCheck.text
            font: overrideCheck.font
            color: editor.labelColor
            verticalAlignment: Text.AlignVCenter
            leftPadding: overrideCheck.indicator.width
                         + overrideCheck.spacing
        }
        text: editor.overrideLabel.length > 0
              ? editor.overrideLabel
              : "Use custom " + editor.scopeLabel + " 3D model settings"
    }

    Label {
        Layout.fillWidth: true
        visible: !overrideCheck.checked
        text: "Inherited source: "
              + (editor.inheritedSource === "platformOverride"
                 ? "platform override"
                 : editor.inheritedSource === "builtInPlatform"
                   ? "LaunchBox built-in platform"
                   : "box fallback")
        color: "#aeb8c5"
    }

    GridLayout {
        Layout.fillWidth: true
        enabled: overrideCheck.checked
        columns: 2
        columnSpacing: 12
        rowSpacing: 8

        Label { text: "Model type"; color: editor.labelColor }
        ComboBox {
            id: modelTypeChoice
            Layout.fillWidth: true
            textRole: "label"
            valueRole: "key"
            model: [
                { key: "box", label: "Box" },
                { key: "dvd", label: "DVD Case" },
                { key: "jewelCase", label: "Jewel Case" },
                { key: "longJewelCase", label: "Long Jewel Case" }
            ]
            displayText: currentIndex >= 0
                         ? model[currentIndex].label
                         : "Custom / future model"
            onActivated: function(index) {
                modelTypeKeyField.text = model[index].key
            }
        }

        Label { text: "Stored model key"; color: editor.labelColor }
        TextField {
            id: modelTypeKeyField
            Layout.fillWidth: true
            placeholderText: "box"
            onTextEdited: editor.selectKnownType(text.trim())
        }

        Label { text: "Case color"; color: editor.labelColor }
        RowLayout {
            Layout.fillWidth: true
            CheckBox {
                id: caseColorCheck
                text: "Force"
                palette.windowText: editor.labelColor
                palette.buttonText: editor.labelColor
                palette.text: editor.labelColor
                contentItem: Text {
                    text: caseColorCheck.text
                    font: caseColorCheck.font
                    color: editor.labelColor
                    verticalAlignment: Text.AlignVCenter
                    leftPadding: caseColorCheck.indicator.width
                                 + caseColorCheck.spacing
                }
            }
            TextField {
                id: caseColorField
                Layout.fillWidth: true
                enabled: caseColorCheck.checked
                placeholderText: "#AARRGGBB"
            }
        }

        Label { text: "Cover color"; color: editor.labelColor }
        RowLayout {
            Layout.fillWidth: true
            CheckBox {
                id: coverColorCheck
                text: "Force"
                palette.windowText: editor.labelColor
                palette.buttonText: editor.labelColor
                palette.text: editor.labelColor
                contentItem: Text {
                    text: coverColorCheck.text
                    font: coverColorCheck.font
                    color: editor.labelColor
                    verticalAlignment: Text.AlignVCenter
                    leftPadding: coverColorCheck.indicator.width
                                 + coverColorCheck.spacing
                }
            }
            TextField {
                id: coverColorField
                Layout.fillWidth: true
                enabled: coverColorCheck.checked
                placeholderText: "#AARRGGBB"
            }
        }

        Label { text: "Full scan"; color: editor.labelColor }
        RowLayout {
            Layout.fillWidth: true
            CheckBox {
                id: fullScanCheck
                text: "Use Box - Full image"
                palette.windowText: editor.labelColor
                palette.buttonText: editor.labelColor
                palette.text: editor.labelColor
                contentItem: Text {
                    text: fullScanCheck.text
                    font: fullScanCheck.font
                    color: editor.labelColor
                    verticalAlignment: Text.AlignVCenter
                    leftPadding: fullScanCheck.indicator.width
                                 + fullScanCheck.spacing
                }
            }
            CheckBox {
                id: fullScanLandscapeCheck
                text: "Landscape model"
                palette.windowText: editor.labelColor
                palette.buttonText: editor.labelColor
                palette.text: editor.labelColor
                contentItem: Text {
                    text: fullScanLandscapeCheck.text
                    font: fullScanLandscapeCheck.font
                    color: editor.labelColor
                    verticalAlignment: Text.AlignVCenter
                    leftPadding: fullScanLandscapeCheck.indicator.width
                                 + fullScanLandscapeCheck.spacing
                }
            }
        }

        Label { text: "Full-scan spine width"; color: editor.labelColor }
        TextField {
            id: fullScanSpineWidthField
            Layout.fillWidth: true
            placeholderText: "0.143"
            validator: DoubleValidator {
                bottom: 0
                top: 1
                notation: DoubleValidator.StandardNotation
            }
        }

        Label { text: "Forced model size"; color: editor.labelColor }
        RowLayout {
            Layout.fillWidth: true
            CheckBox {
                id: forceSizeCheck
                text: "Force"
                palette.windowText: editor.labelColor
                palette.buttonText: editor.labelColor
                palette.text: editor.labelColor
                contentItem: Text {
                    text: forceSizeCheck.text
                    font: forceSizeCheck.font
                    color: editor.labelColor
                    verticalAlignment: Text.AlignVCenter
                    leftPadding: forceSizeCheck.indicator.width
                                 + forceSizeCheck.spacing
                }
            }
            TextField {
                id: modelSizeXField
                Layout.fillWidth: true
                enabled: forceSizeCheck.checked
                placeholderText: "X"
                validator: DoubleValidator { bottom: 0 }
            }
            TextField {
                id: modelSizeYField
                Layout.fillWidth: true
                enabled: forceSizeCheck.checked
                placeholderText: "Y"
                validator: DoubleValidator { bottom: 0 }
            }
            TextField {
                id: modelSizeZField
                Layout.fillWidth: true
                enabled: forceSizeCheck.checked
                placeholderText: "Z"
                validator: DoubleValidator { bottom: 0 }
            }
        }

        Label { text: "Front-spine resource"; color: editor.labelColor }
        TextField {
            id: frontSpineImageField
            Layout.fillWidth: true
            placeholderText: "Opaque LaunchBox resource value"
        }

        Label { text: "Front-spine material"; color: editor.labelColor }
        CheckBox {
            id: frontSpineClearCheck
            text: "Clear / translucent"
            palette.windowText: editor.labelColor
            palette.buttonText: editor.labelColor
            palette.text: editor.labelColor
            contentItem: Text {
                text: frontSpineClearCheck.text
                font: frontSpineClearCheck.font
                color: editor.labelColor
                verticalAlignment: Text.AlignVCenter
                leftPadding: frontSpineClearCheck.indicator.width
                             + frontSpineClearCheck.spacing
            }
        }

        Label { text: "Logo font"; color: editor.labelColor }
        TextField {
            id: logoFontField
            Layout.fillWidth: true
            placeholderText: "Optional LaunchBox font name"
        }

        Label { text: "Logo rotations"; color: editor.labelColor }
        TextField {
            id: logoRotationField
            Layout.fillWidth: true
            placeholderText: "0,0,0,"
        }

        Label { text: "Spine rotations"; color: editor.labelColor }
        TextField {
            id: spineRotationField
            Layout.fillWidth: true
            placeholderText: "0,,0,"
        }
    }

    Label {
        Layout.fillWidth: true
        enabled: overrideCheck.checked
        text: "Colors accept #AARRGGBB or #RRGGBB. Full-scan width is a 0–1 fraction. Resource strings and sparse rotation slots are preserved as LaunchBox data and are never treated as host paths."
        wrapMode: Text.Wrap
        color: "#7d8590"
    }
}
