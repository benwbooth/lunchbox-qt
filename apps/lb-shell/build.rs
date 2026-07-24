use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    println!("cargo:rerun-if-changed=src/library_controller.rs");
    println!("cargo:rerun-if-changed=qml/LaunchBoxWindow.qml");
    println!("cargo:rerun-if-changed=qml/BigBoxWindow.qml");
    println!("cargo:rerun-if-changed=qml/LaunchStartupOverlay.qml");
    println!("cargo:rerun-if-changed=qml/LaunchShutdownOverlay.qml");
    println!("cargo:rerun-if-changed=qml/LaunchPauseOverlay.qml");

    CxxQtBuilder::new_qml_module(
        QmlModule::new("LaunchBoxPort")
            .depend("QtQml.Models")
            .depend("QtMultimedia")
            .qml_files([
                "qml/LaunchBoxWindow.qml",
                "qml/BigBoxWindow.qml",
                "qml/LaunchStartupOverlay.qml",
                "qml/LaunchShutdownOverlay.qml",
                "qml/LaunchPauseOverlay.qml",
            ]),
    )
    .qt_module("Quick")
    .qt_module("QuickControls2")
    .qt_module("Multimedia")
    .file("src/library_controller.rs")
    .build();
}
