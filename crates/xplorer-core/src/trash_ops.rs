use crate::error::CoreError;
use crate::types::TrashItem;
use std::path::Path;

/// Move a file or directory to the system trash/recycle bin.
pub fn move_to_trash(path: &Path) -> Result<(), CoreError> {
    if !path.exists() {
        return Err(CoreError::NotFound(path.display().to_string()));
    }
    trash::delete(path).map_err(|e| CoreError::Other(format!("Failed to move to trash: {}", e)))
}

/// List items currently in the system trash via the `trash` crate.
pub fn get_trash_items() -> Result<Vec<TrashItem>, CoreError> {
    let items = trash::os_limited::list()
        .map_err(|e| CoreError::Other(format!("Failed to list trash: {}", e)))?;

    let result = items
        .into_iter()
        .map(|item| {
            let name = item.name.to_string_lossy().to_string();
            let original_path = item
                .original_parent
                .join(&item.name)
                .to_string_lossy()
                .to_string();
            TrashItem {
                name,
                original_path,
                deleted_at: item.time_deleted,
                size: 0,
                is_dir: false,
            }
        })
        .collect();

    Ok(result)
}

/// Permanently delete all items in the system trash.
pub fn empty_trash() -> Result<(), CoreError> {
    let items = trash::os_limited::list()
        .map_err(|e| CoreError::Other(format!("Failed to list trash: {}", e)))?;
    if items.is_empty() {
        return Ok(());
    }
    trash::os_limited::purge_all(items)
        .map_err(|e| CoreError::Other(format!("Failed to empty trash: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::tempdir;

    #[test]
    fn test_move_to_trash_not_found() {
        let temp = tempdir().unwrap();
        let missing = temp.path().join("nonexistent.txt");
        assert!(move_to_trash(&missing).is_err());
    }

    #[test]
    fn test_get_trash_items_does_not_panic() {
        let _ = get_trash_items();
    }
}
