use std::fs;
use std::path::Path;
use tokio::sync::Mutex;
use std::sync::Arc;

use super::files::*;

/// Recursively scans a directory and builds a FileEntry tree.
/// 
/// - `root_dir`: The root directory for relative path calculation.
/// - `path`: The current file or directory to scan.
/// 
/// Returns `Some(FileEntry)` if successful, or `None` if the entry should be skipped.
pub fn scan_dir<P: AsRef<Path>>(root_dir: &Path, path: P) -> Option<FileEntry> {
    let path = path.as_ref();
    let is_browser_supported = is_browser_supported(path);
    let name = path.file_name()?.to_str()?.to_string();

    // Skip Windows alternate data streams and similar artifacts
    if name.contains("Zone.Identifier") {
        return None;
    }

    // Compute the path relative to the root directory for API/UI use
    let rel_path = path.strip_prefix(root_dir).unwrap_or(path);
    let path_str = rel_path.display().to_string();

    
    let size = get_file_size(path);
    let modified = get_modified_time(path);

    let metadata = fs::metadata(path).ok()?;
    let is_dir = metadata.is_dir();
    let file_type = detect_file_type(path, is_dir);

    if is_dir {
        // If directory, recursively scan its components
        let mut children = vec![];
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if let Some(child) = scan_dir(root_dir, entry_path) {
                    children.push(child);
                }
            }
        }

        Some(FileEntry {
            name,
            path: path_str,
            is_dir,
            file_type,
            children: Some(children),
            size,
            modified,
            is_browser_supported,
            lock: Arc::new(Mutex::new(())), // Per-entry lock for concurrency
        })
    } else {
        // If file, just create the entry
        Some(FileEntry {
            name,
            path: path_str,
            is_dir,
            file_type,
            children: None,
            size,
            modified,
            is_browser_supported,
            lock: Arc::new(Mutex::new(())), // Per-entry lock for concurrency
        })
    }
}