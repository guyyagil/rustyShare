mod server;
mod media;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt}; 
mod cache;

#[tokio::main]

async fn main() {
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
   
    server::start_server().await;
} 

