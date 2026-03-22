use magicmerlin_auto_reply::{parse_slash_command, SlashCommand};
use magicmerlin_channels::framework::{
    format_for_platform, platform_message_limit, split_for_platform, split_text_by_limit,
    OutboundMessage, ParseMode, Platform,
};

// ── Helper to build a simple OutboundMessage ──

fn outbound(text: &str, parse_mode: Option<ParseMode>) -> OutboundMessage {
    OutboundMessage {
        text: text.to_string(),
        reply_to: None,
        media: Vec::new(),
        buttons: None,
        silent: false,
        parse_mode,
    }
}

// ── Platform limits ──

#[test]
fn test_telegram_limit() {
    assert_eq!(platform_message_limit(Platform::Telegram), 4096);
}

#[test]
fn test_discord_limit() {
    assert_eq!(platform_message_limit(Platform::Discord), 2000);
}

#[test]
fn test_whatsapp_limit() {
    assert_eq!(platform_message_limit(Platform::WhatsApp), 65536);
}

#[test]
fn test_signal_limit() {
    assert_eq!(platform_message_limit(Platform::Signal), 4096);
}

#[test]
fn test_slack_limit() {
    assert_eq!(platform_message_limit(Platform::Slack), 4000);
}

#[test]
fn test_imessage_limit() {
    assert_eq!(platform_message_limit(Platform::IMessage), 4096);
}

#[test]
fn test_line_limit() {
    assert_eq!(platform_message_limit(Platform::Line), 5000);
}

#[test]
fn test_web_limit() {
    assert_eq!(platform_message_limit(Platform::Web), 8192);
}

// ── Formatting ──

#[test]
fn test_telegram_markdown_escaping() {
    let msg = outbound("Hello **world** and `code`", Some(ParseMode::Markdown));
    let formatted = format_for_platform(Platform::Telegram, &msg);
    assert!(
        formatted.contains("\\*"),
        "Should escape asterisks for Telegram MarkdownV2, got: {formatted}"
    );
    assert!(
        formatted.contains("\\`"),
        "Should escape backticks for Telegram MarkdownV2, got: {formatted}"
    );
}

#[test]
fn test_telegram_markdown_escapes_special_chars() {
    let msg = outbound(
        "a_b [c] (d) ~e >f #g +h -i =j |k {l} .m !n",
        Some(ParseMode::Markdown),
    );
    let formatted = format_for_platform(Platform::Telegram, &msg);
    assert!(formatted.contains("\\_"), "Should escape underscore");
    assert!(formatted.contains("\\["), "Should escape open bracket");
    assert!(formatted.contains("\\]"), "Should escape close bracket");
    assert!(formatted.contains("\\("), "Should escape open paren");
    assert!(formatted.contains("\\)"), "Should escape close paren");
    assert!(formatted.contains("\\~"), "Should escape tilde");
    assert!(formatted.contains("\\>"), "Should escape greater-than");
    assert!(formatted.contains("\\#"), "Should escape hash");
    assert!(formatted.contains("\\+"), "Should escape plus");
    assert!(formatted.contains("\\-"), "Should escape minus");
    assert!(formatted.contains("\\="), "Should escape equals");
    assert!(formatted.contains("\\|"), "Should escape pipe");
    assert!(formatted.contains("\\{"), "Should escape open brace");
    assert!(formatted.contains("\\}"), "Should escape close brace");
    assert!(formatted.contains("\\."), "Should escape period");
    assert!(formatted.contains("\\!"), "Should escape exclamation");
}

#[test]
fn test_plain_text_passthrough() {
    let msg = outbound("Hello **world**", Some(ParseMode::Plain));
    let formatted = format_for_platform(Platform::Telegram, &msg);
    assert_eq!(formatted, "Hello **world**");
}

#[test]
fn test_plain_text_passthrough_discord() {
    let msg = outbound("Hello **world**", Some(ParseMode::Plain));
    let formatted = format_for_platform(Platform::Discord, &msg);
    assert_eq!(formatted, "Hello **world**");
}

