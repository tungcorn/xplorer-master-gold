use eframe::egui;

use crate::icons;
use crate::state::AppState;
use crate::theme;

pub enum TabAction {
    None,
    Switch(usize),
    Close(usize),
    New,
}

const TAB_MAX_WIDTH: f32 = 180.0;

pub fn show(ui: &mut egui::Ui, state: &mut AppState) -> TabAction {
    let mut action = TabAction::None;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;

        let tab_count = state.tabs.len();
        for i in 0..tab_count {
            let is_active = i == state.active_tab;
            let bg = if is_active {
                theme::BACKGROUND
            } else {
                theme::CAPTION
            };
            let text_color = if is_active {
                theme::TEXT
            } else {
                theme::SECONDARY
            };

            let frame = egui::Frame::NONE
                .fill(bg)
                .corner_radius(egui::CornerRadius {
                    nw: 8,
                    ne: 8,
                    sw: 0,
                    se: 0,
                })
                .inner_margin(egui::Margin::symmetric(12, 6));

            frame.show(ui, |ui| {
                ui.set_max_width(TAB_MAX_WIDTH);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;

                    let icon = egui::RichText::new(icons::FOLDER)
                        .color(theme::FOLDER_YELLOW)
                        .size(14.0);
                    ui.label(icon);

                    let display = truncate_tab_name(&state.tabs[i].display_name, 20);
                    let label = egui::RichText::new(display).color(text_color).size(14.0);
                    let tab_resp = ui.selectable_label(false, label);
                    if tab_resp.clicked() {
                        action = TabAction::Switch(i);
                    }

                    if tab_count > 1 {
                        let close_text = egui::RichText::new(icons::CLOSE)
                            .color(theme::MUTED)
                            .size(14.0);
                        let close_btn = ui.add(egui::Button::new(close_text).frame(false));
                        if close_btn.clicked() {
                            action = TabAction::Close(i);
                        }
                    }
                });
            });
        }

        let plus_text = egui::RichText::new(icons::PLUS)
            .color(theme::SECONDARY)
            .size(16.0);
        if ui
            .add(egui::Button::new(plus_text).frame(false))
            .on_hover_text("New tab  Ctrl+T")
            .clicked()
        {
            action = TabAction::New;
        }
    });

    action
}

fn truncate_tab_name(name: &str, max_chars: usize) -> String {
    if name.chars().count() > max_chars {
        let truncated: String = name.chars().take(max_chars.saturating_sub(1)).collect();
        format!("{}…", truncated)
    } else {
        name.to_string()
    }
}
