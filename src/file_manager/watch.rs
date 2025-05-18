use notify::{RecommendedWatcher, RecursiveMode, Watcher, EventKind};
use std::sync::{Arc, mpsc::channel};
use tokio::sync::Mutex;
use crate::file_manager::{scanner::scan_dir, files::FileEntry};
use std::path::Path;
use tracing::{info, error};
//watching and handling file changes in the media director
//rescanning the directory when changes are detected
pub async fn start_watcher(media_tree: Arc<Mutex<Option<FileEntry>>>, media_dir: &str) {
    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = RecommendedWatcher::new(
        move |res| {
            if let Err(e) = tx.send(res) {
                error!("Watcher error sending event: {}", e);
            }
        },
        notify::Config::default(),
    ).expect("Failed to create watcher");

    watcher
        .watch(Path::new(media_dir), RecursiveMode::Recursive)
        .expect("Failed to watch media directory");

    info!("📡 Watching media directory for changes...");

    for res in rx {
        if let Ok(event) = res {
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
