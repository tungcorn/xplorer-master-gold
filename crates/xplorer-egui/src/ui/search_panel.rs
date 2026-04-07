use std::path::{Path, PathBuf};
use std::sync::mpsc;

use eframe::egui;

use crate::icons;
use crate::theme;

pub struct SearchState {
    pub open: bool,
    pub query: String,
    pub root_path: String,
    pub results: Vec<SearchResult>,
    pub selected_index: usize,
    pub searching: bool,
    pub search_mode: SearchMode,
    result_rx: Option<mpsc::Receiver<SearchMessage>>,
    last_query: String,
    total_scanned: u64,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SearchMode {
    Filename,
    Content,
}

#[derive(Clone)]
pub struct SearchResult {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub line_match: Option<String>,
}

enum SearchMessage {
    Result(SearchResult),
    Progress(u64),
    Done,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            open: false,
            query: String::new(),
            root_path: String::new(),
            results: Vec::new(),
            selected_index: 0,
            searching: false,
            search_mode: SearchMode::Filename,
            result_rx: None,
            last_query: String::new(),
            total_scanned: 0,
        }
    }
}

impl SearchState {
    pub fn open_at(&mut self, root: &str) {
        self.open = true;
        self.root_path = root.to_string();
        self.query.clear();
        self.results.clear();
        self.selected_index = 0;
        self.searching = false;
        self.result_rx = None;
        self.last_query.clear();
        self.total_scanned = 0;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.result_rx = None;
        self.searching = false;
    }

