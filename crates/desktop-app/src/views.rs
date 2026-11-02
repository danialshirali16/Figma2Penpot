//! Designer-facing screens. Pure presentation over `AppState`; work is dispatched as
//! `UiActions` and executed by the app shell.

use std::path::PathBuf;

use conversion_queue::{JobResult, JobState, Stage as QueueStage};
use conversion_report::{FileStatus, Outcome, Severity};
use eframe::egui::{self, Align, Layout, RichText, Vec2};

use crate::components::{self, Icon};
use crate::copy;
use crate::model::{
    ActiveJob, AppState, ConversionOutcome, QueueState, ReportGrouping, Screen, Stage,
};
use crate::settings::ThemeChoice;
use crate::theme::{self, SPACE_1, SPACE_2, SPACE_3, SPACE_4, SPACE_6};

#[derive(Default)]
pub struct UiActions {
    pub add_files: bool,
    pub add_folder: bool,
    pub choose_output: bool,
    pub clear_output: bool,
    pub analyze_selected: bool,
    pub convert_selected: bool,
    pub convert_all: bool,
    pub cancel_batch: bool,
    pub toggle_pause: bool,
    pub retry_failed: bool,
    pub retry_selected: bool,
    pub remove_completed: bool,
    pub export_html: bool,
    pub export_csv: bool,
    pub export_json: bool,
    pub settings_changed: bool,
    pub save_settings: bool,
    pub reset_settings: bool,
    pub remove_file: Option<usize>,
    pub open_folder: Option<PathBuf>,
}

pub fn draw(ui: &mut egui::Ui, state: &mut AppState, actions: &mut UiActions) {
    let content_width = ui.available_width().min(1040.0);
    ui.set_width(content_width);
    ui.add_space(SPACE_4);
    match state.screen {
        Screen::Home => home(ui, state, actions),
        Screen::Queue => files(ui, state, actions),
        Screen::Analysis => details(ui, state, actions),
        Screen::Settings => settings(ui, state, actions),
        Screen::Progress => converting(ui, state, actions),
        Screen::Results => results(ui, state, actions),
        Screen::Report => report(ui, state, actions),
        Screen::Preferences => preferences(ui, state, actions),
    }
}

fn screen_header(ui: &mut egui::Ui, title: &str, subtitle: &str) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(title).size(28.0).strong());
            if !subtitle.is_empty() {
                components::body_muted(ui, subtitle);
            }
        });
    });
    ui.add_space(SPACE_4);
}

pub fn home(ui: &mut egui::Ui, state: &mut AppState, actions: &mut UiActions) {
    let compact = ui.available_width() < 900.0;
    ui.add_space(if compact { SPACE_4 } else { SPACE_6 });
    ui.label(
        RichText::new(copy::HOME_TITLE)
            .size(if compact { 34.0 } else { 40.0 })
            .strong(),
    );
    ui.add_space(SPACE_2);
    ui.label(
        RichText::new(copy::HOME_BODY)
            .size(16.0)
            .color(theme::palette(ui.ctx()).text_muted),
    );
    ui.add_space(if compact { SPACE_4 } else { SPACE_6 });

    let p = theme::palette(ui.ctx());
    let hovering = ui.ctx().input(|input| !input.raw.hovered_files.is_empty());
    let border = if hovering { p.accent } else { p.border };
    let fill = if hovering { p.accent_soft } else { p.surface };
    egui::Frame::default()
        .fill(fill)
        .stroke(egui::Stroke::new(1.0, border))
        .corner_radius(egui::CornerRadius::same(theme::RADIUS_LARGE))
        .inner_margin(egui::Margin::same(28))
        .show(ui, |ui| {
            ui.set_height(if compact { 170.0 } else { 220.0 });
            ui.vertical_centered(|ui| {
                ui.add_space(SPACE_4);
                components::icon_colored(ui, Icon::Upload, 38.0, p.accent);
                ui.add_space(SPACE_3);
                ui.label(
                    RichText::new(if hovering {
                        copy::DROP_RELEASE
                    } else {
                        copy::DROP_FILES
                    })
                    .size(20.0)
                    .strong(),
                );
                ui.add_space(SPACE_3);
                ui.horizontal(|ui| {
                    let width = 330.0;
                    ui.add_space(((ui.available_width() - width) / 2.0).max(0.0));
                    if components::primary_button(ui, copy::CHOOSE_FILES).clicked() {
                        actions.add_files = true;
                    }
                    if components::secondary_button(ui, copy::CHOOSE_FOLDER).clicked() {
                        actions.add_folder = true;
                    }
                });
                ui.add_space(SPACE_2);
                components::caption(ui, copy::PRIVACY_LONG);
            });
        });

    ui.add_space(if compact { SPACE_4 } else { SPACE_6 });
    ui.columns(3, |columns| {
        for (column, (number, title, body)) in columns.iter_mut().zip(copy::HOME_STEPS) {
            egui::Frame::default()
                .fill(p.surface)
                .stroke(egui::Stroke::new(1.0, p.border))
                .corner_radius(egui::CornerRadius::same(theme::RADIUS_CARD))
                .inner_margin(egui::Margin::same(16))
                .show(column, |ui| {
                    ui.horizontal(|ui| {
                        egui::Frame::default()
                            .fill(p.accent_soft)
                            .corner_radius(egui::CornerRadius::same(theme::RADIUS_PILL))
                            .inner_margin(egui::Margin::symmetric(9, 3))
                            .show(ui, |ui| {
                                ui.label(RichText::new(number).size(12.0).strong().color(p.accent));
                            });
                        ui.label(RichText::new(title).size(15.0).strong());
                    });
                    ui.add_space(SPACE_2);
                    components::caption(ui, body);
                });
        }
    });

    let folder = state
        .result
        .as_ref()
        .and_then(|result| result.output.parent().map(|path| path.to_path_buf()))
        .or_else(|| state.settings.output_dir.clone());
    if let (Some(result), Some(folder)) = (state.result.clone(), folder) {
        ui.add_space(SPACE_6);
        egui::Frame::default()
            .fill(p.surface)
            .stroke(egui::Stroke::new(1.0, p.border))
            .corner_radius(egui::CornerRadius::same(theme::RADIUS_CARD))
            .inner_margin(egui::Margin::same(16))
            .show(ui, |ui| {
                components::caption(ui, copy::LAST_SAVED_TO);
                ui.horizontal(|ui| {
                    components::icon(ui, Icon::Folder, 16.0);
                    ui.label(components::middle_truncate(
                        &folder.display().to_string(),
                        80,
                    ))
                    .on_hover_text(folder.display().to_string());
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if components::secondary_button(ui, copy::OPEN_FOLDER).clicked() {
                            actions.open_folder = Some(folder.clone());
                        }
                    });
                });
                if result.report.status == FileStatus::CompletedWithWarnings {
                    ui.label(
                        RichText::new(copy::NEEDS_REVIEW)
                            .color(p.warning)
                            .size(12.0),
                    );
                }
            });
    }
}

