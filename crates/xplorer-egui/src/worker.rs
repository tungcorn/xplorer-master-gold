use std::sync::mpsc;

use eframe::egui;

use crate::state::{DirRequest, DirResponse};

pub fn spawn_directory_worker(
    receiver: mpsc::Receiver<DirRequest>,
    sender: mpsc::Sender<DirResponse>,
    ctx: egui::Context,
) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        while let Ok(request) = receiver.recv() {
            match request {
                DirRequest::LoadDirectory { tab_id, path } => {
                    let result = rt.block_on(xplorer_core::directory::read_directory(&path));
                    let _ = sender.send(DirResponse::DirectoryLoaded {
                        tab_id,
                        path,
                        entries: result.map_err(|e| e.to_string()),
                    });
                    ctx.request_repaint();
                }
            }
        }
    });
}
