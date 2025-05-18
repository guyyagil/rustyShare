use serde::Serialize;
use mime_guess::{Mime,from_path};
use std::fs;
use chrono::{DateTime, Utc};
use tokio::fs::File;
use std::path::PathBuf;
use tokio::sync::Mutex;
use std::sync::Arc;
use axum::{ 
    response::{ IntoResponse, Response}, 
    http::StatusCode};
use crate::utils::config::Config;
use std::path::Path;

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
    pub lock: Arc<Mutex<()>>
}

/// Enum for categorizing file types.
#[derive(Debug, PartialEq, Eq, Clone, Copy,Serialize)]
pub enum FileType {
    Video,
    Audio,
    Image,
    Other,
}

//------------------- Helper functions to get file properties -------------------//

/// Detects the file type based on extension.
pub fn detect_file_type<P : AsRef<Path>>(path: P, is_dir: bool) -> FileType {
    let path = path.as_ref();
    if is_dir {
        FileType::Other
    } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        match ext.to_lowercase().as_str() {
            "mp4" | "mkv" | "avi" | "mov" => FileType::Video,
            "mp3" | "flac" | "wav" | "aac" => FileType::Audio,
            "jpg" | "jpeg" | "png" | "gif" => FileType::Image,
            _ => FileType::Other,
        }
    } else {
        FileType::Other
    }
}

/// Returns the file size in bytes, or None if not available.
pub fn get_file_size<P : AsRef<Path>>(path: P) -> Option<u64> {
    let path = path.as_ref();
    fs::metadata(path).ok().map(|m| m.len())
}

/// Returns the last modified time as an RFC3339 string, or None if not available.
pub fn get_modified_time<P :AsRef<Path>>(path: P) -> Option<String> {
    let path = path.as_ref();
    let metadata = fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    let datetime: DateTime<Utc> = modified.into();
    Some(datetime.to_rfc3339())
}

/// Returns the MIME type for a file path.
pub fn get_mime_type<P: AsRef<Path>>(path: P) -> Mime {
    from_path(path.as_ref()).first_or_octet_stream()
}

/// Async: Returns the file size for an open file handle.
pub async fn file_size(f :&File) -> u64 {
    match f.metadata().await {
        Ok(meta) => meta.len(),
        Err(_) => 0,
    }
}

/// Ensures a given path is safe and within the configured master directory.
pub fn safe_path<P: AsRef<Path>>(path: P) -> Result<PathBuf, Response> {
    let config = Config::from_env();
    let master_dir = PathBuf::from(config.file_dir());
    let joined_path = master_dir.join(path.as_ref());

    let master_canon = master_dir.canonicalize().map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response()
    })?;

    // If the folder exists, great — but use full path canonicalization if available
    match joined_path.canonicalize() {
        Ok(p) => {
            if p.starts_with(&master_canon) {
                Ok(joined_path)
            } else {
                Err((StatusCode::FORBIDDEN, "Path escape blocked").into_response())
            }
        }
        Err(_) => {
            // If canonicalization fails (e.g., file doesn't exist yet), still allow it if the parent exists
            if let Some(parent) = joined_path.parent() {
                if parent.exists() && parent.starts_with(&master_dir) {
                    return Ok(joined_path);
                }
            }
            Err((StatusCode::BAD_REQUEST, "Target folder doesn't exist").into_response())
        }
    }
}

/// Checks if a file extension is supported for browser preview.
pub fn is_browser_supported<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    matches!(
        ext.as_str(),
        // Video
        "mp4" | "webm" | "mov" |
        // Audio
        "mp3" | "ogg" | "wav" | "flac" | "aac" |
        // Images
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" |
        // Documents commonly supported by browsers
        "pdf" | "txt" | "csv" | "html" | "htm" | "md" |
        // Office docs (modern browsers with plugins or Google Docs viewer)
        "doc" | "xls" | "xlsx"
    )
}

/// Recursively searches for a FileEntry by path in the tree.
/// Returns a clone of the entry if found.
pub fn find_entry(file_entry : &mut FileEntry, path: &str) -> Option<FileEntry> {
    if file_entry.path == path {
        return Some(file_entry.clone());
    }

    if let Some(children) = &mut file_entry.children {
        for child in children {
            if let Some(found) = find_entry(child, path) {
                return Some(found);
            }
        }
    }
    None
}