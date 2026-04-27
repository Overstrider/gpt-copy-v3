import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { describe, expect, it, vi } from "vitest";
import { ChatApp } from "./chat-app";

function mockFetchOnce(payload: unknown, ok = true) {
  return vi.fn().mockResolvedValue({
    ok,
    json: async () => payload,
  });
}

function renderChat() {
  const client = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });
  return render(
    <QueryClientProvider client={client}>
      <ChatApp />
    </QueryClientProvider>,
  );
}

describe("ChatApp", () => {
  it("shows backend health state", async () => {
    global.fetch = mockFetchOnce({ status: "ok" });
    renderChat();
    expect(await screen.findByLabelText("Backend Healthy")).toBeInTheDocument();
  });

  it("sends a message and renders the assistant reply", async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce({ ok: true, json: async () => ({ status: "ok" }) })
      .mockResolvedValueOnce({ ok: true, json: async () => [] })
      .mockResolvedValueOnce({
        ok: true,
        body: new ReadableStream({
          start(controller) {
            controller.enqueue(new TextEncoder().encode("event: token\ndata: Hi\n\n"));
            controller.enqueue(new TextEncoder().encode("event: token\ndata: backend\n\n"));
            controller.enqueue(new TextEncoder().encode("event: done\ndata: c1\n\n"));
            controller.close();
          },
        }),
      });
    global.fetch = fetchMock;
    renderChat();

    await userEvent.type(screen.getByLabelText("Message"), "Hello");
    await userEvent.click(screen.getByLabelText("Send message"));

    await waitFor(() => expect(screen.getByText(/Hi backend/)).toBeInTheDocument());
    expect(fetchMock).toHaveBeenCalledWith(
      "http://localhost:8080/api/chat/stream",
      expect.objectContaining({ method: "POST" }),
    );
  });
});
