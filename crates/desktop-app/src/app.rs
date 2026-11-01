//! eframe shell and screens. All conversion work is delegated to
//! [`crate::conversion`]; this module never parses or writes design files.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use conversion_report::{FileStatus, Outcome, Severity};
use eframe::egui::{self, Align, Color32, Layout, RichText};
use eframe::{CreationContext, Frame};

use crate::conversion::{spawn_analysis, spawn_conversion, ConversionRequest, WorkerEvent};
use crate::model::{ActiveJob, AppState, NoticeKind, QueueState, Screen, Stage};
use crate::settings::{Settings, ThemeChoice};
use crate::strings::*;

#[derive(Default)]
struct UiActions {
    add_files: bool,
    choose_output: bool,
    clear_output: bool,
    analyze_selected: bool,
    convert_selected: bool,
    save_settings: bool,
    reset_settings: bool,
    settings_changed: bool,
    open_folder: Option<PathBuf>,
}

pub struct F2pApp {
    state: AppState,
    sender: Sender<WorkerEvent>,
    receiver: Receiver<WorkerEvent>,
    analysis_in_flight: Option<PathBuf>,
}

impl F2pApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        Self::new_with_paths(cc, Vec::new())
    }

    pub fn new_with_paths(_cc: &CreationContext<'_>, paths: Vec<PathBuf>) -> Self {
        let settings_path = Settings::config_path();
        let (settings, warning) = Settings::load(&settings_path);
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
        };
        if !paths.is_empty() {
            app.add_paths(paths);
        }
        app
    }

    /// Construct from an existing state. Used by tests and future embedding.
    pub fn with_state(state: AppState) -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            state,
            sender,
            receiver,
            analysis_in_flight: None,
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
                WorkerEvent::Stage { input, stage } => {
                    self.state.job_stage(&input, stage);
                }
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
                    let name = path
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_else(|| path.display().to_string());
                    self.state.analysis_failed(&path, message.clone());
                    self.state.notify(
                        NoticeKind::Error,
                        format!("{ANALYSIS_FAILED_PREFIX} {name}: {message}"),
                    );
                }
                WorkerEvent::Finished(outcome) => {
                    let open = self.state.settings.open_folder_after_conversion
                        && outcome.report.status != FileStatus::Failed;
                    let folder = outcome.output.parent().map(Path::to_path_buf);
                    self.state.finish_job(outcome);
                    if open {
                        if let Some(folder) = folder {
                            self.open_folder(&folder);
                        }
                    }
                }
                WorkerEvent::Failed(failure) => {
                    let message = failure.message.clone();
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
            self.state.notify(NoticeKind::Warning, BUSY);
            return;
        }
        let Some(path) = self.state.selected_item().map(|item| item.path.clone()) else {
            self.state.notify(NoticeKind::Warning, NO_FILE_SELECTED);
            return;
        };
        if reset {
            self.state.reset_analysis(&path);
            self.state.set_screen(Screen::Analysis);
        }
        self.begin_analysis(path);
    }

    fn start_selected_conversion(&mut self) {
        if self.state.active.is_some() {
            self.state.notify(NoticeKind::Warning, BUSY);
            return;
        }
        let Some(input) = self.state.selected_item().map(|item| item.path.clone()) else {
            self.state.notify(NoticeKind::Warning, NO_FILE_SELECTED);
            return;
        };
        let output = self.state.output_path_for(&input);
        let report = self.state.report_path_for(&input);
        if !self.state.settings.overwrite {
            if output.exists() {
                self.state.notify(
                    NoticeKind::Error,
                    format!(
                        "Output already exists: {}. Enable overwrite or choose another folder.",
                        output.display()
                    ),
                );
                return;
            }
            if report.exists() {
                self.state.notify(
                    NoticeKind::Error,
                    format!(
                        "Report already exists: {}. Enable overwrite or choose another folder.",
                        report.display()
                    ),
                );
                return;
            }
        }
        let output_dir = self.state.settings.resolved_output_dir(&input);
        self.state.start_job(input.clone(), output);
        spawn_conversion(
            ConversionRequest {
                input,
                output_dir,
                force: self.state.settings.overwrite,
            },
            self.sender.clone(),
        );
    }

    fn choose_files(&mut self) {
        let mut dialog = rfd::FileDialog::new()
            .add_filter("Figma", &["fig"])
            .set_title(CHOOSE_FILES);
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

    fn add_paths(&mut self, paths: Vec<PathBuf>) {
        let outcome = self.state.add_paths(paths);
        if outcome.added > 0 {
            self.state.set_screen(Screen::Queue);
        }
        if outcome.added > 0 && outcome.rejected.is_empty() {
            self.state
                .notify(NoticeKind::Success, added_message(outcome.added));
        } else if !outcome.rejected.is_empty() {
            let (path, reason) = &outcome.rejected[0];
            let name = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string());
            let extra = outcome.rejected.len().saturating_sub(1);
            let suffix = if extra > 0 {
                format!(" (+{extra} more)")
            } else {
                String::new()
            };
            self.state.notify(
                NoticeKind::Warning,
                format!("{REJECTED_PREFIX}: {name}{suffix} — {reason}"),
            );
        }
    }

    fn choose_output_dir(&mut self) {
        let mut dialog = rfd::FileDialog::new().set_title(OUTPUT_DIR);
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
            Ok(()) if show_success => self.state.notify(NoticeKind::Success, SETTINGS_SAVED),
            Ok(()) => {}
            Err(error) => self.state.notify(
                NoticeKind::Error,
                format!("{SAVE_SETTINGS_FAILED}: {error}"),
            ),
        }
    }

    fn reset_settings(&mut self) {
        self.state.settings = Settings::default();
        self.save_settings(false);
        self.state.notify(NoticeKind::Info, SETTINGS_SAVED);
    }

    fn open_folder(&mut self, path: &Path) {
        let target = if path.is_file() {
            path.parent().unwrap_or(path)
        } else {
            path
        };
        if let Err(error) = open_path(target) {
            self.state
                .notify(NoticeKind::Error, format!("{OPEN_FOLDER_FAILED}: {error}"));
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

    fn apply_theme(&self, ctx: &egui::Context) {
        ctx.set_theme(theme_preference(self.state.settings.theme));
    }

    fn handle_actions(&mut self, actions: UiActions) {
        if actions.add_files {
            self.choose_files();
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

    fn draw_header(&mut self, root: &mut egui::Ui, actions: &mut UiActions) {
        egui::Panel::top("app_header").show(root, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.heading(APP_TITLE);
                ui.label(RichText::new(APP_SUBTITLE).weak());
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button(ADD_FILES).clicked() {
                        actions.add_files = true;
                    }
                    if ui.ctx().input(|input| !input.raw.hovered_files.is_empty()) {
                        ui.label(RichText::new(DROP_ACTIVE).strong());
                    }
                });
            });
            if let Some(notice) = self.state.notice.clone() {
                ui.add_space(4.0);
                let color = match notice.kind {
                    NoticeKind::Info => ui.visuals().hyperlink_color,
                    NoticeKind::Success => ui.visuals().hyperlink_color,
                    NoticeKind::Warning => ui.visuals().warn_fg_color,
                    NoticeKind::Error => ui.visuals().error_fg_color,
                };
                let mut clear = false;
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(&notice.text).color(color));
                    if ui.small_button("×").clicked() {
                        clear = true;
                    }
                });
                if clear {
                    self.state.clear_notice();
                }
            }
            ui.add_space(6.0);
        });
    }

    fn draw_nav(&mut self, root: &mut egui::Ui) {
        egui::Panel::left("app_nav")
            .resizable(false)
            .exact_size(210.0)
            .show(root, |ui| {
                ui.add_space(12.0);
                for screen in Screen::ALL {
                    let selected = self.state.screen == screen;
                    if ui
                        .add_sized(
                            [ui.available_width(), 30.0],
                            egui::Button::new(screen_label(screen)).selected(selected),
                        )
                        .clicked()
                    {
                        self.state.set_screen(screen);
                    }
                }
                ui.add_space(16.0);
                ui.separator();
                ui.label(RichText::new("Local-first").small().weak());
                ui.label(
                    RichText::new("No telemetry · No network needed")
                        .small()
                        .weak(),
                );
            });
    }

    fn draw_footer(&mut self, root: &mut egui::Ui) {
        egui::Panel::bottom("app_status").show(root, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("{} file(s) staged", self.state.files.len()));
                if self.state.active.is_some() {
                    ui.label(RichText::new("conversion running").color(ui.visuals().warn_fg_color));
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(format!("v{}", env!("CARGO_PKG_VERSION")));
                    let output = self
                        .state
                        .settings
                        .output_dir
                        .as_ref()
                        .map(|dir| dir.display().to_string())
                        .unwrap_or_else(|| OUTPUT_SAME_AS_INPUT.to_owned());
                    let response = ui.label(middle_truncate(&output, 56));
                    if let Some(dir) = &self.state.settings.output_dir {
                        let _ = response.on_hover_text(dir.display().to_string());
                    }
                });
            });
        });
    }

    fn draw_central(&mut self, root: &mut egui::Ui, actions: &mut UiActions) {
        egui::CentralPanel::default().show(root, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_max_width(1100.0);
                    match self.state.screen {
                        Screen::Home => draw_home(ui, &mut self.state, actions),
                        Screen::Queue => draw_queue(ui, &mut self.state, actions),
                        Screen::Analysis => draw_analysis(ui, &mut self.state, actions),
                        Screen::Settings => draw_settings(ui, &mut self.state, actions),
                        Screen::Progress => draw_progress(ui, &self.state),
                        Screen::Results => draw_results(ui, &mut self.state, actions),
                        Screen::Report => draw_report(ui, &mut self.state),
                        Screen::Preferences => draw_preferences(ui, &mut self.state, actions),
                    }
                });
        });
    }
}

