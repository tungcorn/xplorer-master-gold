use eframe::egui;

use crate::state::AppState;
use crate::theme;

pub enum TabAction {
    None,
    Switch(usize),
    Close(usize),
    New,
}

pub fn show(ui: &mut egui::Ui, state: &mut AppState) -> TabAction {
    let mut action = TabAction::None;

    ui.horizontal(|ui| {
        let tab_count = state.tabs.len();
        for i in 0..tab_count {
            let is_active = i == state.active_tab;
            let bg = if is_active {
                theme::SURFACE
            } else {
                theme::CAPTION
            };
            let text_color = if is_active { theme::TEXT } else { theme::MUTED };

            let frame = egui::Frame::NONE
                .fill(bg)
                .corner_radius(egui::CornerRadius {
                    nw: 6,
                    ne: 6,
                    sw: 0,
                    se: 0,
                })
                .inner_margin(egui::Margin::symmetric(10, 4));

            frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    let label = egui::RichText::new(&state.tabs[i].display_name)
                        .color(text_color)
                        .size(13.0);
                    if ui.selectable_label(false, label).clicked() {
                        action = TabAction::Switch(i);
                    }
                    if tab_count > 1 {
                        let close_label = egui::RichText::new("×").color(theme::MUTED).size(11.0);
                        if ui.small_button(close_label).clicked() {
                            action = TabAction::Close(i);
                        }
                    }
                });
            });

            ui.add_space(2.0);
        }

        if ui.small_button("+").clicked() {
            action = TabAction::New;
        }
    });

    action
}
