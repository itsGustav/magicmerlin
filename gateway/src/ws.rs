use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{mpsc, Mutex, RwLock};

pub const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(30);
pub const MAX_MISSED_PONGS: usize = 3;

#[derive(Debug, Clone, Default)]
pub struct WsServerConfig {
    pub auth_bearer_token: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WsServerState {
    config: WsServerConfig,
    clients: Arc<RwLock<HashMap<String, ClientHandle>>>,
}

#[derive(Debug)]
struct ClientHandle {
    tx: mpsc::Sender<WireMessage>,
    state: Arc<Mutex<ClientState>>,
}

#[derive(Debug, Clone)]
pub enum WireMessage {
    Text(String),
    Ping,
    Pong,
    Close,
}

#[derive(Debug)]
struct ClientState {
    id: String,
    connected_at_unix_ms: i64,
    last_pong_at_unix_ms: i64,
    missed_pongs: usize,
    authenticated: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
    #[serde(default)]
    pub id: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonRpcResponse {
    pub jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0",
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn error(id: Option<Value>, code: i64, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0",
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
            }),
            id,
        }
    }
}

impl WsServerState {
    pub fn new(config: WsServerConfig) -> Self {
        Self {
            config,
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn authenticate(
        &self,
        bearer_header: Option<&str>,
        query_token: Option<&str>,
    ) -> bool {
        let Some(expected) = self.config.auth_bearer_token.as_deref() else {
            return true;
        };

        if let Some(token) = bearer_header.and_then(parse_bearer_auth) {
            if token == expected {
                return true;
            }
        }

        query_token.is_some_and(|value| value == expected)
    }

    pub async fn connected_clients(&self) -> Vec<ClientSnapshot> {
        let clients = self.clients.read().await;
        let mut snapshots = Vec::with_capacity(clients.len());
        for client in clients.values() {
            let state = client.state.lock().await;
            snapshots.push(ClientSnapshot {
                id: state.id.clone(),
                connected_at_unix_ms: state.connected_at_unix_ms,
                last_pong_at_unix_ms: state.last_pong_at_unix_ms,
                missed_pongs: state.missed_pongs,
                authenticated: state.authenticated,
            });
        }
        snapshots
    }

    pub async fn register_client(
        &self,
        authenticated: bool,
    ) -> (
        String,
        mpsc::Receiver<WireMessage>,
        mpsc::Sender<WireMessage>,
    ) {
        let id = format!("ws-{}", uuid::Uuid::new_v4());
        let now = now_unix_ms();
        let (tx, rx) = mpsc::channel(256);

        let state = Arc::new(Mutex::new(ClientState {
            id: id.clone(),
            connected_at_unix_ms: now,
            last_pong_at_unix_ms: now,
            missed_pongs: 0,
            authenticated,
        }));

        self.clients.write().await.insert(
            id.clone(),
            ClientHandle {
                tx: tx.clone(),
                state,
            },
        );

        (id, rx, tx)
    }

    pub async fn unregister_client(&self, client_id: &str) {
        self.clients.write().await.remove(client_id);
    }

    pub async fn broadcast_json(&self, payload: &Value) -> usize {
        let message = WireMessage::Text(payload.to_string());
        let clients = self.clients.read().await;
        let mut delivered = 0usize;
        for client in clients.values() {
            if client.tx.send(message.clone()).await.is_ok() {
                delivered += 1;
            }
        }
        delivered
    }

    pub async fn on_pong(&self, client_id: &str) {
        if let Some(client) = self.clients.read().await.get(client_id) {
            let mut state = client.state.lock().await;
            state.missed_pongs = 0;
            state.last_pong_at_unix_ms = now_unix_ms();
        }
    }

    pub async fn keepalive_tick(&self) -> Vec<String> {
        let clients = self.clients.read().await;
        let mut disconnected = Vec::new();
        for (id, client) in clients.iter() {
            let mut state = client.state.lock().await;
            state.missed_pongs += 1;
            if state.missed_pongs > MAX_MISSED_PONGS {
                disconnected.push(id.clone());
                let _ = client.tx.send(WireMessage::Close).await;
            } else {
                let _ = client.tx.send(WireMessage::Ping).await;
            }
        }
        disconnected
    }

    pub fn spawn_keepalive(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(KEEPALIVE_INTERVAL);
            loop {
                ticker.tick().await;
                let stale = self.keepalive_tick().await;
                for client in stale {
                    self.unregister_client(&client).await;
                }
            }
        })
    }

