
mod server;
mod file_manager;
mod utils;
use tracing_subscriber;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::layer::SubscriberExt;
use utils::config;


#[tokio::main] 
async fn main() {

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            config::Config::from_env().rust_log() 
        ))
        .with(tracing_subscriber::fmt::layer()) 
        .init();
   
    
    server::start_server().await;
}

