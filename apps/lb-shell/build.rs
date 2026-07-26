use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    println!("cargo:rerun-if-changed=src/library_controller.rs");
    println!("cargo:rerun-if-changed=qml/LaunchBoxWindow.qml");
    println!("cargo:rerun-if-changed=qml/BigBoxWindow.qml");
    println!("cargo:rerun-if-changed=qml/GameImageViewer.qml");
    println!("cargo:rerun-if-changed=qml/BoxArtView.qml");
    println!("cargo:rerun-if-changed=qml/BoxModelViewer.qml");
    println!("cargo:rerun-if-changed=qml/ModelSettingsEditor.qml");
    println!("cargo:rerun-if-changed=qml/GameMusicPlayer.qml");
    println!("cargo:rerun-if-changed=qml/LaunchStartupOverlay.qml");
    println!("cargo:rerun-if-changed=qml/LaunchShutdownOverlay.qml");
    println!("cargo:rerun-if-changed=qml/LaunchPauseOverlay.qml");

    CxxQtBuilder::new_qml_module(
        QmlModule::new("LaunchBoxPort")
            .depend("QtQml.Models")
            .depend("QtMultimedia")
            .depend("QtQuick3D")
            .depend("QtQuick3D.Helpers")
            .qml_files([
                "qml/LaunchBoxWindow.qml",
                "qml/BigBoxWindow.qml",
                "qml/GameImageViewer.qml",
                "qml/BoxArtView.qml",
                "qml/BoxModelViewer.qml",
                "qml/ModelSettingsEditor.qml",
                "qml/GameMusicPlayer.qml",
                "qml/LaunchStartupOverlay.qml",
                "qml/LaunchShutdownOverlay.qml",
                "qml/LaunchPauseOverlay.qml",
            ]),
    )
    .qt_module("Quick")
    .qt_module("QuickControls2")
    .qt_module("Multimedia")
    .qt_module("Quick3D")
    .file("src/library_controller.rs")
    .build();
}
