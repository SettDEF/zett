import QtQuick
import QtQuick.Layouts

// The details card beside the page: its outline (a click jumps there), what links here, counts.
Rectangle {
    required property var win
    required property var t
    required property var app
    visible: win.detailsShown && win.pages.length > 0 && win.view === "page" && win.sidePage < 0
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
