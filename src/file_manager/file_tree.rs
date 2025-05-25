use serde::Serialize;
use std::fs;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::path::Path;
use super::file_utils::*;

/// Represents a file or directory in the media tree.
#[derive(Debug, Serialize, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub file_type: FileType,
    pub size: Option<u64>,
    pub modified: Option<String>,
    pub children: Option<Vec<FileEntry>>,
    pub is_browser_supported: bool,
    #[serde(skip_serializing)] // Used for locking, not sent to client
    pub lock: Arc<Mutex<()>>,
    in_use : bool,
}

/// Enum for categorizing file types.
#[derive(Debug, PartialEq, Eq, Clone, Copy,Serialize)]
pub enum FileType {
    Video,
    Audio,
    Image,
    Other,
}

pub fn scan_dir<P: AsRef<Path>>(root_dir: &Path, curr_path: P) -> Option<FileEntry> {
    let curr_path = curr_path.as_ref();
    let is_browser_supported = is_browser_supported(curr_path);
    let name = curr_path.file_name()?.to_str()?.to_string();

    // Skip Windows alternate data streams and similar artifacts
    if name.contains("Zone.Identifier") || name == ".gitkeep" {
        return None;
    }

    // Compute the curr_path relative to the root directory for API/UI use
    let rel_path = curr_path.strip_prefix(root_dir).unwrap_or(curr_path);
    let path_str = rel_path.display().to_string();

    
    let size = get_file_size(curr_path);
    let modified = get_modified_time(curr_path);

    let metadata = fs::metadata(curr_path).ok()?;
    let is_dir = metadata.is_dir();
    let file_type = detect_file_type(curr_path, is_dir);

    if is_dir {
        // If directory, recursively scan its components
        let mut children = vec![];
        if let Ok(entries) = fs::read_dir(curr_path) {
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
            lock: Arc::new(Mutex::new(())),
            in_use: false,
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
            in_use: false,
        })  
    }
    
}


