/**
 * Toast notification utilities
 * 
 * Provides consistent toast notifications using sonner.
 * Wraps sonner with error handling and user-friendly messages.
 * 
 * @author @darianrosebrook
 */

import { toast as sonnerToast } from 'sonner';
import { parseApiError, ErrorCode, type AppError } from '../errors';

/**
 * Toast notification options
 */
interface ToastOptions {
  duration?: number;
  action?: {
    label: string;
    onClick: () => void;
  };
  cancel?: {
    label: string;
    onClick?: () => void;
  };
}

/**
 * Show success toast
 */
export function toastSuccess(message: string, options?: ToastOptions) {
  return sonnerToast.success(message, {
    duration: options?.duration || 4000,
    ...options,
  });
}

/**
 * Show error toast with error parsing
 */
export function toastError(error: unknown, options?: ToastOptions) {
  const appError = parseApiError(error);
  return sonnerToast.error(appError.getUserMessage(), {
    duration: options?.duration || 6000,
    ...options,
  });
}

/**
 * Show warning toast
 */
export function toastWarning(message: string, options?: ToastOptions) {
  return sonnerToast.warning(message, {
    duration: options?.duration || 5000,
    ...options,
  });
}

/**
 * Show info toast
 */
export function toastInfo(message: string, options?: ToastOptions) {
  return sonnerToast.info(message, {
    duration: options?.duration || 4000,
    ...options,
  });
}

/**
 * Show loading toast (returns dismiss function)
 */
export function toastLoading(message: string): () => void {
  return sonnerToast.loading(message);
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
    error: messages.error || ((error: unknown) => {
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

