# 2026-06-22 — One-click Claude Desktop extension (.mcpb)

`bwoc-mcp` already worked with Claude Desktop, but only via a hand-edited
`claude_desktop_config.json`. This adds the **MCP Bundle** (`.mcpb`) packaging so
it installs with one double-click.

## Context (verified, not from memory)

The desktop-extension format is **MCPB** (`.mcpb`), the rename of DXT (`.dxt`
still loads). A bundle is a zip of `manifest.json` (required, `manifest_version`
0.3) + the server payload; it supports **compiled binaries** — ideal for our Rust
binary. Official tooling: `@anthropic-ai/mcpb` (`init`/`validate`/`pack`/`sign`).
(Refs: github.com/anthropics/mcpb, claude.com/docs/connectors/building/mcpb.)

## What changed

- **`mcpb/manifest.json`** — `type: "binary"` server, `args: ["--workspace",
  "${user_config.workspace}"]`, a single `user_config.workspace` directory
  picker, and `platform_overrides` for distinct per-OS binaries
  (`bwoc-mcp-darwin` / `-linux` / `-win32.exe`). Read-only by default.
- **`scripts/build-mcpb.sh`** — `cargo build --release`, stage the host binary
  under its platform name, then `mcpb validate` + `mcpb pack` (falls back to
  `npx @anthropic-ai/mcpb`, then a plain `zip`). Verified end-to-end on macOS:
  the official CLI validated the manifest and produced a 1.7 MB `bwoc-mcp.mcpb`.
- **`.github/workflows/release.yml`** — build jobs now `upload-artifact` the raw
  binary; a new `pack-mcpb` job `lipo`s the two macOS arches into a universal2,
  stages Linux-x64 + Windows-x64, packs, and attaches `bwoc-mcp.mcpb` to the
  release.
- **README** — a "one-click (`.mcpb`)" section above the manual config; `.gitignore`
  ignores `mcpb/server/` + `*.mcpb`.

## Decisions

- **Read-only by default.** MCPB `user_config` booleans can't conditionally add a
  flag (they substitute as a literal `true`/`false`), so there's no clean
  one-click toggle for `--allow-write`/`--allow-exec`. Least-privilege wins: the
  bundle is read-only; the write/exec tiers stay on the documented manual-config
  path. A future signed "power" bundle or an MCPB conditional-arg feature can
  revisit.
- **One binary per OS, not per arch.** MCPB `platform_overrides` keys on
  `process.platform` (darwin/linux/win32), not arch. macOS is covered universally
  via `lipo`; Linux/Windows ship x64 in the single bundle. aarch64-linux builds
  from source / uses the tarball (noted in the workflow comment).
- **`bwoc-plugin-claude` is unrelated** — that's a Claude *Code* (CLI) plugin; the
  desktop *app* path is this MCP bundle. Different host, different mechanism.

## Status / deferred

- Shipped: the bundle + build script + release attachment + docs (local build
  verified). The release `pack-mcpb` job is CI-path (runs on the next tag).
- Deferred: signing the bundle (`mcpb sign`) for tamper-evidence; a write/exec
  variant.

## Related

- the `/investigate` that surfaced this (BWOC↔Claude-app support); `bwoc-mcp`
  server; `mcpb/manifest.json`, `scripts/build-mcpb.sh`, `release.yml`.
