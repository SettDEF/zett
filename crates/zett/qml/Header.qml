import QtQuick
import QtQuick.Layouts

// The main window's header over the tabs: a breadcrumb of what is open (folder › path › name), the search pill in the
// middle (Ctrl+P), the view switches as one segmented group at the right.
Item {
    id: hd
    required property var win
    required property var t
    required property var app
    Layout.fillWidth: true
    implicitHeight: 50
    clip: true   // a narrow window (beside Claude) must not spill it over the sidebar
    readonly property bool roomy: width >= 620
    readonly property bool file: win.view === "file"
    readonly property string title: file ? win.filePath.slice(win.filePath.lastIndexOf("/") + 1) : win.view === "page" ? (win.pages[app.current]?.title || "Untitled") : ({ today: "Home", settings: "Settings", search: "Search" })[win.view] ?? win.view
    // the crumbs before the name: the workspace, then the file's folders
    readonly property var crumbs: [app.dir.slice(app.dir.lastIndexOf("/") + 1)].concat(file ? win.filePath.replace(app.dir + "/", "").split("/").slice(0, -1) : [])
    readonly property bool dirty: file && win.fileEditor !== null && win.fileEditor.dirty

    component Seg: Rectangle {
        property string icon
        property string tip
        property bool on: false
        signal hit
        implicitWidth: 30; implicitHeight: 26; radius: 13
        color: on ? Qt.alpha(hd.t.accent, 0.2) : sh.hovered ? Qt.alpha(hd.t.text, 0.07) : "transparent"
        Text { anchors.centerIn: parent; text: parent.icon; color: parent.on ? hd.t.accent : sh.hovered ? hd.t.text : hd.t.dim; font.family: hd.t.icons; font.pixelSize: 16 }
        HoverHandler { id: sh; cursorShape: Qt.PointingHandCursor }
        TapHandler { onTapped: parent.hit() }
        // the name under it on hover
        Rectangle {
            visible: sh.hovered; z: 100
            anchors { top: parent.bottom; topMargin: 6; horizontalCenter: parent.horizontalCenter }
            width: tipT.implicitWidth + 14; height: 22; radius: 8
            color: hd.t.raised; border.width: 1; border.color: Qt.alpha(hd.t.rule, 0.4)
            Text { id: tipT; anchors.centerIn: parent; text: parent.parent.tip; color: hd.t.text; font.family: hd.t.sans; font.pixelSize: 11 }
        }
    }

    // the breadcrumb, at the left
    Row {
        id: crumbRow
        anchors { left: parent.left; leftMargin: 18; verticalCenter: parent.verticalCenter; verticalCenterOffset: 4 }
        width: Math.max(0, Math.min(implicitWidth, (pill.visible ? pill.x : segs.x) - 30))
        clip: true
        spacing: 6
        Text { anchors.verticalCenter: parent.verticalCenter; text: hd.file ? "code" : hd.win.view === "page" ? "description" : hd.win.view === "settings" ? "settings" : "home"; color: hd.t.accent; font.family: hd.t.icons; font.pixelSize: 16 }
        Repeater {
            model: hd.crumbs
            delegate: Row {
                required property string modelData
                anchors.verticalCenter: parent.verticalCenter
                spacing: 6
                Text { anchors.verticalCenter: parent.verticalCenter; text: modelData; color: hd.t.faint; font.family: hd.t.sans; font.pixelSize: 13 }
                Text { anchors.verticalCenter: parent.verticalCenter; text: "chevron_right"; color: hd.t.faint; opacity: 0.6; font.family: hd.t.icons; font.pixelSize: 14 }
            }
        }
        Text { anchors.verticalCenter: parent.verticalCenter; text: hd.title; color: hd.t.text; font.family: hd.t.sans; font.pixelSize: 13; font.weight: Font.DemiBold }
        Rectangle { visible: hd.dirty; anchors.verticalCenter: parent.verticalCenter; width: 7; height: 7; radius: 3.5; color: hd.t.accent }
        Repeater {
            model: hd.file && hd.win.fileEditor ? hd.win.fileEditor.symbolPath : []
            delegate: Row {
                required property var modelData
                anchors.verticalCenter: parent.verticalCenter
                spacing: 6
                Text { anchors.verticalCenter: parent.verticalCenter; text: "chevron_right"; color: hd.t.faint; opacity: 0.6; font.family: hd.t.icons; font.pixelSize: 14 }
                Text { anchors.verticalCenter: parent.verticalCenter; text: modelData.name; color: hd.t.dim; font.family: hd.t.mono; font.pixelSize: 12 }
            }
        }
    }

    // the search pill, centred
    Rectangle {
        id: pill
        visible: hd.roomy
        anchors { horizontalCenter: parent.horizontalCenter; verticalCenter: parent.verticalCenter; verticalCenterOffset: 4 }
        width: Math.max(180, Math.min(420, hd.width - crumbRow.implicitWidth - segs.width - 120))
        height: 32; radius: 16
        color: ph.hovered ? hd.t.raised : Qt.alpha(hd.t.text, 0.05)
        border.width: 1; border.color: Qt.alpha(hd.t.rule, ph.hovered ? 0.5 : 0.25)
        RowLayout {
            anchors { fill: parent; leftMargin: 12; rightMargin: 8 }
            spacing: 8
            Text { text: "search"; color: hd.t.faint; font.family: hd.t.icons; font.pixelSize: 16 }
            Text { Layout.fillWidth: true; text: "Search files, pages and commands"; elide: Text.ElideRight; color: hd.t.faint; font.family: hd.t.sans; font.pixelSize: 12 }
            Rectangle { implicitWidth: kT.implicitWidth + 12; implicitHeight: 20; radius: 6; color: Qt.alpha(hd.t.text, 0.07); border.width: 1; border.color: Qt.alpha(hd.t.rule, 0.3)
                Text { id: kT; anchors.centerIn: parent; text: "Ctrl P"; color: hd.t.dim; font.family: hd.t.mono; font.pixelSize: 10 } }
        }
        HoverHandler { id: ph; cursorShape: Qt.PointingHandCursor }
        TapHandler { onTapped: hd.win.openQuick("") }
    }

    // the view switches: one segmented group
    Rectangle {
        id: segs
        anchors { right: parent.right; rightMargin: 14; verticalCenter: parent.verticalCenter; verticalCenterOffset: 4 }
        width: segRow.implicitWidth + 8; height: 34; radius: 17
        color: Qt.alpha(hd.t.text, 0.05)
        border.width: 1; border.color: Qt.alpha(hd.t.rule, 0.25)
        Row {
            id: segRow
            anchors.centerIn: parent
            spacing: 2
            Seg { visible: hd.file; icon: "splitscreen_right"; tip: "Split  Ctrl \\"; on: hd.win.sideFile.length > 0; onHit: { if (hd.win.sideFile) hd.win.sideFile = ""; else { const o = hd.win.fileTabs.filter(p => p !== hd.win.filePath); hd.win.sideFile = o.length ? o[o.length - 1] : hd.win.filePath } } }
            Seg { icon: "create_new_folder"; tip: "Open Folder  Ctrl K O"; onHit: hd.win.openFolderPick() }
            Seg { icon: "left_panel_open"; tip: "Sidebar"; on: !hd.win.sidebarHidden; onHit: hd.win.sidebarHidden = !hd.win.sidebarHidden }
            Seg { icon: "right_panel_open"; tip: "Details"; on: hd.win.detailsShown; onHit: hd.win.detailsShown = !hd.win.detailsShown }
            Seg { icon: "terminal"; tip: "Panel  Ctrl `"; on: hd.win.drawerShown; onHit: hd.win.toggleDrawer() }
            Rectangle { width: 1; height: 16; anchors.verticalCenter: parent.verticalCenter; color: Qt.alpha(hd.t.rule, 0.5) }
            Seg { icon: "bolt"; tip: "Commands  Ctrl ⇧ P"; onHit: hd.win.openPalette() }
        }
    }
}
