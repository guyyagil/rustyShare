use std::fs;
use std::path:: Path;

use super::files::*;


pub fn scan_dir<P: AsRef<Path>>(path: P) -> Option<FileEntry> {
    
    let path = path.as_ref();
    let is_browser_supported = is_browser_supported(path);
    let name = path.file_name()?.to_str()?.to_string();

    if name.contains("Zone.Identifier") {
        return None;
    }

    let rel_path = path.strip_prefix("media").unwrap_or(path);
    let path_str = rel_path.display().to_string();
    let size = get_file_size(path);
    let modified = get_modified_time(path);

    let metadata = fs::metadata(path).ok()?;
    let is_dir = metadata.is_dir();
    let file_type = detect_file_type(path, is_dir);

    if is_dir {
        let mut children = vec![];

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();

                if let Some(child) = scan_dir(entry_path) {
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
            is_browser_supported
        })
    } else {
        Some(FileEntry {
            name,
            path: path_str,
            is_dir,
            file_type,
            children: None,
            size,
            modified,
            is_browser_supported
        })
    }
}