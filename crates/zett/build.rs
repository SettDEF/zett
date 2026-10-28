use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(QmlModule::new("dev.zett.app").qml_files(["qml/Main.qml", "qml/ContextMenu.qml", "qml/Palette.qml", "qml/Today.qml", "qml/FileView.qml", "qml/Details.qml", "qml/Settings.qml", "qml/SidePage.qml", "qml/Drawer.qml", "qml/Sidebar.qml", "qml/Toolbar.qml", "qml/StatusBar.qml"]))
        .qt_module("Quick")
        .qt_module("QuickControls2")
        .files(["src/window.rs"])
        .build();
}
