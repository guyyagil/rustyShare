use std::env;

pub struct Config {
    pub file_dir: String,
    pub port: String,
    pub password: String
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            file_dir: env::var("MEDIA_DIR").unwrap_or_else(|_| "master".to_string()),
            port: env::var("PORT").unwrap_or_else(|_| "3000".to_string()),
            password : env::var("PASSWORD").unwrap_or_else(|_| "changeme".to_string()),
        }
    }
}
