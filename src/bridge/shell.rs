//! Shell-out path: invoke `bwoc <verb> [args] --json` and parse the result.
//!
//! Every read/write verb whose logic lives in the `bwoc` binary is reached this
//! way. Almost all of them carry a `--json` twin (verified against bwoc-cli:
//! status, fleet, info, memory, inbox, sessions, trust, run, list, team, task,
//! doctor, new, retire, start, stop, …), so the output deserializes straight
//! into an MCP structured result.

use super::Bridge;
use serde_json::Value;
use tokio::process::Command;

#[derive(Debug, thiserror::Error)]
pub enum ShellError {
    #[error("failed to spawn `{bin}`: {source}")]
    Spawn {
        bin: String,
        #[source]
        source: std::io::Error,
    },
    #[error("`bwoc {verb}` exited with {code}: {stderr}")]
    NonZero {
        verb: String,
        code: i32,
        stderr: String,
    },
    #[error("`bwoc {verb}` emitted non-JSON output: {source}\n{raw}")]
    Json {
        verb: String,
        #[source]
        source: serde_json::Error,
        raw: String,
    },
}

impl Bridge {
    /// Run `bwoc <args...> --json` in the workspace and parse stdout as JSON.
    pub async fn json(&self, args: &[&str]) -> Result<Value, ShellError> {
        let verb = args.first().copied().unwrap_or("").to_string();
        let mut cmd = Command::new(&self.bwoc_bin);
        cmd.current_dir(&self.workspace)
            .args(args)
            .arg("--json")
            // Keep the workspace explicit so the child never guesses.
            .env("BWOC_WORKSPACE", &self.workspace);

        let out = cmd.output().await.map_err(|source| ShellError::Spawn {
            bin: self.bwoc_bin.clone(),
            source,
        })?;

        if !out.status.success() {
            return Err(ShellError::NonZero {
                verb,
                code: out.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&out.stderr).trim().to_string(),
            });
        }

        let raw = String::from_utf8_lossy(&out.stdout).to_string();
        serde_json::from_str(&raw).map_err(|source| ShellError::Json { verb, source, raw })
    }

    /// Run `bwoc <args...>` for verbs without a `--json` twin; return stdout.
    pub async fn text(&self, args: &[&str]) -> Result<String, ShellError> {
        let verb = args.first().copied().unwrap_or("").to_string();
        let mut cmd = Command::new(&self.bwoc_bin);
        cmd.current_dir(&self.workspace)
            .args(args)
            .env("BWOC_WORKSPACE", &self.workspace);

        let out = cmd.output().await.map_err(|source| ShellError::Spawn {
            bin: self.bwoc_bin.clone(),
            source,
        })?;

        if !out.status.success() {
            return Err(ShellError::NonZero {
                verb,
                code: out.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&out.stderr).trim().to_string(),
            });
        }
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }
}
