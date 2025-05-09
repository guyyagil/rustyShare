use std::net::SocketAddr;
use tracing::info;
use super::routes;

pub async fn start_server() {
    let app = routes::create_router();
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    info!("🚀 Server starting on {}", addr);
    axum::serve(  tokio::net::TcpListener::bind(addr).await.unwrap(), app.into_make_service() )
    .await
    .unwrap();
} 