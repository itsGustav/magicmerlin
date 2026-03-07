use std::collections::VecDeque;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tokio::process::Command;
use tokio::sync::{broadcast, Mutex, RwLock};

use crate::{MediaError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiUpdate {
    pub event: String,
    #[serde(default)]
    pub payload: Value,
}

#[derive(Debug, Clone)]
pub struct CanvasConfig {
    pub bind_addr: SocketAddr,
    pub chrome_binary: String,
}

impl Default for CanvasConfig {
    fn default() -> Self {
        Self {
            bind_addr: SocketAddr::from(([127, 0, 0, 1], 4100)),
            chrome_binary: std::env::var("CHROME_BIN")
                .unwrap_or_else(|_| "google-chrome".to_string()),
        }
    }
}

#[derive(Debug)]
struct CanvasState {
    html: RwLock<String>,
    current_url: RwLock<String>,
    updates: Mutex<VecDeque<UiUpdate>>,
    updates_tx: broadcast::Sender<String>,
}

#[derive(Debug)]
pub struct CanvasServer {
    config: CanvasConfig,
    state: Arc<CanvasState>,
}

#[derive(Debug)]
pub struct CanvasHandle {
    pub addr: SocketAddr,
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

impl CanvasHandle {
    pub async fn shutdown(self) -> Result<()> {
        self.shutdown_tx
            .send(())
            .map_err(|_| MediaError::Execution("canvas server already stopped".to_string()))
    }
}

impl CanvasServer {
    pub fn new(config: CanvasConfig) -> Self {
        let (updates_tx, _) = broadcast::channel(512);
        let state = CanvasState {
            html: RwLock::new("<html><body><h1>Canvas</h1></body></html>".to_string()),
            current_url: RwLock::new("about:blank".to_string()),
            updates: Mutex::new(VecDeque::new()),
            updates_tx,
        };

        Self {
            config,
            state: Arc::new(state),
        }
    }

    pub async fn start(&self) -> Result<CanvasHandle> {
        let app = self.router();
        let listener = TcpListener::bind(self.config.bind_addr)
            .await
            .map_err(|err| MediaError::Execution(format!("canvas bind failed: {err}")))?;
        let addr = listener
            .local_addr()
            .map_err(|err| MediaError::Execution(format!("canvas local_addr failed: {err}")))?;

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let server = axum::serve(listener, app).with_graceful_shutdown(async move {
            let _ = shutdown_rx.await;
        });

        tokio::spawn(async move {
            let _ = server.await;
        });

        Ok(CanvasHandle { addr, shutdown_tx })
    }

    pub fn router(&self) -> Router {
        Router::new()
            .route("/", get(Self::http_get_html))
            .route("/a2ui/push", post(Self::http_push_update))
            .route("/a2ui/stream", get(Self::http_stream_updates))
            .with_state(self.state.clone())
    }

    pub async fn set_html(&self, html: impl Into<String>) {
        let mut state = self.state.html.write().await;
        *state = html.into();
    }

    pub async fn html(&self) -> String {
        self.state.html.read().await.clone()
    }

    pub async fn navigate_url(&self, url: &str) -> Result<()> {
        if url.starts_with("http://") || url.starts_with("https://") {
            let body = reqwest::Client::new().get(url).send().await?.text().await?;
            self.set_html(body).await;
            *self.state.current_url.write().await = url.to_string();
            self.push_update(UiUpdate {
                event: "navigate".to_string(),
                payload: json!({ "url": url }),
            })
            .await?;
            return Ok(());
        }

        if url.starts_with("data:text/html,") {
            let html = url.trim_start_matches("data:text/html,");
            self.set_html(html).await;
            *self.state.current_url.write().await = "data:text/html".to_string();
            self.push_update(UiUpdate {
                event: "navigate".to_string(),
                payload: json!({ "url": "data:text/html" }),
            })
            .await?;
            return Ok(());
        }

        Err(MediaError::InvalidInput(format!(
            "unsupported navigation URL: {url}"
        )))
    }

    pub async fn current_url(&self) -> String {
        self.state.current_url.read().await.clone()
    }

    pub async fn push_update(&self, update: UiUpdate) -> Result<()> {
        {
            let mut queue = self.state.updates.lock().await;
            queue.push_back(update.clone());
            if queue.len() > 1024 {
                queue.pop_front();
            }
        }

        let line = serde_json::to_string(&update)?;
        let _ = self.state.updates_tx.send(line);
        Ok(())
    }

    pub async fn pop_update(&self) -> Option<UiUpdate> {
        let mut queue = self.state.updates.lock().await;
        queue.pop_front()
    }

    pub async fn evaluate_js(&self, script: &str) -> Result<Value> {
        let html = self.html().await;

        // We intentionally eval in Node to support real JS expression semantics.
        let wrapped = format!(
            "const html = {html:?};\nlet result = (async () => {{ return ({script}); }})();\nPromise.resolve(result).then(v => console.log(JSON.stringify({{ok:true,value:v}}))).catch(err => console.log(JSON.stringify({{ok:false,error:String(err)}})));"
        );

        let output = Command::new("node")
            .arg("-e")
            .arg(wrapped)
            .output()
            .await
            .map_err(|err| MediaError::Execution(format!("node eval launch failed: {err}")))?;

        if !output.status.success() {
            return Err(MediaError::Execution(format!(
                "node eval failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let last_line = stdout
            .lines()
            .last()
            .ok_or_else(|| MediaError::Execution("node eval returned no output".to_string()))?;

        let parsed: Value = serde_json::from_str(last_line).map_err(|err| {
            MediaError::Execution(format!(
                "node eval emitted invalid JSON ({err}): {last_line}"
            ))
        })?;

        if parsed.get("ok").and_then(Value::as_bool) == Some(true) {
            return Ok(parsed.get("value").cloned().unwrap_or(Value::Null));
        }

        Err(MediaError::Execution(format!(
            "node eval error: {}",
            parsed
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
        )))
    }

    pub async fn capture_screenshot(&self, output_path: PathBuf) -> Result<PathBuf> {
        let html = self.html().await;
        let tmp = tempfile::Builder::new()
            .prefix("canvas-shot-")
            .suffix(".html")
            .tempfile()
            .map_err(MediaError::Io)?;
        tokio::fs::write(tmp.path(), html).await?;

        let file_url = format!("file://{}", tmp.path().display());
        let output = Command::new(&self.config.chrome_binary)
            .arg("--headless=new")
            .arg("--disable-gpu")
            .arg(format!("--screenshot={}", output_path.display()))
            .arg("--window-size=1365,768")
            .arg(file_url)
            .output()
            .await
            .map_err(|err| {
                MediaError::Execution(format!("chrome screenshot launch failed: {err}"))
            })?;

        if !output.status.success() {
            return Err(MediaError::Execution(format!(
                "chrome screenshot failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(output_path)
    }

    async fn http_get_html(State(state): State<Arc<CanvasState>>) -> Html<String> {
        Html(state.html.read().await.clone())
    }

    async fn http_push_update(
        State(state): State<Arc<CanvasState>>,
        Json(update): Json<UiUpdate>,
    ) -> Result<StatusCode> {
        {
            let mut queue = state.updates.lock().await;
            queue.push_back(update.clone());
            if queue.len() > 1024 {
                queue.pop_front();
            }
        }

        let line = serde_json::to_string(&update)?;
        let _ = state.updates_tx.send(line);
        Ok(StatusCode::ACCEPTED)
    }

    async fn http_stream_updates(State(state): State<Arc<CanvasState>>) -> Response {
        let receiver = state.updates_tx.subscribe();
        let stream = futures_util::stream::unfold(receiver, |mut rx| async move {
            loop {
                match rx.recv().await {
                    Ok(line) => {
                        return Some((Ok::<_, std::convert::Infallible>(line + "\n"), rx));
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => return None,
                }
            }
        });

        let mut response = Response::new(Body::from_stream(stream));
        response.headers_mut().insert(
            "content-type",
            HeaderValue::from_static("application/x-ndjson"),
        );
        response
    }
}

impl IntoResponse for MediaError {
    fn into_response(self) -> Response {
        let status = match self {
            MediaError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn state_round_trip_html_and_updates() {
        let server = CanvasServer::new(CanvasConfig::default());
        server.set_html("<html><body>hello</body></html>").await;
        assert!(server.html().await.contains("hello"));

        server
            .push_update(UiUpdate {
                event: "patch".to_string(),
                payload: json!({"dom": "updated"}),
            })
            .await
            .expect("push update should succeed");

        let update = server.pop_update().await.expect("update should exist");
        assert_eq!(update.event, "patch");
    }

    #[tokio::test]
    async fn navigation_rejects_unknown_scheme() {
        let server = CanvasServer::new(CanvasConfig::default());
        let result = server.navigate_url("ftp://example.com").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn router_serves_html_endpoint() {
        let server = CanvasServer::new(CanvasConfig::default());
        server.set_html("<html><body>x</body></html>").await;
        let response = CanvasServer::http_get_html(State(server.state.clone())).await;
        assert!(response.0.contains("<body>x</body>"));
    }

    #[tokio::test]
    async fn a2ui_push_route_accepts_event() {
        let server = CanvasServer::new(CanvasConfig::default());
        let status = CanvasServer::http_push_update(
            State(server.state.clone()),
            Json(UiUpdate {
                event: "render".to_string(),
                payload: json!({"id": 1}),
            }),
        )
        .await
        .expect("push should succeed");

        assert_eq!(status, StatusCode::ACCEPTED);
        let update = server.pop_update().await.expect("event should be queued");
        assert_eq!(update.event, "render");
    }

    #[tokio::test]
    async fn evaluate_js_fails_without_node() {
        let mut config = CanvasConfig::default();
        config.chrome_binary = "google-chrome".to_string();
        let server = CanvasServer::new(config);

        let result = server.evaluate_js("1 + 1").await;
        if result.is_err() {
            assert!(format!("{}", result.err().expect("err")).contains("node"));
        }
    }
}
