use eframe::egui;
use egui_phosphor::regular;

use crate::state::Tab;
use crate::theme;

pub fn show_for_tab(ui: &mut egui::Ui, tab: &Tab, undo_count: usize) {
    ui.separator();
    ui.horizontal(|ui| {
        ui.set_min_height(24.0);

        if tab.loading {
            ui.spinner();
            ui.label(
                egui::RichText::new("Loading\u{2026}")
                    .color(theme::SECONDARY)
                    .size(12.0),
            );
        } else {
            let total = tab.entries.len();
            let filtered_count = tab.filtered_cache.len();
            let selected_count = tab.selected_set.len();

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
                    .selected_set
                    .iter()
                    .filter_map(|&i| tab.entries.get(i))
                    .filter(|e| !e.is_dir)
                    .map(|e| e.size)
                    .sum();

                ui.label(
                    egui::RichText::new("\u{00b7}")
                        .color(theme::MUTED)
                        .size(12.0),
                );
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

            if tab.show_hidden {
                ui.label(
                    egui::RichText::new("\u{00b7}")
                        .color(theme::MUTED)
                        .size(12.0),
                );
                ui.label(
                    egui::RichText::new("Hidden visible")
                        .color(theme::SELECTION)
                        .size(12.0),
                );
            }
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

            if undo_count > 0 {
                ui.label(
                    egui::RichText::new("\u{00b7}")
                        .color(theme::MUTED)
                        .size(12.0),
                );
                ui.label(
                    egui::RichText::new(format!("{} undoable", undo_count))
                        .color(theme::SECONDARY)
                        .size(12.0),
                );
            }

            if let Some(ref git) = tab.git_info {
                ui.label(
                    egui::RichText::new("\u{00b7}")
                        .color(theme::MUTED)
                        .size(12.0),
                );
                ui.label(
                    egui::RichText::new(regular::GIT_BRANCH)
                        .color(theme::SELECTION)
                        .size(14.0),
                );
                ui.label(
                    egui::RichText::new(&git.branch)
                        .color(theme::SELECTION)
                        .size(12.0),
                );
            }
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
