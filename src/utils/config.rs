use std::env;

pub struct Config {
    pub media_dir: String,
    pub port: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            media_dir: env::var("MEDIA_DIR").unwrap_or_else(|_| "media".to_string()),
            port: env::var("PORT").unwrap_or_else(|_| "3000".to_string()),
        }
    }
}
