mod state;
mod theme;
mod worker;

use eframe::egui;
use state::AppState;
use std::sync::mpsc;

struct XplorerApp {
    state: AppState,
}

impl XplorerApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply_theme(&cc.egui_ctx);
        theme::setup_fonts(&cc.egui_ctx);

        let (req_tx, req_rx) = mpsc::channel();
        let (resp_tx, resp_rx) = mpsc::channel();

        worker::spawn_directory_worker(req_rx, resp_tx, cc.egui_ctx.clone());

        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("C:\\"));
        let home_str = home.to_string_lossy().to_string();

        let drives = xplorer_core::system::list_drives().unwrap_or_default();
        let bookmarks = xplorer_core::bookmarks::get_bookmarks().unwrap_or_default();

        let mut state = AppState::new(req_tx, resp_rx, home_str.clone());
        state.drives = drives;
        state.bookmarks = bookmarks;

        state.request_load(0, home_str);

        Self { state }
    }
}

impl eframe::App for XplorerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.state.process_responses();

        egui::CentralPanel::default().show(ctx, |ui| {
            let tab = self.state.active_tab();
            if tab.loading {
                ui.spinner();
                ui.label(format!("Loading {}…", tab.path));
            } else if let Some(err) = &tab.error {
                ui.colored_label(egui::Color32::from_rgb(255, 85, 85), err);
            } else {
                ui.label(format!("{} items in {}", tab.entries.len(), tab.path));
                for entry in &tab.entries {
                    ui.label(&entry.name);
                }
            }
        });
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
