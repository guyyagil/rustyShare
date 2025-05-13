use std::net::SocketAddr;
use tracing::info;
use super::routes;
use std::sync::{Arc, Mutex};
use crate::media::{scanner::scan_dir};
use tokio::time::{interval, Duration};

pub async fn start_server() {
    let media_tree = Arc::new(Mutex::new(scan_dir("media")));
    let watcher_tree = media_tree.clone();
    
    tokio::spawn(async {
        let mut interval = interval(Duration::from_secs(60 * 60)); // every hour
        loop {
            interval.tick().await;
            crate::cache::clean_cache_space(10 * 1024 * 1024 * 1024); // 10 GB
        }
    });

    std::thread::spawn(move || {
        crate::media::watch::start_watcher(watcher_tree);
    });
    
    let app = routes::create_router(media_tree.clone());
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    info!("🚀 Server starting on {}", addr);
    axum::serve(  tokio::net::TcpListener::bind(addr).await.unwrap(), app.into_make_service() )
    .await
    .unwrap();
} 