import QtQuick 2.6
import QtQuick.Window 2.2

Window {
    id: window
    visible: true
    width: 500
    height: 300
    title: qsTr("Hello World")

    MainForm {
        anchors.fill: parent
    }
}
