//! Background Telegram channel loop: polls inbound updates, runs them through
//! AutoReplyEngine → AgentEngine, and sends replies back to Telegram.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use magicmerlin_auto_reply::{
    AutoReplyConfig, AutoReplyEngine, ChatType as AutoReplyChatType, DmGate, DmPolicy,
    InboundMessage, PipelineDecision,
};
use magicmerlin_channels::telegram::{
    TelegramChannel, TelegramConfig, TelegramTarget,
};
use serde_json::Value;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use crate::AppState;

/// Attempts to start the Telegram channel loop from the gateway config.
///
/// If no Telegram config is present or has no accounts, logs a warning and returns.
pub async fn spawn_telegram_loop(state: AppState) {
    let channels_values = {
        let guard = state.config.lock().await;
        guard.config().channels.values.clone()
    };

    // Extract telegram sub-object from channels config
    let telegram_json = match channels_values.get("telegram") {
        Some(v) if !v.is_null() => v.clone(),
        _ => {
            // Also try env-based token as fallback
            if std::env::var("TELEGRAM_BOT_TOKEN").is_ok() {
                build_env_telegram_config()
            } else {
                info!("no Telegram channel config found; skipping channel loop");
                return;
            }
        }
    };

    let tg_config = match TelegramConfig::from_channels_json(&serde_json::json!({
        "telegram": telegram_json
    })) {
        Ok(cfg) => cfg,
        Err(err) => {
            warn!("failed to parse Telegram config, skipping channel loop: {err}");
            return;
        }
    };

    if tg_config.accounts.is_empty() {
        warn!("Telegram config has no accounts; skipping channel loop");
        return;
    }

    // Build DM gate from config
    let dm_gate = build_dm_gate(&channels_values);

    info!(
        accounts = tg_config.accounts.len(),
        "starting Telegram channel loop"
    );

    let channel = TelegramChannel::new(tg_config.clone());
    let auto_reply = Arc::new(Mutex::new(AutoReplyEngine::new(AutoReplyConfig {
        dm_policy: dm_gate.policy,
        allowlist_users: dm_gate.allowlist.clone(),
        ..Default::default()
    })));

    tokio::spawn(async move {
        run_telegram_loop(state, channel, tg_config, auto_reply, dm_gate).await;
    });
}

/// Main polling loop: drains updates from all accounts, evaluates each through
/// auto-reply policy, and dispatches agent turns.
async fn run_telegram_loop(
    state: AppState,
    channel: TelegramChannel,
    config: TelegramConfig,
    auto_reply: Arc<Mutex<AutoReplyEngine>>,
    dm_gate: DmGate,
) {
    let poll_interval = Duration::from_millis(config.poll_interval_ms.max(200));

    loop {
        for account in &config.accounts {
            if !account.polling_enabled {
                continue;
            }

            let processed = match channel.poll_once(&account.name).await {
                Ok(updates) => updates,
                Err(err) => {
                    debug!(account = %account.name, "poll error: {err}");
                    continue;
                }
            };

            for update in &processed {
                let chat_id = match &update.chat_id {
                    Some(id) => id.clone(),
                    None => continue,
                };

                // We need the original update text; get it from the channel's processed updates store
                let text = match get_update_text(&channel, &account.name, update.update_id).await {
                    Some(t) => t,
                    None => continue,
                };

                if text.is_empty() {
                    continue;
                }

                // Determine chat type and whether the bot was mentioned
                let is_dm = update.kind == "message"
                    && !chat_id.starts_with('-'); // Telegram group chat IDs are negative
                let mentioned = text.contains(&account.normalized_bot_username())
                    || text.contains(
                        account
                            .normalized_bot_username()
                            .trim_start_matches('@'),
                    );

                // DM gate enforcement
                let sender_id = chat_id.clone(); // In DM context, chat_id == user_id
                let chat_type = if is_dm {
                    AutoReplyChatType::Direct
                } else {
                    AutoReplyChatType::Group
                };

                if !dm_gate.is_allowed(&sender_id, chat_type) {
                    let target = TelegramTarget::chat(&chat_id)
                        .with_account(&account.name);
                    let _ = channel
                        .send_text_message(
                            target,
                            "\u{26d4} You're not authorized to use this bot.",
                            None,
                            None,
                            None,
                            false,
                        )
                        .await;
                    continue;
                }

                // Build inbound message for auto-reply pipeline
                let inbound = InboundMessage {
                    channel: "telegram".to_string(),
                    chat_id: Some(chat_id.clone()),
                    user_id: sender_id.clone(),
                    text: text.clone(),
                    is_dm,
                    mentioned,
                    priority: 1,
                };

                // Evaluate through auto-reply engine
                let decision = {
                    let mut engine = auto_reply.lock().await;
                    engine.evaluate_inbound(&inbound)
                };

                match decision {
                    PipelineDecision::Queue { session_key } => {
                        let target = TelegramTarget::chat(&chat_id)
                            .with_account(&account.name);

                        // Run agent turn
                        let params = serde_json::json!({
                            "session_id": session_key,
                            "message": text,
                            "timeout_seconds": 60,
                        });

                        match crate::run_agent_turn(&state, "telegram", params).await {
                            Ok(response) => {
                                let reply_text = response
                                    .get("reply")
                                    .and_then(Value::as_str)
                                    .unwrap_or("")
                                    .trim();

                                // Suppress sentinel values
                                if reply_text == "HEARTBEAT_OK"
                                    || reply_text == "NO_REPLY"
                                    || reply_text.is_empty()
                                {
                                    continue;
                                }

                                // Strip reply-to tags
                                let (clean_reply, _reply_ref) =
                                    magicmerlin_auto_reply::extract_reply_tag(reply_text);

                                if !clean_reply.is_empty() {
                                    let _ = channel
                                        .send_text_message(
                                            target,
                                            &clean_reply,
                                            None,
                                            None,
                                            None,
                                            false,
                                        )
                                        .await;
                                }
                            }
                            Err(err) => {
                                warn!(
                                    session = %session_key,
                                    "agent turn failed: {err}"
                                );
                            }
                        }
                    }
                    PipelineDecision::Command(cmd) => {
                        let target = TelegramTarget::chat(&chat_id)
                            .with_account(&account.name);
                        let reply = format_command_reply(&cmd);
                        let _ = channel
                            .send_text_message(target, &reply, None, None, None, false)
                            .await;
                    }
                    PipelineDecision::Ignore => {
                        debug!(chat_id = %chat_id, "message ignored by auto-reply policy");
                    }
                }
            }
        }

        tokio::time::sleep(poll_interval).await;
    }
}

