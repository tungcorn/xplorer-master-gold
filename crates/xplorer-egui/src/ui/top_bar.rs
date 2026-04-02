use eframe::egui;

use crate::state::AppState;
use crate::theme;

pub fn show(ctx: &egui::Context, state: &mut AppState) {
    egui::TopBottomPanel::top("top_bar")
        .frame(egui::Frame::side_top_panel(&ctx.style()).fill(theme::CAPTION))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Tab 1");
            });
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("◀").clicked() {}
                if ui.button("▶").clicked() {}
                if ui.button("▲").clicked() {}
                ui.separator();
                let tab = &state.tabs[state.active_tab];
                ui.label(&tab.path);
            });
        });
}
