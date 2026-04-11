//! Tool profile filtering for MCP context optimization.
//!
//! Profiles control which tools are exposed to the LLM client.
//! `Compact` exposes ~30 essential tools, saving ~10K tokens of context.
//! `Full` exposes all tools (default for backward compatibility).

use crate::registry::ToolDef;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    /// All tools visible (default).
    Full,
    /// Essential tools only (~30). Saves ~10K tokens.
    Compact,
    /// Single cvg_ceo tool — routes everything via LLM.
    Ceo,
}

impl Profile {
    pub fn from_env() -> Self {
        match std::env::var("CONVERGIO_MCP_PROFILE")
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "compact" | "c" => Self::Compact,
            "ceo" => Self::Ceo,
            _ => Self::Full,
        }
    }
}

/// Tools included in compact profile. Covers the full agent workflow:
/// solve → plan → execute → evidence → validate, plus essential ops.
const COMPACT_TOOLS: &[&str] = &[
    // Workflow pipeline
    "cvg_solve",
    "cvg_create_plan",
    "cvg_execute_plan",
    "cvg_how_to",
    "cvg_help",
    // Health & diagnostics
    "cvg_health",
    "cvg_doctor_run",
    "cvg_node_readiness",
    // Plan lifecycle
    "cvg_list_plans",
    "cvg_get_plan",
    "cvg_get_execution_tree",
    "cvg_start_plan",
    "cvg_cancel_plan",
    "cvg_resume_plan",
    "cvg_force_resume_plan",
    "cvg_force_complete_plan",
    "cvg_plan_readiness",
    "cvg_plan_import",
    "cvg_validate_plan",
    "cvg_review_plan",
    // Task execution (gate chain)
    "cvg_update_task",
    "cvg_complete_task",
    "cvg_wave_complete",
    "cvg_record_evidence",
    "cvg_evidence_preflight",
    // Agent management
    "cvg_spawn_agent",
    "cvg_list_agent_catalog",
    "cvg_get_agent",
    // Workspace + code graph
    "cvg_check_worktree_owner",
    "cvg_workspace_gc",
    "cvg_codegraph_expand",
    "cvg_codegraph_package_deps",
    // Knowledge store
    "cvg_knowledge_search",
    "cvg_knowledge_write",
    // Org basics
    "cvg_list_orgs",
    "cvg_get_org",
    // Delegation
    "cvg_delegate_spawn",
    "cvg_delegation_status",
    "cvg_delegation_list",
    // Misc essentials
    "cvg_notify",
    "cvg_cost_summary",
];

/// Check if a tool name is included in the given profile.
pub fn is_in_profile(name: &str, profile: Profile) -> bool {
    match profile {
        Profile::Full => true,
        Profile::Compact => COMPACT_TOOLS.contains(&name),
        Profile::Ceo => name == "cvg_ceo",
    }
}

/// Filter tool list by profile.
pub fn filter_by_profile(defs: &[ToolDef], profile: Profile) -> Vec<&ToolDef> {
    match profile {
        Profile::Full => defs.iter().collect(),
        Profile::Compact => defs
            .iter()
            .filter(|t| COMPACT_TOOLS.contains(&t.name.as_str()))
            .collect(),
        Profile::Ceo => defs.iter().filter(|t| t.name == "cvg_ceo").collect(),
    }
}

#[cfg(test)]
mod profile_tests {
    use super::*;
    use crate::registry::HttpMethod;
    use crate::ring::Ring;
    use serde_json::json;

    fn make_tool(name: &str) -> ToolDef {
        ToolDef {
            name: name.into(),
            description: "test".into(),
            method: HttpMethod::Get,
            path: "/test".into(),
            input_schema: json!({}),
            min_ring: Ring::Trusted,
            path_params: vec![],
        }
    }

    #[test]
    fn compact_filters_non_essential() {
        let tools: Vec<ToolDef> = vec![
            make_tool("cvg_health"),
            make_tool("cvg_solve"),
            make_tool("cvg_list_snapshots"),
            make_tool("cvg_tenancy_audit"),
        ];
        let filtered = filter_by_profile(&tools, Profile::Compact);
        let names: Vec<&str> = filtered.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"cvg_health"));
        assert!(names.contains(&"cvg_solve"));
        assert!(!names.contains(&"cvg_list_snapshots"));
        assert!(!names.contains(&"cvg_tenancy_audit"));
    }

    #[test]
    fn full_passes_everything() {
        let tools: Vec<ToolDef> = vec![make_tool("cvg_health"), make_tool("cvg_list_snapshots")];
        let filtered = filter_by_profile(&tools, Profile::Full);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn compact_whitelist_has_reasonable_size() {
        assert!(COMPACT_TOOLS.len() >= 25);
        assert!(COMPACT_TOOLS.len() <= 45);
    }

    #[test]
    fn ceo_profile_exposes_only_cvg_ceo() {
        let tools: Vec<ToolDef> = vec![
            make_tool("cvg_ceo"),
            make_tool("cvg_health"),
            make_tool("cvg_solve"),
        ];
        let filtered = filter_by_profile(&tools, Profile::Ceo);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "cvg_ceo");
    }

    #[test]
    fn ceo_profile_is_in_profile() {
        assert!(is_in_profile("cvg_ceo", Profile::Ceo));
        assert!(!is_in_profile("cvg_health", Profile::Ceo));
        assert!(!is_in_profile("cvg_solve", Profile::Ceo));
    }
}
