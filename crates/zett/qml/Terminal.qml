import QtQuick

// A real terminal in the panel: a shell in a pty on zash's engine (term.rs). The screen comes as rows of rich
// text, redrawn when it changed; keys, paste and the wheel go to the shell.
Rectangle {
    id: tm
    required property var app
    required property var t
    property string cwd: app.dir
    property int sid: -1
    property int seen: -1
    property var scr: ({ rows: [], cursor: [0, 0], show: false })
    readonly property real cw: fm.advanceWidth("M")
    readonly property real ch: Math.ceil(fm.height)
    readonly property int cols: Math.max(10, Math.floor((width - 16) / cw))
    readonly property int lines: Math.max(3, Math.floor((height - 12) / ch))
    signal exited()
    color: scr.bg ?? t.card
    radius: 12
    clip: true

    FontMetrics { id: fm; font.family: tm.t.mono; font.pixelSize: 12 }
    function start() {
        if (sid >= 0 || cw <= 0 || width < 200 || height < 40) return
        sid = app.termStart(cwd, cols, lines, JSON.stringify({ bg: String(t.card), fg: String(t.text) }))
        keys.forceActiveFocus()
    }
    function restart() { if (sid >= 0) app.termClose(sid); sid = -1; seen = -1; ended = false; start() }
    Component.onCompleted: Qt.callLater(start)
    Component.onDestruction: if (sid >= 0) app.termClose(sid)
    // the first size starts it; later ones resize it
    property bool ended: false
    onColsChanged: if (sid >= 0) app.termResize(sid, cols, lines); else if (!ended) start()
    onLinesChanged: if (sid >= 0) app.termResize(sid, cols, lines); else if (!ended) start()
    onVisibleChanged: if (visible) { start(); keys.forceActiveFocus() }

    // redrawn only when the terminal moved on
    Timer {
        interval: 16; repeat: true; running: tm.sid >= 0 && tm.visible
        onTriggered: {
            const r = tm.app.termRev(tm.sid)
            if (r === tm.seen) return
            tm.seen = r
            try { tm.scr = JSON.parse(tm.app.termScreen(tm.sid)) } catch (e) {}
            if (tm.scr.dead) { tm.app.termClose(tm.sid); tm.sid = -1; tm.ended = true; tm.exited() }
        }
    }

    Column {
        x: 8; y: 6
        Repeater {
            model: tm.scr.rows
            delegate: Text {
                required property string modelData
                height: tm.ch
                text: modelData
                textFormat: Text.RichText
                color: tm.scr.fg ?? tm.t.text
                font: fm.font
            }
        }
    }
    // the cursor: a block, hollow when the panel does not have the keyboard
    Rectangle {
        visible: tm.scr.show ?? false
        x: 8 + tm.scr.cursor[1] * tm.cw; y: 6 + tm.scr.cursor[0] * tm.ch
        width: tm.cw; height: tm.ch
        color: keys.activeFocus ? Qt.alpha(tm.t.accent, 0.75) : "transparent"
        border.width: keys.activeFocus ? 0 : 1; border.color: tm.t.accent
    }
    Text {
        visible: tm.ended && tm.sid < 0
        anchors.centerIn: parent
        text: "The shell ended · click to start a new one"
        color: tm.t.faint; font.family: tm.t.sans; font.pixelSize: 12
    }

    Item {
        id: keys
        anchors.fill: parent
        focus: true
        Keys.onPressed: (e) => {
            if (tm.sid < 0) return
            // Ctrl+Shift+V pastes, Ctrl+Shift+C is left to the window
            if (e.key === Qt.Key_V && (e.modifiers & Qt.ControlModifier) && (e.modifiers & Qt.ShiftModifier)) { paste.text = ""; paste.paste(); tm.app.termWrite(tm.sid, paste.text); e.accepted = true; return }
            if (e.key === Qt.Key_Shift || e.key === Qt.Key_Control || e.key === Qt.Key_Alt || e.key === Qt.Key_Meta) return
            tm.app.termKey(tm.sid, e.key, e.text, e.modifiers)
            e.accepted = true
        }
    }
    TextEdit { id: paste; visible: false }
    TapHandler { onTapped: { if (tm.sid < 0) tm.restart(); keys.forceActiveFocus() } }
    WheelHandler {
        onWheel: (w) => { if (tm.sid >= 0) tm.app.termScroll(tm.sid, Math.round(w.angleDelta.y / 40)) }
    }
}
