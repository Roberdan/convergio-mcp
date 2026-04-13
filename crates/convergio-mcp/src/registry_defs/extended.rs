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
        // ── Plan/task management (orchestrator) ─────────────────────────
        ToolDef {
            name: "cvg_get_execution_tree".into(),
            description: "Get execution tree for a plan with waves and tasks.".into(),
            method: HttpMethod::Get,
            path: "/api/plan-db/execution-tree/:plan_id".into(),
            input_schema: json!({
                "type": "object",
                "properties": {"plan_id": {"type": "integer"}},
                "required": ["plan_id"]
            }),
            min_ring: Ring::Community,
            path_params: vec!["plan_id".into()],
        },
        ToolDef {
            name: "cvg_update_task".into(),
            description: "Update task status or notes. Status: pending, in_progress, submitted."
                .into(),
            method: HttpMethod::Post,
            path: "/api/plan-db/task/update".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "task_id": {"type": "integer"},
                    "status": {"type": "string", "enum": ["pending","in_progress","submitted"]},
                    "agent_id": {"type": "string"},
                    "notes": {"type": "string"},
                    "summary": {"type": "string"}
                },
                "required": ["task_id", "status", "agent_id"]
            }),
            min_ring: Ring::Trusted,
            path_params: vec![],
        },
        ToolDef {
            name: "cvg_complete_task".into(),
            description:
                "Atomically complete a task: set notes, record evidence, and submit.".into(),
            method: HttpMethod::Post,
            path: "/api/plan-db/task/complete-flow".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "task_db_id": {"type": "integer"},
                    "agent_id": {"type": "string"},
                    "pr_url": {"type": "string"},
                    "test_command": {"type": "string"},
                    "test_output": {"type": "string"},
                    "test_exit_code": {"type": "integer"},
                    "notes": {"type": "string"}
                },
                "required": ["task_db_id", "agent_id", "pr_url"]
            }),
            min_ring: Ring::Trusted,
            path_params: vec![],
        },
        ToolDef {
            name: "cvg_validate_plan".into(),
            description:
                "Run Thor validation for a plan. All wave tasks must be submitted first.".into(),
            method: HttpMethod::Post,
            path: "/api/plan-db/validate".into(),
            input_schema: json!({
                "type": "object",
                "properties": {"plan_id": {"type": "integer"}},
                "required": ["plan_id"]
            }),
            min_ring: Ring::Trusted,
            path_params: vec![],
        },
        // ── Night agents ────────────────────────────────────────────────
        ToolDef {
            name: "cvg_list_night_agents".into(),
            description: "List all night agent definitions with status.".into(),
            method: HttpMethod::Get,
            path: "/api/night-agents".into(),
            input_schema: json!({"type": "object", "properties": {}}),
            min_ring: Ring::Sandboxed,
            path_params: vec![],
        },
        ToolDef {
            name: "cvg_trigger_night_agent".into(),
            description: "Trigger a night agent run by definition ID.".into(),
            method: HttpMethod::Post,
            path: "/api/night-agents/:agent_id/trigger".into(),
            input_schema: json!({
                "type": "object",
                "properties": {"agent_id": {"type": "integer"}},
                "required": ["agent_id"]
            }),
            min_ring: Ring::Trusted,
            path_params: vec!["agent_id".into()],
        },
    ]
}
