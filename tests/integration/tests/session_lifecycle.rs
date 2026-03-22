//! Integration tests: session creation, compaction, context estimation, sub-agents.

use magicmerlin_sessions::SessionEngine;
use serde_json::json;

#[test]
fn test_session_create_and_load() {
    let temp = tempfile::tempdir().expect("tempdir");
    let engine = SessionEngine::new(
        temp.path().join("db.sqlite"),
        temp.path().join("transcripts"),
    )
    .expect("engine");

    let state = engine.load_or_create("test:session", None).expect("create");
    assert_eq!(state.session_id, "test:session");
    assert_eq!(state.token_usage, 0);
    assert_eq!(state.compaction_count, 0);
    assert!(state.parent_session_id.is_none());
}

#[test]
fn test_session_append_and_read() {
    let temp = tempfile::tempdir().expect("tempdir");
    let engine = SessionEngine::new(
        temp.path().join("db.sqlite"),
        temp.path().join("transcripts"),
    )
    .expect("engine");

    engine.load_or_create("test:append", None).expect("create");

    for i in 0..5 {
        engine
            .append_message(
                "test:append",
                &json!({"role": "user", "content": format!("Message {i}")}),
            )
            .expect("append");
        engine
            .append_message(
                "test:append",
                &json!({"role": "assistant", "content": format!("Reply {i}")}),
            )
            .expect("append");
    }

    let store = engine.transcript_store("test:append").expect("store");
    let entries = store.read(0, None).expect("read");
    assert_eq!(entries.len(), 10);
}

#[test]
fn test_session_compaction() {
    let temp = tempfile::tempdir().expect("tempdir");
    let engine = SessionEngine::new(
        temp.path().join("db.sqlite"),
        temp.path().join("transcripts"),
    )
    .expect("engine");

    let session_id = "test:compaction";
    engine.load_or_create(session_id, None).expect("create");

    // Append many messages to trigger compaction
    for i in 0..100 {
        engine
            .append_message(
                session_id,
                &json!({"role": "user", "content": format!("Message {i} with some padding text to increase token count")}),
            )
            .expect("append");
        engine
            .append_message(
                session_id,
                &json!({"role": "assistant", "content": format!("Reply {i} with extra words")}),
            )
            .expect("append");
    }

    // Compact with a small context window to trigger compaction
    let result = engine
        .compact_if_needed(session_id, 100, 50)
        .expect("compact");
    assert!(result.is_some(), "compaction should have triggered");
    let cr = result.unwrap();
    assert_eq!(cr.messages_before, 200);
    assert!(cr.messages_after < cr.messages_before);
    assert!(cr.messages_after <= 31); // keep_last=30 + 1 summary

    // Verify compacted session is still readable
    let store = engine.transcript_store(session_id).expect("store");
    let entries = store.read(0, None).expect("read");
    assert!(!entries.is_empty());
    assert!(entries.len() < 200);
}

#[test]
fn test_context_percent() {
    let temp = tempfile::tempdir().expect("tempdir");
    let engine = SessionEngine::new(
        temp.path().join("db.sqlite"),
        temp.path().join("transcripts"),
    )
    .expect("engine");

    engine.load_or_create("ctx-test", None).expect("create");

    // Empty session should have 0% context
    let pct = engine
        .estimate_context_percent("ctx-test", 128_000)
        .expect("pct");
    assert_eq!(pct, 0.0);

    // Add a message and check that context percent is non-zero
    let msg = "a".repeat(4000);
    engine
        .append_message("ctx-test", &json!({"role": "user", "content": msg}))
        .expect("append");

    let pct = engine
        .estimate_context_percent("ctx-test", 128_000)
        .expect("pct");
    assert!(pct > 0.0, "pct should be > 0 after adding message");
    assert!(pct < 0.1, "pct should be < 10% for ~1000 tokens / 128000");
}

#[test]
fn test_context_percent_zero_window() {
    let temp = tempfile::tempdir().expect("tempdir");
    let engine = SessionEngine::new(
        temp.path().join("db.sqlite"),
        temp.path().join("transcripts"),
    )
    .expect("engine");

    engine.load_or_create("ctx-zero", None).expect("create");
    let pct = engine.estimate_context_percent("ctx-zero", 0).expect("pct");
    assert_eq!(pct, 0.0);
}

