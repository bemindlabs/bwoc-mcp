//! In-process path: call `bwoc-core` domain primitives directly.
//!
//! This is the half of the Hybrid bridge that needs no subprocess. `bwoc-core`
//! is a thin domain library (manifest, workspace registry, team/task TOML,
//! routing/inbox, deep_memory); the verb-level orchestration lives in the
//! `bwoc` binary and is reached via [`super::shell`] instead.
//!
//! v0.1 wires the team/task surface as the proof-of-linkage. The remaining
//! primitives (manifest load, workspace registry, routing/inbox append,
//! deep_memory read) are enumerated in `docs/PLAN.md` §Tool catalog and slot in
//! here without touching the framework.
//!
//! Deferred seam — not yet wired into the tool router (the full catalog routes
//! through `bridge::shell` today), so the items below are `allow(dead_code)`
//! until the Phase 3 in-proc migration lands.
#![allow(dead_code)]

use super::Bridge;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("team: {0}")]
    Team(String),
    #[error("serialize: {0}")]
    Serde(#[from] serde_json::Error),
}

impl Bridge {
    /// Parse a team's shared task list (`.bwoc/teams/<team>/tasks.jsonl`) via
    /// `bwoc_core::team::parse_tasks` and return it as structured JSON — no
    /// subprocess. The exact on-disk path is resolved by the build-out; this
    /// helper takes the raw JSONL so it stays a pure, testable seam.
    pub fn parse_team_tasks(&self, jsonl: &str) -> Result<serde_json::Value, CoreError> {
        let tasks =
            bwoc_core::team::parse_tasks(jsonl).map_err(|e| CoreError::Team(e.to_string()))?;
        Ok(serde_json::to_value(tasks)?)
    }
}
