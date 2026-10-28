import QtQuick
import QtQuick.Layouts

// Search's Files tab: the query in the workspace's files (grepFiles: not the pages), case / whole word /
// regex, Replace all (replaceFiles), the hits grouped by file; a click opens the file at that line.
ColumnLayout {
    id: sf
    required property var win
    required property var t
    required property var app
    property string query: ""
    property bool caseSens: false
    property bool word: false
    property bool regex: false
    property var hits: []
    property string note: ""
    spacing: 10
    readonly property string opts: JSON.stringify({ case: caseSens, word, regex })
    function run() { try { hits = query ? JSON.parse(app.grepFiles(query, opts)) : [] } catch (e) { hits = [] } }
    onQueryChanged: later.restart()
    onOptsChanged: later.restart()
    Timer { id: later; interval: 200; onTriggered: sf.run() }
    readonly property var groups: {
        const g = []
        for (const h of hits) { if (!g.length || g[g.length - 1].rel !== h.rel) g.push({ rel: h.rel, path: h.path, rows: [] }); g[g.length - 1].rows.push(h) }
        return g
    }
    // the matched part of a line lit
    function mark(h) {
        const esc = x => x.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
        const s = h.text, a = h.col, b = h.col + h.len
        const r = esc(s.slice(0, a)).replace(/^\s+/, "") + `<b><font color="${t.accent}">${esc(s.slice(a, b))}</font></b>` + esc(s.slice(b))
        return r
    }

    component Toggle: Rectangle {
        property string label
        property bool on
        signal flipped()
        implicitWidth: 34; implicitHeight: 30; radius: 10
        color: on ? Qt.alpha(sf.t.accent, 0.22) : th.hovered ? Qt.alpha(sf.t.text, 0.08) : sf.t.card
        Text { anchors.centerIn: parent; text: parent.label; color: parent.on ? sf.t.accent : sf.t.dim; font.family: sf.t.mono; font.pixelSize: 12 }
        HoverHandler { id: th; cursorShape: Qt.PointingHandCursor }
        TapHandler { onTapped: parent.flipped() }
    }

    RowLayout {
        Layout.fillWidth: true; Layout.leftMargin: 24; Layout.rightMargin: 24
        spacing: 6
        Toggle { label: "Aa"; on: sf.caseSens; onFlipped: sf.caseSens = !sf.caseSens }
        Toggle { label: "ab"; on: sf.word; onFlipped: sf.word = !sf.word }
        Toggle { label: ".*"; on: sf.regex; onFlipped: sf.regex = !sf.regex }
        // replace with, then Replace all
        Rectangle {
            Layout.fillWidth: true
            implicitHeight: 34; radius: 12; color: t.card
            border.width: rep.activeFocus ? 1 : 0; border.color: Qt.alpha(t.accent, 0.5)
            Text { id: rIc; x: 10; anchors.verticalCenter: parent.verticalCenter; text: "find_replace"; color: t.dim; font.family: t.icons; font.pixelSize: 17 }
            TextInput {
                id: rep
                anchors { left: rIc.right; right: parent.right; verticalCenter: parent.verticalCenter; leftMargin: 8; rightMargin: 10 }
                color: t.text; font.family: t.mono; font.pixelSize: 13; clip: true
                Text { visible: !rep.text; text: sf.regex ? "Replace with ($1 for a group)" : "Replace with"; color: t.faint; font: rep.font }
            }
        }
        Rectangle {
            implicitWidth: raRow.implicitWidth + 22; implicitHeight: 34; radius: 17
            enabled: sf.hits.length > 0
            opacity: enabled ? 1 : 0.5
            color: raH.hovered ? Qt.lighter(t.accent, 1.08) : t.accent
            Row { id: raRow; anchors.centerIn: parent; spacing: 6
                Text { anchors.verticalCenter: parent.verticalCenter; text: "done_all"; color: t.onAccent; font.family: t.icons; font.pixelSize: 16 }
                Text { anchors.verticalCenter: parent.verticalCenter; text: "Replace all"; color: t.onAccent; font.family: t.sans; font.pixelSize: 13; font.weight: Font.DemiBold } }
            HoverHandler { id: raH; cursorShape: Qt.PointingHandCursor }
            TapHandler { onTapped: { const n = sf.app.replaceFiles(sf.query, rep.text, sf.opts, "[]"); sf.note = `${n} replaced in ${sf.groups.length} files`; sf.run() } }
        }
    }
    Text {
        Layout.leftMargin: 28
        text: sf.note || (!sf.query ? "Search the workspace's files (pages are in the Pages tab)" : sf.hits.length ? `${sf.hits.length}${sf.hits.length >= 2000 ? "+" : ""} results in ${sf.groups.length} files` : "No results")
        color: sf.note ? t.accent : t.faint; font.family: t.sans; font.pixelSize: 12
    }
    Flickable {
        Layout.fillWidth: true; Layout.fillHeight: true
        Layout.leftMargin: 20; Layout.rightMargin: 20
        clip: true
        contentHeight: gcol.implicitHeight + 20
        boundsBehavior: Flickable.StopAtBounds
        Column {
            id: gcol
            width: parent.width
            spacing: 10
            Repeater {
                model: sf.groups
                delegate: Column {
                    id: grp
                    required property var modelData
                    width: gcol.width
                    spacing: 2
                    Row {
                        x: 6; spacing: 8; height: 28
                        Text { anchors.verticalCenter: parent.verticalCenter; text: "draft"; color: t.accent; font.family: t.icons; font.pixelSize: 16 }
                        Text { anchors.verticalCenter: parent.verticalCenter; text: grp.modelData.rel; color: t.text; font.family: t.sans; font.pixelSize: 14; font.weight: Font.DemiBold }
                        Text { anchors.verticalCenter: parent.verticalCenter; text: grp.modelData.rows.length; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
                    }
                    Repeater {
                        model: grp.modelData.rows.slice(0, 50)
                        delegate: Rectangle {
                            required property var modelData
                            width: gcol.width; height: 28; radius: 8
                            color: hH.hovered ? Qt.alpha(t.text, 0.06) : "transparent"
                            Text { id: ln; x: 30; width: 44; anchors.verticalCenter: parent.verticalCenter; horizontalAlignment: Text.AlignRight; text: modelData.line + 1; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
                            Text { anchors { left: ln.right; leftMargin: 12; right: parent.right; rightMargin: 10; verticalCenter: parent.verticalCenter } text: sf.mark(modelData); textFormat: Text.StyledText; elide: Text.ElideRight; color: t.dim; font.family: t.mono; font.pixelSize: 12 }
                            HoverHandler { id: hH; cursorShape: Qt.PointingHandCursor }
                            TapHandler { onTapped: sf.win.openFileAt({ path: modelData.path, rel: modelData.rel }, modelData.line, modelData.col) }
                        }
                    }
                }
            }
        }
    }
}
