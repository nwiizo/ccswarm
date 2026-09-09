import { test, expect } from "@playwright/test";
import { fileURLToPath, pathToFileURL } from "node:url";
import path from "node:path";

const here = path.dirname(fileURLToPath(import.meta.url));
const appDir = process.env.CCSWARM_APP_DIR
  ? path.resolve(process.env.CCSWARM_APP_DIR)
  : path.join(here, "generated");
const indexPath = path.join(appDir, "index.html");
const url = pathToFileURL(indexPath).href;

test.beforeEach(async ({ page }) => {
  await page.goto(url);
  await page.evaluate(() => window.localStorage.clear());
  await page.reload();
});

test("order desk supports add, partial recovery, retry, and persistence", async ({ page }, testInfo) => {
  await expect(page).toHaveTitle("ccswarm order desk");
  await expect(page.getByRole("heading", { name: "Order Desk" })).toBeVisible();
  await expect(page.locator("#metric-orders")).toHaveText("2");
  await expect(page.locator("#metric-partial")).toHaveText("1");
  await expect(page.locator("#summary-ready")).toHaveText("1");

  await page.getByPlaceholder("Describe the work order").fill("Ship E2E dogfood app");
  await page.getByRole("button", { name: "Add order" }).click();
  await expect(page.locator("#metric-orders")).toHaveText("3");
  await expect(page.getByRole("heading", { name: "Ship E2E dogfood app" })).toBeVisible();

  const newOrder = page.locator(".order-card").filter({ hasText: "Ship E2E dogfood app" });
  await expect(newOrder.locator(".status-pill")).toHaveText("queued");

  await newOrder.getByRole("button", { name: "Advance" }).click();
  await expect(newOrder.locator(".status-pill")).toHaveText("running");
  await newOrder.getByRole("button", { name: "Advance" }).click();

  await newOrder.getByRole("button", { name: "Mark partial" }).click();
  await expect(newOrder.locator(".status-pill")).toHaveText("partial");
  await expect(page.locator("#metric-partial")).toHaveText("2");
  await expect(newOrder.locator(".recovery-note")).toContainText("Recovery point saved");

  await newOrder.getByRole("button", { name: "Retry" }).click();
  await expect(newOrder.locator(".status-pill")).toHaveText("running");

  await newOrder.getByRole("button", { name: "Advance" }).click();
  await newOrder.getByRole("button", { name: "Advance" }).click();
  await expect(newOrder.locator(".status-pill")).toHaveText("ready");

  await page.reload();
  await expect(page.getByRole("heading", { name: "Ship E2E dogfood app" })).toBeVisible();
  await expect(page.locator("#summary-ready")).toHaveText("2");
  await page.screenshot({ path: testInfo.outputPath("order-desk.png"), fullPage: true });
});

test("deleting every order stays empty after reload", async ({ page }) => {
  while (await page.locator(".order-card").count()) {
    await page.locator(".order-card").first().getByRole("button", { name: "Delete", exact: true }).click();
  }
  await expect(page.locator("#metric-orders")).toHaveText("0");
  await page.reload();
  await expect(page.locator(".order-card")).toHaveCount(0);
  await expect(page.locator("#metric-orders")).toHaveText("0");
});

test("order titles remain literal text after saving", async ({ page }) => {
  const title = '<img src=x onerror="document.body.dataset.injected=1">';
  await page.getByPlaceholder("Describe the work order").fill(title);
  await page.getByRole("button", { name: "Add order" }).click();
  await page.reload();
  await expect(page.getByRole("heading", { name: title, exact: true })).toBeVisible();
  await expect(page.locator(".order-card img")).toHaveCount(0);
  await expect(page.locator("body")).not.toHaveAttribute("data-injected", "1");
});

test("the desk remains usable on a narrow screen", async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.getByPlaceholder("Describe the work order").fill("A mobile task");
  await page.getByRole("button", { name: "Add order" }).click();
  await expect(page.getByRole("heading", { name: "A mobile task" })).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(390);
  await page.screenshot({ path: testInfo.outputPath("order-desk-mobile.png"), fullPage: true });
});

test("form validation keeps empty orders out of the desk", async ({ page }) => {
  await page.getByRole("button", { name: "Add order" }).click();
  await expect(page.locator("#form-error")).toHaveText("Describe the order first.");
  await expect(page.locator("#metric-orders")).toHaveText("2");
});
