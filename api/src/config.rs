use serde::Deserialize;
use std::path::Path;

#[derive(Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Clone, Deserialize)]
pub struct KafkaConfig {
    pub brokers: String,
    pub topic: String,
}

#[derive(Clone, Deserialize)]
pub struct LoggingConfig {
    /// Optional RUST_LOG-style filter (e.g. "info", "warn,actix_web=info").
    pub level: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct Settings {
    pub server: ServerConfig,
    pub kafka: KafkaConfig,
    pub logging: Option<LoggingConfig>,
}

impl Settings {
    /// Load configuration from a YAML file using the `config` crate.
    /// Expects a path like `config.yaml`.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let mut c = config::Config::builder();
        c = c.add_source(config::File::from(path.as_ref()));

        // allow overriding with environment variables if desired
        c = c.add_source(config::Environment::with_prefix("APP").separator("__"));

        let cfg = c.build()?;
        let s: Settings = cfg.try_deserialize()?;
        Ok(s)
    }
}
