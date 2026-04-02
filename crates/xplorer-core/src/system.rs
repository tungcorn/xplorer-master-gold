use crate::error::CoreError;
use crate::types::DriveInfo;
use std::path::Path;

/// Open a file or URL with the OS default application.
pub fn open_file(path: &Path) -> Result<(), CoreError> {
    if !path.exists() {
        return Err(CoreError::NotFound(path.display().to_string()));
    }
    open::that(path).map_err(|e| CoreError::Other(format!("Failed to open file: {}", e)))
}

/// Reveal a file in the system file manager (Explorer / Finder).
pub fn show_in_folder(path: &Path) -> Result<(), CoreError> {
    if !path.exists() {
        return Err(CoreError::NotFound(path.display().to_string()));
    }

    #[cfg(windows)]
    {
        let arg = format!("/select,{}", path.display());
        std::process::Command::new("explorer")
            .arg(&arg)
            .spawn()
            .map_err(|e| CoreError::Other(format!("Failed to show in folder: {}", e)))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &path.to_string_lossy()])
            .spawn()
            .map_err(|e| CoreError::Other(format!("Failed to show in folder: {}", e)))?;
    }

    #[cfg(target_os = "linux")]
    {
        let dir = if path.is_dir() {
            path.to_path_buf()
        } else {
            path.parent().unwrap_or(path).to_path_buf()
        };
        std::process::Command::new("xdg-open")
            .arg(dir)
            .spawn()
            .map_err(|e| CoreError::Other(format!("Failed to show in folder: {}", e)))?;
    }

    Ok(())
}

/// Check whether a file is hidden (dot-prefix or Windows hidden attribute).
pub fn is_hidden(path: &Path) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;

        if let Some(name) = path.file_name() {
            if name.to_string_lossy().starts_with('.') {
                return true;
            }
        }
        if let Ok(metadata) = std::fs::metadata(path) {
            const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
            return metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0;
        }
        false
    }

    #[cfg(not(windows))]
    {
        path.file_name()
            .map(|name| name.to_string_lossy().starts_with('.'))
            .unwrap_or(false)
    }
}

/// List available drives / mount points.
pub fn list_drives() -> Result<Vec<DriveInfo>, CoreError> {
    #[cfg(windows)]
    {
        list_drives_windows()
    }

    #[cfg(target_os = "macos")]
    {
        list_drives_macos()
    }

    #[cfg(target_os = "linux")]
    {
        list_drives_linux()
    }
}

#[cfg(windows)]
fn list_drives_windows() -> Result<Vec<DriveInfo>, CoreError> {
    #[link(name = "kernel32")]
    extern "system" {
        fn GetLogicalDrives() -> u32;
        fn GetDiskFreeSpaceExW(
            lpDirectoryName: *const u16,
            lpFreeBytesAvailableToCaller: *mut u64,
            lpTotalNumberOfBytes: *mut u64,
            lpTotalNumberOfFreeBytes: *mut u64,
        ) -> i32;
        fn GetVolumeInformationW(
            lpRootPathName: *const u16,
            lpVolumeNameBuffer: *mut u16,
            nVolumeNameSize: u32,
            lpVolumeSerialNumber: *mut u32,
            lpMaximumComponentLength: *mut u32,
            lpFileSystemFlags: *mut u32,
            lpFileSystemNameBuffer: *mut u16,
            nFileSystemNameSize: u32,
        ) -> i32;
    }

    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;

    let bitmask = unsafe { GetLogicalDrives() };
    let mut drives = Vec::new();

    for i in 0..26u32 {
        if bitmask & (1 << i) == 0 {
            continue;
        }
        let letter = (b'A' + i as u8) as char;
        let root = format!("{}:\\", letter);
        let wide: Vec<u16> = OsStr::new(&root).encode_wide().chain(once(0)).collect();

        let mut name_buf = [0u16; 256];
        let label = unsafe {
            let ok = GetVolumeInformationW(
                wide.as_ptr(),
                name_buf.as_mut_ptr(),
                name_buf.len() as u32,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
            );
            if ok != 0 {
                let len = name_buf
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(name_buf.len());
                let s = String::from_utf16_lossy(&name_buf[..len]);
                if s.is_empty() {
                    "Local Disk".to_string()
                } else {
                    s
                }
            } else {
                "Local Disk".to_string()
            }
        };

        let mut free_available: u64 = 0;
        let mut total: u64 = 0;
        let mut _total_free: u64 = 0;
        unsafe {
            GetDiskFreeSpaceExW(
                wide.as_ptr(),
                &mut free_available,
                &mut total,
                &mut _total_free,
            );
        };

        drives.push(DriveInfo {
            name: format!("{} ({}:)", label, letter),
            mount_point: root,
            total_space: total,
            available_space: free_available,
            drive_type: "local".to_string(),
        });
    }

    Ok(drives)
}

#[cfg(target_os = "macos")]
fn list_drives_macos() -> Result<Vec<DriveInfo>, CoreError> {
    let mut drives = vec![DriveInfo {
        name: "Macintosh HD".to_string(),
        mount_point: "/".to_string(),
        total_space: 0,
        available_space: 0,
        drive_type: "local".to_string(),
    }];
    if let Ok(entries) = std::fs::read_dir("/Volumes") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "Macintosh HD" {
                continue;
            }
            drives.push(DriveInfo {
                name: name.clone(),
                mount_point: entry.path().to_string_lossy().to_string(),
                total_space: 0,
                available_space: 0,
                drive_type: "external".to_string(),
            });
        }
    }
    Ok(drives)
}

#[cfg(target_os = "linux")]
fn list_drives_linux() -> Result<Vec<DriveInfo>, CoreError> {
    Ok(vec![DriveInfo {
        name: "Root".to_string(),
        mount_point: "/".to_string(),
        total_space: 0,
        available_space: 0,
        drive_type: "local".to_string(),
    }])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_hidden_dot_prefix() {
        let path = Path::new("/home/user/.hidden_file");
        assert!(is_hidden(path));
    }

    #[test]
    fn test_is_hidden_normal_file() {
        let path = Path::new("/home/user/normal.txt");
        assert!(!is_hidden(path));
    }

    #[test]
    fn test_list_drives_does_not_panic() {
        let result = list_drives();
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_open_file_not_found() {
        let path = Path::new("/nonexistent/file.txt");
        assert!(open_file(path).is_err());
    }

    #[test]
    fn test_show_in_folder_not_found() {
        let path = Path::new("/nonexistent/folder");
        assert!(show_in_folder(path).is_err());
    }
}
