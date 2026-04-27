"use client";

import { useMutation, useQuery } from "@tanstack/react-query";
import { AlertCircle, Bot, Menu, MessageSquare, Send, UserRound, Wifi } from "lucide-react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { fetchConversations, fetchHealth, Message, sendChat } from "@/lib/api";
import { useMemo, useState } from "react";

export function ChatApp() {
  const [message, setMessage] = useState("");
  const [activeConversation, setActiveConversation] = useState<string | undefined>();
  const [messages, setMessages] = useState<Message[]>([]);
  const [sidebarOpen, setSidebarOpen] = useState(false);

  const health = useQuery({ queryKey: ["health"], queryFn: fetchHealth, retry: false });
  const conversations = useQuery({ queryKey: ["conversations"], queryFn: fetchConversations, retry: false });
  const chat = useMutation({
    mutationFn: () => sendChat(message, activeConversation),
    onSuccess: (response) => {
      setActiveConversation(response.conversation_id);
      setMessages((current) => [...current, response.user_message, response.assistant_message]);
      setMessage("");
    },
  });

  const canSend = message.trim().length > 0 && !chat.isPending;
  const status = useMemo(() => {
    if (health.isLoading) return { label: "Checking", tone: "text-zinc-300" };
    if (health.isError) return { label: "Offline", tone: "text-red-300" };
    return { label: "Healthy", tone: "text-emerald-300" };
  }, [health.isError, health.isLoading]);

  return (
    <main className="flex min-h-screen bg-zinc-950 text-zinc-100">
      <aside
        className={`fixed inset-y-0 left-0 z-20 w-72 border-r border-zinc-800 bg-zinc-950 p-4 transition md:static md:translate-x-0 ${
          sidebarOpen ? "translate-x-0" : "-translate-x-full"
        }`}
      >
        <div className="mb-5 flex items-center gap-2 text-sm font-semibold">
          <Bot className="h-5 w-5 text-emerald-300" />
          GPT Copy v3
        </div>
        <button
          className="mb-4 w-full rounded-md border border-zinc-700 px-3 py-2 text-left text-sm hover:bg-zinc-900"
          onClick={() => {
            setActiveConversation(undefined);
            setMessages([]);
            setSidebarOpen(false);
          }}
        >
          New chat
        </button>
        <div className="space-y-2">
          {(conversations.data ?? []).map((conversation) => (
            <button
              key={conversation.id}
              className="flex w-full items-center gap-2 rounded-md px-3 py-2 text-left text-sm text-zinc-300 hover:bg-zinc-900"
              onClick={() => {
                setActiveConversation(conversation.id);
                setSidebarOpen(false);
              }}
            >
              <MessageSquare className="h-4 w-4" />
              <span className="truncate">{conversation.title}</span>
            </button>
          ))}
        </div>
      </aside>

      <section className="flex min-h-screen flex-1 flex-col">
        <header className="flex h-14 items-center justify-between border-b border-zinc-800 px-4">
          <div className="flex items-center gap-3">
            <button className="md:hidden" aria-label="Open conversations" onClick={() => setSidebarOpen(true)}>
              <Menu className="h-5 w-5" />
            </button>
            <h1 className="text-sm font-semibold">Chat</h1>
          </div>
          <div className={`flex items-center gap-2 text-xs ${status.tone}`} aria-label={`Backend ${status.label}`}>
            <Wifi className="h-4 w-4" />
            {status.label}
          </div>
        </header>

        <div className="flex-1 overflow-y-auto px-4 py-6">
          {messages.length === 0 ? (
            <div className="mx-auto flex max-w-2xl flex-col items-center justify-center py-24 text-center">
              <Bot className="mb-4 h-10 w-10 text-emerald-300" />
              <h2 className="text-2xl font-semibold">How can I help?</h2>
              <p className="mt-2 text-sm text-zinc-400">Start a conversation with the local Axum backend.</p>
            </div>
          ) : (
            <div className="mx-auto max-w-3xl space-y-5">
              {messages.map((item) => (
                <article key={item.id} className="flex gap-3">
                  <div className="mt-1 flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-zinc-800">
                    {item.role === "user" ? <UserRound className="h-4 w-4" /> : <Bot className="h-4 w-4" />}
                  </div>
                  <div className="min-w-0 flex-1 rounded-lg bg-zinc-900 px-4 py-3 text-sm leading-6">
                    <ReactMarkdown remarkPlugins={[remarkGfm]}>{item.content}</ReactMarkdown>
                  </div>
                </article>
              ))}
            </div>
          )}
        </div>

        <form
          className="border-t border-zinc-800 p-4"
          onSubmit={(event) => {
            event.preventDefault();
            if (canSend) chat.mutate();
          }}
        >
          {chat.isError ? (
            <div className="mx-auto mb-3 flex max-w-3xl items-center gap-2 rounded-md border border-red-900 bg-red-950 px-3 py-2 text-sm text-red-200">
              <AlertCircle className="h-4 w-4" />
              {(chat.error as Error).message}
            </div>
          ) : null}
          <div className="mx-auto flex max-w-3xl items-end gap-2 rounded-lg border border-zinc-700 bg-zinc-900 p-2">
            <textarea
              aria-label="Message"
              className="max-h-40 min-h-12 flex-1 resize-none bg-transparent px-2 py-2 text-sm outline-none placeholder:text-zinc-500"
              placeholder="Message GPT Copy"
              value={message}
              onChange={(event) => setMessage(event.target.value)}
            />
            <button
              aria-label="Send message"
              className="rounded-md bg-emerald-400 p-3 text-zinc-950 disabled:cursor-not-allowed disabled:bg-zinc-700 disabled:text-zinc-400"
              disabled={!canSend}
              type="submit"
            >
              <Send className="h-4 w-4" />
            </button>
          </div>
        </form>
      </section>
    </main>
  );
}
