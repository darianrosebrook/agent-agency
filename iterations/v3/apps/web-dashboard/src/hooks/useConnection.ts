"use client";

import { useState, useEffect, useCallback } from "react";

export type ConnectionState = "checking" | "online" | "offline" | "degraded";

export interface ConnectionStatus {
  state: ConnectionState;
  apiAvailable: boolean;
  lastChecked: Date | null;
  retryCount: number;
  error: string;
}

export interface UseConnectionReturn {
  connection: ConnectionStatus;
  checkConnection: () => Promise<void>;
  retryConnection: () => Promise<void>;
  isOnline: boolean;
  isOffline: boolean;
  isChecking: boolean;
}

/**
 * Hook for managing connection status to backend services
 * Provides progressive enhancement - works offline with cached data
 */
export function useConnection(apiUrl: string = "/api/health"): UseConnectionReturn {
  const [connection, setConnection] = useState<ConnectionStatus>({
    state: "checking",
    apiAvailable: false,
    lastChecked: null,
    retryCount: 0,
    error: "",
  });

  const checkConnection = useCallback(async () => {
      setConnection(prev => ({ ...prev, state: "checking", error: "" }));

    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 5000); // 5 second timeout

      const response = await fetch(apiUrl, {
        method: "GET",
        signal: controller.signal,
        headers: {
          "Cache-Control": "no-cache",
        },
      });

      clearTimeout(timeoutId);

      const isAvailable = response.ok;
      setConnection(prev => ({
        state: isAvailable ? "online" : "degraded",
        apiAvailable: isAvailable,
        lastChecked: new Date(),
        retryCount: 0,
        error: isAvailable ? "" : `HTTP ${response.status}`,
      }));

    } catch (error) {
      const isNetworkError = error instanceof TypeError && error.message.includes("fetch");
      const isAbortError = error instanceof Error && error.name === "AbortError";

      setConnection(prev => ({
        state: isAbortError ? "degraded" : "offline",
        apiAvailable: false,
        lastChecked: new Date(),
        retryCount: prev.retryCount + 1,
        error: isNetworkError ? "Network unavailable" :
               isAbortError ? "Request timeout" :
               error instanceof Error ? error.message : "Unknown error",
      }));
    }
  }, [apiUrl]);

  const retryConnection = useCallback(async () => {
    await checkConnection();
  }, [checkConnection]);

  // Auto-check connection on mount and periodically
  useEffect(() => {
    checkConnection();

    // Check every 30 seconds when online, every 10 seconds when offline
    const interval = setInterval(() => {
      checkConnection();
    }, connection.state === "online" ? 30000 : 10000);

    return () => clearInterval(interval);
  }, [checkConnection, connection.state]);

  // Listen for online/offline events
  useEffect(() => {
    const handleOnline = () => checkConnection();
    const handleOffline = () => {
      setConnection(prev => ({ ...prev, state: "offline", apiAvailable: false }));
    };

    window.addEventListener("online", handleOnline);
    window.addEventListener("offline", handleOffline);

    return () => {
      window.removeEventListener("online", handleOnline);
      window.removeEventListener("offline", handleOffline);
    };
  }, [checkConnection]);

  return {
    connection,
    checkConnection,
    retryConnection,
    isOnline: connection.state === "online",
    isOffline: connection.state === "offline",
    isChecking: connection.state === "checking",
  };
}
