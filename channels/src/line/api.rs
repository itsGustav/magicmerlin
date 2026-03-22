//! LINE Messaging API HTTP client.
//!
//! Wraps reqwest to call `https://api.line.me/v2/bot/...` with channel access token auth.

use serde_json::Value;

use super::LineMessage;

/// Errors from LINE API calls.
#[derive(Debug, thiserror::Error)]
pub enum LineApiError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("LINE API error: {0}")]
    Api(String),
    #[error("unexpected response shape")]
    BadResponse,
}

type Result<T> = std::result::Result<T, LineApiError>;

/// LINE user profile.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct LineProfile {
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "pictureUrl")]
    pub picture_url: Option<String>,
    #[serde(rename = "statusMessage")]
    pub status_message: Option<String>,
}

/// HTTP client for the LINE Messaging API.
#[derive(Debug, Clone)]
pub struct LineApiClient {
    channel_access_token: String,
    http: reqwest::Client,
    base_url: String,
}

impl LineApiClient {
    pub fn new(channel_access_token: String) -> Self {
        Self {
            channel_access_token,
            http: reqwest::Client::new(),
            base_url: "https://api.line.me".to_string(),
        }
    }

    /// Override base URL (useful for testing).
    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    // ── Reply ───────────────────────────────────────────────────────────

    /// Reply to a webhook event using the reply token (expires in 30s).
    pub async fn reply_message(&self, reply_token: &str, messages: Vec<LineMessage>) -> Result<()> {
        let msgs: Vec<Value> = messages.iter().map(LineMessage::to_json).collect();
        let body = serde_json::json!({
            "replyToken": reply_token,
            "messages": msgs,
        });
        self.post("/v2/bot/message/reply", &body).await?;
        Ok(())
    }

    // ── Push ────────────────────────────────────────────────────────────

    /// Push a message to a user, group, or room (requires messaging API quota).
    pub async fn push_message(&self, to: &str, messages: Vec<LineMessage>) -> Result<()> {
        let msgs: Vec<Value> = messages.iter().map(LineMessage::to_json).collect();
        let body = serde_json::json!({
            "to": to,
            "messages": msgs,
        });
        self.post("/v2/bot/message/push", &body).await?;
        Ok(())
    }

    // ── Profile ─────────────────────────────────────────────────────────

    /// Get user profile.
    pub async fn get_profile(&self, user_id: &str) -> Result<LineProfile> {
        let url = format!("{}/v2/bot/profile/{}", self.base_url, user_id);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.channel_access_token)
            .send()
            .await?;
        if !resp.status().is_success() {
            let body: Value = resp.json().await.unwrap_or_default();
            return Err(LineApiError::Api(
                body["message"]
                    .as_str()
                    .unwrap_or("profile_error")
                    .to_string(),
            ));
        }
        resp.json().await.map_err(|_| LineApiError::BadResponse)
    }

    // ── Rich menu ───────────────────────────────────────────────────────

    /// Set the default rich menu for all users.
    pub async fn set_default_rich_menu(&self, rich_menu_id: &str) -> Result<()> {
        let url = format!(
            "{}/v2/bot/user/all/richmenu/{}",
            self.base_url, rich_menu_id
        );
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.channel_access_token)
            .send()
            .await?;
        if !resp.status().is_success() {
            let body: Value = resp.json().await.unwrap_or_default();
            return Err(LineApiError::Api(
                body["message"]
                    .as_str()
                    .unwrap_or("richmenu_error")
                    .to_string(),
            ));
        }
        Ok(())
    }

    // ── internal ────────────────────────────────────────────────────────

    async fn post(&self, path: &str, body: &Value) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.channel_access_token)
            .json(body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err_body: Value = resp.json().await.unwrap_or_default();
            return Err(LineApiError::Api(
                err_body["message"]
                    .as_str()
                    .unwrap_or("unknown_error")
                    .to_string(),
            ));
        }

        // LINE reply/push return empty body on success
        let text = resp.text().await.unwrap_or_default();
        if text.is_empty() {
            Ok(serde_json::json!({}))
        } else {
            serde_json::from_str(&text).map_err(|_| LineApiError::BadResponse)
        }
    }
}
