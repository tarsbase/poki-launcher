import QtQuick 2.13
import PokiLauncher 1.0
import QtQuick.Layouts 1.13

Rectangle {
    AppsModel {
        id: apps_model
    }

    color: "#282a36"

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 0
        spacing: 20

		Rectangle {
			id: input_box
			color: "#44475a"
			radius: 0
			Layout.preferredWidth: window.width
			Layout.preferredHeight: 50
			Layout.alignment: Qt.AlignHCenter

			TextInput {
				id: input
				focus: true
				color: "#f8f8f2"
				padding: 10
				anchors.verticalCenter: input_box.verticalCenter
			}
		}

        ListView {
            id: app_list
			Layout.alignment: Qt.AlignHCenter
			width: window.width
			height: window.height * 0.8

			model: apps_model
			delegate: Item {
				height: 120
				width: window.width

				Rectangle {
					anchors.fill: parent
					anchors.topMargin: 1
					anchors.bottomMargin: 1
					id: item
					color: "#282a36"
					RowLayout {
						anchors.fill: parent

						Image {
							//asynchronous: true
							Layout.preferredWidth: 100
							Layout.preferredHeight: 100
							fillMode: Image.PreserveAspectFit
							source: icon
						}

						Text {
							Layout.alignment: Qt.AlignLeft
							color: "#f8f8f2"
							text: name
						}
					}
				}

				Rectangle {
					height: 1
					color: "#bd93f9"
					anchors {
						left: item.left
						right: item.right
						bottom: item.top
					}
				}
			}
        }
    }
}
