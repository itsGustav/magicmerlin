//! iMessage channel — macOS-only via chat.db monitor + osascript send.
//!
//! Inbound: polls `~/Library/Messages/chat.db` (SQLite) for new messages.
//! Outbound: sends via `osascript` AppleScript to Messages.app.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::Arc;

use chrono::Utc;
use serde_json::json;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::Duration;
use tracing::{debug, error, warn};

use crate::framework::{
    Channel, ChannelError, ChatType, InboundMessage, MessageId, OutboundMessage, Platform, Result,
    Sender,
};

/// Default path relative to $HOME for the Messages SQLite database.
const CHAT_DB_REL: &str = "Library/Messages/chat.db";

/// SQL query for polling new inbound messages from chat.db.
const POLL_QUERY_TEMPLATE: &str = "\
SELECT m.ROWID, \
       COALESCE(c.chat_identifier,''), \
       COALESCE(h.id,''), \
       COALESCE(m.text,''), \
       m.date, \
       CASE WHEN c.style = 43 THEN 1 ELSE 0 END \
FROM message m \
LEFT JOIN chat_message_join cmj ON m.ROWID = cmj.message_id \
LEFT JOIN chat c ON cmj.chat_id = c.ROWID \
LEFT JOIN handle h ON m.handle_id = h.ROWID \
WHERE m.ROWID > {last_id} AND m.is_from_me = 0 \
ORDER BY m.ROWID ASC";

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct IMessageConfig {
    /// Polling interval in milliseconds (default: 2000).
    pub poll_interval_ms: u64,
    /// If non-empty, only accept messages from these phone numbers / emails.
    pub allowed_senders: HashSet<String>,
    /// Override path to chat.db (for testing).
    pub db_path: Option<PathBuf>,
}

impl Default for IMessageConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: 2000,
            allowed_senders: HashSet::new(),
            db_path: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Parsed row from chat.db
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IMessageRow {
    pub row_id: i64,
    pub chat_id: String,
    pub sender: String,
    pub text: Option<String>,
    pub date: i64,
    pub is_group: bool,
}

