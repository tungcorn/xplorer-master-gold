use std::collections::HashSet;
use std::path::Path;

use eframe::egui;
use egui_extras::{Column, TableBuilder};

use crate::icons;
use crate::state::{AppState, ClipboardOp, NewItemMode, SortColumn, Tab, ViewMode};
use crate::theme;
use crate::ui::context_menu::{self, EmptyAreaAction, FileContextAction};
use crate::ui::dock_viewer::ViewerAction;

pub fn show_for_tab(
    ui: &mut egui::Ui,
    tab: &mut Tab,
    state: &mut AppState,
    actions: &mut Vec<ViewerAction>,
) {
    if let Some(mode) = tab.new_item_mode.clone() {
        show_new_item_input(ui, tab, state, &mode, actions);
    }

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

    tab.ensure_filtered();
    let filtered = &tab.filtered_cache;

    if filtered.is_empty() && tab.filter_text.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(egui::RichText::new("Empty directory").color(theme::MUTED));
        });
        return;
    }

    match tab.view_mode {
        ViewMode::Details => show_details_view(ui, tab, state, actions),
        ViewMode::Grid => show_grid_view(ui, tab, state, actions),
    }
}

fn show_details_view(
    ui: &mut egui::Ui,
    tab: &mut Tab,
    state: &mut AppState,
    actions: &mut Vec<ViewerAction>,
) {
    let filtered = &tab.filtered_cache;
    let sort_col = tab.sort_column;
    let sort_asc = tab.sort_ascending;
    let selected = tab.selected_set.clone();
    let has_clipboard = state.clipboard.is_some();
    let rename_idx = tab.rename_state.as_ref().map(|r| r.entry_index);
    let scroll_target = tab.scroll_to_row.take();
    let cut_paths: HashSet<String> = state
        .clipboard
        .as_ref()
        .filter(|cb| cb.operation == ClipboardOp::Cut)
        .map(|cb| cb.paths.iter().cloned().collect())
        .unwrap_or_default();

    let mut sort_clicked: Option<SortColumn> = None;
    let mut selection_action: Option<SelectionAction> = None;
    let mut double_click_action: Option<DoubleClickAction> = None;
    let mut middle_click_split: Option<String> = None;
    let mut file_ctx_action: Option<(FileContextAction, String, bool)> = None;
    let mut rename_commit: Option<(String, String)> = None;
    let mut any_row_hovered = false;
    let mut right_click_select: Option<usize> = None;

    let text_height = ui.text_style_height(&egui::TextStyle::Body);
    let row_height = (text_height + 16.0).max(32.0);

    let empty_area_resp = ui.interact(
        ui.available_rect_before_wrap(),
        egui::Id::new(("file_list_empty_bg", tab.id)),
        egui::Sense::hover(),
    );

    let table = TableBuilder::new(ui)
        .striped(false)
        .resizable(true)
        .sense(egui::Sense::click())
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .id_salt(("file_table", tab.id))
        .column(Column::remainder().at_least(80.0).clip(true))
        .column(Column::initial(80.0).at_least(50.0).clip(true))
        .column(Column::initial(100.0).at_least(60.0).clip(true))
        .column(Column::initial(150.0).at_least(80.0).clip(true));

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
                let original_idx = filtered[idx];
                let entry = &tab.entries[original_idx];
                let is_selected = selected.contains(&original_idx);
                let is_cut = cut_paths.contains(&entry.path);
                let name_color = if is_cut { theme::MUTED } else { theme::TEXT };
                let detail_color = if is_cut {
                    egui::Color32::from_rgba_premultiplied(86, 95, 137, 100)
                } else {
                    theme::SECONDARY
                };

                row.set_selected(is_selected);

                let (_, name_resp) = row.col(|ui| {
                    if rename_idx == Some(original_idx) {
                        if let Some(ref mut rs) = tab.rename_state {
                            let response = ui.text_edit_singleline(&mut rs.new_name);
                            if !response.has_focus() {
                                response.request_focus();
                            }
                            let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                            let escape = ui.input(|i| i.key_pressed(egui::Key::Escape));
                            if enter {
                                rename_commit = Some((entry.path.clone(), rs.new_name.clone()));
                            }
                            if enter || escape || response.lost_focus() {
                                tab.rename_state = None;
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
                            let icon_c = if is_cut {
                                egui::Color32::from_rgba_premultiplied(
                                    icon_color.r(),
                                    icon_color.g(),
                                    icon_color.b(),
                                    100,
                                )
                            } else {
                                icon_color
                            };
                            ui.label(egui::RichText::new(icon).color(icon_c).size(16.0));
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(&entry.name).color(name_color),
                                )
                                .selectable(false)
                                .truncate(),
                            );
                        });
                    }
                });

                let (_, size_resp) = row.col(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let size_text = if entry.is_dir {
                            "—".to_string()
                        } else {
                            format_size(entry.size)
                        };
                        ui.add(
                            egui::Label::new(egui::RichText::new(size_text).color(detail_color))
                                .selectable(false),
                        );
                    });
                });

                let (_, type_resp) = row.col(|ui| {
                    let type_label = human_file_type(&entry.file_type, entry.is_dir);
                    ui.add(
                        egui::Label::new(egui::RichText::new(type_label).color(detail_color))
                            .selectable(false),
                    );
                });

                let (_, mod_resp) = row.col(|ui| {
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(format_timestamp(entry.modified))
                                .color(detail_color),
                        )
                        .selectable(false),
                    );
                });

                let combined = name_resp | size_resp | type_resp | mod_resp;
                let interact = combined | row.response();
                if interact.hovered() {
                    any_row_hovered = true;
                }

                if scroll_target == Some(original_idx) {
                    interact.scroll_to_me(Some(egui::Align::Center));
                }

                if interact.secondary_clicked() && !is_selected {
                    right_click_select = Some(original_idx);
                }

                let ctx_action =
                    context_menu::file_context_menu(&interact, entry.is_dir, has_clipboard);
                if !matches!(ctx_action, FileContextAction::None) {
                    file_ctx_action = Some((ctx_action, entry.path.clone(), entry.is_dir));
                }

                if rename_idx != Some(original_idx) {
                    if interact.middle_clicked() && entry.is_dir {
                        middle_click_split = Some(entry.path.clone());
                    } else if interact.double_clicked() {
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
                            index: original_idx,
                            ctrl: modifiers.ctrl || modifiers.mac_cmd,
                            shift: modifiers.shift,
                        });
                    }
                }
            });
        });

    if let Some(col) = sort_clicked {
        tab.toggle_sort(col);
    }

    if let Some(action) = selection_action {
        apply_selection(tab, action);
    }

    if let Some(idx) = right_click_select {
        tab.selected_set.clear();
        tab.selected_set.insert(idx);
        tab.last_clicked_index = Some(idx);
    }

    match double_click_action {
        Some(DoubleClickAction::NavigateDir(path)) => {
            actions.push(ViewerAction::Navigate(path));
        }
        Some(DoubleClickAction::OpenFile(path)) => {
            let _ = xplorer_core::system::open_file(Path::new(&path));
        }
        None => {}
    }

    if let Some(path) = middle_click_split {
        actions.push(ViewerAction::SplitRight { path });
    }

    if let Some((action, path, is_dir)) = file_ctx_action {
        handle_file_context_action(tab, state, actions, action, &path, is_dir);
    } else if !any_row_hovered {
        let current_path = tab.path.clone();
        let has_clipboard = state.clipboard.is_some();
        let empty_action = context_menu::empty_area_context_menu(&empty_area_resp, has_clipboard);
        handle_empty_area_action(tab, state, actions, empty_action, &current_path);
    }

    if let Some((old_path, new_name)) = rename_commit {
        state.do_rename(old_path, new_name);
    }
}

