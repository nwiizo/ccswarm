# E2E task for Playwright verification

Build a single-file static web app that meets these acceptance criteria:

1. File `index.html` at the project root (no build tooling, no npm).
2. Document title: `ccswarm counter`.
3. One visible `<h1>` with text `Counter`.
4. One `<button id="inc">+1</button>` button.
5. One `<span id="count">0</span>` that displays the count.
6. Inline `<script>` that increments `#count` by 1 each time `#inc` is clicked.
7. No external dependencies — pure HTML + inline JS.

Keep it minimal. The test harness runs headless Chromium against `file://$PWD/index.html`
and asserts (a) title matches, (b) initial count is `0`, (c) after three `#inc` clicks
the count is `3`.
