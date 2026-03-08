use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CronIdParams {
    pub id: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CronAddParams {
    pub name: String,
    pub schedule: String,
    pub kind: String,
    pub payload: Value,
    #[serde(default)]
    pub max_attempts: Option<i64>,
    #[serde(default)]
    pub backoff_seconds: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CronEditParams {
    pub id: i64,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub schedule: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub payload: Option<Value>,
    #[serde(default)]
    pub max_attempts: Option<i64>,
    #[serde(default)]
    pub backoff_seconds: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CronRunsParams {
    #[serde(default)]
    pub job_id: Option<i64>,
    #[serde(default = "default_runs_limit")]
    pub limit: usize,
}

fn default_runs_limit() -> usize {
    50
}
