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

    background: Item {
        clip: true

        Item {
            id: outerBlurLayer
            anchors.fill: parent
            opacity: 0.42
            visible: appController.albumArt !== ""
            layer.enabled: visible
            layer.smooth: true
            layer.effect: MultiEffect {
                blurEnabled: true
                blur: 1.0
                blurMax: 64
            }

            Item {
                id: albumCloud
                anchors.fill: parent
                layer.enabled: outerBlurLayer.visible
                layer.smooth: true
                layer.effect: MultiEffect {
                    blurEnabled: true
                    blur: 1.0
                    blurMax: 64
                    saturation: 1.35
                }

                Repeater {
                    model: 4
                    Image {
                        required property int index
                        width: Math.max(root.width, root.height) * 0.82
                        height: width
                        x: index % 2 === 0 ? -width * 0.30 : root.width - width * 0.70
                        y: index < 2 ? -height * 0.34 : root.height - height * 0.66
                        source: appController.albumArt
                        fillMode: Image.PreserveAspectCrop
                        transformOrigin: Item.Center
                        NumberAnimation on rotation {
                            from: index % 2 === 0 ? 0 : 360
                            to: index % 2 === 0 ? 360 : 0
                            duration: 40000
                            loops: Animation.Infinite
                            running: appController.albumArt !== ""
                        }
                    }
                }
            }
        }

        Rectangle {
            anchors.fill: parent
            color: root.palette.window
            opacity: appController.albumArt === "" ? 1.0 : 0.76
        }
    }

    header: ToolBar {
        RowLayout {
            anchors.fill: parent
            Label { text: qsTr("UiPlay"); font.bold: true; Layout.leftMargin: 12; Layout.fillWidth: true }
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
                Layout.rightMargin: 6
            }
        }
    }

    Page {
        anchors.fill: parent
        padding: 32
        background: null

        Rectangle {
            id: contentSurface
            anchors.centerIn: parent
            width: Math.min(parent.width - 40, 850)
            height: Math.min(parent.height - 40, Math.max(310, contentLayout.implicitHeight + 48))
            radius: 22
            color: Qt.rgba(root.palette.window.r, root.palette.window.g, root.palette.window.b, 0.88)
            border.color: Qt.rgba(root.palette.mid.r, root.palette.mid.g, root.palette.mid.b, 0.45)
            border.width: 1
        }

        ColumnLayout {
            id: contentLayout
            anchors.centerIn: parent
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
                    Image {
                        id: albumImage
                        anchors.fill: parent
                        source: appController.albumArt
                        sourceSize.width: 360
                        sourceSize.height: 360
                        fillMode: Image.PreserveAspectCrop
                        visible: false
                    }
                    Rectangle {
                        id: albumMask
                        anchors.fill: parent
                        radius: 18
                        visible: false
                        layer.enabled: true
                    }
                    MultiEffect {
                        anchors.fill: parent
                        source: albumImage
                        maskEnabled: true
                        maskSource: albumMask
                        shadowEnabled: true
                        shadowBlur: 0.8
                        shadowOpacity: 0.35
                        shadowVerticalOffset: 5
                    }
                    Layout.preferredWidth: root.portraitLayout
                                           ? Math.min(280, contentSurface.width * 0.62)
                                           : Math.min(320, root.width * 0.36)
                    Layout.preferredHeight: Layout.preferredWidth
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
                            text: qsTr("Previous")
                            display: AbstractButton.IconOnly
                            onClicked: appController.playbackControl("previous")
                            ToolTip.text: text
                            ToolTip.visible: hovered
                        }
                        ToolButton {
                            icon.name: "media-playback-start"
                            text: qsTr("Play or pause")
                            display: AbstractButton.IconOnly
                            onClicked: appController.playbackControl("playpause")
                            ToolTip.text: text
                            ToolTip.visible: hovered
                        }
                        ToolButton {
                            icon.name: "media-skip-forward"
                            text: qsTr("Next")
                            display: AbstractButton.IconOnly
                            onClicked: appController.playbackControl("next")
                            ToolTip.text: text
                            ToolTip.visible: hovered
                        }
                    }
                }
            }

            Rectangle {
                visible: appController.devices.length > 0
                Layout.fillWidth: true
                implicitHeight: deviceLayout.implicitHeight + 32
                radius: 14
                color: Qt.rgba(root.palette.base.r, root.palette.base.g, root.palette.base.b, 0.60)
                border.color: Qt.rgba(root.palette.mid.r, root.palette.mid.g, root.palette.mid.b, 0.35)
                ColumnLayout {
                    id: deviceLayout
                    anchors.fill: parent
                    anchors.margins: 16
                    Label { text: qsTr("Connected devices"); font.bold: true }
                    Repeater {
                        model: appController.devices
                        delegate: RowLayout {
                            required property var modelData
                            Layout.fillWidth: true
                            Rectangle { width: 8; height: 8; radius: 4; color: modelData.connected ? palette.highlight : palette.mid }
                            Label { text: modelData.name || qsTr("AirPlay device"); font.bold: true }
                            Label { text: modelData.format || ""; opacity: 0.7 }
                            ColumnLayout {
                                spacing: 1
                                Layout.fillWidth: true
                                Label { text: modelData.deviceId; opacity: 0.6; Layout.fillWidth: true }
                                Label {
                                    text: modelData.userAgent || ""
                                    visible: text !== ""
                                    opacity: 0.55
                                    font.pixelSize: 11
                                    Layout.fillWidth: true
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
