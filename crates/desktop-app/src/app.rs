//! Native shell: navigation, worker plumbing, dialogs, and action dispatch. Screens live
//! in `views.rs`; design tokens in `theme.rs`; copy in `copy.rs`.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use conversion_queue::{BatchRunner, JobResult, QueueConfig, QueueControl, QueueEvent};
use conversion_report::FileReport;
use eframe::egui::{self, Align, Layout, RichText, Vec2};
use eframe::{CreationContext, Frame};

use crate::components::{self, Icon};
use crate::conversion::{
    spawn_analysis, spawn_conversion, ConversionControl, ConversionRequest, WorkerEvent,
};
use crate::copy;
use crate::model::{
    scan_folder, ActiveJob, AnalysisSummary, AppState, ConversionOutcome, NoticeKind, QueueState,
    Screen, Stage,
};
use crate::settings::{Settings, ThemeChoice};
use crate::theme::{self, SPACE_2, SPACE_4, SPACE_6};
use crate::views::{self, UiActions};

#[derive(Debug, Clone, Copy)]
enum ReportFormat {
    Html,
    Csv,
    Json,
}

pub struct F2pApp {
    state: AppState,
    sender: Sender<WorkerEvent>,
    receiver: Receiver<WorkerEvent>,
    analysis_in_flight: Option<PathBuf>,
    batch_receiver: Option<Receiver<QueueEvent>>,
    batch_control: Option<QueueControl>,
    batch_handle: Option<std::thread::JoinHandle<Vec<JobResult>>>,
    active_control: Option<ConversionControl>,
    gallery: bool,
}

