import QtQuick
import QtQuick.Layouts

// The marketplace (Ctrl+Shift+X): store.nvim's database of Neovim plugins, searched as you type, most stars first.
// Categories on top (Tab), the chosen plugin's details on the right. Install adds a zett.plug line to
// plugins/store/init.lua (zett.store); the plugins run in the terminal zett.
Item {
    id: m
    required property var theme
    required property var app
    property bool opened: false
    property int sel: 0
    property int cat: 0
    property var rows: []
    property string note: ""
    property int rev: 0
    anchors.fill: parent
    z: 900
    visible: opened

    // [icon, label, word added to the search ("" none), installed only]
    readonly property var cats: [
        ["apps", "All", "", false], ["check_circle", "Installed", "", true], ["palette", "Themes", "colorscheme", false],
        ["code", "LSP", "lsp", false], ["smart_toy", "AI", "ai", false], ["commit", "Git", "git", false],
        ["account_tree", "Treesitter", "treesitter", false], ["search", "Telescope", "telescope", false],
        ["terminal", "Terminal", "terminal", false], ["description", "Notes", "markdown", false],
        ["view_agenda", "Statusline", "statusline", false], ["auto_awesome", "Completion", "completion", false]]
    readonly property var cur: rows[sel] ?? null

    function lua(expr, fallback) { try { return JSON.parse(JSON.parse(app.evalLua(expr)).result) } catch (e) { return fallback } }
    readonly property var info: { rev; return opened ? lua("vim.json.encode({ zett.store.market_info() })", [0, 0]) : [0, 0] }
    function search() {
        if (!info[0]) { rows = []; return }
        const c = cats[cat], q = (field.text + " " + c[2]).trim()
        const r = lua(`vim.json.encode((zett.store.search(${JSON.stringify(q)}, 80, ${c[3]})))`, [])
        rows = Array.isArray(r) ? r : []
        sel = Math.min(sel, Math.max(0, rows.length - 1))
    }
    function searchFor(q) { field.text = q; search() }
    function setCat(i) { cat = (i + cats.length) % cats.length; sel = 0; search(); list.positionViewAtBeginning() }
    function move(d) { if (!rows.length) return; sel = Math.max(0, Math.min(rows.length - 1, sel + d)); list.positionViewAtIndex(sel, ListView.Contain) }
    function update() {
        note = "Downloading the plugin list (about 4 MB)…"
        Qt.callLater(() => {
            const r = lua("vim.json.encode({ zett.store.update() })", [false, "failed"])
            note = r[0] ? `Plugin list updated: ${r[1]}` : `Could not update the plugin list: ${r[1]}`
            rev++; search()
        })
    }
    function toggle(e) {
        if (!e || (e.installed && !e.managed)) return
        const call = e.installed ? "remove" : "install"
        if (!e.installed) note = `Installing ${e.repo}…`
        Qt.callLater(() => {
            const r = lua(`vim.json.encode({ zett.store.${call}(${JSON.stringify(e.repo)}) })`, [false, "failed"])
            note = `${e.repo} ${r[0] ? (e.installed ? "removed" : "installed: it runs in the terminal zett") : "failed"}${r[1] ? " (" + r[1] + ")" : ""}`
            search()
        })
    }
    function github(e) { if (e) Qt.openUrlExternally("https://github.com/" + e.repo) }
    function show() { field.text = ""; sel = 0; cat = 0; note = ""; opened = true; rev++; search(); field.forceActiveFocus() }
    function close() { opened = false }
    Timer { id: typeT; interval: 120; onTriggered: { m.sel = 0; m.search() } }

    // a key hint: the key in a chip, then what it does
    component Hint: RowLayout {
        property string k
        property string what
        spacing: 4
        Rectangle { implicitWidth: kt.implicitWidth + 10; implicitHeight: 18; radius: 5; color: m.theme.card
            Text { id: kt; anchors.centerIn: parent; text: parent.parent.k; color: m.theme.dim; font.family: m.theme.mono; font.pixelSize: 10 } }
        Text { text: parent.what; color: m.theme.faint; font.family: m.theme.sans; font.pixelSize: 11 }
    }
    // a pill button
    component Pill: Rectangle {
        property string icon
        property string label
        property bool primary: false
        signal hit
        implicitWidth: pr.implicitWidth + 20; implicitHeight: 32; radius: 16
        color: primary ? (ph.hovered ? Qt.lighter(m.theme.accent, 1.1) : m.theme.accent) : (ph.hovered ? m.theme.raised : m.theme.card)
        RowLayout { id: pr; anchors.centerIn: parent; spacing: 6
            Text { text: parent.parent.icon; color: parent.parent.primary ? m.theme.onAccent : m.theme.dim; font.family: m.theme.icons; font.pixelSize: 16 }
            Text { text: parent.parent.label; color: parent.parent.primary ? m.theme.onAccent : m.theme.text; font.family: m.theme.sans; font.pixelSize: 12 } }
        HoverHandler { id: ph; cursorShape: Qt.PointingHandCursor }
        TapHandler { onTapped: parent.hit() }
    }

    // the dim background closes it
    Rectangle { anchors.fill: parent; color: Qt.alpha("black", 0.3); TapHandler { onTapped: m.close() } }

    Rectangle {
        id: card
        anchors.horizontalCenter: parent.horizontalCenter
        y: Math.max(32, parent.height * 0.07)
        width: Math.min(1040, parent.width - 48)
        height: Math.min(640, parent.height - y - 32)
        radius: 17
        color: m.theme.raised
        border.width: 1; border.color: Qt.alpha(m.theme.rule, 0.5)
        TapHandler { }   // clicks on the card stay on it

        ColumnLayout {
            anchors { fill: parent; margins: 8 }
            spacing: 6
            // the query, the count, Update list
            Rectangle {
                Layout.fillWidth: true
                implicitHeight: 44; radius: 12
                color: m.theme.card
                RowLayout {
                    anchors { fill: parent; leftMargin: 12; rightMargin: 8 }
                    spacing: 10
                    Text { text: "storefront"; color: m.theme.accent; font.family: m.theme.icons; font.pixelSize: 20 }
                    TextInput {
                        id: field
                        Layout.fillWidth: true
                        color: m.theme.text; font.family: m.theme.sans; font.pixelSize: 15
                        clip: true
                        onTextChanged: typeT.restart()
                        Text { visible: !parent.text; text: m.info[0] ? `Search ${m.info[0].toLocaleString()} Neovim plugins…` : "Marketplace: download the plugin list first (Ctrl R)"; color: m.theme.faint; font: parent.font }
                        Keys.onPressed: (e) => {
                            const ctrl = e.modifiers & Qt.ControlModifier
                            if (e.key === Qt.Key_Escape) { if (field.text) field.text = ""; else m.close(); e.accepted = true }
                            else if (e.key === Qt.Key_Down || (e.key === Qt.Key_N && ctrl) || (e.key === Qt.Key_J && ctrl)) { m.move(1); e.accepted = true }
                            else if (e.key === Qt.Key_Up || (e.key === Qt.Key_P && ctrl) || (e.key === Qt.Key_K && ctrl)) { m.move(-1); e.accepted = true }
                            else if (e.key === Qt.Key_PageDown) { m.move(8); e.accepted = true }
                            else if (e.key === Qt.Key_PageUp) { m.move(-8); e.accepted = true }
                            else if (e.key === Qt.Key_Home && ctrl) { m.move(-m.rows.length); e.accepted = true }
                            else if (e.key === Qt.Key_End && ctrl) { m.move(m.rows.length); e.accepted = true }
                            else if (e.key === Qt.Key_Tab) { m.setCat(m.cat + 1); e.accepted = true }
                            else if (e.key === Qt.Key_Backtab) { m.setCat(m.cat - 1); e.accepted = true }
                            else if (ctrl && e.key >= Qt.Key_1 && e.key <= Qt.Key_9) { m.setCat(e.key - Qt.Key_1); e.accepted = true }
                            else if (e.key === Qt.Key_Return || e.key === Qt.Key_Enter) { m.toggle(m.cur); e.accepted = true }
                            else if (ctrl && e.key === Qt.Key_O) { m.github(m.cur); e.accepted = true }
                            else if (ctrl && e.key === Qt.Key_R) { m.update(); e.accepted = true }
                        }
                    }
                    Text { text: m.rows.length ? `${m.rows.length}${m.rows.length >= 80 ? "+" : ""}` : ""; color: m.theme.faint; font.family: m.theme.mono; font.pixelSize: 11 }
                    Pill { icon: m.info[0] ? "refresh" : "download"; label: m.info[0] ? "Update list" : "Download list"; primary: !m.info[0]; implicitHeight: 30; onHit: m.update() }
                    Pill { icon: "close"; label: ""; implicitWidth: 30; implicitHeight: 30; onHit: m.close() }
                }
            }
            // categories: icon chips (Tab / Shift+Tab, Ctrl+1…9)
            Flickable {
                Layout.fillWidth: true; implicitHeight: 34
                contentWidth: chipRow.implicitWidth; clip: true
                boundsBehavior: Flickable.StopAtBounds
                Row {
                    id: chipRow
                    spacing: 6
                    Repeater {
                        model: m.cats
                        delegate: Rectangle {
                            required property var modelData
                            required property int index
                            readonly property bool on: index === m.cat
                            implicitWidth: cr.implicitWidth + 20; implicitHeight: 32; radius: 16
                            color: on ? m.theme.accent : (ch.hovered ? m.theme.card : "transparent")
                            border.width: on ? 0 : 1; border.color: Qt.alpha(m.theme.rule, 0.5)
                            Row { id: cr; anchors.centerIn: parent; spacing: 6
                                Text { anchors.verticalCenter: parent.verticalCenter; text: modelData[0]; color: on ? m.theme.onAccent : m.theme.dim; font.family: m.theme.icons; font.pixelSize: 16 }
                                Text { anchors.verticalCenter: parent.verticalCenter; text: modelData[1]; color: on ? m.theme.onAccent : m.theme.text; font.family: m.theme.sans; font.pixelSize: 12 } }
                            HoverHandler { id: ch; cursorShape: Qt.PointingHandCursor }
                            TapHandler { onTapped: { m.setCat(index); field.forceActiveFocus() } }
                        }
                    }
                }
            }
            // the list, and the chosen plugin beside it
            RowLayout {
                Layout.fillWidth: true; Layout.fillHeight: true
                spacing: 8
                ListView {
                    id: list
                    Layout.fillWidth: true; Layout.fillHeight: true
                    clip: true
                    model: m.rows
                    boundsBehavior: Flickable.StopAtBounds
                    delegate: Rectangle {
                        id: row
                        required property var modelData
                        required property int index
                        readonly property bool on: index === m.sel
                        width: ListView.view.width
                        height: 50; radius: 10
                        color: on ? Qt.alpha(m.theme.accent, 0.16) : hov.hovered ? Qt.alpha(m.theme.text, 0.06) : "transparent"
                        RowLayout {
                            anchors { fill: parent; leftMargin: 12; rightMargin: 12 }
                            spacing: 10
                            Text { text: row.modelData.installed ? "check_circle" : "extension"; color: row.modelData.installed ? m.theme.accent : row.on ? m.theme.text : m.theme.faint; font.family: m.theme.icons; font.pixelSize: 18 }
                            ColumnLayout {
                                Layout.fillWidth: true; spacing: 1
                                Text { Layout.fillWidth: true; text: row.modelData.repo; color: m.theme.text; font.family: m.theme.sans; font.pixelSize: 13; font.weight: Font.DemiBold; elide: Text.ElideRight }
                                Text { Layout.fillWidth: true; text: row.modelData.desc; elide: Text.ElideRight; color: m.theme.faint; font.family: m.theme.sans; font.pixelSize: 12 }
                            }
                            Text { text: "★ " + row.modelData.stars.toLocaleString(); color: m.theme.dim; font.family: m.theme.mono; font.pixelSize: 11 }
                        }
                        HoverHandler { id: hov; cursorShape: Qt.PointingHandCursor }
                        TapHandler { onTapped: { m.sel = row.index; field.forceActiveFocus() } onDoubleTapped: m.toggle(row.modelData) }
                    }
                    Text {
                        anchors.centerIn: parent
                        visible: m.rows.length === 0
                        text: !m.info[0] ? "No plugin list yet: Download list (Ctrl R)" : m.cats[m.cat][3] ? "Nothing installed from the marketplace yet" : `No plugin matches “${field.text}”`
                        color: m.theme.faint; font.family: m.theme.sans; font.pixelSize: 13
                    }
                }
                // the details
                Rectangle {
                    Layout.preferredWidth: 300; Layout.fillHeight: true
                    radius: 14; color: m.theme.card
                    visible: m.cur !== null
                    ColumnLayout {
                        anchors { fill: parent; margins: 16 }
                        spacing: 10
                        Rectangle {
                            implicitWidth: 52; implicitHeight: 52; radius: 16
                            color: Qt.alpha(m.theme.accent, 0.18)
                            Text { anchors.centerIn: parent; text: m.cur ? m.cur.repo.split("/")[1].replace(/^n?vim[-.]?/i, "").charAt(0).toUpperCase() : ""; color: m.theme.accent; font.family: m.theme.sans; font.pixelSize: 24; font.weight: Font.Bold }
                        }
                        ColumnLayout { spacing: 2; Layout.fillWidth: true
                            Text { Layout.fillWidth: true; text: m.cur ? m.cur.repo.split("/")[1] : ""; color: m.theme.text; font.family: m.theme.sans; font.pixelSize: 17; font.weight: Font.DemiBold; elide: Text.ElideRight }
                            Text { Layout.fillWidth: true; text: m.cur ? "by " + m.cur.repo.split("/")[0] : ""; color: m.theme.faint; font.family: m.theme.sans; font.pixelSize: 12; elide: Text.ElideRight } }
                        // stars, last update
                        RowLayout { spacing: 14
                            Row { spacing: 4; Text { text: "star"; color: m.theme.accent; font.family: m.theme.icons; font.pixelSize: 15 } Text { text: m.cur ? m.cur.stars.toLocaleString() : ""; color: m.theme.text; font.family: m.theme.mono; font.pixelSize: 12 } }
                            Row { spacing: 4; visible: !!(m.cur && m.cur.updated); Text { text: "update"; color: m.theme.dim; font.family: m.theme.icons; font.pixelSize: 15 } Text { text: m.cur ? m.cur.updated : ""; color: m.theme.dim; font.family: m.theme.mono; font.pixelSize: 12 } } }
                        Text { Layout.fillWidth: true; text: m.cur ? m.cur.desc : ""; wrapMode: Text.Wrap; maximumLineCount: 6; elide: Text.ElideRight; color: m.theme.text; font.family: m.theme.sans; font.pixelSize: 13; lineHeight: 1.2 }
                        Flow {
                            Layout.fillWidth: true; spacing: 5
                            Repeater {
                                model: m.cur ? m.cur.tags.slice(0, 8) : []
                                delegate: Rectangle { required property var modelData; implicitWidth: tg.implicitWidth + 12; implicitHeight: 20; radius: 10; color: Qt.alpha(m.theme.text, 0.07)
                                    Text { id: tg; anchors.centerIn: parent; text: modelData; color: m.theme.dim; font.family: m.theme.sans; font.pixelSize: 11 } }
                            }
                        }
                        Item { Layout.fillHeight: true }
                        Text { visible: !!(m.cur && m.cur.installed && !m.cur.managed); Layout.fillWidth: true; wrapMode: Text.Wrap; text: "Set up in your own config: take its zett.plug line out there to remove it"; color: m.theme.faint; font.family: m.theme.sans; font.pixelSize: 11 }
                        RowLayout { spacing: 8
                            Pill { visible: !!(m.cur && (!m.cur.installed || m.cur.managed)); icon: m.cur && m.cur.installed ? "delete" : "download"; label: m.cur && m.cur.installed ? "Remove" : "Install"; primary: !!(m.cur && !m.cur.installed); onHit: m.toggle(m.cur) }
                            Pill { icon: "open_in_new"; label: "GitHub"; onHit: m.github(m.cur) } }
                        Text { Layout.fillWidth: true; wrapMode: Text.Wrap; text: "Runs in the terminal zett (zett.plug in plugins/store/init.lua)"; color: m.theme.faint; font.family: m.theme.sans; font.pixelSize: 11 }
                    }
                }
            }
            // keys, or what just happened
            RowLayout {
                Layout.fillWidth: true; Layout.leftMargin: 6; Layout.rightMargin: 6
                spacing: 14
                Text { visible: m.note.length > 0; Layout.fillWidth: true; text: m.note; elide: Text.ElideRight; color: m.theme.accent; font.family: m.theme.sans; font.pixelSize: 11 }
                Hint { visible: !m.note; k: "↑↓"; what: "choose" }
                Hint { visible: !m.note; k: "Enter"; what: "install / remove" }
                Hint { visible: !m.note; k: "Tab"; what: "category" }
                Hint { visible: !m.note; k: "Ctrl O"; what: "GitHub" }
                Hint { visible: !m.note; k: "Ctrl R"; what: "update list" }
                Hint { visible: !m.note; k: "Esc"; what: "clear / close" }
                Item { Layout.fillWidth: true; visible: !m.note }
                Text { visible: m.info[0] > 0; text: `list from ${new Date(m.info[1] * 1000).toLocaleDateString()} · store.nvim`; color: m.theme.faint; font.family: m.theme.sans; font.pixelSize: 11 }
            }
        }
    }
}
