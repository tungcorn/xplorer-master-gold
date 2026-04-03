mod state;
mod theme;
mod ui;
mod worker;

use eframe::egui;
use state::AppState;
use std::sync::mpsc;
use ui::sidebar::SidebarAction;

struct XplorerApp {
    state: AppState,
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

        Self { state }
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
}

impl eframe::App for XplorerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.state.process_responses();
        self.state.process_file_op_responses();

        if ctx.input(|i| i.key_pressed(egui::Key::T) && i.modifiers.ctrl) {
            self.state.new_tab();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::W) && i.modifiers.ctrl) {
            let idx = self.state.active_tab;
            self.state.close_tab(idx);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::B) && i.modifiers.ctrl) {
            self.state.show_sidebar = !self.state.show_sidebar;
        }
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
            self.state.do_delete(false);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F2)) {
            let tab = self.active_tab_snapshot();
            if let Some((idx, name)) = tab {
                self.state.rename_state = Some(state::RenameState {
                    entry_index: idx,
                    new_name: name,
                });
            }
        }

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
