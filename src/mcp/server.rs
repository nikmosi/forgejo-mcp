use axum::{extract::State, Json};
use serde_json::Value;
use tracing::{debug, info};

use crate::mcp::protocol::{JsonRpcRequest, JsonRpcResponse};
use crate::mcp::types::{InitializeResult, ServerCapabilities, ServerInfo, ToolsCapability, ToolsListResult};
use crate::state::AppState;

pub async fn handle_mcp_request(
    State(state): State<AppState>,
    Json(request): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse> {
    debug!("Handling MCP request method: {}", request.method);

    match request.method.as_str() {
        "initialize" => {
            info!("Received initialize request");
            let result = InitializeResult {
                protocol_version: "2024-11-05".to_string(),
                capabilities: ServerCapabilities {
                    tools: ToolsCapability { list_changed: None },
                },
                server_info: ServerInfo {
                    name: "forgejo-mcp".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                },
            };
            Json(JsonRpcResponse::success(
                request.id,
                serde_json::to_value(result).unwrap_or(Value::Null),
            ))
        }
        "ping" => {
            debug!("Received ping request");
            Json(JsonRpcResponse::success(
                request.id,
                serde_json::json!({}),
            ))
        }
        "tools/list" => {
            debug!("Received tools/list request");
            let tools = state.tools();
            let result = ToolsListResult { tools };
            Json(JsonRpcResponse::success(
                request.id,
                serde_json::to_value(result).unwrap_or(Value::Null),
            ))
        }
        "notifications/initialized" => {
            debug!("Received initialized notification");
            Json(JsonRpcResponse::success(
                request.id,
                serde_json::json!({}),
            ))
        }
        other => {
            debug!("Received unknown method: {}", other);
            Json(JsonRpcResponse::method_not_found(request.id, other))
        }
    }
}
