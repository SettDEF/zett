// Clicks in the marketplace popup (category chips, a tag) with real mouse events; gui-check.sh runs it next to a copy of Marketplace.qml
import QtQuick
import QtTest
import "." as Z

Item {
    id: root
    width: 1180; height: 760
    QtObject {
        id: fakeApp
        property string last: ""
        function evalLua(expr) {
            last = expr
            let r = []
            if (expr.indexOf("market_info") >= 0) r = [10, 1793252661]
            else if (expr.indexOf("store.search") >= 0) r = [{ repo: "a/one.nvim", desc: "d", stars: 5, tags: ["lsp", "rust"], installed: false, managed: false, updated: "" }]
            return JSON.stringify({ result: JSON.stringify(r) })
        }
    }
    readonly property var th: ({ raised: "#333", card: "#222", rule: "#555", text: "#eee", dim: "#aaa", faint: "#888", accent: "#c96", onAccent: "#000", sans: "sans", mono: "monospace", icons: "sans" })
    Z.Marketplace { id: mk; theme: root.th; app: fakeApp }
    TestCase {
        name: "Market"; when: windowShown
        function test_chip_click_moving() {
            mk.show(); wait(50)
            const chip = findChild(mk, "chip2")
            mousePress(chip, 10, 10); mouseMove(chip, 14, 11); mouseMove(chip, 18, 12); mouseRelease(chip, 18, 12)
            wait(50)
            compare(mk.cat, 2, "Themes chosen by a click that moved a few pixels")
        }
        function test_tag_click() {
            mk.show(); wait(50)
            mk.setCat(5); wait(20)
            const tag = findChild(mk, "tag_rust")
            verify(tag !== null, "tag shown")
            mouseClick(tag); wait(150)
            compare(mk.cat, 0, "back to All")
            verify(fakeApp.last.indexOf("rust") >= 0 && fakeApp.last.indexOf("git") < 0, "searched for the tag: " + fakeApp.last)
        }
        function test_chip_click() {
            mk.show()
            wait(50)
            const chip = findChild(mk, "chip5")
            verify(chip !== null, "chip found")
            mouseClick(chip)
            wait(50)
            compare(mk.cat, 5, "Git chosen by a click")
            verify(fakeApp.last.indexOf("git") >= 0, "searched with git: " + fakeApp.last)
        }
    }
}
