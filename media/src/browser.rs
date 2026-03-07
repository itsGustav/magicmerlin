use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;

use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration, Instant};
use url::Url;

use crate::{MediaError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TabInfo {
    pub id: String,
    pub title: String,
    pub url: String,
    pub web_socket_debugger_url: String,
}

#[derive(Debug, Clone)]
pub struct BrowserLaunchOptions {
    pub chrome_binary: String,
    pub remote_debugging_port: u16,
    pub headless: bool,
    pub user_data_dir: Option<PathBuf>,
    pub extra_args: Vec<String>,
    pub startup_timeout: Duration,
}

impl Default for BrowserLaunchOptions {
    fn default() -> Self {
        Self {
            chrome_binary: std::env::var("CHROME_BIN")
                .unwrap_or_else(|_| "google-chrome".to_string()),
            remote_debugging_port: 9222,
            headless: true,
            user_data_dir: None,
            extra_args: Vec::new(),
            startup_timeout: Duration::from_secs(10),
        }
    }
}

#[derive(Debug)]
struct WsConnection {
    stream: Arc<Mutex<TcpStream>>,
}

#[derive(Debug)]
pub struct BrowserClient {
    ws: WsConnection,
    next_id: Arc<Mutex<u64>>,
    pub endpoint: String,
}

#[derive(Debug)]
pub struct BrowserProcess {
    pub child: Child,
    pub debugging_url: String,
}

impl BrowserClient {
    pub async fn connect(ws_endpoint: &str) -> Result<Self> {
        let ws = WsConnection::connect(ws_endpoint).await?;
        Ok(Self {
            ws,
            next_id: Arc::new(Mutex::new(0)),
            endpoint: ws_endpoint.to_string(),
        })
    }

    pub async fn from_tab(port: u16) -> Result<Self> {
        let tabs = list_tabs(port).await?;
        let tab = tabs
            .into_iter()
            .find(|t| !t.web_socket_debugger_url.is_empty())
            .ok_or_else(|| MediaError::Execution("no debuggable tab available".to_string()))?;
        Self::connect(&tab.web_socket_debugger_url).await
    }

    pub async fn navigate(&self, url: &str) -> Result<()> {
        self.send_cdp("Page.enable", json!({})).await?;
        self.send_cdp("Page.navigate", json!({ "url": url }))
            .await?;
        Ok(())
    }

    pub async fn snapshot_text(&self) -> Result<String> {
        self.send_cdp("Accessibility.enable", json!({})).await?;
        let response = self
            .send_cdp("Accessibility.getFullAXTree", json!({}))
            .await?;
        Ok(ax_tree_to_text(&response))
    }

    pub async fn screenshot_png(&self) -> Result<Vec<u8>> {
        self.send_cdp("Page.enable", json!({})).await?;
        let result = self
            .send_cdp(
                "Page.captureScreenshot",
                json!({
                    "format": "png",
                    "captureBeyondViewport": true,
                }),
            )
            .await?;

        let encoded = result
            .pointer("/result/data")
            .and_then(Value::as_str)
            .ok_or_else(|| MediaError::Execution("missing screenshot data".to_string()))?;

        base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(|err| MediaError::Execution(format!("invalid screenshot base64: {err}")))
    }

    pub async fn click(&self, element_ref: &str) -> Result<()> {
        let expression = format!(
            "(() => {{ const el = document.querySelector({selector:?}); if (!el) return false; el.click(); return true; }})()",
            selector = element_ref
        );
        self.eval_expect_true(&expression, "click").await
    }

    pub async fn type_text(&self, element_ref: &str, text: &str) -> Result<()> {
        let expression = format!(
            "(() => {{ const el = document.querySelector({selector:?}); if (!el) return false; el.focus(); el.value = {value:?}; el.dispatchEvent(new Event('input', {{ bubbles: true }})); return true; }})()",
            selector = element_ref,
            value = text,
        );
        self.eval_expect_true(&expression, "type_text").await
    }

    pub async fn press_key(&self, key: &str) -> Result<()> {
        let key_payload = json!({
            "type": "keyDown",
            "key": key,
            "windowsVirtualKeyCode": key_to_vk(key),
            "nativeVirtualKeyCode": key_to_vk(key),
        });
        self.send_cdp("Input.dispatchKeyEvent", key_payload).await?;
        let key_up = json!({
            "type": "keyUp",
            "key": key,
            "windowsVirtualKeyCode": key_to_vk(key),
            "nativeVirtualKeyCode": key_to_vk(key),
        });
        self.send_cdp("Input.dispatchKeyEvent", key_up).await?;
        Ok(())
    }

    pub async fn evaluate(&self, expression: &str) -> Result<Value> {
        self.send_cdp(
            "Runtime.evaluate",
            json!({
                "expression": expression,
                "returnByValue": true,
                "awaitPromise": true,
            }),
        )
        .await
    }

    async fn eval_expect_true(&self, expression: &str, action: &str) -> Result<()> {
        let result = self.evaluate(expression).await?;
        let maybe_true = result
            .pointer("/result/result/value")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if !maybe_true {
            return Err(MediaError::Execution(format!(
                "{action} failed for selector"
            )));
        }
        Ok(())
    }

    pub async fn send_cdp(&self, method: &str, params: Value) -> Result<Value> {
        let id = {
            let mut guard = self.next_id.lock().await;
            *guard += 1;
            *guard
        };
        let command = json!({
            "id": id,
            "method": method,
            "params": params,
        });

        self.ws.send_text(&command.to_string()).await?;

        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            if Instant::now() > deadline {
                return Err(MediaError::Execution(format!(
                    "timeout waiting for cdp response to {method}"
                )));
            }

            let payload = self.ws.recv_text().await?;
            let value: Value = serde_json::from_str(&payload)
                .map_err(|err| MediaError::Execution(format!("invalid cdp json: {err}")))?;
            if value.get("id").and_then(Value::as_u64) == Some(id) {
                if let Some(err) = value.get("error") {
                    return Err(MediaError::Execution(format!(
                        "cdp error for {method}: {err}"
                    )));
                }
                return Ok(value);
            }
        }
    }
}