impl F2pApp {
    fn draw_root(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.handle_drops(&ctx);
        self.apply_theme(&ctx);
        let mut actions = UiActions::default();
        self.draw_header(ui, &mut actions);
        self.draw_nav(ui);
        self.draw_footer(ui);
        self.draw_central(ui, &mut actions);
        self.handle_actions(actions);
    }
}

impl eframe::App for F2pApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.poll_worker(ctx);
        self.maybe_start_analysis();
        if self.state.active.is_some() || self.analysis_in_flight.is_some() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        self.draw_root(ui);
    }
}

fn draw_home(ui: &mut egui::Ui, state: &mut AppState, actions: &mut UiActions) {
    ui.add_space(16.0);
    ui.heading(RichText::new(HOME_HEADING).size(28.0));
    ui.label(HOME_BODY);
    ui.add_space(16.0);

    let hovered = ui.ctx().input(|input| !input.raw.hovered_files.is_empty());
    let mut choose = false;
    egui::Frame::group(ui.style())
        .inner_margin(24.0)
        .show(ui, |ui| {
            ui.set_min_height(130.0);
            ui.vertical_centered(|ui| {
                ui.add_space(18.0);
                let hint = if hovered { DROP_ACTIVE } else { DROP_HINT };
                ui.label(RichText::new(hint).size(18.0));
                ui.add_space(10.0);
                choose = ui.button(CHOOSE_FILES).clicked();
                ui.add_space(6.0);
                ui.label(RichText::new("Files stay on this machine.").small().weak());
            });
        });
    if choose {
        actions.add_files = true;
    }

    if let Some(result) = state.result.clone() {
        ui.add_space(18.0);
        ui.heading(RECENT_RESULT);
        egui::Frame::group(ui.style()).show(ui, |ui| {
            status_line(ui, result.report.status);
            ui.label(middle_truncate(&result.input.display().to_string(), 90))
                .on_hover_text(result.input.display().to_string());
            ui.label(format!(
                "{} detected · {} converted",
                result.report.counts.detected, result.report.counts.converted
            ));
            ui.horizontal(|ui| {
                if ui.button(OPEN_RESULTS).clicked() {
                    state.set_screen(Screen::Results);
                }
                if ui.button(VIEW_REPORT).clicked() {
                    state.set_screen(Screen::Report);
                }
            });
        });
    }

    ui.add_space(18.0);
    ui.separator();
    ui.horizontal(|ui| {
        ui.label(RichText::new("Pure Rust").strong());
        ui.label("·");
        ui.label("Offline conversion");
        ui.label("·");
        ui.label("Event-based reports");
    });
}

