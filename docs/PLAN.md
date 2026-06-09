# bwoc-mcp — Implementation Plan

Public **MCP server** that exposes a single BWOC workspace (agents, teams,
tasks, memory, run/send/fleet) to any Model Context Protocol client — Claude
Desktop, Cursor, the BWOC harness's own MCP client, or a remote host.

This is the missing server half of BWOC's MCP story: the framework's harness is
already an MCP **client** (`bwoc-harness/src/mcp.rs`, hand-rolled JSON-RPC); the
server role was deliberately deferred ("network surface, no current need").
`bwoc-mcp` fills it, in a separate repo so the framework's dep-quarantine never
binds the heavy SDK deps.

## Locked decisions

| Axis | Choice |
|---|---|
| Runtime | **Rust + `rmcp` 1.7** (official Rust MCP SDK) |
| Bridge | **Hybrid** — link `bwoc-core` for primitives + shell-out `bwoc <verb> --json` for the rest |
| Transport | **stdio** (default) **+ Streamable HTTP** (`http` feature) |
| Surface | **Full** — every BWOC verb wired |
| Posture | Read tools always on; **write / exec / dangerous gated** behind `--allow-*` |

### Why Hybrid (the finding that shaped it)

`bwoc-core` is a thin domain library (`manifest`, `workspace`, `team`,
`routing`, `deep_memory`, `chat_proto`, `exec`, `ipc`) — **primitives only**.
`bwoc-cli` is a `[[bin]]` with **no `lib` target**: the verb logic for `list`,
`status`, `fleet`, `run`, `chat`, `send`, `new`, `retire`, `doctor`, `info`,
plus all `--json` formatting, lives inside the binary. So "link bwoc-core" alone
reaches ~40% of the surface. Hybrid links core where the logic is already a
library and shells out to `bwoc … --json` (almost every verb has a `--json`
twin) for the rest — full surface today, zero framework changes.

The pure-in-process alternative (extract a `bwoc-cli` lib, move verb handlers to
`pub fn -> Structured`) remains the clean long-term path; it is an upstream
framework PR and is tracked as a future migration, not a blocker.

## Architecture

```
MCP client ──stdio/HTTP──▶ bwoc-mcp (rmcp ServerHandler + ToolRouter)
                              │
                   ┌──────────┴───────────┐
                   ▼                      ▼
         bridge::core (in-proc)   bridge::shell (subprocess)
         bwoc_core::team/…        `bwoc <verb> --json` in $BWOC_WORKSPACE
```

- `src/cli.rs` — clap surface: `--workspace`, `--bwoc-bin`, `--transport`,
  `--http-addr/--http-token/--http-insecure`, `--allow-write/-exec/-dangerous`.
- `src/bridge/` — `Bridge { workspace, bwoc_bin }`; `core.rs` (in-proc),
  `shell.rs` (`json()` / `text()` subprocess helpers).
- `src/server.rs` — `BwocMcp` `#[tool_router]`; one `#[tool]` per verb; posture
  gates at call time.
- `src/transport/http.rs` — axum + `rmcp` Streamable HTTP, bearer-token gate.

One server instance = one workspace. Multi-workspace (peer routes) is out of
scope for v1.

## Tool catalog (full surface)

Tier legend: **R** read (always on) · **W** write (`--allow-write`) ·
**X** exec/backend (`--allow-exec`) · **D** dangerous/lifecycle
(`--allow-dangerous`). Path: **core** = in-proc, **shell** = `bwoc … --json`.

