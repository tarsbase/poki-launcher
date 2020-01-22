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
import QtQuick.Window 2.2
import PokiLauncher 1.0

Window {
    id: window
    visible: launcher.visible
    width: launcher.window_width
    height: launcher.window_height
    title: qsTr("Poki Launcher")
    flags: Qt.WindowActive | Qt.Dialog | Qt.FramelessWindowHint

    PokiLauncher {
        id: launcher
    }

    Component.onCompleted: {
        launcher.init()
        setX(Screen.width / 2 - width / 2 + Screen.virtualX)
        setY(Screen.height / 2 - height / 2 + Screen.virtualY)
    }

    onVisibleChanged: {
        if (visible) {
            raise()
            requestActivate()
            show()
        }
    }

    onClosing: {
        apps_model.exit()
    }


    MainForm {
        anchors.fill: parent
    }
}
