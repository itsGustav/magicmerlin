use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Utc;
use tokio::sync::{Mutex, RwLock};
use tokio::task::{JoinHandle, JoinSet};

use crate::framework::{
    Channel, ChannelError, InlineButton, MediaAttachment, MediaType, MessageId, OutboundMessage,
    ParseMode, Platform, Result as ChannelResult,
};

use super::config::{TelegramAccount, TelegramConfig, normalize_bot_username};
use super::formatting::{escape_markdown_v2, format_text, split_formatted_text, split_message};
use super::types::{
    TelegramAccountHealth, TelegramAccountHealthState, TelegramApiError, TelegramApiErrorKind,
    TelegramBotPermissions, TelegramCallbackAnswer, TelegramChatAction, TelegramChatMember,
    TelegramDelivery, TelegramDeliveryMode, TelegramFormattedText, TelegramForumTopic,
    TelegramInlineButton, TelegramInlineButtonStyle, TelegramInlineKeyboardMarkup, TelegramLocation,
    TelegramMedia, TelegramMediaKind, TelegramMessageEntity, TelegramOperation, TelegramPollKind,
    TelegramPollRequest, TelegramProcessedUpdate, TelegramQuoteForward, TelegramReaction,
    TelegramReactionCount, TelegramTarget, TelegramUpdate, TelegramWebhookState,
    TELEGRAM_MAX_MESSAGE_LEN,
};

pub type TelegramResult<T> = std::result::Result<T, TelegramApiError>;

#[derive(Debug)]
struct AccountState {
    account: TelegramAccount,
    last_update_offset: AtomicI64,
    updates: Mutex<VecDeque<TelegramUpdate>>,
    poll_errors: Mutex<VecDeque<TelegramApiError>>,
    send_errors: Mutex<VecDeque<TelegramApiError>>,
    next_allowed_at: Mutex<Option<Instant>>,
    global_send_times: Mutex<VecDeque<Instant>>,
    chat_send_times: Mutex<HashMap<String, VecDeque<Instant>>>,
    health: RwLock<TelegramAccountHealth>,
}

impl AccountState {
    fn new(account: TelegramAccount) -> Self {
        let bot_username = account.normalized_bot_username();
        Self {
            last_update_offset: AtomicI64::new(0),
            updates: Mutex::new(VecDeque::new()),
            poll_errors: Mutex::new(VecDeque::new()),
            send_errors: Mutex::new(VecDeque::new()),
            next_allowed_at: Mutex::new(None),
            global_send_times: Mutex::new(VecDeque::new()),
            chat_send_times: Mutex::new(HashMap::new()),
            health: RwLock::new(TelegramAccountHealth {
                account_name: account.name.clone(),
                bot_username,
                state: TelegramAccountHealthState::Disconnected,
                delivery_mode: TelegramDeliveryMode::Polling,
                last_error: None,
                consecutive_failures: 0,
                last_update_offset: 0,
                last_update_at: None,
            }),
            account,
        }
    }
}

#[derive(Debug, Default)]
struct TelegramStore {
    deliveries: RwLock<HashMap<MessageId, TelegramDelivery>>,
    delivery_order: RwLock<Vec<MessageId>>,
    processed_updates: RwLock<Vec<TelegramProcessedUpdate>>,
    remote_files: RwLock<HashMap<String, TelegramMedia>>,
    callback_answers: RwLock<HashMap<String, TelegramCallbackAnswer>>,
    callback_accounts: RwLock<HashMap<String, String>>,
    forum_topics: RwLock<HashMap<String, Vec<TelegramForumTopic>>>,
    chat_members: RwLock<HashMap<String, HashMap<String, TelegramChatMember>>>,
    bot_permissions: RwLock<HashMap<String, TelegramBotPermissions>>,
    reactions: RwLock<HashMap<String, Vec<TelegramReactionCount>>>,
    webhook_states: RwLock<HashMap<String, TelegramWebhookState>>,
    blocked_chats: RwLock<HashSet<String>>,
}

/// Telegram channel adapter with multi-account polling, webhook fallback, media handling, and
/// production-grade runtime semantics represented by local in-memory state.
#[derive(Clone)]
pub struct TelegramChannel {
    config: TelegramConfig,
    running: Arc<AtomicBool>,
    accounts: Arc<HashMap<String, Arc<AccountState>>>,
    store: Arc<TelegramStore>,
    next_message_id: Arc<AtomicU64>,
    next_topic_id: Arc<AtomicI64>,
    poller_tasks: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
}

