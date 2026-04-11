//! Generic HTTP bridge: dispatches any ToolDef to the daemon API.
//!
//! Replaces the hand-written match statement in handlers.rs with a single
//! generic function that reads ToolDef metadata and builds the correct
//! HTTP request.

use serde_json::{json, Value};

use crate::http::{http_get, http_post};
use crate::registry::{build_url, HttpMethod, ToolDef};
use crate::ring::McpError;

/// Generic dispatch: given a ToolDef, build the HTTP request and execute it.
pub fn dispatch_tool(
    def: &ToolDef,
    args: &Value,
    daemon_url: &str,
    token: Option<&str>,
) -> Result<Value, McpError> {
    let url = build_url(def, args, daemon_url);

    match def.method {
        HttpMethod::Get => http_get(&url, token),
        HttpMethod::Post | HttpMethod::Put => {
            let body = strip_path_params(args, &def.path_params);
            http_post(&url, token, &body)
        }
        HttpMethod::Delete => http_get(&url, token), // DELETE with no body
    }
}

/// Remove path params from the JSON body (they're already in the URL).
fn strip_path_params(args: &Value, path_params: &[String]) -> Value {
    match args.as_object() {
        Some(obj) => {
            let filtered: serde_json::Map<String, Value> = obj
                .iter()
                .filter(|(k, _)| !path_params.iter().any(|p| p == k.as_str()))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            Value::Object(filtered)
        }
        None => json!({}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_path_params_removes_params() {
        let args = json!({"plan_id": 42, "extra": "keep"});
        let result = strip_path_params(&args, &["plan_id".to_string()]);
        assert!(result.get("plan_id").is_none());
        assert_eq!(result.get("extra").unwrap(), "keep");
    }

    #[test]
    fn strip_path_params_empty() {
        let args = json!({"plan_id": 42});
        let result = strip_path_params(&args, &[]);
        assert_eq!(result.get("plan_id").unwrap(), 42);
    }

    #[test]
    fn static_fallback_defs_not_empty() {
        let defs = crate::registry_defs::all_defs();
        assert!(!defs.is_empty(), "static fallback defs should not be empty");
    }

    #[test]
    fn build_url_interpolates_path_params() {
        use crate::registry::HttpMethod;
        use crate::ring::Ring;
        let def = ToolDef {
            name: "test_get_plan".into(),
            description: "test".into(),
            method: HttpMethod::Get,
            path: "/api/plan-db/json/:plan_id".into(),
            input_schema: json!({}),
            min_ring: Ring::Sandboxed,
            path_params: vec!["plan_id".into()],
        };
        let url = build_url(&def, &json!({"plan_id": 123}), "http://localhost:8420");
        assert_eq!(url, "http://localhost:8420/api/plan-db/json/123");
    }

    #[test]
    fn build_url_get_with_query() {
        use crate::registry::HttpMethod;
        use crate::ring::Ring;
        let def = ToolDef {
            name: "test_list".into(),
            description: "test".into(),
            method: HttpMethod::Get,
            path: "/api/reports".into(),
            input_schema: json!({}),
            min_ring: Ring::Sandboxed,
            path_params: vec![],
        };
        let url = build_url(
            &def,
            &json!({"status": "done", "limit": 10}),
            "http://localhost:8420",
        );
        assert!(url.contains("status=done"));
        assert!(url.contains("limit=10"));
    }
}
