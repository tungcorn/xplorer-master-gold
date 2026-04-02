use eframe::egui;

use crate::state::AppState;
use crate::theme;

pub fn show(ctx: &egui::Context, state: &mut AppState) {
    egui::SidePanel::left("sidebar")
        .default_width(220.0)
        .resizable(true)
        .frame(egui::Frame::side_top_panel(&ctx.style()).fill(theme::CAPTION))
        .show(ctx, |ui| {
            ui.heading("Drives");
            ui.separator();
            for drive in &state.drives {
                ui.label(format!("{}  {}", drive.mount_point, drive.name));
            }

            ui.add_space(16.0);
            ui.heading("Favorites");
            ui.separator();
            if state.bookmarks.is_empty() {
                ui.label("(no bookmarks)");
            } else {
                for bm in &state.bookmarks {
                    ui.label(&bm.name);
                }
            }
        });
}
