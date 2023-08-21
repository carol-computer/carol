use std::{collections::HashMap, path::PathBuf, str::FromStr};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Config {
    pub http_server: HttpServerConfig,
    pub bls_secret_key: carol_bls::KeyPair,
    pub log: LogConfig,
}

impl Config {
    pub fn generate(rng: &mut impl rand::RngCore) -> Self {
        Config {
            http_server: HttpServerConfig::default(),
            bls_secret_key: carol_bls::KeyPair::random(rng),
            log: Default::default(),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct HttpServerConfig {
    pub listen: std::net::SocketAddr,
    #[serde(default)]
    pub resources: HashMap<String, PathBuf>,
}

impl Default for HttpServerConfig {
    fn default() -> Self {
        Self {
            listen: std::net::SocketAddr::from_str("127.0.0.1:8000").unwrap(),
            resources: Default::default(),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

impl From<Level> for tracing::Level {
    fn from(value: Level) -> Self {
        match value {
            Level::Error => tracing::Level::ERROR,
            Level::Warn => tracing::Level::WARN,
            Level::Info => tracing::Level::INFO,
            Level::Debug => tracing::Level::DEBUG,
            Level::Trace => tracing::Level::TRACE,
        }
    }
}

impl From<Level> for tracing::level_filters::LevelFilter {
    fn from(value: Level) -> Self {
        tracing::Level::from(value).into()
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
pub struct LogConfig {
    pub level: Level,
}
