use std::path::Path;

use eframe::egui;
use egui_extras::{Column, TableBuilder};
use xplorer_core::types::FileEntry;

use crate::icons;
use crate::state::{AppState, NewItemMode, SortColumn};
use crate::theme;
use crate::ui::context_menu::{self, EmptyAreaAction, FileContextAction};

pub fn show(ctx: &egui::Context, state: &mut AppState) {
    egui::CentralPanel::default()
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
            let mut any_row_hovered = false;

            let text_height = ui.text_style_height(&egui::TextStyle::Body);
            let row_height = (text_height + 16.0).max(32.0);

            let empty_area_resp = ui.interact(
                ui.available_rect_before_wrap(),
                egui::Id::new("file_list_empty_bg"),
                egui::Sense::hover(),
            );

            let table = TableBuilder::new(ui)
                .striped(false)
                .resizable(true)
                .sense(egui::Sense::click())
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::initial(300.0).at_least(150.0))
                .column(Column::initial(80.0).at_least(50.0))
                .column(Column::initial(100.0).at_least(60.0))
                .column(Column::initial(150.0).at_least(80.0));

            table
                .header(row_height, |mut header| {
                    header.col(|ui| {
                        show_sort_header(
                            ui,
                            "Name",
                            SortColumn::Name,
                            sort_col,
                            sort_asc,
                            &mut sort_clicked,
                        );
                    });
                    header.col(|ui| {
                        show_sort_header(
                            ui,
                            "Size",
                            SortColumn::Size,
                            sort_col,
                            sort_asc,
                            &mut sort_clicked,
                        );
                    });
                    header.col(|ui| {
                        show_sort_header(
                            ui,
                            "Type",
                            SortColumn::Type,
                            sort_col,
                            sort_asc,
                            &mut sort_clicked,
                        );
                    });
                    header.col(|ui| {
                        show_sort_header(
                            ui,
                            "Modified",
                            SortColumn::Modified,
                            sort_col,
                            sort_asc,
                            &mut sort_clicked,
                        );
                    });
                })
                .body(|body| {
                    body.rows(row_height, filtered.len(), |mut row| {
                        let idx = row.index();
                        let (original_idx, entry) = &filtered[idx];
                        let is_selected = selected.contains(original_idx);

                        row.set_selected(is_selected);

                        let (_, name_resp) = row.col(|ui| {
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
                                let ext = Path::new(&entry.name)
                                    .extension()
                                    .map(|e| e.to_string_lossy().to_string())
                                    .unwrap_or_default();
                                let (icon, icon_color) = icons::file_icon(&ext, entry.is_dir);

                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 6.0;
                                    ui.label(
                                        egui::RichText::new(icon).color(icon_color).size(16.0),
                                    );
                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(&entry.name).color(theme::TEXT),
                                        )
                                        .selectable(false)
                                        .truncate(),
                                    );
                                });
                            }
                        });

                        let (_, size_resp) = row.col(|ui| {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let size_text = if entry.is_dir {
                                        "—".to_string()
                                    } else {
                                        format_size(entry.size)
                                    };
                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(size_text).color(theme::SECONDARY),
                                        )
                                        .selectable(false),
                                    );
                                },
                            );
                        });

                        let (_, type_resp) = row.col(|ui| {
                            let type_label = human_file_type(&entry.file_type, entry.is_dir);
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(type_label).color(theme::SECONDARY),
                                )
                                .selectable(false),
                            );
                        });

                        let (_, mod_resp) = row.col(|ui| {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(format_timestamp(entry.modified))
                                        .color(theme::SECONDARY),
                                )
                                .selectable(false),
                            );
                        });

                        let combined = name_resp | size_resp | type_resp | mod_resp;
                        let interact = combined | row.response();
                        if interact.hovered() {
                            any_row_hovered = true;
                        }

                        let ctx_action =
                            context_menu::file_context_menu(&interact, entry.is_dir, has_clipboard);
                        if !matches!(ctx_action, FileContextAction::None) {
                            file_ctx_action = Some((ctx_action, entry.path.clone(), entry.is_dir));
                        }

                        if rename_idx != Some(*original_idx) {
                            if interact.double_clicked() {
                                if entry.is_dir {
                                    double_click_action =
                                        Some(DoubleClickAction::NavigateDir(entry.path.clone()));
                                } else {
                                    double_click_action =
                                        Some(DoubleClickAction::OpenFile(entry.path.clone()));
                                }
                            } else if interact.clicked() {
                                let modifiers = interact.ctx.input(|i| i.modifiers);
                                selection_action = Some(SelectionAction {
                                    index: *original_idx,
                                    ctrl: modifiers.ctrl || modifiers.mac_cmd,
                                    shift: modifiers.shift,
                                });
                            }
                        }
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
            } else if !any_row_hovered {
                let current_path = state.active_tab().path.clone();
                let has_clipboard = state.clipboard.is_some();
                let empty_action =
                    context_menu::empty_area_context_menu(&empty_area_resp, has_clipboard);
                handle_empty_area_action(state, empty_action, &current_path);
            }

            if let Some((old_path, new_name)) = rename_commit {
                state.do_rename(old_path, new_name);
            }
        });
}

