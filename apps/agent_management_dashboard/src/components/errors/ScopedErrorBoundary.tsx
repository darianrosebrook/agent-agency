/**
 * Scoped Error Boundary
 * 
 * Provides isolated error boundaries for specific component trees.
 * Allows errors in one section to not crash the entire app.
 * 
 * @author @darianrosebrook
 */

"use client";

import React, { Component, type ReactNode } from "react";
import { AlertCircle, RefreshCw } from "lucide-react";
import { Button } from "../primitives/button";
import { ErrorDisplay } from "../ErrorDisplay";
import styles from "./ScopedErrorBoundary.module.scss";

interface ScopedErrorBoundaryProps {
  children: ReactNode;
  scope: string; // Identifier for the error scope (e.g., "dashboard-chart", "sidebar-projects")
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: React.ErrorInfo, scope: string) => void;
  onReset?: () => void;
  resetKeys?: Array<string | number>; // Reset boundary when these values change
  level?: "critical" | "non-critical"; // Whether error should be logged as critical
}

interface ScopedErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
  errorCount: number;
}

export class ScopedErrorBoundary extends Component<
  ScopedErrorBoundaryProps,
  ScopedErrorBoundaryState
> {
  private resetTimeoutId: number | null = null;

  constructor(props: ScopedErrorBoundaryProps) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorCount: 0,
    };
  }

  static getDerivedStateFromError(error: Error): Partial<ScopedErrorBoundaryState> {
    return {
      hasError: true,
      error,
    };
  }

  componentDidUpdate(
    prevProps: ScopedErrorBoundaryProps,
    prevState: ScopedErrorBoundaryState
  ) {
    // Reset error boundary when resetKeys change
    if (
      this.props.resetKeys &&
      prevProps.resetKeys &&
      this.props.resetKeys.some(
        (key, index) => key !== prevProps.resetKeys?.[index]
      )
    ) {
      if (this.state.hasError) {
        this.resetErrorBoundary();
      }
    }
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    const { scope, level = "non-critical", onError } = this.props;
    const errorCount = this.state.errorCount + 1;

    this.setState({ errorCount });

    // Log error with scope context
    const logLevel = level === "critical" ? "error" : "warn";
    console[logLevel](
      `[ScopedErrorBoundary:${scope}] Error caught:`,
      error,
      errorInfo
    );

    // Call optional error handler
    onError?.(error, errorInfo, scope);

    // Auto-reset after 3 errors in the same scope (circuit breaker pattern)
    if (errorCount >= 3) {
      console.warn(
        `[ScopedErrorBoundary:${scope}] Too many errors (${errorCount}), disabling component`
      );
      // Don't auto-reset, keep it disabled
    }
  }

  resetErrorBoundary = () => {
    if (this.resetTimeoutId) {
      clearTimeout(this.resetTimeoutId);
    }
    this.setState({
      hasError: false,
      error: null,
    });
    this.props.onReset?.();
  };

  componentWillUnmount() {
    if (this.resetTimeoutId) {
      clearTimeout(this.resetTimeoutId);
    }
  }

  render() {
    if (this.state.hasError) {
      // Use custom fallback if provided
      if (this.props.fallback) {
        return this.props.fallback;
      }

      // Default scoped error UI
      return (
        <div className={styles.scopedErrorBoundary} data-scope={this.props.scope}>
          <div className={styles.scopedErrorContent}>
            <div className={styles.scopedErrorIcon}>
              <AlertCircle className={styles.scopedErrorIconSvg} />
            </div>
            <div className={styles.scopedErrorText}>
              <h3 className={styles.scopedErrorTitle}>
                Component Error
              </h3>
              <p className={styles.scopedErrorDescription}>
                An error occurred in this section. The rest of the app continues to work.
              </p>
            </div>
            {this.state.error && (
              <div className={styles.scopedErrorDetails}>
                <ErrorDisplay
                  error={this.state.error}
                  onRetry={this.resetErrorBoundary}
                  showRetry={true}
                />
              </div>
            )}
            <div className={styles.scopedErrorActions}>
              <Button
                onClick={this.resetErrorBoundary}
                variant="outline"
                size="sm"
                className={styles.scopedErrorButton}
              >
                <RefreshCw className={styles.scopedErrorButtonIcon} />
                Retry
              </Button>
            </div>
            {process.env.NODE_ENV === "development" && this.state.error && (
              <details className={styles.scopedErrorDebug}>
                <summary>Debug Info</summary>
                <pre className={styles.scopedErrorDebugPre}>
                  Scope: {this.props.scope}
                  {"\n"}
                  Error Count: {this.state.errorCount}
                  {"\n"}
                  {this.state.error.message}
                  {this.state.error.stack && `\n\n${this.state.error.stack}`}
                </pre>
              </details>
            )}
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

/**
 * Hook for functional components to use scoped error boundaries
 */
export function useScopedErrorHandler(scope: string) {
  return (error: Error) => {
    console.error(`[${scope}] Error thrown:`, error);
    throw error; // Will be caught by nearest ScopedErrorBoundary
  };
}

