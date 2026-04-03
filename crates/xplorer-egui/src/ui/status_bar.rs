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
                let selected_count = tab.selected_indices.len();

                ui.label(
                    egui::RichText::new(format!("{} items", total))
                        .color(theme::SECONDARY)
                        .size(11.0),
                );

                if selected_count > 0 {
                    let selected_size: u64 = tab
                        .selected_indices
                        .iter()
                        .filter_map(|&i| tab.entries.get(i))
                        .filter(|e| !e.is_dir)
                        .map(|e| e.size)
                        .sum();

                    ui.separator();
                    ui.label(
                        egui::RichText::new(format!(
                            "{} selected ({})",
                            selected_count,
                            format_size(selected_size)
                        ))
                        .color(theme::TEXT)
                        .size(11.0),
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(&tab.path)
                            .color(theme::MUTED)
                            .size(11.0),
                    );
                });
            });
        });
}

fn format_size(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    if bytes < 1024 {
        return format!("{} B", bytes);
    }
    if bytes < 1024 * 1024 {
        return format!("{:.1} KB", bytes as f64 / 1024.0);
    }
    if bytes < 1024 * 1024 * 1024 {
        return format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0));
    }
    format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
}
