use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;
pub mod models;
pub mod scanner;

/// # Safety
/// paths must point to valid UTF-8 JSON and each path must be a NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn photoselect_scan_paths_json(
    root: *const c_char,
    paths_json: *const c_char,
) -> *mut c_char {
    let result = std::panic::catch_unwind(|| {
        if root.is_null() || paths_json.is_null() {
            return Err("Missing root directory or file list".to_string());
        }
        let root = CStr::from_ptr(root).to_str().map_err(|error| error.to_string())?;
        let paths_json = CStr::from_ptr(paths_json).to_str().map_err(|error| error.to_string())?;
        let entries: Vec<(String, u64)> = serde_json::from_str(paths_json).map_err(|error| error.to_string())?;
        let files: Vec<(std::path::PathBuf, u64)> = entries.into_iter()
            .map(|(path, size)| (std::path::PathBuf::from(path), size))
            .collect();
        scanner::scan_file_paths(Path::new(root), &files).map_err(|error| error.to_string())
    });
    let value = match result {
        Ok(Ok(summary)) => serde_json::json!({ "data": summary }),
        Ok(Err(error)) => serde_json::json!({ "error": error }),
        Err(_) => serde_json::json!({ "error": "Scanner panicked" }),
    };
    CString::new(value.to_string()).expect("JSON result must not contain NUL").into_raw()
}

pub fn scan(path: &str, recursive: bool) -> Result<models::ScanSummary, String> {
    let directory = Path::new(path);
    std::fs::read_dir(directory).map_err(|error| error.to_string())?;
    scanner::DirectoryScanner::scan(directory, recursive).map_err(|error| error.to_string())
}

/// # Safety
/// path must point to a valid NUL-terminated UTF-8 string for this call.
#[no_mangle]
pub unsafe extern "C" fn photoselect_scan(path: *const c_char, recursive: bool) -> *mut c_char {
    let result = std::panic::catch_unwind(|| {
        if path.is_null() { return Err("Missing directory".to_string()); }
        let path = CStr::from_ptr(path).to_str().map_err(|error| error.to_string())?;
        scan(path, recursive)
    });
    let value = match result {
        Ok(Ok(summary)) => serde_json::json!({ "data": summary }),
        Ok(Err(error)) => serde_json::json!({ "error": error }),
        Err(_) => serde_json::json!({ "error": "Scanner panicked" }),
    };
    CString::new(value.to_string()).expect("JSON result must not contain NUL").into_raw()
}

/// # Safety
/// value must be a pointer returned by photoselect_scan, freed exactly once.
#[no_mangle]
pub unsafe extern "C" fn photoselect_free(value: *mut c_char) {
    if !value.is_null() { drop(CString::from_raw(value)); }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pairs_sorts_and_leaves_source_files_untouched() {
        let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!("photoselect-harmony-{nonce}"));
        std::fs::create_dir(&root).unwrap();
        for name in ["IMG10.jpg", "IMG2.JPG", "IMG2.NEF", "IMG2.xmp", "orphan.xmp"] {
            std::fs::write(root.join(name), b"fixture").unwrap();
        }
        let result = scan(root.to_str().unwrap(), false).unwrap();
        assert_eq!(result.total_groups, 2);
        assert_eq!(result.raw_plus_jpg_count, 1);
        assert_eq!(result.groups[0].base_name, "IMG2");
        assert!(result.groups[0].xmp_sidecar.is_some());
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 5);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn invalid_path_is_rejected() {
        assert!(scan("", false).is_err());
        unsafe {
            let pointer = photoselect_scan(std::ptr::null(), false);
            assert!(CStr::from_ptr(pointer).to_str().unwrap().contains("error"));
            photoselect_free(pointer);
        }
    }

    #[test]
    fn picker_uri_file_list_keeps_provider_paths_and_pairs_images() {
        let entries = vec![
            (std::path::PathBuf::from("file:///authorized/IMG_10.JPG"), 120),
            (std::path::PathBuf::from("file:///authorized/IMG_2.JPG"), 100),
            (std::path::PathBuf::from("file:///authorized/IMG_2.NEF"), 900),
            (std::path::PathBuf::from("file:///authorized/IMG_2.XMP"), 25),
            (std::path::PathBuf::from("file:///authorized/readme.txt"), 7),
        ];
        let summary = scanner::scan_file_paths(Path::new("file:///authorized"), &entries).unwrap();
        assert_eq!(summary.total_groups, 2);
        assert_eq!(summary.raw_plus_jpg_count, 1);
        assert_eq!(summary.groups[0].base_name, "IMG_2");
        assert!(summary.groups[0].std_image.as_ref().unwrap().to_string_lossy().starts_with("file://"));
        assert!(summary.groups[0].xmp_sidecar.is_some());
    }
}
