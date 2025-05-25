use axum::{
    extract::DefaultBodyLimit,
    routing::get,
    Router
};
use tower_cookies::CookieManagerLayer;
use tokio::sync::Mutex;
use std::sync::Arc;
use tower_http::services::ServeDir;
use crate::file_manager::file_tree::FileEntry;

use super::handlers::*;

/// Creates and configures the application router with all routes.
/// Accepts a shared `file_tree` state for media file management.
pub fn create_router(file_tree: Arc<Mutex<Option<FileEntry>>>) -> Router {
    Router::new()
        .nest_service("/static", ServeDir::new("static"))
        .route("/", static_handler("static/html/home.html"))
        .route("/login", axum::routing::post(login))
        .route("/master", get(master_protection))
        .route("/api/master.json", get(media_json))
        .route("/api/master/{*path}", get(open))
        .route("/health", get(health_check))
        .route("/api/upload", axum::routing::post(upload_file))
        .route("/api/delete", axum::routing::post(delete_file))
        .route("/api/update", axum::routing::post(update_file))
        .route("/api/create_folder", axum::routing::post(create_folder))
        .route("/api/password_required", get(password_required))
        .fallback(static_handler("static/html/error.html"))
        .layer(CookieManagerLayer::new()) // Enables cookie management for authentication
        .layer(axum::extract::Extension(file_tree)) // Shares the file tree state with handlers
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024)) // 1GB upload limit
}

