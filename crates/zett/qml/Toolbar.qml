import QtQuick
import QtQuick.Layouts

// Over the page: the open pages as pill tabs (the menu, middle-click closes) and the Markdown switch.
Item {
    required property var win
    required property var t
    required property var app
    Layout.fillWidth: true
    implicitHeight: 48
    // the open pages as pill tabs
    ListView {
        id: tabList
        anchors { left: parent.left; right: toolRow.left; top: parent.top; leftMargin: 12; rightMargin: 12; topMargin: 8 }
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
            // drag a tab sideways to put it elsewhere among the tabs
            DragHandler {
                id: tabDrag
                target: null
                yAxis.enabled: false
                onActiveChanged: if (!active) {
                    const p = parent.mapToItem(tabList.contentItem, parent.width / 2 + translation.x, parent.height / 2)
                    let to = tabList.indexAt(p.x, p.y)
                    if (to < 0) to = translation.x < 0 ? 0 : win.tabs.length - 1
                    const from = win.tabs.indexOf(modelData)
                    if (to !== from && from >= 0) { const t2 = win.tabs.slice(); t2.splice(from, 1); t2.splice(to, 0, modelData); win.tabs = t2 }
                }
            }
            transform: Translate { x: tabDrag.active ? tabDrag.translation.x : 0 }
            z: tabDrag.active ? 10 : 0
            opacity: tabDrag.active ? 0.85 : 1
        }
    }
    // the open files, after the pages: a code icon, the file name; middle-click or × closes
    Row {
        id: fileTabRow
        anchors { left: tabList.left; leftMargin: Math.min(tabList.contentWidth + (win.tabs.length ? 6 : 0), tabList.width); top: tabList.top }
        spacing: 6
        Repeater {
            model: win.fileTabs
            delegate: Rectangle {
                required property string modelData
                readonly property bool sel: win.view === "file" && win.filePath === modelData
                width: Math.min(200, ftRow.implicitWidth + 24); height: 32; radius: 16
                color: sel ? Qt.alpha(t.accent, 0.16) : ftHov.hovered ? t.raised : t.card
                RowLayout {
                    id: ftRow
                    anchors { fill: parent; leftMargin: 12; rightMargin: 8 }
                    spacing: 6
                    Text { text: "code"; color: parent.parent.sel ? t.accent : t.faint; font.family: t.icons; font.pixelSize: 16 }
                    Text { Layout.maximumWidth: 130; text: modelData.slice(modelData.lastIndexOf("/") + 1); elide: Text.ElideMiddle; color: parent.parent.sel ? t.text : t.dim; font.family: t.mono; font.pixelSize: 12; font.weight: parent.parent.sel ? Font.DemiBold : Font.Normal }
                    Text {
                        text: "close"; color: fcHov.hovered ? t.text : t.faint; font.family: t.icons; font.pixelSize: 14
                        HoverHandler { id: fcHov; cursorShape: Qt.PointingHandCursor }
                        TapHandler { onTapped: win.closeFileTab(modelData) }
                    }
                }
                HoverHandler { id: ftHov; cursorShape: Qt.PointingHandCursor }
                TapHandler { onTapped: win.openFile({ path: modelData, rel: modelData }) }
                TapHandler { acceptedButtons: Qt.MiddleButton; onTapped: win.closeFileTab(modelData) }
            }
        }
    }
    RowLayout {
        id: toolRow
        visible: win.view === "page"
        anchors { right: parent.right; top: parent.top; rightMargin: 14; topMargin: 10 }
        spacing: 6
        // the page width: narrow (680) → wide (960) → full; Settings has the exact width
        Rectangle {
            readonly property int w: win.prefs.width ?? 10000
            readonly property string mode: w >= 4000 ? "full" : w >= 900 ? "wide" : "narrow"
            implicitWidth: 34; implicitHeight: 30; radius: 15
            color: wHov.hovered ? t.raised : t.card
            Text { anchors.centerIn: parent; text: ({ narrow: "width_normal", wide: "width_wide", full: "fit_screen" })[parent.mode]; color: t.dim; font.family: t.icons; font.pixelSize: 18 }
            HoverHandler { id: wHov; cursorShape: Qt.PointingHandCursor }
            TapHandler { onTapped: win.setPref("width", ({ narrow: 960, wide: 10000, full: 680 })[parent.mode]) }
        }
        Rectangle {
            implicitWidth: srcRow.implicitWidth + 24; implicitHeight: 30; radius: 15
            color: win.showSource ? t.accent : (srcHov.hovered ? t.raised : t.card)
            RowLayout { id: srcRow; anchors.centerIn: parent; Text { text: "Markdown"; color: win.showSource ? t.onAccent : t.dim; font.family: t.sans; font.pixelSize: 13 } }
            HoverHandler { id: srcHov; cursorShape: Qt.PointingHandCursor }
            TapHandler { onTapped: win.showSource = !win.showSource }
        }
    }
}
