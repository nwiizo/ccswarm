# ccswarm E2E Playwright Dogfood

This example verifies a small static Order Desk app while exercising ccswarm before
the browser test runs.

The script uses:

- `queue add --file` to register the task spec.
- `queue list --json` to validate queue state.
- `pipeline --dry-run` to validate workflow planning without calling a provider.
- Playwright to use the generated app through the browser.

Run it from the repository root:

```sh
./examples/e2e-playwright/run.sh
```

To reuse an already built binary:

```sh
CCSWARM_BIN=target/debug/ccswarm ./examples/e2e-playwright/run.sh
```

Runtime files are written under `.work/`, `generated/ccswarm-*.{json,txt}`, and
`test-results/`; these are ignored by git.
