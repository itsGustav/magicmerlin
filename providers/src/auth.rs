//! Auth profile loading, API key rotation, and optional OAuth refresh.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use reqwest::header::{HeaderName, HeaderValue, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::error::{ProviderError, Result};

/// OAuth token refresh config for one provider profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokenConfig {
    /// OAuth token endpoint.
    pub token_url: String,
    /// OAuth client ID.
    pub client_id: String,
    /// OAuth client secret.
    pub client_secret: String,
    /// Refresh token.
    pub refresh_token: String,
    /// Current access token if available.
    pub access_token: Option<String>,
    /// Expiry epoch seconds for current access token.
    pub expires_at_epoch: Option<u64>,
    /// Optional static scopes.
    #[serde(default)]
    pub scope: Option<String>,
    /// Optional audience.
    #[serde(default)]
    pub audience: Option<String>,
}

/// Auth profile entry loaded from `auth-profiles.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthProfile {
    /// Provider key.
    pub provider: String,
    /// API keys for round-robin usage.
    #[serde(default)]
    pub api_keys: Vec<String>,
    /// Optional custom header for API key (`authorization` or `x-api-key`).
    pub header: Option<String>,
    /// Optional OAuth refresh config.
    pub oauth: Option<OAuthTokenConfig>,
}

/// Health snapshot for one auth provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthHealth {
    /// Provider name.
    pub provider: String,
    /// Whether at least one auth mechanism is configured.
    pub configured: bool,
    /// Number of API keys configured.
    pub key_count: usize,
    /// Current key index for round-robin.
    pub next_key_idx: usize,
    /// Whether OAuth is enabled.
    pub oauth_enabled: bool,
    /// OAuth token expiry if present.
    pub oauth_expires_at_epoch: Option<u64>,
    /// Seconds until expiry (0 when absent/expired).
    pub oauth_ttl_seconds: u64,
}

/// Internal mutable auth state for one provider.
#[derive(Debug)]
struct ProviderAuthState {
    profile: AuthProfile,
    next_key_idx: usize,
}

/// Auth profile set with runtime mutation for key rotation and OAuth refresh.
#[derive(Clone, Debug, Default)]
pub struct AuthProfiles {
    state: Arc<Mutex<HashMap<String, ProviderAuthState>>>,
}

impl AuthProfiles {
    /// Builds auth profiles from a list.
    pub fn from_profiles(profiles: Vec<AuthProfile>) -> Self {
        let mut map = HashMap::new();
        for profile in profiles {
            map.insert(
                profile.provider.clone(),
                ProviderAuthState {
                    profile,
                    next_key_idx: 0,
                },
            );
        }
        Self {
            state: Arc::new(Mutex::new(map)),
        }
    }

    /// Loads auth profiles from JSON file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let raw = fs::read_to_string(&path).map_err(|source| ProviderError::Io {
            path: path.clone(),
            source,
        })?;

        #[derive(Deserialize)]
        struct FileEnvelope {
            profiles: Vec<AuthProfile>,
        }

        let parsed = serde_json::from_str::<FileEnvelope>(&raw)?;
        Ok(Self::from_profiles(parsed.profiles))
    }

    /// Attempts to load auth profiles from state directory path.
    pub fn load_from_state_dir(state_dir: impl AsRef<Path>) -> Result<Self> {
        let path = state_dir.as_ref().join("auth-profiles.json");
        if !path.exists() {
            return Ok(Self::default());
        }
        Self::load(path)
    }

    /// Returns auth header name and value for provider.
    pub async fn header_for_provider(
        &self,
        provider: &str,
        client: &reqwest::Client,
    ) -> Result<(HeaderName, HeaderValue)> {
        let (header_name, token) = self.next_token(provider, client).await?;
        if token.is_empty() {
            return Err(ProviderError::MissingAuth(provider.to_string()));
        }

        let value = if header_name == AUTHORIZATION {
            HeaderValue::from_str(&format!("Bearer {token}"))
        } else {
            HeaderValue::from_str(&token)
        }
        .map_err(|err| ProviderError::OAuthRefresh {
            provider: provider.to_string(),
            message: format!("invalid header value: {err}"),
        })?;

        Ok((header_name, value))
    }

    /// Rotates API key index for provider after rate limit.
    pub async fn rotate_key(&self, provider: &str) {
        let mut lock = self.state.lock().await;
        if let Some(state) = lock.get_mut(provider) {
            if !state.profile.api_keys.is_empty() {
                state.next_key_idx = (state.next_key_idx + 1) % state.profile.api_keys.len();
            }
        }
    }

    /// Adds an API key to an existing provider.
    pub async fn add_api_key(&self, provider: &str, key: String) -> Result<()> {
        let mut lock = self.state.lock().await;
        let Some(state) = lock.get_mut(provider) else {
            return Err(ProviderError::MissingAuth(provider.to_string()));
        };
        if key.is_empty() {
            return Err(ProviderError::InvalidRequest(
                "api key cannot be empty".to_string(),
            ));
        }
        state.profile.api_keys.push(key);
        Ok(())
    }

    /// Upserts a full auth profile definition.
    pub async fn upsert_profile(&self, profile: AuthProfile) {
        let mut lock = self.state.lock().await;
        lock.insert(
            profile.provider.clone(),
            ProviderAuthState {
                profile,
                next_key_idx: 0,
            },
        );
    }

    /// Returns auth health snapshot for all providers.
    pub async fn health_report(&self) -> Vec<AuthHealth> {
        let lock = self.state.lock().await;
        lock.iter()
            .map(|(provider, state)| {
                let expires_at = state
                    .profile
                    .oauth
                    .as_ref()
                    .and_then(|oauth| oauth.expires_at_epoch);
                let now = now_epoch();
                let ttl = expires_at
                    .map(|expiry| expiry.saturating_sub(now))
                    .unwrap_or(0);

                AuthHealth {
                    provider: provider.clone(),
                    configured: !state.profile.api_keys.is_empty() || state.profile.oauth.is_some(),
                    key_count: state.profile.api_keys.len(),
                    next_key_idx: state.next_key_idx,
                    oauth_enabled: state.profile.oauth.is_some(),
                    oauth_expires_at_epoch: expires_at,
                    oauth_ttl_seconds: ttl,
                }
            })
            .collect()
    }

    /// Returns a sanitized health summary JSON object.
    pub async fn health_json(&self) -> serde_json::Value {
        let report = self.health_report().await;
        serde_json::json!({
            "providers": report,
            "generated_at": now_epoch(),
        })
    }

    async fn next_token(
        &self,
        provider: &str,
        client: &reqwest::Client,
    ) -> Result<(HeaderName, String)> {
        let mut lock = self.state.lock().await;
        let state = lock
            .get_mut(provider)
            .ok_or_else(|| ProviderError::MissingAuth(provider.to_string()))?;

        let header = parse_header_name(state.profile.header.as_deref());

        if let Some(oauth) = state.profile.oauth.as_mut() {
            let now = now_epoch();
            let expired = oauth
                .expires_at_epoch
                .map(|ts| ts <= now + 60)
                .unwrap_or(oauth.access_token.is_none());
            if expired {
                refresh_oauth_token(provider, oauth, client).await?;
            }
            if let Some(token) = oauth.access_token.clone() {
                return Ok((AUTHORIZATION, token));
            }
        }

        if state.profile.api_keys.is_empty() {
            return Err(ProviderError::MissingAuth(provider.to_string()));
        }

        let idx = state.next_key_idx % state.profile.api_keys.len();
        Ok((header, state.profile.api_keys[idx].clone()))
    }
}

