pub mod auth;
pub mod file_operations;
pub mod static_content;
pub mod health;

// Re-export specific functions to avoid naming conflicts
pub use auth::{login, master_protection, password_required};
pub use file_operations::{
    media_json, open, upload_file, delete_file, update_file, create_folder
};
pub use static_content::static_handler;
pub use health::health_check;
