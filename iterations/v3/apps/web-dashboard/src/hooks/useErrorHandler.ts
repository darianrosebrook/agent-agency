'use client';

import { useCallback } from 'react';
import { useToast } from '@/components/providers/ToastProvider';

interface ErrorHandlerOptions {
  showToast?: boolean;
  logError?: boolean;
  fallbackMessage?: string;
}

export function useErrorHandler(options: ErrorHandlerOptions = {}) {
  const { showToast = true, logError = true, fallbackMessage = 'An unexpected error occurred' } = options;
  const { addToast } = useToast();

  const handleError = useCallback((error: unknown, context?: string) => {
    const errorMessage = error instanceof Error ? error.message : fallbackMessage;
    const fullContext = context ? `${context}: ${errorMessage}` : errorMessage;

    if (logError) {
      console.error('Error handled:', { error, context, fullContext });
    }

    if (showToast) {
      addToast({ type: 'error', title: 'Error', message: fullContext });
    }

    return fullContext;
  }, [addToast, logError, fallbackMessage]);

  const handleAsyncError = useCallback(async <T>(
    asyncFn: () => Promise<T>,
    context?: string
  ): Promise<T | null> => {
    try {
      return await asyncFn();
    } catch (error) {
      handleError(error, context);
      return null;
    }
  }, [handleError]);

  return {
    handleError,
    handleAsyncError,
  };
}

interface RetryOptions {
  maxRetries?: number;
  delay?: number;
  backoff?: boolean;
}

export function useRetryableError(options: RetryOptions = {}) {
  const { maxRetries = 3, delay = 1000, backoff = true } = options;
  const { handleError } = useErrorHandler();

  const retry = useCallback(async <T>(
    asyncFn: () => Promise<T>,
    context?: string
  ): Promise<T | null> => {
    let currentDelay = delay;

    for (let attempt = 0; attempt <= maxRetries; attempt++) {
      try {
        return await asyncFn();
      } catch (error) {
        
        if (attempt === maxRetries) {
          handleError(error, `${context} (failed after ${maxRetries + 1} attempts)`);
          return null;
        }

        if (backoff) {
          currentDelay *= 2;
        }

        await new Promise(resolve => setTimeout(resolve, currentDelay));
      }
    }

    return null;
  }, [maxRetries, delay, backoff, handleError]);

  return { retry };
}
