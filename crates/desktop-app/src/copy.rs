//! Every user-visible string and every internal-status → designer-facing mapping.
//!
//! Issue codes, node IDs, and raw converter descriptions stay in the data model and the
//! JSON/HTML/CSV exports. The UI uses the friendly presentation in this module and reveals
//! raw details only behind "Show details".

use conversion_report::{FileStatus, Outcome, Severity, ValidationState};

use crate::model::{AnalysisSummary, QueueState, Screen, Stage};

pub const APP_NAME: &str = "Figma 2 Penpot";
pub const TAGLINE: &str = "Move your designs from Figma to Penpot. Simply.";
pub const PRIVACY_SHORT: &str = "Everything stays on your computer.";
pub const PRIVACY_LONG: &str =
    "No account, no upload, no network calls during conversion. Your files never leave this computer.";

// Navigation
pub const NAV_IMPORT: &str = "Import";
pub const NAV_FILES: &str = "Files";
pub const NAV_DETAILS: &str = "File details";
pub const NAV_CONVERT: &str = "Convert";
pub const NAV_RESULTS: &str = "Results";
pub const NAV_REPORT: &str = "Report";
pub const NAV_SETTINGS: &str = "Settings";
pub const NAV_PREFERENCES: &str = "Preferences";
pub const NAV_WORKFLOW: &str = "MIGRATE";
pub const NAV_SUBTITLE: &str = "Figma → Penpot";
pub const OFFLINE_SHORT: &str = "Offline · no account";
pub const NAV_APP: &str = "APP";

// Global actions
pub const CHOOSE_FILES: &str = "Choose files…";
pub const CHOOSE_FOLDER: &str = "Choose a folder…";
pub const ADD_FILES: &str = "Add files";
pub const ADD_FOLDER: &str = "Add folder";
pub const CONVERT: &str = "Convert";
pub const CONVERT_ALL: &str = "Convert";
pub const CONVERT_THIS_FILE: &str = "Convert this file";
pub const CANCEL: &str = "Cancel";
pub const PAUSE: &str = "Pause";
pub const RESUME: &str = "Resume";
pub const OPEN_FOLDER: &str = "Open folder";
pub const VIEW_REPORT: &str = "View report";
pub const EXPORT: &str = "Export";
pub const EXPORT_HTML: &str = "HTML report";
pub const EXPORT_JSON: &str = "JSON report";
pub const EXPORT_CSV: &str = "CSV report";
pub const DONE: &str = "Done";
pub const BACK: &str = "Back";
pub const TRY_AGAIN: &str = "Try again";
pub const REMOVE: &str = "Remove";
pub const CLEAR_ALL: &str = "Clear all";
pub const CHANGE: &str = "Change";
pub const RETRY_FAILED: &str = "Retry failed";
pub const RETRY_SELECTED: &str = "Retry";
pub const REMOVE_COMPLETED: &str = "Remove converted";
pub const SHOW_DETAILS: &str = "Show details";
pub const HIDE_DETAILS: &str = "Hide details";
pub const COPY: &str = "Copy";
pub const COPY_ISSUE: &str = "Copy advice";
pub const COPY_LAYER: &str = "Copy layer name";
pub const CHECK_AGAIN: &str = "Check again";
pub const NEXT_TO_FILE: &str = "Next to each .fig file";
pub const USE_INPUT_FOLDER: &str = "Use input folder";
pub const OPEN_AFTER_CONVERSION: &str = "Open the output folder after a successful conversion";
pub const PAUSED_NOTE: &str = "Paused. Resume when you’re ready.";
pub const CLEAR_ALL_TITLE: &str = "Clear all files?";
pub const CLEAR_ALL_BODY: &str =
    "This removes every file from the list. Files on your computer aren’t deleted.";
