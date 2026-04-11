//! E2E test: MCP ring enforcement on the tool registry.
//!
//! Validates ring-based access control across all ring levels using
//! the declarative tool registry (no daemon needed).

use convergio_mcp::registry::list_tool_defs_filtered;
use convergio_mcp::registry_defs::all_defs;
use convergio_mcp::ring::Ring;

// ── Ring enforcement on registry ─────────────────────────────────────────────

#[test]
fn sandboxed_ring_cannot_see_trusted_tools() {
    let defs = all_defs();
    let filtered = list_tool_defs_filtered(&defs, Ring::Sandboxed);
    let names: Vec<&str> = filtered.iter().map(|t| t.name.as_str()).collect();
    assert!(
        !names.contains(&"cvg_solve"),
        "sandboxed ring must not see cvg_solve (trusted)"
    );
}

#[test]
fn trusted_ring_sees_workflow_tools() {
    let defs = all_defs();
    let filtered = list_tool_defs_filtered(&defs, Ring::Trusted);
    let names: Vec<&str> = filtered.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"cvg_solve"));
    assert!(names.contains(&"cvg_create_plan"));
    assert!(names.contains(&"cvg_execute_plan"));
    assert!(names.contains(&"cvg_how_to"));
}

#[test]
fn core_ring_sees_all_tools() {
    let defs = all_defs();
    let all = list_tool_defs_filtered(&defs, Ring::Core);
    let trusted = list_tool_defs_filtered(&defs, Ring::Trusted);
    assert!(
        all.len() >= trusted.len(),
        "core ring must see at least as many tools as trusted"
    );
}

#[test]
fn ring_filtering_is_monotonic() {
    let defs = all_defs();
    let core = list_tool_defs_filtered(&defs, Ring::Core).len();
    let trusted = list_tool_defs_filtered(&defs, Ring::Trusted).len();
    let community = list_tool_defs_filtered(&defs, Ring::Community).len();
    let sandboxed = list_tool_defs_filtered(&defs, Ring::Sandboxed).len();
    assert!(core >= trusted, "core >= trusted");
    assert!(trusted >= community, "trusted >= community");
    assert!(community >= sandboxed, "community >= sandboxed");
}

#[test]
fn all_defs_have_valid_structure() {
    for def in &all_defs() {
        assert!(def.name.starts_with("cvg_"), "bad name: {}", def.name);
        assert!(!def.description.is_empty(), "empty desc: {}", def.name);
        assert!(!def.path.is_empty(), "empty path: {}", def.name);
    }
}
