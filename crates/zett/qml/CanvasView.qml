// A .canvas file (JSON Canvas 1.0): cards on an endless board; file cards show the file live, text cards are edited in place
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: cv
    required property var win
    required property var t
    required property var app
    visible: win.view === "canvas"
    Layout.fillWidth: true
    Layout.fillHeight: true

    property string path: ""
    property var nodes: []
    property var edges: []
    property string sel: ""
    property string editing: ""
    // the board: pan (px) and zoom
    property real px: 0
    property real py: 0
    property real zoom: 1
    // a link being drawn from a card's handle: its start card and where the pointer is (board coordinates)
    property string linkFrom: ""
    property point linkTo: Qt.point(0, 0)
    // file cards' text, read again every 2 s while shown (live)
    property var live: ({})
    property int liveTick: 0
    property string note: ""
    // fitted once the view has its size (it is hidden, 0 wide, while the file loads)
    property bool fitted: false
    function fitOnce() { if (!fitted && width > 50 && height > 50) fit() }
    onWidthChanged: fitOnce()
    onHeightChanged: fitOnce()
    onVisibleChanged: fitOnce()
    clip: true

    function load(p) {
        path = p; sel = ""; editing = ""
        let d = {}
        try { d = JSON.parse(app.readFile(p) || "{}") } catch (e) { note = "not a canvas file: " + e }
        nodes = d.nodes ?? []; edges = d.edges ?? []
        px = 0; py = 0; zoom = 1
        fitted = false; Qt.callLater(fitOnce)
        readLive()
    }
    function save() {
        if (!path) return
        app.writeFile(path, JSON.stringify({ nodes: nodes, edges: edges }, null, "\t") + "\n")
    }
    Timer { id: saveLater; interval: 400; onTriggered: cv.save() }
    function changed() { nodes = nodes.slice(); edges = edges.slice(); saveLater.restart() }
    function byId(id) { return nodes.find(n => n.id === id) }
    function newId() { let s = ""; for (let i = 0; i < 16; i++) s += "0123456789abcdef"[Math.floor(Math.random() * 16)]; return s }
    function abs(f) { return f.startsWith("/") ? f : app.dir + "/" + f }
    function readLive() {
        const m = {}
        for (const n of nodes) if (n.type === "file" && n.file) m[n.file] = app.readFile(abs(n.file))
        live = m; liveTick++
    }
    Timer { interval: 2000; repeat: true; running: cv.visible && cv.nodes.some(n => n.type === "file"); onTriggered: cv.readLive() }

    function addText(x, y, text) {
        const n = { id: newId(), type: "text", text: text ?? "", x: Math.round(x), y: Math.round(y), width: 260, height: 140 }
        nodes = nodes.concat([n]); sel = n.id; editing = n.id; saveLater.restart()
        return n
    }
    function addFile(f, x, y) {
        const n = { id: newId(), type: "file", file: f, x: Math.round(x), y: Math.round(y), width: 360, height: 280 }
        nodes = nodes.concat([n]); sel = n.id; readLive(); saveLater.restart()
    }
    function remove(id) {
        nodes = nodes.filter(n => n.id !== id); edges = edges.filter(e => e.fromNode !== id && e.toNode !== id && e.id !== id)
        sel = ""; editing = ""; saveLater.restart()
    }
    function link(a, b) {
        if (!a || !b || a === b || edges.some(e => e.fromNode === a && e.toNode === b)) return
        const na = byId(a), nb = byId(b)
        const right = nb.x > na.x + na.width / 2
        edges = edges.concat([{ id: newId(), fromNode: a, fromSide: right ? "right" : "left", toNode: b, toSide: right ? "left" : "right" }]); saveLater.restart()
    }
    // everything in view, with a margin
    function fit() {
        if (!nodes.length) return
        let x0 = Infinity, y0 = Infinity, x1 = -Infinity, y1 = -Infinity
        for (const n of nodes) { x0 = Math.min(x0, n.x); y0 = Math.min(y0, n.y); x1 = Math.max(x1, n.x + n.width); y1 = Math.max(y1, n.y + n.height) }
        zoom = Math.max(0.15, Math.min(1.5, Math.min(width / (x1 - x0 + 120), height / (y1 - y0 + 120))))
        px = width / 2 - (x0 + x1) / 2 * zoom; py = height / 2 - (y0 + y1) / 2 * zoom
    }
    function zoomAt(f, sx, sy) {
        fitted = true
        const z = Math.max(0.1, Math.min(4, zoom * f))
        px = sx - (sx - px) * z / zoom; py = sy - (sy - py) * z / zoom; zoom = z
    }
    function toBoard(sx, sy) { return Qt.point((sx - px) / zoom, (sy - py) / zoom) }
    // JSON Canvas colours: "1"…"6" are its presets, else a hex
    function colourOf(c) {
        const preset = { "1": t.removed, "2": "#e0a060", "3": "#d8c86a", "4": "#7fbf8a", "5": t.accent, "6": t.changed }
        return !c ? t.rule : preset[c] ?? c
    }
    function side(n, s) {
        switch (s) {
        case "left": return Qt.point(n.x, n.y + n.height / 2)
        case "top": return Qt.point(n.x + n.width / 2, n.y)
        case "bottom": return Qt.point(n.x + n.width / 2, n.y + n.height)
        default: return Qt.point(n.x + n.width, n.y + n.height / 2)
        }
    }

    Rectangle { anchors.fill: parent; color: t.bg; radius: t.rCard }

    // the board: empty space pans (drag), wheel scrolls, Ctrl+wheel / pinch zooms, double-click makes a text card
    MouseArea {
        id: board
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.MiddleButton
        property point last
        onPressed: m => { last = Qt.point(m.x, m.y); cv.fitted = true; cv.sel = ""; cv.editing = ""; cv.forceActiveFocus() }
        onPositionChanged: m => { cv.px += m.x - last.x; cv.py += m.y - last.y; last = Qt.point(m.x, m.y) }
        onDoubleClicked: m => { const p = cv.toBoard(m.x, m.y); cv.addText(p.x - 130, p.y - 70, "") }
        onWheel: w => {
            if (w.modifiers & Qt.ControlModifier) cv.zoomAt(Math.pow(1.0015, w.angleDelta.y), w.x, w.y)
            else { cv.px += w.pixelDelta.x || w.angleDelta.x / 2; cv.py += w.pixelDelta.y || w.angleDelta.y / 2 }
        }
    }
    PinchHandler { target: null; onScaleChanged: d => cv.zoomAt(d, centroid.position.x, centroid.position.y) }
    // a file dropped from the tree or the desktop: a live card of it
    DropArea {
        anchors.fill: parent
        onDropped: d => {
            const p = cv.toBoard(d.x, d.y)
            for (const u of d.urls ?? []) { let f = decodeURIComponent(String(u).replace("file://", "")); if (f.startsWith(app.dir + "/")) f = f.slice(app.dir.length + 1); cv.addFile(f, p.x, p.y) }
        }
    }

    Item {
        id: world
        x: cv.px; y: cv.py; scale: cv.zoom; transformOrigin: Item.TopLeft

        // groups first (under everything), then the links, then the cards
        Repeater {
            model: cv.nodes.filter(n => n.type === "group")
            Rectangle {
                required property var modelData
                x: modelData.x; y: modelData.y; width: modelData.width; height: modelData.height
                radius: t.rCard; color: Qt.alpha(cv.colourOf(modelData.color), 0.08); border.color: Qt.alpha(cv.colourOf(modelData.color), 0.5); border.width: 1.5
                Text { x: 14; y: -22; text: modelData.label ?? ""; color: t.dim; font.family: t.sans; font.pixelSize: 14; font.weight: Font.DemiBold }
            }
        }
        Canvas {
            id: wires
            width: 1; height: 1
            // a canvas as big as the cards' extent, drawn in board coordinates
            property rect box: {
                let x0 = 0, y0 = 0, x1 = 1, y1 = 1
                for (const n of cv.nodes) { x0 = Math.min(x0, n.x); y0 = Math.min(y0, n.y); x1 = Math.max(x1, n.x + n.width); y1 = Math.max(y1, n.y + n.height) }
                if (cv.linkFrom) { x0 = Math.min(x0, cv.linkTo.x); y0 = Math.min(y0, cv.linkTo.y); x1 = Math.max(x1, cv.linkTo.x); y1 = Math.max(y1, cv.linkTo.y) }
                return Qt.rect(x0 - 200, y0 - 200, x1 - x0 + 400, y1 - y0 + 400)
            }
            x: box.x; y: box.y; implicitWidth: box.width; implicitHeight: box.height
            Component.onCompleted: { width = Qt.binding(() => box.width); height = Qt.binding(() => box.height) }
            onPaint: {
                const g = getContext("2d"); g.reset(); g.translate(-box.x, -box.y)
                const curve = (a, as, b, bs, col, w, dash) => {
                    const k = Math.max(40, Math.hypot(b.x - a.x, b.y - a.y) / 3)
                    const off = s => s === "left" ? [-k, 0] : s === "right" ? [k, 0] : s === "top" ? [0, -k] : [0, k]
                    const [ax, ay] = off(as), [bx, by] = off(bs)
                    g.beginPath(); g.moveTo(a.x, a.y); g.bezierCurveTo(a.x + ax, a.y + ay, b.x + bx, b.y + by, b.x, b.y)
                    g.strokeStyle = col; g.lineWidth = w; g.setLineDash(dash); g.stroke()
                    // the arrow head at the end
                    const ang = Math.atan2(b.y - (b.y + by), b.x - (b.x + bx))
                    g.setLineDash([]); g.beginPath(); g.moveTo(b.x, b.y)
                    g.lineTo(b.x - 11 * Math.cos(ang - 0.45), b.y - 11 * Math.sin(ang - 0.45)); g.lineTo(b.x - 11 * Math.cos(ang + 0.45), b.y - 11 * Math.sin(ang + 0.45))
                    g.closePath(); g.fillStyle = col; g.fill()
                    return Qt.point((a.x + b.x) / 2 + (ax + bx) / 8, (a.y + b.y) / 2 + (ay + by) / 8)
                }
                for (const e of cv.edges) {
                    const a = cv.byId(e.fromNode), b = cv.byId(e.toNode)
                    if (!a || !b) continue
                    const col = e.id === cv.sel ? cv.t.accent : cv.colourOf(e.color)
                    const mid = curve(cv.side(a, e.fromSide), e.fromSide ?? "right", cv.side(b, e.toSide), e.toSide ?? "left", col, e.id === cv.sel ? 3 : 2, [])
                    if (e.label) { g.font = "13px sans-serif"; g.fillStyle = cv.t.dim; g.textAlign = "center"; g.fillText(e.label, mid.x, mid.y - 6) }
                }
                if (cv.linkFrom) {
                    const a = cv.byId(cv.linkFrom)
                    if (a) curve(cv.side(a, "right"), "right", cv.linkTo, "left", cv.t.accent, 2, [6, 5])
                }
            }
            Connections { target: cv; function onNodesChanged() { wires.requestPaint() } function onEdgesChanged() { wires.requestPaint() } function onLinkToChanged() { wires.requestPaint() } function onSelChanged() { wires.requestPaint() } }
            onBoxChanged: requestPaint()
        }

        Repeater {
            model: cv.nodes.filter(n => n.type !== "group")
            Rectangle {
                id: card
                required property var modelData
                readonly property bool isSel: cv.sel === modelData.id
                readonly property bool isFile: modelData.type === "file"
                x: modelData.x; y: modelData.y; width: modelData.width; height: modelData.height
                radius: t.rRow + 4
                color: t.card
                border.color: isSel ? t.accent : cv.colourOf(modelData.color)
                border.width: isSel ? 2 : 1
                Behavior on border.color { ColorAnimation { duration: 140 } }

                // move: drag the card; resize: its corner
                MouseArea {
                    anchors.fill: parent
                    property point grab
                    onPressed: m => { cv.sel = card.modelData.id; if (cv.editing !== card.modelData.id) cv.editing = ""; grab = Qt.point(m.x, m.y); cv.forceActiveFocus() }
                    onPositionChanged: m => { card.x += m.x - grab.x; card.y += m.y - grab.y; const n = cv.byId(card.modelData.id); n.x = Math.round(card.x); n.y = Math.round(card.y); wires.requestPaint() }
                    onReleased: cv.changed()
                    onDoubleClicked: { if (card.isFile) cv.win.openFile({ path: cv.abs(card.modelData.file), rel: card.modelData.file }); else cv.editing = card.modelData.id }
                }
                // a file card's name: the file it shows
                Text {
                    id: head
                    visible: card.isFile
                    x: 14; y: 10; width: parent.width - 28; elide: Text.ElideMiddle
                    text: "  " + (card.modelData.file ?? "")
                    color: t.faint; font.family: t.sans; font.pixelSize: 12
                }
                Flickable {
                    id: body
                    x: 14; y: card.isFile ? 32 : 12; width: parent.width - 28; height: parent.height - y - 12
                    clip: true; contentHeight: shown.implicitHeight; interactive: contentHeight > height
                    visible: cv.editing !== card.modelData.id
                    Text {
                        id: shown
                        width: body.width; wrapMode: Text.Wrap
                        textFormat: card.isFile && !/\.(md|markdown)$/.test(card.modelData.file) ? Text.PlainText : Text.MarkdownText
                        text: { cv.liveTick; const s = card.isFile ? (cv.live[card.modelData.file] ?? "") : (card.modelData.text ?? ""); return s.length > 20000 ? s.slice(0, 20000) : s }
                        color: t.text; font.family: card.isFile && textFormat === Text.PlainText ? t.mono : t.sans; font.pixelSize: card.isFile && textFormat === Text.PlainText ? 11 : 14
                        onLinkActivated: l => Qt.openUrlExternally(l)
                    }
                }
                Text { visible: !card.isFile && !(card.modelData.text ?? "").length && cv.editing !== card.modelData.id; anchors.centerIn: parent; text: "double-click to write"; color: t.faint; font.family: t.sans; font.pixelSize: 13 }
                TextArea {
                    visible: cv.editing === card.modelData.id
                    x: 8; y: 6; width: parent.width - 16; height: parent.height - 12
                    text: card.modelData.text ?? ""; wrapMode: TextEdit.Wrap
                    color: t.text; font.family: t.sans; font.pixelSize: 14; background: null
                    onVisibleChanged: if (visible) { forceActiveFocus(); cursorPosition = length }
                    onTextChanged: if (visible) { const n = cv.byId(card.modelData.id); if (n && n.text !== text) { n.text = text; saveLater.restart() } }
                    Keys.onEscapePressed: { cv.editing = ""; cv.changed(); cv.forceActiveFocus() }
                }
                // the link handle: drag it onto another card
                Rectangle {
                    visible: card.isSel || linkHov.hovered
                    width: 14; height: 14; radius: 7; x: parent.width - 7; y: parent.height / 2 - 7
                    color: t.accent; border.color: t.card; border.width: 2
                    HoverHandler { id: linkHov }
                    MouseArea {
                        anchors.fill: parent; anchors.margins: -6
                        cursorShape: Qt.CrossCursor
                        onPressed: { cv.linkFrom = card.modelData.id }
                        onPositionChanged: m => { const p = mapToItem(world, m.x, m.y); cv.linkTo = p }
                        onReleased: m => {
                            const p = mapToItem(world, m.x, m.y)
                            const hit = cv.nodes.find(n => n.type !== "group" && p.x >= n.x && p.x <= n.x + n.width && p.y >= n.y && p.y <= n.y + n.height)
                            if (hit) cv.link(cv.linkFrom, hit.id)
                            cv.linkFrom = ""
                        }
                    }
                }
                Rectangle {
                    width: 12; height: 12; radius: 3; x: parent.width - 14; y: parent.height - 14
                    color: card.isSel ? Qt.alpha(t.accent, 0.6) : "transparent"
                    MouseArea {
                        anchors.fill: parent; anchors.margins: -4; cursorShape: Qt.SizeFDiagCursor
                        property point grab
                        onPressed: m => { cv.sel = card.modelData.id; grab = Qt.point(m.x, m.y) }
                        onPositionChanged: m => {
                            const n = cv.byId(card.modelData.id)
                            n.width = Math.max(120, Math.round(card.width + m.x - grab.x)); n.height = Math.max(60, Math.round(card.height + m.y - grab.y))
                            card.width = n.width; card.height = n.height; wires.requestPaint()
                        }
                        onReleased: cv.changed()
                    }
                }
            }
        }
    }

    // the keys: N a text card, Del removes, arrows move it, +/- zoom, 0 fits, Enter writes in it
    focus: visible
    Keys.onPressed: e => {
        const n = cv.byId(cv.sel)
        const step = e.modifiers & Qt.ShiftModifier ? 100 : 20
        if (e.key === Qt.Key_N) { const p = cv.toBoard(width / 2, height / 2); cv.addText(p.x - 130, p.y - 70, "") }
        else if ((e.key === Qt.Key_Delete || e.key === Qt.Key_Backspace) && cv.sel) cv.remove(cv.sel)
        else if (e.key === Qt.Key_Plus || e.key === Qt.Key_Equal) cv.zoomAt(1.2, width / 2, height / 2)
        else if (e.key === Qt.Key_Minus) cv.zoomAt(1 / 1.2, width / 2, height / 2)
        else if (e.key === Qt.Key_0) cv.fit()
        else if ((e.key === Qt.Key_Return || e.key === Qt.Key_Enter) && n) { if (n.type === "file") cv.win.openFile({ path: cv.abs(n.file), rel: n.file }); else cv.editing = n.id }
        else if (n && [Qt.Key_Left, Qt.Key_Right, Qt.Key_Up, Qt.Key_Down].includes(e.key)) {
            n.x += e.key === Qt.Key_Left ? -step : e.key === Qt.Key_Right ? step : 0
            n.y += e.key === Qt.Key_Up ? -step : e.key === Qt.Key_Down ? step : 0
            cv.changed()
        }
        else if (e.key === Qt.Key_Tab && cv.nodes.length) {
            // Tab walks the cards
            const list = cv.nodes.filter(x => x.type !== "group"), i = list.findIndex(x => x.id === cv.sel)
            cv.sel = list[(i + 1) % list.length].id
        }
        else return
        e.accepted = true
    }

    // the keys, at the bottom: two short rows at most
    Text {
        anchors { left: parent.left; bottom: parent.bottom; margins: 14 }
        text: cv.note || (cv.nodes.length ? "double-click: new card · drag the dot: link · N new · Del remove · arrows move · 0 fit · Ctrl+wheel zoom" : "An empty canvas: double-click to write a card, or drop a file here to show it live")
        color: t.faint; font.family: t.sans; font.pixelSize: 12
    }
    Text {
        anchors { right: parent.right; bottom: parent.bottom; margins: 14 }
        text: Math.round(cv.zoom * 100) + "%"; color: t.faint; font.family: t.mono; font.pixelSize: 12
    }
}
