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
import { Button } from "./ui/button";
import { ErrorDisplay } from "./ErrorDisplay";

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
        <div className="min-h-screen flex items-center justify-center bg-[#0d0d0d] p-8">
          <div className="text-center max-w-2xl">
            <div className="mb-8">
              <div className="inline-flex items-center justify-center w-20 h-20 bg-red-500/20 rounded-full mb-6">
                <AlertCircle className="w-10 h-10 text-red-500" />
              </div>
              <h1 className="text-3xl font-bold text-white mb-4">
                Something went wrong
              </h1>
              <p className="text-gray-400 text-lg mb-6">
                An error occurred in this component. You can try refreshing or
                returning to the dashboard.
              </p>
            </div>

            {this.state.error && (
              <div className="mb-8">
                <ErrorDisplay
                  error={this.state.error}
                  onRetry={this.handleReset}
                  showRetry={true}
                />
              </div>
            )}

            <div className="flex items-center justify-center gap-4">
              <Button
                onClick={this.handleReset}
                variant="default"
                className="flex items-center gap-2"
              >
                <RefreshCw className="w-4 h-4" />
                Try Again
              </Button>
              <Button
                onClick={this.handleGoHome}
                variant="outline"
                className="flex items-center gap-2"
              >
                <Home className="w-4 h-4" />
                Go to Dashboard
              </Button>
            </div>

            {/* Error Details (only in development) */}
            {process.env.NODE_ENV === "development" && this.state.error && (
              <div className="mt-8 text-left bg-[#1a1a1a] border border-red-500/50 rounded-lg p-6">
                <h3 className="text-lg font-semibold text-red-500 mb-4">
                  Error Details
                </h3>
                <pre className="text-xs text-gray-300 overflow-auto">
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
  return (error: Error, errorInfo?: React.ErrorInfo) => {
    // This will be caught by the nearest ErrorBoundary
    throw error;
  };
}

