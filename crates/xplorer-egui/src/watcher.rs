use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use eframe::egui;
use notify::RecursiveMode;
use notify_debouncer_full::new_debouncer;

use crate::state::DirRequest;

pub enum WatchCommand {
    Watch { tab_id: usize, path: String },
    Stop,
}

pub fn spawn_watcher(
    dir_sender: mpsc::Sender<DirRequest>,
    ctx: egui::Context,
) -> mpsc::Sender<WatchCommand> {
    let (cmd_tx, cmd_rx) = mpsc::channel::<WatchCommand>();

    std::thread::spawn(move || {
        let (tx, rx) = mpsc::channel();

        let mut debouncer = match new_debouncer(Duration::from_millis(500), None, tx) {
            Ok(d) => d,
            Err(_) => return,
        };

        let mut current_path: Option<String> = None;
        let mut current_tab_id: Option<usize> = None;
        let mut last_reload = Instant::now();
        let cooldown = Duration::from_secs(2);

        loop {
            if let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    WatchCommand::Watch { tab_id, path } => {
                        if let Some(ref old) = current_path {
                            let _ = debouncer.unwatch(Path::new(old));
                        }
                        let _ = debouncer.watch(Path::new(&path), RecursiveMode::NonRecursive);
                        current_path = Some(path);
                        current_tab_id = Some(tab_id);
                        last_reload = Instant::now();
                    }
                    WatchCommand::Stop => break,
                }
            }

            while let Ok(Ok(_events)) = rx.try_recv() {
                if last_reload.elapsed() >= cooldown {
                    if let (Some(tab_id), Some(ref path)) = (current_tab_id, &current_path) {
                        let _ = dir_sender.send(DirRequest::LoadDirectory {
                            tab_id,
                            path: path.clone(),
                        });
                        ctx.request_repaint();
                        last_reload = Instant::now();
                    }
                }
            }

            std::thread::sleep(Duration::from_millis(100));
        }
    });

    cmd_tx
}
