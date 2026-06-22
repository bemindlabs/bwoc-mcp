# Changelog

All notable changes to `bwoc-mcp` are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **MCP tool annotations on every tool.** All 24 tools now declare the standard MCP behavior hints so a host (Claude Desktop, etc.) can show users which are safe vs mutating: the 13 read tools are `readOnlyHint: true`; writes are `readOnlyHint: false`; `run` (spawns an agent backend) adds `openWorldHint: true`; and the 4 lifecycle tools (`new`/`retire`/`start`/`stop`) add `destructiveHint: true`. This is also the per-tool annotation the Anthropic Connectors Directory requires.
- **One-click Claude Desktop extension (`.mcpb`).** New `mcpb/manifest.json` (MCP Bundle, `manifest_version` 0.3) + `scripts/build-mcpb.sh` package the compiled server as a desktop extension — double-click `bwoc-mcp.mcpb` and Claude Desktop installs it, prompting only for the workspace directory (read-only by default). Binary server with per-platform overrides (macOS universal / Linux x64 / Windows x64). The release workflow now builds + attaches `bwoc-mcp.mcpb` on each tag (macOS binaries `lipo`'d to universal2). Verified with `@anthropic-ai/mcpb validate`/`pack`.

## [1.0.1] - 2026-06-09

### Fixed

- `bwoc-core` is now a git dependency pinned to BWOC-Framework `v2026.6.9-0`
  instead of a sibling path dependency, so the crate builds from a fresh clone
  and via `cargo install --git` (v1.0.0 only built on a box that also checked
  out the framework).

### Added

- GitHub Actions **CI quality gate** — rustfmt + clippy, then build + test on
  Linux, macOS, and Windows for every push to `main` and pull request.
- GitHub Actions **release pipeline** — on a `v*` tag, build release binaries
  (linux x86_64/aarch64, macOS arm64/x86_64, windows x86_64) and attach them to
  the GitHub release.

## [1.0.0] - 2026-06-09

First public release — a Model Context Protocol server that exposes a BWOC
workspace to any MCP client.

### Added

- **Two transports**: stdio (default) and Streamable HTTP (`--features http`,
  `--transport http`) with a bearer-token gate that refuses to serve write/exec
  tiers unauthenticated.
- **Hybrid bridge**: links `bwoc-core` for in-process primitives and shells out
  to `bwoc <verb> --json` for verb-level surface — full coverage with no
  framework changes.
- **Full 24-tool catalog** across four tiers — read (always on), write, exec,
  and lifecycle/dangerous. Read-only by default; mutating tiers are opt-in via
  `--allow-write`, `--allow-exec`, and `--allow-dangerous`, enforced at call
  time.
- **MCP resources**: `bwoc://agents`, `bwoc://fleet`, `bwoc://info`.
- **MCP prompts**: `delegate` (delegate a task to an agent) and `fleet_review`.
- Workspace targeting via `--workspace` / `$BWOC_WORKSPACE`; configurable
  `bwoc` binary via `--bwoc-bin` / `$BWOC_BIN`.

### Notes

- `chat` is intentionally not exposed — it is interactive-only; `run` is the
  headless path.
- team/task tools route through shell-out (`bwoc task … --json`) by design, to
  keep this public adapter decoupled from the framework's on-disk layout.

[1.0.1]: https://github.com/bemindlabs/bwoc-mcp/releases/tag/v1.0.1
[1.0.0]: https://github.com/bemindlabs/bwoc-mcp/releases/tag/v1.0.0
