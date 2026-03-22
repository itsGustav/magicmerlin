use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use tokio::sync::Mutex;

use magicmerlin_gateway::methods::SUPPORTED_METHODS;

// ── Test server infrastructure ──────────────────────────────────────────────

#[derive(Clone)]
struct TestState {
    version: String,
    jobs: Arc<Mutex<HashMap<String, Value>>>,
}

async fn health_handler() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "compatVersion": "test-0.1",
        "fingerprint": "test-fingerprint-abc123",
    }))
}

async fn status_handler() -> impl IntoResponse {
    Json(json!({
        "compat": {
            "compatVersion": "test-0.1",
            "fingerprint": "test-fingerprint-abc123",
        },
        "scheduler": null,
        "openclawStatus": "running",
    }))
}

async fn methods_handler() -> impl IntoResponse {
    Json(json!(SUPPORTED_METHODS))
}

#[derive(serde::Deserialize)]
struct CallRequest {
    method: String,
    #[serde(default)]
    params: Value,
}

async fn call_handler(
    State(state): State<TestState>,
    Json(req): Json<CallRequest>,
) -> impl IntoResponse {
    match req.method.as_str() {
        "health" => (
            StatusCode::OK,
            Json(json!({
                "status": "ok",
                "compatVersion": "test-0.1",
                "fingerprint": "test-fingerprint-abc123"
            })),
        ),
        "status" => (
            StatusCode::OK,
            Json(json!({
                "compat": {"compatVersion": "test-0.1"},
                "scheduler": null
            })),
        ),
        "gateway.status" => (
            StatusCode::OK,
            Json(json!({"version": state.version, "uptime": 42})),
        ),
        "cron.add" => {
            let name = req
                .params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unnamed")
                .to_string();
            let job_id = uuid::Uuid::new_v4().to_string();
            let job = json!({"jobId": &job_id, "name": &name, "enabled": true});
            state.jobs.lock().await.insert(job_id.clone(), job.clone());
            (StatusCode::OK, Json(json!({"jobId": job_id, "ok": true})))
        }
        "cron.list" => {
            let jobs: Vec<Value> = state.jobs.lock().await.values().cloned().collect();
            (StatusCode::OK, Json(json!({"jobs": jobs})))
        }
        "cron.remove" => {
            let job_id = req
                .params
                .get("jobId")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let removed = state.jobs.lock().await.remove(job_id).is_some();
            (StatusCode::OK, Json(json!({"ok": removed})))
        }
        "sessions.list" => (StatusCode::OK, Json(json!({"sessions": []}))),
        "sessions.spawn" => (
            StatusCode::OK,
            Json(json!({"sessionId": uuid::Uuid::new_v4().to_string()})),
        ),
        "config.get" => {
            let key = req
                .params
                .get("key")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            (StatusCode::OK, Json(json!({"key": key, "value": null})))
        }
        "config.list" => (
            StatusCode::OK,
            Json(json!({"keys": ["model", "debug", "log_level"]})),
        ),
        "memory.search" => {
            let query = req
                .params
                .get("query")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            (StatusCode::OK, Json(json!({"results": [], "query": query})))
        }
        "memory.list" => (StatusCode::OK, Json(json!({"memories": []}))),
        "models.list" => (
            StatusCode::OK,
            Json(json!({"models": ["gpt-4", "claude-3", "gemini"]})),
        ),
        "agents.list" => (
            StatusCode::OK,
            Json(json!({"agents": [{"name": "default", "status": "active"}]})),
        ),
        "system.info" => (
            StatusCode::OK,
            Json(json!({"platform": "test", "version": state.version})),
        ),
        "system.event" => (StatusCode::OK, Json(json!({"ok": true}))),
        "plugins.list" => (StatusCode::OK, Json(json!({"plugins": []}))),
        "skills.list" => (StatusCode::OK, Json(json!({"skills": []}))),
        "logs.tail" => (StatusCode::OK, Json(json!({"lines": []}))),
        "browser.status" => (StatusCode::OK, Json(json!({"running": false}))),
        "sandbox.list" => (StatusCode::OK, Json(json!({"sandboxes": []}))),
        "nodes.list" => (StatusCode::OK, Json(json!({"nodes": []}))),
        "directory.search" => (StatusCode::OK, Json(json!({"results": []}))),
        "approvals.list" => (StatusCode::OK, Json(json!({"approvals": []}))),
        "hooks.list" => (StatusCode::OK, Json(json!({"hooks": []}))),
        "channels.list" => (StatusCode::OK, Json(json!({"channels": []}))),
        "subagents.list" => (StatusCode::OK, Json(json!({"subagents": []}))),
        "run.list" => (StatusCode::OK, Json(json!({"runs": []}))),
        _ => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": {
                    "code": "unknown_method",
                    "message": format!("unknown method: {}", req.method)
                }
            })),
        ),
    }
}

