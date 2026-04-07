use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use eframe::egui;
use notify::RecursiveMode;
use notify_debouncer_full::new_debouncer;

use crate::state::DirRequest;

pub enum WatchCommand {
    WatchAll {
        tabs: Vec<(usize, String)>,
    },
    #[allow(dead_code)]
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

        let mut watched: HashMap<PathBuf, Vec<usize>> = HashMap::new();
        let mut last_reload: HashMap<PathBuf, Instant> = HashMap::new();
        let cooldown = Duration::from_secs(2);

        loop {
            if let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    WatchCommand::WatchAll { tabs } => {
                        let mut new_map: HashMap<PathBuf, Vec<usize>> = HashMap::new();
                        for (tab_id, path_str) in &tabs {
                            let pb = PathBuf::from(path_str);
                            new_map.entry(pb).or_default().push(*tab_id);
                        }

                        for old_path in watched.keys() {
                            if !new_map.contains_key(old_path) {
                                let _ = debouncer.unwatch(old_path);
                            }
                        }
                        for new_path in new_map.keys() {
                            if !watched.contains_key(new_path) {
                                let _ = debouncer.watch(new_path, RecursiveMode::NonRecursive);
                            }
                        }

                        watched = new_map;
                    }
                    WatchCommand::Stop => break,
                }
            }

            while let Ok(Ok(events)) = rx.try_recv() {
                let mut reloaded = std::collections::HashSet::new();
                for event in &events {
                    for event_path in &event.paths {
                        let parent = event_path
                            .parent()
                            .map(|p| p.to_path_buf())
                            .unwrap_or_else(|| event_path.clone());

                        let dir = if watched.contains_key(&parent) {
                            Some(parent)
                        } else if watched.contains_key(event_path) {
                            Some(event_path.clone())
                        } else {
                            None
                        };

                        if let Some(dir) = dir {
                            let now = Instant::now();
                            let last = last_reload
                                .get(&dir)
                                .copied()
                                .unwrap_or(Instant::now() - cooldown);
                            if now.duration_since(last) >= cooldown && !reloaded.contains(&dir) {
                                if let Some(tab_ids) = watched.get(&dir) {
                                    for &tab_id in tab_ids {
                                        let _ = dir_sender.send(DirRequest::LoadDirectory {
                                            tab_id,
                                            path: dir.to_string_lossy().to_string(),
                                        });
                                    }
                                }
                                last_reload.insert(dir.clone(), now);
                                reloaded.insert(dir);
                                ctx.request_repaint();
                            }
                        }
                    }
                }
            }

            std::thread::sleep(Duration::from_millis(100));
        }
    });

    cmd_tx
}
