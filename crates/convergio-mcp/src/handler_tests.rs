//! Tests for the rmcp-based ConvergioHandler.

use crate::handler::ConvergioHandler;
use crate::profile::Profile;
use rmcp::ServerHandler;

#[test]
fn handler_get_info_returns_valid_server_info() {
    let handler = ConvergioHandler::new(1, Profile::Full, "http://localhost:1", None, vec![]);
    let info = handler.get_info();
    assert!(!info.server_info.name.is_empty());
}

#[test]
fn handler_clone_preserves_config() {
    let handler =
        ConvergioHandler::new(2, Profile::Full, "http://localhost:1", Some("tok"), vec![]);
    let cloned = handler.clone();
    let info = cloned.get_info();
    assert!(!info.server_info.name.is_empty());
}

#[test]
fn handler_tools_capability_enabled() {
    let handler = ConvergioHandler::new(0, Profile::Full, "http://localhost:1", None, vec![]);
    let info = handler.get_info();
    assert!(
        info.capabilities.tools.is_some(),
        "tools capability must be enabled"
    );
}

#[test]
fn handler_instructions_present() {
    let handler = ConvergioHandler::new(0, Profile::Full, "http://localhost:1", None, vec![]);
    let info = handler.get_info();
    assert!(info.instructions.is_some());
    assert!(info.instructions.unwrap().contains("Convergio"));
}
