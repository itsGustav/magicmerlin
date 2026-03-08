use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentRunParams {
    pub session_id: String,
    pub message: String,
    pub timeout_seconds: Option<u64>,
    pub queue_timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentAbortParams {
    pub session_id: String,
}
