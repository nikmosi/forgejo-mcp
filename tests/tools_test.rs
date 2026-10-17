use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    routing::{get, post},
    Json, Router,
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tower::ServiceExt;

use forgejo_mcp::{
    config::AppConfig,
    create_app,
    mcp::protocol::JsonRpcResponse,
    state::AppState,
    tools::get_tools,
};

fn test_config(target_url: &str) -> AppConfig {
    AppConfig {
        port: 8000,
        host: "0.0.0.0".to_string(),
        authelia_url: "https://auth.xinfra.ru".to_string(),
        resource_url: "https://mcp-forgejo.xinfra.ru/mcp".to_string(),
        static_api_key: Some("test-secret-key".to_string()),
        target_url: target_url.to_string(),
        target_token: "test-target-token".to_string(),
    }
}

#[tokio::test]
async fn test_tools_list_returns_all_36_tools() {
    let state = AppState::new(test_config("http://localhost:1234"));
    let app = create_app(state);

    let tools_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/mcp")
                .method("POST")
                .header(header::AUTHORIZATION, "Bearer test-secret-key")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&tools_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: JsonRpcResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(json.id, Some(json!(1)));

    let tools = json.result.unwrap()["tools"].as_array().unwrap().clone();
    assert_eq!(tools.len(), 36, "Expected 36 tools, got {}", tools.len());

    let tool_names: Vec<String> = tools
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_string())
        .collect();

    let expected_tools = vec![
        // Repositories (12)
        "search_repos",
        "list_my_repos",
        "get_repo",
        "get_file_content",
        "create_file",
        "update_file",
        "delete_file",
        "get_repo_tree",
        "list_repo_commits",
        "list_branches",
        "create_branch",
        "delete_branch",
        // Issues & Milestones (12)
        "search_issues",
        "list_repo_issues",
        "get_issue_by_index",
        "create_issue",
        "update_issue",
        "issue_state_change",
        "list_issue_comments",
        "create_issue_comment",
        "list_repo_labels",
        "add_issue_labels",
        "remove_issue_labels",
        "list_repo_milestones",
        // Pull Requests (6)
        "list_repo_pull_requests",
        "get_pull_request_by_index",
        "get_pull_request_diff",
        "create_pull_request",
        "update_pull_request",
        "merge_pull_request",
        // Releases & Actions (6)
        "list_releases",
        "get_release_by_tag",
        "create_release",
        "list_workflow_runs",
        "get_workflow_run",
        "dispatch_workflow",
    ];

    for expected in expected_tools {
        assert!(
            tool_names.contains(&expected.to_string()),
            "Missing tool: {}",
            expected
        );
    }

    // Verify all tools have valid inputSchema
    for tool in &tools {
        assert!(!tool["description"].as_str().unwrap().is_empty());
        let schema = &tool["inputSchema"];
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].is_object());
    }
}

#[tokio::test]
async fn test_get_tools_function_directly() {
    let tools = get_tools();
    assert_eq!(tools.len(), 36);
    let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"search_repos"));
    assert!(names.contains(&"create_pull_request"));
    assert!(names.contains(&"dispatch_workflow"));
}

#[tokio::test]
async fn test_tools_call_unknown_tool() {
    let state = AppState::new(test_config("http://localhost:1234"));
    let app = create_app(state);

    let call_req = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "unknown_tool",
            "arguments": {}
        }
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/mcp")
                .method("POST")
                .header(header::AUTHORIZATION, "Bearer test-secret-key")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&call_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: JsonRpcResponse = serde_json::from_slice(&body).unwrap();
    let res = json.result.unwrap();
    assert_eq!(res["isError"], true);
    assert!(res["content"][0]["text"].as_str().unwrap().contains("Unknown tool"));
}

#[tokio::test]
async fn test_tools_call_missing_required_arguments() {
    let state = AppState::new(test_config("http://localhost:1234"));
    let app = create_app(state);

    let call_req = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "get_repo",
            "arguments": {} // missing owner and repo
        }
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/mcp")
                .method("POST")
                .header(header::AUTHORIZATION, "Bearer test-secret-key")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&call_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: JsonRpcResponse = serde_json::from_slice(&body).unwrap();
    let res = json.result.unwrap();
    assert_eq!(res["isError"], true);
    assert!(res["content"][0]["text"].as_str().unwrap().contains("Missing required argument"));
}