#[test]
fn test_html_strip_tags() {
    let msg = outbound(
        "<nav>skip</nav><main><h1>Title</h1><p>Content</p></main>",
        Some(ParseMode::Html),
    );
    let formatted = format_for_platform(Platform::Telegram, &msg);
    assert!(formatted.contains("Title"), "Should contain Title");
    assert!(formatted.contains("Content"), "Should contain Content");
    assert!(!formatted.contains("<h1>"), "Should strip h1 tags");
    assert!(!formatted.contains("<nav>"), "Should strip nav tags");
}

#[test]
fn test_html_strip_on_discord() {
    let msg = outbound("<b>bold</b> text", Some(ParseMode::Html));
    let formatted = format_for_platform(Platform::Discord, &msg);
    assert!(formatted.contains("bold"), "Should contain bold text");
    assert!(!formatted.contains("<b>"), "Should strip b tags");
}

#[test]
fn test_no_parse_mode_defaults_to_plain() {
    let msg = outbound("Hello **world**", None);
    let formatted = format_for_platform(Platform::Telegram, &msg);
    assert_eq!(formatted, "Hello **world**");
}

// ── Splitting ──

#[test]
fn test_split_short_text() {
    let chunks = split_text_by_limit("hello world", 100);
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0], "hello world");
}

#[test]
fn test_split_long_text() {
    let text = "word ".repeat(1000);
    let chunks = split_text_by_limit(&text, 100);
    assert!(chunks.len() > 1);
    for chunk in &chunks {
        assert!(chunk.len() <= 100, "Chunk too long: {}", chunk.len());
    }
}

#[test]
fn test_split_empty_text() {
    let chunks = split_text_by_limit("", 100);
    assert!(chunks.is_empty());
}

#[test]
fn test_split_exact_limit() {
    let text = "abcde";
    let chunks = split_text_by_limit(text, 5);
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0], "abcde");
}

#[test]
fn test_split_for_discord() {
    let text = "a ".repeat(2000);
    let msg = outbound(&text, None);
    let chunks = split_for_platform(Platform::Discord, &msg);
    for chunk in &chunks {
        assert!(chunk.len() <= 2000);
    }
}

#[test]
fn test_split_for_telegram() {
    let text = "word ".repeat(2000);
    let msg = outbound(&text, None);
    let chunks = split_for_platform(Platform::Telegram, &msg);
    for chunk in &chunks {
        assert!(
            chunk.len() <= 4096,
            "Telegram chunk should be <= 4096, got {}",
            chunk.len()
        );
    }
}

#[test]
fn test_split_preserves_all_content() {
    let text = "one two three four five six seven eight nine ten";
    let chunks = split_text_by_limit(text, 15);
    let rejoined = chunks.join(" ");
    // All words should be present
    for word in text.split_whitespace() {
        assert!(
            rejoined.contains(word),
            "Missing word '{word}' in split result"
        );
    }
}

// ── Slash command parsing ──

#[test]
fn test_slash_status() {
    assert!(matches!(
        parse_slash_command("/status"),
        Some(SlashCommand::Status)
    ));
}

#[test]
fn test_slash_version() {
    assert!(matches!(
        parse_slash_command("/version"),
        Some(SlashCommand::Version)
    ));
}

#[test]
fn test_slash_help_no_topic() {
    match parse_slash_command("/help") {
        Some(SlashCommand::Help { topic }) => assert!(topic.is_none()),
        other => panic!("Expected Help, got {other:?}"),
    }
}

#[test]
fn test_slash_help_with_topic() {
    match parse_slash_command("/help cron") {
        Some(SlashCommand::Help { topic }) => assert_eq!(topic.unwrap(), "cron"),
        other => panic!("Expected Help with topic, got {other:?}"),
    }
}

#[test]
fn test_slash_model_set() {
    match parse_slash_command("/model sonnet") {
        Some(SlashCommand::Model { name }) => assert_eq!(name.unwrap(), "sonnet"),
        other => panic!("Expected Model, got {other:?}"),
    }
}

#[test]
fn test_slash_model_show() {
    match parse_slash_command("/model") {
        Some(SlashCommand::Model { name }) => assert!(name.is_none()),
        other => panic!("Expected Model, got {other:?}"),
    }
}

#[test]
fn test_slash_approve() {
    match parse_slash_command("/approve abc123 allow-always") {
        Some(SlashCommand::Approve { code, mode }) => {
            assert_eq!(code, "abc123");
            assert_eq!(mode.unwrap(), "allow-always");
        }
        other => panic!("Expected Approve, got {other:?}"),
    }
}

