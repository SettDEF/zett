import QtQuick
import QtQuick.Layouts

// A second page beside the one being edited, read-only (the engine edits one page at a time):
// to read one page while writing another. "Edit here" swaps them; links open in the editor.
Rectangle {
    id: side
    required property var win
    required property var t
    required property var app
    property int page: -1
    readonly property var pg: win.pages[page] ?? {}
    readonly property var blocks: { win.pages; win.blocks; try { return JSON.parse(app.pageBlocks(page)) } catch (e) { return [] } }
    radius: t.rCard
    color: t.panel
    border.width: 1; border.color: Qt.alpha(t.rule, 0.35)

    ColumnLayout {
        anchors { fill: parent; margins: 14 }
        spacing: 8
        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            Text { text: "description"; color: t.accent; font.family: t.icons; font.pixelSize: 18 }
            Text { Layout.fillWidth: true; text: side.pg.title || "Untitled"; elide: Text.ElideRight; color: t.text; font.family: t.sans; font.pixelSize: 15; font.weight: Font.DemiBold }
            Text { text: "read-only"; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
            // small round buttons: edit here, close
            Repeater {
                model: [["edit", "Edit here"], ["close", "Close"]]
                delegate: Rectangle {
                    required property var modelData
                    implicitWidth: 28; implicitHeight: 28; radius: 14
                    color: bh.hovered ? t.raised : "transparent"
                    Text { anchors.centerIn: parent; text: modelData[0]; color: bh.hovered ? t.text : t.dim; font.family: t.icons; font.pixelSize: 17 }
                    HoverHandler { id: bh; cursorShape: Qt.PointingHandCursor }
                    TapHandler {
                        onTapped: {
                            if (modelData[0] === "close") { win.sidePage = -1; return }
                            const other = app.current
                            win.openPage(side.page)
                            win.sidePage = other
                        }
                    }
                }
            }
        }
        Flickable {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            contentHeight: body.implicitHeight + 40
            boundsBehavior: Flickable.StopAtBounds
            ColumnLayout {
                id: body
                width: parent.width
                spacing: 6
                Text { visible: side.blocks.length === 0; text: "An empty page"; color: t.faint; font.family: t.sans; font.pixelSize: 13 }
                Repeater {
                    model: side.blocks
                    delegate: Loader {
                        required property var modelData
                        Layout.fillWidth: true
                        sourceComponent: modelData.kind === "code" ? codeC : textC
                        property var b: modelData
                    }
                }
            }
        }
    }
    // a block as text: headings bigger, to-dos with their box, quotes dim; [[links]] open in the editor
    Component {
        id: textC
        Text {
            readonly property var b: parent ? parent.b : ({})
            width: parent ? parent.width : 0
            wrapMode: Text.Wrap
            textFormat: Text.StyledText
            text: (b.kind === "todo" ? (b.checked ? "☑  " : "☐  ") : b.kind === "bullet" ? "•  " : "") + win.linkify(b.text ?? "")
            color: b.kind === "quote" || (b.kind === "todo" && b.checked) ? t.dim : t.text
            linkColor: t.accent
            onLinkActivated: (l) => app.openTitle(decodeURIComponent(l))
            font.family: t.sans
            font.pixelSize: Math.round((b.kind === "heading" ? [0, 22, 18, 16, 15, 14, 14][b.level ?? 1] : 14) * t.scale)
            font.weight: b.kind === "heading" ? Font.DemiBold : Font.Normal
            font.italic: b.kind === "quote"
            topPadding: b.kind === "heading" ? 6 : 0
            HoverHandler { cursorShape: parent.hoveredLink ? Qt.PointingHandCursor : Qt.ArrowCursor }
        }
    }
    Component {
        id: codeC
        Rectangle {
            readonly property var b: parent ? parent.b : ({})
            width: parent ? parent.width : 0
            implicitHeight: code.implicitHeight + 20
            radius: 10
            color: t.card
            Text {
                id: code
                x: 10; y: 10
                width: parent.width - 20
                textFormat: Text.RichText
                text: app.highlight(parent.b.lang ?? "", parent.b.text ?? "")
                color: t.text; font.family: t.mono; font.pixelSize: Math.round(13 * t.scale)
            }
        }
    }
}
