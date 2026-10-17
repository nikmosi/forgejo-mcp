use reqwest::{Client, Method, RequestBuilder};
use serde::Serialize;
use tracing::debug;

#[derive(Clone, Debug)]
pub struct ForgejoClient {
    pub base_url: String,
    pub token: String,
    client: Client,
}

impl ForgejoClient {
    pub fn new(base_url: impl Into<String>, token: impl Into<String>) -> Self {
        Self::with_client(base_url, token, Client::new())
    }

    pub fn with_client(
        base_url: impl Into<String>,
        token: impl Into<String>,
        client: Client,
    ) -> Self {
        let base = base_url.into().trim_end_matches('/').to_string();
        Self {
            base_url: base,
            token: token.into(),
            client,
        }
    }

    fn api_url(&self, path: &str) -> String {
        let base = self.base_url.trim_end_matches('/');
        let api_base = if base.ends_with("/api/v1") {
            base.to_string()
        } else {
            format!("{}/api/v1", base)
        };
        let clean_path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{}", path)
        };
        format!("{}{}", api_base, clean_path)
    }

    fn request_builder(&self, method: Method, path: &str) -> RequestBuilder {
        let url = self.api_url(path);
        let mut builder = self.client.request(method, url);
        if !self.token.is_empty() {
            builder = builder.header("Authorization", format!("token {}", self.token));
        }
        builder
    }

    async fn execute_request(&self, req: RequestBuilder) -> Result<String, String> {
        let resp = req
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read response body: {}", e))?;

        if status.is_success() {
            if body.trim().is_empty() {
                Ok("{}".to_string())
            } else {
                Ok(body)
            }
        } else {
            Err(format!(
                "Forgejo API error (status {}): {}",
                status,
                body.trim()
            ))
        }
    }

    pub async fn get(&self, path: &str, query: &[(&str, &str)]) -> Result<String, String> {
        debug!("GET {} query={:?}", path, query);
        let mut req = self
            .request_builder(Method::GET, path)
            .header("Accept", "application/json");

        if !query.is_empty() {
            req = req.query(query);
        }

        self.execute_request(req).await
    }

    pub async fn get_raw(&self, path: &str, query: &[(&str, &str)]) -> Result<String, String> {
        debug!("GET raw {} query={:?}", path, query);
        let mut req = self
            .request_builder(Method::GET, path)
            .header("Accept", "*/*");

        if !query.is_empty() {
            req = req.query(query);
        }

        self.execute_request(req).await
    }

    pub async fn post<T: Serialize>(&self, path: &str, body: &T) -> Result<String, String> {
        debug!("POST {}", path);
        let req = self
            .request_builder(Method::POST, path)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(body);

        self.execute_request(req).await
    }

    pub async fn put<T: Serialize>(&self, path: &str, body: &T) -> Result<String, String> {
        debug!("PUT {}", path);
        let req = self
            .request_builder(Method::PUT, path)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(body);

        self.execute_request(req).await
    }

    pub async fn patch<T: Serialize>(&self, path: &str, body: &T) -> Result<String, String> {
        debug!("PATCH {}", path);
        let req = self
            .request_builder(Method::PATCH, path)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(body);

        self.execute_request(req).await
    }

    pub async fn delete(&self, path: &str) -> Result<String, String> {
        debug!("DELETE {}", path);
        let req = self
            .request_builder(Method::DELETE, path)
            .header("Accept", "application/json");

        self.execute_request(req).await
    }

    pub async fn delete_with_body<T: Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<String, String> {
        debug!("DELETE {} with body", path);
        let req = self
            .request_builder(Method::DELETE, path)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(body);

        self.execute_request(req).await
    }
}
