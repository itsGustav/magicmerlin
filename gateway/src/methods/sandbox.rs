use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxStartParams {
    pub name: String,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub workspace: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxStopParams {
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxExecParams {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
}