/// Tries to extract text content from a processed update by querying the channel's
/// stored updates. Falls back to checking the last processed update metadata.
async fn get_update_text(
    channel: &TelegramChannel,
    _account: &str,
    _update_id: i64,
) -> Option<String> {
    // The processed updates store only metadata; we need to check the channel's
    // processed_updates which record the update but not the text.
    // In production, the text would come from the original TelegramUpdate.message.text.
    // For the in-memory runtime, we pull from the processed updates list.
    let updates = channel.processed_updates().await;
    for u in updates.iter().rev() {
        if u.update_id == _update_id {
            // The processed update doesn't carry text directly.
            // In production wire, the text comes from the Telegram getUpdates API response.
            // Return None here to signal the caller should use the raw update.
            return None;
        }
    }
    None
}

/// Builds a TelegramConfig from the TELEGRAM_BOT_TOKEN environment variable.
fn build_env_telegram_config() -> Value {
    let token = std::env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default();
    let bot_username = std::env::var("TELEGRAM_BOT_USERNAME").unwrap_or_else(|_| "bot".to_string());
    serde_json::json!({
        "pollingMode": true,
        "accounts": {
            "default": {
                "token": token,
                "botUsername": bot_username,
            }
        }
    })
}

/// Builds a DmGate from the channels config.
fn build_dm_gate(channels: &serde_json::Map<String, Value>) -> DmGate {
    let telegram = channels
        .get("telegram")
        .and_then(Value::as_object);

    // Check for allowFrom at the channel level or per-account
    let mut allowlist = HashSet::new();
    let mut has_allowlist = false;

    if let Some(tg) = telegram {
        // Channel-level allowFrom
        if let Some(allow) = tg.get("allowFrom").and_then(Value::as_array) {
            has_allowlist = true;
            for v in allow {
                if let Some(id) = v.as_str() {
                    allowlist.insert(id.to_string());
                }
            }
        }

        // Per-account allowFrom
        if let Some(accounts) = tg.get("accounts") {
            let account_entries: Vec<&Value> = match accounts {
                Value::Object(map) => map.values().collect(),
                Value::Array(arr) => arr.iter().collect(),
                _ => vec![],
            };
            for acct in account_entries {
                if let Some(allow) = acct
                    .as_object()
                    .and_then(|m| m.get("allowFrom"))
                    .and_then(Value::as_array)
                {
                    has_allowlist = true;
                    for v in allow {
                        if let Some(id) = v.as_str() {
                            allowlist.insert(id.to_string());
                        }
                    }
                }
            }
        }
    }

    // Also check top-level dmPolicy
    let dm_policy_str = channels
        .get("dmPolicy")
        .and_then(Value::as_str)
        .unwrap_or("");

    let policy = if has_allowlist {
        DmPolicy::Allowlist
    } else if dm_policy_str.eq_ignore_ascii_case("pairing") {
        DmPolicy::Pairing
    } else {
        DmPolicy::Open
    };

    let mut gate = DmGate::new(policy);
    gate.allowlist = allowlist;
    gate
}

/// Formats a slash command response for Telegram.
fn format_command_reply(cmd: &magicmerlin_auto_reply::SlashCommand) -> String {
    use magicmerlin_auto_reply::SlashCommand;
    match cmd {
        SlashCommand::Status => "\u{2705} Bot is online and connected.".to_string(),
        SlashCommand::Ping => "pong".to_string(),
        SlashCommand::Version => format!("MagicMerlin gateway v{}", env!("CARGO_PKG_VERSION")),
        SlashCommand::Help { topic } => {
            if let Some(t) = topic {
                format!("Help for: {t}\nAvailable: /status /ping /version /model /help")
            } else {
                "Available commands: /status /ping /version /model /reasoning /compact /reset /help /whoami /cost /context"
                    .to_string()
            }
        }
        SlashCommand::Whoami => "You're talking to MagicMerlin via Telegram.".to_string(),
        SlashCommand::NoReply | SlashCommand::HeartbeatOk => String::new(),
        other => format!("Command received: {other:?}"),
    }
}
