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

  // Debounce mechanism to prevent rapid successive calls
  const [lastCheckTime, setLastCheckTime] = useState<number>(0);
  const DEBOUNCE_DELAY = 2000; // 2 seconds minimum between checks

  const checkConnection = useCallback(async () => {
    const now = Date.now();
    
    // Debounce: don't check if we've checked recently
    if (now - lastCheckTime < DEBOUNCE_DELAY) {
      return;
    }
    
    setLastCheckTime(now);
    setConnection(_prev => ({ ..._prev, state: "checking", error: "" }));

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
      setConnection(_prev => ({
        state: isAvailable ? "online" : "degraded",
        apiAvailable: isAvailable,
        lastChecked: new Date(),
        retryCount: 0,
        error: isAvailable ? "" : `HTTP ${response.status}`,
      }));

    } catch (error) {
      const isNetworkError = error instanceof TypeError && error.message.includes("fetch");
      const isAbortError = error instanceof Error && error.name === "AbortError";

      setConnection(_prev => ({
        state: isAbortError ? "degraded" : "offline",
        apiAvailable: false,
        lastChecked: new Date(),
        retryCount: _prev.retryCount + 1,
        error: isNetworkError ? "Network unavailable" :
               isAbortError ? "Request timeout" :
               error instanceof Error ? error.message : "Unknown error",
      }));
    }
  }, [apiUrl, lastCheckTime]);

  const retryConnection = useCallback(async () => {
    await checkConnection();
  }, [checkConnection]);

  // Auto-check connection on mount and periodically with proper debouncing
  useEffect(() => {
    let intervalId: NodeJS.Timeout;
    let timeoutId: NodeJS.Timeout;
    
    // Initial check
    checkConnection();
    
    // Debounced interval setup
    const setupInterval = () => {
      // Clear any existing interval
      if (intervalId) {
        clearInterval(intervalId);
      }
      
      // Set up new interval based on connection state
      const interval = connection.state === "online" ? 30000 : 10000;
      intervalId = setInterval(() => {
        checkConnection();
      }, interval);
    };
    
    // Debounce interval setup to prevent rapid recreation
    timeoutId = setTimeout(setupInterval, 100);
    
    return () => {
      if (intervalId) {
        clearInterval(intervalId);
      }
      if (timeoutId) {
        clearTimeout(timeoutId);
      }
    };
  }, [connection.state, checkConnection]);

  // Listen for online/offline events
  useEffect(() => {
    const handleOnline = () => checkConnection();
    const handleOffline = () => {
      setConnection(_prev => ({ ..._prev, state: "offline", apiAvailable: false }));
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
