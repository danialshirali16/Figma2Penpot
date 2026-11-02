//! Design tokens and egui theme application. Screens must use these tokens rather than
//! hard-coded colours, spacing, or type sizes.

use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, TextStyle,
    Theme, ThemePreference,
};
use std::sync::Arc;

use crate::copy::IssueCategory;
use crate::settings::ThemeChoice;

// Spacing scale
pub const SPACE_1: f32 = 4.0;
pub const SPACE_2: f32 = 8.0;
pub const SPACE_3: f32 = 12.0;
pub const SPACE_4: f32 = 16.0;
pub const SPACE_6: f32 = 24.0;
pub const SPACE_8: f32 = 32.0;
pub const SPACE_12: f32 = 48.0;

// Radii
pub const RADIUS_INPUT: u8 = 6;
pub const RADIUS_CARD: u8 = 10;
pub const RADIUS_LARGE: u8 = 14;
pub const RADIUS_PILL: u8 = 255;

const FONT_NAME: &str = "inter";

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub bg: Color32,
    pub surface: Color32,
    pub surface_muted: Color32,
    pub border: Color32,
    pub text: Color32,
    pub text_muted: Color32,
    pub accent: Color32,
    pub accent_soft: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub danger: Color32,
    pub info: Color32,
    pub on_accent: Color32,
}

pub fn light() -> Palette {
    Palette {
        bg: Color32::from_rgb(0xF5, 0xF5, 0xF2),
        surface: Color32::from_rgb(0xFF, 0xFF, 0xFF),
        surface_muted: Color32::from_rgb(0xEC, 0xED, 0xE8),
        border: Color32::from_rgb(0xDA, 0xDB, 0xD4),
        text: Color32::from_rgb(0x1B, 0x1D, 0x1A),
        text_muted: Color32::from_rgb(0x65, 0x68, 0x61),
        accent: Color32::from_rgb(0x0F, 0x7A, 0x63),
        accent_soft: Color32::from_rgb(0xE2, 0xF2, 0xEC),
        success: Color32::from_rgb(0x1E, 0x7A, 0x3A),
        warning: Color32::from_rgb(0x94, 0x61, 0x00),
        danger: Color32::from_rgb(0xB3, 0x26, 0x1E),
        info: Color32::from_rgb(0x24, 0x5F, 0x9E),
        on_accent: Color32::WHITE,
    }
}

pub fn dark() -> Palette {
    Palette {
        bg: Color32::from_rgb(0x14, 0x15, 0x14),
        surface: Color32::from_rgb(0x1D, 0x1F, 0x1D),
        surface_muted: Color32::from_rgb(0x24, 0x26, 0x24),
        border: Color32::from_rgb(0x34, 0x37, 0x32),
        text: Color32::from_rgb(0xF0, 0xF2, 0xEE),
        text_muted: Color32::from_rgb(0xA6, 0xAA, 0xA2),
        accent: Color32::from_rgb(0x6F, 0xD8, 0xB8),
        accent_soft: Color32::from_rgb(0x17, 0x3B, 0x32),
        success: Color32::from_rgb(0x7E, 0xD9, 0x9A),
        warning: Color32::from_rgb(0xF2, 0xC4, 0x6B),
        danger: Color32::from_rgb(0xFF, 0x8A, 0x80),
        info: Color32::from_rgb(0x8D, 0xBE, 0xF5),
        on_accent: Color32::from_rgb(0x0D, 0x1B, 0x16),
    }
}

pub fn palette(ctx: &egui::Context) -> Palette {
    if ctx.theme() == Theme::Dark {
        dark()
    } else {
        light()
    }
}

pub fn category_color(palette: Palette, category: IssueCategory) -> Color32 {
    match category {
        IssueCategory::Attention => palette.danger,
        IssueCategory::NotConverted => palette.warning,
        IssueCategory::Review => palette.warning,
        IssueCategory::AllGood => palette.success,
    }
}

pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        FONT_NAME.to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/Inter-VariableFont_opsz,wght.ttf"
        ))),
    );
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .insert(0, FONT_NAME.to_owned());
    }
    ctx.set_fonts(fonts);
}

pub fn apply(ctx: &egui::Context, choice: ThemeChoice) {
    let preference = match choice {
        ThemeChoice::System => ThemePreference::System,
        ThemeChoice::Light => ThemePreference::Light,
        ThemeChoice::Dark => ThemePreference::Dark,
    };
    for (egui_theme, palette) in [(Theme::Light, light()), (Theme::Dark, dark())] {
        ctx.set_style_of(egui_theme, build_style(ctx, egui_theme, palette));
    }
    ctx.set_theme(preference);
}

fn build_style(ctx: &egui::Context, theme: Theme, palette: Palette) -> egui::Style {
    let mut style = (*ctx.style_of(theme)).clone();
    style.visuals = if theme == Theme::Dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    style.visuals.panel_fill = palette.bg;
    style.visuals.window_fill = palette.surface;
    style.visuals.faint_bg_color = palette.surface_muted;
    style.visuals.extreme_bg_color = palette.surface_muted;
    style.visuals.code_bg_color = palette.surface_muted;
    style.visuals.override_text_color = Some(palette.text);
    style.visuals.hyperlink_color = palette.accent;
    style.visuals.warn_fg_color = palette.warning;
    style.visuals.error_fg_color = palette.danger;
    style.visuals.window_stroke = Stroke::new(1.0, palette.border);
    style.visuals.window_corner_radius = CornerRadius::same(RADIUS_CARD);
    style.visuals.menu_corner_radius = CornerRadius::same(RADIUS_CARD);
    style.visuals.selection.bg_fill = palette.accent_soft;
    style.visuals.selection.stroke = Stroke::new(1.0, palette.accent);
    style.visuals.widgets.noninteractive.bg_fill = palette.surface;
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, palette.border);
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, palette.text);
    style.visuals.widgets.inactive.bg_fill = palette.surface_muted;
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, palette.border);
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, palette.text);
    style.visuals.widgets.hovered.bg_fill = palette.accent_soft;
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, palette.accent);
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, palette.text);
    style.visuals.widgets.active.bg_fill = palette.accent_soft;
    style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, palette.accent);
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, palette.text);
    for widget in [
        &mut style.visuals.widgets.noninteractive,
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
        &mut style.visuals.widgets.open,
    ] {
        widget.corner_radius = CornerRadius::same(RADIUS_INPUT);
    }
    style.spacing.item_spacing = egui::vec2(SPACE_2, SPACE_2);
    style.spacing.button_padding = egui::vec2(SPACE_3, SPACE_2);
    style.spacing.interact_size.y = 34.0;
    style.spacing.indent = 18.0;
    style
        .text_styles
        .insert(TextStyle::Body, FontId::new(14.0, FontFamily::Proportional));
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(14.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(20.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(12.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Monospace,
        FontId::new(12.5, FontFamily::Monospace),
    );
    style
}

pub fn card(ui: &egui::Ui) -> egui::Frame {
    let palette = palette(ui.ctx());
    egui::Frame::default()
        .fill(palette.surface)
        .stroke(Stroke::new(1.0, palette.border))
        .inner_margin(SPACE_4)
        .corner_radius(CornerRadius::same(RADIUS_CARD))
}

pub fn raised(ui: &egui::Ui) -> egui::Frame {
    let palette = palette(ui.ctx());
    egui::Frame::default()
        .fill(palette.surface_muted)
        .stroke(Stroke::new(1.0, palette.border))
        .inner_margin(SPACE_2)
        .corner_radius(CornerRadius::same(RADIUS_INPUT))
}
