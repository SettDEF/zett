import QtQuick
import QtQuick.Layouts

// The drawer's Debug tab: a program run under a debug adapter (gdb -i=dap: Rust, C, C++) with the breakpoints
// set in the gutter; when it stops: the call stack, the variables (a click opens one), the line marked in the editor.
Rectangle {
    id: dbg
    required property var win
    required property var app
    required property var t
    color: "transparent"

    property int sid: -1
    property string status: "idle"        // idle | starting | running | stopped
    property string note: ""
    property int threadId: 1
    property var frames: []
    property int frameSel: 0
    property var vars: []                 // [{name, value, type, ref, depth}]
    property var pending: ({})            // seq → [command, extra]
    property string output: ""
    property alias progText: progField.text

    // the program: the Cargo project's binary, else what is typed
    function guess() {
        const name = app.dir.slice(app.dir.lastIndexOf("/") + 1)
        return app.readFile(app.dir + "/Cargo.toml").length ? `target/debug/${name}` : ""
    }
    function ask(cmd, args, extra) { const s = app.dapRequest(sid, cmd, JSON.stringify(args ?? {})); const p = Object.assign({}, pending); p[s] = [cmd, extra]; pending = p; return s }
    function start() {
        if (sid >= 0) stop()
        const prog = progField.text.trim()
        if (!prog) { note = "Which program? (a built binary, e.g. target/debug/name)"; return }
        sid = app.dapStart(JSON.stringify(["gdb", "-i=dap"]), app.dir)
        if (sid < 0) { note = "gdb did not start (gdb 14 or newer has DAP)"; return }
        status = "starting"; note = ""; output = ""; frames = []; vars = []
        ask("initialize", { adapterID: "zett", clientName: "zett", linesStartAt1: true, columnsStartAt1: true, pathFormat: "path" })
    }
    function stop() {
        if (sid >= 0) { app.dapRequest(sid, "disconnect", JSON.stringify({ terminateDebuggee: true })); app.dapStop(sid) }
        sid = -1; status = "idle"; frames = []; vars = []; win.debugAt = ({})
    }
    function setBreakpoints(path) {
        const lines = (win.breakpoints[path] ?? []).map(l => ({ line: l + 1 }))
        ask("setBreakpoints", { source: { path }, breakpoints: lines })
    }
    // the gutter changed one: sent at once while a session runs
    Connections { target: dbg.win; function onBreakpointsChanged() { if (dbg.sid >= 0 && dbg.win.lastBreakpointFile) dbg.setBreakpoints(dbg.win.lastBreakpointFile) } }

    function step(cmd) { if (sid >= 0 && status === "stopped") { status = "running"; win.debugAt = ({}); ask(cmd, { threadId }) } }
    function showFrame(i) {
        frameSel = i
        const f = frames[i]; if (!f) return
        if (f.source && f.source.path) { win.debugAt = { path: f.source.path, line: f.line - 1 }; win.openFileAt({ path: f.source.path, rel: f.source.path }, f.line - 1, 0) }
        vars = []
        ask("scopes", { frameId: f.id })
    }
    function expand(i) {
        const v = vars[i]
        if (!v.ref) return
        if (v.open) { // close: the children go
            const out = vars.slice(0, i + 1); let j = i + 1
            while (j < vars.length && vars[j].depth > v.depth) j++
            out[i] = Object.assign({}, v, { open: false }); vars = out.concat(vars.slice(j)); return
        }
        ask("variables", { variablesReference: v.ref }, { at: i, depth: v.depth + 1 })
    }

    function handle(m) {
        if (m.type === "gone") { note = note || "The debugger ended"; stop(); return }
        if (m.type === "event") {
            const b = m.body ?? {}
            if (m.event === "initialized") {
                for (const p of Object.keys(win.breakpoints)) setBreakpoints(p)
                ask("configurationDone", {})
            } else if (m.event === "stopped") {
                status = "stopped"; threadId = b.threadId ?? threadId
                note = b.reason === "breakpoint" ? "Stopped at a breakpoint" : b.reason === "step" ? "" : `Stopped (${b.reason ?? "pause"})${b.text ? ": " + b.text : ""}`
                ask("stackTrace", { threadId, startFrame: 0, levels: 40 })
            } else if (m.event === "continued") { status = "running" }
            else if (m.event === "output") { output = (output + (b.output ?? "")).slice(-20000) }
            else if (m.event === "exited") { note = `The program exited with ${b.exitCode}` }
            else if (m.event === "terminated") { stop() }
            return
        }
        if (m.type !== "response") return
        const p = pending[m.request_seq] ?? [m.command]
        const cmd = p[0], extra = p[1]
        if (!m.success) { if (cmd !== "disconnect") note = `${cmd}: ${m.message ?? "failed"}`; if (cmd === "launch") stop(); return }
        const b = m.body ?? {}
        if (cmd === "initialize") {
            const args = argsField.text.trim().length ? argsField.text.trim().split(/\s+/) : []
            const prog = progField.text.trim()
            ask("launch", { program: prog.startsWith("/") ? prog : app.dir + "/" + prog, args, cwd: app.dir })
            status = "running"
        } else if (cmd === "stackTrace") {
            frames = b.stackFrames ?? []
            showFrame(0)
        } else if (cmd === "scopes") {
            const sc = (b.scopes ?? []).filter(s => !s.expensive)
            vars = sc.map(s => ({ name: s.name, value: "", type: "", ref: s.variablesReference, depth: 0, open: false, scope: true }))
            if (sc.length) expand(0)
        } else if (cmd === "variables") {
            const kids = (b.variables ?? []).map(v => ({ name: v.name, value: v.value, type: v.type ?? "", ref: v.variablesReference, depth: extra.depth, open: false }))
            const out = vars.slice(0, extra.at + 1)
            out[extra.at] = Object.assign({}, vars[extra.at], { open: true })
            vars = out.concat(kids, vars.slice(extra.at + 1))
        } else if (cmd === "evaluate") {
            output += `\n› ${extra.expr}\n${b.result}\n`
        }
    }
    Timer { interval: 40; repeat: true; running: dbg.sid >= 0; onTriggered: { let list = []; try { list = JSON.parse(dbg.app.dapPoll(dbg.sid)) } catch (e) {} for (const m of list) dbg.handle(m) } }
    Component.onDestruction: stop()

    ColumnLayout {
        anchors.fill: parent
        spacing: 8
        // the program, its arguments, and the controls
        RowLayout {
            Layout.fillWidth: true
            spacing: 6
            Rectangle {
                Layout.fillWidth: true; implicitHeight: 30; radius: 15; color: dbg.t.raised
                TextInput {
                    id: progField
                    anchors { fill: parent; leftMargin: 12; rightMargin: 12 }
                    verticalAlignment: TextInput.AlignVCenter; clip: true
                    text: dbg.guess()
                    color: dbg.t.text; font.family: dbg.t.mono; font.pixelSize: 12
                    onAccepted: dbg.start()
                    Text { visible: !progField.text; anchors.verticalCenter: parent.verticalCenter; text: "Program (built with debug info)"; color: dbg.t.faint; font: progField.font }
                }
            }
            Rectangle {
                implicitWidth: 160; implicitHeight: 30; radius: 15; color: dbg.t.raised
                TextInput {
                    id: argsField
                    anchors { fill: parent; leftMargin: 12; rightMargin: 12 }
                    verticalAlignment: TextInput.AlignVCenter; clip: true
                    color: dbg.t.text; font.family: dbg.t.mono; font.pixelSize: 12
                    onAccepted: dbg.start()
                    Text { visible: !argsField.text; anchors.verticalCenter: parent.verticalCenter; text: "Arguments"; color: dbg.t.faint; font: argsField.font }
                }
            }
            Repeater {
                model: [["play_arrow", "Start  F5", () => dbg.status === "stopped" ? dbg.step("continue") : dbg.start(), true],
                        ["redo", "Step over  F10", () => dbg.step("next"), dbg.status === "stopped"],
                        ["south", "Step into  F11", () => dbg.step("stepIn"), dbg.status === "stopped"],
                        ["north", "Step out  ⇧F11", () => dbg.step("stepOut"), dbg.status === "stopped"],
                        ["pause", "Pause", () => dbg.ask("pause", { threadId: dbg.threadId }), dbg.status === "running"],
                        ["stop", "Stop  ⇧F5", () => dbg.stop(), dbg.sid >= 0]]
                delegate: Rectangle {
                    required property var modelData
                    implicitWidth: 30; implicitHeight: 30; radius: 15
                    opacity: modelData[3] ? 1 : 0.35
                    color: cH.hovered && modelData[3] ? dbg.t.raised : "transparent"
                    Text { anchors.centerIn: parent; text: modelData[0]; color: modelData[0] === "stop" ? dbg.t.removed : modelData[0] === "play_arrow" ? dbg.t.added : dbg.t.dim; font.family: dbg.t.icons; font.pixelSize: 18 }
                    HoverHandler { id: cH; cursorShape: Qt.PointingHandCursor }
                    TapHandler { onTapped: if (modelData[3]) modelData[2]() }
                }
            }
        }
        Text {
            Layout.fillWidth: true
            text: dbg.note || ({ idle: Object.keys(dbg.win.breakpoints).length ? "Ready: F5 starts the program, it stops at the breakpoints" : "Click left of a line number to set a breakpoint, then F5", starting: "Starting…", running: "Running…", stopped: "Stopped" })[dbg.status]
            color: dbg.status === "stopped" ? dbg.t.accent : dbg.t.faint; font.family: dbg.t.sans; font.pixelSize: 12; elide: Text.ElideRight
        }
        RowLayout {
            Layout.fillWidth: true; Layout.fillHeight: true
            spacing: 8
            // the call stack
            Rectangle {
                Layout.preferredWidth: parent.width * 0.38; Layout.fillHeight: true
                radius: 12; color: dbg.t.card
                ListView {
                    anchors { fill: parent; margins: 6 }
                    clip: true; boundsBehavior: Flickable.StopAtBounds
                    model: dbg.frames
                    header: Text { text: "CALL STACK"; color: dbg.t.faint; font.family: dbg.t.sans; font.pixelSize: 10; font.weight: Font.DemiBold; font.letterSpacing: 0.8; bottomPadding: 4 }
                    delegate: Rectangle {
                        required property var modelData
                        required property int index
                        width: ListView.view.width; height: 24; radius: 8
                        color: index === dbg.frameSel ? Qt.alpha(dbg.t.accent, 0.16) : fH.hovered ? Qt.alpha(dbg.t.text, 0.06) : "transparent"
                        Row { x: 8; anchors.verticalCenter: parent.verticalCenter; spacing: 8
                            Text { text: modelData.name; color: dbg.t.text; font.family: dbg.t.mono; font.pixelSize: 12 }
                            Text { text: (modelData.source ? (modelData.source.name ?? "") : "") + (modelData.line ? ":" + modelData.line : ""); color: dbg.t.faint; font.family: dbg.t.mono; font.pixelSize: 11 } }
                        HoverHandler { id: fH; cursorShape: Qt.PointingHandCursor }
                        TapHandler { onTapped: dbg.showFrame(index) }
                    }
                }
            }
            // the variables, a click opens a struct or a list
            Rectangle {
                Layout.fillWidth: true; Layout.fillHeight: true
                radius: 12; color: dbg.t.card
                ListView {
                    anchors { fill: parent; margins: 6 }
                    clip: true; boundsBehavior: Flickable.StopAtBounds
                    model: dbg.vars
                    header: Text { text: dbg.output.length ? "VARIABLES · output below" : "VARIABLES"; color: dbg.t.faint; font.family: dbg.t.sans; font.pixelSize: 10; font.weight: Font.DemiBold; font.letterSpacing: 0.8; bottomPadding: 4 }
                    delegate: Rectangle {
                        required property var modelData
                        required property int index
                        width: ListView.view.width; height: 22; radius: 6
                        color: vH.hovered && modelData.ref ? Qt.alpha(dbg.t.text, 0.06) : "transparent"
                        Row { x: 6 + modelData.depth * 14; anchors.verticalCenter: parent.verticalCenter; spacing: 6
                            Text { width: 12; text: modelData.ref ? (modelData.open ? "expand_more" : "chevron_right") : ""; color: dbg.t.faint; font.family: dbg.t.icons; font.pixelSize: 13 }
                            Text { text: modelData.name; color: modelData.scope ? dbg.t.faint : dbg.t.accent; font.family: modelData.scope ? dbg.t.sans : dbg.t.mono; font.pixelSize: modelData.scope ? 11 : 12; font.weight: modelData.scope ? Font.DemiBold : Font.Normal }
                            Text { visible: !modelData.scope; text: "= " + modelData.value; color: dbg.t.text; font.family: dbg.t.mono; font.pixelSize: 12; elide: Text.ElideRight; width: Math.max(40, dbg.width * 0.35) }
                            Text { visible: !modelData.scope && modelData.type.length > 0; text: modelData.type; color: dbg.t.faint; font.family: dbg.t.mono; font.pixelSize: 10 } }
                        HoverHandler { id: vH; cursorShape: modelData.ref ? Qt.PointingHandCursor : Qt.ArrowCursor }
                        TapHandler { onTapped: dbg.expand(index) }
                    }
                    footer: Text { visible: dbg.output.length > 0; width: parent ? parent.width : 0; topPadding: 8; wrapMode: Text.WrapAnywhere; text: dbg.output.slice(-3000); color: dbg.t.dim; font.family: dbg.t.mono; font.pixelSize: 11 }
                }
            }
        }
    }
}
