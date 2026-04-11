//! convergio-mcp-server binary entry point.
//!
//! Exposes the Convergio daemon as an MCP server over stdio or
//! Streamable HTTP transport. Uses the rmcp SDK for protocol handling.
//! Tool definitions are fetched dynamically from the daemon at startup.
//! CRITICAL: stdout is JSON-RPC protocol only. All logs must go to stderr.

use std::sync::Arc;

use clap::Parser;
use convergio_mcp::handler::ConvergioHandler;
use convergio_mcp::profile::Profile;
use convergio_mcp::registry::fetch_tool_defs;
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
};
use rmcp::ServiceExt;

#[derive(Parser)]
#[command(name = "convergio-mcp-server", version, about = "Convergio MCP server")]
struct Cli {
    /// Transport: "stdio" (default) or "streamable-http"
    #[arg(long, default_value = "stdio")]
    transport: String,

    /// Bind address for Streamable HTTP transport (only with --transport streamable-http)
    #[arg(long, default_value = "127.0.0.1:8421")]
    bind: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let cli = Cli::parse();

    let ring: u8 = std::env::var("CONVERGIO_MCP_RING")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3);

    let daemon_url = std::env::var("CONVERGIO_DAEMON_URL")
        .unwrap_or_else(|_| "http://localhost:8420".to_string());

    let token = std::env::var("CONVERGIO_API_TOKEN").ok();

    // Discover tools from daemon (falls back to static defs if unreachable)
    let tools = fetch_tool_defs(&daemon_url, token.as_deref());
    let profile = Profile::from_env();
    eprintln!(
        "Discovered {} MCP tools, profile={:?} (effective: {})",
        tools.len(),
        profile,
        convergio_mcp::profile::filter_by_profile(&tools, profile).len()
    );

    tracing::info!(ring, daemon_url = %daemon_url, transport = %cli.transport,
        tools = tools.len(), ?profile, "starting");

    match cli.transport.as_str() {
        "stdio" => {
            let handler =
                ConvergioHandler::new(ring, profile, &daemon_url, token.as_deref(), tools);
            let transport = rmcp::transport::stdio();
            let server = handler.serve(transport).await?;
            server.waiting().await?;
        }
        "streamable-http" | "sse" => {
            let tools = Arc::new(tools);
            let d = daemon_url.clone();
            let t = token.clone();
            let service = StreamableHttpService::new(
                move || {
                    Ok(ConvergioHandler::new(
                        ring,
                        profile,
                        &d,
                        t.as_deref(),
                        (*tools).clone(),
                    ))
                },
                Arc::new(LocalSessionManager::default()),
                StreamableHttpServerConfig::default(),
            );

            let listener = tokio::net::TcpListener::bind(&cli.bind).await?;
            eprintln!("MCP Streamable HTTP on http://{}", cli.bind);
            loop {
                let (stream, _) = listener.accept().await?;
                let svc = hyper_util::service::TowerToHyperService::new(service.clone());
                let io = hyper_util::rt::TokioIo::new(stream);
                tokio::spawn(async move {
                    let builder = hyper_util::server::conn::auto::Builder::new(
                        hyper_util::rt::TokioExecutor::new(),
                    );
                    let conn = builder.serve_connection(io, svc);
                    if let Err(e) = conn.await {
                        eprintln!("Connection error: {e}");
                    }
                });
            }
        }
        other => {
            eprintln!("Unknown transport: {other}. Use 'stdio' or 'streamable-http'.");
            std::process::exit(1);
        }
    }

    Ok(())
}
