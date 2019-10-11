import QtQuick 2.6
import QtQuick.Window 2.2
import PokiLauncher 1.0

Window {
    id: window
    visible: apps_model.visible
    width: 500
    height: 500
    title: qsTr("Poki Launcher")
    flags: Qt.WindowActive //| Qt.WindowStaysOnTopHint

    AppsModel {
        id: apps_model
    }

    Component.onCompleted: {
        setX(Screen.width / 2 - width / 2 + Screen.virtualX);
        setY(Screen.height / 2 - height / 2 + Screen.virtualY);
    }

    onVisibleChanged: {
        if (visible) {
            raise();
            requestActivate();
            show()
        }
    }

    onClosing: {
        apps_model.exit();
    }


    MainForm {
        anchors.fill: parent
    }
}
