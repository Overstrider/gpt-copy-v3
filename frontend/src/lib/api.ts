import { z } from "zod";

const API_BASE = process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:8080";

export const MessageSchema = z.object({
  id: z.string(),
  conversation_id: z.string(),
  role: z.enum(["user", "assistant", "system"]).or(z.string()),
  content: z.string(),
  created_at: z.string(),
});

export const ConversationSchema = z.object({
  id: z.string(),
  title: z.string(),
  created_at: z.string(),
  updated_at: z.string(),
});

const ChatResponseSchema = z.object({
  conversation_id: z.string(),
  user_message: MessageSchema,
  assistant_message: MessageSchema,
});

const HealthSchema = z.object({ status: z.string() });

export type Message = z.infer<typeof MessageSchema>;
export type Conversation = z.infer<typeof ConversationSchema>;

async function parseJson<T>(response: Response, schema: z.ZodType<T>): Promise<T> {
  const payload = await response.json();
  if (!response.ok) {
    const message = payload?.error?.message ?? "Request failed";
    throw new Error(message);
  }
  return schema.parse(payload);
}

export async function fetchHealth() {
  const response = await fetch(`${API_BASE}/health`);
  return parseJson(response, HealthSchema);
}

export async function fetchConversations() {
  const response = await fetch(`${API_BASE}/api/conversations`);
  return parseJson(response, z.array(ConversationSchema));
}

export async function sendChat(message: string, conversationId?: string) {
  const response = await fetch(`${API_BASE}/api/chat`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ message, conversation_id: conversationId }),
  });
  return parseJson(response, ChatResponseSchema);
}