pub fn files(ui: &mut egui::Ui, state: &mut AppState, actions: &mut UiActions) {
    screen_header(ui, copy::FILES_TITLE, copy::FILES_SUBTITLE);
    if state.files.is_empty() {
        components::empty_state(
            ui,
            Icon::File,
            copy::FILES_EMPTY_TITLE,
            copy::FILES_EMPTY_BODY,
        );
        ui.vertical_centered(|ui| {
            if components::primary_button(ui, copy::CHOOSE_FILES).clicked() {
                actions.add_files = true;
            }
            if components::secondary_button(ui, copy::CHOOSE_FOLDER).clicked() {
                actions.add_folder = true;
            }
        });
        return;
    }
    let busy = state.is_busy();
    ui.horizontal_wrapped(|ui| {
        if components::secondary_button(ui, copy::ADD_FILES).clicked() {
            actions.add_files = true;
        }
        if components::ghost_button(ui, copy::ADD_FOLDER).clicked() {
            actions.add_folder = true;
        }
        if !state.files.is_empty() {
            if components::ghost_button(ui, copy::CLEAR_ALL).clicked() {
                state.pending_clear = true;
            }
            if state
                .files
                .iter()
                .any(|item| item.state == QueueState::Converted)
                && components::ghost_button(ui, copy::REMOVE_COMPLETED).clicked()
            {
                actions.remove_completed = true;
            }
        }
        if let Some(item) = state.selected_item() {
            if matches!(item.state, QueueState::Failed(_))
                && components::ghost_button(ui, copy::RETRY_SELECTED).clicked()
            {
                actions.retry_selected = true;
            }
        }
    });
    ui.add_space(SPACE_4);

    struct FileListRow {
        index: usize,
        name: String,
        full_path: String,
        size: u64,
        state: QueueState,
        pages: Option<usize>,
        analysis_error: Option<String>,
    }
    let items: Vec<FileListRow> = state
        .files
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let pages = item.analysis.as_ref().map(|analysis| analysis.pages);
            FileListRow {
                index,
                name: item.display_name(),
                full_path: item.path.display().to_string(),
                size: item.size,
                state: item.state.clone(),
                pages,
                analysis_error: item.analysis_error.clone(),
            }
        })
        .collect();
    let mut select = None;
    let mut remove = None;
    for item in items {
        let (label, reason) = copy::queue_state(&item.state);
        let category = copy::queue_category(&item.state);
        let mut meta = components::format_bytes(item.size);
        if let Some(pages) = item.pages {
            meta.push_str(&format!(" · {pages} pages"));
        }
        let (row, remove_response) = components::file_row(
            ui,
            &item.name,
            &meta,
            category,
            label,
            state.selected == Some(item.index),
            &item.full_path,
        );
        if row.clicked() {
            select = Some(item.index);
        }
        if remove_response.clicked() {
            remove = Some(item.index);
        }
        if let Some(reason) = reason.as_deref().or(item.analysis_error.as_deref()) {
            let p = theme::palette(ui.ctx());
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                components::icon_colored(ui, Icon::Alert, 14.0, p.warning);
                ui.label(RichText::new(reason).size(12.0).color(p.warning));
            });
            ui.add_space(SPACE_1);
        }
        ui.add_space(SPACE_2);
    }
    if let Some(index) = select {
        state.select(index);
        state.set_screen(Screen::Analysis);
    }
    if let Some(index) = remove {
        actions.remove_file = Some(index);
    }
    if state.pending_clear {
        let mut confirm = false;
        let mut cancel = false;
        let response =
            egui::Modal::new(egui::Id::new("confirm_clear_queue")).show(ui.ctx(), |ui| {
                ui.set_max_width(380.0);
                components::section_title(ui, copy::CLEAR_ALL_TITLE);
                ui.label(copy::CLEAR_ALL_BODY);
                ui.add_space(SPACE_4);
                ui.horizontal(|ui| {
                    if components::danger_button(ui, copy::CLEAR_ALL).clicked() {
                        confirm = true;
                    }
                    if components::secondary_button(ui, copy::CANCEL).clicked() {
                        cancel = true;
                    }
                });
            });
        if response.backdrop_response.clicked()
            || ui.ctx().input(|input| input.key_pressed(egui::Key::Escape))
        {
            cancel = true;
        }
        if confirm {
            state.clear_queue();
        } else if cancel {
            state.pending_clear = false;
        }
    }
    let _ = busy;
}

