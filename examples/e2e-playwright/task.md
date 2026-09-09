# E2E task for Playwright verification

Build a working static Order Desk task-management app in the current directory.
Use plain HTML, CSS, and JavaScript with no external dependencies or network calls.
Write the files directly. Do not commit, push, install tools, or delegate work.
The caller runs the browser tests. Do not search for browser tooling or run
browser tests yourself; finish after writing the app and checking its logic.

Acceptance criteria:

1. File `index.html` at the project root, with static `styles.css` and `app.js`.
2. Document title: `ccswarm order desk`.
3. The first screen is the usable desk, not a landing page.
4. Show seeded orders with statuses `ready` and `partial`.
5. Allow adding a new order.
6. Allow advancing an order through plan -> sangha -> implement -> review,
   ending with status `ready`. These are local demo states, not live AI jobs.
7. Allow marking an order partial and retrying it from a recovery point.
8. Persist order state in `localStorage`.
9. Allow deleting an order. Reject blank titles. Render user text safely.
10. Use accessible labels, keyboard controls, responsive layout, and a polished
    navy-and-teal desk with readable typography and clear status indicators.

Browser-test interface:

- Heading `Order Desk`, input placeholder `Describe the work order`, and button
  `Add order`.
- Counters: `#metric-orders` (all orders), `#metric-partial` (partial orders),
  and `#summary-ready` (ready orders).
- Each `.order-card` has its title as a heading, a `.status-pill`, and buttons
  `Advance`, `Mark partial`, `Retry`, and `Delete` when appropriate.
- New orders start `queued`. The first Advance enters plan (`running`); the
  next three advances reach sangha, implement, then review (`ready`).
- Mark partial retains the current stage and shows `.recovery-note` containing
  `Recovery point saved`. Retry restores that stage as `running`.
- Blank submission shows `Describe the order first.` in `#form-error`.
- Seed exactly two orders on first use: one `ready`, one `partial`. An empty
  saved list must remain empty after reload. Explain storage errors visibly.

The test harness runs headless Chromium against `file://$PWD/index.html` and
asserts that the desk can add, advance, mark partial, retry, delete, and persist
state. Include a short README with opening instructions and the local-only limit.
