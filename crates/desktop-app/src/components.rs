//! Reusable UI widgets built from theme tokens. Screens compose these instead of styling
//! individual egui widgets.

use eframe::egui::{
    self, Color32, CornerRadius, FontId, Pos2, Rect, Response, RichText, Sense, Stroke, StrokeKind,
    Vec2,
};

use crate::copy;
use crate::copy::IssueCategory;
use crate::theme::{self, RADIUS_CARD, RADIUS_INPUT, RADIUS_PILL, SPACE_2, SPACE_4};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    File,
    Folder,
    Check,
    Alert,
    XCircle,
    Clock,
    Pause,
    Play,
    Trash,
    ArrowRight,
    ChevronRight,
    Search,
    Settings,
    Info,
    Upload,
    Download,
    Sparkle,
}

pub fn icon(ui: &mut egui::Ui, icon: Icon, size: f32) -> Response {
    let palette = theme::palette(ui.ctx());
    icon_colored(ui, icon, size, palette.text_muted)
}

pub fn icon_colored(ui: &mut egui::Ui, icon: Icon, size: f32, color: Color32) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    paint_icon(ui.painter(), rect, icon, color);
    response
}

pub fn paint_icon(painter: &egui::Painter, rect: Rect, icon: Icon, color: Color32) {
    let stroke = Stroke::new((rect.width() / 14.0).clamp(1.2, 2.0), color);
    let c = rect.center();
    let s = rect.width().min(rect.height());
    let half = s * 0.32;
    match icon {
        Icon::File => {
            let file =
                Rect::from_center_size(c + Vec2::new(0.0, 1.0), Vec2::new(s * 0.58, s * 0.72));
            painter.rect_stroke(file, CornerRadius::same(2), stroke, StrokeKind::Inside);
            painter.line_segment(
                [
                    Pos2::new(file.right() - s * 0.2, file.top()),
                    Pos2::new(file.right(), file.top() + s * 0.2),
                ],
                stroke,
            );
        }
        Icon::Folder => {
            let top = c.y - half * 0.55;
            let points = vec![
                Pos2::new(c.x - half, top + s * 0.1),
                Pos2::new(c.x - half, c.y + half * 0.6),
                Pos2::new(c.x + half, c.y + half * 0.6),
                Pos2::new(c.x + half, top + s * 0.2),
                Pos2::new(c.x - s * 0.05, top + s * 0.2),
                Pos2::new(c.x - s * 0.12, top),
                Pos2::new(c.x - half, top),
            ];
            painter.add(egui::Shape::line(points, stroke));
        }
        Icon::Check => {
            painter.line_segment(
                [
                    Pos2::new(c.x - half * 0.8, c.y),
                    Pos2::new(c.x - half * 0.1, c.y + half * 0.7),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(c.x - half * 0.1, c.y + half * 0.7),
                    Pos2::new(c.x + half, c.y - half * 0.7),
                ],
                stroke,
            );
        }
        Icon::Alert => {
            let points = vec![
                Pos2::new(c.x, c.y - half),
                Pos2::new(c.x - half, c.y + half * 0.8),
                Pos2::new(c.x + half, c.y + half * 0.8),
                Pos2::new(c.x, c.y - half),
            ];
            painter.add(egui::Shape::line(points, stroke));
            painter.line_segment(
                [
                    Pos2::new(c.x, c.y - half * 0.35),
                    Pos2::new(c.x, c.y + half * 0.25),
                ],
                stroke,
            );
            painter.circle_filled(Pos2::new(c.x, c.y + half * 0.52), stroke.width * 0.7, color);
        }
        Icon::XCircle => {
            painter.circle_stroke(c, half, stroke);
            painter.line_segment(
                [
                    Pos2::new(c.x - half * 0.4, c.y - half * 0.4),
                    Pos2::new(c.x + half * 0.4, c.y + half * 0.4),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(c.x + half * 0.4, c.y - half * 0.4),
                    Pos2::new(c.x - half * 0.4, c.y + half * 0.4),
                ],
                stroke,
            );
        }
        Icon::Clock => {
            painter.circle_stroke(c, half, stroke);
            painter.line_segment([c, Pos2::new(c.x, c.y - half * 0.55)], stroke);
            painter.line_segment([c, Pos2::new(c.x + half * 0.45, c.y + half * 0.15)], stroke);
        }
        Icon::Pause => {
            painter.line_segment(
                [
                    Pos2::new(c.x - half * 0.35, c.y - half * 0.7),
                    Pos2::new(c.x - half * 0.35, c.y + half * 0.7),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(c.x + half * 0.35, c.y - half * 0.7),
                    Pos2::new(c.x + half * 0.35, c.y + half * 0.7),
                ],
                stroke,
            );
        }
        Icon::Play => {
            let points = vec![
                Pos2::new(c.x - half * 0.45, c.y - half * 0.7),
                Pos2::new(c.x + half * 0.7, c.y),
                Pos2::new(c.x - half * 0.45, c.y + half * 0.7),
                Pos2::new(c.x - half * 0.45, c.y - half * 0.7),
            ];
            painter.add(egui::Shape::line(points, stroke));
        }
        Icon::Trash => {
            painter.line_segment(
                [
                    Pos2::new(c.x - half * 0.8, c.y - half * 0.6),
                    Pos2::new(c.x + half * 0.8, c.y - half * 0.6),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(c.x - half * 0.4, c.y - half * 0.85),
                    Pos2::new(c.x + half * 0.4, c.y - half * 0.85),
                ],
                stroke,
            );
            let body = Rect::from_min_max(
                Pos2::new(c.x - half * 0.55, c.y - half * 0.4),
                Pos2::new(c.x + half * 0.55, c.y + half * 0.8),
            );
            painter.rect_stroke(body, CornerRadius::same(2), stroke, StrokeKind::Inside);
            for dx in [-0.25, 0.0, 0.25] {
                painter.line_segment(
                    [
                        Pos2::new(c.x + half * dx, c.y - half * 0.1),
                        Pos2::new(c.x + half * dx, c.y + half * 0.5),
                    ],
                    stroke,
                );
            }
        }
        Icon::ArrowRight => {
            painter.line_segment(
                [Pos2::new(c.x - half, c.y), Pos2::new(c.x + half * 0.8, c.y)],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(c.x + half * 0.2, c.y - half * 0.5),
                    Pos2::new(c.x + half * 0.8, c.y),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(c.x + half * 0.2, c.y + half * 0.5),
                    Pos2::new(c.x + half * 0.8, c.y),
                ],
                stroke,
            );
        }
        Icon::ChevronRight => {
            painter.line_segment(
                [
                    Pos2::new(c.x - half * 0.35, c.y - half * 0.5),
                    Pos2::new(c.x + half * 0.35, c.y),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(c.x + half * 0.35, c.y),
                    Pos2::new(c.x - half * 0.35, c.y + half * 0.5),
                ],
                stroke,
            );
        }
        Icon::Search => {
            painter.circle_stroke(
                Pos2::new(c.x - half * 0.2, c.y - half * 0.2),
                half * 0.55,
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(c.x + half * 0.2, c.y + half * 0.2),
                    Pos2::new(c.x + half * 0.8, c.y + half * 0.8),
                ],
                stroke,
            );
        }
        Icon::Settings => {
            painter.circle_stroke(c, half * 0.42, stroke);
            for index in 0..8 {
                let angle = index as f32 * std::f32::consts::TAU / 8.0;
                let inner = half * 0.62;
                let outer = half * 0.95;
                painter.line_segment(
                    [
                        Pos2::new(c.x + angle.cos() * inner, c.y + angle.sin() * inner),
                        Pos2::new(c.x + angle.cos() * outer, c.y + angle.sin() * outer),
                    ],
                    stroke,
                );
            }
        }
        Icon::Info => {
            painter.circle_stroke(c, half, stroke);
            painter.circle_filled(Pos2::new(c.x, c.y - half * 0.55), stroke.width * 0.7, color);
            painter.line_segment(
                [
                    Pos2::new(c.x, c.y - half * 0.15),
                    Pos2::new(c.x, c.y + half * 0.55),
                ],
                stroke,
            );
        }
        Icon::Upload => {
            painter.line_segment(
                [
                    Pos2::new(c.x, c.y + half * 0.8),
                    Pos2::new(c.x, c.y - half * 0.6),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(c.x - half * 0.5, c.y - half * 0.1),
                    Pos2::new(c.x, c.y - half * 0.6),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(c.x + half * 0.5, c.y - half * 0.1),
                    Pos2::new(c.x, c.y - half * 0.6),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(c.x - half * 0.8, c.y + half * 0.7),
                    Pos2::new(c.x + half * 0.8, c.y + half * 0.7),
                ],
                stroke,
            );
        }
        Icon::Download => {
            painter.line_segment(
                [
                    Pos2::new(c.x, c.y - half * 0.8),
                    Pos2::new(c.x, c.y + half * 0.4),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(c.x - half * 0.5, c.y - half * 0.1),
                    Pos2::new(c.x, c.y + half * 0.4),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(c.x + half * 0.5, c.y - half * 0.1),
                    Pos2::new(c.x, c.y + half * 0.4),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(c.x - half * 0.8, c.y + half * 0.7),
                    Pos2::new(c.x + half * 0.8, c.y + half * 0.7),
                ],
                stroke,
            );
        }
        Icon::Sparkle => {
            painter.line_segment(
                [Pos2::new(c.x, c.y - half), Pos2::new(c.x, c.y + half)],
                stroke,
            );
            painter.line_segment(
                [Pos2::new(c.x - half, c.y), Pos2::new(c.x + half, c.y)],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(c.x - half * 0.55, c.y - half * 0.55),
                    Pos2::new(c.x + half * 0.55, c.y + half * 0.55),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(c.x + half * 0.55, c.y - half * 0.55),
                    Pos2::new(c.x - half * 0.55, c.y + half * 0.55),
                ],
                stroke,
            );
        }
    }
}

