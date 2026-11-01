#![forbid(unsafe_code)]
//! Native Figma 2 Penpot desktop application.
//!
//! The UI and the conversion worker are deliberately separate:
//! - [`model`] is a headless state machine with unit tests.
//! - [`conversion`] runs file IO and mapping on worker threads.
//! - [`app`] only draws state and dispatches actions.

pub mod app;
pub mod conversion;
pub mod model;
pub mod settings;
pub mod strings;

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
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(strings::APP_TITLE)
            .with_app_id(APP_ID)
            .with_inner_size([1180.0, 760.0])
            .with_min_inner_size([900.0, 620.0]),
        ..Default::default()
    };
    eframe::run_native(
        strings::APP_TITLE,
        options,
        Box::new(move |cc| Ok(Box::new(F2pApp::new_with_paths(cc, paths)))),
    )
}
