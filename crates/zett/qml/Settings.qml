import QtQuick
import QtQuick.Layouts

// Settings, structured like VS Code's in our look: search, categories on the left, one row a setting (what it
// is, its control, a bar + Reset when changed), a JSON view of what is stored. Saved at once to
// ~/.config/zett/gui.json; the editor's own options stay in init.lua (a button opens it).
Item {
    id: s
    required property var win
    required property var t
    required property var app

    // a row: label + hint on the left, the control on the right
    component Row1: RowLayout {
        default property alias control: slot.data
        property string label
        property string hint
        Layout.fillWidth: true
        spacing: 16
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 2
            Text { Layout.fillWidth: true; text: label; color: t.text; font.family: t.sans; font.pixelSize: 14 }
            Text { visible: hint.length > 0; Layout.fillWidth: true; text: hint; wrapMode: Text.Wrap; color: t.faint; font.family: t.sans; font.pixelSize: 12 }
        }
        Item { id: slot; implicitWidth: childrenRect.width; implicitHeight: childrenRect.height }
    }
    // choose one of a few: pills side by side
    component Choice: Row {
        property var options: []        // [[value, label]]
        property var value
        signal picked(var v)
        spacing: 4
        Repeater {
            model: options
            delegate: Rectangle {
                required property var modelData
                readonly property bool on: modelData[0] === value
                width: cl.implicitWidth + 26; height: 32; radius: 16
                color: on ? t.accent : ch.hovered ? t.raised : t.card
                Text { id: cl; anchors.centerIn: parent; text: modelData[1]; color: parent.on ? t.onAccent : t.dim; font.family: t.sans; font.pixelSize: 13; font.weight: parent.on ? Font.DemiBold : Font.Normal }
                HoverHandler { id: ch; cursorShape: Qt.PointingHandCursor }
                TapHandler { onTapped: picked(modelData[0]) }
            }
        }
    }
    // a number with − and +
    component Stepper: Row {
        property real value
        property real step: 1
        property real from: 0
        property real to: 100
        property string suffix: ""
        signal changed(real v)
        spacing: 4
        Repeater {
            model: ["remove", "", "add"]
            delegate: Rectangle {
                required property string modelData
                required property int index
                width: index === 1 ? 64 : 32; height: 32; radius: 16
                color: index !== 1 && sh.hovered ? t.raised : t.card
                Text {
                    anchors.centerIn: parent
                    text: index === 1 ? `${Math.round(value * 100) / 100}${suffix}` : modelData
                    color: index === 1 ? t.text : t.dim
                    font.family: index === 1 ? t.mono : t.icons; font.pixelSize: index === 1 ? 12 : 18
                }
                HoverHandler { id: sh; cursorShape: index !== 1 ? Qt.PointingHandCursor : Qt.ArrowCursor }
                TapHandler { enabled: index !== 1; onTapped: changed(Math.max(from, Math.min(to, value + (index === 0 ? -step : step)))) }
            }
        }
    }
    component Section: Rectangle {
        default property alias body: inner.data
        property string title
        property string icon
        Layout.fillWidth: true
        implicitHeight: inner.implicitHeight + 32
        radius: 17
        color: t.card
        ColumnLayout {
            id: inner
            anchors { left: parent.left; right: parent.right; top: parent.top; margins: 16 }
            spacing: 14
            RowLayout {
                spacing: 8
                Text { text: icon; color: t.accent; font.family: t.icons; font.pixelSize: 18 }
                Text { text: title; color: t.text; font.family: t.sans; font.pixelSize: 15; font.weight: Font.DemiBold }
            }
        }
    }
    component Button1: Rectangle {
        property string icon
        property string label
        signal clicked
        implicitWidth: br.implicitWidth + 24; implicitHeight: 32; radius: 16
        color: bh.hovered ? t.raised : Qt.alpha(t.text, 0.06)
        RowLayout { id: br; anchors.centerIn: parent; spacing: 6
            Text { text: icon; color: t.dim; font.family: t.icons; font.pixelSize: 16 }
            Text { text: label; color: t.dim; font.family: t.sans; font.pixelSize: 13 } }
        HoverHandler { id: bh; cursorShape: Qt.PointingHandCursor }
        TapHandler { onTapped: parent.clicked() }
    }

    // every setting: key in gui.json, category, label, hint, control (choice / stepper / action), default
    readonly property var schema: [
        { cat: "Editor", key: "keys", label: "Key profile", hint: "Emacs keys type at once; Vim keys start in normal mode (Esc)", type: "choice", options: [["emacs", "Emacs"], ["vim", "Vim"]], def: "emacs" },
        { cat: "Editor", key: "codeLigatures", label: "Ligatures in code", hint: "=> drawn as ⇒ in code blocks and files", type: "choice", options: [[false, "Off"], [true, "On"]], def: false },
        { cat: "Editor", key: "formatOnSave", label: "Format on save", hint: "Ctrl+S in the file editor lets the language server format first", type: "choice", options: [[false, "Off"], [true, "On"]], def: false },
        { cat: "Editor", key: "pumWheel", label: "Wheel moves through completions", hint: "The mouse wheel over the completion list scrolls the list; off: it scrolls the code under it", type: "choice", options: [[true, "On"], [false, "Off"]], def: true },
        { cat: "Editor", key: "trimOnSave", label: "Trim trailing whitespace", hint: "Ctrl+S takes the spaces at line ends away", type: "choice", options: [[true, "On"], [false, "Off"]], def: true },
        { cat: "Editor", key: "finalNewline", label: "Final newline", hint: "Ctrl+S ends the file with a newline", type: "choice", options: [[true, "On"], [false, "Off"]], def: true },
        { cat: "Editor", key: "autoSave", label: "Auto save", hint: "a file is written a second after the last change", type: "choice", options: [[false, "Off"], [true, "After a second"]], def: false },
        { cat: "Editor", key: "wordWrap", label: "Word wrap", hint: "long lines wrap at the window's edge (Alt+Z)", type: "choice", options: [[false, "Off"], [true, "On"]], def: false },
        { cat: "Editor", key: "minimap", label: "Minimap", hint: "the whole file small at the right of the editor", type: "choice", options: [[true, "On"], [false, "Off"]], def: true },
        { cat: "Editor", key: "terminal", label: "Terminal", hint: "what Ctrl+` opens: the panel under the editor, or a real zash window in the workspace folder", type: "choice", options: [["panel", "Panel"], ["zash", "zash"]], def: "panel" },
        { cat: "Editor", key: "newCore", label: "Editor", hint: "zett's own editor (every cursor, folding, big files fast) or the classic one", type: "choice", options: [[true, "zett"], [false, "Classic"]], def: true },
        { cat: "Appearance", key: "scale", label: "Text size", hint: "the page's text, headings with it", type: "stepper", from: 0.8, to: 1.5, step: 0.05, suffix: "×", def: 1 },
        { cat: "Appearance", key: "width", label: "Page width", hint: "the longest a line gets; about 70 characters reads best (the toolbar button: narrow, wide, full)", type: "stepper", from: 480, to: 1400, step: 40, suffix: " px", def: 760 },
        { cat: "Appearance", key: "sans", label: "Font", hint: "for the page and the window", type: "choice", options: [["Google Sans Flex", "Google Sans"], ["Inter", "Inter"], ["Noto Serif", "Serif"]], def: "Google Sans Flex" },
        { cat: "Window", key: "start", label: "Open on", hint: "what the window shows when it starts", type: "choice", options: [["today", "Home"], ["page", "Last page"]], def: "today" },
        { cat: "Window", key: "details", label: "Details card", hint: "outline, links and counts beside the page (Ctrl+\\)", type: "choice", options: [[true, "Shown"], [false, "Hidden"]], def: true },
        { cat: "Window", key: "sideMode", label: "Sidebar shows", hint: "pages and files, or one of them (the activity bar switches it too)", type: "choice", options: [["all", "Both"], ["pages", "Pages"], ["files", "Files"]], def: "all" },
        { cat: "Advanced", key: "", label: "init.lua", hint: app.configPath(), type: "action", actions: [["edit", "Open", () => win.openFile({ path: app.configPath(), rel: "init.lua", dir: false })], ["refresh", "Reload", () => win.applyScript(JSON.parse(app.reloadConfig()))]] },
        { cat: "Advanced", key: "", label: "Workspace", hint: app.dir, type: "action", actions: [["folder_open", "Show folder", () => Qt.openUrlExternally("file://" + app.dir)]] },
        { cat: "Advanced", key: "", label: "Commands", hint: "everything zett can do, with its key", type: "action", actions: [["terminal", "Palette  Ctrl ⇧ P", () => win.openPalette()]] }
    ]
    readonly property var cats: [["Editor", "edit_note"], ["Appearance", "palette"], ["Window", "dashboard"], ["Advanced", "tune"], ["Keyboard Shortcuts", "keyboard"], ["Extensions", "extension"], ["Snippets", "data_object"]]
    property string cat: app.shotEngine.startsWith("settings:Extensions") ? "Extensions" : ""   // "" = all (a test can open one)
    property bool jsonView: false
    function valueOf(x) { return win.prefs[x.key] ?? x.def }
    function changed(x) { return x.key !== "" && win.prefs[x.key] !== undefined && win.prefs[x.key] !== x.def }
    function set(x, v) {
        win.setPref(x.key, v)
        if (x.key === "details") win.detailsShown = v
        if (x.key === "sideMode") win.sideMode = v
    }
    readonly property var shown: {
        const q = search.text.trim().toLowerCase()
        return schema.filter(x => (!cat || x.cat === cat) && (!q || (x.label + " " + x.hint + " " + x.cat + " " + x.key).toLowerCase().includes(q)))
    }

    // ── the categories, on the left ──
    Column {
        id: catCol
        x: 16; y: 96; width: 190
        spacing: 2
        Repeater {
            model: [["", "All settings"]].concat(s.cats.map(c => [c[0], c[0], c[1]]))
            delegate: Rectangle {
                required property var modelData
                readonly property bool on: s.cat === modelData[0]
                readonly property int n: s.schema.filter(x => (!modelData[0] || x.cat === modelData[0]) && s.changed(x)).length
                width: catCol.width; height: 34; radius: 10
                color: on ? Qt.alpha(t.accent, 0.16) : ccH.hovered ? Qt.alpha(t.text, 0.06) : "transparent"
                Row {
                    x: 10; anchors.verticalCenter: parent.verticalCenter; spacing: 10
                    Text { text: modelData[2] ?? "settings"; color: parent.parent.on ? t.accent : t.faint; font.family: t.icons; font.pixelSize: 17 }
                    Text { text: modelData[1]; color: parent.parent.on ? t.text : t.dim; font.family: t.sans; font.pixelSize: 14; font.weight: parent.parent.on ? Font.DemiBold : Font.Normal }
                }
                // how many were changed here
                Text { visible: parent.n > 0; anchors { right: parent.right; rightMargin: 10; verticalCenter: parent.verticalCenter } text: parent.n; color: t.accent; font.family: t.mono; font.pixelSize: 11 }
                HoverHandler { id: ccH; cursorShape: Qt.PointingHandCursor }
                TapHandler { onTapped: s.cat = modelData[0] }
            }
        }
    }

    // ── the title, the search, the JSON switch ──
    ColumnLayout {
        id: top
        x: 24; y: 24; width: s.width - 48
        spacing: 12
        RowLayout {
            Layout.fillWidth: true
            Text { Layout.fillWidth: true; text: "Settings"; color: t.text; font.family: t.sans; font.pixelSize: 26; font.weight: Font.Bold }
            Choice { options: [[false, "Settings"], [true, "JSON"]]; value: s.jsonView; onPicked: (v) => s.jsonView = v }
        }
    }
    Rectangle {
        id: searchBox
        x: catCol.x + catCol.width + 24; y: 92
        width: Math.min(760, s.width - x - 24); height: 38; radius: 12
        color: t.card
        border.width: search.activeFocus ? 1 : 0; border.color: Qt.alpha(t.accent, 0.5)
        visible: !s.jsonView
        Text { id: sIc; x: 12; anchors.verticalCenter: parent.verticalCenter; text: "search"; color: t.dim; font.family: t.icons; font.pixelSize: 18 }
        TextInput {
            id: search
            anchors { left: sIc.right; right: parent.right; verticalCenter: parent.verticalCenter; leftMargin: 10; rightMargin: 12 }
            color: t.text; font.family: t.sans; font.pixelSize: 14; clip: true
            Text { visible: !search.text; text: s.cat === "Extensions" && s.marketInfo[0] ? `Search ${s.marketInfo[0].toLocaleString()} plugins` : "Search settings"; color: t.faint; font: search.font }
        }
    }

    // ── the settings ──
    Flickable {
        visible: !s.jsonView && ["Keyboard Shortcuts", "Extensions", "Snippets"].indexOf(s.cat) < 0
        x: searchBox.x; y: searchBox.y + searchBox.height + 16
        width: searchBox.width; height: s.height - y - 16
        clip: true
        contentHeight: list.implicitHeight + 40
        boundsBehavior: Flickable.StopAtBounds
        ColumnLayout {
            id: list
            width: parent.width
            spacing: 4
            Text { visible: s.shown.length === 0; text: "No setting matches"; color: t.faint; font.family: t.sans; font.pixelSize: 14 }
            Repeater {
                model: s.shown
                delegate: Rectangle {
                    id: row
                    required property var modelData
                    required property int index
                    readonly property bool first: index === 0 || s.shown[index - 1].cat !== modelData.cat
                    Layout.fillWidth: true
                    implicitHeight: rowCol.implicitHeight + 20
                    radius: 12
                    color: rh.hovered ? Qt.alpha(t.text, 0.03) : "transparent"
                    HoverHandler { id: rh }
                    // changed from the default: a bar on the left
                    Rectangle { visible: s.changed(modelData); x: 0; y: (row.first ? 34 : 0) + 8; width: 3; height: rowCol.implicitHeight - (row.first ? 34 : 0) - 4; radius: 1.5; color: t.accent }
                    ColumnLayout {
                        id: rowCol
                        x: 14; y: 10; width: parent.width - 28
                        spacing: 6
                        // the category's name over its first setting
                        Text { visible: row.first; Layout.topMargin: row.index ? 14 : 0; text: modelData.cat.toUpperCase(); color: t.faint; font.family: t.sans; font.pixelSize: 11; font.weight: Font.DemiBold; font.letterSpacing: 0.8 }
                        RowLayout {
                            Layout.fillWidth: true
                            spacing: 8
                            Text { text: modelData.label; color: t.text; font.family: t.sans; font.pixelSize: 14; font.weight: Font.DemiBold }
                            Text { visible: modelData.key.length > 0; text: modelData.key; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
                            Item { Layout.fillWidth: true }
                            Text {
                                visible: s.changed(modelData)
                                text: "Reset"; color: rsH.hovered ? t.accent : t.dim; font.family: t.sans; font.pixelSize: 12
                                HoverHandler { id: rsH; cursorShape: Qt.PointingHandCursor }
                                TapHandler { onTapped: s.set(modelData, modelData.def) }
                            }
                        }
                        Text { Layout.fillWidth: true; text: modelData.hint; wrapMode: Text.Wrap; color: t.faint; font.family: t.sans; font.pixelSize: 12 }
                        Loader {
                            sourceComponent: modelData.type === "choice" ? choiceC : modelData.type === "stepper" ? stepC : actC
                            property var setting: modelData
                        }
                    }
                }
            }
        }
    }
    Component { id: choiceC; Choice { options: parent.setting.options; value: s.valueOf(parent.setting); onPicked: (v) => s.set(parent.setting, v) } }
    Component { id: stepC; Stepper { value: s.valueOf(parent.setting); step: parent.setting.step; from: parent.setting.from; to: parent.setting.to; suffix: parent.setting.suffix; onChanged: (v) => s.set(parent.setting, v) } }
    Component { id: actC; Row { spacing: 6; Repeater { model: parent.setting ? parent.setting.actions : []; delegate: Button1 { required property var modelData; icon: modelData[0]; label: modelData[1]; onClicked: modelData[2]() } } } }

    // ── Keyboard Shortcuts: every command with its keys (the palette's list), searchable; a click runs it ──
    // rebinding: the row being set listens for the next key combination (Esc cancels, Backspace clears)
    property string capturing: ""
    function setKey(name, seq) { const k = Object.assign({}, win.prefs.keys ?? {}); if (seq) k[name] = seq; else delete k[name]; win.setPref("keys", k); capturing = "" }
    Item {
        id: keyCatch
        focus: s.capturing.length > 0
        Keys.onPressed: (e) => {
            if (!s.capturing) return
            e.accepted = true
            if (e.key === Qt.Key_Escape) { s.capturing = ""; return }
            if (e.key === Qt.Key_Backspace && !e.modifiers) { s.setKey(s.capturing, ""); return }
            if ([Qt.Key_Control, Qt.Key_Shift, Qt.Key_Alt, Qt.Key_Meta, Qt.Key_Super_L, Qt.Key_Super_R].includes(e.key)) return
            const mods = (e.modifiers & Qt.ControlModifier ? "Ctrl+" : "") + (e.modifiers & Qt.AltModifier ? "Alt+" : "") + (e.modifiers & Qt.ShiftModifier ? "Shift+" : "") + (e.modifiers & Qt.MetaModifier ? "Meta+" : "")
            const names = { [Qt.Key_Return]: "Return", [Qt.Key_Enter]: "Enter", [Qt.Key_Tab]: "Tab", [Qt.Key_Space]: "Space", [Qt.Key_Delete]: "Delete", [Qt.Key_Up]: "Up", [Qt.Key_Down]: "Down", [Qt.Key_Left]: "Left", [Qt.Key_Right]: "Right", [Qt.Key_Home]: "Home", [Qt.Key_End]: "End", [Qt.Key_PageUp]: "PgUp", [Qt.Key_PageDown]: "PgDown" }
            let k = names[e.key] ?? (e.key >= Qt.Key_F1 && e.key <= Qt.Key_F35 ? "F" + (e.key - Qt.Key_F1 + 1) : String.fromCharCode(e.key).toUpperCase())
            if (!mods && !/^F\d+$/.test(k)) return   // a bare letter would take typing away
            s.setKey(s.capturing, mods + k)
        }
    }
    readonly property var shortcuts: { const q = search.text.trim().toLowerCase(); return s.cat === "Keyboard Shortcuts" ? win.paletteEntries().filter(c => !q || (c.name + " " + c.desc + " " + c.keys).toLowerCase().includes(q)) : [] }
    Flickable {
        visible: !s.jsonView && s.cat === "Keyboard Shortcuts"
        x: searchBox.x; y: searchBox.y + searchBox.height + 16
        width: searchBox.width; height: s.height - y - 16
        clip: true; contentHeight: kCol.implicitHeight + 20
        boundsBehavior: Flickable.StopAtBounds
        Column {
            id: kCol
            width: parent.width
            Repeater {
                model: s.shortcuts
                delegate: Rectangle {
                    required property var modelData
                    width: kCol.width; height: 46; radius: 10
                    color: kH.hovered ? Qt.alpha(t.text, 0.05) : "transparent"
                    Text { id: kI; x: 12; anchors.verticalCenter: parent.verticalCenter; text: modelData.icon ?? "bolt"; color: t.dim; font.family: t.icons; font.pixelSize: 18 }
                    Column { anchors { left: kI.right; leftMargin: 12; right: keyRow.left; rightMargin: 12; verticalCenter: parent.verticalCenter }
                        Text { width: parent.width; text: modelData.name; elide: Text.ElideRight; color: t.text; font.family: t.sans; font.pixelSize: 14 }
                        Text { width: parent.width; text: modelData.desc ?? ""; elide: Text.ElideRight; color: t.faint; font.family: t.sans; font.pixelSize: 11 } }
                    Row {
                        id: keyRow
                        anchors { right: parent.right; rightMargin: 12; verticalCenter: parent.verticalCenter }
                        spacing: 4
                        readonly property string own: (win.prefs.keys ?? {})[modelData.name] ?? ""
                        Repeater {
                            model: s.capturing === modelData.name ? ["press the keys…"] : (keyRow.own ? keyRow.own.split("+") : (modelData.keys ?? "").split(" ")).filter(k => k)
                            delegate: Rectangle { required property string modelData; width: kt.implicitWidth + 12; height: 22; radius: 6; color: Qt.alpha(t.text, 0.06); border.width: 1; border.color: Qt.alpha(t.text, 0.12)
                                Text { id: kt; anchors.centerIn: parent; text: modelData; color: keyRow.own || s.capturing ? t.accent : t.text; font.family: t.mono; font.pixelSize: 11 } }
                        }
                        Text { visible: !modelData.keys && s.capturing !== modelData.name && !keyRow.own; text: "no key"; color: t.faint; font.family: t.sans; font.pixelSize: 11 }
                        // change it: the next key combination pressed
                        Rectangle {
                            width: 24; height: 22; radius: 11
                            color: rbH.hovered ? t.raised : "transparent"
                            Text { anchors.centerIn: parent; text: "edit"; color: t.dim; font.family: t.icons; font.pixelSize: 14 }
                            HoverHandler { id: rbH; cursorShape: Qt.PointingHandCursor }
                            TapHandler { onTapped: { s.capturing = modelData.name; keyCatch.forceActiveFocus() } }
                        }
                    }
                    HoverHandler { id: kH; cursorShape: Qt.PointingHandCursor }
                    TapHandler { onTapped: modelData.run() }
                }
            }
        }
    }

    // the store's list, read again after each install or removal
    property int storeRev: 0
    property string storeNote: ""
    readonly property var storeRows: { storeRev; if (s.cat !== "Extensions") return []; try { return JSON.parse(JSON.parse(app.evalLua("vim.json.encode(zett.store.list())")).result) } catch (e) { return [] } }
    function storeDo(e) {
        const call = e.installed ? "remove" : "install"
        let r; try { r = JSON.parse(JSON.parse(app.evalLua(`vim.json.encode({ zett.store.${call}(${JSON.stringify(e.name)}) })`)).result) } catch (x) { r = [false, String(x)] }
        storeNote = `${e.name} ${r[0] ? (e.installed ? "removed" : "installed") : "failed"}${r[1] ? ": " + r[1] : ""}`
        storeRev++; marketSearch()
    }
    // the marketplace (zett.store.search over store.nvim's database): the search box above searches it on this page
    property int marketRev: 0
    property var marketRows: []
    readonly property var marketInfo: { marketRev; if (s.cat !== "Extensions") return [0, 0]; try { return JSON.parse(JSON.parse(app.evalLua("vim.json.encode({ zett.store.market_info() })")).result) } catch (e) { return [0, 0] } }
    function marketSearch() {
        if (s.cat !== "Extensions" || !marketInfo[0]) { marketRows = []; return }
        let r = []; try { r = JSON.parse(JSON.parse(app.evalLua(`vim.json.encode((zett.store.search(${JSON.stringify(search.text)}, 40)))`)).result) } catch (e) {}
        marketRows = Array.isArray(r) ? r : []
    }
    function marketUpdate() {
        storeNote = "Downloading the plugin list (about 4 MB)…"
        Qt.callLater(() => {
            let r; try { r = JSON.parse(JSON.parse(app.evalLua("vim.json.encode({ zett.store.update() })")).result) } catch (x) { r = [false, String(x)] }
            storeNote = r[0] ? `Plugin list updated: ${r[1]}` : `Could not update the plugin list: ${r[1]}`
            marketRev++; marketSearch()
        })
    }
    Timer { id: marketT; interval: 150; onTriggered: s.marketSearch() }
    // the check settings:Extensions:QUERY types QUERY into the search
    Component.onCompleted: if (app.shotEngine.startsWith("settings:Extensions:")) { search.text = app.shotEngine.slice(20); Qt.callLater(() => { s.marketSearch(); console.log("zett-probe: market " + s.marketInfo[0] + " " + JSON.stringify(s.marketRows.slice(0, 3).map(r => r.repo))) }) }
    Connections { target: search; function onTextChanged() { if (s.cat === "Extensions") marketT.restart() } }
    onCatChanged: if (cat === "Extensions") marketT.restart()
    // ── Extensions (init.lua hooks) and Snippets (snippets.json): lists of what is there, the file to change them ──
    readonly property var extRows: s.cat === "Extensions" ? [].concat(win.hooks.palette.map(h => ({ a: h.name, b: "palette" + (h.keys ? "  ·  " + h.keys : "") })), win.hooks.sidebar.map(h => ({ a: h.name ?? h.label ?? "sidebar item", b: "sidebar" })), win.hooks.status.map(h => ({ a: h.text ?? h.name ?? "status item", b: "status bar" }))) : s.cat === "Snippets" ? Object.keys(win.snippets).map(k => ({ a: k, b: win.snippets[k].split("\n")[0] })) : []
    ColumnLayout {
        visible: !s.jsonView && (s.cat === "Extensions" || s.cat === "Snippets")
        x: searchBox.x; y: searchBox.y + searchBox.height + 16; width: searchBox.width
        spacing: 4
        // the store (zett.store, as :ZettStore): plugins tried in zett, Install / Remove
        Text { visible: s.cat === "Extensions"; text: "STORE"; color: t.faint; font.family: t.sans; font.pixelSize: 11; font.weight: Font.DemiBold; font.letterSpacing: 0.8 }
        Repeater {
            model: s.cat === "Extensions" ? s.storeRows : []
            delegate: Rectangle {
                required property var modelData
                Layout.fillWidth: true; implicitHeight: 52; radius: 14; color: t.card
                RowLayout {
                    anchors { fill: parent; leftMargin: 14; rightMargin: 10 }
                    spacing: 10
                    Text { text: modelData.kind === "wasm" ? "memory" : "extension"; color: t.accent; font.family: t.icons; font.pixelSize: 20 }
                    ColumnLayout {
                        Layout.fillWidth: true; spacing: 1
                        RowLayout { spacing: 8
                            Text { text: modelData.name; color: t.text; font.family: t.sans; font.pixelSize: 14; font.weight: Font.DemiBold }
                            Rectangle { implicitWidth: kT.implicitWidth + 12; implicitHeight: 16; radius: 8; color: Qt.alpha(t.text, 0.08)
                                Text { id: kT; anchors.centerIn: parent; text: modelData.kind === "wasm" ? "WASM" : "Lua"; color: t.dim; font.family: t.mono; font.pixelSize: 10 } } }
                        Text { Layout.fillWidth: true; text: modelData.desc; elide: Text.ElideRight; color: t.faint; font.family: t.sans; font.pixelSize: 12 }
                    }
                    Text { visible: modelData.installed && !modelData.managed; text: "In your config"; color: t.faint; font.family: t.sans; font.pixelSize: 12 }
                    Button1 { visible: !modelData.installed || modelData.managed; icon: modelData.installed ? "delete" : "download"; label: modelData.installed ? "Remove" : "Install"; onClicked: s.storeDo(modelData) }
                }
            }
        }
        Text { visible: s.cat === "Extensions" && s.storeNote.length > 0; Layout.fillWidth: true; wrapMode: Text.Wrap; text: s.storeNote; color: t.accent; font.family: t.sans; font.pixelSize: 12 }
        // the marketplace: store.nvim's database, searched by the box above, most stars first
        Item { visible: s.cat === "Extensions"; implicitHeight: 8; Layout.fillWidth: true }
        RowLayout {
            visible: s.cat === "Extensions"; Layout.fillWidth: true; spacing: 10
            Text { text: "MARKETPLACE"; color: t.faint; font.family: t.sans; font.pixelSize: 11; font.weight: Font.DemiBold; font.letterSpacing: 0.8 }
            Text { Layout.fillWidth: true; text: s.marketInfo[0] ? `${s.marketInfo[0].toLocaleString()} Neovim plugins · list from ${new Date(s.marketInfo[1] * 1000).toLocaleDateString()}` : "store.nvim's list of 7000+ Neovim plugins, not downloaded yet"; elide: Text.ElideRight; color: t.faint; font.family: t.sans; font.pixelSize: 12 }
            Button1 { icon: s.marketInfo[0] ? "refresh" : "download"; label: s.marketInfo[0] ? "Update list" : "Download list"; onClicked: s.marketUpdate() }
        }
        Repeater {
            model: s.cat === "Extensions" ? s.marketRows : []
            delegate: Rectangle {
                required property var modelData
                Layout.fillWidth: true; implicitHeight: 52; radius: 14; color: t.card
                RowLayout {
                    anchors { fill: parent; leftMargin: 14; rightMargin: 10 }
                    spacing: 10
                    Text { text: "extension"; color: t.accent; font.family: t.icons; font.pixelSize: 20 }
                    ColumnLayout {
                        Layout.fillWidth: true; spacing: 1
                        RowLayout { spacing: 8
                            Text { text: modelData.repo; color: t.text; font.family: t.sans; font.pixelSize: 14; font.weight: Font.DemiBold; elide: Text.ElideRight; Layout.maximumWidth: 320 }
                            Text { text: "★ " + modelData.stars.toLocaleString(); color: t.dim; font.family: t.mono; font.pixelSize: 11 } }
                        Text { Layout.fillWidth: true; text: modelData.desc; elide: Text.ElideRight; color: t.faint; font.family: t.sans; font.pixelSize: 12 }
                    }
                    Text { visible: modelData.installed && !modelData.managed; text: "In your config"; color: t.faint; font.family: t.sans; font.pixelSize: 12 }
                    Button1 { visible: !modelData.installed || modelData.managed; icon: modelData.installed ? "delete" : "download"; label: modelData.installed ? "Remove" : "Install"; onClicked: s.storeDo(modelData) }
                }
            }
        }
        Text { visible: s.cat === "Extensions" && s.marketInfo[0] > 0 && !s.marketRows.length; text: "No plugin matches “" + search.text + "”"; color: t.faint; font.family: t.sans; font.pixelSize: 12 }
        Item { visible: s.cat === "Extensions"; implicitHeight: 12; Layout.fillWidth: true }
        Text { Layout.fillWidth: true; wrapMode: Text.Wrap; color: t.faint; font.family: t.sans; font.pixelSize: 12
            text: s.cat === "Extensions" ? "Lua hooks from init.lua: zett.gui.palette, zett.gui.sidebar, zett.gui.status" : "snippets.json: {\"prefix\": \"body\"}; type the prefix in code, then Tab (-e = where the cursor lands)" }
        Button1 { icon: "edit"; label: s.cat === "Extensions" ? "Open init.lua" : "Open snippets.json"; onClicked: { const p = s.cat === "Extensions" ? app.configPath() : app.configPath().replace(/init\.lua$/, "snippets.json"); if (s.cat === "Snippets" && !app.readFile(p).startsWith("{")) app.writeFile(p, "{\n  \"fn\": \"fn $0() {\\n}\"\n}\n"); win.openFile({ path: p, rel: p }) } }
        Text { visible: s.extRows.length === 0; text: "None yet"; color: t.faint; font.family: t.sans; font.pixelSize: 13 }
        Repeater { model: s.extRows; delegate: RowLayout { required property var modelData; Layout.fillWidth: true; spacing: 12
            Text { text: modelData.a; color: t.text; font.family: t.mono; font.pixelSize: 13 }
            Text { Layout.fillWidth: true; text: modelData.b; elide: Text.ElideRight; color: t.faint; font.family: t.sans; font.pixelSize: 12 } } }
    }

    // ── the JSON view: what is stored, as it is ──
    Rectangle {
        visible: s.jsonView
        x: searchBox.x; y: 92
        width: searchBox.width; height: s.height - y - 16
        radius: 14; color: t.card
        RowLayout {
            id: jHead
            anchors { left: parent.left; right: parent.right; top: parent.top; margins: 12 }
            Text { Layout.fillWidth: true; text: "~/.config/zett/gui.json  ·  only what differs from the defaults is worth keeping"; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
            Button1 { icon: "edit"; label: "Open the file"; onClicked: win.openFile({ path: app.configPath().replace(/init\.lua$/, "gui.json"), rel: "gui.json", dir: false }) }
        }
        Flickable {
            anchors { left: parent.left; right: parent.right; top: jHead.bottom; bottom: parent.bottom; margins: 12 }
            clip: true; contentHeight: jText.implicitHeight
            Text { id: jText; width: parent.width; text: app.highlight("json", JSON.stringify(win.prefs, null, 2)); textFormat: Text.RichText; font.family: t.mono; font.pixelSize: 13; color: t.text }
        }
    }
}
