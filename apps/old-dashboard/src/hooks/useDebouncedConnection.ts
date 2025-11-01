/**
 * Debounced Connection Hook
 * Prevents rapid successive health checks with intelligent debouncing
 */

"use client";

import { useState, useEffect, useCallback, useRef } from "react";

export type ConnectionState = "checking" | "online" | "offline" | "degraded";

export interface ConnectionStatus {
  state: ConnectionState;
  apiAvailable: boolean;
  lastChecked: Date | null;
  retryCount: number;
  error: string;
}

export interface UseDebouncedConnectionReturn {
  connection: ConnectionStatus;
  checkConnection: () => Promise<void>;
  retryConnection: () => Promise<void>;
  isOnline: boolean;
  isOffline: boolean;
  isChecking: boolean;
}

/**
 * Debounced connection hook with intelligent rate limiting
 * Prevents rapid successive API calls that can overwhelm the server
 */
export function useDebouncedConnection(apiUrl: string = "/api/health"): UseDebouncedConnectionReturn {
  const [connection, setConnection] = useState<ConnectionStatus>({
    state: "checking",
    apiAvailable: false,
    lastChecked: null,
    retryCount: 0,
    error: "",
  });

  // Debounce and rate limiting
  const lastCheckTime = useRef<number>(0);
  const isCheckingRef = useRef<boolean>(false);
  const intervalRef = useRef<NodeJS.Timeout | null>(null);
  
  // Configurable debounce delays
  const DEBOUNCE_DELAY = 2000; // 2 seconds minimum between checks
  const ONLINE_INTERVAL = 30000; // 30 seconds when online
  const OFFLINE_INTERVAL = 10000; // 10 seconds when offline
  const RETRY_INTERVAL = 5000; // 5 seconds for retries

  const checkConnection = useCallback(async () => {
    const now = Date.now();
    
    // Prevent concurrent checks
    if (isCheckingRef.current) {
      return;
    }
    
    // Debounce: don't check if we've checked recently
    if (now - lastCheckTime.current < DEBOUNCE_DELAY) {
      return;
    }
    
    isCheckingRef.current = true;
    lastCheckTime.current = now;
    
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
      setConnection(() => ({
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
    } finally {
      isCheckingRef.current = false;
    }
  }, [apiUrl]);

  const retryConnection = useCallback(async () => {
    // Force retry by resetting debounce
    lastCheckTime.current = 0;
    await checkConnection();
  }, [checkConnection]);

  // Smart interval management
  useEffect(() => {
    // Clear existing interval
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }

    // Don't set up interval if we're in checking state
    if (connection.state === "checking") {
      return;
    }

    // Determine interval based on connection state
    let interval: number;
    if (connection.state === "online") {
      interval = ONLINE_INTERVAL;
    } else if (connection.state === "offline") {
      interval = OFFLINE_INTERVAL;
    } else {
      interval = RETRY_INTERVAL; // For degraded state
    }

    // Set up new interval
    intervalRef.current = setInterval(() => {
      checkConnection();
    }, interval);

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
    };
  }, [connection.state, checkConnection]);

  // Initial check on mount
  useEffect(() => {
    checkConnection();
  }, [checkConnection]);

  // Listen for online/offline events
  useEffect(() => {
    const handleOnline = () => {
      // Reset debounce on network recovery
      lastCheckTime.current = 0;
      checkConnection();
    };
    
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

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, []);

  return {
    connection,
    checkConnection,
    retryConnection,
    isOnline: connection.state === "online",
    isOffline: connection.state === "offline",
    isChecking: connection.state === "checking",
  };
}
