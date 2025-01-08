use std::sync::Arc;

use anyhow::Result;
use tokio::runtime::Runtime;

use uchat_coreapi::core_api::CoreApi;
use uchat_coreapi::protocol::ServerResponse;

pub mod cxxqt_object;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};

fn main() {
    // Create the application and engine
    let mut app = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();

    // Load the QML path into the engine
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/com/kdab/cxx_qt/demo/resources/main.qml"));
    }

    if let Some(engine) = engine.as_mut() {
        // Listen to a signal from the QML Engine
        engine
            .as_qqmlengine()
            .on_quit(|_| {
                println!("QML Quit!");
            })
            .release();
    }

    // Start the app
    if let Some(app) = app.as_mut() {
        app.exec();
    }
}