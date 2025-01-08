import QtQuick 2.12
import QtQuick.Controls 2.12
import QtQuick.Window 2.12

// This must match the uri and version
// specified in the qml_module in the build.rs script.
import com.kdab.cxx_qt.demo 1.0

ApplicationWindow {
    height: 480
    title: qsTr("Client")
    visible: true
    width: 640
    color: palette.window

    Client {
        id: client
        number: 1
        string: qsTr("My String with my number: %1").arg(client.number)
    }

    Column {
        anchors.fill: parent
        anchors.margins: 10
        spacing: 10

        Label {
            text: qsTr("Number: %1").arg(client.number)
            color: palette.text
        }

        Label {
            text: qsTr("String: %1").arg(client.string)
            color: palette.text
        }

        Button {
            text: qsTr("Increment Number")

            onClicked: client.incrementNumber()
        }

        Button {
            text: qsTr("Say Hi!")

            onClicked: client.sayHi(client.string, client.number)
        }

        Button {
            text: qsTr("Quit")

            onClicked: Qt.quit()
        }
    }
}