pub fn primary_button(ui: &mut egui::Ui, label: &str) -> Response {
    let p = theme::palette(ui.ctx());
    ui.add(
        egui::Button::new(RichText::new(label).color(p.on_accent))
            .fill(p.accent)
            .stroke(Stroke::NONE)
            .corner_radius(CornerRadius::same(RADIUS_INPUT))
            .min_size(Vec2::new(0.0, 36.0)),
    )
}

pub fn secondary_button(ui: &mut egui::Ui, label: &str) -> Response {
    let p = theme::palette(ui.ctx());
    ui.add(
        egui::Button::new(RichText::new(label).color(p.text))
            .fill(p.surface)
            .stroke(Stroke::new(1.0, p.border))
            .corner_radius(CornerRadius::same(RADIUS_INPUT))
            .min_size(Vec2::new(0.0, 36.0)),
    )
}

pub fn ghost_button(ui: &mut egui::Ui, label: &str) -> Response {
    let p = theme::palette(ui.ctx());
    ui.add(
        egui::Button::new(RichText::new(label).color(p.text_muted))
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::NONE)
            .corner_radius(CornerRadius::same(RADIUS_INPUT))
            .min_size(Vec2::new(0.0, 32.0)),
    )
}

pub fn danger_button(ui: &mut egui::Ui, label: &str) -> Response {
    let p = theme::palette(ui.ctx());
    ui.add(
        egui::Button::new(RichText::new(label).color(p.danger))
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::new(1.0, p.danger))
            .corner_radius(CornerRadius::same(RADIUS_INPUT))
            .min_size(Vec2::new(0.0, 34.0)),
    )
}

