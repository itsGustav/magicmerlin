use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogsTailParams {
    #[serde(default = "default_lines")]
    pub lines: usize,
    #[serde(default)]
    pub level: Option<String>,
    #[serde(default)]
    pub component: Option<String>,
}

fn default_lines() -> usize {
    100
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogsQueryParams {
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub level: Option<String>,
    #[serde(default)]
    pub component: Option<String>,
    #[serde(default = "default_query_limit")]
    pub limit: usize,
    #[serde(default)]
    pub since: Option<i64>,
    #[serde(default)]
    pub until: Option<i64>,
}

fn default_query_limit() -> usize {
    200
}
