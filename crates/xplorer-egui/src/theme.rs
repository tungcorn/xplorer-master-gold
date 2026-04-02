use eframe::egui;

pub const CLEAR: egui::Color32 = egui::Color32::from_rgb(0x19, 0x1A, 0x21);
pub const CAPTION: egui::Color32 = egui::Color32::from_rgb(0x21, 0x22, 0x2C);
pub const BACKGROUND: egui::Color32 = egui::Color32::from_rgb(0x28, 0x2A, 0x36);
pub const SURFACE: egui::Color32 = egui::Color32::from_rgb(0x34, 0x37, 0x46);
pub const FOREGROUND: egui::Color32 = egui::Color32::from_rgb(0x44, 0x47, 0x5A);
pub const TEXT: egui::Color32 = egui::Color32::from_rgb(0xF8, 0xF8, 0xF2);
pub const SECONDARY: egui::Color32 = egui::Color32::from_rgb(0x7B, 0x8D, 0xB8);
pub const MUTED: egui::Color32 = egui::Color32::from_rgb(0x62, 0x72, 0xA4);
pub const SELECTION: egui::Color32 = egui::Color32::from_rgb(0xBD, 0x93, 0xF9);
pub const WARNING: egui::Color32 = egui::Color32::from_rgb(0xFF, 0x55, 0x55);
pub const SUCCESS: egui::Color32 = egui::Color32::from_rgb(0x50, 0xFA, 0x7B);
pub const MATCH: egui::Color32 = egui::Color32::from_rgb(0xF1, 0xFA, 0x8C);
pub const ALT_ROW: egui::Color32 = egui::Color32::from_rgb(0x2D, 0x2F, 0x3B);

pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    visuals.window_fill = CAPTION;
    visuals.panel_fill = BACKGROUND;
    visuals.faint_bg_color = ALT_ROW;
    visuals.extreme_bg_color = CLEAR;

    visuals.widgets.noninteractive.bg_fill = SURFACE;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, TEXT);
    visuals.widgets.inactive.bg_fill = SURFACE;
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, SECONDARY);
    visuals.widgets.hovered.bg_fill = FOREGROUND;
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, TEXT);
    visuals.widgets.active.bg_fill = SELECTION;
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, TEXT);

    visuals.selection.bg_fill = SELECTION.gamma_multiply(0.3);
    visuals.selection.stroke = egui::Stroke::new(1.0, SELECTION);

    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(0.0, egui::Color32::TRANSPARENT);

    visuals.window_corner_radius = egui::CornerRadius::same(8);
    visuals.menu_corner_radius = egui::CornerRadius::same(12);

    visuals.popup_shadow = egui::epaint::Shadow {
        offset: [0, 4],
        blur: 12,
        spread: 0,
        color: egui::Color32::from_black_alpha(80),
    };

    visuals.striped = true;

    ctx.set_visuals(visuals);
}

pub fn setup_fonts(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(13.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(13.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::new(11.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(16.0, egui::FontFamily::Proportional),
    );
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.spacing.button_padding = egui::vec2(6.0, 3.0);
    ctx.set_style(style);
}
