use std::cmp::Ordering;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

use egui_dock::DockState;
use xplorer_core::types::{Bookmark, DriveInfo, FileEntry};

use crate::watcher::WatchCommand;

pub struct AppState {
    pub req_sender: mpsc::Sender<DirRequest>,
    pub resp_rx: mpsc::Receiver<DirResponse>,
    pub file_op_sender: mpsc::Sender<FileOpRequest>,
    pub file_op_rx: mpsc::Receiver<FileOpResponse>,
    pub watcher_sender: Option<mpsc::Sender<WatchCommand>>,
    pub show_sidebar: bool,
    pub drives: Vec<DriveInfo>,
    pub bookmarks: Vec<Bookmark>,
    pub clipboard: Option<Clipboard>,
    pub toasts: egui_notify::Toasts,
    pub focus_filter: bool,
    pub next_tab_id: usize,
}

impl AppState {
    pub fn new(
        req_sender: mpsc::Sender<DirRequest>,
        resp_rx: mpsc::Receiver<DirResponse>,
        file_op_sender: mpsc::Sender<FileOpRequest>,
        file_op_rx: mpsc::Receiver<FileOpResponse>,
    ) -> Self {
        Self {
            req_sender,
            resp_rx,
            file_op_sender,
            file_op_rx,
            watcher_sender: None,
            show_sidebar: true,
            drives: Vec::new(),
            bookmarks: Vec::new(),
            clipboard: None,
            toasts: egui_notify::Toasts::default().with_anchor(egui_notify::Anchor::BottomRight),
            focus_filter: false,
            next_tab_id: 1,
        }
    }

    pub fn request_load(&self, tab_id: usize, path: String) {
        let _ = self
            .req_sender
            .send(DirRequest::LoadDirectory { tab_id, path });
    }

    pub fn navigate_tab(&self, tab: &mut Tab, path: &str) {
        tab.navigate(path.to_string());
        self.request_load(tab.id, path.to_string());
    }

    pub fn go_back_tab(&self, tab: &mut Tab) {
        if let Some(path) = tab.go_back() {
            self.request_load(tab.id, path);
        }
    }

    pub fn go_forward_tab(&self, tab: &mut Tab) {
        if let Some(path) = tab.go_forward() {
            self.request_load(tab.id, path);
        }
    }

    pub fn go_up_tab(&self, tab: &mut Tab) {
        let current = tab.path.clone();
        if let Some(parent) = Path::new(&current).parent() {
            let parent_str = parent.to_string_lossy().to_string();
            if parent_str != current {
                self.navigate_tab(tab, &parent_str);
            }
        }
    }

    pub fn refresh_tab(&self, tab: &Tab) {
        self.request_load(tab.id, tab.path.clone());
    }

    pub fn create_tab(&mut self, path: &str) -> Tab {
        let id = self.next_tab_id;
        self.next_tab_id += 1;
        let display = display_name_for_path(path);
        Tab::new(id, path.to_string(), display)
    }

    pub fn do_copy(&mut self, tab: &Tab) {
        let paths = tab.selected_paths();
        if !paths.is_empty() {
            self.clipboard = Some(Clipboard {
                paths,
                operation: ClipboardOp::Copy,
            });
        }
    }

    pub fn do_cut(&mut self, tab: &Tab) {
        let paths = tab.selected_paths();
        if !paths.is_empty() {
            self.clipboard = Some(Clipboard {
                paths,
                operation: ClipboardOp::Cut,
            });
        }
    }

    pub fn do_paste(&mut self, tab: &Tab) {
        if let Some(clipboard) = self.clipboard.clone() {
            let dest_dir = tab.path.clone();
            let request = match clipboard.operation {
                ClipboardOp::Copy => FileOpRequest::Copy {
                    sources: clipboard.paths,
                    dest_dir,
                },
                ClipboardOp::Cut => FileOpRequest::Move {
                    sources: clipboard.paths,
                    dest_dir,
                },
            };
            let _ = self.file_op_sender.send(request);
            if clipboard.operation == ClipboardOp::Cut {
                self.clipboard = None;
            }
        }
    }

    pub fn do_delete(&self, tab: &Tab, to_trash: bool) {
        let paths = tab.selected_paths();
        if !paths.is_empty() {
            let _ = self
                .file_op_sender
                .send(FileOpRequest::Delete { paths, to_trash });
        }
    }

    pub fn do_rename(&self, path: String, new_name: String) {
        let new_path = Path::new(&path)
            .with_file_name(&new_name)
            .to_string_lossy()
            .to_string();
        let _ = self.file_op_sender.send(FileOpRequest::Rename {
            old_path: path,
            new_path,
        });
    }