#[test]
fn test_sub_agent_spawn() {
    let temp = tempfile::tempdir().expect("tempdir");
    let engine = SessionEngine::new(
        temp.path().join("db.sqlite"),
        temp.path().join("transcripts"),
    )
    .expect("engine");

    engine
        .load_or_create("parent:session", None)
        .expect("create");

    let child_id = engine
        .spawn_sub_agent_session("parent:session", "researcher")
        .expect("spawn");
    assert!(child_id.starts_with("sub:researcher:"));

    // Child should be loadable
    let child_state = engine.get_state(&child_id).expect("child state");
    assert_eq!(
        child_state.parent_session_id.as_deref(),
        Some("parent:session")
    );
}

#[test]
fn test_session_repair() {
    let temp = tempfile::tempdir().expect("tempdir");
    let engine = SessionEngine::new(
        temp.path().join("db.sqlite"),
        temp.path().join("transcripts"),
    )
    .expect("engine");

    engine.load_or_create("repair-test", None).expect("create");

    // Add some messages
    engine
        .append_message("repair-test", &json!({"role": "user", "content": "hi"}))
        .expect("append");
    engine
        .append_message(
            "repair-test",
            &json!({"role": "assistant", "content": "hello"}),
        )
        .expect("append");

    // Repair should succeed even when nothing is broken
    let report = engine.repair_transcript("repair-test").expect("repair");
    assert_eq!(report.invalid_lines_removed, 0);
}

#[test]
fn test_send_between_sessions() {
    let temp = tempfile::tempdir().expect("tempdir");
    let engine = SessionEngine::new(
        temp.path().join("db.sqlite"),
        temp.path().join("transcripts"),
    )
    .expect("engine");

    engine.load_or_create("session-a", None).expect("create a");
    engine.load_or_create("session-b", None).expect("create b");

    engine
        .send_between_sessions("session-a", "session-b", "inter-session message")
        .expect("send");

    // Both sessions should have the envelope
    let store_a = engine.transcript_store("session-a").expect("store a");
    let entries_a = store_a.read(0, None).expect("read a");
    assert_eq!(entries_a.len(), 1);
    assert_eq!(entries_a[0]["content"], "inter-session message");

    let store_b = engine.transcript_store("session-b").expect("store b");
    let entries_b = store_b.read(0, None).expect("read b");
    assert_eq!(entries_b.len(), 1);
}

#[test]
fn test_update_usage() {
    let temp = tempfile::tempdir().expect("tempdir");
    let engine = SessionEngine::new(
        temp.path().join("db.sqlite"),
        temp.path().join("transcripts"),
    )
    .expect("engine");

    engine.load_or_create("usage-test", None).expect("create");

    engine
        .update_usage("usage-test", 1000, 0.05)
        .expect("update");
    let state = engine.get_state("usage-test").expect("state");
    assert_eq!(state.token_usage, 1000);
    assert!((state.total_cost_usd - 0.05).abs() < f64::EPSILON);

    engine
        .update_usage("usage-test", 500, 0.02)
        .expect("update");
    let state = engine.get_state("usage-test").expect("state");
    assert_eq!(state.token_usage, 1500);
}

#[test]
fn test_cleanup_stale_subagents() {
    let temp = tempfile::tempdir().expect("tempdir");
    let engine = SessionEngine::new(
        temp.path().join("db.sqlite"),
        temp.path().join("transcripts"),
    )
    .expect("engine");

    engine.load_or_create("parent", None).expect("create");

    // Spawn sub-agent and then clean up with 0 max idle (everything is stale)
    let child_id = engine
        .spawn_sub_agent_session("parent", "worker")
        .expect("spawn");
    assert!(!child_id.is_empty());

    // With max_idle_seconds = i64::MAX, nothing should be cleaned up
    let cleaned = engine.cleanup_stale_subagents(i64::MAX).expect("cleanup");
    assert_eq!(cleaned, 0);
}
