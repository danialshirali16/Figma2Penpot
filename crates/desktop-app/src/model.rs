//! Headless desktop state. No egui types live here, so every transition can be
//! unit-tested without a window or GPU.

use std::fs;
use std::path::{Path, PathBuf};

use conversion_report::{FileReport, FileStatus, Outcome, Severity};
use fig_reader::FigDocument;

use crate::settings::Settings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Home,
    Queue,
    Analysis,
    Settings,
    Progress,
    Results,
    Report,
    Preferences,
}

impl Screen {
    pub const ALL: [Self; 8] = [
        Self::Home,
        Self::Queue,
        Self::Analysis,
        Self::Settings,
        Self::Progress,
        Self::Results,
        Self::Report,
        Self::Preferences,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Reading,
    Converting,
    Writing,
    Validating,
    Saving,
}

impl Stage {
    pub const ALL: [Self; 5] = [
        Self::Reading,
        Self::Converting,
        Self::Writing,
        Self::Validating,
        Self::Saving,
    ];

    pub fn index(self) -> usize {
        match self {
            Self::Reading => 0,
            Self::Converting => 1,
            Self::Writing => 2,
            Self::Validating => 3,
            Self::Saving => 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalysisSummary {
    pub document_name: Option<String>,
    pub container: String,
    pub format: String,
    pub version: u32,
    pub pages: usize,
    pub internal_pages: usize,
    pub nodes: usize,
    pub images: usize,
    pub image_bytes: u64,
    pub warnings: usize,
    pub max_depth: usize,
}

impl AnalysisSummary {
    pub fn from_document(document: &FigDocument) -> Self {
        Self {
            document_name: document.meta_file_name().map(str::to_owned),
            container: document.container.to_string(),
            format: document.prelude.clone(),
            version: document.version,
            pages: document
                .pages
                .iter()
                .filter(|page| !page.internal_only)
                .count(),
            internal_pages: document
                .pages
                .iter()
                .filter(|page| page.internal_only)
                .count(),
            nodes: document.stats.active_nodes,
            images: document.stats.image_count,
            image_bytes: document.stats.image_bytes,
            warnings: document.warnings.len(),
            max_depth: document.stats.max_depth,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueueState {
    Ready,
    Analyzing,
    Analyzed,
    Converted,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct QueueItem {
    pub path: PathBuf,
    pub size: u64,
    pub state: QueueState,
    pub analysis: Option<AnalysisSummary>,
    pub analysis_error: Option<String>,
}

impl QueueItem {
    pub fn new(path: PathBuf, size: u64) -> Self {
        Self {
            path,
            size,
            state: QueueState::Ready,
            analysis: None,
            analysis_error: None,
        }
    }

    pub fn display_name(&self) -> String {
        self.path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.path.display().to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoticeKind {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Notice {
    pub kind: NoticeKind,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct ActiveJob {
    pub input: PathBuf,
    pub output: PathBuf,
    pub stage: Stage,
}

#[derive(Debug, Clone)]
pub struct ConversionOutcome {
    pub input: PathBuf,
    pub output: PathBuf,
    pub report_path: PathBuf,
    pub report: FileReport,
    pub validation_checks: usize,
}

#[derive(Debug, Clone)]
pub struct ConversionFailure {
    pub input: PathBuf,
    pub output: Option<PathBuf>,
    pub report_path: Option<PathBuf>,
    pub report: Option<FileReport>,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct ReportFilter {
    pub severity: Option<Severity>,
    pub outcome: Option<Outcome>,
    pub search: String,
}

impl ReportFilter {
    pub fn matches(&self, event: &conversion_report::ReportEvent) -> bool {
        if let Some(severity) = self.severity {
            if event.severity != severity {
                return false;
            }
        }
        if let Some(outcome) = self.outcome {
            if event.outcome != outcome {
                return false;
            }
        }
        let needle = self.search.trim().to_lowercase();
        if needle.is_empty() {
            return true;
        }
        event.code.to_lowercase().contains(&needle)
            || event.description.to_lowercase().contains(&needle)
            || event.element.name.to_lowercase().contains(&needle)
            || event
                .element
                .page
                .as_deref()
                .is_some_and(|page| page.to_lowercase().contains(&needle))
    }
}

#[derive(Debug, Default)]
pub struct FolderScan {
    pub files: Vec<PathBuf>,
    pub truncated: bool,
}

/// Scan a folder for `.fig` files without following symlinks or descending into
/// hidden folders. The result is bounded so a mistaken folder choice cannot
/// stall the UI.
pub fn scan_folder(root: &Path, max_depth: usize) -> std::io::Result<FolderScan> {
    const MAX_FILES: usize = 512;
    let mut scan = FolderScan::default();
    let mut stack = vec![(root.to_path_buf(), 0usize)];
    while let Some((dir, depth)) = stack.pop() {
        if depth > max_depth || scan.files.len() >= MAX_FILES {
            scan.truncated = true;
            continue;
        }
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_symlink() {
                continue;
            }
            let path = entry.path();
            if file_type.is_dir() {
                let hidden = path
                    .file_name()
                    .map(|name| name.to_string_lossy().starts_with('.'))
                    .unwrap_or(false);
                if !hidden {
                    stack.push((path, depth + 1));
                }
            } else if file_type.is_file()
                && path
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("fig"))
            {
                if scan.files.len() >= MAX_FILES {
                    scan.truncated = true;
                    return Ok(scan);
                }
                scan.files.push(path);
            }
        }
    }
    scan.files.sort();
    Ok(scan)
}

#[derive(Debug, Default)]
pub struct AddOutcome {
    pub added: usize,
    pub rejected: Vec<(PathBuf, String)>,
}

pub struct AppState {
    pub screen: Screen,
    pub files: Vec<QueueItem>,
    pub selected: Option<usize>,
    pub settings: Settings,
    pub active: Option<ActiveJob>,
    pub result: Option<ConversionOutcome>,
    pub notice: Option<Notice>,
    pub filter: ReportFilter,
    pub settings_path: PathBuf,
}

impl AppState {
    pub fn new(settings: Settings, settings_path: PathBuf) -> Self {
        Self {
            screen: Screen::Home,
            files: Vec::new(),
            selected: None,
            settings,
            active: None,
            result: None,
            notice: None,
            filter: ReportFilter::default(),
            settings_path,
        }
    }

    pub fn set_screen(&mut self, screen: Screen) {
        self.screen = screen;
    }

    pub fn notify(&mut self, kind: NoticeKind, text: impl Into<String>) {
        self.notice = Some(Notice {
            kind,
            text: text.into(),
        });
    }

    pub fn clear_notice(&mut self) {
        self.notice = None;
    }

    pub fn add_paths<I>(&mut self, paths: I) -> AddOutcome
    where
        I: IntoIterator<Item = PathBuf>,
    {
        let mut outcome = AddOutcome::default();
        for path in paths {
            if self.files.iter().any(|item| item.path == path) {
                continue;
            }
            let extension = path
                .extension()
                .map(|value| value.to_string_lossy().to_ascii_lowercase())
                .unwrap_or_default();
            if extension != "fig" {
                outcome.rejected.push((
                    path,
                    "Only .fig files are supported; .deck and .jam are out of scope.".to_owned(),
                ));
                continue;
            }
            let metadata = match fs::metadata(&path) {
                Ok(metadata) => metadata,
                Err(error) => {
                    outcome
                        .rejected
                        .push((path, format!("Could not read file: {error}")));
                    continue;
                }
            };
            if !metadata.is_file() {
                outcome
                    .rejected
                    .push((path, "Not a regular file.".to_owned()));
                continue;
            }
            self.files.push(QueueItem::new(path, metadata.len()));
            outcome.added += 1;
        }
        if self.selected.is_none() && !self.files.is_empty() {
            self.selected = Some(0);
        }
        outcome
    }

    pub fn selected_item(&self) -> Option<&QueueItem> {
        self.selected.and_then(|index| self.files.get(index))
    }

    pub fn selected_item_mut(&mut self) -> Option<&mut QueueItem> {
        self.selected.and_then(|index| self.files.get_mut(index))
    }

    pub fn select(&mut self, index: usize) {
        if index < self.files.len() {
            self.selected = Some(index);
        }
    }

    pub fn remove_selected(&mut self) {
        if let Some(index) = self.selected {
            self.remove(index);
        }
    }

    pub fn remove(&mut self, index: usize) {
        if index >= self.files.len() {
            return;
        }
        self.files.remove(index);
        self.selected = if self.files.is_empty() {
            None
        } else {
            Some(index.min(self.files.len() - 1))
        };
    }

    pub fn clear_queue(&mut self) {
        self.files.clear();
        self.selected = None;
    }

    pub fn analysis_needed(&self) -> Option<PathBuf> {
        let item = self.selected_item()?;
        if item.analysis.is_some()
            || item.analysis_error.is_some()
            || item.state == QueueState::Analyzing
        {
            return None;
        }
        Some(item.path.clone())
    }

    pub fn mark_analyzing(&mut self, path: &Path) {
        if let Some(item) = self.files.iter_mut().find(|item| item.path == path) {
            item.state = QueueState::Analyzing;
        }
    }

    pub fn analysis_finished(&mut self, path: &Path, summary: AnalysisSummary) {
        if let Some(item) = self.files.iter_mut().find(|item| item.path == path) {
            item.analysis = Some(summary);
            item.analysis_error = None;
            item.state = QueueState::Analyzed;
        }
    }

    pub fn reset_analysis(&mut self, path: &Path) {
        if let Some(item) = self.files.iter_mut().find(|item| item.path == path) {
            item.analysis = None;
            item.analysis_error = None;
            item.state = QueueState::Ready;
        }
    }

    pub fn analysis_failed(&mut self, path: &Path, message: String) {
        if let Some(item) = self.files.iter_mut().find(|item| item.path == path) {
            item.analysis_error = Some(message);
            item.state = QueueState::Failed("Analysis failed".to_owned());
        }
    }

    pub fn start_job(&mut self, input: PathBuf, output: PathBuf) {
        self.active = Some(ActiveJob {
            input,
            output,
            stage: Stage::Reading,
        });
        self.screen = Screen::Progress;
        self.notice = None;
    }

    pub fn job_stage(&mut self, input: &Path, stage: Stage) {
        if let Some(active) = self.active.as_mut() {
            if active.input == input {
                active.stage = stage;
            }
        }
    }

    pub fn finish_job(&mut self, outcome: ConversionOutcome) {
        if let Some(item) = self
            .files
            .iter_mut()
            .find(|item| item.path == outcome.input)
        {
            item.state = match outcome.report.status {
                FileStatus::Failed => QueueState::Failed("Conversion failed".to_owned()),
                FileStatus::Cancelled => QueueState::Failed("Conversion cancelled".to_owned()),
                FileStatus::Completed | FileStatus::CompletedWithWarnings => QueueState::Converted,
            };
        }
        self.active = None;
        self.result = Some(outcome);
        self.filter = ReportFilter::default();
        self.screen = Screen::Results;
    }

    pub fn fail_job(&mut self, failure: ConversionFailure) {
        if let Some(item) = self
            .files
            .iter_mut()
            .find(|item| item.path == failure.input)
        {
            item.state = QueueState::Failed(failure.message.clone());
        }
        self.active = None;
        if let (Some(report), Some(output), Some(report_path)) =
            (failure.report, failure.output, failure.report_path)
        {
            self.result = Some(ConversionOutcome {
                input: failure.input,
                output,
                report_path,
                report,
                validation_checks: 0,
            });
            self.filter = ReportFilter::default();
            self.screen = Screen::Results;
        }
        self.notify(NoticeKind::Error, failure.message);
    }

    pub fn output_path_for(&self, input: &Path) -> PathBuf {
        let stem = input
            .file_stem()
            .map(|value| value.to_string_lossy().into_owned())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "output".to_owned());
        self.settings
            .resolved_output_dir(input)
            .join(format!("{stem}.penpot"))
    }

    pub fn report_path_for(&self, input: &Path) -> PathBuf {
        let stem = input
            .file_stem()
            .map(|value| value.to_string_lossy().into_owned())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "output".to_owned());
        self.settings
            .resolved_output_dir(input)
            .join(format!("{stem}.report.json"))
    }

    pub fn filtered_events(&self) -> Vec<&conversion_report::ReportEvent> {
        self.result
            .as_ref()
            .map(|result| {
                result
                    .report
                    .events
                    .iter()
                    .filter(|event| self.filter.matches(event))
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_fig(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, b"fig-test").unwrap();
        path
    }

    fn state() -> AppState {
        AppState::new(Settings::default(), PathBuf::from("/tmp/settings.json"))
    }

    fn report(status: FileStatus) -> FileReport {
        FileReport {
            schema_version: 1,
            source: "a.fig".to_owned(),
            output: Some("a.penpot".to_owned()),
            status,
            duration_ms: 5,
            counts: Default::default(),
            events: Vec::new(),
            validation: conversion_report::ValidationState::Passed,
            tool_version: "test".to_owned(),
            target_penpot: "2.18.5".to_owned(),
        }
    }

    #[test]
    fn adds_only_fig_files_and_deduplicates() {
        let dir = tempfile::tempdir().unwrap();
        let fig = temp_fig(dir.path(), "one.fig");
        let txt = temp_fig(dir.path(), "two.txt");
        let mut state = state();
        let outcome = state.add_paths(vec![fig.clone(), fig.clone(), txt]);
        assert_eq!(outcome.added, 1);
        assert_eq!(outcome.rejected.len(), 1);
        assert_eq!(state.files.len(), 1);
        assert_eq!(state.selected, Some(0));
    }

    #[test]
    fn folder_scan_finds_fig_files_and_skips_hidden_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("nested");
        let hidden = dir.path().join(".hidden");
        fs::create_dir_all(&nested).unwrap();
        fs::create_dir_all(&hidden).unwrap();
        temp_fig(dir.path(), "top.fig");
        temp_fig(&nested, "deep.fig");
        temp_fig(&hidden, "skip.fig");
        temp_fig(&nested, "notes.txt");
        let scan = scan_folder(dir.path(), 4).unwrap();
        assert_eq!(scan.files.len(), 2);
        assert!(scan.files.iter().any(|path| path.ends_with("top.fig")));
        assert!(scan.files.iter().any(|path| path.ends_with("deep.fig")));
        assert!(!scan.truncated);
    }

    #[test]
    fn removing_selected_keeps_a_valid_index() {
        let dir = tempfile::tempdir().unwrap();
        let a = temp_fig(dir.path(), "a.fig");
        let b = temp_fig(dir.path(), "b.fig");
        let mut state = state();
        state.add_paths(vec![a, b]);
        state.select(1);
        state.remove_selected();
        assert_eq!(state.files.len(), 1);
        assert_eq!(state.selected, Some(0));
    }

    #[test]
    fn finish_job_moves_to_results_and_marks_the_queue_item() {
        let dir = tempfile::tempdir().unwrap();
        let input = temp_fig(dir.path(), "a.fig");
        let mut state = state();
        state.add_paths(vec![input.clone()]);
        state.start_job(input.clone(), PathBuf::from("/tmp/a.penpot"));
        state.job_stage(&input, Stage::Validating);
        assert_eq!(
            state.active.as_ref().map(|job| job.stage),
            Some(Stage::Validating)
        );
        state.finish_job(ConversionOutcome {
            input: input.clone(),
            output: PathBuf::from("/tmp/a.penpot"),
            report_path: PathBuf::from("/tmp/a.report.json"),
            report: report(FileStatus::CompletedWithWarnings),
            validation_checks: 10,
        });
        assert!(state.active.is_none());
        assert_eq!(state.screen, Screen::Results);
        assert_eq!(state.files[0].state, QueueState::Converted);
    }

    #[test]
    fn failed_job_with_a_report_still_opens_results() {
        let dir = tempfile::tempdir().unwrap();
        let input = temp_fig(dir.path(), "a.fig");
        let mut state = state();
        state.add_paths(vec![input.clone()]);
        state.start_job(input.clone(), PathBuf::from("/tmp/a.penpot"));
        state.fail_job(ConversionFailure {
            input,
            output: Some(PathBuf::from("/tmp/a.penpot")),
            report_path: Some(PathBuf::from("/tmp/a.report.json")),
            report: Some(report(FileStatus::Failed)),
            message: "validation failed".to_owned(),
        });
        assert_eq!(state.screen, Screen::Results);
        assert_eq!(
            state.notice.as_ref().map(|n| n.kind),
            Some(NoticeKind::Error)
        );
    }

    #[test]
    fn report_filter_matches_code_name_and_page() {
        let event = conversion_report::ReportEvent {
            code: "F2P-TEXT-EMPTY".to_owned(),
            severity: Severity::Warning,
            outcome: Outcome::Skipped,
            element: conversion_report::ElementRef {
                figma_node_id: "1:2".to_owned(),
                name: "Label".to_owned(),
                element_type: "TEXT".to_owned(),
                page: Some("Page 1".to_owned()),
                path: vec!["Page 1".to_owned(), "Label".to_owned()],
            },
            description: "empty text".to_owned(),
            reason: None,
            action_taken: "skipped".to_owned(),
            suggested_action: None,
        };
        let mut filter = ReportFilter {
            search: "label".to_owned(),
            ..ReportFilter::default()
        };
        assert!(filter.matches(&event));
        filter.search = "missing".to_owned();
        assert!(!filter.matches(&event));
        filter.search.clear();
        filter.severity = Some(Severity::Warning);
        assert!(filter.matches(&event));
        filter.severity = Some(Severity::Error);
        assert!(!filter.matches(&event));
    }
}
