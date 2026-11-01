//! User-visible strings live here so localization can be added without hunting
//! through widget code.

use conversion_report::{FileStatus, Outcome, Severity};

use crate::model::{Screen, Stage};

pub const APP_TITLE: &str = "Figma 2 Penpot";
pub const APP_SUBTITLE: &str = "Move your designs from Figma to Penpot. Simply.";
pub const APP_ID: &str = "app.figma2penpot.desktop";

pub const HOME_HEADING: &str = "Bring your Figma files across";
pub const HOME_BODY: &str =
    "Drop a .fig file here, or choose files to start. Conversion stays on this machine.";
pub const CHOOSE_FILES: &str = "Choose .fig files";
pub const DROP_HINT: &str = "Drop .fig files anywhere in this window";
pub const DROP_ACTIVE: &str = "Release to add the files";
pub const RECENT_RESULT: &str = "Latest conversion";
pub const OPEN_RESULTS: &str = "Open results";
pub const START_OVER: &str = "Convert another file";

pub const QUEUE_HEADING: &str = "File queue";
pub const QUEUE_BODY: &str =
    "Files are staged here. Analysis and conversion run on a worker thread.";
pub const ADD_FILES: &str = "Add files";
pub const ADD_FOLDER: &str = "Add folder";
pub const FOLDER_EMPTY: &str = "No .fig files found in that folder.";
pub const FOLDER_TRUNCATED: &str = "Folder scan stopped after 512 files.";
pub const FOLDER_SCAN_FAILED: &str = "Could not scan folder";
pub const ANALYZE_AGAIN: &str = "Analyze again";
pub const SELECTED_OUTPUT: &str = "Selected output";
pub const PREVIEW_SELECTED: &str = "Preview for selected file";
pub const LOCAL_FIRST: &str = "Local-first";
pub const NO_TELEMETRY: &str = "No telemetry · No network needed";
pub const PURE_RUST: &str = "Pure Rust";
pub const OFFLINE_CONVERSION: &str = "Offline conversion";
pub const EVENT_REPORTS: &str = "Event-based reports";
pub const NO_CANCEL_YET: &str = "Cancellation arrives with the M5 batch queue.";
pub const APP_ID_LABEL: &str = "App ID";
pub const VALIDATION_CHECKS: &str = "checks";
pub const DURATION_MS: &str = "ms";
pub const SHORTCUTS: &str = "Keyboard shortcuts";
pub const SHORTCUT_MOD: &str = "Cmd/Ctrl";
pub const SHORTCUT_OPEN: &str = "Open .fig files";
pub const SHORTCUT_FOLDER: &str = "Add a folder";
pub const SHORTCUT_ANALYZE: &str = "Analyze selected file";
pub const SHORTCUT_CONVERT: &str = "Convert selected file";
pub const SHORTCUT_PREFS: &str = "Preferences";
pub const SHORTCUT_SCREENS: &str = "Jump to screens 1–8";
pub const CLEAR_QUEUE: &str = "Clear queue";
pub const REMOVE_FILE: &str = "Remove";
pub const ANALYZE_FILE: &str = "Analyze";
pub const CONVERT_FILE: &str = "Convert";
pub const CONVERT_ALL: &str = "Convert all";
pub const PAUSE_BATCH: &str = "Pause";
pub const RESUME_BATCH: &str = "Resume";
pub const CANCEL_BATCH: &str = "Cancel batch";
pub const BATCH_FINISHED: &str = "Batch finished";
pub const QUEUE_EMPTY: &str = "No files staged yet.";
pub const QUEUE_ONLY_FIG: &str = "Only .fig files are supported.";

pub const ANALYSIS_HEADING: &str = "File analysis";
pub const ANALYSIS_BODY: &str =
    "Read-only metadata from the selected file. Nothing is converted here.";
pub const ANALYZE_NOW: &str = "Analyze selected file";
pub const ANALYSIS_EMPTY: &str = "Select a file to analyze it.";
pub const ANALYSIS_PENDING: &str = "Analysis is running on a worker thread…";
pub const ANALYSIS_FAILED: &str = "Analysis failed";
pub const DOCUMENT_NAME: &str = "Document";
pub const CONTAINER_LABEL: &str = "Container";
pub const FORMAT_LABEL: &str = "Format";
pub const PAGES_LABEL: &str = "Visible pages";
pub const INTERNAL_PAGES_LABEL: &str = "Internal pages";
pub const NODES_LABEL: &str = "Active nodes";
pub const IMAGES_LABEL: &str = "Images";
pub const WARNINGS_LABEL: &str = "Parser warnings";
pub const DEPTH_LABEL: &str = "Maximum depth";

pub const SETTINGS_HEADING: &str = "Conversion settings";
pub const SETTINGS_BODY: &str = "These settings apply to every conversion in this session.";
pub const OUTPUT_DIR: &str = "Output folder";
pub const OUTPUT_SAME_AS_INPUT: &str = "Next to each .fig file";
pub const OUTPUT_CHOOSE: &str = "Choose folder";
pub const OUTPUT_CLEAR: &str = "Use input folder";
pub const OVERWRITE: &str = "Overwrite existing .penpot and report files";
pub const OPEN_AFTER: &str = "Open the output folder after a successful conversion";
pub const SAVE_SETTINGS: &str = "Save preferences";
pub const SETTINGS_SAVED: &str = "Preferences saved.";

