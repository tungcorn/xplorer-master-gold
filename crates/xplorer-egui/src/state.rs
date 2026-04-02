use std::path::PathBuf;
use xplorer_core::types::{Bookmark, DriveInfo, FileEntry};

pub struct AppState {
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
    pub req_sender: std::sync::mpsc::Sender<DirRequest>,
    pub resp_receiver: std::sync::mpsc::Receiver<DirResponse>,
    pub sidebar_width: f32,
    pub show_sidebar: bool,
    pub drives: Vec<DriveInfo>,
    pub bookmarks: Vec<Bookmark>,
    next_tab_id: usize,
}

impl AppState {
    pub fn new(
        req_sender: std::sync::mpsc::Sender<DirRequest>,
        resp_receiver: std::sync::mpsc::Receiver<DirResponse>,
        initial_path: String,
    ) -> Self {
        let display = display_name_for_path(&initial_path);
        Self {
            tabs: vec![Tab::new(0, initial_path, display)],
            active_tab: 0,
            req_sender,
            resp_receiver,
            sidebar_width: 200.0,
            show_sidebar: true,
            drives: Vec::new(),
            bookmarks: Vec::new(),
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
        }
    }

    pub fn navigate(&mut self, path: String) {
        // Truncate forward history if we navigated back earlier.
        self.history.truncate(self.history_index + 1);
        self.history.push(path.clone());
        self.history_index = self.history.len() - 1;
        self.path = path;
        self.display_name = display_name_for_path(&self.path);
        self.loading = true;
        self.error = None;
        self.selected_indices.clear();
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

fn display_name_for_path(path: &str) -> String {
    PathBuf::from(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}
