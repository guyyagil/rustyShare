use notify::{RecommendedWatcher, RecursiveMode, Watcher, EventKind};
use std::sync::{Arc, mpsc::channel};
use tokio::sync::Mutex;
use crate::file_manager::{file_tree::scan_dir, file_tree::FileEntry};
use std::path::Path;
use tracing::{info, error};

/// Starts a background watcher on the media directory.
/// 
/// When a file or directory is created, modified, or removed,
/// the entire media tree is rescanned and updated in memory.
/// 
/// # Arguments
/// * `files_tree` - Shared, mutable reference to the in-memory file tree.
/// * `dir` - Path to the directory to watch.
pub async fn start_watcher(tree: Arc<Mutex<Option<FileEntry>>>, dir: &str) {
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
        .watch(Path::new(dir), RecursiveMode::Recursive)
        .expect("Failed to watch media directory");

    info!("📡 Watching media directory for changes...");

    // Listen for file system events and rescan on relevant changes
    for res in rx {
        if let Ok(event) = res {
            // Only rescan on create, modify, or remove events
            if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)) {
                info!("🔄 Change detected: rescanning media directory...");
                let new_tree = scan_dir(Path::new(dir), Path::new(dir));
                let mut tree_lock = tree.lock().await;
                *tree_lock = new_tree;
                info!("✅ Media tree updated.");
            }
        } else if let Err(e) = res {
            error!("Watcher channel error: {:?}", e);
        }
    }
}
