import { describe, expect, it, vi } from "vitest";
import { sendChat } from "./api";

describe("api client", () => {
  it("validates chat responses", async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        conversation_id: "c1",
        user_message: { id: "u1", conversation_id: "c1", role: "user", content: "hello", created_at: "now" },
        assistant_message: { id: "a1", conversation_id: "c1", role: "assistant", content: "world", created_at: "now" },
      }),
    });

    await expect(sendChat("hello")).resolves.toMatchObject({ conversation_id: "c1" });
  });
});
