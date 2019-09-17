import QtQuick 2.6
import QtQuick.Window 2.2
import PokiLauncher 1.0

Window {
    id: window
    visible: apps_model.visible
    width: 500
    height: 500
    title: qsTr("Hello World")

    AppsModel {
        id: apps_model
    }

    Component.onCompleted: {
        setX(Screen.width / 2 - width / 2 + Screen.virtualX);
        setY(Screen.height / 2 - height / 2 + Screen.virtualY);
    }


    MainForm {
        anchors.fill: parent
    }
}
