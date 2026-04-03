use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use xplorer_core::types::{Bookmark, DriveInfo, FileEntry};

pub struct AppState {
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
    pub req_sender: std::sync::mpsc::Sender<DirRequest>,
    pub resp_receiver: std::sync::mpsc::Receiver<DirResponse>,
    pub file_op_sender: std::sync::mpsc::Sender<FileOpRequest>,
    pub file_op_receiver: std::sync::mpsc::Receiver<FileOpResponse>,
    pub sidebar_width: f32,
    pub show_sidebar: bool,
    pub drives: Vec<DriveInfo>,
    pub bookmarks: Vec<Bookmark>,
    pub editing_address_bar: bool,
    pub address_bar_text: String,
    pub clipboard: Option<Clipboard>,
    pub rename_state: Option<RenameState>,
    pub new_item_mode: Option<NewItemMode>,
    pub new_item_name: String,
    pub toasts: egui_notify::Toasts,
    pub focus_filter: bool,
    next_tab_id: usize,
}

impl AppState {
    pub fn new(
        req_sender: std::sync::mpsc::Sender<DirRequest>,
        resp_receiver: std::sync::mpsc::Receiver<DirResponse>,
        file_op_sender: std::sync::mpsc::Sender<FileOpRequest>,
        file_op_receiver: std::sync::mpsc::Receiver<FileOpResponse>,
        initial_path: String,
    ) -> Self {
        let display = display_name_for_path(&initial_path);
        Self {
            tabs: vec![Tab::new(0, initial_path, display)],
            active_tab: 0,
            req_sender,
            resp_receiver,
            file_op_sender,
            file_op_receiver,
            sidebar_width: 200.0,
            show_sidebar: true,
            drives: Vec::new(),
            bookmarks: Vec::new(),
            editing_address_bar: false,
            address_bar_text: String::new(),
            clipboard: None,
            rename_state: None,
            new_item_mode: None,
            new_item_name: String::new(),
            toasts: egui_notify::Toasts::default().with_anchor(egui_notify::Anchor::BottomRight),
            focus_filter: false,
            next_tab_id: 1,
        }
    }

    /// Returns the new tab's id.
    pub fn open_tab(&mut self, path: String) -> usize {
        let id = self.next_tab_id;
        self.next_tab_id += 1;
        let display = display_name_for_path(&path);
        self.tabs.push(Tab::new(id, path, display));
        self.active_tab = self.tabs.len() - 1;
        id
    }

