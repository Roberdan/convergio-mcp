//! convergio-mcp: MCP server binary for Convergio.
//!
//! Exposes daemon capabilities as MCP tools via the rmcp SDK.
//! Supports stdio and Streamable HTTP transports. Ring-based access
//! control filters tools per caller privilege level.
//! Tool definitions are discovered dynamically from the daemon at startup,
//! with static fallback defs for synthetic workflow tools.

pub mod bridge;
pub mod handler;
pub mod http;
pub mod profile;
pub mod registry;
pub mod registry_defs;
pub mod ring;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "handler_tests.rs"]
mod handler_tests;
