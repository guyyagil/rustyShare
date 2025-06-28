use std::net::SocketAddr;
use tracing::info;
use super::routing;
use tokio::sync::Mutex;
use std::sync::Arc;

use crate::file_manager::file_tree::scan_dir;
use crate::utils::config::Config;
use std::path::Path;
use tokio::sync::broadcast;

pub async fn start_server() {
    
    
    let config = Config::from_env();

    
    let file_dir = config.file_dir().to_owned();
    let fil_dir_path = Path::new(&file_dir);

    let file_tree = Arc::new(Mutex::new(scan_dir(fil_dir_path, fil_dir_path)));
    let watcher_tree = file_tree.clone();
    let (tree_tx, _) = broadcast::channel::<()>(16);
    // Start file watcher in a separate async task
    tokio::spawn({
        let file_dir = file_dir.clone();
        let tree_tx = tree_tx.clone();
        async move {
            crate::file_manager::tree_watcher::start_watcher(watcher_tree, &file_dir, tree_tx).await;
        }
    });

    let app = routing::create_router(file_tree.clone(), tree_tx);
    let port: u16 = config.port().parse().unwrap();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("🚀 Server starting on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app.into_make_service())
        .await
        .unwrap();
}