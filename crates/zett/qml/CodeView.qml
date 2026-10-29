import QtQuick
import dev.zett.app

// The new code editor (Settings > Editor > New editor core): CodeDoc (Rust) holds the text, cursors, undo and
// folds; this draws only the rows on screen, each coloured by Rust, the cursors from the code font's grid.
Rectangle {
    id: cv
    required property var win
    required property var t
    required property var app
    property alias doc: doc
    property var fv: null     // the FileView around it: its server, problems, git marks, comment mark, lists
    radius: 17
    color: t.card
    clip: true

    CodeDoc { id: doc }
    // the server follows the text (a moment after typing stops)
    Timer { id: lspSync; interval: 300; onTriggered: if (cv.fv && cv.fv.lsp) cv.app.lspChange(cv.file, doc.text()) }
    Connections { target: doc; function onVersionChanged() { if (doc.dirty) lspSync.restart() } }
    function findBarOpen() { findBar.open(false) }
    // ── Claude in the editor (Ctrl+I): the selection (or the line) rewritten as asked, shown as a diff to take or leave ──
    property var ai: null          // { l0, c0, old, ask, out, busy, err }
    function aiRange() {
        const c = carets[0], a = c[0] < c[2] || (c[0] === c[2] && c[1] < c[3]) ? [c[0], c[1]] : [c[2], c[3]], b = a[0] === c[0] && a[1] === c[1] ? [c[2], c[3]] : [c[0], c[1]]
        if (a[0] === b[0] && a[1] === b[1]) { const s = doc.lineText(a[0]); return { l0: a[0], c0: 0, old: s } }
        doc.select(a[0], a[1], 0); doc.click(b[0], b[1], true, false)
        return { l0: a[0], c0: a[1], old: doc.selectedText() }
    }
    function aiAsk(ask) {
        if (!ask.trim()) return
        const r = aiRange()
        const lang = file.slice(file.lastIndexOf(".") + 1)
        const prompt = `Rewrite this ${lang} code from ${file.replace(app.dir + "/", "")}: ${ask.trim()}. Keep its indentation. Answer with only the rewritten code: no explanation, no code fences.`
        ai = { l0: r.l0, c0: r.c0, old: r.old, ask: ask, out: "", busy: true, err: "" }
        const b64 = Qt.btoa(unescape(encodeURIComponent(r.old)))
        app.runShell(99998, `printf %s ${b64} | base64 -d | claude -p '${prompt.replace(/'/g, "'\\''")}' 2>/dev/null`)
    }
    Connections {
        target: cv.app
        function onRunOutput(id, chunk, done, code) {
            if (id !== 99998 || !cv.ai) return
            const a = Object.assign({}, cv.ai); a.out += chunk
            if (done) { a.busy = false; a.out = a.out.replace(/^\s*```[\w-]*\n/, "").replace(/\n?```\s*$/, ""); if (code !== 0 && !a.out.trim()) a.err = "claude failed (exit " + code + ")" }
            cv.ai = a
        }
    }
    function aiAccept() {
        if (!ai || ai.busy || !ai.out) return
        const old = ai.old, l0 = ai.l0, c0 = ai.c0
        let out = ai.out
        if (!old.endsWith("\n")) out = out.replace(/\n$/, "")
        doc.select(l0, c0, old.length); doc.insert(out)
        ai = null; input.forceActiveFocus()
    }
    function aiReject() { if (ai && ai.busy) cv.app.stopShell(); ai = null; input.forceActiveFocus() }
    function expandSnippet() {
        const w = word(), body = (win.snippets ?? {})[w]
        if (!w || typeof body !== "string" || carets.length > 1) return false
        const l = carets[0][0], w0 = wordStart(), ind = doc.lineText(l).match(/^\s*/)[0]
        const text = body.split("\n").join("\n" + ind), at = text.indexOf("$0")
        doc.select(l, w0, w.length); doc.insert(text.replace("$0", ""))
        if (at >= 0) { const before = text.slice(0, at).split("\n"); doc.select(l + before.length - 1, (before.length > 1 ? 0 : w0) + before[before.length - 1].length, 0) }
        return true
    }
    // the word before the main cursor (what completion filters by) and where it starts
    function wordStart() { const s = doc.lineText(carets[0][0]); let c = carets[0][1]; while (c > 0 && /[\w$]/.test(s[c - 1])) c--; return c }
    function word() { return doc.lineText(carets[0][0]).slice(wordStart(), carets[0][1]) }
    // completion: asked after . :: or Ctrl+Space, then follows the word typed
    property bool completing: false
    Connections {
        target: doc
        function onVersionChanged() {
            if (!cv.fv || !input.activeFocus) return
            const s = doc.lineText(cv.carets[0][0]), c = cv.carets[0][1], prev = s[c - 1] ?? ""
            // opens after . or :: and once a word has two letters; follows the word as it grows
            const w = cv.word()
            if (prev === "." || s.slice(c - 2, c) === "::" || (/[\w$]/.test(prev) && w.length >= 2)) { cv.completing = true; if (cv.fv.lsp) { lspSync.stop(); cv.app.lspChange(cv.file, doc.text()); cv.askHere("completion") } }
            else cv.completing = false
            if (prev === "(" || prev === ",") { lspSync.stop(); cv.app.lspChange(cv.file, doc.text()); cv.askHere("signature") }
            else if (prev === ")") cv.fv.sigLabel = ""
        }
    }
    // the server's items for the word, then the file's own words it does not have (all there is without a server)
    readonly property var comps: {
        if (!cv.completing || !cv.fv) return []
        doc.version
        const w = word(), wl = w.toLowerCase()
        const server = cv.fv.comps.filter(i => !wl || (i.filter || i.label).toLowerCase().includes(wl))
        const have = new Set(server.map(i => i.label))
        let local = []; try { local = JSON.parse(doc.words(w)) } catch (e) {}
        return server.concat(local.filter(x => !have.has(x)).map(x => ({ label: x, insert: x, detail: "in this file" }))).slice(0, 50)
    }
    function takeComp(i) { const it = comps[i]; if (!it) return; const w0 = wordStart(); doc.select(carets[0][0], w0, carets[0][1] - w0); doc.insert(it.insert || it.label); completing = false; cv.fv.comps = [] }
    function askHere(kind) { if (cv.fv) cv.fv.askAt(kind, cv.carets[0][0], cv.carets[0][1]) }
    readonly property string file: win.filePath
    // switching files: unsaved text is kept (win.unsaved) and comes back when the file is opened again
    function openHere(p) {
        if (doc.dirty && doc.path) { const u = Object.assign({}, win.unsaved); u[doc.path] = doc.text(); win.unsaved = u }
        doc.open(p)
        if (win.unsaved[p] !== undefined) { doc.restore(win.unsaved[p]); const u = Object.assign({}, win.unsaved); delete u[p]; win.unsaved = u }
    }
    onFileChanged: if (file && visible) openHere(file)
    onVisibleChanged: if (visible && file && doc.path !== file) openHere(file)

    FontMetrics { id: fm; font.family: cv.t.mono; font.pixelSize: 13 }
    readonly property real cw: fm.advanceWidth("M")
    readonly property real lh: Math.ceil(fm.height * 1.45)
    readonly property real gutter: 64
    readonly property var carets: { try { return JSON.parse(doc.cursors) } catch (e) { return [[0, 0, 0, 0]] } }
    // the first row on screen and how many fit
    property real scrollY: 0
    readonly property int firstRow: Math.max(0, Math.floor(scrollY / lh))
    readonly property int shown: Math.ceil(height / lh) + 1
    function maxTop() { return Math.max(0, doc.rows * lh - height + lh * 4) }
    // keep the main cursor in view
    function follow() {
        if (height <= 0) return   // not laid out yet: following then scrolled the cursor out of view
        const y = doc.lineRow(carets[0][0]) * lh
        if (y < scrollY) scrollY = y
        else if (y + lh > scrollY + height) scrollY = y + lh - height
    }
    Connections { target: doc; function onCursorsChanged() { cv.follow() } }
    function rowColAt(x, y) {
        const row = Math.max(0, Math.min(doc.rows - 1, Math.floor((y + scrollY) / lh)))
        return [doc.rowLine(row), Math.max(0, Math.round((x - gutter - 12) / cw))]
    }
    // clipboard through a hidden text field
    TextEdit { id: clip; visible: false }
    function copy(cut) { const s = doc.selectedText(); if (!s) return; clip.text = s; clip.selectAll(); clip.copy(); if (cut) doc.key(0x01000007, "", 0) }
    function paste() { clip.text = ""; clip.paste(); if (clip.text) doc.insert(clip.text) }

    // git blame of the line the cursor rests on (after a pause): who, when, the commit's summary
    property int blameLine: -1
    property string blameText: ""
    readonly property int caretLine: carets.length ? carets[0][0] : -1
    onCaretLineChanged: { blameText = ""; blameT.restart() }
    Connections { target: cv.win; function onFilePathChanged() { cv.blameText = ""; blameT.restart() } }
    Component.onCompleted: blameT.restart()
    Timer {
        id: blameT; interval: 450
        onTriggered: {
            const path = cv.win.filePath
            if (!path || cv.caretLine < 0 || (cv.win.prefs?.blame ?? true) === false) return
            let r; try { r = JSON.parse(cv.app.git(JSON.stringify(["blame", "--porcelain", "-L", `${cv.caretLine + 1},${cv.caretLine + 1}`, "--", path]))) } catch (e) { return }
            if (r.code !== 0) return
            const get = k => (r.out.split("\n").find(l => l.startsWith(k + " ")) || "").slice(k.length + 1)
            const sha = r.out.slice(0, 40)
            cv.blameLine = cv.caretLine
            if (/^0+$/.test(sha)) { cv.blameText = "You · not committed yet"; return }
            const ago = cv.ago(Number(get("author-time")))
            cv.blameText = `${get("author")} · ${ago} · ${get("summary")}`
        }
    }
    function ago(t) {
        const s = Date.now() / 1000 - t
        const u = [[31536000, "year"], [2592000, "month"], [604800, "week"], [86400, "day"], [3600, "hour"], [60, "minute"]]
        for (const [n, w] of u) if (s >= n) { const k = Math.floor(s / n); return `${k} ${w}${k > 1 ? "s" : ""} ago` }
        return "just now"
    }

    // merge conflicts: <<<<<<< ours (||||||| base) ======= theirs >>>>>>>; a block becomes one side or both
    function conflictAt(line) {
        if (!doc.lineText(line).startsWith("<<<<<<<")) return null
        let base = -1, mid = -1
        for (let l = line + 1; l < Math.min(doc.lines, line + 20000); l++) {
            const s = doc.lineText(l)
            if (s.startsWith("|||||||") && mid < 0) base = l
            else if (s.startsWith("=======") && mid < 0) mid = l
            else if (s.startsWith(">>>>>>>") && mid >= 0) return { a: line, base, mid, e: l }
        }
        return null
    }
    function resolve(line, which) {
        const c = conflictAt(line); if (!c) return
        const take = (from, to) => { const o = []; for (let l = from; l < to; l++) o.push(doc.lineText(l)); return o }
        const ours = take(c.a + 1, c.base >= 0 ? c.base : c.mid), theirs = take(c.mid + 1, c.e)
        const keep = which === "ours" ? ours : which === "theirs" ? theirs : ours.concat(theirs)
        const last = c.e + 1 < doc.lines
        const text = keep.length ? keep.join("\n") + (last ? "\n" : "") : ""
        doc.applyEdits(JSON.stringify([last ? [c.a, 0, c.e + 1, 0, text] : [c.a, 0, c.e, doc.lineText(c.e).length, text.replace(/\n$/, "")]]))
    }

    // the rows on screen
    Repeater {
        model: Math.min(cv.shown, Math.max(0, doc.rows - cv.firstRow))
        delegate: Item {
            required property int index
            readonly property int row: cv.firstRow + index
            readonly property int line: (doc.version, doc.rowLine(row))
            readonly property int fold: (doc.version, doc.foldAt(line))
            readonly property bool cur: cv.carets.some(c => c[0] === line)
            y: row * cv.lh - cv.scrollY
            width: cv.width; height: cv.lh
            // the current line: inset with its own corners, so it stays inside the rounded card
            Rectangle { visible: parent.cur; anchors { fill: parent; leftMargin: 6; rightMargin: 6 } radius: 8; color: Qt.alpha(cv.t.text, 0.05) }
            // the line the debugger stopped at; a breakpoint: a red dot left of the number (a click there or F9)
            readonly property bool bp: (cv.win.breakpoints, (cv.win.breakpoints[cv.win.filePath] ?? []).includes(line))
            readonly property bool here: cv.win.debugAt.path === cv.win.filePath && cv.win.debugAt.line === line
            Rectangle { visible: parent.here; anchors { fill: parent; leftMargin: 6; rightMargin: 6 } radius: 8; color: Qt.alpha("#e5c46f", 0.22); border.width: 1; border.color: Qt.alpha("#e5c46f", 0.5) }
            Rectangle { visible: parent.bp || bpH.hovered; x: 10; anchors.verticalCenter: parent.verticalCenter; width: 10; height: 10; radius: 5; color: parent.bp ? cv.t.removed : Qt.alpha(cv.t.removed, 0.35) }
            Item {
                x: 4; width: 20; height: parent.height
                HoverHandler { id: bpH; cursorShape: Qt.PointingHandCursor }
                TapHandler { onTapped: cv.win.toggleBreakpoint(cv.win.filePath, parent.parent.line) }
            }
            Text {
                x: 0; width: 40; height: parent.height
                horizontalAlignment: Text.AlignRight; verticalAlignment: Text.AlignVCenter
                text: parent.line + 1
                color: parent.cur ? cv.t.accent : cv.t.faint
                font.family: cv.t.mono; font.pixelSize: 12
            }
            // git: a soft dot (added green, changed blue, removed red); a problem: a dot in its colour by the number
            readonly property string git: cv.fv ? (cv.fv.gitMarks, cv.fv.gitAt(line)) : ""
            Rectangle { visible: parent.git.length > 0; x: 2; anchors.verticalCenter: parent.verticalCenter; width: 6; height: 6; radius: 3; color: parent.git === "added" ? "#5fb98a" : parent.git === "changed" ? "#5b9bd5" : cv.t.removed }
            readonly property int sev: { if (!cv.fv) return 9; let s = 9; for (const d of cv.fv.diags) if (d.line === line && d.severity < s) s = d.severity; return s }
            Rectangle { visible: parent.sev <= 2; x: 12; anchors.verticalCenter: parent.verticalCenter; width: 6; height: 6; radius: 3; color: parent.sev === 1 ? cv.t.removed : "#d8a54a" }
            // fold: a chevron for a line that opens a block, "⋯" after one folded
            Text {
                visible: parent.fold > 0
                x: 44; width: 16; height: parent.height; verticalAlignment: Text.AlignVCenter
                text: parent.fold === 2 ? "chevron_right" : "expand_more"
                color: parent.fold === 2 ? cv.t.accent : Qt.alpha(cv.t.faint, fh.hovered || parent.cur ? 1 : 0.4)
                font.family: cv.t.icons; font.pixelSize: 15
                HoverHandler { id: fh; cursorShape: Qt.PointingHandCursor }
                TapHandler { onTapped: doc.toggleFold(parent.parent.line) }
            }
            Text {
                id: code
                x: cv.gutter + 12; height: parent.height; verticalAlignment: Text.AlignVCenter
                text: (doc.version, doc.lineHtml(parent.line))
                textFormat: Text.StyledText
                color: cv.t.text
                font.family: cv.t.mono; font.pixelSize: 13
            }
            Rectangle {
                visible: parent.fold === 2
                x: code.x + code.implicitWidth + 8; anchors.verticalCenter: parent.verticalCenter
                width: 30; height: 16; radius: 8; color: Qt.alpha(cv.t.accent, 0.18)
                Text { anchors.centerIn: parent; text: "⋯"; color: cv.t.accent; font.pixelSize: 12 }
                TapHandler { onTapped: doc.toggleFold(parent.parent.line) }
            }
            // blame, faint after the text of the line the cursor rests on
            Text {
                visible: parent.cur && cv.blameLine === parent.line && cv.blameText.length > 0 && !parent.conflict
                x: code.x + code.implicitWidth + 28; height: parent.height; verticalAlignment: Text.AlignVCenter
                width: Math.max(0, cv.width - x - 24); elide: Text.ElideRight
                text: cv.blameText
                color: Qt.alpha(cv.t.faint, 0.8)
                font.family: cv.t.sans; font.pixelSize: 12; font.italic: true
            }
            // a conflict's first line: keep ours, theirs or both
            readonly property bool conflict: (doc.version, doc.lineText(line).startsWith("<<<<<<<"))
            Row {
                visible: parent.conflict
                x: code.x + code.implicitWidth + 16; anchors.verticalCenter: parent.verticalCenter
                spacing: 6
                Repeater {
                    model: [["ours", "Keep current"], ["theirs", "Keep incoming"], ["both", "Keep both"]]
                    delegate: Rectangle {
                        required property var modelData
                        width: ct.implicitWidth + 18; height: 18; radius: 9
                        color: ch.hovered ? Qt.alpha(cv.t.accent, 0.3) : Qt.alpha(cv.t.accent, 0.16)
                        Text { id: ct; anchors.centerIn: parent; text: modelData[1]; color: cv.t.accent; font.family: cv.t.sans; font.pixelSize: 11 }
                        HoverHandler { id: ch; cursorShape: Qt.PointingHandCursor }
                        TapHandler { onTapped: cv.resolve(parent.parent.parent.line, modelData[0]) }
                    }
                }
            }
        }
    }
    // find matches and problem ranges on the rows on screen: soft rounded bands under the text
    readonly property var marks: {
        doc.version; const out = []
        for (const m of findBar.matches) out.push([m[0], m[1], Math.max(1, m[2]), 0])
        if (cv.fv) for (const d of cv.fv.diags) if (d.severity <= 2) out.push([d.line, d.col, Math.max(1, (d.endLine === d.line ? d.endCol : d.col + 1) - d.col), d.severity])
        return out.filter(m => { const r = doc.lineRow(m[0]); return doc.rowLine(r) === m[0] && r >= cv.firstRow && r < cv.firstRow + cv.shown }).slice(0, 600)
    }
    Repeater {
        model: cv.marks
        delegate: Rectangle {
            required property var modelData
            readonly property bool hit: modelData[3] === 0
            x: cv.gutter + 12 + modelData[1] * cv.cw; width: modelData[2] * cv.cw
            y: doc.lineRow(modelData[0]) * cv.lh - cv.scrollY + (hit ? 2 : cv.lh - 5); height: hit ? cv.lh - 4 : 3
            radius: hit ? 4 : 1.5
            color: hit ? Qt.alpha("#e5c46f", 0.28) : Qt.alpha(modelData[3] === 1 ? cv.t.removed : "#d8a54a", 0.85)
        }
    }
    // the bracket pair at the cursor: two soft rounded boxes
    readonly property var pair: { doc.version; doc.cursors; const s = doc.bracketPair(); return s ? JSON.parse(s) : [] }
    Repeater {
        model: cv.pair.length && input.activeFocus ? [[cv.pair[0], cv.pair[1]], [cv.pair[2], cv.pair[3]]] : []
        delegate: Rectangle {
            required property var modelData
            x: cv.gutter + 12 + modelData[1] * cv.cw - 1; y: doc.lineRow(modelData[0]) * cv.lh - cv.scrollY + 3
            width: cv.cw + 2; height: cv.lh - 6; radius: 4
            color: Qt.alpha(cv.t.accent, 0.22)
        }
    }
    // sticky scroll: the lines that open the scope at the top of the view, over the code in a soft strip
    readonly property var sticky: { doc.version; if (firstRow < 1) return []; try { return JSON.parse(doc.scope(doc.rowLine(firstRow + 1))) } catch (e) { return [] } }
    Rectangle {
        visible: cv.sticky.length > 0
        z: 20; x: 0; y: 0; width: cv.width; height: cv.sticky.length * cv.lh + 6
        color: cv.t.card
        Rectangle { anchors { left: parent.left; right: parent.right; bottom: parent.bottom; leftMargin: 12; rightMargin: 12 } height: 1; color: Qt.alpha(cv.t.rule, 0.4) }
        Repeater {
            model: cv.sticky
            delegate: Item {
                required property int modelData
                required property int index
                y: index * cv.lh + 3; width: cv.width; height: cv.lh
                Text { x: 0; width: 40; height: parent.height; horizontalAlignment: Text.AlignRight; verticalAlignment: Text.AlignVCenter; text: modelData + 1; color: cv.t.faint; font.family: cv.t.mono; font.pixelSize: 12 }
                Text { x: cv.gutter + 12; height: parent.height; verticalAlignment: Text.AlignVCenter; text: (doc.version, doc.lineHtml(modelData)); textFormat: Text.StyledText; color: cv.t.text; font.family: cv.t.mono; font.pixelSize: 13 }
                HoverHandler { cursorShape: Qt.PointingHandCursor }
                TapHandler { onTapped: { doc.select(modelData, 0, 0); input.forceActiveFocus() } }
            }
        }
    }
    // selections: for each cursor, a rounded band per row it covers
    Repeater {
        model: cv.carets.filter(c => c[0] !== c[2] || c[1] !== c[3])
        delegate: Item {
            required property var modelData
            readonly property var a: modelData[0] < modelData[2] || (modelData[0] === modelData[2] && modelData[1] < modelData[3]) ? [modelData[0], modelData[1]] : [modelData[2], modelData[3]]
            readonly property var b: a[0] === modelData[0] && a[1] === modelData[1] ? [modelData[2], modelData[3]] : [modelData[0], modelData[1]]
            Repeater {
                model: Math.min(400, parent.b[0] - parent.a[0] + 1)
                delegate: Rectangle {
                    required property int index
                    readonly property int line: parent.a[0] + index
                    readonly property int row: doc.lineRow(line)
                    visible: doc.rowLine(row) === line && row >= cv.firstRow && row < cv.firstRow + cv.shown
                    readonly property int c0: index === 0 ? parent.a[1] : 0
                    readonly property int c1: line === parent.b[0] ? parent.b[1] : 400
                    x: cv.gutter + 12 + c0 * cv.cw; y: row * cv.lh - cv.scrollY + 2
                    width: Math.max(cv.cw * 0.5, (Math.min(c1, 400) - c0) * cv.cw); height: cv.lh - 4
                    radius: 4; color: Qt.alpha(cv.t.accent, 0.28)
                }
            }
        }
    }
    // the cursors
    Repeater {
        model: cv.carets
        delegate: Rectangle {
            required property var modelData
            required property int index
            x: cv.gutter + 12 + modelData[1] * cv.cw - 1; y: doc.lineRow(modelData[0]) * cv.lh - cv.scrollY + 3
            width: 2; height: cv.lh - 6; radius: 1
            color: cv.t.accent
            visible: input.activeFocus && blink.on
        }
    }
    // the popups at the main cursor: completion, signature, code actions, rename; hover at the mouse
    readonly property real cx: gutter + 12 + carets[0][1] * cw
    readonly property real cy: doc.lineRow(carets[0][0]) * lh - scrollY
    component Pop: Rectangle { radius: 12; z: 50; color: cv.t.raised; border.width: 1; border.color: Qt.alpha(cv.t.rule, 0.4) }
    Pop {
        visible: cv.comps.length > 0 && input.activeFocus
        x: Math.min(cv.cx, cv.width - width - 8); y: cv.cy + cv.lh + 2
        width: 340; height: Math.min(260, cv.comps.length * 26 + 10)
        clip: true
        // its own list: the wheel scrolls it (never the editor under it), the selection stays in view
        ListView {
            id: compCol; anchors { fill: parent; margins: 5 }
            clip: true; boundsBehavior: Flickable.StopAtBounds
            model: cv.comps
            currentIndex: cv.fv ? cv.fv.compSel : -1
            onCurrentIndexChanged: positionViewAtIndex(currentIndex, ListView.Contain)
            // Settings > Editor > Wheel moves through the completion list (pumwheel); off: the editor scrolls
            interactive: cv.win.prefs?.pumWheel ?? true
            WheelHandler { onWheel: (ev) => { if (!(cv.win.prefs?.pumWheel ?? true)) { cv.scrollY = Math.max(0, Math.min(cv.maxTop(), cv.scrollY - ev.angleDelta.y / 120 * cv.lh * 3)); return } compCol.contentY = Math.max(0, Math.min(compCol.contentHeight - compCol.height, compCol.contentY - ev.angleDelta.y / 120 * 26 * 3)) } }
                delegate: Rectangle {
                    required property var modelData
                    required property int index
                    width: compCol.width; height: 26; radius: 8
                    color: index === cv.fv.compSel ? Qt.alpha(cv.t.accent, 0.2) : "transparent"
                    Row { x: 8; anchors.verticalCenter: parent.verticalCenter; spacing: 8
                        Text { text: modelData.label; color: cv.t.text; font.family: cv.t.mono; font.pixelSize: 12 }
                        Text { text: modelData.detail ?? ""; color: cv.t.faint; font.family: cv.t.sans; font.pixelSize: 11; elide: Text.ElideRight; width: 140 } }
                    TapHandler { onTapped: cv.takeComp(index) }
                }
        }
    }
    Pop {
        visible: !!(cv.fv && cv.fv.sigLabel) && input.activeFocus
        x: Math.max(4, cv.cx - 20); y: Math.max(4, cv.cy - height - 4)
        width: Math.min(600, sigT.implicitWidth + 24); height: sigT.implicitHeight + 14
        Text { id: sigT; x: 12; anchors.verticalCenter: parent.verticalCenter; width: Math.min(576, implicitWidth); wrapMode: Text.Wrap; textFormat: Text.StyledText; color: cv.t.dim; font.family: cv.t.mono; font.pixelSize: 12
            readonly property var e: s => s.replace(/&/g, "&amp;").replace(/</g, "&lt;")
            text: !cv.fv ? "" : cv.fv.sigTo > cv.fv.sigFrom ? e(cv.fv.sigLabel.slice(0, cv.fv.sigFrom)) + `<b><font color="${cv.t.accent}">` + e(cv.fv.sigLabel.slice(cv.fv.sigFrom, cv.fv.sigTo)) + "</font></b>" + e(cv.fv.sigLabel.slice(cv.fv.sigTo)) : e(cv.fv.sigLabel) }
    }
    Pop {
        visible: !!(cv.fv && cv.fv.actions.length)
        x: Math.min(cv.cx, cv.width - width - 8); y: cv.cy + cv.lh + 2
        width: 360; height: Math.min(260, actCol2.implicitHeight + 12)
        Column { id: actCol2; x: 6; y: 6; width: parent.width - 12
            Repeater { model: cv.fv ? cv.fv.actions : []
                delegate: Rectangle { required property var modelData; required property int index; width: actCol2.width; height: 28; radius: 8
                    color: index === cv.fv.actionSel ? Qt.alpha(cv.t.accent, 0.18) : "transparent"
                    Row { x: 8; anchors.verticalCenter: parent.verticalCenter; spacing: 8
                        Text { text: "lightbulb"; color: cv.t.accent; font.family: cv.t.icons; font.pixelSize: 15 }
                        Text { text: modelData.title; color: cv.t.text; font.family: cv.t.sans; font.pixelSize: 12 } }
                    TapHandler { onTapped: cv.fv.takeAction(index) } } } }
    }
    // F2: the new name in a small field over the cursor
    Pop {
        id: rename
        visible: false
        function open() { const w0 = cv.wordStart(); let e = cv.carets[0][1]; const s = doc.lineText(cv.carets[0][0]); while (e < s.length && /[\w$]/.test(s[e])) e++; rf.text = s.slice(w0, e); visible = true; rf.selectAll(); rf.forceActiveFocus() }
        function done(ok) { visible = false; if (ok && rf.text.trim() && cv.fv) { cv.fv.renameAsked = true; cv.app.lspChange(cv.file, doc.text()); cv.askHere("rename:" + rf.text.trim()) } input.forceActiveFocus() }
        x: cv.cx - 8; y: cv.cy + cv.lh + 2; width: 240; height: 32
        border.color: Qt.alpha(cv.t.accent, 0.5)
        TextInput { id: rf; anchors { fill: parent; leftMargin: 12; rightMargin: 12 } verticalAlignment: TextInput.AlignVCenter; clip: true; color: cv.t.text; font.family: cv.t.mono; font.pixelSize: 13
            Keys.onReturnPressed: rename.done(true); Keys.onEscapePressed: rename.done(false) }
    }
    // hover: the server's note on the word the mouse rests on
    property point hoverAt: Qt.point(0, 0)
    Timer { id: hoverAsk; interval: 650; onTriggered: if (cv.fv && cv.fv.lsp && hov.hovered) { const [l, c] = cv.rowColAt(cv.hoverAt.x, cv.hoverAt.y); cv.fv.hoverText = ""; cv.fv.askAt("hover", l, c) } }
    Pop {
        visible: !!(cv.fv && cv.fv.hoverText) && hov.hovered
        x: Math.min(cv.hoverAt.x, cv.width - width - 8); y: cv.hoverAt.y + 18
        width: Math.min(560, hT.implicitWidth + 24); height: Math.min(280, hT.implicitHeight + 16); clip: true
        Text { id: hT; x: 12; y: 8; width: Math.min(536, implicitWidth); text: cv.fv ? cv.fv.hoverText : ""; wrapMode: Text.Wrap; color: cv.t.text; font.family: cv.t.mono; font.pixelSize: 12 }
    }
    Timer { id: blink; property bool on: true; interval: 530; repeat: true; running: input.activeFocus; onTriggered: on = !on }
    Connections { target: doc; function onVersionChanged() { blink.on = true; blink.restart() } }

    // input: keys to the core, the mouse places, drags and adds cursors, the wheel scrolls
    Item {
        id: input
        anchors.fill: parent
        focus: true
        Keys.onPressed: (e) => {
            const ctrl = e.modifiers & Qt.ControlModifier
            const shift = e.modifiers & Qt.ShiftModifier, alt = e.modifiers & Qt.AltModifier
            // Claude: Ctrl+I asks; a change shown takes Tab or Ctrl+Enter, Esc leaves it
            if (ctrl && e.key === Qt.Key_I) { aiBar.open(); e.accepted = true; return }
            if (cv.ai && !cv.ai.busy && cv.ai.out && (e.key === Qt.Key_Tab || ((e.key === Qt.Key_Return || e.key === Qt.Key_Enter) && ctrl))) { cv.aiAccept(); e.accepted = true; return }
            if (cv.ai && e.key === Qt.Key_Escape) { cv.aiReject(); e.accepted = true; return }
            // the completion list takes ↑ ↓ Enter Tab Esc while open
            if (cv.fv && cv.fv.actions.length) {
                if (e.key === Qt.Key_Down) { cv.fv.actionSel = (cv.fv.actionSel + 1) % cv.fv.actions.length; e.accepted = true; return }
                if (e.key === Qt.Key_Up) { cv.fv.actionSel = (cv.fv.actionSel - 1 + cv.fv.actions.length) % cv.fv.actions.length; e.accepted = true; return }
                if (e.key === Qt.Key_Return || e.key === Qt.Key_Enter) { cv.fv.takeAction(cv.fv.actionSel); e.accepted = true; return }
                if (e.key === Qt.Key_Escape) { cv.fv.actions = []; e.accepted = true; return }
            }
            if (cv.comps.length) {
                if (e.key === Qt.Key_Down) { cv.fv.compSel = (cv.fv.compSel + 1) % cv.comps.length; e.accepted = true; return }
                if (e.key === Qt.Key_Up) { cv.fv.compSel = (cv.fv.compSel - 1 + cv.comps.length) % cv.comps.length; e.accepted = true; return }
                if (e.key === Qt.Key_Return || e.key === Qt.Key_Enter || e.key === Qt.Key_Tab) { cv.takeComp(cv.fv.compSel); e.accepted = true; return }
                if (e.key === Qt.Key_Escape) { cv.completing = false; cv.fv.comps = []; e.accepted = true; return }
            }
            // a snippet: its prefix before the cursor, then Tab (snippets.json; $0 is where the cursor goes)
            if (e.key === Qt.Key_Tab && !shift && !ctrl && cv.expandSnippet()) { e.accepted = true; return }
            if (ctrl && alt && (e.key === Qt.Key_Up || e.key === Qt.Key_Down)) { doc.addCursorLine(e.key === Qt.Key_Up ? -1 : 1); e.accepted = true; return }
            if (ctrl && e.key === Qt.Key_Space) { cv.completing = true; if (cv.fv && cv.fv.lsp) { cv.app.lspChange(cv.file, doc.text()); cv.askHere("completion") } e.accepted = true; return }
            if (ctrl && e.key === Qt.Key_Period) { cv.fv && cv.fv.askActions(cv.carets[0][0], cv.carets[0][1]); e.accepted = true; return }
            if (e.key === Qt.Key_F2 && cv.fv && cv.fv.lsp) { rename.open(); e.accepted = true; return }
            if (e.key === Qt.Key_Escape && cv.fv && cv.fv.sigLabel) { cv.fv.sigLabel = ""; e.accepted = true; return }
            if (ctrl && e.key === Qt.Key_S) { if (doc.save()) { cv.fv && cv.fv.lsp && cv.app.lspAsk("save", cv.file, 0, 0); cv.fv && cv.fv.loadGit() } e.accepted = true; return }
            // line editing, find, the server's keys
            if (ctrl && e.key === Qt.Key_Slash) { doc.lineOp("comment:" + (cv.fv ? cv.fv.commentMark : "//")); e.accepted = true; return }
            if (alt && !ctrl && (e.key === Qt.Key_Up || e.key === Qt.Key_Down)) { doc.lineOp((shift ? "copy" : "") + (e.key === Qt.Key_Up ? "up" : "down")); e.accepted = true; return }
            if (ctrl && shift && e.key === Qt.Key_K) { doc.lineOp("delete"); e.accepted = true; return }
            if (ctrl && !shift && (e.key === Qt.Key_F || e.key === Qt.Key_H)) { findBar.open(e.key === Qt.Key_H); e.accepted = true; return }
            if (e.key === Qt.Key_F3 || (ctrl && e.key === Qt.Key_G)) { findBar.step(shift ? -1 : 1); e.accepted = true; return }
            if (e.key === Qt.Key_F12) { cv.askHere(shift ? "references" : "definition"); e.accepted = true; return }
            if (ctrl && shift && e.key === Qt.Key_O) { cv.askHere("symbols"); e.accepted = true; return }
            if (shift && alt && e.key === Qt.Key_F) { cv.askHere("format"); e.accepted = true; return }
            if (e.key === Qt.Key_Escape && findBar.visible) { findBar.visible = false; e.accepted = true; return }
            if (ctrl && e.key === Qt.Key_C) { cv.copy(false); e.accepted = true; return }
            if (ctrl && e.key === Qt.Key_X) { cv.copy(true); e.accepted = true; return }
            if (ctrl && e.key === Qt.Key_V) { cv.paste(); e.accepted = true; return }
            e.accepted = doc.key(e.key, e.text, e.modifiers)
        }
        TapHandler {
            acceptedModifiers: Qt.KeyboardModifierMask
            // one click places (Shift extends, Alt adds a cursor), two select the word, three the line
            onTapped: (ev, button) => {
                input.forceActiveFocus()
                const [l, c] = cv.rowColAt(ev.position.x, ev.position.y)
                if (tapCount >= 3) doc.selectUnit(l, c, true)
                else if (tapCount === 2) doc.selectUnit(l, c, false)
                else doc.click(l, c, (ev.modifiers & Qt.ShiftModifier) !== 0, (ev.modifiers & Qt.AltModifier) !== 0)
            }
        }
        // Shift+Alt drag: a cursor on every line, at the column the pointer is in
        DragHandler {
            target: null
            acceptedModifiers: Qt.ShiftModifier | Qt.AltModifier
            onCentroidChanged: if (active) { const [l0] = cv.rowColAt(centroid.pressPosition.x, centroid.pressPosition.y); const [l1, c1] = cv.rowColAt(centroid.position.x, centroid.position.y); doc.columns(l0, l1, c1) }
        }
        DragHandler {
            target: null
            acceptedModifiers: Qt.NoModifier
            onActiveChanged: if (active) { input.forceActiveFocus(); const [l, c] = cv.rowColAt(centroid.pressPosition.x, centroid.pressPosition.y); doc.click(l, c, false, false) }
            onCentroidChanged: if (active) { const [l, c] = cv.rowColAt(centroid.position.x, centroid.position.y); doc.drag(l, c) }
        }
        WheelHandler {
            onWheel: (ev) => { cv.scrollY = Math.max(0, Math.min(cv.maxTop(), cv.scrollY - ev.angleDelta.y / 120 * cv.lh * 3)) }
        }
        HoverHandler { id: hov; cursorShape: Qt.IBeamCursor; onPointChanged: { cv.hoverAt = point.position; if (cv.fv) cv.fv.hoverText = ""; hoverAsk.restart() } }
    }
    // find / replace (Ctrl+F, Ctrl+H): over the top right, matches lit in the text, Enter / F3 next, Shift previous
    Rectangle {
        id: findBar
        visible: false
        property bool replacing: false
        property bool caseOn: false
        property bool regexOn: false
        property int cur: 0
        readonly property string opts: JSON.stringify({ case: caseOn, regex: regexOn })
        readonly property var matches: { doc.version; if (!visible || !q.text) return []; try { return JSON.parse(doc.find(q.text, opts)) } catch (e) { return [] } }
        function open(rep) { replacing = rep; visible = true; const s = doc.selectedText(); if (s && !s.includes("\n")) q.text = s; q.forceActiveFocus(); q.selectAll() }
        function step(d) { if (!matches.length) return; cur = (cur + d + matches.length) % matches.length; const m = matches[cur]; doc.select(m[0], m[1], m[2]) }
        anchors { right: parent.right; top: parent.top; margins: 10 }
        width: 380; height: col.implicitHeight + 16; radius: 14; z: 30
        color: cv.t.raised; border.width: 1; border.color: Qt.alpha(cv.t.rule, 0.4)
        component Tog: Rectangle {
            property string label
            property bool on
            signal hit
            width: 28; height: 24; radius: 8
            color: on ? Qt.alpha(cv.t.accent, 0.24) : th.hovered ? Qt.alpha(cv.t.text, 0.08) : "transparent"
            Text { anchors.centerIn: parent; text: parent.label; color: parent.on ? cv.t.accent : cv.t.dim; font.family: cv.t.mono; font.pixelSize: 11 }
            HoverHandler { id: th; cursorShape: Qt.PointingHandCursor }
            TapHandler { onTapped: parent.hit() }
        }
        Column {
            id: col
            anchors { left: parent.left; right: parent.right; top: parent.top; margins: 8 }
            spacing: 6
            Row {
                spacing: 6
                Rectangle { width: 200; height: 28; radius: 9; color: cv.t.card
                    TextInput { id: q; anchors { fill: parent; leftMargin: 10; rightMargin: 8 } verticalAlignment: TextInput.AlignVCenter; clip: true; color: cv.t.text; font.family: cv.t.mono; font.pixelSize: 12; selectionColor: Qt.alpha(cv.t.accent, 0.35)
                        onTextChanged: findBar.cur = 0
                        Keys.onReturnPressed: (e) => findBar.step(e.modifiers & Qt.ShiftModifier ? -1 : 1)
                        Keys.onEscapePressed: { findBar.visible = false; input.forceActiveFocus() }
                        Text { visible: !q.text; anchors.verticalCenter: parent.verticalCenter; text: "Find"; color: cv.t.faint; font: q.font } } }
                Tog { label: "Aa"; on: findBar.caseOn; onHit: findBar.caseOn = !findBar.caseOn }
                Tog { label: ".*"; on: findBar.regexOn; onHit: findBar.regexOn = !findBar.regexOn }
                Text { anchors.verticalCenter: parent.verticalCenter; width: 50; text: findBar.matches.length ? `${findBar.cur + 1}/${findBar.matches.length}` : q.text ? "none" : ""; color: cv.t.faint; font.family: cv.t.mono; font.pixelSize: 10 }
                Tog { label: "↑"; onHit: findBar.step(-1) }
                Tog { label: "↓"; onHit: findBar.step(1) }
            }
            Row {
                visible: findBar.replacing
                spacing: 6
                Rectangle { width: 200; height: 28; radius: 9; color: cv.t.card
                    TextInput { id: rq; anchors { fill: parent; leftMargin: 10; rightMargin: 8 } verticalAlignment: TextInput.AlignVCenter; clip: true; color: cv.t.text; font.family: cv.t.mono; font.pixelSize: 12
                        Keys.onReturnPressed: rall.hit()
                        Keys.onEscapePressed: { findBar.visible = false; input.forceActiveFocus() }
                        Text { visible: !rq.text; anchors.verticalCenter: parent.verticalCenter; text: "Replace"; color: cv.t.faint; font: rq.font } } }
                Rectangle { id: rall; signal hit; onHit: { const n = doc.replaceAll(q.text, rq.text, findBar.opts); if (cv.fv) { cv.fv.note = `${n} replaced`; } }
                    width: raT.implicitWidth + 22; height: 28; radius: 14; color: cv.t.accent
                    Text { id: raT; anchors.centerIn: parent; text: "Replace all"; color: cv.t.onAccent; font.family: cv.t.sans; font.pixelSize: 12; font.weight: Font.DemiBold }
                    HoverHandler { cursorShape: Qt.PointingHandCursor }
                    TapHandler { onTapped: rall.hit() } }
            }
        }
    }
    // the minimap: each line a bar as long as its text (indent left out), drawn once per change, the view framed
    Canvas {
        id: mini
        visible: (cv.win.prefs.minimap ?? true) && doc.rows > 40 && cv.width > 700
        anchors { right: parent.right; top: parent.top; bottom: parent.bottom; rightMargin: 14; topMargin: 8; bottomMargin: 8 }
        width: 70
        readonly property real per: Math.min(3, height / Math.max(1, doc.rows))
        onPaint: {
            const g = getContext("2d"); g.reset()
            const n = doc.rows, step = Math.max(1, Math.ceil(n / height)), col = Qt.alpha(cv.t.text, 0.28)
            g.fillStyle = col
            for (let r = 0, y = 0; r < n; r += step, y += per * step) {
                const s = doc.lineText(doc.rowLine(r)), lead = s.length - s.replace(/^\s+/, "").length
                const len = s.length - lead
                if (len > 0) g.fillRect(Math.min(lead, 40) * 0.6, y, Math.min(len, 110) * 0.55, Math.max(1, per * step - 0.5))
            }
        }
        Connections { target: doc; function onVersionChanged() { repaint.restart() } }
        Timer { id: repaint; interval: 120; onTriggered: mini.requestPaint() }
        onVisibleChanged: if (visible) requestPaint()
        Rectangle {
            y: cv.scrollY / cv.lh * mini.per; width: parent.width; height: Math.max(10, cv.height / cv.lh * mini.per)
            radius: 4; color: Qt.alpha(cv.t.text, mmH.hovered ? 0.14 : 0.08); border.width: 1; border.color: Qt.alpha(cv.t.text, 0.12)
        }
        function scrollTo(y) { cv.scrollY = Math.max(0, Math.min(cv.maxTop(), y / per * cv.lh - cv.height / 2)) }
        HoverHandler { id: mmH; cursorShape: Qt.PointingHandCursor }
        TapHandler { onTapped: (ev) => mini.scrollTo(ev.position.y) }
        DragHandler { target: null; onCentroidChanged: if (active) mini.scrollTo(centroid.position.y) }
    }
    // Claude's prompt at the cursor: what to do, or one of the quick ones
    Pop {
        id: aiBar
        visible: false
        function open() { visible = true; aiQ.text = ""; aiQ.forceActiveFocus() }
        function go(ask) { visible = false; cv.aiAsk(ask) }
        x: Math.max(8, Math.min(cv.cx - 20, cv.width - width - 8)); y: Math.min(cv.cy + cv.lh + 4, cv.height - height - 8)
        width: 460; height: aiCol.implicitHeight + 20; z: 60
        border.color: Qt.alpha(cv.t.accent, 0.5)
        Column {
            id: aiCol; x: 10; y: 10; width: parent.width - 20; spacing: 8
            Row { spacing: 8
                Text { text: "smart_toy"; color: cv.t.accent; font.family: cv.t.icons; font.pixelSize: 18; anchors.verticalCenter: parent.verticalCenter }
                TextInput { id: aiQ; width: aiCol.width - 30; anchors.verticalCenter: parent.verticalCenter; color: cv.t.text; font.family: cv.t.sans; font.pixelSize: 14; clip: true; selectionColor: Qt.alpha(cv.t.accent, 0.35)
                    Keys.onReturnPressed: aiBar.go(text)
                    Keys.onEscapePressed: { aiBar.visible = false; input.forceActiveFocus() }
                    Text { visible: !aiQ.text; text: "Ask Claude to change this code…"; color: cv.t.faint; font: aiQ.font } } }
            Flow { width: aiCol.width; spacing: 6
                Repeater {
                    model: [["build", "Fix it", "fix the bugs in it"], ["science", "Add tests", "add unit tests for it, in the language's usual style"], ["description", "Document", "add short doc comments to it"], ["auto_fix_high", "Simplify", "make it simpler and clearer without changing what it does"], ["help", "Explain", ""]]
                    delegate: Rectangle {
                        required property var modelData
                        width: qr.implicitWidth + 20; height: 26; radius: 13
                        color: qch.hovered ? Qt.alpha(cv.t.accent, 0.22) : Qt.alpha(cv.t.text, 0.06)
                        Row { id: qr; anchors.centerIn: parent; spacing: 5
                            Text { text: modelData[0]; color: cv.t.accent; font.family: cv.t.icons; font.pixelSize: 14 }
                            Text { text: modelData[1]; color: cv.t.text; font.family: cv.t.sans; font.pixelSize: 12 } }
                        HoverHandler { id: qch; cursorShape: Qt.PointingHandCursor }
                        // Explain goes to the Claude panel with the code; the rest change it here
                        TapHandler { onTapped: { if (modelData[2]) aiBar.go(modelData[2]); else { aiBar.visible = false; cv.win.askClaude("Explain this code from " + cv.file.replace(cv.app.dir + "/", "") + ":\n\n" + (doc.selectedText() || doc.lineText(cv.carets[0][0]))) } } }
                    }
                }
            }
        }
    }
    // the change: the old lines (red) and the new (green) under the code, Accept / Reject
    Pop {
        visible: cv.ai !== null
        x: 40; width: cv.width - 80
        y: Math.min(Math.max(8, (doc.lineRow(cv.ai ? cv.ai.l0 : 0) + 1) * cv.lh - cv.scrollY + 4), cv.height - height - 8)
        height: Math.min(cv.height * 0.6, diffCol.implicitHeight + 20); z: 58; clip: true
        border.color: Qt.alpha(cv.t.accent, 0.4)
        Column {
            id: diffCol; x: 12; y: 10; width: parent.width - 24; spacing: 6
            Row { spacing: 8
                Text { text: "smart_toy"; color: cv.t.accent; font.family: cv.t.icons; font.pixelSize: 16; anchors.verticalCenter: parent.verticalCenter }
                Text { text: cv.ai ? (cv.ai.busy ? "Claude is writing… (" + cv.ai.ask + ")" : cv.ai.err || cv.ai.ask) : ""; color: cv.ai && cv.ai.err ? cv.t.removed : cv.t.dim; font.family: cv.t.sans; font.pixelSize: 12; anchors.verticalCenter: parent.verticalCenter; elide: Text.ElideRight; width: diffCol.width - 260 }
                Rectangle { visible: cv.ai && !cv.ai.busy && !!cv.ai.out; width: acT.implicitWidth + 24; height: 26; radius: 13; color: cv.t.accent
                    Text { id: acT; anchors.centerIn: parent; text: "Accept  Tab"; color: cv.t.onAccent; font.family: cv.t.sans; font.pixelSize: 12; font.weight: Font.DemiBold }
                    TapHandler { onTapped: cv.aiAccept() } HoverHandler { cursorShape: Qt.PointingHandCursor } }
                Rectangle { width: rjT.implicitWidth + 24; height: 26; radius: 13; color: Qt.alpha(cv.t.text, 0.08)
                    Text { id: rjT; anchors.centerIn: parent; text: cv.ai && cv.ai.busy ? "Stop  Esc" : "Reject  Esc"; color: cv.t.text; font.family: cv.t.sans; font.pixelSize: 12 }
                    TapHandler { onTapped: cv.aiReject() } HoverHandler { cursorShape: Qt.PointingHandCursor } }
            }
            Repeater {
                model: cv.ai ? cv.ai.old.split("\n").filter((l, i, a) => i < a.length - 1 || l).map(l => ["-", l]).concat((cv.ai.out || "").split("\n").filter((l, i, a) => i < a.length - 1 || l).map(l => ["+", l])).slice(0, 400) : []
                delegate: Rectangle {
                    required property var modelData
                    width: diffCol.width; height: cv.lh - 2; radius: 5
                    color: Qt.alpha(modelData[0] === "+" ? cv.t.added : cv.t.removed, 0.14)
                    Text { x: 8; anchors.verticalCenter: parent.verticalCenter; text: modelData[0] + " " + modelData[1].replace(/\t/g, "    "); color: cv.t.text; font.family: cv.t.mono; font.pixelSize: 12 }
                }
            }
        }
    }
    // the scrollbar: a slim rounded thumb
    Rectangle {
        visible: doc.rows * cv.lh > cv.height
        anchors.right: parent.right; anchors.rightMargin: 4
        y: cv.scrollY / Math.max(1, doc.rows * cv.lh) * cv.height
        width: 6; radius: 3
        height: Math.max(30, cv.height * cv.height / Math.max(1, doc.rows * cv.lh))
        color: Qt.alpha(cv.t.text, sbH.hovered ? 0.3 : 0.15)
        HoverHandler { id: sbH }
        DragHandler { target: null; yAxis.enabled: true; xAxis.enabled: false; property real from: 0
            onActiveChanged: if (active) from = cv.scrollY
            onTranslationChanged: if (active) cv.scrollY = Math.max(0, Math.min(cv.maxTop(), from + translation.y * doc.rows * cv.lh / cv.height)) }
    }
}
