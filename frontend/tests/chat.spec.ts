import { expect, test } from "@playwright/test";

test("sends a chat message", async ({ page }) => {
  await page.route("**/health", async (route) => {
    await route.fulfill({ json: { status: "ok" } });
  });
  await page.route("**/api/conversations", async (route) => {
    if (route.request().method() === "GET") {
      await route.fulfill({ json: [] });
    } else {
      await route.fulfill({ json: { id: "c1", title: "New chat", created_at: "now", updated_at: "now" } });
    }
  });
  await page.route("**/api/chat/stream", async (route) => {
    await route.fulfill({
      contentType: "text/event-stream",
      body: "event: token\ndata: Hello\n\nevent: token\ndata: back\n\nevent: done\ndata: c1\n\n",
    });
  });

  await page.goto("/");
  await expect(page.getByLabel("Backend Healthy")).toBeVisible();
  await page.getByRole("textbox", { name: "Message" }).fill("Hello");
  await page.getByLabel("Send message").click();
  await expect(page.getByText("Hello back")).toBeVisible();
});
