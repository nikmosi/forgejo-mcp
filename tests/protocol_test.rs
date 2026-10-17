use std::time::{SystemTime, UNIX_EPOCH};
use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tower::ServiceExt;

use forgejo_mcp::{
    auth::{Jwk, Jwks},
    create_app,
    config::AppConfig,
    state::AppState,
    mcp::protocol::JsonRpcResponse,
};

const TEST_PRIVATE_KEY_PEM: &str = r#"-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDFRlNr+9CI0HDf
qSwwLSrmnzKsk/8RuU5h65Xi2RN5eH7xGqyHXCOBh4gGUVIGqnIVbFqRxJk7skPk
Wn2XHDms/o28r/DBB5f8sCPwiAHBdSWbCQTFQeM7zn3/q+2SESt4yaj5g/NcVOWK
OtFVGqxIydwbgeyiYH69HBy74ChaaPFwByW/y8ZyocuWsM4vb9ulbmplwiAxpyKb
2dCRGWU0UTU/xBTZJNjcYBpaLh69Zo7chOcINVpv6bMbFrFqlRdMeAuLJa2XJ3zK
hh6X2NSDwL6YzagrCCO4zP7mCvZuCB7lSnloGm/6D44YU2FO1jbuftmLR3y09hAb
yvmWg1VJAgMBAAECggEAEba+o2XdZAeZdw/lAp9vqH03iuUQJIqH5gWGNDQrSj8+
YM51GwN0QbuGhJC1RZl0/IT5/fhSDRj0/TJ97/1uFWFXH1AKFX8lz+AOHhlMVQ98
lLcC2WaQ7awdRsGsp67IgURUWrFVPx91bAvbg6GNQkzz6XlpYuw4vSji0KDwZqu4
lP4zSZNOg98cVwAIN7sKH13XDMg3V3x5PKYbAqutZwf/CkNLJUKPaql7QOKgTD0O
rSp7eghwhBHGmkiJuXCU0srAjYhS9kYnZNOZQSe9Q84NYqy1pG+uqKXiDj4zhfiU
DM/K9Yfqh+H84alpHNjB34P6AHbbaY49T8MJNuC5IQKBgQD7EiUFcb7MRSE+LhRe
AFcIbPhBXWPbkDwP1tzbjX38RuFGxDCJO41BvWfvx3N87u45TYtgGY3LbNgsuSWG
KnldcSvJxM5RVFvt0JBOVuBXg5fiOhfWE0Pj/HdgVXQFI7+vDIU/FMH9/GAJPYf/
NwLdEEkYEWJZrC90N403r6KC+QKBgQDJJc62RFh4tQy9eUy+UwpYX8zU0V6qQ8mj
NqUINVU3PSNHB/y3Unn9fnCHpu4hew+NQ6pV+wuiLKIQixE9m/5Zu6WNVjKfeoFx
8IJ5fQPS+FZP+m7xEixeFf2mIPOcz+gZptvADyk+HMRmQVPWfnP+0Rb3Xf6XTwwQ
7GpZKbuo0QKBgFM/RusxNdRBL66QVMx+TF1M1FvH/W41utR9p6lcluNpFRUDxBwc
lM+xX58PgOlxx6dUYNwyXOOQaXUaxqhfuPIUrjIugUJSOFD4EIEJKUWbKmcmX/Tp
Wl9jvKlCbD+wBMSj4CJ0XYY7ghRORKOIt/hYiAPk77lP8EQWj0phf/7xAoGBAJvx
tM34NML4UnU+6nGPvbCALTQ9UfAc9+GShuXTO0rFGmmCVrnltnACK8nvcCUC3C6/
6/oW1JEZm1fKCUbDfrEg2pu13S9o8ZxdQzpe4q0YHQqKdKJvXXhgAjNezp+fTCk/
BJVk66uc7+L6+L2Nce53Oh/zfmLeRNOGBnkz8qpRAoGBALWfjJdPJYj4NK7nGl4I
6k/gqPcDbLInphQHspA36adCYW2lFXSYh3rPK1sYAX/sydVqKZXlSVrt7x47RANW
gCTJbD5zL62e1YG4z7vs1D2W5aSR6l37UkAQh2OTi05UZXhmmw4YQHj5OSDD8Rgd
b0xPeZ17EBpBvtXGqLNqaKZ6
-----END PRIVATE KEY-----"#;

