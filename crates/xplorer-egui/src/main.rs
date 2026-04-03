mod icons;
mod state;
mod theme;
mod ui;
mod watcher;
mod worker;

use eframe::egui;
use state::AppState;
use std::sync::mpsc;
use ui::sidebar::SidebarAction;

struct XplorerApp {
    state: AppState,
    confirm_delete: Option<ConfirmDelete>,
}

struct ConfirmDelete {
    paths: Vec<String>,
}

impl XplorerApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply_theme(&cc.egui_ctx);
        theme::setup_fonts(&cc.egui_ctx);

        let (req_tx, req_rx) = mpsc::channel();
        let (resp_tx, resp_rx) = mpsc::channel();
        let (file_op_tx, file_op_rx) = mpsc::channel();
        let (file_op_resp_tx, file_op_resp_rx) = mpsc::channel();

        worker::spawn_directory_worker(req_rx, resp_tx, cc.egui_ctx.clone());
        worker::spawn_file_op_worker(file_op_rx, file_op_resp_tx, cc.egui_ctx.clone());

        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("C:\\"));
        let home_str = home.to_string_lossy().to_string();

        let drives = xplorer_core::system::list_drives().unwrap_or_default();
        let bookmarks = xplorer_core::bookmarks::get_bookmarks().unwrap_or_default();

        let mut state = AppState::new(
            req_tx,
            resp_rx,
            file_op_tx,
            file_op_resp_rx,
            home_str.clone(),
        );
        state.drives = drives;
        state.bookmarks = bookmarks;

        state.request_load(0, home_str);

        let watcher_sender = watcher::spawn_watcher(state.req_sender.clone(), cc.egui_ctx.clone());
        state.watcher_sender = Some(watcher_sender);
        state.update_watcher();

        Self {
            state,
            confirm_delete: None,
        }
    }

    fn active_tab_snapshot(&self) -> Option<(usize, String)> {
        let tab = self.state.active_tab();
        if tab.selected_indices.len() == 1 {
            let idx = tab.selected_indices[0];
            tab.entries.get(idx).map(|e| (idx, e.name.clone()))
        } else {
            None
        }
    }

    fn process_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        let text_focused = ctx.wants_keyboard_input();

        if ctx.input(|i| i.key_pressed(egui::Key::T) && i.modifiers.ctrl) {
            self.state.new_tab();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::W) && i.modifiers.ctrl) {
            let idx = self.state.active_tab;
            self.state.close_tab(idx);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Tab) && i.modifiers.ctrl && !i.modifiers.shift) {
            let next = (self.state.active_tab + 1) % self.state.tabs.len();
            self.state.switch_tab(next);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Tab) && i.modifiers.ctrl && i.modifiers.shift) {
            let prev = if self.state.active_tab == 0 {
                self.state.tabs.len() - 1
            } else {
                self.state.active_tab - 1
            };
            self.state.switch_tab(prev);
        }

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft) && i.modifiers.alt) {
            self.state.go_back_nav();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight) && i.modifiers.alt) {
            self.state.go_forward_nav();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp) && i.modifiers.alt) {
            self.state.go_up();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::L) && i.modifiers.ctrl) {
            self.state.editing_address_bar = true;
            self.state.address_bar_text = self.state.active_tab().path.clone();
        }
        if !text_focused && ctx.input(|i| i.key_pressed(egui::Key::Backspace)) {
            self.state.go_up();
        }

        if ctx.input(|i| i.key_pressed(egui::Key::B) && i.modifiers.ctrl) {
            self.state.show_sidebar = !self.state.show_sidebar;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F) && i.modifiers.ctrl) {
            self.state.focus_filter = true;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F5)) {
            self.state.refresh_active_tab();
        }

        if !text_focused {
            if ctx.input(|i| i.key_pressed(egui::Key::C) && i.modifiers.ctrl) {
                self.state.do_copy();
            }
            if ctx.input(|i| i.key_pressed(egui::Key::X) && i.modifiers.ctrl) {
                self.state.do_cut();
            }
            if ctx.input(|i| i.key_pressed(egui::Key::V) && i.modifiers.ctrl) {
                self.state.do_paste();
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Delete) && !i.modifiers.shift) {
                self.state.do_delete(true);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Delete) && i.modifiers.shift) {
                let paths = self.state.selected_paths();
                if !paths.is_empty() {
                    self.confirm_delete = Some(ConfirmDelete { paths });
                }
            }
            if ctx.input(|i| i.key_pressed(egui::Key::F2)) {
                if let Some((idx, name)) = self.active_tab_snapshot() {
                    self.state.rename_state = Some(state::RenameState {
                        entry_index: idx,
                        new_name: name,
                    });
                }
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                let tab = self.state.active_tab();
                if let Some(&idx) = tab.selected_indices.first() {
                    if let Some(entry) = tab.entries.get(idx) {
                        let path = entry.path.clone();
                        let is_dir = entry.is_dir;
                        if is_dir {
                            self.state.navigate_to(&path);
                        } else {
                            let _ = xplorer_core::system::open_file(std::path::Path::new(&path));
                        }
                    }
                }
            }
            if ctx.input(|i| i.key_pressed(egui::Key::A) && i.modifiers.ctrl) {
                let tab = &mut self.state.tabs[self.state.active_tab];
                tab.selected_indices = (0..tab.entries.len()).collect();
            }
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown) && !i.modifiers.alt) {
                let tab = &mut self.state.tabs[self.state.active_tab];
                let current = tab.selected_indices.first().copied().unwrap_or(0);
                let next = (current + 1).min(tab.entries.len().saturating_sub(1));
                tab.selected_indices = vec![next];
                tab.last_clicked_index = Some(next);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp) && !i.modifiers.alt) {
                let tab = &mut self.state.tabs[self.state.active_tab];
                let current = tab.selected_indices.first().copied().unwrap_or(0);
                let prev = current.saturating_sub(1);
                tab.selected_indices = vec![prev];
                tab.last_clicked_index = Some(prev);
            }
        }
    }
}

