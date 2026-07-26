#!/usr/bin/env bash
# End-to-end verification for the ccswarm Order Desk dogfood app.
#
# This script intentionally exercises ccswarm features before the browser test:
#   1. queue add --file
#   2. queue list --json
#   3. pipeline --dry-run
#   4. Playwright against the static app
#
# Run from the repo root:
#   ./examples/e2e-playwright/run.sh
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
WORK="$HERE/.work"
TASK="$(cat "$HERE/task.md")"

run_ccswarm() {
  if [ -n "${CCSWARM_BIN:-}" ]; then
    "$CCSWARM_BIN" "$@"
  else
    cargo run --manifest-path "$ROOT/crates/ccswarm/Cargo.toml" -- "$@"
  fi
}

rm -rf "$WORK"
mkdir -p "$WORK"
git -C "$WORK" init -b main >/dev/null
git -C "$WORK" config user.email "ccswarm-e2e@local"
git -C "$WORK" config user.name "ccswarm e2e"

cat >"$WORK/README.md" <<'README'
# ccswarm E2E work repo

Temporary repository used by examples/e2e-playwright/run.sh.
README
git -C "$WORK" add README.md
git -C "$WORK" commit -m "init e2e work repo" >/dev/null

run_ccswarm --repo "$WORK" queue add --file "$HERE/task.md" --flow quick
run_ccswarm --repo "$WORK" --json queue list >"$HERE/generated/ccswarm-queue.json"
run_ccswarm --repo "$WORK" --provider codex pipeline --task "$TASK" --flow quick --dry-run \
  >"$HERE/generated/ccswarm-dry-run.txt"

grep -q '"total": 1' "$HERE/generated/ccswarm-queue.json"
grep -q 'Build a static ccswarm Order Desk app' "$HERE/generated/ccswarm-dry-run.txt"
grep -q 'Order Desk' "$HERE/task.md"

cd "$HERE"
if [ ! -d "node_modules/@playwright/test" ]; then
  npm install
fi
if ! npx playwright install --list | grep -q 'chromium_headless_shell'; then
  npx playwright install chromium chromium-headless-shell
fi

npx playwright test

echo
echo "OK E2E verification passed."
