use std::time::{Duration, Instant};
use axum::{
    body::Body,
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use tracing::{debug, warn};
use url::Url;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jwk {
    pub kty: String,
    #[serde(default)]
    pub kid: Option<String>,
    #[serde(default)]
    pub alg: Option<String>,
    #[serde(default)]
    pub r#use: Option<String>,
    pub n: String,
    pub e: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Jwks {
    pub keys: Vec<Jwk>,
}

#[derive(Clone, Default)]
pub struct JwksCache {
    pub jwks: Option<Jwks>,
    pub last_fetched: Option<Instant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    #[serde(default)]
    pub iss: Option<String>,
    #[serde(default)]
    pub sub: Option<String>,
    #[serde(default)]
    pub aud: Option<serde_json::Value>,
    #[serde(default)]
    pub exp: Option<usize>,
    #[serde(default)]
    pub nbf: Option<usize>,
    #[serde(default)]
    pub iat: Option<usize>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub client_id: Option<String>,
}

impl Claims {
    pub fn static_token() -> Self {
        Self {
            iss: Some("static".to_string()),
            sub: Some("static-client".to_string()),
            aud: None,
            exp: None,
            nbf: None,
            iat: None,
            scope: Some("all".to_string()),
            client_id: Some("static".to_string()),
        }
    }
}

pub fn resource_metadata_url(resource_url: &str) -> String {
    if let Ok(mut url) = Url::parse(resource_url) {
        url.set_path("/.well-known/oauth-protected-resource");
        url.to_string()
    } else {
        "https://mcp-forgejo.xinfra.ru/.well-known/oauth-protected-resource".to_string()
    }
}

pub fn unauthorized_response(resource_url: &str, error: Option<&str>) -> Response {
    let metadata_url = resource_metadata_url(resource_url);
    let auth_header = match error {
        Some(err) => format!(
            "Bearer resource_metadata=\"{}\", error=\"{}\"",
            metadata_url, err
        ),
        None => format!("Bearer resource_metadata=\"{}\"", metadata_url),
    };

    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(header::WWW_AUTHENTICATE, auth_header)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "error": "unauthorized",
                "message": "Missing or invalid bearer token"
            })
            .to_string(),
        ))
        .unwrap()
}

pub fn constant_time_eq_str(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.as_bytes().ct_eq(b.as_bytes()).into()
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(h) if h.starts_with("Bearer ") => &h[7..],
        _ => return unauthorized_response(&state.config.resource_url, None),
    };

    // 1. Check static token fallback using constant-time comparison
    if let Some(ref static_key) = state.config.static_api_key {
        if constant_time_eq_str(token, static_key) {
            debug!("Request authenticated via static API key");
            req.extensions_mut().insert(Claims::static_token());
            return next.run(req).await;
        }
    }

    // 2. RS256 JWT validation against cached JWKS
    match validate_jwt(token, &state).await {
        Ok(claims) => {
            debug!("Request authenticated via Authelia JWT");
            req.extensions_mut().insert(claims);
            next.run(req).await
        }
        Err(err) => {
            warn!("JWT validation failed: {}", err);
            unauthorized_response(&state.config.resource_url, Some("invalid_token"))
        }
    }
}

pub async fn validate_jwt(token: &str, state: &AppState) -> Result<Claims, String> {
    let header = decode_header(token).map_err(|e| format!("Invalid JWT header: {e}"))?;
    if header.alg != Algorithm::RS256 {
        return Err(format!("Unsupported algorithm: {:?}", header.alg));
    }

    let kid = header
        .kid
        .as_deref()
        .ok_or_else(|| "Missing 'kid' in JWT header".to_string())?;

    let jwk = get_jwk(kid, state).await?;
    let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
        .map_err(|e| format!("Invalid RSA components: {e}"))?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[&state.config.authelia_url]);

    let mut audiences = vec![state.config.resource_url.clone()];
    if let Ok(url) = Url::parse(&state.config.resource_url) {
        if let Some(host) = url.host_str() {
            audiences.push(host.to_string());
            audiences.push(format!("https://{}", host));
        }
    }
    validation.set_audience(&audiences);

    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|e| format!("Token decoding error: {e}"))?;

    Ok(token_data.claims)
}

async fn get_jwk(kid: &str, state: &AppState) -> Result<Jwk, String> {
    const JWKS_REFETCH_COOLDOWN: Duration = Duration::from_secs(60);

    // 1. Check cache first
    {
        let cache = state.jwks_cache.read().await;
        if let Some(ref jwks) = cache.jwks {
            if let Some(found) = find_jwk(jwks, kid) {
                return Ok(found.clone());
            }

            // Cooldown check: if cache was fetched recently, do not refetch from Authelia
            if let Some(last_fetched) = cache.last_fetched {
                if last_fetched.elapsed() < JWKS_REFETCH_COOLDOWN {
                    return Err(format!(
                        "Key ID '{}' not found in JWKS cache (refetch cooldown active)",
                        kid
                    ));
                }
            }
        }
    }

    // 2. Fetch fresh JWKS if not found in cache and cooldown allows
    let fetched_jwks = fetch_jwks(&state.config.authelia_url).await?;
    let found = find_jwk(&fetched_jwks, kid).cloned();

    // Update cache regardless of whether kid was found in the new keyset
    {
        let mut cache = state.jwks_cache.write().await;
        cache.jwks = Some(fetched_jwks);
        cache.last_fetched = Some(Instant::now());
    }

    found.ok_or_else(|| format!("Key ID '{}' not found in JWKS", kid))
}

pub fn find_jwk<'a>(jwks: &'a Jwks, kid: &str) -> Option<&'a Jwk> {
    jwks.keys.iter().find(|k| k.kid.as_deref() == Some(kid))
}

async fn fetch_jwks(authelia_url: &str) -> Result<Jwks, String> {
    let jwks_url = format!("{}/jwks.json", authelia_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

    let res = client
        .get(&jwks_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch JWKS from {jwks_url}: {e}"))?;

    if !res.status().is_success() {
        return Err(format!("JWKS endpoint returned status {}", res.status()));
    }

    let jwks: Jwks = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse JWKS JSON: {e}"))?;

    Ok(jwks)
}
