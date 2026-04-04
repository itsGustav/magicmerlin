//! Integration test: memory search via MemoryManager.
//!
//! This tests the memory file layer that underlies the semantic search tool.
//! Full fastembed vector search requires model download and is tested at the
//! unit level in agent-tools. Here we verify the storage layer works correctly
//! for read/write/search scenarios.

use chrono::NaiveDate;
use magicmerlin_storage::MemoryManager;

/// Write known content to memory files and verify it can be found by reading.
#[test]
fn test_memory_search_by_content() {
    let temp = tempfile::tempdir().expect("tmp");
    let manager = MemoryManager::new(temp.path()).expect("manager");

    // Write MEMORY.md with searchable content
    manager
        .write_memory_md(
            "# Memory\n\n\
             - The user's favorite color is blue\n\
             - Project deadline is April 15\n\
             - Database password is stored in 1Password vault\n",
        )
        .expect("write");

    // Write daily entries with searchable content
    let day1 = NaiveDate::from_ymd_opt(2026, 4, 1).expect("date");
    let day2 = NaiveDate::from_ymd_opt(2026, 4, 2).expect("date");
    manager
        .append_daily_entry(day1, "Deployed version 0.9.0 to staging")
        .expect("append");
    manager
        .append_daily_entry(day1, "Fixed memory leak in session handler")
        .expect("append");
    manager
        .append_daily_entry(day2, "User requested dark mode support")
        .expect("append");

    // Verify MEMORY.md contains expected snippets
    let memory_md = manager.read_memory_md().expect("read").expect("content");
    assert!(memory_md.contains("favorite color is blue"));
    assert!(memory_md.contains("April 15"));

    // Verify daily files contain expected content
    let daily1 = manager.read_daily(day1).expect("read").expect("content");
    assert!(daily1.contains("version 0.9.0"));
    assert!(daily1.contains("memory leak"));

    let daily2 = manager.read_daily(day2).expect("read").expect("content");
    assert!(daily2.contains("dark mode"));

    // Missing day returns None
    let day3 = NaiveDate::from_ymd_opt(2026, 4, 3).expect("date");
    assert!(manager.read_daily(day3).expect("read").is_none());
}

/// Overwriting MEMORY.md replaces previous content entirely.
#[test]
fn test_memory_md_overwrite() {
    let temp = tempfile::tempdir().expect("tmp");
    let manager = MemoryManager::new(temp.path()).expect("manager");

    manager.write_memory_md("first version").expect("write1");
    manager.write_memory_md("second version").expect("write2");

    let content = manager.read_memory_md().expect("read").expect("content");
    assert!(!content.contains("first"), "old content should be gone");
    assert!(content.contains("second"));
}

/// Daily entries append, not overwrite.
#[test]
fn test_daily_entries_append() {
    let temp = tempfile::tempdir().expect("tmp");
    let manager = MemoryManager::new(temp.path()).expect("manager");
    let date = NaiveDate::from_ymd_opt(2026, 4, 4).expect("date");

    manager.append_daily_entry(date, "entry one").expect("a1");
    manager.append_daily_entry(date, "entry two").expect("a2");

    let content = manager.read_daily(date).expect("read").expect("content");
    assert!(content.contains("entry one"));
    assert!(content.contains("entry two"));
}