#[tokio::test]
async fn test_tools_call_with_mock_forgejo_server() {
    let auth_verified = Arc::new(AtomicBool::new(false));
    let auth_clone = auth_verified.clone();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mock_addr = listener.local_addr().unwrap();

    let mock_forgejo = Router::new()
        .route(
            "/api/v1/repos/{owner}/{repo}",
            get({
                let auth_clone = auth_clone.clone();
                move |headers: axum::http::HeaderMap| {
                    let auth_clone = auth_clone.clone();
                    async move {
                        if let Some(auth) = headers.get(header::AUTHORIZATION) {
                            if auth.to_str().unwrap_or("") == "token test-target-token" {
                                auth_clone.store(true, Ordering::SeqCst);
                            }
                        }
                        Json(json!({
                            "id": 42,
                            "name": "infra",
                            "owner": { "login": "nikmosi" },
                            "default_branch": "main"
                        }))
                    }
                }
            }),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/contents/{*filepath}",
            get(|| async {
                Json(json!({
                    "name": "README.md",
                    "path": "README.md",
                    "sha": "abc123sha",
                    "type": "file",
                    "size": 11,
                    "encoding": "base64",
                    "content": "SGVsbG8gV29ybGQ=" // "Hello World"
                }))
            }),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/pulls/{pull_file}",
            get(|axum::extract::Path((_owner, _repo, pull_file)): axum::extract::Path<(String, String, String)>| async move {
                if pull_file.ends_with(".diff") {
                    "diff --git a/src/main.rs b/src/main.rs\nindex 000..111 100644\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1,1 +1,2 @@\n+println!(\"hello\");\n"
                } else {
                    "mock pull request details"
                }
            }),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/pulls",
            post(|Json(body): Json<Value>| async move {
                Json(json!({
                    "id": 101,
                    "number": 5,
                    "title": body["title"],
                    "state": "open"
                }))
            }),
        );

    tokio::spawn(async move {
        axum::serve(listener, mock_forgejo).await.unwrap();
    });

    let state = AppState::new(test_config(&format!("http://{}", mock_addr)));
    let app = create_app(state);

    // 1. Call get_repo
    let call_repo = json!({
        "jsonrpc": "2.0",
        "id": 10,
        "method": "tools/call",
        "params": {
            "name": "get_repo",
            "arguments": {
                "owner": "nikmosi",
                "repo": "infra"
            }
        }
    });

    let resp = app.clone()
        .oneshot(
            Request::builder()
                .uri("/mcp")
                .method("POST")
                .header(header::AUTHORIZATION, "Bearer test-secret-key")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&call_repo).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: JsonRpcResponse = serde_json::from_slice(&body).unwrap();
    let res = json.result.unwrap();
    assert_eq!(res["isError"], false);
    let text = res["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("infra"));
    assert!(auth_verified.load(Ordering::SeqCst), "Authorization token must match");

    // 2. Call get_file_content (plain text by default)
    let call_file = json!({
        "jsonrpc": "2.0",
        "id": 11,
        "method": "tools/call",
        "params": {
            "name": "get_file_content",
            "arguments": {
                "owner": "nikmosi",
                "repo": "infra",
                "filePath": "README.md"
            }
        }
    });

    let resp = app.clone()
        .oneshot(
            Request::builder()
                .uri("/mcp")
                .method("POST")
                .header(header::AUTHORIZATION, "Bearer test-secret-key")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&call_file).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: JsonRpcResponse = serde_json::from_slice(&body).unwrap();
    let res = json.result.unwrap();
    assert_eq!(res["isError"], false);
    assert_eq!(res["content"][0]["text"].as_str().unwrap(), "Hello World");

    // 3. Call get_pull_request_diff
    let call_diff = json!({
        "jsonrpc": "2.0",
        "id": 12,
        "method": "tools/call",
        "params": {
            "name": "get_pull_request_diff",
            "arguments": {
                "owner": "nikmosi",
                "repo": "infra",
                "index": 5
            }
        }
    });

    let resp = app.clone()
        .oneshot(
            Request::builder()
                .uri("/mcp")
                .method("POST")
                .header(header::AUTHORIZATION, "Bearer test-secret-key")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&call_diff).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: JsonRpcResponse = serde_json::from_slice(&body).unwrap();
    let res = json.result.unwrap();
    assert_eq!(res["isError"], false);
    assert!(res["content"][0]["text"].as_str().unwrap().contains("println!(\"hello\")"));

    // 4. Call create_pull_request
    let call_pr = json!({
        "jsonrpc": "2.0",
        "id": 13,
        "method": "tools/call",
        "params": {
            "name": "create_pull_request",
            "arguments": {
                "owner": "nikmosi",
                "repo": "infra",
                "head": "feature",
                "base": "main",
                "title": "Add feature"
            }
        }
    });

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/mcp")
                .method("POST")
                .header(header::AUTHORIZATION, "Bearer test-secret-key")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&call_pr).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: JsonRpcResponse = serde_json::from_slice(&body).unwrap();
    let res = json.result.unwrap();
    assert_eq!(res["isError"], false);
    assert!(res["content"][0]["text"].as_str().unwrap().contains("Add feature"));
}