fn draw_queue(ui: &mut egui::Ui, state: &mut AppState, actions: &mut UiActions) {
    ui.heading(QUEUE_HEADING);
    ui.label(QUEUE_BODY);
    ui.add_space(8.0);
    let busy = state.active.is_some();
    ui.horizontal_wrapped(|ui| {
        if ui
            .add_enabled(!busy, egui::Button::new(ADD_FILES))
            .clicked()
        {
            actions.add_files = true;
        }
        if ui
            .add_enabled(!state.files.is_empty(), egui::Button::new(CLEAR_QUEUE))
            .clicked()
        {
            state.clear_queue();
        }
        if ui
            .add_enabled(
                !busy && state.selected_item().is_some(),
                egui::Button::new(ANALYZE_FILE),
            )
            .clicked()
        {
            actions.analyze_selected = true;
        }
        if ui
            .add_enabled(
                !busy && state.selected_item().is_some(),
                egui::Button::new(CONVERT_FILE),
            )
            .clicked()
        {
            actions.convert_selected = true;
        }
    });
    ui.add_space(8.0);

    if state.files.is_empty() {
        ui.label(RichText::new(QUEUE_EMPTY).weak());
        return;
    }

    let items: Vec<(usize, String, String, u64, QueueState)> = state
        .files
        .iter()
        .enumerate()
        .map(|(index, item)| {
            (
                index,
                item.display_name(),
                item.path.display().to_string(),
                item.size,
                item.state.clone(),
            )
        })
        .collect();
    let mut select = None;
    let mut remove = None;
    for (index, name, full_path, size, queue_state) in items {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(state.selected == Some(index), name)
                    .on_hover_text(&full_path)
                    .clicked()
                {
                    select = Some(index);
                }
                ui.add_space(12.0);
                ui.label(format_bytes(size));
                ui.label(RichText::new(queue_state_label(&queue_state)).small());
                if ui.small_button(REMOVE_FILE).clicked() {
                    remove = Some(index);
                }
            });
            let _ = ui
                .label(
                    RichText::new(middle_truncate(&full_path, 96))
                        .small()
                        .weak(),
                )
                .on_hover_text(full_path);
        });
    }
    if let Some(index) = select {
        state.select(index);
    }
    if let Some(index) = remove {
        state.remove(index);
    }

    if let Some(item) = state.selected_item() {
        ui.add_space(10.0);
        let output = state.output_path_for(&item.path);
        ui.label(RichText::new("Selected output").strong());
        ui.label(middle_truncate(&output.display().to_string(), 100))
            .on_hover_text(output.display().to_string());
    }
}

