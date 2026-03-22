//! Shared gateway RPC helper used by all gateway-dispatch tools.

use std::time::Duration;

use serde_json::{json, Value};

use crate::error::{Result, ToolError};
use crate::registry::{ToolContext, ToolResult};

/// Derives gateway URL from env or config.
pub fn gateway_url(ctx: &ToolContext) -> String {
    if let Ok(url) = std::env::var("MAGICMERLIN_GATEWAY_URL") {
        return url;
    }
    let port = ctx.config.gateway.port.unwrap_or(18789);
    let bind = ctx.config.gateway.bind.as_deref().unwrap_or("127.0.0.1");
    format!("http://{bind}:{port}")
}

/// POSTs a JSON-RPC-style call to the gateway with a 30-second timeout.
pub async fn gateway_call(ctx: &ToolContext, method: &str, params: Value) -> Result<ToolResult> {
    let url = format!("{}/call", gateway_url(ctx));
    let body = json!({ "method": method, "params": params });
    let resp = reqwest::Client::new()
        .post(&url)
        .json(&body)
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| ToolError::Execution(format!("gateway unreachable: {e}")))?;
    let status = resp.status().as_u16();
    let value = resp
        .json::<Value>()
        .await
        .unwrap_or_else(|_| json!({"error": "non-json response"}));
    if status >= 400 {
        return Ok(ToolResult::failure(format!(
            "gateway returned {status}: {}",
            serde_json::to_string(&value).unwrap_or_default()
        )));
    }
    Ok(ToolResult::success(value))
}