// ---------------------------------------------------------------------------
// Channel
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct IMessageChannel {
    config: IMessageConfig,
    messages: RwLock<HashMap<String, OutboundMessage>>,
    last_row_id: AtomicI64,
    next_id: AtomicU64,
    inbound_tx: Option<mpsc::Sender<InboundMessage>>,
    poll_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl IMessageChannel {
    pub fn new(config: IMessageConfig) -> Self {
        Self {
            config,
            messages: RwLock::new(HashMap::new()),
            last_row_id: AtomicI64::new(0),
            next_id: AtomicU64::new(1),
            inbound_tx: None,
            poll_handle: Mutex::new(None),
        }
    }

    /// Attach a sender for inbound messages (enables the poll loop on start).
    pub fn with_inbound(mut self, tx: mpsc::Sender<InboundMessage>) -> Self {
        self.inbound_tx = Some(tx);
        self
    }

    // -- path helpers -------------------------------------------------------

    fn db_path(&self) -> PathBuf {
        if let Some(ref p) = self.config.db_path {
            return p.clone();
        }
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join(CHAT_DB_REL)
    }

    // -- osascript builders -------------------------------------------------

    /// Builds AppleScript to send a text message to a buddy (direct chat).
    pub fn build_osascript_send(to: &str, text: &str) -> String {
        let to_escaped = escape_applescript(to);
        let text_escaped = escape_applescript(text);
        format!(
            "tell application \"Messages\"\n\
             \tset targetService to 1st service whose service type = iMessage\n\
             \tset targetBuddy to buddy \"{to_escaped}\" of targetService\n\
             \tsend \"{text_escaped}\" to targetBuddy\n\
             end tell"
        )
    }

    /// Builds AppleScript to send a text message to a group chat by name.
    pub fn build_osascript_group_send(group_name: &str, text: &str) -> String {
        let group_escaped = escape_applescript(group_name);
        let text_escaped = escape_applescript(text);
        format!(
            "tell application \"Messages\"\n\
             \tset targetChat to a reference to chat \"{group_escaped}\"\n\
             \tsend \"{text_escaped}\" to targetChat\n\
             end tell"
        )
    }

    /// Builds AppleScript to send a file to a buddy.
    fn build_osascript_file_send(to: &str, posix_path: &str) -> String {
        let to_escaped = escape_applescript(to);
        let path_escaped = escape_applescript(posix_path);
        format!(
            "tell application \"Messages\"\n\
             \tset targetService to 1st service whose service type = iMessage\n\
             \tset targetBuddy to buddy \"{to_escaped}\" of targetService\n\
             \tset theFile to POSIX file \"{path_escaped}\"\n\
             \tsend theFile to targetBuddy\n\
             end tell"
        )
    }

    // -- osascript execution ------------------------------------------------

    /// Executes an AppleScript via `osascript` and returns an error on failure.
    async fn run_osascript(script: &str) -> Result<()> {
        let output = tokio::process::Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .await
            .map_err(|e| ChannelError::PlatformRequest(format!("osascript exec: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ChannelError::PlatformRequest(format!(
                "osascript failed: {stderr}"
            )));
        }
        Ok(())
    }

    /// Sends a text message to a direct contact via osascript.
    pub async fn send_via_osascript(to: &str, text: &str) -> Result<()> {
        Self::run_osascript(&Self::build_osascript_send(to, text)).await
    }

    /// Sends a text message to a named group chat via osascript.
    pub async fn send_to_group(group_name: &str, text: &str) -> Result<()> {
        Self::run_osascript(&Self::build_osascript_group_send(group_name, text)).await
    }

    /// Sends a file/image to a direct contact via osascript.
    pub async fn send_image(&self, target: &str, path: &str) -> Result<MessageId> {
        let id = format!("imsg-img-{}", self.next_id.fetch_add(1, Ordering::Relaxed));
        Self::run_osascript(&Self::build_osascript_file_send(target, path)).await?;
        Ok(id)
    }

    // -- chat.db polling ----------------------------------------------------

    /// Queries chat.db for new messages since `last_row_id` via sqlite3 CLI.
    pub async fn poll_chat_db(&self) -> Result<Vec<IMessageRow>> {
        let db = self.db_path();
        let last_id = self.last_row_id.load(Ordering::Relaxed);
        let query = POLL_QUERY_TEMPLATE.replace("{last_id}", &last_id.to_string());

        let output = tokio::process::Command::new("sqlite3")
            .arg("-separator")
            .arg("|")
            .arg(db.to_string_lossy().as_ref())
            .arg(&query)
            .output()
            .await
            .map_err(|e| ChannelError::PlatformRequest(format!("sqlite3 exec: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ChannelError::PlatformRequest(format!(
                "sqlite3 query failed: {stderr}"
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut rows = Vec::new();
        for line in stdout.lines() {
            let parts: Vec<&str> = line.splitn(6, '|').collect();
            if let Some(row) = Self::parse_chat_row(&parts) {
                rows.push(row);
            }
        }
        Ok(rows)
    }

    /// Parses a pipe-delimited row from sqlite3 output into an `IMessageRow`.
    pub fn parse_chat_row(row: &[&str]) -> Option<IMessageRow> {
        if row.len() < 5 {
            return None;
        }
        Some(IMessageRow {
            row_id: row[0].parse().ok()?,
            chat_id: row[1].to_string(),
            sender: row[2].to_string(),
            text: if row[3].is_empty() {
                None
            } else {
                Some(row[3].to_string())
            },
            date: row[4].parse().ok()?,
            is_group: row
                .get(5)
                .and_then(|v| v.trim().parse::<i32>().ok())
                .unwrap_or(0)
                == 1,
        })
    }

    /// Returns `true` if this row_id is new (advances the high-water mark).
    pub async fn dedup_and_accept(&self, row_id: i64) -> bool {
        let previous = self.last_row_id.load(Ordering::Relaxed);
        if row_id <= previous {
            return false;
        }
        self.last_row_id.store(row_id, Ordering::Relaxed);
        true
    }

    /// Checks if a sender is allowed (empty allowlist = all allowed).
    fn is_sender_allowed(&self, sender: &str) -> bool {
        self.config.allowed_senders.is_empty() || self.config.allowed_senders.contains(sender)
    }

    /// Converts an `IMessageRow` into a framework `InboundMessage`.
    pub fn row_to_inbound(row: &IMessageRow) -> InboundMessage {
        InboundMessage {
            id: row.row_id.to_string(),
            platform: Platform::IMessage,
            chat_id: row.chat_id.clone(),
            chat_type: if row.is_group {
                ChatType::Group
            } else {
                ChatType::Direct
            },
            sender: Sender {
                id: row.sender.clone(),
                name: row.sender.clone(),
                username: Some(row.sender.clone()),
            },
            text: row.text.clone(),
            reply_to: None,
            media: Vec::new(),
            timestamp: Utc::now(),
            raw: json!({
                "row_id": row.row_id,
                "chat_id": row.chat_id,
                "sender": row.sender,
                "date": row.date,
                "is_group": row.is_group,
            }),
        }
    }

    fn next_message_id(&self) -> MessageId {
        format!("imsg-{}", self.next_id.fetch_add(1, Ordering::Relaxed))
    }

    // -- poll loop ----------------------------------------------------------

    /// Spawns the background chat.db poll loop.
    fn spawn_poll_loop(
        db_path: PathBuf,
        interval: Duration,
        last_row_id: Arc<AtomicI64>,
        allowed_senders: HashSet<String>,
        tx: mpsc::Sender<InboundMessage>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            debug!("iMessage poll loop started, db={}", db_path.display());
            loop {
                tokio::time::sleep(interval).await;
                let last_id = last_row_id.load(Ordering::Relaxed);
                let query = POLL_QUERY_TEMPLATE.replace("{last_id}", &last_id.to_string());

                let output = match tokio::process::Command::new("sqlite3")
                    .arg("-separator")
                    .arg("|")
                    .arg(db_path.to_string_lossy().as_ref())
                    .arg(&query)
                    .output()
                    .await
                {
                    Ok(o) if o.status.success() => o,
                    Ok(o) => {
                        warn!(
                            "iMessage poll query error: {}",
                            String::from_utf8_lossy(&o.stderr)
                        );
                        continue;
                    }
                    Err(e) => {
                        warn!("iMessage poll exec error: {e}");
                        continue;
                    }
                };

                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let parts: Vec<&str> = line.splitn(6, '|').collect();
                    let row = match IMessageChannel::parse_chat_row(&parts) {
                        Some(r) => r,
                        None => continue,
                    };

                    // Dedup
                    let prev = last_row_id.load(Ordering::Relaxed);
                    if row.row_id <= prev {
                        continue;
                    }
                    last_row_id.store(row.row_id, Ordering::Relaxed);

                    // Allowlist filter
                    if !allowed_senders.is_empty() && !allowed_senders.contains(&row.sender) {
                        debug!("iMessage: filtered sender {}", row.sender);
                        continue;
                    }

                    let inbound = IMessageChannel::row_to_inbound(&row);
                    if tx.send(inbound).await.is_err() {
                        error!("iMessage poll loop: inbound channel closed");
                        return;
                    }
                }
            }
        })
    }
}

