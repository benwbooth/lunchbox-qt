use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    println!("cargo:rerun-if-changed=src/library_controller.rs");
    println!("cargo:rerun-if-changed=qml/LaunchBoxWindow.qml");
    println!("cargo:rerun-if-changed=qml/BigBoxWindow.qml");
    println!("cargo:rerun-if-changed=qml/LaunchStartupOverlay.qml");
    println!("cargo:rerun-if-changed=qml/LaunchShutdownOverlay.qml");

    CxxQtBuilder::new_qml_module(
        QmlModule::new("LaunchBoxPort")
            .depend("QtQml.Models")
            .qml_files([
                "qml/LaunchBoxWindow.qml",
                "qml/BigBoxWindow.qml",
                "qml/LaunchStartupOverlay.qml",
                "qml/LaunchShutdownOverlay.qml",
            ]),
    )
    .qt_module("Quick")
    .qt_module("QuickControls2")
    .file("src/library_controller.rs")
    .build();
}
