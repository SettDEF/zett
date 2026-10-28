import QtQuick
import QtQuick.Layouts

// The command palette (Ctrl+K): every command with its key, searched by name and description.
// entries: [{icon, name, desc, keys, run}]; the window builds them (engine commands + its own).
Item {
    id: p
    required property var theme
    property var entries: []
    property bool opened: false
    property int sel: 0
    anchors.fill: parent
    z: 900
    visible: opened

    // all words of the query in the name or description, the name's start first
    readonly property var shown: {
        const q = field.text.trim().toLowerCase()
        if (!q) return entries.slice(0, 200)
        const words = q.split(/\s+/)
        return entries.map(e => {
            const hay = (e.name + " " + (e.desc ?? "")).toLowerCase()
            if (!words.every(w => hay.includes(w))) return null
            const n = e.name.toLowerCase()
            return { e, score: n.startsWith(q) ? 0 : n.includes(q) ? 1 : 2 }
        }).filter(x => x).sort((a, b) => a.score - b.score).map(x => x.e).slice(0, 200)
    }
    onShownChanged: sel = 0

    function show() { field.text = ""; sel = 0; opened = true; field.forceActiveFocus() }
    function close() { opened = false }
    function run(e) { if (!e) return; close(); e.run() }

    // the dim background closes it
    Rectangle { anchors.fill: parent; color: Qt.alpha("black", 0.25); TapHandler { onTapped: p.close() } }

    Rectangle {
        id: card
        anchors.horizontalCenter: parent.horizontalCenter
        y: Math.max(40, parent.height * 0.12)
        width: Math.min(640, parent.width - 48)
        height: Math.min(col.implicitHeight + 16, parent.height - y - 40)
        radius: 17
        color: p.theme.raised
        border.width: 1; border.color: Qt.alpha(p.theme.rule, 0.5)
        TapHandler { }   // clicks on the card stay on it

        ColumnLayout {
            id: col
            anchors { fill: parent; margins: 8 }
            spacing: 4
            // the query
            Rectangle {
                Layout.fillWidth: true
                implicitHeight: 44; radius: 12
                color: p.theme.card
                RowLayout {
                    anchors { fill: parent; leftMargin: 12; rightMargin: 12 }
                    spacing: 10
                    Text { text: "terminal"; color: p.theme.accent; font.family: p.theme.icons; font.pixelSize: 20 }
                    TextInput {
                        id: field
                        Layout.fillWidth: true
                        color: p.theme.text; font.family: p.theme.sans; font.pixelSize: 15
                        clip: true
                        Text { visible: !parent.text; text: "Run a command…"; color: p.theme.faint; font: parent.font }
                        Keys.onPressed: (e) => {
                            if (e.key === Qt.Key_Escape) { p.close(); e.accepted = true }
                            else if (e.key === Qt.Key_Down || (e.key === Qt.Key_N && e.modifiers & Qt.ControlModifier)) { p.sel = Math.min(p.shown.length - 1, p.sel + 1); list.positionViewAtIndex(p.sel, ListView.Contain); e.accepted = true }
                            else if (e.key === Qt.Key_Up || (e.key === Qt.Key_P && e.modifiers & Qt.ControlModifier)) { p.sel = Math.max(0, p.sel - 1); list.positionViewAtIndex(p.sel, ListView.Contain); e.accepted = true }
                            else if (e.key === Qt.Key_Return || e.key === Qt.Key_Enter) { p.run(p.shown[p.sel]); e.accepted = true }
                        }
                        onActiveFocusChanged: if (!activeFocus && p.opened) p.close()   // focus elsewhere: gone
                    }
                    Text { text: field.text ? `${p.shown.length}` : `${p.entries.length}`; color: p.theme.faint; font.family: p.theme.mono; font.pixelSize: 11 }
                }
            }
            ListView {
                id: list
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.preferredHeight: Math.min(contentHeight, 420)
                clip: true
                model: p.shown
                boundsBehavior: Flickable.StopAtBounds
                delegate: Rectangle {
                    id: row
                    required property var modelData
                    required property int index
                    readonly property bool on: index === p.sel
                    width: ListView.view.width
                    height: 38; radius: 10
                    color: on ? Qt.alpha(p.theme.accent, 0.16) : hov.hovered ? Qt.alpha(p.theme.text, 0.06) : "transparent"
                    RowLayout {
                        anchors { fill: parent; leftMargin: 12; rightMargin: 12 }
                        spacing: 10
                        Text { text: row.modelData.icon ?? "bolt"; color: row.on ? p.theme.accent : p.theme.faint; font.family: p.theme.icons; font.pixelSize: 17 }
                        Text { text: row.modelData.name; color: p.theme.text; font.family: p.theme.sans; font.pixelSize: 13; font.weight: row.on ? Font.DemiBold : Font.Normal }
                        Text { Layout.fillWidth: true; text: row.modelData.desc ?? ""; elide: Text.ElideRight; color: p.theme.faint; font.family: p.theme.sans; font.pixelSize: 12 }
                        // the key, so the palette teaches it
                        Rectangle {
                            visible: !!row.modelData.keys
                            implicitWidth: keyText.implicitWidth + 12; implicitHeight: 22; radius: 6
                            color: p.theme.card
                            Text { id: keyText; anchors.centerIn: parent; text: row.modelData.keys ?? ""; color: p.theme.dim; font.family: p.theme.mono; font.pixelSize: 11 }
                        }
                    }
                    HoverHandler { id: hov; cursorShape: Qt.PointingHandCursor; onHoveredChanged: if (hovered) p.sel = row.index }
                    TapHandler { onTapped: p.run(row.modelData) }
                }
            }
            Text {
                Layout.fillWidth: true
                Layout.leftMargin: 6
                text: "↑↓ choose  ·  Enter run  ·  Esc close  ·  the key on the right runs it directly next time"
                color: p.theme.faint; font.family: p.theme.sans; font.pixelSize: 11
            }
        }
    }
}