pub fn details(ui: &mut egui::Ui, state: &mut AppState, actions: &mut UiActions) {
    let Some(item) = state.selected_item().cloned() else {
        screen_header(ui, copy::DETAILS_TITLE, "");
        components::empty_state(
            ui,
            Icon::File,
            copy::FILES_EMPTY_TITLE,
            copy::FILES_EMPTY_BODY,
        );
        if components::secondary_button(ui, copy::BACK).clicked() {
            state.set_screen(Screen::Queue);
        }
        return;
    };
    ui.horizontal(|ui| {
        if components::ghost_button(ui, &format!("← {}", copy::DETAILS_BACK)).clicked() {
            state.set_screen(Screen::Queue);
        }
    });
    ui.add_space(SPACE_2);
    screen_header(ui, copy::DETAILS_TITLE, &item.display_name());
    let p = theme::palette(ui.ctx());
    let _ = ui
        .label(components::middle_truncate(
            &item.path.display().to_string(),
            100,
        ))
        .on_hover_text(item.path.display().to_string());
    ui.add_space(SPACE_4);

    if let Some(error) = &item.analysis_error {
        components::banner(ui, copy::IssueCategory::Attention, copy::ANALYSIS_FAILED);
        ui.collapsing(copy::TECHNICAL_DETAILS, |ui| {
            components::mono(ui, error);
        });
        ui.add_space(SPACE_3);
    } else if item.state == QueueState::Analyzing || item.analysis.is_none() {
        ui.horizontal(|ui| {
            ui.spinner();
            components::body_muted(ui, copy::ANALYSIS_PENDING);
        });
        ui.add_space(SPACE_3);
    }

    if let Some(summary) = &item.analysis {
        let counts = [
            (copy::DETAILS_COUNTS[0], summary.pages),
            (copy::DETAILS_COUNTS[1], summary.components),
            (copy::DETAILS_COUNTS[2], summary.styles),
            (copy::DETAILS_COUNTS[3], summary.variables),
            (copy::DETAILS_COUNTS[4], summary.images),
            (copy::DETAILS_COUNTS[5], summary.text_layers),
        ];
        let gap = SPACE_2;
        let card_width = ((ui.available_width() - 2.0 * gap) / 3.0).max(0.0);
        for row in counts.chunks(3) {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                for (label, value) in row {
                    metric_card(ui, label, *value, card_width, p);
                }
            });
            ui.add_space(SPACE_2);
        }
        ui.add_space(SPACE_4);
        components::section_title(ui, copy::THINGS_TO_KNOW);
        let hints = copy::analysis_hints(summary);
        if hints.is_empty() {
            components::banner(ui, copy::IssueCategory::AllGood, copy::NOTHING_UNUSUAL);
        } else {
            for hint in hints {
                components::banner(ui, copy::IssueCategory::Review, &hint);
                ui.add_space(SPACE_2);
            }
        }
        ui.add_space(SPACE_4);
        ui.collapsing(copy::TECHNICAL_DETAILS, |ui| {
            components::mono(
                ui,
                &format!(
                    "{} · version {} · {} nodes · max depth {} · {} parser warning(s)",
                    summary.format,
                    summary.version,
                    summary.nodes,
                    summary.max_depth,
                    summary.warnings
                ),
            );
            components::mono(ui, &summary.container);
        });
    }

    ui.add_space(SPACE_6);
    ui.horizontal(|ui| {
        if components::primary_button(ui, copy::CONVERT_THIS_FILE).clicked() {
            actions.convert_selected = true;
        }
        if components::secondary_button(ui, copy::CHECK_AGAIN).clicked() {
            actions.analyze_selected = true;
        }
    });
}

pub fn settings(ui: &mut egui::Ui, state: &mut AppState, actions: &mut UiActions) {
    screen_header(ui, copy::SETTINGS_TITLE, "");
    let p = theme::palette(ui.ctx());
    let mut changed = false;

    let output_card_width = (ui.available_width() - 34.0).max(0.0);
    egui::Frame::default()
        .fill(p.surface)
        .stroke(egui::Stroke::new(1.0, p.border))
        .corner_radius(egui::CornerRadius::same(theme::RADIUS_CARD))
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.set_min_width(output_card_width);
            ui.set_max_width(output_card_width);
            components::caption(ui, copy::WHERE_SAVE);
            let path = state
                .settings
                .output_dir
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| copy::NEXT_TO_FILE.to_owned());
            ui.horizontal(|ui| {
                components::icon(ui, Icon::Folder, 18.0);
                ui.add(
                    egui::Label::new(components::middle_truncate(&path, 64))
                        .truncate()
                        .sense(egui::Sense::hover()),
                )
                .on_hover_text(path);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if components::secondary_button(ui, copy::CHANGE).clicked() {
                        actions.choose_output = true;
                    }
                    if state.settings.output_dir.is_some()
                        && components::ghost_button(ui, copy::USE_INPUT_FOLDER).clicked()
                    {
                        actions.clear_output = true;
                    }
                });
            });
        });
    ui.add_space(SPACE_6);

    components::section_title(ui, copy::WHAT_MATTERS);
    let mut keep = state.settings.keep_token_bindings;
    let choice_gap = SPACE_3;
    let choice_width = ((ui.available_width() - choice_gap) / 2.0).max(0.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = choice_gap;
        if choice_card(
            ui,
            keep,
            true,
            copy::KEEPS_TOKENS,
            copy::KEEPS_TOKENS_NOTE,
            choice_width,
        ) {
            keep = true;
        }
        if choice_card(
            ui,
            keep,
            false,
            copy::LOOKS_IDENTICAL,
            copy::LOOKS_IDENTICAL_NOTE,
            choice_width,
        ) {
            keep = false;
        }
    });
    if keep != state.settings.keep_token_bindings {
        state.settings.keep_token_bindings = keep;
        changed = true;
    }

    ui.add_space(SPACE_4);
    let before_overwrite = state.settings.overwrite;
    let before_open = state.settings.open_folder_after_conversion;
    ui.collapsing(copy::ADVANCED, |ui| {
        ui.label(RichText::new(copy::OVERWRITE_POLICY).strong());
        ui.radio_value(&mut state.settings.overwrite, false, copy::OVERWRITE_ASK);
        ui.radio_value(&mut state.settings.overwrite, true, copy::OVERWRITE_REPLACE);
        ui.add_enabled(
            false,
            egui::RadioButton::new(false, copy::OVERWRITE_KEEP_BOTH),
        );
        components::caption(ui, copy::OVERWRITE_NOTE);
        ui.add_space(SPACE_2);
        ui.checkbox(
            &mut state.settings.open_folder_after_conversion,
            copy::OPEN_AFTER_CONVERSION,
        );
    });
    if before_overwrite != state.settings.overwrite
        || before_open != state.settings.open_folder_after_conversion
    {
        changed = true;
    }
    if changed {
        actions.settings_changed = true;
    }
    ui.add_space(SPACE_6);
    if components::primary_button(ui, copy::DONE).clicked() {
        state.set_screen(if state.files.is_empty() {
            Screen::Home
        } else {
            Screen::Queue
        });
    }
}

