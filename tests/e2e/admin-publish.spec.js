const { test, expect } = require("@playwright/test");

const ONE_PIXEL_PNG = Buffer.from(
  "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+/p9sAAAAASUVORK5CYII=",
  "base64",
);

const adminUsername = process.env.E2E_ADMIN_USERNAME || process.env.ADMIN_USERNAME;
const adminPassword = process.env.E2E_ADMIN_PASSWORD || process.env.ADMIN_PASSWORD;

function slugify(title) {
  let slug = "";
  let needsHyphen = false;

  for (const char of title) {
    const lower = char.toLowerCase();
    if (/^[a-z0-9]$/.test(lower)) {
      if (needsHyphen && slug) {
        slug += "-";
      }
      slug += lower;
      needsHyphen = false;
    } else if (slug) {
      needsHyphen = true;
    }

    if (slug.length >= 180) {
      break;
    }
  }

  return slug.replace(/-+$/, "") || "post";
}

async function expectResponseOk(response) {
  if (response.ok()) {
    return;
  }

  throw new Error(
    `${response.request().method()} ${response.url()} failed with ${response.status()}: ${await response.text()}`,
  );
}

test.describe("admin publishing flow", () => {
  test.skip(
    !adminUsername || !adminPassword,
    "Set E2E_ADMIN_USERNAME/E2E_ADMIN_PASSWORD or ADMIN_USERNAME/ADMIN_PASSWORD to run this flow.",
  );

  test("admin logs in, uploads media, publishes a post, and views it publicly", async ({ page }) => {
    const runId = Date.now();
    const title = `E2E Uploaded Media Post ${runId}`;
    const slug = slugify(title);
    const intro = `This post proves the E2E media publishing flow ${runId}.`;

    await page.goto("/admin");
    await expect(page).toHaveURL(/\/admin\/login$/);

    await page.getByLabel("Username").fill(adminUsername);
    await page.getByLabel("Password").fill(adminPassword);
    await Promise.all([
      page.waitForURL(/\/admin\/?$/),
      page.getByRole("button", { name: "Sign in" }).click(),
    ]);

    await expect(page.getByRole("heading", { name: "Publishing workspace" })).toBeVisible();

    await page.goto("/admin/posts/new");
    await expect(page.locator("[data-post-editor]")).toBeVisible();
    await page.locator("[data-post-category]").selectOption({ label: "Thoughts" });

    await page.locator("[data-post-media-upload] input[type='file']").setInputFiles({
      name: `e2e-media-${runId}.png`,
      mimeType: "image/png",
      buffer: ONE_PIXEL_PNG,
    });

    const uploadResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/admin/api/media/upload") &&
        response.request().method() === "POST",
    );
    await page.locator("[data-post-media-upload] button[type='submit']").click();
    const uploadResponse = await uploadResponsePromise;
    await expectResponseOk(uploadResponse);

    const insertButton = page.locator("[data-post-media-grid] button", { hasText: "Insert" }).last();
    await expect(insertButton).toBeVisible();
    await insertButton.click();

    const body = page.locator("[data-post-body]");
    await expect(body).toHaveValue(/!\[media:media\//);
    const embeddedMedia = await body.inputValue();

    await page.locator("[data-post-title]").fill(title);
    await page.locator("[data-post-status]").selectOption("published");
    await body.fill(`${intro}\n\n${embeddedMedia}`);

    const saveResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/admin/api/posts") &&
        response.request().method() === "POST",
    );
    await page.locator("[data-post-editor-form] button[type='submit']").click();
    const saveResponse = await saveResponsePromise;
    await expectResponseOk(saveResponse);
    const savedPost = await saveResponse.json();
    expect(savedPost.slug).toBe(slug);

    await expect(page).toHaveURL(/\/admin\/posts\/edit\?id=\d+$/);

    await page.goto(`/posts/${slug}`);
    await expect(page.getByRole("heading", { name: title })).toBeVisible();
    await expect(page.getByRole("link", { name: "Thoughts" })).toBeVisible();
    await expect(page.getByText(intro)).toBeVisible();
    await expect(page.locator("article img").first()).toBeVisible();
  });
});