fn show_grid_view(
    ui: &mut egui::Ui,
    tab: &mut Tab,
    state: &mut AppState,
    actions: &mut Vec<ViewerAction>,
) {
    let filtered = tab.filtered_cache.clone();
    let selected = tab.selected_set.clone();
    let has_clipboard = state.clipboard.is_some();
    let cut_paths: HashSet<String> = state
        .clipboard
        .as_ref()
        .filter(|cb| cb.operation == ClipboardOp::Cut)
        .map(|cb| cb.paths.iter().cloned().collect())
        .unwrap_or_default();

    let mut selection_action: Option<SelectionAction> = None;
    let mut double_click_action: Option<DoubleClickAction> = None;
    let mut file_ctx_action: Option<(FileContextAction, String, bool)> = None;
    let mut right_click_select: Option<usize> = None;
    let mut any_item_interacted = false;

    let cell_width = 88.0_f32;
    let cell_height = 88.0_f32;

    let empty_area_resp = ui.interact(
        ui.available_rect_before_wrap(),
        egui::Id::new(("grid_empty_bg", tab.id)),
        egui::Sense::hover(),
    );

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);

            for &original_idx in &filtered {
                let entry = &tab.entries[original_idx];
                let is_selected = selected.contains(&original_idx);
                let is_cut = cut_paths.contains(&entry.path);

                let (rect, resp) = ui
                    .allocate_exact_size(egui::vec2(cell_width, cell_height), egui::Sense::click());

                let bg = if is_selected {
                    theme::HOVER
                } else if resp.hovered() {
                    egui::Color32::from_rgba_premultiplied(47, 51, 57, 80)
                } else {
                    egui::Color32::TRANSPARENT
                };
                ui.painter().rect_filled(rect, 4.0, bg);

                let ext = Path::new(&entry.name)
                    .extension()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or_default();
                let (icon, icon_color) = icons::file_icon(&ext, entry.is_dir);

                let icon_c = if is_cut {
                    egui::Color32::from_rgba_premultiplied(
                        icon_color.r(),
                        icon_color.g(),
                        icon_color.b(),
                        100,
                    )
                } else {
                    icon_color
                };
                let name_color = if is_cut { theme::MUTED } else { theme::TEXT };

                ui.painter().text(
                    rect.center_top() + egui::vec2(0.0, 28.0),
                    egui::Align2::CENTER_CENTER,
                    icon,
                    egui::FontId::proportional(26.0),
                    icon_c,
                );

                let name_galley = ui.painter().layout(
                    entry.name.clone(),
                    egui::FontId::proportional(10.0),
                    name_color,
                    cell_width - 6.0,
                );
                let name_pos = rect.center_top() + egui::vec2(0.0, 52.0);
                let name_rect = egui::Align2::CENTER_TOP.anchor_size(name_pos, name_galley.size());
                ui.painter().galley(name_rect.min, name_galley, name_color);

                if resp.hovered() {
                    any_item_interacted = true;
                    resp.clone().on_hover_text(&entry.name);
                }

                let ctx_action =
                    context_menu::file_context_menu(&resp, entry.is_dir, has_clipboard);
                if !matches!(ctx_action, FileContextAction::None) {
                    file_ctx_action = Some((ctx_action, entry.path.clone(), entry.is_dir));
                    any_item_interacted = true;
                }

                if resp.secondary_clicked() && !is_selected {
                    right_click_select = Some(original_idx);
                }

                if resp.double_clicked() {
                    if entry.is_dir {
                        double_click_action =
                            Some(DoubleClickAction::NavigateDir(entry.path.clone()));
                    } else {
                        double_click_action = Some(DoubleClickAction::OpenFile(entry.path.clone()));
                    }
                } else if resp.clicked() {
                    let modifiers = resp.ctx.input(|i| i.modifiers);
                    selection_action = Some(SelectionAction {
                        index: original_idx,
                        ctrl: modifiers.ctrl || modifiers.mac_cmd,
                        shift: modifiers.shift,
                    });
                }
            }
        });
    });

    if let Some(action) = selection_action {
        apply_selection(tab, action);
    }
    if let Some(idx) = right_click_select {
        tab.selected_set.clear();
        tab.selected_set.insert(idx);
        tab.last_clicked_index = Some(idx);
    }

    match double_click_action {
        Some(DoubleClickAction::NavigateDir(path)) => {
            actions.push(ViewerAction::Navigate(path));
        }
        Some(DoubleClickAction::OpenFile(path)) => {
            let _ = xplorer_core::system::open_file(Path::new(&path));
        }
        None => {}
    }

    if let Some((action, path, is_dir)) = file_ctx_action {
        handle_file_context_action(tab, state, actions, action, &path, is_dir);
    } else if !any_item_interacted {
        let current_path = tab.path.clone();
        let has_clipboard = state.clipboard.is_some();
        let empty_action = context_menu::empty_area_context_menu(&empty_area_resp, has_clipboard);
        handle_empty_area_action(tab, state, actions, empty_action, &current_path);
    }
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

