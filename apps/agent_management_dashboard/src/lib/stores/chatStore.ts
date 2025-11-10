/**
 * Zustand store for chat state management
 *
 * Uses Zod schemas to validate API responses before updating state.
 * Implements optimistic updates with rollback on failure.
 *
 * Adapted from open-webui patterns for agent-agency.
 *
 * @author @darianrosebrook
 */

import { create } from "zustand";
import { devtools } from "zustand/middleware";
import { z } from "zod";
import {
  MessageSchema,
  ChatSessionResponseSchema,
  ChatSessionsResponseSchema,
  ChatMessageResponseSchema,
  ChatMessagesResponseSchema,
  CreateChatSessionRequestSchema,
  type ChatData,
  type Message,
} from "../schemas/chat";
import { toastError, toastSuccess, toastLoading } from "../utils/toast";
import { parseApiError } from "../errors";
import { apiGet, apiPost } from "../utils/api";

interface ChatState {
  // State
  chats: ChatData[];
  currentChatId: string | null;
  isLoading: boolean;
  error: Error | null;

  // Computed getters
  getCurrentChat: () => ChatData | null;
  getChatById: (chatId: string) => ChatData | undefined;

  // Actions
  setChats: (chats: ChatData[]) => void;
  setCurrentChatId: (chatId: string | null) => void;
  createNewChat: () => string;
  switchToChat: (chatId: string) => void;
  addMessageToCurrentChat: (message: Message) => void;
  updateMessageInCurrentChat: (
    messageId: string,
    updates: Partial<Message>
  ) => void;

  // API actions (with Zod validation)
  fetchChatSessions: () => Promise<void>;
  createChatSession: (request: { title?: string }) => Promise<string>;
  fetchChatMessages: (sessionId: string) => Promise<void>;
  addMessage: (
    sessionId: string,
    message: Omit<Message, "id" | "timestamp">
  ) => Promise<void>;

  // Optimistic updates
  optimisticAddMessage: (message: Message) => void;
  rollbackOptimisticUpdate: (messageId: string) => void;

  // Error handling
  setError: (error: Error | null) => void;
  clearError: () => void;
}

// Random chat title generator (preserved from original)
const chatTitleTemplates = [
  "New application design",
  "API integration help",
  "Database schema planning",
  "UI component brainstorm",
  "Bug fixing session",
  "Feature implementation",
  "Code review discussion",
  "Architecture planning",
  "Performance optimization",
  "Testing strategy",
  "Deployment questions",
  "Security improvements",
  "User experience ideas",
  "Mobile responsive design",
  "Animation concepts",
  "Data visualization",
  "Form validation logic",
  "Authentication setup",
  "State management help",
  "CSS styling questions",
];

function generateChatTitle(): string {
  return chatTitleTemplates[
    Math.floor(Math.random() * chatTitleTemplates.length)
  ];
}

/**
 * Helper to validate and transform API response
 */
function validateApiResponse<T>(
  schema: z.ZodSchema<T>,
  data: unknown,
  context: string
): T {
  try {
    return schema.parse(data);
  } catch (error) {
    if (error instanceof z.ZodError) {
      const issues = (error as z.ZodError).issues;
      console.error(`Validation error in ${context}:`, issues);
      throw new Error(
        `Invalid API response format in ${context}: ${issues
          .map((e) => e.message)
          .join(", ")}`
      );
    }
    throw error;
  }
}

