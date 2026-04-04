//! Integration test: session compaction with memory extraction.

use magicmerlin_sessions::SessionEngine;
use magicmerlin_storage::MemoryManager;
use serde_json::json;

/// Create a session with 50 messages, trigger compaction (threshold 0%), and verify
/// the session is trimmed down and a summary record is present.
#[test]
fn test_session_compaction() {
    let temp = tempfile::tempdir().expect("tmp");
    let db_path = temp.path().join("sessions.db");
    let transcript_root = temp.path().join("transcripts");

    let engine = SessionEngine::new(&db_path, &transcript_root).expect("engine");

    let session_id = "agent:merlin:main";
    engine.load_or_create(session_id, None).expect("create");

    // Append 50 messages
    for i in 0..50 {
        let msg = json!({
            "role": if i % 2 == 0 { "user" } else { "assistant" },
            "content": format!("Message number {i} with some padding text to contribute tokens.")
        });
        engine.append_message(session_id, &msg).expect("append");
    }

    // Verify we have 50 messages
    let store = engine.transcript_store(session_id).expect("store");
    let before = store.read(0, None).expect("read");
    assert_eq!(before.len(), 50, "should have 50 messages before compaction");

    // Trigger compaction with 0% threshold (forces compaction)
    let result = engine
        .compact_if_needed(session_id, 1000, 0)
        .expect("compact");

    let result = result.expect("compaction should have been triggered");
    assert_eq!(result.messages_before, 50);
    // Compaction keeps last 30 + 1 summary = 31
    assert!(
        result.messages_after <= 31,
        "after compaction should have <= 31 messages, got {}",
        result.messages_after
    );

    // Read back transcript
    let after = store.read(0, None).expect("read");
    assert!(
        after.len() <= 31,
        "transcript should have <= 31 entries, got {}",
        after.len()
    );

    // First entry should be the summary record
    let first = &after[0];
    assert_eq!(
        first.get("type").and_then(|v| v.as_str()),
        Some("summary"),
        "first entry should be a summary record"
    );
    assert!(
        first.get("count").is_some(),
        "summary should have a count field"
    );

    // Compaction count should be incremented
    let state = engine.get_state(session_id).expect("state");
    assert_eq!(state.compaction_count, 1, "compaction_count should be 1");
}

/// Memory manager should be able to persist extracted memory candidates.
#[test]
fn test_memory_extraction_during_compaction() {
    let temp = tempfile::tempdir().expect("tmp");
    let manager = MemoryManager::new(temp.path()).expect("manager");

    // Simulate writing extracted memory candidates (as the compaction code would)
    manager
        .write_memory_md("# Extracted Memories\n\n- User prefers dark mode\n- API key stored in vault\n")
        .expect("write");

    let date = chrono::NaiveDate::from_ymd_opt(2026, 4, 4).expect("date");
    manager
        .append_daily_entry(date, "Session compacted: 50 -> 31 messages")
        .expect("append");

    // Verify
    let content = manager.read_memory_md().expect("read").expect("content");
    assert!(content.contains("dark mode"));
    assert!(content.contains("API key"));

    let daily = manager.read_daily(date).expect("read").expect("daily");
    assert!(daily.contains("compacted"));
}
