use serde_json::{Map, Value};

use crate::framework::{ChannelError, Result};

pub const TELEGRAM_DEFAULT_POLL_INTERVAL_MS: u64 = 500;
pub const TELEGRAM_DEFAULT_POLL_TIMEOUT_SECONDS: u64 = 30;
pub const TELEGRAM_DEFAULT_RETRY_LIMIT: usize = 6;
pub const TELEGRAM_DEFAULT_GLOBAL_RATE_LIMIT_PER_SECOND: usize = 30;
pub const TELEGRAM_DEFAULT_CHAT_RATE_LIMIT: usize = 20;
pub const TELEGRAM_DEFAULT_CHAT_RATE_WINDOW_SECONDS: u64 = 60;

/// Telegram bot account configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelegramAccount {
    /// Stable account name used in config and runtime lookups.
    pub name: String,
    /// Bot token.
    pub token: String,
    /// Public bot username used to route inbound events.
    pub bot_username: String,
    /// Enables long polling for this account when channel polling is enabled.
    pub polling_enabled: bool,
    /// Optional media directory override.
    pub media_dir: Option<String>,
    /// Optional webhook secret token.
    pub webhook_secret: Option<String>,
}

impl TelegramAccount {
    /// Returns normalized username with a leading `@`.
    pub fn normalized_bot_username(&self) -> String {
        normalize_bot_username(&self.bot_username)
    }
}

/// Telegram runtime configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelegramConfig {
    /// All configured Telegram bot accounts.
    pub accounts: Vec<TelegramAccount>,
    /// Enables long polling globally.
    pub polling_mode: bool,
    /// Optional shared webhook URL.
    pub webhook_url: Option<String>,
    /// Long-poll request timeout.
    pub poll_timeout_seconds: u64,
    /// Delay between poll loops.
    pub poll_interval_ms: u64,
    /// Maximum updates requested per polling round.
    pub max_updates_per_poll: usize,
    /// When true, start polling if webhook delivery is not active.
    pub webhook_fallback_to_polling: bool,
    /// When true, send chat actions before message dispatch.
    pub auto_send_chat_actions: bool,
    /// Default local media directory.
    pub default_media_dir: String,
    /// Retry budget for retryable send and poll failures.
    pub retry_limit: usize,
    /// Timeout budget used for simulated network retries.
    pub network_timeout_seconds: u64,
    /// Global per-account send budget.
    pub global_rate_limit_per_second: usize,
    /// Per-chat send budget.
    pub per_chat_rate_limit: usize,
    /// Sliding window for the per-chat send budget.
    pub per_chat_rate_window_seconds: u64,
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self {
            accounts: Vec::new(),
            polling_mode: true,
            webhook_url: None,
            poll_timeout_seconds: TELEGRAM_DEFAULT_POLL_TIMEOUT_SECONDS,
            poll_interval_ms: TELEGRAM_DEFAULT_POLL_INTERVAL_MS,
            max_updates_per_poll: 100,
            webhook_fallback_to_polling: true,
            auto_send_chat_actions: true,
            default_media_dir: "/tmp/magicmerlin-telegram".to_string(),
            retry_limit: TELEGRAM_DEFAULT_RETRY_LIMIT,
            network_timeout_seconds: 10,
            global_rate_limit_per_second: TELEGRAM_DEFAULT_GLOBAL_RATE_LIMIT_PER_SECOND,
            per_chat_rate_limit: TELEGRAM_DEFAULT_CHAT_RATE_LIMIT,
            per_chat_rate_window_seconds: TELEGRAM_DEFAULT_CHAT_RATE_WINDOW_SECONDS,
        }
    }
}

impl TelegramConfig {
    /// Loads Telegram settings from a JSON object shaped like `channels.telegram`.
    pub fn from_channels_json(root: &Value) -> Result<Self> {
        let telegram = resolve_telegram_root(root)
            .ok_or_else(|| ChannelError::PlatformRequest("missing channels.telegram config".to_string()))?;
        Self::from_telegram_map(telegram)
    }

    /// Loads Telegram settings from a `channels.telegram` JSON object.
    pub fn from_telegram_map(map: &Map<String, Value>) -> Result<Self> {
        let mut config = Self::default();

        if let Some(value) = get_bool(map, "pollingMode") {
            config.polling_mode = value;
        }
        if let Some(value) = get_string(map, "webhookUrl") {
            config.webhook_url = Some(value);
        }
        if let Some(value) = get_u64(map, "pollTimeoutSeconds") {
            config.poll_timeout_seconds = value.max(1);
        }
        if let Some(value) = get_u64(map, "pollIntervalMs") {
            config.poll_interval_ms = value.max(1);
        }
        if let Some(value) = get_u64(map, "maxUpdatesPerPoll") {
            config.max_updates_per_poll = value.max(1) as usize;
        }
        if let Some(value) = get_bool(map, "webhookFallbackToPolling") {
            config.webhook_fallback_to_polling = value;
        }
        if let Some(value) = get_bool(map, "autoSendChatActions") {
            config.auto_send_chat_actions = value;
        }
        if let Some(value) = get_string(map, "defaultMediaDir") {
            config.default_media_dir = value;
        }
        if let Some(value) = get_u64(map, "retryLimit") {
            config.retry_limit = value.max(1) as usize;
        }
        if let Some(value) = get_u64(map, "networkTimeoutSeconds") {
            config.network_timeout_seconds = value.max(1);
        }
        if let Some(value) = get_u64(map, "globalRateLimitPerSecond") {
            config.global_rate_limit_per_second = value.max(1) as usize;
        }
        if let Some(value) = get_u64(map, "perChatRateLimit") {
            config.per_chat_rate_limit = value.max(1) as usize;
        }
        if let Some(value) = get_u64(map, "perChatRateWindowSeconds") {
            config.per_chat_rate_window_seconds = value.max(1);
        }

        if let Some(accounts) = map.get("accounts") {
            config.accounts = parse_accounts(accounts)?;
        }

        Ok(config)
    }

