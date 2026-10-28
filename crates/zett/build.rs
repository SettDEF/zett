use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(QmlModule::new("dev.zett.app").qml_files(["qml/Main.qml", "qml/ContextMenu.qml", "qml/Palette.qml", "qml/Today.qml", "qml/FileView.qml", "qml/Details.qml"]))
        .qt_module("Quick")
        .qt_module("QuickControls2")
        .files(["src/window.rs"])
        .build();
}
