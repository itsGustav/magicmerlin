//! Slack Web API HTTP client.
//!
//! Wraps reqwest to call `https://slack.com/api/{method}` with bot token auth.

use serde_json::Value;

/// Errors from Slack API calls.
#[derive(Debug, thiserror::Error)]
pub enum SlackApiError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Slack API error: {0}")]
    Api(String),
    #[error("unexpected response shape")]
    BadResponse,
}

type Result<T> = std::result::Result<T, SlackApiError>;

/// HTTP client for the Slack Web API.
#[derive(Debug, Clone)]
pub struct SlackApiClient {
    bot_token: String,
    http: reqwest::Client,
    base_url: String,
}

impl SlackApiClient {
    pub fn new(bot_token: String) -> Self {
        Self {
            bot_token,
            http: reqwest::Client::new(),
            base_url: "https://slack.com/api".to_string(),
        }
    }

    /// Override base URL (useful for testing).
    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    // ── generic caller ──────────────────────────────────────────────────

    async fn call(&self, method: &str, params: Value) -> Result<Value> {
        let url = format!("{}/{}", self.base_url, method);
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.bot_token)
            .json(&params)
            .send()
            .await?;

        let body: Value = resp.json().await?;
        if body.get("ok").and_then(Value::as_bool) == Some(true) {
            Ok(body)
        } else {
            let error = body["error"]
                .as_str()
                .unwrap_or("unknown_error")
                .to_string();
            Err(SlackApiError::Api(error))
        }
    }

    // ── auth ────────────────────────────────────────────────────────────

    /// Calls `auth.test` to resolve the bot user ID.
    pub async fn auth_test(&self) -> Result<String> {
        let resp = self.call("auth.test", serde_json::json!({})).await?;
        resp["user_id"]
            .as_str()
            .map(ToString::to_string)
            .ok_or(SlackApiError::BadResponse)
    }

    // ── chat ────────────────────────────────────────────────────────────

    /// Post a message. Returns the message `ts`.
    pub async fn chat_post_message(
        &self,
        channel: &str,
        text: &str,
        blocks: Option<Value>,
    ) -> Result<String> {
        let mut params = serde_json::json!({
            "channel": channel,
            "text": text,
        });
        if let Some(b) = blocks {
            params["blocks"] = b;
        }
        let resp = self.call("chat.postMessage", params).await?;
        resp["ts"]
            .as_str()
            .map(ToString::to_string)
            .ok_or(SlackApiError::BadResponse)
    }

    /// Post a threaded reply. Returns the message `ts`.
    pub async fn chat_post_message_threaded(
        &self,
        channel: &str,
        text: &str,
        blocks: Option<Value>,
        thread_ts: &str,
    ) -> Result<String> {
        let mut params = serde_json::json!({
            "channel": channel,
            "text": text,
            "thread_ts": thread_ts,
        });
        if let Some(b) = blocks {
            params["blocks"] = b;
        }
        let resp = self.call("chat.postMessage", params).await?;
        resp["ts"]
            .as_str()
            .map(ToString::to_string)
            .ok_or(SlackApiError::BadResponse)
    }

    /// Update an existing message.
    pub async fn chat_update(&self, channel: &str, ts: &str, text: &str) -> Result<()> {
        self.call(
            "chat.update",
            serde_json::json!({
                "channel": channel,
                "ts": ts,
                "text": text,
            }),
        )
        .await?;
        Ok(())
    }

    /// Delete a message.
    pub async fn chat_delete(&self, channel: &str, ts: &str) -> Result<()> {
        self.call(
            "chat.delete",
            serde_json::json!({
                "channel": channel,
                "ts": ts,
            }),
        )
        .await?;
        Ok(())
    }

    // ── reactions ───────────────────────────────────────────────────────

    pub async fn reactions_add(&self, channel: &str, ts: &str, name: &str) -> Result<()> {
        self.call(
            "reactions.add",
            serde_json::json!({
                "channel": channel,
                "timestamp": ts,
                "name": name,
            }),
        )
        .await?;
        Ok(())
    }

    pub async fn reactions_remove(&self, channel: &str, ts: &str, name: &str) -> Result<()> {
        self.call(
            "reactions.remove",
            serde_json::json!({
                "channel": channel,
                "timestamp": ts,
                "name": name,
            }),
        )
        .await?;
        Ok(())
    }

    // ── conversations ───────────────────────────────────────────────────

    pub async fn conversations_list(&self) -> Result<Vec<super::SlackConversation>> {
        let resp = self
            .call(
                "conversations.list",
                serde_json::json!({"types": "public_channel,private_channel,im,mpim"}),
            )
            .await?;
        let channels = resp["channels"].as_array().cloned().unwrap_or_default();
        let mut out = Vec::with_capacity(channels.len());
        for ch in channels {
            if let Ok(c) = serde_json::from_value(ch) {
                out.push(c);
            }
        }
        Ok(out)
    }

    // ── users ───────────────────────────────────────────────────────────

    pub async fn users_info(&self, user_id: &str) -> Result<super::SlackUser> {
        let resp = self
            .call("users.info", serde_json::json!({"user": user_id}))
            .await?;
        serde_json::from_value(resp["user"].clone()).map_err(|_| SlackApiError::BadResponse)
    }

    // ── files ───────────────────────────────────────────────────────────

    /// Upload file content. Returns the file ID.
    pub async fn files_upload(
        &self,
        channel: &str,
        filename: &str,
        content: &[u8],
    ) -> Result<String> {
        let url = format!("{}/files.uploadV2", self.base_url);
        let part =
            reqwest::multipart::Part::bytes(content.to_vec()).file_name(filename.to_string());
        let form = reqwest::multipart::Form::new()
            .text("channel_id", channel.to_string())
            .text("filename", filename.to_string())
            .part("file", part);

        let resp: Value = self
            .http
            .post(&url)
            .bearer_auth(&self.bot_token)
            .multipart(form)
            .send()
            .await?
            .json()
            .await?;

        if resp.get("ok").and_then(Value::as_bool) == Some(true) {
            resp["file"]["id"]
                .as_str()
                .map(ToString::to_string)
                .ok_or(SlackApiError::BadResponse)
        } else {
            Err(SlackApiError::Api(
                resp["error"]
                    .as_str()
                    .unwrap_or("upload_failed")
                    .to_string(),
            ))
        }
    }
}
