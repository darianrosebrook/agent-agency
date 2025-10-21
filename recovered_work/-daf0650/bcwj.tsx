"use client";

import React, { useState, useEffect, useCallback, useRef } from "react";

// Declare window for ESLint - WebSocket support
// eslint-disable-next-line @typescript-eslint/no-explicit-any
declare const window: any;
import {
  ChatInterfaceProps,
  ChatSession,
  ChatMessage,
  ChatContext,
  ConnectionState,
  ChatMessagePayload,
} from "@/types/chat";
import { ChatApiClient, ChatApiError } from "@/lib/chat-api";
import { WebSocketClient } from "@/lib/websocket/WebSocketClient";
import MessageList from "./MessageList";
import MessageInput from "./MessageInput";
import ContextPanel from "./ContextPanel";
import ConnectionStatus from "./ConnectionStatus";
import styles from "./ChatInterface.module.scss";

interface ChatInterfaceState {
  session: ChatSession | null;
  messages: ChatMessage[];
  context: ChatContext;
  isLoading: boolean;
  error: string | null;
  connectionState: ConnectionState;
}

export default function ChatInterface({
  sessionId,
  initialContext,
  onSessionCreate,
  onSessionUpdate,
  onError,
}: ChatInterfaceProps) {
  const [state, setState] = useState<ChatInterfaceState>({
    session: null,
    messages: [],
    context: {},
    isLoading: true,
    error: null,
    connectionState: "disconnected",
  });

  const chatApiRef = useRef<ChatApiClient>(new ChatApiClient());
  const wsClientRef = useRef<WebSocketClient | null>(null);

  // Initialize session
  useEffect(() => {
    if (sessionId) {
      loadExistingSession(sessionId);
    } else {
      createNewSession();
    }
  }, [sessionId]);

  const createNewSession = useCallback(async () => {
    try {
      setState((prev) => ({ ...prev, isLoading: true, error: null }));

      const response = await chatApiRef.current.createSession(
        initialContext ? { initial_context: initialContext } : {}
      );

      const newSession = response.session;

      // Initialize WebSocket connection
      initializeWebSocket(newSession.id, response.websocket_url);

      setState((prev) => ({
        ...prev,
        session: newSession,
        context: newSession.context ?? {},
        isLoading: false,
      }));

      onSessionCreate?.(newSession);
    } catch (error) {
      const errorMessage =
        error instanceof ChatApiError
          ? error.message
          : "Failed to create chat session";

      setState((prev) => ({
        ...prev,
        isLoading: false,
        error: errorMessage,
      }));

      onError?.(error instanceof Error ? error : new Error(errorMessage));
    }
  }, [initialContext, onSessionCreate, onError]);

  const loadExistingSession = useCallback(
    async (id: string) => {
      try {
        setState((prev) => ({ ...prev, isLoading: true, error: null }));

        const response = await chatApiRef.current.getSession(id);
        const session = response.session;

        // Initialize WebSocket connection
        const wsUrl =
          typeof window !== "undefined"
            ? // eslint-disable-next-line no-undef
              `${window.location.protocol === "https:" ? "wss:" : "ws:"}//${
                window.location.host
              }/api/proxy/chat/ws/${id}`
            : `ws://localhost:8080/chat/ws/${id}`;
        initializeWebSocket(session.id, wsUrl);

        setState((prev) => ({
          ...prev,
          session,
          messages: response.messages,
          context: response.context,
          isLoading: false,
        }));
      } catch (error) {
        const errorMessage =
          error instanceof ChatApiError
            ? error.message
            : "Failed to load chat session";

        setState((prev) => ({
          ...prev,
          isLoading: false,
          error: errorMessage,
        }));

        onError?.(error instanceof Error ? error : new Error(errorMessage));
      }
    },
    [onError]
  );

  const initializeWebSocket = useCallback(
    (sessionId: string, wsUrl: string) => {
      // Clean up existing connection
      if (wsClientRef.current) {
        wsClientRef.current.destroy();
      }

      wsClientRef.current = new WebSocketClient({
        url: wsUrl,
        sessionId,
        reconnectInterval: 1000,
        maxReconnectAttempts: 5,
        heartbeatInterval: 30000,
        onMessage: handleWebSocketMessage,
        onStateChange: (connectionState) => {
          setState((prev) => ({ ...prev, connectionState }));
        },
        onError: (error) => {
          console.error("WebSocket error:", error);
          onError?.(error);
        },
      });
    },
    [onError]
  );

  const handleWebSocketMessage = useCallback(
    (message: any) => {
      console.log("Received WebSocket message:", message);

      switch (message.type) {
        case "message":
          // Add new message to the list
          setState((prev) => ({
            ...prev,
            messages: [...prev.messages, message.data],
          }));
          break;

        case "session_update":
          // Update session state
          setState((prev) => ({
            ...prev,
            session: message.data,
            context: message.data.context || prev.context,
          }));
          onSessionUpdate?.(message.data);
          break;

        case "typing":
          // Handle typing indicators if needed
          break;

        case "error":
          setState((prev) => ({
            ...prev,
            error: message.data.message,
          }));
          onError?.(new Error(message.data.message));
          break;

        default:
          console.warn("Unknown WebSocket message type:", message.type);
      }
    },
    [onSessionUpdate, onError]
  );

  const handleSendMessage = useCallback(
    async (payload: ChatMessagePayload) => {
      if (!state.session || !wsClientRef.current) {
        return;
      }

      try {
        // Send via WebSocket for real-time experience
        wsClientRef.current.sendMessage(
          payload.content,
          payload.context,
          payload.intent_hint
        );

        // Optimistically add user message to UI
        const userMessage: ChatMessage = {
          id: `temp-${Date.now()}`,
          session_id: state.session.id,
          role: "user",
          content: payload.content,
          timestamp: new Date().toISOString(),
        };

        setState((prev) => ({
          ...prev,
          messages: [...prev.messages, userMessage],
        }));
      } catch (error) {
        console.error("Failed to send message:", error);
        onError?.(
          error instanceof Error ? error : new Error("Failed to send message")
        );
      }
    },
    [state.session, onError]
  );

  const handleContextUpdate = useCallback(
    async (updates: Partial<ChatContext>) => {
      if (!state.session) return;

      try {
        const updatedSession = await chatApiRef.current.updateSessionContext(
          state.session.id,
          updates
        );

        setState((prev) => ({
          ...prev,
          session: updatedSession,
          context: { ...prev.context, ...updates },
        }));
      } catch (error) {
        console.error("Failed to update context:", error);
        onError?.(
          error instanceof Error ? error : new Error("Failed to update context")
        );
      }
    },
    [state.session, onError]
  );

  const handleRetryConnection = useCallback(() => {
    if (wsClientRef.current) {
      wsClientRef.current.reconnect();
    }
  }, []);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (wsClientRef.current) {
        wsClientRef.current.destroy();
      }
    };
  }, []);

  if (state.isLoading) {
    return (
      <div className={styles.chatInterface}>
        <div className={styles.loading}>
          <div className={styles.spinner}></div>
          <p>Initializing chat session...</p>
        </div>
      </div>
    );
  }

  if (state.error && !state.session) {
    return (
      <div className={styles.chatInterface}>
        <div className={styles.error}>
          <h3>Failed to initialize chat</h3>
          <p>{state.error}</p>
          <button
            onClick={
              sessionId
                ? () => loadExistingSession(sessionId)
                : createNewSession
            }
          >
            Retry
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.chatInterface}>
      <div className={styles.header}>
        <h2 className={styles.title}>
          {state.session?.title ??
            `Chat Session ${state.session?.id.slice(-8)}`}
        </h2>
        <ConnectionStatus
          state={state.connectionState}
          onRetry={handleRetryConnection}
        />
      </div>

      <div className={styles.chatContainer}>
        <div className={styles.messagesArea}>
          <MessageList messages={state.messages} isLoading={state.isLoading} />
          <MessageInput
            onSendMessage={handleSendMessage}
            disabled={state.connectionState !== "connected"}
            placeholder={
              state.connectionState === "connected"
                ? "Ask me about tasks, code, or system status..."
                : "Waiting for connection..."
            }
          />
        </div>

        <div className={styles.contextArea}>
          <ContextPanel
            context={state.context}
            session={state.session!}
            onContextUpdate={handleContextUpdate}
          />
        </div>
      </div>

      {state.error && (
        <div className={styles.errorBanner}>
          <p>{state.error}</p>
          <button
            onClick={() => setState((prev) => ({ ...prev, error: null }))}
          >
            Dismiss
          </button>
        </div>
      )}
    </div>
  );
}
