use std::sync::Arc;
use tokio::sync::RwLock;

use crate::auth::JwksCache;
use crate::config::AppConfig;
use crate::mcp::types::Tool;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub jwks_cache: Arc<RwLock<JwksCache>>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            jwks_cache: Arc::new(RwLock::new(JwksCache::default())),
        }
    }

    pub fn tools(&self) -> Vec<Tool> {
        // Will be connected to client/tools in Task 2
        Vec::new()
    }
}
