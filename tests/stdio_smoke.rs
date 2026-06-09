//! End-to-end stdio smoke test.
//!
//! Spawns the built `bwoc-mcp` binary with a *stub* `bwoc` on the shell-out
//! path (so no real workspace/LLM is needed), drives it with newline-delimited
//! JSON-RPC over stdio, and asserts the MCP handshake, the read path, and a
//! posture gate refusal.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

/// Write a stub `bwoc` script into `dir` that answers the verbs the test hits.
fn write_stub_bwoc(dir: &std::path::Path) -> std::path::PathBuf {
    let path = dir.join("bwoc");
    std::fs::write(
        &path,
        r#"#!/bin/sh
# Minimal stub: route by first arg. Ignores the trailing --json the server adds.
case "$1" in
  list) echo '{"agents":[{"id":"agent-test","status":"active","role":"tester"}]}' ;;
  send) echo "sent to $2" ;;
  *)    echo '{}' ;;
esac
"#,
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    path
}

/// Run the server with the given extra args, feeding `requests` (one JSON-RPC
/// object per line) and returning every JSON-RPC object it writes back.
fn drive(extra_args: &[&str], requests: &[&str]) -> Vec<serde_json::Value> {
    let tmp = tempfile::tempdir().unwrap();
    let stub = write_stub_bwoc(tmp.path());

    let mut child = Command::new(env!("CARGO_BIN_EXE_bwoc-mcp"))
        .arg("--workspace")
        .arg(tmp.path())
        .arg("--bwoc-bin")
        .arg(&stub)
        .args(extra_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn bwoc-mcp");

    {
        let mut stdin = child.stdin.take().unwrap();
        for r in requests {
            writeln!(stdin, "{r}").unwrap();
        }
        // Drop stdin → EOF → server flushes responses and exits.
    }

    let stdout = child.stdout.take().unwrap();
    let mut out = Vec::new();
    for line in BufReader::new(stdout).lines() {
        let line = line.unwrap();
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
            out.push(v);
        }
    }
    let _ = child.wait();
    out
}

const INIT: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"t","version":"0"}}}"#;
const INITED: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;

fn by_id(msgs: &[serde_json::Value], id: i64) -> Option<&serde_json::Value> {
    msgs.iter().find(|m| m.get("id").and_then(|v| v.as_i64()) == Some(id))
}

#[test]
fn handshake_reports_bwoc_mcp_identity() {
    let msgs = drive(&[], &[INIT, INITED]);
    let init = by_id(&msgs, 1).expect("initialize response");
    let si = &init["result"]["serverInfo"];
    assert_eq!(si["name"], "bwoc-mcp");
    assert_eq!(si["version"], env!("CARGO_PKG_VERSION"));
}

#[test]
fn tools_list_exposes_full_catalog() {
    let list = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;
    let msgs = drive(&[], &[INIT, INITED, list]);
    let tools = by_id(&msgs, 2).unwrap()["result"]["tools"]
        .as_array()
        .unwrap();
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    // A representative cross-section of every tier must be present.
    for expected in [
        "bwoc_ping",
        "bwoc_list",
        "bwoc_send",
        "bwoc_task_add",
        "bwoc_run",
        "bwoc_retire",
    ] {
        assert!(names.contains(&expected), "missing tool {expected}");
    }
    assert!(names.len() >= 20, "expected the full catalog, got {}", names.len());
}

#[test]
fn read_tool_is_ungated_and_returns_data() {
    let call = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"bwoc_list","arguments":{}}}"#;
    let msgs = drive(&[], &[INIT, INITED, call]);
    let res = by_id(&msgs, 3).unwrap();
    let text = res["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("agent-test"), "stub list output not surfaced: {text}");
}

#[test]
fn write_tool_is_refused_without_allow_write() {
    let call = r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"bwoc_send","arguments":{"agent":"x","message":"hi"}}}"#;
    let msgs = drive(&[], &[INIT, INITED, call]);
    let res = by_id(&msgs, 4).unwrap();
    let err = res.get("error").or_else(|| res["result"].get("isError").map(|_| &res["result"]));
    assert!(err.is_some(), "expected refusal, got {res}");
    let blob = res.to_string();
    assert!(blob.contains("--allow-write"), "refusal should name the flag: {blob}");
}

#[test]
fn resources_are_listed() {
    let list = r#"{"jsonrpc":"2.0","id":6,"method":"resources/list"}"#;
    let msgs = drive(&[], &[INIT, INITED, list]);
    let uris: Vec<String> = by_id(&msgs, 6).unwrap()["result"]["resources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["uri"].as_str().unwrap().to_string())
        .collect();
    assert!(uris.iter().any(|u| u == "bwoc://agents"), "got {uris:?}");
}

#[test]
fn delegate_prompt_interpolates_args() {
    let get = r#"{"jsonrpc":"2.0","id":7,"method":"prompts/get","params":{"name":"delegate","arguments":{"agent":"yudi","task":"audit X"}}}"#;
    let msgs = drive(&[], &[INIT, INITED, get]);
    let text = by_id(&msgs, 7).unwrap()["result"]["messages"][0]["content"]["text"]
        .as_str()
        .unwrap();
    assert!(text.contains("yudi") && text.contains("audit X"), "got {text}");
}

#[test]
fn write_tool_runs_with_allow_write() {
    let call = r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"bwoc_send","arguments":{"agent":"yudi","message":"hi"}}}"#;
    let msgs = drive(&["--allow-write"], &[INIT, INITED, call]);
    let res = by_id(&msgs, 5).unwrap();
    let text = res["result"]["content"][0]["text"].as_str().unwrap_or("");
    assert!(text.contains("sent to yudi"), "stub send output not surfaced: {res}");
}