    pub fn process_responses(&mut self, dock: &mut DockState<Tab>) {
        for resp in self.resp_rx.try_iter() {
            match resp {
                DirResponse::DirectoryLoaded {
                    tab_id,
                    path,
                    entries,
                } => {
                    if let Some(tab) = find_tab_by_id_mut(dock, tab_id) {
                        tab.loading = false;
                        match entries {
                            Ok(e) => {
                                let is_refresh = tab.path == path;
                                let saved_paths: Vec<String> = if is_refresh {
                                    tab.selected_set
                                        .iter()
                                        .filter_map(|&i| tab.entries.get(i))
                                        .map(|entry| entry.path.clone())
                                        .collect()
                                } else {
                                    Vec::new()
                                };

                                tab.path = path;
                                tab.display_name = display_name_for_path(&tab.path);
                                tab.entries = e;
                                tab.error = None;
                                tab.clear_selection();
                                tab.sort_entries();

                                if !saved_paths.is_empty() {
                                    let restored: Vec<usize> = tab
                                        .entries
                                        .iter()
                                        .enumerate()
                                        .filter(|(_, entry)| saved_paths.contains(&entry.path))
                                        .map(|(i, _)| i)
                                        .collect();
                                    for i in restored {
                                        tab.selected_set.insert(i);
                                    }
                                }
                            }
                            Err(e) => {
                                tab.error = Some(e);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn process_file_op_responses(&mut self, dock: &DockState<Tab>) {
        for resp in self.file_op_rx.try_iter() {
            match resp {
                FileOpResponse::Success { message } => {
                    self.toasts.success(message);
                    if let Some(tab) = focused_tab(dock) {
                        self.request_load(tab.id, tab.path.clone());
                    }
                }
                FileOpResponse::Error { message } => {
                    self.toasts.error(message);
                }
            }
        }
    }

    pub fn update_watcher(&self, dock: &DockState<Tab>) {
        if let Some(ref sender) = self.watcher_sender {
            if let Some(tab) = focused_tab(dock) {
                let _ = sender.send(WatchCommand::Watch {
                    tab_id: tab.id,
                    path: tab.path.clone(),
                });
            }
        }
    }
}

pub fn focused_tab(dock: &DockState<Tab>) -> Option<&Tab> {
    let (surface, node_idx) = dock.focused_leaf()?;
    let node = &dock[surface][node_idx];
    match node {
        egui_dock::Node::Leaf { tabs, active, .. } => tabs.get(active.0),
        _ => None,
    }
}

pub fn focused_tab_mut(dock: &mut DockState<Tab>) -> Option<&mut Tab> {
    let (surface, node_idx) = dock.focused_leaf()?;
    let node = &mut dock[surface][node_idx];
    match node {
        egui_dock::Node::Leaf { tabs, active, .. } => tabs.get_mut(active.0),
        _ => None,
    }
}

pub fn find_tab_by_id_mut(dock: &mut DockState<Tab>, id: usize) -> Option<&mut Tab> {
    for (_surface_index, node) in dock.iter_all_nodes_mut() {
        if let Some(tabs) = node.tabs_mut() {
            if let Some(tab) = tabs.iter_mut().find(|t| t.id == id) {
                return Some(tab);
            }
        }
    }
    None
}

pub struct Tab {
    pub id: usize,
    pub path: String,
    pub display_name: String,
    pub entries: Vec<FileEntry>,
    pub loading: bool,
    pub error: Option<String>,
    pub history: Vec<String>,
    pub history_index: usize,
    pub sort_column: SortColumn,
    pub sort_ascending: bool,
    pub filter_text: String,
    pub selected_set: HashSet<usize>,
    pub last_clicked_index: Option<usize>,
    pub editing_address_bar: bool,
    pub address_bar_text: String,
    pub rename_state: Option<RenameState>,
    pub new_item_mode: Option<NewItemMode>,
    pub new_item_name: String,
    pub filtered_cache: Vec<usize>,
    pub filter_dirty: bool,
    prev_filter_text: String,
}

impl Tab {
    pub fn new(id: usize, path: String, display_name: String) -> Self {
        Self {
            id,
            path: path.clone(),
            display_name,
            entries: Vec::new(),
            loading: true,
            error: None,
            history: vec![path],
            history_index: 0,
            sort_column: SortColumn::Name,
            sort_ascending: true,
            filter_text: String::new(),
            selected_set: HashSet::new(),
            last_clicked_index: None,
            editing_address_bar: false,
            address_bar_text: String::new(),
            rename_state: None,
            new_item_mode: None,
            new_item_name: String::new(),
            filtered_cache: Vec::new(),
            filter_dirty: true,
            prev_filter_text: String::new(),
        }
    }

    pub fn navigate(&mut self, path: String) {
        self.history.truncate(self.history_index + 1);
        self.history.push(path.clone());
        self.history_index = self.history.len() - 1;
        self.path = path;
        self.display_name = display_name_for_path(&self.path);
        self.loading = true;
        self.error = None;
        self.clear_selection();
        self.filter_text.clear();
        self.filter_dirty = true;
    }

    pub fn go_back(&mut self) -> Option<String> {
        if self.history_index > 0 {
            self.history_index -= 1;
            let path = self.history[self.history_index].clone();
            self.path = path.clone();
            self.display_name = display_name_for_path(&self.path);
            self.loading = true;
            self.error = None;
            self.clear_selection();
            Some(path)
        } else {
            None
        }
    }

    pub fn go_forward(&mut self) -> Option<String> {
        if self.history_index + 1 < self.history.len() {
            self.history_index += 1;
            let path = self.history[self.history_index].clone();
            self.path = path.clone();
            self.display_name = display_name_for_path(&self.path);
            self.loading = true;
            self.error = None;
            self.clear_selection();
            Some(path)
        } else {
            None
        }
    }

    pub fn toggle_sort(&mut self, column: SortColumn) {
        if self.sort_column == column {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_column = column;
            self.sort_ascending = true;
        }
        self.sort_entries();
    }

    pub fn sort_entries(&mut self) {
        let col = self.sort_column;
        let asc = self.sort_ascending;
        self.entries.sort_by(|a, b| {
            let dir_cmp = b.is_dir.cmp(&a.is_dir);
            if dir_cmp != Ordering::Equal {
                return dir_cmp;
            }
            let cmp = match col {
                SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortColumn::Size => a.size.cmp(&b.size),
                SortColumn::Type => a.file_type.cmp(&b.file_type),
                SortColumn::Modified => a.modified.cmp(&b.modified),
            };
            if asc {
                cmp
            } else {
                cmp.reverse()
            }
        });
        self.clear_selection();
        self.filter_dirty = true;
    }

    pub fn selected_paths(&self) -> Vec<String> {
        self.selected_set
            .iter()
            .filter_map(|&i| self.entries.get(i))
            .map(|e| e.path.clone())
            .collect()
    }

    pub fn clear_selection(&mut self) {
        self.selected_set.clear();
        self.last_clicked_index = None;
    }

    /// Rebuild filtered_cache if filter_text changed or filter_dirty is set.
    pub fn ensure_filtered(&mut self) {
        if self.filter_dirty || self.filter_text != self.prev_filter_text {
            let lower_filter = self.filter_text.to_lowercase();
            self.filtered_cache = self
                .entries
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    lower_filter.is_empty() || e.name.to_lowercase().contains(&lower_filter)
                })
                .map(|(i, _)| i)
                .collect();
            self.prev_filter_text = self.filter_text.clone();
            self.filter_dirty = false;
        }
    }

    /// Navigate to next/prev item in filtered list. Returns the original entry index.
    pub fn next_filtered_index(&self, forward: bool) -> Option<usize> {
        if self.filtered_cache.is_empty() {
            return None;
        }
        let current_orig = self.selected_set.iter().min().copied();
        let current_pos = current_orig
            .and_then(|orig| self.filtered_cache.iter().position(|&i| i == orig))
            .unwrap_or(0);
        let next_pos = if forward {
            (current_pos + 1).min(self.filtered_cache.len() - 1)
        } else {
            current_pos.saturating_sub(1)
        };
        Some(self.filtered_cache[next_pos])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Name,
    Size,
    Type,
    Modified,
}

pub enum DirRequest {
    LoadDirectory { tab_id: usize, path: String },
}

pub enum DirResponse {
    DirectoryLoaded {
        tab_id: usize,
        path: String,
        entries: Result<Vec<FileEntry>, String>,
    },
}

#[derive(Clone)]
pub struct Clipboard {
    pub paths: Vec<String>,
    pub operation: ClipboardOp,
}

#[derive(Clone, PartialEq)]
pub enum ClipboardOp {
    Copy,
    Cut,
}

pub struct RenameState {
    pub entry_index: usize,
    pub new_name: String,
}

#[derive(Clone, PartialEq)]
pub enum NewItemMode {
    Folder,
    File,
}

pub enum FileOpRequest {
    Copy {
        sources: Vec<String>,
        dest_dir: String,
    },
    Move {
        sources: Vec<String>,
        dest_dir: String,
    },
    Delete {
        paths: Vec<String>,
        to_trash: bool,
    },
    Rename {
        old_path: String,
        new_path: String,
    },
    CreateFolder {
        path: String,
    },
    CreateFile {
        path: String,
    },
}

pub enum FileOpResponse {
    Success { message: String },
    Error { message: String },
}

pub fn display_name_for_path(path: &str) -> String {
    PathBuf::from(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}
