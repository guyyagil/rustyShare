use notify::{RecommendedWatcher, RecursiveMode, Watcher, EventKind};
use std::sync::{Arc, mpsc::channel};
use tokio::sync::{Mutex, broadcast};
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
pub async fn start_watcher(
    tree: Arc<Mutex<Option<FileEntry>>>,
    dir: &str,
    tree_tx: broadcast::Sender<()>,
) {
    let dir = dir.to_string();
    let dir_for_thread = dir.clone();
    let (tx, rx) = channel();

    // Clone tree_tx for the thread
    let tree_tx_thread = tree_tx.clone();

    // Spawn the blocking watcher in a separate thread
    std::thread::spawn(move || {
        let mut watcher: RecommendedWatcher = RecommendedWatcher::new(
            move |res| {
                if let Err(e) = tx.send(res) {
                    error!("Watcher error sending event: {}", e);
                }
            },
            notify::Config::default(),
        ).expect("Failed to create watcher");

        watcher
            .watch(Path::new(&dir_for_thread), RecursiveMode::Recursive)
            .expect("Failed to watch media directory");

        info!("📡 Watching media directory for changes...");

        for res in rx {
            if let Ok(event) = res {
                if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)) {
                    info!("🔄 Change detected: rescanning media directory...");
                    // Instead of updating the tree here, just notify the async runtime
                    if let Err(e) = tree_tx_thread.send(()) {
                        error!("Failed to send tree update notification: {e}");
                    }
                }
            } else if let Err(e) = res {
                error!("Watcher channel error: {:?}", e);
            }
        }
    });

    // Async task: listen for notifications and update the tree
    let mut rx = tree_tx.subscribe();
    loop {
        if rx.recv().await.is_ok() {
            let new_tree = scan_dir(Path::new(&dir), Path::new(&dir));
            let mut tree_lock = tree.lock().await;
            *tree_lock = new_tree;
            info!("✅ Media tree updated.");
        }
    }
}





