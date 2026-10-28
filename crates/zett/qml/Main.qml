import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window
import dev.zett.app

// zett's first window: the pages of a workspace folder and the open page as
// blocks. Every edit goes to Rust (window.rs) and is saved to its .md file.
ApplicationWindow {
    id: win
    width: 1180
    height: 760
    visible: true
    title: app.title.length ? `${app.title} — zett` : "zett"
    color: t.bg

    Zett { id: app }
    // ZETT_SHOT_ENGINE=timing: the time of the first frame on stdout, then quit (start-up measured from outside)
    property bool firstFrame: false
    // ZETT_SHOT_ENGINE=typing:N types 60 letters into block N, one per frame, and prints the time from each
    // letter to the frame that shows it (median, 95th, worst)
    property var typeTimes: []
    property real typeAt: 0
    property int typeBlock: -1
    function typeNext() {
        if (typeTimes.length >= 60) {
            const s = typeTimes.slice().sort((a, b) => a - b)
            console.log(`zett-probe: typing block ${typeBlock}: median ${s[30].toFixed(1)} ms, p95 ${s[56].toFixed(1)} ms, worst ${s[59].toFixed(1)} ms`)
            Qt.quit(); return
        }
        const d = list.itemAtIndex(typeBlock)
        if (!d) { console.log("zett-probe: typing: no block " + typeBlock); Qt.quit(); return }
        typeAt = Date.now()
        d.insertForTest("abcdefghij"[typeTimes.length % 10])
    }
    onFrameSwapped: {
        if (!firstFrame) { firstFrame = true; if (app.shotEngine === "timing") { console.log("zett-probe: frame " + Date.now()); Qt.quit() } }
        if (typeAt > 0) { typeTimes = typeTimes.concat([Date.now() - typeAt]); typeAt = 0; Qt.callLater(typeNext) }
    }
    // for the parts in their own files (Today, FileView, Details): `win: win` there would bind to itself
    readonly property var theme: t
    readonly property var engine: app
    // the window's settings (Settings.qml): ~/.config/zett/gui.json, applied as they change
    property var prefs: { try { return JSON.parse(app.settings()) } catch (e) { return {} } }
    function setPref(k, v) {
        const p = Object.assign({}, prefs); p[k] = v; prefs = p
        app.saveSettings(JSON.stringify(p))
        if (k === "keys") applyKeys()
    }
    function applyKeys() { if (prefs.keys) app.evalLua(`zett.opt.keys = "${prefs.keys}"`) }

    // ── colours from tinct (same file the shell and the other apps read) ──
    QtObject {
        id: t
        readonly property var c: { try { return JSON.parse(app.theme) } catch (e) { return {} } }
        function pick(k, d) { return c[k] ?? d }
        readonly property color bg: pick("surface", "#1a1c1e")
        readonly property color panel: pick("surface_container_low", "#1e2023")
        readonly property color card: pick("surface_container", "#232528")
        readonly property color raised: pick("surface_container_high", "#2a2c30")
        readonly property color text: pick("on_surface", "#e3e2e5")
        readonly property color dim: pick("on_surface_variant", "#c2c7ce")
        readonly property color faint: Qt.alpha(pick("on_surface_variant", "#c2c7ce"), 0.55)
        readonly property color accent: pick("primary", "#a8caeb")
        readonly property color onAccent: pick("on_primary", "#0b334e")
        readonly property color rule: pick("outline_variant", "#42474d")
        // states (git in the file tree): from the theme, like the rest
        readonly property color changed: pick("tertiary", "#d4bee6")
        readonly property color added: pick("primary", "#a8caeb")
        readonly property color removed: pick("error", "#ffb4ab")
        readonly property string sans: win.prefs.sans ?? "Google Sans Flex"
        readonly property real scale: win.prefs.scale ?? 1
        readonly property string mono: "MonaspiceNe Nerd Font"
        readonly property string icons: "Material Symbols Rounded"
        readonly property int rCard: 23
        readonly property int rRow: 12
    }

    readonly property var pages: { try { return JSON.parse(app.pages) } catch (e) { return [] } }
    readonly property var blocks: { try { return JSON.parse(app.blocks) } catch (e) { return [] } }
    property int focusAfter: -1          // a block to focus once the list is rebuilt
    property bool showSource: false
    property string vimMode: ""          // NORMAL or INSERT while Vim keys are in use

    // the slash menu: "/" at the start of an empty block offers the block kinds
    property int slashFor: -1
    property string slashQuery: ""
    property int slashSel: 0
    property real slashX: 0
    property real slashY: 0
    readonly property var slashItems: [
        { k: "paragraph", label: "Text", hint: "" }, { k: "h1", label: "Heading 1", hint: "#" }, { k: "h2", label: "Heading 2", hint: "##" },
        { k: "h3", label: "Heading 3", hint: "###" }, { k: "todo", label: "To-do", hint: "[ ]" }, { k: "bullet", label: "Bullet", hint: "-" },
        { k: "quote", label: "Quote", hint: ">" }, { k: "code", label: "Code", hint: "```" }
    ]
    readonly property var slashMatches: slashItems.filter(i => i.label.toLowerCase().includes(slashQuery.toLowerCase()) || i.k.startsWith(slashQuery.toLowerCase()))
    onSlashQueryChanged: slashSel = 0
    function slashApply(i) {
        const m = slashMatches[i]
        const b = slashFor
        slashFor = -1
        if (!m || b < 0) return
        focusAfter = b
        app.setText(b, "")
        app.setKind(b, m.k)
    }

    function vimEnter(i) {
        const d = list.itemAtIndex(i)
        if (d) d.vimHere()
    }

    // a block to focus: set before or after the change that makes it (`focusAfter = app.addBlock(…)`
    // assigns once the list was already rebuilt), so focus a frame later, when every change is in
    onFocusAfterChanged: if (focusAfter >= 0) {
        focusRetry.target = focusAfter
        focusRetry.tries = 0
        focusAfter = -1
        focusRetry.restart()
    }
    onBlocksChanged: if (focusRetry.running) focusRetry.tries = 0
    // the list builds a new block's view a frame or two after the model changes: try until it is there
    Timer {
        id: focusRetry
        interval: 16
        repeat: true
        property int target: -1
        property int tries: 0
        onTriggered: {
            const d = list.itemAtIndex(target)
            if (!d) {
                if (++tries > 30) stop()
                return
            }
            stop()
            d.focusEditor(0)
            if (win.typeAfter.length > 0) { d.typeForTest(win.typeAfter); win.typeAfter = "" }
        }
    }

    // ── the Lua engine: keys (SPC tree, :, user maps), which-key, commands ──
    property string typeAfter: ""    // test hook: typed into the block focused next
    property var keySeq: []          // keys typed so far in a pending sequence
    property var whichKey: []        // what can follow them
    property var notes: []           // notices shown in the corner
    property bool sidebarHidden: false
    property bool detailsShown: prefs.details ?? true
    // a second page beside the editor, read-only (SidePage.qml); -1 when none
    property int sidePage: -1
    // moving blocks: where a dragged block would land (its index), the block the cursor is in
    property int dropAt: -1
    property int curBlock: -1
    // the focus leaves the block first, so what was just typed is saved before the move
    function moveCur(d) { const c = curBlock; if (c < 0) return; rootRow.forceActiveFocus(); if (app.moveBlock(c, c + d)) win.focusAfter = c + d }
    property bool drawerShown: false
    function toggleDrawer() { drawerShown = !drawerShown; if (drawerShown) Qt.callLater(drawer.focusInput) }
    // "today" (the start screen) or "page"; headless shots open on the page unless they ask for today
    property string view: app.shot.length > 0 ? (app.shotEngine === "today" ? "today" : app.shotEngine === "settings" ? "settings" : "page") : (prefs.start === "page" ? "page" : "today")
    // ── hooks from init.lua: zett.gui = { status = {...}, sidebar = {...}, palette = {...} } ──
    //   status:  strings, or functions returning a string or {text, icon} (read every 5 s)
    //   sidebar: { title = "Links", items = { {label=, icon=, cmd=} } }   (cmd: a command or `lua …`)
    //   palette: { {name=, desc=, keys=, cmd=} }
    property var hooks: ({ status: [], sidebar: [], palette: [] })
    readonly property string hooksLua: `local g = zett.gui or {}
local function val(x) if type(x) == "function" then local ok, r = pcall(x); if ok then return r end return nil end return x end
local st, sb, pl = {}, {}, {}
for _, s in ipairs(g.status or {}) do local v = val(s); if v ~= nil then st[#st + 1] = type(v) == "table" and { text = tostring(v.text or ""), icon = v.icon } or { text = tostring(v) } end end
for _, sec in ipairs(g.sidebar or {}) do
  local items = {}
  for _, it in ipairs(val(sec.items) or {}) do items[#items + 1] = { label = tostring(it.label or it[1] or ""), icon = it.icon, cmd = it.cmd or it[2] } end
  sb[#sb + 1] = { title = tostring(sec.title or ""), items = items }
end
for _, p in ipairs(g.palette or {}) do pl[#pl + 1] = { name = tostring(p.name or p[1] or ""), desc = p.desc, keys = p.keys, cmd = p.cmd or p[2] } end
return vim.json.encode({ status = st, sidebar = sb, palette = pl })`
    function readHooks() {
        let h = {}
        try { h = JSON.parse(JSON.parse(app.evalLua(hooksLua)).result || "{}") } catch (e) { h = {} }
        const arr = x => Array.isArray(x) ? x : []    // an empty Lua table comes back as {}
        hooks = { status: arr(h.status), sidebar: arr(h.sidebar).map(s => ({ title: s.title, items: arr(s.items) })), palette: arr(h.palette) }
    }
    function runHook(cmd) { if (cmd) win.applyScript(JSON.parse(app.runCommand(cmd))) }
    Timer { interval: 5000; running: win.active && win.hooks.status.length > 0; repeat: true; onTriggered: win.readHooks() }

    function openPage(i) { win.view = "page"; app.open(i) }
    function openQuick(text) { quickField.text = text; quick.open(); quickField.forceActiveFocus() }
    function jumpToBlock(i) { list.positionViewAtIndex(i, ListView.Beginning); win.focusAfter = i }
    // the sidebar's Files and Tags, read again when pages change and every few seconds
    property var files: []
    property var tagList: []
    property var closedDirs: []
    property bool filesOpen: true
    property bool tagsOpen: true
    function refreshSide() {
        try { files = JSON.parse(app.files()) } catch (e) { files = [] }
        try { tagList = JSON.parse(app.tags()) } catch (e) { tagList = [] }
    }
    onPagesChanged: sideLater.restart()
    Timer { id: sideLater; interval: 300; onTriggered: win.refreshSide() }
    Timer { interval: 5000; running: win.active; repeat: true; onTriggered: win.refreshSide() }
    // a file: a .md page opens as a page, a folder folds, anything else in the file view
    property string filePath: ""
    property string fileText: ""
    function openFile(x) {
        if (x.dir) { closedDirs = closedDirs.includes(x.rel) ? closedDirs.filter(d => d !== x.rel) : closedDirs.concat([x.rel]); return }
        const pi = win.pages.findIndex(p => p.path === x.path)
        if (pi >= 0) { win.openPage(pi); return }
        filePath = x.path; fileText = app.readFile(x.path); view = "file"
    }
    // drops: workspace files move into the folder, outside files are copied in
    function dropInto(d, dir) {
        let moved = 0, copied = 0
        const outside = []
        for (const u of d.urls) {
            const p = decodeURIComponent(String(u).replace(/^file:\/\//, ""))
            if (p.startsWith(app.dir + "/")) { if (app.movePath(p, dir)) moved++ }
            else outside.push(String(u))
        }
        if (outside.length) copied = app.importFiles(outside.join("\n"), dir)
        d.acceptProposedAction()
        win.refreshSide()
        win.note(moved + copied > 0 ? `${moved ? moved + " moved" : ""}${moved && copied ? ", " : ""}${copied ? copied + " copied in" : ""}` : "nothing to move there")
    }
    // a page or file dropped on the open page: a [[link]] to it at the end
    function dropLink(d) {
        let names = []
        if (d.hasFormat("application/x-zett-page")) names = [d.getDataAsString("application/x-zett-page")]
        else for (const u of d.urls) {
            let p = decodeURIComponent(String(u).replace(/^file:\/\//, ""))
            if (!p.startsWith(app.dir + "/")) { if (app.importFiles(String(u), app.dir) > 0) p = app.dir + "/" + p.slice(p.lastIndexOf("/") + 1); else continue }
            const pg = win.pages.find(x => x.path === p)
            names.push(pg ? (pg.title || "Untitled") : p.slice(app.dir.length + 1))
        }
        if (!names.length) return
        d.acceptProposedAction()
        const n = app.addBlock(win.blocks.length - 1, "paragraph")
        app.setText(n, names.map(x => `[[${x}]]`).join(" "))
        win.refreshSide()
    }
    function fileMenu(x, px, py) {
        const row = clipRow(x.rel, null, null, [
            { icon: "draft", label: "Copy path", run: () => win.copyText(x.path) },
            { icon: "link", label: "Copy as a link", run: () => win.copyText(`[[${x.rel}]]`) }
        ])
        row[3].run = () => ctx.run(row[3])
        ctx.show(px, py, row, [
            { icon: x.dir ? "unfold_more" : "open_in_new", label: x.dir ? "Fold / unfold" : "Open", run: () => win.openFile(x) },
            { icon: "open_in_browser", label: "Open with the system", enabled: !x.dir, run: () => Qt.openUrlExternally("file://" + x.path) },
            { icon: "folder_open", label: "Show in folder", run: () => Qt.openUrlExternally("file://" + (x.dir ? x.path : x.path.slice(0, x.path.lastIndexOf("/")))) },
            { icon: "refresh", label: "Refresh", run: () => win.refreshSide() }
        ])
    }
    // the open pages (indices into pages), the focus stack's tabs
    property var tabs: []
    function addTab(i) { if (i >= 0 && !tabs.includes(i)) tabs = tabs.concat([i]) }
    function closeTab(i) {
        const at = tabs.indexOf(i)
        tabs = tabs.filter(x => x !== i)
        if (i === app.current && tabs.length) win.openPage(tabs[Math.min(at, tabs.length - 1)])
    }
    Connections { target: app; function onCurrentChanged() { win.addTab(app.current); win.view = "page" } }
    Component.onCompleted: { addTab(app.current); refreshSide(); applyKeys(); readHooks() }
    Shortcut { sequence: "Ctrl+W"; onActivated: win.closeTab(app.current) }
    Shortcut { sequence: "Ctrl+Tab"; onActivated: if (win.tabs.length > 1) win.openPage(win.tabs[(win.tabs.indexOf(app.current) + 1) % win.tabs.length]) }
    Shortcut { sequence: "Ctrl+\\"; onActivated: win.detailsShown = !win.detailsShown }
    Shortcut { sequence: "F2"; onActivated: win.renameReq++ }
    Shortcut { sequence: "Ctrl+Shift+P"; onActivated: win.openPalette() }
    // undo / redo across blocks (the engine); inside a block being typed in, its own undo comes first
    Shortcut { sequence: "Alt+Up"; onActivated: win.selecting ? win.selMove(-1) : win.moveCur(-1) }
    Shortcut { sequence: "Alt+Down"; onActivated: win.selecting ? win.selMove(1) : win.moveCur(1) }
    Shortcut { sequences: ["Ctrl+Z"]; onActivated: if (!app.undo()) win.note("nothing to undo") }
    Shortcut { sequences: ["Ctrl+Shift+Z", "Ctrl+Y"]; onActivated: if (!app.redo()) win.note("nothing to redo") }
    Shortcut { sequence: "Ctrl+,"; onActivated: win.view = "settings" }
    Shortcut { sequence: "Ctrl+`"; onActivated: win.toggleDrawer() }
    // the palette: the window's own actions, then every engine command with its key
    // the page's file into the system share sheet (apps, phone, Telegram…)
    function sharePage() {
        const p = win.pages[app.current] ?? {}
        if (!p.path || !app.shareItems(p.path)) win.note("the share sheet is not installed (~/.scripts/apps/share/share)")
    }
    function openPalette() {
        const own = [
            { icon: "wb_sunny", name: "Today", desc: "the start screen", keys: "Ctrl ⇧ D", run: () => win.view = "today" },
            { icon: "search", name: "Go to page", desc: "quick open", keys: "Ctrl P", run: () => { quickField.text = ""; quick.open(); quickField.forceActiveFocus() } },
            { icon: "add", name: "New page", desc: "an empty page", keys: "", run: () => { app.newPage("Untitled"); win.focusAfter = 0 } },
            { icon: "edit", name: "Rename page", desc: "the open page", keys: "F2", run: () => win.renameReq++ },
            { icon: "right_panel_open", name: "Toggle details", desc: "outline, links, info", keys: "Ctrl \\", run: () => win.detailsShown = !win.detailsShown },
            { icon: "left_panel_close", name: "Toggle sidebar", desc: "", keys: "", run: () => win.sidebarHidden = !win.sidebarHidden },
            { icon: "close", name: "Close tab", desc: "", keys: "Ctrl W", run: () => win.closeTab(app.current) },
            { icon: "markdown", name: "Show Markdown", desc: "the file as on disk", keys: "", run: () => win.showSource = !win.showSource },
            { icon: "ios_share", name: "Share page", desc: "the system share sheet", keys: "", run: () => win.sharePage() },
            { icon: "refresh", name: "Reload config", desc: "init.lua again", keys: "", run: () => win.applyScript(JSON.parse(app.reloadConfig())) },
            { icon: "terminal", name: "Toggle shell drawer", desc: "a command in the workspace", keys: "Ctrl `", run: () => win.toggleDrawer() },
            { icon: "undo", name: "Undo", desc: "the last change, across blocks", keys: "Ctrl Z", run: () => { if (!app.undo()) win.note("nothing to undo") } },
            { icon: "redo", name: "Redo", desc: "", keys: "Ctrl ⇧ Z", run: () => { if (!app.redo()) win.note("nothing to redo") } },
            { icon: "settings", name: "Settings", desc: "keys, text, window, config", keys: "Ctrl ,", run: () => win.view = "settings" }
        ].concat(win.hooks.palette.map(h => ({ icon: "extension", name: h.name, desc: (h.desc ?? "") + "  ·  init.lua", keys: h.keys ?? "", run: () => win.runHook(h.cmd) })))
        let binds = []
        try { binds = JSON.parse(app.bindingList()).bindings || [] } catch (e) {}
        const keyOf = {}
        for (const b of binds) if (b.desc && !keyOf[b.desc]) keyOf[b.desc] = b.keys
        let cmds = []
        try { cmds = JSON.parse(app.commandList()).commands || [] } catch (e) {}
        palette.entries = own.concat(cmds.map(c => ({ icon: c.source === "zett" ? "bolt" : "extension", name: c.name, desc: c.desc + (c.source && c.source !== "zett" ? `  ·  ${c.source}` : ""), keys: keyOf[c.desc] ?? keyOf[c.name] ?? "", run: () => win.applyScript(JSON.parse(app.runCommand(c.name))) })))
        palette.show()
    }
    Shortcut { sequence: "Ctrl+Shift+D"; onActivated: win.view = "today" }

    // ── context menus: one component, each place passes its own items ──
    TextEdit { id: clip; visible: false }
    function copyText(s) { clip.text = s; clip.selectAll(); clip.copy() }
    function pasteText() { clip.text = ""; clip.paste(); return clip.text }
    function kindName(b) { return b.kind === "heading" ? "h" + (b.level ?? 1) : b.kind }
    function blockMd(b) {
        const pre = { h1: "# ", h2: "## ", h3: "### ", todo: b.checked ? "- [x] " : "- [ ] ", bullet: "- ", quote: "> " }[kindName(b)] ?? ""
        return b.kind === "code" ? "```\n" + b.text + "\n```" : pre + b.text
    }
    function pageMd() { return `# ${app.title}\n\n` + win.blocks.map(blockMd).join("\n\n") }
    // the icon row; copy/cut act on the selection when there is one, else on the whole thing
    function clipRow(copyS, cut, paste, share) {
        return [
            { icon: "content_copy", label: "Copy", run: () => win.copyText(copyS) },
            { icon: "content_cut", label: "Cut", enabled: !!cut, run: cut },
            { icon: "content_paste", label: "Paste", enabled: !!paste && clip.canPaste, run: paste },
            { icon: "ios_share", label: "Share", sub: share, run: null }
        ]
    }
    function shareItems(text, link) {
        return [
            { icon: "markdown", label: "Copy as Markdown", run: () => win.copyText(text) },
            { icon: "link", label: "Copy link", key: "[[ ]]", run: () => win.copyText(link) },
            { icon: "description", label: "Copy whole page", run: () => win.copyText(win.pageMd()) }
,            { icon: "ios_share", label: "Share sheet…", key: "apps, phone", run: () => { if (!app.shareItems(text)) win.note("the share sheet is not installed") } }
        ]
    }
    property int renameReq: 0
    // wraps the selection in a Markdown mark (bold, italic, code, link)
    function wrapSel(ed, a, z) {
        const s = ed.selectionStart, e = ed.selectionEnd, t = ed.selectedText
        ed.remove(s, e); ed.insert(s, a + t + z)
    }
    // ── several blocks at once: selected with Shift/Ctrl+click on the grip or Ctrl+A ──
    property var selBlocks: []          // indices in reading order, sorted
    property int selAnchor: -1
    Connections { target: win; function onBlocksChanged() { if (win.selBlocks.some(i => i >= win.blocks.length)) win.selBlocks = win.selBlocks.filter(i => i < win.blocks.length) } }
    function selectBlock(i, how) {
        rootRow.forceActiveFocus()      // out of the text: keys act on the selection (what was typed is saved)
        if (how === "range" && selAnchor >= 0) {
            const a = Math.min(selAnchor, i), b = Math.max(selAnchor, i)
            selBlocks = Array.from({ length: b - a + 1 }, (_, k) => a + k)
        } else if (how === "toggle") {
            selBlocks = selBlocks.includes(i) ? selBlocks.filter(x => x !== i) : selBlocks.concat([i]).sort((x, y) => x - y)
            selAnchor = i
        } else { selBlocks = [i]; selAnchor = i }
    }
    function clearSel() { selBlocks = []; selAnchor = -1 }
    readonly property bool selecting: selBlocks.length > 0
    // the actions; indices shift as blocks go, so deletes run from the last one up
    function selDelete() { for (const i of selBlocks.slice().reverse()) app.remove(i); clearSel() }
    function selKind(k) { for (const i of selBlocks) app.setKind(i, k) }
    function selCopy() { win.copyText(selBlocks.map(i => blockMd(win.blocks[i])).join("\n")); win.note(`${selBlocks.length} blocks copied as Markdown`) }
    function selMove(d) {
        const s = selBlocks
        if (d < 0 && s[0] === 0 || d > 0 && s[s.length - 1] >= blocks.length - 1) return
        const order = d < 0 ? s : s.slice().reverse()
        for (const i of order) app.moveBlock(i, i + d)
        selBlocks = s.map(i => i + d)
    }
    function selNest(out) { for (const i of (out ? selBlocks.slice().reverse() : selBlocks)) out ? app.outdent(i) : app.indent(i) }
    function selMenu(x, y) {
        const n = selBlocks.length
        ctx.show(x, y, clipRow(selBlocks.map(i => blockMd(win.blocks[i])).join("\n"), () => { win.selCopy(); win.selDelete() }, null, null).slice(0, 2), [
            { icon: "transform", label: `Turn ${n} into`, sub: win.slashItems.map(s => ({ label: s.label, key: s.hint, run: () => win.selKind(s.k) })) },
            { icon: "format_indent_increase", label: "Nest", key: "Tab", run: () => win.selNest(false) },
            { icon: "format_indent_decrease", label: "Un-nest", key: "⇧ Tab", run: () => win.selNest(true) },
            { icon: "arrow_upward", label: "Move up", key: "Alt ↑", run: () => win.selMove(-1) },
            { icon: "arrow_downward", label: "Move down", key: "Alt ↓", run: () => win.selMove(1) },
            { icon: "deselect", label: "Clear selection", key: "Esc", run: () => win.clearSel() },
            { sep: true },
            { icon: "delete", label: `Delete ${n} blocks`, key: "Del", danger: true, run: () => win.selDelete() }
        ])
    }
    // the keys while blocks are selected (no text has the focus then)
    Shortcut { enabled: win.selecting; sequences: ["Delete", "Backspace"]; onActivated: win.selDelete() }
    Shortcut { enabled: win.selecting; sequence: "Escape"; onActivated: win.clearSel() }
    Shortcut { enabled: win.selecting; sequence: "Ctrl+C"; onActivated: win.selCopy() }
    Shortcut { enabled: win.selecting; sequence: "Ctrl+X"; onActivated: { win.selCopy(); win.selDelete() } }
    Shortcut { enabled: win.selecting; sequence: "Tab"; onActivated: win.selNest(false) }
    Shortcut { enabled: win.selecting; sequence: "Shift+Tab"; onActivated: win.selNest(true) }
    // Ctrl+A while no text has the focus: every block
    Shortcut { enabled: !(win.activeFocusItem instanceof TextEdit) && win.view === "page"; sequence: "Ctrl+A"; onActivated: { win.selBlocks = win.blocks.map((_, i) => i); win.selAnchor = 0 } }

    function blockMenu(i, x, y, ed, link) {
        const b = win.blocks[i]
        if (!b) return
        if (win.selBlocks.length > 1 && win.selBlocks.includes(i)) { win.selMenu(x, y); return }
        // on a [[link]]: what to do with that page
        if (link) {
            const title = decodeURIComponent(link)
            ctx.show(x, y, clipRow(`[[${title}]]`, null, null, null).slice(0, 1), [
                { icon: "open_in_new", label: "Open " + title, run: () => app.openTitle(title) },
                { icon: "tab", label: "Open in a tab", run: () => { app.openTitle(title); win.addTab(app.current) } },
            { icon: "splitscreen_right", label: "Open to the side", enabled: win.pages.some(p => p.title === title), run: () => win.sidePage = win.pages.findIndex(p => p.title === title) },
                { icon: "link_off", label: "Remove link, keep text", run: () => app.setText(i, b.text.split(`[[${title}]]`).join(title)) }
            ])
            return
        }
        const sel = ed && ed.selectedText.length ? ed.selectedText : ""
        const row = clipRow(sel || b.text,
            () => { if (sel) { ed.cut() } else { win.copyText(b.text); app.remove(i) } },
            () => { const s = win.pasteText(); if (ed) ed.insert(ed.cursorPosition, s); else app.setText(i, b.text + s) },
            shareItems(blockMd(b), `[[${app.title}]]`))
        row[3].run = () => ctx.run(row[3])
        // with a selection: format it first
        const fmt = sel ? [
            { icon: "format_bold", label: "Bold", key: "**", run: () => win.wrapSel(ed, "**", "**") },
            { icon: "format_italic", label: "Italic", key: "*", run: () => win.wrapSel(ed, "*", "*") },
            { icon: "code", label: "Code", key: "`", run: () => win.wrapSel(ed, "`", "`") },
            { icon: "link", label: "Link to a page", key: "[[ ]]", run: () => win.wrapSel(ed, "[[", "]]") },
            { icon: "search", label: "Search for it", run: () => { quickField.text = sel; quick.open(); quickField.forceActiveFocus() } },
            { sep: true }
        ] : []
        ctx.show(x, y, row, fmt.concat([
            { icon: "transform", label: "Turn into", sub: win.slashItems.map(s => ({ icon: { paragraph: "notes", h1: "title", h2: "title", h3: "title", todo: "check_box", bullet: "format_list_bulleted", quote: "format_quote", code: "code" }[s.k], label: s.label, key: s.hint, enabled: s.k !== kindName(b), run: () => app.setKind(i, s.k) })) },
            { icon: "play_arrow", label: "Run in the drawer", enabled: b.kind === "code" && /^(sh|bash|zsh|shell|console)?$/.test(b.lang ?? ""), run: () => { win.drawerShown = true; drawer.run(b.text) } },
            { icon: "add", label: "Add block below", key: "Enter", run: () => win.focusAfter = app.addBlock(i, "paragraph") },
            { icon: "arrow_upward", label: "Move up", key: "Alt ↑", enabled: i > 0, run: () => { if (app.moveBlock(i, i - 1)) win.focusAfter = i - 1 } },
            { icon: "arrow_downward", label: "Move down", key: "Alt ↓", enabled: i < win.blocks.length - 1, run: () => { if (app.moveBlock(i, i + 1)) win.focusAfter = i + 1 } },
            { icon: "content_copy", label: "Duplicate", run: () => { const n = app.addBlock(i, kindName(b)); app.setText(n, b.text) } },
            { icon: "check_box", label: b.kind === "todo" ? (b.checked ? "Mark not done" : "Mark done") : "Make a to-do", run: () => b.kind === "todo" ? app.toggle(i) : app.setKind(i, "todo") },
            { sep: true },
            { icon: "delete", label: "Delete block", danger: true, run: () => app.remove(i) }
        ]))
    }
    function pageMenu(i, x, y) {
        const p = win.pages[i] ?? {}
        const title = p.title || "Untitled"
        const row = clipRow(title, null, null, [
            { icon: "link", label: "Copy link", key: "[[ ]]", run: () => win.copyText(`[[${title}]]`) },
            { icon: "draft", label: "Copy file path", enabled: !!p.path, run: () => win.copyText(p.path) },
            { icon: "markdown", label: "Copy as Markdown", enabled: i === app.current, run: () => win.copyText(win.pageMd()) }
        ])
        row[3].run = () => ctx.run(row[3])
        ctx.show(x, y, row, [
            { icon: "open_in_new", label: "Open", run: () => win.openPage(i) },
            { icon: "tab", label: "Keep in a tab", enabled: !win.tabs.includes(i), run: () => win.addTab(i) },
            { icon: "splitscreen_right", label: "Open to the side", enabled: i !== app.current, run: () => win.sidePage = i },
            { icon: "edit", label: "Rename", key: "F2", run: () => { win.openPage(i); win.renameReq++ } },
            { icon: "folder_open", label: "Show in folder", run: () => Qt.openUrlExternally("file://" + app.dir) },
            { icon: "add", label: "New page", run: () => { app.newPage("Untitled"); win.focusAfter = 0 } },
            { sep: true },
            { icon: "close", label: "Close tab", enabled: win.tabs.includes(i), run: () => win.closeTab(i) },
            { icon: "delete", label: "Move to trash", danger: true, run: () => {
                if (app.trashPage(i)) { win.tabs = win.tabs.filter(x => x !== i).map(x => x > i ? x - 1 : x); win.note(`"${title}" moved to the trash`) }
                else win.note(`could not move "${title}" to the trash`)
            } }
        ])
    }
    function tabMenu(i, x, y) {
        ctx.show(x, y, [], [
            { icon: "close", label: "Close", key: "Ctrl W", run: () => win.closeTab(i) },
            { icon: "splitscreen_right", label: "Open to the side", enabled: i !== app.current, run: () => win.sidePage = i },
            { icon: "tab_close_right", label: "Close others", enabled: win.tabs.length > 1, run: () => { win.tabs = [i]; win.openPage(i) } },
            { icon: "link", label: "Copy link", run: () => win.copyText(`[[${(win.pages[i] ?? {}).title || "Untitled"}]]`) }
        ])
    }
    function keyToken(e) {
        const named = { [Qt.Key_Escape]: "<Esc>", [Qt.Key_Return]: "<CR>", [Qt.Key_Enter]: "<CR>", [Qt.Key_Backspace]: "<BS>", [Qt.Key_Tab]: "<Tab>", [Qt.Key_Left]: "<Left>", [Qt.Key_Right]: "<Right>", [Qt.Key_Up]: "<Up>", [Qt.Key_Down]: "<Down>", [Qt.Key_Space]: "<Space>" }
        if (named[e.key]) return named[e.key]
        if (e.modifiers & Qt.ControlModifier && e.key >= Qt.Key_A && e.key <= Qt.Key_Z) return `<C-${String.fromCharCode(e.key + 32)}>`
        if (e.modifiers & Qt.AltModifier && e.text) return `<A-${e.text}>`
        return e.text === "<" ? "<lt>" : (e.text || "")
    }
    // A key in normal mode goes to the engine first. Returns true when the engine took it;
    // otherwise `replay` gets every key of the sequence (they were Vim's after all).
    function engineKey(e, replay) {
        return engineTok(keyToken(e), replay)
    }
    function engineTok(tok, replay) {
        if (!tok) return false
        if (tok === "<Esc>" && keySeq.length > 0) { keySeq = []; whichKey = []; keyWait.stop(); return true }
        const seq = keySeq.concat([tok])
        const r = JSON.parse(app.feedKeys(seq.join("\u001f")))
        if (r.state === "pending") { keySeq = seq; whichKey = r.menu; keyWait.restart(); return true }
        keySeq = []; whichKey = []; keyWait.stop()
        applyScript(r)
        if (r.state === "ran") return true
        if (replay) for (const k of seq) replay(k)
        return replay !== null && replay !== undefined
    }
    // what scripts asked for: UI to open, notices to show
    function applyScript(r) {
        for (const n of (r.notes || [])) note(n)
        for (const u of (r.ui || [])) {
            switch (u.name) {
            case "quickOpen": quickField.text = ""; quick.open(); quickField.forceActiveFocus(); break
            case "search": quickField.text = ""; quick.open(); quickField.forceActiveFocus(); break
            case "commandLine": openCommandLine(u.arg); break
            case "help": openHelp(u.arg); break
            case "reload": applyScript(JSON.parse(app.reloadConfig())); break
            case "openConfig": Qt.openUrlExternally("file://" + app.configPath()); break
            case "toggleSidebar": sidebarHidden = !sidebarHidden; break
            case "focusBlock": focusAfter = parseInt(u.arg) - 1; break
            default: note(`unknown UI request: ${u.name}`)
            }
        }
    }
    function note(text) {
        if (!text) return
        notes = notes.concat([{ text: text, at: Date.now() }]).slice(-4)
        noteClear.restart()
    }
    // keys that are exactly a mapping run after a moment; a prefix alone keeps its menu open
    Timer { id: keyWait; interval: 900; onTriggered: { const r = JSON.parse(app.keyTimeout(win.keySeq.join("\u001f"))); if (r.state === "ran") { win.keySeq = []; win.whichKey = []; win.applyScript(r) } } }
    Timer { id: noteClear; interval: 4000; onTriggered: win.notes = [] }

    // [[Page]] → a link (the text is escaped first: it is shown as StyledText)
    function linkify(s) {
        const esc = s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/\n/g, "<br>")
        return esc.replace(/\[\[([^\]]+)\]\]/g, (m, title) => `<a href="${encodeURIComponent(title)}"><u>${title}</u></a>`)
    }

    function focusBlock(i, end) {
        const d = list.itemAtIndex(i)
        if (d) d.focusEditor(end ? -1 : 0)
    }

    // pages edited elsewhere (Neovim, zett add, git) show up within two seconds
    Timer { interval: 2000; running: true; repeat: true; onTriggered: app.checkDisk() }

    Timer {
        running: app.shot.length > 0
        interval: 1500
        onTriggered: {
            if (app.shotType.length > 0) {
                win.typeAfter = app.shotType
                win.focusAfter = app.addBlock(-1, "paragraph")
            }
            if (app.shotEngine === "menu") { win.blockMenu(1, 340, 180, null); console.log(`zett-probe: menu opened=${ctx.opened} visible=${ctx.visible} items=${ctx.shown.length} h=${ctx.implicitHeight} root=${win.contentItem.parent}`); grabLater.start(); return }
            if (app.shotEngine === "dnd") { const a = app.movePath(app.dir + "/src/main.rs", app.dir); const b = app.movePath(app.dir + "/src", app.dir + "/src"); const c = app.importFiles("file://" + app.dir + "/../ui-demo-v4.png", app.dir + "/src"); console.log(`zett-probe: move=${a} intoSelf=${b} imported=${c}`); grabLater.start(); return }
            if (app.shotEngine === "side") { win.sidePage = win.pages.findIndex(p => p.title === "fan-curve"); grabLater.start(); return }
            if (app.shotEngine === "drawer") { win.drawerShown = true; drawer.run("ls; echo; git status --short | head -5; printf 'done in %s\\n' \"$(pwd)\""); grabLater.start(); return }
            if (app.shotEngine === "hooks") { console.log("zett-probe: hooks " + JSON.stringify(win.hooks)); grabLater.start(); return }
            if (app.shotEngine === "codeedit") { win.focusAfter = 1; grabLater.start(); return }
            if (app.shotEngine.startsWith("typing:")) { win.typeBlock = +app.shotEngine.split(":")[1]; win.typeNext(); return }
            if (app.shotEngine === "grip") { win.dropAt = 2; grabLater.start(); return }
            if (app.shotEngine === "multisel") { win.selectBlock(1, "one"); win.selectBlock(2, "range"); grabLater.start(); return }
            if (app.shotEngine === "multi") {
                const d = () => win.blocks.map(b => b.text + ":" + (b.depth ?? 0)).join(" ")
                const a = d(); win.selectBlock(1, "one"); win.selectBlock(2, "range"); const s1 = JSON.stringify(win.selBlocks)
                win.selMove(1); const b = d() + " sel " + JSON.stringify(win.selBlocks)
                win.selNest(false); const c = d()
                win.selDelete(); const e = d()
                console.log(`zett-probe: multi ${a} | select ${s1} | down: ${b} | nest: ${c} | delete: ${e}`)
                Qt.quit(); return
            }
            if (app.shotEngine === "nest") {
                const d = () => win.blocks.map(b => b.text + ":" + (b.depth ?? 0)).join(" ")
                const a = d(); const i1 = app.indent(1); const b = d(); const i2 = app.indent(2); const c = d(); const o = app.outdent(2); const e = d()
                console.log(`zett-probe: nest ${a} | indent1 ${i1}: ${b} | indent2 ${i2}: ${c} | outdent2 ${o}: ${e}`)
                console.log("zett-probe: md " + JSON.stringify(app.markdown()))
                grabLater.start(); return
            }
            if (app.shotEngine === "move") {
                const before = win.blocks.map(b => b.text.slice(0, 12))
                const ok = app.moveBlock(0, 1); const mid = win.blocks.map(b => b.text.slice(0, 12))
                const back = app.undo(); const after = win.blocks.map(b => b.text.slice(0, 12))
                console.log(`zett-probe: move ${JSON.stringify(before)} -> (${ok}) ${JSON.stringify(mid)} -> undo(${back}) ${JSON.stringify(after)}`)
                Qt.quit(); return
            }
            if (app.shotEngine === "undo") {
                const n0 = win.blocks.length
                const b = app.addBlock(n0 - 1, "paragraph"); app.setText(b, "to be undone")
                const n1 = win.blocks.length
                const u = app.undo(); const n2 = win.blocks.length
                const r = app.redo(); const n3 = win.blocks.length
                console.log(`zett-probe: undo blocks ${n0} -> add ${n1} -> undo(${u}) ${n2} -> redo(${r}) ${n3}; last="${(win.blocks[n3 - 1] ?? {}).text}"`)
                Qt.quit(); return
            }
            if (app.shotEngine === "palette") { win.openPalette(); grabLater.start(); return }
            if (app.shotEngine === "file") { win.refreshSide(); const x = win.files.find(x => !x.dir && !x.name.endsWith(".md")); if (x) win.openFile(x); grabLater.start(); return }
            if (app.shotEngine === "pagemenu") { win.pageMenu(0, 120, 170); grabLater.start(); return }
            if (app.shotEngine === "trash") { const n = win.pages.length; const ok = app.trashPage(n - 1); console.log(`zett-probe: trash ok=${ok} pages ${n} -> ${win.pages.length}`); grabLater.start(); return }
            if (app.shotEngine === "selmenu") { const d = list.itemAtIndex(1); win.blockMenu(1, 340, 150, null, ""); grabLater.start(); return }
            if (app.shotEngine.length > 0) {
                for (const k of app.shotEngine.split(" ")) win.engineTok(k, null)
                keyWait.stop()
                console.log(`zett-probe: seq=[${win.keySeq}] whichKey=${win.whichKey.length} cmdLine=${cmdLine.opened} quick=${quick.opened} help=${helpPop.opened} commands=${cmdLine.commands.length} notes=${JSON.stringify(win.notes.map(n => n.text))}`)
            }
            if (app.shotKeys.length > 0) {
                win.vimEnter(1)
                const d = list.itemAtIndex(1)
                for (const k of app.shotKeys.split(" ")) d.vimKeyForTest(k)
            }
            if (app.shotType.length > 0) probeLater.start()
            grabLater.start()
        }
    }
    Timer { id: probeLater; interval: 500; onTriggered: console.log(`zett-probe: slashFor=${win.slashFor} query=${win.slashQuery} matches=${win.slashMatches.map(m => m.label)} focus=${win.activeFocusItem}`) }

    Rectangle {
    id: rootRow
    anchors.fill: parent
    color: t.bg
    // with no block being edited, keys go to the engine (SPC, :, the user's maps)
    focus: true
    Keys.onPressed: (e) => { if (win.engineKey(e, null)) e.accepted = true }
    Timer { id: grabLater; interval: 700; onTriggered: rootRow.grabToImage(r => { r.saveToFile(app.shot); Qt.quit() })  }
    // the page sits in one card on the background, like the shell's panels
    Rectangle { x: pageCol.x; y: pageCol.y; width: pageCol.width; height: pageCol.height; radius: t.rCard; color: t.panel; border.color: pageDrop.containsDrag ? t.accent : Qt.alpha(t.rule, 0.35)
        // a page or file dropped on the open page becomes a [[link]]
        DropArea { id: pageDrop; anchors.fill: parent; enabled: win.view === "page"; onDropped: (d) => win.dropLink(d) }
        border.width: pageDrop.containsDrag ? 2 : 1
    }
    ContextMenu { id: ctx; theme: t; z: 1000 }
    Palette { id: palette; theme: t }

    RowLayout {
        anchors.fill: parent
        spacing: 0

        // ───────── sidebar ─────────
        Sidebar { win: ApplicationWindow.window; t: ApplicationWindow.window.theme; app: ApplicationWindow.window.engine }

        // ───────── the page ─────────
        ColumnLayout {
            id: pageCol
            Layout.topMargin: 8; Layout.rightMargin: 8; Layout.bottomMargin: 8
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 0

            // toolbar: title, and the tools below the window-button corner
            Toolbar { win: ApplicationWindow.window; t: ApplicationWindow.window.theme; app: ApplicationWindow.window.engine }

            // ───────── a file from the tree, read-only and highlighted ─────────
            FileView { win: ApplicationWindow.window; t: ApplicationWindow.window.theme; app: ApplicationWindow.window.engine }

            // ───────── Today: the start screen ─────────
            Today { win: ApplicationWindow.window; t: ApplicationWindow.window.theme; app: ApplicationWindow.window.engine }
            Settings { visible: win.view === "settings"; Layout.fillWidth: true; Layout.fillHeight: true; win: ApplicationWindow.window; t: ApplicationWindow.window.theme; app: ApplicationWindow.window.engine }

            // empty workspace
            ColumnLayout {
                visible: win.pages.length === 0 && win.view === "page"
                Layout.alignment: Qt.AlignCenter
                Layout.fillHeight: true
                spacing: 10
                Text { Layout.alignment: Qt.AlignHCenter; text: "No pages yet"; color: t.text; font.family: t.sans; font.pixelSize: 22; font.weight: Font.DemiBold }
                Text { Layout.alignment: Qt.AlignHCenter; text: "Make one with  + New page, or drop .md files into " + app.dir; color: t.dim; font.family: t.sans; font.pixelSize: 14 }
            }

            // Markdown source (read-only view of what is on disk)
            ScrollView {
                visible: win.showSource && win.pages.length > 0 && win.view === "page"
                Layout.fillWidth: true
                Layout.fillHeight: true
                TextArea {
                    readOnly: true
                    text: win.showSource ? (app.blocks, app.title, app.markdown()) : ""
                    color: t.text
                    font.family: t.mono; font.pixelSize: 14
                    leftPadding: 48; topPadding: 24
                    background: null
                    wrapMode: TextEdit.Wrap
                }
            }

            ListView {
                id: list
                visible: !win.showSource && win.pages.length > 0 && win.view === "page"
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                spacing: 2
                topMargin: 28
                bottomMargin: 120
                model: win.blocks
                boundsBehavior: Flickable.StopAtBounds
                readonly property real column: Math.min(win.prefs.width ?? 760, width - 96)

                header: Item {
                    width: list.width
                    height: titleEdit.implicitHeight + 18
                    TextInput {
                        id: titleEdit
                        Connections { target: win; function onRenameReqChanged() { titleEdit.forceActiveFocus(); titleEdit.selectAll() } }
                        x: (list.width - list.column) / 2
                        width: list.column
                        text: app.title
                        color: t.text
                        font.family: t.sans; font.pixelSize: 34; font.weight: Font.Bold
                        selectByMouse: true
                        onEditingFinished: if (text !== app.title) app.renamePage(text)
                        Keys.onReturnPressed: { app.renamePage(text); win.focusBlock(0, false) }
                        Keys.onDownPressed: win.focusBlock(0, false)
                    }
                }

                delegate: Item {
                    id: blockItem
                    required property var modelData
                    required property int index
                    width: list.width
                    height: body.implicitHeight + (modelData.kind === "heading" ? 14 : 6)

                    TapHandler { acceptedButtons: Qt.RightButton; onTapped: (ev) => win.blockMenu(blockItem.index, ev.scenePosition.x, ev.scenePosition.y, edit, shown.visible ? shown.hoveredLink : "") }
                    function vimKeyForTest(k) { edit.applyVim(app.vimKey(k)) }
                    function typeForTest(s) { edit.forceActiveFocus(); edit.text = s }
                    function insertForTest(s) { edit.forceActiveFocus(); edit.insert(edit.cursorPosition, s) }

                    function vimHere() {
                        edit.forceActiveFocus()
                        edit.applyVim(app.vimBegin(index, edit.text, 1))
                    }

                    function focusEditor(pos) {
                        edit.forceActiveFocus()
                        edit.cursorPosition = pos < 0 ? edit.length : pos
                    }

                    readonly property string kind: modelData.kind
                    readonly property int level: modelData.level ?? 1
                    readonly property bool checked: modelData.checked ?? false

                    // a selected block (Shift/Ctrl+click on its grip, Ctrl+A): a tint behind it
                    Rectangle {
                        visible: win.selBlocks.includes(blockItem.index)
                        x: body.x - 8; y: body.y - 3; width: body.width + 16; height: body.height + 6
                        radius: t.rRow
                        color: Qt.alpha(t.accent, 0.14)
                        border.width: 1; border.color: Qt.alpha(t.accent, 0.35)
                    }
                    // the grip left of the block (on hover): drag it onto another block to move it there
                    HoverHandler { id: blockHov }
                    Rectangle {
                        id: grip
                        x: body.x - 30; y: body.y + 2
                        width: 22; height: 24; radius: 6
                        opacity: blockHov.hovered || gripDrag.active ? 1 : 0
                        Behavior on opacity { NumberAnimation { duration: 120 } }
                        color: gripHov.hovered ? Qt.alpha(t.text, 0.08) : "transparent"
                        Text { anchors.centerIn: parent; text: "drag_indicator"; color: t.faint; font.family: t.icons; font.pixelSize: 18 }
                        HoverHandler { id: gripHov; cursorShape: gripDrag.active ? Qt.ClosedHandCursor : Qt.OpenHandCursor }
                        DragHandler { id: gripDrag; target: null; onActiveChanged: if (!active && win.dropAt >= 0) { const to = win.dropAt; win.dropAt = -1; if (app.moveBlock(blockItem.index, to)) win.focusAfter = to } }
                        Drag.active: gripDrag.active
                        Drag.keys: ["zett-block"]
                        Drag.hotSpot.x: 10; Drag.hotSpot.y: 12
                        // a click: the block menu; Shift+click: select from the last selected block to here; Ctrl+click: add / remove this one
                        TapHandler { acceptedModifiers: Qt.NoModifier; onTapped: (ev) => win.blockMenu(blockItem.index, ev.scenePosition.x, ev.scenePosition.y, null, "") }
                        TapHandler { acceptedModifiers: Qt.ShiftModifier; onTapped: win.selectBlock(blockItem.index, "range") }
                        TapHandler { acceptedModifiers: Qt.ControlModifier; onTapped: win.selectBlock(blockItem.index, "toggle") }
                    }
                    // where a dragged block would land: a line above this one
                    DropArea {
                        anchors.fill: parent
                        keys: ["zett-block"]
                        onEntered: win.dropAt = blockItem.index
                        onExited: if (win.dropAt === blockItem.index) win.dropAt = -1
                    }
                    Rectangle { visible: win.dropAt === blockItem.index; x: body.x; width: body.width; height: 2; radius: 1; color: t.accent; y: 0 }

                    // nested list items step in, 26 px a level
                    readonly property int depth: modelData.depth ?? 0
                    RowLayout {
                        id: body
                        x: (list.width - list.column) / 2 + blockItem.depth * 26
                        width: list.column - blockItem.depth * 26
                        y: blockItem.kind === "heading" ? 10 : 2
                        spacing: 10

                        // the block's marker: checkbox, bullet, quote bar
                        Item {
                            visible: ["todo", "bullet", "quote"].includes(blockItem.kind)
                            Layout.preferredWidth: 20
                            Layout.fillHeight: blockItem.kind === "quote"
                            Layout.alignment: Qt.AlignTop
                            implicitHeight: 24
                            Rectangle {
                                visible: blockItem.kind === "todo"
                                anchors.centerIn: parent
                                width: 18; height: 18; radius: 5
                                color: blockItem.checked ? t.accent : "transparent"
                                border.width: blockItem.checked ? 0 : 2
                                border.color: t.dim
                                Text { anchors.centerIn: parent; visible: blockItem.checked; text: "✓"; color: t.onAccent; font.pixelSize: 13; font.weight: Font.Bold }
                                TapHandler { onTapped: app.toggle(blockItem.index) }
                                HoverHandler { cursorShape: Qt.PointingHandCursor }
                            }
                            Rectangle { visible: blockItem.kind === "bullet"; anchors.centerIn: parent; width: 6; height: 6; radius: 3; color: t.dim }
                            Rectangle { visible: blockItem.kind === "quote"; anchors { left: parent.left; top: parent.top; bottom: parent.bottom; leftMargin: 6 } width: 3; radius: 1.5; color: t.accent }
                        }

                        Rectangle {
                            Layout.fillWidth: true
                            implicitHeight: edit.implicitHeight + (blockItem.kind === "code" ? 24 : 0)
                            radius: 10
                            color: blockItem.kind === "code" ? t.card : "transparent"

                            // a code block, coloured by its language; while it is edited it lies under the (then
                            // transparent) editor, which keeps the cursor and the selection: colours as you type
                            Text {
                                id: codeShown
                                anchors.fill: edit
                                visible: blockItem.kind === "code"
                                text: visible ? app.highlight(blockItem.modelData.lang ?? "", edit.text) : ""
                                textFormat: Text.RichText
                                font: edit.font
                                color: edit.color
                                HoverHandler { cursorShape: Qt.IBeamCursor }
                                TapHandler { enabled: !edit.activeFocus; onTapped: edit.forceActiveFocus() }
                                z: edit.activeFocus ? 0 : 2   // editing: under the editor (declared after it), over the block's fill
                            }
                            // the language, top right of a code block
                            Text {
                                visible: blockItem.kind === "code" && (blockItem.modelData.lang ?? "").length > 0
                                anchors { right: parent.right; top: parent.top; margins: 8 }
                                text: blockItem.modelData.lang ?? ""
                                color: t.faint; font.family: t.mono; font.pixelSize: 11
                                z: 3
                            }
                            // shown instead of the raw text while not editing: [[links]] are clickable
                            Text {
                                id: shown
                                anchors.fill: edit
                                visible: !edit.activeFocus && edit.text.indexOf("[[") >= 0 && blockItem.kind !== "code"
                                text: win.linkify(edit.text)
                                textFormat: Text.StyledText
                                wrapMode: Text.Wrap
                                font: edit.font
                                color: edit.color
                                linkColor: t.accent
                                onLinkActivated: (l) => app.openTitle(decodeURIComponent(l))
                                HoverHandler { cursorShape: shown.hoveredLink ? Qt.PointingHandCursor : Qt.IBeamCursor }
                                TapHandler { onTapped: if (!shown.hoveredLink) edit.forceActiveFocus() }
                                z: 2
                            }

                            TextEdit {
                                id: edit
                                opacity: shown.visible || (codeShown.visible && !edit.activeFocus) ? 0 : 1
                                anchors { fill: parent; margins: blockItem.kind === "code" ? 12 : 0 }
                                text: blockItem.modelData.text
                                wrapMode: blockItem.kind === "code" ? TextEdit.NoWrap : TextEdit.Wrap   // code: lines as in the coloured copy under it
                                selectByMouse: true
                                color: blockItem.kind === "code" && activeFocus ? "transparent" : blockItem.kind === "todo" && blockItem.checked ? t.faint : blockItem.kind === "quote" ? t.dim : t.text
                                cursorDelegate: Rectangle { width: 2; color: t.accent; visible: edit.cursorVisible }
                                font.family: blockItem.kind === "code" ? t.mono : t.sans
                                font.pixelSize: Math.round((blockItem.kind === "heading" ? [0, 28, 22, 18, 16, 15, 14][blockItem.level] : blockItem.kind === "code" ? 14 : 16) * t.scale)
                                font.weight: blockItem.kind === "heading" ? Font.DemiBold : Font.Normal
                                font.italic: blockItem.kind === "quote"
                                font.strikeout: blockItem.kind === "todo" && blockItem.checked
                                readonly property bool visualSel: win.vimMode === "VISUAL" || win.vimMode === "V-LINE"
                                selectionColor: visualSel ? Qt.alpha(t.accent, 0.42) : normal ? t.accent : Qt.alpha(t.accent, 0.4)
                                selectedTextColor: blockItem.kind === "code" ? t.text : normal && !visualSel ? t.onAccent : t.text
                                // Vim normal mode: keys go to zett-edit (window.rs), the cursor is a block
                                property bool normal: false
                                property bool applying: false
                                property bool pastePending: false
                                function applyVim(json) {
                                    const s = JSON.parse(json)
                                    if (s.text === undefined) return
                                    applying = true
                                    if (s.text !== text) text = s.text
                                    applying = false
                                    // visual modes are normal-mode keys with a selection drawn
                                    normal = s.mode !== "insert"
                                    win.vimMode = ({ normal: "NORMAL", insert: "INSERT", visual: "VISUAL", "visual-line": "V-LINE" })[s.mode] || ""
                                    cursorPosition = Math.min(s.cursor, length)
                                    if (s.sel) select(Math.min(s.sel[0], length), Math.min(Math.max(s.sel[1], s.sel[0] + 1), length))
                                    else if (normal && length > 0) select(Math.min(s.cursor, length - 1), Math.min(s.cursor, length - 1) + 1)
                                    else deselect()
                                }
                                // one key for Vim in normal mode
                                function vimTok(k) {
                                    if (k === "<Esc>") { normal = false; deselect(); win.vimMode = ""; rootRow.forceActiveFocus(); return }
                                    if (k === "<Space>") k = " "
                                    if (k === "<CR>" && !normal) return
                                    const before = cursorPosition
                                    applyVim(app.vimKey(k))
                                    // j on the last line, k on the first: on to the next or previous block
                                    if ((k === "j" || k === "<Down>") && cursorPosition === before && blockItem.index < win.blocks.length - 1) win.vimEnter(blockItem.index + 1)
                                    else if ((k === "k" || k === "<Up>") && cursorPosition === before && blockItem.index > 0) win.vimEnter(blockItem.index - 1)
                                }
                                function vimKeyName(e) {
                                    const named = { [Qt.Key_Escape]: "Esc", [Qt.Key_Return]: "CR", [Qt.Key_Enter]: "CR", [Qt.Key_Backspace]: "BS", [Qt.Key_Left]: "Left", [Qt.Key_Right]: "Right", [Qt.Key_Up]: "Up", [Qt.Key_Down]: "Down", [Qt.Key_Home]: "Home", [Qt.Key_End]: "End", [Qt.Key_Delete]: "Del", [Qt.Key_Tab]: "Tab" }
                                    if (named[e.key]) return `<${named[e.key]}>`
                                    if (e.modifiers & Qt.ControlModifier && e.key >= Qt.Key_A && e.key <= Qt.Key_Z) return `<C-${String.fromCharCode(e.key + 32)}>`
                                    if (!e.text) return ""
                                    return e.text === "<" ? "<lt>" : e.text
                                }

                                // a placeholder on an empty, focused block
                                Text {
                                    visible: edit.length === 0 && edit.activeFocus
                                    text: blockItem.kind === "heading" ? "Heading" : "Type, or start with  # · - [ ] · - · > · ```"
                                    color: t.faint
                                    font: edit.font
                                }

                                // saved a moment after typing stops
                                Timer { id: saveLater; interval: 350; onTriggered: app.setText(blockItem.index, edit.text) }
                                onTextChanged: {
                                    if (!activeFocus || applying) return
                                    if (edit.pastePending) {
                                        edit.pastePending = false
                                        saveLater.restart()
                                        return
                                    }
                                    // "/word" in an empty text block: the slash menu, filtered by what follows
                                    if (blockItem.kind === "paragraph" && text.startsWith("/") && !/\s/.test(text)) {
                                        const p = edit.mapToItem(rootRow, 0, edit.height)
                                        win.slashX = p.x
                                        win.slashY = p.y + 6
                                        win.slashQuery = text.slice(1)
                                        win.slashFor = blockItem.index
                                        return
                                    } else if (win.slashFor === blockItem.index) {
                                        win.slashFor = -1
                                    }
                                    // Markdown shortcuts at the start of an empty block turn it into another kind
                                    const shortcuts = [["# ", "h1"], ["## ", "h2"], ["### ", "h3"], ["- [ ] ", "todo"], ["[] ", "todo"], ["- ", "bullet"], ["* ", "bullet"], ["> ", "quote"], ["```", "code"]]
                                    if (blockItem.kind === "paragraph") {
                                        for (const [p, k] of shortcuts) {
                                            if (text === p) {
                                                win.focusAfter = blockItem.index
                                                app.setText(blockItem.index, "")
                                                app.setKind(blockItem.index, k)
                                                return
                                            }
                                        }
                                    }
                                    saveLater.restart()
                                }
                                onActiveFocusChanged: if (!activeFocus) {
                                    if (saveLater.running) { saveLater.stop(); app.setText(blockItem.index, edit.text) }
                                    if (normal) { normal = false; deselect(); win.vimMode = "" }
                                } else win.curBlock = blockItem.index   // Alt+Up/Down move this one

                                Keys.onPressed: (e) => {
                                    // Tab / Shift+Tab: a list item nests under the one above / comes back out (saved first)
                                    if ((e.key === Qt.Key_Tab || e.key === Qt.Key_Backtab) && (blockItem.kind === "bullet" || blockItem.kind === "todo")) {
                                        const i = blockItem.index, out = e.key === Qt.Key_Backtab || (e.modifiers & Qt.ShiftModifier)
                                        if (saveLater.running) { saveLater.stop(); app.setText(i, edit.text) }
                                        if (out ? app.outdent(i) : app.indent(i)) win.focusAfter = i
                                        e.accepted = true
                                        return
                                    }
                                    if (normal) {
                                        // the engine first (SPC tree, :, the user's own maps); what it leaves is Vim's
                                        win.engineKey(e, vimTok)
                                        e.accepted = true
                                        return
                                    }
                                    // Ctrl+V: a paste is coming; don't treat its text as typed shortcuts
                                    if (e.key === Qt.Key_V && (e.modifiers & Qt.ControlModifier)) {
                                        edit.pastePending = true
                                    }
                                    if (win.slashFor === blockItem.index) {
                                        if (e.key === Qt.Key_Down) { win.slashSel = Math.min(win.slashSel + 1, win.slashMatches.length - 1); e.accepted = true; return }
                                        if (e.key === Qt.Key_Up) { win.slashSel = Math.max(win.slashSel - 1, 0); e.accepted = true; return }
                                        if (e.key === Qt.Key_Return || e.key === Qt.Key_Enter) { win.slashApply(win.slashSel); e.accepted = true; return }
                                        if (e.key === Qt.Key_Escape) { win.slashFor = -1; e.accepted = true; return }
                                    }
                                    if (e.key === Qt.Key_Escape) {
                                        saveLater.stop()
                                        app.setText(blockItem.index, edit.text)
                                        applyVim(app.vimBegin(blockItem.index, edit.text, cursorPosition))
                                        e.accepted = true
                                        return
                                    }
                                    const code = blockItem.kind === "code"
                                    if ((e.key === Qt.Key_Return || e.key === Qt.Key_Enter) && !(e.modifiers & Qt.ShiftModifier) && !code) {
                                        // Enter: a new block after this one (a list continues as a list)
                                        saveLater.stop()
                                        app.setText(blockItem.index, edit.text)
                                        const next = ["todo", "bullet"].includes(blockItem.kind) && edit.length > 0 ? blockItem.kind : "paragraph"
                                        win.focusAfter = app.addBlock(blockItem.index, next)
                                        e.accepted = true
                                    } else if (e.key === Qt.Key_Backspace && edit.length === 0) {
                                        // Backspace in an empty block: a list item becomes text, text goes away
                                        if (blockItem.kind !== "paragraph") { win.focusAfter = blockItem.index; app.setKind(blockItem.index, "paragraph") }
                                        else if (blockItem.index > 0) { const prev = blockItem.index - 1; app.remove(blockItem.index); Qt.callLater(() => win.focusBlock(prev, true)) }
                                        e.accepted = true
                                    } else if (e.key === Qt.Key_Up && edit.cursorRectangle.y < 4 && blockItem.index > 0) {
                                        win.focusBlock(blockItem.index - 1, true); e.accepted = true
                                    } else if (e.key === Qt.Key_Down && edit.cursorRectangle.y + edit.cursorRectangle.height > edit.height - 4 && blockItem.index < win.blocks.length - 1) {
                                        win.focusBlock(blockItem.index + 1, false); e.accepted = true
                                    } else if (e.key === Qt.Key_Escape) {
                                        edit.focus = false; e.accepted = true
                                    }
                                }
                            }
                        }
                    }
                }

                // a click below the last block continues the page; then the pages linking here
                footer: Item {
                    width: list.width
                    height: 160 + backCol.implicitHeight + (startCol.visible ? startCol.implicitHeight + 20 : 0)
                    // an empty page: ways to begin instead of a blank card
                    ColumnLayout {
                        id: startCol
                        visible: win.blocks.length === 0
                        x: (list.width - list.column) / 2
                        y: 12
                        width: list.column
                        spacing: 10
                        Text { text: "Start with"; color: t.faint; font.family: t.sans; font.pixelSize: 12; font.weight: Font.DemiBold }
                        Flow {
                            Layout.fillWidth: true
                            spacing: 8
                            Repeater {
                                model: [
                                    { k: "paragraph", icon: "notes", label: "Text", key: "Enter" }, { k: "h1", icon: "title", label: "Heading", key: "#" },
                                    { k: "todo", icon: "check_box", label: "To-do", key: "[ ]" }, { k: "bullet", icon: "format_list_bulleted", label: "List", key: "-" },
                                    { k: "quote", icon: "format_quote", label: "Quote", key: ">" }, { k: "code", icon: "code", label: "Code", key: "```" }
                                ]
                                delegate: Rectangle {
                                    required property var modelData
                                    implicitWidth: startRow.implicitWidth + 28; implicitHeight: 40; radius: t.rRow
                                    color: startHov.hovered ? t.raised : t.card
                                    RowLayout {
                                        id: startRow
                                        anchors.centerIn: parent
                                        spacing: 8
                                        Text { text: modelData.icon; color: t.accent; font.family: t.icons; font.pixelSize: 18 }
                                        Text { text: modelData.label; color: t.text; font.family: t.sans; font.pixelSize: 13 }
                                        Text { text: modelData.key; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
                                    }
                                    HoverHandler { id: startHov; cursorShape: Qt.PointingHandCursor }
                                    TapHandler { onTapped: win.focusAfter = app.addBlock(-1, modelData.k) }
                                }
                            }
                        }
                        Text { text: "or type  /  in a block for every kind,  [[  to link a page"; color: t.faint; font.family: t.sans; font.pixelSize: 12 }
                    }
                    ColumnLayout {
                        id: backCol
                        x: (list.width - list.column) / 2
                        y: 140
                        width: list.column
                        spacing: 6
                        readonly property var items: { try { return JSON.parse(app.backlinks) } catch (e) { return [] } }
                        visible: items.length > 0
                        Rectangle { Layout.fillWidth: true; implicitHeight: 1; color: t.rule; opacity: 0.6 }
                        Text { text: `Linked from ${backCol.items.length} ${backCol.items.length === 1 ? "place" : "places"}`; color: t.faint; font.family: t.sans; font.pixelSize: 12; font.weight: Font.DemiBold; Layout.topMargin: 6 }
                        Repeater {
                            model: backCol.items
                            delegate: Rectangle {
                                required property var modelData
                                Layout.fillWidth: true
                                implicitHeight: blCol.implicitHeight + 16
                                radius: 10
                                color: blHov.hovered ? t.raised : t.card
                                ColumnLayout {
                                    id: blCol
                                    anchors { left: parent.left; right: parent.right; verticalCenter: parent.verticalCenter; margins: 12 }
                                    spacing: 2
                                    Text { text: modelData.title; color: t.accent; font.family: t.sans; font.pixelSize: 14; font.weight: Font.DemiBold }
                                    Text { Layout.fillWidth: true; text: modelData.text; color: t.dim; font.family: t.sans; font.pixelSize: 13; elide: Text.ElideRight }
                                }
                                HoverHandler { id: blHov; cursorShape: Qt.PointingHandCursor }
                                TapHandler { onTapped: app.openTitle(modelData.title) }
                            }
                        }
                    }
                    Item {
                        width: parent.width
                        height: 130
                    TapHandler {
                        onTapped: {
                            const last = win.blocks.length - 1
                            if (last >= 0 && win.blocks[last].kind === "paragraph" && win.blocks[last].text === "") win.focusBlock(last, false)
                            else win.focusAfter = app.addBlock(-1, "paragraph")
                        }
                    }
                    }
                }
            }

            // status bar
            // the drawer: a shell in the workspace (Drawer.qml, Ctrl+`)
            Drawer {
                id: drawer
                visible: win.drawerShown
                Layout.fillWidth: true
                Layout.preferredHeight: Math.max(160, pageCol.height * 0.32)
                Layout.leftMargin: 12; Layout.rightMargin: 12
                win: ApplicationWindow.window; t: ApplicationWindow.window.theme; app: ApplicationWindow.window.engine
            }
            StatusBar { win: ApplicationWindow.window; t: ApplicationWindow.window.theme; app: ApplicationWindow.window.engine }
        }

        // ───────── details: outline, links, info ─────────
        SidePage {
            visible: win.sidePage >= 0 && win.view === "page"
            page: win.sidePage
            Layout.fillWidth: true; Layout.fillHeight: true
            Layout.topMargin: 8; Layout.rightMargin: 8; Layout.bottomMargin: 8
            win: ApplicationWindow.window; t: ApplicationWindow.window.theme; app: ApplicationWindow.window.engine
        }
        Details { win: ApplicationWindow.window; t: ApplicationWindow.window.theme; app: ApplicationWindow.window.engine }
    }
    }

    // ── Ctrl+P: quick open — type to filter, Enter opens, or makes a page with that name ──
    Shortcut { sequence: "Ctrl+P"; onActivated: { quickField.text = ""; quick.open(); quickField.forceActiveFocus() } }
    Popup {
        id: quick
        x: (win.width - width) / 2
        y: 80
        width: Math.min(560, win.width - 80)
        padding: 10
        modal: true
        focus: true
        background: Rectangle { radius: 16; color: t.raised; border.width: 1; border.color: t.rule }
        readonly property var matches: {
            const q = quickField.text.trim().toLowerCase()
            const all = win.pages.map(p => p.title)
            const hit = q.length === 0 ? all : all.filter(tl => tl.toLowerCase().includes(q))
            const exact = all.some(tl => tl.toLowerCase() === q)
            return q.length > 0 && !exact ? hit.concat([{ create: quickField.text.trim() }]) : hit
        }
        property int sel: 0
        onMatchesChanged: sel = 0
        function choose(i) {
            const m = matches[i]
            if (m === undefined) return
            app.openTitle(typeof m === "string" ? m : m.create)
            quick.close()
        }
        contentItem: ColumnLayout {
            spacing: 8
            TextField {
                id: quickField
                Layout.fillWidth: true
                placeholderText: "Go to a page, or name a new one"
                font.family: t.sans; font.pixelSize: 16
                color: t.text
                placeholderTextColor: t.faint
                background: Rectangle { radius: 10; color: t.card }
                Keys.onDownPressed: quick.sel = Math.min(quick.sel + 1, quick.matches.length - 1)
                Keys.onUpPressed: quick.sel = Math.max(quick.sel - 1, 0)
                Keys.onReturnPressed: quick.choose(quick.sel)
                Keys.onEscapePressed: quick.close()
            }
            ListView {
                Layout.fillWidth: true
                Layout.preferredHeight: Math.min(contentHeight, 360)
                clip: true
                model: quick.matches
                delegate: Rectangle {
                    required property var modelData
                    required property int index
                    width: ListView.view.width
                    height: 36
                    radius: 8
                    color: index === quick.sel ? Qt.alpha(t.accent, 0.18) : "transparent"
                    Text {
                        anchors { left: parent.left; leftMargin: 12; verticalCenter: parent.verticalCenter }
                        text: typeof modelData === "string" ? modelData : `+  New page “${modelData.create}”`
                        color: typeof modelData === "string" ? t.text : t.accent
                        font.family: t.sans; font.pixelSize: 14
                    }
                    HoverHandler { cursorShape: Qt.PointingHandCursor; onHoveredChanged: if (hovered) quick.sel = index }
                    TapHandler { onTapped: quick.choose(index) }
                }
            }
        }
    }

    // the slash menu, drawn over the page; the block keeps the keyboard
    Rectangle {
        parent: rootRow
        z: 100
        visible: win.slashFor >= 0 && win.slashMatches.length > 0
        x: win.slashX
        y: win.slashY
        width: 240
        height: slashCol.implicitHeight + 12
        radius: 12
        color: t.raised
        border.width: 1
        border.color: t.rule
        ColumnLayout {
            id: slashCol
            anchors { left: parent.left; right: parent.right; top: parent.top; margins: 6 }
            spacing: 1
            Text { text: "Turn into"; color: t.faint; font.family: t.sans; font.pixelSize: 11; Layout.leftMargin: 8; Layout.topMargin: 2; Layout.bottomMargin: 2 }
            Repeater {
                model: win.slashMatches
                delegate: Rectangle {
                    required property var modelData
                    required property int index
                    Layout.fillWidth: true
                    implicitHeight: 32
                    radius: 8
                    color: index === win.slashSel ? Qt.alpha(t.accent, 0.18) : "transparent"
                    RowLayout {
                        anchors { fill: parent; leftMargin: 10; rightMargin: 10 }
                        Text { Layout.fillWidth: true; text: modelData.label; color: t.text; font.family: t.sans; font.pixelSize: 14 }
                        Text { text: modelData.hint; color: t.faint; font.family: t.mono; font.pixelSize: 12 }
                    }
                    HoverHandler { cursorShape: Qt.PointingHandCursor; onHoveredChanged: if (hovered) win.slashSel = index }
                    TapHandler { onTapped: win.slashApply(index) }
                }
            }
        }
    }

    // ── which-key: what the keys typed so far can continue with ──
    Rectangle {
        parent: rootRow
        z: 90
        visible: win.whichKey.length > 0
        anchors { left: parent.left; right: parent.right; bottom: parent.bottom; margins: 14 }
        height: wkFlow.implicitHeight + 46
        radius: 16
        color: t.raised
        border.width: 1
        border.color: t.rule
        Text {
            id: wkHead
            x: 18; y: 12
            text: win.keySeq.map(k => k === "<Space>" ? "SPC" : k).join(" ") + "  …"
            color: t.accent; font.family: t.mono; font.pixelSize: 12; font.weight: Font.Bold
        }
        Flow {
            id: wkFlow
            anchors { left: parent.left; right: parent.right; top: wkHead.bottom; margins: 18; topMargin: 8 }
            spacing: 6
            Repeater {
                model: win.whichKey
                delegate: Row {
                    required property var modelData
                    width: Math.max(200, (wkFlow.width - 12) / 4)
                    spacing: 10
                    Rectangle {
                        width: Math.max(26, keyText.implicitWidth + 12); height: 24; radius: 7
                        color: t.card
                        Text { id: keyText; anchors.centerIn: parent; text: modelData.key === "<Space>" ? "SPC" : modelData.key; color: t.text; font.family: t.mono; font.pixelSize: 13; font.weight: Font.Bold }
                    }
                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: modelData.group ? "+" + modelData.desc : modelData.desc
                        color: modelData.group ? t.accent : t.dim
                        font.family: t.sans; font.pixelSize: 13
                    }
                }
            }
        }
    }

    // ── : the command line — any command by name, `lua …`, `page Title`, `capture text` ──
    function openCommandLine(prefill) {
        cmdLine.commands = JSON.parse(app.commandList()).commands || []
        cmdField.text = prefill || ""
        cmdLine.sel = 0
        cmdLine.open()
        cmdField.forceActiveFocus()
    }
    Popup {
        id: cmdLine
        x: (win.width - width) / 2
        y: 80
        width: Math.min(620, win.width - 80)
        padding: 10
        modal: true
        focus: true
        property var commands: []
        property int sel: 0
        readonly property var matches: {
            const q = cmdField.text.trim().toLowerCase()
            if (q.includes(" ")) return []
            return commands.filter(c => c.name.includes(q) || c.desc.toLowerCase().includes(q)).slice(0, 12)
        }
        background: Rectangle { radius: 16; color: t.raised; border.width: 1; border.color: t.rule }
        function accept(line) {
            cmdLine.close()
            if (line.trim().length > 0) win.applyScript(JSON.parse(app.runCommand(line)))
        }
        contentItem: ColumnLayout {
            spacing: 8
            RowLayout {
                spacing: 8
                Text { text: ":"; color: t.accent; font.family: t.mono; font.pixelSize: 18; font.weight: Font.Bold }
                TextField {
                    id: cmdField
                    Layout.fillWidth: true
                    placeholderText: "command, lua …, page Title, capture text"
                    font.family: t.mono; font.pixelSize: 15
                    color: t.text
                    placeholderTextColor: t.faint
                    background: Rectangle { radius: 10; color: t.card }
                    Keys.onDownPressed: cmdLine.sel = Math.min(cmdLine.sel + 1, cmdLine.matches.length - 1)
                    Keys.onUpPressed: cmdLine.sel = Math.max(cmdLine.sel - 1, 0)
                    Keys.onTabPressed: { const m = cmdLine.matches[cmdLine.sel]; if (m) text = m.name }
                    Keys.onReturnPressed: {
                        const m = cmdLine.matches[cmdLine.sel]
                        // the picked command, unless what was typed is already a full line
                        const typed = text.trim()
                        cmdLine.accept(m && !cmdLine.commands.some(c => c.name === typed) && !typed.includes(" ") ? m.name : typed)
                    }
                    Keys.onEscapePressed: cmdLine.close()
                }
            }
            ListView {
                Layout.fillWidth: true
                Layout.preferredHeight: Math.min(contentHeight, 380)
                clip: true
                model: cmdLine.matches
                delegate: Rectangle {
                    required property var modelData
                    required property int index
                    width: ListView.view.width
                    height: 34
                    radius: 8
                    color: index === cmdLine.sel ? Qt.alpha(t.accent, 0.18) : "transparent"
                    RowLayout {
                        anchors { fill: parent; leftMargin: 12; rightMargin: 12 }
                        spacing: 12
                        Text { text: modelData.name; color: t.text; font.family: t.mono; font.pixelSize: 13; Layout.preferredWidth: 170 }
                        Text { Layout.fillWidth: true; text: modelData.desc; color: t.dim; font.family: t.sans; font.pixelSize: 13; elide: Text.ElideRight }
                        Text { text: modelData.source === "zett" ? "" : modelData.source; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
                    }
                    HoverHandler { cursorShape: Qt.PointingHandCursor; onHoveredChanged: if (hovered) cmdLine.sel = index }
                    TapHandler { onTapped: cmdLine.accept(modelData.name) }
                }
            }
        }
    }

    // ── help: every key, or every command ──
    function openHelp(what) {
        if (what === "commands") {
            helpPop.title = "Commands"
            helpPop.rows = (JSON.parse(app.commandList()).commands || []).map(c => ({ a: c.name, b: c.desc, c: c.source === "zett" ? "" : c.source }))
        } else {
            helpPop.title = "Keys (normal mode)"
            helpPop.rows = (JSON.parse(app.bindingList()).bindings || []).map(b => ({ a: b.keys, b: b.desc, c: "" }))
        }
        helpPop.open()
    }
    Popup {
        id: helpPop
        x: (win.width - width) / 2
        y: 60
        width: Math.min(640, win.width - 80)
        height: Math.min(560, win.height - 120)
        padding: 16
        modal: true
        focus: true
        property string title: ""
        property var rows: []
        background: Rectangle { radius: 16; color: t.raised; border.width: 1; border.color: t.rule }
        contentItem: ColumnLayout {
            spacing: 10
            Text { text: helpPop.title; color: t.text; font.family: t.sans; font.pixelSize: 18; font.weight: Font.DemiBold }
            Text { text: "Change any of these in " + app.configPath(); color: t.faint; font.family: t.mono; font.pixelSize: 11 }
            ListView {
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                model: helpPop.rows
                delegate: RowLayout {
                    required property var modelData
                    width: ListView.view.width
                    height: 28
                    spacing: 14
                    Text { text: modelData.a; color: t.accent; font.family: t.mono; font.pixelSize: 13; Layout.preferredWidth: 170 }
                    Text { Layout.fillWidth: true; text: modelData.b; color: t.text; font.family: t.sans; font.pixelSize: 13; elide: Text.ElideRight }
                    Text { text: modelData.c; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
                }
            }
        }
    }

    // ── notices from scripts (print, zett.notify, errors) ──
    Column {
        parent: rootRow
        z: 95
        anchors { right: parent.right; top: parent.top; margins: 16 }
        spacing: 6
        Repeater {
            model: win.notes
            delegate: Rectangle {
                required property var modelData
                width: Math.min(420, noteText.implicitWidth + 28)
                height: noteText.implicitHeight + 18
                radius: 12
                color: t.raised
                border.width: 1
                border.color: t.rule
                Text { id: noteText; anchors.centerIn: parent; width: Math.min(392, implicitWidth); text: modelData.text; color: t.text; font.family: t.sans; font.pixelSize: 13; wrapMode: Text.Wrap }
            }
        }
    }
}