fn apply_selection(tab: &mut Tab, action: SelectionAction) {
    if action.shift {
        if let Some(anchor) = tab.last_clicked_index {
            let start = anchor.min(action.index);
            let end = anchor.max(action.index);
            tab.selected_set = (start..=end).collect();
        } else {
            tab.selected_set.clear();
            tab.selected_set.insert(action.index);
        }
    } else if action.ctrl {
        if tab.selected_set.contains(&action.index) {
            tab.selected_set.remove(&action.index);
        } else {
            tab.selected_set.insert(action.index);
        }
        tab.last_clicked_index = Some(action.index);
    } else {
        tab.selected_set.clear();
        tab.selected_set.insert(action.index);
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
    tab: &mut Tab,
    state: &mut AppState,
    actions: &mut Vec<ViewerAction>,
    action: FileContextAction,
    path: &str,
    is_dir: bool,
) {
    match action {
        FileContextAction::Open => {
            if is_dir {
                actions.push(ViewerAction::Navigate(path.to_string()));
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
                open_terminal_at(&dir);
            }
        }
        FileContextAction::Copy => state.do_copy(tab),
        FileContextAction::Cut => state.do_cut(tab),
        FileContextAction::Paste => state.do_paste(tab),
        FileContextAction::Delete => state.do_delete(tab, false),
        FileContextAction::MoveToTrash => state.do_delete(tab, true),
        FileContextAction::Rename => {
            let name = Path::new(path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            if let Some(idx) = tab.entries.iter().position(|e| e.path == path) {
                tab.rename_state = Some(crate::state::RenameState {
                    entry_index: idx,
                    new_name: name,
                });
            }
        }
        FileContextAction::None => {}
    }
}

fn handle_empty_area_action(
    tab: &mut Tab,
    state: &mut AppState,
    actions: &mut Vec<ViewerAction>,
    action: EmptyAreaAction,
    current_path: &str,
) {
    match action {
        EmptyAreaAction::Refresh => {
            actions.push(ViewerAction::RequestLoad {
                tab_id: tab.id,
                path: tab.path.clone(),
            });
        }
        EmptyAreaAction::OpenInTerminal => {
            #[cfg(windows)]
            {
                open_terminal_at(current_path);
            }
        }
        EmptyAreaAction::NewFolder => {
            tab.new_item_mode = Some(NewItemMode::Folder);
            tab.new_item_name = "New Folder".to_string();
        }
        EmptyAreaAction::NewFile => {
            tab.new_item_mode = Some(NewItemMode::File);
            tab.new_item_name = "New File.txt".to_string();
        }
        EmptyAreaAction::Paste => state.do_paste(tab),
        EmptyAreaAction::None => {}
    }
}

fn show_new_item_input(
    ui: &mut egui::Ui,
    tab: &mut Tab,
    state: &AppState,
    mode: &NewItemMode,
    _actions: &mut Vec<ViewerAction>,
) {
    let label = match mode {
        NewItemMode::Folder => "New folder name:",
        NewItemMode::File => "New file name:",
    };
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).color(theme::MUTED).size(12.0));
        let response = ui.text_edit_singleline(&mut tab.new_item_name);
        if !response.has_focus() {
            response.request_focus();
        }
        let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
        let escape = ui.input(|i| i.key_pressed(egui::Key::Escape));
        if enter && !tab.new_item_name.trim().is_empty() {
            let full_path = format!("{}\\{}", tab.path, tab.new_item_name.trim());
            let request = match mode {
                NewItemMode::Folder => {
                    crate::state::FileOpRequest::CreateFolder { path: full_path }
                }
                NewItemMode::File => crate::state::FileOpRequest::CreateFile { path: full_path },
            };
            let _ = state.file_op_sender.send(request);
            tab.new_item_mode = None;
            tab.new_item_name.clear();
        } else if escape {
            tab.new_item_mode = None;
            tab.new_item_name.clear();
        }
    });
    ui.add_space(4.0);
}

#[cfg(windows)]
fn open_terminal_at(dir: &str) {
    if std::process::Command::new("wt")
        .args(["-d", dir])
        .spawn()
        .is_err()
    {
        let _ = std::process::Command::new("cmd")
            .args(["/c", "start", "cmd", "/k", &format!("cd /d {}", dir)])
            .spawn();
    }
}
