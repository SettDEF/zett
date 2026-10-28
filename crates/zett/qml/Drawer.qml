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
    radius: 17
    color: t.card

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
            if (done) { d.running = false; d.lastCode = code; if (code !== 0) out.text += `(exit ${code})\n` }
            flick.contentY = Math.max(0, flick.contentHeight - flick.height)
        }
    }

    ColumnLayout {
        anchors { fill: parent; margins: 10 }
        spacing: 6
        RowLayout {
            Layout.fillWidth: true
            spacing: 6
            Text { text: "terminal"; color: t.accent; font.family: t.icons; font.pixelSize: 17 }
            Text { text: "Shell"; color: t.text; font.family: t.sans; font.pixelSize: 13; font.weight: Font.DemiBold }
            Text { text: d.running ? "running…" : d.history.length ? (d.lastCode === 0 ? "ok" : `exit ${d.lastCode}`) : app.dir; elide: Text.ElideLeft; Layout.fillWidth: true; color: d.running ? t.accent : d.lastCode === 0 ? t.faint : "#ffb4ab"; font.family: t.mono; font.pixelSize: 11 }
            Repeater {
                // [icon, label, action]
                model: [["stop_circle", "Stop", "stop"], ["delete_sweep", "Clear", "clear"], ["open_in_new", "zash here", "zash"], ["close", "", "close"]]
                delegate: Rectangle {
                    required property var modelData
                    visible: modelData[2] !== "stop" || d.running
                    implicitWidth: bRow.implicitWidth + (modelData[1] ? 20 : 10); implicitHeight: 26; radius: 13
                    color: bh.hovered ? t.raised : "transparent"
                    RowLayout { id: bRow; anchors.centerIn: parent; spacing: 4
                        Text { text: modelData[0]; color: modelData[2] === "stop" ? "#ffb4ab" : t.dim; font.family: t.icons; font.pixelSize: 15 }
                        Text { visible: !!modelData[1]; text: modelData[1]; color: t.dim; font.family: t.sans; font.pixelSize: 12 } }
                    HoverHandler { id: bh; cursorShape: Qt.PointingHandCursor }
                    TapHandler {
                        onTapped: {
                            const a = modelData[2]
                            if (a === "stop") app.stopShell()
                            else if (a === "clear") out.text = ""
                            else if (a === "zash") app.runShell(-1, 'setsid -f "$HOME/.local/bin/zash" >/dev/null 2>&1')
                            else win.drawerShown = false
                        }
                    }
                }
            }
        }
        Flickable {
            id: flick
            Layout.fillWidth: true
            Layout.fillHeight: true
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
