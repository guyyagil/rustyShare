
use serde::Serialize;
use mime_guess::{Mime,from_path};
use std::fs;
use chrono::{DateTime, Utc};
use tokio::fs::File;
use std::path::PathBuf;
use axum::{ response::{ IntoResponse, Response}, http::StatusCode};
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
}


#[derive(Debug, PartialEq, Eq, Clone, Copy,Serialize)]
pub enum FileType {
    Video,
    Audio,
    Image,
    Other,
}

use std::path::Path;

pub fn detect_file_type(path: &Path, is_dir: bool) -> FileType {
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

pub fn get_file_size(path: &Path) -> Option<u64> {
    fs::metadata(path).ok().map(|m| m.len())
}

pub fn get_modified_time(path: &Path) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    let datetime: DateTime<Utc> = modified.into();
    Some(datetime.to_rfc3339())
}

pub fn get_mime_type<P: AsRef<Path>>(path: P) -> Mime {
    from_path(path.as_ref()).first_or_octet_stream()
}

pub async fn file_size(f :&File) -> u64 {
    match f.metadata().await {
        Ok(meta) => meta.len(),
        Err(_) => 0,
    }
}
pub fn safe_path<P: AsRef<Path>>(path: P) -> Result<PathBuf, Response> {
    let media_dir = std::fs::canonicalize("media")
        .unwrap_or_else(|_| PathBuf::from("media"));

    let joined_path = media_dir.join(path.as_ref());

    match joined_path.canonicalize() {
        Ok(canonical) if canonical.starts_with(&media_dir) => Ok(canonical),
        _ => Err((StatusCode::FORBIDDEN, "Forbidden").into_response()),
    }
}
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
        "doc" | "docx" | "xls" | "xlsx"
    )
}