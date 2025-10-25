'use client';

import { Component, ReactNode } from 'react';
import { AlertCircle, RefreshCw, Home } from 'lucide-react';
import { Button } from '@/design-system/primitives';
import styles from './ErrorBoundary.module.scss';

interface ErrorBoundaryState {
  hasError: boolean;
  error?: Error;
}

interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: any) => void;
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: any) {
    console.error('ErrorBoundary caught an error:', error, errorInfo);
    this.props.onError?.(error, errorInfo);
  }

  handleRetry = () => {
    this.setState({ hasError: false });
  };

  handleGoHome = () => {
    window.location.href = '/';
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <div className={styles.errorBoundary}>
          <div className={styles.content}>
            <AlertCircle size={48} className={styles.icon} />
            <h2 className={styles.title}>Something went wrong</h2>
            <p className={styles.message}>
              We're sorry, but something unexpected happened. Please try again.
            </p>
            {process.env.NODE_ENV === 'development' && this.state.error && (
              <details className={styles.errorDetails}>
                <summary>Error Details</summary>
                <pre>{this.state.error.stack}</pre>
              </details>
            )}
            <div className={styles.actions}>
              <Button
                variant="primary"
                onClick={this.handleRetry}
                className={styles.button || ''}
              >
                <RefreshCw size={16} />
                Try Again
              </Button>
              <Button
                variant="secondary"
                onClick={this.handleGoHome}
                className={styles.button || ''}
              >
                <Home size={16} />
                Go Home
              </Button>
            </div>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

interface ErrorFallbackProps {
  error?: Error;
  resetError?: () => void;
}

export function ErrorFallback({ error, resetError }: ErrorFallbackProps) {
  return (
    <div className={styles.errorFallback}>
      <div className={styles.content}>
        <AlertCircle size={48} className={styles.icon} />
        <h2 className={styles.title}>Oops! Something went wrong</h2>
        <p className={styles.message}>
          We encountered an unexpected error. Please try refreshing the page.
        </p>
        {process.env.NODE_ENV === 'development' && error && (
          <details className={styles.errorDetails}>
            <summary>Error Details</summary>
            <pre>{error.stack}</pre>
          </details>
        )}
        <div className={styles.actions}>
          <Button
            variant="primary"
            onClick={() => window.location.reload()}
            className={styles.button || ''}
          >
            <RefreshCw size={16} />
            Refresh Page
          </Button>
          {resetError && (
            <Button
              variant="secondary"
              onClick={resetError}
              className={styles.button || ''}
            >
              Try Again
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}
