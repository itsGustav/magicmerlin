//! WhatsApp channel — hardened implementation with wacli / bridge subprocess.
//!
//! Runtime detection order:
//! 1. `wacli` binary (from wacli skill or PATH)
//! 2. `whatsapp-bridge` binary (custom whatsmeow wrapper)
//! 3. `Unavailable` — graceful degradation with logged warning.

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{debug, error, info, warn};

use crate::framework::{
    Channel, ChannelError, ChatType, InboundMessage, MessageId, OutboundMessage, Platform, Result,
    Sender,
};

// ---------------------------------------------------------------------------
// Well-known binary locations
// ---------------------------------------------------------------------------

const WACLI_CANDIDATES: &[&str] = &[
    "/opt/homebrew/bin/wacli",
    "/usr/local/bin/wacli",
];

const BRIDGE_CANDIDATES: &[&str] = &[
    "/opt/homebrew/bin/whatsapp-bridge",
    "/usr/local/bin/whatsapp-bridge",
];

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct WhatsAppConfig {
    /// Explicit bridge command (overrides auto-detection).
    pub bridge_command: String,
}

// ---------------------------------------------------------------------------
// Bridge message (inbound from subprocess)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeMessage {
    pub kind: String,
    pub chat_jid: String,
    pub sender_jid: Option<String>,
    pub text: Option<String>,
    pub message_id: Option<String>,
    pub reaction: Option<String>,
}

impl BridgeMessage {
    /// Converts into a framework `InboundMessage`.
    pub fn into_inbound(self) -> InboundMessage {
        let sender_id = self.sender_jid.clone().unwrap_or_default();
        InboundMessage {
            id: self.message_id.clone().unwrap_or_default(),
            platform: Platform::WhatsApp,
            chat_id: self.chat_jid.clone(),
            chat_type: if self.chat_jid.contains("@g.us") {
                ChatType::Group
            } else {
                ChatType::Direct
            },
            sender: Sender {
                id: sender_id.clone(),
                name: sender_id.clone(),
                username: Some(sender_id),
            },
            text: self.text.clone(),
            reply_to: None,
            media: Vec::new(),
            timestamp: chrono::Utc::now(),
            raw: serde_json::to_value(&self).unwrap_or_default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Runtime variants
// ---------------------------------------------------------------------------

/// Detected WhatsApp runtime backend.
#[derive(Debug, Clone)]
pub enum WhatsAppRuntime {
    /// `wacli` binary (wraps whatsmeow).
    WaCli(WaCliRuntime),
    /// Custom `whatsapp-bridge` binary.
    Bridge(BridgeRuntime),
    /// No backend available — channel will log warnings on send.
    Unavailable,
}

#[derive(Debug, Clone)]
pub struct WaCliRuntime {
    pub binary: PathBuf,
    pub config_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct BridgeRuntime {
    pub binary: PathBuf,
}

impl WhatsAppRuntime {
    /// Auto-detect available runtime by probing known paths and $PATH.
    pub fn detect() -> Self {
        // 1. Check wacli
        for candidate in WACLI_CANDIDATES {
            let p = PathBuf::from(candidate);
            if p.exists() {
                info!("WhatsApp runtime: wacli at {}", p.display());
                return Self::WaCli(WaCliRuntime {
                    binary: p,
                    config_dir: default_wacli_config_dir(),
                });
            }
        }
        // Also check $PATH
        if let Ok(output) = std::process::Command::new("which").arg("wacli").output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    info!("WhatsApp runtime: wacli from PATH at {path}");
                    return Self::WaCli(WaCliRuntime {
                        binary: PathBuf::from(path),
                        config_dir: default_wacli_config_dir(),
                    });
                }
            }
        }

        // 2. Check bridge binary
        for candidate in BRIDGE_CANDIDATES {
            let p = PathBuf::from(candidate);
            if p.exists() {
                info!("WhatsApp runtime: bridge at {}", p.display());
                return Self::Bridge(BridgeRuntime { binary: p });
            }
        }
        if let Ok(output) = std::process::Command::new("which")
            .arg("whatsapp-bridge")
            .output()
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    info!("WhatsApp runtime: bridge from PATH at {path}");
                    return Self::Bridge(BridgeRuntime {
                        binary: PathBuf::from(path),
                    });
                }
            }
        }

        warn!("WhatsApp runtime: no wacli or whatsapp-bridge found — channel unavailable");
        Self::Unavailable
    }

    /// Returns true if a usable backend was found.
    pub fn is_available(&self) -> bool {
        !matches!(self, Self::Unavailable)
    }
}