fn metric_card(ui: &mut egui::Ui, label: &str, value: usize, width: f32, p: theme::Palette) {
    egui::Frame::default()
        .fill(p.surface)
        .stroke(egui::Stroke::new(1.0, p.border))
        .corner_radius(egui::CornerRadius::same(theme::RADIUS_CARD))
        .inner_margin(egui::Margin::same(14))
        .show(ui, |ui| {
            ui.set_width((width - 30.0).max(0.0));
            ui.vertical(|ui| {
                components::caption(ui, label);
                ui.label(RichText::new(value.to_string()).size(24.0).strong());
            });
        });
}

fn choice_card(
    ui: &mut egui::Ui,
    current: bool,
    value: bool,
    title: &str,
    note: &str,
    width: f32,
) -> bool {
    let p = theme::palette(ui.ctx());
    let selected = current == value;
    let response = egui::Frame::default()
        .fill(if selected { p.accent_soft } else { p.surface })
        .stroke(egui::Stroke::new(
            1.0,
            if selected { p.accent } else { p.border },
        ))
        .corner_radius(egui::CornerRadius::same(theme::RADIUS_CARD))
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.set_width((width - 34.0).max(0.0));
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(18.0), egui::Sense::hover());
                    ui.painter().circle_stroke(
                        rect.center(),
                        8.0,
                        egui::Stroke::new(1.5, if selected { p.accent } else { p.text_muted }),
                    );
                    if selected {
                        ui.painter().circle_filled(rect.center(), 4.5, p.accent);
                    }
                    ui.label(RichText::new(title).size(16.0).strong());
                });
                ui.add_space(SPACE_1);
                components::caption(ui, note);
            });
        })
        .response;
    response.interact(egui::Sense::click()).clicked()
}

fn queue_stage_label(stage: QueueStage) -> &'static str {
    match stage {
        QueueStage::Reading => copy::STAGE_READING,
        QueueStage::Converting => copy::STAGE_CONVERTING,
        QueueStage::Writing => copy::STAGE_WRITING,
        QueueStage::Validating => copy::STAGE_VALIDATING,
        QueueStage::Saving => copy::STAGE_SAVING,
    }
}

pub fn converting(ui: &mut egui::Ui, state: &mut AppState, actions: &mut UiActions) {
    screen_header(ui, copy::CONVERTING_TITLE, "");
    let p = theme::palette(ui.ctx());
    if let Some(batch) = state.batch.clone() {
        ui.label(format!(
            "{} / {} files finished",
            batch.finished, batch.total
        ));
        ui.add_space(SPACE_3);
        ui.horizontal_wrapped(|ui| {
            if !batch.done {
                let pause = if batch.paused {
                    copy::RESUME
                } else {
                    copy::PAUSE
                };
                if components::secondary_button(ui, pause).clicked() {
                    actions.toggle_pause = true;
                }
                if components::danger_button(ui, copy::CANCEL).clicked() {
                    actions.cancel_batch = true;
                }
            }
        });
        ui.add_space(SPACE_4);
        for item in state.files.iter().filter(|item| item.job_id.is_some()) {
            let (label, _) = copy::queue_state(&item.state);
            let category = copy::queue_category(&item.state);
            egui::Frame::default()
                .fill(p.surface)
                .stroke(egui::Stroke::new(1.0, p.border))
                .corner_radius(egui::CornerRadius::same(theme::RADIUS_CARD))
                .inner_margin(egui::Margin::same(14))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        components::icon(ui, Icon::File, 20.0);
                        ui.label(components::middle_truncate(&item.display_name(), 48))
                            .on_hover_text(item.path.display().to_string());
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            components::chip(ui, category, label);
                            if let Some(stage) = item.batch_stage {
                                ui.label(
                                    RichText::new(queue_stage_label(stage))
                                        .size(12.0)
                                        .color(p.text_muted),
                                );
                            }
                        });
                    });
                });
            ui.add_space(SPACE_2);
        }
        ui.add_space(SPACE_3);
        components::note_banner(ui, copy::NO_OUTPUT_ON_CANCEL);
        components::caption(ui, copy::KEEP_WORKING);
        return;
    }
    let Some(active) = state.active.as_ref() else {
        components::empty_state(ui, Icon::Clock, copy::NO_ACTIVE_JOB, "");
        return;
    };
    let active_name = active
        .input
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| active.input.display().to_string());
    ui.label(RichText::new(active_name).size(18.0).strong());
    components::caption(
        ui,
        &components::middle_truncate(&active.input.display().to_string(), 100),
    );
    ui.add_space(SPACE_3);
    ui.horizontal_wrapped(|ui| {
        let pause = if active.paused {
            copy::RESUME
        } else {
            copy::PAUSE
        };
        if components::secondary_button(ui, pause).clicked() {
            actions.toggle_pause = true;
        }
        if components::danger_button(ui, copy::CANCEL).clicked() {
            actions.cancel_batch = true;
        }
        if active.paused {
            components::caption(ui, copy::PAUSED_NOTE);
        }
    });
    ui.add_space(SPACE_4);
    stage_trail(ui, active);
    ui.add_space(SPACE_4);
    components::note_banner(ui, copy::NO_OUTPUT_ON_CANCEL);
    components::caption(ui, copy::KEEP_WORKING_SINGLE);
}

fn stage_trail(ui: &mut egui::Ui, active: &ActiveJob) {
    let p = theme::palette(ui.ctx());
    let current = active.stage.index();
    for (index, stage) in Stage::ALL.iter().enumerate() {
        ui.horizontal(|ui| {
            if index < current {
                components::icon_colored(ui, Icon::Check, 18.0, p.success);
            } else if index == current && active.paused {
                components::icon_colored(ui, Icon::Pause, 16.0, p.warning);
            } else if index == current {
                ui.spinner();
            } else {
                components::icon_colored(ui, Icon::Clock, 16.0, p.text_muted);
            }
            let text = RichText::new(copy::stage(*stage));
            ui.label(if index <= current {
                text.strong()
            } else {
                text.weak()
            });
        });
        if index + 1 < Stage::ALL.len() {
            ui.add_space(SPACE_1);
        }
    }
}

struct ResultRow {
    input: PathBuf,
    output: PathBuf,
    report_path: PathBuf,
    report: Option<conversion_report::FileReport>,
    status: FileStatus,
    message: Option<String>,
}

