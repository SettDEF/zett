import QtQuick
import QtQuick.Layouts

// A second file beside the editor or page (split editor: Ctrl+\ in a file, or the file menu's Open to the side):
// coloured, line numbers, editable (Ctrl+S saves, ● while unsaved); the same file as the main editor is shown
// read-only, so two buffers never overwrite each other. Edit here makes it the main file, × closes it.
Rectangle {
    id: sf
    required property var win
    required property var t
    required property var app
    readonly property string path: win.sideFile
    readonly property bool same: path === win.filePath && win.view === "file"
    property string saved: ""
    property string shownText: ""
    readonly property bool dirty: sideEd.text !== saved
    readonly property string ext: path.slice(path.lastIndexOf(".") + 1)
    function load() { saved = path ? app.readFile(path) : ""; sideEd.text = saved; shownText = saved }
    onPathChanged: load()
    Component.onCompleted: load()
    function save() { if (!same && app.writeFile(path, sideEd.text)) { saved = sideEd.text; if (win.filePath === path) win.fileText = saved } }
    Timer { id: rehl; interval: 150; onTriggered: sf.shownText = sideEd.text }
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
            Text { Layout.fillWidth: true; text: sf.path.replace(sf.app.dir + "/", "") + (sf.dirty ? "  ●" : "") + (sf.same ? "  (read-only here)" : ""); elide: Text.ElideMiddle; color: t.text; font.family: t.sans; font.pixelSize: 14; font.weight: Font.DemiBold }
            Repeater {
                model: (sf.dirty ? [["save", "Save  Ctrl S", () => sf.save()]] : []).concat([["edit", "Edit here", () => { const p = sf.path; if (sf.dirty) sf.save(); sf.win.sideFile = ""; sf.win.openFile({ path: p, rel: p }) }], ["close", "Close", () => sf.win.sideFile = ""]])
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
                id: sideFlick
                anchors { fill: parent; margins: 10 }
                clip: true
                contentWidth: Math.max(width, sideEd.contentWidth + 50)
                contentHeight: sideEd.contentHeight + 20
                boundsBehavior: Flickable.StopAtBounds
                Text {
                    width: 34; horizontalAlignment: Text.AlignRight
                    text: sf.shownText.split("\n").map((_, i) => i + 1).join("\n")
                    color: t.faint; font.family: t.mono; font.pixelSize: 12
                    lineHeightMode: Text.FixedHeight; lineHeight: sideEd.contentHeight / Math.max(1, sideEd.lineCount)
                }
                // the coloured copy, under the editor
                Text {
                    x: 44
                    text: sf.visible ? sf.app.highlight(sf.ext, sf.shownText) : ""
                    textFormat: Text.RichText
                    color: t.text; font: sideEd.font
                }
                // the editor: its glyphs hidden, its cursor and selection over the copy
                TextEdit {
                    id: sideEd
                    x: 44
                    width: Math.max(sideFlick.width - 50, contentWidth)
                    readOnly: sf.same
                    color: "transparent"
                    selectedTextColor: "transparent"
                    selectionColor: Qt.alpha(t.accent, 0.3)
                    selectByMouse: true
                    wrapMode: TextEdit.NoWrap
                    font.family: t.mono; font.pixelSize: 12
                    cursorDelegate: Rectangle { width: 2; color: t.accent; visible: sideEd.cursorVisible }
                    onTextChanged: rehl.restart()
                    onCursorRectangleChanged: {
                        const r = cursorRectangle
                        if (r.y < sideFlick.contentY) sideFlick.contentY = r.y
                        else if (r.y + r.height > sideFlick.contentY + sideFlick.height) sideFlick.contentY = r.y + r.height - sideFlick.height
                    }
                    Keys.onPressed: (e) => {
                        if (e.key === Qt.Key_S && (e.modifiers & Qt.ControlModifier)) { sf.save(); e.accepted = true }
                        else if (e.key === Qt.Key_Tab && !sf.same) { sideEd.insert(sideEd.cursorPosition, "    "); e.accepted = true }
                    }
                }
            }
        }
    }
}
