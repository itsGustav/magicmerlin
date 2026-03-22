//! Signal channel implementation — `signal-cli` subprocess wrapper with
//! presage native runtime extension point.
//!
//! # Runtime selection
//!
//! The channel uses `signal-cli` as its backend. To add native Signal
//! protocol support via presage, add the following to `Cargo.toml`:
//!
//! ```toml
//! [target.'cfg(not(target_os = "windows"))'.dependencies]
//! presage = { git = "https://github.com/whisperfish/presage", branch = "main", optional = true }
//! presage-store-sled = { git = "https://github.com/whisperfish/presage", branch = "main", optional = true }
//! ```

mod cli;
mod monitor;
mod types;

pub use cli::SignalCliWrapper;
pub use monitor::SignalMonitor;
pub use types::{DataMessage, GroupInfo, SignalAttachment, SignalEnvelope};

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tokio::sync::{mpsc, RwLock};

use crate::framework::{
    Channel, ChannelError, InboundMessage, MessageId, OutboundMessage, Platform, Result,
};

/// Signal channel configuration.
#[derive(Debug, Clone)]
pub struct SignalConfig {
    /// Registered Signal phone number (e.g. "+1234567890").
    pub phone_number: String,
    /// Data/config directory for signal-cli.
    pub data_dir: Option<PathBuf>,
    /// Path to signal-cli binary (defaults to "signal-cli").
    pub cli_path: String,
    /// Whether group messaging is enabled.
    pub group_support: bool,
    /// Whether media/attachment handling is enabled.
    pub media_support: bool,
    /// Polling interval for the receive loop in seconds.
    pub poll_interval_secs: u64,
}

impl SignalConfig {
    /// Creates a config with the given phone number and sensible defaults.
    pub fn new(phone_number: impl Into<String>) -> Self {
        Self {
            phone_number: phone_number.into(),
            data_dir: None,
            cli_path: "signal-cli".to_string(),
            group_support: true,
            media_support: true,
            poll_interval_secs: 2,
        }
    }
}

/// Signal protocol runtime backend.
pub enum SignalRuntime {
    /// Subprocess wrapper around the `signal-cli` binary.
    Cli(SignalCliWrapper),
    // Future: Presage(PresageRuntime) for native Rust Signal protocol.
    // Enable by adding presage deps and a `signal-presage` feature.
}

/// Signal channel implementing the unified [`Channel`] trait.
pub struct SignalChannel {
    config: SignalConfig,
    runtime: SignalRuntime,
    messages: RwLock<HashMap<String, OutboundMessage>>,
    next_id: AtomicU64,
}