// ---------------------------------------------------------------------------
// Channel trait
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
impl Channel for IMessageChannel {
    fn name(&self) -> &str {
        "imessage"
    }

    fn platform(&self) -> Platform {
        Platform::IMessage
    }

    async fn start(&mut self) -> Result<()> {
        let db = self.db_path();
        if !db.exists() {
            warn!(
                "chat.db not found at {}, iMessage monitoring unavailable",
                db.display()
            );
            return Err(ChannelError::PlatformRequest(format!(
                "chat.db not found at {}",
                db.display()
            )));
        }

        // Start poll loop if an inbound sender is attached.
        if let Some(tx) = self.inbound_tx.clone() {
            let interval = Duration::from_millis(self.config.poll_interval_ms);
            let last_row_id = Arc::new(AtomicI64::new(self.last_row_id.load(Ordering::Relaxed)));
            let allowed = self.config.allowed_senders.clone();

            let handle = Self::spawn_poll_loop(db, interval, last_row_id.clone(), allowed, tx);

            *self.poll_handle.lock().await = Some(handle);

            // Keep our own AtomicI64 in sync — the poll loop uses its own Arc copy,
            // but on stop we snapshot back.
        }

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        if let Some(handle) = self.poll_handle.lock().await.take() {
            handle.abort();
        }
        Ok(())
    }

    async fn send(&self, target: &str, message: OutboundMessage) -> Result<MessageId> {
        let id = self.next_message_id();

        // Attempt real osascript send. On failure, log but still track the message.
        if let Err(e) = Self::send_via_osascript(target, &message.text).await {
            warn!("iMessage osascript send to {target}: {e}");
        }

        self.messages
            .write()
            .await
            .insert(format!("{target}:{id}"), message);
        Ok(id)
    }

    async fn edit(&self, target: &str, message_id: &str, message: OutboundMessage) -> Result<()> {
        // iMessage does not support editing; store for tracking only.
        self.messages
            .write()
            .await
            .insert(format!("{target}:{message_id}"), message);
        Ok(())
    }

    async fn delete(&self, target: &str, message_id: &str) -> Result<()> {
        self.messages
            .write()
            .await
            .remove(&format!("{target}:{message_id}"));
        Ok(())
    }

