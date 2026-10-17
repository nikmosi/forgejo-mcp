pub mod auth;
pub mod client;
pub mod config;
pub mod mcp;
pub mod state;
pub mod tools;

pub use client::ForgejoClient;

use axum::{
    extract::State,
    middleware,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde_json::json;

pub use config::AppConfig;
pub use state::AppState;

pub fn create_app(state: AppState) -> Router {
    let auth_layer = middleware::from_fn_with_state(state.clone(), auth::auth_middleware);

    Router::new()
        .route("/healthz", get(healthz))
        .route("/.well-known/oauth-protected-resource", get(oauth_protected_resource))
        .route("/mcp", post(mcp::server::handle_mcp_request).layer(auth_layer))
        .with_state(state)
}

pub async fn healthz() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}

pub async fn oauth_protected_resource(State(state): State<AppState>) -> impl IntoResponse {
    let metadata = mcp::types::ProtectedResourceMetadata {
        resource: state.config.resource_url.clone(),
        authorization_servers: vec![state.config.authelia_url.clone()],
        scopes: vec!["openid".to_string(), "profile".to_string(), "email".to_string()],
        scopes_supported: vec!["openid".to_string(), "profile".to_string(), "email".to_string()],
        bearer_methods: vec!["header".to_string()],
        bearer_methods_supported: vec!["header".to_string()],
    };
    Json(metadata)
}