#[test]
fn test_slash_approve_no_mode() {
    match parse_slash_command("/approve xyz789") {
        Some(SlashCommand::Approve { code, mode }) => {
            assert_eq!(code, "xyz789");
            assert!(mode.is_none());
        }
        other => panic!("Expected Approve, got {other:?}"),
    }
}

#[test]
fn test_slash_approve_needs_code() {
    // /approve with no code should return None (the ? in the parser)
    assert!(parse_slash_command("/approve").is_none());
}

#[test]
fn test_slash_compact() {
    assert!(matches!(
        parse_slash_command("/compact"),
        Some(SlashCommand::Compact)
    ));
}

#[test]
fn test_slash_sessions() {
    assert!(matches!(
        parse_slash_command("/sessions"),
        Some(SlashCommand::Sessions)
    ));
}

#[test]
fn test_slash_ping() {
    assert!(matches!(
        parse_slash_command("/ping"),
        Some(SlashCommand::Ping)
    ));
}

#[test]
fn test_slash_agents() {
    assert!(matches!(
        parse_slash_command("/agents"),
        Some(SlashCommand::Agents)
    ));
}

#[test]
fn test_slash_spawn() {
    match parse_slash_command("/spawn write a poem") {
        Some(SlashCommand::Spawn { prompt }) => assert_eq!(prompt, "write a poem"),
        other => panic!("Expected Spawn, got {other:?}"),
    }
}

#[test]
fn test_slash_spawn_needs_prompt() {
    assert!(parse_slash_command("/spawn").is_none());
}

#[test]
fn test_slash_kill() {
    match parse_slash_command("/kill session-123") {
        Some(SlashCommand::Kill { session }) => assert_eq!(session, "session-123"),
        other => panic!("Expected Kill, got {other:?}"),
    }
}

#[test]
fn test_slash_kill_needs_session() {
    assert!(parse_slash_command("/kill").is_none());
}

#[test]
fn test_not_a_slash_command() {
    assert!(parse_slash_command("hello world").is_none());
}

#[test]
fn test_unknown_slash_command() {
    match parse_slash_command("/foobar arg1 arg2") {
        Some(SlashCommand::Unknown { name, args }) => {
            assert_eq!(name, "foobar");
            assert_eq!(args, vec!["arg1", "arg2"]);
        }
        other => panic!("Expected Unknown, got {other:?}"),
    }
}

#[test]
fn test_slash_cron() {
    match parse_slash_command("/cron list") {
        Some(SlashCommand::Cron { action, args }) => {
            assert_eq!(action.unwrap(), "list");
            assert!(args.is_empty());
        }
        other => panic!("Expected Cron, got {other:?}"),
    }
}

#[test]
fn test_slash_cron_with_args() {
    match parse_slash_command("/cron add every 5m") {
        Some(SlashCommand::Cron { action, args }) => {
            assert_eq!(action.unwrap(), "add");
            assert_eq!(args, vec!["every", "5m"]);
        }
        other => panic!("Expected Cron with args, got {other:?}"),
    }
}

#[test]
fn test_slash_cron_no_action() {
    match parse_slash_command("/cron") {
        Some(SlashCommand::Cron { action, args }) => {
            assert!(action.is_none());
            assert!(args.is_empty());
        }
        other => panic!("Expected Cron with no action, got {other:?}"),
    }
}

#[test]
fn test_slash_config() {
    match parse_slash_command("/config model gpt-4") {
        Some(SlashCommand::Config { key, value }) => {
            assert_eq!(key.unwrap(), "model");
            assert_eq!(value.unwrap(), "gpt-4");
        }
        other => panic!("Expected Config, got {other:?}"),
    }
}

#[test]
fn test_slash_config_no_args() {
    match parse_slash_command("/config") {
        Some(SlashCommand::Config { key, value }) => {
            assert!(key.is_none());
            assert!(value.is_none());
        }
        other => panic!("Expected Config with no args, got {other:?}"),
    }
}

#[test]
fn test_slash_history() {
    match parse_slash_command("/history 50") {
        Some(SlashCommand::History { count }) => assert_eq!(count, Some(50)),
        other => panic!("Expected History, got {other:?}"),
    }
}

