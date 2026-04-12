---
version: "1.0"
last_updated: "2026-04-08"
author: "convergio-team"
tags: ["adr", "mcp"]
---

# ADR-037: Native MCP Server with Dynamic Tool Discovery

**Status:** Accepted
**Date:** 2026-04-08
**Deciders:** Roberto D'Angelo

## Context

AI coding agents (Copilot CLI, Claude Code, Cursor) support Model Context Protocol
(MCP) as the standard tool interface. Convergio agents previously interacted via
shell-exec of `cvg` CLI commands — parsing stdout text, no structured errors, no
streaming. Issue #438 requested a native MCP server.

The first approach hardcoded ~29 tool definitions in `convergio-mcp`. This quickly
became a maintenance problem: every new daemon route required manual addition to
the MCP crate's registry, which was easy to forget.

## Decision

### 1. Adopt rmcp v1.3.0 SDK

Use the official Rust MCP SDK (`rmcp` crate) instead of a custom JSON-RPC
implementation. This provides protocol compliance (initialize, list_tools,
call_tool, notifications, capability negotiation) out of the box.

### 2. Dual transport: stdio + Streamable HTTP

- **stdio**: `convergio-mcp-server` (default) — for local agent integration
- **Streamable HTTP**: `convergio-mcp-server --transport http` on port 8421 —
  for IDE clients (VS Code, Cursor) and remote agents

### 3. Generic HTTP bridge

Instead of implementing a handler per tool, a single `bridge_call()` function
maps any MCP tool call to a daemon HTTP request using the tool's `ToolDef`
metadata (method, path, path_params). This means adding a new tool requires
zero handler code.

### 4. Dynamic tool discovery via Extension trait

Each Extension crate declares its MCP tools by overriding `mcp_tools()` on the
Extension trait:

```rust
// In convergio-types/src/extension.rs
pub struct McpToolDef {
    pub name: String,
    pub description: String,
    pub method: String,       // "GET", "POST", etc.
    pub path: String,         // "/api/plan-db/json/:plan_id"
    pub input_schema: Value,  // JSON Schema
    pub min_ring: String,     // "sandboxed", "community", "trusted", "core"
    pub path_params: Vec<String>,
}

// Default implementation returns empty vec (backward compatible)
fn mcp_tools(&self) -> Vec<McpToolDef> { vec![] }
```

The daemon aggregates all Extension tools at `/api/meta/mcp-tools`. The MCP
binary fetches this at startup, falling back to static definitions if the
daemon is unreachable.

### 5. Ring-based access control

Each tool carries a `min_ring` field. The MCP server filters tools based on the
client's trust level (Sandboxed < Community < Trusted < Core), preventing
untrusted agents from accessing admin operations.

## Consequences

### Positive

- **Auto-updating**: When a crate adds `mcp_tools()`, the MCP server discovers
  the new tools at next startup — no changes needed in convergio-mcp itself
- **Protocol compliance**: Full MCP spec via rmcp SDK
- **IDE integration**: Streamable HTTP enables VS Code / Cursor MCP clients
- **Single handler**: Bridge pattern means zero per-tool handler code
- **Backward compatible**: Extension trait default returns empty vec

### Negative

- **Startup dependency**: MCP server makes HTTP call to daemon at startup
  (mitigated by static fallback)
- **No hot-reload**: Tool list is cached at startup; new extensions require
  MCP server restart (acceptable trade-off vs complexity)

### Tool count

- ~36 dynamic tools from 12 Extension crates
- 7 synthetic workflow tools (static: solve, create_plan, execute_plan, etc.)
- Total: ~43 tools covering plan lifecycle, task execution, evidence, agent
  spawning, skill access, kernel inference, and system observability

## Alternatives Considered

1. **Keep hardcoded registry**: Rejected — maintenance burden, tools become stale
2. **Auto-generate from OpenAPI**: Rejected — daemon doesn't have OpenAPI spec,
   would expose raw routes instead of curated agent-friendly tools
3. **Hot-reload via polling**: Deferred — adds complexity for minimal benefit
   since extensions don't change at runtime