pub fn icon_button(ui: &mut egui::Ui, icon_kind: Icon, tooltip: &str) -> Response {
    let p = theme::palette(ui.ctx());
    let (rect, response) = ui.allocate_exact_size(Vec2::new(32.0, 32.0), Sense::click());
    if response.hovered() {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(RADIUS_INPUT), p.surface_muted);
    }
    paint_icon(ui.painter(), rect, icon_kind, p.text_muted);
    response.on_hover_text(tooltip)
}

pub fn chip(ui: &mut egui::Ui, category: IssueCategory, label: &str) {
    let p = theme::palette(ui.ctx());
    let color = theme::category_color(p, category);
    egui::Frame::default()
        .fill(p.surface_muted)
        .stroke(Stroke::new(1.0, color))
        .corner_radius(CornerRadius::same(RADIUS_PILL))
        .inner_margin(egui::Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                icon_colored(ui, category_icon(category), 14.0, color);
                ui.label(RichText::new(label).size(12.0).color(color));
            });
        });
}

pub fn category_icon(category: IssueCategory) -> Icon {
    match category {
        IssueCategory::Attention => Icon::XCircle,
        IssueCategory::NotConverted => Icon::Alert,
        IssueCategory::Review => Icon::Alert,
        IssueCategory::AllGood => Icon::Check,
    }
}

