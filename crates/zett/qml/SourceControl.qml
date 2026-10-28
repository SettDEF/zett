import QtQuick
import QtQuick.Layouts

// Source Control, as VS Code's: the branch, a message and Commit, the staged and the changed files (a click opens
// one, + stages, − unstages). git runs in the workspace through app.git.
Rectangle {
    id: sc
    required property var win
    required property var t
    required property var app
    Layout.fillHeight: true
    visible: !win.sidebarHidden && win.sideMode === "git"
    implicitWidth: 268
    color: "transparent"

    property string branch: ""
    property var staged: []
    property var changes: []
    property string note: ""
    property bool isRepo: true
    function git(args) { try { return JSON.parse(app.git(JSON.stringify(args))) } catch (e) { return { code: -1, out: "" } } }
    function refresh() {
        const r = git(["status", "--porcelain=v1", "-b", "-uall"])
        isRepo = r.code === 0
        if (!isRepo) { staged = []; changes = []; return }
        const st = [], ch = []
        for (const l of r.out.split("\n")) {
            if (l.startsWith("## ")) { branch = l.slice(3).split("...")[0]; continue }
            if (l.length < 4) continue
            const x = l[0], y = l[1], path = l.slice(3).split(" -> ").pop()
            if (x !== " " && x !== "?") st.push({ path, code: x })
            if (y !== " ") ch.push({ path, code: x === "?" ? "U" : y })
        }
        staged = st; changes = ch
    }
    function run(args, done) { const r = git(args); if (r.code !== 0) note = r.out.trim().split("\n").pop(); else if (done) note = done; refresh(); noteClear.restart() }
    function commit() {
        const msg = msgField.text.trim()
        if (!msg) { note = "Write a message first"; noteClear.restart(); return }
        // nothing staged: every change goes in, as VS Code offers
        if (!staged.length) git(["add", "-A"])
        run(["commit", "-m", msg], "Committed")
        if (note === "Committed") msgField.text = ""
    }
    onVisibleChanged: if (visible) refresh()
    Timer { interval: 3000; running: sc.visible && sc.win.active; repeat: true; onTriggered: sc.refresh() }
    Timer { id: noteClear; interval: 4000; onTriggered: sc.note = "" }

    component Head: RowLayout {
        property string label
        property int count
        property string action
        signal acted
        Layout.fillWidth: true; Layout.leftMargin: 8; Layout.topMargin: 12
        Text { text: parent.label; color: sc.t.faint; font.family: sc.t.sans; font.pixelSize: 11; font.weight: Font.DemiBold; font.letterSpacing: 0.8 }
        Text { text: parent.count; color: sc.t.faint; font.family: sc.t.mono; font.pixelSize: 11 }
        Item { Layout.fillWidth: true; implicitHeight: 22 }
        Rectangle {
            visible: parent.count > 0
            implicitWidth: 24; implicitHeight: 22; radius: 11
            color: hh.hovered ? sc.t.raised : "transparent"
            Text { anchors.centerIn: parent; text: parent.parent.action; color: sc.t.dim; font.family: sc.t.icons; font.pixelSize: 16 }
            HoverHandler { id: hh; cursorShape: Qt.PointingHandCursor }
            TapHandler { onTapped: parent.parent.acted() }
        }
    }
    component Row_: Rectangle {
        id: row
        property var item
        property bool canDiscard: false
        property bool armed: false        // discard asks once: the first click arms it (red), a second within 3 s throws the change away
        signal discarded
        Timer { id: disarm; interval: 3000; onTriggered: row.armed = false }
        property string action
        signal acted
        Layout.fillWidth: true
        implicitHeight: 30; radius: sc.t.rRow
        color: rh.hovered ? Qt.alpha(sc.t.text, 0.06) : "transparent"
        readonly property color tint: item.code === "D" ? sc.t.removed : item.code === "A" || item.code === "U" ? sc.t.added : sc.t.accent
        RowLayout {
            anchors { fill: parent; leftMargin: 10; rightMargin: 6 }
            spacing: 8
            Text { text: "draft"; color: sc.t.faint; font.family: sc.t.icons; font.pixelSize: 15 }
            Text { text: row.item.path.slice(row.item.path.lastIndexOf("/") + 1); color: row.item.code === "D" ? sc.t.faint : sc.t.text; font.strikeout: row.item.code === "D"; font.family: sc.t.mono; font.pixelSize: 12 }
            Text { Layout.fillWidth: true; text: row.item.path.includes("/") ? row.item.path.slice(0, row.item.path.lastIndexOf("/")) : ""; elide: Text.ElideLeft; color: sc.t.faint; font.family: sc.t.sans; font.pixelSize: 11 }
            Rectangle {
                visible: (rh.hovered || row.armed) && row.canDiscard && row.item.code !== "U"
                implicitWidth: row.armed ? dT.implicitWidth + 30 : 22; implicitHeight: 22; radius: 11
                color: row.armed ? Qt.alpha(sc.t.removed, 0.22) : dh.hovered ? sc.t.raised : "transparent"
                Row { anchors.centerIn: parent; spacing: 4
                    Text { text: "undo"; color: row.armed ? sc.t.removed : sc.t.dim; font.family: sc.t.icons; font.pixelSize: 16 }
                    Text { id: dT; visible: row.armed; text: "Discard?"; color: sc.t.removed; font.family: sc.t.sans; font.pixelSize: 11 } }
                HoverHandler { id: dh; cursorShape: Qt.PointingHandCursor }
                TapHandler { onTapped: { if (row.armed) { row.armed = false; row.discarded() } else { row.armed = true; disarm.restart() } } }
            }
            Rectangle {
                visible: rh.hovered
                implicitWidth: 22; implicitHeight: 22; radius: 11
                color: ah.hovered ? sc.t.raised : "transparent"
                Text { anchors.centerIn: parent; text: row.action; color: sc.t.dim; font.family: sc.t.icons; font.pixelSize: 16 }
                HoverHandler { id: ah; cursorShape: Qt.PointingHandCursor }
                TapHandler { onTapped: row.acted() }
            }
            // the status letter in a soft pill, as the shell's chips
            Rectangle {
                implicitWidth: 20; implicitHeight: 18; radius: 9
                color: Qt.alpha(row.tint, 0.16)
                Text { anchors.centerIn: parent; text: row.item.code; color: row.tint; font.family: sc.t.mono; font.pixelSize: 10; font.weight: Font.DemiBold }
            }
        }
        HoverHandler { id: rh; cursorShape: Qt.PointingHandCursor }
        // a click opens the file at its changes (an untracked one as it is)
        TapHandler { onTapped: if (row.item.code !== "D") { sc.win.openFile({ path: sc.app.dir + "/" + row.item.path, rel: row.item.path }); if (row.item.code !== "U") Qt.callLater(() => sc.win.fileEditor.showDiff = true) } }
    }

    Rectangle { anchors { fill: parent; topMargin: 8; bottomMargin: 8; rightMargin: 8 } radius: sc.t.rCard; color: sc.t.panel; border.width: 1; border.color: Qt.alpha(sc.t.rule, 0.35) }
    ColumnLayout {
        anchors { fill: parent; margins: 14; topMargin: 18; rightMargin: 20; bottomMargin: 16 }
        spacing: 4
        RowLayout {
            Layout.leftMargin: 6; Layout.bottomMargin: 10
            spacing: 10
            Rectangle {
                implicitWidth: 34; implicitHeight: 34; radius: 10
                color: Qt.alpha(sc.t.accent, 0.18)
                Text { anchors.centerIn: parent; text: "commit"; color: sc.t.accent; font.family: sc.t.icons; font.pixelSize: 20 }
            }
            ColumnLayout {
                spacing: 0
                Text { text: "Source Control"; color: sc.t.text; font.family: sc.t.sans; font.pixelSize: 17; font.weight: Font.DemiBold }
                Text { text: sc.isRepo ? sc.branch : "not a git repository"; color: sc.t.faint; font.family: sc.t.mono; font.pixelSize: 11 }
            }
        }
        // the message and Commit
        Rectangle {
            visible: sc.isRepo
            Layout.fillWidth: true
            implicitHeight: Math.max(38, msgField.implicitHeight + 18)
            radius: 18; color: sc.t.card
            border.width: msgField.activeFocus ? 1 : 0; border.color: Qt.alpha(sc.t.accent, 0.5)
            TextEdit {
                id: msgField
                anchors { left: parent.left; right: parent.right; verticalCenter: parent.verticalCenter; leftMargin: 14; rightMargin: 14 }
                wrapMode: TextEdit.Wrap
                color: sc.t.text; font.family: sc.t.sans; font.pixelSize: 13
                selectionColor: Qt.alpha(sc.t.accent, 0.35)
                Keys.onPressed: (e) => { if ((e.key === Qt.Key_Return || e.key === Qt.Key_Enter) && (e.modifiers & Qt.ControlModifier)) { sc.commit(); e.accepted = true } }
                Text { visible: !msgField.text; text: "Message (Ctrl Enter commits)"; color: sc.t.faint; font: msgField.font }
            }
        }
        Rectangle {
            visible: sc.isRepo
            Layout.fillWidth: true; Layout.topMargin: 6
            implicitHeight: 36; radius: 18
            readonly property int n: sc.staged.length || sc.changes.length
            opacity: n ? 1 : 0.5
            color: cH.hovered && n ? Qt.lighter(sc.t.accent, 1.08) : sc.t.accent
            Row {
                anchors.centerIn: parent; spacing: 6
                Text { anchors.verticalCenter: parent.verticalCenter; text: "check"; color: sc.t.onAccent; font.family: sc.t.icons; font.pixelSize: 17 }
                Text { anchors.verticalCenter: parent.verticalCenter; text: sc.staged.length ? `Commit ${sc.staged.length}` : sc.changes.length ? `Commit all ${sc.changes.length}` : "Nothing to commit"; color: sc.t.onAccent; font.family: sc.t.sans; font.pixelSize: 13; font.weight: Font.DemiBold }
            }
            HoverHandler { id: cH; cursorShape: Qt.PointingHandCursor }
            TapHandler { onTapped: if (parent.n) sc.commit() }
        }
        Text { visible: sc.note.length > 0; Layout.leftMargin: 8; Layout.fillWidth: true; wrapMode: Text.Wrap; text: sc.note; color: sc.t.accent; font.family: sc.t.sans; font.pixelSize: 12 }
        Flickable {
            Layout.fillWidth: true; Layout.fillHeight: true
            clip: true
            contentHeight: lists.implicitHeight
            boundsBehavior: Flickable.StopAtBounds
            ColumnLayout {
                id: lists
                width: parent.width
                spacing: 2
                Head { visible: sc.staged.length > 0; label: "STAGED"; count: sc.staged.length; action: "remove"; onActed: sc.run(["reset", "-q"]) }
                Repeater { model: sc.staged; delegate: Row_ { required property var modelData; item: modelData; action: "remove"; onActed: sc.run(["reset", "-q", "--", modelData.path]) } }
                Head { label: "CHANGES"; count: sc.changes.length; action: "add"; onActed: sc.run(["add", "-A"]) }
                Repeater { model: sc.changes; delegate: Row_ { required property var modelData; item: modelData; action: "add"; onActed: sc.run(["add", "--", modelData.path]); canDiscard: true; onDiscarded: sc.run(["checkout", "--", modelData.path], "Discarded") } }
                Text { visible: sc.isRepo && !sc.changes.length && !sc.staged.length; Layout.leftMargin: 10; Layout.topMargin: 6; text: "No changes"; color: sc.t.faint; font.family: sc.t.sans; font.pixelSize: 12 }
            }
        }
    }
}
