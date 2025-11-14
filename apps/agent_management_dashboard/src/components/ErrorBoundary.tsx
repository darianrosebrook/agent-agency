/**
 * Error Boundary Component
 *
 * Catches React component errors and displays a fallback UI.
 * Prevents the entire app from crashing when a component fails.
 *
 * @author @darianrosebrook
 */

"use client";

import React, { Component, type ReactNode } from "react";
import { AlertCircle, RefreshCw, Home } from "lucide-react";
import { Button } from "./primitives/button";
import { ErrorDisplay } from "./ErrorDisplay";
import { env } from "../lib/utils/env";
import styles from "./ErrorBoundary.module.scss";

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: React.ErrorInfo) => void;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    // Log error to error tracking service
    console.error("ErrorBoundary caught an error:", error, errorInfo);

    // Call optional error handler
    this.props.onError?.(error, errorInfo);
  }

  handleReset = () => {
    this.setState({ hasError: false, error: null });
  };

  handleGoHome = () => {
    window.location.href = "/";
  };

  render() {
    if (this.state.hasError) {
      // Use custom fallback if provided
      if (this.props.fallback) {
        return this.props.fallback;
      }

      // Default error UI
      return (
        <div className={styles.errorBoundary}>
          <div className={styles.errorContent}>
            <div className={styles.errorIconContainer}>
              <div className={styles.errorIconWrapper}>
                <AlertCircle className={styles.errorIcon} />
              </div>
              <h1 className={styles.errorTitle}>
                Something went wrong
              </h1>
              <p className={styles.errorDescription}>
                An error occurred in this component. You can try refreshing or
                returning to the dashboard.
              </p>
            </div>

            {this.state.error && (
              <div style={{ marginBottom: '2rem' }}>
                <ErrorDisplay
                  error={this.state.error}
                  onRetry={this.handleReset}
                  showRetry={true}
                />
              </div>
            )}

            <div className={styles.errorActions}>
              <Button
                onClick={this.handleReset}
                variant="default"
                className={styles.errorActionButton}
              >
                <RefreshCw className={styles.buttonIcon} />
                Try Again
              </Button>
              <Button
                onClick={this.handleGoHome}
                variant="outline"
                className={styles.errorActionButton}
              >
                <Home className={styles.buttonIcon} />
                Go to Dashboard
              </Button>
            </div>

            {/* Error Details (only in development) */}
            {env.DEV && this.state.error && (
              <div className={styles.errorDetails}>
                <h3 className={styles.errorDetailsTitle}>
                  Error Details
                </h3>
                <pre className={styles.errorDetailsPre}>
                  {this.state.error.message}
                  {this.state.error.stack && `\n\n${this.state.error.stack}`}
                </pre>
              </div>
            )}
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

/**
 * Hook for error boundary (for functional components)
 */
export function useErrorHandler() {
  return (error: Error) => {
    // This will be caught by the nearest ErrorBoundary
    throw error;
  };
}
