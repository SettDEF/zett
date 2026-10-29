use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(QmlModule::new("dev.zett.app").qml_files(["qml/Main.qml", "qml/ContextMenu.qml", "qml/Palette.qml", "qml/Today.qml", "qml/FileView.qml", "qml/Details.qml", "qml/Settings.qml", "qml/SidePage.qml", "qml/Drawer.qml", "qml/Sidebar.qml", "qml/Toolbar.qml", "qml/StatusBar.qml", "qml/Search.qml", "qml/PageProps.qml", "qml/FindBar.qml", "qml/ActivityBar.qml", "qml/SideFile.qml", "qml/SearchFiles.qml", "qml/SourceControl.qml", "qml/FileOutline.qml", "qml/Header.qml", "qml/Welcome.qml", "qml/CodeView.qml", "qml/Terminal.qml", "qml/Debugger.qml", "qml/Marketplace.qml", "qml/CanvasView.qml", "qml/DocView.qml"]))
        .qt_module("Quick")
        .qt_module("QuickControls2")
        .files(["src/window.rs", "src/doc.rs"])
        .build();
}
