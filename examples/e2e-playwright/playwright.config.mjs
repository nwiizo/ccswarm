import { defineConfig } from "@playwright/test";

const executablePath = process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE;

export default defineConfig({
  testDir: ".",
  testMatch: "playwright.spec.mjs",
  use: {
    browserName: "chromium",
    headless: true,
    launchOptions: executablePath ? { executablePath } : {},
  },
});
