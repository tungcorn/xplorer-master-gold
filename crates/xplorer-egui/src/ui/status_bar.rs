use eframe::egui;

use crate::state::AppState;
use crate::theme;

pub fn show(ctx: &egui::Context, state: &AppState) {
    egui::TopBottomPanel::bottom("status_bar")
        .frame(
            egui::Frame::NONE
                .fill(theme::CAPTION)
                .inner_margin(egui::Margin::symmetric(12, 0)),
        )
        .min_height(28.0)
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                let tab = state.active_tab();
                let total = tab.entries.len();
                let filtered_count = if tab.filter_text.is_empty() {
                    total
                } else {
                    tab.entries
                        .iter()
                        .filter(|e| {
                            e.name
                                .to_lowercase()
                                .contains(&tab.filter_text.to_lowercase())
                        })
                        .count()
                };
                let selected_count = tab.selected_indices.len();

                if !tab.filter_text.is_empty() {
                    ui.label(
                        egui::RichText::new(format!("{} of {} items", filtered_count, total))
                            .color(theme::SECONDARY)
                            .size(12.0),
                    );
                } else {
                    ui.label(
                        egui::RichText::new(format!("{} items", total))
                            .color(theme::SECONDARY)
                            .size(12.0),
                    );
                }

                if selected_count > 0 {
                    let selected_size: u64 = tab
                        .selected_indices
                        .iter()
                        .filter_map(|&i| tab.entries.get(i))
                        .filter(|e| !e.is_dir)
                        .map(|e| e.size)
                        .sum();

                    ui.label(egui::RichText::new("·").color(theme::MUTED).size(12.0));
                    ui.label(
                        egui::RichText::new(format!(
                            "{} selected ({})",
                            selected_count,
                            format_size(selected_size)
                        ))
                        .color(theme::TEXT)
                        .size(12.0),
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(&tab.path)
                                .color(theme::MUTED)
                                .size(12.0),
                        )
                        .truncate(),
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
