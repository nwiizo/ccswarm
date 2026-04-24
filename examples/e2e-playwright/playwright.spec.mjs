// Playwright spec that verifies the app ccswarm generated in ./generated/.
// Run with: node --test playwright.spec.mjs  (after `npm install playwright`).
import { test, expect } from "@playwright/test";
import { fileURLToPath, pathToFileURL } from "node:url";
import path from "node:path";

const here = path.dirname(fileURLToPath(import.meta.url));
const indexPath = path.join(here, "generated", "index.html");
const url = pathToFileURL(indexPath).href;

test("counter increments from 0 to 3", async ({ page }) => {
  await page.goto(url);
  await expect(page).toHaveTitle("ccswarm counter");
  await expect(page.locator("#count")).toHaveText("0");

  await page.click("#inc");
  await page.click("#inc");
  await page.click("#inc");

  await expect(page.locator("#count")).toHaveText("3");
});
