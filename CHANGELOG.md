# Changelog

All notable changes to `bwoc-mcp` are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