impl WsConnection {
    async fn connect(endpoint: &str) -> Result<Self> {
        let url = Url::parse(endpoint)
            .map_err(|err| MediaError::InvalidInput(format!("invalid websocket URL: {err}")))?;
        if url.scheme() != "ws" {
            return Err(MediaError::InvalidInput(
                "only ws:// endpoints are supported".to_string(),
            ));
        }

        let host = url
            .host_str()
            .ok_or_else(|| MediaError::InvalidInput("websocket host missing".to_string()))?;
        let port = url
            .port_or_known_default()
            .ok_or_else(|| MediaError::InvalidInput("websocket port missing".to_string()))?;
        let path = if url.path().is_empty() {
            "/"
        } else {
            url.path()
        };
        let query = url.query().map(|q| format!("?{q}")).unwrap_or_default();
        let full_path = format!("{path}{query}");

        let mut stream = TcpStream::connect((host, port))
            .await
            .map_err(|err| MediaError::Execution(format!("tcp connect failed: {err}")))?;

        let key = generate_websocket_key();
        let request = format!(
            "GET {full_path} HTTP/1.1\r\nHost: {host}:{port}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n"
        );

        stream
            .write_all(request.as_bytes())
            .await
            .map_err(|err| MediaError::Execution(format!("ws handshake write failed: {err}")))?;

        let mut response = Vec::new();
        let mut buf = [0u8; 1024];
        loop {
            let n = stream
                .read(&mut buf)
                .await
                .map_err(|err| MediaError::Execution(format!("ws handshake read failed: {err}")))?;
            if n == 0 {
                return Err(MediaError::Execution(
                    "websocket handshake closed unexpectedly".to_string(),
                ));
            }
            response.extend_from_slice(&buf[..n]);
            if response.windows(4).any(|w| w == b"\r\n\r\n") {
                break;
            }
            if response.len() > 8192 {
                return Err(MediaError::Execution(
                    "websocket handshake too large".to_string(),
                ));
            }
        }

        let header_text = String::from_utf8_lossy(&response);
        if !header_text.starts_with("HTTP/1.1 101") && !header_text.starts_with("HTTP/1.0 101") {
            return Err(MediaError::Execution(format!(
                "websocket handshake failed: {header_text}"
            )));
        }

        Ok(Self {
            stream: Arc::new(Mutex::new(stream)),
        })
    }

