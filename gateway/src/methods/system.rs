use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SystemEventParams {
    pub name: String,
    #[serde(default)]
    pub payload: Value,
    #[serde(default)]
    pub target_client: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HeartbeatParams {
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresenceResponse {
    pub online: bool,
    pub connected_clients: usize,
    pub last_heartbeat_at: i64,
}