pub fn note_banner(ui: &mut egui::Ui, text: &str) {
    let p = theme::palette(ui.ctx());
    egui::Frame::default()
        .fill(p.surface_muted)
        .stroke(Stroke::new(1.0, p.border))
        .corner_radius(CornerRadius::same(RADIUS_CARD))
        .inner_margin(egui::Margin::symmetric(12, 9))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                icon_colored(ui, Icon::Info, 16.0, p.text_muted);
                ui.label(RichText::new(text).color(p.text));
            });
        });
}

pub fn banner(ui: &mut egui::Ui, category: IssueCategory, text: &str) {
    let p = theme::palette(ui.ctx());
    let color = theme::category_color(p, category);
    egui::Frame::default()
        .fill(p.surface_muted)
        .stroke(Stroke::new(1.0, color))
        .corner_radius(CornerRadius::same(RADIUS_CARD))
        .inner_margin(egui::Margin::symmetric(12, 9))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                icon_colored(ui, category_icon(category), 16.0, color);
                ui.label(RichText::new(text).color(p.text));
            });
        });
}

pub fn empty_state(ui: &mut egui::Ui, icon_kind: Icon, title: &str, body: &str) {
    let p = theme::palette(ui.ctx());
    ui.vertical_centered(|ui| {
        ui.add_space(SPACE_4);
        icon_colored(ui, icon_kind, 38.0, p.text_muted);
        ui.add_space(SPACE_2);
        ui.label(RichText::new(title).size(18.0).strong());
        ui.label(RichText::new(body).color(p.text_muted));
        ui.add_space(SPACE_2);
    });
}

pub fn section_title(ui: &mut egui::Ui, title: &str) {
    ui.label(RichText::new(title).size(20.0).strong());
    ui.add_space(SPACE_2);
}

pub fn caption(ui: &mut egui::Ui, text: &str) {
    let p = theme::palette(ui.ctx());
    ui.label(RichText::new(text).size(12.0).color(p.text_muted));
}

pub fn body_muted(ui: &mut egui::Ui, text: &str) {
    let p = theme::palette(ui.ctx());
    ui.label(RichText::new(text).color(p.text_muted));
}

pub fn search_field(ui: &mut egui::Ui, value: &mut String, hint: &str) {
    search_field_sized(ui, value, hint, 240.0);
}

pub fn search_field_sized(ui: &mut egui::Ui, value: &mut String, hint: &str, width: f32) {
    let p = theme::palette(ui.ctx());
    ui.horizontal(|ui| {
        icon_colored(ui, Icon::Search, 15.0, p.text_muted);
        ui.add(
            egui::TextEdit::singleline(value)
                .hint_text(hint)
                .desired_width(width)
                .margin(Vec2::new(8.0, 7.0)),
        );
    });
}

