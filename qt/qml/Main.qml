import QtQuick
import QtQuick.Controls
import QtQuick.Effects
import QtQuick.Layouts

ApplicationWindow {
    id: root
    width: 900
    height: 650
    minimumWidth: 680
    minimumHeight: 480
    visible: true
    title: qsTr("UiPlay")
    readonly property bool portraitLayout: width < height
    readonly property real bassLevel: Math.max(
        appController.visualizerLevels[0] || 0,
        appController.visualizerLevels[1] || 0,
        appController.visualizerLevels[2] || 0)
    property bool devicePanelOpen: false
    palette.highlight: appController.accentColor

    component PlaybackVisualizer: Item {
        implicitWidth: 150
        implicitHeight: 38

        Row {
            anchors.fill: parent
            spacing: 5

            Repeater {
                model: 12
                Rectangle {
                    id: visualizerBar
                    required property int index
                    width: 7
                    height: parent.height * (0.08 + 0.92 * (appController.visualizerLevels[index] || 0))
                    anchors.bottom: parent.bottom
                    radius: width / 2
                    color: root.palette.highlight
                    opacity: 0.55 + (index % 4) * 0.1

                    Behavior on height {
                        NumberAnimation { duration: 35; easing.type: Easing.OutCubic }
                    }
                }
            }
        }
    }

    background: Item {
        clip: true

        Item {
            id: outerBlurLayer
            anchors.fill: parent
            opacity: 0.54
            visible: appController.albumArt !== ""
            layer.enabled: visible
            layer.smooth: true
            layer.effect: MultiEffect {
                blurEnabled: true
                blur: 1.0
                blurMax: 72
            }

            Item {
                id: blendedArtwork
                anchors.fill: parent
                anchors.margins: -Math.max(root.width, root.height) * 0.16
                scale: 1.08
                layer.enabled: outerBlurLayer.visible
                layer.smooth: true
                layer.effect: MultiEffect {
                    blurEnabled: true
                    blur: 1.0
                    blurMax: 96
                    saturation: 1.14
                    contrast: 0.04
                }

                Image {
                    id: backgroundArtworkSource
                    anchors.fill: parent
                    source: appController.albumArt
                    fillMode: Image.PreserveAspectCrop
                    sourceSize.width: 640
                    sourceSize.height: 640
                    visible: false
                }

                ShaderEffect {
                    anchors.fill: parent
                    property variant source: backgroundArtworkSource
                    property real time: 0.0
                    property real bass: root.bassLevel
                    fragmentShader: "qrc:/shaders/blended_background.frag.qsb"

                    Behavior on bass {
                        NumberAnimation { duration: 55; easing.type: Easing.OutCubic }
                    }

                    NumberAnimation on time {
                        from: 0.0
                        to: 100.0
                        duration: 240000
                        loops: Animation.Infinite
                        running: outerBlurLayer.visible
                    }
                }

            }
        }

        Rectangle {
            anchors.fill: parent
            visible: appController.albumArt !== ""
            opacity: 0.20
            gradient: Gradient {
                orientation: Gradient.Horizontal
                GradientStop {
                    position: 0.0
                    color: Qt.rgba(appController.accentColor.r, appController.accentColor.g,
                                   appController.accentColor.b, 0.9)
                }
                GradientStop { position: 0.48; color: "transparent" }
                GradientStop {
                    position: 1.0
                    color: Qt.rgba(root.palette.window.r, root.palette.window.g,
                                   root.palette.window.b, 0.8)
                }
            }
        }

        Rectangle {
            anchors.fill: parent
            color: root.palette.window
            opacity: appController.albumArt === "" ? 1.0 : 0.58
        }
    }

    Page {
        id: mainPage
        anchors.fill: parent
        padding: 32
        background: null

        Rectangle {
            id: contentSurface
            anchors.horizontalCenter: parent.horizontalCenter
            y: Math.max(playbackVisualizer.visible ? playbackVisualizer.height + 32 : 20,
                        (parent.height - height
                             - (root.devicePanelOpen ? deviceSurface.height + 12 : 0)) / 2)
            width: Math.min(parent.width - 40, 850)
            height: root.portraitLayout
                    ? Math.max(310, contentLayout.implicitHeight + 48)
                    : Math.min(parent.height - 40, 430)
            radius: 22
            color: Qt.rgba(root.palette.window.r, root.palette.window.g, root.palette.window.b, 0.38)
            border.color: Qt.rgba(root.palette.mid.r, root.palette.mid.g, root.palette.mid.b, 0.28)
            border.width: 1

            ToolButton {
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                anchors.margins: 12
                visible: !root.devicePanelOpen
                icon.name: "go-down"
                text: qsTr("Show devices and settings")
                display: AbstractButton.IconOnly
                onClicked: root.devicePanelOpen = true
                ToolTip.text: text
                ToolTip.visible: hovered
            }
        }

        PlaybackVisualizer {
            id: playbackVisualizer
            anchors.horizontalCenter: contentSurface.horizontalCenter
            anchors.bottom: contentSurface.top
            anchors.bottomMargin: 12
            visible: appController.title !== ""
            z: contentSurface.z + 1
        }

        ColumnLayout {
            id: contentLayout
            anchors.centerIn: contentSurface
            width: contentSurface.width - 48
            spacing: 24

            ColumnLayout {
                visible: appController.title === ""
                Layout.alignment: Qt.AlignHCenter
                spacing: 8
                Label { text: "◉"; font.pixelSize: 72; color: palette.highlight; Layout.alignment: Qt.AlignHCenter }
                Label { text: qsTr("UiPlay"); font.bold: true; font.pixelSize: 36; Layout.alignment: Qt.AlignHCenter }
                Label { text: qsTr("Ready for AirPlay"); opacity: 0.7; font.pixelSize: 18; Layout.alignment: Qt.AlignHCenter }
                Label {
                    text: appController.running ? qsTr("Receiver is running") : qsTr("Receiver is not running")
                    color: appController.running ? palette.highlight : palette.brightText
                    Layout.alignment: Qt.AlignHCenter
                }
            }

            GridLayout {
                visible: appController.title !== ""
                columns: root.portraitLayout ? 1 : 2
                columnSpacing: 32
                rowSpacing: 20
                Layout.fillWidth: true

                Item {
                    readonly property real artExtent: Math.max(1, root.portraitLayout
                                                               ? contentSurface.width - 48
                                                               : contentSurface.height - 48)
                    Image {
                        id: coverSource
                        anchors.fill: parent
                        source: appController.albumArt
                        fillMode: Image.PreserveAspectCrop
                        asynchronous: true
                        cache: true
                    }
                    ShaderEffectSource {
                        id: coverTexture
                        anchors.fill: parent
                        sourceItem: coverSource
                        hideSource: true
                        live: true
                    }
                    ShaderEffect {
                        anchors.fill: parent
                        property variant source: coverTexture
                        property real cornerRadius: 24 / Math.max(1, width)
                        fragmentShader: "qrc:/shaders/rounded_cover.frag.qsb"
                    }
                    Layout.preferredWidth: artExtent
                    Layout.preferredHeight: artExtent
                    Layout.alignment: Qt.AlignHCenter | Qt.AlignVCenter
                }
                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.alignment: Qt.AlignVCenter
                    Label { text: appController.title; font.bold: true; font.pixelSize: 28; wrapMode: Text.Wrap; Layout.fillWidth: true }
                    Label { text: appController.artist; opacity: 0.8; font.pixelSize: 22; wrapMode: Text.Wrap; Layout.fillWidth: true }
                    Label {
                        text: [appController.album, appController.genre].filter(value => value !== "").join(" · ")
                        opacity: 0.65
                        font.pixelSize: 16
                        wrapMode: Text.Wrap
                        Layout.fillWidth: true
                    }
                    ProgressBar {
                        from: 0
                        to: Math.max(1, appController.lengthSeconds)
                        value: appController.progressSeconds
                        Layout.fillWidth: true
                        Layout.topMargin: 16
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        Label { text: appController.progressText; opacity: 0.65 }
                        Item { Layout.fillWidth: true }
                        Label { text: appController.lengthText; opacity: 0.65 }
                    }
                    RowLayout {
                        Layout.alignment: Qt.AlignHCenter
                        ToolButton {
                            icon.name: "media-skip-backward"
                            icon.width: 28
                            icon.height: 28
                            implicitWidth: 54
                            implicitHeight: 54
                            text: qsTr("Previous")
                            display: AbstractButton.IconOnly
                            onClicked: appController.playbackControl("previous")
                            ToolTip.text: text
                            ToolTip.visible: hovered
                        }
                        ToolButton {
                            icon.name: appController.playing ? "media-playback-pause" : "media-playback-start"
                            icon.width: 34
                            icon.height: 34
                            implicitWidth: 64
                            implicitHeight: 64
                            text: appController.playing ? qsTr("Pause") : qsTr("Play")
                            display: AbstractButton.IconOnly
                            onClicked: appController.playbackControl("playpause")
                            ToolTip.text: text
                            ToolTip.visible: hovered
                        }
                        ToolButton {
                            icon.name: "media-skip-forward"
                            icon.width: 28
                            icon.height: 28
                            implicitWidth: 54
                            implicitHeight: 54
                            text: qsTr("Next")
                            display: AbstractButton.IconOnly
                            onClicked: appController.playbackControl("next")
                            ToolTip.text: text
                            ToolTip.visible: hovered
                        }
                    }
                    Label {
                        text: appController.playing ? qsTr("Playing") : qsTr("Paused")
                        color: appController.playing ? palette.highlight : palette.mid
                        font.bold: true
                        font.pixelSize: 11
                        Layout.alignment: Qt.AlignHCenter
                    }
                }
            }

            Rectangle {
                id: deviceSurface
                parent: contentSurface
                x: 0
                y: contentSurface.height + 12
                width: contentSurface.width
                visible: root.devicePanelOpen
                implicitHeight: deviceLayout.implicitHeight + 28
                radius: 18
                color: Qt.rgba(root.palette.window.r, root.palette.window.g, root.palette.window.b, 0.58)
                border.color: Qt.rgba(root.palette.mid.r, root.palette.mid.g, root.palette.mid.b, 0.28)
                ColumnLayout {
                    id: deviceLayout
                    anchors.fill: parent
                    anchors.margins: 14
                    spacing: 10

                    RowLayout {
                        Layout.fillWidth: true
                        Label {
                            text: qsTr("AirPlay devices")
                            font.bold: true
                            font.pixelSize: 15
                            Layout.fillWidth: true
                        }
                        Label {
                            text: appController.devices.length === 0
                                  ? qsTr("Waiting")
                                  : qsTr("%1 known").arg(appController.devices.length)
                            opacity: 0.55
                            font.pixelSize: 12
                        }
                        ToolButton {
                            icon.name: "go-up"
                            text: qsTr("Hide device panel")
                            display: AbstractButton.IconOnly
                            onClicked: root.devicePanelOpen = false
                            ToolTip.text: text
                            ToolTip.visible: hovered
                        }
                    }

                    Repeater {
                        model: appController.devices
                        delegate: Rectangle {
                            required property var modelData
                            Layout.fillWidth: true
                            implicitHeight: deviceRow.implicitHeight + 20
                            radius: 12
                            color: Qt.rgba(root.palette.window.r, root.palette.window.g,
                                           root.palette.window.b, 0.72)
                            border.color: Qt.rgba(root.palette.mid.r, root.palette.mid.g,
                                                 root.palette.mid.b, 0.22)

                            RowLayout {
                                id: deviceRow
                                anchors.fill: parent
                                anchors.margins: 10
                                spacing: 10

                                Rectangle {
                                    width: 34
                                    height: 34
                                    radius: 10
                                    color: Qt.rgba(root.palette.highlight.r, root.palette.highlight.g,
                                                   root.palette.highlight.b, modelData.connected ? 0.18 : 0.08)
                                    Label {
                                        anchors.centerIn: parent
                                        text: "◉"
                                        color: modelData.connected ? root.palette.highlight : root.palette.mid
                                        font.pixelSize: 17
                                    }
                                }

                                ColumnLayout {
                                    spacing: 2
                                    Layout.fillWidth: true
                                    Label {
                                        text: modelData.name || qsTr("AirPlay device")
                                        font.bold: true
                                        elide: Text.ElideRight
                                        Layout.fillWidth: true
                                    }
                                    Label {
                                        text: [modelData.format || "", modelData.userAgent || ""]
                                              .filter(value => value !== "").join(" · ")
                                        visible: text !== ""
                                        opacity: 0.58
                                        font.pixelSize: 11
                                        elide: Text.ElideRight
                                        Layout.fillWidth: true
                                    }
                                    Label {
                                        text: modelData.deviceId
                                        opacity: 0.42
                                        font.pixelSize: 10
                                        elide: Text.ElideMiddle
                                        Layout.fillWidth: true
                                    }
                                }

                                Rectangle {
                                    implicitWidth: connectionLabel.implicitWidth + 16
                                    implicitHeight: 24
                                    radius: 12
                                    color: Qt.rgba(root.palette.highlight.r, root.palette.highlight.g,
                                                   root.palette.highlight.b, modelData.connected ? 0.16 : 0.06)
                                    Label {
                                        id: connectionLabel
                                        anchors.centerIn: parent
                                        text: modelData.connected ? qsTr("Connected") : qsTr("Offline")
                                        color: modelData.connected ? root.palette.highlight : root.palette.mid
                                        font.pixelSize: 11
                                        font.bold: true
                                    }
                                }

                                ToolButton {
                                    visible: !modelData.connected
                                    icon.name: "edit-delete"
                                    text: qsTr("Forget device")
                                    display: AbstractButton.IconOnly
                                    onClicked: appController.forgetDevice(modelData.deviceId)
                                    ToolTip.text: text
                                    ToolTip.visible: hovered
                                }
                            }
                        }
                    }

                    RowLayout {
                        Layout.fillWidth: true
                        Label {
                            text: appController.running ? qsTr("Receiver active") : qsTr("Receiver stopped")
                            opacity: 0.52
                            font.pixelSize: 11
                            Layout.fillWidth: true
                        }
                        ToolButton {
                            icon.name: "utilities-terminal"
                            text: qsTr("Terminal")
                            display: AbstractButton.IconOnly
                            onClicked: terminalDialog.open()
                            ToolTip.text: text
                            ToolTip.visible: hovered
                        }
                        ToolButton {
                            icon.name: "settings-configure"
                            text: qsTr("Settings")
                            display: AbstractButton.IconOnly
                            onClicked: settingsDialog.open()
                            ToolTip.text: text
                            ToolTip.visible: hovered
                        }
                    }
                }
            }
        }
    }

    Dialog {
        id: settingsDialog
        title: qsTr("Settings")
        anchors.centerIn: parent
        width: Math.min(520, root.width - 48)
        modal: true
        standardButtons: Dialog.Close
        background: Rectangle {
            radius: 16
            color: root.palette.window
            border.color: root.palette.mid
        }
        ColumnLayout {
            width: parent.width
            spacing: 10
            Label { text: qsTr("AirPlay receiver name") }
            TextField {
                id: receiverNameField
                text: appController.receiverName
                placeholderText: qsTr("Receiver name")
                Layout.fillWidth: true
            }
            Label { text: qsTr("Provider"); Layout.topMargin: 8 }
            ComboBox {
                id: providerBox
                model: [qsTr("Shairport Sync (recommended)"), qsTr("UxPlay")]
                currentIndex: appController.provider === "shairport" ? 0 : 1
                Layout.fillWidth: true
            }
            Button {
                text: qsTr("Save and restart receiver")
                icon.name: "document-save"
                Layout.alignment: Qt.AlignRight
                Layout.topMargin: 12
                onClicked: {
                    appController.receiverName = receiverNameField.text
                    appController.provider = providerBox.currentIndex === 0 ? "shairport" : "uxplay"
                    appController.restartReceiver()
                    settingsDialog.close()
                }
            }
        }
    }

    Dialog {
        id: terminalDialog
        title: qsTr("Receiver terminal")
        anchors.centerIn: parent
        width: Math.min(780, root.width - 48)
        height: Math.min(520, root.height - 48)
        modal: true
        standardButtons: Dialog.Close
        background: Rectangle {
            radius: 16
            color: root.palette.window
            border.color: root.palette.mid
        }
        ScrollView {
            anchors.fill: parent
            TextArea {
                text: appController.logs
                font.family: "monospace"
                readOnly: true
                selectByMouse: true
                wrapMode: TextEdit.NoWrap
            }
        }
    }
}
