use std::sync::Arc;

use eframe::egui;

/// Deepest background (text input wells)
pub const CLEAR: egui::Color32 = egui::Color32::from_rgb(0x11, 0x13, 0x16);
/// Chrome panels (sidebar, toolbar, status bar)
pub const CAPTION: egui::Color32 = egui::Color32::from_rgb(0x18, 0x1A, 0x1D);
/// Main content area
pub const BACKGROUND: egui::Color32 = egui::Color32::from_rgb(0x1E, 0x21, 0x25);
/// Raised surface (active tab, cards)
pub const SURFACE: egui::Color32 = egui::Color32::from_rgb(0x26, 0x29, 0x2E);
pub const HOVER: egui::Color32 = egui::Color32::from_rgb(0x2F, 0x33, 0x39);
pub const BORDER: egui::Color32 = egui::Color32::from_rgb(0x2A, 0x2E, 0x33);
pub const TEXT: egui::Color32 = egui::Color32::from_rgb(0xE5, 0xE7, 0xEA);
pub const SECONDARY: egui::Color32 = egui::Color32::from_rgb(0x8B, 0x90, 0x97);
pub const MUTED: egui::Color32 = egui::Color32::from_rgb(0x5A, 0x60, 0x6B);
pub const SELECTION: egui::Color32 = egui::Color32::from_rgb(0x1F, 0xA6, 0xE5);
pub const WARNING: egui::Color32 = egui::Color32::from_rgb(0xF4, 0x4B, 0x4B);
pub const SUCCESS: egui::Color32 = egui::Color32::from_rgb(0x34, 0xD3, 0x99);
/// Search match highlight
pub const MATCH: egui::Color32 = egui::Color32::from_rgb(0xFA, 0xCC, 0x15);
pub const FOLDER_YELLOW: egui::Color32 = egui::Color32::from_rgb(0xF2, 0xC4, 0x4E);

pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    visuals.window_fill = CAPTION;
    visuals.panel_fill = BACKGROUND;
    visuals.faint_bg_color = SURFACE;
    visuals.extreme_bg_color = CLEAR;

    visuals.widgets.noninteractive.bg_fill = SURFACE;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, SECONDARY);
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::NONE;

    visuals.widgets.inactive.bg_fill = SURFACE;
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, SECONDARY);
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, BORDER);

    visuals.widgets.hovered.bg_fill = HOVER;
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, TEXT);
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, SELECTION);

    visuals.widgets.active.bg_fill = SELECTION;
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, TEXT);

    visuals.widgets.open.bg_fill = SURFACE;
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, TEXT);

    visuals.selection.bg_fill = SELECTION.gamma_multiply(0.25);
    visuals.selection.stroke = egui::Stroke::new(1.0, SELECTION);

    visuals.window_corner_radius = egui::CornerRadius::same(12);
    visuals.menu_corner_radius = egui::CornerRadius::same(10);
    visuals.window_stroke = egui::Stroke::new(1.0, BORDER);

    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 2],
        blur: 16,
        spread: 4,
        color: egui::Color32::from_black_alpha(60),
    };
    visuals.popup_shadow = egui::epaint::Shadow {
        offset: [0, 4],
        blur: 12,
        spread: 2,
        color: egui::Color32::from_black_alpha(80),
    };

    visuals.striped = false;
    visuals.indent_has_left_vline = false;
    visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);

    ctx.set_visuals(visuals);
}

pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    if let Ok(data) = std::fs::read(r"C:\Windows\Fonts\segoeui.ttf") {
        fonts.font_data.insert(
            "system-ui".to_owned(),
            Arc::new(egui::FontData::from_owned(data)),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "system-ui".to_owned());
    }

    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

    ctx.set_fonts(fonts);

    // ── Text styles (File Pilot density) ───────────────
    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(14.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(14.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::new(12.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(16.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::new(13.0, egui::FontFamily::Monospace),
    );
    // ── Spacing (8 px grid) ────────────────────────────
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    style.spacing.indent = 16.0;
    style.spacing.interact_size = egui::vec2(32.0, 32.0);
    style.spacing.scroll = egui::style::ScrollStyle::floating();

    ctx.set_style(style);
}

pub fn dock_style(egui_style: &egui::Style) -> egui_dock::Style {
    let mut style = egui_dock::Style::from_egui(egui_style);

    style.tab_bar.bg_fill = CAPTION;
    style.tab_bar.hline_color = BORDER;
    style.tab_bar.height = 32.0;

    style.tab.active.bg_fill = BACKGROUND;
    style.tab.active.text_color = TEXT;
    style.tab.active.outline_color = BORDER;

    style.tab.inactive.bg_fill = CAPTION;
    style.tab.inactive.text_color = SECONDARY;
    style.tab.inactive.outline_color = BORDER;

    style.tab.focused.bg_fill = BACKGROUND;
    style.tab.focused.text_color = TEXT;
    style.tab.focused.outline_color = SELECTION;

    style.tab.hovered.bg_fill = HOVER;
    style.tab.hovered.text_color = TEXT;
    style.tab.hovered.outline_color = SELECTION;

    style.tab.tab_body.bg_fill = BACKGROUND;
    style.tab.tab_body.inner_margin = egui::Margin::same(0);
    style.tab.tab_body.stroke = egui::Stroke::new(1.0, BORDER);

    style.separator.width = 1.0;
    style.separator.extra_interact_width = 4.0;
    style.separator.color_idle = BORDER;
    style.separator.color_hovered = SELECTION;
    style.separator.color_dragged = SELECTION;

    style.overlay.selection_color = SELECTION.gamma_multiply(0.3);
    style.overlay.overlay_type = egui_dock::OverlayType::HighlightedAreas;

    style.buttons.close_tab_color = MUTED;
    style.buttons.close_tab_active_color = WARNING;
    style.buttons.add_tab_color = MUTED;
    style.buttons.add_tab_active_color = SELECTION;

    style
}