pub fn segmented<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    current: T,
    options: &[(T, &'static str)],
) -> Option<T> {
    let p = theme::palette(ui.ctx());
    let mut changed = None;
    egui::Frame::default()
        .fill(p.surface_muted)
        .corner_radius(CornerRadius::same(RADIUS_PILL))
        .inner_margin(egui::Margin::same(3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                for (value, label) in options {
                    let selected = current == *value;
                    let response = ui.add(
                        egui::Button::new(RichText::new(*label).size(13.0).color(if selected {
                            p.text
                        } else {
                            p.text_muted
                        }))
                        .fill(if selected {
                            p.surface
                        } else {
                            Color32::TRANSPARENT
                        })
                        .stroke(if selected {
                            Stroke::new(1.0, p.border)
                        } else {
                            Stroke::NONE
                        })
                        .corner_radius(CornerRadius::same(RADIUS_PILL))
                        .min_size(Vec2::new(0.0, 28.0)),
                    );
                    if response.clicked() && !selected {
                        changed = Some(*value);
                    }
                }
            });
        });
    changed
}

pub fn file_row(
    ui: &mut egui::Ui,
    name: &str,
    meta: &str,
    category: IssueCategory,
    status: &str,
    selected: bool,
    full_path: &str,
) -> (Response, Response) {
    let p = theme::palette(ui.ctx());
    let fill = if selected { p.accent_soft } else { p.surface };
    let mut remove_response = None;
    // Set the inner width explicitly: `available_width` inside a `Frame` would otherwise
    // re-add the margins and push the row past the content column.
    let content_width = (ui.available_width() - 30.0).max(0.0);
    let response = egui::Frame::default()
        .fill(fill)
        .stroke(Stroke::new(1.0, if selected { p.accent } else { p.border }))
        .corner_radius(CornerRadius::same(RADIUS_CARD))
        .inner_margin(egui::Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.set_min_width(content_width);
            ui.set_max_width(content_width);
            let right_width = 166.0;
            let left_width = (ui.available_width() - right_width).max(0.0);
            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    Vec2::new(left_width, 46.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        egui::Frame::default()
                            .fill(p.surface_muted)
                            .corner_radius(CornerRadius::same(8))
                            .inner_margin(egui::Margin::same(8))
                            .show(ui, |ui| {
                                icon_colored(ui, Icon::File, 22.0, p.text_muted);
                            });
                        ui.add_space(SPACE_2);
                        ui.vertical(|ui| {
                            ui.label(RichText::new(middle_truncate(name, 52)).size(16.0).strong())
                                .on_hover_text(name);
                            ui.add_space(1.0);
                            caption(ui, meta);
                        });
                    },
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    remove_response = Some(icon_button(ui, Icon::Trash, copy::REMOVE));
                    ui.add_space(SPACE_2);
                    chip(ui, category, status);
                });
            });
        })
        .response;
    let row = response
        .interact(Sense::click())
        .on_hover_text(full_path.to_owned());
    (row.clone(), remove_response.unwrap_or(row))
}

pub fn card_heading(ui: &mut egui::Ui, title: &str, subtitle: Option<&str>) {
    ui.label(RichText::new(title).size(18.0).strong());
    if let Some(subtitle) = subtitle {
        body_muted(ui, subtitle);
    }
    ui.add_space(SPACE_2);
}

pub fn label_value(ui: &mut egui::Ui, label: &str, value: &str) {
    let p = theme::palette(ui.ctx());
    ui.vertical(|ui| {
        caption(ui, label);
        ui.label(RichText::new(value).size(18.0).strong().color(p.text));
    });
}

pub fn divider(ui: &mut egui::Ui) {
    ui.add_space(SPACE_2);
    let p = theme::palette(ui.ctx());
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
    ui.painter().rect_filled(rect, 0, p.border);
    ui.add_space(SPACE_2);
}

pub fn page_background(ui: &mut egui::Ui) {
    let p = theme::palette(ui.ctx());
    ui.painter().rect_filled(ui.max_rect(), 0, p.bg);
}

#[allow(dead_code)]
pub fn mono(ui: &mut egui::Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .font(FontId::monospace(12.5))
            .color(theme::palette(ui.ctx()).text_muted),
    );
}

#[allow(dead_code)]
pub fn spacer(ui: &mut egui::Ui, amount: f32) {
    ui.add_space(amount);
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
