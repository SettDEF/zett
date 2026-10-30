import QtQuick
import QtQuick.Layouts

// The sidebar: search, Today, the pages, the workspace folder with git status, tags, settings; drops land here.
Rectangle {
    id: side
    required property var win
    // the foot's templates (templates/*.md become a new page)
    property bool templatesOpen: false
    // the Explorer's New file / New folder: the name typed in a field under the head (a/b.rs makes the folders too)
    // a file's badge in the Explorer: a short name for its type and that language's usual colour
    function fileBadge(name) {
        const n = name.toLowerCase(), e = n.includes(".") ? n.slice(n.lastIndexOf(".") + 1) : n
        const m = { rs: ["rs", "#e8916b"], toml: ["tm", "#b4a7d6"], lock: ["lk", "#8a8f98"], qml: ["qml", "#6fcf97"], js: ["js", "#f0d75a"], mjs: ["js", "#f0d75a"], ts: ["ts", "#5fa8f5"], tsx: ["tsx", "#5fa8f5"], jsx: ["jsx", "#5fd0f0"], json: ["{}", "#f0b45a"], md: ["md", "#9aa4b2"], py: ["py", "#f5d06e"], sh: ["sh", "#8fd18f"], bash: ["sh", "#8fd18f"], zsh: ["sh", "#8fd18f"], lua: ["lua", "#7a8cf5"], c: ["c", "#7fa7d9"], h: ["h", "#a58fd9"], cpp: ["c++", "#6f9bd9"], hpp: ["h++", "#a58fd9"], go: ["go", "#5fd0e0"], html: ["<>", "#f08a5a"], css: ["css", "#7a9cf5"], scss: ["scs", "#e58fb4"], yml: ["yml", "#e57a7a"], yaml: ["yml", "#e57a7a"], svg: ["svg", "#f0b45a"], png: ["img", "#c38fe5"], jpg: ["img", "#c38fe5"], txt: ["txt", "#9aa4b2"], nix: ["nix", "#7ab4f5"], makefile: ["mk", "#d9a06f"], dockerfile: ["dk", "#5fa8f5"], gitignore: ["git", "#e57a5a"], vue: ["vue", "#6fcf97"], java: ["jv", "#e5946f"], kt: ["kt", "#b48fe5"], rb: ["rb", "#e5605a"], php: ["php", "#8f9be5"], sql: ["sql", "#e5c46f"], conf: ["cfg", "#9aa4b2"], ini: ["cfg", "#9aa4b2"], env: ["env", "#e5c46f"] }
        return m[e] ?? [e.slice(0, 3) || "·", "#9aa4b2"]
    }
    // the icon file for a type (~/.local/share/zett/icons/filetypes, from packaging/icons/filetypes)
    function fileIcon(name) {
        const n = name.toLowerCase(), e = n.includes(".") ? n.slice(n.lastIndexOf(".") + 1) : n
        const alias = { mjs: "js", bash: "sh", zsh: "sh", hpp: "h", cc: "cpp", yaml: "yml", png: "img", jpg: "img", jpeg: "img", webp: "img", gif: "img", makefile: "make", gitignore: "git", ini: "conf", cfg: "conf" }
        const have = ["rs", "toml", "lock", "qml", "js", "ts", "tsx", "jsx", "json", "md", "py", "sh", "lua", "c", "h", "cpp", "go", "html", "css", "yml", "svg", "img", "txt", "nix", "make", "git", "java", "rb", "sql", "conf"]
        const k = alias[e] ?? e
        return "file://" + win.homeDir + "/.local/share/zett/icons/filetypes/" + (have.includes(k) ? k : "file") + ".svg"
    }
    property string newKind: ""
    property bool openOpen: true
    property bool recentOpen: true
    property bool foldersOpen: true
    property bool todoOpen: true
    function create(name) {
        name = name.trim().replace(/^\/+/, ""); const kind = newKind; newKind = ""
        if (!name || name.includes("..")) return
        const p = app.dir + "/" + name
        if (kind === "folder") app.spawn(JSON.stringify(["mkdir", "-p", p]))
        else { const d = p.slice(0, p.lastIndexOf("/")); app.spawn(JSON.stringify(["sh", "-c", 'mkdir -p "$1" && touch "$2"', "sh", d, p])); openLater.path = p; openLater.start() }
        sideRefresh.start()
    }
    Timer { id: sideRefresh; interval: 300; onTriggered: side.win.refreshSide() }
    Timer { id: openLater; property string path; interval: 400; onTriggered: { side.win.refreshSide(); side.win.openFile({ path, rel: path }) } }
    readonly property var templates: win.files.filter(x => !x.dir && /^templates\/[^/]+\.md$/.test(x.rel))
    required property var t
    required property var app
    Layout.fillHeight: true
    visible: !win.sidebarHidden && win.sideMode !== "git"
    implicitWidth: 268
    color: "transparent"
    // files dropped anywhere on the sidebar land in the workspace folder
    DropArea { id: sideDrop; anchors.fill: parent; onDropped: (d) => win.dropInto(d, app.dir) }
    // the sidebar sits in its own card, as the page does: the dark gaps round both off
    Rectangle { anchors { fill: parent; topMargin: 8; bottomMargin: 8; rightMargin: 8 } radius: t.rCard; color: t.panel; border.width: 1; border.color: Qt.alpha(t.rule, 0.35) }
    Rectangle { anchors.fill: parent; anchors.margins: 4; radius: t.rCard; color: "transparent"; border.width: sideDrop.containsDrag ? 2 : 0; border.color: Qt.alpha(t.accent, 0.6) }
    ColumnLayout {
        anchors { fill: parent; margins: 14; topMargin: 18; rightMargin: 20; bottomMargin: 16 }
        spacing: 4
        // the header: a card with the workspace (its initial, name, folder) and its state as soft chips
        Rectangle {
            id: hdr
            Layout.fillWidth: true
            Layout.bottomMargin: 10
            readonly property bool code: win.sideMode === "files" || win.sideMode === "git"
            readonly property string name: code ? app.dir.slice(app.dir.lastIndexOf("/") + 1) : "zett"
            readonly property int changed: win.files.filter(x => !x.dir && x.git.length > 0).length
            implicitHeight: hdrCol.implicitHeight + 24
            radius: 18
            color: t.card
            border.width: 1; border.color: Qt.alpha(t.rule, 0.25)
            ColumnLayout {
                id: hdrCol
                anchors { left: parent.left; right: parent.right; top: parent.top; margins: 12 }
                spacing: 10
                RowLayout {
                    spacing: 10
                    // zett's logo (packaging/logo.glsl, installed with the icons)
                    Image { source: "file://" + win.homeDir + "/.local/share/icons/hicolor/128x128/apps/zett.png"; sourceSize: Qt.size(72, 72); Layout.preferredWidth: 36; Layout.preferredHeight: 36; smooth: true; mipmap: true }
                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 1
                        Text { Layout.fillWidth: true; text: hdr.name; elide: Text.ElideRight; color: t.text; font.family: t.sans; font.pixelSize: 15; font.weight: Font.DemiBold }
                        Text { Layout.fillWidth: true; text: app.dir.replace(win.homeDir, "~"); elide: Text.ElideLeft; color: t.faint; font.family: t.mono; font.pixelSize: 10 }
                    }
                }
                // state chips: one line, the same height, each an icon and a number or a name
                Flow {
                    Layout.fillWidth: true
                    spacing: 6
                    Repeater {
                        model: (hdr.code
                            ? [[win.gitBranch ? "commit" : "", win.gitBranch], ["draft", `${win.files.filter(x => !x.dir).length} files`], [hdr.changed ? "edit" : "", hdr.changed ? `${hdr.changed} changed` : ""]]
                            : [["description", `${win.pages.length} pages`], ["sell", win.tagList.length ? `${win.tagList.length} tags` : ""]]).filter(c => c[1])
                        delegate: Rectangle {
                            required property var modelData
                            implicitWidth: chipRow.implicitWidth + 16; implicitHeight: 22; radius: 11
                            color: Qt.alpha(modelData[0] === "edit" ? t.accent : t.text, modelData[0] === "edit" ? 0.14 : 0.06)
                            Row {
                                id: chipRow; anchors.centerIn: parent; spacing: 4
                                Text { anchors.verticalCenter: parent.verticalCenter; text: modelData[0]; color: modelData[0] === "edit" ? t.accent : t.faint; font.family: t.icons; font.pixelSize: 13 }
                                Text { anchors.verticalCenter: parent.verticalCenter; text: modelData[1]; color: modelData[0] === "edit" ? t.accent : t.dim; font.family: t.mono; font.pixelSize: 10 }
                            }
                        }
                    }
                }
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
            Layout.fillHeight: false   // only as tall as its pages: the sections under it follow straight after
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
            property var more: []          // more buttons before it: [icon, tip, run]
            signal acted
            Layout.fillWidth: true
            Layout.leftMargin: 8
            Layout.topMargin: 10
            Text { text: parent.open ? "expand_more" : "chevron_right"; color: t.faint; font.family: t.icons; font.pixelSize: 16 }
            Text { text: parent.label; color: t.faint; font.family: t.sans; font.pixelSize: 11; font.weight: Font.DemiBold; font.letterSpacing: 0.8 }
            Text { text: parent.count; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
            Item { Layout.fillWidth: true; implicitHeight: 22 }
            Repeater {
                model: parent.more
                delegate: Rectangle {
                    required property var modelData
                    implicitWidth: 24; implicitHeight: 22; radius: 11
                    color: mH.hovered ? t.raised : "transparent"
                    Text { anchors.centerIn: parent; text: modelData[0]; color: t.dim; font.family: t.icons; font.pixelSize: 15 }
                    HoverHandler { id: mH; cursorShape: Qt.PointingHandCursor }
                    TapHandler { onTapped: modelData[2]() }
                }
            }
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
        SectionHead { visible: win.sideMode !== "pages"; label: "FILES"; count: win.files.filter(x => !x.dir).length; open: win.filesOpen; onToggled: win.filesOpen = !win.filesOpen; action: "unfold_less"; more: [["note_add", "New file", () => side.newKind = "file"], ["create_new_folder", "New folder", () => side.newKind = "folder"]]; onActed: win.closedDirs = win.files.filter(x => x.dir).map(x => x.rel) }
        Rectangle {
            visible: side.newKind.length > 0
            Layout.fillWidth: true; Layout.leftMargin: 6
            implicitHeight: 32; radius: 16; color: t.card
            border.width: 1; border.color: Qt.alpha(t.accent, 0.5)
            onVisibleChanged: if (visible) { newName.text = ""; newName.forceActiveFocus() }
            RowLayout {
                anchors { fill: parent; leftMargin: 12; rightMargin: 12 }
                spacing: 8
                Text { text: side.newKind === "folder" ? "create_new_folder" : "note_add"; color: t.accent; font.family: t.icons; font.pixelSize: 16 }
                TextInput {
                    id: newName
                    Layout.fillWidth: true
                    color: t.text; font.family: t.mono; font.pixelSize: 12; clip: true
                    Keys.onReturnPressed: side.create(text)
                    Keys.onEnterPressed: side.create(text)
                    Keys.onEscapePressed: side.newKind = ""
                    Text { visible: !newName.text; text: side.newKind === "folder" ? "folder name" : "file name (a/b.rs makes the folder)"; color: t.faint; font: newName.font }
                }
            }
        }
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
                Text { visible: modelData.dir; text: parent.parent.shut ? "folder" : "folder_open"; color: parent.parent.shut ? t.dim : t.accent; font.family: t.icons; font.pixelSize: 16 }
                    // files: the type's vector icon (a page in its language's colour)
                    Image { visible: !modelData.dir; source: modelData.dir ? "" : side.fileIcon(modelData.name); sourceSize: Qt.size(18, 18); Layout.preferredWidth: 18; Layout.preferredHeight: 18 }
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

        // ── OPEN EDITORS: every open tab, a click shows it; RECENT: the files opened last ──
        component QuickRow: Rectangle {
            property string icon
            property string label
            property string sub
            property bool on: false
            signal hit
            Layout.fillWidth: true
            implicitHeight: 28; radius: t.rRow
            color: on ? Qt.alpha(t.accent, 0.14) : qrH.hovered ? Qt.alpha(t.text, 0.06) : "transparent"
            RowLayout {
                anchors { fill: parent; leftMargin: 10; rightMargin: 10 }
                spacing: 8
                Text { text: parent.parent.icon; color: parent.parent.on ? t.accent : t.faint; font.family: t.icons; font.pixelSize: 15 }
                Text { text: parent.parent.label; color: parent.parent.on ? t.text : t.dim; font.family: t.sans; font.pixelSize: 12; elide: Text.ElideRight; Layout.maximumWidth: 150 }
                Text { Layout.fillWidth: true; text: parent.parent.sub; elide: Text.ElideLeft; color: t.faint; font.family: t.sans; font.pixelSize: 10 }
            }
            HoverHandler { id: qrH; cursorShape: Qt.PointingHandCursor }
            TapHandler { onTapped: parent.hit() }
        }
        readonly property var openEditors: win.tabs.map(i => ({ page: i })).concat(win.fileTabs.map(p => ({ file: p })))
        readonly property var recent: (win.prefs.recentFiles ?? []).filter(p => !win.fileTabs.includes(p) && p.startsWith(app.dir + "/")).slice(0, 6)
        readonly property var allTodos: { if (win.sideMode === "files") return []; win.pages.length; try { return JSON.parse(app.todos()).slice(0, 8) } catch (e) { return [] } }
        // the pages opened last (newest first), then, for the rest, the ones changed last
        readonly property var lastPages: {
            const opened = win.prefs.recentPages ?? []
            const rank = o => { const r = opened.indexOf(o.p.path); return r < 0 ? 1e9 : r }
            return win.pages.map((p, i) => ({ p, i })).filter(o => o.i !== app.current)
                .sort((a, b) => (rank(a) - rank(b)) || ((b.p.modified ?? 0) - (a.p.modified ?? 0))).slice(0, 5)
        }
        SectionHead { visible: parent.allTodos.length > 0; label: "TO DO"; count: parent.allTodos.length; open: side.todoOpen; onToggled: side.todoOpen = !side.todoOpen }
        Repeater {
            model: side.todoOpen ? parent.allTodos : []
            delegate: QuickRow { required property var modelData; icon: "check_box_outline_blank"; label: modelData.text; sub: modelData.title; onHit: { win.openPage(modelData.page); Qt.callLater(() => win.jumpToBlock(modelData.block)) } }
        }
        SectionHead { visible: parent.lastPages.length > 1 && win.sideMode === "pages"; label: "RECENT"; count: parent.lastPages.length; open: side.recentOpen; onToggled: side.recentOpen = !side.recentOpen }
        Repeater {
            model: side.recentOpen && win.sideMode === "pages" && parent.lastPages.length > 1 ? parent.lastPages : []
            delegate: QuickRow { required property var modelData; icon: "history"; label: modelData.p.title || "Untitled"; sub: modelData.p.blocks + " blocks"; onHit: win.openPage(modelData.i) }
        }
        SectionHead { visible: parent.openEditors.length > 1; label: "OPEN EDITORS"; count: parent.openEditors.length; open: side.openOpen; onToggled: side.openOpen = !side.openOpen }
        Repeater {
            model: side.openOpen && parent.openEditors.length > 1 ? parent.openEditors : []
            delegate: QuickRow {
                required property var modelData
                icon: modelData.file ? "code" : "description"
                label: modelData.file ? modelData.file.slice(modelData.file.lastIndexOf("/") + 1) : (win.pages[modelData.page]?.title || "Untitled")
                sub: modelData.file ? modelData.file.replace(app.dir + "/", "").split("/").slice(0, -1).join("/") : "page"
                on: modelData.file ? (win.view === "file" && win.filePath === modelData.file) : (win.view === "page" && app.current === modelData.page)
                onHit: modelData.file ? win.openFile({ path: modelData.file, rel: modelData.file }) : win.openPage(modelData.page)
            }
        }
        readonly property var folders: (win.prefs.recentFolders ?? []).filter(f => f !== app.dir).slice(0, 5)
        SectionHead { visible: parent.folders.length > 0 && win.sideMode === "files"; label: "FOLDERS"; count: parent.folders.length; open: side.foldersOpen; onToggled: side.foldersOpen = !side.foldersOpen; action: "create_new_folder"; onActed: win.openFolderPick() }
        Repeater {
            model: side.foldersOpen && win.sideMode === "files" ? parent.folders : []
            delegate: QuickRow { required property string modelData; icon: "folder"; label: modelData.slice(modelData.lastIndexOf("/") + 1); sub: modelData.replace(win.homeDir, "~").split("/").slice(0, -1).join("/"); onHit: win.openFolder(modelData) }
        }
        SectionHead { visible: parent.recent.length > 0 && win.sideMode !== "pages"; label: "RECENT"; count: parent.recent.length; open: side.recentOpen; onToggled: side.recentOpen = !side.recentOpen }
        Repeater {
            model: side.recentOpen && win.sideMode !== "pages" ? parent.recent : []
            delegate: QuickRow { required property string modelData; icon: "history"; label: modelData.slice(modelData.lastIndexOf("/") + 1); sub: modelData.replace(app.dir + "/", "").split("/").slice(0, -1).join("/"); onHit: win.openFile({ path: modelData, rel: modelData }) }
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
        // the free room goes here, above the foot (in the Explorer the file tree takes it)
        Item { visible: !(win.filesOpen && win.sideMode !== "pages"); Layout.fillHeight: true }
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
