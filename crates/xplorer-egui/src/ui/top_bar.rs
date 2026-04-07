use std::path::Path;

use eframe::egui;

use crate::icons;
use crate::state::{OmnibarMode, Tab};
use crate::theme;
use crate::ui::dock_viewer::ViewerAction;

pub fn show_for_tab(
    ui: &mut egui::Ui,
    tab: &mut Tab,
    _state: &crate::state::AppState,
    actions: &mut Vec<ViewerAction>,
) {
    ui.horizontal(|ui| {
        ui.set_min_height(32.0);
        show_nav_buttons(ui, tab, actions);
        ui.add_space(8.0);
        show_omnibar(ui, tab, actions);
    });
}

fn show_nav_buttons(ui: &mut egui::Ui, tab: &Tab, actions: &mut Vec<ViewerAction>) {
    let can_back = tab.history_index > 0;
    let can_forward = tab.history_index + 1 < tab.history.len();

    ui.spacing_mut().item_spacing.x = 2.0;

    let back_text = egui::RichText::new(icons::ARROW_LEFT).size(16.0);
    if ui
        .add_enabled(can_back, egui::Button::new(back_text).frame(false))
        .on_hover_text("Back  Alt+\u{2190}")
        .clicked()
    {
        actions.push(ViewerAction::GoBack);
    }

    let fwd_text = egui::RichText::new(icons::ARROW_RIGHT).size(16.0);
    if ui
        .add_enabled(can_forward, egui::Button::new(fwd_text).frame(false))
        .on_hover_text("Forward  Alt+\u{2192}")
        .clicked()
    {
        actions.push(ViewerAction::GoForward);
    }

    let up_text = egui::RichText::new(icons::ARROW_UP).size(16.0);
    if ui
        .button(up_text)
        .on_hover_text("Up  Alt+\u{2191}")
        .clicked()
    {
        actions.push(ViewerAction::GoUp);
    }
}

fn show_omnibar(ui: &mut egui::Ui, tab: &mut Tab, actions: &mut Vec<ViewerAction>) {
    let mode = tab.omnibar_mode;

    let bar_rect = ui.available_rect_before_wrap();

    let bar_resp = ui
        .allocate_ui(egui::vec2(bar_rect.width(), 28.0), |ui| {
            egui::Frame::new()
                .fill(theme::BACKGROUND.gamma_multiply(0.6))
                .corner_radius(4.0)
                .stroke(egui::Stroke::new(
                    1.0,
                    if mode == OmnibarMode::Breadcrumb {
                        theme::BORDER
                    } else {
                        theme::SELECTION
                    },
                ))
                .inner_margin(egui::Margin::symmetric(8, 2))
                .show(ui, |ui| {
                    ui.horizontal_centered(|ui| match mode {
                        OmnibarMode::Breadcrumb => show_breadcrumb_mode(ui, tab, actions),
                        OmnibarMode::GoTo => show_goto_mode(ui, tab, actions),
                        OmnibarMode::Filter => show_filter_mode(ui, tab),
                    });
                });
        })
        .response;

    if mode == OmnibarMode::Breadcrumb && bar_resp.clicked() {
        tab.omnibar_mode = OmnibarMode::GoTo;
        tab.address_bar_text = tab.path.clone();
    }
}

fn mode_pill(ui: &mut egui::Ui, label: &str, color: egui::Color32) {
    egui::Frame::new()
        .fill(color.gamma_multiply(0.2))
        .corner_radius(3.0)
        .inner_margin(egui::Margin::symmetric(6, 1))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(label).color(color).size(10.0).strong());
        });
}

fn show_breadcrumb_mode(ui: &mut egui::Ui, tab: &mut Tab, actions: &mut Vec<ViewerAction>) {
    let path_str = tab.path.clone();
    let path = Path::new(&path_str);

    let mut accumulated = std::path::PathBuf::new();
    let mut nav_target: Option<String> = None;

    ui.spacing_mut().item_spacing.x = 2.0;
    let components: Vec<_> = path.components().collect();
    for (i, component) in components.iter().enumerate() {
        accumulated.push(component);
        let segment = component.as_os_str().to_string_lossy();

        let btn = ui.selectable_label(
            false,
            egui::RichText::new(segment.as_ref())
                .color(theme::SECONDARY)
                .size(13.0),
        );
        if btn.clicked() {
            nav_target = Some(accumulated.to_string_lossy().to_string());
        }

        if i < components.len() - 1 {
            ui.label(
                egui::RichText::new(icons::BREADCRUMB_SEP)
                    .color(theme::MUTED)
                    .size(11.0),
            );
        }
    }

    if !tab.filter_text.is_empty() {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let clear_btn = ui.add(
                egui::Button::new(
                    egui::RichText::new(icons::CLOSE)
                        .color(theme::MUTED)
                        .size(10.0),
                )
                .frame(false),
            );
            if clear_btn.clicked() {
                tab.filter_text.clear();
                tab.filter_dirty = true;
            }
            mode_pill(
                ui,
                &format!("{} {}", icons::FILTER, &tab.filter_text),
                theme::SELECTION,
            );
        });
    }

    if let Some(target) = nav_target {
        actions.push(ViewerAction::Navigate(target));
    }
}

fn show_goto_mode(ui: &mut egui::Ui, tab: &mut Tab, actions: &mut Vec<ViewerAction>) {
    mode_pill(ui, "Navigate", theme::SELECTION);
    ui.add_space(4.0);

    ui.label(
        egui::RichText::new(icons::FOLDER)
            .color(theme::SELECTION)
            .size(14.0),
    );

    let response = ui.add(
        egui::TextEdit::singleline(&mut tab.address_bar_text)
            .desired_width(ui.available_width())
            .font(egui::TextStyle::Body),
    );

    if !response.has_focus() {
        response.request_focus();
    }

    let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
    let escape_pressed = ui.input(|i| i.key_pressed(egui::Key::Escape));

    if enter_pressed {
        let target = tab.address_bar_text.trim().to_string();
        if !target.is_empty() {
            actions.push(ViewerAction::Navigate(target));
        }
        tab.omnibar_mode = OmnibarMode::Breadcrumb;
    } else if escape_pressed || response.lost_focus() {
        tab.omnibar_mode = OmnibarMode::Breadcrumb;
    }
}

fn show_filter_mode(ui: &mut egui::Ui, tab: &mut Tab) {
    mode_pill(ui, "Filter", theme::SELECTION);
    ui.add_space(4.0);

    ui.label(
        egui::RichText::new(icons::FILTER)
            .color(theme::SELECTION)
            .size(14.0),
    );

    let response = ui.add(
        egui::TextEdit::singleline(&mut tab.filter_text)
            .hint_text("Type to filter files...")
            .desired_width(ui.available_width()),
    );

    if !response.has_focus() {
        response.request_focus();
    }

    if !tab.filter_text.is_empty() {
        let count = tab.filtered_cache.len();
        ui.label(
            egui::RichText::new(format!("{} matches", count))
                .color(theme::MUTED)
                .size(11.0),
        );
    }

    let escape_pressed = ui.input(|i| i.key_pressed(egui::Key::Escape));
    if escape_pressed {
        tab.omnibar_mode = OmnibarMode::Breadcrumb;
    }
}
