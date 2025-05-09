use axum::{
    http::StatusCode,
    response::Html,
    routing::{get, MethodRouter},
    Router,
};
use tower_http::services::ServeDir;


pub fn create_router() -> Router {
    Router::new()
       
        .route("/", static_handler("html/home.html"))
        .route("/media", static_handler("html/media.html"))
        .route("/health", get(health_check))
        .fallback(static_handler("html/error.html"))
}


fn static_handler(path: &'static str) -> MethodRouter {
    get(move || async move  {
        let content = std::fs::read_to_string(path)
            .unwrap_or_else(|_| format!("⚠️ Critical file not found: {}", path));
        Html(content)
    })
}


async fn health_check() -> StatusCode {
    StatusCode::OK
}
