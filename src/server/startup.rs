use std::net::SocketAddr;
use tracing::info;
use super::routes;
use std::sync::{Arc, Mutex};
use crate::media::scanner::scan_dir;
use crate::utils::config::Config;
use std::path::Path;
use tower_http::limit::RequestBodyLimitLayer;

pub async fn start_server() {
    // Load config from environment
    let config = Config::from_env();

    // Use media_dir from config
    let media_dir = config.media_dir.clone();
    let media_dir_path = Path::new(&media_dir);

    let media_tree = Arc::new(Mutex::new(scan_dir(media_dir_path, media_dir_path)));
    let watcher_tree = media_tree.clone();
    let media_dir_for_watcher = media_dir.clone();

    std::thread::spawn(move || {
        crate::media::watch::start_watcher(watcher_tree, &media_dir_for_watcher);
    });

    let app = routes::create_router(media_tree.clone())
        .layer(RequestBodyLimitLayer::new(1024 * 1024 * 1024)); // 1 GB
    let port: u16 = config.port.parse().unwrap_or(3000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("🚀 Server starting on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app.into_make_service())
        .await
        .unwrap();
}