| Tool | Tier | Path | Maps to |
|---|---|---|---|
| `bwoc_ping` | R | core | liveness / workspace echo |
| `bwoc_list` | R | shell | `list` |
| `bwoc_status` | R | shell | `status <agent>` |
| `bwoc_info` | R | shell | `info` |
| `bwoc_fleet` | R | shell | `fleet` |
| `bwoc_sessions` | R | shell | `sessions` |
| `bwoc_trust` | R | shell | `trust <agent>` |
| `bwoc_peer` | R | shell | `peer` |
| `bwoc_team_list` | R | core | `team::*` (parse teams) |
| `bwoc_task_list` | R | core | `team::parse_tasks` |
| `bwoc_memory_read` | R | core/shell | `deep_memory` / `memory` |
| `bwoc_inbox_read` | R | core | `routing::resolve` + read jsonl |
| `bwoc_send` | W | shell | `send <agent> <msg>` |
| `bwoc_task_add` | W | core | `team::add_task` |
| `bwoc_task_claim` | W | core | `team::claim_task` |
| `bwoc_task_complete` | W | core | `team::complete_task` |
| `bwoc_note_write` | W | shell | `notes`/`doc` |
| `bwoc_run` | X | shell | `run --task <t> <agent>` |
| `bwoc_new` | D | shell | `new` |
| `bwoc_retire` | D | shell | `retire` |
| `bwoc_start` / `bwoc_stop` | D | shell | `start` / `stop` |

Resources (Phase 3): expose `AGENTS.md`, `MEMORY.md`, team task lists as MCP
resources. Prompts (Phase 3): "delegate to `<agent>`" templates.

## Security posture

- **Read-only by default.** A bare `bwoc-mcp` serves only R tools. W/X/D are
  refused at call time with a message naming the flag to enable them.
- **HTTP never unauthenticated when mutating.** `--transport http` requires
  `--http-token` unless `--http-insecure` (which is refused if any W/X/D tier is
  on). Bearer-token middleware fronts `/mcp`.
- **Workspace is explicit.** Every shell-out sets `BWOC_WORKSPACE` and
  `current_dir`; no ambient guessing.
- **No secrets in transport.** Tokens via env/flag only; never logged.
- Future: per-tool allow/deny lists, rate limiting, and SSRF guards mirrored
  from `bwoc-a2a/src/ssrf.rs` if outbound calls are ever added.

## Phases

1. **P1 — stdio MVP** ✅: cli, bridge (core+shell), `BwocMcp`, stdio serve,
   posture gates. `cargo check` clean, smoked vs a real workspace.
2. **P2 — Streamable HTTP** ✅: `transport/http.rs` on the rmcp 1.7
   `StreamableHttpService`; bearer-token gate; refuses unauth when any
   write/exec tier is on. Loopback-smoked: 401 on bad token, MCP `initialize`
   over SSE on good token.
3. **P3 — full catalog** ✅: 24 tools across all four tiers, gates enforced,
   plus MCP **resources** (`bwoc://agents|fleet|info`) and **prompts**
   (`delegate`, `fleet_review`). `chat` intentionally excluded (interactive-only
   — `run` is the headless path).
   - **Decision — team/task stays shell-out, NOT in-proc.** The in-proc path
     would couple this public adapter to the framework's on-disk team layout
     (`.bwoc/teams/…`), which is framework-owned and not a stable API. Shell-out
     via `bwoc task … --json` is the *more correct* choice for a decoupled
     adapter, not merely the safer one. `bridge::core` remains a seam for future
     primitives that ARE library-stable.
4. **P4 — hardening** *(in progress)*: 7 integration tests (stub `bwoc` +
   tempdir, drives JSON-RPC over stdio; default + `http` feature), clippy clean.
   Remaining: CI matrix, README install docs, release artifacts, Homebrew
   formula.

## Versioning & distribution

- Dev: `bwoc-core` via path dep to the sibling `../bwoc-framework` checkout.
- Release: pin `bwoc-core` to a framework git tag so the in-proc half is
  version-locked while `bwoc-mcp` keeps its own release cadence.
- `bwoc` binary (shell-out half) discovered via `--bwoc-bin`/`$BWOC_BIN`, else
  `bwoc` on `PATH`. Document the version floor it expects.
- Ship: `cargo install`, a Homebrew formula (mirror the framework's `Formula/`),
  and a `claude_desktop_config.json` snippet.

## Testing

- Unit: `bridge::shell` against a stub `bwoc` script; `bridge::core` against
  fixture JSONL; posture gates.
- Integration: tempdir workspace (`bwoc init` + `bwoc new`) driven through the
  stdio server with an in-process rmcp client.
- No live LLM in CI — `run`/`chat` covered with a stub backend.
