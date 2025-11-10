/**
 * WebSocket hook for real-time communication
 *
 * Provides a React hook for managing WebSocket connections with automatic
 * reconnection and channel-based message routing.
 *
 * Adapted from open-webui patterns for agent-agency.
 *
 * @author @darianrosebrook
 */

import { useEffect, useRef, useState, useCallback } from "react";

export type Transport = "websocket" | "polling" | "auto";

export interface WebSocketOptions {
  url: string;
  token?: string;
  reconnect?: boolean;
  reconnectDelay?: number;
  reconnectDelayMax?: number;
  randomizationFactor?: number; // Randomization factor for reconnection delay (0-1)
  transport?: Transport; // Transport preference: 'websocket', 'polling', or 'auto' (default)
  onMessage?: (data: any) => void;
  onError?: (error: Event) => void;
  onOpen?: () => void;
  onClose?: () => void;
  onTransportChange?: (transport: Transport) => void; // Called when transport changes
}

export interface WebSocketState {
  connected: boolean;
  connecting: boolean;
  error: Error | null;
  transport: Transport; // Current transport being used
  reconnectAttempts: number; // Number of reconnection attempts
}

/**
 * Hook for managing WebSocket connections
 *
 * @example
 * ```tsx
 * const { send, state } = useWebSocket({
 *   url: 'ws://localhost:3000/ws',
 *   token: 'your-token',
 *   onMessage: (data) => console.log('Received:', data),
 * });
 *
 * // Send a message
 * send({ type: 'subscribe', channel: 'agent:123' });
 * ```
 */
export function useWebSocket(options: WebSocketOptions) {
  const {
    url,
    token,
    reconnect = true,
    reconnectDelay = 1000,
    reconnectDelayMax = 5000,
    randomizationFactor = 0.5, // Default randomization factor matching open-webui
    transport: preferredTransport = "auto",
    onMessage,
    onError,
    onOpen,
    onClose,
    onTransportChange,
  } = options;

  const [state, setState] = useState<WebSocketState>({
    connected: false,
    connecting: false,
    error: null,
    transport: preferredTransport === "auto" ? "websocket" : preferredTransport,
    reconnectAttempts: 0,
  });

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttemptsRef = useRef(0);
  const shouldReconnectRef = useRef(reconnect);
  const currentTransportRef = useRef<Transport>(
    preferredTransport === "auto" ? "websocket" : preferredTransport
  );
  const failedTransportsRef = useRef<Set<Transport>>(new Set());

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      return;
    }

    setState((prev) => ({ ...prev, connecting: true, error: null }));

    try {
      const wsUrl = new URL(url);
      if (token) {
        wsUrl.searchParams.set("token", token);
      }

      const ws = new WebSocket(wsUrl.toString());

      ws.onopen = () => {
        setState({
          connected: true,
          connecting: false,
          error: null,
          transport: currentTransportRef.current,
          reconnectAttempts: 0,
        });
        reconnectAttemptsRef.current = 0;
        failedTransportsRef.current.clear(); // Reset failed transports on successful connection
        onOpen?.();
      };

      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          onMessage?.(data);
        } catch (e) {
          // If not JSON, pass raw data
          onMessage?.(event.data);
        }
      };

      ws.onerror = (error) => {
        setState((prev) => ({
          ...prev,
          error: new Error("WebSocket error"),
          reconnectAttempts: reconnectAttemptsRef.current,
        }));
        onError?.(error);

        // If WebSocket fails and we're in auto mode, mark for fallback
        if (
          preferredTransport === "auto" &&
          currentTransportRef.current === "websocket"
        ) {
          // Will fallback to polling on close if multiple failures occur
        }
      };

      ws.onclose = (event) => {
        setState((prev) => ({
          ...prev,
          connected: false,
          connecting: false,
          reconnectAttempts: reconnectAttemptsRef.current,
        }));
        onClose?.();

        // If WebSocket failed and we're in auto mode, try polling fallback
        if (
          preferredTransport === "auto" &&
          currentTransportRef.current === "websocket"
        ) {
          // Mark WebSocket as failed after multiple attempts
          if (reconnectAttemptsRef.current >= 3) {
            failedTransportsRef.current.add("websocket");

            // Fall back to polling (SSE-based)
            if (!failedTransportsRef.current.has("polling")) {
              currentTransportRef.current = "polling";
              setState((prev) => ({ ...prev, transport: "polling" }));
              onTransportChange?.("polling");

              // Try connecting with polling
              setTimeout(() => {
                connect();
              }, reconnectDelay);
              return;
            }
          }
        }

        // Attempt reconnection if enabled
        if (shouldReconnectRef.current) {
          // Calculate delay with randomization factor (matching open-webui pattern)
          const baseDelay = Math.min(
            reconnectDelay * Math.pow(2, reconnectAttemptsRef.current),
            reconnectDelayMax
          );
          const randomOffset =
            baseDelay * randomizationFactor * (Math.random() - 0.5);
          const delay = Math.max(0, baseDelay + randomOffset);

          reconnectAttemptsRef.current += 1;

          reconnectTimeoutRef.current = setTimeout(() => {
            connect();
          }, delay);
        }
      };

      wsRef.current = ws;
    } catch (error) {
      setState((prev) => ({
        ...prev,
        connected: false,
        connecting: false,
        error:
          error instanceof Error
            ? error
            : new Error("Failed to create WebSocket"),
      }));
    }
  }, [
    url,
    token,
    reconnectDelay,
    reconnectDelayMax,
    onMessage,
    onError,
    onOpen,
    onClose,
  ]);

  const disconnect = useCallback(() => {
    shouldReconnectRef.current = false;
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }
  }, []);

  const send = useCallback((data: any) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      const message = typeof data === "string" ? data : JSON.stringify(data);
      wsRef.current.send(message);
    } else {
      console.warn("WebSocket is not connected. Message not sent:", data);
    }
  }, []);

  useEffect(() => {
    connect();
    return () => {
      disconnect();
    };
  }, [connect, disconnect]);

  return {
    send,
    state,
    connect,
    disconnect,
  };
}