#[tokio::test]
async fn test_tools_call_file_operations_and_line_slicing() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mock_addr = listener.local_addr().unwrap();

    let created_body = Arc::new(tokio::sync::Mutex::new(Value::Null));
    let created_clone = created_body.clone();

    let mock_forgejo = Router::new()
        .route(
            "/api/v1/repos/{owner}/{repo}/contents/{*filepath}",
            get(|| async {
                // Return 5 lines: line 1\nline 2\nline 3\nline 4\nline 5
                // Base64: bGluZSAxCmxpbmUgMgpsaW5lIDMKbGluZSA0CmxpbmUgNQ==
                Json(json!({
                    "name": "multiline.txt",
                    "path": "multiline.txt",
                    "sha": "multi123",
                    "type": "file",
                    "encoding": "base64",
                    "content": "bGluZSAxCmxpbmUgMgpsaW5lIDMKbGluZSA0CmxpbmUgNQ=="
                }))
            })
            .post({
                let created_clone = created_clone.clone();
                |Json(body): Json<Value>| async move {
                    *created_clone.lock().await = body;
                    Json(json!({ "content": { "name": "new.txt" } }))
                }
            }),
        );

    tokio::spawn(async move {
        axum::serve(listener, mock_forgejo).await.unwrap();
    });

    let state = AppState::new(test_config(&format!("http://{}", mock_addr)));
    let app = create_app(state);

    // 1. Get file content with line slicing (lines 2 to 3)
    let call_slice = json!({
        "jsonrpc": "2.0",
        "id": 20,
        "method": "tools/call",
        "params": {
            "name": "get_file_content",
            "arguments": {
                "owner": "nikmosi",
                "repo": "infra",
                "filePath": "multiline.txt",
                "start_line": 2,
                "end_line": 3
            }
        }
    });

    let resp = app.clone()
        .oneshot(
            Request::builder()
                .uri("/mcp")
                .method("POST")
                .header(header::AUTHORIZATION, "Bearer test-secret-key")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&call_slice).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: JsonRpcResponse = serde_json::from_slice(&body).unwrap();
    let res = json.result.unwrap();
    assert_eq!(res["isError"], false);
    assert_eq!(res["content"][0]["text"].as_str().unwrap(), "line 2\nline 3");

    // 2. Create file - verify plain text is automatically base64-encoded
    let call_create = json!({
        "jsonrpc": "2.0",
        "id": 21,
        "method": "tools/call",
        "params": {
            "name": "create_file",
            "arguments": {
                "owner": "nikmosi",
                "repo": "infra",
                "filePath": "src/hello.rs",
                "content": "fn main() {}",
                "message": "Add hello.rs",
                "branch_name": "main"
            }
        }
    });

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/mcp")
                .method("POST")
                .header(header::AUTHORIZATION, "Bearer test-secret-key")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&call_create).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: JsonRpcResponse = serde_json::from_slice(&body).unwrap();
    let res = json.result.unwrap();
    assert_eq!(res["isError"], false);

    let recorded = created_body.lock().await;
    // "fn main() {}" base64 is "Zm4gbWFpbigpIHt9"
    assert_eq!(recorded["content"], "Zm4gbWFpbigpIHt9");
    assert_eq!(recorded["branch"], "main");
    assert_eq!(recorded["message"], "Add hello.rs");
}

