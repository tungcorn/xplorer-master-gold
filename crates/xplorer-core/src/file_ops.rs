use crate::error::CoreError;
use std::fs;
use std::path::Path;

/// Copy a single file. Creates parent directories if needed.
pub fn copy_file(src: &Path, dst: &Path) -> Result<(), CoreError> {
    if !src.exists() {
        return Err(CoreError::NotFound(src.display().to_string()));
    }
    if !src.is_file() {
        return Err(CoreError::Other(format!(
            "Source is not a file: {}",
            src.display()
        )));
    }
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(src, dst)?;
    Ok(())
}

/// Copy a directory recursively. Skips symlinks for safety.
pub fn copy_dir(src: &Path, dst: &Path) -> Result<(), CoreError> {
    if !src.exists() {
        return Err(CoreError::NotFound(src.display().to_string()));
    }
    if !src.is_dir() {
        return Err(CoreError::Other(format!(
            "Source is not a directory: {}",
            src.display()
        )));
    }
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if let Ok(meta) = fs::symlink_metadata(&src_path) {
            if meta.file_type().is_symlink() {
                continue;
            }
        }

        if src_path.is_file() {
            fs::copy(&src_path, &dst_path)?;
        } else if src_path.is_dir() {
            copy_dir(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Move a file or directory. Tries rename first; falls back to copy+delete
/// for cross-filesystem moves.
pub fn move_entry(src: &Path, dst: &Path) -> Result<(), CoreError> {
    if !src.exists() {
        return Err(CoreError::NotFound(src.display().to_string()));
    }
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }

    if fs::rename(src, dst).is_ok() {
        return Ok(());
    }

    if src.is_file() {
        copy_file(src, dst)?;
        fs::remove_file(src)?;
    } else if src.is_dir() {
        copy_dir(src, dst)?;
        fs::remove_dir_all(src)?;
    }
    Ok(())
}

/// Rename a file or directory (same parent, new name).
pub fn rename(old: &Path, new: &Path) -> Result<(), CoreError> {
    if !old.exists() {
        return Err(CoreError::NotFound(old.display().to_string()));
    }
    fs::rename(old, new)?;
    Ok(())
}

/// Permanently delete a file.
pub fn delete_file(path: &Path) -> Result<(), CoreError> {
    if !path.exists() {
        return Err(CoreError::NotFound(path.display().to_string()));
    }
    if path.is_dir() {
        return Err(CoreError::Other(
            "Path is a directory, use delete_dir instead".to_string(),
        ));
    }
    fs::remove_file(path)?;
    Ok(())
}

/// Permanently delete a directory and all its contents.
pub fn delete_dir(path: &Path) -> Result<(), CoreError> {
    if !path.exists() {
        return Err(CoreError::NotFound(path.display().to_string()));
    }
    if !path.is_dir() {
        return Err(CoreError::Other("Path is not a directory".to_string()));
    }
    fs::remove_dir_all(path)?;
    Ok(())
}

/// Create an empty file, creating parent directories if needed.
pub fn create_file(path: &Path) -> Result<(), CoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::File::create(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::tempdir;

    #[test]
    fn test_copy_file_basic() {
        let temp = tempdir().unwrap();
        let src = temp.path().join("src.txt");
        let dst = temp.path().join("dst.txt");
        fs::write(&src, "hello").unwrap();

        copy_file(&src, &dst).unwrap();
        assert_eq!(fs::read_to_string(&dst).unwrap(), "hello");
    }

    #[test]
    fn test_copy_file_not_found() {
        let temp = tempdir().unwrap();
        let src = temp.path().join("missing.txt");
        let dst = temp.path().join("dst.txt");
        assert!(copy_file(&src, &dst).is_err());
    }

    #[test]
    fn test_copy_dir_recursive() {
        let temp = tempdir().unwrap();
        let src = temp.path().join("src_dir");
        let dst = temp.path().join("dst_dir");
        fs::create_dir_all(src.join("nested")).unwrap();
        fs::write(src.join("a.txt"), "aaa").unwrap();
        fs::write(src.join("nested").join("b.txt"), "bbb").unwrap();

        copy_dir(&src, &dst).unwrap();
        assert_eq!(fs::read_to_string(dst.join("a.txt")).unwrap(), "aaa");
        assert_eq!(
            fs::read_to_string(dst.join("nested").join("b.txt")).unwrap(),
            "bbb"
        );
    }

    #[test]
    fn test_move_entry_file() {
        let temp = tempdir().unwrap();
        let src = temp.path().join("move_src.txt");
        let dst = temp.path().join("move_dst.txt");
        fs::write(&src, "move me").unwrap();

        move_entry(&src, &dst).unwrap();
        assert!(!src.exists());
        assert_eq!(fs::read_to_string(&dst).unwrap(), "move me");
    }

    #[test]
    fn test_rename_basic() {
        let temp = tempdir().unwrap();
        let old = temp.path().join("old.txt");
        let new = temp.path().join("new.txt");
        fs::write(&old, "x").unwrap();

        rename(&old, &new).unwrap();
        assert!(!old.exists());
        assert!(new.exists());
    }

    #[test]
    fn test_delete_file_basic() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("del.txt");
        fs::write(&path, "bye").unwrap();

        delete_file(&path).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn test_delete_file_rejects_directory() {
        let temp = tempdir().unwrap();
        let dir = temp.path().join("a_dir");
        fs::create_dir(&dir).unwrap();

        assert!(delete_file(&dir).is_err());
    }

    #[test]
    fn test_delete_dir_basic() {
        let temp = tempdir().unwrap();
        let dir = temp.path().join("to_delete");
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(dir.join("sub").join("f.txt"), "data").unwrap();

        delete_dir(&dir).unwrap();
        assert!(!dir.exists());
    }

    #[test]
    fn test_create_file_basic() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("nested").join("new.txt");

        create_file(&path).unwrap();
        assert!(path.exists());
    }
}
