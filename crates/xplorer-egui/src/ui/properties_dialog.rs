use std::path::Path;
use std::sync::mpsc;
use std::time::SystemTime;

use eframe::egui;

use crate::theme;

pub struct PropertiesDialog {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub file_type: String,
    pub size: u64,
    pub size_label: String,
    pub created: i64,
    pub modified: i64,
    pub accessed: i64,
    pub is_readonly: bool,
    pub is_hidden: bool,
    pub dir_size_rx: Option<mpsc::Receiver<u64>>,
    pub dir_computing: bool,
    pub item_count: Option<u64>,
    pub item_count_rx: Option<mpsc::Receiver<u64>>,
}

impl PropertiesDialog {
    pub fn open(path: &str) -> Self {
        let p = Path::new(path);
        let name = p
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string());

        let meta = std::fs::metadata(p).ok();
        let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        let modified = meta
            .as_ref()
            .and_then(|m| m.modified().ok())
            .map(to_ts)
            .unwrap_or(0);
        let created = meta
            .as_ref()
            .and_then(|m| m.created().ok())
            .map(to_ts)
            .unwrap_or(0);
        let accessed = meta
            .as_ref()
            .and_then(|m| m.accessed().ok())
            .map(to_ts)
            .unwrap_or(0);
        let is_readonly = meta
            .as_ref()
            .map(|m| m.permissions().readonly())
            .unwrap_or(false);

        let ext = p
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();
        let file_type = if is_dir {
            "Folder".to_string()
        } else if ext.is_empty() {
            "File".to_string()
        } else {
            format!("{} File", ext.to_uppercase())
        };

        let is_hidden = name.starts_with('.');

        let mut dialog = Self {
            path: path.to_string(),
            name,
            is_dir,
            file_type,
            size,
            size_label: if is_dir {
                "Calculating\u{2026}".to_string()
            } else {
                format_size_detailed(size)
            },
            created,
            modified,
            accessed,
            is_readonly,
            is_hidden,
            dir_size_rx: None,
            dir_computing: is_dir,
            item_count: if is_dir { None } else { Some(0) },
            item_count_rx: None,
        };

        if is_dir {
            let (size_tx, size_rx) = mpsc::channel();
            let (count_tx, count_rx) = mpsc::channel();
            let dir_path = path.to_string();
            std::thread::spawn(move || {
                let (total_size, count) = compute_dir_size(&dir_path);
                let _ = size_tx.send(total_size);
                let _ = count_tx.send(count);
            });
            dialog.dir_size_rx = Some(size_rx);
            dialog.item_count_rx = Some(count_rx);
        }

        dialog
    }

    pub fn poll(&mut self) {
        if let Some(ref rx) = self.dir_size_rx {
            if let Ok(size) = rx.try_recv() {
                self.size = size;
                self.size_label = format_size_detailed(size);
                self.dir_computing = false;
                self.dir_size_rx = None;
            }
        }
        if let Some(ref rx) = self.item_count_rx {
            if let Ok(count) = rx.try_recv() {
                self.item_count = Some(count);
                self.item_count_rx = None;
            }
        }
    }
}

pub fn show(ctx: &egui::Context, dialog: &mut Option<PropertiesDialog>) -> bool {
    let Some(ref mut props) = dialog else {
        return false;
    };
    props.poll();

    let mut close = false;
    egui::Window::new("Properties")
        .collapsible(false)
        .resizable(false)
        .default_width(360.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.set_min_width(340.0);

            ui.horizontal(|ui| {
                let ext = Path::new(&props.name)
                    .extension()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or_default();
                let (icon, icon_color) = crate::icons::file_icon(&ext, props.is_dir);
                ui.label(egui::RichText::new(icon).color(icon_color).size(28.0));
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(&props.name)
                            .color(theme::TEXT)
                            .size(16.0)
                            .strong(),
                    );
                    ui.label(
                        egui::RichText::new(&props.file_type)
                            .color(theme::SECONDARY)
                            .size(12.0),
                    );
                });
            });

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            egui::Grid::new("properties_grid")
                .num_columns(2)
                .spacing([16.0, 6.0])
                .show(ui, |ui| {
                    prop_row(ui, "Location", &props.path);
                    ui.end_row();

                    let size_text = if props.dir_computing {
                        format!("{} (calculating\u{2026})", props.size_label)
                    } else {
                        props.size_label.clone()
                    };
                    prop_row(ui, "Size", &size_text);
                    ui.end_row();

                    if let Some(count) = props.item_count {
                        if props.is_dir {
                            prop_row(ui, "Contents", &format!("{} items", count));
                            ui.end_row();
                        }
                    }

                    if props.created != 0 {
                        prop_row(ui, "Created", &format_timestamp(props.created));
                        ui.end_row();
                    }

                    prop_row(ui, "Modified", &format_timestamp(props.modified));
                    ui.end_row();

                    if props.accessed != 0 {
                        prop_row(ui, "Accessed", &format_timestamp(props.accessed));
                        ui.end_row();
                    }

                    let mut attrs = Vec::new();
                    if props.is_readonly {
                        attrs.push("Read-only");
                    }
                    if props.is_hidden {
                        attrs.push("Hidden");
                    }
                    if attrs.is_empty() {
                        attrs.push("Normal");
                    }
                    prop_row(ui, "Attributes", &attrs.join(", "));
                    ui.end_row();
                });

            ui.add_space(12.0);

            ui.horizontal(|ui| {
                if ui.button("Copy Path").clicked() {
                    if let Ok(mut cb) = arboard::Clipboard::new() {
                        let _ = cb.set_text(&props.path);
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Close").clicked() {
                        close = true;
                    }
                });
            });
        });

    if close || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        *dialog = None;
    }
    close
}

fn prop_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.label(
        egui::RichText::new(label)
            .color(theme::SECONDARY)
            .size(12.0),
    );
    ui.add(
        egui::Label::new(egui::RichText::new(value).color(theme::TEXT).size(12.0))
            .wrap_mode(egui::TextWrapMode::Truncate),
    );
}

fn compute_dir_size(path: &str) -> (u64, u64) {
    let mut total_size = 0u64;
    let mut count = 0u64;
    let mut stack = vec![std::path::PathBuf::from(path)];
    while let Some(dir) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                count += 1;
                if let Ok(meta) = entry.metadata() {
                    if meta.is_dir() {
                        stack.push(entry.path());
                    } else {
                        total_size += meta.len();
                    }
                }
            }
        }
    }
    (total_size, count)
}

fn format_size_detailed(bytes: u64) -> String {
    if bytes == 0 {
        return "0 bytes".to_string();
    }
    let human = if bytes < 1024 {
        format!("{} bytes", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    };

    if bytes >= 1024 {
        format!("{} ({} bytes)", human, format_bytes_with_commas(bytes))
    } else {
        human
    }
}

fn format_bytes_with_commas(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

fn format_timestamp(ts: i64) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|dt| {
            let local = dt.with_timezone(&chrono::Local);
            local.format("%B %-d, %Y at %-I:%M %p").to_string()
        })
        .unwrap_or_else(|| "\u{2014}".to_string())
}

fn to_ts(t: SystemTime) -> i64 {
    t.duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
