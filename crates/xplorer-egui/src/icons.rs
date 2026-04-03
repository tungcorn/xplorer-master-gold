use eframe::egui;
use egui_phosphor::regular;

use crate::theme;

pub const HOME: &str = regular::HOUSE;
pub const DESKTOP: &str = regular::DESKTOP;
pub const DOCUMENTS: &str = regular::FILE_TEXT;
pub const DOWNLOADS: &str = regular::DOWNLOAD;
pub const DRIVE: &str = regular::HARD_DRIVES;
pub const STAR: &str = regular::STAR;
pub const BOOKMARK: &str = regular::BOOKMARK_SIMPLE;

pub const ARROW_LEFT: &str = regular::ARROW_LEFT;
pub const ARROW_RIGHT: &str = regular::ARROW_RIGHT;
pub const ARROW_UP: &str = regular::ARROW_UP;
pub const BREADCRUMB_SEP: &str = regular::CARET_RIGHT;

pub const CLOSE: &str = regular::X;
pub const PLUS: &str = regular::PLUS;
pub const FILTER: &str = regular::FUNNEL;
pub const FOLDER: &str = regular::FOLDER;
pub const FILE_GENERIC: &str = regular::FILE;
pub const FILE_CODE: &str = regular::FILE_CODE;
pub const FILE_IMAGE: &str = regular::FILE_IMAGE;
pub const FILE_TEXT: &str = regular::FILE_TEXT;

pub fn file_icon(ext: &str, is_dir: bool) -> (&'static str, egui::Color32) {
    if is_dir {
        return (FOLDER, theme::FOLDER_YELLOW);
    }
    match ext.to_lowercase().as_str() {
        "rs" | "toml" => (FILE_CODE, egui::Color32::from_rgb(0xDE, 0xA5, 0x84)),
        "js" | "jsx" | "mjs" => (FILE_CODE, egui::Color32::from_rgb(0xF7, 0xDF, 0x1E)),
        "ts" | "tsx" => (FILE_CODE, egui::Color32::from_rgb(0x31, 0x78, 0xC6)),
        "py" => (FILE_CODE, egui::Color32::from_rgb(0x35, 0x72, 0xA5)),
        "go" => (FILE_CODE, egui::Color32::from_rgb(0x00, 0xAD, 0xD8)),
        "c" | "h" => (FILE_CODE, egui::Color32::from_rgb(0x55, 0x55, 0xFF)),
        "cpp" | "hpp" | "cc" => (FILE_CODE, egui::Color32::from_rgb(0xF3, 0x4B, 0x7D)),
        "java" => (FILE_CODE, egui::Color32::from_rgb(0xB0, 0x72, 0x19)),
        "rb" => (FILE_CODE, egui::Color32::from_rgb(0xCC, 0x34, 0x2D)),
        "cs" => (FILE_CODE, egui::Color32::from_rgb(0x17, 0x8E, 0x00)),
        "swift" => (FILE_CODE, egui::Color32::from_rgb(0xF0, 0x52, 0x38)),
        "html" | "htm" => (FILE_CODE, egui::Color32::from_rgb(0xE3, 0x4C, 0x26)),
        "css" | "scss" | "sass" | "less" => (FILE_CODE, egui::Color32::from_rgb(0x56, 0x3D, 0x7C)),
        "json" | "yaml" | "yml" | "xml" | "ini" | "env" | "conf" | "cfg" => {
            (FILE_CODE, theme::SECONDARY)
        }
        "sh" | "bash" | "zsh" | "fish" | "ps1" | "bat" | "cmd" => (FILE_CODE, theme::SECONDARY),

        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "webp" | "ico" | "tiff" => {
            (FILE_IMAGE, egui::Color32::from_rgb(0xA8, 0x5D, 0xDB))
        }

        "mp3" | "wav" | "flac" | "ogg" | "aac" | "wma" | "m4a" => {
            (FILE_GENERIC, egui::Color32::from_rgb(0x1D, 0xB9, 0x54))
        }

        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" => {
            (FILE_GENERIC, egui::Color32::from_rgb(0xE5, 0x3E, 0x3E))
        }

        "pdf" => (FILE_GENERIC, egui::Color32::from_rgb(0xDB, 0x44, 0x37)),
        "doc" | "docx" | "odt" | "rtf" => (FILE_GENERIC, egui::Color32::from_rgb(0x29, 0x5D, 0xAA)),
        "xls" | "xlsx" | "csv" | "ods" => (FILE_GENERIC, egui::Color32::from_rgb(0x20, 0x7C, 0x45)),

        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "zst" => {
            (FILE_GENERIC, egui::Color32::from_rgb(0xDA, 0x8A, 0x29))
        }

        "txt" | "md" | "log" | "readme" | "license" => (FILE_TEXT, theme::SECONDARY),

        "exe" | "msi" | "dll" | "so" | "dylib" => {
            (FILE_GENERIC, egui::Color32::from_rgb(0xE0, 0x70, 0x70))
        }

        _ => (FILE_GENERIC, theme::MUTED),
    }
}
