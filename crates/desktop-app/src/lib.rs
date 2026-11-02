#![forbid(unsafe_code)]
//! Native Figma 2 Penpot desktop application.
//!
//! The UI and conversion workers are separate:
//! - `model` is a headless state machine with unit tests.
//! - `conversion` and `conversion-queue` run IO and mapping on worker threads.
//! - `views` only draws state; `app` dispatches actions.

pub mod app;
pub mod components;
pub mod conversion;
pub mod copy;
pub mod model;
pub mod settings;
pub mod theme;
pub mod views;

pub use app::F2pApp;
pub use settings::{Settings, ThemeChoice, APP_ID};

use eframe::egui;

/// Launch the native application.
pub fn run() -> eframe::Result {
    run_with_paths(Vec::new())
}

/// Launch the native application with files staged in the queue, as passed by
/// the operating system or the command line.
pub fn run_with_paths(paths: Vec<std::path::PathBuf>) -> eframe::Result {
    let (width, height, min_width, min_height) = match std::env::var("F2P_PREVIEW_SIZE")
        .unwrap_or_default()
        .as_str()
    {
        "compact" => (900.0, 640.0, 900.0, 620.0),
        "large" => (1440.0, 900.0, 900.0, 620.0),
        _ => (1180.0, 760.0, 900.0, 620.0),
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(copy::APP_NAME)
            .with_app_id(APP_ID)
            .with_inner_size([width, height])
            .with_min_inner_size([min_width, min_height]),
        ..Default::default()
    };
    eframe::run_native(
        copy::APP_NAME,
        options,
        Box::new(move |cc| Ok(Box::new(F2pApp::new_with_paths(cc, paths)))),
    )
}