    pub async fn send_rpc_response(&self, client_id: &str, response: &JsonRpcResponse) {
        let clients = self.clients.read().await;
        if let Some(client) = clients.get(client_id) {
            let payload = serde_json::to_string(response).unwrap_or_else(|_| {
                "{\"jsonrpc\":\"2.0\",\"error\":{\"code\":-32603,\"message\":\"serialization failed\"}}".to_string()
            });
            let _ = client.tx.send(WireMessage::Text(payload)).await;
        }
    }

    pub async fn handle_incoming_json(
        &self,
        client_id: &str,
        text: &str,
    ) -> Result<JsonRpcRequest, JsonRpcResponse> {
        let request = serde_json::from_str::<JsonRpcRequest>(text)
            .map_err(|err| JsonRpcResponse::error(None, -32700, format!("parse error: {err}")))?;

        if request.jsonrpc != "2.0" {
            return Err(JsonRpcResponse::error(
                request.id,
                -32600,
                "invalid jsonrpc version",
            ));
        }

        if self.clients.read().await.get(client_id).is_none() {
            return Err(JsonRpcResponse::error(
                request.id,
                -32603,
                "client disconnected",
            ));
        }

        Ok(request)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientSnapshot {
    pub id: String,
    pub connected_at_unix_ms: i64,
    pub last_pong_at_unix_ms: i64,
    pub missed_pongs: usize,
    pub authenticated: bool,
}

pub fn parse_bearer_auth(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    let lower = trimmed.to_ascii_lowercase();
    if !lower.starts_with("bearer ") {
        return None;
    }
    trimmed.split_once(' ').map(|(_, token)| token.trim())
}

pub fn should_retry_connection(err: &str) -> bool {
    let err = err.to_ascii_lowercase();
    err.contains("timeout")
        || err.contains("reset")
        || err.contains("broken pipe")
        || err.contains("temporarily unavailable")
}

pub fn reconnect_backoff(attempt: usize) -> Duration {
    let capped = attempt.min(8);
    Duration::from_secs(2u64.saturating_pow(capped as u32))
}

fn now_unix_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bearer_header() {
        assert_eq!(parse_bearer_auth("Bearer abc123"), Some("abc123"));
        assert_eq!(parse_bearer_auth("bearer abc123"), Some("abc123"));
        assert_eq!(parse_bearer_auth("Token abc123"), None);
        assert_eq!(parse_bearer_auth("abc123"), None);
    }

    #[test]
    fn retries_only_transient_errors() {
        assert!(should_retry_connection("connection reset by peer"));
        assert!(should_retry_connection("i/o timeout"));
        assert!(!should_retry_connection("invalid auth token"));
    }

    #[test]
    fn backoff_is_exponential_and_capped() {
        assert_eq!(reconnect_backoff(0), Duration::from_secs(1));
        assert_eq!(reconnect_backoff(1), Duration::from_secs(2));
        assert_eq!(reconnect_backoff(8), Duration::from_secs(256));
        assert_eq!(reconnect_backoff(20), Duration::from_secs(256));
    }

    #[tokio::test]
    async fn auth_accepts_matching_query() {
        let state = WsServerState::new(WsServerConfig {
            auth_bearer_token: Some("secret".to_string()),
        });

        let allowed = state.authenticate(None, Some("secret")).await;
        assert!(allowed);
    }

    #[tokio::test]
    async fn auth_rejects_missing_when_required() {
        let state = WsServerState::new(WsServerConfig {
            auth_bearer_token: Some("secret".to_string()),
        });

        let allowed = state.authenticate(None, None).await;
        assert!(!allowed);
    }

    #[tokio::test]
    async fn client_lifecycle_register_and_unregister() {
        let state = WsServerState::new(WsServerConfig::default());
        let (id, _rx, _tx) = state.register_client(true).await;
        assert_eq!(state.connected_clients().await.len(), 1);
        state.unregister_client(&id).await;
        assert!(state.connected_clients().await.is_empty());
    }

    #[tokio::test]
    async fn keepalive_disconnects_stale_clients() {
        let state = WsServerState::new(WsServerConfig::default());
        let (id, _rx, _tx) = state.register_client(true).await;

        for _ in 0..=MAX_MISSED_PONGS {
            let _ = state.keepalive_tick().await;
        }

        let stale = state.keepalive_tick().await;
        assert!(stale.contains(&id));
    }

    #[tokio::test]
    async fn request_validation_checks_jsonrpc_version() {
        let state = WsServerState::new(WsServerConfig::default());
        let (id, _rx, _tx) = state.register_client(true).await;

        let err = state
            .handle_incoming_json(&id, r#"{"jsonrpc":"1.0","method":"health","params":{}}"#)
            .await
            .expect_err("must reject invalid version");
        assert_eq!(err.error.expect("error").code, -32600);
    }
}