const TEST_JWK_N: &str = "xUZTa_vQiNBw36ksMC0q5p8yrJP_EblOYeuV4tkTeXh-8Rqsh1wjgYeIBlFSBqpyFWxakcSZO7JD5Fp9lxw5rP6NvK_wwQeX_LAj8IgBwXUlmwkExUHjO859_6vtkhEreMmo-YPzXFTlijrRVRqsSMncG4HsomB-vRwcu-AoWmjxcAclv8vGcqHLlrDOL2_bpW5qZcIgMacim9nQkRllNFE1P8QU2STY3GAaWi4evWaO3ITnCDVab-mzGxaxapUXTHgLiyWtlyd8yoYel9jUg8C-mM2oKwgjuMz-5gr2bgge5Up5aBpv-g-OGFNhTtY27n7Zi0d8tPYQG8r5loNVSQ";
const TEST_JWK_E: &str = "AQAB";

#[derive(Debug, Serialize, Deserialize)]
struct TestJwtClaims {
    iss: String,
    sub: String,
    aud: String,
    exp: usize,
    nbf: usize,
    iat: usize,
}

fn test_config() -> AppConfig {
    AppConfig {
        port: 8000,
        host: "0.0.0.0".to_string(),
        authelia_url: "https://auth.xinfra.ru".to_string(),
        resource_url: "https://mcp-forgejo.xinfra.ru/mcp".to_string(),
        static_api_key: Some("test-secret-key".to_string()),
        target_url: "http://forgejo:3000".to_string(),
        target_token: "forgejo-token".to_string(),
    }
}

