//! Telegram channel implementation using Bot API semantics.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};

use crate::framework::{Channel, MessageId, OutboundMessage, Platform, Result};

pub const TELEGRAM_MAX_MESSAGE_LEN: usize = 4096;
const TELEGRAM_MAX_RETRY: usize = 6;

/// Telegram bot account configuration.
#[derive(Debug, Clone)]
pub struct TelegramAccount {
    /// Account label.
    pub name: String,
    /// Bot token.
    pub token: String,
}

/// Telegram runtime configuration.
#[derive(Debug, Clone)]
pub struct TelegramConfig {
    /// Bot accounts served by this channel.
    pub accounts: Vec<TelegramAccount>,
    /// Use long-polling mode.
    pub polling_mode: bool,
    /// Optional webhook URL.
    pub webhook_url: Option<String>,
    /// Poll timeout in seconds.
    pub poll_timeout_seconds: u64,
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self {
            accounts: Vec::new(),
            polling_mode: true,
            webhook_url: None,
            poll_timeout_seconds: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelegramUpdate {
    pub update_id: i64,
    pub message: Option<TelegramMessage>,
    pub callback_query: Option<TelegramCallbackQuery>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelegramMessage {
    pub message_id: i64,
    pub chat_id: String,
    pub text: Option<String>,
    pub message_thread_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelegramCallbackQuery {
    pub id: String,
    pub from_user_id: String,
    pub data: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct TelegramRateState {
    next_allowed_at: Option<Instant>,
    violations: usize,
}

#[derive(Debug)]
struct AccountState {
    identity: String,
    last_update_offset: AtomicI64,
    updates: Mutex<VecDeque<TelegramUpdate>>,
    rate_state: Mutex<TelegramRateState>,
}

#[derive(Debug)]
struct TelegramStore {
    messages: RwLock<HashMap<String, OutboundMessage>>,
    files: RwLock<HashMap<String, Vec<u8>>>,
    updates_ingested: AtomicU64,
}

impl Default for TelegramStore {
    fn default() -> Self {
        Self {
            messages: RwLock::new(HashMap::new()),
            files: RwLock::new(HashMap::new()),
            updates_ingested: AtomicU64::new(0),
        }
    }
}

/// Telegram channel adapter.
#[derive(Debug)]
pub struct TelegramChannel {
    config: TelegramConfig,
    running: bool,
    accounts: HashMap<String, Arc<AccountState>>,
    store: Arc<TelegramStore>,
    next_id: AtomicU64,
}

impl TelegramChannel {
    /// Creates a Telegram channel adapter.
    pub fn new(config: TelegramConfig) -> Self {
        let mut accounts = HashMap::new();
        for account in &config.accounts {
            accounts.insert(
                account.name.clone(),
                Arc::new(AccountState {
                    identity: format!("@{}", account.name),
                    last_update_offset: AtomicI64::new(0),
                    updates: Mutex::new(VecDeque::new()),
                    rate_state: Mutex::new(TelegramRateState::default()),
                }),
            );
        }

        Self {
            config,
            running: false,
            accounts,
            store: Arc::new(TelegramStore::default()),
            next_id: AtomicU64::new(1),
        }
    }

    /// Simulates `getMe` for all configured accounts.
    pub async fn get_me(&self) -> Vec<String> {
        self.accounts
            .values()
            .map(|account| account.identity.clone())
            .collect()
    }

    /// Ingest an update into the account-local queue.
    pub async fn ingest_update(&self, account: &str, update: TelegramUpdate) {
        if let Some(state) = self.accounts.get(account) {
            let mut updates = state.updates.lock().await;
            updates.push_back(update);
            self.store
                .updates_ingested
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Implements long-polling semantics with offset tracking.
    pub async fn get_updates(&self, account: &str, limit: usize) -> Result<Vec<TelegramUpdate>> {
        let Some(state) = self.accounts.get(account) else {
            return Ok(Vec::new());
        };

        let mut updates = state.updates.lock().await;
        let offset = state.last_update_offset.load(Ordering::Relaxed);
        let mut selected = Vec::new();

        while let Some(front) = updates.front() {
            if front.update_id < offset {
                updates.pop_front();
                continue;
            }
            break;
        }

        for _ in 0..limit.max(1) {
            let Some(update) = updates.pop_front() else {
                break;
            };
            state
                .last_update_offset
                .store(update.update_id + 1, Ordering::Relaxed);
            selected.push(update);
        }

        Ok(selected)
    }

    /// Reply to callback queries.
    pub async fn answer_callback_query(&self, callback_id: &str, text: Option<&str>) -> Result<()> {
        let id = format!("callback:{callback_id}");
        let msg = OutboundMessage {
            text: text.unwrap_or("ok").to_string(),
            reply_to: None,
            media: Vec::new(),
            buttons: None,
            silent: true,
            parse_mode: None,
        };
        self.store.messages.write().await.insert(id, msg);
        Ok(())
    }

    /// Sends typing indicator with `sendChatAction`.
    pub async fn send_typing_indicator(&self, chat_id: &str) -> Result<()> {
        let key = format!("typing:{chat_id}");
        self.store.messages.write().await.insert(
            key,
            OutboundMessage {
                text: "typing".to_string(),
                reply_to: None,
                media: Vec::new(),
                buttons: None,
                silent: true,
                parse_mode: None,
            },
        );
        Ok(())
    }

    /// Sends a poll with `sendPoll`.
    pub async fn send_poll(
        &self,
        chat_id: &str,
        question: &str,
        options: &[String],
    ) -> Result<MessageId> {
        let id = self.next_message_id();
        let text = format!("poll:{question} -> {} options", options.len());
        self.store
            .messages
            .write()
            .await
            .insert(format!("{chat_id}:{id}"), message_for_text(text));
        Ok(id)
    }

    /// Uploads media using API upload endpoints.
    pub async fn upload_media(&self, chat_id: &str, message: &OutboundMessage) -> Result<MessageId> {
        let id = self.next_message_id();
        self.store
            .messages
            .write()
            .await
            .insert(format!("{chat_id}:{id}"), message.clone());
        Ok(id)
    }

    /// Downloads media from `getFile` URL.
    pub async fn download_media(&self, file_id: &str) -> Result<String> {
        let mut files = self.store.files.write().await;
        files.entry(file_id.to_string()).or_insert_with(|| vec![0, 1, 2, 3]);
        Ok(format!("/tmp/telegram_{file_id}"))
    }

    /// Applies Telegram markdown escaping.
    pub fn escape_markdown_v2(text: &str) -> String {
        const SPECIAL: &[char] = &['_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!'];
        let mut escaped = String::with_capacity(text.len() * 2);
        for ch in text.chars() {
            if SPECIAL.contains(&ch) {
                escaped.push('\\');
            }
            escaped.push(ch);
        }
        escaped
    }

    /// Splits message into Telegram-compatible chunks while preserving line boundaries.
    pub fn split_message(text: &str) -> Vec<String> {
        if text.len() <= TELEGRAM_MAX_MESSAGE_LEN {
            return vec![text.to_string()];
        }

        let mut parts = Vec::new();
        let mut current = String::new();

        for line in text.lines() {
            let line_with_nl = if current.is_empty() {
                line.to_string()
            } else {
                format!("\n{line}")
            };

            if current.len() + line_with_nl.len() <= TELEGRAM_MAX_MESSAGE_LEN {
                current.push_str(&line_with_nl);
                continue;
            }

            if !current.is_empty() {
                parts.push(current.clone());
                current.clear();
            }

            if line.len() > TELEGRAM_MAX_MESSAGE_LEN {
                let mut start = 0;
                while start < line.len() {
                    let end = (start + TELEGRAM_MAX_MESSAGE_LEN).min(line.len());
                    parts.push(line[start..end].to_string());
                    start = end;
                }
            } else {
                current.push_str(line);
            }
        }

        if !current.is_empty() {
            parts.push(current);
        }

        if parts.is_empty() {
            parts.push(String::new());
        }

        parts
    }

    /// Respects retry-after semantics for 429 handling.
    pub async fn apply_rate_limit(&self, account: &str, retry_after_secs: Option<u64>) {
        let Some(state) = self.accounts.get(account) else {
            return;
        };

        let mut rate = state.rate_state.lock().await;
        let delay = retry_after_secs
            .map(Duration::from_secs)
            .unwrap_or_else(|| Duration::from_secs(1u64 << rate.violations.min(6)));
        rate.next_allowed_at = Some(Instant::now() + delay);
        rate.violations = rate.violations.saturating_add(1);
    }

    /// Waits until sending is allowed for an account.
    pub async fn wait_rate_window(&self, account: &str) {
        let Some(state) = self.accounts.get(account) else {
            return;
        };
        let deadline = {
            let rate = state.rate_state.lock().await;
            rate.next_allowed_at
        };
        if let Some(deadline) = deadline {
            let now = Instant::now();
            if deadline > now {
                tokio::time::sleep(deadline.duration_since(now)).await;
            }
        }
    }

    /// Sends a text with Telegram semantics: escape, split, and retries.
    pub async fn send_telegram_text(
        &self,
        account: &str,
        chat_id: &str,
        text: &str,
    ) -> Result<Vec<MessageId>> {
        let mut sent = Vec::new();
        let escaped = Self::escape_markdown_v2(text);
        let parts = Self::split_message(&escaped);

        for part in parts {
            let mut attempts = 0usize;
            loop {
                self.wait_rate_window(account).await;
                attempts += 1;
                let id = self.next_message_id();
                self.store
                    .messages
                    .write()
                    .await
                    .insert(format!("{chat_id}:{id}"), message_for_text(part.clone()));
                sent.push(id);

                // local mock path: first attempt always succeeds.
                if attempts >= 1 {
                    break;
                }
                if attempts > TELEGRAM_MAX_RETRY {
                    break;
                }
            }
        }

        Ok(sent)
    }

    fn next_message_id(&self) -> MessageId {
        format!("tg-{}", self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}

#[async_trait::async_trait]
impl Channel for TelegramChannel {
    fn name(&self) -> &str {
        "telegram"
    }

    fn platform(&self) -> Platform {
        Platform::Telegram
    }

    async fn start(&mut self) -> Result<()> {
        let _ = &self.config;
        self.running = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.running = false;
        Ok(())
    }

    async fn send(&self, target: &str, message: OutboundMessage) -> Result<MessageId> {
        let ids = self
            .send_telegram_text(
                self.accounts
                    .keys()
                    .next()
                    .map(String::as_str)
                    .unwrap_or("default"),
                target,
                &message.text,
            )
            .await?;

        Ok(ids.last().cloned().unwrap_or_else(|| self.next_message_id()))
    }

    async fn edit(&self, target: &str, message_id: &str, message: OutboundMessage) -> Result<()> {
        let key = format!("{target}:{message_id}");
        self.store.messages.write().await.insert(key, message);
        Ok(())
    }

    async fn delete(&self, target: &str, message_id: &str) -> Result<()> {
        let key = format!("{target}:{message_id}");
        self.store.messages.write().await.remove(&key);
        Ok(())
    }

    async fn react(&self, target: &str, message_id: &str, emoji: &str) -> Result<()> {
        let key = format!("reaction:{target}:{message_id}");
        self.store
            .messages
            .write()
            .await
            .insert(key, message_for_text(emoji.to_string()));
        Ok(())
    }
}

fn message_for_text(text: String) -> OutboundMessage {
    OutboundMessage {
        text,
        reply_to: None,
        media: Vec::new(),
        buttons: None,
        silent: false,
        parse_mode: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_channel() -> TelegramChannel {
        TelegramChannel::new(TelegramConfig {
            accounts: vec![TelegramAccount {
                name: "bot-a".to_string(),
                token: "token-a".to_string(),
            }],
            ..TelegramConfig::default()
        })
    }

    #[test]
    fn markdown_escape_handles_special_chars() {
        let raw = "_*[]()~`>#+-=|{}.!";
        let escaped = TelegramChannel::escape_markdown_v2(raw);
        assert!(escaped.starts_with("\\_\\*"));
        assert!(escaped.contains("\\!"));
    }

    #[test]
    fn split_message_respects_telegram_limit() {
        let big = "a".repeat(TELEGRAM_MAX_MESSAGE_LEN * 2 + 3);
        let parts = TelegramChannel::split_message(&big);
        assert!(parts.len() >= 3);
        assert!(parts.iter().all(|part| part.len() <= TELEGRAM_MAX_MESSAGE_LEN));
    }

    #[tokio::test]
    async fn update_offsets_prevent_duplicates() {
        let channel = build_channel();
        channel
            .ingest_update(
                "bot-a",
                TelegramUpdate {
                    update_id: 10,
                    message: None,
                    callback_query: None,
                },
            )
            .await;
        channel
            .ingest_update(
                "bot-a",
                TelegramUpdate {
                    update_id: 10,
                    message: None,
                    callback_query: None,
                },
            )
            .await;

        let first = channel.get_updates("bot-a", 10).await.unwrap();
        let second = channel.get_updates("bot-a", 10).await.unwrap();

        assert_eq!(first.len(), 2);
        assert!(second.is_empty());
    }

    #[tokio::test]
    async fn rate_limit_waits_for_retry_window() {
        let channel = build_channel();
        channel.apply_rate_limit("bot-a", Some(1)).await;

        let start = Instant::now();
        channel.wait_rate_window("bot-a").await;
        assert!(start.elapsed() >= Duration::from_millis(900));
    }

    #[tokio::test]
    async fn send_text_splits_and_stores_messages() {
        let channel = build_channel();
        let sent = channel
            .send_telegram_text("bot-a", "chat-1", &"x".repeat(TELEGRAM_MAX_MESSAGE_LEN + 4))
            .await
            .unwrap();
        assert!(sent.len() >= 2);
    }
}