impl std::fmt::Debug for SignalChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalChannel")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl SignalChannel {
    /// Creates a new Signal channel from the given configuration.
    ///
    /// Uses the `signal-cli` subprocess wrapper as the runtime backend.
    pub fn new(config: SignalConfig) -> Self {
        let cli = SignalCliWrapper::new(&config.cli_path, &config.phone_number);
        let cli = match &config.data_dir {
            Some(dir) => cli.with_data_dir(dir.clone()),
            None => cli,
        };

        Self {
            config,
            runtime: SignalRuntime::Cli(cli),
            messages: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }

    /// Creates a Signal channel with an explicit runtime.
    pub fn with_runtime(config: SignalConfig, runtime: SignalRuntime) -> Self {
        Self {
            config,
            runtime,
            messages: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }

    /// Runs the receive loop, polling for new messages and forwarding them
    /// through the provided channel sender.
    ///
    /// Returns `Ok(())` on clean shutdown (receiver dropped).
    /// Returns `Err` on unrecoverable receive error.
    pub async fn run_receive_loop(&self, tx: mpsc::Sender<InboundMessage>) -> Result<()> {
        let interval = Duration::from_secs(self.config.poll_interval_secs);

        loop {
            let messages = match &self.runtime {
                SignalRuntime::Cli(cli) => cli.receive_once().await?,
            };

            for msg in messages {
                if tx.send(msg).await.is_err() {
                    return Ok(()); // receiver dropped — clean shutdown
                }
            }

            tokio::time::sleep(interval).await;
        }
    }

    /// Sends a text message to a recipient (phone number) or group.
    ///
    /// Targets starting with `+` are treated as direct messages;
    /// all other targets are treated as group IDs.
    pub async fn send_message(&self, target: &str, text: &str) -> Result<String> {
        match &self.runtime {
            SignalRuntime::Cli(cli) => {
                if target.starts_with('+') {
                    cli.send(target, text).await?;
                } else {
                    cli.send_to_group(target, text).await?;
                }
            }
        }
        Ok(self.next_message_id())
    }

    /// Sends a text message with a file attachment.
    pub async fn send_with_attachment(
        &self,
        target: &str,
        text: &str,
        attachment: &Path,
    ) -> Result<String> {
        match &self.runtime {
            SignalRuntime::Cli(cli) => {
                cli.send_with_attachment(target, text, attachment).await?;
            }
        }
        Ok(self.next_message_id())
    }

    /// Returns whether the underlying runtime is available and ready.
    pub fn is_runtime_available(&self) -> bool {
        match &self.runtime {
            SignalRuntime::Cli(cli) => cli.is_available(),
        }
    }

    fn next_message_id(&self) -> MessageId {
        format!("signal-{}", self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}

#[async_trait::async_trait]
impl Channel for SignalChannel {
    fn name(&self) -> &str {
        "signal"
    }

    fn platform(&self) -> Platform {
        Platform::Signal
    }

    async fn start(&mut self) -> Result<()> {
        if !self.is_runtime_available() {
            return Err(ChannelError::PlatformRequest(
                "signal-cli binary not found; install signal-cli or provide an explicit path"
                    .to_string(),
            ));
        }
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    async fn send(&self, target: &str, message: OutboundMessage) -> Result<MessageId> {
        let id = if let Some(attachment) = message.media.first().and_then(|m| m.file_path.as_ref())
        {
            self.send_with_attachment(target, &message.text, Path::new(attachment))
                .await?
        } else {
            self.send_message(target, &message.text).await?
        };

        self.messages
            .write()
            .await
            .insert(format!("{target}:{id}"), message);
        Ok(id)
    }

    async fn edit(&self, target: &str, message_id: &str, message: OutboundMessage) -> Result<()> {
        // Signal doesn't support message editing via signal-cli; store locally.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framework::{Channel, ChatType, MediaType, Platform};

    fn test_config() -> SignalConfig {
        SignalConfig::new("+1555000000")
    }

    #[test]
    fn config_defaults() {
        let cfg = SignalConfig::new("+1234");
        assert_eq!(cfg.phone_number, "+1234");
        assert_eq!(cfg.cli_path, "signal-cli");
        assert!(cfg.group_support);
        assert!(cfg.media_support);
        assert_eq!(cfg.poll_interval_secs, 2);
    }

    #[test]
    fn envelope_direct_message_converts() {
        let env = SignalEnvelope {
            source: None,
            source_number: Some("+1555111222".to_string()),
            source_name: Some("Alice".to_string()),
            timestamp: Some(1700000000000),
            data_message: Some(DataMessage {
                timestamp: Some(1700000000000),
                message: Some("hello".to_string()),
                group_info: None,
                attachments: None,
            }),
        };

        let inbound = env.into_inbound().expect("should produce InboundMessage");
        assert_eq!(inbound.platform, Platform::Signal);
        assert_eq!(inbound.chat_type, ChatType::Direct);
        assert_eq!(inbound.chat_id, "+1555111222");
        assert_eq!(inbound.sender.id, "+1555111222");
        assert_eq!(inbound.sender.name, "Alice");
        assert_eq!(inbound.text.as_deref(), Some("hello"));
        assert!(inbound.media.is_empty());
    }

    #[test]
    fn envelope_group_message_converts() {
        let env = SignalEnvelope {
            source: None,
            source_number: Some("+1555111222".to_string()),
            source_name: Some("Bob".to_string()),
            timestamp: Some(1700000000000),
            data_message: Some(DataMessage {
                timestamp: Some(1700000000000),
                message: Some("hey group".to_string()),
                group_info: Some(GroupInfo {
                    group_id: "abc123group".to_string(),
                    group_type: Some("DELIVER".to_string()),
                }),
                attachments: None,
            }),
        };

        let inbound = env.into_inbound().expect("should produce InboundMessage");
        assert_eq!(inbound.chat_type, ChatType::Group);
        assert_eq!(inbound.chat_id, "abc123group");
        assert_eq!(inbound.text.as_deref(), Some("hey group"));
    }

    #[test]
    fn envelope_without_data_message_returns_none() {
        let env = SignalEnvelope {
            source: Some("+1555000000".to_string()),
            source_number: None,
            source_name: None,
            timestamp: Some(1700000000000),
            data_message: None,
        };

        assert!(env.into_inbound().is_none());
    }

    #[test]
    fn envelope_with_attachments_parses_media() {
        let env = SignalEnvelope {
            source: None,
            source_number: Some("+1555111222".to_string()),
            source_name: None,
            timestamp: Some(1700000000000),
            data_message: Some(DataMessage {
                timestamp: Some(1700000000000),
                message: Some("photo".to_string()),
                group_info: None,
                attachments: Some(vec![
                    SignalAttachment {
                        content_type: Some("image/jpeg".to_string()),
                        filename: Some("photo.jpg".to_string()),
                        id: Some("att-1".to_string()),
                        size: Some(4096),
                    },
                    SignalAttachment {
                        content_type: Some("application/pdf".to_string()),
                        filename: Some("doc.pdf".to_string()),
                        id: Some("att-2".to_string()),
                        size: None,
                    },
                ]),
            }),
        };

        let inbound = env.into_inbound().unwrap();
        assert_eq!(inbound.media.len(), 2);
        assert_eq!(inbound.media[0].kind, MediaType::Image);
        assert_eq!(inbound.media[0].mime_type.as_deref(), Some("image/jpeg"));
        assert_eq!(inbound.media[0].file_path.as_deref(), Some("photo.jpg"));
        assert_eq!(inbound.media[0].platform_id.as_deref(), Some("att-1"));
        assert_eq!(inbound.media[1].kind, MediaType::Document);
    }

    #[test]
    fn envelope_source_fallback() {
        let env = SignalEnvelope {
            source: Some("+legacy".to_string()),
            source_number: None,
            source_name: None,
            timestamp: Some(1000),
            data_message: Some(DataMessage {
                timestamp: Some(1000),
                message: Some("hi".to_string()),
                group_info: None,
                attachments: None,
            }),
        };

        let inbound = env.into_inbound().unwrap();
        assert_eq!(inbound.sender.id, "+legacy");
        assert_eq!(inbound.sender.name, "+legacy");
    }

    #[test]
    fn cli_wrapper_build_send_args() {
        let cli = SignalCliWrapper::new("signal-cli".to_string(), "+1000".to_string());
        let args = cli.build_send_args("+2000", "hello");
        assert!(args.contains(&"send".to_string()));
        assert!(args.contains(&"+2000".to_string()));
        assert!(args.contains(&"-m".to_string()));
        assert!(args.contains(&"hello".to_string()));
    }

    #[test]
    fn cli_wrapper_nonexistent_binary_not_available() {
        let cli = SignalCliWrapper::new(
            "/nonexistent/signal-cli-binary".to_string(),
            "+1000".to_string(),
        );
        assert!(!cli.is_available());
    }

    #[test]
    fn message_id_generation() {
        let channel = SignalChannel::new(test_config());
        let id1 = channel.next_message_id();
        let id2 = channel.next_message_id();
        assert_eq!(id1, "signal-1");
        assert_eq!(id2, "signal-2");
    }

    #[tokio::test]
    async fn channel_edit_and_delete() {
        let channel = SignalChannel::new(test_config());
        let msg = OutboundMessage {
            text: "edited".to_string(),
            reply_to: None,
            media: Vec::new(),
            buttons: None,
            silent: false,
            parse_mode: None,
        };

        channel
            .edit("+1000", "msg-1", msg)
            .await
            .expect("edit should succeed");
        {
            let messages = channel.messages.read().await;
            assert!(messages.contains_key("+1000:msg-1"));
        }

        channel
            .delete("+1000", "msg-1")
            .await
            .expect("delete should succeed");
        {
            let messages = channel.messages.read().await;
            assert!(!messages.contains_key("+1000:msg-1"));
        }
    }

    #[tokio::test]
    async fn channel_react_stores_emoji() {
        let channel = SignalChannel::new(test_config());
        channel
            .react("+1000", "msg-1", "👍")
            .await
            .expect("react should succeed");

        let messages = channel.messages.read().await;
        let reaction = messages.get("reaction:+1000:msg-1").unwrap();
        assert_eq!(reaction.text, "👍");
        assert!(reaction.silent);
    }
}