impl eframe::App for XplorerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.state.process_responses();
        self.state.process_file_op_responses();
        self.process_keyboard_shortcuts(ctx);

        let tab = &self.state.tabs[self.state.active_tab];
        let title = format!("{} — Xplorer", tab.display_name);
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));

        let sidebar_action = ui::sidebar::show(ctx, &mut self.state);
        match sidebar_action {
            SidebarAction::Navigate(path) => self.state.navigate_to(&path),
            SidebarAction::RemoveBookmark(path) => {
                let _ = xplorer_core::bookmarks::remove_bookmark(&path);
                self.state.bookmarks = xplorer_core::bookmarks::get_bookmarks().unwrap_or_default();
            }
            SidebarAction::None => {}
        }
        ui::top_bar::show(ctx, &mut self.state);
        ui::status_bar::show(ctx, &self.state);
        ui::file_list::show(ctx, &mut self.state);
        self.state.toasts.show(ctx);

        let mut dismiss_dialog = false;
        let mut do_permanent_delete = false;
        if let Some(ref dialog) = self.confirm_delete {
            let count = dialog.paths.len();
            egui::Window::new("Confirm Delete")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(format!(
                        "Permanently delete {} item(s)? This cannot be undone.",
                        count
                    ));
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            dismiss_dialog = true;
                        }
                        if ui
                            .button(egui::RichText::new("Delete").color(theme::WARNING))
                            .clicked()
                        {
                            do_permanent_delete = true;
                        }
                    });
                });
        }
        if do_permanent_delete {
            if let Some(dialog) = self.confirm_delete.take() {
                let _ = self
                    .state
                    .file_op_sender
                    .send(state::FileOpRequest::Delete {
                        paths: dialog.paths,
                        to_trash: false,
                    });
            }
        } else if dismiss_dialog || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.confirm_delete = None;
        }
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Xplorer")
            .with_inner_size([1280.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "xplorer-egui",
        options,
        Box::new(|cc| Ok(Box::new(XplorerApp::new(cc)))),
    )
}
