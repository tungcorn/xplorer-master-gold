use eframe::egui;
use egui_dock::{NodeIndex, SurfaceIndex, TabViewer};

use crate::state::{AppState, Tab};
use crate::ui::{file_list, status_bar, top_bar};

pub enum ViewerAction {
    Navigate(String),
    GoBack,
    GoForward,
    GoUp,
    RequestLoad { tab_id: usize, path: String },
    SplitRight { path: String },
}

pub struct XplorerTabViewer<'a> {
    pub state: &'a mut AppState,
    pub actions: Vec<ViewerAction>,
    pub new_tabs: Vec<Tab>,
}

impl<'a> TabViewer for XplorerTabViewer<'a> {
    type Tab = Tab;

    fn id(&mut self, tab: &mut Tab) -> egui::Id {
        egui::Id::new(tab.id)
    }

    fn title(&mut self, tab: &mut Tab) -> egui::WidgetText {
        let icon = crate::icons::FOLDER;
        format!("{} {}", icon, tab.display_name).into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Tab) {
        egui::TopBottomPanel::bottom(egui::Id::new(("tab_status", tab.id)))
            .resizable(false)
            .show_separator_line(false)
            .show_inside(ui, |ui| {
                status_bar::show_for_tab(ui, tab);
            });

        top_bar::show_for_tab(ui, tab, self.state, &mut self.actions);
        ui.separator();
        file_list::show_for_tab(ui, tab, &mut self.state, &mut self.actions);
    }

    fn scroll_bars(&self, _tab: &Self::Tab) -> [bool; 2] {
        [false, false]
    }

    fn on_close(&mut self, _tab: &mut Tab) -> bool {
        true
    }

    fn on_add(&mut self, _surface: SurfaceIndex, _node: NodeIndex) {
        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("C:\\"));
        let home_str = home.to_string_lossy().to_string();
        let tab = self.state.create_tab(&home_str);
        self.actions.push(ViewerAction::RequestLoad {
            tab_id: tab.id,
            path: home_str,
        });
        self.new_tabs.push(tab);
    }

    fn context_menu(
        &mut self,
        ui: &mut egui::Ui,
        tab: &mut Tab,
        _surface: SurfaceIndex,
        _node: NodeIndex,
    ) {
        if ui.button("Close").clicked() {
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Split Right").clicked() {
            self.actions.push(ViewerAction::SplitRight {
                path: tab.path.clone(),
            });
            ui.close_menu();
        }
    }
}
