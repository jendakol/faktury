use config::File;
use err_context::AnyError;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct HttpConfig {
    pub listen: String,
}

#[derive(Debug, Deserialize)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub db_name: String,
    pub username: String,
    pub password: String,
    pub prefer_socket: bool,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub http: HttpConfig,
    pub database: DbConfig,
}

impl AppConfig {
    pub fn load(path: &str) -> Result<Self, AnyError> {
        let mut config = config::Config::new();

        config.merge(File::with_name("config/default"))?;

        let content: String = {
            use std::fs::File;
            use std::io::prelude::*;
            let mut file = File::open(path)
                .map_err(|e| AnyError::from(format!("Could not open file {}: {}", path, e)))?;
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| AnyError::from(format!("Could not read from file {}: {}", path, e)))?;
            content
        };

        config.merge(config::File::from_str(
            content.as_ref(),
            config::FileFormat::Toml,
        ))?;

        config
            .try_into()
            .map_err(|e| AnyError::from(format!("Could not map config: {}", e)))
    }
}
