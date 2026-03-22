use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemorySearchParams {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub agent: Option<String>,
}

fn default_limit() -> usize {
    20
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryGetParams {
    pub key: String,
    #[serde(default)]
    pub agent: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryListParams {
    #[serde(default = "default_list_limit")]
    pub limit: usize,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub prefix: Option<String>,
}

fn default_list_limit() -> usize {
    50
}
