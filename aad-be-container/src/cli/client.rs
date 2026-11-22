//! Authenticated REST API client and URL normalization helpers for `aad-be ctl`.

use reqwest::{Client, Method, RequestBuilder};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt;
use std::fs;
use std::path::Path;
use std::time::Duration;
use uuid::Uuid;

use crate::cli::error::CtlError;
use crate::config::WebServiceConfig;
use crate::models::agent::Agent;
use crate::models::common::ListPages;
use crate::models::skill::Skill;
use crate::models::tool::{RegisterToolRequest, RegisterToolResponse, SyncToolResponse, Tool};
use crate::models::traits::TraitContract;

/// Normalizes a user-supplied base URL (strips trailing slashes, validates scheme).
pub fn normalize_base_url(raw: &str) -> Result<String, CtlError> {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err(CtlError::InvalidUrl {
            url: raw.to_string(),
            reason: "URL cannot be empty".to_string(),
        });
    }
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return Err(CtlError::InvalidUrl {
            url: raw.to_string(),
            reason: "URL must start with 'http://' or 'https://'".to_string(),
        });
    }
    url::Url::parse(trimmed).map_err(|e| CtlError::InvalidUrl {
        url: raw.to_string(),
        reason: e.to_string(),
    })?;
    Ok(trimmed.to_string())
}

/// Derives a normalized REST API base URL from `WebServiceConfig` without double slashes
/// or duplicated `/v1` path segments.
pub fn base_url_from_webservice_config(ws: &WebServiceConfig) -> Result<String, CtlError> {
    let host_port = ws.address.to_string().replace("0.0.0.0", "127.0.0.1");
    let prefix = ws.api_prefix.trim_matches('/');
    let raw = if prefix.is_empty() {
        format!("http://{}", host_port)
    } else {
        format!("http://{}/{}", host_port, prefix)
    };
    normalize_base_url(&raw)
}

/// Loads a Bearer JWT from either `--token` or `--token-file` (trimmed).
pub fn resolve_bearer_token(
    token: Option<&str>,
    token_file: Option<&Path>,
) -> Result<Option<String>, CtlError> {
    if let Some(raw) = token {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }
        return Ok(Some(trimmed.to_string()));
    }

    let Some(path) = token_file else {
        return Ok(None);
    };
    let raw = fs::read_to_string(path).map_err(|source| CtlError::Io {
        path: path.display().to_string(),
        source,
    })?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    Ok(Some(trimmed.to_string()))
}

/// Authenticated HTTP client for the AAD REST API.
/// Redacts Bearer token material in `Debug` output.
pub struct AadApiClient {
    base_url: String,
    bearer_token: Option<String>,
    http: Client,
}

impl fmt::Debug for AadApiClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AadApiClient")
            .field("base_url", &self.base_url)
            .field(
                "bearer_token",
                &self.bearer_token.as_ref().map(|_| "<redacted>"),
            )
            .finish()
    }
}

impl AadApiClient {
    pub fn new(
        base_url: String,
        bearer_token: Option<String>,
        timeout: Duration,
    ) -> Result<Self, CtlError> {
        let normalized = normalize_base_url(&base_url)?;
        let http = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| CtlError::Config(format!("Failed to build HTTP client: {}", e)))?;
        Ok(Self {
            base_url: normalized,
            bearer_token,
            http,
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    fn endpoint(&self, path: &str) -> String {
        format!("{}/{}", self.base_url, path.trim_start_matches('/'))
    }

    fn request(&self, method: Method, url: &str) -> RequestBuilder {
        let mut req = self.http.request(method, url);
        if let Some(token) = &self.bearer_token {
            req = req.bearer_auth(token);
        }
        req
    }

    async fn send_json<T: DeserializeOwned>(
        &self,
        method: Method,
        url: &str,
        body: Option<&impl Serialize>,
    ) -> Result<T, CtlError> {
        let mut req = self.request(method.clone(), url);
        if let Some(payload) = body {
            req = req.json(payload);
        }
        let resp = req.send().await.map_err(|source| CtlError::HttpTransport {
            method: method.to_string(),
            url: url.to_string(),
            source,
        })?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(CtlError::ApiError {
                method: method.to_string(),
                url: url.to_string(),
                status: status.as_u16(),
                body: text,
            });
        }

        resp.json::<T>().await.map_err(|source| CtlError::HttpTransport {
            method: method.to_string(),
            url: url.to_string(),
            source,
        })
    }

    async fn send_empty(&self, method: Method, url: &str) -> Result<(), CtlError> {
        let resp = self
            .request(method.clone(), url)
            .send()
            .await
            .map_err(|source| CtlError::HttpTransport {
                method: method.to_string(),
                url: url.to_string(),
                source,
            })?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(CtlError::ApiError {
                method: method.to_string(),
                url: url.to_string(),
                status: status.as_u16(),
                body: text,
            });
        }
        Ok(())
    }

    // --- Traits ---

    /// Paginates `GET /traits` (`ListPages`) and fetches full `TraitContract` records.
    pub async fn list_traits(&self) -> Result<Vec<TraitContract>, CtlError> {
        const PAGE_SIZE: i64 = 100;
        let mut page = 0;
        let mut all_ids = Vec::new();

        loop {
            let url = format!(
                "{}?page={}&size={}",
                self.endpoint("traits"),
                page,
                PAGE_SIZE
            );
            let list: ListPages = self
                .send_json::<ListPages>(Method::GET, &url, Option::<&()>::None)
                .await?;
            let count = list.ids.len() as i64;
            all_ids.extend(list.ids);
            if count < PAGE_SIZE {
                break;
            }
            page += 1;
        }

        let mut traits = Vec::with_capacity(all_ids.len());
        for id in all_ids {
            traits.push(self.get_trait(id).await?);
        }
        Ok(traits)
    }

