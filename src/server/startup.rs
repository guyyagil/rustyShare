use std::net::SocketAddr;
use tracing::info;
use super::routes;
use tokio::sync::Mutex;
use std::sync::Arc;

use crate::file_manager::scanner::scan_dir;
use crate::utils::config::Config;
use std::path::Path;

pub async fn start_server() {
    
    
    let config = Config::from_env();

    
    let file_dir = config.file_dir().to_owned();
    let fil_dir_path = Path::new(&file_dir);

    let file_tree = Arc::new(Mutex::new(scan_dir(fil_dir_path, fil_dir_path)));
    let watcher_tree = file_tree.clone();
    
    // Start file watcher in a separate async task
    tokio::spawn(async move {
        crate::file_manager::watch::start_watcher(watcher_tree, &(file_dir.clone())).await;
    });

    let app = routes::create_router(file_tree.clone());
    let port: u16 = config.port().parse().unwrap();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("🚀 Server starting on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app.into_make_service())
        .await
        .unwrap();
}