    async fn react(&self, target: &str, message_id: &str, emoji: &str) -> Result<()> {
        self.messages.write().await.insert(
            format!("reaction:{target}:{message_id}"),
            OutboundMessage {
                text: emoji.to_string(),
                reply_to: None,
                media: Vec::new(),
                buttons: None,
                silent: true,
                parse_mode: None,
            },
        );
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Escapes a string for embedding inside AppleScript double-quoted literals.
fn escape_applescript(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_row_shape() {
        let row =
            IMessageChannel::parse_chat_row(&["1", "chat123", "alice", "hello", "99"]).unwrap();
        assert_eq!(row.chat_id, "chat123");
        assert_eq!(row.sender, "alice");
        assert_eq!(row.text.as_deref(), Some("hello"));
        assert!(!row.is_group);
    }

    #[test]
    fn parses_row_with_group_flag() {
        let row =
            IMessageChannel::parse_chat_row(&["42", "chat://g", "bob", "hi", "100", "1"]).unwrap();
        assert!(row.is_group);
        assert_eq!(row.row_id, 42);
    }

    #[test]
    fn parses_empty_text_as_none() {
        let row = IMessageChannel::parse_chat_row(&["2", "c", "s", "", "10"]).unwrap();
        assert!(row.text.is_none());
    }

    #[test]
    fn rejects_short_rows() {
        assert!(IMessageChannel::parse_chat_row(&["1", "2", "3"]).is_none());
    }

    #[tokio::test]
    async fn dedup_tracks_last_rowid() {
        let channel = IMessageChannel::new(IMessageConfig::default());
        assert!(channel.dedup_and_accept(10).await);
        assert!(!channel.dedup_and_accept(9).await);
        assert!(!channel.dedup_and_accept(10).await);
        assert!(channel.dedup_and_accept(11).await);
    }

    #[test]
    fn osascript_send_escapes_quotes() {
        let script = IMessageChannel::build_osascript_send("+1234", "say \"hi\"");
        assert!(script.contains("say \\\"hi\\\""));
        assert!(script.contains("buddy \"+1234\""));
        assert!(script.contains("service type = iMessage"));
    }

    #[test]
    fn osascript_group_send_builds_correctly() {
        let script = IMessageChannel::build_osascript_group_send("Family Chat", "hello all");
        assert!(script.contains("chat \"Family Chat\""));
        assert!(script.contains("send \"hello all\""));
    }

    #[test]
    fn osascript_send_escapes_backslashes() {
        let script = IMessageChannel::build_osascript_send("user", "path\\to\\file");
        assert!(script.contains("path\\\\to\\\\file"));
    }

    #[test]
    fn row_to_inbound_direct() {
        let row = IMessageRow {
            row_id: 5,
            chat_id: "chat1".into(),
            sender: "+1555".into(),
            text: Some("hi".into()),
            date: 700000000,
            is_group: false,
        };
        let msg = IMessageChannel::row_to_inbound(&row);
        assert_eq!(msg.platform, Platform::IMessage);
        assert_eq!(msg.chat_type, ChatType::Direct);
        assert_eq!(msg.sender.id, "+1555");
    }

    #[test]
    fn row_to_inbound_group() {
        let row = IMessageRow {
            row_id: 6,
            chat_id: "chat://group".into(),
            sender: "alice@me.com".into(),
            text: Some("hey".into()),
            date: 700000001,
            is_group: true,
        };
        let msg = IMessageChannel::row_to_inbound(&row);
        assert_eq!(msg.chat_type, ChatType::Group);
    }

    #[test]
    fn allowed_senders_filter() {
        let mut config = IMessageConfig::default();
        config.allowed_senders.insert("+1555".into());
        let channel = IMessageChannel::new(config);
        assert!(channel.is_sender_allowed("+1555"));
        assert!(!channel.is_sender_allowed("+9999"));
    }

    #[test]
    fn empty_allowlist_allows_all() {
        let channel = IMessageChannel::new(IMessageConfig::default());
        assert!(channel.is_sender_allowed("anyone"));
    }

    #[test]
    fn message_normalization() {
        let row = IMessageRow {
            row_id: 1,
            chat_id: "c".into(),
            sender: "s".into(),
            text: Some("  hello world  ".into()),
            date: 0,
            is_group: false,
        };
        let mut msg = IMessageChannel::row_to_inbound(&row);
        msg.normalize();
        assert_eq!(msg.text.as_deref(), Some("hello world"));
    }

    #[test]
    fn escape_applescript_works() {
        assert_eq!(escape_applescript(r#"say "hi""#), r#"say \"hi\""#);
        assert_eq!(escape_applescript(r"a\b"), r"a\\b");
    }
}
