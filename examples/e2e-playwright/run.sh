#!/usr/bin/env bash
# End-to-end verification: ccswarm generates a web app, Playwright tests it.
#
# Requires: ANTHROPIC_API_KEY exported, `claude` CLI logged in, Node.js 20+.
# Run from the repo root:
#   ./examples/e2e-playwright/run.sh
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GEN="$HERE/generated"
TASK="$(cat "$HERE/task.md")"

# 1. Clean previous run artifacts.
rm -rf "$GEN"
mkdir -p "$GEN"

# 2. Run the pipeline inside the generated/ dir so ccswarm writes index.html there.
cd "$GEN"
cargo run --manifest-path "$HERE/../../crates/ccswarm/Cargo.toml" --release -- \
  pipeline --task "$TASK" --piece quick --timeout 600

# 3. Install Playwright locally (first run only).
cd "$HERE"
if [ ! -d "node_modules/@playwright/test" ]; then
  npm init -y >/dev/null
  npm install --save-dev @playwright/test
  npx playwright install chromium
fi

# 4. Run the browser test against the generated file:// URL.
npx playwright test playwright.spec.mjs

echo
echo "✓ E2E verification passed."
