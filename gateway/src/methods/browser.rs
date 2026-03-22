use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserNavigateParams {
    pub url: String,
    #[serde(default)]
    pub tab_id: Option<String>,
    #[serde(default)]
    pub wait_until: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserScreenshotParams {
    #[serde(default)]
    pub tab_id: Option<String>,
    #[serde(default)]
    pub full_page: Option<bool>,
    #[serde(default)]
    pub format: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserActParams {
    pub action: String,
    #[serde(default)]
    pub selector: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub tab_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserSnapshotParams {
    #[serde(default)]
    pub tab_id: Option<String>,
}
