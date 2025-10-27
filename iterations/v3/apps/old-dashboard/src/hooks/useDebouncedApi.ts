/**
 * Debounced API Hook
 * Provides debounced API calls to prevent rapid successive requests
 */

"use client";

import { useCallback, useRef } from "react";

interface DebouncedApiOptions {
  debounceDelay?: number;
  maxRetries?: number;
  retryDelay?: number;
}

/**
 * Hook for making debounced API calls
 * Prevents rapid successive requests that can overwhelm the server
 */
export function useDebouncedApi(options: DebouncedApiOptions = {}) {
  const {
    debounceDelay = 1000, // 1 second default debounce
  } = options;

  const lastCallTime = useRef<number>(0);
  const pendingCalls = useRef<Map<string, AbortController>>(new Map());

  const debouncedFetch = useCallback(async (
    url: string,
    options: RequestInit = {},
    key?: string
  ): Promise<Response> => {
    const now = Date.now();
    const callKey = key || url;
    
    // Cancel any pending request with the same key
    if (pendingCalls.current.has(callKey)) {
      pendingCalls.current.get(callKey)?.abort();
    }

    // Debounce: don't make request if we've made one recently
    if (now - lastCallTime.current < debounceDelay) {
      return new Promise((resolve, reject) => {
        setTimeout(() => {
          debouncedFetch(url, options, key).then(resolve).catch(reject);
        }, debounceDelay - (now - lastCallTime.current));
      });
    }

    lastCallTime.current = now;

    // Create abort controller for this request
    const controller = new AbortController();
    pendingCalls.current.set(callKey, controller);

    try {
      const response = await fetch(url, {
        ...options,
        signal: controller.signal,
      });

      // Remove from pending calls on success
      pendingCalls.current.delete(callKey);
      return response;
    } catch (error) {
      // Remove from pending calls on error
      pendingCalls.current.delete(callKey);
      throw error;
    }
  }, [debounceDelay]);

  const debouncedApiCall = useCallback(async <T>(
    apiCall: () => Promise<T>,
    key?: string
  ): Promise<T> => {
    const now = Date.now();
    
    // Debounce: don't make request if we've made one recently
    if (now - lastCallTime.current < debounceDelay) {
      return new Promise((resolve, reject) => {
        setTimeout(() => {
          debouncedApiCall(apiCall, key).then(resolve).catch(reject);
        }, debounceDelay - (now - lastCallTime.current));
      });
    }

    lastCallTime.current = now;
    return apiCall();
  }, [debounceDelay]);

  const cancelPendingCalls = useCallback(() => {
    pendingCalls.current.forEach(controller => controller.abort());
    pendingCalls.current.clear();
  }, []);

  return {
    debouncedFetch,
    debouncedApiCall,
    cancelPendingCalls,
  };
}
