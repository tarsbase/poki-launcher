/***
 * This file is part of Poki Launcher.
 *
 * Poki Launcher is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Poki Launcher is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Poki Launcher.  If not, see <https://www.gnu.org/licenses/>.
 */
import QtQuick 2.6
import QtQuick.Layouts 1.0
import QtQuick.Controls 1.1

Rectangle {
    color: launcher.background_color
	border.color: launcher.border_color
	border.width: 2

	function run() {
		launcher.run();
		input.clear();
		launcher.hide();
	}

	function scan() {
		window.title = qsTr("Poki Launcher - Scanning...");
		launcher.scan();
		window.title = qsTr("Poki Launcher");
		launcher.search(input.text)
	}

	function hide() {
		input.clear();
		launcher.hide();
	}

	Shortcut {
		sequence: "F5"
		onActivated: scan()
	}

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 0
        spacing: 0

		Rectangle {
			id: input_box
			color: launcher.input_box_color
			radius: 0
			Layout.preferredWidth: window.width
			Layout.preferredHeight: window.height * 0.1
			Layout.alignment: Qt.AlignHCenter

			TextInput {
				id: input
				focus: true
				color: launcher.input_text_color
				padding: 10
				anchors.verticalCenter: input_box.verticalCenter
				// font.pixelSize: window.height * 0.1 * 0.4
				font.pointSize: launcher.input_font_size
				onTextChanged: launcher.search(text)
				Keys.onUpPressed: launcher.up()
				Keys.onDownPressed: launcher.down()
				Keys.onReturnPressed: run()
				Keys.onEscapePressed: hide()
			}

			BusyIndicator {
				id: scan_ind
				running: launcher.scanning
				anchors.right: input_box.right
				anchors.verticalCenter: input_box.verticalCenter
				anchors.rightMargin: input_box.height * 0.1
				height: input_box.height * 0.8
				width: input_box.height * 0.8
			}
		}

        ListView {
            id: app_list
			Layout.alignment: Qt.AlignHCenter
			Layout.preferredWidth: window.width
			Layout.preferredHeight: window.height * 0.9
			interactive: false

			model: launcher.model
			delegate: Item {
				height: app_list.height * 0.2
				width: window.width

				Rectangle {
					id: item
					anchors.fill: parent
					anchors.topMargin: 1
					anchors.bottomMargin: 1
					color: (uuid == launcher.selected) ? launcher.selected_app_color : launcher.background_color
					RowLayout {
						anchors.fill: parent

						Image {
							asynchronous: true
							Layout.preferredWidth: item.height * 0.8
							Layout.preferredHeight: item.height * 0.8
							Layout.alignment: Qt.AlignLeft
							fillMode: Image.PreserveAspectFit
							source: "image://icon/" + icon
						}

						Text {
							Layout.alignment: Qt.AlignLeft
							color: launcher.app_text_color
							text: name
							font.pointSize: launcher.app_font_size
							// font.pixelSize: item.height * 0.4
						}
					}
				}

				Rectangle {
					height: 1
					color: launcher.app_separator_color
					anchors {
						left: item.left
						right: item.right
						bottom: item.top
					}
				}

				MouseArea {
					anchors.fill: parent
					onClicked: {
						launcher.selected = uuid
						run()
					}
				}
			}
        }
    }
}
