import QtQuick
import QtQuick.Layouts

// Under a page's title: its properties as chips (the front matter besides the title: tags, status,
// dates…); a click on one edits it, + adds one. The page's icon (the "icon" property) is drawn by Main
// above the title; IconPicker below sets it.
Flow {
    id: pp
    required property var win
    required property var t
    required property var app
    // [[key, value]], read again when the page changes
    property var props: { app.title; app.blocks; try { return JSON.parse(app.pageProps()) } catch (e) { return [] } }
    readonly property var shown: props.filter(p => p[0] !== "icon")
    property string editing: ""          // the key being edited, "+" for a new one
    spacing: 6

    function save(key, value) { if (key.trim()) app.setProp(key.trim(), value); editing = "" }
    function iconFor(k) { return ({ tags: "sell", tag: "sell", status: "flag", date: "event", due: "event", created: "schedule", author: "person", project: "folder", url: "link", priority: "priority_high" })[k.toLowerCase()] ?? "label" }

    Repeater {
        model: pp.shown
        delegate: Rectangle {
            id: chip
            required property var modelData
            readonly property bool open: pp.editing === modelData[0]
            implicitWidth: open ? 260 : chipRow.implicitWidth + 20
            implicitHeight: 28; radius: 14
            color: hov.hovered || open ? t.raised : t.card
            RowLayout {
                id: chipRow
                visible: !chip.open
                anchors.centerIn: parent
                spacing: 6
                Text { text: pp.iconFor(chip.modelData[0]); color: t.accent; font.family: t.icons; font.pixelSize: 14 }
                Text { text: chip.modelData[0]; color: t.faint; font.family: t.sans; font.pixelSize: 12 }
                // tags: each one a small pill
                Repeater {
                    model: /^tags?$/i.test(chip.modelData[0]) ? chip.modelData[1].split(/[,\s]+/).filter(x => x) : [chip.modelData[1]]
                    delegate: Text { required property string modelData; text: /^tags?$/i.test(chip.modelData[0]) ? "#" + modelData : modelData; color: t.text; font.family: t.sans; font.pixelSize: 12 }
                }
            }
            // editing: the value in place, Enter keeps it, an empty value removes the property
            TextInput {
                visible: chip.open
                anchors { fill: parent; leftMargin: 12; rightMargin: 12 }
                verticalAlignment: TextInput.AlignVCenter
                text: chip.modelData[1]
                color: t.text; font.family: t.sans; font.pixelSize: 12
                clip: true
                onVisibleChanged: if (visible) { forceActiveFocus(); selectAll() }
                Keys.onReturnPressed: pp.save(chip.modelData[0], text)
                Keys.onEscapePressed: pp.editing = ""
                onActiveFocusChanged: if (!activeFocus && chip.open) pp.save(chip.modelData[0], text)
            }
            HoverHandler { id: hov; cursorShape: Qt.PointingHandCursor }
            TapHandler { enabled: !chip.open; onTapped: pp.editing = chip.modelData[0] }
        }
    }
    // + a property: "key: value" typed in one field
    Rectangle {
        implicitWidth: pp.editing === "+" ? 260 : addRow.implicitWidth + 20
        implicitHeight: 28; radius: 14
        color: addHov.hovered || pp.editing === "+" ? t.raised : "transparent"
        border.width: pp.editing === "+" ? 0 : 1; border.color: Qt.alpha(t.rule, 0.6)
        RowLayout {
            id: addRow
            visible: pp.editing !== "+"
            anchors.centerIn: parent
            spacing: 4
            Text { text: "add"; color: t.faint; font.family: t.icons; font.pixelSize: 14 }
            Text { text: "Property"; color: t.faint; font.family: t.sans; font.pixelSize: 12 }
        }
        TextInput {
            visible: pp.editing === "+"
            anchors { fill: parent; leftMargin: 12; rightMargin: 12 }
            verticalAlignment: TextInput.AlignVCenter
            color: t.text; font.family: t.sans; font.pixelSize: 12
            clip: true
            Text { visible: !parent.text; anchors.verticalCenter: parent.verticalCenter; text: "key: value  (tags: a, b · status: draft)"; color: t.faint; font: parent.font }
            onVisibleChanged: if (visible) { text = ""; forceActiveFocus() }
            Keys.onReturnPressed: { const i = text.indexOf(":"); if (i > 0) pp.save(text.slice(0, i), text.slice(i + 1)); else pp.editing = "" }
            Keys.onEscapePressed: pp.editing = ""
            onActiveFocusChanged: if (!activeFocus && pp.editing === "+") pp.editing = ""
        }
        HoverHandler { id: addHov; cursorShape: Qt.PointingHandCursor }
        TapHandler { enabled: pp.editing !== "+"; onTapped: pp.editing = "+" }
    }
}
