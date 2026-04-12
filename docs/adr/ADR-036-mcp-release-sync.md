# ADR-036: MCP Release Sync + Doctor-Backed Coverage

**Status:** Accepted
**Date:** 2026-04-08

## Context

Convergio keeps adding plan, validation, and operations capabilities through the
daemon API, but the MCP surface and release artifacts can drift if they are not
updated in the same change. That creates three failure modes:

1. A new daemon feature exists but is not callable through `convergio-mcp-server`.
2. A tagged release ships the daemon without the matching MCP companion binary.
3. CI stays green even though doctor/E2E coverage does not exercise the updated
   MCP/release contract.

## Decision

1. **Every tagged release ships both binaries**: `convergio` and
   `convergio-mcp-server`.

2. **Release CI must explicitly run MCP + doctor checks** in addition to the
   workspace tests. The release workflow is now required to execute:
   - `cargo test -p convergio-mcp --tests`
   - `cargo test -p convergio-doctor --lib --tests`

3. **New externally useful daemon capabilities must be reflected in MCP** during
   the same wave or release. For Theta this includes:
   - execution-tree retrieval
   - atomic task completion
   - plan validation / Thor gate invocation
   - task updates with the required `agent_id`

4. **Doctor enforces the contract** via the beta check `mcp_release_sync`, which
   verifies the MCP registry, release workflow, and docs remain aligned so
   **API, CLI, and MCP** stay in lockstep for every shipped feature.

## Consequences

- Releases now publish an MCP artifact that matches the daemon version and tool
  surface.
- Drift becomes visible before shipping: doctor catches missing MCP wiring,
  missing release gates, or missing documentation.
- ADR/documentation stay part of the same contract rather than an afterthought.
