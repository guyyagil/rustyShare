use serde::Serialize;
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
use std::fs;
use chrono::{DateTime, Utc};

pub fn get_file_size(path: &Path) -> Option<u64> {
    fs::metadata(path).ok().map(|m| m.len())
}

pub fn get_modified_time(path: &Path) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    let datetime: DateTime<Utc> = modified.into();
    Some(datetime.to_rfc3339())
}

