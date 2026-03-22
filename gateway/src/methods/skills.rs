use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsListParams {
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsGetParams {
    pub name: String,
    #[serde(default)]
    pub agent: Option<String>,
}
