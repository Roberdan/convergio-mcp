//! Declarative tool registry: ToolDef maps MCP tool names to daemon HTTP routes.
//!
//! Two sources of tool definitions:
//! 1. **Dynamic**: fetched from daemon at `/api/meta/mcp-tools` (auto-discovers extensions)
//! 2. **Static**: synthetic tools in `registry_defs` (composite workflows not tied to a single route)
//!
//! Dynamic tools are preferred and refreshed on each startup.

use serde_json::Value;

use crate::ring::Ring;

// ── ToolDef ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub method: HttpMethod,
    pub path: String,
    pub input_schema: Value,
    pub min_ring: Ring,
    pub path_params: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

// ── Dynamic discovery ────────────────────────────────────────────────────────

/// Fetch tool definitions from the daemon's `/api/meta/mcp-tools` endpoint.
/// Falls back to static defs from `registry_defs` if daemon is unreachable.
pub fn fetch_tool_defs(daemon_url: &str, token: Option<&str>) -> Vec<ToolDef> {
    let url = format!("{daemon_url}/api/meta/mcp-tools");
    let client = reqwest::blocking::Client::new();
    let mut req = client.get(&url);
    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {t}"));
    }

    let dynamic = match req.send().and_then(|r| r.json::<serde_json::Value>()) {
        Ok(body) => body
            .get("tools")
            .and_then(|t| t.as_array())
            .map(|tools| tools.iter().filter_map(parse_mcp_tool_def).collect())
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    };

    // Merge: dynamic tools take precedence, static fill gaps (synthetic tools)
    let static_defs = crate::registry_defs::all_defs();
    if dynamic.is_empty() {
        return static_defs;
    }
    let mut merged = dynamic;
    let names: std::collections::HashSet<String> = merged.iter().map(|t| t.name.clone()).collect();
    for def in static_defs {
        if !names.contains(&def.name) {
            merged.push(def);
        }
    }
    merged
}

fn parse_mcp_tool_def(v: &Value) -> Option<ToolDef> {
    let name = v.get("name")?.as_str()?;
    let description = v.get("description")?.as_str()?;
    let method = match v.get("method")?.as_str()? {
        "GET" => HttpMethod::Get,
        "POST" => HttpMethod::Post,
        "PUT" => HttpMethod::Put,
        "DELETE" => HttpMethod::Delete,
        _ => return None,
    };
    let ring = match v.get("min_ring")?.as_str()? {
        "sandboxed" => Ring::Sandboxed,
        "community" => Ring::Community,
        "trusted" => Ring::Trusted,
        "core" => Ring::Core,
        _ => Ring::Trusted,
    };
    let path_params: Vec<String> = v
        .get("path_params")
        .and_then(|p| p.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    Some(ToolDef {
        name: name.to_string(),
        description: description.to_string(),
        method,
        path: v.get("path")?.as_str()?.to_string(),
        input_schema: v
            .get("input_schema")
            .cloned()
            .unwrap_or(Value::Object(Default::default())),
        min_ring: ring,
        path_params,
    })
}

// ── Registry queries ─────────────────────────────────────────────────────────

pub fn list_tool_defs_filtered(defs: &[ToolDef], caller_ring: Ring) -> Vec<&ToolDef> {
    defs.iter()
        .filter(|t| caller_ring.can_access(t.min_ring))
        .collect()
}

pub fn find_tool_def<'a>(defs: &'a [ToolDef], name: &str) -> Option<&'a ToolDef> {
    defs.iter().find(|t| t.name == name)
}

// ── URL builder ──────────────────────────────────────────────────────────────

pub fn build_url(def: &ToolDef, args: &Value, daemon_url: &str) -> String {
    let mut path = def.path.clone();
    for param in &def.path_params {
        if let Some(val) = args.get(param.as_str()) {
            let replacement = match val {
                Value::Number(n) => n.to_string(),
                Value::String(s) => s.clone(),
                _ => val.to_string(),
            };
            path = path.replace(&format!(":{param}"), &replacement);
        }
    }
    if def.method == HttpMethod::Get {
        if let Some(obj) = args.as_object() {
            let qs: Vec<String> = obj
                .iter()
                .filter(|(k, v)| !def.path_params.contains(k) && !v.is_null())
                .map(|(k, v)| {
                    let val = match v {
                        Value::String(s) => s.clone(),
                        _ => v.to_string(),
                    };
                    format!("{k}={val}")
                })
                .collect();
            if !qs.is_empty() {
                return format!("{daemon_url}{path}?{}", qs.join("&"));
            }
        }
    }
    format!("{daemon_url}{path}")
}
