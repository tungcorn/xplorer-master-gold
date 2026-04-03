use std::path::Path;
use std::sync::mpsc;

use eframe::egui;

use crate::state::{DirRequest, DirResponse, FileOpRequest, FileOpResponse};

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

pub fn spawn_file_op_worker(
    receiver: mpsc::Receiver<FileOpRequest>,
    sender: mpsc::Sender<FileOpResponse>,
    ctx: egui::Context,
) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        while let Ok(request) = receiver.recv() {
            let response = match request {
                FileOpRequest::Copy { sources, dest_dir } => run_batch(
                    &sources,
                    |src| {
                        let name = file_name_of(src);
                        let dest = Path::new(&dest_dir).join(&name);
                        let src_path = Path::new(src);
                        if src_path.is_dir() {
                            xplorer_core::file_ops::copy_dir(src_path, &dest)
                        } else {
                            xplorer_core::file_ops::copy_file(src_path, &dest)
                        }
                    },
                    "Copied",
                ),
                FileOpRequest::Move { sources, dest_dir } => run_batch(
                    &sources,
                    |src| {
                        let name = file_name_of(src);
                        let dest = Path::new(&dest_dir).join(&name);
                        xplorer_core::file_ops::move_entry(Path::new(src), &dest)
                    },
                    "Moved",
                ),
                FileOpRequest::Delete { paths, to_trash } => run_batch(
                    &paths,
                    |p| {
                        let path = Path::new(p);
                        if to_trash {
                            xplorer_core::trash_ops::move_to_trash(path)
                        } else if path.is_dir() {
                            xplorer_core::file_ops::delete_dir(path)
                        } else {
                            xplorer_core::file_ops::delete_file(path)
                        }
                    },
                    if to_trash { "Trashed" } else { "Deleted" },
                ),
                FileOpRequest::Rename { old_path, new_path } => {
                    match xplorer_core::file_ops::rename(Path::new(&old_path), Path::new(&new_path))
                    {
                        Ok(()) => FileOpResponse::Success {
                            message: format!("Renamed to {}", file_name_of(&new_path)),
                        },
                        Err(e) => FileOpResponse::Error {
                            message: e.to_string(),
                        },
                    }
                }
                FileOpRequest::CreateFolder { path } => {
                    match rt.block_on(xplorer_core::directory::create_dir_recursive(&path)) {
                        Ok(()) => FileOpResponse::Success {
                            message: format!("Created folder {}", file_name_of(&path)),
                        },
                        Err(e) => FileOpResponse::Error {
                            message: e.to_string(),
                        },
                    }
                }
                FileOpRequest::CreateFile { path } => {
                    match xplorer_core::file_ops::create_file(Path::new(&path)) {
                        Ok(()) => FileOpResponse::Success {
                            message: format!("Created file {}", file_name_of(&path)),
                        },
                        Err(e) => FileOpResponse::Error {
                            message: e.to_string(),
                        },
                    }
                }
            };
            let _ = sender.send(response);
            ctx.request_repaint();
        }
    });
}

fn file_name_of(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

fn run_batch<F>(paths: &[String], op: F, verb: &str) -> FileOpResponse
where
    F: Fn(&str) -> Result<(), xplorer_core::error::CoreError>,
{
    let mut errors = Vec::new();
    for p in paths {
        if let Err(e) = op(p) {
            errors.push(format!("{}: {}", file_name_of(p), e));
        }
    }
    if errors.is_empty() {
        FileOpResponse::Success {
            message: format!("{} {} item(s)", verb, paths.len()),
        }
    } else {
        FileOpResponse::Error {
            message: errors.join("\n"),
        }
    }
}
