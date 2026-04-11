//! Static fallback tool definitions (used when daemon is unreachable).
//! Also includes synthetic tools that orchestrate multiple endpoints.

use serde_json::json;

use crate::registry::{HttpMethod, ToolDef};
use crate::ring::Ring;

mod extended;

pub fn all_defs() -> Vec<ToolDef> {
    let mut defs = Vec::with_capacity(16);
    defs.push(ceo_tool_def());
    defs.extend(synthetic_workflow_defs());
    defs.extend(extended::action_defs());
    defs
}

/// The CEO tool — routes natural-language instructions to internal endpoints.
fn ceo_tool_def() -> ToolDef {
    ToolDef {
        name: "cvg_ceo".into(),
        description: "Send a natural language instruction to Convergio. \
                       Routes to the correct internal tool automatically. \
                       Use this for ALL operations."
            .into(),
        method: HttpMethod::Post,
        path: "/api/ceo".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "instruction": {
                    "type": "string",
                    "description": "What you want Convergio to do"
                },
                "context": {
                    "type": "object",
                    "description": "Optional context: org_id, plan_id, etc."
                }
            },
            "required": ["instruction"]
        }),
        min_ring: Ring::Sandboxed,
        path_params: vec![],
    }
}

/// Synthetic tools that compose multiple daemon endpoints
/// (not discoverable from a single Extension).
fn synthetic_workflow_defs() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "cvg_solve".into(),
            description: "REQUIRED first step. Analyze a problem before planning.".into(),
            method: HttpMethod::Post,
            path: "/api/workflow/solve".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {"type": "string"},
                    "problem_description": {"type": "string"}
                },
                "required": ["project_id", "problem_description"]
            }),
            min_ring: Ring::Trusted,
            path_params: vec![],
        },
        ToolDef {
            name: "cvg_create_plan".into(),
            description: "Create execution plan from solve output.".into(),
            method: HttpMethod::Post,
            path: "/api/workflow/plan".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "solve_session_id": {"type": "string"},
                    "plan_name": {"type": "string"},
                    "project_id": {"type": "string"}
                },
                "required": ["plan_name"]
            }),
            min_ring: Ring::Trusted,
            path_params: vec![],
        },
        ToolDef {
            name: "cvg_execute_plan".into(),
            description: "Execute an approved plan.".into(),
            method: HttpMethod::Post,
            path: "/api/workflow/execute".into(),
            input_schema: json!({
                "type": "object",
                "properties": {"plan_id": {"type": "integer"}},
                "required": ["plan_id"]
            }),
            min_ring: Ring::Trusted,
            path_params: vec![],
        },
        ToolDef {
            name: "cvg_how_to".into(),
            description: "Ask Convergio how to do something.".into(),
            method: HttpMethod::Get,
            path: "/api/workflow/howto".into(),
            input_schema: json!({
                "type": "object",
                "properties": {"question": {"type": "string"}},
                "required": ["question"]
            }),
            min_ring: Ring::Sandboxed,
            path_params: vec![],
        },
    ]
}
