import QtQuick
import QtQuick.Layouts

// The details card beside the page: its outline (a click jumps there), what links here, counts.
Rectangle {
    required property var win
    required property var t
    required property var app
    // narrow windows leave it out rather than cut it off
    visible: win.detailsShown && win.pages.length > 0 && win.view === "page" && win.sidePage < 0 && win.width >= 900
    Layout.fillHeight: true
    Layout.topMargin: 8; Layout.rightMargin: 8; Layout.bottomMargin: 8
    implicitWidth: 248
    Layout.minimumWidth: 248
    Layout.maximumWidth: 248
    radius: t.rCard
    color: t.panel
    border.width: 1; border.color: Qt.alpha(t.rule, 0.35)
    readonly property var heads: win.blocks.map((b, i) => ({ i, b })).filter(o => /^(h[1-3]|heading)$/.test(o.b.kind) && (o.b.text ?? "").length)
    readonly property int curHead: { let c = -1; for (const h of heads) if (h.i <= win.topBlock) c = h.i; return c >= 0 ? c : (heads.length ? heads[0].i : -1) }
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
                // the section the page is scrolled to: the last heading at or above the top block
                readonly property bool cur: modelData.i === parent.parent.curHead
                color: cur ? Qt.alpha(t.accent, 0.12) : olHov.hovered ? Qt.alpha(t.text, 0.06) : "transparent"
                Rectangle { visible: parent.cur; x: 0; anchors.verticalCenter: parent.verticalCenter; width: 3; height: 16; radius: 1.5; color: t.accent }
                Text {
                    anchors { fill: parent; leftMargin: 10 + ((modelData.b.level ?? 1) - 1) * 12; rightMargin: 8 }
                    verticalAlignment: Text.AlignVCenter
                    text: modelData.b.text; elide: Text.ElideRight
                    color: parent.cur ? t.accent : t.dim; font.family: t.sans; font.pixelSize: (modelData.b.level ?? 1) === 1 ? 13 : 12
                    font.weight: parent.cur ? Font.DemiBold : Font.Normal
                }
                HoverHandler { id: olHov; cursorShape: Qt.PointingHandCursor }
                TapHandler { onTapped: win.jumpToBlock(modelData.i) }
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