    async fn send_text(&self, text: &str) -> Result<()> {
        let mut stream = self.stream.lock().await;
        let frame = encode_ws_text_frame(text.as_bytes());
        stream
            .write_all(&frame)
            .await
            .map_err(|err| MediaError::Execution(format!("ws send failed: {err}")))
    }

    async fn recv_text(&self) -> Result<String> {
        let mut stream = self.stream.lock().await;
        loop {
            let message = decode_ws_frame(&mut stream).await?;
            match message {
                WsMessage::Text(text) => return Ok(text),
                WsMessage::Binary => continue,
                WsMessage::Ping(data) => {
                    let pong = encode_ws_control_frame(0xA, &data);
                    stream
                        .write_all(&pong)
                        .await
                        .map_err(|err| MediaError::Execution(format!("ws pong failed: {err}")))?;
                }
                WsMessage::Pong => continue,
                WsMessage::Close => {
                    return Err(MediaError::Execution(
                        "websocket closed by peer".to_string(),
                    ));
                }
            }
        }
    }
}

#[derive(Debug)]
enum WsMessage {
    Text(String),
    Binary,
    Ping(Vec<u8>),
    Pong,
    Close,
}

async fn decode_ws_frame(stream: &mut TcpStream) -> Result<WsMessage> {
    let mut header = [0u8; 2];
    stream
        .read_exact(&mut header)
        .await
        .map_err(|err| MediaError::Execution(format!("ws read header failed: {err}")))?;

    let opcode = header[0] & 0x0F;
    let masked = (header[1] & 0x80) != 0;
    let mut len = (header[1] & 0x7F) as u64;

    if len == 126 {
        let mut ext = [0u8; 2];
        stream
            .read_exact(&mut ext)
            .await
            .map_err(|err| MediaError::Execution(format!("ws read ext16 failed: {err}")))?;
        len = u16::from_be_bytes(ext) as u64;
    } else if len == 127 {
        let mut ext = [0u8; 8];
        stream
            .read_exact(&mut ext)
            .await
            .map_err(|err| MediaError::Execution(format!("ws read ext64 failed: {err}")))?;
        len = u64::from_be_bytes(ext);
    }

    let mut mask = [0u8; 4];
    if masked {
        stream
            .read_exact(&mut mask)
            .await
            .map_err(|err| MediaError::Execution(format!("ws read mask failed: {err}")))?;
    }

    let mut payload = vec![0u8; len as usize];
    if len > 0 {
        stream
            .read_exact(&mut payload)
            .await
            .map_err(|err| MediaError::Execution(format!("ws read payload failed: {err}")))?;
    }

    if masked {
        for (index, byte) in payload.iter_mut().enumerate() {
            *byte ^= mask[index % 4];
        }
    }

    match opcode {
        0x1 => {
            let text = String::from_utf8(payload)
                .map_err(|err| MediaError::Execution(format!("ws text decode failed: {err}")))?;
            Ok(WsMessage::Text(text))
        }
        0x2 => Ok(WsMessage::Binary),
        0x8 => Ok(WsMessage::Close),
        0x9 => Ok(WsMessage::Ping(payload)),
        0xA => Ok(WsMessage::Pong),
        _ => Ok(WsMessage::Binary),
    }
}

fn encode_ws_text_frame(data: &[u8]) -> Vec<u8> {
    encode_ws_frame(0x1, data)
}