impl ResultRow {
    fn from_outcome(outcome: &ConversionOutcome) -> Self {
        Self {
            input: outcome.input.clone(),
            output: outcome.output.clone(),
            report_path: outcome.report_path.clone(),
            report: Some(outcome.report.clone()),
            status: outcome.report.status,
            message: None,
        }
    }

    fn from_job(job: &JobResult) -> Self {
        let status = job
            .report
            .as_ref()
            .map(|report| report.status)
            .unwrap_or(match job.state {
                JobState::Completed => FileStatus::Completed,
                JobState::Failed => FileStatus::Failed,
                JobState::Cancelled => FileStatus::Cancelled,
            });
        Self {
            input: job.input.clone(),
            output: job.output.clone(),
            report_path: job.report_path.clone(),
            report: job.report.clone(),
            status,
            message: job.message.clone(),
        }
    }

    fn category(&self) -> copy::IssueCategory {
        copy::file_status(self.status).1
    }
}

pub fn results(ui: &mut egui::Ui, state: &mut AppState, actions: &mut UiActions) {
    let failed = state
        .batch
        .as_ref()
        .map(|batch| batch.failed)
        .unwrap_or_else(|| {
            usize::from(
                state
                    .result
                    .as_ref()
                    .is_some_and(|result| result.report.status == FileStatus::Failed),
            )
        });
    let converted = state
        .batch
        .as_ref()
        .map(|batch| batch.completed)
        .unwrap_or_else(|| usize::from(state.result.is_some() && failed == 0));
    let cancelled = state
        .batch
        .as_ref()
        .map(|batch| batch.cancelled)
        .unwrap_or_default();
    let title = if failed > 0 {
        copy::RESULTS_TITLE_ATTENTION
    } else if converted == 0 && cancelled > 0 {
        copy::RESULTS_TITLE_CANCELLED
    } else {
        copy::RESULTS_TITLE
    };
    screen_header(ui, title, "");
    let p = theme::palette(ui.ctx());
    let rows: Vec<ResultRow> = if let Some(batch) = &state.batch {
        batch.results.iter().map(ResultRow::from_job).collect()
    } else if let Some(result) = &state.result {
        vec![ResultRow::from_outcome(result)]
    } else {
        Vec::new()
    };
    if rows.is_empty() {
        components::empty_state(ui, Icon::Check, copy::RESULTS_EMPTY, "");
        return;
    }
    if let Some(batch) = &state.batch {
        let review = rows
            .iter()
            .filter(|row| {
                row.status == FileStatus::CompletedWithWarnings
                    || row
                        .report
                        .as_ref()
                        .is_some_and(|report| copy::review_count(report) > 0)
            })
            .count();
        ui.label(copy::batch_summary(
            batch.completed,
            review,
            batch.failed,
            batch.cancelled,
        ));
        ui.add_space(SPACE_4);
    }

    let mut view_report: Option<ConversionOutcome> = None;
    for row in &rows {
        let (label, _) = copy::file_status(row.status);
        let category = row.category();
        egui::Frame::default()
            .fill(p.surface)
            .stroke(egui::Stroke::new(1.0, p.border))
            .corner_radius(egui::CornerRadius::same(theme::RADIUS_CARD))
            .inner_margin(egui::Margin::same(16))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    components::icon_colored(
                        ui,
                        components::category_icon(category),
                        20.0,
                        theme::category_color(p, category),
                    );
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(
                                row.input
                                    .file_name()
                                    .map(|name| name.to_string_lossy().into_owned())
                                    .unwrap_or_else(|| row.input.display().to_string()),
                            )
                            .size(16.0)
                            .strong(),
                        );
                        components::caption(ui, label);
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if row.report.is_some()
                            && components::secondary_button(ui, copy::VIEW_REPORT).clicked()
                        {
                            if let Some(report) = &row.report {
                                view_report = Some(ConversionOutcome {
                                    input: row.input.clone(),
                                    output: row.output.clone(),
                                    report_path: row.report_path.clone(),
                                    report: report.clone(),
                                    validation_checks: 0,
                                });
                            }
                        }
                        if components::ghost_button(ui, copy::OPEN_FOLDER).clicked() {
                            actions.open_folder =
                                Some(row.output.parent().unwrap_or(&row.output).to_path_buf());
                        }
                    });
                });
                if let Some(report) = &row.report {
                    let review = copy::review_count(report);
                    if review > 0 {
                        ui.add_space(SPACE_1);
                        ui.label(
                            RichText::new(format!("{review} {}", copy::NEEDS_CHECK))
                                .color(p.warning)
                                .size(13.0),
                        );
                    }
                }
                if let Some(message) = &row.message {
                    ui.add_space(SPACE_1);
                    ui.label(
                        RichText::new(copy::failure_message(message))
                            .color(p.danger)
                            .size(12.0),
                    );
                    egui::CollapsingHeader::new(copy::SHOW_DETAILS).show(ui, |ui| {
                        components::mono(ui, message);
                    });
                }
            });
        ui.add_space(SPACE_2);
    }
    if let Some(outcome) = view_report {
        state.result = Some(outcome);
        state.filter = Default::default();
        state.set_screen(Screen::Report);
    }

    ui.add_space(SPACE_3);
    let mut bottom_view_report = false;
    ui.horizontal_wrapped(|ui| {
        if components::primary_button(ui, copy::OPEN_FOLDER).clicked() {
            if let Some(folder) = rows
                .iter()
                .find_map(|row| row.output.parent().map(|path| path.to_path_buf()))
            {
                actions.open_folder = Some(folder);
            }
        }
        if state.result.is_some() && components::secondary_button(ui, copy::VIEW_REPORT).clicked() {
            bottom_view_report = true;
        }
        if state.batch.as_ref().is_some_and(|batch| batch.failed > 0)
            && components::secondary_button(ui, copy::RETRY_FAILED).clicked()
        {
            actions.retry_failed = true;
        }
    });
    if bottom_view_report {
        state.filter = Default::default();
        state.set_screen(Screen::Report);
    }
    ui.add_space(SPACE_6);
    egui::Frame::default()
        .fill(p.surface)
        .stroke(egui::Stroke::new(1.0, p.border))
        .corner_radius(egui::CornerRadius::same(theme::RADIUS_CARD))
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.set_min_width((ui.available_width() - 0.0).max(0.0));
            components::section_title(ui, copy::BRING_INTO_PENPOT);
            ui.label(copy::IMPORT_GUIDE);
        });
}

