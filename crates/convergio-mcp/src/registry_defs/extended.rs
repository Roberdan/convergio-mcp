//! Synthetic action tools — system-level tools not tied to a single extension.

use serde_json::json;

use crate::registry::{HttpMethod, ToolDef};
use crate::ring::Ring;

pub fn action_defs() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "cvg_help".into(),
            description:
                "CALL THIS FIRST. Get mandatory workflow, key tools, and rules for using Convergio. You MUST follow the workflow described here.".into(),
            method: HttpMethod::Get,
            path: "/api/meta/mcp-help".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "tool": {
                        "type": "string",
                        "description": "Optional: pass a tool name (e.g. 'cvg_complete_task') to get its full schema, method, path, and parameters."
                    }
                }
            }),
            min_ring: Ring::Sandboxed,
            path_params: vec![],
        },
        ToolDef {
            name: "cvg_notify".into(),
            description: "Send a notification via configured channel.".into(),
            method: HttpMethod::Post,
            path: "/api/notify".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "message": {"type": "string"},
                    "title": {"type": "string"},
                    "severity": {"type": "string", "enum": ["info","warning","error"]}
                },
                "required": ["message"]
            }),
            min_ring: Ring::Trusted,
            path_params: vec![],
        },
        ToolDef {
            name: "cvg_health".into(),
            description: "Check if the Convergio daemon is running.".into(),
            method: HttpMethod::Get,
            path: "/api/health".into(),
            input_schema: json!({"type": "object", "properties": {}}),
            min_ring: Ring::Sandboxed,
            path_params: vec![],
        },
        ToolDef {
            name: "cvg_health_deep".into(),
            description: "Deep health check — component-level status.".into(),
            method: HttpMethod::Get,
            path: "/api/health/deep".into(),
            input_schema: json!({"type": "object", "properties": {}}),
            min_ring: Ring::Community,
            path_params: vec![],
        },
        ToolDef {
            name: "cvg_depgraph".into(),
            description: "Get the dependency graph of all extensions.".into(),
            method: HttpMethod::Get,
            path: "/api/depgraph".into(),
            input_schema: json!({"type": "object", "properties": {}}),
            min_ring: Ring::Community,
            path_params: vec![],
        },
        ToolDef {
            name: "cvg_depgraph_validate".into(),
            description: "Validate the dependency graph (detect cycles).".into(),
            method: HttpMethod::Get,
            path: "/api/depgraph/validate".into(),
            input_schema: json!({"type": "object", "properties": {}}),
            min_ring: Ring::Community,
            path_params: vec![],
        },
        ToolDef {
            name: "cvg_capabilities".into(),
            description: "List all extension capabilities.".into(),
            method: HttpMethod::Get,
            path: "/api/capabilities".into(),
            input_schema: json!({"type": "object", "properties": {}}),
            min_ring: Ring::Community,
            path_params: vec![],
        },
        ToolDef {
            name: "cvg_openapi".into(),
            description: "Get OpenAPI specification.".into(),
            method: HttpMethod::Get,
            path: "/api/openapi".into(),
            input_schema: json!({"type": "object", "properties": {}}),
            min_ring: Ring::Community,
            path_params: vec![],
        },
        ToolDef {
            name: "cvg_removal_check".into(),
            description: "Check impact of removing an extension module.".into(),
            method: HttpMethod::Get,
            path: "/api/depgraph/removal-check/:module_id".into(),
            input_schema: json!({
                "type": "object",
                "properties": {"module_id": {"type": "string"}},
                "required": ["module_id"]
            }),
            min_ring: Ring::Community,
            path_params: vec!["module_id".into()],
        },
    ]
}