#[tokio::test]
async fn test_healthz_endpoint() {
    let state = AppState::new(test_config());
    let app = create_app(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn test_oauth_protected_resource_metadata() {
    let state = AppState::new(test_config());
    let app = create_app(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/.well-known/oauth-protected-resource")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["resource"], "https://mcp-forgejo.xinfra.ru/mcp");
    assert_eq!(json["authorization_servers"][0], "https://auth.xinfra.ru");
    assert!(
        json["scopes"].as_array().unwrap().iter().any(|s| s == "openid")
            || json["scopes_supported"].as_array().unwrap().iter().any(|s| s == "openid")
    );
    assert!(
        json["bearer_methods"].as_array().unwrap().iter().any(|s| s == "header")
            || json["bearer_methods_supported"].as_array().unwrap().iter().any(|s| s == "header")
    );
}

#[tokio::test]
async fn test_unauthenticated_mcp_returns_401_with_www_authenticate() {
    let state = AppState::new(test_config());
    let app = create_app(state);

    let rpc_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "ping"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/mcp")
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&rpc_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let auth_header = response.headers().get(header::WWW_AUTHENTICATE);
    assert!(auth_header.is_some(), "WWW-Authenticate header must be present");
    let auth_val = auth_header.unwrap().to_str().unwrap();
    assert!(
        auth_val.contains("resource_metadata=\"https://mcp-forgejo.xinfra.ru/.well-known/oauth-protected-resource\""),
        "WWW-Authenticate must contain resource_metadata RFC 9728 link, got: {}",
        auth_val
    );
}

#[tokio::test]
async fn test_authenticated_static_token_mcp_methods() {
    let state = AppState::new(test_config());
    let app = create_app(state);

    // 1. ping
    let ping_req = json!({
        "jsonrpc": "2.0",
        "id": "req-1",
        "method": "ping"
    });
    let response = app.clone()
        .oneshot(
            Request::builder()
                .uri("/mcp")
                .method("POST")
                .header(header::AUTHORIZATION, "Bearer test-secret-key")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&ping_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: JsonRpcResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(json.jsonrpc, "2.0");
    assert_eq!(json.id, Some(json!("req-1")));
    assert!(json.result.is_some());
    assert!(json.error.is_none());

    // 2. initialize
    let init_req = json!({
        "jsonrpc": "2.0",
        "id": 10,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "1.0.0" }
        }
    });
    let response = app.clone()
        .oneshot(
            Request::builder()
                .uri("/mcp")
                .method("POST")
                .header(header::AUTHORIZATION, "Bearer test-secret-key")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&init_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: JsonRpcResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(json.id, Some(json!(10)));
    let res = json.result.unwrap();
    assert_eq!(res["protocolVersion"], "2024-11-05");
    assert_eq!(res["serverInfo"]["name"], "forgejo-mcp");

    // 3. tools/list
    let tools_req = json!({
        "jsonrpc": "2.0",
        "id": 20,
        "method": "tools/list"
    });
    let response = app.clone()
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
    assert_eq!(json.id, Some(json!(20)));
    assert!(json.result.unwrap()["tools"].is_array());

    // 4. unknown method
    let unknown_req = json!({
        "jsonrpc": "2.0",
        "id": 99,
        "method": "foo/bar"
    });
    let response = app.clone()
        .oneshot(
            Request::builder()
                .uri("/mcp")
                .method("POST")
                .header(header::AUTHORIZATION, "Bearer test-secret-key")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&unknown_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: JsonRpcResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(json.id, Some(json!(99)));
    assert!(json.result.is_none());
    assert_eq!(json.error.unwrap().code, -32601);
}

#[tokio::test]
async fn test_invalid_bearer_token_returns_401() {
    let state = AppState::new(test_config());
    let app = create_app(state);

    let ping_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "ping"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/mcp")
                .method("POST")
                .header(header::AUTHORIZATION, "Bearer wrong-key")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&ping_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let auth_header = response.headers().get(header::WWW_AUTHENTICATE);
    assert!(auth_header.is_some());
}

#[tokio::test]
async fn test_authelia_rs256_jwt_auth_success() {
    let state = AppState::new(test_config());

    // Pre-populate JWKS cache with test key
    let test_jwk = Jwk {
        kty: "RSA".to_string(),
        kid: Some("test-key-id-1".to_string()),
        alg: Some("RS256".to_string()),
        r#use: Some("sig".to_string()),
        n: TEST_JWK_N.to_string(),
        e: TEST_JWK_E.to_string(),
    };
    {
        let mut cache = state.jwks_cache.write().await;
        cache.jwks = Some(Jwks {
            keys: vec![test_jwk],
        });
    }

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
    let claims = TestJwtClaims {
        iss: "https://auth.xinfra.ru".to_string(),
        sub: "user-123".to_string(),
        aud: "https://mcp-forgejo.xinfra.ru/mcp".to_string(),
        exp: now + 3600,
        nbf: now - 10,
        iat: now,
    };

    let mut header = Header::new(jsonwebtoken::Algorithm::RS256);
    header.kid = Some("test-key-id-1".to_string());

    let token = encode(
        &header,
        &claims,
        &EncodingKey::from_rsa_pem(TEST_PRIVATE_KEY_PEM.as_bytes()).unwrap(),
    )
    .unwrap();

    let app = create_app(state);
    let ping_req = json!({
        "jsonrpc": "2.0",
        "id": 100,
        "method": "ping"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/mcp")
                .method("POST")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&ping_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: JsonRpcResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(json.id, Some(json!(100)));
    assert!(json.result.is_some());
}

#[tokio::test]
async fn test_authelia_rs256_jwt_auth_expired() {
    let state = AppState::new(test_config());

    let test_jwk = Jwk {
        kty: "RSA".to_string(),
        kid: Some("test-key-id-1".to_string()),
        alg: Some("RS256".to_string()),
        r#use: Some("sig".to_string()),
        n: TEST_JWK_N.to_string(),
        e: TEST_JWK_E.to_string(),
    };
    {
        let mut cache = state.jwks_cache.write().await;
        cache.jwks = Some(Jwks {
            keys: vec![test_jwk],
        });
    }

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
    let claims = TestJwtClaims {
        iss: "https://auth.xinfra.ru".to_string(),
        sub: "user-123".to_string(),
        aud: "https://mcp-forgejo.xinfra.ru/mcp".to_string(),
        exp: now - 3600, // expired
        nbf: now - 7200,
        iat: now - 7200,
    };

    let mut header = Header::new(jsonwebtoken::Algorithm::RS256);
    header.kid = Some("test-key-id-1".to_string());

    let token = encode(
        &header,
        &claims,
        &EncodingKey::from_rsa_pem(TEST_PRIVATE_KEY_PEM.as_bytes()).unwrap(),
    )
    .unwrap();

    let app = create_app(state);
    let ping_req = json!({
        "jsonrpc": "2.0",
        "id": 101,
        "method": "ping"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/mcp")
                .method("POST")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&ping_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let auth_header = response.headers().get(header::WWW_AUTHENTICATE);
    assert!(auth_header.is_some());
    let auth_val = auth_header.unwrap().to_str().unwrap();
    assert!(auth_val.contains("error=\"invalid_token\""));
}

#[tokio::test]
async fn test_authelia_rs256_jwt_auth_missing_kid() {
    let state = AppState::new(test_config());

    let test_jwk = Jwk {
        kty: "RSA".to_string(),
        kid: Some("test-key-id-1".to_string()),
        alg: Some("RS256".to_string()),
        r#use: Some("sig".to_string()),
        n: TEST_JWK_N.to_string(),
        e: TEST_JWK_E.to_string(),
    };
    {
        let mut cache = state.jwks_cache.write().await;
        cache.jwks = Some(Jwks {
            keys: vec![test_jwk],
        });
    }

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
    let claims = TestJwtClaims {
        iss: "https://auth.xinfra.ru".to_string(),
        sub: "user-123".to_string(),
        aud: "https://mcp-forgejo.xinfra.ru/mcp".to_string(),
        exp: now + 3600,
        nbf: now - 10,
        iat: now,
    };

    // Header without kid
    let header = Header::new(jsonwebtoken::Algorithm::RS256);

    let token = encode(
        &header,
        &claims,
        &EncodingKey::from_rsa_pem(TEST_PRIVATE_KEY_PEM.as_bytes()).unwrap(),
    )
    .unwrap();

    let app = create_app(state);
    let ping_req = json!({
        "jsonrpc": "2.0",
        "id": 102,
        "method": "ping"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/mcp")
                .method("POST")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&ping_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let auth_header = response.headers().get(header::WWW_AUTHENTICATE);
    assert!(auth_header.is_some());
    let auth_val = auth_header.unwrap().to_str().unwrap();
    assert!(auth_val.contains("error=\"invalid_token\""));
}

#[tokio::test]
async fn test_authelia_rs256_jwks_refetch_cooldown() {
    let state = AppState::new(test_config());

    // Pre-populate JWKS cache with a key and recent last_fetched timestamp
    let test_jwk = Jwk {
        kty: "RSA".to_string(),
        kid: Some("known-key".to_string()),
        alg: Some("RS256".to_string()),
        r#use: Some("sig".to_string()),
        n: TEST_JWK_N.to_string(),
        e: TEST_JWK_E.to_string(),
    };
    {
        let mut cache = state.jwks_cache.write().await;
        cache.jwks = Some(Jwks {
            keys: vec![test_jwk],
        });
        cache.last_fetched = Some(std::time::Instant::now());
    }

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
    let claims = TestJwtClaims {
        iss: "https://auth.xinfra.ru".to_string(),
        sub: "user-123".to_string(),
        aud: "https://mcp-forgejo.xinfra.ru/mcp".to_string(),
        exp: now + 3600,
        nbf: now - 10,
        iat: now,
    };

    // Header with unknown kid
    let mut header = Header::new(jsonwebtoken::Algorithm::RS256);
    header.kid = Some("unknown-attacker-key".to_string());

    let token = encode(
        &header,
        &claims,
        &EncodingKey::from_rsa_pem(TEST_PRIVATE_KEY_PEM.as_bytes()).unwrap(),
    )
    .unwrap();

    let app = create_app(state);
    let ping_req = json!({
        "jsonrpc": "2.0",
        "id": 103,
        "method": "ping"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/mcp")
                .method("POST")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&ping_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let auth_header = response.headers().get(header::WWW_AUTHENTICATE);
    assert!(auth_header.is_some());
    let auth_val = auth_header.unwrap().to_str().unwrap();
    assert!(auth_val.contains("error=\"invalid_token\""));
}

#[test]
fn test_jsonrpc_null_id_serialization() {
    let resp = JsonRpcResponse::success(None, json!({ "status": "ok" }));
    let serialized = serde_json::to_string(&resp).unwrap();
    assert!(
        serialized.contains(r#""id":null"#),
        "JsonRpcResponse must serialize id as null when None: got {}",
        serialized
    );
}

#[tokio::test]
async fn test_authelia_cold_cache_miss_updates_last_fetched_and_cooldown() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use axum::routing::get;
    use axum::Json;
    use axum::Router;

    let fetch_count = Arc::new(AtomicUsize::new(0));
    let fetch_count_clone = fetch_count.clone();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let mock_auth = Router::new().route(
        "/jwks.json",
        get(move || {
            let count = fetch_count_clone.clone();
            async move {
                count.fetch_add(1, Ordering::SeqCst);
                Json(json!({
                    "keys": [{
                        "kty": "RSA",
                        "kid": "mock-key-only",
                        "alg": "RS256",
                        "use": "sig",
                        "n": TEST_JWK_N,
                        "e": TEST_JWK_E
                    }]
                }))
            }
        }),
    );

    tokio::spawn(async move {
        axum::serve(listener, mock_auth).await.unwrap();
    });

    let config = AppConfig {
        port: 8000,
        host: "0.0.0.0".to_string(),
        authelia_url: format!("http://{}", addr),
        resource_url: "https://mcp-forgejo.xinfra.ru/mcp".to_string(),
        static_api_key: None,
        target_url: "http://forgejo:3000".to_string(),
        target_token: "".to_string(),
    };

    let state = AppState::new(config);
    // Cold cache: jwks is None, last_fetched is None
    assert!(state.jwks_cache.read().await.last_fetched.is_none());

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
    let claims = TestJwtClaims {
        iss: format!("http://{}", addr),
        sub: "user-123".to_string(),
        aud: "https://mcp-forgejo.xinfra.ru/mcp".to_string(),
        exp: now + 3600,
        nbf: now - 10,
        iat: now,
    };

    let mut header = Header::new(jsonwebtoken::Algorithm::RS256);
    header.kid = Some("unknown-kid".to_string());

    let token = encode(
        &header,
        &claims,
        &EncodingKey::from_rsa_pem(TEST_PRIVATE_KEY_PEM.as_bytes()).unwrap(),
    )
    .unwrap();

    let app = create_app(state.clone());
    let ping_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "ping"
    });

    // 1st request: cold cache triggers remote fetch, kid not found
    let response1 = app.clone()
        .oneshot(
            Request::builder()
                .uri("/mcp")
                .method("POST")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&ping_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response1.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(fetch_count.load(Ordering::SeqCst), 1);

    // Verify cache.last_fetched was updated despite kid not being in the keyset!
    assert!(state.jwks_cache.read().await.last_fetched.is_some());
    assert!(state.jwks_cache.read().await.jwks.is_some());

    // 2nd request: cooldown active, should NOT fetch remotely again
    let response2 = app
        .oneshot(
            Request::builder()
                .uri("/mcp")
                .method("POST")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&ping_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response2.status(), StatusCode::UNAUTHORIZED);
    // fetch_count must remain 1 because cooldown prevented remote refetch
    assert_eq!(fetch_count.load(Ordering::SeqCst), 1);
}
