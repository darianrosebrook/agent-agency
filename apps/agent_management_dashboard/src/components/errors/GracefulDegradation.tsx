/**
 * Graceful Degradation Component
 * 
 * Wraps components to provide fallback UI when they fail,
 * allowing the rest of the app to continue functioning.
 * 
 * @author @darianrosebrook
 */

"use client";

import React, { type ReactNode } from "react";
import { ScopedErrorBoundary } from "./ScopedErrorBoundary";
import { AlertCircle } from "lucide-react";
import styles from "./GracefulDegradation.module.scss";

interface GracefulDegradationProps {
  children: ReactNode;
  scope: string;
  fallback?: ReactNode;
  showFallback?: boolean; // Whether to show fallback UI or hide component
  level?: "critical" | "non-critical";
}

/**
 * Default fallback UI for graceful degradation
 */
function DefaultFallback({ scope }: { scope: string }) {
  return (
    <div className={styles.gracefulFallback} data-scope={scope}>
      <div className={styles.gracefulFallbackIcon}>
        <AlertCircle className={styles.gracefulFallbackIconSvg} />
      </div>
      <div className={styles.gracefulFallbackText}>
        <p className={styles.gracefulFallbackMessage}>
          This section is temporarily unavailable.
        </p>
        <p className={styles.gracefulFallbackHint}>
          The rest of the application continues to work normally.
        </p>
      </div>
    </div>
  );
}

/**
 * GracefulDegradation wrapper component
 * 
 * Automatically catches errors and shows fallback UI instead of crashing
 */
export function GracefulDegradation({
  children,
  scope,
  fallback,
  showFallback = true,
  level = "non-critical",
}: GracefulDegradationProps) {
  const fallbackUI = fallback || (showFallback ? <DefaultFallback scope={scope} /> : null);

  return (
    <ScopedErrorBoundary
      scope={scope}
      fallback={fallbackUI}
      level={level}
    >
      {children}
    </ScopedErrorBoundary>
  );
}

/**
 * Hook for conditional rendering with graceful degradation
 */
export function useGracefulRender<T>(
  data: T | null | undefined,
  renderFn: (data: T) => ReactNode,
  fallback?: ReactNode
): ReactNode {
  if (data == null) {
    return fallback || null;
  }

  try {
    return renderFn(data);
  } catch (error) {
    console.error("Graceful render failed:", error);
    return fallback || null;
  }
}

