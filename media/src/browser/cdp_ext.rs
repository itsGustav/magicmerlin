use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::{ax_tree_to_text, BrowserClient};
use crate::{MediaError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BrowserElementRef {
    pub node_id: i64,
    pub backend_node_id: i64,
    pub object_id: Option<String>,
    pub selector: Option<String>,
    pub text: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BrowserSnapshot {
    pub aria_text: String,
    pub dom_node_count: usize,
    pub title: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClickOptions {
    pub x: f64,
    pub y: f64,
    #[serde(default = "default_click_count")]
    pub click_count: u8,
    #[serde(default)]
    pub button: String,
}

fn default_click_count() -> u8 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputTextOptions {
    pub text: String,
    #[serde(default)]
    pub submit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyInput {
    pub key: String,
    #[serde(default)]
    pub modifiers: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvaluateOptions {
    pub expression: String,
    #[serde(default)]
    pub await_promise: bool,
    #[serde(default)]
    pub return_by_value: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum BrowserAction {
    Navigate { url: String },
    Snapshot,
    Screenshot { format: String, quality: Option<u8> },
    Evaluate { expression: String },
    Click { x: f64, y: f64 },
    Type { text: String },
    Press { key: String },
}

impl BrowserClient {
    pub async fn build_snapshot(&self) -> Result<BrowserSnapshot> {
        let aria_payload = self
            .send_cdp("Accessibility.getFullAXTree", json!({}))
            .await?;
        let aria_text = ax_tree_to_text(&aria_payload);

        let doc = self
            .send_cdp("DOM.getDocument", json!({"depth": -1}))
            .await?;
        let dom_node_count =
            count_dom_nodes(doc.pointer("/result/root").ok_or_else(|| {
                MediaError::Execution("DOM root missing in snapshot".to_string())
            })?);

        let eval = self
            .send_cdp(
                "Runtime.evaluate",
                json!({
                    "expression": "({ title: document.title, url: location.href })",
                    "returnByValue": true,
                    "awaitPromise": false
                }),
            )
            .await?;

        let title = eval
            .pointer("/result/result/value/title")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        let url = eval
            .pointer("/result/result/value/url")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);

        Ok(BrowserSnapshot {
            aria_text,
            dom_node_count,
            title,
            url,
        })
    }

    pub async fn capture_screenshot(
        &self,
        format: &str,
        quality: Option<u8>,
        clip: Option<Value>,
    ) -> Result<Vec<u8>> {
        let mut params = json!({
            "format": format,
            "captureBeyondViewport": true,
        });
        if let Some(q) = quality {
            params["quality"] = json!(q.min(100));
        }
        if let Some(clip_value) = clip {
            params["clip"] = clip_value;
        }
        let result = self.send_cdp("Page.captureScreenshot", params).await?;
        let encoded = result
            .pointer("/result/data")
            .and_then(Value::as_str)
            .ok_or_else(|| MediaError::Execution("missing screenshot data".to_string()))?;
        base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(|e| MediaError::Execution(format!("invalid screenshot base64: {e}")))
    }

    pub async fn evaluate_script(&self, options: EvaluateOptions) -> Result<Value> {
        self.send_cdp(
            "Runtime.evaluate",
            json!({
                "expression": options.expression,
                "returnByValue": options.return_by_value,
                "awaitPromise": options.await_promise,
            }),
        )
        .await
    }

    pub async fn click_at(&self, options: ClickOptions) -> Result<()> {
        let button = if options.button.trim().is_empty() {
            "left"
        } else {
            options.button.as_str()
        };

        for event_type in ["mouseMoved", "mousePressed", "mouseReleased"] {
            self.send_cdp(
                "Input.dispatchMouseEvent",
                json!({
                    "type": event_type,
                    "x": options.x,
                    "y": options.y,
                    "button": button,
                    "clickCount": options.click_count.max(1),
                }),
            )
            .await?;
        }
        Ok(())
    }

    pub async fn type_into_focused(&self, options: InputTextOptions) -> Result<()> {
        for chunk in options.text.chars() {
            self.send_cdp(
                "Input.dispatchKeyEvent",
                json!({
                    "type": "char",
                    "text": chunk.to_string(),
                    "unmodifiedText": chunk.to_string(),
                }),
            )
            .await?;
        }

        if options.submit {
            self.send_key(KeyInput {
                key: "Enter".to_string(),
                modifiers: 0,
            })
            .await?;
        }

        Ok(())
    }

    pub async fn send_key(&self, input: KeyInput) -> Result<()> {
        let code = virtual_key_code(&input.key);
        self.send_cdp(
            "Input.dispatchKeyEvent",
            json!({
                "type": "keyDown",
                "key": input.key,
                "modifiers": input.modifiers,
                "windowsVirtualKeyCode": code,
                "nativeVirtualKeyCode": code,
            }),
        )
        .await?;
        self.send_cdp(
            "Input.dispatchKeyEvent",
            json!({
                "type": "keyUp",
                "key": input.key,
                "modifiers": input.modifiers,
                "windowsVirtualKeyCode": code,
                "nativeVirtualKeyCode": code,
            }),
        )
        .await?;
        Ok(())
    }

    pub async fn resolve_element_by_selector(
        &self,
        selector: &str,
    ) -> Result<Option<BrowserElementRef>> {
        let expression = format!(
            "(() => {{ const el = document.querySelector({selector:?}); if (!el) return null; const r = el.getBoundingClientRect(); return {{ selector: {selector:?}, text: (el.innerText || el.textContent || '').trim().slice(0,200), role: el.getAttribute('role') || '', x: r.x, y: r.y }}; }})()"
        );
        let value = self
            .send_cdp(
                "Runtime.evaluate",
                json!({
                    "expression": expression,
                    "returnByValue": true,
                    "awaitPromise": false,
                }),
            )
            .await?;

        let data = value
            .pointer("/result/result/value")
            .cloned()
            .unwrap_or(Value::Null);
        if data.is_null() {
            return Ok(None);
        }

        Ok(Some(BrowserElementRef {
            node_id: 0,
            backend_node_id: 0,
            object_id: None,
            selector: data
                .get("selector")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
            text: data
                .get("text")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
            role: data
                .get("role")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
        }))
    }

    pub async fn run_action(&self, action: BrowserAction) -> Result<Value> {
        match action {
            BrowserAction::Navigate { url } => {
                self.navigate(&url).await?;
                Ok(json!({"ok": true, "navigated": url}))
            }
            BrowserAction::Snapshot => {
                let snapshot = self.build_snapshot().await?;
                Ok(serde_json::to_value(snapshot)?)
            }
            BrowserAction::Screenshot { format, quality } => {
                let bytes = self.capture_screenshot(&format, quality, None).await?;
                Ok(json!({"bytes": bytes.len(), "format": format}))
            }
            BrowserAction::Evaluate { expression } => {
                self.evaluate_script(EvaluateOptions {
                    expression,
                    await_promise: true,
                    return_by_value: true,
                })
                .await
            }
            BrowserAction::Click { x, y } => {
                self.click_at(ClickOptions {
                    x,
                    y,
                    click_count: 1,
                    button: "left".to_string(),
                })
                .await?;
                Ok(json!({"ok": true}))
            }
            BrowserAction::Type { text } => {
                self.type_into_focused(InputTextOptions {
                    text,
                    submit: false,
                })
                .await?;
                Ok(json!({"ok": true}))
            }
            BrowserAction::Press { key } => {
                self.send_key(KeyInput { key, modifiers: 0 }).await?;
                Ok(json!({"ok": true}))
            }
        }
    }
}

fn count_dom_nodes(node: &Value) -> usize {
    let mut count = 1;
    if let Some(children) = node.get("children").and_then(Value::as_array) {
        for child in children {
            count += count_dom_nodes(child);
        }
    }
    count
}

fn virtual_key_code(key: &str) -> u32 {
    match key {
        "Enter" => 13,
        "Tab" => 9,
        "Escape" => 27,
        "Backspace" => 8,
        "ArrowLeft" => 37,
        "ArrowUp" => 38,
        "ArrowRight" => 39,
        "ArrowDown" => 40,
        "Home" => 36,
        "End" => 35,
        "PageUp" => 33,
        "PageDown" => 34,
        "Delete" => 46,
        "Insert" => 45,
        "Space" => 32,
        _ => key.chars().next().map(|c| c as u32).unwrap_or(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vkey_maps_named_keys() {
        assert_eq!(virtual_key_code("Enter"), 13);
        assert_eq!(virtual_key_code("ArrowDown"), 40);
        assert_eq!(virtual_key_code("A"), 65);
    }

    #[test]
    fn dom_counter_handles_tree() {
        let tree = json!({
            "nodeName": "HTML",
            "children": [
                {"nodeName": "BODY", "children": [{"nodeName": "DIV"}]},
                {"nodeName": "SCRIPT"}
            ]
        });
        assert_eq!(count_dom_nodes(&tree), 4);
    }
}