pub const BRING_INTO_PENPOT: &str = "Bring it into Penpot";
pub const REPORT_FOR: &str = "Report for";
pub const LICENSE_FONT: &str = "Inter is licensed under the SIL Open Font License 1.1.";
pub const LICENSE_APP: &str = "Figma 2 Penpot is licensed under MIT OR Apache-2.0.";
pub const LICENSE_SCROLL: &str = "Scroll to read the full license.";
pub const RESET_TITLE: &str = "Reset preferences?";
pub const RESET_BODY: &str = "Theme, output folder, and conversion choices return to their defaults. Your files and converted outputs aren’t touched.";
pub const RESET_ACTION: &str = "Reset";
pub const DISMISS: &str = "Dismiss";
pub const FIGMA_FILTER: &str = "Figma";
pub const PREFS_SAVED: &str = "Preferences saved.";
pub const PREFS_RESET: &str = "Preferences reset.";
pub const NO_FAILED_TO_RETRY: &str = "There are no failed files to retry.";
pub const CANCELLING_BATCH: &str = "Cancelling the batch…";
pub const CANCELLING_FILE: &str = "Cancelling this file…";
pub const SCAN_TRUNCATED: &str = "We stopped after 512 files. Add the rest in a second pass.";
pub const REPORT_EXPORTED: &str = "Report exported.";
pub const REPORT_EXPORT_FAILED: &str = "We couldn’t export the report: ";
pub const OUTPUT_EXISTS: &str =
    "A file with this name already exists. Choose another folder or replace it in Advanced.";
pub const REPORT_EXISTS: &str =
    "A report with this name already exists. Choose another folder or replace it in Advanced.";
pub const REPORT_EXISTS_SHORT: &str =
    "A report with this name already exists. Choose another folder or replace it in Advanced.";
pub const CANCELLED_NO_OUTPUT: &str = "Conversion cancelled. No output was written.";
pub const OPEN_FOLDER_FAILED: &str = "We couldn’t open that folder: ";
pub const SAVE_PREFS_FAILED: &str = "We couldn’t save your preferences: ";
pub const OPEN_FOLDER_UNSUPPORTED: &str = "Opening folders isn’t supported on this platform.";
pub const NOT_AVAILABLE: &str = "Not available";
pub const UNKNOWN: &str = "Unknown";

// Home
pub const HOME_TITLE: &str = "Bring your Figma work to Penpot.";
pub const HOME_BODY: &str =
    "Drop your Figma files here. We’ll convert them and tell you exactly what to check.";
pub const DROP_FILES: &str = "Drop your Figma files here";
pub const DROP_RELEASE: &str = "Release to add these files";
pub const HOME_STEPS: [(&str, &str, &str); 3] = [
    ("1", "Add files", "Choose a folder or drop .fig files."),
    (
        "2",
        "Convert",
        "We keep the structure and record every change.",
    ),
    (
        "3",
        "Review in Penpot",
        "Open the folder and check what we flag.",
    ),
];
pub const LAST_SAVED_TO: &str = "Last saved to";

// Files
pub const FILES_TITLE: &str = "Your files";
pub const FILES_SUBTITLE: &str = "Nothing is converted until you press Convert.";
pub const FILES_EMPTY_TITLE: &str = "No files yet.";
pub const FILES_EMPTY_BODY: &str = "Add a .fig file or a folder to get started.";
pub const SAVE_TO: &str = "Save to";
pub const FILES_COUNT: &str = "files";
pub const FILE_ONLY_FIG: &str = "This isn’t a Figma file. Choose a .fig file.";
pub const FILE_UNREADABLE: &str = "Can’t be opened — try saving it again from Figma.";
pub const READY: &str = "Ready";
pub const ANALYZING: &str = "Analyzing…";
pub const QUEUED: &str = "Queued";
pub const CONVERTING: &str = "Converting…";
pub const CONVERTED: &str = "Converted";
pub const COULDNT_CONVERT: &str = "Couldn’t be converted";
pub const CANCELLED: &str = "Cancelled";

// Details
pub const DETAILS_TITLE: &str = "What’s in this file?";
pub const DETAILS_BACK: &str = "Files";
pub const DETAILS_COUNTS: [&str; 6] = [
    "Pages",
    "Components",
    "Styles",
    "Variables",
    "Images",
    "Text layers",
];
pub const THINGS_TO_KNOW: &str = "Things to be aware of";
pub const NOTHING_UNUSUAL: &str = "Nothing unusual found.";
pub const TECHNICAL_DETAILS: &str = "Technical details";
pub const ANALYSIS_PENDING: &str = "Reading this file…";
pub const ANALYSIS_FAILED: &str = "We couldn’t read this file. It may be damaged or saved by a newer version of Figma. Try saving it again from Figma.";