fn build_test_router() -> (Router, TestState) {
    let state = TestState {
        version: "0.0.0-test".to_string(),
        jobs: Arc::new(Mutex::new(HashMap::new())),
    };
    let router = Router::new()
        .route("/health", get(health_handler))
        .route("/status", get(status_handler))
        .route("/methods", get(methods_handler))
        .route("/call", post(call_handler))
        .with_state(state.clone());
    (router, state)
}

async fn start_test_server() -> (u16, tokio::task::JoinHandle<()>) {
    let (router, _state) = build_test_router();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    // Give server a moment to start accepting connections
    tokio::time::sleep(Duration::from_millis(50)).await;
    (port, handle)
}

fn base_url(port: u16) -> String {
    format!("http://127.0.0.1:{port}")
}

async fn call_method(port: u16, method: &str, params: Value) -> Value {
    reqwest::Client::new()
        .post(format!("{}/call", base_url(port)))
        .json(&json!({"method": method, "params": params}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

// ── HTTP endpoint tests ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_health_endpoint() {
    let (port, handle) = start_test_server().await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/health", base_url(port)))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
    assert!(body["compatVersion"].is_string());
    assert!(body["fingerprint"].is_string());
    handle.abort();
}

#[tokio::test]
async fn test_health_returns_correct_values() {
    let (port, handle) = start_test_server().await;
    let client = reqwest::Client::new();
    let body: Value = client
        .get(format!("{}/health", base_url(port)))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(body["compatVersion"], "test-0.1");
    assert_eq!(body["fingerprint"], "test-fingerprint-abc123");
    handle.abort();
}

#[tokio::test]
async fn test_status_endpoint() {
    let (port, handle) = start_test_server().await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/status", base_url(port)))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert!(body["compat"]["compatVersion"].is_string());
    assert_eq!(body["openclawStatus"], "running");
    handle.abort();
}

#[tokio::test]
async fn test_methods_endpoint() {
    let (port, handle) = start_test_server().await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/methods", base_url(port)))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let methods = body.as_array().unwrap();
    assert!(
        methods.len() > 100,
        "Should have 100+ methods, got {}",
        methods.len()
    );
    // Verify some key methods exist
    let methods_str: Vec<&str> = methods.iter().filter_map(|v| v.as_str()).collect();
    assert!(methods_str.contains(&"health"));
    assert!(methods_str.contains(&"cron.list"));
    assert!(methods_str.contains(&"sessions.list"));
    assert!(methods_str.contains(&"gateway.status"));
    handle.abort();
}

#[tokio::test]
async fn test_nonexistent_route_returns_404() {
    let (port, handle) = start_test_server().await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/nonexistent", base_url(port)))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
    handle.abort();
}

// ── /call method dispatch tests ─────────────────────────────────────────────

#[tokio::test]
async fn test_call_health() {
    let (port, handle) = start_test_server().await;
    let resp = call_method(port, "health", json!({})).await;
    assert_eq!(resp["status"], "ok");
    handle.abort();
}

#[tokio::test]
async fn test_call_gateway_status() {
    let (port, handle) = start_test_server().await;
    let resp = call_method(port, "gateway.status", json!({})).await;
    assert!(resp["version"].is_string());
    assert_eq!(resp["version"], "0.0.0-test");
    assert_eq!(resp["uptime"], 42);
    handle.abort();
}

#[tokio::test]
async fn test_cron_add_list_remove() {
    let (port, handle) = start_test_server().await;

    // Add a job
    let add = call_method(
        port,
        "cron.add",
        json!({"name": "test-job", "schedule": "* * * * *"}),
    )
    .await;
    assert_eq!(add["ok"], true);
    let job_id = add["jobId"].as_str().unwrap().to_string();

    // List -- should include our job
    let list = call_method(port, "cron.list", json!({})).await;
    let jobs = list["jobs"].as_array().unwrap();
    assert!(jobs.iter().any(|j| j["jobId"] == job_id));

    // Remove
    let rm = call_method(port, "cron.remove", json!({"jobId": job_id})).await;
    assert_eq!(rm["ok"], true);

    // Verify removed
    let list2 = call_method(port, "cron.list", json!({})).await;
    let jobs2 = list2["jobs"].as_array().unwrap();
    assert!(!jobs2.iter().any(|j| j["jobId"] == job_id));

    handle.abort();
}

#[tokio::test]
async fn test_sessions_list() {
    let (port, handle) = start_test_server().await;
    let resp = call_method(port, "sessions.list", json!({})).await;
    assert!(resp["sessions"].is_array());
    handle.abort();
}

#[tokio::test]
async fn test_sessions_spawn() {
    let (port, handle) = start_test_server().await;
    let resp = call_method(port, "sessions.spawn", json!({})).await;
    assert!(resp["sessionId"].is_string());
    // UUIDs are 36 chars: 8-4-4-4-12
    let sid = resp["sessionId"].as_str().unwrap();
    assert_eq!(sid.len(), 36);
    handle.abort();
}

#[tokio::test]
async fn test_config_operations() {
    let (port, handle) = start_test_server().await;
    let get_resp = call_method(port, "config.get", json!({"key": "model"})).await;
    assert_eq!(get_resp["key"], "model");
    let list_resp = call_method(port, "config.list", json!({})).await;
    assert!(list_resp["keys"].is_array());
    let keys: Vec<&str> = list_resp["keys"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(keys.contains(&"model"));
    assert!(keys.contains(&"debug"));
    handle.abort();
}

#[tokio::test]
async fn test_memory_operations() {
    let (port, handle) = start_test_server().await;
    let search = call_method(port, "memory.search", json!({"query": "test"})).await;
    assert!(search["results"].is_array());
    assert_eq!(search["query"], "test");
    let list = call_method(port, "memory.list", json!({})).await;
    assert!(list["memories"].is_array());
    handle.abort();
}

#[tokio::test]
async fn test_models_list() {
    let (port, handle) = start_test_server().await;
    let resp = call_method(port, "models.list", json!({})).await;
    assert!(resp["models"].is_array());
    let models: Vec<&str> = resp["models"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(!models.is_empty());
    assert!(models.contains(&"claude-3"));
    handle.abort();
}

#[tokio::test]
async fn test_system_info_and_event() {
    let (port, handle) = start_test_server().await;
    let info = call_method(port, "system.info", json!({})).await;
    assert!(info["version"].is_string());
    assert_eq!(info["platform"], "test");
    let event = call_method(port, "system.event", json!({"text": "test event"})).await;
    assert_eq!(event["ok"], true);
    handle.abort();
}

#[tokio::test]
async fn test_agents_list() {
    let (port, handle) = start_test_server().await;
    let resp = call_method(port, "agents.list", json!({})).await;
    let agents = resp["agents"].as_array().unwrap();
    assert!(!agents.is_empty());
    assert_eq!(agents[0]["name"], "default");
    assert_eq!(agents[0]["status"], "active");
    handle.abort();
}

#[tokio::test]
async fn test_unknown_method_returns_error() {
    let (port, handle) = start_test_server().await;
    let resp = call_method(port, "nonexistent.method", json!({})).await;
    assert!(resp["error"].is_object());
    assert_eq!(resp["error"]["code"], "unknown_method");
    assert!(resp["error"]["message"]
        .as_str()
        .unwrap()
        .contains("nonexistent.method"));
    handle.abort();
}

#[tokio::test]
async fn test_multiple_cron_jobs() {
    let (port, handle) = start_test_server().await;

    // Add multiple jobs
    let add1 = call_method(port, "cron.add", json!({"name": "job-1"})).await;
    let add2 = call_method(port, "cron.add", json!({"name": "job-2"})).await;
    let add3 = call_method(port, "cron.add", json!({"name": "job-3"})).await;

    let id1 = add1["jobId"].as_str().unwrap().to_string();
    let id2 = add2["jobId"].as_str().unwrap().to_string();
    let id3 = add3["jobId"].as_str().unwrap().to_string();

    // List all
    let list = call_method(port, "cron.list", json!({})).await;
    assert_eq!(list["jobs"].as_array().unwrap().len(), 3);

    // Remove middle one
    call_method(port, "cron.remove", json!({"jobId": id2})).await;
    let list2 = call_method(port, "cron.list", json!({})).await;
    assert_eq!(list2["jobs"].as_array().unwrap().len(), 2);

    // Remaining jobs should be 1 and 3
    let remaining: Vec<&str> = list2["jobs"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|j| j["jobId"].as_str())
        .collect();
    assert!(remaining.contains(&id1.as_str()));
    assert!(remaining.contains(&id3.as_str()));

    handle.abort();
}

#[tokio::test]
async fn test_all_list_methods_return_arrays() {
    let (port, handle) = start_test_server().await;

    let list_methods = vec![
        ("plugins.list", "plugins"),
        ("skills.list", "skills"),
        ("hooks.list", "hooks"),
        ("channels.list", "channels"),
        ("subagents.list", "subagents"),
        ("nodes.list", "nodes"),
        ("sandbox.list", "sandboxes"),
        ("run.list", "runs"),
        ("approvals.list", "approvals"),
    ];

    for (method, key) in list_methods {
        let resp = call_method(port, method, json!({})).await;
        assert!(
            resp[key].is_array(),
            "Method {method} should return {key} as array, got: {resp}"
        );
    }

    handle.abort();
}

#[tokio::test]
async fn test_browser_status() {
    let (port, handle) = start_test_server().await;
    let resp = call_method(port, "browser.status", json!({})).await;
    assert_eq!(resp["running"], false);
    handle.abort();
}

#[tokio::test]
async fn test_directory_search() {
    let (port, handle) = start_test_server().await;
    let resp = call_method(port, "directory.search", json!({})).await;
    assert!(resp["results"].is_array());
    handle.abort();
}

#[tokio::test]
async fn test_logs_tail() {
    let (port, handle) = start_test_server().await;
    let resp = call_method(port, "logs.tail", json!({})).await;
    assert!(resp["lines"].is_array());
    handle.abort();
}

#[tokio::test]
async fn test_cron_remove_nonexistent_returns_false() {
    let (port, handle) = start_test_server().await;
    let resp = call_method(
        port,
        "cron.remove",
        json!({"jobId": "does-not-exist-12345"}),
    )
    .await;
    assert_eq!(resp["ok"], false);
    handle.abort();
}

// ── SUPPORTED_METHODS constant tests ────────────────────────────────────────

#[test]
fn test_supported_methods_count() {
    assert!(
        SUPPORTED_METHODS.len() >= 100,
        "Should have at least 100 supported methods, got {}",
        SUPPORTED_METHODS.len()
    );
}

#[test]
fn test_supported_methods_no_duplicates() {
    let mut seen = std::collections::HashSet::new();
    for method in SUPPORTED_METHODS {
        assert!(seen.insert(method), "Duplicate method: {method}");
    }
}

#[test]
fn test_supported_methods_format() {
    for method in SUPPORTED_METHODS {
        assert!(!method.is_empty(), "Empty method name");
        assert!(!method.contains(' '), "Method contains space: {method}");
        // Methods should be ASCII alphanumeric with dots/hyphens as separators
        assert!(
            method
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_'),
            "Method has invalid characters: {method}"
        );
    }
}

#[test]
fn test_supported_methods_contains_core_methods() {
    let methods: std::collections::HashSet<&&str> = SUPPORTED_METHODS.iter().collect();
    let required = [
        "health",
        "status",
        "agent.run",
        "sessions.list",
        "sessions.spawn",
        "cron.list",
        "cron.add",
        "config.get",
        "config.list",
        "approvals.list",
        "plugins.list",
        "system.info",
        "system.event",
        "memory.search",
        "models.list",
        "agents.list",
        "gateway.status",
    ];
    for name in &required {
        assert!(
            methods.contains(name),
            "SUPPORTED_METHODS missing core method: {name}"
        );
    }
}
