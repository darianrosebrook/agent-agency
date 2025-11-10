/**
 * Toast notification utilities
 * 
 * Provides consistent toast notifications using sonner.
 * Wraps sonner with error handling and user-friendly messages.
 * 
 * @author @darianrosebrook
 */

import { toast as sonnerToast, type ExternalToast, type Action } from 'sonner';
import { parseApiError, ErrorMessages } from '../errors';
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
 * Show error toast with error parsing
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
    
    // Persist notification
    addNotification({
      type: 'error',
      message: displayMessage,
      errorCode: appError.code,
      errorDetails: appError.details,
    });

    const toastOptions: ExternalToast = {
      duration: options?.duration ?? 6000,
      ...(options?.action && { action: options.action }),
      ...(options?.cancel && { cancel: options.cancel }),
    };
    
    return sonnerToast.error(displayMessage, toastOptions);
  } catch (err) {
    // If anything fails, show a generic error toast
    console.error('Error in toastError:', err);
    return sonnerToast.error('An unexpected error occurred', {
      duration: 6000,
    });
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

