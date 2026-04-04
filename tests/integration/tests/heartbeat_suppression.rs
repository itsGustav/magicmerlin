//! Integration test: HEARTBEAT_OK suppression.
//!
//! When the heartbeat finds no actionable tasks the outcome is `HeartbeatOutcome::Ok`
//! and the status string is `"HEARTBEAT_OK"`. The channel loop must NOT relay that
//! status to end-users. This test verifies the detection side: an empty heartbeat
//! file correctly produces the suppressed outcome.

use magicmerlin_agent::{
    run_heartbeat, run_heartbeat_with_state, HeartbeatOutcome, HeartbeatState,
};

/// An empty (comment-only) HEARTBEAT.md should yield `HeartbeatOutcome::Ok`.
#[test]
fn test_heartbeat_ok_on_empty_file() {
    let temp = tempfile::tempdir().expect("tmp");
    std::fs::write(
        temp.path().join("HEARTBEAT.md"),
        "# Heartbeat\n\n<!-- nothing to do -->\n",
    )
    .expect("write");

    let outcome = run_heartbeat(temp.path()).expect("heartbeat");
    assert_eq!(outcome, HeartbeatOutcome::Ok, "empty file should yield Ok");
}

/// No HEARTBEAT.md at all should also yield `Ok` (suppressed).
#[test]
fn test_heartbeat_ok_when_missing() {
    let temp = tempfile::tempdir().expect("tmp");
    let outcome = run_heartbeat(temp.path()).expect("heartbeat");
    assert_eq!(
        outcome,
        HeartbeatOutcome::Ok,
        "missing file should yield Ok"
    );
}

/// `run_heartbeat_with_state` must return status `"HEARTBEAT_OK"` (the suppression
/// sentinel the channel loop checks) when there are no tasks.
#[test]
fn test_heartbeat_ok_suppressed_status_string() {
    let temp = tempfile::tempdir().expect("tmp");
    std::fs::write(temp.path().join("HEARTBEAT.md"), "# noop\n").expect("write");

    let mut state = HeartbeatState::default();
    let result =
        run_heartbeat_with_state(temp.path(), None, None, &mut state).expect("heartbeat");

    assert_eq!(
        result.status, "HEARTBEAT_OK",
        "status must be HEARTBEAT_OK for suppression"
    );
    assert!(
        result.tasks_to_run.is_empty(),
        "no tasks should be scheduled"
    );
}

/// When there ARE tasks, the outcome should NOT be `Ok` — it should be `Tasks`.
#[test]
fn test_heartbeat_tasks_not_suppressed() {
    let temp = tempfile::tempdir().expect("tmp");
    std::fs::write(
        temp.path().join("HEARTBEAT.md"),
        "# Tasks\n- [ ] Check memory usage\n- [ ] Refresh session cache\n",
    )
    .expect("write");

    let outcome = run_heartbeat(temp.path()).expect("heartbeat");
    match outcome {
        HeartbeatOutcome::Tasks(tasks) => {
            assert_eq!(tasks.len(), 2);
        }
        HeartbeatOutcome::Ok => panic!("expected Tasks, got Ok"),
    }

    let mut state = HeartbeatState::default();
    let result =
        run_heartbeat_with_state(temp.path(), None, None, &mut state).expect("heartbeat");
    assert_eq!(result.status, "HEARTBEAT_TASKS");
    assert!(!result.tasks_to_run.is_empty());
}
