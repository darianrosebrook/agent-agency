/**
 * Error Boundary Component for Agent Agency V3 Dashboard
 * 
 * @author @darianrosebrook
 * 
 * Comprehensive error boundary with user-friendly error display,
 * recovery options, and detailed error reporting.
 */

'use client';

import React, { Component, ErrorInfo, ReactNode } from 'react';
import { AlertTriangle, RefreshCw, Home, Bug, Copy } from 'lucide-react';
import { Button, Text } from '@/design-system/primitives';
import { errorHandler, ErrorContext } from '@/lib/error-handler';
import styles from './ErrorBoundary.module.scss';

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
  errorId: string | null;
  retryCount: number;
}

interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
  context?: ErrorContext;
  showDetails?: boolean;
  maxRetries?: number;
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  private retryTimeoutId: ReturnType<typeof setTimeout> | null = null;

  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
      errorId: null,
      retryCount: 0,
    };
  }

  static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
    return {
      hasError: true,
      error,
    };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    const { onError, context } = this.props;
    
    // Generate unique error ID
    const errorId = `error-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    
    // Handle error through centralized error handler
    const errorContext: ErrorContext = {
      component: 'ErrorBoundary',
      action: 'componentDidCatch',
      timestamp: new Date(),
      ...context,
    };

    const handledError = errorHandler.handleError(error, errorContext);
    
    this.setState({
      error,
      errorInfo,
      errorId,
    });

    // Call custom error handler if provided
    if (onError) {
      onError(error, errorInfo);
    }

    // Log error details
    console.error('ErrorBoundary caught error:', {
      error,
      errorInfo,
      errorId,
      context: errorContext,
      handledError,
    });
  }

  componentWillUnmount() {
    if (this.retryTimeoutId) {
      clearTimeout(this.retryTimeoutId);
    }
  }

  private handleRetry = () => {
    const { maxRetries = 3 } = this.props;
    const { retryCount } = this.state;

    if (retryCount >= maxRetries) {
      console.warn('Maximum retry attempts reached');
      return;
    }

    this.setState(prevState => ({
      hasError: false,
      error: null,
      errorInfo: null,
      errorId: null,
      retryCount: prevState.retryCount + 1,
    }));
  };

  private handleReset = () => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null,
      errorId: null,
      retryCount: 0,
    });
  };

  private handleGoHome = () => {
    window.location.href = '/';
  };

  private handleCopyErrorDetails = async () => {
    const { error, errorInfo, errorId } = this.state;
    
    const errorDetails = {
      errorId,
      message: error?.message,
      stack: error?.stack,
      componentStack: errorInfo?.componentStack,
      timestamp: new Date().toISOString(),
      userAgent: typeof navigator !== 'undefined' ? navigator.userAgent : 'Unknown',
      url: typeof window !== 'undefined' ? window.location.href : 'Unknown',
    };

    try {
      if (typeof navigator !== 'undefined' && navigator.clipboard) {
        await navigator.clipboard.writeText(JSON.stringify(errorDetails, null, 2));
      }
      // TODO: Show toast notification
      console.log('Error details copied to clipboard');
    } catch (err) {
      console.error('Failed to copy error details:', err);
    }
  };

  private renderErrorDetails() {
    const { error, errorInfo, errorId } = this.state;
    const { showDetails = process.env.NODE_ENV === 'development' } = this.props;

    if (!showDetails) return null;

    return (
      <details className={styles.errorDetails}>
        <summary className={styles.errorDetailsSummary}>
          <Bug size={16} />
          Technical Details
        </summary>
        <div className={styles.errorDetailsContent}>
          <div className={styles.errorDetailsSection}>
            <Text variant="caption" weight="medium">Error ID:</Text>
            <code className={styles.errorCode}>{errorId}</code>
          </div>
          
          {error ? (
            <div className={styles.errorDetailsSection}>
              <Text variant="caption" weight="medium">Error Message:</Text>
              <code className={styles.errorCode}>{error.message}</code>
            </div>
          ) : null}
          
          {error?.stack ? (
            <div className={styles.errorDetailsSection}>
              <Text variant="caption" weight="medium">Stack Trace:</Text>
              <pre className={styles.errorStack}>
                {error.stack}
              </pre>
            </div>
          ) : null}
          
          {errorInfo?.componentStack ? (
            <div className={styles.errorDetailsSection}>
              <Text variant="caption" weight="medium">Component Stack:</Text>
              <pre className={styles.errorStack}>
                {errorInfo.componentStack}
              </pre>
            </div>
          ) : null}
        </div>
      </details>
    );
  }

  render() {
    const { hasError, error, retryCount } = this.state;
    const { children, fallback } = this.props;

    if (hasError) {
      // Use custom fallback if provided
      if (fallback) {
        return fallback;
      }

      // Get error information
      const errorInfo = errorHandler.handleError(error, {
        component: 'ErrorBoundary',
        action: 'render',
        timestamp: new Date(),
      });

      const severityInfo = errorInfo.severity;
      const isRecoverable = errorHandler.isRecoverable(error);
      const recoverySuggestions = errorHandler.getRecoverySuggestions(errorInfo);

      return (
        <div className={styles.errorBoundary} role="alert" aria-live="assertive">
          <div className={styles.errorContainer}>
            <div className={styles.errorHeader}>
              <AlertTriangle 
                className={styles.errorIcon} 
                size={48} 
                aria-hidden="true"
              />
              <div className={styles.errorContent}>
                <Text variant="h2" weight="medium" className={styles.errorTitle}>
                  {severityInfo.title}
                </Text>
                <Text variant="paragraph-medium" color="secondary" className={styles.errorDescription}>
                  {errorInfo.userMessage}
                </Text>
            {recoverySuggestions.length > 0 ? (
              <div className={styles.errorSuggestions}>
                <Text variant="paragraph-small" weight="medium">
                  What you can do:
                </Text>
                <ul className={styles.errorSuggestionsList}>
                  {recoverySuggestions.map((suggestion) => (
                    <li key={`suggestion-${suggestion.slice(0, 20)}-${suggestion.length}`}>
                      <Text variant="paragraph-small" color="secondary">
                        {suggestion}
                      </Text>
                    </li>
                  ))}
                </ul>
              </div>
            ) : null}
              </div>
            </div>

            <div className={styles.errorActions}>
              {isRecoverable && retryCount < (this.props.maxRetries || 3) ? (
                <Button
                  variant="primary"
                  size="md"
                  onClick={this.handleRetry}
                  leftIcon={<RefreshCw size={16} />}
                  aria-label="Try again to recover from error"
                >
                  Try Again
                </Button>
              ) : null}
              
              <Button
                variant="secondary"
                size="md"
                onClick={this.handleReset}
                leftIcon={<Home size={16} />}
                aria-label="Reset to initial state"
              >
                Reset
              </Button>
              
              <Button
                variant="tertiary"
                size="md"
                onClick={this.handleGoHome}
                leftIcon={<Home size={16} />}
                aria-label="Return to dashboard home"
              >
                Go Home
              </Button>

              {process.env.NODE_ENV === 'development' ? (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={this.handleCopyErrorDetails}
                  leftIcon={<Copy size={14} />}
                  aria-label="Copy error details to clipboard"
                >
                  Copy Details
                </Button>
              ) : null}
            </div>

            {this.renderErrorDetails()}
          </div>
        </div>
      );
    }

    return children;
  }
}

export default ErrorBoundary;