// Settings
pub const SETTINGS_TITLE: &str = "Conversion settings";
pub const WHERE_SAVE: &str = "Where should we save the converted files?";
pub const WHAT_MATTERS: &str = "What matters more?";
pub const LOOKS_IDENTICAL: &str = "Looks identical";
pub const LOOKS_IDENTICAL_NOTE: &str = "Keep the visual result, even if tokens aren’t linked.";
pub const KEEPS_TOKENS: &str = "Keeps tokens connected";
pub const KEEPS_TOKENS_NOTE: &str = "Prefer token and style links where Penpot supports them.";
pub const ADVANCED: &str = "Advanced";
pub const OVERWRITE_POLICY: &str = "If a file with this name exists";
pub const OVERWRITE_ASK: &str = "Ask me";
pub const OVERWRITE_REPLACE: &str = "Replace";
pub const OVERWRITE_KEEP_BOTH: &str = "Keep both";
pub const OVERWRITE_NOTE: &str =
    "“Keep both” is not available yet; we won’t silently rename files.";

// Converting
pub const CONVERTING_TITLE: &str = "Converting your files";
pub const STAGE_READING: &str = "Reading your file";
pub const STAGE_CONVERTING: &str = "Converting layers";
pub const STAGE_WRITING: &str = "Packaging for Penpot";
pub const STAGE_VALIDATING: &str = "Checking the result";
pub const STAGE_SAVING: &str = "Saving";
pub const KEEP_WORKING: &str =
    "You can keep working. Pause stops new files; Cancel stops the batch.";
pub const KEEP_WORKING_SINGLE: &str =
    "You can keep working. Pause holds this file between steps; Cancel stops it.";
pub const NO_OUTPUT_ON_CANCEL: &str = "Cancelled files leave no output behind.";
pub const NO_ACTIVE_JOB: &str = "No conversion is running.";

// Results
pub const RESULTS_TITLE: &str = "Your conversion is ready";
pub const RESULTS_TITLE_ATTENTION: &str = "Your conversion finished — some files need attention";
pub const RESULTS_TITLE_CANCELLED: &str = "Conversion cancelled";
pub const RESULTS_EMPTY: &str = "No conversion has finished yet.";
pub const CONVERTED_LABEL: &str = "Converted";
pub const NEEDS_REVIEW: &str = "Converted — needs a quick review";
pub const NEEDS_REVIEW_SHORT: &str = "Needs review";
pub const FAILED_LABEL: &str = "Couldn’t be converted";
pub const CANCELLED_LABEL: &str = "Cancelled";
pub const NEEDS_CHECK: &str = "things to check in Penpot";
pub const NOT_CONVERTED: &str = "Not converted";
pub const SKIPPED_LABEL: &str = "Skipped";
pub const IMPORT_GUIDE: &str =
    "In Penpot, open Drafts → Options → Import Penpot files, then choose the .penpot file.";
pub const BATCH_SUMMARY: &str = "converted";
pub const BATCH_REVIEW: &str = "need a quick review";
pub const BATCH_FAILED: &str = "couldn’t be converted";

// Report
pub const REPORT_TITLE: &str = "Conversion report";
pub const REPORT_EMPTY: &str = "No report is available.";
pub const REPORT_SEARCH: &str = "Search layers or pages";
pub const REPORT_ALL: &str = "All";
pub const REPORT_SEVERITY: &str = "Severity";
pub const REPORT_RESULT: &str = "Result";
pub const REPORT_CATEGORY: &str = "What to do";
pub const REPORT_GROUP: &str = "Group by";
pub const GROUP_NONE: &str = "None";
pub const GROUP_PAGE: &str = "Page";
pub const GROUP_CATEGORY: &str = "What to do";
pub const REPORT_ALL_GOOD: &str = "All good";
pub const REPORT_EMPTY_FILTER: &str = "Nothing matches this filter.";
pub const REPORT_NOTHING_ATTENTION: &str = "Nothing needs your attention.";
pub const WHAT_HAPPENED: &str = "What happened";
pub const WHAT_WE_DID: &str = "What we did instead";
pub const WHAT_TO_DO: &str = "What to do in Penpot";
pub const RAW_DETAILS: &str = "Converter details";
pub const PAGE: &str = "Page";
pub const LAYER_PATH: &str = "Layer path";
pub const EXPORT_REPORT: &str = "Export report";
pub const REPORT_TRACEABLE: &str =
    "Every issue keeps its original layer and page, even when grouped.";

// Preferences
pub const PREFERENCES_TITLE: &str = "Preferences";
pub const APPEARANCE: &str = "Appearance";
pub const THEME_SYSTEM: &str = "Follow system";
pub const THEME_LIGHT: &str = "Light";
pub const THEME_DARK: &str = "Dark";
pub const DEFAULT_OUTPUT: &str = "Default output folder";
pub const LANGUAGE: &str = "Language";
pub const LANGUAGE_ENGLISH: &str = "English";
pub const LANGUAGE_NOTE: &str = "More languages are planned.";
pub const PRIVACY_TITLE: &str = "Privacy";
pub const ABOUT: &str = "About";
pub const LICENSES: &str = "Licenses";
pub const VERSION: &str = "Version";
pub const RESET_PREFERENCES: &str = "Reset preferences";
pub const SETTINGS_FILE: &str = "Settings file";

