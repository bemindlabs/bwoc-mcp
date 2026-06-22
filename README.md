<h1 align="center">bwoc-mcp</h1>

<p align="center">
  <strong>BWOC → Model Context Protocol</strong> — expose a <a href="https://github.com/bemindlabs/BWOC-Framework">BWOC</a> agent workspace to any MCP client.
</p>

<p align="center">
  <img alt="License: MIT" src="https://img.shields.io/badge/License-MIT-yellow.svg">
  <img alt="Rust" src="https://img.shields.io/badge/built%20with-Rust-dea584">
  <img alt="MCP" src="https://img.shields.io/badge/protocol-MCP-1f6feb">
  <img alt="Release" src="https://img.shields.io/badge/release-v1.0.1-brightgreen">
  <img alt="CI" src="https://github.com/bemindlabs/bwoc-mcp/actions/workflows/ci.yml/badge.svg">
  <img alt="Part of BWOC" src="https://img.shields.io/badge/part%20of-BWOC-6f42c1">
</p>

---

## Overview

`bwoc-mcp` is a public **MCP server** that puts a whole BWOC workspace behind the
Model Context Protocol. Point Claude Desktop, Cursor, or any MCP host at it and
they can list agents, read fleet health, send work, run headless tasks, and
drive teams — over **stdio** locally or **Streamable HTTP** when hosted.

It is the server counterpart to the BWOC harness's MCP *client*. Mechanism is
**Hybrid**: it links `bwoc-core` for in-process primitives and shells out to
`bwoc <verb> --json` for the rest, so the full verb surface is available without
changing the framework.

## What it exposes

| Tier | Tools | Default |
|---|---|---|
| Read | `list` · `status` · `fleet` · `info` · `sessions` · `trust` · `team`/`task` · `memory` · `inbox` | ✅ on |
| Write | `send` · `task add/claim/complete` · note/doc writes | `--allow-write` |
| Exec | `run` · `chat` (spawns agent backends) | `--allow-exec` |
| Lifecycle | `new` · `retire` · `start` · `stop` | `--allow-dangerous` |

Read-only by default — mutating tiers are opt-in. See [`docs/PLAN.md`](docs/PLAN.md)
for the full catalog and security posture.

## Quickstart

```bash
# build (stdio only; add --features http for the HTTP transport)
cargo build --release

# run against a workspace, read-only
./target/release/bwoc-mcp --workspace /path/to/bwoc-workspace

# enable writes + headless runs
./target/release/bwoc-mcp --workspace . --allow-write --allow-exec
```

### Claude Desktop — one-click (`.mcpb`)

The easiest path. Download `bwoc-mcp.mcpb` from the [latest release](https://github.com/bemindlabs/bwoc-mcp/releases/latest) (or build it: `./scripts/build-mcpb.sh`) and **double-click it** — Claude Desktop installs the bundled server and asks only for your **workspace directory**. It runs **read-only** by default.

`.mcpb` is the [MCP Bundle](https://github.com/anthropics/mcpb) format (one-click local MCP servers for desktop apps). The bundle ships the compiled `bwoc-mcp` binary, so there's nothing to compile and no config file to hand-edit.

### Claude Desktop — manual (`claude_desktop_config.json`)

Or wire it by hand (and the only way to enable the write/exec tiers today — add the `--allow-*` flags to `args`):

```jsonc
{
  "mcpServers": {
    "bwoc": {
      "command": "/abs/path/to/bwoc-mcp",
      "args": ["--workspace", "/abs/path/to/workspace"]
    }
  }
}
```

HTTP (public hosting) — requires `--features http` and a bearer token:

```bash
bwoc-mcp --transport http --http-addr 127.0.0.1:8765 --http-token "$BWOC_MCP_TOKEN"
```

## Status

**v1.0.1.** Both transports (**stdio** + **Streamable HTTP** with bearer auth),
the **full 24-tool catalog** (read open; write / exec / lifecycle gated), and
MCP **resources** (`bwoc://agents|fleet|info`) + **prompts** (`delegate`,
`fleet_review`) are shipped and tested (7 integration tests, clippy clean). See
the [changelog](CHANGELOG.md) and [`docs/PLAN.md`](docs/PLAN.md).

## License

MIT © 2026 Bemind Technology.
