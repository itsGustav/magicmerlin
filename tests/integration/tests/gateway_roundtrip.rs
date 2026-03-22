//! Integration tests: spawn gateway subprocess and verify HTTP endpoints.

use std::process::Stdio;
use std::time::Duration;

use magicmerlin_integration_tests::find_free_port;
use serde_json::Value;

struct TestGateway {
    port: u16,
    child: tokio::process::Child,
    _temp: tempfile::TempDir,
}

impl TestGateway {
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
            .arg("--db-path")
            .arg(db_path.to_str().unwrap())
            .arg("--log-level")
            .arg("silent")
            .env("OPENCLAW_STATE_DIR", temp.path())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .expect("spawn gateway");

        // Wait for it to be ready (poll health endpoint).
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
        panic!("gateway did not become healthy within 4s on port {port}");
    }

    async fn http_get(&self, path: &str) -> Value {
        reqwest::Client::new()
            .get(format!("http://127.0.0.1:{}{}", self.port, path))
            .send()
            .await
            .expect("http_get send")
            .json::<Value>()
            .await
            .expect("http_get json")
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
async fn test_gateway_health() {
    let gw = TestGateway::start().await;
    let resp = gw.http_get("/health").await;
    assert_eq!(resp["status"], "ok");
    assert!(resp.get("compatVersion").is_some());
    assert!(resp.get("fingerprint").is_some());
    gw.stop().await;
}

#[tokio::test]
async fn test_gateway_status() {
    let gw = TestGateway::start().await;
    let resp = gw.http_get("/status").await;
    assert!(resp.get("compat").is_some());
    assert!(resp.get("openclawStatus").is_some());
    gw.stop().await;
}

#[tokio::test]
async fn test_call_health() {
    let gw = TestGateway::start().await;
    let resp = gw.call("health", serde_json::json!({})).await;
    assert_eq!(resp["status"], "ok");
    gw.stop().await;
}

#[tokio::test]
async fn test_call_gateway_status() {
    let gw = TestGateway::start().await;
    let resp = gw.call("status", serde_json::json!({})).await;
    assert!(resp.get("compat").is_some());
    gw.stop().await;
}

#[tokio::test]
async fn test_cron_list_empty() {
    let gw = TestGateway::start().await;
    let resp = gw.call("cron.list", serde_json::json!({})).await;
    assert!(resp["jobs"].as_array().is_some());
    assert_eq!(resp["jobs"].as_array().unwrap().len(), 0);
    gw.stop().await;
}

#[tokio::test]
async fn test_methods_endpoint() {
    let gw = TestGateway::start().await;
    let resp = gw.http_get("/methods").await;
    assert!(resp.as_array().is_some() || resp.get("methods").is_some());
    gw.stop().await;
}

#[tokio::test]
async fn test_sessions_list_empty() {
    let gw = TestGateway::start().await;
    let resp = gw.call("sessions.list", serde_json::json!({})).await;
    // Should return a list (possibly empty)
    let has_sessions = resp.get("sessions").is_some() || resp.as_array().is_some();
    let has_error = resp.get("error").is_some();
    assert!(has_sessions || has_error, "unexpected response: {resp}");
    gw.stop().await;
}

#[tokio::test]
async fn test_cron_add_and_list() {
    let gw = TestGateway::start().await;

    let add_resp = gw
        .call(
            "cron.add",
            serde_json::json!({
                "name": "test-job",
                "schedule": "every:3600",
                "kind": "http_get",
                "payload": {"url": "http://localhost/noop"},
                "enabled": false,
            }),
        )
        .await;
    assert_eq!(add_resp["ok"], true, "add_resp: {add_resp}");
    assert!(add_resp.get("id").is_some());

    let list_resp = gw.call("cron.list", serde_json::json!({})).await;
    let jobs = list_resp["jobs"].as_array().unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0]["name"], "test-job");

    gw.stop().await;
}

#[tokio::test]
async fn test_tools_endpoint() {
    let gw = TestGateway::start().await;
    let resp = gw.http_get("/tools").await;
    assert!(resp.get("tools").is_some());
    gw.stop().await;
}

#[tokio::test]
async fn test_snapshots_endpoint() {
    let gw = TestGateway::start().await;
    let resp = gw.http_get("/snapshots").await;
    assert!(resp.get("compatVersion").is_some());
    gw.stop().await;
}