fn draw_analysis(ui: &mut egui::Ui, state: &mut AppState, actions: &mut UiActions) {
    ui.heading(ANALYSIS_HEADING);
    ui.label(ANALYSIS_BODY);
    ui.add_space(8.0);
    let Some(item) = state.selected_item().cloned() else {
        ui.label(RichText::new(ANALYSIS_EMPTY).weak());
        return;
    };
    let _ = ui
        .label(RichText::new(item.display_name()).strong())
        .on_hover_text(item.path.display().to_string());
    let _ = ui
        .label(middle_truncate(&item.path.display().to_string(), 100))
        .on_hover_text(item.path.display().to_string());
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if item.analysis.is_some() {
            if ui.button("Analyze again").clicked() {
                actions.analyze_selected = true;
            }
        } else if item.state == QueueState::Analyzing {
            ui.spinner();
            ui.label(ANALYSIS_PENDING);
        } else if ui.button(ANALYZE_NOW).clicked() {
            actions.analyze_selected = true;
        }
        if ui.button(CONVERT_FILE).clicked() {
            actions.convert_selected = true;
        }
    });

    if let Some(error) = &item.analysis_error {
        ui.add_space(8.0);
        ui.colored_label(
            ui.visuals().error_fg_color,
            format!("{ANALYSIS_FAILED}: {error}"),
        );
    }
    if let Some(summary) = &item.analysis {
        ui.add_space(12.0);
        egui::Grid::new("analysis_grid")
            .num_columns(2)
            .spacing([28.0, 8.0])
            .show(ui, |ui| {
                analysis_row(
                    ui,
                    DOCUMENT_NAME,
                    summary.document_name.as_deref().unwrap_or("Unknown"),
                );
                analysis_row(ui, CONTAINER_LABEL, &summary.container);
                analysis_row(
                    ui,
                    FORMAT_LABEL,
                    &format!("{} v{}", summary.format, summary.version),
                );
                analysis_row(ui, PAGES_LABEL, &summary.pages.to_string());
                analysis_row(
                    ui,
                    INTERNAL_PAGES_LABEL,
                    &summary.internal_pages.to_string(),
                );
                analysis_row(ui, NODES_LABEL, &summary.nodes.to_string());
                analysis_row(
                    ui,
                    IMAGES_LABEL,
                    &format!("{} ({})", summary.images, format_bytes(summary.image_bytes)),
                );
                analysis_row(ui, WARNINGS_LABEL, &summary.warnings.to_string());
                analysis_row(ui, DEPTH_LABEL, &summary.max_depth.to_string());
            });
    }
}

