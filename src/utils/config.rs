use std::env;

/// Holds application configuration loaded from environment variables.
pub struct Config {
    file_dir: String,  
    port: String,     
    password: String,  
}

impl Config {
    
    pub fn from_env() -> Self {
        Self {
            file_dir: env::var("MEDIA_DIR").unwrap_or_else(|_| "master".to_string()),
            port: env::var("PORT").unwrap_or_else(|_| "3000".to_string()),
            password : env::var("PASSWORD").unwrap_or_else(|_| "changeme".to_string()),
        }
    }


    pub fn file_dir(&self) -> &str {
        &self.file_dir
    }

  
    pub fn port(&self) -> &str {
        &self.port
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}

