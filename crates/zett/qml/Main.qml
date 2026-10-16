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

    function vimEnter(i) {
        const d = list.itemAtIndex(i)
        if (d) d.vimHere()
    }

    onBlocksChanged: if (focusAfter >= 0) {
        const i = focusAfter
        focusAfter = -1
        Qt.callLater(() => { const d = list.itemAtIndex(i); if (d) d.focusEditor(0) })
    }

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
            if (app.shotKeys.length > 0) {
                win.vimEnter(1)
                const d = list.itemAtIndex(1)
                for (const k of app.shotKeys.split(" ")) d.vimKeyForTest(k)
            }
            Qt.callLater(() => rootRow.grabToImage(r => { r.saveToFile(app.shot); Qt.quit() }))
        }
    }

    Rectangle {
    id: rootRow
    anchors.fill: parent
    color: t.bg
    RowLayout {
        anchors.fill: parent
        spacing: 0

        // ───────── sidebar ─────────
        Rectangle {
            Layout.fillHeight: true
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
                                selectionColor: normal ? t.accent : Qt.alpha(t.accent, 0.4)
                                selectedTextColor: normal ? t.onAccent : t.text
                                // Vim normal mode: keys go to zett-edit (window.rs), the cursor is a block
                                property bool normal: false
                                property bool applying: false
                                function applyVim(json) {
                                    const s = JSON.parse(json)
                                    if (s.text === undefined) return
                                    applying = true
                                    if (s.text !== text) text = s.text
                                    applying = false
                                    normal = s.mode === "normal"
                                    win.vimMode = normal ? "NORMAL" : "INSERT"
                                    cursorPosition = Math.min(s.cursor, length)
                                    if (normal && length > 0) select(Math.min(s.cursor, length - 1), Math.min(s.cursor, length - 1) + 1)
                                    else deselect()
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
                                        if (e.key === Qt.Key_Escape) { normal = false; deselect(); win.vimMode = ""; edit.focus = false; e.accepted = true; return }
                                        const k = vimKeyName(e)
                                        if (!k) return
                                        const before = cursorPosition
                                        applyVim(app.vimKey(k))
                                        // j on the last line, k on the first: on to the next or previous block
                                        if ((k === "j" || k === "<Down>") && cursorPosition === before && blockItem.index < win.blocks.length - 1) win.vimEnter(blockItem.index + 1)
                                        else if ((k === "k" || k === "<Up>") && cursorPosition === before && blockItem.index > 0) win.vimEnter(blockItem.index - 1)
                                        e.accepted = true
                                        return
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
                    Text { visible: win.vimMode.length > 0; text: win.vimMode; color: win.vimMode === "NORMAL" ? t.accent : t.dim; font.family: t.mono; font.pixelSize: 11; font.weight: Font.Bold }
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
}
