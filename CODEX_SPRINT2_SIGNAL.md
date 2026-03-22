# Sprint 2 — Agent A: Signal Channel

## Context
Magic Merlin is a Rust OpenClaw clone at ~/Projects/magicmerlin.
Channels are in `channels/src/`. Signal is completely absent — must build from scratch.
The channel framework is in `channels/src/framework/` — all channels implement it.

## Your Mission
Build a fully functional Signal channel integration using the `presage` Rust library.

## Step 1: Add dependencies to channels/Cargo.toml

```toml
[target.'cfg(not(target_os = "windows"))'.dependencies]
presage = { git = "https://github.com/whisperfish/presage", branch = "main" }
presage-store-sled = { git = "https://github.com/whisperfish/presage", branch = "main" }
```

If presage has compilation issues or its API has changed, fall back to a subprocess wrapper:
- Check if `signal-cli` is installed at runtime
- Build a `SignalCliChannel` that shells out to `signal-cli` for send/receive

## Step 2: Create channels/src/signal/

Create `channels/src/signal/mod.rs` with:

```rust
// SignalChannel implements the channel framework trait
pub struct SignalChannel {
    config: SignalConfig,
    runtime: SignalRuntime,
}

pub struct SignalConfig {
    pub phone_number: String,          // registered Signal number
    pub data_dir: PathBuf,             // ~/.local/share/signal-cli or presage store
    pub group_support: bool,
    pub media_support: bool,
}

// SignalRuntime handles the actual connection
pub enum SignalRuntime {
    Presage(PresageRuntime),    // native Rust
    CliWrapper(SignalCliWrapper), // subprocess fallback
}
```

## Step 3: Implement message receive loop

```rust
impl SignalChannel {
    pub async fn run_receive_loop(&self, tx: mpsc::Sender<InboundMessage>) {
        loop {
            // Pull next message from Signal
            // Normalize to InboundMessage:
            // InboundMessage {
            //   id: uuid,
            //   sender_id: phone_number_or_group_id,
            //   sender_name: contact_name_or_phone,
            //   chat_type: ChatType::Direct or ChatType::Group,
            //   text: Option<String>,
            //   attachments: Vec<Attachment>,
            //   timestamp: DateTime<Utc>,
            //   platform: Platform::Signal,
            // }
            tx.send(msg).await?;
        }
    }
}
```

## Step 4: Implement send

```rust
impl SignalChannel {
    pub async fn send_message(&self, target: &str, text: &str) -> Result<String> {
        // target is either a phone number or group ID
        // Returns message ID on success
    }
    
    pub async fn send_with_attachment(&self, target: &str, text: &str, attachment: &Path) -> Result<String> {
        // Send media message
    }
}
```

## Step 5: Register in channel framework

In `channels/src/lib.rs`:
```rust
pub mod signal;
```

In `channels/src/framework/registry.rs`, add Signal to the channel registry builder.

## Step 6: Monitor with auto-reconnect

```rust
pub struct SignalMonitor {
    channel: Arc<SignalChannel>,
    retry_delay: Duration,
    max_retries: u32,
}

impl SignalMonitor {
    pub async fn run(&self, tx: mpsc::Sender<InboundMessage>) {
        loop {
            match self.channel.run_receive_loop(tx.clone()).await {
                Ok(()) => break,  // clean shutdown
                Err(e) => {
                    tracing::error!("Signal receive loop error: {e}");
                    tokio::time::sleep(self.retry_delay).await;
                }
            }
        }
    }
}
```

## Step 7: CLI subprocess fallback

If presage doesn't compile or isn't available, implement a `signal-cli` subprocess wrapper:

```rust
pub struct SignalCliWrapper {
    binary: PathBuf,   // path to signal-cli binary
    account: String,   // phone number
    data_dir: PathBuf,
}

impl SignalCliWrapper {
    // Sends: signal-cli -u {account} send -m {text} {recipient}
    pub async fn send(&self, recipient: &str, text: &str) -> Result<()>;
    
    // Receives: signal-cli -u {account} receive --output=json (line-by-line JSON)
    pub async fn receive_loop(&self, tx: mpsc::Sender<InboundMessage>) -> Result<()>;
}
```

## Rules
- `cargo build --workspace` must pass clean
- `#[cfg(test)]` unit tests for config parsing and message normalization
- Feature flag the entire Signal module: `#[cfg(feature = "signal")]` or make it runtime-optional

## Completion

```bash
openclaw system event --text "Sprint 2 Agent A done: Signal channel implemented (presage native + signal-cli fallback), receive loop, send, monitor, registered in framework" --mode now
```
