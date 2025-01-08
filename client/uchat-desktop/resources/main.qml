import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    width: 400
    height: 300
    color: "#ffffff"

    Column {
        anchors.centerIn: parent
        spacing: 10

        TextField {
            id: userIdField
            placeholderText: "User ID (数字)"
        }

        TextField {
            id: passwordField
            placeholderText: "Password"
            echoMode: TextInput.Password
        }

        Row {
            spacing: 10
            Button {
                text: "Login"
                onClicked: {
                    // 调用 Rust 端暴露的 doLogin 方法
                    myGuiObject.doLogin(userIdField.text, passwordField.text)
                }
            }

            Button {
                text: "Online Users"
                onClicked: {
                    myGuiObject.getOnlineUsers()
                }
            }
        }

        TextArea {
            id: messageArea
            wrapMode: TextArea.Wrap
            text: "等待服务器消息...\n"
            readOnly: true
            width: 360
            height: 130
        }
    }
}