fn show_sort_header(
    ui: &mut egui::Ui,
    name: &str,
    col: SortColumn,
    current: SortColumn,
    ascending: bool,
    sort_clicked: &mut Option<SortColumn>,
) {
    let is_active = col == current;
    let text_color = if is_active {
        theme::TEXT
    } else {
        theme::SECONDARY
    };

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        let label = egui::RichText::new(name)
            .color(text_color)
            .size(13.0)
            .strong();
        if ui.selectable_label(false, label).clicked() {
            *sort_clicked = Some(col);
        }
        if is_active {
            let arrow = if ascending { "▲" } else { "▼" };
            ui.label(
                egui::RichText::new(arrow)
                    .color(theme::SELECTION)
                    .size(10.0),
            );
        }
    });
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
        .map(|dt| {
            let local = dt.with_timezone(&chrono::Local);
            let now = chrono::Local::now();
            let today = now.date_naive();
            let file_date = local.date_naive();
            let days_ago = today.signed_duration_since(file_date).num_days();

            if days_ago == 0 {
                format!("Today, {}", local.format("%-I:%M %p"))
            } else if days_ago == 1 {
                format!("Yesterday, {}", local.format("%-I:%M %p"))
            } else if days_ago < 7 {
                local.format("%A, %-I:%M %p").to_string()
            } else {
                local.format("%b %-d, %Y").to_string()
            }
        })
        .unwrap_or_else(|| "—".to_string())
}

fn human_file_type(raw_type: &str, is_dir: bool) -> String {
    if is_dir {
        return "Folder".to_string();
    }
    let ext = raw_type.trim_start_matches('.').to_lowercase();
    match ext.as_str() {
        "rs" => "Rust Source".to_string(),
        "toml" => "TOML Config".to_string(),
        "js" | "mjs" => "JavaScript".to_string(),
        "jsx" => "React JSX".to_string(),
        "ts" => "TypeScript".to_string(),
        "tsx" => "React TSX".to_string(),
        "py" => "Python".to_string(),
        "go" => "Go Source".to_string(),
        "c" => "C Source".to_string(),
        "h" => "C Header".to_string(),
        "cpp" | "cc" => "C++ Source".to_string(),
        "hpp" => "C++ Header".to_string(),
        "java" => "Java Source".to_string(),
        "rb" => "Ruby".to_string(),
        "cs" => "C# Source".to_string(),
        "swift" => "Swift".to_string(),
        "html" | "htm" => "HTML".to_string(),
        "css" => "CSS".to_string(),
        "scss" | "sass" => "SCSS".to_string(),
        "json" => "JSON".to_string(),
        "yaml" | "yml" => "YAML".to_string(),
        "xml" => "XML".to_string(),
        "md" => "Markdown".to_string(),
        "txt" => "Text".to_string(),
        "log" => "Log File".to_string(),
        "png" => "PNG Image".to_string(),
        "jpg" | "jpeg" => "JPEG Image".to_string(),
        "gif" => "GIF Image".to_string(),
        "svg" => "SVG Image".to_string(),
        "webp" => "WebP Image".to_string(),
        "bmp" => "Bitmap".to_string(),
        "ico" => "Icon".to_string(),
        "pdf" => "PDF Document".to_string(),
        "doc" | "docx" => "Word Document".to_string(),
        "xls" | "xlsx" => "Excel Sheet".to_string(),
        "ppt" | "pptx" => "PowerPoint".to_string(),
        "csv" => "CSV Data".to_string(),
        "zip" => "ZIP Archive".to_string(),
        "rar" => "RAR Archive".to_string(),
        "7z" => "7z Archive".to_string(),
        "tar" => "TAR Archive".to_string(),
        "gz" => "GZip Archive".to_string(),
        "mp3" => "MP3 Audio".to_string(),
        "wav" => "WAV Audio".to_string(),
        "flac" => "FLAC Audio".to_string(),
        "mp4" => "MP4 Video".to_string(),
        "mkv" => "MKV Video".to_string(),
        "avi" => "AVI Video".to_string(),
        "mov" => "MOV Video".to_string(),
        "exe" => "Executable".to_string(),
        "msi" => "Installer".to_string(),
        "dll" => "DLL Library".to_string(),
        "bat" | "cmd" => "Batch Script".to_string(),
        "ps1" => "PowerShell".to_string(),
        "sh" | "bash" => "Shell Script".to_string(),
        "ini" | "cfg" | "conf" => "Config".to_string(),
        "env" => "Environment".to_string(),
        "lock" => "Lock File".to_string(),
        "gitignore" => "Git Ignore".to_string(),
        "" => "File".to_string(),
        other => format!("{} File", other.to_uppercase()),
    }
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
