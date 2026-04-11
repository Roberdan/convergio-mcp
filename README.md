# convergio-mcp

[![CI](https://github.com/Roberdan/convergio-mcp/actions/workflows/ci.yml/badge.svg)](https://github.com/Roberdan/convergio-mcp/actions/workflows/ci.yml)
[![License: Convergio Community](https://img.shields.io/badge/license-Convergio%20Community-blue)](https://github.com/Roberdan/convergio-mcp/blob/main/LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange)](https://www.rust-lang.org/)
[![Zero Warnings](https://img.shields.io/badge/warnings-0-brightgreen)](#)

MCP server for Convergio — exposes daemon tools via rmcp SDK

Part of the [Convergio](https://github.com/Roberdan/convergio) ecosystem.

## Architecture

```mermaid
graph LR
    CRATE[convergio-mcp] --> SDK[convergio-sdk]
    SDK --> T[types]
    SDK --> TEL[telemetry]
    SDK --> DB[db]
    SDK --> SEC[security]

    style CRATE fill:#e94560,stroke:#1a1a2e,color:#fff
    style SDK fill:#0f3460,stroke:#e94560,color:#fff
```

## Quality gates

| Gate | Status |
|------|--------|
| Zero warnings (`-Dwarnings`) | CI enforced |
| All tests (unit + integration) | CI enforced |
| Dependency audit (`cargo audit`) | CI enforced |
| License policy (`cargo deny`) | CI enforced |
| Format (`cargo fmt`) | CI enforced |
| Auto-release | release-please |

## Usage

```toml
[dependencies]
convergio-mcp = { git = "https://github.com/Roberdan/convergio-mcp", tag = "v0.1.0" }
```

## Development

```bash
cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" cargo clippy --workspace --all-targets --locked
cargo test --workspace --locked
cargo deny check
```

## Related

- [convergio-sdk](https://github.com/Roberdan/convergio-sdk) — Core types, telemetry, security, db
- [convergio](https://github.com/Roberdan/convergio) — Main daemon

## License

Convergio Community License v1.3 — see [LICENSE](LICENSE).

---

## Agentic Manifesto

See the full [Agentic Manifesto](https://github.com/Roberdan/convergio/blob/main/AgenticManifesto.md) — the guiding philosophy behind Convergio.

---

© 2025-present Roberto D'Angelo. All rights reserved.
