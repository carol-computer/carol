use std::str::FromStr;

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
    pub dns: dns::Config,
}

impl Default for HttpServerConfig {
    fn default() -> Self {
        Self {
            listen: std::net::SocketAddr::from_str("127.0.0.1:8000").unwrap(),
            dns: Default::default(),
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

pub mod dns {

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Default)]
    pub struct Config {
        /// Matches requests that are directly for the carol host.
        ///
        /// e.g. 580048519c50c7d767edaeb1e582e2f766bc4eac4b7f0517d857bd8124bdf7.carol.computer
        ///
        /// If base domain was carol.computer then this would mean the hex is interpreted as a
        /// machine id. Also anything CNAME'd to this domain would resolve to this.
        #[serde(default)]
        pub base_domain: Option<hickory_resolver::Name>,
        /// Passthrough requests matching this domain to the HTTP API
        #[serde(default)]
        pub api_host: Option<hickory_resolver::Name>,
        #[serde(default)]
        pub hickory_conf: hickory_resolver::config::ResolverConfig,
        #[serde(default)]
        pub hickory_opts: hickory_resolver::config::ResolverOpts,
    }

    impl Config {
        pub fn into_resolver(self) -> crate::http::resolver::Resolver {
            crate::http::resolver::Resolver::new(self)
        }
    }
}
