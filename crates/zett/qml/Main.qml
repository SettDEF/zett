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
        readonly property string sans: "Google Sans Flex"
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
    property bool detailsShown: true
    // the open pages (indices into pages), the focus stack's tabs
    property var tabs: []
    function addTab(i) { if (i >= 0 && !tabs.includes(i)) tabs = tabs.concat([i]) }
    function closeTab(i) {
        const at = tabs.indexOf(i)
        tabs = tabs.filter(x => x !== i)
        if (i === app.current && tabs.length) app.open(tabs[Math.min(at, tabs.length - 1)])
    }
    Connections { target: app; function onCurrentChanged() { win.addTab(app.current) } }
    Component.onCompleted: addTab(app.current)
    Shortcut { sequence: "Ctrl+W"; onActivated: win.closeTab(app.current) }
    Shortcut { sequence: "Ctrl+Tab"; onActivated: if (win.tabs.length > 1) app.open(win.tabs[(win.tabs.indexOf(app.current) + 1) % win.tabs.length]) }
    Shortcut { sequence: "Ctrl+\\"; onActivated: win.detailsShown = !win.detailsShown }
    Shortcut { sequence: "F2"; onActivated: win.renameReq++ }

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
        ]
    }
    property int renameReq: 0
    // wraps the selection in a Markdown mark (bold, italic, code, link)
    function wrapSel(ed, a, z) {
        const s = ed.selectionStart, e = ed.selectionEnd, t = ed.selectedText
        ed.remove(s, e); ed.insert(s, a + t + z)
    }
    function blockMenu(i, x, y, ed, link) {
        const b = win.blocks[i]
        if (!b) return
        // on a [[link]]: what to do with that page
        if (link) {
            const title = decodeURIComponent(link)
            ctx.show(x, y, clipRow(`[[${title}]]`, null, null, null).slice(0, 1), [
                { icon: "open_in_new", label: "Open " + title, run: () => app.openTitle(title) },
                { icon: "tab", label: "Open in a tab", run: () => { app.openTitle(title); win.addTab(app.current) } },
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
            { icon: "add", label: "Add block below", key: "Enter", run: () => win.focusAfter = app.addBlock(i, "paragraph") },
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
            { icon: "open_in_new", label: "Open", run: () => app.open(i) },
            { icon: "tab", label: "Keep in a tab", enabled: !win.tabs.includes(i), run: () => win.addTab(i) },
            { icon: "edit", label: "Rename", key: "F2", run: () => { app.open(i); win.renameReq++ } },
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
            { icon: "tab_close_right", label: "Close others", enabled: win.tabs.length > 1, run: () => { win.tabs = [i]; app.open(i) } },
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
    Rectangle { x: pageCol.x; y: pageCol.y; width: pageCol.width; height: pageCol.height; radius: t.rCard; color: t.panel; border.width: 1; border.color: Qt.alpha(t.rule, 0.35) }
    ContextMenu { id: ctx; theme: t; z: 1000 }

    RowLayout {
        anchors.fill: parent
        spacing: 0

        // ───────── sidebar ─────────
        Rectangle {
            Layout.fillHeight: true
            visible: !win.sidebarHidden
            implicitWidth: 260
            color: "transparent"
            ColumnLayout {
                anchors { fill: parent; margins: 12; topMargin: 16 }
                spacing: 4
                RowLayout {
                    Layout.leftMargin: 6
                    Layout.bottomMargin: 12
                    spacing: 10
                    Rectangle {
                        implicitWidth: 34; implicitHeight: 34; radius: 10
                        color: t.accent
                        Text { anchors.centerIn: parent; text: "z"; color: t.onAccent; font.family: t.sans; font.pixelSize: 20; font.weight: Font.Bold }
                    }
                    ColumnLayout {
                        spacing: 0
                        Text { text: "zett"; color: t.text; font.family: t.sans; font.pixelSize: 17; font.weight: Font.DemiBold }
                        Text { text: `${win.pages.length} pages`; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
                    }
                }
                // search: opens quick open
                Rectangle {
                    Layout.fillWidth: true
                    Layout.bottomMargin: 10
                    implicitHeight: 36
                    radius: 18
                    color: searchHov.hovered ? t.raised : t.card
                    RowLayout {
                        anchors { fill: parent; leftMargin: 12; rightMargin: 12 }
                        spacing: 8
                        Text { text: "search"; color: t.dim; font.family: t.icons; font.pixelSize: 18 }
                        Text { Layout.fillWidth: true; text: "Search pages"; color: t.faint; font.family: t.sans; font.pixelSize: 13 }
                        Text { text: "Ctrl P"; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
                    }
                    HoverHandler { id: searchHov; cursorShape: Qt.PointingHandCursor }
                    TapHandler { onTapped: { quickField.text = ""; quick.open(); quickField.forceActiveFocus() } }
                }
                // section header with the new-page button
                RowLayout {
                    Layout.fillWidth: true
                    Layout.leftMargin: 8
                    Layout.bottomMargin: 2
                    Text { text: "PAGES"; color: t.faint; font.family: t.sans; font.pixelSize: 11; font.weight: Font.DemiBold; font.letterSpacing: 0.8 }
                    Text { text: win.pages.length; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
                    Item { Layout.fillWidth: true }
                    Rectangle {
                        implicitWidth: 26; implicitHeight: 26; radius: 13
                        color: newHov.hovered ? t.raised : "transparent"
                        Text { anchors.centerIn: parent; text: "add"; color: newHov.hovered ? t.accent : t.dim; font.family: t.icons; font.pixelSize: 18 }
                        HoverHandler { id: newHov; cursorShape: Qt.PointingHandCursor }
                        TapHandler { onTapped: { app.newPage("Untitled"); win.focusAfter = 0 } }
                    }
                }
                ListView {
                    id: pageList
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    spacing: 2
                    model: win.pages
                    delegate: Rectangle {
                        required property var modelData
                        required property int index
                        width: ListView.view.width
                        height: 36
                        radius: t.rRow
                        readonly property bool sel: index === app.current
                        color: sel ? Qt.alpha(t.accent, 0.16) : rowHov.hovered ? Qt.alpha(t.text, 0.06) : "transparent"
                        RowLayout {
                            anchors { fill: parent; leftMargin: 10; rightMargin: 12 }
                            spacing: 10
                            Text { text: "description"; color: parent.parent.sel ? t.accent : t.faint; font.family: t.icons; font.pixelSize: 18 }
                            Text {
                                Layout.fillWidth: true
                                text: modelData.title || "Untitled"
                                elide: Text.ElideRight
                                color: parent.parent.sel ? t.text : t.dim
                                font.family: t.sans; font.pixelSize: 14
                                font.weight: parent.parent.sel ? Font.DemiBold : Font.Normal
                            }
                            Text { text: modelData.blocks; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
                        }
                        HoverHandler { id: rowHov; cursorShape: Qt.PointingHandCursor }
                        TapHandler { onTapped: app.open(index) }
                        TapHandler { acceptedButtons: Qt.RightButton; onTapped: (ev) => win.pageMenu(index, ev.scenePosition.x, ev.scenePosition.y) }
                    }
                }
                Text {
                    Layout.fillWidth: true
                    text: app.dir
                    elide: Text.ElideLeft
                    color: t.faint
                    font.family: t.mono; font.pixelSize: 11
                }
            }
        }

        // ───────── the page ─────────
        ColumnLayout {
            id: pageCol
            Layout.topMargin: 8; Layout.rightMargin: 8; Layout.bottomMargin: 8
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 0

            // toolbar: title, and the tools below the window-button corner
            Item {
                Layout.fillWidth: true
                implicitHeight: 64
                // the open pages as pill tabs
                ListView {
                    anchors { left: parent.left; right: toolRow.left; top: parent.top; leftMargin: 12; rightMargin: 12; topMargin: 24 }
                    height: 34
                    orientation: ListView.Horizontal
                    spacing: 6
                    clip: true
                    model: win.tabs
                    delegate: Rectangle {
                        required property int modelData
                        readonly property bool sel: modelData === app.current
                        readonly property var pg: win.pages[modelData] ?? {}
                        width: Math.min(200, tabRow.implicitWidth + 24); height: 32; radius: 16
                        color: sel ? Qt.alpha(t.accent, 0.16) : tabHov.hovered ? t.raised : t.card
                        RowLayout {
                            id: tabRow
                            anchors { fill: parent; leftMargin: 12; rightMargin: 8 }
                            spacing: 6
                            Text { text: "description"; color: parent.parent.sel ? t.accent : t.faint; font.family: t.icons; font.pixelSize: 16 }
                            Text { Layout.fillWidth: true; Layout.maximumWidth: 130; text: parent.parent.pg.title || "Untitled"; elide: Text.ElideRight; color: parent.parent.sel ? t.text : t.dim; font.family: t.sans; font.pixelSize: 13; font.weight: parent.parent.sel ? Font.DemiBold : Font.Normal }
                            Text {
                                text: "close"; color: closeHov.hovered ? t.text : t.faint; font.family: t.icons; font.pixelSize: 14
                                HoverHandler { id: closeHov; cursorShape: Qt.PointingHandCursor }
                                TapHandler { onTapped: win.closeTab(modelData) }
                            }
                        }
                        HoverHandler { id: tabHov; cursorShape: Qt.PointingHandCursor }
                        TapHandler { onTapped: app.open(modelData) }
                        TapHandler { acceptedButtons: Qt.RightButton; onTapped: (ev) => win.tabMenu(modelData, ev.scenePosition.x, ev.scenePosition.y) }
                        TapHandler { acceptedButtons: Qt.MiddleButton; onTapped: win.closeTab(modelData) }
                    }
                }
                RowLayout {
                    id: toolRow
                    anchors { right: parent.right; top: parent.top; rightMargin: 14; topMargin: 28 }
                    spacing: 6
                    Rectangle {
                        implicitWidth: srcRow.implicitWidth + 24; implicitHeight: 30; radius: 15
                        color: win.showSource ? t.accent : (srcHov.hovered ? t.raised : t.card)
                        RowLayout { id: srcRow; anchors.centerIn: parent; Text { text: "Markdown"; color: win.showSource ? t.onAccent : t.dim; font.family: t.sans; font.pixelSize: 13 } }
                        HoverHandler { id: srcHov; cursorShape: Qt.PointingHandCursor }
                        TapHandler { onTapped: win.showSource = !win.showSource }
                    }
                }
            }

            // empty workspace
            ColumnLayout {
                visible: win.pages.length === 0
                Layout.alignment: Qt.AlignCenter
                Layout.fillHeight: true
                spacing: 10
                Text { Layout.alignment: Qt.AlignHCenter; text: "No pages yet"; color: t.text; font.family: t.sans; font.pixelSize: 22; font.weight: Font.DemiBold }
                Text { Layout.alignment: Qt.AlignHCenter; text: "Make one with  + New page, or drop .md files into " + app.dir; color: t.dim; font.family: t.sans; font.pixelSize: 14 }
            }

            // Markdown source (read-only view of what is on disk)
            ScrollView {
                visible: win.showSource && win.pages.length > 0
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
                visible: !win.showSource && win.pages.length > 0
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                spacing: 2
                topMargin: 28
                bottomMargin: 120
                model: win.blocks
                boundsBehavior: Flickable.StopAtBounds
                readonly property real column: Math.min(760, width - 96)

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

                    RowLayout {
                        id: body
                        x: (list.width - list.column) / 2
                        width: list.column
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

                            // a code block, coloured by its language while not being edited
                            Text {
                                id: codeShown
                                anchors.fill: edit
                                visible: !edit.activeFocus && blockItem.kind === "code"
                                text: visible ? app.highlight(blockItem.modelData.lang ?? "", edit.text) : ""
                                textFormat: Text.RichText
                                font: edit.font
                                color: edit.color
                                HoverHandler { cursorShape: Qt.IBeamCursor }
                                TapHandler { onTapped: edit.forceActiveFocus() }
                                z: 2
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
                                opacity: shown.visible || codeShown.visible ? 0 : 1
                                anchors { fill: parent; margins: blockItem.kind === "code" ? 12 : 0 }
                                text: blockItem.modelData.text
                                wrapMode: TextEdit.Wrap
                                selectByMouse: true
                                color: blockItem.kind === "todo" && blockItem.checked ? t.faint : blockItem.kind === "quote" ? t.dim : t.text
                                font.family: blockItem.kind === "code" ? t.mono : t.sans
                                font.pixelSize: blockItem.kind === "heading" ? [0, 28, 22, 18, 16, 15, 14][blockItem.level] : blockItem.kind === "code" ? 14 : 16
                                font.weight: blockItem.kind === "heading" ? Font.DemiBold : Font.Normal
                                font.italic: blockItem.kind === "quote"
                                font.strikeout: blockItem.kind === "todo" && blockItem.checked
                                readonly property bool visualSel: win.vimMode === "VISUAL" || win.vimMode === "V-LINE"
                                selectionColor: visualSel ? Qt.alpha(t.accent, 0.42) : normal ? t.accent : Qt.alpha(t.accent, 0.4)
                                selectedTextColor: normal && !visualSel ? t.onAccent : t.text
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
                                }

                                Keys.onPressed: (e) => {
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
            Rectangle {
                Layout.fillWidth: true
                implicitHeight: 40
                color: "transparent"
                // status chips, as in the shell's bar
                component Chip: Rectangle {
                    property string icon
                    property string label
                    property color tint: t.dim
                    implicitWidth: chipRow.implicitWidth + 20; implicitHeight: 24; radius: 12
                    color: t.card
                    RowLayout {
                        id: chipRow
                        anchors.centerIn: parent
                        spacing: 5
                        Text { visible: parent.parent.icon.length > 0; text: parent.parent.icon; color: parent.parent.tint; font.family: t.icons; font.pixelSize: 14 }
                        Text { text: parent.parent.label; color: parent.parent.tint; font.family: t.mono; font.pixelSize: 11 }
                    }
                }
                RowLayout {
                    anchors { fill: parent; leftMargin: 12; rightMargin: 12 }
                    spacing: 6
                    Chip { visible: win.vimMode.length > 0; label: win.vimMode; tint: win.vimMode === "INSERT" ? t.dim : t.accent }
                    Chip { icon: "check_circle"; label: "saved"; tint: t.accent }
                    Chip { icon: "notes"; label: `${win.blocks.length} blocks` }
                    // the hints take what is left and shorten, so nothing runs past the card
                    Text { Layout.fillWidth: true; Layout.minimumWidth: 0; horizontalAlignment: Text.AlignRight; elide: Text.ElideLeft; text: "Ctrl P go to  ·  Ctrl \\ details  ·  / block kinds  ·  Esc Vim"; color: t.faint; font.family: t.sans; font.pixelSize: 11 }
                    Rectangle {
                        implicitWidth: 28; implicitHeight: 24; radius: 12
                        color: detHov.hovered ? t.raised : t.card
                        Text { anchors.centerIn: parent; text: win.detailsShown ? "right_panel_close" : "right_panel_open"; color: t.dim; font.family: t.icons; font.pixelSize: 16 }
                        HoverHandler { id: detHov; cursorShape: Qt.PointingHandCursor }
                        TapHandler { onTapped: win.detailsShown = !win.detailsShown }
                    }
                }
            }
        }

        // ───────── details: outline, links, info ─────────
        Rectangle {
            visible: win.detailsShown && win.pages.length > 0
            Layout.fillHeight: true
            Layout.topMargin: 8; Layout.rightMargin: 8; Layout.bottomMargin: 8
            implicitWidth: 248
            radius: t.rCard
            color: t.panel
            border.width: 1; border.color: Qt.alpha(t.rule, 0.35)
            readonly property var heads: win.blocks.map((b, i) => ({ i, b })).filter(o => /^(h[1-3]|heading)$/.test(o.b.kind) && (o.b.text ?? "").length)
            readonly property var links: { try { return JSON.parse(app.backlinks) } catch (e) { return [] } }
            readonly property int words: win.blocks.reduce((n, b) => n + ((b.text ?? "").match(/\S+/g)?.length ?? 0), 0)
            component Head: Text { color: t.faint; font.family: t.sans; font.pixelSize: 11; font.weight: Font.DemiBold; font.letterSpacing: 0.8; Layout.leftMargin: 8; Layout.topMargin: 10 }
            component Line: Text { color: t.faint; font.family: t.sans; font.pixelSize: 12; Layout.leftMargin: 8; Layout.fillWidth: true; elide: Text.ElideRight }
            ColumnLayout {
                anchors { fill: parent; margins: 12 }
                spacing: 2
                Head { text: "OUTLINE"; Layout.topMargin: 4 }
                Line { visible: parent.parent.heads.length === 0; text: "No headings yet" }
                Repeater {
                    model: parent.parent.heads
                    delegate: Rectangle {
                        required property var modelData
                        Layout.fillWidth: true
                        implicitHeight: 30; radius: t.rRow
                        color: olHov.hovered ? Qt.alpha(t.text, 0.06) : "transparent"
                        Text {
                            anchors { fill: parent; leftMargin: 8 + (modelData.b.kind === "h3" ? 24 : modelData.b.kind === "h2" ? 12 : 0); rightMargin: 8 }
                            verticalAlignment: Text.AlignVCenter
                            text: modelData.b.text; elide: Text.ElideRight
                            color: t.dim; font.family: t.sans; font.pixelSize: 13
                        }
                        HoverHandler { id: olHov; cursorShape: Qt.PointingHandCursor }
                        TapHandler { onTapped: { list.positionViewAtIndex(modelData.i, ListView.Beginning); win.focusAfter = modelData.i } }
                    }
                }
                Head { text: "LINKED FROM  " + parent.parent.links.length }
                Line { visible: parent.parent.links.length === 0; text: "Nothing links here yet" }
                Repeater {
                    model: parent.parent.links
                    delegate: Line { required property var modelData; text: "↳  " + (modelData.title ?? modelData.page ?? ""); color: t.dim }
                }
                Head { text: "INFO" }
                Line { text: `${win.blocks.length} blocks  ·  ${parent.parent.words} words` }
                Line { text: app.dir; elide: Text.ElideLeft }
                Item { Layout.fillHeight: true }
            }
        }
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
