use axum::{
    response::Html,
    routing::{get, MethodRouter},
};

/// Generic handler for serving static HTML content.
/// Used for routes that do not require dynamic logic.
pub fn static_handler(path: &'static str) -> MethodRouter {
    get(move || async move {
        let content = std::fs::read_to_string(path).unwrap();
        Html(content)
    })
}
