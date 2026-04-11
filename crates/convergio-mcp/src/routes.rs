//! HTTP API routes for convergio-mcp.

use axum::Router;

/// Returns the router for this crate's API endpoints.
pub fn routes() -> Router {
    Router::new()
    // .route("/api/mcp/health", get(health))
}
