//! Integration tests: cron job scheduling via gateway subprocess.

use std::process::Stdio;
use std::time::Duration;

use magicmerlin_integration_tests::find_free_port;
use serde_json::Value;

struct TestGatewayDaemon {
    port: u16,
    child: tokio::process::Child,
    _temp: tempfile::TempDir,
}

impl TestGatewayDaemon {
    async fn start() -> Self {
        let port = find_free_port();
        let temp = tempfile::tempdir().expect("tempdir");
        let db_path = temp.path().join("test.db");

        let gateway_bin = magicmerlin_integration_tests::cargo_bin("magicmerlin-gateway");
        assert!(
            gateway_bin.exists(),
            "gateway binary not found at {}; run `cargo build --workspace` first",
            gateway_bin.display()
        );

        let child = tokio::process::Command::new(&gateway_bin)
            .arg("--serve")
            .arg(port.to_string())
            .arg("--daemon")
            .arg("--db-path")
            .arg(db_path.to_str().unwrap())
            .arg("--log-level")
            .arg("silent")
            .env("OPENCLAW_STATE_DIR", temp.path())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .expect("spawn gateway with daemon");

        // Wait for health.
        let client = reqwest::Client::new();
        let url = format!("http://127.0.0.1:{port}/health");
        for _ in 0..40 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            if client.get(&url).send().await.is_ok() {
                return Self {
                    port,
                    child,
                    _temp: temp,
                };
            }
        }
        panic!("daemon gateway did not start within 4s on port {port}");
    }

    async fn call(&self, method: &str, params: Value) -> Value {
        reqwest::Client::new()
            .post(format!("http://127.0.0.1:{}/call", self.port))
            .json(&serde_json::json!({"method": method, "params": params}))
            .send()
            .await
            .expect("call send")
            .json::<Value>()
            .await
            .expect("call json")
    }

    async fn stop(mut self) {
        let _ = self.child.kill().await;
    }
}

#[tokio::test]
async fn test_cron_add_remove() {
    let gw = TestGatewayDaemon::start().await;

    // Add a job
    let resp = gw
        .call(
            "cron.add",
            serde_json::json!({
                "name": "test-add-remove",
                "schedule": "every:3600",
                "kind": "http_get",
                "payload": {"url": "http://localhost:1/noop"},
                "enabled": false,
            }),
        )
        .await;
    assert_eq!(resp["ok"], true, "add failed: {resp}");
    let job_id = resp["id"].as_i64().expect("id");

    // Remove it
    let resp = gw
        .call("cron.remove", serde_json::json!({"id": job_id}))
        .await;
    // Check the removal succeeded (no error)
    assert!(
        resp.get("error").is_none() || resp["ok"] == true,
        "remove failed: {resp}"
    );

    // List should be empty again
    let list = gw.call("cron.list", serde_json::json!({})).await;
    assert_eq!(list["jobs"].as_array().unwrap().len(), 0);

    gw.stop().await;
}

#[tokio::test]
async fn test_cron_pause_resume() {
    let gw = TestGatewayDaemon::start().await;

    let resp = gw
        .call(
            "cron.add",
            serde_json::json!({
                "name": "pause-test",
                "schedule": "every:3600",
                "kind": "http_get",
                "payload": {"url": "http://localhost:1/noop"},
            }),
        )
        .await;
    let job_id = resp["id"].as_i64().expect("id");

    // Pause
    let pause_resp = gw
        .call("cron.pause", serde_json::json!({"id": job_id}))
        .await;
    assert!(
        pause_resp.get("error").is_none(),
        "pause failed: {pause_resp}"
    );

    // Verify it's disabled
    let list = gw.call("cron.list", serde_json::json!({})).await;
    let job = &list["jobs"].as_array().unwrap()[0];
    assert_eq!(job["enabled"], false);

    // Resume
    let resume_resp = gw
        .call("cron.resume", serde_json::json!({"id": job_id}))
        .await;
    assert!(
        resume_resp.get("error").is_none(),
        "resume failed: {resume_resp}"
    );

    // Verify it's enabled
    let list = gw.call("cron.list", serde_json::json!({})).await;
    let job = &list["jobs"].as_array().unwrap()[0];
    assert_eq!(job["enabled"], true);

    gw.stop().await;
}

#[tokio::test]
async fn test_cron_job_fires() {
    let gw = TestGatewayDaemon::start().await;

    // Add a 1-second interval job with minimal backoff for fast retries.
    let resp = gw
        .call(
            "cron.add",
            serde_json::json!({
                "name": "fast-fire",
                "schedule": "every:1",
                "kind": "http_get",
                "payload": {"url": "http://127.0.0.1:1/noop"},
                "maxAttempts": 100,
                "backoffSeconds": 1,
            }),
        )
        .await;
    assert_eq!(resp["ok"], true, "add failed: {resp}");
    let job_id = resp["id"].as_i64().expect("id");

    // Wait 4.5s for the scheduler daemon to fire the job multiple times.
    tokio::time::sleep(Duration::from_millis(4500)).await;

    // Check run history
    let runs_resp = gw
        .call(
            "cron.runs",
            serde_json::json!({"jobId": job_id, "limit": 50}),
        )
        .await;

    // runs may be nested under "runs" key or may be direct array
    let runs = runs_resp
        .get("runs")
        .and_then(|v| v.as_array())
        .or_else(|| runs_resp.as_array())
        .cloned()
        .unwrap_or_default();

    assert!(
        runs.len() >= 2,
        "Expected ≥2 runs, got {}. Response: {runs_resp}",
        runs.len()
    );

    gw.stop().await;
}

#[tokio::test]
async fn test_cron_scheduler_state() {
    let gw = TestGatewayDaemon::start().await;

    let state = gw.call("cron.status", serde_json::json!({})).await;
    // Response may be {"scheduler":{"jobCount":...}} or {"jobCount":...}
    let has_job_count = state.get("jobCount").is_some()
        || state.get("job_count").is_some()
        || state
            .get("scheduler")
            .and_then(|s| s.get("jobCount").or_else(|| s.get("job_count")))
            .is_some();
    assert!(has_job_count, "scheduler state missing jobCount: {state}");

    gw.stop().await;
}

#[tokio::test]
async fn test_cron_dead_letters_empty() {
    let gw = TestGatewayDaemon::start().await;

    let resp = gw
        .call("cron.deadLetters", serde_json::json!({"limit": 10}))
        .await;
    // Should return an array (empty)
    let letters = resp
        .get("deadLetters")
        .and_then(|v| v.as_array())
        .or_else(|| resp.as_array());
    assert!(letters.is_some(), "unexpected response: {resp}");

    gw.stop().await;
}
