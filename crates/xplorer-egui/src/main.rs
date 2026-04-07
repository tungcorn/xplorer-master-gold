mod icons;
mod session;
mod state;
mod theme;
mod ui;
mod watcher;
mod worker;

use eframe::egui;
use egui_dock::{DockArea, DockState, NodeIndex, SurfaceIndex, TabIndex};
use state::{all_tab_paths, focused_tab, focused_tab_mut, AppState, Tab};
use std::sync::mpsc;
use ui::command_palette::PaletteAction;
use ui::dock_viewer::{ViewerAction, XplorerTabViewer};
use ui::sidebar::SidebarAction;

struct XplorerApp {
    dock_state: DockState<Tab>,
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
        let (file_op_tx, file_op_rx) = mpsc::channel();

        let (resp_tx, resp_rx) = mpsc::channel();
        let (file_op_resp_tx, file_op_resp_rx) = mpsc::channel();
        let (progress_tx, progress_rx) = mpsc::channel();

        worker::spawn_directory_worker(req_rx, resp_tx, cc.egui_ctx.clone());
        worker::spawn_file_op_worker(
            file_op_rx,
            file_op_resp_tx,
            progress_tx.clone(),
            cc.egui_ctx.clone(),
        );

        let drives = xplorer_core::system::list_drives().unwrap_or_default();
        let bookmarks = xplorer_core::bookmarks::get_bookmarks().unwrap_or_default();

        let mut state = AppState::new(req_tx, resp_rx, file_op_tx, file_op_resp_rx, progress_rx);
        state.drives = drives;
        state.bookmarks = bookmarks;