export const useChatStore = create<ChatState>()(
  devtools(
    (set, get) => ({
      // Initial state
      chats: [],
      currentChatId: null,
      isLoading: false,
      error: null,

      // Computed getters
      getCurrentChat: () => {
        const { currentChatId, chats } = get();
        if (!currentChatId) return null;
        return chats.find((chat) => chat.id === currentChatId) ?? null;
      },

      getChatById: (chatId: string) => {
        return get().chats.find((chat) => chat.id === chatId);
      },

      // Basic actions
      setChats: (chats) => set({ chats }),
      setCurrentChatId: (chatId) => set({ currentChatId: chatId }),

      createNewChat: () => {
        const newChatId = `chat-${Date.now()}`;
        const newChat: ChatData = {
          id: newChatId,
          title: generateChatTitle(),
          messages: [],
          createdAt: new Date(),
        };

        set((state) => ({
          chats: [newChat, ...state.chats],
          currentChatId: newChatId,
        }));

        return newChatId;
      },

      switchToChat: (chatId: string) => {
        set({ currentChatId: chatId });
      },

      addMessageToCurrentChat: (message: Message) => {
        const { currentChatId } = get();
        if (!currentChatId) return;

        set((state) => ({
          chats: state.chats.map((chat) =>
            chat.id === currentChatId
              ? { ...chat, messages: [...chat.messages, message] }
              : chat
          ),
        }));
      },

      updateMessageInCurrentChat: (
        messageId: string,
        updates: Partial<Message>
      ) => {
        const { currentChatId } = get();
        if (!currentChatId) return;

        set((state) => ({
          chats: state.chats.map((chat) =>
            chat.id === currentChatId
              ? {
                  ...chat,
                  messages: chat.messages.map((msg) =>
                    msg.id === messageId ? { ...msg, ...updates } : msg
                  ),
                }
              : chat
          ),
        }));
      },

      // API actions with Zod validation
      fetchChatSessions: async () => {
        set({ isLoading: true, error: null });
        const loadingToast = toastLoading("Loading chat sessions...");

        try {
          const apiUrl =
            process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8080";
          const data = await apiGet<unknown>(`${apiUrl}/api/v1/chat/sessions`, {
            retry: { maxAttempts: 3, initialDelay: 1000 },
          });

          const validatedSessions = validateApiResponse(
            ChatSessionsResponseSchema,
            data,
            "fetchChatSessions"
          );

          // Transform API response to ChatData format
          const chats: ChatData[] = validatedSessions.map((session) => ({
            id: session.id,
            title: session.title,
            messages: [],
            createdAt: session.created_at,
            groupId: undefined,
          }));

          set({ chats, isLoading: false });
          loadingToast();
        } catch (error) {
          const appError = parseApiError(error);
          set({ error: appError, isLoading: false });
          loadingToast();
          toastError(error);
          throw appError;
        }
      },

      createChatSession: async (request) => {
        // Validate request
        const validatedRequest = CreateChatSessionRequestSchema.parse(request);

        set({ isLoading: true, error: null });

        try {
          const apiUrl =
            process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8080";
          const data = await apiPost<unknown>(
            `${apiUrl}/api/v1/chat/sessions`,
            validatedRequest
          );

          const validatedSession = validateApiResponse(
            ChatSessionResponseSchema,
            data,
            "createChatSession"
          );

          // Transform to ChatData format
          const newChat: ChatData = {
            id: validatedSession.id,
            title: validatedSession.title,
            messages: [],
            createdAt: validatedSession.created_at,
          };

          set((state) => ({
            chats: [newChat, ...state.chats],
            currentChatId: validatedSession.id,
            isLoading: false,
          }));

          toastSuccess("Chat session created");
          return validatedSession.id;
        } catch (error) {
          const appError = parseApiError(error);
          set({ error: appError, isLoading: false });
          toastError(error);
          throw appError;
        }
      },

      fetchChatMessages: async (sessionId: string) => {
        set({ isLoading: true, error: null });
        const loadingToast = toastLoading("Loading messages...");

        try {
          const apiUrl =
            process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8080";
          const data = await apiGet<unknown>(
            `${apiUrl}/api/v1/chat/sessions/${sessionId}/messages`,
            { retry: { maxAttempts: 3, initialDelay: 1000 } }
          );

          const validatedMessages = validateApiResponse(
            ChatMessagesResponseSchema,
            data,
            "fetchChatMessages"
          );

          // Transform API messages to Message format
          const messages: Message[] = validatedMessages.map((msg) =>
            MessageSchema.parse({
              id: msg.id,
              role: msg.role as "user" | "assistant",
              content: msg.content,
              timestamp: msg.timestamp,
            })
          );

          // Update chat with messages
          set((state) => ({
            chats: state.chats.map((chat) =>
              chat.id === sessionId ? { ...chat, messages } : chat
            ),
            isLoading: false,
          }));
          loadingToast();
        } catch (error) {
          const appError = parseApiError(error);
          set({ error: appError, isLoading: false });
          loadingToast();
          toastError(error);
          throw appError;
        }
      },

      addMessage: async (
        sessionId: string,
        message: Omit<Message, "id" | "timestamp">
      ) => {
        // Optimistic update
        const optimisticMessage: Message = {
          ...message,
          id: `temp-${Date.now()}`,
          timestamp: new Date(),
        };
        get().optimisticAddMessage(optimisticMessage);

        try {
          const apiUrl =
            process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8080";
          const data = await apiPost<unknown>(
            `${apiUrl}/api/v1/chat/sessions/${sessionId}/messages`,
            {
              role: message.role,
              content: message.content,
              contextFiles: message.contextFiles,
            }
          );

          const validatedMessage = validateApiResponse(
            ChatMessageResponseSchema,
            data,
            "addMessage"
          );

          // Replace optimistic message with validated one
          const finalMessage: Message = MessageSchema.parse({
            id: validatedMessage.id,
            role: validatedMessage.role as "user" | "assistant",
            content: validatedMessage.content,
            timestamp: validatedMessage.timestamp,
            isLoading: message.isLoading,
            tasks: message.tasks,
            contextFiles: message.contextFiles,
          });

          set((state) => ({
            chats: state.chats.map((chat) =>
              chat.id === sessionId
                ? {
                    ...chat,
                    messages: chat.messages.map((msg) =>
                      msg.id === optimisticMessage.id ? finalMessage : msg
                    ),
                  }
                : chat
            ),
          }));
        } catch (error) {
          // Rollback optimistic update
          get().rollbackOptimisticUpdate(optimisticMessage.id);
          const appError = parseApiError(error);
          set({ error: appError });
          toastError(error);
          throw appError;
        }
      },

      // Optimistic updates
      optimisticAddMessage: (message: Message) => {
        const { currentChatId } = get();
        if (!currentChatId) return;

        set((state) => ({
          chats: state.chats.map((chat) =>
            chat.id === currentChatId
              ? { ...chat, messages: [...chat.messages, message] }
              : chat
          ),
        }));
      },

      rollbackOptimisticUpdate: (messageId: string) => {
        const { currentChatId } = get();
        if (!currentChatId) return;

        set((state) => ({
          chats: state.chats.map((chat) =>
            chat.id === currentChatId
              ? {
                  ...chat,
                  messages: chat.messages.filter((msg) => msg.id !== messageId),
                }
              : chat
          ),
        }));
      },

      // Error handling
      setError: (error) => set({ error }),
      clearError: () => set({ error: null }),
    }),
    { name: "ChatStore" }
  )
);