fn analysis_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.label(RichText::new(label).strong());
    ui.label(value);
    ui.end_row();
}

fn draw_settings(ui: &mut egui::Ui, state: &mut AppState, actions: &mut UiActions) {
    ui.heading(SETTINGS_HEADING);
    ui.label(SETTINGS_BODY);
    ui.add_space(12.0);

    ui.label(RichText::new(OUTPUT_DIR).strong());
    let output = state
        .settings
        .output_dir
        .as_ref()
        .map(|dir| dir.display().to_string())
        .unwrap_or_else(|| OUTPUT_SAME_AS_INPUT.to_owned());
    ui.label(middle_truncate(&output, 96));
    ui.horizontal(|ui| {
        if ui.button(OUTPUT_CHOOSE).clicked() {
            actions.choose_output = true;
        }
        if state.settings.output_dir.is_some() && ui.button(OUTPUT_CLEAR).clicked() {
            actions.clear_output = true;
        }
    });
    ui.add_space(10.0);

    if ui
        .checkbox(&mut state.settings.overwrite, OVERWRITE)
        .changed()
    {
        actions.settings_changed = true;
    }
    if ui
        .checkbox(&mut state.settings.open_folder_after_conversion, OPEN_AFTER)
        .changed()
    {
        actions.settings_changed = true;
    }
    ui.add_space(12.0);
    if ui.button(SAVE_SETTINGS).clicked() {
        actions.save_settings = true;
    }

    if let Some(item) = state.selected_item() {
        ui.add_space(14.0);
        ui.separator();
        ui.label(RichText::new("Preview for selected file").strong());
        let output = state.output_path_for(&item.path);
        let report = state.report_path_for(&item.path);
        ui.label(middle_truncate(&output.display().to_string(), 100))
            .on_hover_text(output.display().to_string());
        ui.label(middle_truncate(&report.display().to_string(), 100));
    }
}

fn draw_progress(ui: &mut egui::Ui, state: &AppState) {
    ui.heading(PROGRESS_HEADING);
    ui.label(PROGRESS_BODY);
    ui.add_space(12.0);
    let Some(active) = state.active.as_ref() else {
        ui.label(RichText::new(NO_ACTIVE_JOB).weak());
        return;
    };
    progress_job(ui, active);
}

fn progress_job(ui: &mut egui::Ui, active: &ActiveJob) {
    ui.label(RichText::new(active.input.display().to_string()).strong());
    ui.label(middle_truncate(&active.output.display().to_string(), 100));
    ui.add_space(12.0);
    let current = active.stage.index();
    for (index, stage) in Stage::ALL.iter().enumerate() {
        ui.horizontal(|ui| {
            if index < current {
                ui.label(RichText::new("✓").color(ui.visuals().hyperlink_color));
            } else if index == current {
                ui.spinner();
            } else {
                ui.label(RichText::new("·").weak());
            }
            let text = RichText::new(stage_label(*stage));
            ui.label(if index <= current {
                text.strong()
            } else {
                text.weak()
            });
        });
    }
}

fn draw_results(ui: &mut egui::Ui, state: &mut AppState, actions: &mut UiActions) {
    ui.heading(RESULTS_HEADING);
    let Some(result) = state.result.clone() else {
        ui.label(RichText::new(RESULTS_EMPTY).weak());
        return;
    };
    ui.add_space(8.0);
    status_line(ui, result.report.status);
    ui.add_space(6.0);
    let _ = ui
        .label(middle_truncate(&result.input.display().to_string(), 100))
        .on_hover_text(result.input.display().to_string());
    ui.add_space(10.0);
    counts_grid(ui, &result.report);
    ui.add_space(8.0);
    egui::Grid::new("result_paths")
        .num_columns(2)
        .spacing([18.0, 6.0])
        .show(ui, |ui| {
            ui.label(RichText::new(OUTPUT_LABEL).strong());
            let response = ui.label(middle_truncate(&result.output.display().to_string(), 82));
            let _ = response.on_hover_text(result.output.display().to_string());
            ui.end_row();
            ui.label(RichText::new(REPORT_LABEL).strong());
            let response = ui.label(middle_truncate(
                &result.report_path.display().to_string(),
                82,
            ));
            let _ = response.on_hover_text(result.report_path.display().to_string());
            ui.end_row();
            ui.label(RichText::new(VALIDATION_LABEL).strong());
            ui.label(format!(
                "{} ({} checks)",
                validation_label(result.report.validation),
                result.validation_checks
            ));
            ui.end_row();
            ui.label(RichText::new(DURATION_LABEL).strong());
            ui.label(format!("{} ms", result.report.duration_ms));
            ui.end_row();
        });
    ui.add_space(12.0);
    ui.horizontal_wrapped(|ui| {
        if ui.button(OPEN_FOLDER).clicked() {
            actions.open_folder = Some(
                result
                    .output
                    .parent()
                    .unwrap_or(&result.output)
                    .to_path_buf(),
            );
        }
        if ui.button(VIEW_REPORT).clicked() {
            state.set_screen(Screen::Report);
        }
        if ui.button(START_OVER).clicked() {
            state.set_screen(Screen::Home);
        }
    });
}