impl F2pApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        Self::new_with_paths(cc, Vec::new())
    }

    pub fn new_with_paths(cc: &CreationContext<'_>, paths: Vec<PathBuf>) -> Self {
        theme::install_fonts(&cc.egui_ctx);
        let settings_path = Settings::config_path();
        let (mut settings, warning) = Settings::load(&settings_path);
        if let Ok(theme) = std::env::var("F2P_PREVIEW_THEME") {
            settings.theme = match theme.as_str() {
                "light" => ThemeChoice::Light,
                "dark" => ThemeChoice::Dark,
                _ => ThemeChoice::System,
            };
        }
        let mut state = AppState::new(settings, settings_path);
        if let Some(warning) = warning {
            state.notify(NoticeKind::Warning, warning);
        }
        let (sender, receiver) = mpsc::channel();
        let mut app = Self {
            state,
            sender,
            receiver,
            analysis_in_flight: None,
            batch_receiver: None,
            batch_control: None,
            batch_handle: None,
            active_control: None,
            gallery: std::env::var("F2P_GALLERY").is_ok(),
        };
        if !paths.is_empty() {
            app.add_paths(paths);
        }
        app.apply_preview();
        app
    }

    pub fn with_state(state: AppState) -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            state,
            sender,
            receiver,
            analysis_in_flight: None,
            batch_receiver: None,
            batch_control: None,
            batch_handle: None,
            active_control: None,
            gallery: false,
        }
    }

    fn apply_preview(&mut self) {
        let quiet = std::env::var("F2P_PREVIEW_QUIET").is_ok();
        if let Ok(dir) = std::env::var("F2P_PREVIEW_OUTPUT") {
            self.state.settings.output_dir = Some(PathBuf::from(dir));
        }
        if let Ok(screen) = std::env::var("F2P_PREVIEW_SCREEN") {
            if let Some(screen) = preview_screen(&screen) {
                self.state.set_screen(screen);
            }
        }
        if let Ok(path) = std::env::var("F2P_PREVIEW_REPORT") {
            if let Ok(text) = std::fs::read_to_string(&path) {
                if let Ok(report) = serde_json::from_str::<FileReport>(&text) {
                    let input = PathBuf::from(&report.source);
                    let output = report
                        .output
                        .clone()
                        .map(PathBuf::from)
                        .unwrap_or_else(|| input.with_extension("penpot"));
                    self.state.result = Some(ConversionOutcome {
                        input,
                        output,
                        report_path: PathBuf::from(path),
                        report,
                        validation_checks: 0,
                    });
                }
            }
        }
        if std::env::var("F2P_PREVIEW_ANALYZE").is_ok() {
            if let Some(path) = self.state.selected_item().map(|item| item.path.clone()) {
                if let Ok(document) = fig_reader::read_fig(&path) {
                    self.state
                        .analysis_finished(&path, AnalysisSummary::from_document(&document));
                }
            }
        }
        if let Ok(stage_name) = std::env::var("F2P_PREVIEW_STAGE") {
            if let Some(item) = self.state.selected_item() {
                let stage = preview_stage(&stage_name).unwrap_or(Stage::Converting);
                self.state.active = Some(ActiveJob {
                    input: item.path.clone(),
                    output: self.state.output_path_for(&item.path),
                    stage,
                    paused: false,
                });
                self.state.set_screen(Screen::Progress);
            }
        }
        if let Ok(dialog) = std::env::var("F2P_PREVIEW_DIALOG") {
            match dialog.as_str() {
                "licenses" => {
                    self.state.set_screen(Screen::Preferences);
                    self.state.show_licenses = true;
                }
                "reset" => {
                    self.state.set_screen(Screen::Preferences);
                    self.state.pending_reset = true;
                }
                "clear" => {
                    self.state.set_screen(Screen::Queue);
                    self.state.pending_clear = true;
                }
                _ => {}
            }
        }
        if std::env::var("F2P_PREVIEW_RUN_BATCH").is_ok() {
            self.state.settings.open_folder_after_conversion = false;
            self.start_batch();
        }
        if quiet {
            self.state.notice = None;
        }
    }

    fn poll_worker(&mut self, ctx: &egui::Context) {
        if self.poll_worker_events() {
            ctx.request_repaint();
        }
    }

    fn poll_worker_events(&mut self) -> bool {
        let mut changed = false;
        while let Ok(event) = self.receiver.try_recv() {
            changed = true;
            match event {
                WorkerEvent::Stage { input, stage } => self.state.job_stage(&input, stage),
                WorkerEvent::AnalysisFinished { path, summary } => {
                    if self.analysis_in_flight.as_ref() == Some(&path) {
                        self.analysis_in_flight = None;
                    }
                    self.state.analysis_finished(&path, summary);
                }
                WorkerEvent::AnalysisFailed { path, message } => {
                    if self.analysis_in_flight.as_ref() == Some(&path) {
                        self.analysis_in_flight = None;
                    }
                    self.state.analysis_failed(&path, message);
                }
                WorkerEvent::Finished(outcome) => {
                    self.active_control = None;
                    let open = self.state.settings.open_folder_after_conversion
                        && outcome.report.status != conversion_report::FileStatus::Failed;
                    let folder = outcome.output.parent().map(Path::to_path_buf);
                    self.state.finish_job(outcome);
                    if open {
                        if let Some(folder) = folder {
                            self.open_folder(&folder);
                        }
                    }
                }
                WorkerEvent::Cancelled { input, .. } => {
                    self.active_control = None;
                    self.state.cancel_job(&input);
                    self.state
                        .notify(NoticeKind::Info, copy::CANCELLED_NO_OUTPUT);
                }
                WorkerEvent::Failed(failure) => {
                    self.active_control = None;
                    let message = copy::failure_message(&failure.message);
                    self.state.fail_job(failure);
                    if self.state.screen == Screen::Progress {
                        self.state.set_screen(Screen::Queue);
                    }
                    self.state.notify(NoticeKind::Error, message);
                }
            }
        }
        changed
    }

    fn poll_batch(&mut self) {
        let Some(receiver) = self.batch_receiver.take() else {
            return;
        };
        let mut finished = false;
        while let Ok(event) = receiver.try_recv() {
            match event {
                QueueEvent::JobStarted { id } => self.state.batch_job_started(id),
                QueueEvent::Stage { id, stage } => self.state.batch_stage(id, stage),
                QueueEvent::JobFinished { result } => self.state.batch_result(*result),
                QueueEvent::BatchFinished => finished = true,
            }
        }
        if finished {
            let summary = self
                .state
                .batch
                .as_ref()
                .map(|batch| {
                    (
                        copy::batch_finished(batch.completed, batch.failed, batch.cancelled),
                        batch.failed > 0,
                    )
                })
                .unwrap_or_default();
            self.state.finish_batch();
            if !summary.0.is_empty() {
                self.state.notify(
                    if summary.1 {
                        NoticeKind::Warning
                    } else {
                        NoticeKind::Success
                    },
                    summary.0,
                );
            }
            self.batch_control = None;
            if let Some(handle) = self.batch_handle.take() {
                let _ = handle.join();
            }
        } else {
            self.batch_receiver = Some(receiver);
        }
    }

    fn maybe_start_analysis(&mut self) {
        if self.state.screen != Screen::Analysis || self.analysis_in_flight.is_some() {
            return;
        }
        if let Some(path) = self.state.analysis_needed() {
            self.begin_analysis(path);
        }
    }

    fn begin_analysis(&mut self, path: PathBuf) {
        self.state.mark_analyzing(&path);
        self.analysis_in_flight = Some(path.clone());
        spawn_analysis(path, self.sender.clone());
    }

    fn analyze_selected(&mut self, reset: bool) {
        if self.state.active.is_some() {
            self.state.notify(NoticeKind::Warning, copy::KEEP_WORKING);
            return;
        }
        let Some(path) = self.state.selected_item().map(|item| item.path.clone()) else {
            self.state.notify(NoticeKind::Warning, copy::FILE_ONLY_FIG);
            return;
        };
        if reset {
            self.state.reset_analysis(&path);
            self.state.set_screen(Screen::Analysis);
        }
        self.begin_analysis(path);
    }

    fn start_selected_conversion(&mut self) {
        if self.state.is_busy() {
            self.state.notify(NoticeKind::Warning, copy::KEEP_WORKING);
            return;
        }
        let Some(input) = self.state.selected_item().map(|item| item.path.clone()) else {
            self.state.notify(NoticeKind::Warning, copy::FILE_ONLY_FIG);
            return;
        };
        let output = self.state.output_path_for(&input);
        let report = self.state.report_path_for(&input);
        if !self.state.settings.overwrite {
            if output.exists() {
                self.state
                    .notify(NoticeKind::Error, copy::output_exists_message(&output));
                return;
            }
            if report.exists() {
                self.state.notify(NoticeKind::Error, copy::REPORT_EXISTS);
                return;
            }
        }
        let output_dir = self.state.settings.resolved_output_dir(&input);
        let control = ConversionControl::default();
        self.active_control = Some(control.clone());
        self.state.start_job(input.clone(), output);
        spawn_conversion(
            ConversionRequest {
                input,
                output_dir,
                force: self.state.settings.overwrite,
                keep_token_bindings: self.state.settings.keep_token_bindings,
                control,
            },
            self.sender.clone(),
        );
    }

    fn start_batch(&mut self) {
        let paths = self
            .state
            .files
            .iter()
            .map(|item| item.path.clone())
            .collect();
        self.start_batch_for(paths);
    }

    fn start_retry_failed(&mut self) {
        let paths = self.state.failed_paths();
        if paths.is_empty() {
            self.state
                .notify(NoticeKind::Info, copy::NO_FAILED_TO_RETRY);
            return;
        }
        self.start_batch_for(paths);
    }

    fn start_retry_selected(&mut self) {
        let Some(item) = self.state.selected_item() else {
            return;
        };
        if !matches!(item.state, QueueState::Failed(_)) {
            return;
        }
        let path = item.path.clone();
        self.start_batch_for(vec![path]);
    }

    fn start_batch_for(&mut self, paths: Vec<PathBuf>) {
        if self.state.is_busy() {
            self.state.notify(NoticeKind::Warning, copy::KEEP_WORKING);
            return;
        }
        if paths.is_empty() {
            self.state
                .notify(NoticeKind::Warning, copy::FILES_EMPTY_BODY);
            return;
        }
        for item in &mut self.state.files {
            item.job_id = None;
            item.batch_stage = None;
        }
        let specs: Vec<(PathBuf, PathBuf)> = paths
            .iter()
            .map(|path| (path.clone(), self.state.settings.resolved_output_dir(path)))
            .collect();
        let mut runner = BatchRunner::new(QueueConfig::bounded(2));
        for (path, output_dir) in specs {
            let id = runner.add_job(
                path.clone(),
                output_dir,
                self.state.settings.overwrite,
                self.state.settings.keep_token_bindings,
            );
            if let Some(item) = self.state.files.iter_mut().find(|item| item.path == path) {
                item.job_id = Some(id);
                item.state = QueueState::Queued;
                item.batch_stage = None;
                item.output = None;
                item.report_path = None;
            }
        }
        let control = runner.control();
        let (receiver, handle) = runner.spawn();
        let total = paths.len();
        self.state.start_batch(total);
        self.batch_receiver = Some(receiver);
        self.batch_control = Some(control);
        self.batch_handle = Some(handle);
    }

    fn cancel_batch(&mut self) {
        if let Some(control) = &self.batch_control {
            control.cancel();
            self.state
                .notify(NoticeKind::Warning, copy::CANCELLING_BATCH);
            return;
        }
        if let Some(control) = &self.active_control {
            control.cancel();
            self.state.notify(NoticeKind::Info, copy::CANCELLING_FILE);
        }
    }

    fn toggle_pause_batch(&mut self) {
        if let Some(control) = &self.batch_control {
            let paused = !control.is_paused();
            control.pause(paused);
            if let Some(batch) = self.state.batch.as_mut() {
                batch.paused = paused;
            }
            return;
        }
        if let Some(control) = &self.active_control {
            let paused = !control.is_paused();
            control.set_paused(paused);
            self.state.set_active_paused(paused);
        }
    }

    fn choose_files(&mut self) {
        let mut dialog = rfd::FileDialog::new()
            .add_filter(copy::FIGMA_FILTER, &["fig"])
            .set_title(copy::CHOOSE_FILES);
        if let Some(parent) = self
            .state
            .selected_item()
            .and_then(|item| item.path.parent())
        {
            dialog = dialog.set_directory(parent);
        }
        if let Some(paths) = dialog.pick_files() {
            self.add_paths(paths);
        }
    }

    fn choose_folder(&mut self) {
        let mut dialog = rfd::FileDialog::new().set_title(copy::CHOOSE_FOLDER);
        if let Some(parent) = self
            .state
            .selected_item()
            .and_then(|item| item.path.parent())
        {
            dialog = dialog.set_directory(parent);
        }
        if let Some(dir) = dialog.pick_folder() {
            match scan_folder(&dir, 6) {
                Ok(scan) if scan.files.is_empty() => {
                    self.state
                        .notify(NoticeKind::Warning, copy::COULDNT_SCAN_FOLDER);
                }
                Ok(scan) => {
                    let truncated = scan.truncated;
                    self.add_paths(scan.files);
                    if truncated {
                        self.state.notify(NoticeKind::Warning, copy::SCAN_TRUNCATED);
                    }
                }
                Err(error) => self.state.notify(
                    NoticeKind::Error,
                    format!("{}: {error}", copy::COULDNT_SCAN_FOLDER),
                ),
            }
        }
    }

    fn add_paths(&mut self, paths: Vec<PathBuf>) {
        let outcome = self.state.add_paths(paths);
        if outcome.added > 0 {
            self.state.set_screen(Screen::Queue);
        }
        if outcome.added > 0 && outcome.rejected.is_empty() {
            self.state
                .notify(NoticeKind::Success, copy::added_message(outcome.added));
        } else if !outcome.rejected.is_empty() {
            let (path, reason) = &outcome.rejected[0];
            let name = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string());
            self.state.notify(
                NoticeKind::Warning,
                format!("{} {name} — {reason}", copy::REJECTED),
            );
        }
    }

    fn choose_output_dir(&mut self) {
        let mut dialog = rfd::FileDialog::new().set_title(copy::WHERE_SAVE);
        if let Some(dir) = &self.state.settings.output_dir {
            dialog = dialog.set_directory(dir);
        }
        if let Some(dir) = dialog.pick_folder() {
            self.state.settings.output_dir = Some(dir);
            self.save_settings(false);
        }
    }

    fn save_settings(&mut self, show_success: bool) {
        match self.state.settings.save(&self.state.settings_path) {
            Ok(()) if show_success => {
                self.state.notify(NoticeKind::Success, copy::PREFS_SAVED);
            }
            Ok(()) => {}
            Err(error) => self.state.notify(
                NoticeKind::Error,
                format!("{}{error}", copy::SAVE_PREFS_FAILED),
            ),
        }
    }

    fn reset_settings(&mut self) {
        self.state.settings = Settings::default();
        self.save_settings(false);
        self.state.notify(NoticeKind::Info, copy::PREFS_RESET);
    }

    fn export_report(&mut self, format: ReportFormat) {
        let Some(result) = self.state.result.clone() else {
            self.state.notify(NoticeKind::Warning, copy::REPORT_EMPTY);
            return;
        };
        let stem = result
            .input
            .file_stem()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_else(|| "report".to_owned());
        let extension = match format {
            ReportFormat::Html => "html",
            ReportFormat::Csv => "csv",
            ReportFormat::Json => "json",
        };
        let mut dialog = rfd::FileDialog::new()
            .set_file_name(format!("{stem}.report.{extension}"))
            .add_filter(extension.to_ascii_uppercase(), &[extension]);
        if let Some(parent) = result.report_path.parent() {
            dialog = dialog.set_directory(parent);
        }
        let Some(path) = dialog.save_file() else {
            return;
        };
        let document = match format {
            ReportFormat::Html => result.report.to_html(),
            ReportFormat::Csv => result.report.to_csv(),
            ReportFormat::Json => match result.report.to_json_pretty() {
                Ok(json) => json,
                Err(error) => {
                    self.state.notify(
                        NoticeKind::Error,
                        format!("{}{error}", copy::REPORT_EXPORT_FAILED),
                    );
                    return;
                }
            },
        };
        match penpot_writer::write_atomic(&path, document.as_bytes()) {
            Ok(()) => self
                .state
                .notify(NoticeKind::Success, copy::REPORT_EXPORTED),
            Err(error) => self.state.notify(
                NoticeKind::Error,
                format!("{}{error}", copy::REPORT_EXPORT_FAILED),
            ),
        }
    }

    fn open_folder(&mut self, path: &Path) {
        let target = if path.is_file() {
            path.parent().unwrap_or(path)
        } else {
            path
        };
        if let Err(error) = open_path(target) {
            self.state.notify(
                NoticeKind::Error,
                format!("{}{error}", copy::OPEN_FOLDER_FAILED),
            );
        }
    }

    fn handle_drops(&mut self, ctx: &egui::Context) {
        let dropped: Vec<PathBuf> = ctx.input(|input| {
            input
                .raw
                .dropped_files
                .iter()
                .map(|file| file.path().to_path_buf())
                .collect()
        });
        if !dropped.is_empty() {
            self.add_paths(dropped);
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context, actions: &mut UiActions) {
        let mut screen = None;
        ctx.input(|input| {
            if !input.modifiers.command {
                return;
            }
            if input.modifiers.shift && input.key_pressed(egui::Key::O) {
                actions.add_folder = true;
            } else if input.key_pressed(egui::Key::O) {
                actions.add_files = true;
            }
            if input.modifiers.shift && input.key_pressed(egui::Key::A) {
                actions.analyze_selected = true;
            }
            if input.key_pressed(egui::Key::Enter) {
                actions.convert_selected = true;
            }
            if input.key_pressed(egui::Key::Comma) {
                screen = Some(Screen::Preferences);
            }
            let digits = [
                egui::Key::Num1,
                egui::Key::Num2,
                egui::Key::Num3,
                egui::Key::Num4,
                egui::Key::Num5,
                egui::Key::Num6,
                egui::Key::Num7,
                egui::Key::Num8,
            ];
            for (index, key) in digits.into_iter().enumerate() {
                if input.key_pressed(key) {
                    screen = Some(Screen::ALL[index]);
                }
            }
        });
        if let Some(screen) = screen {
            self.state.set_screen(screen);
        }
    }

    fn handle_actions(&mut self, actions: UiActions) {
        if actions.add_files {
            self.choose_files();
        }
        if actions.add_folder {
            self.choose_folder();
        }
        if actions.choose_output {
            self.choose_output_dir();
        }
        if actions.clear_output {
            self.state.settings.output_dir = None;
            self.save_settings(false);
        }
        if actions.analyze_selected {
            self.analyze_selected(true);
        }
        if actions.convert_selected {
            self.start_selected_conversion();
        }
        if actions.convert_all {
            self.start_batch();
        }
        if actions.cancel_batch {
            self.cancel_batch();
        }
        if actions.toggle_pause {
            self.toggle_pause_batch();
        }
        if actions.retry_failed {
            self.start_retry_failed();
        }
        if actions.retry_selected {
            self.start_retry_selected();
        }
        if actions.remove_completed {
            self.state.remove_completed();
        }
        if let Some(index) = actions.remove_file {
            self.state.remove(index);
        }
        if actions.export_html {
            self.export_report(ReportFormat::Html);
        }
        if actions.export_csv {
            self.export_report(ReportFormat::Csv);
        }
        if actions.export_json {
            self.export_report(ReportFormat::Json);
        }
        if actions.settings_changed {
            self.save_settings(false);
        }
        if actions.save_settings {
            self.save_settings(true);
        }
        if actions.reset_settings {
            self.reset_settings();
        }
        if let Some(path) = actions.open_folder {
            self.open_folder(&path);
        }
    }

    fn draw_nav(&mut self, root: &mut egui::Ui) {
        egui::Panel::left("nav")
            .resizable(false)
            .exact_size(216.0)
            .show(root, |ui| {
                let p = theme::palette(ui.ctx());
                ui.add_space(SPACE_4);
                ui.horizontal(|ui| {
                    components::icon_colored(ui, Icon::Sparkle, 22.0, p.accent);
                    ui.label(RichText::new(copy::APP_NAME).size(16.0).strong());
                });
                components::caption(ui, copy::NAV_SUBTITLE);
                ui.add_space(SPACE_6);
                components::caption(ui, copy::NAV_WORKFLOW);
                nav_item(
                    ui,
                    &mut self.state,
                    Screen::Home,
                    Icon::Upload,
                    copy::NAV_IMPORT,
                );
                nav_item(
                    ui,
                    &mut self.state,
                    Screen::Queue,
                    Icon::File,
                    copy::NAV_FILES,
                );
                nav_item(
                    ui,
                    &mut self.state,
                    Screen::Analysis,
                    Icon::Search,
                    copy::NAV_DETAILS,
                );
                nav_item(
                    ui,
                    &mut self.state,
                    Screen::Progress,
                    Icon::Clock,
                    copy::NAV_CONVERT,
                );
                nav_item(
                    ui,
                    &mut self.state,
                    Screen::Results,
                    Icon::Check,
                    copy::NAV_RESULTS,
                );
                nav_item(
                    ui,
                    &mut self.state,
                    Screen::Report,
                    Icon::File,
                    copy::NAV_REPORT,
                );
                ui.add_space(SPACE_4);
                components::caption(ui, copy::NAV_APP);
                nav_item(
                    ui,
                    &mut self.state,
                    Screen::Settings,
                    Icon::Settings,
                    copy::NAV_SETTINGS,
                );
                nav_item(
                    ui,
                    &mut self.state,
                    Screen::Preferences,
                    Icon::Settings,
                    copy::NAV_PREFERENCES,
                );
                ui.add_space(SPACE_6);
                components::caption(ui, copy::OFFLINE_SHORT);
                components::caption(ui, copy::PRIVACY_SHORT);
            });
    }

    fn draw_action_bar(&mut self, root: &mut egui::Ui, actions: &mut UiActions) {
        if self.state.screen != Screen::Queue || self.state.files.is_empty() {
            return;
        }
        egui::Panel::bottom("files_action_bar").show(root, |ui| {
            let p = theme::palette(ui.ctx());
            ui.add_space(SPACE_2);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("{} {}", self.state.files.len(), copy::FILES_COUNT))
                        .strong(),
                );
                ui.label(RichText::new("·").color(p.text_muted));
                let folder = self
                    .state
                    .settings
                    .output_dir
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| copy::NEXT_TO_FILE.to_owned());
                components::caption(ui, &format!("{}: {folder}", copy::SAVE_TO));
                if components::ghost_button(ui, copy::CHANGE).clicked() {
                    actions.choose_output = true;
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let busy = self.state.is_busy();
                    if ui
                        .add_enabled(
                            !busy,
                            egui::Button::new(RichText::new(copy::CONVERT).color(p.on_accent))
                                .fill(p.accent)
                                .stroke(egui::Stroke::NONE)
                                .corner_radius(egui::CornerRadius::same(theme::RADIUS_INPUT))
                                .min_size(Vec2::new(112.0, 38.0)),
                        )
                        .clicked()
                    {
                        actions.convert_all = true;
                    }
                });
            });
            ui.add_space(SPACE_2);
        });
    }

    fn draw_notice(&mut self, root: &mut egui::Ui) {
        let Some(notice) = self.state.notice.clone() else {
            return;
        };
        egui::Panel::top("notice").show(root, |ui| {
            let p = theme::palette(ui.ctx());
            let category = match notice.kind {
                NoticeKind::Success => copy::IssueCategory::AllGood,
                NoticeKind::Warning => copy::IssueCategory::Review,
                NoticeKind::Error => copy::IssueCategory::Attention,
                NoticeKind::Info => copy::IssueCategory::Review,
            };
            let color = theme::category_color(p, category);
            egui::Frame::default()
                .fill(p.surface_muted)
                .stroke(egui::Stroke::new(1.0, color))
                .corner_radius(egui::CornerRadius::same(theme::RADIUS_INPUT))
                .inner_margin(egui::Margin::symmetric(12, 7))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.horizontal(|ui| {
                        components::icon_colored(
                            ui,
                            components::category_icon(category),
                            15.0,
                            color,
                        );
                        ui.label(RichText::new(&notice.text).size(13.0));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.small_button(copy::DISMISS).clicked() {
                                self.state.clear_notice();
                            }
                        });
                    });
                });
        });
    }

    fn draw_root(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.handle_drops(&ctx);
        theme::apply(&ctx, self.state.settings.theme);
        let mut actions = UiActions::default();
        self.handle_shortcuts(&ctx, &mut actions);
        if self.gallery {
            egui::CentralPanel::default().show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        egui::Frame::default()
                            .inner_margin(egui::Margin::symmetric(32, 0))
                            .show(ui, |ui| {
                                ui.set_max_width(1040.0);
                                ui.add_space(SPACE_6);
                                views::gallery(ui);
                            });
                    });
            });
            return;
        }
        self.draw_nav(ui);
        self.draw_notice(ui);
        self.draw_action_bar(ui, &mut actions);
        egui::CentralPanel::default().show(ui, |ui| {
            let p = theme::palette(ui.ctx());
            ui.painter().rect_filled(ui.max_rect(), 0, p.bg);
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::Frame::default()
                        .inner_margin(egui::Margin::symmetric(32, 0))
                        .show(ui, |ui| {
                            views::draw(ui, &mut self.state, &mut actions);
                            ui.add_space(theme::SPACE_8);
                        });
                });
        });
        self.handle_actions(actions);
    }
}