fn encode_ws_control_frame(opcode: u8, data: &[u8]) -> Vec<u8> {
    encode_ws_frame(opcode, data)
}

fn encode_ws_frame(opcode: u8, data: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(2 + data.len() + 16);
    frame.push(0x80 | (opcode & 0x0F));

    let mask_bit = 0x80u8;
    if data.len() < 126 {
        frame.push(mask_bit | data.len() as u8);
    } else if data.len() <= 65535 {
        frame.push(mask_bit | 126);
        frame.extend_from_slice(&(data.len() as u16).to_be_bytes());
    } else {
        frame.push(mask_bit | 127);
        frame.extend_from_slice(&(data.len() as u64).to_be_bytes());
    }

    let mask = [0x13u8, 0x37u8, 0xC0u8, 0xDEu8];
    frame.extend_from_slice(&mask);
    for (index, b) in data.iter().enumerate() {
        frame.push(*b ^ mask[index % 4]);
    }
    frame
}

fn generate_websocket_key() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let mut seed = [0u8; 16];
    for (index, slot) in seed.iter_mut().enumerate() {
        *slot = ((nanos >> ((index % 8) * 8)) & 0xFF) as u8 ^ (index as u8).wrapping_mul(17);
    }
    base64::engine::general_purpose::STANDARD.encode(seed)
}

pub async fn start_chrome(options: BrowserLaunchOptions) -> Result<BrowserProcess> {
    let mut command = Command::new(&options.chrome_binary);
    command
        .arg(format!(
            "--remote-debugging-port={}",
            options.remote_debugging_port
        ))
        .arg("--disable-gpu")
        .arg("--no-first-run")
        .arg("--no-default-browser-check")
        .arg("--disable-background-networking")
        .arg("--disable-sync")
        .arg("about:blank")
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    if options.headless {
        command.arg("--headless=new");
    }
    if let Some(dir) = &options.user_data_dir {
        command.arg(format!("--user-data-dir={}", dir.display()));
    }
    for arg in &options.extra_args {
        command.arg(arg);
    }

    let child = command
        .spawn()
        .map_err(|err| MediaError::Execution(format!("failed to launch chrome: {err}")))?;

    wait_for_debugging_endpoint(options.remote_debugging_port, options.startup_timeout).await?;

    Ok(BrowserProcess {
        child,
        debugging_url: format!("http://127.0.0.1:{}", options.remote_debugging_port),
    })
}

pub async fn stop_chrome(process: &mut BrowserProcess) -> Result<()> {
    process
        .child
        .kill()
        .await
        .map_err(|err| MediaError::Execution(format!("chrome kill failed: {err}")))?;
    Ok(())
}

pub async fn wait_for_debugging_endpoint(port: u16, timeout: Duration) -> Result<()> {
    let http = reqwest::Client::new();
    let started = Instant::now();
    loop {
        if started.elapsed() > timeout {
            return Err(MediaError::Execution(format!(
                "chrome debugging endpoint did not start on port {port}"
            )));
        }
        let url = format!("http://127.0.0.1:{port}/json/version");
        if let Ok(response) = http.get(&url).send().await {
            if response.status().is_success() {
                return Ok(());
            }
        }
        sleep(Duration::from_millis(100)).await;
    }
}

pub async fn list_tabs(port: u16) -> Result<Vec<TabInfo>> {
    let url = format!("http://127.0.0.1:{port}/json/list");
    let response = reqwest::Client::new().get(url).send().await?;
    let raw: Vec<Value> = response.json().await?;
    let tabs = raw
        .into_iter()
        .filter_map(|item| {
            Some(TabInfo {
                id: item.get("id")?.as_str()?.to_string(),
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
                web_socket_debugger_url: item
                    .get("webSocketDebuggerUrl")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            })
        })
        .collect();
    Ok(tabs)
}

pub async fn new_tab(port: u16, navigate_to: Option<&str>) -> Result<TabInfo> {
    let target = navigate_to.unwrap_or("about:blank");
    let url = format!("http://127.0.0.1:{port}/json/new?{target}");
    let response = reqwest::Client::new()
        .put(url)
        .send()
        .await
        .map_err(|err| MediaError::Execution(format!("new tab request failed: {err}")))?;
    let item: Value = response.json().await?;
    tab_from_value(item)
}

