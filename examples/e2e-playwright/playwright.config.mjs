import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: ".",
  testMatch: /.*\.spec\.mjs$/,
  use: { headless: true },
});
