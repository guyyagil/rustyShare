use std::net::SocketAddr;
use tracing::info;
use super::routes;
use std::sync::{Arc, Mutex};
use crate::fileManager::scanner::scan_dir;
use crate::utils::config::Config;
use std::path::Path;


pub async fn start_server() {
    // Load config from environment
    let config = Config::from_env();

    // Use file_dir from config
    let file_dir = config.file_dir().to_owned();
    let fil_dir_path = Path::new(&file_dir);

    let file_tree = Arc::new(Mutex::new(scan_dir(fil_dir_path, fil_dir_path)));
    let watcher_tree = file_tree.clone();
    

    std::thread::spawn(move || {
        crate::fileManager::watch::start_watcher(watcher_tree, &(file_dir.clone()));
    });

    let app = routes::create_router(file_tree.clone());
    let port: u16 = config.port().parse().unwrap();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("🚀 Server starting on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app.into_make_service())
        .await
        .unwrap();
}