fn parse_header_name(header: Option<&str>) -> HeaderName {
    if let Some(name) = header {
        if name.eq_ignore_ascii_case("x-api-key") {
            return HeaderName::from_static("x-api-key");
        }
        if name.eq_ignore_ascii_case("api-key") {
            return HeaderName::from_static("api-key");
        }
    }
    AUTHORIZATION
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}

async fn refresh_oauth_token(
    provider: &str,
    oauth: &mut OAuthTokenConfig,
    client: &reqwest::Client,
) -> Result<()> {
    let mut form = vec![
        ("grant_type", "refresh_token".to_string()),
        ("client_id", oauth.client_id.clone()),
        ("client_secret", oauth.client_secret.clone()),
        ("refresh_token", oauth.refresh_token.clone()),
    ];
    if let Some(scope) = oauth.scope.clone() {
        form.push(("scope", scope));
    }
    if let Some(audience) = oauth.audience.clone() {
        form.push(("audience", audience));
    }

    let resp = client.post(&oauth.token_url).form(&form).send().await?;
    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        return Err(ProviderError::OAuthRefresh {
            provider: provider.to_string(),
            message: format!("status={status} body={body}"),
        });
    }

    #[derive(Deserialize)]
    struct TokenResponse {
        access_token: String,
        expires_in: Option<u64>,
    }

    let token = resp.json::<TokenResponse>().await?;
    oauth.access_token = Some(token.access_token);
    oauth.expires_at_epoch = token.expires_in.map(|ttl| now_epoch() + ttl);
    Ok(())
}

/// Returns canonical default auth profile path for config state.
pub fn default_auth_profiles_path(state_paths: &magicmerlin_config::StatePaths) -> PathBuf {
    state_paths.state_dir.join("auth-profiles.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn generates_authorization_header_and_rotates() {
        let profiles = AuthProfiles::from_profiles(vec![AuthProfile {
            provider: "openai".to_string(),
            api_keys: vec!["k1".to_string(), "k2".to_string()],
            header: None,
            oauth: None,
        }]);

        let client = reqwest::Client::new();
        let (_, v1) = profiles
            .header_for_provider("openai", &client)
            .await
            .expect("header");
        assert_eq!(v1.to_str().expect("str"), "Bearer k1");

        profiles.rotate_key("openai").await;
        let (_, v2) = profiles
            .header_for_provider("openai", &client)
            .await
            .expect("header");
        assert_eq!(v2.to_str().expect("str"), "Bearer k2");
    }

    #[tokio::test]
    async fn generates_x_api_key_for_anthropic() {
        let profiles = AuthProfiles::from_profiles(vec![AuthProfile {
            provider: "anthropic".to_string(),
            api_keys: vec!["ant-key".to_string()],
            header: Some("x-api-key".to_string()),
            oauth: None,
        }]);

        let client = reqwest::Client::new();
        let (name, value) = profiles
            .header_for_provider("anthropic", &client)
            .await
            .expect("header");
        assert_eq!(name, HeaderName::from_static("x-api-key"));
        assert_eq!(value.to_str().expect("str"), "ant-key");
    }

    #[tokio::test]
    async fn reports_health_without_exposing_tokens() {
        let profiles = AuthProfiles::from_profiles(vec![AuthProfile {
            provider: "google".to_string(),
            api_keys: vec!["g1".to_string()],
            header: Some("x-api-key".to_string()),
            oauth: None,
        }]);

        let health = profiles.health_json().await.to_string();
        assert!(health.contains("google"));
        assert!(!health.contains("g1"));
    }
}
