import QtQuick
import QtQuick.Layouts

// A file from the tree, as an editor: coloured by language, line numbers (the cursor's lit), the current line,
// Tab / Enter / brackets as in code blocks (win.codeKey), Ctrl+S saves (writeFile, inside the workspace only).
// Main.qml shows it for view "file".
ColumnLayout {
    id: fv
    required property var win
    required property var t
    required property var app
    visible: win.view === "file"
    Layout.fillWidth: true
    Layout.fillHeight: true
    spacing: 8

    readonly property string ext: win.filePath.slice(win.filePath.lastIndexOf(".") + 1)
    property string saved: win.fileText            // what is on disk
    readonly property bool dirty: ed.text !== saved
    property string note: ""
    // the coloured copy follows the text a moment after typing (big files stay quick)
    property string shownText: win.fileText
    Timer { id: rehl; interval: 150; onTriggered: fv.shownText = ed.text }
    Connections { target: fv.win; function onFileTextChanged() { fv.saved = fv.win.fileText; fv.shownText = fv.win.fileText } }

    function save() {
        if (app.writeFile(win.filePath, ed.text)) { saved = ed.text; note = "Saved" } else note = "Could not save"
        noteT.restart()
    }
    function revert() { ed.text = saved; shownText = saved }
    function testEdit(s) { ed.forceActiveFocus(); ed.cursorPosition = 0; ed.insert(0, s); save() }
    function testFind(q) { findBar.open("find"); findBar.q.text = q }
    Timer { id: noteT; interval: 1800; onTriggered: fv.note = "" }

    RowLayout {
        Layout.fillWidth: true
        Layout.leftMargin: 24; Layout.rightMargin: 24
        spacing: 8
        Text { text: "draft"; color: t.accent; font.family: t.icons; font.pixelSize: 18 }
        Text { Layout.fillWidth: true; text: win.filePath.replace(app.dir + "/", "").split("/").join("  ›  ") + (fv.dirty ? "  ●" : ""); elide: Text.ElideMiddle; color: t.text; font.family: t.sans; font.pixelSize: 14; font.weight: Font.DemiBold }
        Text { visible: fv.note.length > 0; text: fv.note; color: t.accent; font.family: t.sans; font.pixelSize: 12 }
        Text { text: `Ln ${ed.line + 1}, Col ${ed.col + 1}  ·  ${ed.lineCount} lines  ·  ${fv.ext}`; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
        Repeater {
            model: fv.dirty ? [["undo", "Revert", () => fv.revert()], ["save", "Save  Ctrl S", () => fv.save()]] : [["open_in_new", "Open with the system", () => Qt.openUrlExternally("file://" + fv.win.filePath)]]
            delegate: Rectangle {
                required property var modelData
                implicitWidth: bRow.implicitWidth + 20; implicitHeight: 28; radius: 14
                color: modelData[0] === "save" ? (bHov.hovered ? Qt.lighter(t.accent, 1.1) : t.accent) : (bHov.hovered ? t.raised : t.card)
                RowLayout { id: bRow; anchors.centerIn: parent; spacing: 5
                    Text { text: modelData[0]; color: modelData[0] === "save" ? t.onAccent : t.dim; font.family: t.icons; font.pixelSize: 15 }
                    Text { text: modelData[1]; color: modelData[0] === "save" ? t.onAccent : t.dim; font.family: t.sans; font.pixelSize: 12 } }
                HoverHandler { id: bHov; cursorShape: Qt.PointingHandCursor }
                TapHandler { onTapped: modelData[2]() }
            }
        }
    }
    // find / replace / go to line (FindBar.qml)
    FindBar { id: findBar; t: fv.t; editor: ed; Layout.fillWidth: true; Layout.leftMargin: 16; Layout.rightMargin: 16 }
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
            contentWidth: Math.max(width, ed.contentWidth + 70)
            contentHeight: ed.contentHeight + 40
            boundsBehavior: Flickable.StopAtBounds
            // keep the cursor in view
            function follow(r) {
                if (height <= 0) return     // opening: not laid out yet, nothing to follow
                if (r.y < contentY) contentY = r.y
                else if (r.y + r.height > contentY + height) contentY = r.y + r.height - height
                if (r.x + 60 < contentX) contentX = Math.max(0, r.x - 40)
                else if (r.x + 60 > contentX + width) contentX = r.x + 60 - width + 40
            }
            // the current line
            Rectangle {
                visible: ed.activeFocus
                x: 0; width: fileFlick.contentWidth
                y: ed.cursorRectangle.y; height: ed.cursorRectangle.height
                color: Qt.alpha(t.text, 0.05)
            }
            // line numbers, the cursor's lit
            Column {
                x: 0; width: 40
                Repeater {
                    model: ed.lineCount
                    delegate: Text {
                        required property int index
                        width: 40; height: ed.lineH
                        horizontalAlignment: Text.AlignRight; verticalAlignment: Text.AlignVCenter
                        text: index + 1
                        color: index === ed.line && ed.activeFocus ? t.accent : t.faint
                        font.family: t.mono; font.pixelSize: 12
                    }
                }
            }
            // the matches of the find bar, the current one stronger
            Repeater {
                model: findBar.matches.slice(0, 500)
                delegate: Rectangle {
                    required property var modelData
                    required property int index
                    readonly property rect a: ed.positionToRectangle(modelData[0])
                    readonly property rect b: ed.positionToRectangle(modelData[1])
                    x: ed.x + a.x; y: a.y; width: Math.max(2, b.x - a.x); height: a.height
                    radius: 3
                    color: index === findBar.cur ? Qt.alpha(t.accent, 0.45) : Qt.alpha(t.accent, 0.18)
                }
            }
            // the coloured copy, under the editor
            Text {
                x: 54
                text: fv.visible ? app.highlight(fv.ext, fv.shownText) : ""
                textFormat: Text.RichText
                color: t.text; font: ed.font
            }
            // the editor: its own glyphs hidden, its cursor and selection shown over the copy
            TextEdit {
                id: ed
                x: 54
                width: Math.max(fileFlick.width - 60, contentWidth)
                text: fv.win.fileText
                color: "transparent"
                selectedTextColor: "transparent"
                selectionColor: Qt.alpha(t.accent, 0.3)
                selectByMouse: true
                wrapMode: TextEdit.NoWrap
                font.family: t.mono; font.pixelSize: 13
                font.features: fv.win.codeLigatures ? {} : { "liga": 0, "calt": 0, "dlig": 0 }
                cursorDelegate: Rectangle { width: 2; color: t.accent; visible: ed.cursorVisible }
                readonly property real lineH: contentHeight / Math.max(1, lineCount)
                readonly property int line: text.slice(0, cursorPosition).split("\n").length - 1
                readonly property int col: cursorPosition - (text.lastIndexOf("\n", cursorPosition - 1) + 1)
                onTextChanged: rehl.restart()
                onCursorRectangleChanged: fileFlick.follow(cursorRectangle)
                Keys.onPressed: (e) => {
                    if (e.key === Qt.Key_S && (e.modifiers & Qt.ControlModifier)) { fv.save(); e.accepted = true; return }
                    if (e.modifiers & Qt.ControlModifier) {
                        const m = { [Qt.Key_F]: "find", [Qt.Key_H]: "replace", [Qt.Key_G]: "line" }[e.key]
                        if (m) { findBar.open(m); e.accepted = true; return }
                    }
                    if (e.key === Qt.Key_Escape && findBar.mode) { findBar.close(); e.accepted = true; return }
                    if (!e.modifiers || (e.modifiers & Qt.ShiftModifier)) fv.win.codeKey(ed, e)
                }
            }
        }
    }
}
