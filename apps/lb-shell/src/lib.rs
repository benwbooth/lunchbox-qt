pub mod library_controller;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShellMode {
    LaunchBox,
    BigBox,
}

impl ShellMode {
    fn qml_url(self) -> &'static str {
        match self {
            Self::LaunchBox => "qrc:/qt/qml/LaunchBoxPort/qml/LaunchBoxWindow.qml",
            Self::BigBox => "qrc:/qt/qml/LaunchBoxPort/qml/BigBoxWindow.qml",
        }
    }
}

pub fn initialize_qt() {
    cxx_qt::init_crate!(cxx_qt);
    cxx_qt::init_crate!(cxx_qt_lib);
    cxx_qt::init_crate!(lb_shell);
    cxx_qt::init_qml_module!("LaunchBoxPort");
}

pub fn run(mode: ShellMode) {
    initialize_qt();

    std::env::set_var("QT_QUICK_CONTROLS_STYLE", "Basic");

    let mut application = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();
    let url = QUrl::from(mode.qml_url());

    eprintln!("Loading {url:?}");
    let engine = engine
        .as_mut()
        .expect("Qt did not construct a QQmlApplicationEngine");
    engine.load(&url);

    let application = application
        .as_mut()
        .expect("Qt did not construct a QGuiApplication");
    let exit_code = application.exec();
    eprintln!("Qt event loop exited with status {exit_code}.");
}
