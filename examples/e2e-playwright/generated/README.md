# ccswarm order desk

Open `index.html` directly in a modern browser. Keep `styles.css` and `app.js`
beside it. No server, build step, installation, or internet connection is needed.

- Enter a title and click **Add order** (or press Enter). New orders are queued.
- Click **Advance** four times to visit plan → sangha → implement → review.
  Reaching review marks the order ready.
- **Mark partial** pauses a running order at its current stage. **Retry** returns
  that stage to running; use Advance to continue.
- Use the status filters to find orders, and **Delete** to remove them.

This is a local-only demo. Stages are changed manually; no AI jobs, accounts,
network calls, or background work are involved. Two example orders appear on
first use: one ready and one partial.

Orders are saved in this browser's `localStorage`, under
`ccswarm-order-desk-v1`. Deleting every order saves an empty desk, including after
reload. Storage is specific to the browser/profile and may also depend on the
file's location; moving the files or clearing browser data can reset the desk.
Use one tab at a time; simultaneous edits across tabs are not synchronized.

This newly generated demo uses separate storage from the v0.10.0 sample
(`ccswarm-order-desk:v1`). It does not import or delete that sample's orders.
To read the old orders, open the v0.10.0 app at its original file location in
the same browser profile. Keep the old files before replacing them if those
orders matter to you.

Storage errors appear on the desk. If saved data cannot be read or is invalid,
the app leaves it untouched and allows unsaved work in the current tab. If a
write fails, work stays in the current tab and the next change retries saving.
Unsaved changes may be lost on reload.

## Tests

With Node.js 22 or newer already available, run the dependency-free unit tests:

```sh
node --test tests/orders.test.cjs
```

Browser tests are maintained in the parent example directory. From the
repository root, run:

```sh
examples/e2e-playwright/run.sh
```

See the [example guide](../README.md) for browser setup, screenshots, and live
generation. The current generation instructions are in [task.md](../task.md).
