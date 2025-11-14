/**
 * Resilient Component Wrapper
 * 
 * Example implementation showing how to use error resilience patterns
 * in dashboard components.
 * 
 * @author @darianrosebrook
 */

"use client";

import React, { type ReactNode } from "react";
import { ErrorIsolation } from "./ErrorIsolation";
import { useErrorRecovery } from "../../hooks/useErrorRecovery";
import { safeAsync } from "../../lib/utils/errorGuards";

interface ResilientComponentProps<T> {
  children: (data: T) => ReactNode;
  dataFetcher: () => Promise<T>;
  scope: string;
  fallback?: ReactNode;
  loadingFallback?: ReactNode;
  emptyFallback?: ReactNode;
}

/**
 * Example: Resilient component with automatic error recovery
 */
export function ResilientComponent<T>({
  children,
  dataFetcher,
  scope,
  fallback,
  loadingFallback = <div>Loading...</div>,
  emptyFallback = <div>No data available</div>,
}: ResilientComponentProps<T>) {
  const { executeWithRecovery, error, isRecovering } = useErrorRecovery({
    maxRetries: 3,
    retryDelay: 1000,
    onError: (err) => {
      console.error(`[${scope}] Data fetch failed:`, err);
    },
  });

  const [data, setData] = React.useState<T | null>(null);
  const [isLoading, setIsLoading] = React.useState(true);

  React.useEffect(() => {
    let mounted = true;

    async function fetchData() {
      setIsLoading(true);
      const result = await safeAsync(
        () => executeWithRecovery(dataFetcher),
        null,
        scope
      );

      if (mounted) {
        setData(result);
        setIsLoading(false);
      }
    }

    fetchData();

    return () => {
      mounted = false;
    };
  }, [dataFetcher, executeWithRecovery, scope]);

  if (isLoading || isRecovering) {
    return <>{loadingFallback}</>;
  }

  if (error) {
    return <>{fallback}</>;
  }

  if (data == null) {
    return <>{emptyFallback}</>;
  }

  return (
    <ErrorIsolation scope={scope} fallback={fallback}>
      {children(data)}
    </ErrorIsolation>
  );
}

