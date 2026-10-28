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
    readonly property int curLine: ed.line
    readonly property int curCol: ed.col
    readonly property int lineCount: ed.lineCount
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
    // `auto`: written by auto save, so the text under the cursor is left as typed (no trimming)
    function write(auto) {
        if (!auto && (win.prefs.trimOnSave ?? true)) {
            let s = ed.text.replace(/[ \t]+$/gm, "")
            if ((win.prefs.finalNewline ?? true) && s.length && !s.endsWith("\n")) s += "\n"
            if (s !== ed.text) { const c = ed.cursorPosition; ed.text = s; ed.cursorPosition = Math.min(c, s.length) }
        }
        if (app.writeFile(win.filePath, ed.text)) { saved = ed.text; note = "Saved"; loadGit(); if (lsp) app.lspAsk("save", win.filePath, 0, 0) } else note = "Could not save"
        noteT.restart()
    }
    function revert() { ed.text = saved; shownText = saved }
    // auto save: a second after the last change (Settings > Editor)
    Timer { id: autoSave; interval: 1000; onTriggered: if (fv.dirty && win.view === "file") fv.write(true) }
    // the bracket at (or before) the cursor and its partner, for the pair highlight; [] when none
    readonly property var bracketPair: {
        const s = ed.text, p = ed.cursorPosition, open = "([{", close = ")]}"
        for (const at of [p, p - 1]) {
            const c = s[at]; if (c === undefined) continue
            let k = open.indexOf(c), dir = 1
            if (k < 0) { k = close.indexOf(c); dir = -1 }
            if (k < 0) continue
            const me = c, other = dir > 0 ? close[k] : open[k]
            let depth = 0
            for (let i = at; i >= 0 && i < s.length && Math.abs(i - at) < 20000; i += dir) {
                if (s[i] === me) depth++
                else if (s[i] === other && --depth === 0) return [at, i]
            }
            return []
        }
        return []
    }
    function testEdit(s) { ed.forceActiveFocus(); ed.cursorPosition = 0; ed.insert(0, s); save() }
    function testFind(q) { findBar.open("find"); findBar.q.text = q }
    function testProblems() { showProblems = true }
    // gui-check: the server asked once it has the file (symbols, or rename:LINE:COL:NEW); what comes back is logged
    function testLsp(spec) { lspTest.spec = spec; lspTest.start() }
    Timer {
        id: lspTest; property string spec; interval: 1800
        onTriggered: { const p = spec.split(":"); if (p[0] === "symbols") fv.ask("symbols", 0); else { fv.goTo(+p[1], +p[2]); renameField.text = p[3]; renameBox.done(true) } }
    }
    onJumpsChanged: if (app.shot.length && jumps.length) console.log("zett-probe: jumps " + JSON.stringify(jumps.map(j => j.name ?? j.line)))
    // the cursor to a 0-based line and column (search results, problems)
    function cursorPos() { return ed.cursorPosition }
    function goTo(line, col) { ed.cursorPosition = posOf(line, col); ed.forceActiveFocus() }
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
    // the file's changes since the last commit, as a diff (Changes in the header, or a click in Source Control)
    property bool showDiff: false
    property var diffRows: []
    function loadDiff() {
        let r; try { r = JSON.parse(app.git(JSON.stringify(["diff", "--no-color", "-U3", "HEAD", "--", win.filePath]))) } catch (e) { r = { out: "" } }
        const rows = []; let a = 0, b = 0
        for (const l of r.out.split("\n")) {
            const h = /^@@ -(\d+)(?:,\d+)? \+(\d+)/.exec(l)
            if (h) { a = +h[1]; b = +h[2]; rows.push({ k: "hunk", text: l.replace(/^@@[^@]*@@ ?/, "") || "…" }); continue }
            if (!rows.length || /^(\+\+\+|---) /.test(l)) continue
            if (l[0] === "+") rows.push({ k: "add", b: b++, text: l.slice(1) })
            else if (l[0] === "-") rows.push({ k: "del", a: a++, text: l.slice(1) })
            else if (l[0] === " ") rows.push({ k: "same", a: a++, b: b++, text: l.slice(1) })
        }
        diffRows = rows
    }
    onShowDiffChanged: if (showDiff) loadDiff()
    Connections { target: fv.win; function onFilePathChanged() { fv.showDiff = false } }
    function gitAt(i) { for (const m of gitMarks) { if (m.kind === "removed" ? m.line === i : (i >= m.line && i < m.line + m.count)) return m.kind } return "" }
    function openLsp() { if (visible && win.filePath) lsp = app.lspOpen(win.filePath, win.fileText) }
    Timer { id: lspSync; interval: 300; onTriggered: if (fv.lsp) fv.app.lspChange(fv.win.filePath, ed.text) }
    Connections {
        target: fv.app
        function onLspEvent(json) {
            let e; try { e = JSON.parse(json) } catch (x) { return }
            if (e.type === "diagnostics") { const m = Object.assign({}, fv.win.allDiags); if (e.items.length) m[e.path] = e.items; else delete m[e.path]; fv.win.allDiags = m }
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
            } else {
                // another file (a rename reaching it): changed on disk
                let s = fv.app.readFile(f.path)
                const at = (l, c) => { const ls = s.split("\n"); let p = 0; for (let i = 0; i < Math.min(l, ls.length - 1); i++) p += ls[i].length + 1; return p + c }
                for (const x of f.edits.slice().reverse()) { const a = at(x[0], x[1]), b = at(x[2], x[3]); s = s.slice(0, a) + x[4] + s.slice(b) }
                if (fv.app.writeFile(f.path, s)) fv.renamedFiles++
            }
            if (e.type === "edits" && fv.renameAsked) { fv.renameAsked = false; if (fv.app.shot.length) { console.log("zett-probe: renamed"); fv.write() } fv.note = fv.renamedFiles ? `Renamed, ${fv.renamedFiles + 1} files changed` : "Renamed"; fv.renamedFiles = 0; noteT.restart() }
            if (e.type === "references" || e.type === "symbols") { fv.jumpKind = e.type; fv.jumps = e.items; fv.jumpSel = 0 }
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
        Repeater {
            model: fv.dirty ? [["undo", "Revert", () => fv.revert()], ["save", "Save  Ctrl S", () => fv.save()]] : (fv.gitMarks.length || fv.showDiff ? [[fv.showDiff ? "edit" : "difference", fv.showDiff ? "Edit" : "Changes", () => fv.showDiff = !fv.showDiff]] : []).concat([["open_in_new", "Open with the system", () => Qt.openUrlExternally("file://" + fv.win.filePath)]])
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
    // the diff: old and new line numbers, removed and added lines in soft tints, hunks as headers
    Rectangle {
        visible: fv.showDiff
        Layout.fillWidth: true; Layout.fillHeight: true
        Layout.leftMargin: 16; Layout.rightMargin: 16
        radius: 17; color: t.card
        ListView {
            anchors { fill: parent; margins: 12 }
            clip: true; boundsBehavior: Flickable.StopAtBounds
            model: fv.diffRows
            delegate: Rectangle {
                required property var modelData
                width: ListView.view.width; height: modelData.k === "hunk" ? 30 : 20; radius: 6
                color: modelData.k === "add" ? Qt.alpha(t.added, 0.14) : modelData.k === "del" ? Qt.alpha(t.removed, 0.14) : "transparent"
                Row {
                    anchors.verticalCenter: parent.verticalCenter; spacing: 8
                    Text { width: 36; horizontalAlignment: Text.AlignRight; text: modelData.a !== undefined && modelData.k !== "add" ? modelData.a : ""; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
                    Text { width: 36; horizontalAlignment: Text.AlignRight; text: modelData.b !== undefined && modelData.k !== "del" ? modelData.b : ""; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
                    Text { width: 14; text: modelData.k === "add" ? "+" : modelData.k === "del" ? "−" : ""; color: modelData.k === "add" ? t.added : t.removed; font.family: t.mono; font.pixelSize: 12 }
                    Text { text: modelData.k === "hunk" ? modelData.text : modelData.text.replace(/\t/g, "    "); color: modelData.k === "hunk" ? t.accent : t.text; font.family: modelData.k === "hunk" ? t.sans : t.mono; font.pixelSize: 12 }
                }
            }
            Text { visible: !fv.diffRows.length; anchors.centerIn: parent; text: "No changes since the last commit"; color: t.faint; font.family: t.sans; font.pixelSize: 13 }
        }
    }
    // find / replace / go to line (FindBar.qml)
    FindBar { id: findBar; t: fv.t; editor: ed; Layout.fillWidth: true; Layout.leftMargin: 16; Layout.rightMargin: 16 }
    Rectangle {
        visible: !fv.showDiff
        Layout.fillWidth: true
        Layout.fillHeight: true
        Layout.leftMargin: 16; Layout.rightMargin: 16
        radius: 17
        color: t.card
        // the minimap: the whole file small, the part in view framed; a click or drag scrolls there;
        // errors and find hits as marks on its edge
        Item {
            id: miniMap
            visible: ed.lineCount > 40 && parent.width > 700
            anchors { right: parent.right; top: parent.top; bottom: parent.bottom; margins: 10 }
            width: 86
            clip: true
            readonly property real k: 0.13
            readonly property real total: Math.max(1, ed.contentHeight)
            // the small file follows the view when it is taller than the map
            readonly property real off: Math.max(0, Math.min(total * k - height, fileFlick.contentY * k - (height - fileFlick.height * k) / 2))
            Text {
                y: -miniMap.off
                transformOrigin: Item.TopLeft
                scale: miniMap.k
                text: fv.visible ? app.highlight(fv.ext, fv.shownText) : ""
                textFormat: Text.RichText
                color: t.text; font: ed.font
                opacity: 0.75
            }
            Rectangle {
                y: fileFlick.contentY * miniMap.k - miniMap.off
                width: parent.width; height: Math.max(12, fileFlick.height * miniMap.k)
                radius: 4
                color: Qt.alpha(t.text, mmH.hovered || mmDrag.active ? 0.14 : 0.08)
                border.width: 1; border.color: Qt.alpha(t.text, 0.12)
            }
            Repeater {
                model: fv.diags.filter(d => d.severity <= 2)
                delegate: Rectangle { required property var modelData; x: miniMap.width - 4; width: 4; height: 3; y: modelData.line * ed.lineH * miniMap.k - miniMap.off; color: modelData.severity === 1 ? t.removed : "#d8a54a" }
            }
            Repeater {
                model: findBar.matches.slice(0, 300)
                delegate: Rectangle { required property var modelData; x: 0; width: 4; height: 2; y: ed.positionToRectangle(modelData[0]).y * miniMap.k - miniMap.off; color: t.accent }
            }
            function scrollTo(y) { fileFlick.contentY = Math.max(0, Math.min(fileFlick.contentHeight - fileFlick.height, (y + off) / k - fileFlick.height / 2)) }
            HoverHandler { id: mmH; cursorShape: Qt.PointingHandCursor }
            TapHandler { onTapped: (ev) => miniMap.scrollTo(ev.position.y) }
            DragHandler { id: mmDrag; target: null; onCentroidChanged: if (active) miniMap.scrollTo(centroid.position.y) }
        }
        Flickable {
            id: fileFlick
            anchors { fill: parent; margins: 14; rightMargin: miniMap.visible ? 14 + miniMap.width + 8 : 14 }
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
            // F2: the new name for the symbol at the cursor, in a small field over it
            Rectangle {
                id: renameBox
                visible: false
                function open() { const w = fv.wordStart(ed.cursorPosition); let b = ed.cursorPosition; while (b < ed.text.length && /[\w$]/.test(ed.text[b])) b++; renameField.text = ed.text.slice(w, b); renameField.selectAll(); visible = true; renameField.forceActiveFocus() }
                function done(ok) { visible = false; if (ok && renameField.text.trim()) { fv.renameAsked = true; fv.app.lspChange(fv.win.filePath, ed.text); fv.ask("rename:" + renameField.text.trim(), ed.cursorPosition) } ed.forceActiveFocus() }
                x: ed.x + ed.cursorRectangle.x - 8; y: ed.cursorRectangle.y + ed.cursorRectangle.height + 4
                width: 240; height: 34; radius: 12; z: 60
                color: t.raised; border.width: 1; border.color: Qt.alpha(t.accent, 0.5)
                TextInput {
                    id: renameField
                    anchors { fill: parent; leftMargin: 12; rightMargin: 12 }
                    verticalAlignment: TextInput.AlignVCenter
                    color: t.text; font.family: t.mono; font.pixelSize: 13; selectionColor: Qt.alpha(t.accent, 0.4); clip: true
                    Keys.onReturnPressed: renameBox.done(true)
                    Keys.onEnterPressed: renameBox.done(true)
                    Keys.onEscapePressed: renameBox.done(false)
                    onActiveFocusChanged: if (!activeFocus && renameBox.visible) renameBox.done(false)
                }
            }
            // the bracket pair at the cursor: two soft rounded boxes
            Repeater {
                model: ed.activeFocus ? fv.bracketPair : []
                delegate: Rectangle {
                    required property int modelData
                    readonly property rect r: ed.positionToRectangle(modelData)
                    x: ed.x + r.x - 1; y: r.y; width: Math.max(8, ed.positionToRectangle(modelData + 1).x - r.x) + 2; height: r.height
                    radius: 4; color: Qt.alpha(t.accent, 0.22)
                }
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
                    if (fv.win.prefs.autoSave === true) autoSave.restart()
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
                    // the jump list takes ↑ ↓ Enter Esc while open
                    if (fv.jumps.length) {
                        if (e.key === Qt.Key_Down) { fv.jumpSel = (fv.jumpSel + 1) % fv.jumps.length; e.accepted = true; return }
                        if (e.key === Qt.Key_Up) { fv.jumpSel = (fv.jumpSel - 1 + fv.jumps.length) % fv.jumps.length; e.accepted = true; return }
                        if (e.key === Qt.Key_Return || e.key === Qt.Key_Enter) { fv.jumpTo(fv.jumps[fv.jumpSel]); e.accepted = true; return }
                        if (e.key === Qt.Key_Escape) { fv.jumps = []; e.accepted = true; return }
                    }
                    // the language server's keys, as in VS Code
                    if (e.key === Qt.Key_F12 && (e.modifiers & Qt.ShiftModifier)) { fv.ask("references", ed.cursorPosition); e.accepted = true; return }
                    if (e.key === Qt.Key_F12) { fv.ask("definition", ed.cursorPosition); e.accepted = true; return }
                    if (e.key === Qt.Key_F2 && fv.lsp) { renameBox.open(); e.accepted = true; return }
                    if (e.key === Qt.Key_F && (e.modifiers & Qt.ShiftModifier) && (e.modifiers & Qt.AltModifier)) { fv.ask("format", 0); e.accepted = true; return }
                    if (e.key === Qt.Key_O && (e.modifiers & Qt.ControlModifier) && (e.modifiers & Qt.ShiftModifier)) { fv.ask("symbols", 0); e.accepted = true; return }
                    if (e.key === Qt.Key_Space && (e.modifiers & Qt.ControlModifier)) { fv.compFrom = fv.wordStart(ed.cursorPosition); fv.ask("completion", ed.cursorPosition); e.accepted = true; return }
                    // Ctrl+D: the next occurrence of the selection (or of the word at the cursor) selected; Ctrl+Shift+L: all of
                    // them into find & replace, so one Replace all changes every place (a TextEdit has one cursor)
                    if (e.key === Qt.Key_D && (e.modifiers & Qt.ControlModifier) && !(e.modifiers & Qt.ShiftModifier)) {
                        let w = ed.selectedText
                        if (!w) { const a = fv.wordStart(ed.cursorPosition); let b = ed.cursorPosition; while (b < ed.text.length && /[\w$]/.test(ed.text[b])) b++; ed.select(a, b); e.accepted = true; return }
                        let i = ed.text.indexOf(w, ed.selectionEnd); if (i < 0) i = ed.text.indexOf(w)
                        if (i >= 0) ed.select(i, i + w.length)
                        e.accepted = true; return
                    }
                    if (e.key === Qt.Key_L && (e.modifiers & Qt.ControlModifier) && (e.modifiers & Qt.ShiftModifier) && ed.selectedText) { findBar.open("replace"); findBar.r.forceActiveFocus(); e.accepted = true; return }
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
    // references (Shift F12) or the file's symbols (Ctrl Shift O): a list under the editor, Enter or a click jumps
    property var jumps: []
    property string jumpKind: ""
    property int jumpSel: 0
    property bool renameAsked: false
    property int renamedFiles: 0
    function jumpTo(j) {
        if (!j) return
        if (j.path && j.path !== win.filePath) win.openFileAt({ path: j.path, rel: j.path }, j.line, j.col)
        else goTo(j.line, j.col)
        jumps = []
    }
    Rectangle {
        visible: fv.jumps.length > 0
        Layout.fillWidth: true; Layout.leftMargin: 16; Layout.rightMargin: 16
        implicitHeight: Math.min(220, jumpCol.implicitHeight + 44)
        radius: 14; color: t.card
        RowLayout {
            id: jumpHead
            anchors { left: parent.left; right: parent.right; top: parent.top; margins: 10; leftMargin: 14 }
            Text { text: fv.jumpKind === "symbols" ? "data_object" : "link"; color: t.accent; font.family: t.icons; font.pixelSize: 16 }
            Text { Layout.fillWidth: true; text: fv.jumpKind === "symbols" ? `${fv.jumps.length} symbols` : `${fv.jumps.length} references`; color: t.text; font.family: t.sans; font.pixelSize: 13; font.weight: Font.DemiBold }
            Text { text: "↑ ↓  Enter  Esc"; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
            Text { text: "close"; color: t.dim; font.family: t.icons; font.pixelSize: 16; TapHandler { onTapped: fv.jumps = [] } HoverHandler { cursorShape: Qt.PointingHandCursor } }
        }
        Flickable {
            anchors { left: parent.left; right: parent.right; bottom: parent.bottom; top: jumpHead.bottom; margins: 8; topMargin: 6 }
            clip: true; contentHeight: jumpCol.implicitHeight
            Column {
                id: jumpCol
                width: parent.width
                Repeater {
                    model: fv.jumps
                    delegate: Rectangle {
                        required property var modelData
                        required property int index
                        width: jumpCol.width; height: 28; radius: 8
                        color: index === fv.jumpSel ? Qt.alpha(t.accent, 0.16) : jH.hovered ? Qt.alpha(t.text, 0.06) : "transparent"
                        Row {
                            x: 8 + (modelData.depth ?? 0) * 14; anchors.verticalCenter: parent.verticalCenter; spacing: 10
                            Text { text: `${modelData.line + 1}`; color: t.faint; font.family: t.mono; font.pixelSize: 11; width: 34; horizontalAlignment: Text.AlignRight }
                            Text { text: modelData.name ?? (modelData.path ?? "").replace(fv.app.dir + "/", ""); color: t.text; font.family: t.mono; font.pixelSize: 12 }
                        }
                        HoverHandler { id: jH; cursorShape: Qt.PointingHandCursor }
                        TapHandler { onTapped: fv.jumpTo(modelData) }
                    }
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
