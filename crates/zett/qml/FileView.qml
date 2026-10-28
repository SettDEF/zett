import QtQuick
import QtQuick.Layouts

// A file from the tree, read-only and highlighted, with line numbers (Main.qml shows it for view "file").
ColumnLayout {
    required property var win
    required property var t
    required property var app
    visible: win.view === "file"
    Layout.fillWidth: true
    Layout.fillHeight: true
    spacing: 8
    RowLayout {
        Layout.fillWidth: true
        Layout.leftMargin: 24; Layout.rightMargin: 24
        spacing: 8
        Text { text: "draft"; color: t.accent; font.family: t.icons; font.pixelSize: 18 }
        Text { Layout.fillWidth: true; text: win.filePath.replace(app.dir + "/", "").split("/").join("  ›  "); elide: Text.ElideMiddle; color: t.text; font.family: t.sans; font.pixelSize: 14; font.weight: Font.DemiBold }
        Text { text: `${win.fileText.split("\n").length} lines`; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
        Rectangle {
            implicitWidth: oRow.implicitWidth + 20; implicitHeight: 28; radius: 14
            color: oHov.hovered ? t.raised : t.card
            RowLayout { id: oRow; anchors.centerIn: parent; spacing: 5
                Text { text: "open_in_new"; color: t.dim; font.family: t.icons; font.pixelSize: 15 }
                Text { text: "Open with the system"; color: t.dim; font.family: t.sans; font.pixelSize: 12 } }
            HoverHandler { id: oHov; cursorShape: Qt.PointingHandCursor }
            TapHandler { onTapped: Qt.openUrlExternally("file://" + win.filePath) }
        }
    }
    Rectangle {
        Layout.fillWidth: true
        Layout.fillHeight: true
        Layout.leftMargin: 16; Layout.rightMargin: 16
        radius: 17
        color: t.card
        Flickable {
            id: fileFlick
            anchors { fill: parent; margins: 14 }
            clip: true
            contentWidth: Math.max(width, fileBody.implicitWidth + 50)
            contentHeight: fileBody.implicitHeight
            boundsBehavior: Flickable.StopAtBounds
            // line numbers beside the code, both in the cell font
            Text {
                id: gutter
                text: win.fileText.split("\n").map((_, i) => i + 1).join("\n")
                color: t.faint; font.family: t.mono; font.pixelSize: 13
                horizontalAlignment: Text.AlignRight
                width: 40
            }
            Text {
                id: fileBody
                x: 52
                text: win.view === "file" ? app.highlight(win.filePath.slice(win.filePath.lastIndexOf(".") + 1), win.fileText) : ""
                textFormat: Text.RichText
                color: t.text; font.family: t.mono; font.pixelSize: 13
            }
        }
    }
}
