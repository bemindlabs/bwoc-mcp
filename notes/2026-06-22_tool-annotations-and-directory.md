# 2026-06-22 — Tool annotations + Connectors-Directory / signing findings

Follow-up to the `.mcpb` work, chasing "can `bwoc-mcp` go in a Claude
marketplace / directory, and can the bundle be signed?"

## What changed

- **MCP tool annotations on all 24 tools** (`src/server.rs`). Each `#[tool(...)]`
  now declares behavior hints via rmcp 1.7's `annotations(...)`:
  - read tier (13) → `read_only_hint = true`
  - write tier (6) → `read_only_hint = false`
  - exec `run` (1) → `read_only_hint = false, open_world_hint = true` (spawns a
    model backend = external interaction)
  - dangerous `new`/`retire`/`start`/`stop` (4) → `read_only_hint = false,
    destructive_hint = true`
  - Applied by tier (derived from each tool's `--allow-*` description) via a
    one-off script; the two multi-line `#[tool(...)]` attributes (`run`,
    `retire`) were edited by hand. fmt + clippy + 7 tests green.

## Why (the directory research)

The **Anthropic Connectors Directory** (claude.ai) lists **remote/hosted** MCP
servers and gates on: a public **privacy policy** (missing = auto-reject), **per-
tool annotations** (`readOnlyHint`/`destructiveHint` + title), an OAuth callback
(`claude.ai/api/mcp/auth_callback`), test-account access, and a Team/Enterprise
org to submit from. `bwoc-mcp` is a **local-workspace** tool — the right
distribution channel is the **`.mcpb` desktop extension** (already shipped), not
the remote directory. So we did the one prerequisite that helps regardless of
ever submitting: **tool annotations**, which improve the read-only-vs-mutating UX
in *any* MCP host (including the desktop app via the `.mcpb`).

## Signing — deferred (upstream bug)

`mcpb sign` was the other "finish" item. Verified it does **not work** in
`@anthropic-ai/mcpb` **2.1.2**: both `--self-signed` and an explicit openssl
`--cert`/`--key` report *"Successfully signed"*, but the archive is byte-identical
afterward and `mcpb verify` / `mcpb info` both report **"not signed"** — the
sign→verify round-trip is broken. Not shipping a signing step that produces
unsigned bundles (it would also fail a `verify` gate). Revisit when the CLI fixes
it, or sign out-of-band.

## Status / deferred

- Done: tool annotations.
- Deferred: bundle **signing** (upstream CLI bug); **remote Connectors-Directory**
  listing (wrong fit for a local tool — needs hosting + OAuth + privacy policy).
- The plugin (`bwoc-plugin-claude`) is the *Claude Code* path and is separately
  directory-ready (submit via clau.de/plugin-directory-submission).

## Related

- the `/investigate` (BWOC↔Claude-app); `src/server.rs`; `mcpb/manifest.json`
  (the `.mcpb` work). Refs: claude.com/docs/connectors/building/submission.