#[tokio::test]
async fn test_tools_call_releases_filtering_and_actions() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mock_addr = listener.local_addr().unwrap();

    let dispatched_body = Arc::new(tokio::sync::Mutex::new(Value::Null));
    let dispatched_clone = dispatched_body.clone();

    let mock_forgejo = Router::new()
        .route(
            "/api/v1/repos/{owner}/{repo}/releases",
            get(|| async {
                Json(json!([
                    { "id": 1, "tag_name": "v0.1.0-draft", "draft": true, "prerelease": false },
                    { "id": 2, "tag_name": "v0.1.0-rc1", "draft": false, "prerelease": true },
                    { "id": 3, "tag_name": "v0.1.0", "draft": false, "prerelease": false }
                ]))
            }),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/actions/workflows/{workflow}/dispatches",
            post({
                let dispatched_clone = dispatched_clone.clone();
                |Json(body): Json<Value>| async move {
                    *dispatched_clone.lock().await = body;
                    StatusCode::NO_CONTENT
                }
            }),
        );

    tokio::spawn(async move {
        axum::serve(listener, mock_forgejo).await.unwrap();
    });

    let state = AppState::new(test_config(&format!("http://{}", mock_addr)));
    let app = create_app(state);

    // 1. List published releases only
    let call_releases = json!({
        "jsonrpc": "2.0",
        "id": 30,
        "method": "tools/call",
        "params": {
            "name": "list_releases",
            "arguments": {
                "owner": "nikmosi",
                "repo": "forgejo-mcp",
                "state": "published"
            }
        }
    });

    let resp = app.clone()
        .oneshot(
            Request::builder()
                .uri("/mcp")
                .method("POST")
                .header(header::AUTHORIZATION, "Bearer test-secret-key")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&call_releases).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: JsonRpcResponse = serde_json::from_slice(&body).unwrap();
    let res = json.result.unwrap();
    assert_eq!(res["isError"], false);
    let items: Vec<Value> = serde_json::from_str(res["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["tag_name"], "v0.1.0");

    // 2. Dispatch workflow
    let call_dispatch = json!({
        "jsonrpc": "2.0",
        "id": 31,
        "method": "tools/call",
        "params": {
            "name": "dispatch_workflow",
            "arguments": {
                "owner": "nikmosi",
                "repo": "forgejo-mcp",
                "workflow": "image.yml",
                "ref": "main",
                "inputs": { "tag": "v0.1.0" }
            }
        }
    });

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/mcp")
                .method("POST")
                .header(header::AUTHORIZATION, "Bearer test-secret-key")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&call_dispatch).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: JsonRpcResponse = serde_json::from_slice(&body).unwrap();
    let res = json.result.unwrap();
    assert_eq!(res["isError"], false);

    let recorded = dispatched_body.lock().await;
    assert_eq!(recorded["ref"], "main");
    assert_eq!(recorded["inputs"]["tag"], "v0.1.0");
}

#[tokio::test]
async fn test_tools_call_upstream_api_error_returns_is_error_true() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mock_addr = listener.local_addr().unwrap();

    let mock_forgejo = Router::new().route(
        "/api/v1/repos/{owner}/{repo}",
        get(|| async {
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "message": "repository does not exist" })),
            )
        }),
    );

    tokio::spawn(async move {
        axum::serve(listener, mock_forgejo).await.unwrap();
    });

    let state = AppState::new(test_config(&format!("http://{}", mock_addr)));
    let app = create_app(state);

    let call_req = json!({
        "jsonrpc": "2.0",
        "id": 40,
        "method": "tools/call",
        "params": {
            "name": "get_repo",
            "arguments": {
                "owner": "nikmosi",
                "repo": "nonexistent"
            }
        }
    });

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/mcp")
                .method("POST")
                .header(header::AUTHORIZATION, "Bearer test-secret-key")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&call_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: JsonRpcResponse = serde_json::from_slice(&body).unwrap();
    let res = json.result.unwrap();
    assert_eq!(res["isError"], true);
    let err_msg = res["content"][0]["text"].as_str().unwrap();
    assert!(err_msg.contains("status 404"));
    assert!(err_msg.contains("repository does not exist"));
}
