use notify::{RecommendedWatcher, RecursiveMode, Watcher, EventKind};
use std::sync::{Arc, mpsc::channel};
use tokio::sync::Mutex;
use crate::file_manager::{scanner::scan_dir, files::FileEntry};
use std::path::Path;
use tracing::{info, error};

/// Starts a background watcher on the media directory.
/// 
/// When a file or directory is created, modified, or removed,
/// the entire media tree is rescanned and updated in memory.
/// 
/// # Arguments
/// * `media_tree` - Shared, mutable reference to the in-memory file tree.
/// * `media_dir` - Path to the directory to watch.
pub async fn start_watcher(media_tree: Arc<Mutex<Option<FileEntry>>>, media_dir: &str) {
    let (tx, rx) = channel();

    // Create a watcher that sends events to the channel
    let mut watcher: RecommendedWatcher = RecommendedWatcher::new(
        move |res| {
            if let Err(e) = tx.send(res) {
                error!("Watcher error sending event: {}", e);
            }
        },
        notify::Config::default(),
    ).expect("Failed to create watcher");

    // Start watching the media directory recursively
    watcher
        .watch(Path::new(media_dir), RecursiveMode::Recursive)
        .expect("Failed to watch media directory");

    info!("📡 Watching media directory for changes...");

    // Listen for file system events and rescan on relevant changes
    for res in rx {
        if let Ok(event) = res {
            // Only rescan on create, modify, or remove events
            if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)) {
                info!("🔄 Change detected: rescanning media directory...");
                let new_tree = scan_dir(Path::new(media_dir), Path::new(media_dir));
                let mut tree_lock = media_tree.lock().await;
                *tree_lock = new_tree;
                info!("✅ Media tree updated.");
            }
        } else if let Err(e) = res {
            error!("Watcher channel error: {:?}", e);
        }
    }
}