// Notices / errors
pub const ADDED_FILES: &str = "file(s) added.";

pub fn added_message(count: usize) -> String {
    format!("Added {count} file{}.", if count == 1 { "" } else { "s" })
}

pub fn output_exists_message(path: &std::path::Path) -> String {
    format!("{OUTPUT_EXISTS} ({})", path.display())
}

pub fn batch_finished(completed: usize, failed: usize, cancelled: usize) -> String {
    format!("Batch finished: {completed} completed, {failed} failed, {cancelled} cancelled.")
}
pub const REJECTED: &str = "Couldn’t add";
pub const COULDNT_SCAN_FOLDER: &str = "We couldn’t read that folder";
pub const FALLBACK_ISSUE_TITLE: &str = "Something changed";
pub const FALLBACK_ISSUE_HAPPENED: &str =
    "This part could not be matched exactly during conversion.";
pub const FALLBACK_ISSUE_ACTION: &str =
    "Open the file in Penpot and compare this layer with the original.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueCategory {
    Attention,
    NotConverted,
    Review,
    AllGood,
}

impl IssueCategory {
    pub const ALL: [Self; 4] = [
        Self::Attention,
        Self::NotConverted,
        Self::Review,
        Self::AllGood,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Attention => "Needs your attention",
            Self::NotConverted => "Not converted",
            Self::Review => "Slightly different",
            Self::AllGood => "All good",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IssueCopy {
    pub title: &'static str,
    pub happened: &'static str,
    pub action: &'static str,
    pub category: IssueCategory,
}

const REVIEW_ACTION: &str = "Compare this layer in Penpot and adjust it if needed.";
const RECREATE_ACTION: &str = "Recreate this detail in Penpot.";
const NO_ACTION: &str = "Nothing to do.";

pub fn issue(code: &str) -> IssueCopy {
    let (title, happened, action, category) = match code {
        "F2P-NODE-CONVERTED" => (
            "Layer converted",
            "The layer was converted normally.",
            NO_ACTION,
            IssueCategory::AllGood,
        ),
        "F2P-NODE-APPROXIMATED" => (
            "Layer looks slightly different",
            "This layer needed a small approximation.",
            REVIEW_ACTION,
            IssueCategory::Review,
        ),
        "F2P-NODE-UNSUPPORTED" => (
            "Layer type isn’t supported",
            "Penpot has no matching layer type for this Figma layer.",
            RECREATE_ACTION,
            IssueCategory::NotConverted,
        ),
        "F2P-NODE-FAILED" => (
            "Layer couldn’t be converted",
            "This layer had geometry or data we couldn’t use.",
            "Check this layer in Figma, then try converting the file again.",
            IssueCategory::Attention,
        ),
        "F2P-GROUP-EMPTY" => (
            "Empty group skipped",
            "The group had no visible layers inside.",
            NO_ACTION,
            IssueCategory::NotConverted,
        ),
        "F2P-PAINT-UNSUPPORTED" => (
            "A fill was left out",
            "This fill type isn’t supported, so the other fills were kept.",
            RECREATE_ACTION,
            IssueCategory::Review,
        ),
        "F2P-STROKE-UNSUPPORTED" => (
            "A stroke was left out",
            "This stroke type isn’t supported, so the other strokes were kept.",
            RECREATE_ACTION,
            IssueCategory::Review,
        ),
        "F2P-STROKE-ALIGNMENT" => (
            "Stroke position changed",
            "The stroke alignment wasn’t recognized, so it was centered.",
            "Check the stroke position in Penpot.",
            IssueCategory::Review,
        ),
        "F2P-STROKE-WIDTH" => (
            "Stroke width was missing",
            "The stroke had an invalid width and was omitted.",
            "Reapply the stroke width in Penpot.",
            IssueCategory::Review,
        ),
        "F2P-VECTOR-STROKE-OUTLINE" => (
            "Line became a filled shape",
            "The original line had no fill geometry, so its outline was converted.",
            "Edit the shape as a filled path, or redraw it as a stroke.",
            IssueCategory::Review,
        ),
        "F2P-VECTOR-NETWORK-LOST" => (
            "Vector handles were simplified",
            "The rendered path is preserved, but the original handle network isn’t.",
            "Edit the path in Penpot if you need to change its curves.",
            IssueCategory::Review,
        ),
        "F2P-EFFECT-UNSUPPORTED" => (
            "An effect was left out",
            "Penpot has no matching effect for this one.",
            RECREATE_ACTION,
            IssueCategory::Review,
        ),
        "F2P-EFFECT-DUPLICATE" => (
            "Extra blur was combined",
            "Penpot keeps one layer blur and one background blur per layer.",
            "Check the blur in Penpot.",
            IssueCategory::Review,
        ),
        "F2P-LAYOUT-PARTIAL" => (
            "Auto layout sizing changed",
            "The layout direction and spacing were kept, but some child sizing wasn’t.",
            "Check how the container resizes in Penpot.",
            IssueCategory::Review,
        ),
        "F2P-LAYOUT-GRID-UNSUPPORTED" => (
            "Grid layout wasn’t kept",
            "The container was converted without its grid layout.",
            "Recreate the grid in Penpot.",
            IssueCategory::NotConverted,
        ),
        "F2P-MASK-APPROXIMATED" => (
            "Mask may look different",
            "The mask became a Penpot masked group, but its mode wasn’t preserved.",
            "Check the clipping in Penpot.",
            IssueCategory::Review,
        ),
        "F2P-MASK-NO-TARGET" => (
            "Mask had nothing to clip",
            "The mask had no following sibling layers.",
            "Check this layer in Penpot.",
            IssueCategory::Review,
        ),
        "F2P-BOOLEAN-FLATTENED" => (
            "Boolean became editable shapes",
            "The rendered result was kept, but the original boolean operands weren’t.",
            "Rebuild the boolean in Penpot if you need to edit its parts.",
            IssueCategory::Review,
        ),
        "F2P-BLEND-MODE-LOST" => (
            "Blend mode changed",
            "This blend mode has no exact Penpot equivalent.",
            "Reapply the blend mode in Penpot.",
            IssueCategory::Review,
        ),
        "F2P-CONSTRAINT-APPROXIMATED" => (
            "Resize behavior changed",
            "A fixed-size constraint was mapped to the closest Penpot option.",
            "Check how this layer resizes in Penpot.",
            IssueCategory::Review,
        ),
        "F2P-STYLE-EFFECT-INLINED" => (
            "Effect style was copied onto the layer",
            "The effect is there, but it isn’t linked to a shared effect style.",
            "Edit the effect directly, or create a shared style in Penpot.",
            IssueCategory::Review,
        ),
        "F2P-STYLE-GRID-LOST" => (
            "Grid style wasn’t kept",
            "The shared grid style was not converted.",
            "Recreate the grid style in Penpot.",
            IssueCategory::NotConverted,
        ),
        "F2P-INSTANCE-DETACHED" => (
            "Component copy was detached",
            "The main component wasn’t available in the converted pages.",
            "Reconnect this layer to a local component in Penpot.",
            IssueCategory::Attention,
        ),
        "F2P-INSTANCE-OVERRIDE-LOST" => (
            "A component change was reset",
            "An instance override couldn’t be matched to the main component.",
            "Reapply this change in Penpot.",
            IssueCategory::Review,
        ),
        "F2P-INSTANCE-ATTRIBUTES-UNSUPPORTED" => (
            "Some component settings need checking",
            "Some instance settings have no conversion mapping yet.",
            "Check these settings against the original design in Penpot.",
            IssueCategory::Review,
        ),
        "F2P-TOKEN-NAME-NORMALIZED" => (
            "A token name was adjusted",
            "Penpot needs a different name for this token.",
            "Use the adjusted name when connecting other designs.",
            IssueCategory::Review,
        ),
        "F2P-TEXT-MIXED-STYLES-LOST" => (
            "Mixed text styles were flattened",
            "The text is intact, but per-character styles were reduced to the base style.",
            "Restyle individual characters in Penpot.",
            IssueCategory::Review,
        ),
        "F2P-VECTOR-EMPTY" => (
            "Empty vector skipped",
            "The vector had no usable geometry.",
            NO_ACTION,
            IssueCategory::NotConverted,
        ),
        "F2P-VECTOR-FAILED" => (
            "Vector couldn’t be converted",
            "The path data was invalid or incomplete.",
            "Check this vector in Figma, then try again.",
            IssueCategory::Attention,
        ),
        "F2P-TEXT-EMPTY" => (
            "Empty text skipped",
            "The text layer had no characters.",
            NO_ACTION,
            IssueCategory::NotConverted,
        ),
        _ => (
            FALLBACK_ISSUE_TITLE,
            FALLBACK_ISSUE_HAPPENED,
            FALLBACK_ISSUE_ACTION,
            IssueCategory::Review,
        ),
    };
    IssueCopy {
        title,
        happened,
        action,
        category,
    }
}

pub const KNOWN_ISSUE_CODES: [&str; 30] = [
    "F2P-NODE-CONVERTED",
    "F2P-NODE-APPROXIMATED",
    "F2P-NODE-UNSUPPORTED",
    "F2P-NODE-FAILED",
    "F2P-GROUP-EMPTY",
    "F2P-PAINT-UNSUPPORTED",
    "F2P-STROKE-UNSUPPORTED",
    "F2P-STROKE-ALIGNMENT",
    "F2P-STROKE-WIDTH",
    "F2P-VECTOR-STROKE-OUTLINE",
    "F2P-VECTOR-NETWORK-LOST",
    "F2P-EFFECT-UNSUPPORTED",
    "F2P-EFFECT-DUPLICATE",
    "F2P-LAYOUT-PARTIAL",
    "F2P-LAYOUT-GRID-UNSUPPORTED",
    "F2P-MASK-APPROXIMATED",
    "F2P-MASK-NO-TARGET",
    "F2P-BOOLEAN-FLATTENED",
    "F2P-BLEND-MODE-LOST",
    "F2P-CONSTRAINT-APPROXIMATED",
    "F2P-STYLE-EFFECT-INLINED",
    "F2P-STYLE-GRID-LOST",
    "F2P-INSTANCE-DETACHED",
    "F2P-INSTANCE-OVERRIDE-LOST",
    "F2P-INSTANCE-ATTRIBUTES-UNSUPPORTED",
    "F2P-TOKEN-NAME-NORMALIZED",
    "F2P-TEXT-MIXED-STYLES-LOST",
    "F2P-VECTOR-EMPTY",
    "F2P-VECTOR-FAILED",
    "F2P-TEXT-EMPTY",
];

pub fn what_we_did(category: IssueCategory) -> &'static str {
    match category {
        IssueCategory::Attention => {
            "We skipped this layer so the rest of the file could finish converting."
        }
        IssueCategory::NotConverted => {
            "We left this part out and kept everything else that could be converted."
        }
        IssueCategory::Review => "We kept the closest Penpot equivalent.",
        IssueCategory::AllGood => "We converted it normally.",
    }
}

pub fn category_of(event: &conversion_report::ReportEvent) -> IssueCategory {
    if event.severity == Severity::Error || event.outcome == Outcome::Failed {
        IssueCategory::Attention
    } else if matches!(event.outcome, Outcome::Unsupported | Outcome::Skipped) {
        IssueCategory::NotConverted
    } else if event.outcome == Outcome::ConvertedWithModification {
        IssueCategory::Review
    } else {
        IssueCategory::AllGood
    }
}

pub fn file_status(status: FileStatus) -> (&'static str, IssueCategory) {
    match status {
        FileStatus::Completed => (CONVERTED_LABEL, IssueCategory::AllGood),
        FileStatus::CompletedWithWarnings => (NEEDS_REVIEW, IssueCategory::Review),
        FileStatus::Failed => (FAILED_LABEL, IssueCategory::Attention),
        FileStatus::Cancelled => (CANCELLED_LABEL, IssueCategory::NotConverted),
    }
}

pub fn validation(state: ValidationState) -> &'static str {
    match state {
        ValidationState::Passed => "File check passed",
        ValidationState::Failed => "File check failed — open it in Penpot and look it over",
        ValidationState::NotRun => "File check didn’t run — look it over in Penpot",
    }
}