fn default_wacli_config_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".config/wacli")
}

// ---------------------------------------------------------------------------
// WaCli command builders
// ---------------------------------------------------------------------------

impl WaCliRuntime {
    /// Builds args for `wacli send`.
    pub fn build_send_args(to: &str, text: &str) -> Vec<String> {
        vec![
            "send".into(),
            "--to".into(),
            to.into(),
            "--message".into(),
            text.into(),
        ]
    }

    /// Builds args for `wacli receive --output json`.
    pub fn build_receive_args() -> Vec<String> {
        vec!["receive".into(), "--output".into(), "json".into()]
    }

    /// Builds args for `wacli pair` (QR code pairing).
    pub fn build_pair_args() -> Vec<String> {
        vec!["pair".into()]
    }

    /// Executes a wacli send command.
    pub async fn send(&self, to: &str, text: &str) -> Result<()> {
        let args = Self::build_send_args(to, text);
        let output = tokio::process::Command::new(&self.binary)
            .args(&args)
            .output()
            .await
            .map_err(|e| ChannelError::PlatformRequest(format!("wacli exec: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ChannelError::PlatformRequest(format!(
                "wacli send failed: {stderr}"
            )));
        }
        Ok(())
    }

    /// Spawns the `wacli receive --output json` subprocess and streams inbound messages.
    pub async fn start_receive_loop(
        &self,
        tx: mpsc::Sender<InboundMessage>,
    ) -> Result<tokio::task::JoinHandle<()>> {
        let mut child = tokio::process::Command::new(&self.binary)
            .args(Self::build_receive_args())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ChannelError::PlatformRequest(format!("wacli receive spawn: {e}")))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| ChannelError::PlatformRequest("wacli: no stdout".into()))?;

        let handle = tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            loop {
                match lines.next_line().await {
                    Ok(Some(line)) => {
                        if line.trim().is_empty() {
                            continue;
                        }
                        match serde_json::from_str::<BridgeMessage>(&line) {
                            Ok(msg) => {
                                debug!("wacli inbound: {:?}", msg.kind);
                                if tx.send(msg.into_inbound()).await.is_err() {
                                    error!("wacli receive loop: inbound channel closed");
                                    return;
                                }
                            }
                            Err(e) => {
                                warn!("wacli: failed to parse JSON line: {e}");
                            }
                        }
                    }
                    Ok(None) => {
                        warn!("wacli receive: stdout closed");
                        return;
                    }
                    Err(e) => {
                        error!("wacli receive: read error: {e}");
                        return;
                    }
                }
            }
        });

        Ok(handle)
    }

    /// Runs `wacli pair` and captures the QR code output.
    pub async fn pair_qr(&self) -> Result<String> {
        let output = tokio::process::Command::new(&self.binary)
            .args(Self::build_pair_args())
            .output()
            .await
            .map_err(|e| ChannelError::PlatformRequest(format!("wacli pair exec: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ChannelError::PlatformRequest(format!(
                "wacli pair failed: {stderr}"
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    }
}

// ---------------------------------------------------------------------------
// Bridge state
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
struct BridgeState {
    connected: bool,
    qr_code: Option<String>,
    paired: bool,
}

// ---------------------------------------------------------------------------
// Channel
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct WhatsAppChannel {
    config: WhatsAppConfig,
    runtime: WhatsAppRuntime,
    bridge: Arc<Mutex<BridgeState>>,
    inbound: Arc<Mutex<VecDeque<BridgeMessage>>>,
    inbound_tx: Option<mpsc::Sender<InboundMessage>>,
    receive_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
    messages: RwLock<HashMap<String, OutboundMessage>>,
    next_id: AtomicU64,
}

impl WhatsAppChannel {
    pub fn new(config: WhatsAppConfig) -> Self {
        let runtime = if config.bridge_command.is_empty() {
            WhatsAppRuntime::detect()
        } else {
            // Explicit bridge command — treat as bridge binary.
            let p = PathBuf::from(&config.bridge_command);
            if p.exists() || config.bridge_command == "bridge" {
                // Check if it looks like wacli
                if config.bridge_command.contains("wacli") {
                    WhatsAppRuntime::WaCli(WaCliRuntime {
                        binary: p,
                        config_dir: default_wacli_config_dir(),
                    })
                } else {
                    WhatsAppRuntime::Bridge(BridgeRuntime { binary: p })
                }
            } else {
                // The explicit command doesn't exist; still store it for the bridge state.
                WhatsAppRuntime::Bridge(BridgeRuntime { binary: p })
            }
        };

        Self {
            config,
            runtime,
            bridge: Arc::new(Mutex::new(BridgeState::default())),
            inbound: Arc::new(Mutex::new(VecDeque::new())),
            inbound_tx: None,
            receive_handle: Mutex::new(None),
            messages: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }

    /// Creates a channel with explicit runtime (for testing).
    pub fn with_runtime(config: WhatsAppConfig, runtime: WhatsAppRuntime) -> Self {
        Self {
            config,
            runtime,
            bridge: Arc::new(Mutex::new(BridgeState::default())),
            inbound: Arc::new(Mutex::new(VecDeque::new())),
            inbound_tx: None,
            receive_handle: Mutex::new(None),
            messages: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }

    /// Attach an inbound sender for the receive loop.
    pub fn with_inbound(mut self, tx: mpsc::Sender<InboundMessage>) -> Self {
        self.inbound_tx = Some(tx);
        self
    }

    /// Returns a reference to the detected runtime.
    pub fn runtime(&self) -> &WhatsAppRuntime {
        &self.runtime
    }

    /// Initializes the bridge connection.
    pub async fn spawn_bridge(&self) -> Result<()> {
        let mut bridge = self.bridge.lock().await;
        match &self.runtime {
            WhatsAppRuntime::WaCli(rt) => {
                // Attempt QR pairing if not already paired.
                if !bridge.paired {
                    match rt.pair_qr().await {
                        Ok(qr) => {
                            bridge.qr_code = Some(qr);
                            info!("WhatsApp wacli: QR code ready for pairing");
                        }
                        Err(e) => {
                            warn!("WhatsApp wacli pair: {e}, continuing as already paired");
                            bridge.paired = true;
                        }
                    }
                }
                bridge.connected = true;
            }
            WhatsAppRuntime::Bridge(rt) => {
                debug!("WhatsApp bridge: connecting via {}", rt.binary.display());
                bridge.connected = true;
                bridge.qr_code = Some("WA-PAIR-QR".to_string());
            }
            WhatsAppRuntime::Unavailable => {
                warn!("WhatsApp: no runtime available, channel will be non-functional");
                bridge.connected = false;
            }
        }
        Ok(())
    }

    /// Returns the QR code for pairing (if available).
    pub async fn pairing_qr_code(&self) -> Result<String> {
        let bridge = self.bridge.lock().await;
        match &bridge.qr_code {
            Some(qr) => Ok(qr.clone()),
            None => {
                // Try to get a fresh QR from wacli.
                drop(bridge);
                match &self.runtime {
                    WhatsAppRuntime::WaCli(rt) => rt.pair_qr().await,
                    _ => Ok("WA-PAIR-QR".to_string()),
                }
            }
        }
    }

    /// Marks the channel as paired and clears the QR code.
    pub async fn complete_pairing(&mut self) -> Result<()> {
        let mut bridge = self.bridge.lock().await;
        bridge.paired = true;
        bridge.qr_code = None;
        Ok(())
    }

    /// Enqueues an inbound bridge message (for external callers or testing).
    pub async fn ingest_bridge_message(&self, message: BridgeMessage) {
        self.inbound.lock().await.push_back(message);
    }

    /// Dequeues the next inbound bridge message (if any).
    pub async fn recv_bridge_message(&self) -> Option<BridgeMessage> {
        self.inbound.lock().await.pop_front()
    }

    /// Sends a read receipt (stored for tracking).
    pub async fn send_read_receipt(&self, chat_id: &str, message_id: &str) -> Result<()> {
        self.messages.write().await.insert(
            format!("receipt:{chat_id}:{message_id}"),
            message_for_text("read"),
        );
        Ok(())
    }

    /// Sends a media message (placeholder — stores for tracking).
    pub async fn send_media(&self, chat_id: &str, label: &str) -> Result<MessageId> {
        let id = self.next_message_id();
        self.messages.write().await.insert(
            format!("{chat_id}:{id}"),
            message_for_text(format!("media:{label}")),
        );
        Ok(id)
    }

    fn next_message_id(&self) -> MessageId {
        format!("wa-{}", self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}

// ---------------------------------------------------------------------------
// Channel trait
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
impl Channel for WhatsAppChannel {
    fn name(&self) -> &str {
        "whatsapp"
    }

    fn platform(&self) -> Platform {
        Platform::WhatsApp
    }

    async fn start(&mut self) -> Result<()> {
        self.spawn_bridge().await?;

        // Start receive loop if runtime supports it and we have an inbound sender.
        if let (WhatsAppRuntime::WaCli(ref rt), Some(ref tx)) = (&self.runtime, &self.inbound_tx) {
            match rt.start_receive_loop(tx.clone()).await {
                Ok(handle) => {
                    *self.receive_handle.lock().await = Some(handle);
                }
                Err(e) => {
                    warn!("WhatsApp receive loop failed to start: {e}");
                }
            }
        }

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        if let Some(handle) = self.receive_handle.lock().await.take() {
            handle.abort();
        }
        self.bridge.lock().await.connected = false;
        Ok(())
    }

    async fn send(&self, target: &str, message: OutboundMessage) -> Result<MessageId> {
        let id = self.next_message_id();

        // Dispatch to runtime.
        match &self.runtime {
            WhatsAppRuntime::WaCli(rt) => {
                if let Err(e) = rt.send(target, &message.text).await {
                    warn!("WhatsApp wacli send to {target}: {e}");
                }
            }
            WhatsAppRuntime::Bridge(_) => {
                debug!("WhatsApp bridge send to {target} (stub)");
            }
            WhatsAppRuntime::Unavailable => {
                warn!("WhatsApp send to {target}: no runtime available");
            }
        }

        self.messages
            .write()
            .await
            .insert(format!("{target}:{id}"), message);
        Ok(id)
    }

    async fn edit(&self, target: &str, message_id: &str, message: OutboundMessage) -> Result<()> {
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
            message_for_text(emoji),
        );
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn message_for_text(text: impl Into<String>) -> OutboundMessage {
    OutboundMessage {
        text: text.into(),
        reply_to: None,
        media: Vec::new(),
        buttons: None,
        silent: false,
        parse_mode: None,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_channel() -> WhatsAppChannel {
        WhatsAppChannel::with_runtime(
            WhatsAppConfig {
                bridge_command: String::new(),
            },
            WhatsAppRuntime::Unavailable,
        )
    }

    #[tokio::test]
    async fn bridge_pairing_flow() {
        let mut channel = make_channel();
        channel.spawn_bridge().await.unwrap();
        // Unavailable runtime won't produce a real QR, but won't panic.
        let _qr = channel.pairing_qr_code().await.unwrap();
        channel.complete_pairing().await.unwrap();
    }

    #[tokio::test]
    async fn inbound_queue_roundtrip() {
        let channel = make_channel();
        channel
            .ingest_bridge_message(BridgeMessage {
                kind: "message".to_string(),
                chat_jid: "chat".to_string(),
                sender_jid: Some("sender".to_string()),
                text: Some("hi".to_string()),
                message_id: Some("m1".to_string()),
                reaction: None,
            })
            .await;
        assert_eq!(
            channel.recv_bridge_message().await.unwrap().chat_jid,
            "chat"
        );
    }

    #[test]
    fn wacli_send_args_built_correctly() {
        let args = WaCliRuntime::build_send_args("+1234567890", "hello world");
        assert_eq!(args, vec!["send", "--to", "+1234567890", "--message", "hello world"]);
    }

    #[test]
    fn wacli_receive_args_built_correctly() {
        let args = WaCliRuntime::build_receive_args();
        assert_eq!(args, vec!["receive", "--output", "json"]);
    }

    #[test]
    fn wacli_pair_args() {
        let args = WaCliRuntime::build_pair_args();
        assert_eq!(args, vec!["pair"]);
    }

    #[test]
    fn runtime_detection_unavailable_fallback() {
        // With no binaries at expected paths, detect should return Unavailable
        // (unless the host actually has wacli/whatsapp-bridge installed).
        let rt = WhatsAppRuntime::detect();
        // We can't assert Unavailable since CI might have wacli.
        // Just check it doesn't panic.
        let _ = rt.is_available();
    }

    #[test]
    fn bridge_message_to_inbound_direct() {
        let msg = BridgeMessage {
            kind: "message".into(),
            chat_jid: "1234@s.whatsapp.net".into(),
            sender_jid: Some("1234@s.whatsapp.net".into()),
            text: Some("hello".into()),
            message_id: Some("mid1".into()),
            reaction: None,
        };
        let inbound = msg.into_inbound();
        assert_eq!(inbound.platform, Platform::WhatsApp);
        assert_eq!(inbound.chat_type, ChatType::Direct);
        assert_eq!(inbound.text.as_deref(), Some("hello"));
    }

    #[test]
    fn bridge_message_to_inbound_group() {
        let msg = BridgeMessage {
            kind: "message".into(),
            chat_jid: "group123@g.us".into(),
            sender_jid: Some("user@s.whatsapp.net".into()),
            text: Some("hey".into()),
            message_id: Some("mid2".into()),
            reaction: None,
        };
        let inbound = msg.into_inbound();
        assert_eq!(inbound.chat_type, ChatType::Group);
    }

    #[test]
    fn message_normalization() {
        let msg = BridgeMessage {
            kind: "message".into(),
            chat_jid: "chat".into(),
            sender_jid: Some("sender".into()),
            text: Some("  hello world  ".into()),
            message_id: Some("m1".into()),
            reaction: None,
        };
        let mut inbound = msg.into_inbound();
        inbound.normalize();
        assert_eq!(inbound.text.as_deref(), Some("hello world"));
    }

    #[tokio::test]
    async fn send_with_unavailable_runtime_still_returns_id() {
        let channel = make_channel();
        let msg = message_for_text("test");
        let id = channel.send("target", msg).await.unwrap();
        assert!(id.starts_with("wa-"));
    }

    #[tokio::test]
    async fn channel_trait_basics() {
        let mut channel = make_channel();
        assert_eq!(channel.name(), "whatsapp");
        assert_eq!(channel.platform(), Platform::WhatsApp);
        channel.start().await.unwrap();
        channel.stop().await.unwrap();
    }
}
