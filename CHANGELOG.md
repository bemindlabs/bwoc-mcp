# Changelog

All notable changes to `bwoc-mcp` are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[1.0.0]: https://github.com/bemindlabs/bwoc-mcp/releases/tag/v1.0.0
