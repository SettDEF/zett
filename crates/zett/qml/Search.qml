import QtQuick
import QtQuick.Layouts

// Search everything (Ctrl+Shift+F): every block of every page with all the words, grouped by page, the
// words marked; Enter or a click opens the page at that block. Searches as you type (150 ms after a key).
ColumnLayout {
    id: s
    required property var win
    required property var t
    required property var app
    spacing: 10
    property var hits: []
    property int sel: 0
    // the hits grouped by page, in order: [{page, title, rows: [hit]}]
    readonly property var groups: {
        const g = []
        for (const h of hits) {
            if (!g.length || g[g.length - 1].page !== h.page) g.push({ page: h.page, title: h.title, rows: [] })
            if (!h.titleHit) g[g.length - 1].rows.push(h)
        }
        return g
    }
    readonly property var flatHits: hits.filter(h => !h.titleHit || !hits.some(x => x.page === h.page && !x.titleHit))
    function focusField() { field.forceActiveFocus(); field.selectAll() }
    function open(h) { if (!h) return; win.openPage(h.page); if (h.block >= 0) win.jumpToBlock(h.block) }
    Timer { id: later; interval: 150; onTriggered: { try { s.hits = JSON.parse(s.app.search(field.text)) } catch (e) { s.hits = [] } s.sel = 0 } }
    function query(q) { field.text = q; later.stop(); try { s.hits = JSON.parse(s.app.search(q)) } catch (e) { s.hits = [] } s.sel = 0 }

    // the words of the query marked in a block's text
    function mark(text) {
        const esc = x => x.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
        let h = esc(text.length > 240 ? text.slice(0, 240) + "…" : text).replace(/\n/g, " · ")
        for (const w of field.text.trim().split(/\s+/).filter(x => x)) {
            const re = new RegExp("(" + esc(w).replace(/[.*+?^${}()|[\]\\]/g, "\\$&") + ")", "gi")
            h = h.replace(re, `<b><font color="${t.accent}">$1</font></b>`)
        }
        return h
    }

    // the query
    Rectangle {
        Layout.fillWidth: true
        Layout.leftMargin: 24; Layout.rightMargin: 24; Layout.topMargin: 8
        implicitHeight: 46; radius: 23
        color: t.card
        border.width: field.activeFocus ? 1 : 0; border.color: Qt.alpha(t.accent, 0.6)
        RowLayout {
            anchors { fill: parent; leftMargin: 16; rightMargin: 16 }
            spacing: 10
            Text { text: "manage_search"; color: t.accent; font.family: t.icons; font.pixelSize: 22 }
            TextInput {
                id: field
                Layout.fillWidth: true
                color: t.text; font.family: t.sans; font.pixelSize: 16
                clip: true
                onTextChanged: later.restart()
                Text { visible: !parent.text; text: "Search every page…"; color: t.faint; font: parent.font }
                Keys.onPressed: (e) => {
                    const n = s.flatHits.length
                    if (e.key === Qt.Key_Down) { s.sel = Math.min(n - 1, s.sel + 1); e.accepted = true }
                    else if (e.key === Qt.Key_Up) { s.sel = Math.max(0, s.sel - 1); e.accepted = true }
                    else if (e.key === Qt.Key_Return || e.key === Qt.Key_Enter) { s.open(s.flatHits[s.sel]); e.accepted = true }
                    else if (e.key === Qt.Key_Escape) { s.win.view = "page"; e.accepted = true }
                }
            }
            Text { text: field.text ? `${s.flatHits.length}${s.hits.length >= 300 ? "+" : ""} in ${s.groups.length} pages` : ""; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
        }
    }

    // the hits, by page
    Flickable {
        Layout.fillWidth: true
        Layout.fillHeight: true
        clip: true
        contentHeight: res.implicitHeight + 40
        boundsBehavior: Flickable.StopAtBounds
        ColumnLayout {
            id: res
            x: 24; width: parent.width - 48
            spacing: 14
            Text { visible: !!field.text && s.hits.length === 0; text: "Nothing has all of these words."; color: t.faint; font.family: t.sans; font.pixelSize: 13 }
            Text { visible: !field.text; text: "All words must appear in a block (in any order); a page whose title has them is listed too."; color: t.faint; font.family: t.sans; font.pixelSize: 13 }
            Repeater {
                model: s.groups
                delegate: Rectangle {
                    id: grp
                    required property var modelData
                    Layout.fillWidth: true
                    implicitHeight: groupCol.implicitHeight + 20
                    radius: 17
                    color: t.card
                    ColumnLayout {
                        id: groupCol
                        anchors { left: parent.left; right: parent.right; top: parent.top; margins: 10 }
                        spacing: 2
                        // the page
                        Rectangle {
                            Layout.fillWidth: true
                            implicitHeight: 32; radius: 10
                            color: phov.hovered ? Qt.alpha(t.text, 0.06) : "transparent"
                            RowLayout {
                                anchors { fill: parent; leftMargin: 8; rightMargin: 8 }
                                spacing: 8
                                Text { text: "description"; color: t.accent; font.family: t.icons; font.pixelSize: 17 }
                                Text { Layout.fillWidth: true; textFormat: Text.StyledText; text: s.mark(grp.modelData.title || "Untitled"); elide: Text.ElideRight; color: t.text; font.family: t.sans; font.pixelSize: 14; font.weight: Font.DemiBold }
                                Text { text: grp.modelData.rows.length ? `${grp.modelData.rows.length}` : "title"; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
                            }
                            HoverHandler { id: phov; cursorShape: Qt.PointingHandCursor }
                            TapHandler { onTapped: s.open({ page: grp.modelData.page, block: -1 }) }
                        }
                        // its blocks with the words
                        Repeater {
                            model: grp.modelData.rows
                            delegate: Rectangle {
                                id: row
                                required property var modelData
                                readonly property bool on: s.flatHits[s.sel] === modelData
                                Layout.fillWidth: true
                                implicitHeight: rt.implicitHeight + 12; radius: 10
                                color: on ? Qt.alpha(t.accent, 0.16) : rhov.hovered ? Qt.alpha(t.text, 0.06) : "transparent"
                                RowLayout {
                                    anchors { fill: parent; leftMargin: 34; rightMargin: 8 }
                                    spacing: 8
                                    Text { text: ({ heading: "title", todo: "check_box_outline_blank", bullet: "fiber_manual_record", quote: "format_quote", code: "code" })[row.modelData.kind] ?? "notes"; color: t.faint; font.family: t.icons; font.pixelSize: 14 }
                                    Text { id: rt; Layout.fillWidth: true; textFormat: Text.StyledText; wrapMode: Text.Wrap; maximumLineCount: 3; elide: Text.ElideRight
                                        text: s.mark(row.modelData.text); color: t.dim; font.family: row.modelData.kind === "code" ? t.mono : t.sans; font.pixelSize: 13 }
                                }
                                HoverHandler { id: rhov; cursorShape: Qt.PointingHandCursor }
                                TapHandler { onTapped: s.open(row.modelData) }
                            }
                        }
                    }
                }
            }
        }
    }
}
