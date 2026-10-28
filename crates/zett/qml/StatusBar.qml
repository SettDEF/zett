import QtQuick
import QtQuick.Layouts

// Under the page: state chips (Vim mode, saved, blocks, init.lua's zett.gui.status), the main keys, the details switch.
Rectangle {
    required property var win
    required property var t
    required property var app
    Layout.fillWidth: true
    implicitHeight: 40
    color: "transparent"
    // status chips, as in the shell's bar
    component Chip: Rectangle {
        property string icon
        property string label
        property color tint: t.dim
        implicitWidth: chipRow.implicitWidth + 20; implicitHeight: 24; radius: 12
        color: t.card
        RowLayout {
            id: chipRow
            anchors.centerIn: parent
            spacing: 5
            Text { visible: parent.parent.icon.length > 0; text: parent.parent.icon; color: parent.parent.tint; font.family: t.icons; font.pixelSize: 14 }
            Text { text: parent.parent.label; color: parent.parent.tint; font.family: t.mono; font.pixelSize: 11 }
        }
    }
    RowLayout {
        anchors { fill: parent; leftMargin: 12; rightMargin: 12 }
        spacing: 6
        Chip { visible: win.vimMode.length > 0; label: win.vimMode; tint: win.vimMode === "INSERT" ? t.dim : t.accent }
        Chip { icon: "check_circle"; label: "saved"; tint: t.accent }
        Chip { icon: "notes"; label: `${win.blocks.length} blocks` }
        // init.lua's chips (zett.gui.status)
        Repeater { model: win.hooks.status; delegate: Chip { required property var modelData; icon: modelData.icon ?? ""; label: modelData.text } }
        // the hints take what is left and shorten, so nothing runs past the card
        Text { Layout.fillWidth: true; Layout.minimumWidth: 0; horizontalAlignment: Text.AlignRight; elide: Text.ElideLeft; text: "Ctrl P go to  ·  Ctrl \\ details  ·  / block kinds  ·  Esc Vim"; color: t.faint; font.family: t.sans; font.pixelSize: 11 }
        Rectangle {
            implicitWidth: 28; implicitHeight: 24; radius: 12
            color: detHov.hovered ? t.raised : t.card
            Text { anchors.centerIn: parent; text: win.detailsShown ? "right_panel_close" : "right_panel_open"; color: t.dim; font.family: t.icons; font.pixelSize: 16 }
            HoverHandler { id: detHov; cursorShape: Qt.PointingHandCursor }
            TapHandler { onTapped: win.detailsShown = !win.detailsShown }
        }
    }
}