pub fn severity(severity: Severity) -> &'static str {
    match severity {
        Severity::Info => "Good to know",
        Severity::Warning => "Needs a quick look",
        Severity::Error => "Needs your attention",
    }
}

pub fn outcome(outcome: Outcome) -> &'static str {
    match outcome {
        Outcome::Converted => "Converted",
        Outcome::ConvertedWithModification => "Slightly different",
        Outcome::Skipped => "Skipped",
        Outcome::Unsupported => "Not converted",
        Outcome::Failed => "Couldn’t convert",
    }
}

pub fn stage(stage: Stage) -> &'static str {
    match stage {
        Stage::Reading => STAGE_READING,
        Stage::Converting => STAGE_CONVERTING,
        Stage::Writing => STAGE_WRITING,
        Stage::Validating => STAGE_VALIDATING,
        Stage::Saving => STAGE_SAVING,
    }
}

pub fn queue_state(state: &QueueState) -> (&'static str, Option<String>) {
    match state {
        QueueState::Ready => ("Ready", None),
        QueueState::Analyzing => ("Analyzing…", None),
        QueueState::Analyzed => ("Ready", None),
        QueueState::Queued => ("Queued", None),
        QueueState::Converting => ("Converting…", None),
        QueueState::Converted => ("Converted", None),
        QueueState::Failed(message) => ("Couldn’t be converted", Some(failure_message(message))),
        QueueState::Cancelled => ("Cancelled", None),
    }
}

