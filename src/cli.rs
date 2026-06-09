//! Command-line surface for the BWOC MCP server.
//!
//! One server instance targets exactly one BWOC workspace. The transport and
//! the write/exec posture are chosen here and threaded into the server.

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
#[command(
    name = "bwoc-mcp",
    about = "Public MCP server exposing a BWOC workspace to any MCP client",
    version
)]
pub struct Cli {
    /// Workspace root to operate on. Defaults to $BWOC_WORKSPACE, then CWD.
    #[arg(long, env = "BWOC_WORKSPACE")]
    pub workspace: Option<PathBuf>,

    /// Path to the `bwoc` binary used for shell-out verbs (Hybrid bridge).
    #[arg(long, env = "BWOC_BIN", default_value = "bwoc")]
    pub bwoc_bin: String,

    /// Transport to serve on.
    #[arg(long, value_enum, default_value_t = Transport::Stdio)]
    pub transport: Transport,

    /// Bind address for the `http` transport (Streamable HTTP).
    #[arg(long, default_value = "127.0.0.1:8765")]
    pub http_addr: String,

    /// Bearer token required by the `http` transport. If unset, HTTP refuses to
    /// start unless --http-insecure is given (never expose write/exec unauthed).
    #[arg(long, env = "BWOC_MCP_TOKEN")]
    pub http_token: Option<String>,

    /// Allow the HTTP transport to start without a bearer token. Loopback +
    /// read-only only; refuses if write/exec tools are enabled.
    #[arg(long)]
    pub http_insecure: bool,

    /// Enable write tools (send, task add/claim/complete, note/doc writes…).
    /// Off by default even though the full surface is wired — the operator
    /// opts in explicitly. See docs/PLAN.md §Security posture.
    #[arg(long)]
    pub allow_write: bool,

    /// Enable exec tools that spawn agent backends (run, chat, spawn) — these
    /// can invoke LLM backends and cost money / take time.
    #[arg(long)]
    pub allow_exec: bool,

    /// Enable lifecycle-mutating tools (new, retire, start, stop) — highest
    /// blast radius. Implies nothing about --allow-write/--allow-exec.
    #[arg(long)]
    pub allow_dangerous: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Transport {
    /// Line-delimited JSON-RPC over stdio (Claude Desktop, Cursor, local hosts).
    Stdio,
    /// Streamable HTTP (remote / "public" hosting). Requires the `http` feature.
    Http,
}

/// Which tool tiers are exposed, derived from the --allow-* flags.
#[derive(Debug, Clone, Copy)]
pub struct Posture {
    pub write: bool,
    pub exec: bool,
    pub dangerous: bool,
}

impl Cli {
    pub fn posture(&self) -> Posture {
        Posture {
            write: self.allow_write,
            exec: self.allow_exec,
            dangerous: self.allow_dangerous,
        }
    }

    /// Resolve the workspace root: explicit flag → env → current dir.
    pub fn workspace_root(&self) -> std::io::Result<PathBuf> {
        match &self.workspace {
            Some(p) => Ok(p.clone()),
            None => std::env::current_dir(),
        }
    }
}
