import QtQuick
import QtQuick.Layouts

// Over the page: the open pages as pill tabs (the menu, middle-click closes) and the Markdown switch.
Item {
    required property var win
    required property var t
    required property var app
    Layout.fillWidth: true
    implicitHeight: 64
    // the open pages as pill tabs
    ListView {
        anchors { left: parent.left; right: toolRow.left; top: parent.top; leftMargin: 12; rightMargin: 12; topMargin: 24 }
        height: 34
        orientation: ListView.Horizontal
        spacing: 6
        clip: true
        model: win.tabs
        delegate: Rectangle {
            required property int modelData
            readonly property bool sel: modelData === app.current && win.view === "page"
            readonly property var pg: win.pages[modelData] ?? {}
            width: Math.min(200, tabRow.implicitWidth + 24); height: 32; radius: 16
            color: sel ? Qt.alpha(t.accent, 0.16) : tabHov.hovered ? t.raised : t.card
            RowLayout {
                id: tabRow
                anchors { fill: parent; leftMargin: 12; rightMargin: 8 }
                spacing: 6
                Text { text: "description"; color: parent.parent.sel ? t.accent : t.faint; font.family: t.icons; font.pixelSize: 16 }
                Text { Layout.fillWidth: true; Layout.maximumWidth: 130; text: parent.parent.pg.title || "Untitled"; elide: Text.ElideRight; color: parent.parent.sel ? t.text : t.dim; font.family: t.sans; font.pixelSize: 13; font.weight: parent.parent.sel ? Font.DemiBold : Font.Normal }
                Text {
                    text: "close"; color: closeHov.hovered ? t.text : t.faint; font.family: t.icons; font.pixelSize: 14
                    HoverHandler { id: closeHov; cursorShape: Qt.PointingHandCursor }
                    TapHandler { onTapped: win.closeTab(modelData) }
                }
            }
            HoverHandler { id: tabHov; cursorShape: Qt.PointingHandCursor }
            TapHandler { onTapped: win.openPage(modelData) }
            TapHandler { acceptedButtons: Qt.RightButton; onTapped: (ev) => win.tabMenu(modelData, ev.scenePosition.x, ev.scenePosition.y) }
            TapHandler { acceptedButtons: Qt.MiddleButton; onTapped: win.closeTab(modelData) }
        }
    }
    RowLayout {
        id: toolRow
        visible: win.view === "page"
        anchors { right: parent.right; top: parent.top; rightMargin: 14; topMargin: 28 }
        spacing: 6
        Rectangle {
            implicitWidth: srcRow.implicitWidth + 24; implicitHeight: 30; radius: 15
            color: win.showSource ? t.accent : (srcHov.hovered ? t.raised : t.card)
            RowLayout { id: srcRow; anchors.centerIn: parent; Text { text: "Markdown"; color: win.showSource ? t.onAccent : t.dim; font.family: t.sans; font.pixelSize: 13 } }
            HoverHandler { id: srcHov; cursorShape: Qt.PointingHandCursor }
            TapHandler { onTapped: win.showSource = !win.showSource }
        }
    }
}
