import QtQuick
import QtQuick.Layouts

// Today, the start screen: the date, today's journal, recent pages, open to-dos, the main keys.
Flickable {
    required property var win
    required property var t
    required property var app
    id: today
    visible: win.view === "today"
    Layout.fillWidth: true
    Layout.fillHeight: true
    clip: true
    contentHeight: todayCol.implicitHeight + 60
    boundsBehavior: Flickable.StopAtBounds
    readonly property string iso: Qt.formatDate(new Date(), "yyyy-MM-dd")
    readonly property var recent: win.pages.map((p, i) => ({ p, i })).filter(o => o.p.modified > 0).sort((a, b) => b.p.modified - a.p.modified).slice(0, 6)
    readonly property var todos: { win.pages; win.blocks; try { return JSON.parse(app.todos()) } catch (e) { return [] } }
    readonly property int journal: win.pages.findIndex(p => p.title === iso)
    function ago(s) {
        const d = Date.now() / 1000 - s
        return d < 60 ? "now" : d < 3600 ? `${Math.floor(d / 60)} min` : d < 86400 ? `${Math.floor(d / 3600)} h` : `${Math.floor(d / 86400)} d`
    }
    component Card: Rectangle {
        default property alias body: inner.data
        property string icon
        property string title
        property string extra
        Layout.fillWidth: true
        implicitHeight: inner.implicitHeight + 28
        radius: 17
        color: t.card
        ColumnLayout {
            id: inner
            anchors { left: parent.left; right: parent.right; top: parent.top; margins: 14 }
            spacing: 4
            RowLayout {
                spacing: 8
                Layout.bottomMargin: 4
                Text { text: icon; color: t.accent; font.family: t.icons; font.pixelSize: 18 }
                Text { text: title; color: t.text; font.family: t.sans; font.pixelSize: 14; font.weight: Font.DemiBold }
                Item { Layout.fillWidth: true }
                Text { text: extra; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
            }
        }
    }
    component Row2: Rectangle {
        property string icon
        property string label
        property string side
        property bool dim: false
        signal clicked
        Layout.fillWidth: true
        implicitHeight: 34; radius: t.rRow
        color: rHov.hovered ? Qt.alpha(t.text, 0.06) : "transparent"
        RowLayout {
            anchors { fill: parent; leftMargin: 8; rightMargin: 8 }
            spacing: 10
            Text { text: icon; color: t.faint; font.family: t.icons; font.pixelSize: 17 }
            Text { Layout.fillWidth: true; text: label; elide: Text.ElideRight; color: dim ? t.faint : t.text; font.family: t.sans; font.pixelSize: 13 }
            Text { text: side; color: t.faint; font.family: t.mono; font.pixelSize: 11 }
        }
        HoverHandler { id: rHov; cursorShape: Qt.PointingHandCursor }
        TapHandler { onTapped: parent.clicked() }
    }
    ColumnLayout {
        id: todayCol
        x: Math.max(24, (today.width - width) / 2)
        y: 28
        width: Math.min(760, today.width - 48)
        spacing: 14
        Text { text: Qt.formatDate(new Date(), "dddd, d MMMM"); color: t.text; font.family: t.sans; font.pixelSize: 30; font.weight: Font.Bold }
        Text { Layout.topMargin: -8; text: `${win.pages.length} pages  ·  ${today.todos.length} open to-dos`; color: t.faint; font.family: t.mono; font.pixelSize: 12 }
        GridLayout {
            Layout.fillWidth: true
            columns: today.width > 720 ? 2 : 1
            columnSpacing: 14; rowSpacing: 14
            Card {
                icon: "edit_note"; title: "Journal"; extra: today.iso
                Layout.alignment: Qt.AlignTop
                Text { Layout.fillWidth: true; wrapMode: Text.Wrap; text: today.journal >= 0 ? `${win.pages[today.journal].blocks} blocks written today` : "Nothing written today yet"; color: t.dim; font.family: t.sans; font.pixelSize: 13 }
                Rectangle {
                    Layout.topMargin: 6
                    implicitWidth: jRow.implicitWidth + 28; implicitHeight: 34; radius: 17
                    color: jHov.hovered ? Qt.lighter(t.accent, 1.08) : t.accent
                    RowLayout { id: jRow; anchors.centerIn: parent; spacing: 6
                        Text { text: today.journal >= 0 ? "arrow_forward" : "add"; color: t.onAccent; font.family: t.icons; font.pixelSize: 16 }
                        Text { text: today.journal >= 0 ? "Open today" : "Start today"; color: t.onAccent; font.family: t.sans; font.pixelSize: 13; font.weight: Font.DemiBold } }
                    HoverHandler { id: jHov; cursorShape: Qt.PointingHandCursor }
                    TapHandler { onTapped: { win.view = "page"; app.openTitle(today.iso) } }
                }
            }
            Card {
                icon: "history"; title: "Recent"; extra: ""
                Layout.alignment: Qt.AlignTop
                Text { visible: today.recent.length === 0; text: "No pages yet"; color: t.faint; font.family: t.sans; font.pixelSize: 13 }
                Repeater {
                    model: today.recent
                    delegate: Row2 { required property var modelData; icon: "description"; label: modelData.p.title || "Untitled"; side: today.ago(modelData.p.modified); onClicked: win.openPage(modelData.i) }
                }
            }
        }
        Card {
            icon: "check_box"; title: "Open to-dos"; extra: today.todos.length ? `${today.todos.length}` : ""
            Text { visible: today.todos.length === 0; text: "All done. Write  [ ]  in any page to add one."; color: t.faint; font.family: t.sans; font.pixelSize: 13 }
            Repeater {
                model: today.todos.slice(0, 12)
                delegate: Row2 {
                    required property var modelData
                    icon: "check_box_outline_blank"; label: modelData.text || "(empty)"; side: modelData.title
                    onClicked: { win.openPage(modelData.page); win.focusAfter = modelData.block }
                }
            }
        }
        Card {
            icon: "keyboard"; title: "Keys"; extra: ""
            Flow {
                Layout.fillWidth: true
                spacing: 8
                Repeater {
                    model: [["Ctrl P", "go to a page"], ["Ctrl \\", "details"], ["F2", "rename"], ["Ctrl W", "close tab"], ["Ctrl Tab", "next tab"], ["/", "block kinds"], ["Esc", "Vim keys"], ["right-click", "menus"]]
                    delegate: Rectangle {
                        required property var modelData
                        implicitWidth: kr.implicitWidth + 20; implicitHeight: 28; radius: 14
                        color: t.raised
                        RowLayout { id: kr; anchors.centerIn: parent; spacing: 6
                            Text { text: modelData[0]; color: t.accent; font.family: t.mono; font.pixelSize: 11 }
                            Text { text: modelData[1]; color: t.dim; font.family: t.sans; font.pixelSize: 12 } }
                    }
                }
            }
        }
    }
}
