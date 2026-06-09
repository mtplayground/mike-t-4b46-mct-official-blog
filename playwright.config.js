const { defineConfig, devices } = require("@playwright/test");

const baseURL = process.env.E2E_BASE_URL || "http://127.0.0.1:8080";
const startServer = process.env.E2E_START_SERVER === "1";

module.exports = defineConfig({
  testDir: "./tests/e2e",
  timeout: 60_000,
  expect: {
    timeout: 10_000,
  },
  use: {
    baseURL,
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
  webServer: startServer
    ? {
        command: process.env.E2E_WEB_SERVER_COMMAND || "cargo run --no-default-features --features ssr",
        url: baseURL,
        reuseExistingServer: true,
        timeout: 120_000,
      }
    : undefined,
});