        let dock_state = if let Some(saved) = session::load() {
            state.show_sidebar = saved.show_sidebar;
            state.show_preview = saved.show_preview;
            let tabs: Vec<Tab> = saved
                .tabs
                .iter()
                .filter(|t| std::path::Path::new(&t.path).exists())
                .map(|t| {
                    let tab = state.create_tab(&t.path);
                    state.request_load(tab.id, t.path.clone());
                    tab
                })
                .collect();
            if tabs.is_empty() {
                let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("C:\\"));
                let home_str = home.to_string_lossy().to_string();
                let tab = state.create_tab(&home_str);
                state.request_load(tab.id, home_str);
                DockState::new(vec![tab])
            } else {
                DockState::new(tabs)
            }
        } else {
            let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("C:\\"));
            let home_str = home.to_string_lossy().to_string();
            let tab = state.create_tab(&home_str);
            state.request_load(tab.id, home_str);
            DockState::new(vec![tab])
        };

        let watcher_sender = watcher::spawn_watcher(state.req_sender.clone(), cc.egui_ctx.clone());
        state.watcher_sender = Some(watcher_sender);
        state.update_watcher(&dock_state);

        Self {
            dock_state,
            state,
            confirm_delete: None,
        }
    }

    fn process_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::K) && i.modifiers.ctrl) {
            self.state.command_palette.toggle();
            return;
        }
        if self.state.command_palette.open {
            return;
        }
        if self.state.shortcut_overlay.open {
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.state.shortcut_overlay.open = false;
            }
            return;
        }

        let text_focused = ctx.wants_keyboard_input();

        if ctx.input(|i| i.key_pressed(egui::Key::T) && i.modifiers.ctrl) {
            let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("C:\\"));
            let home_str = home.to_string_lossy().to_string();
            let tab = self.state.create_tab(&home_str);
            self.state.request_load(tab.id, home_str);
            self.dock_state.push_to_focused_leaf(tab);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::W) && i.modifiers.ctrl) {
            if let Some((surface, node)) = self.dock_state.focused_leaf() {
                let tab_to_remove = {
                    let node_ref = &self.dock_state[surface][node];
                    match node_ref {
                        egui_dock::Node::Leaf { tabs, active, .. } => {
                            if tabs.len() > 1 || self.dock_state.main_surface().num_tabs() > 1 {
                                Some(*active)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                };
                if let Some(active) = tab_to_remove {
                    self.dock_state.remove_tab((surface, node, active));
                }
            }
        }

        // Tab key: switch focus between panels
        if !text_focused && ctx.input(|i| i.key_pressed(egui::Key::Tab) && !i.modifiers.ctrl) {
            let leaves: Vec<NodeIndex> = self
                .dock_state
                .main_surface()
                .iter()
                .enumerate()
                .filter_map(|(i, node)| {
                    if node.is_leaf() {
                        Some(NodeIndex(i))
                    } else {
                        None
                    }
                })
                .collect();
            if leaves.len() > 1 {
                let current = self.dock_state.main_surface().focused_leaf();
                let current_pos = current.and_then(|c| leaves.iter().position(|l| *l == c));
                let next = match current_pos {
                    Some(pos) => leaves[(pos + 1) % leaves.len()],
                    None => leaves[0],
                };
                self.dock_state
                    .set_focused_node_and_surface((SurfaceIndex::main(), next));
                self.state.update_watcher(&self.dock_state);
                if let Some(tab) = focused_tab(&self.dock_state) {
                    self.state.refresh_tab(tab);
                }
            }
        }

        // Ctrl+Tab / Ctrl+Shift+Tab: cycle tabs within focused panel
        if ctx.input(|i| i.key_pressed(egui::Key::Tab) && i.modifiers.ctrl) {
            if let Some((surface, node)) = self.dock_state.focused_leaf() {
                let next_tab = {
                    let node_ref = &self.dock_state[surface][node];
                    match node_ref {
                        egui_dock::Node::Leaf { tabs, active, .. } if tabs.len() > 1 => {
                            let shift = ctx.input(|i| i.modifiers.shift);
                            let next_idx = if shift {
                                if active.0 == 0 {
                                    tabs.len() - 1
                                } else {
                                    active.0 - 1
                                }
                            } else {
                                (active.0 + 1) % tabs.len()
                            };
                            Some(TabIndex(next_idx))
                        }
                        _ => None,
                    }
                };
                if let Some(tab_idx) = next_tab {
                    self.dock_state.set_active_tab((surface, node, tab_idx));
                    self.state.update_watcher(&self.dock_state);
                    if let Some(tab) = focused_tab(&self.dock_state) {
                        self.state.refresh_tab(tab);
                    }
                }
            }
        }

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft) && i.modifiers.alt) {
            if let Some(tab) = focused_tab_mut(&mut self.dock_state) {
                self.state.go_back_tab(tab);
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight) && i.modifiers.alt) {
            if let Some(tab) = focused_tab_mut(&mut self.dock_state) {
                self.state.go_forward_tab(tab);
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp) && i.modifiers.alt) {
            if let Some(tab) = focused_tab_mut(&mut self.dock_state) {
                self.state.go_up_tab(tab);
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::L) && i.modifiers.ctrl) {
            if let Some(tab) = focused_tab_mut(&mut self.dock_state) {
                tab.editing_address_bar = true;
                tab.address_bar_text = tab.path.clone();
            }
        }
        if !text_focused && ctx.input(|i| i.key_pressed(egui::Key::Backspace)) {
            if let Some(tab) = focused_tab_mut(&mut self.dock_state) {
                self.state.go_up_tab(tab);
            }
        }

        if ctx.input(|i| i.key_pressed(egui::Key::B) && i.modifiers.ctrl) {
            self.state.show_sidebar = !self.state.show_sidebar;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F) && i.modifiers.ctrl) {
            self.state.focus_filter = true;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::H) && i.modifiers.ctrl) {
            if let Some(tab) = focused_tab_mut(&mut self.dock_state) {
                tab.show_hidden = !tab.show_hidden;
                tab.filter_dirty = true;
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::P) && i.modifiers.ctrl) {
            self.state.show_preview = !self.state.show_preview;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num1) && i.modifiers.ctrl) {
            if let Some(tab) = focused_tab_mut(&mut self.dock_state) {
                tab.view_mode = state::ViewMode::Details;
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num2) && i.modifiers.ctrl) {
            if let Some(tab) = focused_tab_mut(&mut self.dock_state) {
                tab.view_mode = state::ViewMode::Grid;
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F5)) {
            if let Some(tab) = focused_tab(&self.dock_state) {
                self.state.request_load(tab.id, tab.path.clone());
            }
        }

        if !text_focused {
            if ctx.input(|i| i.key_pressed(egui::Key::C) && i.modifiers.ctrl) {
                if let Some(tab) = focused_tab(&self.dock_state) {
                    self.state.do_copy(tab);
                }
            }
            if ctx.input(|i| i.key_pressed(egui::Key::X) && i.modifiers.ctrl) {
                if let Some(tab) = focused_tab(&self.dock_state) {
                    self.state.do_cut(tab);
                }
            }
            if ctx.input(|i| i.key_pressed(egui::Key::V) && i.modifiers.ctrl) {
                if let Some(tab) = focused_tab(&self.dock_state) {
                    self.state.do_paste(tab);
                }
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Delete) && !i.modifiers.shift) {
                if let Some(tab) = focused_tab(&self.dock_state) {
                    self.state.do_delete(tab, true);
                }
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Delete) && i.modifiers.shift) {
                if let Some(tab) = focused_tab(&self.dock_state) {
                    let paths = tab.selected_paths();
                    if !paths.is_empty() {
                        self.confirm_delete = Some(ConfirmDelete { paths });
                    }
                }
            }
            if ctx.input(|i| i.key_pressed(egui::Key::F2)) {
                if let Some(tab) = focused_tab_mut(&mut self.dock_state) {
                    if tab.selected_set.len() == 1 {
                        let idx = *tab.selected_set.iter().next().unwrap();
                        if let Some(entry) = tab.entries.get(idx) {
                            let name = entry.name.clone();
                            tab.rename_state = Some(state::RenameState {
                                entry_index: idx,
                                new_name: name,
                            });
                        }
                    }
                }
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Some(tab) = focused_tab(&self.dock_state) {
                    if let Some(&idx) = tab.selected_set.iter().next() {
                        if let Some(entry) = tab.entries.get(idx) {
                            let path = entry.path.clone();
                            let is_dir = entry.is_dir;
                            if is_dir {
                                if let Some(tab) = focused_tab_mut(&mut self.dock_state) {
                                    self.state.navigate_tab(tab, &path);
                                    self.state.update_watcher(&self.dock_state);
                                }
                            } else {
                                let _ =
                                    xplorer_core::system::open_file(std::path::Path::new(&path));
                            }
                        }
                    }
                }
            }
            let has_question_mark = ctx.input(|i| {
                i.events
                    .iter()
                    .any(|e| matches!(e, egui::Event::Text(t) if t == "?"))
            });
            if has_question_mark {
                self.state.shortcut_overlay.open = true;
            }
            if ctx.input(|i| i.key_pressed(egui::Key::A) && i.modifiers.ctrl) {
                if let Some(tab) = focused_tab_mut(&mut self.dock_state) {
                    tab.ensure_filtered();
                    tab.selected_set = tab.filtered_cache.iter().copied().collect();
                }
            }
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown) && !i.modifiers.alt) {
                if let Some(tab) = focused_tab_mut(&mut self.dock_state) {
                    tab.ensure_filtered();
                    if let Some(next_orig) = tab.next_filtered_index(true) {
                        tab.selected_set.clear();
                        tab.selected_set.insert(next_orig);
                        tab.last_clicked_index = Some(next_orig);
                        tab.scroll_to_row = Some(next_orig);
                    }
                }
            }
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp) && !i.modifiers.alt) {
                if let Some(tab) = focused_tab_mut(&mut self.dock_state) {
                    tab.ensure_filtered();
                    if let Some(prev_orig) = tab.next_filtered_index(false) {
                        tab.selected_set.clear();
                        tab.selected_set.insert(prev_orig);
                        tab.last_clicked_index = Some(prev_orig);
                        tab.scroll_to_row = Some(prev_orig);
                    }
                }
            }
        }
    }

    fn handle_palette_action(&mut self, action: PaletteAction) {
        match action {
            PaletteAction::ToggleSidebar => {
                self.state.show_sidebar = !self.state.show_sidebar;
            }
            PaletteAction::TogglePreview => {
                self.state.show_preview = !self.state.show_preview;
            }
            PaletteAction::ToggleHiddenFiles => {
                if let Some(tab) = focused_tab_mut(&mut self.dock_state) {
                    tab.show_hidden = !tab.show_hidden;
                    tab.filter_dirty = true;
                }
            }
            PaletteAction::NewTab => {
                let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("C:\\"));
                let home_str = home.to_string_lossy().to_string();
                let tab = self.state.create_tab(&home_str);
                self.state.request_load(tab.id, home_str);
                self.dock_state.push_to_focused_leaf(tab);
            }
            PaletteAction::CloseTab => {
                if let Some((surface, node)) = self.dock_state.focused_leaf() {
                    let tab_to_remove = {
                        let node_ref = &self.dock_state[surface][node];
                        match node_ref {
                            egui_dock::Node::Leaf { tabs, active, .. } => {
                                if tabs.len() > 1 || self.dock_state.main_surface().num_tabs() > 1 {
                                    Some(*active)
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        }
                    };
                    if let Some(active) = tab_to_remove {
                        self.dock_state.remove_tab((surface, node, active));
                    }
                }
            }
            PaletteAction::GoBack => {
                if let Some(tab) = focused_tab_mut(&mut self.dock_state) {
                    self.state.go_back_tab(tab);
                }
                self.state.update_watcher(&self.dock_state);
            }
            PaletteAction::GoForward => {
                if let Some(tab) = focused_tab_mut(&mut self.dock_state) {
                    self.state.go_forward_tab(tab);
                }
                self.state.update_watcher(&self.dock_state);
            }
            PaletteAction::GoUp => {
                if let Some(tab) = focused_tab_mut(&mut self.dock_state) {
                    self.state.go_up_tab(tab);
                }
                self.state.update_watcher(&self.dock_state);
            }
            PaletteAction::GoHome => {
                let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("C:\\"));
                let home_str = home.to_string_lossy().to_string();
                if let Some(tab) = focused_tab_mut(&mut self.dock_state) {
                    self.state.navigate_tab(tab, &home_str);
                }
                self.state.update_watcher(&self.dock_state);
            }
            PaletteAction::Refresh => {
                if let Some(tab) = focused_tab(&self.dock_state) {
                    self.state.refresh_tab(tab);
                }
            }
            PaletteAction::FocusFilter => {
                self.state.focus_filter = true;
            }
            PaletteAction::EditAddressBar => {
                if let Some(tab) = focused_tab_mut(&mut self.dock_state) {
                    tab.editing_address_bar = true;
                    tab.address_bar_text = tab.path.clone();
                }
            }
            PaletteAction::SplitRight => {
                if let Some((surface, node)) = self.dock_state.focused_leaf() {
                    let path = focused_tab(&self.dock_state)
                        .map(|t| t.path.clone())
                        .unwrap_or_default();
                    let new_tab = self.state.create_tab(&path);
                    self.state.request_load(new_tab.id, path);
                    self.dock_state.split(
                        (surface, node),
                        egui_dock::Split::Right,
                        0.5,
                        egui_dock::Node::leaf(new_tab),
                    );
                    self.state.update_watcher(&self.dock_state);
                }
            }
            PaletteAction::CopySelection => {
                if let Some(tab) = focused_tab(&self.dock_state) {
                    self.state.do_copy(tab);
                }
            }
            PaletteAction::CutSelection => {
                if let Some(tab) = focused_tab(&self.dock_state) {
                    self.state.do_cut(tab);
                }
            }
            PaletteAction::PasteClipboard => {
                if let Some(tab) = focused_tab(&self.dock_state) {
                    self.state.do_paste(tab);
                }
            }
            PaletteAction::Rename => {
                if let Some(tab) = focused_tab_mut(&mut self.dock_state) {
                    if tab.selected_set.len() == 1 {
                        let idx = *tab.selected_set.iter().next().unwrap();
                        if let Some(entry) = tab.entries.get(idx) {
                            let name = entry.name.clone();
                            tab.rename_state = Some(state::RenameState {
                                entry_index: idx,
                                new_name: name,
                            });
                        }
                    }
                }
            }
            PaletteAction::MoveToTrash => {
                if let Some(tab) = focused_tab(&self.dock_state) {
                    self.state.do_delete(tab, true);
                }
            }
            PaletteAction::DeletePermanently => {
                if let Some(tab) = focused_tab(&self.dock_state) {
                    let paths = tab.selected_paths();
                    if !paths.is_empty() {
                        self.confirm_delete = Some(ConfirmDelete { paths });
                    }
                }
            }
            PaletteAction::SelectAll => {
                if let Some(tab) = focused_tab_mut(&mut self.dock_state) {
                    tab.ensure_filtered();
                    tab.selected_set = tab.filtered_cache.iter().copied().collect();
                }
            }
            PaletteAction::NavigateTo(path) => {
                if let Some(tab) = focused_tab_mut(&mut self.dock_state) {
                    self.state.navigate_tab(tab, &path);
                }
                self.state.update_watcher(&self.dock_state);
            }
        }
    }

    fn process_viewer_actions(&mut self, actions: Vec<ViewerAction>) {
        for action in actions {
            match action {
                ViewerAction::Navigate(path) => {
                    if let Some(tab) = focused_tab_mut(&mut self.dock_state) {
                        self.state.navigate_tab(tab, &path);
                    }
                    self.state.update_watcher(&self.dock_state);
                }
                ViewerAction::GoBack => {
                    if let Some(tab) = focused_tab_mut(&mut self.dock_state) {
                        self.state.go_back_tab(tab);
                    }
                    self.state.update_watcher(&self.dock_state);
                }
                ViewerAction::GoForward => {
                    if let Some(tab) = focused_tab_mut(&mut self.dock_state) {
                        self.state.go_forward_tab(tab);
                    }
                    self.state.update_watcher(&self.dock_state);
                }
                ViewerAction::GoUp => {
                    if let Some(tab) = focused_tab_mut(&mut self.dock_state) {
                        self.state.go_up_tab(tab);
                    }
                    self.state.update_watcher(&self.dock_state);
                }
                ViewerAction::RequestLoad { tab_id, path } => {
                    self.state.request_load(tab_id, path);
                }
                ViewerAction::SplitRight { path } => {
                    if let Some((surface, node)) = self.dock_state.focused_leaf() {
                        let new_tab = self.state.create_tab(&path);
                        self.state.request_load(new_tab.id, path);
                        self.dock_state.split(
                            (surface, node),
                            egui_dock::Split::Right,
                            0.5,
                            egui_dock::Node::leaf(new_tab),
                        );
                        self.state.update_watcher(&self.dock_state);
                    }
                }
            }
        }
    }
}

