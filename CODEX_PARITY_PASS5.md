# Parity Pass 5: Telegram Deepening — Full Production Implementation

Current channels crate: 11.5K lines. Telegram needs to be production-ready for OpenClaw parity.

## Deepen channels/src/telegram/

### 1. Multi-Account Support
- Load all accounts from config (`channels.telegram.accounts.*`)
- Route messages by botUsername to correct account
- Concurrent polling across all accounts (tokio::spawn per account)
- Account health tracking (per-account connection status)

### 2. getUpdates Polling
- Long polling with offset (don't reprocess old updates)
- Handle getUpdates errors (429 → backoff, 500 → retry, 401 → auth error)
- Parallel update processing (process multiple updates concurrently)
- Update frequency: 0.5s polling interval, configurable

### 3. Media Handling (Full)
- **Download**: getFile → downloadFile to local media dir, return path
- **Upload**: sendPhoto, sendVoice (OGG Opus), sendDocument, sendVideoNote, sendVideo
- **Sticker support**: sendSticker, parse sticker metadata
- **Animation**: sendAnimation (GIF/MP4)
- **Location**: sendLocation with live location
- **Poll**: sendPoll (quiz/regular, anonymous/public)
- **Voice**: sendVoice (OGG Opus, duration detection)

### 4. Inline Keyboards
- Parse callback_data from callback_query updates
- AnswerCallbackQuery (show alert or URL)
- InlineKeyboardMarkup: multiple rows, callback_data, url, switch_inline_query
- Inline keyboard button styles (primary/success/danger)

### 5. Reactions
- setMessageReaction API (emoji, custom emoji)
- Reaction count parsing
- Reaction handling (reaction updates)

### 6. Chat Actions
- sendChatAction (typing, upload_photo, record_voice)
- Auto-send typing indicator on agent turn start

### 7. Advanced Formatting
- **MarkdownV2**: Parse entities, escape special chars (| * _ ` [ ])
- **HTML**: Parse <b>, <i>, <u>, <s>, <code>, <pre>, <a>
- **Message splitting**: Auto-split long messages with continuation markers
- **Quote forwarding**: ForwardMessage with quote

### 8. Group/Forum Support
- MessageThreadId for forum topics
- Forum topic creation
- Group member management (getChatMember, ban/kick)
- Bot permissions check

### 9. Webhooks (Optional)
- Webhook setup/teardown (setWebhook/deleteWebhook)
- Webhook server (axum endpoint for setWebhook)
- Webhook-to-polling fallback

### 10. Error Handling
- Rate limit backoff (30 messages/sec, 20 messages/min per chat)
- Flood wait handling (wait N seconds)
- Bot blocked user handling
- Network timeout retries

## Tests
- Multi-account concurrent polling
- Media download/upload roundtrip
- Inline keyboard callback cycle
- Long message splitting
- MarkdownV2/HTML parsing + entity preservation
- Rate limit backoff simulation
- Forum topic threading

Target: **channels crate → 20K lines**. Commit when done.

When finished:
openclaw system event --text "Parity Pass 5 complete: Telegram fully production-ready (20K lines)" --mode now
