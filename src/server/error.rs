use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    // File not found error
    NotFound(String),
    // Internal server error
    Internal(String),
}

// Implement Display for AppError
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal Server Error: {}", msg),
        }
    }
}

// Implement IntoResponse for AppError
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, message).into_response()
    }
}

// Helper function to create NotFound error
pub fn not_found(msg: impl Into<String>) -> AppError {
    AppError::NotFound(msg.into())
}

// Helper function to create Internal error
pub fn internal_error(msg: impl Into<String>) -> AppError {
    AppError::Internal(msg.into())
} 