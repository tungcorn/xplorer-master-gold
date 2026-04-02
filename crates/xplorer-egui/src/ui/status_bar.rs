use eframe::egui;

use crate::state::AppState;
use crate::theme;

pub fn show(ctx: &egui::Context, state: &AppState) {
    egui::TopBottomPanel::bottom("status_bar")
        .frame(egui::Frame::side_top_panel(&ctx.style()).fill(theme::CAPTION))
        .exact_height(24.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let tab = state.active_tab();
                let total = tab.entries.len();
                let selected = tab.selected_indices.len();
                ui.label(format!("{} items", total));
                if selected > 0 {
                    ui.separator();
                    ui.label(format!("{} selected", selected));
                }
            });
        });
}
