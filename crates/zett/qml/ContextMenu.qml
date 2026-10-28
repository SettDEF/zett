import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// The one context menu: an icon row (Copy, Cut, Paste, Share) on top, the actions for what was clicked below.
// row: [{icon, label, enabled, run}]; items: [{icon, label, key, danger, enabled, run, sub: [items]}] or {sep: true}
Item {
    id: m
    anchors.fill: parent
    z: 1000
    visible: opened
    property bool opened: false
    property real px: 0
    property real py: 0
    required property var theme
    property var row: []
    property var items: []
    property var stack: []               // open submenus, for the back row
    readonly property var shown: stack.length ? stack[stack.length - 1].sub : items
    function close() { opened = false }
    Keys.onEscapePressed: close()
    // a click anywhere outside closes it
    TapHandler { acceptedButtons: Qt.AllButtons; onTapped: m.close() }

    function show(px, py, r, its) {
        row = r; items = its; stack = []
        m.px = px; m.py = py
        opened = true
        forceActiveFocus()
    }
    function run(it) {
        if (it.enabled === false) return
        if (it.sub) { stack = stack.concat([it]); return }
        close()
        it.run?.()
    }

    Rectangle {
        id: card
        // kept inside the window
        x: Math.max(8, Math.min(m.px, m.width - width - 8))
        y: Math.max(8, Math.min(m.py, m.height - height - 8))
        width: 260
        height: col.implicitHeight + 12
        radius: 17
        color: m.theme.raised
        border.width: 1; border.color: Qt.alpha(m.theme.rule, 0.5)
        opacity: m.opened ? 1 : 0
        scale: m.opened ? 1 : 0.96
        Behavior on opacity { NumberAnimation { duration: 110 } }
        Behavior on scale { NumberAnimation { duration: 140; easing.type: Easing.OutCubic } }
        TapHandler { }   // clicks on the card do not close it
    ColumnLayout {
        id: col
        anchors { left: parent.left; right: parent.right; top: parent.top; margins: 6 }
        spacing: 2
        // icon row: the same four in every menu
        RowLayout {
            visible: m.row.length > 0 && m.stack.length === 0
            Layout.fillWidth: true
            Layout.bottomMargin: 4
            spacing: 4
            Repeater {
                model: m.row
                delegate: Rectangle {
                    required property var modelData
                    readonly property bool on: modelData.enabled !== false
                    Layout.fillWidth: true
                    implicitHeight: 54; radius: 12
                    color: on && rowHov.hovered ? Qt.alpha(m.theme.text, 0.08) : "transparent"
                    opacity: on ? 1 : 0.38
                    ColumnLayout {
                        anchors.centerIn: parent
                        spacing: 2
                        Text { Layout.alignment: Qt.AlignHCenter; text: modelData.icon; color: m.theme.accent; font.family: m.theme.icons; font.pixelSize: 20 }
                        Text { Layout.alignment: Qt.AlignHCenter; text: modelData.label; color: m.theme.dim; font.family: m.theme.sans; font.pixelSize: 11 }
                    }
                    HoverHandler { id: rowHov; cursorShape: parent.on ? Qt.PointingHandCursor : Qt.ArrowCursor }
                    TapHandler { onTapped: m.run(modelData) }
                }
            }
        }
        Rectangle { visible: m.row.length > 0 && m.stack.length === 0; Layout.fillWidth: true; Layout.bottomMargin: 4; implicitHeight: 1; color: Qt.alpha(m.theme.rule, 0.6) }

        // back row inside a submenu
        Rectangle {
            visible: m.stack.length > 0
            Layout.fillWidth: true
            implicitHeight: 34; radius: 10
            color: backHov.hovered ? Qt.alpha(m.theme.text, 0.08) : "transparent"
            RowLayout {
                anchors { fill: parent; leftMargin: 10; rightMargin: 10 }
                spacing: 10
                Text { text: "arrow_back"; color: m.theme.dim; font.family: m.theme.icons; font.pixelSize: 18 }
                Text { Layout.fillWidth: true; text: m.stack.length ? m.stack[m.stack.length - 1].label : ""; color: m.theme.text; font.family: m.theme.sans; font.pixelSize: 13; font.weight: Font.DemiBold }
            }
            HoverHandler { id: backHov; cursorShape: Qt.PointingHandCursor }
            TapHandler { onTapped: m.stack = m.stack.slice(0, -1) }
        }

        Repeater {
            model: m.shown
            delegate: Loader {
                required property var modelData
                Layout.fillWidth: true
                sourceComponent: modelData.sep ? sepC : itemC
                property var it: modelData
            }
        }
        Component { id: sepC; Item { implicitHeight: 9; Rectangle { anchors.centerIn: parent; width: parent.width - 8; height: 1; color: Qt.alpha(m.theme.rule, 0.6) } } }
        Component {
            id: itemC
            Rectangle {
                readonly property var it: parent ? parent.it : ({})
                readonly property bool on: it.enabled !== false
                implicitHeight: 34; radius: 10
                color: on && hov.hovered ? (it.danger ? Qt.alpha("#ffb4ab", 0.12) : Qt.alpha(m.theme.text, 0.08)) : "transparent"
                opacity: on ? 1 : 0.38
                RowLayout {
                    anchors { fill: parent; leftMargin: 10; rightMargin: 10 }
                    spacing: 10
                    Text { text: it.icon ?? ""; color: it.danger ? "#ffb4ab" : m.theme.dim; font.family: m.theme.icons; font.pixelSize: 18 }
                    Text { Layout.fillWidth: true; text: it.label ?? ""; elide: Text.ElideRight; color: it.danger ? "#ffb4ab" : m.theme.text; font.family: m.theme.sans; font.pixelSize: 13 }
                    Text { visible: !!it.key; text: it.key ?? ""; color: m.theme.faint; font.family: m.theme.mono; font.pixelSize: 11 }
                    Text { visible: !!it.sub; text: "chevron_right"; color: m.theme.faint; font.family: m.theme.icons; font.pixelSize: 18 }
                }
                HoverHandler { id: hov; cursorShape: parent.on ? Qt.PointingHandCursor : Qt.ArrowCursor }
                TapHandler { onTapped: m.run(it) }
            }
        }
    }
    }
}
