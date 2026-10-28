import QtQuick
import dev.zett.app

// The new code editor (Settings > Editor > New editor core): CodeDoc (Rust) holds the text, cursors, undo and
// folds; this draws only the rows on screen, each coloured by Rust, the cursors from the code font's grid.
Rectangle {
    id: cv
    required property var win
    required property var t
    required property var app
    property alias doc: doc
    radius: 17
    color: t.card
    clip: true

    CodeDoc { id: doc }
    readonly property string file: win.filePath
    onFileChanged: if (file && visible) doc.open(file)
    onVisibleChanged: if (visible && file && doc.path !== file) doc.open(file)

    FontMetrics { id: fm; font.family: cv.t.mono; font.pixelSize: 13 }
    readonly property real cw: fm.advanceWidth("M")
    readonly property real lh: Math.ceil(fm.height * 1.45)
    readonly property real gutter: 64
    readonly property var carets: { try { return JSON.parse(doc.cursors) } catch (e) { return [[0, 0, 0, 0]] } }
    // the first row on screen and how many fit
    property real scrollY: 0
    readonly property int firstRow: Math.max(0, Math.floor(scrollY / lh))
    readonly property int shown: Math.ceil(height / lh) + 1
    function maxTop() { return Math.max(0, doc.rows * lh - height + lh * 4) }
    // keep the main cursor in view
    function follow() {
        const y = doc.lineRow(carets[0][0]) * lh
        if (y < scrollY) scrollY = y
        else if (y + lh > scrollY + height) scrollY = y + lh - height
    }
    Connections { target: doc; function onCursorsChanged() { cv.follow() } }
    function rowColAt(x, y) {
        const row = Math.max(0, Math.min(doc.rows - 1, Math.floor((y + scrollY) / lh)))
        return [doc.rowLine(row), Math.max(0, Math.round((x - gutter - 12) / cw))]
    }
    // clipboard through a hidden text field
    TextEdit { id: clip; visible: false }
    function copy(cut) { const s = doc.selectedText(); if (!s) return; clip.text = s; clip.selectAll(); clip.copy(); if (cut) doc.key(0x01000007, "", 0) }
    function paste() { clip.text = ""; clip.paste(); if (clip.text) doc.insert(clip.text) }

    // the rows on screen
    Repeater {
        model: Math.min(cv.shown, Math.max(0, doc.rows - cv.firstRow))
        delegate: Item {
            required property int index
            readonly property int row: cv.firstRow + index
            readonly property int line: (doc.version, doc.rowLine(row))
            readonly property int fold: (doc.version, doc.foldAt(line))
            readonly property bool cur: cv.carets.some(c => c[0] === line)
            y: row * cv.lh - cv.scrollY
            width: cv.width; height: cv.lh
            Rectangle { visible: parent.cur; anchors.fill: parent; color: Qt.alpha(cv.t.text, 0.04) }
            Text {
                x: 0; width: 40; height: parent.height
                horizontalAlignment: Text.AlignRight; verticalAlignment: Text.AlignVCenter
                text: parent.line + 1
                color: parent.cur ? cv.t.accent : cv.t.faint
                font.family: cv.t.mono; font.pixelSize: 12
            }
            // fold: a chevron for a line that opens a block, "⋯" after one folded
            Text {
                visible: parent.fold > 0
                x: 44; width: 16; height: parent.height; verticalAlignment: Text.AlignVCenter
                text: parent.fold === 2 ? "chevron_right" : "expand_more"
                color: parent.fold === 2 ? cv.t.accent : Qt.alpha(cv.t.faint, fh.hovered || parent.cur ? 1 : 0.4)
                font.family: cv.t.icons; font.pixelSize: 15
                HoverHandler { id: fh; cursorShape: Qt.PointingHandCursor }
                TapHandler { onTapped: doc.toggleFold(parent.parent.line) }
            }
            Text {
                id: code
                x: cv.gutter + 12; height: parent.height; verticalAlignment: Text.AlignVCenter
                text: (doc.version, doc.lineHtml(parent.line))
                textFormat: Text.StyledText
                color: cv.t.text
                font.family: cv.t.mono; font.pixelSize: 13
            }
            Rectangle {
                visible: parent.fold === 2
                x: code.x + code.implicitWidth + 8; anchors.verticalCenter: parent.verticalCenter
                width: 30; height: 16; radius: 8; color: Qt.alpha(cv.t.accent, 0.18)
                Text { anchors.centerIn: parent; text: "⋯"; color: cv.t.accent; font.pixelSize: 12 }
                TapHandler { onTapped: doc.toggleFold(parent.parent.line) }
            }
        }
    }
    // selections: for each cursor, a rounded band per row it covers
    Repeater {
        model: cv.carets.filter(c => c[0] !== c[2] || c[1] !== c[3])
        delegate: Item {
            required property var modelData
            readonly property var a: modelData[0] < modelData[2] || (modelData[0] === modelData[2] && modelData[1] < modelData[3]) ? [modelData[0], modelData[1]] : [modelData[2], modelData[3]]
            readonly property var b: a[0] === modelData[0] && a[1] === modelData[1] ? [modelData[2], modelData[3]] : [modelData[0], modelData[1]]
            Repeater {
                model: Math.min(400, parent.b[0] - parent.a[0] + 1)
                delegate: Rectangle {
                    required property int index
                    readonly property int line: parent.a[0] + index
                    readonly property int row: doc.lineRow(line)
                    visible: doc.rowLine(row) === line && row >= cv.firstRow && row < cv.firstRow + cv.shown
                    readonly property int c0: index === 0 ? parent.a[1] : 0
                    readonly property int c1: line === parent.b[0] ? parent.b[1] : 400
                    x: cv.gutter + 12 + c0 * cv.cw; y: row * cv.lh - cv.scrollY + 2
                    width: Math.max(cv.cw * 0.5, (Math.min(c1, 400) - c0) * cv.cw); height: cv.lh - 4
                    radius: 4; color: Qt.alpha(cv.t.accent, 0.28)
                }
            }
        }
    }
    // the cursors
    Repeater {
        model: cv.carets
        delegate: Rectangle {
            required property var modelData
            required property int index
            x: cv.gutter + 12 + modelData[1] * cv.cw - 1; y: doc.lineRow(modelData[0]) * cv.lh - cv.scrollY + 3
            width: 2; height: cv.lh - 6; radius: 1
            color: cv.t.accent
            visible: input.activeFocus && blink.on
        }
    }
    Timer { id: blink; property bool on: true; interval: 530; repeat: true; running: input.activeFocus; onTriggered: on = !on }
    Connections { target: doc; function onVersionChanged() { blink.on = true; blink.restart() } }

    // input: keys to the core, the mouse places, drags and adds cursors, the wheel scrolls
    Item {
        id: input
        anchors.fill: parent
        focus: true
        Keys.onPressed: (e) => {
            const ctrl = e.modifiers & Qt.ControlModifier
            if (ctrl && e.key === Qt.Key_S) { doc.save(); e.accepted = true; return }
            if (ctrl && e.key === Qt.Key_C) { cv.copy(false); e.accepted = true; return }
            if (ctrl && e.key === Qt.Key_X) { cv.copy(true); e.accepted = true; return }
            if (ctrl && e.key === Qt.Key_V) { cv.paste(); e.accepted = true; return }
            e.accepted = doc.key(e.key, e.text, e.modifiers)
        }
        TapHandler {
            acceptedModifiers: Qt.KeyboardModifierMask
            onTapped: (ev) => {
                input.forceActiveFocus()
                const [l, c] = cv.rowColAt(ev.position.x, ev.position.y)
                doc.click(l, c, (ev.modifiers & Qt.ShiftModifier) !== 0, (ev.modifiers & Qt.AltModifier) !== 0)
            }
        }
        DragHandler {
            target: null
            acceptedModifiers: Qt.NoModifier
            onActiveChanged: if (active) { input.forceActiveFocus(); const [l, c] = cv.rowColAt(centroid.pressPosition.x, centroid.pressPosition.y); doc.click(l, c, false, false) }
            onCentroidChanged: if (active) { const [l, c] = cv.rowColAt(centroid.position.x, centroid.position.y); doc.drag(l, c) }
        }
        WheelHandler {
            onWheel: (ev) => { cv.scrollY = Math.max(0, Math.min(cv.maxTop(), cv.scrollY - ev.angleDelta.y / 120 * cv.lh * 3)) }
        }
        HoverHandler { cursorShape: Qt.IBeamCursor }
    }
    // the scrollbar: a slim rounded thumb
    Rectangle {
        visible: doc.rows * cv.lh > cv.height
        anchors.right: parent.right; anchors.rightMargin: 4
        y: cv.scrollY / Math.max(1, doc.rows * cv.lh) * cv.height
        width: 6; radius: 3
        height: Math.max(30, cv.height * cv.height / Math.max(1, doc.rows * cv.lh))
        color: Qt.alpha(cv.t.text, sbH.hovered ? 0.3 : 0.15)
        HoverHandler { id: sbH }
        DragHandler { target: null; yAxis.enabled: true; xAxis.enabled: false; property real from: 0
            onActiveChanged: if (active) from = cv.scrollY
            onTranslationChanged: if (active) cv.scrollY = Math.max(0, Math.min(cv.maxTop(), from + translation.y * doc.rows * cv.lh / cv.height)) }
    }
}
