/**
 * Zod schemas for chat API validation
 * 
 * Validates API responses before they enter Zustand stores,
 * ensuring type safety and runtime validation.
 * 
 * @author @darianrosebrook
 */

import { z } from 'zod';

/**
 * Task schema for chat message tasks
 */
export const TaskSchema = z.object({
  id: z.string(),
  name: z.string(),
  status: z.enum(['pending', 'in-progress', 'completed', 'failed']),
  result: z.string().optional(),
  timestamp: z.date().or(z.string().transform((str) => new Date(str))),
});

/**
 * Message schema for chat messages
 */
export const MessageSchema = z.object({
  id: z.string(),
  role: z.enum(['user', 'assistant']),
  content: z.string(),
  timestamp: z.date().or(z.string().transform((str) => new Date(str))),
  isLoading: z.boolean().optional(),
  tasks: z.array(TaskSchema).optional(),
  contextFiles: z.array(z.string()).optional(),
  isPhasePlan: z.boolean().optional(),
  isGeneratingPlan: z.boolean().optional(),
});

/**
 * Chat session schema
 */
export const ChatSessionSchema = z.object({
  id: z.string(),
  title: z.string(),
  createdAt: z.date().or(z.string().transform((str) => new Date(str))),
  updatedAt: z.date().or(z.string().transform((str) => new Date(str))).optional(),
  messageCount: z.number().int().nonnegative().default(0),
  groupId: z.string().optional(),
});

/**
 * Chat data schema (session with messages)
 */
export const ChatDataSchema = z.object({
  id: z.string(),
  title: z.string(),
  messages: z.array(MessageSchema).default([]),
  createdAt: z.date().or(z.string().transform((str) => new Date(str))),
  groupId: z.string().optional(),
});

/**
 * API response schemas
 */
export const ChatSessionResponseSchema = z.object({
  id: z.string(),
  title: z.string(),
  created_at: z.string().transform((str) => new Date(str)),
  updated_at: z.string().transform((str) => new Date(str)),
  message_count: z.number().int().nonnegative().default(0),
});

export const ChatSessionsResponseSchema = z.array(ChatSessionResponseSchema);

export const ChatMessageResponseSchema = z.object({
  id: z.string(),
  role: z.string(),
  content: z.string(),
  timestamp: z.string().transform((str) => new Date(str)),
  metadata: z.record(z.unknown()).optional(),
});

export const ChatMessagesResponseSchema = z.array(ChatMessageResponseSchema);

/**
 * Create chat session request schema
 */
export const CreateChatSessionRequestSchema = z.object({
  title: z.string().optional(),
});

/**
 * Stream agent request schema
 */
export const StreamAgentRequestSchema = z.object({
  agent_id: z.string(),
  session_id: z.string(),
  message: z.string(),
  context_files: z.array(z.string()).optional(),
});

/**
 * Stream event schema (SSE)
 */
export const StreamEventSchema = z.object({
  content: z.string().optional(),
  done: z.boolean().default(false),
  error: z.string().optional(),
});

// Type exports derived from schemas
export type Task = z.infer<typeof TaskSchema>;
export type Message = z.infer<typeof MessageSchema>;
export type ChatSession = z.infer<typeof ChatSessionSchema>;
export type ChatData = z.infer<typeof ChatDataSchema>;
export type ChatSessionResponse = z.infer<typeof ChatSessionResponseSchema>;
export type ChatMessageResponse = z.infer<typeof ChatMessageResponseSchema>;
export type CreateChatSessionRequest = z.infer<typeof CreateChatSessionRequestSchema>;
export type StreamAgentRequest = z.infer<typeof StreamAgentRequestSchema>;
export type StreamEvent = z.infer<typeof StreamEventSchema>;