fn nav_item(ui: &mut egui::Ui, state: &mut AppState, target: Screen, icon: Icon, label: &str) {
    let p = theme::palette(ui.ctx());
    let selected = state.screen == target;
    let response = egui::Frame::default()
        .fill(if selected {
            p.accent_soft
        } else {
            egui::Color32::TRANSPARENT
        })
        .corner_radius(egui::CornerRadius::same(theme::RADIUS_INPUT))
        .inner_margin(egui::Margin::symmetric(10, 7))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                components::icon_colored(
                    ui,
                    icon,
                    17.0,
                    if selected { p.accent } else { p.text_muted },
                );
                ui.label(RichText::new(label).size(14.0).color(if selected {
                    p.text
                } else {
                    p.text_muted
                }));
            });
        })
        .response;
    if response.interact(egui::Sense::click()).clicked() {
        state.set_screen(target);
    }
}

impl eframe::App for F2pApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.poll_worker(ctx);
        self.poll_batch();
        self.maybe_start_analysis();
        if self.state.active.is_some() || self.analysis_in_flight.is_some() || self.state.is_busy()
        {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        self.draw_root(ui);
    }
}

fn preview_screen(value: &str) -> Option<Screen> {
    match value {
        "home" | "import" => Some(Screen::Home),
        "files" | "queue" => Some(Screen::Queue),
        "details" | "analysis" => Some(Screen::Analysis),
        "settings" => Some(Screen::Settings),
        "converting" | "progress" => Some(Screen::Progress),
        "results" => Some(Screen::Results),
        "report" => Some(Screen::Report),
        "preferences" => Some(Screen::Preferences),
        _ => None,
    }
}

