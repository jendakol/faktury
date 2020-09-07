use chrono::Duration;
use config::File;
use err_context::AnyError;
use serde::de::{Error as DeError, Unexpected};
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize, Clone)]
pub struct HttpConfig {
    pub listen: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub db_name: String,
    pub username: String,
    pub password: String,
    pub prefer_socket: bool,
    pub max_pool_size: u8,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AccountsConfig {
    #[serde(deserialize_with = "deserialize_duration")]
    pub login_ttl: Duration,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub http: HttpConfig,
    pub database: DbConfig,
    pub accounts: AccountsConfig,
}

impl AppConfig {
    pub fn load(path: &str) -> Result<Self, AnyError> {
        let mut config = config::Config::new();

        config.merge(File::with_name("config/default"))?;

        let content: String = {
            use std::fs::File;
            use std::io::prelude::*;
            let mut file = File::open(path).map_err(|e| AnyError::from(format!("Could not open file {}: {}", path, e)))?;
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| AnyError::from(format!("Could not read from file {}: {}", path, e)))?;
            content
        };

        config.merge(config::File::from_str(content.as_ref(), config::FileFormat::Toml))?;

        config
            .try_into()
            .map_err(|e| AnyError::from(format!("Could not map config: {}", e)))
    }
}

fn deserialize_duration<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
    let s = String::deserialize(d)?;
    humantime::parse_duration(&s)
        .map_err(|_| DeError::invalid_value(Unexpected::Str(&s), &"Human readable duration"))
        .map(|d| Duration::milliseconds(d.as_millis() as i64))
}
