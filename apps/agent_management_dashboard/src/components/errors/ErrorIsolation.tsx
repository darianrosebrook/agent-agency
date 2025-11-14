/**
 * Error Isolation Component
 * 
 * Isolates errors to prevent them from propagating to parent components.
 * Useful for non-critical features that shouldn't crash the app.
 * 
 * @author @darianrosebrook
 */

"use client";

import React, { type ReactNode, useState, useCallback } from "react";
import { ScopedErrorBoundary } from "./ScopedErrorBoundary";
import { GracefulDegradation } from "./GracefulDegradation";

interface ErrorIsolationProps {
  children: ReactNode;
  scope: string;
  fallback?: ReactNode;
  isolate?: boolean; // If true, errors won't propagate
  onError?: (error: Error) => void;
}

/**
 * ErrorIsolation component
 * 
 * Wraps children in error boundaries to prevent error propagation
 */
export function ErrorIsolation({
  children,
  scope,
  fallback,
  isolate = true,
  onError,
}: ErrorIsolationProps) {
  const [error, setError] = useState<Error | null>(null);

  const handleError = useCallback(
    (error: Error, _errorInfo: React.ErrorInfo, _scope: string) => {
      setError(error);
      onError?.(error);

      if (!isolate) {
        // Re-throw to propagate to parent boundary
        throw error;
      }
    },
    [isolate, onError]
  );

  if (error && !isolate) {
    throw error;
  }

  return (
    <ScopedErrorBoundary
      scope={scope}
      fallback={fallback}
      onError={handleError}
      level="non-critical"
    >
      <GracefulDegradation scope={scope} fallback={fallback}>
        {children}
      </GracefulDegradation>
    </ScopedErrorBoundary>
  );
}

/**
 * Hook for isolated error handling
 */
export function useIsolatedError(scope: string) {
  const [error, setError] = useState<Error | null>(null);

  const handleError = useCallback((error: Error) => {
    console.error(`[${scope}] Isolated error:`, error);
    setError(error);
  }, [scope]);

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  return {
    error,
    handleError,
    clearError,
    hasError: error !== null,
  };
}