#[test]
fn test_slash_history_no_count() {
    match parse_slash_command("/history") {
        Some(SlashCommand::History { count }) => assert!(count.is_none()),
        other => panic!("Expected History with no count, got {other:?}"),
    }
}

#[test]
fn test_slash_reasoning_on() {
    match parse_slash_command("/reasoning on") {
        Some(SlashCommand::Reasoning { on }) => assert_eq!(on, Some(true)),
        other => panic!("Expected Reasoning on, got {other:?}"),
    }
}

#[test]
fn test_slash_reasoning_off() {
    match parse_slash_command("/reasoning off") {
        Some(SlashCommand::Reasoning { on }) => assert_eq!(on, Some(false)),
        other => panic!("Expected Reasoning off, got {other:?}"),
    }
}

#[test]
fn test_slash_reasoning_no_arg() {
    match parse_slash_command("/reasoning") {
        Some(SlashCommand::Reasoning { on }) => assert!(on.is_none()),
        other => panic!("Expected Reasoning with no arg, got {other:?}"),
    }
}

#[test]
fn test_slash_verbose_on() {
    match parse_slash_command("/verbose on") {
        Some(SlashCommand::Verbose { on }) => assert_eq!(on, Some(true)),
        other => panic!("Expected Verbose on, got {other:?}"),
    }
}

#[test]
fn test_slash_debug() {
    assert!(matches!(
        parse_slash_command("/debug"),
        Some(SlashCommand::Debug)
    ));
}

#[test]
fn test_slash_reset() {
    assert!(matches!(
        parse_slash_command("/reset"),
        Some(SlashCommand::Reset)
    ));
}

#[test]
fn test_slash_pause() {
    assert!(matches!(
        parse_slash_command("/pause"),
        Some(SlashCommand::Pause)
    ));
}

#[test]
fn test_slash_resume() {
    assert!(matches!(
        parse_slash_command("/resume"),
        Some(SlashCommand::Resume)
    ));
}

#[test]
fn test_slash_context() {
    assert!(matches!(
        parse_slash_command("/context"),
        Some(SlashCommand::Context)
    ));
}

#[test]
fn test_slash_cost() {
    assert!(matches!(
        parse_slash_command("/cost"),
        Some(SlashCommand::Cost)
    ));
}

#[test]
fn test_slash_whoami() {
    assert!(matches!(
        parse_slash_command("/whoami"),
        Some(SlashCommand::Whoami)
    ));
}

#[test]
fn test_slash_clear() {
    assert!(matches!(
        parse_slash_command("/clear"),
        Some(SlashCommand::Clear)
    ));
}

#[test]
fn test_slash_logs() {
    match parse_slash_command("/logs 100") {
        Some(SlashCommand::Logs { tail }) => assert_eq!(tail, Some(100)),
        other => panic!("Expected Logs, got {other:?}"),
    }
}

#[test]
fn test_slash_session_with_key() {
    match parse_slash_command("/session my-session") {
        Some(SlashCommand::Session { key }) => assert_eq!(key.unwrap(), "my-session"),
        other => panic!("Expected Session, got {other:?}"),
    }
}

#[test]
fn test_slash_memory_with_query() {
    match parse_slash_command("/memory how to deploy") {
        Some(SlashCommand::Memory { query }) => assert_eq!(query.unwrap(), "how to deploy"),
        other => panic!("Expected Memory, got {other:?}"),
    }
}

#[test]
fn test_empty_slash() {
    // Just "/" by itself
    assert!(parse_slash_command("/").is_none());
}

#[test]
fn test_whitespace_only() {
    assert!(parse_slash_command("   ").is_none());
}

#[test]
fn test_slash_feedback() {
    match parse_slash_command("/feedback great work") {
        Some(SlashCommand::Feedback { text }) => assert_eq!(text.unwrap(), "great work"),
        other => panic!("Expected Feedback, got {other:?}"),
    }
}

#[test]
fn test_slash_subscribe() {
    match parse_slash_command("/subscribe errors") {
        Some(SlashCommand::Subscribe { event }) => assert_eq!(event.unwrap(), "errors"),
        other => panic!("Expected Subscribe, got {other:?}"),
    }
}
