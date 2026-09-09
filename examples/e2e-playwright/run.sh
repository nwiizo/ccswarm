#!/usr/bin/env bash
# End-to-end verification for the ccswarm Order Desk dogfood app.
#
# By default, test the checked-in app and preview without calling a provider.
# With --live, generate a fresh app through ccswarm and test that exact output.
#
# Run from the repo root:
#   ./examples/e2e-playwright/run.sh
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
TASK="$(cat "$HERE/task.md")"
LIVE=false
case "${1:-}" in
  --live) LIVE=true ;;
  "") ;;
  *) echo "Usage: $0 [--live]" >&2; exit 2 ;;
esac
if [ "$#" -gt 1 ]; then
  echo "Usage: $0 [--live]" >&2
  exit 2
fi
mkdir -p "$HERE/.work"
WORK="$(mktemp -d "$HERE/.work/run.XXXXXX")"
# Give provider CLIs a project boundary separate from the ccswarm repository.
git -C "$WORK" init -b main >/dev/null

# Resolve relative binary paths before Playwright changes the working directory.
if [ -n "${CCSWARM_BIN:-}" ] && [[ "$CCSWARM_BIN" == */* ]]; then
  CCSWARM_BIN="$(cd "$(dirname "$CCSWARM_BIN")" && pwd)/$(basename "$CCSWARM_BIN")"
fi

run_ccswarm() {
  if [ -n "${CCSWARM_BIN:-}" ]; then
    "$CCSWARM_BIN" "$@"
  else
    cargo run --manifest-path "$ROOT/crates/ccswarm/Cargo.toml" -- "$@"
  fi
}

run_ccswarm --repo "$WORK" queue add --file "$HERE/task.md" --flow quick
run_ccswarm --repo "$WORK" --json queue list >"$WORK/queue.json"
run_ccswarm --repo "$WORK" --provider codex pipeline --task "$TASK" --flow quick --dry-run \
  >"$WORK/preview.txt"

grep -q '"total": 1' "$WORK/queue.json"
grep -q 'Build a working static Order Desk' "$WORK/preview.txt"

if [ "$LIVE" = true ]; then
  echo "Generating a fresh app in $WORK (uses your authenticated provider)."
  run_ccswarm --repo "$WORK" pipeline --provider "${CCSWARM_PROVIDER:-codex}" \
    --task "$TASK" --flow quick --timeout 900 --output-format json \
    --output-file "$WORK/pipeline-result.json" < /dev/null
  for file in index.html styles.css app.js; do
    test -s "$WORK/$file"
  done
  export CCSWARM_APP_DIR="$WORK"
else
  echo "Testing the checked-in app; preview only, no provider execution."
  export CCSWARM_APP_DIR="$HERE/generated"
fi

cd "$HERE"
if [ ! -d "node_modules/@playwright/test" ]; then
  npm ci
fi
if [ -z "${PLAYWRIGHT_CHROMIUM_EXECUTABLE:-}" ] && ! npx playwright install --list | grep -q 'chromium_headless_shell'; then
  npx playwright install chromium chromium-headless-shell
fi

npx playwright test

echo
echo "OK Browser verification passed for $CCSWARM_APP_DIR"
echo "Run evidence: $WORK"
