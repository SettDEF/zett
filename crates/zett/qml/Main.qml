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
    Timer { id: grabLater; interval: 700; onTriggered: rootRow.grabToImage(r => { r.saveToFile(app.shot); Qt.quit() }) }

    RowLayout {
        anchors.fill: parent
        spacing: 0

        // ───────── sidebar ─────────
        Rectangle {
            Layout.fillHeight: true
            visible: !win.sidebarHidden
            implicitWidth: 260
            color: t.panel
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
                // new page
                Rectangle {
                    Layout.fillWidth: true
                    Layout.bottomMargin: 8
                    implicitHeight: 38
                    radius: 19
                    color: newHov.hovered ? Qt.lighter(t.accent, 1.08) : t.accent
                    Text { anchors.centerIn: parent; text: "+  New page"; color: t.onAccent; font.family: t.sans; font.pixelSize: 14; font.weight: Font.DemiBold }
                    HoverHandler { id: newHov; cursorShape: Qt.PointingHandCursor }
                    TapHandler { onTapped: { app.newPage("Untitled"); win.focusAfter = 0 } }
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
                        radius: 9
                        readonly property bool sel: index === app.current
                        color: sel ? Qt.alpha(t.accent, 0.16) : rowHov.hovered ? Qt.alpha(t.text, 0.06) : "transparent"
                        Rectangle { visible: parent.sel; anchors { left: parent.left; verticalCenter: parent.verticalCenter } width: 3; height: 16; radius: 1.5; color: t.accent }
                        RowLayout {
                            anchors { fill: parent; leftMargin: 12; rightMargin: 10 }
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
            Rectangle { anchors { right: parent.right; top: parent.top; bottom: parent.bottom } width: 1; color: t.rule; opacity: 0.6 }
        }

        // ───────── the page ─────────
        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 0

            // toolbar: title, and the tools below the window-button corner
            Item {
                Layout.fillWidth: true
                implicitHeight: 64
                RowLayout {
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
                Rectangle { anchors { left: parent.left; right: parent.right; bottom: parent.bottom } height: 1; color: t.rule; opacity: 0.6 }
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

                            // shown instead of the raw text while not editing: [[links]] are clickable
                            Text {
                                id: shown
                                anchors.fill: edit
                                visible: !edit.activeFocus && edit.text.indexOf("[[") >= 0
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
                                opacity: shown.visible ? 0 : 1
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
                    height: 160 + backCol.implicitHeight
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
                implicitHeight: 26
                color: t.panel
                Rectangle { anchors { left: parent.left; right: parent.right; top: parent.top } height: 1; color: t.rule; opacity: 0.6 }
                RowLayout {
                    anchors { fill: parent; leftMargin: 14; rightMargin: 14 }
                    Rectangle { implicitWidth: 7; implicitHeight: 7; radius: 3.5; color: "#5fb98a" }
                    Text { text: `${win.blocks.length} blocks · saved as Markdown`; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
                    Text { visible: win.vimMode.length > 0; text: win.vimMode; color: win.vimMode === "INSERT" ? t.dim : t.accent; font.family: t.mono; font.pixelSize: 11; font.weight: Font.Bold }
                    Item { Layout.fillWidth: true }
                    Text { text: "Ctrl+P go to · [[Page]] links · Esc Vim keys · Enter new block · Backspace on empty removes · # - [ ] - > ``` change kind"; color: t.faint; font.family: t.sans; font.pixelSize: 11 }
                }
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
