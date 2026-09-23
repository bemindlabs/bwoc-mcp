//! Contract test against a **real** `bwoc` binary: every tool and resource the
//! server exposes must build an argv the installed CLI accepts.
//!
//! `stdio_smoke.rs` drives a stub that answers `{}` to anything, so an argv the
//! CLI rejects (bwoc 3.x's bare `fleet` refusing `--json`) passed unnoticed.
//! Here the stub forwards each argv to the real CLI with `--help` appended:
//! clap still parses every subcommand and flag first — an unknown one exits 2 —
//! but nothing runs, so mutating and lifecycle tools are checked safely with
//! placeholder arguments and no workspace fixtures.
//!
//! Runs only when `BWOC_CONTRACT_BIN` names the `bwoc` to check (CI installs
//! the latest release); skipped otherwise.
#![cfg(unix)]

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use serde_json::{Value, json};

fn forwarding_stub(dir: &std::path::Path, real: &str) -> std::path::PathBuf {
    let path = dir.join("bwoc");
    let err = dir.join("stderr");
    std::fs::write(
        &path,
        format!(
            "#!/bin/sh\n\"{real}\" \"$@\" --help >/dev/null 2>\"{err}\" || {{ cat \"{err}\" >&2; exit 2; }}\necho '{{}}'\n",
            err = err.display()
        ),
    )
    .unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path
}

/// A placeholder for each required argument, by its JSON-schema type.
fn placeholder_args(schema: &Value) -> Value {
    let props = schema["properties"]
        .as_object()
        .cloned()
        .unwrap_or_default();
    let required: Vec<&str> = schema["required"]
        .as_array()
        .map(|r| r.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let mut args = serde_json::Map::new();
    for name in required {
        let ty = props
            .get(name)
            .and_then(|p| p["type"].as_str())
            .unwrap_or("string");
        let v = match ty {
            "array" => json!(["x"]),
            "boolean" => json!(false),
            "integer" | "number" => json!(1),
            _ => json!("x"),
        };
        args.insert(name.to_string(), v);
    }
    Value::Object(args)
}

struct Server {
    child: std::process::Child,
    stdin: std::process::ChildStdin,
    stdout: BufReader<std::process::ChildStdout>,
    next_id: u64,
}

impl Server {
    fn request(&mut self, method: &str, params: Value) -> Value {
        self.next_id += 1;
        let id = self.next_id;
        let msg = json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
        writeln!(self.stdin, "{msg}").unwrap();
        self.stdin.flush().unwrap();
        loop {
            let mut line = String::new();
            assert!(
                self.stdout.read_line(&mut line).unwrap() > 0,
                "server closed"
            );
            let v: Value = serde_json::from_str(&line).unwrap();
            if v["id"] == json!(id) {
                return v;
            }
        }
    }
}

/// `Some(reason)` when a call came back as a JSON-RPC error or a tool error.
fn failure(reply: &Value) -> Option<String> {
    if let Some(e) = reply.get("error") {
        return Some(e["message"].as_str().unwrap_or("error").to_string());
    }
    if reply["result"]["isError"] == json!(true) {
        return Some(reply["result"]["content"][0]["text"].to_string());
    }
    None
}

#[test]
fn every_tool_and_resource_builds_an_argv_the_cli_accepts() {
    let Ok(real) = std::env::var("BWOC_CONTRACT_BIN") else {
        eprintln!("skipped: set BWOC_CONTRACT_BIN to a bwoc binary to run the CLI contract");
        return;
    };
    let tmp = tempfile::tempdir().unwrap();
    let stub = forwarding_stub(tmp.path(), &real);
    let mut child = Command::new(env!("CARGO_BIN_EXE_bwoc-mcp"))
        .arg("--workspace")
        .arg(tmp.path())
        .arg("--bwoc-bin")
        .arg(&stub)
        .args(["--allow-write", "--allow-exec", "--allow-dangerous"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn bwoc-mcp");
    let mut server = Server {
        stdin: child.stdin.take().unwrap(),
        stdout: BufReader::new(child.stdout.take().unwrap()),
        child,
        next_id: 0,
    };

    server.request(
        "initialize",
        json!({"protocolVersion": "2025-06-18", "capabilities": {},
               "clientInfo": {"name": "cli-contract", "version": "0"}}),
    );
    writeln!(
        server.stdin,
        "{}",
        json!({"jsonrpc": "2.0", "method": "notifications/initialized"})
    )
    .unwrap();

    let tools = server.request("tools/list", json!({}))["result"]["tools"]
        .as_array()
        .cloned()
        .expect("tools/list");
    assert!(!tools.is_empty(), "the server listed no tools");

    let mut broken = Vec::new();
    for tool in &tools {
        let name = tool["name"].as_str().unwrap();
        let args = placeholder_args(&tool["inputSchema"]);
        let reply = server.request("tools/call", json!({"name": name, "arguments": args}));
        if let Some(why) = failure(&reply) {
            broken.push(format!("tool {name}: {why}"));
        }
    }
    let resources = server.request("resources/list", json!({}))["result"]["resources"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    for res in &resources {
        let uri = res["uri"].as_str().unwrap();
        let reply = server.request("resources/read", json!({"uri": uri}));
        if let Some(why) = failure(&reply) {
            broken.push(format!("resource {uri}: {why}"));
        }
    }
    let _ = server.child.kill();

    assert!(
        broken.is_empty(),
        "{} of {} tools/resources build an argv `{real}` rejects:\n{}",
        broken.len(),
        tools.len() + resources.len(),
        broken.join("\n")
    );
    eprintln!(
        "cli contract: {} tools + {} resources accepted by {real}",
        tools.len(),
        resources.len()
    );
}