/// Translate worker/engine errors into designer-facing text. The raw message is kept in
/// reports and behind "Show details"; this function only decides what is shown first.
pub fn failure_message(raw: &str) -> String {
    if raw == "Analysis failed" {
        ANALYSIS_FAILED.to_owned()
    } else if raw == "Conversion failed" {
        "We couldn’t convert this file. Try again; if it keeps happening, check it in Figma."
            .to_owned()
    } else if raw == "Conversion cancelled" {
        "Conversion cancelled. No output was written.".to_owned()
    } else if raw.starts_with("Output already exists") || raw.starts_with("Report already exists") {
        "A file with this name already exists in that folder. Choose another folder, or allow replacing files in Advanced settings.".to_owned()
    } else if raw.starts_with("Reading ") || raw.starts_with("Could not read") {
        "We couldn’t read this file. It may be damaged or saved by a newer version of Figma. Try re-saving it from Figma.".to_owned()
    } else if raw.starts_with("Converting ") {
        "We couldn’t convert this file. Try again; if it keeps happening, check it in Figma."
            .to_owned()
    } else if raw.starts_with("Generated archive failed validation") {
        "The converted file didn’t pass our final check. Try again; if it keeps happening, report it.".to_owned()
    } else if raw.starts_with("Writing ") || raw.starts_with("Could not create output directory") {
        "We couldn’t save the converted file. Check the output folder and try again.".to_owned()
    } else if raw.starts_with("Building the Penpot archive failed") {
        "We couldn’t package the converted file. Try again; if it keeps happening, report it."
            .to_owned()
    } else {
        "Something went wrong while converting this file. Try again; the technical details are below.".to_owned()
    }
}

