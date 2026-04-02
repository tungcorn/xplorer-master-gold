use std::path::Path;

/// Determine a broad file type category from the file extension.
pub fn get_file_type(path: &Path, is_dir: bool) -> String {
    if is_dir {
        return "directory".to_string();
    }
    match path.extension().and_then(|e| e.to_str()) {
        Some("txt") | Some("md") | Some("log") | Some("csv") | Some("ini") | Some("cfg")
        | Some("toml") | Some("yaml") | Some("yml") | Some("xml") => "text".to_string(),
        Some("rs") | Some("ts") | Some("tsx") | Some("js") | Some("jsx") | Some("py")
        | Some("go") | Some("c") | Some("cpp") | Some("h") | Some("java") | Some("cs")
        | Some("rb") | Some("php") | Some("swift") | Some("kt") | Some("lua") | Some("sh")
        | Some("bat") | Some("ps1") | Some("html") | Some("css") | Some("scss") | Some("json") => {
            "code".to_string()
        }
        Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("bmp") | Some("webp")
        | Some("svg") | Some("ico") => "image".to_string(),
        Some("mp4") | Some("avi") | Some("mkv") | Some("mov") | Some("wmv") | Some("flv") => {
            "video".to_string()
        }
        Some("mp3") | Some("wav") | Some("flac") | Some("aac") | Some("ogg") | Some("wma") => {
            "audio".to_string()
        }
        Some("pdf") => "pdf".to_string(),
        Some("doc") | Some("docx") | Some("odt") | Some("rtf") => "document".to_string(),
        Some("xls") | Some("xlsx") | Some("ods") => "spreadsheet".to_string(),
        Some("ppt") | Some("pptx") | Some("odp") => "presentation".to_string(),
        Some("zip") | Some("rar") | Some("7z") | Some("tar") | Some("gz") | Some("bz2")
        | Some("xz") | Some("zst") => "archive".to_string(),
        Some("exe") | Some("msi") | Some("deb") | Some("rpm") | Some("dmg") | Some("appimage") => {
            "executable".to_string()
        }
        Some("dll") | Some("so") | Some("dylib") => "library".to_string(),
        Some("iso") | Some("img") => "disk_image".to_string(),
        Some("ttf") | Some("otf") | Some("woff") | Some("woff2") => "font".to_string(),
        Some("db") | Some("sqlite") | Some("sqlite3") => "database".to_string(),
        _ => "file".to_string(),
    }
}

/// Get the MIME type for common file extensions.
pub fn get_mime_type(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_str()?;
    let mime = match ext {
        "txt" => "text/plain",
        "md" => "text/markdown",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        "csv" => "text/csv",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "bmp" => "image/bmp",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "avi" => "video/x-msvideo",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "flac" => "audio/flac",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "gz" => "application/gzip",
        "tar" => "application/x-tar",
        "7z" => "application/x-7z-compressed",
        "rar" => "application/vnd.rar",
        "exe" => "application/x-msdownload",
        "wasm" => "application/wasm",
        _ => return None,
    };
    Some(mime.to_string())
}
