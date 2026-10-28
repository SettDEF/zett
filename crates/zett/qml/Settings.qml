import QtQuick
import QtQuick.Layouts

// Settings for the window: keys, text, layout, start. Every change applies at once and is saved to
// ~/.config/zett/gui.json; the editor's own options stay in init.lua (a button opens it).
Flickable {
    id: s
    required property var win
    required property var t
    required property var app
    clip: true
    contentHeight: col.implicitHeight + 60
    boundsBehavior: Flickable.StopAtBounds

    // a row: label + hint on the left, the control on the right
    component Row1: RowLayout {
        default property alias control: slot.data
        property string label
        property string hint
        Layout.fillWidth: true
        spacing: 16
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 2
            Text { Layout.fillWidth: true; text: label; color: t.text; font.family: t.sans; font.pixelSize: 14 }
            Text { visible: hint.length > 0; Layout.fillWidth: true; text: hint; wrapMode: Text.Wrap; color: t.faint; font.family: t.sans; font.pixelSize: 12 }
        }
        Item { id: slot; implicitWidth: childrenRect.width; implicitHeight: childrenRect.height }
    }
    // choose one of a few: pills side by side
    component Choice: Row {
        property var options: []        // [[value, label]]
        property var value
        signal picked(var v)
        spacing: 4
        Repeater {
            model: options
            delegate: Rectangle {
                required property var modelData
                readonly property bool on: modelData[0] === value
                width: cl.implicitWidth + 26; height: 32; radius: 16
                color: on ? t.accent : ch.hovered ? t.raised : t.card
                Text { id: cl; anchors.centerIn: parent; text: modelData[1]; color: parent.on ? t.onAccent : t.dim; font.family: t.sans; font.pixelSize: 13; font.weight: parent.on ? Font.DemiBold : Font.Normal }
                HoverHandler { id: ch; cursorShape: Qt.PointingHandCursor }
                TapHandler { onTapped: picked(modelData[0]) }
            }
        }
    }
    // a number with − and +
    component Stepper: Row {
        property real value
        property real step: 1
        property real from: 0
        property real to: 100
        property string suffix: ""
        signal changed(real v)
        spacing: 4
        Repeater {
            model: ["remove", "", "add"]
            delegate: Rectangle {
                required property string modelData
                required property int index
                width: index === 1 ? 64 : 32; height: 32; radius: 16
                color: index !== 1 && sh.hovered ? t.raised : t.card
                Text {
                    anchors.centerIn: parent
                    text: index === 1 ? `${Math.round(value * 100) / 100}${suffix}` : modelData
                    color: index === 1 ? t.text : t.dim
                    font.family: index === 1 ? t.mono : t.icons; font.pixelSize: index === 1 ? 12 : 18
                }
                HoverHandler { id: sh; cursorShape: index !== 1 ? Qt.PointingHandCursor : Qt.ArrowCursor }
                TapHandler { enabled: index !== 1; onTapped: changed(Math.max(from, Math.min(to, value + (index === 0 ? -step : step)))) }
            }
        }
    }
    component Section: Rectangle {
        default property alias body: inner.data
        property string title
        property string icon
        Layout.fillWidth: true
        implicitHeight: inner.implicitHeight + 32
        radius: 17
        color: t.card
        ColumnLayout {
            id: inner
            anchors { left: parent.left; right: parent.right; top: parent.top; margins: 16 }
            spacing: 14
            RowLayout {
                spacing: 8
                Text { text: icon; color: t.accent; font.family: t.icons; font.pixelSize: 18 }
                Text { text: title; color: t.text; font.family: t.sans; font.pixelSize: 15; font.weight: Font.DemiBold }
            }
        }
    }
    component Button1: Rectangle {
        property string icon
        property string label
        signal clicked
        implicitWidth: br.implicitWidth + 24; implicitHeight: 32; radius: 16
        color: bh.hovered ? t.raised : Qt.alpha(t.text, 0.06)
        RowLayout { id: br; anchors.centerIn: parent; spacing: 6
            Text { text: icon; color: t.dim; font.family: t.icons; font.pixelSize: 16 }
            Text { text: label; color: t.dim; font.family: t.sans; font.pixelSize: 13 } }
        HoverHandler { id: bh; cursorShape: Qt.PointingHandCursor }
        TapHandler { onTapped: parent.clicked() }
    }

    ColumnLayout {
        id: col
        x: Math.max(24, (s.width - width) / 2)
        y: 28
        width: Math.min(760, s.width - 48)
        spacing: 14

        Text { text: "Settings"; color: t.text; font.family: t.sans; font.pixelSize: 30; font.weight: Font.Bold }
        Text { Layout.topMargin: -8; text: "saved as you change them  ·  ~/.config/zett/gui.json"; color: t.faint; font.family: t.mono; font.pixelSize: 12 }

        Section {
            title: "Keys"; icon: "keyboard"
            Row1 {
                label: "Key profile"; hint: "Emacs keys type at once; Vim keys start in normal mode (Esc)"
                Choice { options: [["emacs", "Emacs"], ["vim", "Vim"]]; value: win.prefs.keys ?? "emacs"; onPicked: (v) => win.setPref("keys", v) }
            }
        }

        Section {
            title: "Text"; icon: "text_fields"
            Row1 {
                label: "Text size"; hint: "the page's text, headings with it"
                Stepper { value: win.prefs.scale ?? 1; step: 0.05; from: 0.8; to: 1.5; suffix: "×"; onChanged: (v) => win.setPref("scale", v) }
            }
            Row1 {
                label: "Page width"; hint: "the longest a line gets; about 70 characters reads best"
                Stepper { value: win.prefs.width ?? 760; step: 40; from: 480; to: 1400; suffix: " px"; onChanged: (v) => win.setPref("width", v) }
            }
            Row1 {
                label: "Font"; hint: "for the page and the window"
                Choice { options: [["Google Sans Flex", "Google Sans"], ["Inter", "Inter"], ["Noto Serif", "Serif"]]; value: win.prefs.sans ?? "Google Sans Flex"; onPicked: (v) => win.setPref("sans", v) }
            }
            Row1 {
                label: "Ligatures in code"; hint: "=> drawn as ⇒ in code blocks"
                Choice { options: [[false, "Off"], [true, "On"]]; value: win.prefs.codeLigatures ?? false; onPicked: (v) => win.setPref("codeLigatures", v) }
            }
            Row1 {
                label: "Format on save"; hint: "Ctrl+S in the file editor lets the language server format first"
                Choice { options: [[false, "Off"], [true, "On"]]; value: win.prefs.formatOnSave ?? false; onPicked: (v) => win.setPref("formatOnSave", v) }
            }
        }

        Section {
            title: "Window"; icon: "dashboard"
            Row1 {
                label: "Open on"; hint: "what the window shows when it starts"
                Choice { options: [["today", "Today"], ["page", "Last page"]]; value: win.prefs.start ?? "today"; onPicked: (v) => win.setPref("start", v) }
            }
            Row1 {
                label: "Details card"; hint: "outline, links and counts beside the page (Ctrl+\\)"
                Choice { options: [[true, "Shown"], [false, "Hidden"]]; value: win.prefs.details ?? true; onPicked: (v) => { win.setPref("details", v); win.detailsShown = v } }
            }
            Row1 {
                label: "Sidebar"
                Choice { options: [[false, "Shown"], [true, "Hidden"]]; value: win.sidebarHidden; onPicked: (v) => win.sidebarHidden = v }
            }
        }

        Section {
            title: "Configuration"; icon: "tune"
            Row1 {
                label: "init.lua"; hint: app.configPath()
                Row {
                    spacing: 6
                    Button1 { icon: "edit"; label: "Open"; onClicked: win.openFile({ path: app.configPath(), rel: "init.lua", dir: false }) }
                    Button1 { icon: "refresh"; label: "Reload"; onClicked: win.applyScript(JSON.parse(app.reloadConfig())) }
                }
            }
            Row1 {
                label: "Workspace"; hint: app.dir
                Button1 { icon: "folder_open"; label: "Show folder"; onClicked: Qt.openUrlExternally("file://" + app.dir) }
            }
            Row1 {
                label: "Commands"; hint: "everything zett can do, with its key"
                Button1 { icon: "terminal"; label: "Palette  Ctrl ⇧ P"; onClicked: win.openPalette() }
            }
        }
    }
}
