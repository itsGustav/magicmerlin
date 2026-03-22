use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HooksAddParams {
    pub url: String,
    #[serde(default)]
    pub events: Option<Vec<String>>,
    #[serde(default)]
    pub secret: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HooksRemoveParams {
    pub url: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HooksTestParams {
    pub url: String,
    #[serde(default)]
    pub event: Option<String>,
}
