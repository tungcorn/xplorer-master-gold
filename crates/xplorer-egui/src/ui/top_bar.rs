use std::path::Path;

use eframe::egui;

use crate::icons;
use crate::state::AppState;
use crate::theme;
use crate::ui::tab_bar::{self, TabAction};

pub fn show(ctx: &egui::Context, state: &mut AppState) {
    egui::TopBottomPanel::top("top_bar")
        .frame(
            egui::Frame::NONE
                .fill(theme::CAPTION)
                .inner_margin(egui::Margin::symmetric(8, 4)),
        )
        .show(ctx, |ui| {
            let tab_action = tab_bar::show(ui, state);
            match tab_action {
                TabAction::Switch(i) => state.switch_tab(i),
                TabAction::Close(i) => {
                    state.close_tab(i);
                }
                TabAction::New => state.new_tab(),
                TabAction::None => {}
            }

            ui.add_space(2.0);
            let sep_rect = ui.available_rect_before_wrap();
            ui.painter().line_segment(
                [
                    egui::pos2(sep_rect.left(), sep_rect.top()),
                    egui::pos2(sep_rect.right(), sep_rect.top()),
                ],
                egui::Stroke::new(1.0, theme::BORDER),
            );
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.set_min_height(32.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    show_filter_input(ui, state);
                    ui.add_space(8.0);
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        show_nav_buttons(ui, state);
                        ui.add_space(8.0);
                        show_path_area(ui, state);
                    });
                });
            });
        });
}

fn show_nav_buttons(ui: &mut egui::Ui, state: &mut AppState) {
    let can_back = state.active_tab().history_index > 0;
    let can_forward = state.active_tab().history_index + 1 < state.active_tab().history.len();

    ui.spacing_mut().item_spacing.x = 2.0;

    let back_text = egui::RichText::new(icons::ARROW_LEFT).size(16.0);
    if ui
        .add_enabled(can_back, egui::Button::new(back_text).frame(false))
        .on_hover_text("Back  Alt+←")
        .clicked()
    {
        state.go_back_nav();
    }

    let fwd_text = egui::RichText::new(icons::ARROW_RIGHT).size(16.0);
    if ui
        .add_enabled(can_forward, egui::Button::new(fwd_text).frame(false))
        .on_hover_text("Forward  Alt+→")
        .clicked()
    {
        state.go_forward_nav();
    }

    let up_text = egui::RichText::new(icons::ARROW_UP).size(16.0);
    if ui.button(up_text).on_hover_text("Up  Alt+↑").clicked() {
        state.go_up();
    }
}

fn show_path_area(ui: &mut egui::Ui, state: &mut AppState) {
    if state.editing_address_bar {
        show_address_bar(ui, state);
    } else {
        show_breadcrumb(ui, state);
    }
}

fn show_breadcrumb(ui: &mut egui::Ui, state: &mut AppState) {
    let path_str = state.active_tab().path.clone();
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
        state.editing_address_bar = true;
        state.address_bar_text = state.active_tab().path.clone();
    }

    if let Some(target) = nav_target {
        state.navigate_to(&target);
    }
}

fn show_address_bar(ui: &mut egui::Ui, state: &mut AppState) {
    let response = ui.add(
        egui::TextEdit::singleline(&mut state.address_bar_text)
            .desired_width(ui.available_width())
            .font(egui::TextStyle::Body),
    );

    if !response.has_focus() {
        response.request_focus();
    }

    let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
    let escape_pressed = ui.input(|i| i.key_pressed(egui::Key::Escape));

    if enter_pressed {
        let target = state.address_bar_text.trim().to_string();
        if !target.is_empty() {
            state.navigate_to(&target);
        }
        state.editing_address_bar = false;
    } else if escape_pressed || response.lost_focus() {
        state.editing_address_bar = false;
    }
}

fn show_filter_input(ui: &mut egui::Ui, state: &mut AppState) {
    let idx = state.active_tab;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;

        ui.label(
            egui::RichText::new(icons::FILTER)
                .color(theme::MUTED)
                .size(14.0),
        );

        let response = ui.add(
            egui::TextEdit::singleline(&mut state.tabs[idx].filter_text)
                .hint_text("Filter")
                .desired_width(160.0),
        );

        if !state.tabs[idx].filter_text.is_empty() {
            let clear_text = egui::RichText::new(icons::CLOSE)
                .color(theme::MUTED)
                .size(12.0);
            if ui.add(egui::Button::new(clear_text).frame(false)).clicked() {
                state.tabs[idx].filter_text.clear();
            }
        }

        if state.focus_filter {
            response.request_focus();
            state.focus_filter = false;
        }

        if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            state.tabs[idx].filter_text.clear();
            response.surrender_focus();
        }
    });
}
