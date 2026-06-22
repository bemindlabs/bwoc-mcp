#!/usr/bin/env bash
# build-mcpb.sh — build the one-click Claude Desktop extension (bwoc-mcp.mcpb).
#
# An `.mcpb` (MCP Bundle, formerly DXT) is a zip of `mcpb/manifest.json` + the
# compiled server under `mcpb/server/`. Claude Desktop installs it with one
# click and prompts only for the workspace directory (see mcpb/manifest.json).
#
# This stages the HOST platform's binary. For a cross-platform bundle, drop the
# other targets' binaries into mcpb/server/ (names below) before packing — the
# release workflow does this from the per-OS release artifacts.
#
#   mcpb/server/bwoc-mcp-darwin     (macOS)
#   mcpb/server/bwoc-mcp-linux      (Linux)
#   mcpb/server/bwoc-mcp-win32.exe  (Windows)

set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
srvdir="$root/mcpb/server"
mkdir -p "$srvdir"

echo "▸ cargo build --release --bin bwoc-mcp"
cargo build --release --bin bwoc-mcp

case "$(uname -s)" in
  Darwin) staged="bwoc-mcp-darwin" ;;
  Linux)  staged="bwoc-mcp-linux" ;;
  *) echo "unsupported host $(uname -s); build on macOS/Linux (Windows binary is staged from CI)"; exit 1 ;;
esac
cp "$root/target/release/bwoc-mcp" "$srvdir/$staged"
chmod +x "$srvdir/$staged"
echo "▸ staged mcpb/server/$staged"

# Prefer the official MCPB CLI (validates the manifest + zips correctly). It also
# supports `mcpb sign` for a signed, tamper-evident bundle.
if command -v mcpb >/dev/null 2>&1; then
  mcpb validate "$root/mcpb/manifest.json"
  mcpb pack "$root/mcpb" "$root/bwoc-mcp.mcpb"
elif command -v npx >/dev/null 2>&1; then
  npx -y @anthropic-ai/mcpb@latest validate "$root/mcpb/manifest.json"
  npx -y @anthropic-ai/mcpb@latest pack "$root/mcpb" "$root/bwoc-mcp.mcpb"
else
  # Fallback: an .mcpb is a plain zip with manifest.json at the archive root.
  echo "▸ @anthropic-ai/mcpb not found — packing with zip (install it for validation + signing)"
  ( cd "$root/mcpb" && zip -qr "$root/bwoc-mcp.mcpb" manifest.json server )
fi

echo "✓ wrote $root/bwoc-mcp.mcpb — double-click to install into Claude Desktop"