fn preview_stage(value: &str) -> Option<Stage> {
    match value {
        "reading" => Some(Stage::Reading),
        "converting" => Some(Stage::Converting),
        "writing" => Some(Stage::Writing),
        "validating" => Some(Stage::Validating),
        "saving" => Some(Stage::Saving),
        _ => None,
    }
}

#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
fn open_path(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    let mut command = std::process::Command::new("open");
    #[cfg(target_os = "windows")]
    let mut command = std::process::Command::new("explorer");
    #[cfg(all(target_os = "linux", not(target_os = "macos")))]
    let mut command = std::process::Command::new("xdg-open");
    command.arg(path).spawn().map(|_| ())
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn open_path(_path: &Path) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        copy::OPEN_FOLDER_UNSUPPORTED,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ConversionOutcome;
    use conversion_report::{Counts, FileStatus, ReportEvent, ValidationState};

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
            counts: Counts {
                detected: 2,
                converted: 1,
                approximated: 1,
                ..Counts::default()
            },
            events: vec![ReportEvent {
                code: "F2P-NODE-APPROXIMATED".to_owned(),
                severity: Severity::Warning,
                outcome: Outcome::ConvertedWithModification,
                element: conversion_report::ElementRef {
                    figma_node_id: "1:2".to_owned(),
                    name: "Card".to_owned(),
                    element_type: "FRAME".to_owned(),
                    page: Some("Page 1".to_owned()),
                    path: vec!["Page 1".to_owned(), "Card".to_owned()],
                },
                description: "raw".to_owned(),
                reason: None,
                action_taken: "kept".to_owned(),
                suggested_action: None,
            }],
            validation: ValidationState::Passed,
            tool_version: "test".to_owned(),
            target_penpot: "2.18.5".to_owned(),
        }
    }

    #[test]
    fn every_screen_draws_headlessly() {
        use eframe::egui::{Rect, Vec2};
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("a.fig");
        std::fs::write(&input, b"fig").unwrap();
        let mut state = state();
        state.add_paths(vec![input.clone()]);
        state.result = Some(ConversionOutcome {
            input: input.clone(),
            output: dir.path().join("a.penpot"),
            report_path: dir.path().join("a.report.json"),
            report: report(FileStatus::CompletedWithWarnings),
            validation_checks: 10,
        });
        let mut app = F2pApp::with_state(state);
        let ctx = egui::Context::default();
        for screen in Screen::ALL {
            app.state.screen = screen;
            let input = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(1180.0, 760.0),
                )),
                ..Default::default()
            };
            let mut output = ctx.run_ui(input, |ui| app.draw_root(ui));
            output.textures_delta.clear();
        }
    }

    #[test]
    fn command_number_jumps_to_a_screen_headlessly() {
        use eframe::egui::{Event, Key, Modifiers, Rect, Vec2};
        let mut app = F2pApp::with_state(state());
        let ctx = egui::Context::default();
        let mut input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                egui::Pos2::ZERO,
                Vec2::new(1180.0, 760.0),
            )),
            ..Default::default()
        };
        let modifiers = Modifiers {
            command: true,
            ..Modifiers::default()
        };
        input.events.push(Event::ModifiersChanged(modifiers));
        input.events.push(Event::Key {
            key: Key::Num2,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        });
        let mut output = ctx.run_ui(input, |ui| app.draw_root(ui));
        output.textures_delta.clear();
        assert_eq!(app.state.screen, Screen::Queue);
    }

    #[test]
    fn batch_queue_updates_files_and_results() {
        let dir = tempfile::tempdir().unwrap();
        let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures");
        let mut state = AppState::new(
            Settings {
                output_dir: Some(dir.path().to_path_buf()),
                open_folder_after_conversion: false,
                ..Settings::default()
            },
            dir.path().join("settings.json"),
        );
        state.add_paths(vec![
            fixture_dir.join("circle.fig"),
            fixture_dir.join("text-group.fig"),
        ]);
        let mut app = F2pApp::with_state(state);
        app.start_batch();
        for _ in 0..300 {
            app.poll_batch();
            if app.state.batch.as_ref().is_some_and(|batch| batch.done) {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let batch = app.state.batch.as_ref().expect("batch");
        assert!(batch.done);
        assert_eq!(batch.completed, 2);
        assert_eq!(app.state.screen, Screen::Results);
        assert!(dir.path().join("circle.penpot").exists());
    }

    use conversion_report::{Outcome, Severity};
}