fn draw_report(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading(REPORT_HEADING);
    let Some(result) = state.result.clone() else {
        ui.label(RichText::new(REPORT_EMPTY).weak());
        return;
    };
    ui.add_space(6.0);
    status_line(ui, result.report.status);
    let _ = ui
        .label(middle_truncate(&result.input.display().to_string(), 100))
        .on_hover_text(result.input.display().to_string());
    counts_grid(ui, &result.report);
    ui.add_space(10.0);

    ui.horizontal_wrapped(|ui| {
        ui.label(REPORT_SEARCH);
        ui.add(
            egui::TextEdit::singleline(&mut state.filter.search)
                .hint_text(REPORT_SEARCH)
                .desired_width(250.0),
        );
        ui.separator();
        ui.label(REPORT_SEVERITY);
        egui::ComboBox::from_id_salt("severity_filter")
            .selected_text(
                state
                    .filter
                    .severity
                    .map(severity_label)
                    .unwrap_or(REPORT_ALL),
            )
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.filter.severity, None, REPORT_ALL);
                ui.selectable_value(
                    &mut state.filter.severity,
                    Some(Severity::Info),
                    severity_label(Severity::Info),
                );
                ui.selectable_value(
                    &mut state.filter.severity,
                    Some(Severity::Warning),
                    severity_label(Severity::Warning),
                );
                ui.selectable_value(
                    &mut state.filter.severity,
                    Some(Severity::Error),
                    severity_label(Severity::Error),
                );
            });
        ui.separator();
        ui.label(REPORT_OUTCOME);
        egui::ComboBox::from_id_salt("outcome_filter")
            .selected_text(
                state
                    .filter
                    .outcome
                    .map(outcome_label)
                    .unwrap_or(REPORT_ALL),
            )
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.filter.outcome, None, REPORT_ALL);
                for outcome in [
                    Outcome::Converted,
                    Outcome::ConvertedWithModification,
                    Outcome::Skipped,
                    Outcome::Unsupported,
                    Outcome::Failed,
                ] {
                    ui.selectable_value(
                        &mut state.filter.outcome,
                        Some(outcome),
                        outcome_label(outcome),
                    );
                }
            });
    });

    let events: Vec<&conversion_report::ReportEvent> = result
        .report
        .events
        .iter()
        .filter(|event| state.filter.matches(event))
        .collect();
    ui.add_space(6.0);
    ui.label(format!("{} {}", events.len(), REPORT_EVENTS));
    if events.is_empty() {
        ui.label(RichText::new(REPORT_NO_MATCHES).weak());
        return;
    }
    ui.separator();
    ui.label(RichText::new(REPORT_TRACEABLE).small().weak());
    egui::ScrollArea::vertical()
        .max_height(430.0)
        .show(ui, |ui| {
            for (index, event) in events.iter().enumerate() {
                ui.push_id(index, |ui| {
                    let color = severity_color(ui, event.severity);
                    let title = format!(
                        "{} · {} · {}",
                        severity_label(event.severity),
                        event.code,
                        event.element.name
                    );
                    egui::CollapsingHeader::new(RichText::new(title).color(color))
                        .default_open(event.severity == Severity::Error)
                        .show(ui, |ui| {
                            ui.label(&event.description);
                            ui.label(format!(
                                "{}: {}",
                                REPORT_ELEMENT,
                                event.element.path.join(" / ")
                            ));
                            if let Some(page) = &event.element.page {
                                ui.label(format!("{REPORT_PAGE}: {page}"));
                            }
                            ui.label(format!(
                                "{}: {}",
                                REPORT_OUTCOME,
                                outcome_label(event.outcome)
                            ));
                            if let Some(reason) = &event.reason {
                                ui.label(format!("{REPORT_REASON}: {reason}"));
                            }
                            ui.label(format!("{REPORT_ACTION}: {}", event.action_taken));
                            if let Some(suggested) = &event.suggested_action {
                                ui.label(format!("{REPORT_SUGGESTED}: {suggested}"));
                            }
                        });
                });
            }
        });
}

