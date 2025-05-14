use std::net::SocketAddr;
use tracing::info;
use super::routes;
use std::sync::{Arc, Mutex};
use crate::media::{scanner::scan_dir};
use tower_http::limit::RequestBodyLimitLayer;

pub async fn start_server() {
    let media_tree = Arc::new(Mutex::new(scan_dir("media")));
    let watcher_tree = media_tree.clone();
    
    std::thread::spawn(move || {
        crate::media::watch::start_watcher(watcher_tree);
    });
    
    let app = routes::create_router(media_tree.clone())
        .layer(RequestBodyLimitLayer::new(1024 * 1024 * 1024)); // 1 GB
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    info!("🚀 Server starting on {}", addr);
    axum::serve(  tokio::net::TcpListener::bind(addr).await.unwrap(), app.into_make_service() )
    .await
    .unwrap();
}