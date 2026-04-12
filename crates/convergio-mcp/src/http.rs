//! HTTP bridge to the Convergio daemon API.
//!
//! Each MCP tool handler calls the daemon over HTTP.
//! Uses reqwest::blocking with a 5-second timeout and a shared client.

use serde_json::Value;
use std::sync::OnceLock;
use std::time::Duration;

use crate::ring::McpError;

// ── Client ───────────────────────────────────────────────────────────────────

static HTTP_CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();

fn get_client() -> &'static reqwest::blocking::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new())
    })
}

fn map_send_error(e: reqwest::Error) -> McpError {
    if e.is_connect() || e.is_timeout() {
        McpError::DaemonUnreachable
    } else {
        McpError::DaemonError(e.to_string())
    }
}

fn handle_response(resp: reqwest::blocking::Response) -> Result<Value, McpError> {
    let status = resp.status();
    if !status.is_success() {
        let body_text = resp.text().unwrap_or_default();
        return Err(McpError::DaemonError(format!(
            "HTTP {}\n{}",
            status.as_u16(),
            body_text
        )));
    }
    resp.json::<Value>()
        .map_err(|e| McpError::DaemonError(e.to_string()))
}

/// GET request to daemon, returning parsed JSON.
pub fn http_get(url: &str, token: Option<&str>) -> Result<Value, McpError> {
    let mut req = get_client().get(url);
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    let resp = req.send().map_err(map_send_error)?;
    handle_response(resp)
}

/// POST request to daemon with JSON body, returning parsed JSON.
pub fn http_post(url: &str, token: Option<&str>, body: &Value) -> Result<Value, McpError> {
    let mut req = get_client().post(url).json(body);
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    let resp = req.send().map_err(map_send_error)?;
    handle_response(resp)
}

/// DELETE request to daemon, returning parsed JSON.
pub fn http_delete(url: &str, token: Option<&str>) -> Result<Value, McpError> {
    let mut req = get_client().delete(url);
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    let resp = req.send().map_err(map_send_error)?;
    handle_response(resp)
}
