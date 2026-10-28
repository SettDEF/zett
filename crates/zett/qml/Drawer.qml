import QtQuick
import QtQuick.Layouts

// The drawer under the page (Ctrl+`): a shell command in the workspace, its output as it comes
// (runShell on its own thread), Stop, history; "zash here" opens a real terminal in the folder.
// Code blocks in sh/bash/zsh run here from their menu.
Rectangle {
    id: d
    required property var win
    required property var t
    required property var app
    property bool running: false
    property int runId: 0
    property int lastCode: 0
    property var history: []
    property int back: -1
    property string tab: "shell"          // shell | problems
    // cargo and rustc style errors in what a command printed: error[E0308]: … / --> src/main.rs:4:9
    function readProblems(text) {
        const found = []; let last = null
        for (const l of text.split("\n")) {
            const h = /^(error|warning)(\[\w+\])?: (.+)/.exec(l)
            if (h) { last = { severity: h[1] === "error" ? 1 : 2, message: h[3] }; continue }
            const at = /^\s*--> ([^:]+):(\d+):(\d+)/.exec(l)
            if (at && last) { found.push({ path: (at[1].startsWith("/") ? "" : app.dir + "/") + at[1], line: +at[2] - 1, col: +at[3] - 1, severity: last.severity, message: last.message, from: "task" }); last = null }
        }
        win.taskDiags = found
    }
    radius: 17
    color: t.card
    // docked at the bottom: drag the top edge to set its height (kept)
    Rectangle {
        anchors { horizontalCenter: parent.horizontalCenter; top: parent.top; topMargin: 4 }
        width: 44; height: 4; radius: 2; z: 5
        color: Qt.alpha(t.text, gripH.hovered || gripD.active ? 0.35 : 0.14)
    }
    Item {
        anchors { left: parent.left; right: parent.right; top: parent.top }
        height: 10; z: 6
        HoverHandler { id: gripH; cursorShape: Qt.SizeVerCursor }
        DragHandler {
            id: gripD; target: null; yAxis.enabled: true; xAxis.enabled: false
            property real from: 0
            onActiveChanged: if (active) from = d.height; else d.win.setPref("drawerHeight", Math.round(d.height))
            onTranslationChanged: if (active) d.win.drawerH = Math.max(120, from - translation.y)
        }
    }

    function run(cmd) {
        cmd = (cmd ?? "").trim()
        if (!cmd || running) return
        history = [cmd].concat(history.filter(h => h !== cmd)).slice(0, 50); back = -1
        out.text += (out.text ? "\n" : "") + "$ " + cmd + "\n"
        runId++
        running = app.runShell(runId, cmd)
        if (!running) out.text += "(could not start bash)\n"
        win.drawerShown = true
    }
    Connections {
        target: d.app
        function onRunOutput(id, chunk, done, code) {
            if (id !== d.runId) return
            if (chunk.length) out.text = (out.text + chunk).slice(-200000)   // the last 200 kB
            if (done) { d.running = false; d.lastCode = code; d.readProblems(out.text.slice(Math.max(0, out.text.lastIndexOf("\n$ ")))); if (code !== 0) out.text += `(exit ${code})\n` }
            flick.contentY = Math.max(0, flick.contentHeight - flick.height)
        }
    }

    ColumnLayout {
        anchors { fill: parent; margins: 10 }
        spacing: 6
        RowLayout {
            Layout.fillWidth: true
            spacing: 6
            // the panel's tabs, a segmented control
            Rectangle {
                implicitWidth: segRow.implicitWidth + 6; implicitHeight: 28; radius: 14; color: t.raised
                Row {
                    id: segRow; anchors.centerIn: parent; spacing: 2
                    Repeater {
                        model: [["shell", "terminal", "Terminal"], ["problems", "error", d.win.problems.length ? `Problems  ${d.win.problems.length}` : "Problems"]]
                        delegate: Rectangle {
                            required property var modelData
                            readonly property bool on: d.tab === modelData[0]
                            width: segT.implicitWidth + 34; height: 24; radius: 12
                            color: on ? Qt.alpha(t.accent, 0.22) : segH.hovered ? Qt.alpha(t.text, 0.06) : "transparent"
                            Row { anchors.centerIn: parent; spacing: 5
                                Text { text: modelData[1]; color: parent.parent.on ? t.accent : t.faint; font.family: t.icons; font.pixelSize: 14 }
                                Text { id: segT; text: modelData[2]; color: parent.parent.on ? t.text : t.dim; font.family: t.sans; font.pixelSize: 12; font.weight: parent.parent.on ? Font.DemiBold : Font.Normal } }
                            HoverHandler { id: segH; cursorShape: Qt.PointingHandCursor }
                            TapHandler { onTapped: d.tab = modelData[0] }
                        }
                    }
                }
            }
            Text { text: d.running ? "running…" : d.history.length ? (d.lastCode === 0 ? "ok" : `exit ${d.lastCode}`) : app.dir; elide: Text.ElideLeft; Layout.fillWidth: true; color: d.running ? t.accent : d.lastCode === 0 ? t.faint : t.removed; font.family: t.mono; font.pixelSize: 11 }
            // the panel's buttons: icons in one soft pill (pin, maximize, clear, zash, close), the name on hover
            Rectangle {
                implicitWidth: pillRow.implicitWidth + 8; implicitHeight: 34; radius: 17
                color: Qt.alpha(t.text, 0.05)
                Row {
                    id: pillRow; anchors.centerIn: parent; spacing: 2
                    Repeater {
                        // [icon, name, action, on]
                        model: [["stop_circle", "Stop", "stop", false], ["keep", d.win.drawerPinned ? "Unpin" : "Pin: stays open, also at start", "pin", d.win.drawerPinned], [d.win.drawerMax ? "close_fullscreen" : "open_in_full", d.win.drawerMax ? "Restore" : "Maximize", "max", d.win.drawerMax], ["delete_sweep", "Clear", "clear", false], ["open_in_new", "zash here", "zash", false], ["close", "Close  Ctrl `", "close", false]]
                        delegate: Rectangle {
                            required property var modelData
                            visible: modelData[2] !== "stop" || d.running
                            width: 32; height: 28; radius: 14
                            color: modelData[3] ? Qt.alpha(t.accent, 0.2) : bh.hovered ? Qt.alpha(t.text, 0.08) : "transparent"
                            Text { anchors.centerIn: parent; text: modelData[0]; color: modelData[2] === "stop" ? t.removed : modelData[3] ? t.accent : bh.hovered ? t.text : t.dim; font.family: t.icons; font.pixelSize: 17 }
                            HoverHandler { id: bh; cursorShape: Qt.PointingHandCursor }
                            Rectangle {
                                visible: bh.hovered; z: 100
                                anchors { bottom: parent.top; bottomMargin: 6; horizontalCenter: parent.horizontalCenter }
                                width: tipT.implicitWidth + 14; height: 22; radius: 8
                                color: t.raised; border.width: 1; border.color: Qt.alpha(t.rule, 0.4)
                                Text { id: tipT; anchors.centerIn: parent; text: modelData[1]; color: t.text; font.family: t.sans; font.pixelSize: 11 }
                            }
                            TapHandler {
                                onTapped: {
                                    const a = modelData[2]
                                    if (a === "stop") app.stopShell()
                                    else if (a === "pin") d.win.setPref("drawerPinned", !d.win.drawerPinned)
                                    else if (a === "max") d.win.drawerMax = !d.win.drawerMax
                                    else if (a === "clear") out.text = ""
                                    else if (a === "zash") d.win.openZash()
                                    else { d.win.drawerShown = false; d.win.drawerMax = false }
                                }
                            }
                        }
                    }
                }
            }
        }
        // Problems: every file's, a click opens the file there
        ListView {
            visible: d.tab === "problems"
            Layout.fillWidth: true; Layout.fillHeight: true
            clip: true; boundsBehavior: Flickable.StopAtBounds
            model: d.win.problems
            delegate: Rectangle {
                required property var modelData
                width: ListView.view.width; height: 28; radius: 8
                color: pbH.hovered ? Qt.alpha(t.text, 0.06) : "transparent"
                Row {
                    x: 8; anchors.verticalCenter: parent.verticalCenter; spacing: 8
                    Text { text: modelData.severity === 1 ? "error" : "warning"; color: modelData.severity === 1 ? t.removed : "#d8a54a"; font.family: t.icons; font.pixelSize: 16 }
                    Text { text: modelData.path.replace(d.app.dir + "/", "") + `:${modelData.line + 1}`; color: t.dim; font.family: t.mono; font.pixelSize: 11 }
                    Text { text: modelData.message.split("\n")[0]; color: t.text; font.family: t.sans; font.pixelSize: 12 }
                    Text { text: modelData.from; color: t.faint; font.family: t.sans; font.pixelSize: 10 }
                }
                HoverHandler { id: pbH; cursorShape: Qt.PointingHandCursor }
                TapHandler { onTapped: d.win.openFileAt({ path: modelData.path, rel: modelData.path }, modelData.line, modelData.col) }
            }
            Text { visible: !d.win.problems.length; anchors.centerIn: parent; text: "No problems"; color: t.faint; font.family: t.sans; font.pixelSize: 13 }
        }
        Flickable {
            id: flick
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: d.tab === "shell"
            clip: true
            contentHeight: out.implicitHeight
            boundsBehavior: Flickable.StopAtBounds
            TextEdit {
                id: out
                width: flick.width
                readOnly: true
                selectByMouse: true
                wrapMode: TextEdit.WrapAnywhere
                color: t.dim; selectionColor: Qt.alpha(t.accent, 0.35)
                font.family: t.mono; font.pixelSize: 12
            }
        }
        Rectangle {
            visible: d.tab === "shell"
            Layout.fillWidth: true
            implicitHeight: 32; radius: 10
            color: t.raised
            RowLayout {
                anchors { fill: parent; leftMargin: 10; rightMargin: 10 }
                spacing: 8
                Text { text: "$"; color: t.accent; font.family: t.mono; font.pixelSize: 13 }
                TextInput {
                    id: field
                    Layout.fillWidth: true
                    color: t.text; font.family: t.mono; font.pixelSize: 13
                    clip: true
                    Text { visible: !parent.text; text: d.running ? "running… (Stop above)" : "a command in the workspace, Enter runs it"; color: t.faint; font: parent.font }
                    Keys.onPressed: (e) => {
                        if (e.key === Qt.Key_Return || e.key === Qt.Key_Enter) { d.run(text); text = ""; e.accepted = true }
                        else if (e.key === Qt.Key_Up && d.history.length) { d.back = Math.min(d.history.length - 1, d.back + 1); text = d.history[d.back]; e.accepted = true }
                        else if (e.key === Qt.Key_Down) { d.back = Math.max(-1, d.back - 1); text = d.back >= 0 ? d.history[d.back] : ""; e.accepted = true }
                        else if (e.key === Qt.Key_C && (e.modifiers & Qt.ControlModifier) && d.running) { app.stopShell(); e.accepted = true }
                        else if (e.key === Qt.Key_Escape) { win.drawerShown = false; e.accepted = true }
                    }
                }
            }
        }
    }
    function focusInput() { field.forceActiveFocus() }
}
