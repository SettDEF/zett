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
    readonly property bool dirty: newCore ? codeView.doc.dirty : ed.text !== saved   // the new core keeps its own text
    readonly property int curLine: newCore ? coreView.carets[0][0] : ed.line   // the cursor of whichever editor is in use
    readonly property int curCol: ed.col
    readonly property int lineCount: lineStarts.length
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
        // the new core: its text is the one to write (the old editor's would be stale)
        if (newCore) {
            // the same tidying as the classic editor: trailing spaces, a final newline (Settings > Editor)
            if (!auto && (win.prefs.trimOnSave ?? true)) codeView.doc.replaceAll("[ \\t]+$", "", JSON.stringify({ regex: true, multiline: true, case: true }))
            if (!auto && (win.prefs.finalNewline ?? true)) { const n = codeView.doc.lines, last = codeView.doc.lineText(n - 1); if (last.length) codeView.doc.applyEdits(JSON.stringify([[n - 1, last.length, n - 1, last.length, "\n"]])) }
            if (codeView.doc.save()) { note = "Saved"; loadGit(); if (lsp) app.lspAsk("save", win.filePath, 0, 0); fv.fire(["BufWritePost"]) } else note = "Could not save"; noteT.restart(); return }
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
    // gui-check: cursors on lines 1-3 at column 0, "// " typed at all, saved
    // gui-check: scrolled to the bottom, so the scope's first line sticks
    function testSticky() { fileFlick.contentY = Math.max(0, fileFlick.contentHeight - fileFlick.height); Qt.callLater(() => console.log("zett-probe: sticky " + JSON.stringify(sticky.heads.map(h => h.s.trim())))) }
    // gui-check: comment line 1, move it down, copy it down, save
    function testLines() { ed.forceActiveFocus(); ed.cursorPosition = 0; toggleComment(); moveLines(1); copyLines(1); save() }
    function testMulti() { ed.forceActiveFocus(); ed.cursorPosition = posOf(1, 0); addCursorLine(1); addCursorLine(1); multiEdit("// ", 0); multiEdit("", -1); multiEdit(" ", 0); save() }
    // gui-check: the server asked once it has the file (symbols, or rename:LINE:COL:NEW); what comes back is logged
    function testLsp(spec) { lspTest.spec = spec; lspTest.start() }
    Timer {
        id: lspTest; property string spec; interval: 1800; property int tries: 0
        onTriggered: { const p = spec.split(":"); if (p[0] === "symbols") fv.ask("symbols", 0); else if (p[0] === "sig") { fv.goTo(+p[1], +p[2]); fv.ask("signature", ed.cursorPosition); sigLog.start() } else if (p[0] === "act") { fv.goTo(+p[1], +p[2]); if (fv.diags.length || ++tries > 8) fv.askActions(); else restart() } else { fv.goTo(+p[1], +p[2]); renameField.text = p[3]; renameBox.done(true) } }
    }
    Timer { id: sigLog; interval: 1200; onTriggered: console.log("zett-probe: signature " + fv.sigLabel + " [" + fv.sigLabel.slice(fv.sigFrom, fv.sigTo) + "]") }
    onJumpsChanged: if (app.shot.length && jumps.length) console.log("zett-probe: jumps " + JSON.stringify(jumps.map(j => j.name ?? j.line)))
    // the cursor to a 0-based line and column (search results, problems)
    function cursorPos() { return ed.cursorPosition }
    function goTo(line, col) { if (newCore) { codeView.doc.select(line, col, 0); return } ed.cursorPosition = posOf(line, col); ed.forceActiveFocus() }
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
    Connections { target: fv.win; function onFilePathChanged() { fv.diags = []; fv.comps = []; fv.openLsp(); fv.loadGit(); fv.checkKept(); Qt.callLater(() => fv.fire(["BufReadPost", "FileType", "BufEnter"])) } }
    // the open file lent to Lua as its current buffer (plugins' commands and maps act on it): kept current a
    // moment after each change; what Lua changes comes back here
    // the events plugins wait for, fired with the file lent (its file type set first, as Neovim's detection does)
    function fire(events) {
        if (!fv.newCore || !fv.win.filePath) return
        fv.lendNow()
        const p = JSON.stringify(fv.win.filePath), list = events.map(e => JSON.stringify(e)).join(", ")
        fv.app.evalLua(`local ft = vim.filetype and vim.filetype.match({ filename = ${p} }) or "" if ft ~= "" and vim.bo.filetype ~= ft then vim.bo.filetype = ft end for _, ev in ipairs({ ${list} }) do vim._zett_fire(ev, ${p}) end`)
    }
    Timer { id: changedT; interval: 400; onTriggered: fv.fire(["TextChanged"]) }
    function lendNow() { const c = codeView.carets[0] ?? [0, 0]; fv.app.lendFile(fv.newCore && fv.win.filePath ? fv.win.filePath : "", fv.newCore ? codeView.doc.text() : "", c[0], c[1]) }
    Timer { id: lendT; interval: 150; onTriggered: fv.lendNow() }
    Connections { target: codeView.doc; function onVersionChanged() { lendT.restart(); changedT.restart() } }
    Connections { target: fv.app; function onLentChanged(text, line, col) { if (!fv.newCore || !fv.win.filePath) return; codeView.doc.restore(text); codeView.doc.click(line, col, false, false) } }
    // unsaved changes a zett that did not end well kept for this file: offered back (Restore / Drop)
    property var kept: null
    function checkKept() { const k = fv.win.filePath ? fv.app.keptUnsaved(fv.win.filePath) : ""; fv.kept = k ? JSON.parse(k) : null }
    function restoreKept() { if (!fv.kept) return; if (fv.newCore) codeView.doc.restore(fv.kept.text); else ed.text = fv.kept.text; fv.kept = null }
    function dropKept() { fv.app.dropUnsaved(fv.win.filePath); fv.kept = null }
    // git: the lines changed since the last commit, as bars in the gutter (green added, blue changed, red removed)
    property var gitMarks: []
    function loadGit() { try { gitMarks = visible && win.filePath ? JSON.parse(app.gitLines(win.filePath)) : [] } catch (e) { gitMarks = [] } }
    // the file's changes since the last commit, as a diff (Changes in the header, or a click in Source Control)
    // more cursors than the TextEdit's one (Alt+click, Ctrl+Alt+Up/Down): positions; typing, Backspace and Delete act at all
    property var extra: []
    function addCursor(p) { if (p >= 0 && p !== ed.cursorPosition && !extra.includes(p)) extra = extra.concat([p]) }
    function addCursorLine(d) {
        const from = extra.length ? extra[extra.length - 1] : ed.cursorPosition
        const [l, c] = lineCol(from), ls = ed.text.split("\n"), to = l + d
        if (to < 0 || to >= ls.length) return
        addCursor(posOf(to, Math.min(c, ls[to].length)))
    }
    // one edit at every cursor, from the last so earlier positions stay right: ins text, or del -1 / +1 chars
    function multiEdit(ins, del) {
        const all = extra.concat([ed.cursorPosition]).filter((p, i, a) => a.indexOf(p) === i).sort((a, b) => a - b)
        const main = ed.cursorPosition
        for (let i = all.length - 1; i >= 0; i--) {
            const p = all[i]
            if (del < 0 && p > 0) ed.remove(p - 1, p)
            else if (del > 0 && p < ed.text.length) ed.remove(p, p + 1)
            else if (ins) ed.insert(p, ins)
        }
        const step = ins ? ins.length : del < 0 ? -1 : 0
        const moved = all.map((p, i) => p + step * (i + 1) - (del < 0 && p === 0 ? step : 0))
        const mi = all.indexOf(main)
        ed.cursorPosition = Math.max(0, moved[mi])
        extra = moved.filter((_, i) => i !== mi).map(p => Math.max(0, p))
    }
    // word wrap (Alt+Z, Settings > Editor): the editor and its coloured copy wrap alike; line numbers follow the wrapped lines
    property bool wrapTest: false     // gui-check turns wrap on without touching the settings
    readonly property bool wrap: wrapTest || (win.prefs.wordWrap ?? false)
    readonly property var lineStarts: { const s = [0]; const t2 = shownText; for (let i = t2.indexOf("\n"); i >= 0; i = t2.indexOf("\n", i + 1)) s.push(i + 1); return s }
    // ── line editing as in VS Code: Ctrl+/ comment, Alt+Up/Down move, Shift+Alt+Up/Down copy, Ctrl+Shift+K delete ──
    readonly property string commentMark: /^(py|sh|bash|zsh|toml|ya?ml|rb|nix|conf|r|pl|mk|cmake)$/.test(ext) || /(^|\/)(Makefile|Dockerfile)$/.test(win.filePath) ? "#" : /^(lua|sql|hs|elm)$/.test(ext) ? "--" : "//"
    // the lines the cursor or selection covers: [first, last], 0-based
    function selLines() { const a = lineCol(ed.selectionStart)[0], e = ed.selectionEnd; let b = lineCol(e)[0]; if (b > a && ed.selectedText.length && ed.text[e - 1] === "\n") b--; return [a, b] }
    // lines a..b replaced by `rows` (one undo step); the cursor kept on its line, moved by `shift` lines
    function replaceLines(a, b, rows, shift) {
        const ls = ed.text.split("\n"), [cl, cc] = lineCol(ed.cursorPosition)
        const from = posOf(a, 0), to = b + 1 < ls.length ? posOf(b + 1, 0) : ed.text.length
        const tail = b + 1 < ls.length ? "\n" : ""
        ed.remove(from, to); ed.insert(from, rows.join("\n") + tail)
        const nl = Math.max(0, Math.min(cl + shift, ed.text.split("\n").length - 1))
        ed.cursorPosition = Math.min(posOf(nl, 0) + cc, posOf(nl, 0) + (ed.text.split("\n")[nl] ?? "").length)
    }
    function toggleComment() {
        const [a, b] = selLines(), ls = ed.text.split("\n").slice(a, b + 1), m = commentMark
        const lead = s => s.length - s.replace(/^\s+/, "").length
        const all = ls.filter(s => s.trim()).every(s => s.trim().startsWith(m))
        const ind = Math.min(...ls.filter(s => s.trim()).map(lead).concat([1e9]))
        replaceLines(a, b, ls.map(s => !s.trim() ? s : all ? s.replace(new RegExp("^(\\s*)" + m.replace(/[/]/g, "\\/") + " ?"), "$1") : s.slice(0, ind) + m + " " + s.slice(ind)), 0)
    }
    function moveLines(d) {
        const [a, b] = selLines(), ls = ed.text.split("\n")
        if (d < 0 && a === 0 || d > 0 && b >= ls.length - 1) return
        if (d < 0) replaceLines(a - 1, b, ls.slice(a, b + 1).concat([ls[a - 1]]), -1)
        else replaceLines(a, b + 1, [ls[b + 1]].concat(ls.slice(a, b + 1)), 1)
    }
    function copyLines(d) { const [a, b] = selLines(), rows = ed.text.split("\n").slice(a, b + 1); replaceLines(a, b, rows.concat(rows), d > 0 ? b - a + 1 : 0) }
    function deleteLines() { const [a, b] = selLines(), ls = ed.text.split("\n"); if (ls.length === 1) { ed.text = ""; return } const from = posOf(a, 0), to = b + 1 < ls.length ? posOf(b + 1, 0) : ed.text.length; ed.remove(a > 0 && b + 1 >= ls.length ? from - 1 : from, to) }
    // the file's declarations (fn, impl, class, def …): the outline card and the header's breadcrumb read it
    readonly property var outline: {
        const out = [], ls = shownText.split("\n")
        const re = /^(\s*)(?:pub(?:\([^)]*\))?\s+|export\s+|async\s+|static\s+|default\s+)*(fn|impl|struct|enum|trait|mod|type|class|def|function|interface|const fn)\s+([\w:<>, ]+?)\s*[({:<=\[]/
        for (let i = 0; i < ls.length && out.length < 400; i++) { const m = re.exec(ls[i]); if (m) out.push({ line: i, depth: Math.floor(m[1].replace(/\t/g, "    ").length / 4), kind: m[2], name: m[3].trim() }) }
        return out
    }
    // the declaration the cursor is in, and the ones around it (impl › fn)
    readonly property var symbolPath: { const p = []; for (const it of outline) { if (it.line > curLine) break; while (p.length && p[p.length - 1].depth >= it.depth) p.pop(); p.push(it) } return p }
    property bool showDiff: false
    property var diffRows: []
    // side by side: old on the left, new on the right; removed and added lines paired up row by row
    property bool diffSplit: win.prefs?.diffSplit ?? false
    readonly property var splitRows: {
        const out = []; let dels = [], adds = []
        const flush = () => { for (let i = 0; i < Math.max(dels.length, adds.length); i++) out.push({ k: "pair", l: dels[i] ?? null, r: adds[i] ?? null }); dels = []; adds = [] }
        for (const r of diffRows) {
            if (r.k === "del") { dels.push({ n: r.a, t: r.text, k: "del" }); continue }
            if (r.k === "add") { adds.push({ n: r.b, t: r.text, k: "add" }); continue }
            flush()
            if (r.k === "hunk") out.push(r)
            else out.push({ k: "pair", l: { n: r.a, t: r.text, k: "same" }, r: { n: r.b, t: r.text, k: "same" } })
        }
        flush()
        return out
    }
    function loadDiff() {
        let r; try { r = JSON.parse(app.git(JSON.stringify(["diff", "--no-color", "-U3", "HEAD", "--", win.filePath]))) } catch (e) { r = { out: "" } }
        const rows = []; let a = 0, b = 0
        for (const l of r.out.split("\n")) {
            const h = /^@@ -(\d+)(?:,\d+)? \+(\d+)/.exec(l)
            if (h) { a = +h[1]; b = +h[2]; rows.push({ k: "hunk", text: l.replace(/^@@[^@]*@@ ?/, "") || "…", body: [] }); continue }
            if (!rows.length || /^(\+\+\+|---) /.test(l)) continue
            // each change keeps its own lines, to find it again in the index diff (stage, discard)
            for (let i = rows.length - 1; i >= 0; i--) if (rows[i].k === "hunk") { if (/^[-+ ]/.test(l)) rows[i].body.push(l); break }
            if (l[0] === "+") rows.push({ k: "add", b: b++, text: l.slice(1) })
            else if (l[0] === "-") rows.push({ k: "del", a: a++, text: l.slice(1) })
            else if (l[0] === " ") rows.push({ k: "same", a: a++, b: b++, text: l.slice(1) })
        }
        diffRows = rows
    }
    // one change (hunk) of the diff: stage it (git apply --cached), or throw it away in the file (git apply -R);
    // found in the index's diff by its lines, so changes staged before do not shift it
    property string hunkNote: ""
    onHunkNoteChanged: if (hunkNote) { note = hunkNote; noteT.restart(); hunkNote = "" }
    function applyHunk(body, mode) {
        if (mode !== "stage" && fv.dirty) { hunkNote = "Save first: discarding works on the file on disk"; return }
        let r; try { r = JSON.parse(app.git(JSON.stringify(["diff", "--no-color", "-U3", "--", win.filePath]))) } catch (e) { return }
        const lines = r.out.split("\n"), head = [], hunks = []
        for (const l of lines) {
            if (l.startsWith("@@")) hunks.push([l])
            else if (hunks.length) { if (/^[-+ \\]/.test(l)) hunks[hunks.length - 1].push(l) }
            else head.push(l)
        }
        const want = body.join("\n")
        const h = hunks.find(x => x.slice(1).filter(l => !l.startsWith("\\")).join("\n") === want)
        if (!h) { hunkNote = mode === "stage" ? "Already staged" : "Not in the working copy any more"; loadDiff(); return }
        // in the repository's own .git folder (writeFile keeps to the workspace)
        const gd = JSON.parse(app.git(JSON.stringify(["rev-parse", "--absolute-git-dir"]))).out.trim()
        const patch = gd + "/zett-hunk.patch"
        app.writeFile(patch, head.join("\n") + "\n" + h.join("\n") + "\n")
        const res = JSON.parse(app.git(JSON.stringify(mode === "stage" ? ["apply", "--cached", patch] : ["apply", "-R", patch])))
        hunkNote = res.code === 0 ? (mode === "stage" ? "Staged" : "Discarded") : res.out.trim().split("\n").pop()
        if (mode !== "stage" && res.code === 0) { win.fileText = app.readFile(win.filePath); if (fv.newCore) codeView.openHere(win.filePath) }
        loadDiff()
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
                const word = (fv.newCore ? codeView.word() : ed.text.slice(fv.compFrom, ed.cursorPosition)).toLowerCase()
                fv.comps = e.items.filter(i => (i.filter || i.label).toLowerCase().includes(word)).sort((a, b) => (a.sort || a.label).localeCompare(b.sort || b.label)).slice(0, 50)
                fv.compSel = 0
            }
            else if (e.type === "hover") fv.hoverText = e.text.replace(/```\w*\n?/g, "").trim()
            else if (e.type === "definition" && e.items.length) {
                const d = e.items[0]
                if (d.path !== fv.win.filePath) fv.win.openFile({ path: d.path, rel: d.path })
                Qt.callLater(() => fv.goTo(d.line, d.col))
            }
            else if (e.type === "edits") for (const f of e.files) if (f.path === fv.win.filePath) {
                if (fv.newCore) { codeView.doc.applyEdits(JSON.stringify(f.edits)); continue }
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
            if (e.type === "signature") { fv.sigLabel = e.label; fv.sigFrom = e.from; fv.sigTo = e.to }
            if (e.type === "actions") { fv.actions = e.items; fv.actionSel = 0; if (!e.items.length) { fv.note = "No code actions here"; noteT.restart() } else if (fv.app.shot.length) fv.takeAction(0) }
            if (e.type === "references" || e.type === "symbols") { fv.jumpKind = e.type; fv.jumps = e.items; fv.jumpSel = 0 }
            if (e.type === "edits" && fv.savePending) { fv.savePending = false; saveWait.stop(); fv.write() }
        }
    }
    function takeComp(i) {
        const it = comps[i]; if (!it) return
        ed.remove(compFrom, ed.cursorPosition); ed.insert(compFrom, it.insert || it.label)
        comps = []
    }

    // the file's buttons: in the tab row (win.fileActionSlot), so the code starts right under the tabs
    RowLayout {
        parent: fv.win.fileActionSlot
        visible: parent !== null
        width: implicitWidth; height: 30
        spacing: 8
        Text { visible: fv.note.length > 0; text: fv.note; color: t.accent; font.family: t.sans; font.pixelSize: 12 }
        // the server: its errors and warnings for this file
        Text { visible: fv.lsp && fv.errors + fv.warnings > 0; text: `${fv.errors} errors  ${fv.warnings} warnings`; color: fv.errors ? t.removed : fv.warnings ? "#d8a54a" : t.faint; font.family: t.mono; font.pixelSize: 11 
            HoverHandler { cursorShape: fv.diags.length ? Qt.PointingHandCursor : Qt.ArrowCursor }
            TapHandler { onTapped: fv.showProblems = !fv.showProblems }
        }
        Repeater {
            model: fv.kept ? [["restore_page", "Restore unsaved changes", () => fv.restoreKept()], ["delete", "Drop", () => fv.dropKept()]] : fv.dirty ? [["undo", "Revert", () => fv.revert()], ["save", "Save  Ctrl S", () => fv.save()]] : (fv.gitMarks.length || fv.showDiff ? [[fv.showDiff ? "edit" : "difference", fv.showDiff ? "Edit" : "Changes", () => fv.showDiff = !fv.showDiff]] : []).concat([["open_in_new", "Open with the system", () => Qt.openUrlExternally("file://" + fv.win.filePath)]])
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
        Layout.leftMargin: 8; Layout.rightMargin: 8
        radius: 17; color: t.card
        // Unified / Side by side, kept as a preference
        Rectangle {
            z: 5; anchors { top: parent.top; right: parent.right; margins: 10 }
            width: swRow.implicitWidth + 6; height: 26; radius: 13; color: t.raised
            Row {
                id: swRow; anchors.centerIn: parent; spacing: 2
                Repeater {
                    model: [[false, "view_agenda", "Unified"], [true, "view_column", "Side by side"]]
                    delegate: Rectangle {
                        required property var modelData
                        readonly property bool on: fv.diffSplit === modelData[0]
                        width: swT.implicitWidth + 30; height: 22; radius: 11
                        color: on ? Qt.alpha(t.accent, 0.22) : swH.hovered ? Qt.alpha(t.text, 0.06) : "transparent"
                        Row { anchors.centerIn: parent; spacing: 4
                            Text { text: modelData[1]; color: parent.parent.on ? t.accent : t.faint; font.family: t.icons; font.pixelSize: 14 }
                            Text { id: swT; text: modelData[2]; color: parent.parent.on ? t.text : t.dim; font.family: t.sans; font.pixelSize: 11 } }
                        HoverHandler { id: swH; cursorShape: Qt.PointingHandCursor }
                        TapHandler { onTapped: fv.win.setPref("diffSplit", modelData[0]) }
                    }
                }
            }
        }
        ListView {
            visible: fv.diffSplit
            anchors { fill: parent; margins: 12; topMargin: 42 }
            clip: true; boundsBehavior: Flickable.StopAtBounds
            model: fv.splitRows
            delegate: Item {
                id: sr
                required property var modelData
                width: ListView.view.width; height: modelData.k === "hunk" ? 30 : 20
                Text { visible: sr.modelData.k === "hunk"; x: 8; anchors.verticalCenter: parent.verticalCenter; text: sr.modelData.text ?? ""; color: t.accent; font.family: t.sans; font.pixelSize: 12 }
                Repeater {
                    model: sr.modelData.k === "pair" ? [sr.modelData.l, sr.modelData.r] : []
                    delegate: Rectangle {
                        required property var modelData
                        required property int index
                        x: index * (sr.width / 2 + 2); width: sr.width / 2 - 2; height: 20; radius: 6
                        color: !modelData ? Qt.alpha(t.text, 0.03) : modelData.k === "add" ? Qt.alpha(t.added, 0.14) : modelData.k === "del" ? Qt.alpha(t.removed, 0.14) : "transparent"
                        clip: true
                        Row {
                            visible: !!modelData
                            anchors.verticalCenter: parent.verticalCenter; spacing: 8
                            Text { width: 36; horizontalAlignment: Text.AlignRight; text: modelData ? modelData.n : ""; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
                            Text { text: modelData ? modelData.t.replace(/\t/g, "    ") : ""; color: t.text; font.family: t.mono; font.pixelSize: 12 }
                        }
                    }
                }
            }
        }
        ListView {
            visible: !fv.diffSplit
            anchors { fill: parent; margins: 12; topMargin: 42 }
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
                // a change: Stage it, or Discard it in the file (asks once: a second click within 3 s)
                Row {
                    visible: modelData.k === "hunk"
                    anchors { right: parent.right; rightMargin: 4; verticalCenter: parent.verticalCenter }
                    spacing: 6
                    Repeater {
                        model: [["stage", "add", "Stage"], ["discard", "undo", "Discard"]]
                        delegate: Rectangle {
                            required property var modelData
                            property bool armed: false
                            Timer { id: disarmT; interval: 3000; onTriggered: parent.armed = false }
                            width: hr.implicitWidth + 18; height: 22; radius: 11
                            color: armed ? Qt.alpha(t.removed, 0.25) : hh.hovered ? t.raised : Qt.alpha(t.text, 0.06)
                            Row { id: hr; anchors.centerIn: parent; spacing: 4
                                Text { text: modelData[1]; color: parent.parent.armed ? t.removed : t.dim; font.family: t.icons; font.pixelSize: 14 }
                                Text { text: parent.parent.armed ? "Discard?" : modelData[2]; color: parent.parent.armed ? t.removed : t.dim; font.family: t.sans; font.pixelSize: 11 } }
                            HoverHandler { id: hh; cursorShape: Qt.PointingHandCursor }
                            TapHandler { onTapped: {
                                const body = parent.parent.parent.parent.modelData.body
                                if (modelData[0] === "stage") fv.applyHunk(body, "stage")
                                else if (parent.armed) { parent.armed = false; fv.applyHunk(body, "discard") }
                                else { parent.armed = true; disarmT.restart() }
                            } }
                        }
                    }
                }
            }
            Text { visible: !fv.diffRows.length; anchors.centerIn: parent; text: "No changes since the last commit"; color: t.faint; font.family: t.sans; font.pixelSize: 13 }
        }
    }
    // the new editor core (Settings > Editor > New editor core): CodeView over CodeDoc, in place of the TextEdit
    readonly property alias codeDoc: codeView.doc
    readonly property alias codeView: codeView
    readonly property alias coreView: codeView
    // the default now; Settings can go back to the classic editor. Headless checks use the classic one unless they test the core
    readonly property bool newCore: app.shot.length ? app.shotEngine.startsWith("core") : (win.prefs.newCore ?? true)
    CodeView {
        id: codeView
        visible: fv.newCore && !fv.showDiff
        Layout.fillWidth: true; Layout.fillHeight: true
        Layout.leftMargin: 8; Layout.rightMargin: 8
        win: fv.win; t: fv.t; app: fv.app; fv: fv
    }
    // a server question at a line and column (the new core has no TextEdit position)
    function askAt(kind, l, c) { if (lsp) app.lspAsk(kind, win.filePath, l, c) }
    // find / replace / go to line (FindBar.qml)
    FindBar { id: findBar; t: fv.t; editor: ed; Layout.fillWidth: true; Layout.leftMargin: 8; Layout.rightMargin: 8 }
    Rectangle {
        visible: !fv.showDiff && !fv.newCore
        Layout.fillWidth: true
        Layout.fillHeight: true
        Layout.leftMargin: 8; Layout.rightMargin: 8
        radius: 17
        color: t.card
        // the minimap: the whole file small, the part in view framed; a click or drag scrolls there;
        // errors and find hits as marks on its edge
        Item {
            id: miniMap
            visible: (fv.win.prefs.minimap ?? true) && ed.lineCount > 40 && parent.width > 700
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
        // sticky scroll: the lines that open the scope at the top of the view (fn … {, impl, class, def …:), at most 3
        Rectangle {
            id: sticky
            readonly property var heads: {
                if (!fv.visible || ed.lineH <= 0) return []
                const top = Math.floor(fileFlick.contentY / ed.lineH) + 1
                if (top < 2) return []
                const ls = fv.shownText.split("\n"), ind = s => s.length - s.replace(/^\s+/, "").length
                let min = ind(ls[Math.min(top, ls.length - 1)] ?? ""), out = []
                for (let i = Math.min(top, ls.length) - 1; i >= 0 && min > 0 && out.length < 3; i--) {
                    const s = ls[i]; if (!s.trim()) continue
                    if (ind(s) < min && (/[{:(]\s*$/.test(s) || /^\s*(pub |async |export )*(fn|impl|struct|enum|trait|mod|class|def|function|interface)\b/.test(s))) { out.unshift({ i, s }); min = ind(s) }
                }
                return out
            }
            visible: heads.length > 0
            anchors { left: fileFlick.left; right: fileFlick.right; top: parent.top; topMargin: 8 }
            height: stickyCol.implicitHeight + 8; radius: 12; z: 10
            color: t.raised; border.width: 1; border.color: Qt.alpha(t.rule, 0.3)
            Column {
                id: stickyCol
                anchors { left: parent.left; right: parent.right; verticalCenter: parent.verticalCenter }
                Repeater {
                    model: sticky.heads
                    delegate: Item {
                        required property var modelData
                        width: stickyCol.width; height: ed.lineH
                        Text { x: 0; width: 40; horizontalAlignment: Text.AlignRight; anchors.verticalCenter: parent.verticalCenter; text: modelData.i + 1; color: t.faint; font.family: t.mono; font.pixelSize: 12 }
                        Text { x: 54; anchors.verticalCenter: parent.verticalCenter; text: modelData.s.replace(/\t/g, "    "); color: t.text; font: ed.font; elide: Text.ElideRight; width: parent.width - 60 }
                        HoverHandler { cursorShape: Qt.PointingHandCursor }
                        TapHandler { onTapped: fv.goTo(modelData.i, 0) }
                    }
                }
            }
        }
        Flickable {
            id: fileFlick
            anchors { fill: parent; margins: 10; rightMargin: miniMap.visible ? 10 + miniMap.width + 6 : 10 }
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
            // line numbers, the cursor's lit; with wrap each at its line's first row
            Item {
                x: 0; width: 40
                Repeater {
                    model: fv.wrap ? fv.lineStarts.length : ed.lineCount
                    delegate: Text {
                        required property int index
                        y: fv.wrap ? (ed.contentHeight, ed.width, ed.positionToRectangle(fv.lineStarts[index] ?? 0).y) : index * ed.lineH   // re-read when the layout changes
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
                width: fv.wrap ? ed.width : undefined
                wrapMode: fv.wrap ? Text.WordWrap : Text.NoWrap
                text: !fv.visible ? "" : fv.wrap ? app.highlight(fv.ext, fv.shownText).replace("white-space:pre\"", "white-space:pre-wrap\"") : app.highlight(fv.ext, fv.shownText)
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
            // signature help over the cursor: the active parameter in bold
            Rectangle {
                visible: fv.sigLabel.length > 0 && ed.activeFocus
                x: Math.max(0, ed.x + ed.cursorRectangle.x - 20); y: Math.max(0, ed.cursorRectangle.y - height - 6)
                width: Math.min(620, sigT.implicitWidth + 24); height: sigT.implicitHeight + 14; radius: 12; z: 55
                color: t.raised; border.width: 1; border.color: Qt.alpha(t.rule, 0.4)
                Text {
                    id: sigT; x: 12; anchors.verticalCenter: parent.verticalCenter; width: Math.min(596, implicitWidth)
                    readonly property var e: s => s.replace(/&/g, "&amp;").replace(/</g, "&lt;")
                    text: fv.sigTo > fv.sigFrom ? e(fv.sigLabel.slice(0, fv.sigFrom)) + `<b><font color="${t.accent}">` + e(fv.sigLabel.slice(fv.sigFrom, fv.sigTo)) + "</font></b>" + e(fv.sigLabel.slice(fv.sigTo)) : e(fv.sigLabel)
                    textFormat: Text.StyledText; wrapMode: Text.Wrap
                    color: t.dim; font.family: t.mono; font.pixelSize: 12
                }
            }
            // code actions (Ctrl+.) under the cursor: ↑ ↓ Enter Esc
            Rectangle {
                visible: fv.actions.length > 0
                x: ed.x + ed.cursorRectangle.x; y: ed.cursorRectangle.y + ed.cursorRectangle.height + 4
                width: 360; height: Math.min(260, actCol.implicitHeight + 12); radius: 12; z: 56
                color: t.raised; border.width: 1; border.color: Qt.alpha(t.rule, 0.4)
                Column {
                    id: actCol; x: 6; y: 6; width: parent.width - 12
                    Repeater {
                        model: fv.actions
                        delegate: Rectangle {
                            required property var modelData
                            required property int index
                            width: actCol.width; height: 28; radius: 8
                            color: index === fv.actionSel ? Qt.alpha(t.accent, 0.18) : acH.hovered ? Qt.alpha(t.text, 0.06) : "transparent"
                            Row { x: 8; anchors.verticalCenter: parent.verticalCenter; spacing: 8
                                Text { text: "lightbulb"; color: t.accent; font.family: t.icons; font.pixelSize: 15 }
                                Text { text: modelData.title; color: t.text; font.family: t.sans; font.pixelSize: 12; elide: Text.ElideRight; width: actCol.width - 40 } }
                            HoverHandler { id: acH; cursorShape: Qt.PointingHandCursor }
                            TapHandler { onTapped: fv.takeAction(index) }
                        }
                    }
                }
            }
            // the extra cursors
            Repeater {
                model: fv.extra
                delegate: Rectangle { required property int modelData; readonly property rect r: ed.positionToRectangle(modelData); x: ed.x + r.x; y: r.y; width: 2; height: r.height; radius: 1; color: t.accent; z: 40 }
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
                width: fv.wrap ? fileFlick.width - 60 : Math.max(fileFlick.width - 60, contentWidth)
                text: fv.win.fileText
                color: "transparent"
                selectedTextColor: "transparent"
                selectionColor: Qt.alpha(t.accent, 0.3)
                selectByMouse: true
                wrapMode: fv.wrap ? TextEdit.WordWrap : TextEdit.NoWrap
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
                    // signature help: asked after ( and , ; gone after )
                    if (fv.lsp && activeFocus && (prev === "(" || prev === ",")) { lspSync.stop(); fv.app.lspChange(fv.win.filePath, text); fv.ask("signature", p) }
                    else if (prev === ")") fv.sigLabel = ""
                }
                // hover: the mouse resting on a word asks the server about it
                HoverHandler { id: edHover; onPointChanged: { fv.hoverText = ""; hoverAsk.restart() } }
                Timer { id: hoverAsk; interval: 650; onTriggered: if (edHover.hovered && fv.lsp) { const pt = edHover.point.position; fv.hoverAt = Qt.point(ed.x + pt.x, pt.y); fv.ask("hover", ed.positionAt(pt.x, pt.y)) } }
                // Shift+Alt+drag: a cursor on every line from where the drag began, at the column it is at
                DragHandler {
                    acceptedModifiers: Qt.ShiftModifier | Qt.AltModifier
                    target: null
                    onCentroidChanged: if (active) {
                        const p0 = ed.positionAt(centroid.pressPosition.x, centroid.pressPosition.y), p1 = ed.positionAt(centroid.position.x, centroid.position.y)
                        const [l0] = fv.lineCol(p0), [l1, c1] = fv.lineCol(p1), ls = ed.text.split("\n"), ps = []
                        for (let l = Math.min(l0, l1); l <= Math.max(l0, l1); l++) ps.push(fv.posOf(l, Math.min(c1, ls[l].length)))
                        ed.cursorPosition = ps[0]; fv.extra = ps.slice(1)
                    }
                }
                // Alt+click: one more cursor there
                TapHandler { acceptedModifiers: Qt.AltModifier; onTapped: (ev) => fv.addCursor(ed.positionAt(ev.position.x, ev.position.y)) }
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
                    // more cursors: new ones with Ctrl+Alt+Up/Down; text, Backspace, Delete at all; Esc or an arrow back to one
                    if ((e.modifiers & Qt.ControlModifier) && (e.modifiers & Qt.AltModifier) && (e.key === Qt.Key_Up || e.key === Qt.Key_Down)) { fv.addCursorLine(e.key === Qt.Key_Up ? -1 : 1); e.accepted = true; return }
                    if (fv.extra.length) {
                        if (e.key === Qt.Key_Escape || e.key === Qt.Key_Left || e.key === Qt.Key_Right || e.key === Qt.Key_Up || e.key === Qt.Key_Down) { fv.extra = []; if (e.key === Qt.Key_Escape) { e.accepted = true; return } }
                        else if (e.key === Qt.Key_Backspace) { fv.multiEdit("", -1); e.accepted = true; return }
                        else if (e.key === Qt.Key_Delete) { fv.multiEdit("", 1); e.accepted = true; return }
                        else if (e.key === Qt.Key_Return || e.key === Qt.Key_Enter) { fv.multiEdit("\n", 0); e.accepted = true; return }
                        else if (e.text.length && !(e.modifiers & Qt.ControlModifier) && e.text.charCodeAt(0) >= 32) { fv.multiEdit(e.text, 0); e.accepted = true; return }
                    }
                    if (fv.actions.length) {
                        if (e.key === Qt.Key_Down) { fv.actionSel = (fv.actionSel + 1) % fv.actions.length; e.accepted = true; return }
                        if (e.key === Qt.Key_Up) { fv.actionSel = (fv.actionSel - 1 + fv.actions.length) % fv.actions.length; e.accepted = true; return }
                        if (e.key === Qt.Key_Return || e.key === Qt.Key_Enter) { fv.takeAction(fv.actionSel); e.accepted = true; return }
                        if (e.key === Qt.Key_Escape) { fv.actions = []; e.accepted = true; return }
                    }
                    if (e.key === Qt.Key_Period && (e.modifiers & Qt.ControlModifier) && fv.lsp) { fv.askActions(); e.accepted = true; return }
                    if (e.key === Qt.Key_Escape && fv.sigLabel) { fv.sigLabel = ""; e.accepted = true; return }
                    // line editing
                    if (e.key === Qt.Key_Slash && (e.modifiers & Qt.ControlModifier)) { fv.toggleComment(); e.accepted = true; return }
                    if (e.key === Qt.Key_K && (e.modifiers & Qt.ControlModifier) && (e.modifiers & Qt.ShiftModifier)) { fv.deleteLines(); e.accepted = true; return }
                    if ((e.key === Qt.Key_Up || e.key === Qt.Key_Down) && (e.modifiers & Qt.AltModifier) && !(e.modifiers & Qt.ControlModifier) && !fv.comps.length && !fv.jumps.length && !fv.actions.length) { if (e.modifiers & Qt.ShiftModifier) fv.copyLines(e.key === Qt.Key_Up ? -1 : 1); else fv.moveLines(e.key === Qt.Key_Up ? -1 : 1); e.accepted = true; return }
                    // the jump list takes ↑ ↓ Enter Esc while open
                    if (fv.jumps.length) {
                        if (e.key === Qt.Key_Down) { fv.jumpSel = (fv.jumpSel + 1) % fv.jumps.length; e.accepted = true; return }
                        if (e.key === Qt.Key_Up) { fv.jumpSel = (fv.jumpSel - 1 + fv.jumps.length) % fv.jumps.length; e.accepted = true; return }
                        if (e.key === Qt.Key_Return || e.key === Qt.Key_Enter) { fv.jumpTo(fv.jumps[fv.jumpSel]); e.accepted = true; return }
                        if (e.key === Qt.Key_Escape) { fv.jumps = []; e.accepted = true; return }
                    }
                    if (e.key === Qt.Key_Z && (e.modifiers & Qt.AltModifier)) { fv.win.setPref("wordWrap", !fv.wrap); e.accepted = true; return }
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
    // signature help (after '(' or ',') and code actions (Ctrl+.)
    property string sigLabel: ""
    property int sigFrom: 0
    property int sigTo: 0
    property var actions: []
    property int actionSel: 0
    function askActions(atLine, atCol) {
        const l = atLine ?? ed.line
        const ds = diags.filter(d => d.line <= l && (d.endLine ?? d.line) >= l).map(d => ({ range: { start: { line: d.line, character: d.col }, end: { line: d.endLine ?? d.line, character: d.endCol ?? d.col } }, severity: d.severity, message: d.message }))
        if (atLine !== undefined) { app.lspChange(win.filePath, codeView.doc.text()); askAt("actions:" + JSON.stringify(ds), atLine, atCol); return }
        app.lspChange(win.filePath, ed.text); ask("actions:" + JSON.stringify(ds), ed.cursorPosition)
    }
    // edits per file: the open one in the buffer, others on disk
    function applyFiles(files) {
        for (const f of files) {
            if (f.path === win.filePath) { if (newCore) { codeView.doc.applyEdits(JSON.stringify(f.edits)); continue } for (const x of f.edits.slice().reverse()) { const a = posOf(x[0], x[1]), b = posOf(x[2], x[3]); ed.remove(a, b); ed.insert(a, x[4]) } continue }
            let s = app.readFile(f.path)
            const at = (l, c) => { const ls = s.split("\n"); let p = 0; for (let i = 0; i < Math.min(l, ls.length - 1); i++) p += ls[i].length + 1; return p + c }
            for (const x of f.edits.slice().reverse()) { const a = at(x[0], x[1]), b = at(x[2], x[3]); s = s.slice(0, a) + x[4] + s.slice(b) }
            app.writeFile(f.path, s)
        }
    }
    function takeAction(i) { const a = actions[i]; actions = []; if (!a) return; applyFiles(a.files); note = a.title; noteT.restart(); if (app.shot.length) { console.log("zett-probe: action " + a.title); write() } }
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
        Layout.fillWidth: true; Layout.leftMargin: 8; Layout.rightMargin: 8
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
        Layout.fillWidth: true; Layout.leftMargin: 8; Layout.rightMargin: 8
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
