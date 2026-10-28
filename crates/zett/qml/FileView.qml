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

    // Ctrl+S: with Format on save (Settings) the server formats first, the save follows its edits (or 1.5 s)
    property bool savePending: false
    Timer { id: saveWait; interval: 1500; onTriggered: if (fv.savePending) { fv.savePending = false; fv.write() } }
    function save() {
        if (win.prefs.formatOnSave && lsp && !savePending) { savePending = true; ask("format", 0); saveWait.restart(); return }
        write()
    }
    function write() {
        if (app.writeFile(win.filePath, ed.text)) { saved = ed.text; note = "Saved"; loadGit(); if (lsp) app.lspAsk("save", win.filePath, 0, 0) } else note = "Could not save"
        noteT.restart()
    }
    function revert() { ed.text = saved; shownText = saved }
    function testEdit(s) { ed.forceActiveFocus(); ed.cursorPosition = 0; ed.insert(0, s); save() }
    function testFind(q) { findBar.open("find"); findBar.q.text = q }
    function testProblems() { showProblems = true }
    Timer { id: noteT; interval: 1800; onTriggered: fv.note = "" }

    // ── the language server (Rust: lsp.rs): diagnostics, completion, hover, definition, format ──
    property bool lsp: false
    property var diags: []                 // this file's: {line, col, endLine, endCol, severity, message}
    property var comps: []                 // completion items shown
    property int compSel: 0
    property int compFrom: -1              // where the word being completed starts
    property string hoverText: ""
    property point hoverAt: Qt.point(0, 0)
    readonly property int errors: diags.filter(d => d.severity === 1).length
    readonly property int warnings: diags.filter(d => d.severity === 2).length
    function lineCol(pos) { const before = ed.text.slice(0, pos); const l = before.split("\n").length - 1; return [l, pos - (before.lastIndexOf("\n") + 1)] }
    function posOf(line, col) { const ls = ed.text.split("\n"); let p = 0; for (let i = 0; i < Math.min(line, ls.length - 1); i++) p += ls[i].length + 1; return p + col }
    function wordStart(pos) { let s = pos; while (s > 0 && /[\w$]/.test(ed.text[s - 1])) s--; return s }
    function ask(kind, pos) { if (!lsp) return; const [l, c] = lineCol(pos); app.lspAsk(kind, win.filePath, l, c) }
    onVisibleChanged: if (visible) { openLsp(); loadGit() }
    Connections { target: fv.win; function onFilePathChanged() { fv.diags = []; fv.comps = []; fv.openLsp(); fv.loadGit() } }
    // git: the lines changed since the last commit, as bars in the gutter (green added, blue changed, red removed)
    property var gitMarks: []
    function loadGit() { try { gitMarks = visible && win.filePath ? JSON.parse(app.gitLines(win.filePath)) : [] } catch (e) { gitMarks = [] } }
    function gitAt(i) { for (const m of gitMarks) { if (m.kind === "removed" ? m.line === i : (i >= m.line && i < m.line + m.count)) return m.kind } return "" }
    function openLsp() { if (visible && win.filePath) lsp = app.lspOpen(win.filePath, win.fileText) }
    Timer { id: lspSync; interval: 300; onTriggered: if (fv.lsp) fv.app.lspChange(fv.win.filePath, ed.text) }
    Connections {
        target: fv.app
        function onLspEvent(json) {
            let e; try { e = JSON.parse(json) } catch (x) { return }
            if (e.type === "diagnostics" && e.path === fv.win.filePath) fv.diags = e.items
            else if (e.type === "completion") {
                const word = ed.text.slice(fv.compFrom, ed.cursorPosition).toLowerCase()
                fv.comps = e.items.filter(i => (i.filter || i.label).toLowerCase().includes(word)).sort((a, b) => (a.sort || a.label).localeCompare(b.sort || b.label)).slice(0, 50)
                fv.compSel = 0
            }
            else if (e.type === "hover") fv.hoverText = e.text.replace(/```\w*\n?/g, "").trim()
            else if (e.type === "definition" && e.items.length) {
                const d = e.items[0]
                if (d.path !== fv.win.filePath) fv.win.openFile({ path: d.path, rel: d.path })
                Qt.callLater(() => { ed.cursorPosition = fv.posOf(d.line, d.col); ed.forceActiveFocus() })
            }
            else if (e.type === "edits") for (const f of e.files) if (f.path === fv.win.filePath) {
                // from the end, so earlier positions stay right
                for (const x of f.edits.slice().reverse()) { const a = fv.posOf(x[0], x[1]), b = fv.posOf(x[2], x[3]); ed.remove(a, b); ed.insert(a, x[4]) }
            }
            if (e.type === "edits" && fv.savePending) { fv.savePending = false; saveWait.stop(); fv.write() }
        }
    }
    function takeComp(i) {
        const it = comps[i]; if (!it) return
        ed.remove(compFrom, ed.cursorPosition); ed.insert(compFrom, it.insert || it.label)
        comps = []
    }

    RowLayout {
        Layout.fillWidth: true
        Layout.leftMargin: 24; Layout.rightMargin: 24
        spacing: 8
        Text { text: "draft"; color: t.accent; font.family: t.icons; font.pixelSize: 18 }
        Text { Layout.fillWidth: true; text: win.filePath.replace(app.dir + "/", "").split("/").join("  ›  ") + (fv.dirty ? "  ●" : ""); elide: Text.ElideMiddle; color: t.text; font.family: t.sans; font.pixelSize: 14; font.weight: Font.DemiBold }
        Text { visible: fv.note.length > 0; text: fv.note; color: t.accent; font.family: t.sans; font.pixelSize: 12 }
        // the server: its errors and warnings for this file
        Text { visible: fv.lsp; text: fv.errors + fv.warnings ? `${fv.errors} errors  ${fv.warnings} warnings` : "no problems"; color: fv.errors ? t.removed : fv.warnings ? "#d8a54a" : t.faint; font.family: t.mono; font.pixelSize: 11 
            HoverHandler { cursorShape: fv.diags.length ? Qt.PointingHandCursor : Qt.ArrowCursor }
            TapHandler { onTapped: fv.showProblems = !fv.showProblems }
        }
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
                        readonly property string git: fv.gitAt(index)
                        Rectangle { visible: parent.git === "added" || parent.git === "changed"; x: 44; width: 3; height: parent.height; color: parent.git === "added" ? "#5fb98a" : "#5b9bd5" }
                        Rectangle { visible: parent.git === "removed"; x: 43; y: -3; width: 6; height: 6; rotation: 45; color: fv.t.removed }
                        readonly property int sev: { let s = 9; for (const d of fv.diags) if (d.line === index && d.severity < s) s = d.severity; return s }
                        Rectangle { visible: parent.sev <= 2; x: 2; anchors.verticalCenter: parent.verticalCenter; width: 6; height: 6; radius: 3; color: parent.sev === 1 ? fv.t.removed : "#d8a54a" }
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
            // diagnostics: errors red, warnings yellow, under their range
            Repeater {
                model: fv.diags.slice(0, 300)
                delegate: Rectangle {
                    required property var modelData
                    readonly property rect a: ed.positionToRectangle(fv.posOf(modelData.line, modelData.col))
                    readonly property rect b: ed.positionToRectangle(fv.posOf(modelData.endLine, modelData.endCol))
                    x: ed.x + a.x; y: a.y + a.height - 2
                    width: Math.max(6, (b.y === a.y ? b.x : ed.contentWidth) - a.x); height: 2; radius: 1
                    color: modelData.severity === 1 ? t.removed : modelData.severity === 2 ? "#d8a54a" : t.faint
                }
            }
            // the coloured copy, under the editor
            Text {
                x: 54
                text: fv.visible ? app.highlight(fv.ext, fv.shownText) : ""
                textFormat: Text.RichText
                color: t.text; font: ed.font
            }
            // completion at the cursor
            Rectangle {
                visible: fv.comps.length > 0 && ed.activeFocus
                x: ed.x + ed.cursorRectangle.x; y: ed.cursorRectangle.y + ed.cursorRectangle.height + 4
                width: 380; height: Math.min(8, fv.comps.length) * 30 + 8; radius: 12
                color: t.raised; border.width: 1; border.color: Qt.alpha(t.rule, 0.4)
                z: 50
                ListView {
                    id: compList
                    anchors { fill: parent; margins: 4 }
                    clip: true
                    model: fv.comps
                    currentIndex: fv.compSel
                    onCurrentIndexChanged: positionViewAtIndex(currentIndex, ListView.Contain)
                    delegate: Rectangle {
                        required property var modelData
                        required property int index
                        width: ListView.view.width; height: 30; radius: 8
                        color: index === fv.compSel ? Qt.alpha(t.accent, 0.18) : "transparent"
                        Text { id: ck; x: 8; anchors.verticalCenter: parent.verticalCenter; text: ({ 2: "function", 3: "function", 4: "build", 5: "data_object", 6: "variable_insert", 7: "category", 9: "inventory_2", 10: "tune", 13: "list", 14: "key", 15: "content_paste", 22: "category" })[modelData.kind] ?? "label"; color: t.accent; font.family: t.icons; font.pixelSize: 15 }
                        Text { anchors { left: ck.right; leftMargin: 8; verticalCenter: parent.verticalCenter } width: 170; elide: Text.ElideRight; text: modelData.label; color: t.text; font.family: t.mono; font.pixelSize: 13 }
                        Text { anchors { right: parent.right; rightMargin: 8; verticalCenter: parent.verticalCenter } width: 150; horizontalAlignment: Text.AlignRight; elide: Text.ElideRight; text: modelData.detail; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
                        TapHandler { onTapped: fv.takeComp(index) }
                    }
                }
            }
            // hover: the server's note on the word under the mouse
            Rectangle {
                visible: fv.hoverText.length > 0
                x: Math.min(fv.hoverAt.x, fileFlick.contentWidth - width); y: fv.hoverAt.y + 22
                width: Math.min(560, hoverT.implicitWidth + 24); height: Math.min(280, hoverT.implicitHeight + 16); radius: 12
                color: t.raised; border.width: 1; border.color: Qt.alpha(t.rule, 0.4)
                clip: true
                z: 50
                Text { id: hoverT; x: 12; y: 8; width: Math.min(536, implicitWidth); text: fv.hoverText; wrapMode: Text.Wrap; color: t.text; font.family: t.mono; font.pixelSize: 12 }
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
                onTextChanged: {
                    rehl.restart(); lspSync.restart()
                    // completion: after . or :: it opens; while open it follows the word typed
                    const p = cursorPosition, prev = text[p - 1] ?? ""
                    if (fv.lsp && activeFocus && (prev === "." || text.slice(p - 2, p) === "::")) { fv.compFrom = p; lspSync.stop(); fv.app.lspChange(fv.win.filePath, text); fv.ask("completion", p) }
                    else if (fv.comps.length && /[\w$]/.test(prev)) { lspSync.stop(); fv.app.lspChange(fv.win.filePath, text); fv.ask("completion", p) }
                    else if (fv.comps.length) fv.comps = []
                }
                // hover: the mouse resting on a word asks the server about it
                HoverHandler { id: edHover; onPointChanged: { fv.hoverText = ""; hoverAsk.restart() } }
                Timer { id: hoverAsk; interval: 650; onTriggered: if (edHover.hovered && fv.lsp) { const pt = edHover.point.position; fv.hoverAt = Qt.point(ed.x + pt.x, pt.y); fv.ask("hover", ed.positionAt(pt.x, pt.y)) } }
                // Ctrl+click: go to the definition
                TapHandler { acceptedModifiers: Qt.ControlModifier; onTapped: (ev) => fv.ask("definition", ed.positionAt(ev.position.x, ev.position.y)) }
                onCursorRectangleChanged: fileFlick.follow(cursorRectangle)
                Keys.onPressed: (e) => {
                // the completion list takes ↑ ↓ Enter Tab Esc while open
                    if (fv.comps.length) {
                        if (e.key === Qt.Key_Down) { fv.compSel = (fv.compSel + 1) % fv.comps.length; e.accepted = true; return }
                        if (e.key === Qt.Key_Up) { fv.compSel = (fv.compSel - 1 + fv.comps.length) % fv.comps.length; e.accepted = true; return }
                        if (e.key === Qt.Key_Return || e.key === Qt.Key_Enter || e.key === Qt.Key_Tab) { fv.takeComp(fv.compSel); e.accepted = true; return }
                        if (e.key === Qt.Key_Escape) { fv.comps = []; e.accepted = true; return }
                    }
                    if (e.key === Qt.Key_Space && (e.modifiers & Qt.ControlModifier)) { fv.compFrom = fv.wordStart(ed.cursorPosition); fv.ask("completion", ed.cursorPosition); e.accepted = true; return }
                    if (e.key === Qt.Key_F12) { fv.ask("definition", ed.cursorPosition); e.accepted = true; return }
                    if (e.key === Qt.Key_F && (e.modifiers & Qt.AltModifier) && (e.modifiers & Qt.ShiftModifier)) { fv.ask("format", 0); e.accepted = true; return }
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
    // the problems of this file (click the counts in the header): a click jumps there
    property bool showProblems: false
    Rectangle {
        visible: fv.showProblems && fv.diags.length > 0
        Layout.fillWidth: true; Layout.leftMargin: 16; Layout.rightMargin: 16
        implicitHeight: Math.min(180, probCol.implicitHeight + 16)
        radius: 14; color: t.card
        Flickable {
            anchors { fill: parent; margins: 8 }
            clip: true; contentHeight: probCol.implicitHeight
            Column {
                id: probCol
                width: parent.width
                Repeater {
                    model: fv.diags.slice().sort((a, b) => a.severity - b.severity || a.line - b.line)
                    delegate: Rectangle {
                        required property var modelData
                        width: probCol.width; height: 28; radius: 8
                        color: pH.hovered ? Qt.alpha(t.text, 0.06) : "transparent"
                        Row {
                            x: 8; anchors.verticalCenter: parent.verticalCenter; spacing: 8
                            Text { text: modelData.severity === 1 ? "error" : modelData.severity === 2 ? "warning" : "info"; color: modelData.severity === 1 ? t.removed : modelData.severity === 2 ? "#d8a54a" : t.faint; font.family: t.icons; font.pixelSize: 16 }
                            Text { text: `${modelData.line + 1}:${modelData.col + 1}`; color: t.faint; font.family: t.mono; font.pixelSize: 11; width: 50 }
                            Text { text: modelData.message.split("\n")[0]; color: t.text; font.family: t.sans; font.pixelSize: 12; elide: Text.ElideRight; width: probCol.width - 110 }
                        }
                        HoverHandler { id: pH; cursorShape: Qt.PointingHandCursor }
                        TapHandler { onTapped: { ed.cursorPosition = fv.posOf(modelData.line, modelData.col); ed.forceActiveFocus() } }
                    }
                }
            }
        }
    }
}
