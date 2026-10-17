#[derive(Debug, Clone)]
pub struct AppConfig {
    pub port: u16,
    pub host: String,
    pub authelia_url: String,
    pub resource_url: String,
    pub static_api_key: Option<String>,
    pub target_url: String,
    pub target_token: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8000),
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            authelia_url: std::env::var("AUTHELIA_URL")
                .unwrap_or_else(|_| "https://auth.xinfra.ru".to_string()),
            resource_url: std::env::var("RESOURCE_URL")
                .unwrap_or_else(|_| "https://mcp-forgejo.xinfra.ru/mcp".to_string()),
            static_api_key: std::env::var("STATIC_API_KEY").ok().filter(|s| !s.is_empty()),
            target_url: std::env::var("TARGET_URL")
                .unwrap_or_else(|_| "http://forgejo-http.developer-tools.svc.cluster.local:3000".to_string()),
            target_token: std::env::var("TARGET_TOKEN").unwrap_or_default(),
        }
    }
}