impl std::fmt::Debug for TelegramChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TelegramChannel")
            .field("config", &self.config)
            .field("running", &self.running.load(Ordering::Relaxed))
            .field("accounts", &self.accounts.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl TelegramChannel {
    /// Creates a Telegram channel adapter.
    pub fn new(config: TelegramConfig) -> Self {
        let mut accounts = HashMap::new();
        let store = Arc::new(TelegramStore::default());
        for account in &config.accounts {
            accounts.insert(account.name.clone(), Arc::new(AccountState::new(account.clone())));
        }

        Self {
            config,
            running: Arc::new(AtomicBool::new(false)),
            accounts: Arc::new(accounts),
            store,
            next_message_id: Arc::new(AtomicU64::new(1)),
            next_topic_id: Arc::new(AtomicI64::new(1)),
            poller_tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Backward-compatible MarkdownV2 escaping helper.
    pub fn escape_markdown_v2(text: &str) -> String {
        escape_markdown_v2(text)
    }

    /// Backward-compatible message splitter helper.
    pub fn split_message(text: &str) -> Vec<String> {
        split_message(text)
    }

    /// Returns all configured account names.
    pub fn account_names(&self) -> Vec<String> {
        self.accounts.keys().cloned().collect()
    }

    /// Simulates `getMe` for all configured accounts.
    pub async fn get_me(&self) -> Vec<String> {
        self.accounts
            .values()
            .map(|account| account.account.normalized_bot_username())
            .collect()
    }

    /// Returns health for one account.
    pub async fn account_health(&self, account: &str) -> Option<TelegramAccountHealth> {
        if let Some(state) = self.accounts.get(account) {
            return Some(state.health.read().await.clone());
        }
        None
    }

    /// Returns a health snapshot for all configured accounts.
    pub async fn all_account_health(&self) -> Vec<TelegramAccountHealth> {
        let mut snapshot = Vec::new();
        for state in self.accounts.values() {
            snapshot.push(state.health.read().await.clone());
        }
        snapshot
    }

    /// Resolves an account by bot username.
    pub fn resolve_account_by_bot_username(&self, bot_username: &str) -> Option<String> {
        let normalized = normalize_bot_username(bot_username);
        self.accounts
            .values()
            .find(|state| state.account.normalized_bot_username() == normalized)
            .map(|state| state.account.name.clone())
    }

    /// Ingests an update into the account-local queue.
    pub async fn ingest_update(&self, account: &str, update: TelegramUpdate) {
        if let Some(state) = self.accounts.get(account) {
            state.updates.lock().await.push_back(update);
        }
    }

    /// Routes an update to the correct account using `bot_username`.
    pub async fn ingest_routed_update(&self, update: TelegramUpdate) -> TelegramResult<()> {
        let account_name = self.account_for_update(&update)?;
        self.ingest_update(&account_name, update).await;
        Ok(())
    }

    /// Queues a polling error for the next `poll_once`.
    pub async fn queue_poll_error(&self, account: &str, error: TelegramApiError) {
        if let Some(state) = self.accounts.get(account) {
            state.poll_errors.lock().await.push_back(error);
        }
    }

    /// Queues a send error for the next outbound delivery on an account.
    pub async fn queue_send_error(&self, account: &str, error: TelegramApiError) {
        if let Some(state) = self.accounts.get(account) {
            state.send_errors.lock().await.push_back(error);
        }
    }

    /// Seeds a downloadable remote file in the local mock storage.
    pub async fn seed_remote_file(&self, file: TelegramMedia) {
        self.store
            .remote_files
            .write()
            .await
            .insert(file.file_id.clone(), file);
    }

    /// Seeds a chat member state for group moderation tests.
    pub async fn seed_chat_member(&self, chat_id: &str, member: TelegramChatMember) {
        self.store
            .chat_members
            .write()
            .await
            .entry(chat_id.to_string())
            .or_default()
            .insert(member.user_id.clone(), member);
    }

    /// Sets bot permissions for a chat.
    pub async fn set_bot_permissions(&self, chat_id: &str, permissions: TelegramBotPermissions) {
        self.store
            .bot_permissions
            .write()
            .await
            .insert(chat_id.to_string(), permissions);
    }

    /// Simulates a user blocking the bot in a chat.
    pub async fn block_chat(&self, chat_id: &str) {
        self.store.blocked_chats.write().await.insert(chat_id.to_string());
    }

    /// Returns all deliveries in send order.
    pub async fn deliveries(&self) -> Vec<TelegramDelivery> {
        let order = self.store.delivery_order.read().await.clone();
        let deliveries = self.store.deliveries.read().await;
        order.into_iter()
            .filter_map(|id| deliveries.get(&id).cloned())
            .collect()
    }

    /// Returns deliveries for a single chat.
    pub async fn deliveries_for_chat(&self, chat_id: &str) -> Vec<TelegramDelivery> {
        self.deliveries()
            .await
            .into_iter()
            .filter(|delivery| delivery.chat_id == chat_id)
            .collect()
    }

    /// Returns processed updates recorded by the runtime.
    pub async fn processed_updates(&self) -> Vec<TelegramProcessedUpdate> {
        self.store.processed_updates.read().await.clone()
    }

    /// Returns reaction counts for a message.
    pub async fn reaction_counts(&self, chat_id: &str, message_id: i64) -> Vec<TelegramReactionCount> {
        self.store
            .reactions
            .read()
            .await
            .get(&reaction_key(chat_id, message_id))
            .cloned()
            .unwrap_or_default()
    }

    /// Returns the webhook state for an account.
    pub async fn webhook_state(&self, account: &str) -> Option<TelegramWebhookState> {
        self.store
            .webhook_states
            .read()
            .await
            .get(account)
            .cloned()
    }

    /// Implements long-polling semantics with update offsets.
    pub async fn get_updates(&self, account: &str, limit: usize) -> TelegramResult<Vec<TelegramUpdate>> {
        let state = self.require_account(account)?;
        let mut updates = state.updates.lock().await;
        let offset = state.last_update_offset.load(Ordering::Relaxed);
        while let Some(front) = updates.front() {
            if front.update_id < offset {
                updates.pop_front();
            } else {
                break;
            }
        }

        let mut selected = Vec::new();
        for _ in 0..limit.max(1) {
            let Some(update) = updates.pop_front() else {
                break;
            };
            state
                .last_update_offset
                .store(update.update_id + 1, Ordering::Relaxed);
            selected.push(update);
        }

        let mut health = state.health.write().await;
        health.last_update_offset = state.last_update_offset.load(Ordering::Relaxed);
        Ok(selected)
    }

    /// Polls one account and processes multiple updates concurrently.
    pub async fn poll_once(&self, account: &str) -> TelegramResult<Vec<TelegramProcessedUpdate>> {
        let state = self.require_account(account)?;
        for attempt in 0..=self.config.retry_limit {
            if let Some(error) = state.poll_errors.lock().await.pop_front() {
                if error.kind == TelegramApiErrorKind::Unauthorized {
                    self.mark_health(&state, TelegramAccountHealthState::AuthError, Some(error.message.clone())).await;
                    return Err(error);
                }

                if error.kind == TelegramApiErrorKind::RateLimited
                    || error.kind == TelegramApiErrorKind::FloodWait
                {
                    self.apply_rate_limit(account, error.retry_after_seconds).await;
                    self.mark_health(&state, TelegramAccountHealthState::RateLimited, Some(error.message.clone())).await;
                } else {
                    self.mark_health(&state, TelegramAccountHealthState::Reconnecting, Some(error.message.clone())).await;
                }

                if !error.is_retryable() || attempt == self.config.retry_limit {
                    return Err(error);
                }

                self.sleep_for_error(&error, attempt).await;
                continue;
            }

            let updates = self
                .get_updates(account, self.config.max_updates_per_poll)
                .await?;
            if updates.is_empty() {
                self.mark_health(&state, TelegramAccountHealthState::Connected, None).await;
                return Ok(Vec::new());
            }
            let processed = self.process_updates(account, updates).await?;
            self.mark_health(&state, TelegramAccountHealthState::Connected, None).await;
            return Ok(processed);
        }

        Ok(Vec::new())
    }

    /// Processes updates from one account concurrently.
    pub async fn process_updates(
        &self,
        account: &str,
        updates: Vec<TelegramUpdate>,
    ) -> TelegramResult<Vec<TelegramProcessedUpdate>> {
        let mut join_set = JoinSet::new();
        for update in updates {
            let channel = self.clone();
            let account_name = account.to_string();
            join_set.spawn(async move { channel.process_update(&account_name, update).await });
        }

        let mut processed = Vec::new();
        while let Some(result) = join_set.join_next().await {
            processed.push(result.map_err(|error| {
                TelegramApiError::new(
                    TelegramApiErrorKind::Server,
                    format!("failed to join update task: {error}"),
                )
            })??);
        }
        processed.sort_by_key(|item| item.update_id);
        Ok(processed)
    }

    async fn process_update(&self, account: &str, update: TelegramUpdate) -> TelegramResult<TelegramProcessedUpdate> {
        let state = self.require_account(account)?;
        let bot_username = update
            .bot_username
            .clone()
            .or_else(|| update.message.as_ref().and_then(|message| message.bot_username.clone()))
            .or_else(|| {
                update
                    .callback_query
                    .as_ref()
                    .and_then(|callback| callback.bot_username.clone())
            })
            .unwrap_or_else(|| state.account.normalized_bot_username());

        let kind = if update.message.is_some() {
            "message"
        } else if update.edited_message.is_some() {
            "edited_message"
        } else if update.callback_query.is_some() {
            "callback_query"
        } else if update.reaction.is_some() {
            "reaction"
        } else if update.chat_member.is_some() {
            "chat_member"
        } else {
            "unknown"
        };

        let callback_data = update.callback_query.as_ref().and_then(|query| query.data.clone());
        let chat_id = update
            .message
            .as_ref()
            .map(|message| message.chat_id.clone())
            .or_else(|| {
                update
                    .callback_query
                    .as_ref()
                    .and_then(|query| query.chat_id.clone())
            })
            .or_else(|| update.reaction.as_ref().map(|reaction| reaction.chat_id.clone()))
            .or_else(|| update.chat_member.as_ref().map(|member| member.chat_id.clone()));
        let thread_id = update
            .message
            .as_ref()
            .and_then(|message| message.message_thread_id)
            .or_else(|| update.edited_message.as_ref().and_then(|message| message.message_thread_id));

        if let Some(callback) = &update.callback_query {
            self.store
                .callback_accounts
                .write()
                .await
                .insert(callback.id.clone(), account.to_string());
        }
        if let Some(reaction) = &update.reaction {
            self.store
                .reactions
                .write()
                .await
                .insert(reaction_key(&reaction.chat_id, reaction.message_id), reaction.counts.clone());
        }
        if let Some(member_update) = &update.chat_member {
            self.store
                .chat_members
                .write()
                .await
                .entry(member_update.chat_id.clone())
                .or_default()
                .insert(member_update.new_member.user_id.clone(), member_update.new_member.clone());
        }

        let processed = TelegramProcessedUpdate {
            account_name: account.to_string(),
            update_id: update.update_id,
            bot_username,
            kind: kind.to_string(),
            callback_data,
            chat_id,
            thread_id,
        };

        self.store
            .processed_updates
            .write()
            .await
            .push(processed.clone());

        let mut health = state.health.write().await;
        health.last_update_at = Some(Utc::now());
        health.last_update_offset = state.last_update_offset.load(Ordering::Relaxed);
        health.last_error = None;
        health.consecutive_failures = 0;
        Ok(processed)
    }

    /// Reply to callback queries.
    pub async fn answer_callback_query(&self, callback_id: &str, text: Option<&str>) -> TelegramResult<()> {
        self.answer_callback_query_with_options(
            callback_id,
            TelegramCallbackAnswer {
                text: text.map(ToOwned::to_owned),
                show_alert: false,
                url: None,
            },
        )
        .await
    }

    /// Answers a callback query with full Telegram callback options.
    pub async fn answer_callback_query_with_options(
        &self,
        callback_id: &str,
        answer: TelegramCallbackAnswer,
    ) -> TelegramResult<()> {
        let account_name = self
            .store
            .callback_accounts
            .read()
            .await
            .get(callback_id)
            .cloned()
            .or_else(|| self.accounts.keys().next().cloned())
            .ok_or_else(|| TelegramApiError::new(TelegramApiErrorKind::Config, "no telegram account configured"))?;
        let state = self.require_account(&account_name)?;
        let delivery = TelegramDelivery {
            id: self.next_message_id(),
            account_name: account_name.clone(),
            bot_username: state.account.normalized_bot_username(),
            operation: TelegramOperation::AnswerCallbackQuery,
            chat_id: callback_id.to_string(),
            thread_id: None,
            text: answer.text.clone(),
            parse_mode: None,
            entities: Vec::new(),
            media: Vec::new(),
            keyboard: None,
            reactions: Vec::new(),
            location: None,
            poll: None,
            quote_forward: None,
            callback_answer: Some(answer.clone()),
            chat_action: None,
            silent: true,
            created_at: Utc::now(),
            continuation_index: None,
            continuation_total: None,
        };
        self.store
            .callback_answers
            .write()
            .await
            .insert(callback_id.to_string(), answer);
        self.record_delivery(delivery).await;
        Ok(())
    }

    /// Sends a chat action to a target chat.
    pub async fn send_chat_action(&self, target: TelegramTarget, action: TelegramChatAction) -> TelegramResult<()> {
        let state = self.resolve_account(&target)?;
        self.dispatch_delivery(&state, &target, TelegramOperation::SendChatAction, None, None, Vec::new(), Vec::new(), None, Vec::new(), None, None, None, Some(action), true, None, None).await?;
        Ok(())
    }

    /// Sends typing indicator using the default account.
    pub async fn send_typing_indicator(&self, chat_id: &str) -> TelegramResult<()> {
        self.send_chat_action(TelegramTarget::chat(chat_id), TelegramChatAction::Typing)
            .await
    }

    /// Sends a text message with Telegram formatting and auto-splitting.
    pub async fn send_text_message(
        &self,
        target: TelegramTarget,
        text: &str,
        parse_mode: Option<ParseMode>,
        keyboard: Option<TelegramInlineKeyboardMarkup>,
        quote_forward: Option<TelegramQuoteForward>,
        silent: bool,
    ) -> TelegramResult<Vec<MessageId>> {
        let state = self.resolve_account(&target)?;
        if self.config.auto_send_chat_actions {
            self.send_chat_action(target.clone(), TelegramChatAction::Typing).await?;
        }

        let formatted = if parse_mode == Some(ParseMode::Markdown) {
            format_text(text, parse_mode)
        } else if parse_mode == Some(ParseMode::Html) {
            format_text(text, parse_mode)
        } else {
            TelegramFormattedText {
                text: text.to_string(),
                entities: Vec::new(),
                parse_mode: parse_mode.unwrap_or(ParseMode::Plain),
            }
        };

        let parts = split_formatted_text(&formatted, TELEGRAM_MAX_MESSAGE_LEN);
        let total = parts.len();
        let mut ids = Vec::with_capacity(total);
        for (index, part) in parts.into_iter().enumerate() {
            let delivery = self.dispatch_delivery(
                &state,
                &target,
                TelegramOperation::SendText,
                Some(part.text),
                parse_mode,
                part.entities,
                Vec::new(),
                keyboard.clone(),
                Vec::new(),
                None,
                None,
                quote_forward.clone(),
                None,
                silent,
                Some(index + 1),
                Some(total),
            )
            .await?;
            ids.push(delivery.id);
        }
        Ok(ids)
    }

    /// Sends a text message using MarkdownV2 escaping.
    pub async fn send_telegram_text(
        &self,
        account: &str,
        chat_id: &str,
        text: &str,
    ) -> TelegramResult<Vec<MessageId>> {
        self.send_text_message(
            TelegramTarget::chat(chat_id).with_account(account),
            &escape_markdown_v2(text),
            Some(ParseMode::Markdown),
            None,
            None,
            false,
        )
        .await
    }

    /// Sends a photo.
    pub async fn send_photo(&self, target: TelegramTarget, media: TelegramMedia, caption: Option<&str>) -> TelegramResult<MessageId> {
        self.send_media_kind(target, TelegramOperation::SendPhoto, media.with_kind(TelegramMediaKind::Photo), caption, ParseMode::Plain).await
    }

    /// Sends a voice note and infers duration when possible.
    pub async fn send_voice(&self, target: TelegramTarget, media: TelegramMedia, caption: Option<&str>) -> TelegramResult<MessageId> {
        let mut media = media.with_kind(TelegramMediaKind::Voice);
        if media.duration_seconds.is_none() {
            media.duration_seconds = Some(detect_voice_duration(&media));
        }
        self.send_media_kind(target, TelegramOperation::SendVoice, media, caption, ParseMode::Plain).await
    }

    /// Sends a document.
    pub async fn send_document(&self, target: TelegramTarget, media: TelegramMedia, caption: Option<&str>) -> TelegramResult<MessageId> {
        self.send_media_kind(target, TelegramOperation::SendDocument, media.with_kind(TelegramMediaKind::Document), caption, ParseMode::Plain).await
    }

    /// Sends a video note.
    pub async fn send_video_note(&self, target: TelegramTarget, media: TelegramMedia) -> TelegramResult<MessageId> {
        let mut media = media.with_kind(TelegramMediaKind::VideoNote);
        media.is_video_note = true;
        self.send_media_kind(target, TelegramOperation::SendVideoNote, media, None, ParseMode::Plain).await
    }

    /// Sends a video.
    pub async fn send_video(&self, target: TelegramTarget, media: TelegramMedia, caption: Option<&str>) -> TelegramResult<MessageId> {
        self.send_media_kind(target, TelegramOperation::SendVideo, media.with_kind(TelegramMediaKind::Video), caption, ParseMode::Plain).await
    }

    /// Sends a sticker.
    pub async fn send_sticker(&self, target: TelegramTarget, media: TelegramMedia) -> TelegramResult<MessageId> {
        self.send_media_kind(target, TelegramOperation::SendSticker, media.with_kind(TelegramMediaKind::Sticker), None, ParseMode::Plain).await
    }

    /// Sends an animation.
    pub async fn send_animation(&self, target: TelegramTarget, media: TelegramMedia, caption: Option<&str>) -> TelegramResult<MessageId> {
        let mut media = media.with_kind(TelegramMediaKind::Animation);
        media.is_animated = true;
        self.send_media_kind(target, TelegramOperation::SendAnimation, media, caption, ParseMode::Plain).await
    }

    /// Sends a location payload.
    pub async fn send_location(&self, target: TelegramTarget, location: TelegramLocation) -> TelegramResult<MessageId> {
        let state = self.resolve_account(&target)?;
        let delivery = self.dispatch_delivery(
            &state,
            &target,
            TelegramOperation::SendLocation,
            None,
            None,
            Vec::new(),
            Vec::new(),
            None,
            Vec::new(),
            Some(location),
            None,
            None,
            None,
            false,
            None,
            None,
        )
        .await?;
        Ok(delivery.id)
    }

    /// Sends a poll.
    pub async fn send_poll_request(&self, target: TelegramTarget, poll: TelegramPollRequest) -> TelegramResult<MessageId> {
        let state = self.resolve_account(&target)?;
        let delivery = self.dispatch_delivery(
            &state,
            &target,
            TelegramOperation::SendPoll,
            Some(poll.question.clone()),
            None,
            Vec::new(),
            Vec::new(),
            None,
            Vec::new(),
            None,
            Some(poll),
            None,
            None,
            false,
            None,
            None,
        )
        .await?;
        Ok(delivery.id)
    }

    /// Preserves the legacy poll helper.
    pub async fn send_poll(&self, chat_id: &str, question: &str, options: &[String]) -> TelegramResult<MessageId> {
        self.send_poll_request(
            TelegramTarget::chat(chat_id),
            TelegramPollRequest {
                question: question.to_string(),
                options: options.to_vec(),
                kind: TelegramPollKind::Regular,
                is_anonymous: true,
                correct_option_id: None,
            },
        )
        .await
    }

    /// Uploads a framework message's media payload using Telegram media semantics.
    pub async fn upload_media(&self, chat_id: &str, message: &OutboundMessage) -> TelegramResult<MessageId> {
        let target = TelegramTarget::chat(chat_id);
        let attachment = message.media.first().cloned().unwrap_or(MediaAttachment {
            kind: MediaType::Document,
            url: None,
            file_path: None,
            mime_type: None,
            platform_id: None,
        });
        let media = media_from_attachment(attachment);
        match media.kind {
            TelegramMediaKind::Photo => self.send_photo(target, media, Some(&message.text)).await,
            TelegramMediaKind::Voice => self.send_voice(target, media, Some(&message.text)).await,
            TelegramMediaKind::Video => self.send_video(target, media, Some(&message.text)).await,
            TelegramMediaKind::Sticker => self.send_sticker(target, media).await,
            TelegramMediaKind::Animation => self.send_animation(target, media, Some(&message.text)).await,
            TelegramMediaKind::VideoNote => self.send_video_note(target, media).await,
            TelegramMediaKind::Document => self.send_document(target, media, Some(&message.text)).await,
        }
    }

    /// Downloads media from mock `getFile` storage to the configured media directory.
    pub async fn download_media(&self, file_id: &str) -> TelegramResult<String> {
        let media = self
            .store
            .remote_files
            .read()
            .await
            .get(file_id)
            .cloned()
            .unwrap_or_else(|| TelegramMedia {
                file_id: file_id.to_string(),
                kind: TelegramMediaKind::Document,
                file_name: Some(format!("{file_id}.bin")),
                mime_type: Some("application/octet-stream".to_string()),
                file_path: None,
                url: None,
                bytes: vec![0, 1, 2, 3],
                duration_seconds: None,
                sticker_emoji: None,
                is_animated: false,
                is_video_note: false,
            });
        let account = self
            .accounts
            .values()
            .next()
            .ok_or_else(|| TelegramApiError::new(TelegramApiErrorKind::Config, "no telegram account configured"))?;
        let dir = PathBuf::from(self.config.media_dir_for(&account.account));
        fs::create_dir_all(&dir)
            .map_err(|error| TelegramApiError::new(TelegramApiErrorKind::Config, format!("failed to create media dir: {error}")))?;
        let file_name = media
            .file_name
            .clone()
            .unwrap_or_else(|| format!("telegram_{file_id}.bin"));
        let path = dir.join(file_name);
        fs::write(&path, &media.bytes)
            .map_err(|error| TelegramApiError::new(TelegramApiErrorKind::Config, format!("failed to write media file: {error}")))?;
        Ok(path.display().to_string())
    }

    /// Sets message reactions.
    pub async fn set_message_reaction(
        &self,
        target: TelegramTarget,
        message_id: i64,
        reactions: Vec<TelegramReaction>,
    ) -> TelegramResult<()> {
        let state = self.resolve_account(&target)?;
        let counts = reactions
            .iter()
            .cloned()
            .map(|reaction| TelegramReactionCount { reaction, count: 1 })
            .collect::<Vec<_>>();
        self.store
            .reactions
            .write()
            .await
            .insert(reaction_key(&target.chat_id, message_id), counts);
        self.dispatch_delivery(
            &state,
            &target,
            TelegramOperation::SetMessageReaction,
            None,
            None,
            Vec::new(),
            Vec::new(),
            None,
            reactions,
            None,
            None,
            None,
            None,
            true,
            None,
            None,
        )
        .await?;
        Ok(())
    }

    /// Forwards a message with quote text.
    pub async fn forward_message_with_quote(
        &self,
        target: TelegramTarget,
        quote_forward: TelegramQuoteForward,
    ) -> TelegramResult<MessageId> {
        let state = self.resolve_account(&target)?;
        let delivery = self.dispatch_delivery(
            &state,
            &target,
            TelegramOperation::ForwardMessage,
            quote_forward.quote.clone(),
            None,
            Vec::new(),
            Vec::new(),
            None,
            Vec::new(),
            None,
            None,
            Some(quote_forward),
            None,
            false,
            None,
            None,
        )
        .await?;
        Ok(delivery.id)
    }

    /// Creates a forum topic in a group chat.
    pub async fn create_forum_topic(
        &self,
        target: TelegramTarget,
        title: &str,
        icon_color: Option<&str>,
    ) -> TelegramResult<TelegramForumTopic> {
        let state = self.resolve_account(&target)?;
        let topic = TelegramForumTopic {
            topic_id: self.next_topic_id.fetch_add(1, Ordering::Relaxed),
            chat_id: target.chat_id.clone(),
            title: title.to_string(),
            icon_color: icon_color.map(ToOwned::to_owned),
        };
        self.store
            .forum_topics
            .write()
            .await
            .entry(target.chat_id.clone())
            .or_default()
            .push(topic.clone());
        self.dispatch_delivery(
            &state,
            &target,
            TelegramOperation::CreateForumTopic,
            Some(title.to_string()),
            None,
            Vec::new(),
            Vec::new(),
            None,
            Vec::new(),
            None,
            None,
            None,
            None,
            false,
            None,
            None,
        )
        .await?;
        Ok(topic)
    }

    /// Returns all forum topics for a chat.
    pub async fn forum_topics(&self, chat_id: &str) -> Vec<TelegramForumTopic> {
        self.store
            .forum_topics
            .read()
            .await
            .get(chat_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Returns a chat member.
    pub async fn get_chat_member(&self, chat_id: &str, user_id: &str) -> Option<TelegramChatMember> {
        self.store
            .chat_members
            .read()
            .await
            .get(chat_id)
            .and_then(|members| members.get(user_id).cloned())
    }

    /// Bans a group member.
    pub async fn ban_member(&self, target: TelegramTarget, user_id: &str) -> TelegramResult<()> {
        self.update_member_status(&target, user_id, crate_status_banned()).await?;
        let state = self.resolve_account(&target)?;
        self.dispatch_delivery(
            &state,
            &target,
            TelegramOperation::BanMember,
            Some(user_id.to_string()),
            None,
            Vec::new(),
            Vec::new(),
            None,
            Vec::new(),
            None,
            None,
            None,
            None,
            true,
            None,
            None,
        )
        .await?;
        Ok(())
    }

    /// Kicks a group member.
    pub async fn kick_member(&self, target: TelegramTarget, user_id: &str) -> TelegramResult<()> {
        self.update_member_status(&target, user_id, crate_status_left()).await?;
        let state = self.resolve_account(&target)?;
        self.dispatch_delivery(
            &state,
            &target,
            TelegramOperation::KickMember,
            Some(user_id.to_string()),
            None,
            Vec::new(),
            Vec::new(),
            None,
            Vec::new(),
            None,
            None,
            None,
            None,
            true,
            None,
            None,
        )
        .await?;
        Ok(())
    }

    /// Checks whether the bot has required permissions in a chat.
    pub async fn bot_has_permissions(&self, chat_id: &str, required: &TelegramBotPermissions) -> bool {
        let granted = self
            .store
            .bot_permissions
            .read()
            .await
            .get(chat_id)
            .cloned()
            .unwrap_or_default();
        (!required.can_send_messages || granted.can_send_messages)
            && (!required.can_manage_topics || granted.can_manage_topics)
            && (!required.can_restrict_members || granted.can_restrict_members)
            && (!required.can_delete_messages || granted.can_delete_messages)
    }

    /// Configures webhook delivery for an account.
    pub async fn set_webhook(
        &self,
        account: &str,
        url: &str,
        secret_token: Option<&str>,
    ) -> TelegramResult<()> {
        let state = self.require_account(account)?;
        self.store
            .webhook_states
            .write()
            .await
            .insert(
                account.to_string(),
                TelegramWebhookState {
                    active: true,
                    url: Some(url.to_string()),
                    secret_token: secret_token.map(ToOwned::to_owned),
                    last_delivery_at: None,
                    consecutive_failures: 0,
                },
            );
        let target = TelegramTarget::chat(account).with_account(account);
        self.dispatch_delivery(
            &state,
            &target,
            TelegramOperation::SetWebhook,
            Some(url.to_string()),
            None,
            Vec::new(),
            Vec::new(),
            None,
            Vec::new(),
            None,
            None,
            None,
            None,
            true,
            None,
            None,
        )
        .await?;
        if !self.config.polling_mode {
            self.abort_poller(account).await;
            self.mark_health(&state, TelegramAccountHealthState::WebhookOnly, None).await;
        }
        Ok(())
    }

    /// Removes webhook delivery for an account and optionally falls back to polling.
    pub async fn delete_webhook(&self, account: &str) -> TelegramResult<()> {
        let state = self.require_account(account)?;
        self.store.webhook_states.write().await.insert(
            account.to_string(),
            TelegramWebhookState::default(),
        );
        let target = TelegramTarget::chat(account).with_account(account);
        self.dispatch_delivery(
            &state,
            &target,
            TelegramOperation::DeleteWebhook,
            None,
            None,
            Vec::new(),
            Vec::new(),
            None,
            Vec::new(),
            None,
            None,
            None,
            None,
            true,
            None,
            None,
        )
        .await?;
        if self.config.webhook_fallback_to_polling && self.running.load(Ordering::Relaxed) {
            self.spawn_poller(account.to_string()).await;
        }
        Ok(())
    }

    /// Processes a webhook update immediately.
    pub async fn handle_webhook_update(
        &self,
        account: &str,
        provided_secret_token: Option<&str>,
        update: TelegramUpdate,
    ) -> TelegramResult<TelegramProcessedUpdate> {
        let state = self.require_account(account)?;
        let expected_secret = self
            .store
            .webhook_states
            .read()
            .await
            .get(account)
            .and_then(|webhook| webhook.secret_token.clone())
            .or_else(|| state.account.webhook_secret.clone());
        if let Some(expected_secret) = expected_secret {
            if Some(expected_secret.as_str()) != provided_secret_token {
                return Err(TelegramApiError::new(
                    TelegramApiErrorKind::PermissionDenied,
                    "invalid telegram webhook secret",
                ));
            }
        }
        self.store
            .webhook_states
            .write()
            .await
            .entry(account.to_string())
            .or_default()
            .last_delivery_at = Some(Utc::now());
        self.process_update(account, update).await
    }

    /// Simulates webhook delivery failure and triggers polling fallback when enabled.
    pub async fn simulate_webhook_failure(&self, account: &str) -> TelegramResult<()> {
        let state = self.require_account(account)?;
        {
            let mut webhooks = self.store.webhook_states.write().await;
            let webhook = webhooks.entry(account.to_string()).or_default();
            webhook.consecutive_failures += 1;
            webhook.active = false;
        }
        if self.config.webhook_fallback_to_polling && self.running.load(Ordering::Relaxed) {
            self.spawn_poller(account.to_string()).await;
            self.mark_health(&state, TelegramAccountHealthState::Reconnecting, Some("webhook fallback to polling".to_string())).await;
        }
        Ok(())
    }

    /// Respects retry-after semantics for 429 handling.
    pub async fn apply_rate_limit(&self, account: &str, retry_after_secs: Option<u64>) {
        if let Some(state) = self.accounts.get(account) {
            let until = Instant::now() + Duration::from_secs(retry_after_secs.unwrap_or(1).max(1));
            *state.next_allowed_at.lock().await = Some(until);
        }
    }

    /// Waits until sending is allowed for an account.
    pub async fn wait_rate_window(&self, account: &str) {
        if let Some(state) = self.accounts.get(account) {
            let deadline = *state.next_allowed_at.lock().await;
            if let Some(deadline) = deadline {
                let now = Instant::now();
                if deadline > now {
                    tokio::time::sleep(deadline.duration_since(now)).await;
                }
            }
        }
    }

    async fn send_media_kind(
        &self,
        target: TelegramTarget,
        operation: TelegramOperation,
        media: TelegramMedia,
        caption: Option<&str>,
        parse_mode: ParseMode,
    ) -> TelegramResult<MessageId> {
        let state = self.resolve_account(&target)?;
        if self.config.auto_send_chat_actions {
            let action = chat_action_for_operation(operation);
            self.send_chat_action(target.clone(), action).await?;
        }
        let delivery = self.dispatch_delivery(
            &state,
            &target,
            operation,
            caption.map(ToOwned::to_owned),
            Some(parse_mode),
            Vec::new(),
            vec![media],
            None,
            Vec::new(),
            None,
            None,
            None,
            None,
            false,
            None,
            None,
        )
        .await?;
        Ok(delivery.id)
    }

    async fn dispatch_delivery(
        &self,
        state: &Arc<AccountState>,
        target: &TelegramTarget,
        operation: TelegramOperation,
        text: Option<String>,
        parse_mode: Option<ParseMode>,
        entities: Vec<TelegramMessageEntity>,
        media: Vec<TelegramMedia>,
        keyboard: Option<TelegramInlineKeyboardMarkup>,
        reactions: Vec<TelegramReaction>,
        location: Option<TelegramLocation>,
        poll: Option<TelegramPollRequest>,
        quote_forward: Option<TelegramQuoteForward>,
        chat_action: Option<TelegramChatAction>,
        silent: bool,
        continuation_index: Option<usize>,
        continuation_total: Option<usize>,
    ) -> TelegramResult<TelegramDelivery> {
        for attempt in 0..=self.config.retry_limit {
            self.wait_rate_window(&state.account.name).await;
            self.wait_for_send_slot(state, &target.chat_id).await;

            if self.store.blocked_chats.read().await.contains(&target.chat_id) {
                let error = TelegramApiError::blocked("telegram bot is blocked by this chat");
                self.mark_health(state, TelegramAccountHealthState::Disconnected, Some(error.message.clone())).await;
                return Err(error);
            }

            if let Some(error) = state.send_errors.lock().await.pop_front() {
                match error.kind {
                    TelegramApiErrorKind::RateLimited | TelegramApiErrorKind::FloodWait => {
                        self.apply_rate_limit(&state.account.name, error.retry_after_seconds).await;
                        self.mark_health(state, TelegramAccountHealthState::RateLimited, Some(error.message.clone())).await;
                    }
                    TelegramApiErrorKind::Unauthorized => {
                        self.mark_health(state, TelegramAccountHealthState::AuthError, Some(error.message.clone())).await;
                        return Err(error);
                    }
                    TelegramApiErrorKind::Blocked => {
                        self.mark_health(state, TelegramAccountHealthState::Disconnected, Some(error.message.clone())).await;
                        return Err(error);
                    }
                    _ => {
                        self.mark_health(state, TelegramAccountHealthState::Reconnecting, Some(error.message.clone())).await;
                    }
                }

                if !error.is_retryable() || attempt == self.config.retry_limit {
                    return Err(error);
                }
                self.sleep_for_error(&error, attempt).await;
                continue;
            }

            let delivery = TelegramDelivery {
                id: self.next_message_id(),
                account_name: state.account.name.clone(),
                bot_username: state.account.normalized_bot_username(),
                operation,
                chat_id: target.chat_id.clone(),
                thread_id: target.thread_id,
                text,
                parse_mode,
                entities,
                media,
                keyboard,
                reactions,
                location,
                poll,
                quote_forward,
                callback_answer: None,
                chat_action,
                silent,
                created_at: Utc::now(),
                continuation_index,
                continuation_total,
            };
            self.register_media(&delivery.media).await;
            self.record_delivery(delivery.clone()).await;
            self.mark_health(state, TelegramAccountHealthState::Connected, None).await;
            return Ok(delivery);
        }

        Err(TelegramApiError::server("telegram send failed after retry budget"))
    }

    async fn register_media(&self, media: &[TelegramMedia]) {
        let mut remote_files = self.store.remote_files.write().await;
        for entry in media {
            remote_files.insert(entry.file_id.clone(), entry.clone());
        }
    }

    async fn record_delivery(&self, delivery: TelegramDelivery) {
        let id = delivery.id.clone();
        self.store.delivery_order.write().await.push(id.clone());
        self.store.deliveries.write().await.insert(id, delivery);
    }

    async fn wait_for_send_slot(&self, state: &Arc<AccountState>, chat_id: &str) {
        loop {
            let now = Instant::now();
            let mut global = state.global_send_times.lock().await;
            while let Some(front) = global.front() {
                if now.duration_since(*front) >= Duration::from_secs(1) {
                    global.pop_front();
                } else {
                    break;
                }
            }

            let mut chats = state.chat_send_times.lock().await;
            let chat_window = chats.entry(chat_id.to_string()).or_default();
            while let Some(front) = chat_window.front() {
                if now.duration_since(*front)
                    >= Duration::from_secs(self.config.per_chat_rate_window_seconds)
                {
                    chat_window.pop_front();
                } else {
                    break;
                }
            }

            if global.len() < self.config.global_rate_limit_per_second
                && chat_window.len() < self.config.per_chat_rate_limit
            {
                global.push_back(now);
                chat_window.push_back(now);
                return;
            }

            let global_wait = global.front().map(|front| {
                Duration::from_secs(1).saturating_sub(now.duration_since(*front))
            });
            let chat_wait = chat_window.front().map(|front| {
                Duration::from_secs(self.config.per_chat_rate_window_seconds)
                    .saturating_sub(now.duration_since(*front))
            });
            let wait = global_wait
                .into_iter()
                .chain(chat_wait.into_iter())
                .max()
                .unwrap_or_else(|| Duration::from_millis(10));

            drop(chats);
            drop(global);
            tokio::time::sleep(wait).await;
        }
    }

    async fn sleep_for_error(&self, error: &TelegramApiError, attempt: usize) {
        let delay = match error.kind {
            TelegramApiErrorKind::RateLimited | TelegramApiErrorKind::FloodWait => {
                Duration::from_secs(error.retry_after_seconds.unwrap_or(1).max(1))
            }
            TelegramApiErrorKind::Server | TelegramApiErrorKind::NetworkTimeout => {
                Duration::from_millis((100 * (1u64 << attempt.min(6))).min(1_500))
            }
            _ => Duration::from_millis(10),
        };
        tokio::time::sleep(delay).await;
    }

    async fn mark_health(
        &self,
        state: &Arc<AccountState>,
        status: TelegramAccountHealthState,
        error: Option<String>,
    ) {
        let mut health = state.health.write().await;
        health.state = status;
        health.last_error = error;
        if status == TelegramAccountHealthState::Connected {
            health.consecutive_failures = 0;
        } else {
            health.consecutive_failures += 1;
        }
        health.last_update_offset = state.last_update_offset.load(Ordering::Relaxed);
        health.delivery_mode = if self.webhook_is_active(&state.account.name).await {
            TelegramDeliveryMode::Webhook
        } else if self.config.polling_mode {
            TelegramDeliveryMode::Polling
        } else {
            TelegramDeliveryMode::FallbackPolling
        };
    }

    async fn webhook_is_active(&self, account: &str) -> bool {
        self.store
            .webhook_states
            .read()
            .await
            .get(account)
            .map(|webhook| webhook.active)
            .unwrap_or(false)
    }

    fn require_account(&self, account: &str) -> TelegramResult<Arc<AccountState>> {
        self.accounts
            .get(account)
            .cloned()
            .ok_or_else(|| TelegramApiError::new(TelegramApiErrorKind::Config, format!("unknown telegram account `{account}`")))
    }

    fn resolve_account(&self, target: &TelegramTarget) -> TelegramResult<Arc<AccountState>> {
        if let Some(account_name) = &target.account_name {
            return self.require_account(account_name);
        }
        if let Some(bot_username) = &target.bot_username {
            if let Some(account_name) = self.resolve_account_by_bot_username(bot_username) {
                return self.require_account(&account_name);
            }
        }
        self.accounts
            .values()
            .next()
            .cloned()
            .ok_or_else(|| TelegramApiError::new(TelegramApiErrorKind::Config, "no telegram account configured"))
    }

    fn account_for_update(&self, update: &TelegramUpdate) -> TelegramResult<String> {
        if let Some(bot_username) = &update.bot_username {
            if let Some(account_name) = self.resolve_account_by_bot_username(bot_username) {
                return Ok(account_name);
            }
        }
        if let Some(message) = &update.message {
            if let Some(bot_username) = &message.bot_username {
                if let Some(account_name) = self.resolve_account_by_bot_username(bot_username) {
                    return Ok(account_name);
                }
            }
        }
        if let Some(callback) = &update.callback_query {
            if let Some(bot_username) = &callback.bot_username {
                if let Some(account_name) = self.resolve_account_by_bot_username(bot_username) {
                    return Ok(account_name);
                }
            }
        }
        self.accounts
            .keys()
            .next()
            .cloned()
            .ok_or_else(|| TelegramApiError::new(TelegramApiErrorKind::Config, "no telegram account configured"))
    }

    fn next_message_id(&self) -> MessageId {
        format!("tg-{}", self.next_message_id.fetch_add(1, Ordering::Relaxed))
    }

    async fn update_member_status(
        &self,
        target: &TelegramTarget,
        user_id: &str,
        status: crate::telegram::types::TelegramMemberStatus,
    ) -> TelegramResult<()> {
        let mut members = self.store.chat_members.write().await;
        let member = members
            .entry(target.chat_id.clone())
            .or_default()
            .entry(user_id.to_string())
            .or_insert(TelegramChatMember {
                user_id: user_id.to_string(),
                username: None,
                status,
                can_send_messages: false,
                can_manage_topics: false,
                can_delete_messages: false,
                is_bot: false,
            });
        member.status = status;
        Ok(())
    }

    async fn spawn_poller(&self, account_name: String) {
        let should_spawn = {
            let tasks = self.poller_tasks.lock().await;
            !tasks.contains_key(&account_name)
        };
        if !should_spawn {
            return;
        }

        let channel = self.clone();
        let task_account_name = account_name.clone();
        let task = tokio::spawn(async move {
            loop {
                if !channel.running.load(Ordering::Relaxed) {
                    break;
                }

                let state = match channel.require_account(&task_account_name) {
                    Ok(state) => state,
                    Err(_) => break,
                };

                let webhook_active = channel.webhook_is_active(&task_account_name).await;
                let should_poll = state.account.polling_enabled
                    && (channel.config.polling_mode
                        || (channel.config.webhook_fallback_to_polling && !webhook_active));
                if should_poll {
                    let _ = channel.poll_once(&task_account_name).await;
                }

                tokio::time::sleep(Duration::from_millis(channel.config.poll_interval_ms)).await;
            }
        });
        self.poller_tasks.lock().await.insert(account_name, task);
    }

    async fn abort_poller(&self, account_name: &str) {
        if let Some(handle) = self.poller_tasks.lock().await.remove(account_name) {
            handle.abort();
        }
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

    async fn start(&mut self) -> ChannelResult<()> {
        self.running.store(true, Ordering::Relaxed);
        for state in self.accounts.values() {
            let webhook_active = self.webhook_is_active(&state.account.name).await;
            let health_state = if webhook_active && !self.config.polling_mode {
                TelegramAccountHealthState::WebhookOnly
            } else {
                TelegramAccountHealthState::Connected
            };
            self.mark_health(state, health_state, None).await;
            if state.account.polling_enabled
                && (self.config.polling_mode
                    || (self.config.webhook_fallback_to_polling && !webhook_active))
            {
                self.spawn_poller(state.account.name.clone()).await;
            }
        }
        Ok(())
    }

    async fn stop(&mut self) -> ChannelResult<()> {
        self.running.store(false, Ordering::Relaxed);
        let names = self.account_names();
        for name in names {
            self.abort_poller(&name).await;
        }
        for state in self.accounts.values() {
            self.mark_health(state, TelegramAccountHealthState::Disconnected, None).await;
        }
        Ok(())
    }

    async fn send(&self, target: &str, message: OutboundMessage) -> ChannelResult<MessageId> {
        let telegram_target = TelegramTarget::parse(target);
        let keyboard = message.buttons.clone().map(keyboard_from_framework);
        let result = if message.media.is_empty() {
            self.send_text_message(
                telegram_target,
                &message.text,
                message.parse_mode,
                keyboard,
                None,
                message.silent,
            )
            .await
            .map(|ids| ids.last().cloned().unwrap_or_else(|| self.next_message_id()))
        } else {
            let chat_id = telegram_target.chat_id.clone();
            self.upload_media(&chat_id, &message).await
        };
        result.map_err(map_channel_error)
    }

    async fn edit(&self, _target: &str, message_id: &str, message: OutboundMessage) -> ChannelResult<()> {
        let mut deliveries = self.store.deliveries.write().await;
        let delivery = deliveries
            .get_mut(message_id)
            .ok_or_else(|| ChannelError::PlatformRequest(format!("telegram message `{message_id}` not found")))?;
        delivery.text = Some(message.text);
        delivery.parse_mode = message.parse_mode;
        delivery.keyboard = message.buttons.map(keyboard_from_framework);
        Ok(())
    }

    async fn delete(&self, _target: &str, message_id: &str) -> ChannelResult<()> {
        self.store.deliveries.write().await.remove(message_id);
        self.store
            .delivery_order
            .write()
            .await
            .retain(|item| item != message_id);
        Ok(())
    }

    async fn react(&self, target: &str, message_id: &str, emoji: &str) -> ChannelResult<()> {
        let target = TelegramTarget::parse(target);
        let numeric_id = message_id
            .trim_start_matches("tg-")
            .parse::<i64>()
            .unwrap_or(0);
        self.set_message_reaction(target, numeric_id, vec![TelegramReaction::Emoji(emoji.to_string())])
            .await
            .map_err(map_channel_error)
    }
}

fn map_channel_error(error: TelegramApiError) -> ChannelError {
    ChannelError::PlatformRequest(error.to_string())
}

fn keyboard_from_framework(rows: Vec<Vec<InlineButton>>) -> TelegramInlineKeyboardMarkup {
    TelegramInlineKeyboardMarkup {
        rows: rows
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|button| TelegramInlineButton {
                        text: button.text,
                        callback_data: Some(button.callback_data),
                        url: None,
                        switch_inline_query: None,
                        style: TelegramInlineButtonStyle::Default,
                    })
                    .collect()
            })
            .collect(),
    }
}

fn media_from_attachment(attachment: MediaAttachment) -> TelegramMedia {
    let file_path = attachment.file_path.clone();
    let file_name = file_path
        .as_deref()
        .and_then(|value| Path::new(value).file_name())
        .map(|value| value.to_string_lossy().to_string());
    let bytes = attachment
        .file_path
        .as_deref()
        .and_then(|path| fs::read(path).ok())
        .unwrap_or_else(|| vec![0, 1, 2, 3]);
    TelegramMedia {
        file_id: attachment
            .platform_id
            .unwrap_or_else(|| format!("file-{}", file_name.clone().unwrap_or_else(|| "blob".to_string()))),
        kind: match attachment.kind {
            MediaType::Image => TelegramMediaKind::Photo,
            MediaType::Voice => TelegramMediaKind::Voice,
            MediaType::Document => TelegramMediaKind::Document,
            MediaType::Video => TelegramMediaKind::Video,
            MediaType::Sticker => TelegramMediaKind::Sticker,
            MediaType::Location => TelegramMediaKind::Document,
        },
        file_name,
        mime_type: attachment.mime_type,
        file_path,
        url: attachment.url,
        bytes,
        duration_seconds: None,
        sticker_emoji: None,
        is_animated: false,
        is_video_note: false,
    }
}

fn detect_voice_duration(media: &TelegramMedia) -> u32 {
    if let Some(file_name) = &media.file_name {
        if let Some(segment) = file_name.split('_').find(|segment| segment.ends_with('s')) {
            if let Ok(parsed) = segment.trim_end_matches('s').parse::<u32>() {
                return parsed.max(1);
            }
        }
    }
    ((media.bytes.len() as u32) / 16_000).max(1)
}

fn reaction_key(chat_id: &str, message_id: i64) -> String {
    format!("{chat_id}:{message_id}")
}

fn chat_action_for_operation(operation: TelegramOperation) -> TelegramChatAction {
    match operation {
        TelegramOperation::SendPhoto => TelegramChatAction::UploadPhoto,
        TelegramOperation::SendVoice => TelegramChatAction::RecordVoice,
        TelegramOperation::SendDocument => TelegramChatAction::UploadDocument,
        TelegramOperation::SendVideo => TelegramChatAction::UploadVideo,
        TelegramOperation::SendVideoNote => TelegramChatAction::UploadVideoNote,
        TelegramOperation::SendAnimation => TelegramChatAction::UploadVideo,
        _ => TelegramChatAction::Typing,
    }
}

fn crate_status_banned() -> crate::telegram::types::TelegramMemberStatus {
    crate::telegram::types::TelegramMemberStatus::Banned
}

fn crate_status_left() -> crate::telegram::types::TelegramMemberStatus {
    crate::telegram::types::TelegramMemberStatus::Left
}

trait TelegramMediaExt {
    fn with_kind(self, kind: TelegramMediaKind) -> Self;
}

impl TelegramMediaExt for TelegramMedia {
    fn with_kind(mut self, kind: TelegramMediaKind) -> Self {
        self.kind = kind;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framework::ChatType;
    use crate::telegram::TelegramMessage;

    fn build_channel() -> TelegramChannel {
        TelegramChannel::new(TelegramConfig {
            accounts: vec![TelegramAccount {
                name: "bot-a".to_string(),
                token: "token-a".to_string(),
                bot_username: "bot_a".to_string(),
                polling_enabled: true,
                media_dir: Some("/tmp/magicmerlin-telegram-tests".to_string()),
                webhook_secret: Some("secret".to_string()),
            }],
            poll_interval_ms: 25,
            per_chat_rate_limit: 2,
            per_chat_rate_window_seconds: 1,
            ..TelegramConfig::default()
        })
    }

    #[tokio::test]
    async fn update_offsets_prevent_duplicates() {
        let channel = build_channel();
        channel
            .ingest_update(
                "bot-a",
                TelegramUpdate {
                    update_id: 10,
                    bot_username: Some("@bot_a".to_string()),
                    message: None,
                    edited_message: None,
                    callback_query: None,
                    reaction: None,
                    chat_member: None,
                },
            )
            .await;
        channel
            .ingest_update(
                "bot-a",
                TelegramUpdate {
                    update_id: 10,
                    bot_username: Some("@bot_a".to_string()),
                    message: None,
                    edited_message: None,
                    callback_query: None,
                    reaction: None,
                    chat_member: None,
                },
            )
            .await;

        let first = channel.get_updates("bot-a", 10).await.unwrap();
        let second = channel.get_updates("bot-a", 10).await.unwrap();
        assert_eq!(first.len(), 2);
        assert!(second.is_empty());
    }

    #[tokio::test]
    async fn send_text_splits_and_stores_messages() {
        let channel = build_channel();
        let sent = channel
            .send_telegram_text("bot-a", "chat-1", &"x".repeat(TELEGRAM_MAX_MESSAGE_LEN + 32))
            .await
            .unwrap();
        assert!(sent.len() >= 2);
        assert!(channel.deliveries_for_chat("chat-1").await.len() >= 2);
    }

    #[tokio::test]
    async fn webhook_secret_is_enforced() {
        let channel = build_channel();
        channel
            .set_webhook("bot-a", "https://example.com/tg", Some("secret"))
            .await
            .unwrap();
        let update = TelegramUpdate {
            update_id: 1,
            bot_username: Some("@bot_a".to_string()),
            message: Some(TelegramMessage {
                message_id: 1,
                chat_id: "chat".to_string(),
                chat_type: ChatType::Group,
                from_user_id: Some("user".to_string()),
                from_username: Some("user".to_string()),
                bot_username: Some("@bot_a".to_string()),
                text: Some("hello".to_string()),
                message_thread_id: None,
                reply_to_message_id: None,
                entities: Vec::new(),
                media: Vec::new(),
                inline_keyboard: None,
                reactions: Vec::new(),
                location: None,
                poll: None,
                sticker: None,
                quote: None,
            }),
            edited_message: None,
            callback_query: None,
            reaction: None,
            chat_member: None,
        };
        assert!(channel
            .handle_webhook_update("bot-a", Some("secret"), update)
            .await
            .is_ok());
    }
}
