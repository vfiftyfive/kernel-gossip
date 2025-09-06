use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingVar(String),
}

#[derive(Debug, Clone)]
pub struct Config {
    pub webhook_port: u16,
    pub metrics_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let webhook_port = std::env::var("WEBHOOK_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8080);

        let metrics_port = std::env::var("METRICS_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(9090);

        Ok(Config {
            webhook_port,
            metrics_port,
        })
    }
}