use axum::http::StatusCode;

/// Simple health check endpoint.
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}
