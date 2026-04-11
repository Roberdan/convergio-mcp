//! Tests for ring enforcement, tool registry, and bridge logic.

use crate::registry::list_tool_defs_filtered;
use crate::registry_defs::all_defs;
use crate::ring::{check_ring_access, McpError, Ring};

// ── Ring access ──────────────────────────────────────────────────────────────

#[test]
fn ring_core_can_access_all() {
    assert!(check_ring_access(Ring::Core, Ring::Core).is_ok());
    assert!(check_ring_access(Ring::Core, Ring::Trusted).is_ok());
    assert!(check_ring_access(Ring::Core, Ring::Community).is_ok());
    assert!(check_ring_access(Ring::Core, Ring::Sandboxed).is_ok());
}

#[test]
fn ring_trusted_cannot_access_core() {
    assert!(check_ring_access(Ring::Trusted, Ring::Core).is_err());
    assert!(check_ring_access(Ring::Trusted, Ring::Trusted).is_ok());
    assert!(check_ring_access(Ring::Trusted, Ring::Community).is_ok());
}

#[test]
fn ring_community_cannot_access_trusted() {
    assert!(check_ring_access(Ring::Community, Ring::Core).is_err());
    assert!(check_ring_access(Ring::Community, Ring::Trusted).is_err());
    assert!(check_ring_access(Ring::Community, Ring::Community).is_ok());
    assert!(check_ring_access(Ring::Community, Ring::Sandboxed).is_ok());
}

#[test]
fn ring_sandboxed_only_sandboxed() {
    assert!(check_ring_access(Ring::Sandboxed, Ring::Core).is_err());
    assert!(check_ring_access(Ring::Sandboxed, Ring::Trusted).is_err());
    assert!(check_ring_access(Ring::Sandboxed, Ring::Community).is_err());
    assert!(check_ring_access(Ring::Sandboxed, Ring::Sandboxed).is_ok());
}

#[test]
fn ring_violation_error_contains_levels() {
    let err = check_ring_access(Ring::Community, Ring::Trusted).unwrap_err();
    match err {
        McpError::RingViolation { caller, required } => {
            assert_eq!(caller, 2);
            assert_eq!(required, 1);
        }
        _ => panic!("expected RingViolation, got {err:?}"),
    }
}

#[test]
fn ring_default_is_sandboxed() {
    assert_eq!(Ring::default(), Ring::Sandboxed);
}

#[test]
fn ring_from_u8_overflow_becomes_sandboxed() {
    assert_eq!(Ring::from_u8(255), Ring::Sandboxed);
}

// ── Static tool registry (fallback defs) ─────────────────────────────────────

#[test]
fn static_defs_not_empty() {
    let defs = all_defs();
    assert!(!defs.is_empty(), "static fallback defs must not be empty");
}

#[test]
fn static_defs_all_have_cvg_prefix() {
    for def in &all_defs() {
        assert!(
            def.name.starts_with("cvg_"),
            "tool '{}' must start with cvg_",
            def.name
        );
    }
}

#[test]
fn static_defs_have_valid_schema() {
    for def in &all_defs() {
        assert!(!def.name.is_empty());
        assert!(!def.description.is_empty());
        assert_eq!(
            def.input_schema.get("type").and_then(|v| v.as_str()),
            Some("object"),
            "tool {} schema must have type:object",
            def.name
        );
    }
}

#[test]
fn workflow_tools_in_static_defs() {
    let defs = all_defs();
    let names: Vec<&str> = defs.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"cvg_solve"), "cvg_solve must be registered");
    assert!(names.contains(&"cvg_create_plan"));
    assert!(names.contains(&"cvg_execute_plan"));
    assert!(names.contains(&"cvg_how_to"));
}

#[test]
fn workflow_tools_have_required_fields() {
    let defs = all_defs();
    let solve = defs.iter().find(|t| t.name == "cvg_solve").unwrap();
    let required = solve
        .input_schema
        .get("required")
        .and_then(|r| r.as_array())
        .expect("cvg_solve must have required fields");
    let req_strs: Vec<&str> = required.iter().filter_map(|v| v.as_str()).collect();
    assert!(req_strs.contains(&"project_id"));
    assert!(req_strs.contains(&"problem_description"));
}

// ── Ring filtering on static defs ────────────────────────────────────────────

#[test]
fn ring_filtering_trusted_sees_workflow_tools() {
    let defs = all_defs();
    let filtered = list_tool_defs_filtered(&defs, Ring::Trusted);
    let names: Vec<&str> = filtered.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"cvg_solve"));
    assert!(names.contains(&"cvg_how_to"));
}

#[test]
fn ring_filtering_sandboxed_excludes_trusted() {
    let defs = all_defs();
    let filtered = list_tool_defs_filtered(&defs, Ring::Sandboxed);
    let names: Vec<&str> = filtered.iter().map(|t| t.name.as_str()).collect();
    assert!(
        names.contains(&"cvg_how_to"),
        "cvg_how_to must be visible at sandboxed"
    );
    assert!(
        !names.contains(&"cvg_solve"),
        "cvg_solve must NOT be visible at sandboxed"
    );
}

// ── McpError display ─────────────────────────────────────────────────────────

#[test]
fn mcp_error_display() {
    let e = McpError::DaemonUnreachable;
    assert!(e.to_string().contains("unreachable"));

    let e = McpError::InvalidParams("test");
    assert!(e.to_string().contains("test"));

    let e = McpError::DaemonError("bad".into());
    assert_eq!(e.json_rpc_code(), -32003);
}

#[test]
fn mcp_error_ring_violation_code() {
    let e = McpError::RingViolation {
        caller: 3,
        required: 0,
    };
    assert_eq!(e.json_rpc_code(), -32001);
    assert!(e.message().contains("3"));
    assert!(e.message().contains("0"));
}
