// A document (PDF; Word, Excel, PowerPoint, OpenDocument through LibreOffice to PDF): pages, search inside, zoom
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Pdf

Item {
    id: dv
    required property var win
    required property var t
    required property var app
    visible: win.view === "doc"
    Layout.fillWidth: true
    Layout.fillHeight: true
    clip: true

    property string path: ""
    property string note: ""
    property alias query: find.text
    readonly property int pages: doc.pageCount
    readonly property int matches: view.searchModel.count
    readonly property var office: /\.(docx?|xlsx?|pptx?|odt|ods|odp|rtf)$/i
    // the page as wide as the view (scaleToWidth sets 0 while the view has no size yet)
    function fitWidth() { const w = doc.pageCount ? doc.pagePointSize(0).width : 0; if (w > 0 && view.width > 50) view.renderScale = Math.max(0.1, (view.width - 40) / w) }
    function load(p) {
        path = p; note = ""; find.text = ""
        if (office.test(p)) {
            note = "converting with LibreOffice…"
            const pdf = app.officeToPdf(p)
            if (!pdf.length) { note = "LibreOffice could not convert this file"; return }
            note = ""; doc.source = "file://" + pdf
        } else doc.source = "file://" + p
    }

    PdfDocument { id: doc; onStatusChanged: if (doc.status === PdfDocument.Error) dv.note = "cannot read this document: " + doc.error }

    Rectangle { anchors.fill: parent; color: t.bg; radius: t.rCard }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 8
        spacing: 6

        // one row: search, its count and steps, the page, zoom
        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            Rectangle {
                Layout.preferredWidth: 280; Layout.preferredHeight: 30; radius: t.rRow; color: t.card
                border.color: find.activeFocus ? t.accent : "transparent"
                TextField {
                    id: find
                    anchors.fill: parent; leftPadding: 30; background: null
                    placeholderText: "Search in the document"; placeholderTextColor: t.faint
                    color: t.text; font.family: t.sans; font.pixelSize: 13
                    onTextChanged: view.searchString = text
                    Keys.onReturnPressed: e => e.modifiers & Qt.ShiftModifier ? view.searchBack() : view.searchForward()
                    Keys.onEscapePressed: { text = ""; view.forceActiveFocus() }
                }
                Text { x: 9; anchors.verticalCenter: parent.verticalCenter; text: "search"; color: t.faint; font.family: t.icons; font.pixelSize: 16 }
            }
            Text {
                visible: find.text.length > 0
                text: view.searchModel.count ? (view.searchModel.currentResult + 1) + " of " + view.searchModel.count : "no match"
                color: t.dim; font.family: t.mono; font.pixelSize: 12
            }
            Item { Layout.fillWidth: true }
            Text { text: doc.pageCount ? "page " + (view.currentPage + 1) + " of " + doc.pageCount : ""; color: t.dim; font.family: t.mono; font.pixelSize: 12 }
            Repeater {
                model: [["remove", "zoom out (-)", () => view.renderScale /= 1.2], ["add", "zoom in (+)", () => view.renderScale *= 1.2], ["fit_width", "fit width (0)", () => dv.fitWidth()]]
                Rectangle {
                    required property var modelData
                    width: 30; height: 30; radius: t.rRow; color: zh.hovered ? t.raised : t.card
                    HoverHandler { id: zh }
                    Text { anchors.centerIn: parent; text: modelData[0]; color: t.dim; font.family: t.icons; font.pixelSize: 16 }
                    TapHandler { onTapped: modelData[2]() }
                    ToolTip.visible: zh.hovered; ToolTip.text: modelData[1]
                }
            }
        }

        PdfMultiPageView {
            id: view
            Layout.fillWidth: true
            Layout.fillHeight: true
            document: doc
            visible: dv.note === ""
            onWidthChanged: if (doc.status === PdfDocument.Ready && renderScale <= 0.1) dv.fitWidth()
            Connections { target: doc; function onStatusChanged() { if (doc.status === PdfDocument.Ready) Qt.callLater(() => dv.fitWidth()) } }
        }
        Text { visible: dv.note !== ""; Layout.alignment: Qt.AlignCenter; text: dv.note; color: t.faint; font.family: t.sans; font.pixelSize: 14 }
    }

    // keys: Ctrl+F or / search, n / N next and previous match, + - 0 zoom
    focus: visible
    Keys.onPressed: e => {
        if (e.key === Qt.Key_Slash || (e.key === Qt.Key_F && e.modifiers & Qt.ControlModifier)) find.forceActiveFocus()
        else if (e.key === Qt.Key_N) e.modifiers & Qt.ShiftModifier ? view.searchBack() : view.searchForward()
        else if (e.key === Qt.Key_Plus || e.key === Qt.Key_Equal) view.renderScale *= 1.2
        else if (e.key === Qt.Key_Minus) view.renderScale /= 1.2
        else if (e.key === Qt.Key_0) dv.fitWidth()
        else return
        e.accepted = true
    }
}
