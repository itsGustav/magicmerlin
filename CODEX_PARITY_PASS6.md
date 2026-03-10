# Parity Pass 6: Discord Deepening — Full Production Implementation

Current goal: deepen Discord support in Magic Merlin to near OpenClaw parity.

## Rules
- Read existing `channels/src/discord/` and related framework code first.
- EXTEND existing code; do not rewrite from scratch.
- Add real implementations, not stubs.
- Add tests.
- Ensure `cargo check` and `cargo test` pass.
- Commit at the end and push to BOTH remotes: `origin` and `private`.

## Deepen Discord support

### 1. Gateway / Session Layer
- Real Discord gateway connection lifecycle:
  - Identify
  - Heartbeat
  - Resume
  - Reconnect on disconnect
  - Sequence tracking
- Track per-guild, per-channel, per-thread context.

### 2. REST API Coverage
Implement real helpers for:
- send message
- edit message
- delete message
- add/remove reactions
- create thread
- list channels
- fetch message history
- set typing indicator
- upload files/media
- embeds support

### 3. Slash Commands
- Register slash commands
- Parse inbound interaction payloads
- Support command routing for:
  - /status
  - /model
  - /compact
  - /help
- Support deferred responses and followups.

### 4. Formatting
- Discord-specific markdown formatting
- Message splitting for 2000-char limit
- Embed fallback when content too long
- Attachment + embed coexistence

### 5. Threads and Forums
- Thread creation
- Reply in existing thread
- Forum-like flows where relevant
- Session mapping per thread/channel/user

### 6. Presence / Activity
- Presence updates
- Activity text support
- Health tracking per bot connection

### 7. Permissions / Safety
- Mention gating in guilds
- Role/channel allowlists
- Respect DM/group policy equivalents for Discord

### 8. Tests
Add coverage for:
- gateway reconnect logic
- slash command parsing
- message splitting
- session mapping
- reaction calls
- thread routing

## Output target
- Deepen `channels` crate substantially
- Prefer adding multiple focused modules under `channels/src/discord/`
- Keep code production-shaped and well-documented

When complete:
1. `cargo check`
2. `cargo test`
3. `git add -A`
4. `git commit -m "feat: Parity Pass 6 — Discord deepening"`
5. `git push origin main`
6. `git push private main`
7. Run:
   `openclaw system event --text "Magic Merlin Pass 6 complete: Discord deepening finished" --mode now`
