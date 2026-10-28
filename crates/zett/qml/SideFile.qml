import QtQuick
import QtQuick.Layouts

// A second file beside the editor or page (the file menu's Open to the side): coloured, line numbers,
// read-only; Edit here makes it the main file, × closes it.
Rectangle {
    id: sf
    required property var win
    required property var t
    required property var app
    readonly property string path: win.sideFile
    readonly property string text: path ? app.readFile(path) : ""
    readonly property string ext: path.slice(path.lastIndexOf(".") + 1)
    visible: path.length > 0 && (win.view === "page" || win.view === "file")
    radius: t.rCard
    color: t.panel
    border.width: 1; border.color: Qt.alpha(t.rule, 0.35)

    ColumnLayout {
        anchors { fill: parent; margins: 12 }
        spacing: 8
        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            Text { text: "draft"; color: t.accent; font.family: t.icons; font.pixelSize: 17 }
            Text { Layout.fillWidth: true; text: sf.path.replace(sf.app.dir + "/", ""); elide: Text.ElideMiddle; color: t.text; font.family: t.sans; font.pixelSize: 14; font.weight: Font.DemiBold }
            Repeater {
                model: [["edit", "Edit here", () => { const p = sf.path; sf.win.sideFile = ""; sf.win.openFile({ path: p, rel: p }) }], ["close", "Close", () => sf.win.sideFile = ""]]
                delegate: Rectangle {
                    required property var modelData
                    width: 28; height: 28; radius: 14
                    color: bh.hovered ? t.raised : "transparent"
                    Text { anchors.centerIn: parent; text: modelData[0]; color: t.dim; font.family: t.icons; font.pixelSize: 16 }
                    HoverHandler { id: bh; cursorShape: Qt.PointingHandCursor }
                    TapHandler { onTapped: modelData[2]() }
                }
            }
        }
        Rectangle {
            Layout.fillWidth: true; Layout.fillHeight: true
            radius: 14; color: t.card
            Flickable {
                anchors { fill: parent; margins: 10 }
                clip: true
                contentWidth: Math.max(width, body.implicitWidth + 50)
                contentHeight: body.implicitHeight
                boundsBehavior: Flickable.StopAtBounds
                Text {
                    width: 34; horizontalAlignment: Text.AlignRight
                    text: sf.text.split("\n").map((_, i) => i + 1).join("\n")
                    color: t.faint; font.family: t.mono; font.pixelSize: 12
                    lineHeightMode: Text.FixedHeight; lineHeight: body.contentHeight / Math.max(1, sf.text.split("\n").length)
                }
                Text {
                    id: body
                    x: 44
                    text: sf.visible ? sf.app.highlight(sf.ext, sf.text) : ""
                    textFormat: Text.RichText
                    color: t.text; font.family: t.mono; font.pixelSize: 12
                }
            }
        }
    }
}
