# 2026-06-23 — First-stable releases: bwoc-mcp v1.1.0 + bwoc-plugin-claude v1.0.0

Closed out the BWOC↔Claude integration arc by cutting public releases on both
sides of the integration.

## What shipped

- **bwoc-mcp v1.1.0** — the one-click Claude Desktop story. Release attaches
  `bwoc-mcp.mcpb` (10.5 MB: macOS universal2 + Linux x64 + Windows x64) alongside
  the 5 platform tarballs/zip. Also ships MCP tool annotations on all 24 tools.
  https://github.com/bemindlabs/bwoc-mcp/releases/tag/v1.1.0
- **bwoc-plugin-claude v1.0.0** — first stable of the Claude Code plugin (8
  commands, 1 agent, 2 skills, hooks, marketplace metadata, CI gate). First
  release this repo has ever had.
  https://github.com/bemindlabs/bwoc-plugin-claude/releases/tag/v1.0.0

## Version reasoning (the one judgement call)

The user said "all v1.0.0". That applies literally only to the plugin (was
v0.1.0, no prior releases → clean v1.0.0). **bwoc-mcp was already public at
v1.0.1**, so it cannot go to v1.0.0 — the pending `.mcpb` + annotations work is a
backward-compatible feature add → **minor bump to v1.1.0**. Confirmed by the user
("bwoc-mcp 1.1.0") mid-run.

## Process notes

- bwoc-mcp release is fully automated: push a `v*` tag → `release.yml` builds the
  matrix, the `pack-mcpb` job `lipo`s the darwin binaries to universal2, runs
  `mcpb validate`+`pack`, and `softprops/action-gh-release@v2` creates the release
  with all assets. The action creates the release with an **empty body** — set the
  notes afterward with `gh release edit --notes`.
- bwoc-plugin-claude has **no** release workflow → tag + `gh release create`
  manually.
- Both repos accepted `gh pr merge --squash --auto` (auto-merge armed fine even
  though I'd noted plugin auto-merge as "OFF" — the flag took without error).
- Cleanup: squash-merges leave branch tips that are NOT ancestors of main, so
  `git merge-base --is-ancestor` reports them un-merged. Verify via
  `gh pr list --state merged --head <branch>` before deleting remote branches.
  Pruned 3 stale merged branches on the plugin repo.

## Related

- the `.mcpb` work (`2026-06-22_mcpb-desktop-extension.md`), tool annotations
  (`2026-06-22_tool-annotations-and-directory.md`), and the `/investigate`
  BWOC↔Claude-app thread that kicked this off.
