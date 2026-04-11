//! rmcp ServerHandler implementation for Convergio.
//!
//! Bridges MCP protocol to the Convergio daemon HTTP API.
//! At startup, fetches tool definitions dynamically from the daemon
//! via `/api/meta/mcp-tools`. Falls back to static defs if unreachable.
//! Ring-based access control filters tools per caller privilege level.

use rmcp::model::*;
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler};
use serde_json::json;
use std::sync::Arc;

use crate::bridge::dispatch_tool;
use crate::profile::{is_in_profile, Profile};
use crate::registry::{find_tool_def, list_tool_defs_filtered, ToolDef};
use crate::ring::{check_ring_access, Ring};

// ── Handler ──────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ConvergioHandler {
    ring: Ring,
    profile: Profile,
    daemon_url: String,
    api_token: Option<String>,
    /// Tool definitions fetched at startup and cached.
    tools: Arc<Vec<ToolDef>>,
}

impl ConvergioHandler {
    pub fn new(
        ring_level: u8,
        profile: Profile,
        daemon_url: &str,
        token: Option<&str>,
        tools: Vec<ToolDef>,
    ) -> Self {
        Self {
            ring: Ring::from_u8(ring_level),
            profile,
            daemon_url: daemon_url.to_string(),
            api_token: token.map(|t| t.to_string()),
            tools: Arc::new(tools),
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn value_to_schema(v: serde_json::Value) -> Arc<serde_json::Map<String, serde_json::Value>> {
    match v {
        serde_json::Value::Object(map) => Arc::new(map),
        _ => Arc::new(serde_json::Map::new()),
    }
}

// ── ServerHandler trait ──────────────────────────────────────────────────────

impl ServerHandler for ConvergioHandler {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.server_info = Implementation::from_build_env();
        info.instructions = Some(
            "Convergio daemon MCP server. Use tools to manage plans, \
             agents, mesh nodes, and more. Tools are auto-discovered \
             from daemon extensions."
                .into(),
        );
        info
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let tools: Vec<Tool> = list_tool_defs_filtered(&self.tools, self.ring)
            .into_iter()
            .filter(|t| is_in_profile(&t.name, self.profile))
            .map(|t| {
                Tool::new(
                    t.name.clone(),
                    t.description.clone(),
                    value_to_schema(t.input_schema.clone()),
                )
            })
            .collect();

        Ok(ListToolsResult {
            tools,
            next_cursor: None,
            meta: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let name = &request.name;
        let args = request
            .arguments
            .as_ref()
            .map(|m| serde_json::to_value(m).unwrap_or(json!({})))
            .unwrap_or(json!({}));

        let def = match find_tool_def(&self.tools, name) {
            Some(d) => d,
            None => {
                return Err(ErrorData::new(
                    ErrorCode::INVALID_REQUEST,
                    format!("Unknown tool: {name}"),
                    None::<serde_json::Value>,
                ));
            }
        };

        if let Err(e) = check_ring_access(self.ring, def.min_ring) {
            return Err(ErrorData::new(
                ErrorCode::INVALID_REQUEST,
                e.message(),
                None::<serde_json::Value>,
            ));
        }

        match dispatch_tool(def, &args, &self.daemon_url, self.api_token.as_deref()) {
            Ok(result) => {
                let text =
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string());
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.message())])),
        }
    }
}