pub fn queue_category(state: &QueueState) -> IssueCategory {
    match state {
        QueueState::Ready | QueueState::Analyzed => IssueCategory::AllGood,
        QueueState::Analyzing | QueueState::Queued | QueueState::Converting => {
            IssueCategory::Review
        }
        QueueState::Converted => IssueCategory::AllGood,
        QueueState::Failed(_) => IssueCategory::Attention,
        QueueState::Cancelled => IssueCategory::NotConverted,
    }
}

pub fn analysis_hints(summary: &AnalysisSummary) -> Vec<String> {
    let mut hints = Vec::new();
    if summary.mixed_text_layers > 0 {
        hints.push(format!(
            "{} text layer(s) use mixed character styles. They will be flattened to the base style.",
            summary.mixed_text_layers
        ));
    }
    if summary.boolean_layers > 0 {
        hints.push(format!(
            "{} boolean shape(s) become editable paths. The original operands won’t be reconstructed.",
            summary.boolean_layers
        ));
    }
    if summary.mask_layers > 0 {
        hints.push(format!(
            "{} mask(s) become masked groups. The mask mode may look slightly different.",
            summary.mask_layers
        ));
    }
    if summary.vector_networks > 0 {
        hints.push(format!(
            "{} vector network(s) keep their rendered path, but not the original handles.",
            summary.vector_networks
        ));
    }
    if summary.auto_layouts > 0 {
        hints.push(format!(
            "{} auto layout container(s) keep direction and spacing; some child sizing may differ.",
            summary.auto_layouts
        ));
    }
    hints
}

pub fn screen(screen: Screen) -> &'static str {
    match screen {
        Screen::Home => NAV_IMPORT,
        Screen::Queue => NAV_FILES,
        Screen::Analysis => NAV_DETAILS,
        Screen::Settings => NAV_SETTINGS,
        Screen::Progress => NAV_CONVERT,
        Screen::Results => NAV_RESULTS,
        Screen::Report => NAV_REPORT,
        Screen::Preferences => NAV_PREFERENCES,
    }
}

