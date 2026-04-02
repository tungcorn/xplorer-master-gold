use crate::error::CoreError;
use crate::file_utils;
use crate::types::{system_time_to_timestamp, FileEntry};
use rayon::prelude::*;
use std::fs;
use std::path::Path;

/// Read a directory and return its entries.
/// Directories are sorted first, then files, both alphabetically by name.
pub async fn read_directory(path: &str) -> Result<Vec<FileEntry>, CoreError> {
    let path = path.to_string();
    tokio::task::spawn_blocking(move || read_directory_sync(&path))
        .await
        .map_err(|e| CoreError::Other(e.to_string()))?
}

/// Synchronous version of read_directory (runs on blocking thread).
fn read_directory_sync(path: &str) -> Result<Vec<FileEntry>, CoreError> {
    let path = Path::new(path);

    if !path.exists() {
        return Err(CoreError::NotFound(
            path.to_string_lossy().to_string(),
        ));
    }

    if !path.is_dir() {
        return Err(CoreError::Other(format!(
            "Path is not a directory: {}",
            path.display()
        )));
    }

    let raw_entries: Vec<_> = fs::read_dir(path)
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                CoreError::PermissionDenied(path.to_string_lossy().to_string())
            } else {
                CoreError::Io(e)
            }
        })?
        .filter_map(|e| e.ok())
        .collect();

    let mut files: Vec<FileEntry> = raw_entries
        .par_iter()
        .filter_map(|entry| {
            let entry_path = entry.path();
            let metadata = entry.metadata().ok()?;
            let name = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();
            let is_dir = metadata.is_dir();
            let file_type = file_utils::get_file_type(&entry_path, is_dir);
            let mime_type = file_utils::get_mime_type(&entry_path);
            let is_readonly = metadata.permissions().readonly();

            Some(FileEntry {
                name,
                path: entry_path.to_string_lossy().to_string(),
                is_dir,
                size: metadata.len(),
                modified: system_time_to_timestamp(
                    metadata
                        .modified()
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                ),
                file_type,
                mime_type,
                is_readonly,
            })
        })
        .collect();

    // Sort: directories first, then files, both alphabetically
    files.sort_by_cached_key(|f| (!f.is_dir, f.name.to_lowercase()));

    Ok(files)
}

/// Check if a path is a directory.
pub fn is_dir(path: &str) -> bool {
    Path::new(path).is_dir()
}

/// Create directories recursively (like `mkdir -p`).
pub async fn create_dir_recursive(path: &str) -> Result<(), CoreError> {
    let path = path.to_string();
    tokio::task::spawn_blocking(move || {
        fs::create_dir_all(Path::new(&path))?;
        Ok(())
    })
    .await
    .map_err(|e| CoreError::Other(e.to_string()))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_read_directory_current() {
        let entries = read_directory(".").await.unwrap();
        assert!(!entries.is_empty(), "current directory should have entries");
    }

    #[tokio::test]
    async fn test_read_directory_not_found() {
        let result = read_directory("__nonexistent_dir_12345__").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_read_directory_sorts_dirs_first() {
        let entries = read_directory(".").await.unwrap();
        let mut seen_file = false;
        for entry in &entries {
            if !entry.is_dir {
                seen_file = true;
            }
            if entry.is_dir && seen_file {
                panic!("Directory '{}' appeared after a file in the sorted list", entry.name);
            }
        }
    }

    #[tokio::test]
    async fn test_create_dir_recursive() {
        let temp = tempfile::tempdir().unwrap();
        let nested = temp.path().join("a").join("b").join("c");
        create_dir_recursive(nested.to_str().unwrap()).await.unwrap();
        assert!(nested.exists());
    }

    #[test]
    fn test_is_dir() {
        assert!(is_dir("."));
        assert!(!is_dir("__nonexistent__"));
    }
}