pub fn report(ui: &mut egui::Ui, state: &mut AppState, actions: &mut UiActions) {
    screen_header(ui, copy::REPORT_TITLE, "");
    let Some(result) = state.result.clone() else {
        components::empty_state(ui, Icon::File, copy::REPORT_EMPTY, "");
        return;
    };
    let p = theme::palette(ui.ctx());
    let mut switch_to = None;
    if let Some(batch) = &state.batch {
        if batch.results.len() > 1 {
            ui.horizontal(|ui| {
                ui.label(copy::REPORT_FOR);
                egui::ComboBox::from_id_salt("report_file")
                    .selected_text(components::middle_truncate(
                        &result.input.display().to_string(),
                        52,
                    ))
                    .show_ui(ui, |ui| {
                        for job in &batch.results {
                            if job.report.is_some() {
                                let selected = job.input == result.input;
                                if ui
                                    .selectable_label(
                                        selected,
                                        components::middle_truncate(
                                            &job.input.display().to_string(),
                                            52,
                                        ),
                                    )
                                    .clicked()
                                {
                                    switch_to = Some(job.clone());
                                }
                            }
                        }
                    });
            });
            ui.add_space(SPACE_2);
        }
    }
    if let Some(job) = switch_to {
        if let Some(report) = job.report {
            state.result = Some(ConversionOutcome {
                input: job.input,
                output: job.output,
                report_path: job.report_path,
                report,
                validation_checks: 0,
            });
            state.filter = Default::default();
            return;
        }
    }

    let (status_label, category) = copy::file_status(result.report.status);
    ui.horizontal_wrapped(|ui| {
        components::icon_colored(
            ui,
            components::category_icon(category),
            20.0,
            theme::category_color(p, category),
        );
        ui.label(RichText::new(status_label).size(18.0).strong());
        components::caption(ui, copy::validation(result.report.validation));
    });
    ui.add_space(SPACE_2);
    let _ = ui
        .label(components::middle_truncate(
            &result.input.display().to_string(),
            100,
        ))
        .on_hover_text(result.input.display().to_string());
    ui.add_space(SPACE_3);
    counts_grid(ui, &result.report, p);
    ui.add_space(SPACE_3);
    if result.report.validation != conversion_report::ValidationState::Passed {
        components::banner(
            ui,
            copy::IssueCategory::Review,
            copy::validation(result.report.validation),
        );
        ui.add_space(SPACE_3);
    }
    let compact = ui.available_width() < 900.0;
    let search_width = if compact { 180.0 } else { 240.0 };
    let severity_combo = |ui: &mut egui::Ui, state: &mut AppState| {
        egui::ComboBox::from_id_salt("severity_filter")
            .selected_text(
                state
                    .filter
                    .severity
                    .map(copy::severity)
                    .unwrap_or(copy::REPORT_ALL),
            )
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.filter.severity, None, copy::REPORT_ALL);
                for value in [Severity::Info, Severity::Warning, Severity::Error] {
                    ui.selectable_value(
                        &mut state.filter.severity,
                        Some(value),
                        copy::severity(value),
                    );
                }
            });
    };
    let outcome_combo = |ui: &mut egui::Ui, state: &mut AppState| {
        egui::ComboBox::from_id_salt("outcome_filter")
            .selected_text(
                state
                    .filter
                    .outcome
                    .map(copy::outcome)
                    .unwrap_or(copy::REPORT_ALL),
            )
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.filter.outcome, None, copy::REPORT_ALL);
                for value in [
                    Outcome::Converted,
                    Outcome::ConvertedWithModification,
                    Outcome::Unsupported,
                    Outcome::Failed,
                ] {
                    ui.selectable_value(
                        &mut state.filter.outcome,
                        Some(value),
                        copy::outcome(value),
                    );
                }
            });
    };
    let group_combo = |ui: &mut egui::Ui, state: &mut AppState| {
        egui::ComboBox::from_id_salt("grouping")
            .selected_text(match state.filter.grouping {
                ReportGrouping::Category => copy::GROUP_CATEGORY,
                ReportGrouping::Page => copy::GROUP_PAGE,
                ReportGrouping::None => copy::GROUP_NONE,
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut state.filter.grouping,
                    ReportGrouping::Category,
                    copy::GROUP_CATEGORY,
                );
                ui.selectable_value(
                    &mut state.filter.grouping,
                    ReportGrouping::Page,
                    copy::GROUP_PAGE,
                );
                ui.selectable_value(
                    &mut state.filter.grouping,
                    ReportGrouping::None,
                    copy::GROUP_NONE,
                );
            });
    };
    let export_menu = |ui: &mut egui::Ui, actions: &mut UiActions| {
        ui.menu_button(copy::EXPORT, |ui| {
            if ui.button(copy::EXPORT_HTML).clicked() {
                actions.export_html = true;
                ui.close();
            }
            if ui.button(copy::EXPORT_CSV).clicked() {
                actions.export_csv = true;
                ui.close();
            }
            if ui.button(copy::EXPORT_JSON).clicked() {
                actions.export_json = true;
                ui.close();
            }
        });
    };
    if compact {
        ui.horizontal_wrapped(|ui| {
            components::search_field_sized(
                ui,
                &mut state.filter.search,
                copy::REPORT_SEARCH,
                search_width,
            );
            ui.label(copy::REPORT_SEVERITY);
            severity_combo(ui, state);
            ui.label(copy::REPORT_RESULT);
            outcome_combo(ui, state);
        });
        ui.add_space(SPACE_2);
        ui.horizontal_wrapped(|ui| {
            ui.label(copy::REPORT_GROUP);
            group_combo(ui, state);
            export_menu(ui, actions);
        });
    } else {
        ui.horizontal_wrapped(|ui| {
            components::search_field_sized(
                ui,
                &mut state.filter.search,
                copy::REPORT_SEARCH,
                search_width,
            );
            ui.separator();
            ui.label(copy::REPORT_SEVERITY);
            severity_combo(ui, state);
            ui.separator();
            ui.label(copy::REPORT_RESULT);
            outcome_combo(ui, state);
            ui.separator();
            ui.label(copy::REPORT_GROUP);
            group_combo(ui, state);
            ui.separator();
            export_menu(ui, actions);
        });
    }
    ui.add_space(SPACE_3);

    let events: Vec<&conversion_report::ReportEvent> = result
        .report
        .events
        .iter()
        .filter(|event| state.filter.matches(event))
        .collect();
    if events.is_empty() {
        components::empty_state(ui, Icon::Check, copy::REPORT_EMPTY_FILTER, "");
        return;
    }
    ui.label(
        RichText::new(copy::report_summary(
            result.report.counts.converted,
            result.report.counts.approximated,
            result.report.counts.unsupported + result.report.counts.skipped,
        ))
        .color(p.text_muted),
    );
    ui.add_space(SPACE_2);
    egui::ScrollArea::vertical()
        .max_height(520.0)
        .show(ui, |ui| match state.filter.grouping {
            ReportGrouping::Category => {
                for category in copy::IssueCategory::ALL {
                    let group: Vec<_> = events
                        .iter()
                        .copied()
                        .filter(|event| copy::category_of(event) == category)
                        .collect();
                    if group.is_empty() {
                        continue;
                    }
                    egui::CollapsingHeader::new(format!("{} ({})", category.label(), group.len()))
                        .default_open(category != copy::IssueCategory::AllGood)
                        .show(ui, |ui| {
                            for event in group {
                                issue_card(ui, event);
                            }
                        });
                }
            }
            ReportGrouping::Page => {
                let mut groups: std::collections::BTreeMap<String, Vec<_>> =
                    std::collections::BTreeMap::new();
                for event in &events {
                    groups
                        .entry(
                            event
                                .element
                                .page
                                .clone()
                                .unwrap_or_else(|| copy::UNKNOWN.to_owned()),
                        )
                        .or_default()
                        .push(*event);
                }
                for (page, group) in groups {
                    egui::CollapsingHeader::new(format!("{page} ({})", group.len()))
                        .default_open(true)
                        .show(ui, |ui| {
                            for event in group {
                                issue_card(ui, event);
                            }
                        });
                }
            }
            ReportGrouping::None => {
                for event in events {
                    issue_card(ui, event);
                }
            }
        });
}