pub async fn close_tab(port: u16, target_id: &str) -> Result<()> {
    let url = format!("http://127.0.0.1:{port}/json/close/{target_id}");
    let response = reqwest::Client::new().get(url).send().await?;
    if !response.status().is_success() {
        return Err(MediaError::Execution(format!(
            "close tab failed with status {}",
            response.status()
        )));
    }
    Ok(())
}

fn tab_from_value(item: Value) -> Result<TabInfo> {
    Ok(TabInfo {
        id: item
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| MediaError::Execution("tab id missing".to_string()))?
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
        web_socket_debugger_url: item
            .get("webSocketDebuggerUrl")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
    })
}

fn key_to_vk(key: &str) -> u32 {
    match key {
        "Enter" => 13,
        "Tab" => 9,
        "Escape" => 27,
        "Backspace" => 8,
        "ArrowLeft" => 37,
        "ArrowUp" => 38,
        "ArrowRight" => 39,
        "ArrowDown" => 40,
        _ => key.chars().next().map(|c| c as u32).unwrap_or(0),
    }
}

fn ax_tree_to_text(response: &Value) -> String {
    let nodes = response
        .pointer("/result/nodes")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut named_nodes = HashMap::new();
    for node in &nodes {
        let node_id = node
            .get("nodeId")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let role = node
            .pointer("/role/value")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let name = node
            .pointer("/name/value")
            .and_then(Value::as_str)
            .unwrap_or_default();

        if !name.trim().is_empty() {
            named_nodes.insert(node_id.to_string(), format!("[{role}] {name}"));
        }
    }

    let mut lines: Vec<_> = named_nodes.into_values().collect();
    lines.sort();
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_translation_supports_named_and_literal() {
        assert_eq!(key_to_vk("Enter"), 13);
        assert_eq!(key_to_vk("A"), 65);
        assert_eq!(key_to_vk(""), 0);
    }

    #[test]
    fn ax_tree_conversion_extracts_named_nodes() {
        let payload = json!({
            "result": {
                "nodes": [
                    {
                        "nodeId": "1",
                        "role": {"value": "button"},
                        "name": {"value": "Save"}
                    },
                    {
                        "nodeId": "2",
                        "role": {"value": "text"},
                        "name": {"value": ""}
                    },
                    {
                        "nodeId": "3",
                        "role": {"value": "heading"},
                        "name": {"value": "Dashboard"}
                    }
                ]
            }
        });

        let text = ax_tree_to_text(&payload);
        assert!(text.contains("[button] Save"));
        assert!(text.contains("[heading] Dashboard"));
        assert!(!text.contains("[text]"));
    }

    #[test]
    fn tab_parser_requires_id() {
        let tab = tab_from_value(json!({
            "id": "abc",
            "title": "Blank",
            "url": "about:blank",
            "webSocketDebuggerUrl": "ws://127.0.0.1:9222/devtools/page/abc"
        }))
        .expect("tab should parse");

        assert_eq!(tab.id, "abc");
        assert_eq!(tab.title, "Blank");
    }

    #[tokio::test]
    async fn wait_for_debugging_endpoint_times_out_when_unavailable() {
        let result = wait_for_debugging_endpoint(65530, Duration::from_millis(200)).await;
        assert!(result.is_err());
    }

    #[test]
    fn launch_options_defaults_are_stable() {
        let options = BrowserLaunchOptions::default();
        assert_eq!(options.remote_debugging_port, 9222);
        assert!(options.headless);
    }

    #[test]
    fn websocket_frame_encoding_sets_mask_bit() {
        let frame = encode_ws_text_frame(b"hello");
        assert_eq!(frame[0] & 0x0F, 0x1);
        assert_eq!(frame[1] & 0x80, 0x80);
    }
}
