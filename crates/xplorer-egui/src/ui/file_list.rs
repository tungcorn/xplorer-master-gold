use eframe::egui;

use crate::state::AppState;
use crate::theme;

pub fn show(ctx: &egui::Context, state: &mut AppState) {
    egui::CentralPanel::default()
        .frame(egui::Frame::central_panel(&ctx.style()).fill(theme::BACKGROUND))
        .show(ctx, |ui| {
            let tab = &state.tabs[state.active_tab];
            if tab.loading {
                ui.centered_and_justified(|ui| {
                    ui.spinner();
                });
            } else if let Some(err) = &tab.error {
                ui.colored_label(theme::WARNING, format!("Error: {}", err));
            } else {
                ui.label(format!(
                    "File list: {} entries (table in Task 7)",
                    tab.entries.len()
                ));
            }
        });
}
