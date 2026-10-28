import QtQuick
import QtQuick.Layouts

// The sidebar: search, Today, the pages, the workspace folder with git status, tags, settings; drops land here.
Rectangle {
    id: side
    required property var win
    // the foot's templates (templates/*.md become a new page)
    property bool templatesOpen: false
    readonly property var templates: win.files.filter(x => !x.dir && /^templates\/[^/]+\.md$/.test(x.rel))
    required property var t
    required property var app
    Layout.fillHeight: true
    visible: !win.sidebarHidden && win.sideMode !== "git"
    implicitWidth: 260
    color: "transparent"
    // files dropped anywhere on the sidebar land in the workspace folder
    DropArea { id: sideDrop; anchors.fill: parent; onDropped: (d) => win.dropInto(d, app.dir) }
    Rectangle { anchors.fill: parent; anchors.margins: 4; radius: t.rCard; color: "transparent"; border.width: sideDrop.containsDrag ? 2 : 0; border.color: Qt.alpha(t.accent, 0.6) }
    ColumnLayout {
        anchors { fill: parent; margins: 12; topMargin: 16 }
        spacing: 4
        RowLayout {
            Layout.leftMargin: 6
            Layout.bottomMargin: 12
            spacing: 10
            Rectangle {
                implicitWidth: 34; implicitHeight: 34; radius: 10
                color: t.accent
                Text { anchors.centerIn: parent; text: "z"; color: t.onAccent; font.family: t.sans; font.pixelSize: 20; font.weight: Font.Bold }
            }
            ColumnLayout {
                spacing: 0
                // Explorer: the folder, as an editor names it; otherwise zett and its pages
                readonly property bool code: win.sideMode === "files"
                Text { text: parent.code ? app.dir.slice(app.dir.lastIndexOf("/") + 1) : "zett"; color: t.text; font.family: t.sans; font.pixelSize: 17; font.weight: Font.DemiBold }
                Text { text: parent.code ? `${win.files.filter(x => !x.dir).length} files${win.gitBranch ? "  ·  " + win.gitBranch : ""}` : `${win.pages.length} pages`; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
            }
        }
        // search: opens quick open
        Rectangle {
            Layout.fillWidth: true
            Layout.bottomMargin: 10
            implicitHeight: 36
            radius: 18
            color: searchHov.hovered ? t.raised : t.card
            RowLayout {
                anchors { fill: parent; leftMargin: 12; rightMargin: 12 }
                spacing: 8
                Text { text: "search"; color: t.dim; font.family: t.icons; font.pixelSize: 18 }
                Text { Layout.fillWidth: true; text: win.sideMode === "files" ? "Go to file" : "Search pages"; color: t.faint; font.family: t.sans; font.pixelSize: 13 }
                Text { text: "Ctrl P"; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
            }
            HoverHandler { id: searchHov; cursorShape: Qt.PointingHandCursor }
            TapHandler { onTapped: win.openQuick("") }
        }
        // places: Today
        Rectangle {
            Layout.fillWidth: true
            visible: win.sideMode !== "files"
            Layout.bottomMargin: 8
            implicitHeight: 36; radius: t.rRow
            readonly property bool sel: win.view === "today"
            color: sel ? Qt.alpha(t.accent, 0.16) : todayHov.hovered ? Qt.alpha(t.text, 0.06) : "transparent"
            RowLayout {
                anchors { fill: parent; leftMargin: 10; rightMargin: 12 }
                spacing: 10
                Text { text: "wb_sunny"; color: parent.parent.sel ? t.accent : t.faint; font.family: t.icons; font.pixelSize: 18 }
                Text { Layout.fillWidth: true; text: "Today"; color: parent.parent.sel ? t.text : t.dim; font.family: t.sans; font.pixelSize: 14; font.weight: parent.parent.sel ? Font.DemiBold : Font.Normal }
                Text { text: "Ctrl ⇧ D"; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
            }
            HoverHandler { id: todayHov; cursorShape: Qt.PointingHandCursor }
            TapHandler { onTapped: win.view = "today" }
        }
        // section header with the new-page button
        RowLayout {
            visible: win.sideMode !== "files"   // the activity bar picks what the sidebar shows
            Layout.fillWidth: true
            Layout.leftMargin: 8
            Layout.bottomMargin: 2
            Text { text: "PAGES"; color: t.faint; font.family: t.sans; font.pixelSize: 11; font.weight: Font.DemiBold; font.letterSpacing: 0.8 }
            Text { text: win.pages.length; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
            Item { Layout.fillWidth: true }
            Rectangle {
                implicitWidth: 26; implicitHeight: 26; radius: 13
                color: newHov.hovered ? t.raised : "transparent"
                Text { anchors.centerIn: parent; text: "add"; color: newHov.hovered ? t.accent : t.dim; font.family: t.icons; font.pixelSize: 18 }
                HoverHandler { id: newHov; cursorShape: Qt.PointingHandCursor }
                TapHandler { onTapped: { app.newPage("Untitled"); win.focusAfter = 0 } }
            }
        }
        ListView {
            id: pageList
            visible: win.sideMode !== "files"
            Layout.fillWidth: true
            // as tall as its pages, up to half the sidebar; the file tree takes the rest
            Layout.preferredHeight: Math.min(contentHeight, Math.max(120, (parent.height - 200) * 0.5))
            Layout.fillHeight: !win.filesOpen || win.sideMode === "pages"
            clip: true
            spacing: 2
            model: win.pages
            delegate: Rectangle {
                required property var modelData
                required property int index
                width: ListView.view.width
                height: 36
                radius: t.rRow
                readonly property bool sel: index === app.current && win.view === "page"
                color: sel ? Qt.alpha(t.accent, 0.16) : rowHov.hovered ? Qt.alpha(t.text, 0.06) : "transparent"
                RowLayout {
                    anchors { fill: parent; leftMargin: 10; rightMargin: 12 }
                    spacing: 10
                    Text { text: "description"; color: parent.parent.sel ? t.accent : t.faint; font.family: t.icons; font.pixelSize: 18 }
                    Text {
                        Layout.fillWidth: true
                        text: modelData.title || "Untitled"
                        elide: Text.ElideRight
                        color: parent.parent.sel ? t.text : t.dim
                        font.family: t.sans; font.pixelSize: 14
                        font.weight: parent.parent.sel ? Font.DemiBold : Font.Normal
                    }
                    Text { text: modelData.blocks; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
                }
                DragHandler { id: pDrag; target: null }
                Drag.active: pDrag.active
                Drag.dragType: Drag.Automatic
                Drag.supportedActions: Qt.MoveAction | Qt.CopyAction
                Drag.mimeData: ({ "text/uri-list": "file://" + (modelData.path ?? ""), "text/plain": modelData.path ?? "", "application/x-zett-page": modelData.title || "Untitled" })
                HoverHandler { id: rowHov; cursorShape: Qt.PointingHandCursor }
                TapHandler { onTapped: win.openPage(index) }
                TapHandler { acceptedButtons: Qt.RightButton; onTapped: (ev) => win.pageMenu(index, ev.scenePosition.x, ev.scenePosition.y) }
            }
        }

        // ── FILES: the workspace folder with git status ──
        component SectionHead: RowLayout {
            property string label
            property string count
            property bool open: true
            signal toggled
            property string action: ""      // an icon button at the right (collapse all)
            signal acted
            Layout.fillWidth: true
            Layout.leftMargin: 8
            Layout.topMargin: 10
            Text { text: parent.open ? "expand_more" : "chevron_right"; color: t.faint; font.family: t.icons; font.pixelSize: 16 }
            Text { text: parent.label; color: t.faint; font.family: t.sans; font.pixelSize: 11; font.weight: Font.DemiBold; font.letterSpacing: 0.8 }
            Text { text: parent.count; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
            Item { Layout.fillWidth: true; implicitHeight: 22 }
            Rectangle {
                visible: parent.action.length > 0
                implicitWidth: 24; implicitHeight: 22; radius: 11
                color: actHov.hovered ? t.raised : "transparent"
                Text { anchors.centerIn: parent; text: parent.parent.action; color: t.dim; font.family: t.icons; font.pixelSize: 15 }
                HoverHandler { id: actHov; cursorShape: Qt.PointingHandCursor }
                TapHandler { onTapped: parent.parent.acted() }
            }
            TapHandler { onTapped: parent.toggled() }
            HoverHandler { cursorShape: Qt.PointingHandCursor }
        }
        SectionHead { visible: win.sideMode !== "pages"; label: "FILES"; count: win.files.filter(x => !x.dir).length; open: win.filesOpen; onToggled: win.filesOpen = !win.filesOpen; action: "unfold_less"; onActed: win.closedDirs = win.files.filter(x => x.dir).map(x => x.rel) }
        ListView {
            id: fileList
            visible: win.filesOpen && win.sideMode !== "pages"
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.minimumHeight: 60
            clip: true
            // a folder closed in the tree hides what is under it
            model: win.files.filter(x => !win.closedDirs.some(d => x.rel.startsWith(d + "/")))
            delegate: Rectangle {
                required property var modelData
                readonly property bool sel: win.view === "file" && win.filePath === modelData.path
                readonly property bool shut: win.closedDirs.includes(modelData.rel)
                width: ListView.view.width
                height: 28; radius: 8
                color: sel ? Qt.alpha(t.accent, 0.16) : fHov.hovered ? Qt.alpha(t.text, 0.06) : "transparent"
                RowLayout {
                    anchors { fill: parent; leftMargin: 8 + modelData.depth * 14; rightMargin: 10 }
                    spacing: 6
                    Text { text: modelData.dir ? (parent.parent.shut ? "folder" : "folder_open") : modelData.name.endsWith(".md") ? "description" : "draft"; color: modelData.dir ? t.dim : t.faint; font.family: t.icons; font.pixelSize: 16 }
                    Text { Layout.fillWidth: true; text: modelData.name; elide: Text.ElideRight; color: modelData.git === "M" ? t.changed : modelData.git === "U" || modelData.git === "A" ? t.added : modelData.git === "D" ? t.removed : t.dim; font.family: t.sans; font.pixelSize: 13 }
                    Text { visible: modelData.git.length > 0; text: modelData.git; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
                }
                // drag: a real system drag (into a folder here, onto the page, or out to other apps)
                DragHandler { id: fDrag; target: null }
                Drag.active: fDrag.active
                Drag.dragType: Drag.Automatic
                Drag.supportedActions: Qt.MoveAction | Qt.CopyAction
                Drag.mimeData: ({ "text/uri-list": "file://" + modelData.path, "text/plain": modelData.path })
                // a folder takes drops
                DropArea {
                    id: fDrop
                    anchors.fill: parent
                    enabled: modelData.dir
                    onDropped: (d) => win.dropInto(d, modelData.path)
                }
                border.width: fDrop.containsDrag ? 1 : 0
                border.color: t.accent
                HoverHandler { id: fHov; cursorShape: Qt.PointingHandCursor }
                TapHandler { onTapped: win.openFile(modelData) }
                TapHandler { acceptedButtons: Qt.RightButton; onTapped: (ev) => win.fileMenu(modelData, ev.scenePosition.x, ev.scenePosition.y) }
            }
        }

        // ── TAGS ──
        SectionHead { visible: win.tagList.length > 0 && win.sideMode !== "files"; label: "TAGS"; count: win.tagList.length; open: win.tagsOpen; onToggled: win.tagsOpen = !win.tagsOpen }
        Flow {
            visible: win.tagsOpen && win.tagList.length > 0 && win.sideMode !== "files"
            Layout.fillWidth: true
            Layout.leftMargin: 8
            Layout.bottomMargin: 6
            spacing: 6
            Repeater {
                model: win.tagList.slice(0, 24)
                delegate: Rectangle {
                    required property var modelData
                    implicitWidth: tagRow.implicitWidth + 16; implicitHeight: 24; radius: 12
                    color: tagHov.hovered ? t.raised : t.card
                    RowLayout { id: tagRow; anchors.centerIn: parent; spacing: 4
                        Text { text: "#" + modelData.tag; color: t.accent; font.family: t.sans; font.pixelSize: 12 }
                        Text { text: modelData.count; color: t.faint; font.family: t.mono; font.pixelSize: 10 } }
                    HoverHandler { id: tagHov; cursorShape: Qt.PointingHandCursor }
                    TapHandler { onTapped: win.openQuick("#" + modelData.tag) }
                }
            }
        }
        // init.lua's sections (zett.gui.sidebar): a title and rows that run a command
        Repeater {
            model: win.hooks.sidebar
            delegate: ColumnLayout {
                id: hs
                required property var modelData
                Layout.fillWidth: true
                spacing: 2
                Text { Layout.leftMargin: 8; Layout.topMargin: 10; text: hs.modelData.title.toUpperCase(); color: t.faint; font.family: t.sans; font.pixelSize: 11; font.weight: Font.DemiBold; font.letterSpacing: 0.8 }
                Repeater {
                    model: hs.modelData.items
                    delegate: Rectangle {
                        required property var modelData
                        Layout.fillWidth: true
                        implicitHeight: 30; radius: t.rRow
                        color: hiHov.hovered ? Qt.alpha(t.text, 0.06) : "transparent"
                        RowLayout {
                            anchors { fill: parent; leftMargin: 10; rightMargin: 12 }
                            spacing: 10
                            Text { text: modelData.icon ?? "chevron_right"; color: t.faint; font.family: t.icons; font.pixelSize: 16 }
                            Text { Layout.fillWidth: true; text: modelData.label; elide: Text.ElideRight; color: t.dim; font.family: t.sans; font.pixelSize: 13 }
                        }
                        HoverHandler { id: hiHov; cursorShape: Qt.PointingHandCursor }
                        TapHandler { onTapped: win.runHook(modelData.cmd) }
                    }
                }
            }
        }
        // the foot, as in Notion: templates (templates/*.md become a new page), import, trash
        Repeater {
            // notes things: not in the Explorer
            model: win.sideMode === "files" ? [] : [
                { icon: "auto_awesome_mosaic", label: "Templates", hint: side.templates.length ? `${side.templates.length}` : "", run: () => side.templatesOpen = !side.templatesOpen },
                { icon: "download", label: "Import", hint: "drop files here", run: () => Qt.openUrlExternally("file://" + app.dir) },
                { icon: "delete", label: "Trash", hint: "", run: () => Qt.openUrlExternally("trash:///") }
            ]
            delegate: Rectangle {
                required property var modelData
                Layout.fillWidth: true
                implicitHeight: 32; radius: t.rRow
                color: footHov.hovered ? Qt.alpha(t.text, 0.06) : "transparent"
                RowLayout {
                    anchors { fill: parent; leftMargin: 10; rightMargin: 12 }
                    spacing: 10
                    Text { text: modelData.icon; color: t.faint; font.family: t.icons; font.pixelSize: 18 }
                    Text { Layout.fillWidth: true; text: modelData.label; color: t.dim; font.family: t.sans; font.pixelSize: 14 }
                    Text { text: modelData.hint; color: t.faint; font.family: t.sans; font.pixelSize: 11 }
                }
                HoverHandler { id: footHov; cursorShape: Qt.PointingHandCursor }
                TapHandler { onTapped: modelData.run() }
            }
        }
        // the templates, when open: a click copies one into the workspace as a new page
        Text { visible: side.templatesOpen && !side.templates.length; Layout.leftMargin: 38; Layout.fillWidth: true; wrapMode: Text.Wrap; text: "Put .md files in templates/ to start pages from them"; color: t.faint; font.family: t.sans; font.pixelSize: 12 }
        Repeater {
            model: side.templatesOpen ? side.templates : []
            delegate: Rectangle {
                required property var modelData
                Layout.fillWidth: true; Layout.leftMargin: 28
                implicitHeight: 28; radius: t.rRow
                color: tplHov.hovered ? Qt.alpha(t.accent, 0.12) : "transparent"
                Text { x: 10; anchors.verticalCenter: parent.verticalCenter; text: modelData.name.replace(/\.md$/, ""); color: t.text; font.family: t.sans; font.pixelSize: 13 }
                HoverHandler { id: tplHov; cursorShape: Qt.PointingHandCursor }
                TapHandler { onTapped: { app.importFiles("file://" + modelData.path, app.dir); side.templatesOpen = false } }
            }
        }
        // settings, at the foot of the sidebar
        Rectangle {
            Layout.fillWidth: true
            implicitHeight: 34; radius: t.rRow
            readonly property bool sel: win.view === "settings"
            color: sel ? Qt.alpha(t.accent, 0.16) : setHov.hovered ? Qt.alpha(t.text, 0.06) : "transparent"
            RowLayout {
                anchors { fill: parent; leftMargin: 10; rightMargin: 12 }
                spacing: 10
                Text { text: "settings"; color: parent.parent.sel ? t.accent : t.faint; font.family: t.icons; font.pixelSize: 18 }
                Text { Layout.fillWidth: true; text: "Settings"; color: parent.parent.sel ? t.text : t.dim; font.family: t.sans; font.pixelSize: 14 }
                Text { text: "Ctrl ,"; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
            }
            HoverHandler { id: setHov; cursorShape: Qt.PointingHandCursor }
            TapHandler { onTapped: win.view = "settings" }
        }
        Text {
            Layout.fillWidth: true
            text: app.dir
            elide: Text.ElideLeft
            color: t.faint
            font.family: t.mono; font.pixelSize: 11
        }
    }
}
