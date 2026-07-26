# E2E task for Playwright verification

Build a static ccswarm Order Desk app that demonstrates the product concepts
from `docs/CCSWARM_PRODUCT_ABSTRACTION_PLAN.md`.

Acceptance criteria:

1. File `index.html` at the project root, with static `styles.css` and `app.js`.
2. Document title: `ccswarm order desk`.
3. The first screen is the usable desk, not a landing page.
4. Show seeded orders with statuses `ready` and `partial`.
5. Allow adding a new order.
6. Allow advancing an order through plan -> sangha -> implement -> review.
7. Allow marking an order partial and retrying it from a recovery point.
8. Persist order state in `localStorage`.

The test harness runs headless Chromium against `file://$PWD/index.html` and
asserts that the desk can add, advance, mark partial, retry, and persist state.