impl eframe::App for XplorerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.state.process_responses(&mut self.dock_state);
        self.state.process_file_op_responses(&self.dock_state);
        self.state.process_progress();
        self.process_keyboard_shortcuts(ctx);

        let title = focused_tab(&self.dock_state)
            .map(|t| format!("{} — Xplorer", t.display_name))
            .unwrap_or_else(|| "Xplorer".to_string());
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));

        let focused_path = focused_tab(&self.dock_state)
            .map(|t| t.path.clone())
            .unwrap_or_default();

        let sidebar_action = ui::sidebar::show(ctx, &self.state, &focused_path);
        match sidebar_action {
            SidebarAction::Navigate(path) => {
                if let Some(tab) = focused_tab_mut(&mut self.dock_state) {
                    self.state.navigate_tab(tab, &path);
                }
                self.state.update_watcher(&self.dock_state);
            }
            SidebarAction::RemoveBookmark(path) => {
                let _ = xplorer_core::bookmarks::remove_bookmark(&path);
                self.state.bookmarks = xplorer_core::bookmarks::get_bookmarks().unwrap_or_default();
            }
            SidebarAction::None => {}
        }

        let dock_style = theme::dock_style(ctx.style().as_ref());

        let mut viewer = XplorerTabViewer {
            state: &mut self.state,
            actions: Vec::new(),
            new_tabs: Vec::new(),
        };

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).fill(theme::BACKGROUND))
            .show(ctx, |ui| {
                DockArea::new(&mut self.dock_state)
                    .style(dock_style)
                    .draggable_tabs(true)
                    .show_add_buttons(true)
                    .show_close_buttons(true)
                    .show_inside(ui, &mut viewer);
            });

        let actions = std::mem::take(&mut viewer.actions);
        let new_tabs = std::mem::take(&mut viewer.new_tabs);
        drop(viewer);

        for tab in new_tabs {
            self.dock_state.push_to_focused_leaf(tab);
        }
        self.process_viewer_actions(actions);

        if self.state.focus_filter {
            self.state.focus_filter = false;
        }

        self.state.toasts.show(ctx);

        ui::progress_panel::show(ctx, &self.state.active_operations);

        if let Some(action) = ui::command_palette::show(ctx, &mut self.state.command_palette) {
            self.handle_palette_action(action);
        }

        ui::shortcut_overlay::show(ctx, &mut self.state.shortcut_overlay);

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
                        id: 0,
                        paths: dialog.paths,
                        to_trash: false,
                    });
            }
        } else if dismiss_dialog || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.confirm_delete = None;
        }

        if !self.state.active_operations.is_empty() {
            ctx.request_repaint();
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let tabs: Vec<session::TabSession> = all_tab_paths(&self.dock_state)
            .into_iter()
            .map(|path| session::TabSession { path })
            .collect();
        let sess = session::Session {
            tabs,
            show_sidebar: self.state.show_sidebar,
            show_preview: self.state.show_preview,
        };
        session::save(&sess);
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
