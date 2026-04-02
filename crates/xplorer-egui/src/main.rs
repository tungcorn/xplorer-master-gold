use eframe::egui;

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
        Box::new(|_cc| Ok(Box::new(XplorerApp::default()))),
    )
}

#[derive(Default)]
struct XplorerApp;

impl eframe::App for XplorerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Xplorer — egui Edition");
            ui.label("If you can see this, eframe+wgpu is working.");
        });
    }
}
