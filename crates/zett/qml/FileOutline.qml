import QtQuick
import QtQuick.Layouts

// The details card for a file (Ctrl+\ toggles it on pages; in files it sits beside the split): its outline from the
// declaration lines (fn, impl, struct, class, def …), indented by depth, the one at the cursor lit; a click jumps.
Rectangle {
    id: fo
    required property var win
    required property var t
    readonly property var fe: win.fileEditor
    visible: win.detailsShown && win.view === "file" && !win.sideFile && win.width >= (win.claudeShown ? 1600 : 900) && !win.zen   // beside Claude only when there is room
    Layout.fillHeight: true
    Layout.topMargin: 8; Layout.rightMargin: 8; Layout.bottomMargin: 8
    implicitWidth: 248; Layout.minimumWidth: 248; Layout.maximumWidth: 248
    radius: t.rCard
    color: t.panel
    border.width: 1; border.color: Qt.alpha(t.rule, 0.35)
    readonly property var items: visible && fe ? fe.outline : []
    readonly property int cur: { if (!fe) return -1; let c = -1; for (const it of items) if (it.line <= fe.curLine) c = it.line; return c }
    ColumnLayout {
        anchors { fill: parent; margins: 12 }
        spacing: 2
        Text { text: "OUTLINE  " + fo.items.length; color: t.faint; font.family: t.sans; font.pixelSize: 11; font.weight: Font.DemiBold; font.letterSpacing: 0.8; Layout.leftMargin: 8; Layout.topMargin: 4 }
        Text { visible: !fo.items.length; text: "No declarations found"; color: t.faint; font.family: t.sans; font.pixelSize: 12; Layout.leftMargin: 8 }
        ListView {
            Layout.fillWidth: true; Layout.fillHeight: true
            clip: true; boundsBehavior: Flickable.StopAtBounds
            model: fo.items
            delegate: Rectangle {
                required property var modelData
                readonly property bool on: modelData.line === fo.cur
                width: ListView.view.width; height: 28; radius: t.rRow
                color: on ? Qt.alpha(t.accent, 0.14) : oh.hovered ? Qt.alpha(t.text, 0.06) : "transparent"
                Row {
                    x: 8 + Math.min(modelData.depth, 6) * 12; anchors.verticalCenter: parent.verticalCenter; spacing: 6
                    // the kind as a small soft pill
                    Rectangle {
                        width: kT.implicitWidth + 10; height: 16; radius: 8; anchors.verticalCenter: parent.verticalCenter
                        color: Qt.alpha(/^(fn|def|function|const fn)$/.test(modelData.kind) ? t.accent : t.added, 0.16)
                        Text { id: kT; anchors.centerIn: parent; text: modelData.kind; color: /^(fn|def|function|const fn)$/.test(modelData.kind) ? t.accent : t.added; font.family: t.mono; font.pixelSize: 9 }
                    }
                    Text { text: modelData.name; color: parent.parent.on ? t.text : t.dim; font.family: t.mono; font.pixelSize: 12; font.weight: parent.parent.on ? Font.DemiBold : Font.Normal; elide: Text.ElideRight; width: fo.width - 90 - Math.min(modelData.depth, 6) * 12 }
                }
                HoverHandler { id: oh; cursorShape: Qt.PointingHandCursor }
                TapHandler { onTapped: fo.fe.goTo(modelData.line, 0) }
            }
        }
    }
}