    /// Resolves the effective media directory for an account.
    pub fn media_dir_for(&self, account: &TelegramAccount) -> String {
        account
            .media_dir
            .clone()
            .unwrap_or_else(|| self.default_media_dir.clone())
    }
}

fn resolve_telegram_root(root: &Value) -> Option<&Map<String, Value>> {
    root.as_object()
        .and_then(|map| map.get("channels").or_else(|| map.get("telegram")).unwrap_or(root).as_object())
        .and_then(|map| {
            if map.get("accounts").is_some() {
                Some(map)
            } else {
                map.get("telegram")?.as_object()
            }
        })
}

fn parse_accounts(value: &Value) -> Result<Vec<TelegramAccount>> {
    match value {
        Value::Object(entries) => entries
            .iter()
            .map(|(name, value)| parse_account(name, value))
            .collect(),
        Value::Array(entries) => entries
            .iter()
            .enumerate()
            .map(|(index, value)| {
                let fallback_name = format!("account-{}", index + 1);
                let name = value
                    .as_object()
                    .and_then(|map| get_string(map, "name"))
                    .unwrap_or(fallback_name);
                parse_account(&name, value)
            })
            .collect(),
        _ => Err(ChannelError::PlatformRequest(
            "channels.telegram.accounts must be an object or array".to_string(),
        )),
    }
}

fn parse_account(name: &str, value: &Value) -> Result<TelegramAccount> {
    let map = value.as_object().ok_or_else(|| {
        ChannelError::PlatformRequest(format!("telegram account `{name}` must be a JSON object"))
    })?;
    let token = get_string(map, "token").ok_or_else(|| {
        ChannelError::PlatformRequest(format!("telegram account `{name}` is missing token"))
    })?;
    let bot_username = get_string(map, "botUsername")
        .or_else(|| get_string(map, "bot_username"))
        .unwrap_or_else(|| name.to_string());

    Ok(TelegramAccount {
        name: name.to_string(),
        token,
        bot_username,
        polling_enabled: get_bool(map, "pollingEnabled").unwrap_or(true),
        media_dir: get_string(map, "mediaDir"),
        webhook_secret: get_string(map, "webhookSecret"),
    })
}

fn get_string(map: &Map<String, Value>, key: &str) -> Option<String> {
    map.get(key)?.as_str().map(ToOwned::to_owned)
}

fn get_u64(map: &Map<String, Value>, key: &str) -> Option<u64> {
    map.get(key)?.as_u64()
}

fn get_bool(map: &Map<String, Value>, key: &str) -> Option<bool> {
    map.get(key)?.as_bool()
}

pub(crate) fn normalize_bot_username(value: &str) -> String {
    if value.starts_with('@') {
        value.to_string()
    } else {
        format!("@{value}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_channels_telegram_accounts_object() {
        let root = json!({
            "channels": {
                "telegram": {
                    "pollIntervalMs": 750,
                    "accounts": {
                        "primary": {
                            "token": "token-1",
                            "botUsername": "bot_one"
                        },
                        "secondary": {
                            "token": "token-2",
                            "botUsername": "@bot_two",
                            "mediaDir": "/tmp/two"
                        }
                    }
                }
            }
        });

        let config = TelegramConfig::from_channels_json(&root).unwrap();
        assert_eq!(config.poll_interval_ms, 750);
        assert_eq!(config.accounts.len(), 2);
        assert_eq!(config.accounts[0].normalized_bot_username(), "@bot_one");
        assert_eq!(config.media_dir_for(&config.accounts[1]), "/tmp/two");
    }

    #[test]
    fn parses_channels_telegram_accounts_array() {
        let root = json!({
            "telegram": {
                "accounts": [
                    {
                        "name": "first",
                        "token": "token-a",
                        "botUsername": "bot_a"
                    },
                    {
                        "name": "second",
                        "token": "token-b",
                        "bot_username": "bot_b",
                        "pollingEnabled": false
                    }
                ]
            }
        });

        let config = TelegramConfig::from_channels_json(&root).unwrap();
        assert_eq!(config.accounts[0].name, "first");
        assert!(!config.accounts[1].polling_enabled);
    }
}
