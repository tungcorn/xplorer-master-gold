use eframe::egui;
use egui_extras::{Column, TableBuilder};
use xplorer_core::types::FileEntry;

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
                return;
            }
            if let Some(err) = &tab.error {
                ui.colored_label(theme::WARNING, format!("Error: {}", err));
                return;
            }

            let filtered: Vec<(usize, &FileEntry)> = tab
                .entries
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    tab.filter_text.is_empty()
                        || e.name
                            .to_lowercase()
                            .contains(&tab.filter_text.to_lowercase())
                })
                .collect();

            if filtered.is_empty() && tab.filter_text.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(egui::RichText::new("Empty directory").color(theme::MUTED));
                });
                return;
            }

            let row_height = 28.0;
            let table = TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::initial(300.0).at_least(150.0))
                .column(Column::initial(80.0).at_least(50.0))
                .column(Column::initial(100.0).at_least(60.0))
                .column(Column::initial(150.0).at_least(80.0));

            table
                .header(row_height, |mut header| {
                    header.col(|ui| {
                        ui.strong("Name");
                    });
                    header.col(|ui| {
                        ui.strong("Size");
                    });
                    header.col(|ui| {
                        ui.strong("Type");
                    });
                    header.col(|ui| {
                        ui.strong("Modified");
                    });
                })
                .body(|body| {
                    body.rows(row_height, filtered.len(), |mut row| {
                        let idx = row.index();
                        let (_original_idx, entry) = &filtered[idx];

                        row.col(|ui| {
                            let prefix = if entry.is_dir { "📁" } else { "  " };
                            ui.label(format!("{} {}", prefix, entry.name));
                        });
                        row.col(|ui| {
                            if entry.is_dir {
                                ui.label("--");
                            } else {
                                ui.label(format_size(entry.size));
                            }
                        });
                        row.col(|ui| {
                            ui.label(&entry.file_type);
                        });
                        row.col(|ui| {
                            ui.label(format_timestamp(entry.modified));
                        });
                    });
                });
        });
}

fn format_size(bytes: u64) -> String {
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

fn format_timestamp(ts: i64) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "--".to_string())
}
