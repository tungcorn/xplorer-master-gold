use std::path::Path;

use eframe::egui;

use crate::state::Tab;
use crate::theme;

const MAX_PREVIEW_BYTES: u64 = 512 * 1024;
const MAX_PREVIEW_LINES: usize = 200;
const PANEL_WIDTH: f32 = 340.0;

struct PreviewInfo {
    name: String,
    path: String,
    is_dir: bool,
    size: u64,
    file_type: String,
    modified: i64,
}

pub fn show_for_tab(ui: &mut egui::Ui, tab: &Tab) {
    let selected: Vec<usize> = tab.selected_set.iter().copied().collect();

    match selected.len() {
        0 => show_empty(ui),
        1 => {
            if let Some(entry) = tab.entries.get(selected[0]) {
                let info = PreviewInfo {
                    name: entry.name.clone(),
                    path: entry.path.clone(),
                    is_dir: entry.is_dir,
                    size: entry.size,
                    file_type: entry.file_type.clone(),
                    modified: entry.modified,
                };
                show_single(ui, &info);
            }
        }
        n => show_multi(ui, tab, n),
    }
}

fn show_empty(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() / 3.0);
        ui.label(
            egui::RichText::new("No selection")
                .color(theme::MUTED)
                .size(13.0),
        );
    });
}

fn show_multi(ui: &mut egui::Ui, tab: &Tab, count: usize) {
    let total_size: u64 = tab
        .selected_set
        .iter()
        .filter_map(|&i| tab.entries.get(i))
        .filter(|e| !e.is_dir)
        .map(|e| e.size)
        .sum();
    let dir_count = tab
        .selected_set
        .iter()
        .filter_map(|&i| tab.entries.get(i))
        .filter(|e| e.is_dir)
        .count();
    let file_count = count - dir_count;

    ui.vertical(|ui| {
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(format!("{} items selected", count))
                .color(theme::TEXT)
                .size(14.0)
                .strong(),
        );
        ui.add_space(4.0);
        if file_count > 0 {
            detail_row(ui, "Files", &file_count.to_string());
        }
        if dir_count > 0 {
            detail_row(ui, "Folders", &dir_count.to_string());
        }
        if total_size > 0 {
            detail_row(ui, "Total size", &format_size(total_size));
        }
    });
}

fn show_single(ui: &mut egui::Ui, info: &PreviewInfo) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(8.0);

        ui.label(
            egui::RichText::new(&info.name)
                .color(theme::TEXT)
                .size(14.0)
                .strong(),
        );
        ui.add_space(2.0);

        let type_label = if info.is_dir {
            "Folder".to_string()
        } else {
            let ext = info.file_type.trim_start_matches('.').to_uppercase();
            if ext.is_empty() {
                "File".to_string()
            } else {
                ext
            }
        };
        ui.label(
            egui::RichText::new(type_label)
                .color(theme::SELECTION)
                .size(11.0)
                .background_color(egui::Color32::from_rgba_premultiplied(122, 162, 247, 30)),
        );

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        if !info.is_dir {
            detail_row(ui, "Size", &format_size(info.size));
        }
        detail_row(ui, "Modified", &format_timestamp(info.modified));
        detail_row(ui, "Path", &info.path);

        ui.add_space(8.0);

        if !info.is_dir {
            show_content_preview(ui, &info.path, info.size);
        }
    });
}

fn show_content_preview(ui: &mut egui::Ui, path: &str, size: u64) {
    let ext = Path::new(path)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    if is_image_ext(&ext) {
        ui.separator();
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("Preview")
                .color(theme::SECONDARY)
                .size(11.0),
        );
        ui.add_space(4.0);
        let uri = format!("file://{}", path.replace('\\', "/"));
        let image = egui::Image::new(uri)
            .max_width(ui.available_width())
            .max_height(300.0)
            .corner_radius(4.0);
        ui.add(image);
        return;
    }

    if is_text_ext(&ext) && size <= MAX_PREVIEW_BYTES {
        if let Ok(content) = std::fs::read_to_string(path) {
            ui.separator();
            ui.add_space(4.0);
            let total_lines = content.lines().count();
            let preview_lines: String = content
                .lines()
                .take(MAX_PREVIEW_LINES)
                .collect::<Vec<_>>()
                .join("\n");

            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Content")
                        .color(theme::SECONDARY)
                        .size(11.0),
                );
                if total_lines > MAX_PREVIEW_LINES {
                    ui.label(
                        egui::RichText::new(format!(
                            "({} of {} lines)",
                            MAX_PREVIEW_LINES, total_lines
                        ))
                        .color(theme::MUTED)
                        .size(10.0),
                    );
                }
            });
            ui.add_space(4.0);

            egui::Frame::new()
                .fill(egui::Color32::from_rgb(0x1a, 0x1b, 0x26))
                .corner_radius(4.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(&preview_lines)
                                .color(theme::SECONDARY)
                                .size(11.0)
                                .family(egui::FontFamily::Monospace),
                        )
                        .wrap(),
                    );
                });
            return;
        }
    }

    if size > MAX_PREVIEW_BYTES {
        ui.separator();
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("File too large to preview")
                .color(theme::MUTED)
                .size(11.0)
                .italics(),
        );
    }
}

fn detail_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).color(theme::MUTED).size(11.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add(
                egui::Label::new(egui::RichText::new(value).color(theme::TEXT).size(11.0))
                    .truncate(),
            );
        });
    });
}

fn is_image_ext(ext: &str) -> bool {
    matches!(
        ext,
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" | "ico"
    )
}

fn is_text_ext(ext: &str) -> bool {
    matches!(
        ext,
        "txt"
            | "md"
            | "rs"
            | "toml"
            | "json"
            | "yaml"
            | "yml"
            | "xml"
            | "html"
            | "htm"
            | "css"
            | "scss"
            | "sass"
            | "js"
            | "mjs"
            | "jsx"
            | "ts"
            | "tsx"
            | "py"
            | "go"
            | "c"
            | "h"
            | "cpp"
            | "hpp"
            | "cc"
            | "java"
            | "rb"
            | "cs"
            | "swift"
            | "kt"
            | "sh"
            | "bash"
            | "ps1"
            | "bat"
            | "cmd"
            | "sql"
            | "graphql"
            | "proto"
            | "env"
            | "ini"
            | "cfg"
            | "conf"
            | "log"
            | "csv"
            | "gitignore"
            | "dockerignore"
            | "editorconfig"
            | "lock"
            | "vue"
            | "svelte"
            | "astro"
            | "mdx"
    )
}

fn format_size(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    if bytes < 1024 {
        return format!("{} B", bytes);
    }
    if bytes < 1024 * 1024 {
        return format!("{:.1} KB", bytes as f64 / 1024.0);
    }
    if bytes < 1024 * 1024 * 1024 {
        return format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0));
    }
    format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
}

fn format_timestamp(ts: i64) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|dt| {
            let local = dt.with_timezone(&chrono::Local);
            local.format("%b %-d, %Y  %-I:%M %p").to_string()
        })
        .unwrap_or_else(|| "—".to_string())
}

pub const PREVIEW_PANEL_WIDTH: f32 = PANEL_WIDTH;