fn draw_preferences(ui: &mut egui::Ui, state: &mut AppState, actions: &mut UiActions) {
    ui.heading(PREFERENCES_HEADING);
    ui.label(PREFERENCES_BODY);
    ui.add_space(10.0);
    ui.label(RichText::new(THEME_LABEL).strong());
    let before = state.settings.theme;
    ui.horizontal(|ui| {
        ui.radio_value(&mut state.settings.theme, ThemeChoice::System, THEME_SYSTEM);
        ui.radio_value(&mut state.settings.theme, ThemeChoice::Light, THEME_LIGHT);
        ui.radio_value(&mut state.settings.theme, ThemeChoice::Dark, THEME_DARK);
    });
    if state.settings.theme != before {
        ui.ctx().set_theme(theme_preference(state.settings.theme));
        actions.settings_changed = true;
    }
    ui.add_space(12.0);
    if ui.button(RESET_PREFERENCES).clicked() {
        actions.reset_settings = true;
    }
    ui.add_space(14.0);
    ui.separator();
    ui.label(RichText::new(SETTINGS_FILE).strong());
    ui.label(middle_truncate(
        &state.settings_path.display().to_string(),
        96,
    ))
    .on_hover_text(state.settings_path.display().to_string());
    ui.add_space(10.0);
    ui.label(RichText::new(ABOUT_LABEL).strong());
    ui.label(ABOUT_BODY);
    ui.label(format!("{}: {}", VERSION_LABEL, env!("CARGO_PKG_VERSION")));
    ui.label(format!("App ID: {APP_ID}"));
}

fn status_line(ui: &mut egui::Ui, status: FileStatus) {
    let color = match status {
        FileStatus::Completed => ui.visuals().hyperlink_color,
        FileStatus::CompletedWithWarnings => ui.visuals().warn_fg_color,
        FileStatus::Failed => ui.visuals().error_fg_color,
        FileStatus::Cancelled => ui.visuals().weak_text_color(),
    };
    ui.label(
        RichText::new(file_status_label(status))
            .strong()
            .color(color),
    );
}

fn counts_grid(ui: &mut egui::Ui, report: &conversion_report::FileReport) {
    egui::Grid::new("counts_grid")
        .num_columns(6)
        .spacing([22.0, 5.0])
        .show(ui, |ui| {
            count_cell(ui, DETECTED_LABEL, report.counts.detected);
            count_cell(ui, CONVERTED_LABEL, report.counts.converted);
            count_cell(ui, APPROXIMATED_LABEL, report.counts.approximated);
            ui.end_row();
            count_cell(ui, SKIPPED_LABEL, report.counts.skipped);
            count_cell(ui, UNSUPPORTED_LABEL, report.counts.unsupported);
            count_cell(ui, FAILED_LABEL, report.counts.failed);
            ui.end_row();
        });
}

fn count_cell(ui: &mut egui::Ui, label: &str, value: u64) {
    ui.label(RichText::new(label).small().weak());
    ui.label(RichText::new(value.to_string()).strong());
}

fn severity_color(ui: &egui::Ui, severity: Severity) -> Color32 {
    match severity {
        Severity::Info => ui.visuals().text_color(),
        Severity::Warning => ui.visuals().warn_fg_color,
        Severity::Error => ui.visuals().error_fg_color,
    }
}

fn queue_state_label(state: &QueueState) -> String {
    match state {
        QueueState::Ready => "— Ready".to_owned(),
        QueueState::Analyzing => "… Analyzing".to_owned(),
        QueueState::Analyzed => "✓ Analyzed".to_owned(),
        QueueState::Converted => "✓ Converted".to_owned(),
        QueueState::Failed(message) => format!("✕ {message}"),
    }
}