    /// Returns `false` if it was the last tab (caller should quit or open a default).
    pub fn close_tab(&mut self, index: usize) -> bool {
        if self.tabs.len() <= 1 {
            return false;
        }
        self.tabs.remove(index);
        if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        }
        true
    }

    pub fn active_tab(&self) -> &Tab {
        &self.tabs[self.active_tab]
    }

    pub fn active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab]
    }

    pub fn request_load(&self, tab_id: usize, path: String) {
        let _ = self
            .req_sender
            .send(DirRequest::LoadDirectory { tab_id, path });
    }

    pub fn navigate_to(&mut self, path: &str) {
        let tab = self.active_tab_mut();
        tab.navigate(path.to_string());
        let tab_id = tab.id;
        self.request_load(tab_id, path.to_string());
    }

    pub fn go_back_nav(&mut self) {
        let tab = self.active_tab_mut();
        if let Some(path) = tab.go_back() {
            let tab_id = tab.id;
            self.request_load(tab_id, path);
        }
    }

    pub fn go_forward_nav(&mut self) {
        let tab = self.active_tab_mut();
        if let Some(path) = tab.go_forward() {
            let tab_id = tab.id;
            self.request_load(tab_id, path);
        }
    }

    pub fn go_up(&mut self) {
        let current = self.active_tab().path.clone();
        if let Some(parent) = std::path::Path::new(&current).parent() {
            let parent_str = parent.to_string_lossy().to_string();
            if parent_str != current {
                self.navigate_to(&parent_str);
            }
        }
    }

    pub fn new_tab(&mut self) {
        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("C:\\"));
        let home_str = home.to_string_lossy().to_string();
        let id = self.open_tab(home_str.clone());
        self.request_load(id, home_str);
    }

    pub fn switch_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_tab = index;
        }
    }

    /// Drain pending responses from the worker and apply them to tabs.
    pub fn process_responses(&mut self) {
        while let Ok(resp) = self.resp_receiver.try_recv() {
            match resp {
                DirResponse::DirectoryLoaded {
                    tab_id,
                    path,
                    entries,
                } => {
                    if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == tab_id) {
                        tab.loading = false;
                        match entries {
                            Ok(e) => {
                                tab.path = path;
                                tab.display_name = display_name_for_path(&tab.path);
                                tab.entries = e;
                                tab.error = None;
                                tab.selected_indices.clear();
                                tab.sort_entries();
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

    pub fn process_file_op_responses(&mut self) {
        while let Ok(resp) = self.file_op_receiver.try_recv() {
            match resp {
                FileOpResponse::Success { message } => {
                    self.toasts.success(message);
                    self.refresh_active_tab();
                }
                FileOpResponse::Error { message } => {
                    self.toasts.error(message);
                }
            }
        }
    }

    pub fn refresh_active_tab(&self) {
        let tab = self.active_tab();
        self.request_load(tab.id, tab.path.clone());
    }

    pub fn do_copy(&mut self) {
        let paths = self.selected_paths();
        if !paths.is_empty() {
            self.clipboard = Some(Clipboard {
                paths,
                operation: ClipboardOp::Copy,
            });
        }
    }

    pub fn do_cut(&mut self) {
        let paths = self.selected_paths();
        if !paths.is_empty() {
            self.clipboard = Some(Clipboard {
                paths,
                operation: ClipboardOp::Cut,
            });
        }
    }

    pub fn do_paste(&mut self) {
        if let Some(clipboard) = self.clipboard.clone() {
            let dest_dir = self.active_tab().path.clone();
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

    pub fn do_delete(&mut self, to_trash: bool) {
        let paths = self.selected_paths();
        if !paths.is_empty() {
            let _ = self
                .file_op_sender
                .send(FileOpRequest::Delete { paths, to_trash });
        }
    }

    pub fn do_rename(&mut self, path: String, new_name: String) {
        let parent = Path::new(&path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let new_path = format!("{}\\{}", parent, new_name);
        let _ = self.file_op_sender.send(FileOpRequest::Rename {
            old_path: path,
            new_path,
        });
    }

    pub fn do_create_folder(&mut self) {
        self.new_item_mode = Some(NewItemMode::Folder);
        self.new_item_name = "New Folder".to_string();
    }

    pub fn do_create_file(&mut self) {
        self.new_item_mode = Some(NewItemMode::File);
        self.new_item_name = "New File.txt".to_string();
    }

    fn selected_paths(&self) -> Vec<String> {
        let tab = self.active_tab();
        tab.selected_indices
            .iter()
            .filter_map(|&i| tab.entries.get(i))
            .map(|e| e.path.clone())
            .collect()
    }
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
    pub selected_indices: Vec<usize>,
    pub last_clicked_index: Option<usize>,
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
            selected_indices: Vec::new(),
            last_clicked_index: None,
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
        self.selected_indices.clear();
        self.filter_text.clear();
    }

    pub fn go_back(&mut self) -> Option<String> {
        if self.history_index > 0 {
            self.history_index -= 1;
            let path = self.history[self.history_index].clone();
            self.path = path.clone();
            self.display_name = display_name_for_path(&self.path);
            self.loading = true;
            self.error = None;
            self.selected_indices.clear();
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
            self.selected_indices.clear();
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
        self.selected_indices.clear();
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

fn display_name_for_path(path: &str) -> String {
    PathBuf::from(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}
