/**
 * Toast notification utilities
 * 
 * Provides consistent toast notifications using sonner.
 * Wraps sonner with error handling and user-friendly messages.
 * 
 * @author @darianrosebrook
 */

import { toast as sonnerToast, type ExternalToast, type Action } from 'sonner';
import { parseApiError, ErrorMessages, ErrorCode } from '../errors';
import { addNotification, type NotificationType } from '../stores/notificationStore';
import type React from 'react';

/**
 * Toast notification options
 */
interface ToastOptions {
  duration?: number;
  action?: Action | React.ReactNode;
  cancel?: Action | React.ReactNode;
}

/**
 * Debounce tracking for error toasts
 * Prevents spam by tracking recent error messages and their timestamps
 */
interface ErrorDebounceEntry {
  message: string;
  code: ErrorCode;
  timestamp: number;
  count: number;
}

const ERROR_DEBOUNCE_WINDOW_MS = 5000; // 5 seconds
const errorDebounceMap = new Map<string, ErrorDebounceEntry>();

/**
 * Generate a key for error debouncing based on message and code
 */
function getErrorKey(message: string, code: ErrorCode): string {
  return `${code}:${message}`;
}

/**
 * Check if error should be shown (debounced)
 * Returns true if error should be shown, false if it should be suppressed
 */
function shouldShowError(message: string, code: ErrorCode): boolean {
  const now = Date.now();
  const key = getErrorKey(message, code);
  const entry = errorDebounceMap.get(key);

  if (!entry) {
    // First occurrence - allow it
    errorDebounceMap.set(key, {
      message,
      code,
      timestamp: now,
      count: 1,
    });
    return true;
  }

  const timeSinceLastShow = now - entry.timestamp;

  if (timeSinceLastShow < ERROR_DEBOUNCE_WINDOW_MS) {
    // Within debounce window - suppress toast but increment count
    entry.count++;
    entry.timestamp = now;
    return false;
  }

  // Outside debounce window - allow it and reset count
  entry.count = 1;
  entry.timestamp = now;
  return true;
}

/**
 * Clean up old debounce entries periodically
 */
function cleanupDebounceMap() {
  const now = Date.now();
  for (const [key, entry] of errorDebounceMap.entries()) {
    if (now - entry.timestamp > ERROR_DEBOUNCE_WINDOW_MS * 2) {
      errorDebounceMap.delete(key);
    }
  }
}

// Clean up old entries every 30 seconds
if (typeof window !== 'undefined') {
  setInterval(cleanupDebounceMap, 30000);
}

/**
 * Show success toast
 */
export function toastSuccess(message: string, options?: ToastOptions) {
  // Persist notification
  addNotification({
    type: 'success',
    message,
  });

  const toastOptions: ExternalToast = {
    duration: options?.duration ?? 4000,
    ...(options?.action && { action: options.action }),
    ...(options?.cancel && { cancel: options.cancel }),
  };
  return sonnerToast.success(message, toastOptions);
}

/**
 * Show error toast with error parsing and debouncing
 */
export function toastError(error: unknown, options?: ToastOptions) {
  try {
  const appError = parseApiError(error);
    const message = appError.getUserMessage();
    
    // Ensure we always have a valid, non-empty string message
    let displayMessage: string;
    if (typeof message === 'string' && message.trim().length > 0) {
      displayMessage = message.trim();
    } else {
      // Fallback to error code message or default
      displayMessage = ErrorMessages[appError.code] || 'An unexpected error occurred';
    }
    
    // Ensure displayMessage is a string (defensive check)
    if (typeof displayMessage !== 'string' || displayMessage.length === 0) {
      displayMessage = 'An unexpected error occurred';
    }
    
    // Always persist notification to store (deduplication will prevent duplicates)
    // This ensures all errors are available in the notification center
    addNotification({
      type: 'error',
      message: displayMessage,
      errorCode: appError.code,
      errorDetails: appError.details,
    });

    // Check if we should show the toast (debounced)
    const shouldShow = shouldShowError(displayMessage, appError.code);
    
    if (!shouldShow) {
      // Suppress toast but log that it was suppressed
      const key = getErrorKey(displayMessage, appError.code);
      const entry = errorDebounceMap.get(key);
      if (entry && entry.count > 1) {
        // Log suppression only if count > 1 to avoid spam in console too
        console.debug(`[Toast Debounce] Suppressed duplicate error: "${displayMessage}" (${entry.count} occurrences)`);
      }
      return;
    }

    // Show toast with optional count indicator if there were suppressed duplicates
    const key = getErrorKey(displayMessage, appError.code);
    const entry = errorDebounceMap.get(key);
    let finalMessage = displayMessage;
    
    if (entry && entry.count > 1) {
      // Add count indicator if there were suppressed duplicates
      finalMessage = `${displayMessage} (${entry.count}x)`;
    }

  const toastOptions: ExternalToast = {
    duration: options?.duration ?? 6000,
    ...(options?.action && { action: options.action }),
    ...(options?.cancel && { cancel: options.cancel }),
  };
    
    return sonnerToast.error(finalMessage, toastOptions);
  } catch (err) {
    // If anything fails, show a generic error toast (but still debounce it)
    console.error('Error in toastError:', err);
    const genericMessage = 'An unexpected error occurred';
    // Use OPERATION_FAILED as fallback since UNKNOWN_ERROR doesn't exist
    if (shouldShowError(genericMessage, ErrorCode.OPERATION_FAILED)) {
      return sonnerToast.error(genericMessage, {
        duration: 6000,
      });
    }
  }
}

/**
 * Show warning toast
 */
export function toastWarning(message: string, options?: ToastOptions) {
  // Persist notification
  addNotification({
    type: 'warning',
    message,
  });

  const toastOptions: ExternalToast = {
    duration: options?.duration ?? 5000,
    ...(options?.action && { action: options.action }),
    ...(options?.cancel && { cancel: options.cancel }),
  };
  return sonnerToast.warning(message, toastOptions);
}

/**
 * Show info toast
 */
export function toastInfo(message: string, options?: ToastOptions) {
  // Persist notification
  addNotification({
    type: 'info',
    message,
  });

  const toastOptions: ExternalToast = {
    duration: options?.duration ?? 4000,
    ...(options?.action && { action: options.action }),
    ...(options?.cancel && { cancel: options.cancel }),
  };
  return sonnerToast.info(message, toastOptions);
}

/**
 * Show loading toast (returns dismiss function)
 */
export function toastLoading(message: string): () => void {
  const toastId = sonnerToast.loading(message);
  return () => sonnerToast.dismiss(toastId);
}

/**
 * Show promise toast (loading -> success/error)
 * Automatically parses errors for user-friendly messages
 */
export function toastPromise<T>(
  promise: Promise<T>,
  messages: {
    loading: string;
    success: string | ((data: T) => string);
    error?: string | ((error: unknown) => string);
  }
) {
  return sonnerToast.promise(promise, {
    loading: messages.loading,
    success: messages.success,
    error: messages.error ?? ((error: unknown) => {
      const appError = parseApiError(error);
      return appError.getUserMessage();
    }),
  });
}

/**
 * Dismiss all toasts
 */
export function dismissAllToasts() {
  sonnerToast.dismiss();
}

/**
 * Dismiss specific toast
 */
export function dismissToast(toastId: string | number) {
  sonnerToast.dismiss(toastId);
}

// Re-export toast for direct access if needed
export { sonnerToast as toast };

