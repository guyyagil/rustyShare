use std::fs;
use std::path::{Path, PathBuf};


pub fn cache_path_for(original: &Path) -> PathBuf {
    let mut cache = PathBuf::from("cache");
    std::fs::create_dir_all(&cache).ok();
    let mut name = original.file_name().unwrap().to_os_string();
    name.push(".mp4");
    cache.push(name);
    cache
}

/// Cleans the cache directory if its total size exceeds `max_bytes`.
/// Deletes oldest files first.
pub fn clean_cache_space(max_bytes: u64) {
    let cache_dir = Path::new("cache");
    let entries: Vec<_> = match fs::read_dir(cache_dir) {
        Ok(entries) => entries.filter_map(Result::ok).collect(),
        Err(_) => return,
    };

    // Collect file metadata (path, modified time, size)
    let mut files: Vec<_> = entries.iter()
        .filter_map(|entry| {
            let path = entry.path();
            entry.metadata().ok().and_then(|meta| {
                if meta.is_file() {
                    Some((path, meta.modified().ok()?, meta.len()))
                } else {
                    None
                }
            })
        })
        .collect();

    // Sort by oldest modified first
    files.sort_by_key(|(_, modified, _)| *modified);

    // Calculate total size
    let mut total_size: u64 = files.iter().map(|(_, _, size)| *size).sum();

    // Remove oldest files until under limit
    for (path, _, size) in files {
        if total_size <= max_bytes {
            break;
        }
        let _ = fs::remove_file(&path);
        total_size = total_size.saturating_sub(size);
    }
}