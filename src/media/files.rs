pub use super::file_type::FileType;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub file_type: FileType,
    pub size: Option<u64>,
    pub modified: Option<String>,
    pub children: Option<Vec<FileEntry>>,
}

