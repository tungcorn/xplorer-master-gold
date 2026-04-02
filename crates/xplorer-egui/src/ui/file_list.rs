use eframe::egui;
use egui_extras::{Column, TableBuilder};
use xplorer_core::types::FileEntry;

use crate::state::{AppState, SortColumn};
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

            let sort_col = tab.sort_column;
            let sort_asc = tab.sort_ascending;
            let selected = tab.selected_indices.clone();

            let mut sort_clicked: Option<SortColumn> = None;
            let mut selection_action: Option<SelectionAction> = None;

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
                        if ui
                            .selectable_label(
                                false,
                                sort_header("Name", SortColumn::Name, sort_col, sort_asc),
                            )
                            .clicked()
                        {
                            sort_clicked = Some(SortColumn::Name);
                        }
                    });
                    header.col(|ui| {
                        if ui
                            .selectable_label(
                                false,
                                sort_header("Size", SortColumn::Size, sort_col, sort_asc),
                            )
                            .clicked()
                        {
                            sort_clicked = Some(SortColumn::Size);
                        }
                    });
                    header.col(|ui| {
                        if ui
                            .selectable_label(
                                false,
                                sort_header("Type", SortColumn::Type, sort_col, sort_asc),
                            )
                            .clicked()
                        {
                            sort_clicked = Some(SortColumn::Type);
                        }
                    });
                    header.col(|ui| {
                        if ui
                            .selectable_label(
                                false,
                                sort_header("Modified", SortColumn::Modified, sort_col, sort_asc),
                            )
                            .clicked()
                        {
                            sort_clicked = Some(SortColumn::Modified);
                        }
                    });
                })
                .body(|body| {
                    body.rows(row_height, filtered.len(), |mut row| {
                        let idx = row.index();
                        let (original_idx, entry) = &filtered[idx];
                        let is_selected = selected.contains(original_idx);

                        row.col(|ui| {
                            let prefix = if entry.is_dir { "📁" } else { "  " };
                            let label = format!("{} {}", prefix, entry.name);
                            let response = ui.selectable_label(is_selected, label);
                            if response.clicked() {
                                let modifiers = ui.input(|i| i.modifiers);
                                selection_action = Some(SelectionAction {
                                    index: *original_idx,
                                    ctrl: modifiers.ctrl || modifiers.mac_cmd,
                                    shift: modifiers.shift,
                                });
                            }
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

            if let Some(col) = sort_clicked {
                state.active_tab_mut().toggle_sort(col);
            }

            if let Some(action) = selection_action {
                apply_selection(state, action);
            }
        });
}

struct SelectionAction {
    index: usize,
    ctrl: bool,
    shift: bool,
}

fn apply_selection(state: &mut AppState, action: SelectionAction) {
    let tab = state.active_tab_mut();
    if action.shift {
        if let Some(anchor) = tab.last_clicked_index {
            let start = anchor.min(action.index);
            let end = anchor.max(action.index);
            tab.selected_indices = (start..=end).collect();
        } else {
            tab.selected_indices = vec![action.index];
        }
    } else if action.ctrl {
        if let Some(pos) = tab.selected_indices.iter().position(|&i| i == action.index) {
            tab.selected_indices.remove(pos);
        } else {
            tab.selected_indices.push(action.index);
        }
        tab.last_clicked_index = Some(action.index);
    } else {
        tab.selected_indices = vec![action.index];
        tab.last_clicked_index = Some(action.index);
    }
}

fn sort_header(name: &str, col: SortColumn, current: SortColumn, ascending: bool) -> String {
    if col == current {
        format!("{} {}", name, if ascending { "▲" } else { "▼" })
    } else {
        name.to_string()
    }
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
