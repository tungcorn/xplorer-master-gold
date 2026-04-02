use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    /// Unix timestamp in seconds (signed for chrono compatibility)
    pub modified: i64,
    pub file_type: String,
    pub mime_type: Option<String>,
    pub is_readonly: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub drive_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrashItem {
    pub name: String,
    pub original_path: String,
    pub deleted_at: i64,
    pub size: u64,
    pub is_dir: bool,
}

/// Convert a SystemTime to a Unix timestamp (seconds since epoch).
pub fn system_time_to_timestamp(time: SystemTime) -> i64 {
    time.duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
