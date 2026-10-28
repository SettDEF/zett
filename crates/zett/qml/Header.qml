import QtQuick
import QtQuick.Layouts

// The main window's header over the tabs: what is open (icon, name, where), a search pill in the middle (Ctrl+P),
// round buttons at the right (split, sidebar, details, palette).
Item {
    id: hd
    required property var win
    required property var t
    required property var app
    Layout.fillWidth: true
    implicitHeight: 52
    readonly property bool file: win.view === "file"
    readonly property string title: file ? win.filePath.slice(win.filePath.lastIndexOf("/") + 1) : win.view === "page" ? (win.pages[app.current]?.title || "Untitled") : ({ today: "Home", settings: "Settings", search: "Search" })[win.view] ?? win.view
    readonly property string where: file ? win.filePath.replace(app.dir + "/", "").split("/").slice(0, -1).join("  ›  ") : app.dir.slice(app.dir.lastIndexOf("/") + 1)
    component Btn: Rectangle {
        property string icon
        property bool on: false
        signal hit
        implicitWidth: 32; implicitHeight: 32; radius: 16
        color: on ? Qt.alpha(hd.t.accent, 0.2) : bh.hovered ? hd.t.raised : "transparent"
        Text { anchors.centerIn: parent; text: parent.icon; color: parent.on ? hd.t.accent : hd.t.dim; font.family: hd.t.icons; font.pixelSize: 18 }
        HoverHandler { id: bh; cursorShape: Qt.PointingHandCursor }
        TapHandler { onTapped: parent.hit() }
    }
    RowLayout {
        anchors { fill: parent; leftMargin: 16; rightMargin: 12; topMargin: 10 }
        spacing: 10
        Rectangle {
            implicitWidth: 30; implicitHeight: 30; radius: 10
            color: Qt.alpha(hd.t.accent, 0.16)
            Text { anchors.centerIn: parent; text: hd.file ? "code" : hd.win.view === "page" ? "description" : hd.win.view === "settings" ? "settings" : "home"; color: hd.t.accent; font.family: hd.t.icons; font.pixelSize: 17 }
        }
        ColumnLayout {
            spacing: 0
            Layout.preferredWidth: Math.min(implicitWidth, hd.width * 0.3)
            Text { Layout.fillWidth: true; text: hd.title; elide: Text.ElideRight; color: hd.t.text; font.family: hd.file ? hd.t.mono : hd.t.sans; font.pixelSize: 14; font.weight: Font.DemiBold }
            Text { Layout.fillWidth: true; visible: hd.where.length > 0; text: hd.where; elide: Text.ElideLeft; color: hd.t.faint; font.family: hd.t.sans; font.pixelSize: 11 }
        }
        Item { Layout.fillWidth: true }
        // the search pill
        Rectangle {
            Layout.preferredWidth: Math.min(380, hd.width * 0.36); implicitHeight: 34; radius: 17
            color: sh.hovered ? hd.t.raised : hd.t.card
            border.width: 1; border.color: Qt.alpha(hd.t.rule, 0.3)
            RowLayout {
                anchors { fill: parent; leftMargin: 14; rightMargin: 12 }
                spacing: 8
                Text { text: "search"; color: hd.t.dim; font.family: hd.t.icons; font.pixelSize: 17 }
                Text { Layout.fillWidth: true; text: "Go to file"; elide: Text.ElideRight; color: hd.t.faint; font.family: hd.t.sans; font.pixelSize: 13 }
                Rectangle { implicitWidth: kT.implicitWidth + 12; implicitHeight: 20; radius: 10; color: Qt.alpha(hd.t.text, 0.07)
                    Text { id: kT; anchors.centerIn: parent; text: "Ctrl P"; color: hd.t.faint; font.family: hd.t.mono; font.pixelSize: 10 } }
            }
            HoverHandler { id: sh; cursorShape: Qt.PointingHandCursor }
            TapHandler { onTapped: hd.win.openQuick("") }
        }
        Item { Layout.fillWidth: true }
        Btn { visible: hd.file; icon: "splitscreen_right"; on: hd.win.sideFile.length > 0; onHit: { if (hd.win.sideFile) hd.win.sideFile = ""; else { const o = hd.win.fileTabs.filter(p => p !== hd.win.filePath); hd.win.sideFile = o.length ? o[o.length - 1] : hd.win.filePath } } }
        Btn { icon: "left_panel_open"; on: !hd.win.sidebarHidden; onHit: hd.win.sidebarHidden = !hd.win.sidebarHidden }
        Btn { icon: "right_panel_open"; on: hd.win.detailsShown; onHit: hd.win.detailsShown = !hd.win.detailsShown }
        Btn { icon: "terminal"; on: hd.win.drawerShown; onHit: hd.win.toggleDrawer() }
        Btn { icon: "bolt"; onHit: hd.win.openPalette() }
    }
}