pub const PROGRESS_HEADING: &str = "Progress";
pub const PROGRESS_BODY: &str = "Real stages only. No fabricated percentages.";
pub const NO_ACTIVE_JOB: &str = "No conversion is running.";
pub const STAGE_READING: &str = "Reading the .fig container";
pub const STAGE_CONVERTING: &str = "Mapping nodes and recording diagnostics";
pub const STAGE_WRITING: &str = "Building the .penpot archive";
pub const STAGE_VALIDATING: &str = "Running the structural validator";
pub const STAGE_SAVING: &str = "Writing output and report atomically";

pub const RESULTS_HEADING: &str = "Results";
pub const RESULTS_EMPTY: &str = "No conversion has finished yet.";
pub const OUTPUT_LABEL: &str = "Output file";
pub const REPORT_LABEL: &str = "Report file";
pub const OPEN_FOLDER: &str = "Open output folder";
pub const VIEW_REPORT: &str = "View report";
pub const VALIDATION_LABEL: &str = "Validation";
pub const DURATION_LABEL: &str = "Duration";
pub const DETECTED_LABEL: &str = "Detected";
pub const CONVERTED_LABEL: &str = "Converted";
pub const APPROXIMATED_LABEL: &str = "Approximated";
pub const SKIPPED_LABEL: &str = "Skipped";
pub const UNSUPPORTED_LABEL: &str = "Unsupported";
pub const FAILED_LABEL: &str = "Failed";

pub const REPORT_HEADING: &str = "Conversion report";
pub const REPORT_EMPTY: &str = "No report is available.";
pub const REPORT_SEARCH: &str = "Search code, element, or page";
pub const REPORT_SEVERITY: &str = "Severity";
pub const REPORT_OUTCOME: &str = "Outcome";
pub const REPORT_ALL: &str = "All";
pub const REPORT_EVENTS: &str = "events";
pub const REPORT_NO_MATCHES: &str = "No events match the current filter.";
pub const REPORT_ELEMENT: &str = "Element";
pub const REPORT_PAGE: &str = "Page";
pub const REPORT_ACTION: &str = "Action taken";
pub const REPORT_REASON: &str = "Reason";
pub const REPORT_SUGGESTED: &str = "Suggested action";
pub const REPORT_TRACEABLE: &str = "Element-level traceability is preserved for grouped issues.";

pub const PREFERENCES_HEADING: &str = "Preferences";
pub const PREFERENCES_BODY: &str = "Appearance and storage. Conversion never needs the network.";
pub const THEME_LABEL: &str = "Appearance";
pub const THEME_SYSTEM: &str = "Follow system";
pub const THEME_LIGHT: &str = "Light";
pub const THEME_DARK: &str = "Dark";
pub const SETTINGS_FILE: &str = "Settings file";
pub const RESET_PREFERENCES: &str = "Reset preferences";
pub const ABOUT_LABEL: &str = "About";
pub const ABOUT_BODY: &str = "Native Rust desktop app. No Electron, no webview, no telemetry.";
pub const VERSION_LABEL: &str = "Version";

pub const DROP_ADDED: &str = "Added to the queue.";

pub fn added_message(count: usize) -> String {
    format!("Added {count} file(s) to the queue.")
}

pub fn files_staged(count: usize) -> String {
    format!("{count} file(s) staged")
}

pub fn validation_checks(count: usize) -> String {
    format!("{} {VALIDATION_CHECKS}", count)
}

pub fn duration_ms(milliseconds: u64) -> String {
    format!("{milliseconds} {DURATION_MS}")
}
pub const REJECTED_PREFIX: &str = "Rejected";
pub const OPEN_FOLDER_FAILED: &str = "Could not open the output folder";
pub const SAVE_SETTINGS_FAILED: &str = "Could not save preferences";
pub const NO_FILE_SELECTED: &str = "Select a .fig file first.";
pub const BUSY: &str = "A conversion is already running.";
pub const ANALYSIS_FAILED_PREFIX: &str = "Could not analyze";

pub fn screen_label(screen: Screen) -> &'static str {
    match screen {
        Screen::Home => "Home / Import",
        Screen::Queue => "File queue",
        Screen::Analysis => "File analysis",
        Screen::Settings => "Conversion settings",
        Screen::Progress => "Progress",
        Screen::Results => "Results",
        Screen::Report => "Report",
        Screen::Preferences => "Preferences",
    }
}

pub fn stage_label(stage: Stage) -> &'static str {
    match stage {
        Stage::Reading => STAGE_READING,
        Stage::Converting => STAGE_CONVERTING,
        Stage::Writing => STAGE_WRITING,
        Stage::Validating => STAGE_VALIDATING,
        Stage::Saving => STAGE_SAVING,
    }
}

pub fn file_status_label(status: FileStatus) -> &'static str {
    match status {
        FileStatus::Completed => "✓ Completed",
        FileStatus::CompletedWithWarnings => "⚠ Completed with warnings",
        FileStatus::Failed => "✕ Failed",
        FileStatus::Cancelled => "◼ Cancelled",
    }
}

pub fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Info => "Info",
        Severity::Warning => "Warning",
        Severity::Error => "Error",
    }
}

pub fn outcome_label(outcome: Outcome) -> &'static str {
    match outcome {
        Outcome::Converted => "Converted",
        Outcome::ConvertedWithModification => "Converted with modification",
        Outcome::Skipped => "Skipped",
        Outcome::Unsupported => "Unsupported",
        Outcome::Failed => "Failed",
    }
}

pub fn validation_label(state: conversion_report::ValidationState) -> &'static str {
    match state {
        conversion_report::ValidationState::Passed => "✓ Passed",
        conversion_report::ValidationState::Failed => "✕ Failed",
        conversion_report::ValidationState::NotRun => "— Not run",
    }
}
