"use client";

import {
  ReactNode,
  createContext,
  useCallback,
  useContext,
  useEffect,
  useState,
} from "react";
import {
  createChatSession,
  getChatMessages,
  getChatSessions,
  sendChatMessage,
  type ChatMessageResponse,
  type ChatSessionResponse,
} from "../lib/api/chat";
import { getCurrentUser } from "../lib/api/users";
import type { Message } from "../lib/schemas/chat";

interface ChatData {
  id: string;
  title: string;
  messages: Message[];
  createdAt: Date;
  groupId?: string;
}

interface ChatContextType {
  chats: ChatData[];
  currentChatId: string | null;
  isLoading: boolean;
  error: string | null;
  getCurrentChat: () => ChatData | null;
  createNewChat: () => Promise<string>;
  switchToChat: (chatId: string) => void;
  addMessageToCurrentChat: (message: Message) => Promise<void>;
  updateMessageInCurrentChat: (
    messageId: string,
    updates: Partial<Message>
  ) => void;
}

const ChatContext = createContext<ChatContextType | undefined>(undefined);

// Random chat title generator
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

export function ChatProvider({ children }: { children: ReactNode }) {
  const [chats, setChats] = useState<ChatData[]>([]);
  const [currentChatId, setCurrentChatId] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Load chat sessions from database on mount
  useEffect(() => {
    loadChatSessions();
  }, []);

  // Load messages for current chat when it changes
  useEffect(() => {
    if (currentChatId) {
      loadChatMessages(currentChatId);
    }
  }, [currentChatId]);

  const loadChatSessions = useCallback(async () => {
    try {
      setIsLoading(true);
      setError(null);

      // Get workspace_id from current user (if available)
      // Note: Current user API may not include workspace_id yet - backend will extract from auth token if undefined
      let workspaceId: string | undefined;
      try {
        const currentUser = await getCurrentUser();
        // Try to extract workspace_id from user preferences or metadata
        workspaceId = (currentUser.preferences?.workspace_id as string) || undefined;
      } catch (err) {
        // If getCurrentUser fails, proceed with undefined (backend will extract from auth token)
        console.warn("Failed to fetch current user for workspace_id:", err);
      }

      // Fetch chat sessions from API
      // Backend will extract workspace_id from auth token if not provided
      const sessions = await getChatSessions(workspaceId, {
        archived: false,
        limit: 100,
        offset: 0,
      });

      // Map API responses to ChatData format
      const chatData: ChatData[] = await Promise.all(
        sessions.map(async (session: ChatSessionResponse) => {
          // Load messages for each session (limit to recent messages for performance)
          const messages = await getChatMessages(session.id, {
            limit: 50,
            offset: 0,
          }).catch(() => []); // If messages fail to load, continue with empty array

          return {
            id: session.id,
            title: session.title || "Untitled Chat",
            messages: messages.map(mapMessageResponse),
            createdAt: new Date(session.created_at),
            groupId: session.workspace_id,
          };
        })
      );

      // Sort by last activity (most recent first)
      chatData.sort((a, b) => {
        const aLastMessage = a.messages[a.messages.length - 1]?.timestamp;
        const bLastMessage = b.messages[b.messages.length - 1]?.timestamp;
        if (aLastMessage && bLastMessage) {
          return bLastMessage.getTime() - aLastMessage.getTime();
        }
        return b.createdAt.getTime() - a.createdAt.getTime();
      });

      setChats(chatData);
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to load chat sessions";
      setError(errorMessage);
      console.error("Failed to load chat sessions:", err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const loadChatMessages = useCallback(async (sessionId: string) => {
    try {
      const messages = await getChatMessages(sessionId, {
        limit: 100,
        offset: 0,
      });

      // Update messages for this chat session
      setChats((prev) =>
        prev.map((chat) => {
          if (chat.id === sessionId) {
            return {
              ...chat,
              messages: messages.map(mapMessageResponse),
            };
          }
          return chat;
        })
      );
    } catch (err) {
      console.error(`Failed to load messages for session ${sessionId}:`, err);
    }
  }, []);

  // Map API message response to UI Message format
  const mapMessageResponse = (msg: ChatMessageResponse): Message => {
    return {
      id: msg.id,
      role: msg.role === "user" || msg.role === "assistant" ? msg.role : "user",
      content: msg.content,
      timestamp: new Date(msg.created_at),
      isLoading: false,
    };
  };

  const getCurrentChat = () => {
    if (!currentChatId) return null;
    return chats.find((chat) => chat.id === currentChatId) ?? null;
  };

  const createNewChat = useCallback(async (): Promise<string> => {
    try {
      const title = generateChatTitle();

      // Get workspace_id from current user (if available)
      // Note: Current user API may not include workspace_id yet - backend will extract from auth token if undefined
      let workspaceId: string | undefined;
      try {
        const currentUser = await getCurrentUser();
        // Try to extract workspace_id from user preferences or metadata
        workspaceId = (currentUser.preferences?.workspace_id as string) || undefined;
      } catch (err) {
        // If getCurrentUser fails, proceed with undefined (backend will extract from auth token)
        console.warn("Failed to fetch current user for workspace_id:", err);
      }

      // Create chat session in database via API
      // Backend will extract workspace_id from auth token if not provided
      const session = await createChatSession(
        { title },
        workspaceId
      );

      // Create local ChatData from API response
      const newChat: ChatData = {
        id: session.id,
        title: session.title || title,
        messages: [],
        createdAt: new Date(session.created_at),
        groupId: session.workspace_id,
      };

      // Add to local state
      setChats((prev) => [newChat, ...prev]);
      setCurrentChatId(session.id);

      return session.id;
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to create chat session";
      setError(errorMessage);
      console.error("Failed to create chat session:", err);

      // Fallback: create local-only chat if API fails
      const fallbackId = `chat-${Date.now()}`;
      const fallbackChat: ChatData = {
        id: fallbackId,
        title: generateChatTitle(),
        messages: [],
        createdAt: new Date(),
      };

      setChats((prev) => [fallbackChat, ...prev]);
      setCurrentChatId(fallbackId);

      return fallbackId;
    }
  }, []);

  const switchToChat = (chatId: string) => {
    setCurrentChatId(chatId);
  };

  const addMessageToCurrentChat = useCallback(
    async (message: Message) => {
      if (!currentChatId) return;

      try {
        // Send message to API and get response
        const apiMessage = await sendChatMessage(
          currentChatId,
          message.content,
          message.role,
          message.metadata as Record<string, unknown> | undefined
        );

        // Map API response to UI Message format
        const mappedMessage = mapMessageResponse(apiMessage);

        // Update local state with both user message and API response
        setChats((prev) =>
          prev.map((chat) => {
            if (chat.id === currentChatId) {
              // Add user message if it's not already in the list
              const hasUserMessage = chat.messages.some(
                (m) => m.id === message.id
              );
              const messagesToAdd = hasUserMessage
                ? [mappedMessage]
                : [message, mappedMessage];

              return {
                ...chat,
                messages: [...chat.messages, ...messagesToAdd],
              };
            }
            return chat;
          })
        );
      } catch (err) {
        console.error("Failed to send message:", err);

        // Fallback: add message to local state only
        setChats((prev) =>
          prev.map((chat) => {
            if (chat.id === currentChatId) {
              return {
                ...chat,
                messages: [...chat.messages, message],
              };
            }
            return chat;
          })
        );
      }
    },
    [currentChatId]
  );

  const updateMessageInCurrentChat = (
    messageId: string,
    updates: Partial<Message>
  ) => {
    if (!currentChatId) return;

    setChats((prev) =>
      prev.map((chat) => {
        if (chat.id === currentChatId) {
          return {
            ...chat,
            messages: chat.messages.map((msg) =>
              msg.id === messageId ? { ...msg, ...updates } : msg
            ),
          };
        }
        return chat;
      })
    );
  };

  return (
    <ChatContext.Provider
      value={{
        chats,
        currentChatId,
        isLoading,
        error,
        getCurrentChat,
        createNewChat,
        switchToChat,
        addMessageToCurrentChat,
        updateMessageInCurrentChat,
      }}
    >
      {children}
    </ChatContext.Provider>
  );
}

export function useChatContext() {
  const context = useContext(ChatContext);
  if (context === undefined) {
    throw new Error("useChatContext must be used within a ChatProvider");
  }
  return context;
}
