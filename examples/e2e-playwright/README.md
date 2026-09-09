# ccswarm E2E Playwright Dogfood

Open [`generated/index.html`](generated/index.html) in a browser to use Order
Desk. Add work, advance it through local demo stages, pause and retry it, or
delete it. State is stored in that browser's `localStorage`. This app does not
run AI jobs or synchronize data between browsers.
The regenerated app uses separate storage from the previous sample; see the
[app guide](generated/README.md) before replacing a copy with saved orders.

## Test the included app

Run from the repository root with Node.js/npm and a Rust toolchain available:

```sh
./examples/e2e-playwright/run.sh
```

This checks queue registration and prompt preview, then tests the included app
in Chromium. It does **not** call an AI provider or prove that generation works.
The browser tests cover creation, stage advancement, recovery, persistence,
deletion, literal user text, blank validation, and a narrow viewport.

To reuse an already built binary (relative paths are supported):

```sh
CCSWARM_BIN=target/debug/ccswarm ./examples/e2e-playwright/run.sh
```

## Generate and test a fresh app

Install and authenticate Codex CLI first, then run:

```sh
CCSWARM_BIN=target/release/ccswarm ./examples/e2e-playwright/run.sh --live
```

`--live` invokes the provider and consumes its usage allowance. It runs the
`quick` flow in a new `.work/run.XXXXXX/` directory, requires nonempty HTML,
CSS, and JavaScript output, and points Playwright at those exact files. It
never falls back to the included app. Set `CCSWARM_PROVIDER=claude` to use an
authenticated Claude Code CLI instead. No automatic commit or PR is requested.

The printed run directory retains the app, `pipeline-result.json`, queue and
preview output, and `.ccswarm/runs/` events for inspection. Each invocation
keeps prior runs. To use the fresh app, open that directory's `index.html`.

Browser-only verification of an existing run is also available:

```sh
cd examples/e2e-playwright
CCSWARM_APP_DIR=/absolute/path/to/run npm test
```

Set `PLAYWRIGHT_CHROMIUM_EXECUTABLE` to an installed Chromium/Chrome executable
to skip downloading a browser. Otherwise the script installs Playwright's
Chromium. Screenshots and failure details are saved in `test-results/`.
Both `.work/` and `test-results/` are ignored by Git.
