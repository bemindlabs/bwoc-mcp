//! The Hybrid bridge to a BWOC workspace.
//!
//! Two paths, picked per-verb in [`crate::server`]:
//!
//! - [`core`] — link `bwoc-core` and call its domain primitives in-process
//!   (manifest, workspace registry, team/task TOML, routing/inbox, deep_memory).
//!   Fast, no subprocess, structured by construction.
//! - [`shell`] — spawn `bwoc <verb> --json` for everything whose logic lives in
//!   the `bwoc` binary, not in `bwoc-core` (list, status, fleet, run, chat,
//!   send, new, retire, doctor, info, sessions, trust, peer …).
//!
//! See `docs/PLAN.md` §Tool catalog for the full per-verb assignment.

pub mod core;
pub mod shell;

use std::path::PathBuf;

/// Shared context handed to every tool: where the workspace is and how to
/// reach the `bwoc` binary.
#[derive(Debug, Clone)]
pub struct Bridge {
    pub workspace: PathBuf,
    pub bwoc_bin: String,
}

impl Bridge {
    pub fn new(workspace: PathBuf, bwoc_bin: String) -> Self {
        Self { workspace, bwoc_bin }
    }
}
