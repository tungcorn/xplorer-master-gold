use std::path::Path;

use eframe::egui;

use crate::icons;
use crate::state::{AppState, Tab};
use crate::theme;
use crate::ui::dock_viewer::ViewerAction;

pub fn show_for_tab(
    ui: &mut egui::Ui,
    tab: &mut Tab,
    state: &AppState,
    actions: &mut Vec<ViewerAction>,
) {
    ui.horizontal(|ui| {
        ui.set_min_height(32.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            show_filter_input(ui, tab, state);
            ui.add_space(8.0);
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                show_nav_buttons(ui, tab, actions);
                ui.add_space(8.0);
                show_path_area(ui, tab, actions);
            });
        });
    });
}

fn show_nav_buttons(ui: &mut egui::Ui, tab: &Tab, actions: &mut Vec<ViewerAction>) {
    let can_back = tab.history_index > 0;
    let can_forward = tab.history_index + 1 < tab.history.len();

    ui.spacing_mut().item_spacing.x = 2.0;

    let back_text = egui::RichText::new(icons::ARROW_LEFT).size(16.0);
    if ui
        .add_enabled(can_back, egui::Button::new(back_text).frame(false))
        .on_hover_text("Back  Alt+←")
        .clicked()
    {
        actions.push(ViewerAction::GoBack);
    }

    let fwd_text = egui::RichText::new(icons::ARROW_RIGHT).size(16.0);
    if ui
        .add_enabled(can_forward, egui::Button::new(fwd_text).frame(false))
        .on_hover_text("Forward  Alt+→")
        .clicked()
    {
        actions.push(ViewerAction::GoForward);
    }

    let up_text = egui::RichText::new(icons::ARROW_UP).size(16.0);
    if ui.button(up_text).on_hover_text("Up  Alt+↑").clicked() {
        actions.push(ViewerAction::GoUp);
    }
}

fn show_path_area(ui: &mut egui::Ui, tab: &mut Tab, actions: &mut Vec<ViewerAction>) {
    if tab.editing_address_bar {
        show_address_bar(ui, tab, actions);
    } else {
        show_breadcrumb(ui, tab, actions);
    }
}

fn show_breadcrumb(ui: &mut egui::Ui, tab: &mut Tab, actions: &mut Vec<ViewerAction>) {
    let path_str = tab.path.clone();
    let path = Path::new(&path_str);

    let mut accumulated = std::path::PathBuf::new();
    let mut nav_target: Option<String> = None;
    let mut clicked_breadcrumb_bg = false;

    let response = ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        let components: Vec<_> = path.components().collect();
        for (i, component) in components.iter().enumerate() {
            accumulated.push(component);
            let segment = component.as_os_str().to_string_lossy();

            let btn = ui.selectable_label(
                false,
                egui::RichText::new(segment.as_ref())
                    .color(theme::SECONDARY)
                    .size(14.0),
            );
            if btn.clicked() {
                nav_target = Some(accumulated.to_string_lossy().to_string());
            }

            if i < components.len() - 1 {
                ui.label(
                    egui::RichText::new(icons::BREADCRUMB_SEP)
                        .color(theme::MUTED)
                        .size(12.0),
                );
            }
        }
    });

    if response.response.clicked() && nav_target.is_none() {
        clicked_breadcrumb_bg = true;
    }

    if clicked_breadcrumb_bg {
        tab.editing_address_bar = true;
        tab.address_bar_text = tab.path.clone();
    }

    if let Some(target) = nav_target {
        actions.push(ViewerAction::Navigate(target));
    }
}

fn show_address_bar(ui: &mut egui::Ui, tab: &mut Tab, actions: &mut Vec<ViewerAction>) {
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
        tab.editing_address_bar = false;
    } else if escape_pressed || response.lost_focus() {
        tab.editing_address_bar = false;
    }
}

fn show_filter_input(ui: &mut egui::Ui, tab: &mut Tab, state: &AppState) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;

        ui.label(
            egui::RichText::new(icons::FILTER)
                .color(theme::MUTED)
                .size(14.0),
        );

        let response = ui.add(
            egui::TextEdit::singleline(&mut tab.filter_text)
                .hint_text("Filter")
                .desired_width(160.0),
        );

        if !tab.filter_text.is_empty() {
            let clear_text = egui::RichText::new(icons::CLOSE)
                .color(theme::MUTED)
                .size(12.0);
            if ui.add(egui::Button::new(clear_text).frame(false)).clicked() {
                tab.filter_text.clear();
            }
        }

        if state.focus_filter {
            response.request_focus();
        }

        if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            tab.filter_text.clear();
            response.surrender_focus();
        }
    });
}
