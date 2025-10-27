/**
 * Error Handler Hook
 * Provides standardized error handling for React components
 *
 * @author @darianrosebrook
 */

import { useCallback } from 'react';
import { ApiError, ErrorCode, normalizeError, logError } from '@/lib/errors';

interface ErrorHandlerOptions {
  showToast?: boolean;
  logError?: boolean;
  context?: string;
}

export function useErrorHandler() {
  const handleError = useCallback((
    error: ApiError | any,
    options: ErrorHandlerOptions = {}
  ): ApiError => {
    const {
      showToast = true,
      logError: shouldLogError = true,
      context
    } = options;

    // Normalize the error
    const normalizedError = normalizeError(error, context);

    // Log the error if requested
    if (shouldLogError) {
      logError(normalizedError, context);
    }

    // Show toast notification if requested
    if (showToast) {
      showErrorToast(normalizedError);
    }

    return normalizedError;
  }, []);

  const createSuccessResponse = useCallback(<T>(data: T) => ({
    success: true,
    data,
    timestamp: new Date().toISOString()
  }), []);

  const handleApiResponse = useCallback(<T>(
    response: { success: boolean; data?: T; error?: any },
    options: ErrorHandlerOptions = {}
  ): T | null => {
    if (response.success && response.data) {
      return response.data;
    }

    if (response.error) {
      handleError(response.error, options);
    }

    return null;
  }, [handleError]);

  return {
    handleError,
    createSuccessResponse,
    handleApiResponse,
    normalizeError
  };
}

/**
 * Show error toast notification
 * This is a placeholder - integrate with your toast system
 */
function showErrorToast(error: ApiError) {
  // Placeholder for toast notification
  // Replace with your actual toast implementation
  console.warn('Error Toast:', error.message);

  // Example integration with a toast library:
  // toast.error(error.message, {
  //   description: error.details?.description,
  //   duration: getToastDuration(error.severity),
  //   action: error.retryable ? {
  //     label: 'Retry',
  //     onClick: () => retryFunction()
  //   } : undefined
  // });
}

/**
 * Get toast duration based on error severity
 */
function getToastDuration(severity: string): number {
  switch (severity) {
    case 'critical':
      return 10000; // 10 seconds
    case 'high':
      return 7000;  // 7 seconds
    case 'medium':
      return 5000;  // 5 seconds
    case 'low':
    default:
      return 4000;  // 4 seconds
  }
}