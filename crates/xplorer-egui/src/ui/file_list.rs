use std::path::Path;

use eframe::egui;
use egui_extras::{Column, TableBuilder};
use xplorer_core::types::FileEntry;

use crate::state::{AppState, NewItemMode, SortColumn};
use crate::theme;
use crate::ui::context_menu::{self, EmptyAreaAction, FileContextAction};

pub fn show(ctx: &egui::Context, state: &mut AppState) {
    let panel_response = egui::CentralPanel::default()
        .frame(egui::Frame::central_panel(&ctx.style()).fill(theme::BACKGROUND))
        .show(ctx, |ui| {
            if let Some(mode) = state.new_item_mode.clone() {
                show_new_item_input(ui, state, &mode);
            }

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
            let has_clipboard = state.clipboard.is_some();
            let rename_idx = state.rename_state.as_ref().map(|r| r.entry_index);

            let mut sort_clicked: Option<SortColumn> = None;
            let mut selection_action: Option<SelectionAction> = None;
            let mut double_click_action: Option<DoubleClickAction> = None;
            let mut file_ctx_action: Option<(FileContextAction, String, bool)> = None;
            let mut rename_commit: Option<(String, String)> = None;

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
                            if rename_idx == Some(*original_idx) {
                                if let Some(ref mut rs) = state.rename_state {
                                    let response = ui.text_edit_singleline(&mut rs.new_name);
                                    if !response.has_focus() {
                                        response.request_focus();
                                    }
                                    let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                                    let escape = ui.input(|i| i.key_pressed(egui::Key::Escape));
                                    if enter {
                                        rename_commit =
                                            Some((entry.path.clone(), rs.new_name.clone()));
                                    }
                                    if enter || escape || response.lost_focus() {
                                        state.rename_state = None;
                                    }
                                }
                            } else {
                                let prefix = if entry.is_dir { "📁" } else { "  " };
                                let label = format!("{} {}", prefix, entry.name);
                                let response = ui.selectable_label(is_selected, label);

                                let ctx = context_menu::file_context_menu(
                                    &response,
                                    entry.is_dir,
                                    has_clipboard,
                                );
                                if !matches!(ctx, FileContextAction::None) {
                                    file_ctx_action = Some((ctx, entry.path.clone(), entry.is_dir));
                                }

                                if response.double_clicked() {
                                    if entry.is_dir {
                                        double_click_action = Some(DoubleClickAction::NavigateDir(
                                            entry.path.clone(),
                                        ));
                                    } else {
                                        double_click_action =
                                            Some(DoubleClickAction::OpenFile(entry.path.clone()));
                                    }
                                } else if response.clicked() {
                                    let modifiers = ui.input(|i| i.modifiers);
                                    selection_action = Some(SelectionAction {
                                        index: *original_idx,
                                        ctrl: modifiers.ctrl || modifiers.mac_cmd,
                                        shift: modifiers.shift,
                                    });
                                }
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

            match double_click_action {
                Some(DoubleClickAction::NavigateDir(path)) => {
                    state.navigate_to(&path);
                }
                Some(DoubleClickAction::OpenFile(path)) => {
                    let _ = xplorer_core::system::open_file(Path::new(&path));
                }
                None => {}
            }

            if let Some((action, path, is_dir)) = file_ctx_action {
                handle_file_context_action(state, action, &path, is_dir);
            }

            if let Some((old_path, new_name)) = rename_commit {
                state.do_rename(old_path, new_name);
            }
        });

    let current_path = state.active_tab().path.clone();
    let has_clipboard = state.clipboard.is_some();
    let empty_action =
        context_menu::empty_area_context_menu(&panel_response.response, has_clipboard);
    handle_empty_area_action(state, empty_action, &current_path);
}

struct SelectionAction {
    index: usize,
    ctrl: bool,
    shift: bool,
}

enum DoubleClickAction {
    NavigateDir(String),
    OpenFile(String),
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

fn handle_file_context_action(
    state: &mut AppState,
    action: FileContextAction,
    path: &str,
    is_dir: bool,
) {
    match action {
        FileContextAction::Open => {
            if is_dir {
                state.navigate_to(path);
            } else {
                let _ = xplorer_core::system::open_file(Path::new(path));
            }
        }
        FileContextAction::ShowInFolder => {
            let _ = xplorer_core::system::show_in_folder(Path::new(path));
        }
        FileContextAction::CopyPath => {
            let mut clipboard = arboard::Clipboard::new().ok();
            if let Some(cb) = clipboard.as_mut() {
                let _ = cb.set_text(path);
            }
        }
        FileContextAction::AddToFavorites => {
            let name = Path::new(path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string());
            let _ = xplorer_core::bookmarks::add_bookmark(name, path.to_string());
            state.bookmarks = xplorer_core::bookmarks::get_bookmarks().unwrap_or_default();
        }
        FileContextAction::OpenInTerminal => {
            #[cfg(windows)]
            {
                let dir = if is_dir {
                    path.to_string()
                } else {
                    Path::new(path)
                        .parent()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default()
                };
                let _ = std::process::Command::new("cmd")
                    .args(["/c", "start", "cmd", "/k", &format!("cd /d {}", dir)])
                    .spawn();
            }
        }
        FileContextAction::Copy => state.do_copy(),
        FileContextAction::Cut => state.do_cut(),
        FileContextAction::Paste => state.do_paste(),
        FileContextAction::Delete => state.do_delete(false),
        FileContextAction::MoveToTrash => state.do_delete(true),
        FileContextAction::Rename => {
            let name = Path::new(path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let tab = state.active_tab();
            if let Some(idx) = tab.entries.iter().position(|e| e.path == path) {
                state.rename_state = Some(crate::state::RenameState {
                    entry_index: idx,
                    new_name: name,
                });
            }
        }
        FileContextAction::None => {}
    }
}

fn handle_empty_area_action(state: &mut AppState, action: EmptyAreaAction, current_path: &str) {
    match action {
        EmptyAreaAction::Refresh => {
            let tab = state.active_tab();
            let tab_id = tab.id;
            let path = tab.path.clone();
            state.request_load(tab_id, path);
        }
        EmptyAreaAction::OpenInTerminal => {
            #[cfg(windows)]
            {
                let _ = std::process::Command::new("cmd")
                    .args([
                        "/c",
                        "start",
                        "cmd",
                        "/k",
                        &format!("cd /d {}", current_path),
                    ])
                    .spawn();
            }
        }
        EmptyAreaAction::NewFolder => state.do_create_folder(),
        EmptyAreaAction::NewFile => state.do_create_file(),
        EmptyAreaAction::Paste => state.do_paste(),
        EmptyAreaAction::None => {}
    }
}

fn show_new_item_input(ui: &mut egui::Ui, state: &mut AppState, mode: &NewItemMode) {
    let label = match mode {
        NewItemMode::Folder => "New folder name:",
        NewItemMode::File => "New file name:",
    };
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).color(theme::MUTED).size(12.0));
        let response = ui.text_edit_singleline(&mut state.new_item_name);
        if !response.has_focus() {
            response.request_focus();
        }
        let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
        let escape = ui.input(|i| i.key_pressed(egui::Key::Escape));
        if enter && !state.new_item_name.trim().is_empty() {
            let parent = state.active_tab().path.clone();
            let full_path = format!("{}\\{}", parent, state.new_item_name.trim());
            let request = match mode {
                NewItemMode::Folder => {
                    crate::state::FileOpRequest::CreateFolder { path: full_path }
                }
                NewItemMode::File => crate::state::FileOpRequest::CreateFile { path: full_path },
            };
            let _ = state.file_op_sender.send(request);
            state.new_item_mode = None;
            state.new_item_name.clear();
        } else if escape {
            state.new_item_mode = None;
            state.new_item_name.clear();
        }
    });
    ui.add_space(4.0);
}
