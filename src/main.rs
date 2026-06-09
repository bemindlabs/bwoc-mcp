//! `bwoc-mcp` — public MCP server for a BWOC workspace.
//!
//! Serves the BWOC verb surface as MCP tools over stdio (default) or Streamable
//! HTTP (the `http` cargo feature + `--transport http`). See `docs/PLAN.md`.

mod bridge;
mod cli;
mod server;
#[cfg(feature = "http")]
mod transport;

use bridge::Bridge;
use clap::Parser;
use cli::{Cli, Transport};
use rmcp::{ServiceExt, transport::stdio};
use server::BwocMcp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Logs go to stderr so stdout stays a clean JSON-RPC channel on stdio.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "bwoc_mcp=info,rmcp=warn".into()),
        )
        .init();

    let args = Cli::parse();
    let workspace = args.workspace_root()?;
    let bridge = Bridge::new(workspace.clone(), args.bwoc_bin.clone());
    let posture = args.posture();

    tracing::info!(workspace = %workspace.display(), ?posture, transport = ?args.transport, "starting bwoc-mcp");

    match args.transport {
        Transport::Stdio => {
            let service = BwocMcp::new(bridge, posture).serve(stdio()).await?;
            service.waiting().await?;
        }
        Transport::Http => {
            #[cfg(feature = "http")]
            {
                transport::http::serve(args, bridge, posture).await?;
            }
            #[cfg(not(feature = "http"))]
            {
                anyhow::bail!(
                    "the `http` transport requires building with `--features http`"
                );
            }
        }
    }

    Ok(())
}
