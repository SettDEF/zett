import QtQuick
import QtQuick.Layouts

// Find and replace in the file editor (Ctrl+F, Ctrl+H) and go to line (Ctrl+G): case / whole word / regex,
// "3 of 12", Enter next, Shift+Enter previous, Replace and Replace all; `matches` are drawn by FileView.
Rectangle {
    id: bar
    required property var t
    required property var editor        // the TextEdit searched
    property string mode: ""            // "" closed, "find", "replace", "line"
    property bool caseSens: false
    property bool word: false
    property bool regex: false
    property int cur: -1
    visible: mode !== ""
    implicitHeight: col.implicitHeight + 16
    radius: 14
    color: t.raised
    border.width: 1; border.color: Qt.alpha(t.rule, 0.4)

    // every match: [start, end]; recomputed when the text or the query changes
    readonly property var matches: {
        if (mode === "" || mode === "line" || !q.text) return []
        let re
        try {
            const src = regex ? q.text : q.text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")
            re = new RegExp(word ? `\\b${src}\\b` : src, caseSens ? "g" : "gi")
        } catch (e) { return [] }
        const out = [], s = editor.text
        let m
        while ((m = re.exec(s)) && out.length < 2000) { if (m[0].length === 0) { re.lastIndex++; continue } out.push([m.index, m.index + m[0].length]) }
        return out
    }
    onMatchesChanged: { const p = editor.cursorPosition; const i = matches.findIndex(m => m[0] >= p); cur = matches.length ? (i >= 0 ? i : 0) : -1 }

    function open(m) { mode = m; q.forceActiveFocus(); q.selectAll(); if (m !== "line" && editor.selectedText && !editor.selectedText.includes("\n")) q.text = editor.selectedText }
    function close() { mode = ""; editor.forceActiveFocus() }
    function go(d) {
        if (!matches.length) return
        cur = (cur + d + matches.length) % matches.length
        editor.select(matches[cur][0], matches[cur][1])
    }
    function replaceOne() {
        if (cur < 0 || !matches.length) return
        const m = matches[cur]
        editor.remove(m[0], m[1]); editor.insert(m[0], r.text)
        editor.cursorPosition = m[0] + r.text.length
    }
    function replaceAll() {
        // from the end, so earlier positions stay right
        for (const m of matches.slice().reverse()) { editor.remove(m[0], m[1]); editor.insert(m[0], r.text) }
    }
    function goLine() {
        const n = parseInt(q.text) - 1
        if (isNaN(n)) return
        const lines = editor.text.split("\n")
        let pos = 0
        for (let i = 0; i < Math.min(n, lines.length - 1); i++) pos += lines[i].length + 1
        editor.cursorPosition = pos
        close()
    }

    component Toggle: Rectangle {
        property string label: ""
        property bool on: false
        property string tip: ""
        signal flipped()
        implicitWidth: 30; implicitHeight: 26; radius: 8
        color: on ? Qt.alpha(bar.t.accent, 0.22) : th.hovered ? Qt.alpha(bar.t.text, 0.08) : "transparent"
        Text { anchors.centerIn: parent; text: parent.label; color: parent.on ? bar.t.accent : bar.t.dim; font.family: bar.t.mono; font.pixelSize: 12 }
        HoverHandler { id: th; cursorShape: Qt.PointingHandCursor }
        TapHandler { onTapped: parent.flipped() }
    }
    component Btn: Rectangle {
        property string label: ""
        signal clicked()
        implicitWidth: bt.implicitWidth + 18; implicitHeight: 26; radius: 13
        color: bh.hovered ? Qt.alpha(bar.t.text, 0.1) : Qt.alpha(bar.t.text, 0.05)
        Text { id: bt; anchors.centerIn: parent; text: parent.label; color: bar.t.text; font.family: bar.t.sans; font.pixelSize: 12 }
        HoverHandler { id: bh; cursorShape: Qt.PointingHandCursor }
        TapHandler { onTapped: parent.clicked() }
    }
    component Input: Rectangle {
        property alias input: ti
        property string placeholder: ""
        signal accepted(bool back)
        Layout.fillWidth: true
        implicitHeight: 30; radius: 10
        color: bar.t.card
        border.width: ti.activeFocus ? 1 : 0; border.color: Qt.alpha(bar.t.accent, 0.5)
        TextInput {
            id: ti
            anchors { fill: parent; leftMargin: 10; rightMargin: 10 }
            verticalAlignment: TextInput.AlignVCenter
            clip: true
            color: bar.t.text; font.family: bar.t.mono; font.pixelSize: 13
            selectionColor: Qt.alpha(bar.t.accent, 0.35)
            Keys.onPressed: (e) => {
                if (e.key === Qt.Key_Escape) { bar.close(); e.accepted = true }
                else if (e.key === Qt.Key_Return || e.key === Qt.Key_Enter) { parent.accepted(!!(e.modifiers & Qt.ShiftModifier)); e.accepted = true }
            }
            Text { visible: !ti.text; text: parent.parent.placeholder; color: bar.t.faint; font: ti.font; anchors.verticalCenter: parent.verticalCenter }
        }
    }

    ColumnLayout {
        id: col
        anchors { left: parent.left; right: parent.right; top: parent.top; margins: 8 }
        spacing: 6
        RowLayout {
            Layout.fillWidth: true
            spacing: 6
            Text { text: bar.mode === "line" ? "format_list_numbered" : "search"; color: bar.t.dim; font.family: bar.t.icons; font.pixelSize: 18 }
            Input {
                id: qBox
                placeholder: bar.mode === "line" ? `Go to line (1 – ${bar.editor.lineCount})` : "Find"
                onAccepted: (back) => bar.mode === "line" ? bar.goLine() : bar.go(back ? -1 : 1)
            }
            Toggle { visible: bar.mode !== "line"; label: "Aa"; on: bar.caseSens; onFlipped: bar.caseSens = !bar.caseSens }
            Toggle { visible: bar.mode !== "line"; label: "ab"; on: bar.word; onFlipped: bar.word = !bar.word }
            Toggle { visible: bar.mode !== "line"; label: ".*"; on: bar.regex; onFlipped: bar.regex = !bar.regex }
            Text {
                visible: bar.mode !== "line"
                Layout.minimumWidth: 70; horizontalAlignment: Text.AlignRight
                text: !qBox.input.text ? "" : bar.matches.length ? `${bar.cur + 1} of ${bar.matches.length}` : "No results"
                color: bar.matches.length || !qBox.input.text ? bar.t.faint : bar.t.removed; font.family: bar.t.mono; font.pixelSize: 11
            }
            Btn { visible: bar.mode !== "line"; label: "↑"; onClicked: bar.go(-1) }
            Btn { visible: bar.mode !== "line"; label: "↓"; onClicked: bar.go(1) }
            Btn { visible: bar.mode === "find"; label: "Replace"; onClicked: { bar.mode = "replace"; rBox.input.forceActiveFocus() } }
            Btn { label: "✕"; onClicked: bar.close() }
        }
        RowLayout {
            visible: bar.mode === "replace"
            Layout.fillWidth: true
            spacing: 6
            Text { text: "find_replace"; color: bar.t.dim; font.family: bar.t.icons; font.pixelSize: 18 }
            Input { id: rBox; placeholder: "Replace with"; onAccepted: bar.replaceOne() }
            Btn { label: "Replace"; onClicked: bar.replaceOne() }
            Btn { label: "Replace all"; onClicked: bar.replaceAll() }
        }
    }
    readonly property alias q: qBox.input
    readonly property alias r: rBox.input
}
