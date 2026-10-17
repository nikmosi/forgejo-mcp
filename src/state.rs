use std::sync::Arc;
use tokio::sync::RwLock;

use crate::auth::JwksCache;
use crate::client::ForgejoClient;
use crate::config::AppConfig;
use crate::mcp::types::Tool;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub jwks_cache: Arc<RwLock<JwksCache>>,
    pub client: ForgejoClient,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let client = ForgejoClient::new(&config.target_url, &config.target_token);
        Self {
            config,
            jwks_cache: Arc::new(RwLock::new(JwksCache::default())),
            client,
        }
    }

    pub fn tools(&self) -> Vec<Tool> {
        crate::tools::get_tools()
    }
}