fn theme_preference(theme: ThemeChoice) -> egui::ThemePreference {
    match theme {
        ThemeChoice::System => egui::ThemePreference::System,
        ThemeChoice::Light => egui::ThemePreference::Light,
        ThemeChoice::Dark => egui::ThemePreference::Dark,
    }
}

pub fn middle_truncate(text: &str, max_chars: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max_chars || max_chars < 4 {
        return text.to_owned();
    }
    let keep = max_chars.saturating_sub(1);
    let front = keep / 2;
    let back = keep - front;
    let mut out = String::new();
    out.extend(&chars[..front]);
    out.push('…');
    out.extend(&chars[chars.len() - back..]);
    out
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KiB", "MiB", "GiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
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
        "opening folders is not supported on this platform",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ConversionOutcome;

    #[test]
    fn truncates_long_paths_in_the_middle() {
        let short = middle_truncate("short", 20);
        assert_eq!(short, "short");
        let long = middle_truncate("abcdefghijklmnopqrstuvwxyz", 12);
        assert!(long.contains('…'));
        assert!(long.chars().count() <= 12);
        assert!(long.starts_with("abcde"));
        assert!(long.ends_with("xyz"));
    }

    #[test]
    fn formats_bytes_without_fabricated_precision() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1.0 KiB");
        assert_eq!(format_bytes(1536), "1.5 KiB");
    }

    #[test]
    fn every_screen_draws_headlessly() {
        use eframe::egui::{Rect, Vec2};
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("a.fig");
        std::fs::write(&input, b"fig").unwrap();
        let mut state = AppState::new(Settings::default(), dir.path().join("settings.json"));
        state.add_paths(vec![input.clone()]);
        state.result = Some(ConversionOutcome {
            input: input.clone(),
            output: dir.path().join("a.penpot"),
            report_path: dir.path().join("a.report.json"),
            report: conversion_report::FileReport {
                schema_version: 1,
                source: input.display().to_string(),
                output: Some("a.penpot".to_owned()),
                status: FileStatus::CompletedWithWarnings,
                duration_ms: 3,
                counts: conversion_report::Counts {
                    detected: 2,
                    converted: 1,
                    approximated: 1,
                    ..Default::default()
                },
                events: vec![conversion_report::ReportEvent {
                    code: "F2P-TEST".to_owned(),
                    severity: Severity::Warning,
                    outcome: Outcome::ConvertedWithModification,
                    element: conversion_report::ElementRef {
                        figma_node_id: "1:2".to_owned(),
                        name: "Rect".to_owned(),
                        element_type: "RECTANGLE".to_owned(),
                        page: Some("Page 1".to_owned()),
                        path: vec!["Page 1".to_owned(), "Rect".to_owned()],
                    },
                    description: "test event".to_owned(),
                    reason: Some("unit test".to_owned()),
                    action_taken: "kept the shape".to_owned(),
                    suggested_action: None,
                }],
                validation: conversion_report::ValidationState::Passed,
                tool_version: "test".to_owned(),
                target_penpot: "2.18.5".to_owned(),
            },
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
    fn worker_result_reaches_results_and_report_screens() {
        use crate::conversion::{run_conversion, ConversionRequest};
        use eframe::egui::{Rect, Vec2};
        let dir = tempfile::tempdir().unwrap();
        let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/circle.fig");
        let mut state = AppState::new(
            Settings {
                output_dir: Some(dir.path().to_path_buf()),
                open_folder_after_conversion: false,
                ..Settings::default()
            },
            dir.path().join("settings.json"),
        );
        state.add_paths(vec![input.clone()]);
        let output = state.output_path_for(&input);
        state.start_job(input.clone(), output.clone());
        let mut app = F2pApp::with_state(state);
        run_conversion(
            ConversionRequest {
                input,
                output_dir: dir.path().to_path_buf(),
                force: false,
            },
            &app.sender,
        );
        assert!(app.poll_worker_events());
        assert_eq!(app.state.screen, Screen::Results);
        let result = app.state.result.as_ref().expect("result");
        assert!(result.output.exists());
        assert!(result.report_path.exists());
        assert!(result.validation_checks > 0);

        let ctx = egui::Context::default();
        for screen in [Screen::Results, Screen::Report] {
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
    fn stage_index_is_ordered() {
        assert_eq!(Stage::Reading.index(), 0);
        assert_eq!(Stage::Saving.index(), 4);
        assert!(Stage::ALL
            .windows(2)
            .all(|pair| pair[0].index() < pair[1].index()));
    }
}
