#[derive(serde::Deserialize)]
pub struct Config {
    pub http_server: HttpServerConfig,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct HttpServerConfig {
    pub listen: std::net::SocketAddr,
}
