/**
 * Error Recovery Hook
 * 
 * Provides automatic error recovery with exponential backoff
 * and circuit breaker patterns for resilient data fetching.
 * 
 * @author @darianrosebrook
 */

import { useState, useEffect, useCallback, useRef } from "react";
import { safeRetry, CircuitBreaker, withTimeout } from "../lib/utils/errorGuards";
import { ErrorCode, AppError } from "../lib/errors/types";

interface UseErrorRecoveryOptions {
  maxRetries?: number;
  retryDelay?: number;
  timeout?: number;
  enableCircuitBreaker?: boolean;
  onError?: (error: Error) => void;
  onRecovery?: () => void;
}

interface ErrorRecoveryState {
  error: Error | null;
  isRecovering: boolean;
  retryCount: number;
  lastErrorTime: number | null;
}

/**
 * Hook for automatic error recovery with retry logic
 */
export function useErrorRecovery(options: UseErrorRecoveryOptions = {}) {
  const {
    maxRetries = 3,
    retryDelay = 1000,
    timeout,
    enableCircuitBreaker = true,
    onError,
    onRecovery,
  } = options;

  const [state, setState] = useState<ErrorRecoveryState>({
    error: null,
    isRecovering: false,
    retryCount: 0,
    lastErrorTime: null,
  });

  const circuitBreakerRef = useRef<CircuitBreaker | null>(
    enableCircuitBreaker ? new CircuitBreaker() : null
  );

  const executeWithRecovery = useCallback(
    async <T>(fn: () => Promise<T>): Promise<T | null> => {
      setState((prev) => ({ ...prev, isRecovering: true, error: null }));

      try {
        let operation = fn();

        // Add timeout if specified
        if (timeout) {
          operation = withTimeout(operation, timeout);
        }

        // Use circuit breaker if enabled
        if (circuitBreakerRef.current) {
          operation = circuitBreakerRef.current.execute(() => operation);
        }

        // Execute with retry logic
        const result = await safeRetry(
          () => operation,
          {
            maxRetries,
            delay: retryDelay,
            backoff: true,
            onRetry: (attempt, error) => {
              setState((prev) => ({
                ...prev,
                retryCount: attempt,
                error: error,
                lastErrorTime: Date.now(),
              }));
              console.warn(`[ErrorRecovery] Retry attempt ${attempt}:`, error);
            },
            shouldRetry: (error) => {
              // Don't retry on client errors (4xx)
              if (error instanceof AppError) {
                return [
                  ErrorCode.NETWORK_ERROR,
                  ErrorCode.TIMEOUT,
                  ErrorCode.CONNECTION_FAILED,
                  ErrorCode.SERVER_ERROR,
                ].includes(error.code);
              }
              return true;
            },
          }
        );

        setState({
          error: null,
          isRecovering: false,
          retryCount: 0,
          lastErrorTime: null,
        });

        if (state.retryCount > 0) {
          onRecovery?.();
        }

        return result;
      } catch (error) {
        const appError =
          error instanceof Error
            ? error
            : new AppError(ErrorCode.OPERATION_FAILED, String(error));

        setState({
          error: appError,
          isRecovering: false,
          retryCount: state.retryCount + 1,
          lastErrorTime: Date.now(),
        });

        onError?.(appError);

        return null;
      }
    },
    [maxRetries, retryDelay, timeout, onError, onRecovery, state.retryCount]
  );

  const reset = useCallback(() => {
    circuitBreakerRef.current?.reset();
    setState({
      error: null,
      isRecovering: false,
      retryCount: 0,
      lastErrorTime: null,
    });
  }, []);

  return {
    executeWithRecovery,
    reset,
    error: state.error,
    isRecovering: state.isRecovering,
    retryCount: state.retryCount,
    lastErrorTime: state.lastErrorTime,
  };
}