    fn poll_results(&mut self) {
        if let Some(ref rx) = self.result_rx {
            let mut count = 0;
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    SearchMessage::Result(r) => {
                        if self.results.len() < 500 {
                            self.results.push(r);
                        }
                    }
                    SearchMessage::Progress(n) => {
                        self.total_scanned = n;
                    }
                    SearchMessage::Done => {
                        self.searching = false;
                        self.result_rx = None;
                        return;
                    }
                }
                count += 1;
                if count > 100 {
                    break;
                }
            }
        }
    }

    fn start_search(&mut self) {
        let query = self.query.trim().to_string();
        if query.len() < 2 {
            self.results.clear();
            self.searching = false;
            return;
        }

        self.results.clear();
        self.selected_index = 0;
        self.searching = true;
        self.total_scanned = 0;
        self.last_query = query.clone();

        let (tx, rx) = mpsc::channel();
        self.result_rx = Some(rx);

        let root = self.root_path.clone();
        let mode = self.search_mode;

        std::thread::spawn(move || {
            let lower_query = query.to_lowercase();
            let mut scanned = 0u64;
            let mut stack = vec![PathBuf::from(&root)];

            while let Some(dir) = stack.pop() {
                let entries = match std::fs::read_dir(&dir) {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                for entry in entries.flatten() {
                    let path = entry.path();
                    let meta = match entry.metadata() {
                        Ok(m) => m,
                        Err(_) => continue,
                    };

                    scanned += 1;
                    if scanned % 500 == 0 {
                        let _ = tx.send(SearchMessage::Progress(scanned));
                    }

                    let name = entry.file_name().to_string_lossy().to_string();

                    if meta.is_dir() {
                        if !name.starts_with('.') {
                            stack.push(path.clone());
                        }
                        if mode == SearchMode::Filename
                            && name.to_lowercase().contains(&lower_query)
                        {
                            let _ = tx.send(SearchMessage::Result(SearchResult {
                                path: path.to_string_lossy().to_string(),
                                name,
                                is_dir: true,
                                line_match: None,
                            }));
                        }
                        continue;
                    }

                    match mode {
                        SearchMode::Filename => {
                            if name.to_lowercase().contains(&lower_query) {
                                let _ = tx.send(SearchMessage::Result(SearchResult {
                                    path: path.to_string_lossy().to_string(),
                                    name,
                                    is_dir: false,
                                    line_match: None,
                                }));
                            }
                        }
                        SearchMode::Content => {
                            if meta.len() > 2 * 1024 * 1024 {
                                continue;
                            }
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                for (line_num, line) in content.lines().enumerate() {
                                    if line.to_lowercase().contains(&lower_query) {
                                        let preview = line.trim();
                                        let preview = if preview.len() > 120 {
                                            format!("{}...", &preview[..120])
                                        } else {
                                            preview.to_string()
                                        };
                                        let _ = tx.send(SearchMessage::Result(SearchResult {
                                            path: path.to_string_lossy().to_string(),
                                            name: name.clone(),
                                            is_dir: false,
                                            line_match: Some(format!(
                                                "{}:{}: {}",
                                                line_num + 1,
                                                name,
                                                preview
                                            )),
                                        }));
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let _ = tx.send(SearchMessage::Done);
        });
    }
}

pub enum SearchAction {
    NavigateTo(String),
    OpenFile(String),
}

pub fn show(ctx: &egui::Context, state: &mut SearchState) -> Option<SearchAction> {
    if !state.open {
        return None;
    }

    state.poll_results();

    let screen_rect = ctx.screen_rect();
    let mut action: Option<SearchAction> = None;

    egui::Area::new(egui::Id::new("search_backdrop"))
        .fixed_pos(screen_rect.min)
        .show(ctx, |ui| {
            let resp = ui.allocate_rect(screen_rect, egui::Sense::click());
            ui.painter()
                .rect_filled(screen_rect, 0.0, egui::Color32::from_black_alpha(140));
            if resp.clicked() {
                state.close();
            }
        });

    let panel_width = 600.0_f32.min(screen_rect.width() - 40.0);
    let panel_x = (screen_rect.width() - panel_width) / 2.0;
    let panel_y = screen_rect.height() * 0.12;

    egui::Area::new(egui::Id::new("search_panel"))
        .fixed_pos(egui::pos2(panel_x, panel_y))
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(theme::SURFACE)
                .corner_radius(8.0)
                .stroke(egui::Stroke::new(1.0, theme::BORDER))
                .inner_margin(16.0)
                .show(ui, |ui| {
                    ui.set_width(panel_width);

                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Search")
                                .color(theme::TEXT)
                                .size(14.0)
                                .strong(),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let mode_label = match state.search_mode {
                                SearchMode::Filename => "Filename",
                                SearchMode::Content => "Content",
                            };
                            if ui
                                .selectable_label(
                                    false,
                                    egui::RichText::new(mode_label)
                                        .color(theme::SELECTION)
                                        .size(11.0),
                                )
                                .clicked()
                            {
                                state.search_mode = match state.search_mode {
                                    SearchMode::Filename => SearchMode::Content,
                                    SearchMode::Content => SearchMode::Filename,
                                };
                                if !state.query.is_empty() {
                                    state.start_search();
                                }
                            }
                            ui.label(egui::RichText::new("Mode:").color(theme::MUTED).size(11.0));
                        });
                    });

                    ui.add_space(8.0);

                    let input = egui::TextEdit::singleline(&mut state.query)
                        .desired_width(panel_width - 32.0)
                        .hint_text("Search files...")
                        .font(egui::TextStyle::Body);
                    let resp = ui.add(input);
                    if !resp.has_focus() {
                        resp.request_focus();
                    }

                    let enter = ctx.input(|i| i.key_pressed(egui::Key::Enter));
                    let escape = ctx.input(|i| i.key_pressed(egui::Key::Escape));
                    let arrow_down = ctx.input(|i| i.key_pressed(egui::Key::ArrowDown));
                    let arrow_up = ctx.input(|i| i.key_pressed(egui::Key::ArrowUp));

                    if escape {
                        state.close();
                        return;
                    }

                    if state.query.trim() != state.last_query && state.query.trim().len() >= 2 {
                        state.start_search();
                    } else if state.query.trim().len() < 2 && !state.results.is_empty() {
                        state.results.clear();
                        state.searching = false;
                    }

                    if arrow_down && !state.results.is_empty() {
                        state.selected_index =
                            (state.selected_index + 1).min(state.results.len() - 1);
                    }
                    if arrow_up && state.selected_index > 0 {
                        state.selected_index -= 1;
                    }

                    if enter && !state.results.is_empty() {
                        let result = &state.results[state.selected_index];
                        if result.is_dir {
                            action = Some(SearchAction::NavigateTo(result.path.clone()));
                        } else {
                            let parent = Path::new(&result.path)
                                .parent()
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or_default();
                            action = Some(SearchAction::NavigateTo(parent));
                        }
                        state.close();
                        return;
                    }

                    ui.add_space(4.0);

                    if state.searching {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(
                                egui::RichText::new(format!(
                                    "Searching... {} found, {} scanned",
                                    state.results.len(),
                                    state.total_scanned
                                ))
                                .color(theme::SECONDARY)
                                .size(11.0),
                            );
                        });
                    } else if !state.results.is_empty() {
                        ui.label(
                            egui::RichText::new(format!("{} results", state.results.len()))
                                .color(theme::SECONDARY)
                                .size(11.0),
                        );
                    }

                    let max_visible = 12;
                    let visible_results = state.results.len().min(max_visible);
                    let mut clicked_index: Option<usize> = None;

                    if visible_results > 0 {
                        ui.add_space(4.0);
                        egui::ScrollArea::vertical()
                            .max_height(visible_results as f32 * 32.0)
                            .show(ui, |ui| {
                                for (i, result) in state.results.iter().enumerate() {
                                    if i >= 200 {
                                        break;
                                    }
                                    let is_selected = i == state.selected_index;
                                    let bg = if is_selected {
                                        theme::HOVER
                                    } else {
                                        egui::Color32::TRANSPARENT
                                    };

                                    let row_resp = ui.allocate_ui_with_layout(
                                        egui::vec2(ui.available_width(), 28.0),
                                        egui::Layout::left_to_right(egui::Align::Center),
                                        |ui| {
                                            let rect = ui.max_rect();
                                            ui.painter().rect_filled(rect, 3.0, bg);

                                            let ext = Path::new(&result.name)
                                                .extension()
                                                .map(|e| e.to_string_lossy().to_string())
                                                .unwrap_or_default();
                                            let (icon, icon_color) =
                                                icons::file_icon(&ext, result.is_dir);
                                            ui.label(
                                                egui::RichText::new(icon)
                                                    .color(icon_color)
                                                    .size(14.0),
                                            );

                                            ui.add(
                                                egui::Label::new(
                                                    egui::RichText::new(&result.name)
                                                        .color(theme::TEXT)
                                                        .size(12.0),
                                                )
                                                .truncate(),
                                            );

                                            let rel_path = result
                                                .path
                                                .strip_prefix(&state.root_path)
                                                .unwrap_or(&result.path);
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    ui.add(
                                                        egui::Label::new(
                                                            egui::RichText::new(rel_path)
                                                                .color(theme::MUTED)
                                                                .size(10.0),
                                                        )
                                                        .truncate(),
                                                    );
                                                },
                                            );
                                        },
                                    );

                                    if row_resp.response.clicked() {
                                        clicked_index = Some(i);
                                    }
                                }
                            });
                    }

                    if let Some(idx) = clicked_index {
                        let result = &state.results[idx];
                        if result.is_dir {
                            action = Some(SearchAction::NavigateTo(result.path.clone()));
                        } else {
                            let parent = Path::new(&result.path)
                                .parent()
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or_default();
                            action = Some(SearchAction::NavigateTo(parent));
                        }
                        state.close();
                    }
                });
        });

    if state.searching {
        ctx.request_repaint();
    }

    action
}
