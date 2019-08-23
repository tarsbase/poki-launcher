import QtQuick 2.6
import QtQuick.Window 2.2

Window {
    id: window
    visible: true
    width: 500
    height: 500
    title: qsTr("Hello World")

    Component.onCompleted: {
        setX(Screen.width / 2 - width / 2 + Screen.virtualX);
        setY(Screen.height / 2 - height / 2 + Screen.virtualY);
    }


    MainForm {
        anchors.fill: parent
    }
}
