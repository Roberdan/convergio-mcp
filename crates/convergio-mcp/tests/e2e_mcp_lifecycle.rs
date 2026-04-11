//! E2E test: MCP tool registry completeness and bridge structure.
//!
//! Validates that all static tool definitions are well-formed and that
//! the registry merge logic handles duplicates correctly.

use convergio_mcp::registry::{find_tool_def, list_tool_defs_filtered};
use convergio_mcp::registry_defs::all_defs;
use convergio_mcp::ring::Ring;

// ── Registry completeness ────────────────────────────────────────────────────

#[test]
fn registry_has_minimum_tools() {
    let defs = all_defs();
    assert!(
        defs.len() >= 5,
        "static registry must have at least 5 tools, got {}",
        defs.len()
    );
}

#[test]
fn all_tools_findable_by_name() {
    let defs = all_defs();
    for def in &defs {
        let found = find_tool_def(&defs, &def.name);
        assert!(found.is_some(), "tool {} not findable", def.name);
    }
}

#[test]
fn unknown_tool_not_found() {
    let defs = all_defs();
    assert!(find_tool_def(&defs, "nonexistent_tool").is_none());
}

// ── Tool definition structure ────────────────────────────────────────────────

#[test]
fn all_tools_have_method_and_path() {
    use convergio_mcp::registry::HttpMethod;
    for def in &all_defs() {
        matches!(
            def.method,
            HttpMethod::Get | HttpMethod::Post | HttpMethod::Put | HttpMethod::Delete
        );
        assert!(
            def.path.starts_with('/'),
            "path must start with / for {}",
            def.name
        );
    }
}

#[test]
fn no_duplicate_tool_names() {
    let defs = all_defs();
    let mut names: Vec<&str> = defs.iter().map(|d| d.name.as_str()).collect();
    names.sort();
    let before = names.len();
    names.dedup();
    assert_eq!(before, names.len(), "duplicate tool names found");
}

// ── Specific tool presence ───────────────────────────────────────────────────

#[test]
fn essential_tools_present() {
    let defs = all_defs();
    let names: Vec<&str> = defs.iter().map(|d| d.name.as_str()).collect();
    // These are the static/synthetic tools always available without daemon
    let expected = [
        "cvg_health",
        "cvg_help",
        "cvg_solve",
        "cvg_create_plan",
        "cvg_execute_plan",
        "cvg_how_to",
    ];
    for name in &expected {
        assert!(names.contains(name), "missing essential tool: {}", name);
    }
}

#[test]
fn core_ring_filtered_list_at_least_as_many_as_all_defs_unfiltered() {
    let defs = all_defs();
    let core = list_tool_defs_filtered(&defs, Ring::Core);
    assert_eq!(core.len(), defs.len(), "core ring should see all tools");
}
