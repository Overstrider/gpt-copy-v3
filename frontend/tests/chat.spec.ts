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
  await page.route("**/api/chat", async (route) => {
    await route.fulfill({
      json: {
        conversation_id: "c1",
        user_message: { id: "u1", conversation_id: "c1", role: "user", content: "Hello", created_at: "now" },
        assistant_message: { id: "a1", conversation_id: "c1", role: "assistant", content: "Hello back", created_at: "now" },
      },
    });
  });

  await page.goto("/");
  await expect(page.getByLabel("Backend Healthy")).toBeVisible();
  await page.getByRole("textbox", { name: "Message" }).fill("Hello");
  await page.getByLabel("Send message").click();
  await expect(page.getByText("Hello back")).toBeVisible();
});
