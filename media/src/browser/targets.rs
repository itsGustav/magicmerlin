use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::time::{sleep, Duration, Instant};

use super::TabInfo;
use crate::{MediaError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TargetInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub title: String,
    pub url: String,
    pub attached: bool,
    pub ws_url: Option<String>,
}

pub async fn list_targets(port: u16) -> Result<Vec<TargetInfo>> {
    let url = format!("http://127.0.0.1:{port}/json/list");
    let response = reqwest::Client::new().get(url).send().await?;
    let status = response.status();
    let body = response.text().await?;
    if !status.is_success() {
        return Err(MediaError::Execution(format!(
            "target list failed with {status}: {body}"
        )));
    }

    let raw: Vec<Value> = serde_json::from_str(&body).map_err(|e| {
        MediaError::Execution(format!("target list JSON parse failed: {e}; body={body}"))
    })?;

    Ok(raw
        .into_iter()
        .map(|item| TargetInfo {
            id: item
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            kind: item
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("page")
                .to_string(),
            title: item
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            url: item
                .get("url")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            attached: item
                .get("attached")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            ws_url: item
                .get("webSocketDebuggerUrl")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
        })
        .collect())
}

pub async fn focus_tab(port: u16, tab_id: &str) -> Result<()> {
    let url = format!("http://127.0.0.1:{port}/json/activate/{tab_id}");
    let response = reqwest::Client::new().get(url).send().await?;
    if response.status().is_success() {
        return Ok(());
    }
    Err(MediaError::Execution(format!(
        "activate tab failed with status {}",
        response.status()
    )))
}

pub async fn new_tab_with_wait(port: u16, navigate_to: &str, timeout: Duration) -> Result<TabInfo> {
    let url = format!("http://127.0.0.1:{port}/json/new?{navigate_to}");
    let response = reqwest::Client::new().put(url).send().await?;
    let status = response.status();
    let body = response.text().await?;
    if !status.is_success() {
        return Err(MediaError::Execution(format!(
            "new tab failed with {status}: {body}"
        )));
    }

    let created: Value = serde_json::from_str(&body)
        .map_err(|e| MediaError::Execution(format!("new tab parse failed: {e}; body={body}")))?;
    let id = created
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| MediaError::Execution("new tab response missing id".to_string()))?
        .to_string();

    let deadline = Instant::now() + timeout;
    loop {
        let targets = list_targets(port).await?;
        if let Some(t) = targets.into_iter().find(|t| t.id == id) {
            return Ok(TabInfo {
                id: t.id,
                title: t.title,
                url: t.url,
                web_socket_debugger_url: t.ws_url.unwrap_or_default(),
            });
        }
        if Instant::now() >= deadline {
            return Err(MediaError::Execution(
                "timeout waiting for new tab".to_string(),
            ));
        }
        sleep(Duration::from_millis(75)).await;
    }
}

pub async fn close_tab_with_retry(port: u16, tab_id: &str, timeout: Duration) -> Result<()> {
    let start = Instant::now();
    let client = reqwest::Client::new();
    loop {
        let url = format!("http://127.0.0.1:{port}/json/close/{tab_id}");
        let response = client.get(&url).send().await?;
        if response.status().is_success() {
            return Ok(());
        }
        if start.elapsed() > timeout {
            return Err(MediaError::Execution(format!(
                "close tab timeout for {}",
                tab_id
            )));
        }
        sleep(Duration::from_millis(100)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_info_deserializes() {
        let value = serde_json::json!({
            "id": "123",
            "type": "page",
            "title": "hello",
            "url": "https://example.com",
            "attached": false,
            "webSocketDebuggerUrl": "ws://127.0.0.1:9222/devtools/page/123"
        });
        let item = TargetInfo {
            id: value["id"].as_str().unwrap_or_default().to_string(),
            kind: value["type"].as_str().unwrap_or_default().to_string(),
            title: value["title"].as_str().unwrap_or_default().to_string(),
            url: value["url"].as_str().unwrap_or_default().to_string(),
            attached: value["attached"].as_bool().unwrap_or(false),
            ws_url: value["webSocketDebuggerUrl"]
                .as_str()
                .map(ToOwned::to_owned),
        };
        assert_eq!(item.kind, "page");
        assert!(item.ws_url.is_some());
    }
}