fn counts_grid(ui: &mut egui::Ui, report: &conversion_report::FileReport, p: theme::Palette) {
    ui.horizontal_wrapped(|ui| {
        for (label, value) in [
            (copy::CONVERTED_LABEL, report.counts.converted),
            (copy::NEEDS_REVIEW_SHORT, report.counts.approximated),
            (
                copy::FAILED_LABEL,
                report.counts.failed + report.counts.unsupported,
            ),
            (copy::SKIPPED_LABEL, report.counts.skipped),
        ] {
            egui::Frame::default()
                .fill(p.surface)
                .stroke(egui::Stroke::new(1.0, p.border))
                .corner_radius(egui::CornerRadius::same(theme::RADIUS_CARD))
                .inner_margin(egui::Margin::symmetric(14, 8))
                .show(ui, |ui| {
                    components::caption(ui, label);
                    ui.label(RichText::new(value.to_string()).size(20.0).strong());
                });
        }
    });
}

fn issue_card(ui: &mut egui::Ui, event: &conversion_report::ReportEvent) {
    let p = theme::palette(ui.ctx());
    let category = copy::category_of(event);
    let advice = copy::issue(&event.code);
    let content_width = (ui.available_width() - 30.0).max(0.0);
    egui::Frame::default()
        .fill(p.surface)
        .stroke(egui::Stroke::new(1.0, p.border))
        .corner_radius(egui::CornerRadius::same(theme::RADIUS_CARD))
        .inner_margin(egui::Margin::same(14))
        .show(ui, |ui| {
            ui.set_min_width(content_width);
            ui.set_max_width(content_width);
            ui.horizontal(|ui| {
                components::icon_colored(
                    ui,
                    components::category_icon(category),
                    18.0,
                    theme::category_color(p, category),
                );
                ui.label(RichText::new(advice.title).size(15.0).strong());
            });
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(&event.element.name).strong());
                if let Some(page) = &event.element.page {
                    components::caption(ui, page);
                }
            });
            let path = event.element.path.join(" / ");
            if !path.is_empty() {
                let _ = ui
                    .label(RichText::new(&path).size(12.0).color(p.text_muted))
                    .on_hover_text(path);
            }
            ui.add_space(SPACE_2);
            labeled_line(ui, copy::WHAT_HAPPENED, advice.happened, p);
            labeled_line(ui, copy::WHAT_WE_DID, copy::what_we_did(category), p);
            labeled_line(ui, copy::WHAT_TO_DO, advice.action, p);
            ui.add_space(SPACE_1);
            if ui.small_button(copy::COPY_ISSUE).clicked() {
                ui.ctx().copy_text(format!(
                    "{} — {} — {}",
                    advice.title, event.element.name, advice.action
                ));
            }
            egui::CollapsingHeader::new(copy::SHOW_DETAILS)
                .id_salt((
                    "issue_details",
                    event.element.figma_node_id.as_str(),
                    event.code.as_str(),
                ))
                .show(ui, |ui| {
                    components::mono(
                        ui,
                        &format!("{} · node {}", event.code, event.element.figma_node_id),
                    );
                    components::mono(ui, &event.description);
                    if let Some(reason) = &event.reason {
                        components::mono(ui, reason);
                    }
                    if ui.small_button(copy::COPY_LAYER).clicked() {
                        ui.ctx().copy_text(event.element.name.clone());
                    }
                });
        });
    ui.add_space(SPACE_2);
}

fn labeled_line(ui: &mut egui::Ui, label: &str, text: &str, p: theme::Palette) {
    ui.horizontal_wrapped(|ui| {
        ui.label(
            RichText::new(format!("{label}:"))
                .size(12.0)
                .color(p.text_muted),
        );
        ui.label(RichText::new(text).size(13.0));
    });
}

