import QtQuick
import QtQuick.Layouts

// The start page, as a code editor has it: start actions, recent folders and files, the keys worth knowing,
// and this folder at a glance. Shown at start when no file comes back; Home (notes) is one click away.
Flickable {
    id: wl
    required property var win
    required property var t
    required property var app
    visible: win.view === "welcome"
    Layout.fillWidth: true
    Layout.fillHeight: true
    clip: true
    contentHeight: wcol.implicitHeight + 60
    boundsBehavior: Flickable.StopAtBounds
    readonly property var recentFiles: (win.prefs.recentFiles ?? []).filter(p => p.startsWith(app.dir + "/")).slice(0, 8)
    readonly property var recentFolders: (win.prefs.recentFolders ?? []).filter(f => f !== app.dir).slice(0, 6)
    readonly property int changed: win.files.filter(x => !x.dir && x.git.length > 0).length

    component Card: Rectangle {
        default property alias body: inner.data
        property string title
        Layout.fillWidth: true
        Layout.alignment: Qt.AlignTop
        implicitHeight: inner.implicitHeight + 50
        radius: 18; color: wl.t.card
        border.width: 1; border.color: Qt.alpha(wl.t.rule, 0.25)
        Text { x: 18; y: 16; text: parent.title; color: wl.t.faint; font.family: wl.t.sans; font.pixelSize: 11; font.weight: Font.DemiBold; font.letterSpacing: 0.8 }
        ColumnLayout { id: inner; anchors { left: parent.left; right: parent.right; top: parent.top; margins: 10; topMargin: 38 } spacing: 2 }
    }
    component Item_: Rectangle {
        property string icon
        property string label
        property string sub
        property string keys
        signal hit
        Layout.fillWidth: true
        implicitHeight: 36; radius: 12
        color: ih.hovered ? Qt.alpha(wl.t.text, 0.06) : "transparent"
        RowLayout {
            anchors { fill: parent; leftMargin: 10; rightMargin: 10 }
            spacing: 10
            Rectangle { implicitWidth: 26; implicitHeight: 26; radius: 9; color: Qt.alpha(wl.t.accent, 0.14)
                Text { anchors.centerIn: parent; text: parent.parent.parent.icon; color: wl.t.accent; font.family: wl.t.icons; font.pixelSize: 15 } }
            Text { text: parent.parent.label; color: wl.t.text; font.family: wl.t.sans; font.pixelSize: 13 }
            Text { Layout.fillWidth: true; text: parent.parent.sub; elide: Text.ElideLeft; color: wl.t.faint; font.family: wl.t.sans; font.pixelSize: 11 }
            Repeater {
                model: parent.parent.keys ? parent.parent.keys.split(" ") : []
                delegate: Rectangle { required property string modelData; implicitWidth: kk.implicitWidth + 10; implicitHeight: 20; radius: 6; color: Qt.alpha(wl.t.text, 0.06); border.width: 1; border.color: Qt.alpha(wl.t.rule, 0.3)
                    Text { id: kk; anchors.centerIn: parent; text: modelData; color: wl.t.dim; font.family: wl.t.mono; font.pixelSize: 10 } }
            }
        }
        HoverHandler { id: ih; cursorShape: Qt.PointingHandCursor }
        TapHandler { onTapped: parent.hit() }
    }

    ColumnLayout {
        id: wcol
        x: 40; y: 40
        width: Math.min(1100, wl.width - 80)
        spacing: 20
        // the title: the icon, zett, this folder
        RowLayout {
            spacing: 16
            Image { source: "file://" + wl.win.homeDir + "/.local/share/icons/hicolor/scalable/apps/zett.svg"; sourceSize: Qt.size(64, 64); Layout.preferredWidth: 64; Layout.preferredHeight: 64 }
            ColumnLayout {
                spacing: 2
                Text { text: "zett"; color: wl.t.text; font.family: wl.t.sans; font.pixelSize: 30; font.weight: Font.Bold }
                Text { text: "Editing " + wl.app.dir.replace(wl.win.homeDir, "~"); color: wl.t.faint; font.family: wl.t.mono; font.pixelSize: 12 }
            }
        }
        GridLayout {
            Layout.fillWidth: true
            columns: wl.width > 900 ? 2 : 1
            columnSpacing: 16; rowSpacing: 16
            Card {
                title: "START"
                Item_ { icon: "note_add"; label: "New file…"; sub: "in this folder"; onHit: { wl.win.sideMode = "files"; wl.win.sidebarHidden = false; wl.win.newFileHere() } }
                Item_ { icon: "folder_open"; label: "Open folder…"; sub: "a project in a new window"; keys: "Ctrl K O"; onHit: wl.win.openFolderPick() }
                Item_ { icon: "search"; label: "Go to file"; sub: "fuzzy, every file"; keys: "Ctrl P"; onHit: wl.win.openQuick("") }
                Item_ { icon: "bolt"; label: "Command palette"; sub: "everything zett can do"; keys: "Ctrl ⇧ P"; onHit: wl.win.openPalette() }
                Item_ { icon: "terminal"; label: "Terminal"; sub: "in this folder"; keys: "Ctrl `"; onHit: wl.win.toggleDrawer() }
            }
            Card {
                title: "THIS FOLDER"
                Item_ { icon: "commit"; label: wl.win.gitBranch || "not a git repository"; sub: wl.changed ? `${wl.changed} changed` : wl.win.gitBranch ? "clean" : ""; keys: wl.win.gitBranch ? "Ctrl ⇧ G" : ""; onHit: { wl.win.sideMode = "git"; wl.win.sidebarHidden = false } }
                Item_ { icon: "draft"; label: `${wl.win.files.filter(x => !x.dir).length} files`; sub: "the Explorer"; onHit: { wl.win.sideMode = "files"; wl.win.sidebarHidden = false } }
                Item_ { icon: "error"; label: wl.win.problems.length ? `${wl.win.problems.length} problems` : "No problems"; sub: "every file"; onHit: { wl.win.drawerShown = true; drawerTab.go() } }
                Item_ { visible: wl.win.projectTasks().length > 0; icon: "play_arrow"; label: "Tasks"; sub: wl.win.projectTasks().slice(0, 3).join(" · "); onHit: wl.win.openPalette() }
                Item_ { icon: "wb_sunny"; label: "Notes: Home"; sub: `${wl.win.pages.length} pages`; keys: "Ctrl ⇧ D"; onHit: wl.win.view = "today" }
            }
            Card {
                title: "RECENT FILES"
                Text { visible: !wl.recentFiles.length; Layout.leftMargin: 10; text: "Files you open show here"; color: wl.t.faint; font.family: wl.t.sans; font.pixelSize: 12 }
                Repeater { model: wl.recentFiles; delegate: Item_ { required property string modelData; icon: "code"; label: modelData.slice(modelData.lastIndexOf("/") + 1); sub: modelData.replace(wl.app.dir + "/", "").split("/").slice(0, -1).join("/"); onHit: wl.win.openFile({ path: modelData, rel: modelData }) } }
            }
            Card {
                title: "RECENT FOLDERS"
                Text { visible: !wl.recentFolders.length; Layout.leftMargin: 10; text: "Folders you open show here"; color: wl.t.faint; font.family: wl.t.sans; font.pixelSize: 12 }
                Repeater { model: wl.recentFolders; delegate: Item_ { required property string modelData; icon: "folder"; label: modelData.slice(modelData.lastIndexOf("/") + 1); sub: modelData.replace(wl.win.homeDir, "~"); onHit: wl.win.openFolder(modelData) } }
            }
        }
        // the keys worth knowing, as chips
        Card {
            title: "KEYS"
            Flow {
                Layout.fillWidth: true; Layout.margins: 8
                spacing: 8
                Repeater {
                    model: [["Ctrl P", "go to file"], ["Ctrl ⇧ P", "commands"], ["Ctrl ⇧ F", "search"], ["F12", "definition"], ["F2", "rename"], ["Ctrl .", "quick fix"], ["Ctrl /", "comment"], ["Alt ↑↓", "move line"], ["Ctrl \\", "split"], ["Alt Z", "wrap"], ["Ctrl K Z", "zen"], ["Ctrl `", "terminal"]]
                    delegate: Rectangle {
                        required property var modelData
                        implicitWidth: kr.implicitWidth + 16; implicitHeight: 28; radius: 14
                        color: Qt.alpha(wl.t.text, 0.05)
                        Row { id: kr; anchors.centerIn: parent; spacing: 6
                            Text { text: modelData[0]; color: wl.t.accent; font.family: wl.t.mono; font.pixelSize: 11 }
                            Text { text: modelData[1]; color: wl.t.dim; font.family: wl.t.sans; font.pixelSize: 12 } }
                    }
                }
            }
        }
    }
    QtObject { id: drawerTab; function go() { wl.win.showProblemsTab() } }
}