pub fn batch_summary(converted: usize, review: usize, failed: usize, cancelled: usize) -> String {
    let review_text = if review == 1 {
        "1 needs a quick review"
    } else {
        &format!("{review} need a quick review")
    };
    format!(
        "{converted} converted · {review_text} · {failed} couldn’t be converted · {cancelled} cancelled"
    )
}

pub fn report_summary(converted: u64, review: u64, not_converted: u64) -> String {
    format!(
        "{converted} layers converted · {review} need a quick review · {not_converted} weren’t converted"
    )
}

pub fn review_count(report: &conversion_report::FileReport) -> usize {
    let mut keys = std::collections::BTreeSet::new();
    for event in &report.events {
        if event.severity != Severity::Info || event.outcome != Outcome::Converted {
            keys.insert(event.element.figma_node_id.as_str());
        }
    }
    keys.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_issue_code_has_designer_copy() {
        for code in KNOWN_ISSUE_CODES {
            let copy = issue(code);
            assert!(!copy.title.is_empty(), "{code} title");
            assert!(!copy.happened.is_empty(), "{code} happened");
            assert!(!copy.action.is_empty(), "{code} action");
        }
    }

    #[test]
    fn unknown_issue_falls_back_without_hiding_it() {
        let copy = issue("F2P-NOT-REAL");
        assert_eq!(copy.title, FALLBACK_ISSUE_TITLE);
        assert_eq!(copy.category, IssueCategory::Review);
    }

    #[test]
    fn every_status_outcome_severity_and_validation_maps() {
        for status in [
            FileStatus::Completed,
            FileStatus::CompletedWithWarnings,
            FileStatus::Failed,
            FileStatus::Cancelled,
        ] {
            assert!(!file_status(status).0.is_empty());
        }
        for state in [
            ValidationState::Passed,
            ValidationState::Failed,
            ValidationState::NotRun,
        ] {
            assert!(!validation(state).is_empty());
        }
        for value in [Severity::Info, Severity::Warning, Severity::Error] {
            assert!(!severity(value).is_empty());
        }
        for value in [
            Outcome::Converted,
            Outcome::ConvertedWithModification,
            Outcome::Skipped,
            Outcome::Unsupported,
            Outcome::Failed,
        ] {
            assert!(!outcome(value).is_empty());
        }
        for value in Stage::ALL {
            assert!(!stage(value).is_empty());
        }
        for value in [
            QueueState::Ready,
            QueueState::Analyzing,
            QueueState::Analyzed,
            QueueState::Queued,
            QueueState::Converting,
            QueueState::Converted,
            QueueState::Failed("reason".to_owned()),
            QueueState::Cancelled,
        ] {
            assert!(!queue_state(&value).0.is_empty());
        }
        for value in [
            IssueCategory::Attention,
            IssueCategory::NotConverted,
            IssueCategory::Review,
            IssueCategory::AllGood,
        ] {
            assert!(!value.label().is_empty());
        }
    }

    #[test]
    fn raw_worker_errors_get_designer_copy() {
        assert!(failure_message("Output already exists: /tmp/a.penpot").contains("already exists"));
        assert!(failure_message("Reading /tmp/a.fig failed: corrupt").contains("couldn’t read"));
        assert!(failure_message("something unexpected").contains("technical details"));
    }

    #[test]
    fn categories_keep_bad_news_visible() {
        let mut event = conversion_report::ReportEvent {
            code: "F2P-NODE-UNSUPPORTED".to_owned(),
            severity: Severity::Warning,
            outcome: Outcome::Unsupported,
            element: conversion_report::ElementRef {
                figma_node_id: "1:1".to_owned(),
                name: "Layer".to_owned(),
                element_type: "TEXT".to_owned(),
                page: None,
                path: vec!["Layer".to_owned()],
            },
            description: "raw".to_owned(),
            reason: None,
            action_taken: "skipped".to_owned(),
            suggested_action: None,
        };
        assert_eq!(category_of(&event), IssueCategory::NotConverted);
        event.outcome = Outcome::Failed;
        event.severity = Severity::Error;
        assert_eq!(category_of(&event), IssueCategory::Attention);
        event.outcome = Outcome::ConvertedWithModification;
        event.severity = Severity::Warning;
        assert_eq!(category_of(&event), IssueCategory::Review);
        event.outcome = Outcome::Converted;
        event.severity = Severity::Info;
        assert_eq!(category_of(&event), IssueCategory::AllGood);
    }
}
