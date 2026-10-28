import QtQuick
import QtQuick.Layouts

// The activity bar, left of the sidebar as in VS Code: what the sidebar shows (pages, files, both) and the
// views (search, today, outline, Claude); settings at the foot. The active one sits in a soft pill.
Rectangle {
    id: bar
    required property var win
    required property var t
    Layout.fillHeight: true
    implicitWidth: 52
    visible: !win.zen
    property bool tipTest: false   // gui-check: one label shown without a mouse
    z: 20   // its hover labels reach over the sidebar beside it, which comes later and would paint over them
    color: "transparent"

    function pick(mode) { win.sideMode = win.sideMode === mode ? "all" : mode; win.setPref("sideMode", win.sideMode); win.sidebarHidden = false }
    readonly property var items: [
        { icon: "description", tip: "Pages", on: win.sideMode === "pages", run: () => pick("pages") },
        { icon: "folder_open", tip: "Files", on: win.sideMode === "files", run: () => pick("files") },
        { icon: "commit", tip: "Source Control  Ctrl ⇧ G", on: win.sideMode === "git", run: () => pick("git") },
        { icon: "search", tip: "Search everything  Ctrl ⇧ F", on: win.view === "search", run: () => win.openSearch() },
        { icon: "home", tip: "Home  Ctrl ⇧ D", on: win.view === "today", run: () => win.view = "today" },
        { icon: "toc", tip: "Outline and links  Ctrl \\", on: win.detailsShown, run: () => win.detailsShown = !win.detailsShown },
        { icon: "smart_toy", tip: "Claude  Ctrl ⇧ A", on: win.claudeShown, run: () => win.claudeShown = !win.claudeShown },
        { icon: "terminal", tip: "Shell  Ctrl `", on: win.drawerShown, run: () => win.toggleDrawer() }
    ]

    component Act: Item {
        id: act
        property var item
        implicitWidth: 52; implicitHeight: 46
        Rectangle {
            anchors.centerIn: parent
            width: 40; height: 40; radius: 14
            color: act.item.on ? Qt.alpha(bar.t.accent, 0.24) : ah.hovered ? Qt.alpha(bar.t.text, 0.07) : "transparent"
            Text { anchors.centerIn: parent; text: act.item.icon; color: act.item.on ? bar.t.accent : ah.hovered ? bar.t.text : bar.t.dim; font.family: bar.t.icons; font.pixelSize: 21 }
        }
        HoverHandler { id: ah; cursorShape: Qt.PointingHandCursor }
        TapHandler { onTapped: act.item.run() }
        // the name, beside it on hover
        Rectangle {
            visible: ah.hovered || (bar.tipTest && act.item.icon === "folder_open")
            x: 50; anchors.verticalCenter: parent.verticalCenter
            width: tipT.implicitWidth + 16; height: 26; radius: 8
            color: bar.t.raised; border.width: 1; border.color: Qt.alpha(bar.t.rule, 0.4)
            z: 100
            Text { id: tipT; anchors.centerIn: parent; text: act.item.tip; color: bar.t.text; font.family: bar.t.sans; font.pixelSize: 12 }
        }
    }

    ColumnLayout {
        anchors { fill: parent; topMargin: 16; bottomMargin: 12 }
        spacing: 2
        Repeater { model: bar.items; delegate: Act { required property var modelData; item: modelData } }
        Item { Layout.fillHeight: true }
        Act { item: ({ icon: "settings", tip: "Settings  Ctrl ,", on: bar.win.view === "settings", run: () => bar.win.view = "settings" }) }
    }
}
