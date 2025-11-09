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

import { useEffect, useRef, useState, useCallback } from 'react';

export interface WebSocketOptions {
  url: string;
  token?: string;
  reconnect?: boolean;
  reconnectDelay?: number;
  reconnectDelayMax?: number;
  onMessage?: (data: any) => void;
  onError?: (error: Event) => void;
  onOpen?: () => void;
  onClose?: () => void;
}

export interface WebSocketState {
  connected: boolean;
  connecting: boolean;
  error: Error | null;
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
    onMessage,
    onError,
    onOpen,
    onClose,
  } = options;

  const [state, setState] = useState<WebSocketState>({
    connected: false,
    connecting: false,
    error: null,
  });

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttemptsRef = useRef(0);
  const shouldReconnectRef = useRef(reconnect);

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      return;
    }

    setState((prev) => ({ ...prev, connecting: true, error: null }));

    try {
      const wsUrl = new URL(url);
      if (token) {
        wsUrl.searchParams.set('token', token);
      }

      const ws = new WebSocket(wsUrl.toString());

      ws.onopen = () => {
        setState({
          connected: true,
          connecting: false,
          error: null,
        });
        reconnectAttemptsRef.current = 0;
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
          error: new Error('WebSocket error'),
        }));
        onError?.(error);
      };

      ws.onclose = () => {
        setState((prev) => ({
          ...prev,
          connected: false,
          connecting: false,
        }));
        onClose?.();

        // Attempt reconnection if enabled
        if (shouldReconnectRef.current) {
          const delay = Math.min(
            reconnectDelay * Math.pow(2, reconnectAttemptsRef.current),
            reconnectDelayMax
          );
          reconnectAttemptsRef.current += 1;

          reconnectTimeoutRef.current = setTimeout(() => {
            connect();
          }, delay);
        }
      };

      wsRef.current = ws;
    } catch (error) {
      setState({
        connected: false,
        connecting: false,
        error: error instanceof Error ? error : new Error('Failed to create WebSocket'),
      });
    }
  }, [url, token, reconnectDelay, reconnectDelayMax, onMessage, onError, onOpen, onClose]);

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
      const message = typeof data === 'string' ? data : JSON.stringify(data);
      wsRef.current.send(message);
    } else {
      console.warn('WebSocket is not connected. Message not sent:', data);
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