pub fn preferences(ui: &mut egui::Ui, state: &mut AppState, actions: &mut UiActions) {
    screen_header(ui, copy::PREFERENCES_TITLE, "");
    let p = theme::palette(ui.ctx());
    let before = state.settings.theme;
    ui.label(RichText::new(copy::APPEARANCE).strong());
    if let Some(choice) = components::segmented(
        ui,
        state.settings.theme,
        &[
            (ThemeChoice::System, copy::THEME_SYSTEM),
            (ThemeChoice::Light, copy::THEME_LIGHT),
            (ThemeChoice::Dark, copy::THEME_DARK),
        ],
    ) {
        state.settings.theme = choice;
        theme::apply(ui.ctx(), choice);
        actions.settings_changed = true;
    }
    let _ = before;
    ui.add_space(SPACE_4);
    ui.label(RichText::new(copy::DEFAULT_OUTPUT).strong());
    ui.horizontal(|ui| {
        let path = state
            .settings
            .output_dir
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| copy::NEXT_TO_FILE.to_owned());
        ui.label(components::middle_truncate(&path, 68))
            .on_hover_text(path);
        if components::secondary_button(ui, copy::CHANGE).clicked() {
            actions.choose_output = true;
        }
    });
    ui.add_space(SPACE_4);
    ui.horizontal(|ui| {
        ui.label(RichText::new(copy::LANGUAGE).strong());
        ui.label(copy::LANGUAGE_ENGLISH);
        components::caption(ui, copy::LANGUAGE_NOTE);
    });
    ui.add_space(SPACE_6);
    egui::Frame::default()
        .fill(p.surface)
        .stroke(egui::Stroke::new(1.0, p.border))
        .corner_radius(egui::CornerRadius::same(theme::RADIUS_CARD))
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            components::section_title(ui, copy::PRIVACY_TITLE);
            ui.label(copy::PRIVACY_LONG);
        });
    ui.add_space(SPACE_4);
    ui.horizontal(|ui| {
        components::caption(
            ui,
            &format!("{} {}", copy::VERSION, env!("CARGO_PKG_VERSION")),
        );
        ui.label("·");
        if components::ghost_button(ui, copy::LICENSES).clicked() {
            state.show_licenses = true;
        }
    });
    ui.add_space(SPACE_4);
    if components::ghost_button(ui, copy::RESET_PREFERENCES).clicked() {
        state.pending_reset = true;
    }

    if state.show_licenses {
        let mut close = false;
        let response = egui::Modal::new(egui::Id::new("licenses")).show(ui.ctx(), |ui| {
            ui.set_max_width(620.0);
            components::section_title(ui, copy::LICENSES);
            ui.label(copy::LICENSE_FONT);
            ui.label(copy::LICENSE_APP);
            ui.add_space(SPACE_2);
            components::caption(ui, copy::LICENSE_SCROLL);
            egui::ScrollArea::vertical()
                .max_height(320.0)
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(include_str!("../assets/fonts/OFL.txt"))
                            .size(11.0)
                            .color(theme::palette(ui.ctx()).text_muted),
                    );
                });
            ui.add_space(SPACE_2);
            if components::secondary_button(ui, copy::DONE).clicked() {
                close = true;
            }
        });
        if response.backdrop_response.clicked()
            || ui.ctx().input(|input| input.key_pressed(egui::Key::Escape))
            || close
        {
            state.show_licenses = false;
        }
    }

    if state.pending_reset {
        let mut confirm = false;
        let mut cancel = false;
        let response = egui::Modal::new(egui::Id::new("reset_preferences")).show(ui.ctx(), |ui| {
            ui.set_max_width(380.0);
            components::section_title(ui, copy::RESET_TITLE);
            ui.label(copy::RESET_BODY);
            ui.add_space(SPACE_4);
            ui.horizontal(|ui| {
                if components::danger_button(ui, copy::RESET_ACTION).clicked() {
                    confirm = true;
                }
                if components::secondary_button(ui, copy::CANCEL).clicked() {
                    cancel = true;
                }
            });
        });
        if response.backdrop_response.clicked()
            || ui.ctx().input(|input| input.key_pressed(egui::Key::Escape))
        {
            cancel = true;
        }
        if confirm {
            state.pending_reset = false;
            actions.reset_settings = true;
        } else if cancel {
            state.pending_reset = false;
        }
    }
}

pub fn gallery(ui: &mut egui::Ui) {
    let p = theme::palette(ui.ctx());
    ui.label(RichText::new("Component gallery").size(28.0).strong());
    components::caption(ui, "Dev-only. Check both themes and long text.");
    ui.add_space(SPACE_4);
    components::section_title(ui, "Buttons");
    ui.horizontal_wrapped(|ui| {
        components::primary_button(ui, copy::CONVERT);
        components::secondary_button(ui, copy::CHOOSE_FILES);
        components::ghost_button(ui, copy::CANCEL);
        components::danger_button(ui, copy::REMOVE_COMPLETED);
    });
    ui.add_space(SPACE_4);
    components::section_title(ui, "Status chips");
    ui.horizontal_wrapped(|ui| {
        for category in copy::IssueCategory::ALL {
            components::chip(ui, category, category.label());
        }
    });
    ui.add_space(SPACE_4);
    components::section_title(ui, "Banners");
    components::banner(ui, copy::IssueCategory::AllGood, "Everything converted.");
    components::banner(
        ui,
        copy::IssueCategory::Review,
        "Three things need a quick look.",
    );
    components::banner(
        ui,
        copy::IssueCategory::Attention,
        "One layer couldn’t be converted.",
    );
    ui.add_space(SPACE_4);
    components::section_title(ui, "File row");
    let _ = components::file_row(
        ui,
        "Landing page with a very long file name that must survive translation.fig",
        "2 pages · 1.2 MB",
        copy::IssueCategory::Review,
        copy::NEEDS_REVIEW,
        true,
        "/Users/designer/Projects/Penpot migration/Landing page.fig",
    );
    ui.add_space(SPACE_4);
    components::section_title(ui, "Type and spacing");
    ui.label(RichText::new("Title 28").size(28.0).strong());
    ui.label(RichText::new("Section 20").size(20.0).strong());
    ui.label(RichText::new("Body 14").size(14.0));
    components::caption(ui, "Caption 12 · metadata");
    components::mono(ui, "F2P-EXAMPLE · technical details only");
    let _ = p;
}
