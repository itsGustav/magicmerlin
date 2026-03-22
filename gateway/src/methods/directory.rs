use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectorySearchParams {
    pub query: String,
    #[serde(default)]
    pub channel: Option<String>,
    #[serde(default = "default_dir_limit")]
    pub limit: usize,
}

fn default_dir_limit() -> usize {
    25
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryGetParams {
    pub id: String,
    #[serde(default)]
    pub channel: Option<String>,
}
