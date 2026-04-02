use crate::error::CoreError;
use crate::types::Bookmark;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex, MutexGuard};

static BOOKMARKS_CACHE: LazyLock<Mutex<Option<Vec<Bookmark>>>> = LazyLock::new(|| Mutex::new(None));

/// Return the path to the bookmarks JSON file in the app data directory.
fn bookmarks_path() -> Result<PathBuf, CoreError> {
    let dir = dirs::data_dir()
        .ok_or_else(|| CoreError::Other("Failed to get app data directory".to_string()))?
        .join("xplorer-egui");
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("bookmarks.json"))
}

fn load_from_disk() -> Result<Vec<Bookmark>, CoreError> {
    let path = bookmarks_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = std::fs::read_to_string(&path)?;
    let bookmarks: Vec<Bookmark> = serde_json::from_str(&data)
        .map_err(|e| CoreError::Other(format!("Failed to parse bookmarks: {}", e)))?;
    Ok(bookmarks)
}

fn ensure_cache() -> Result<MutexGuard<'static, Option<Vec<Bookmark>>>, CoreError> {
    let mut guard = BOOKMARKS_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if guard.is_none() {
        let data = load_from_disk()?;
        *guard = Some(data);
    }
    Ok(guard)
}

fn flush_to_disk(guard: &MutexGuard<'static, Option<Vec<Bookmark>>>) -> Result<(), CoreError> {
    let bookmarks = guard
        .as_ref()
        .ok_or_else(|| CoreError::Other("Bookmarks cache not initialized".to_string()))?;
    let path = bookmarks_path()?;
    let data = serde_json::to_string_pretty(bookmarks)
        .map_err(|e| CoreError::Other(format!("Failed to serialize bookmarks: {}", e)))?;
    std::fs::write(&path, data)?;
    Ok(())
}

/// Get all bookmarks.
pub fn get_bookmarks() -> Result<Vec<Bookmark>, CoreError> {
    let guard = ensure_cache()?;
    Ok(guard.as_ref().cloned().unwrap_or_default())
}

/// Add a bookmark. Deduplicates by path.
pub fn add_bookmark(name: String, path: String) -> Result<Bookmark, CoreError> {
    let bookmark = Bookmark {
        name,
        path: path.clone(),
    };
    let mut guard = ensure_cache()?;
    let bookmarks = guard
        .as_mut()
        .ok_or_else(|| CoreError::Other("Bookmarks cache not initialized".to_string()))?;
    bookmarks.retain(|b| b.path != path);
    bookmarks.push(bookmark.clone());
    flush_to_disk(&guard)?;
    Ok(bookmark)
}

/// Remove a bookmark by path.
pub fn remove_bookmark(path: &str) -> Result<(), CoreError> {
    let mut guard = ensure_cache()?;
    let bookmarks = guard
        .as_mut()
        .ok_or_else(|| CoreError::Other("Bookmarks cache not initialized".to_string()))?;
    bookmarks.retain(|b| b.path != path);
    flush_to_disk(&guard)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bookmarks_round_trip() {
        let _ = remove_bookmark("/test/bookmark");

        let bm = add_bookmark("Test".to_string(), "/test/bookmark".to_string()).unwrap();
        assert_eq!(bm.name, "Test");
        assert_eq!(bm.path, "/test/bookmark");

        let all = get_bookmarks().unwrap();
        assert!(all.iter().any(|b| b.path == "/test/bookmark"));

        remove_bookmark("/test/bookmark").unwrap();
        let all = get_bookmarks().unwrap();
        assert!(!all.iter().any(|b| b.path == "/test/bookmark"));
    }
}