    pub async fn get_trait(&self, id: Uuid) -> Result<TraitContract, CtlError> {
        let url = self.endpoint(&format!("traits/{}", id));
        self.send_json(Method::GET, &url, Option::<&()>::None).await
    }

    pub async fn upsert_trait(&self, payload: &TraitContract) -> Result<TraitContract, CtlError> {
        let url = self.endpoint("traits");
        self.send_json(Method::POST, &url, Some(payload)).await
    }

    pub async fn delete_trait(&self, id: Uuid) -> Result<(), CtlError> {
        let url = self.endpoint(&format!("traits/{}", id));
        self.send_empty(Method::DELETE, &url).await
    }

    // --- Tools ---

    pub async fn list_tools(&self) -> Result<Vec<Tool>, CtlError> {
        let url = self.endpoint("agents/tools");
        self.send_json(Method::GET, &url, Option::<&()>::None).await
    }

    pub async fn get_tool(&self, id: Uuid) -> Result<Tool, CtlError> {
        let url = self.endpoint(&format!("agents/tools/{}", id));
        self.send_json(Method::GET, &url, Option::<&()>::None).await
    }

    pub async fn register_tool(
        &self,
        payload: &RegisterToolRequest,
    ) -> Result<RegisterToolResponse, CtlError> {
        let url = self.endpoint("agents/tools/register");
        self.send_json(Method::POST, &url, Some(payload)).await
    }

    pub async fn sync_tool(&self, id: Uuid) -> Result<SyncToolResponse, CtlError> {
        let url = self.endpoint(&format!("agents/tools/{}/sync", id));
        self.send_json(Method::POST, &url, Option::<&()>::None).await
    }

    pub async fn delete_tool(&self, id: Uuid) -> Result<(), CtlError> {
        let url = self.endpoint(&format!("agents/tools/{}", id));
        self.send_empty(Method::DELETE, &url).await
    }

    // --- Skills ---

    pub async fn list_skills(&self) -> Result<Vec<Skill>, CtlError> {
        let url = self.endpoint("skills");
        self.send_json(Method::GET, &url, Option::<&()>::None).await
    }

    pub async fn get_skill(&self, id: Uuid) -> Result<Skill, CtlError> {
        let url = self.endpoint(&format!("skills/{}", id));
        self.send_json(Method::GET, &url, Option::<&()>::None).await
    }

    pub async fn upsert_skill(&self, payload: &Skill) -> Result<Skill, CtlError> {
        let url = self.endpoint("skills");
        self.send_json(Method::POST, &url, Some(payload)).await
    }

    pub async fn delete_skill(&self, id: Uuid) -> Result<(), CtlError> {
        let url = self.endpoint(&format!("skills/{}", id));
        self.send_empty(Method::DELETE, &url).await
    }

    // --- Agents ---

    pub async fn list_agents(&self) -> Result<Vec<Agent>, CtlError> {
        let url = self.endpoint("agents");
        self.send_json(Method::GET, &url, Option::<&()>::None).await
    }

    pub async fn get_agent(&self, id: Uuid) -> Result<Agent, CtlError> {
        let url = self.endpoint(&format!("agents/{}", id));
        self.send_json(Method::GET, &url, Option::<&()>::None).await
    }

    pub async fn create_agent(&self, payload: &Agent) -> Result<Agent, CtlError> {
        let url = self.endpoint("agents");
        self.send_json(Method::POST, &url, Some(payload)).await
    }

    pub async fn update_agent(&self, id: Uuid, payload: &Agent) -> Result<Agent, CtlError> {
        let url = self.endpoint(&format!("agents/{}", id));
        self.send_json(Method::PUT, &url, Some(payload)).await
    }

    pub async fn delete_agent(&self, id: Uuid, hard: bool) -> Result<(), CtlError> {
        let url = if hard {
            format!("{}?hard=true", self.endpoint(&format!("agents/{}", id)))
        } else {
            self.endpoint(&format!("agents/{}", id))
        };
        self.send_empty(Method::DELETE, &url).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_base_url_strips_trailing_slash() {
        assert_eq!(
            normalize_base_url("http://localhost:8080/api/aad/v1/").unwrap(),
            "http://localhost:8080/api/aad/v1"
        );
        assert_eq!(
            normalize_base_url("https://aad.k8s/api/aad/v1///").unwrap(),
            "https://aad.k8s/api/aad/v1"
        );
    }

    #[test]
    fn test_normalize_base_url_rejects_missing_scheme() {
        let err = normalize_base_url("localhost:8080/api/aad/v1").unwrap_err();
        assert!(matches!(err, CtlError::InvalidUrl { .. }));
    }

    #[test]
    fn test_base_url_from_webservice_config_avoids_double_slash_and_duplicate_v1() {
        let ws = WebServiceConfig {
            address: "0.0.0.0:8080".parse().unwrap(),
            api_prefix: "/api/aad/v1".to_string(),
        };
        let url = base_url_from_webservice_config(&ws).unwrap();
        assert_eq!(url, "http://127.0.0.1:8080/api/aad/v1");
        assert!(!url.contains("//api"));
        assert!(!url.contains("/v1/v1"));
    }

    #[test]
    fn test_debug_redacts_bearer_token() {
        let client = AadApiClient::new(
            "http://localhost:8080/api/aad/v1".to_string(),
            Some("super-secret-jwt-token".to_string()),
            Duration::from_secs(5),
        )
        .unwrap();
        let debug_str = format!("{:?}", client);
        assert!(!debug_str.contains("super-secret-jwt-token"));
        assert!(debug_str.contains("<redacted>"));
    }
}
