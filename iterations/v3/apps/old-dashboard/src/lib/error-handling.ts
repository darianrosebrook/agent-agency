/**
 * Comprehensive Error Handling and Recovery System
 * Provides intelligent error classification, recovery strategies, and user feedback
 *
 * @author @darianrosebrook
 */

import React from 'react';

import { ApiError } from './api-client';

// Error categories for classification
export enum ErrorCategory {
  NETWORK = 'network',
  TIMEOUT = 'timeout',
  ABORTED = 'aborted',
  RATE_LIMIT = 'rate_limit',
  AUTHENTICATION = 'authentication',
  AUTHORIZATION = 'authorization',
  VALIDATION = 'validation',
  SERVER = 'server',
  CLIENT = 'client',
  UNKNOWN = 'unknown',
}

// Error severity levels
export enum ErrorSeverity {
  LOW = 'low',         // Minor issues, user can continue
  MEDIUM = 'medium',   // Significant but recoverable errors
  HIGH = 'high',       // Critical errors requiring attention
  CRITICAL = 'critical' // System-breaking errors
}

// Recovery strategies
export enum RecoveryStrategy {
  RETRY = 'retry',
  REFRESH = 'refresh',
  REAUTHENTICATE = 'reauthenticate',
  RECONNECT = 'reconnect',
  FALLBACK = 'fallback',
  NOTIFY_USER = 'notify_user',
  ESCALATE = 'escalate',
}

// Enhanced error interface
export interface AppError {
  id: string;
  category: ErrorCategory;
  severity: ErrorSeverity;
  message: string;
  originalError?: Error;
  context?: Record<string, any>;
  timestamp: string;
  recoveryStrategies: RecoveryStrategy[];
  userMessage: string;
  technicalDetails?: string;
  retryCount: number;
  maxRetries: number;
  isRecoverable: boolean;
}

// Error recovery configuration
export interface ErrorRecoveryConfig {
  maxRetries: number;
  retryDelay: number;
  backoffMultiplier: number;
  maxBackoffDelay: number;
  enableAutoRecovery: boolean;
}

// Default recovery configuration
const DEFAULT_RECOVERY_CONFIG: ErrorRecoveryConfig = {
  maxRetries: 3,
  retryDelay: 1000,
  backoffMultiplier: 2,
  maxBackoffDelay: 30000,
  enableAutoRecovery: true,
};

// Error classification patterns
const ERROR_PATTERNS = {
  [ErrorCategory.NETWORK]: [
    /fetch/i, /network/i, /connection/i, /ECONNREFUSED/i, /ENOTFOUND/i,
    /CORS/i, /cross-origin/i
  ],
  [ErrorCategory.TIMEOUT]: [
    /timeout/i, /timed out/i, /request timeout/i
  ],
  [ErrorCategory.ABORTED]: [
    /aborted/i, /abort/i, /cancelled/i
  ],
  [ErrorCategory.RATE_LIMIT]: [
    /rate limit/i, /too many requests/i, /429/i, /quota/i
  ],
  [ErrorCategory.AUTHENTICATION]: [
    /unauthorized/i, /401/i, /auth/i, /login/i, /token/i
  ],
  [ErrorCategory.AUTHORIZATION]: [
    /forbidden/i, /403/i, /permission/i, /access denied/i
  ],
  [ErrorCategory.VALIDATION]: [
    /validation/i, /invalid/i, /required/i, /400/i, /bad request/i
  ],
  [ErrorCategory.SERVER]: [
    /500/i, /internal server error/i, /502/i, /503/i, /504/i,
    /service unavailable/i, /server error/i
  ],
  [ErrorCategory.CLIENT]: [
    /4\d\d/i, /client error/i, /bad request/i
  ],
};

// User-friendly error messages
const USER_MESSAGES = {
  [ErrorCategory.NETWORK]: "Connection issue. Please check your internet connection and try again.",
  [ErrorCategory.TIMEOUT]: "Request timed out. The server may be busy. Please try again.",
  [ErrorCategory.ABORTED]: "Request was cancelled. Please try again if needed.",
  [ErrorCategory.RATE_LIMIT]: "Too many requests. Please wait a moment before trying again.",
  [ErrorCategory.AUTHENTICATION]: "Authentication required. Please log in to continue.",
  [ErrorCategory.AUTHORIZATION]: "Access denied. You don't have permission to perform this action.",
  [ErrorCategory.VALIDATION]: "Please check your input and try again.",
  [ErrorCategory.SERVER]: "Server error occurred. Our team has been notified.",
  [ErrorCategory.CLIENT]: "Request error. Please check your input and try again.",
  [ErrorCategory.UNKNOWN]: "An unexpected error occurred. Please try again.",
};

// Recovery strategies by error category
const RECOVERY_STRATEGIES = {
  [ErrorCategory.NETWORK]: [RecoveryStrategy.RETRY, RecoveryStrategy.RECONNECT],
  [ErrorCategory.TIMEOUT]: [RecoveryStrategy.RETRY],
  [ErrorCategory.ABORTED]: [RecoveryStrategy.NOTIFY_USER], // User cancelled, no auto-retry
  [ErrorCategory.RATE_LIMIT]: [RecoveryStrategy.RETRY],
  [ErrorCategory.AUTHENTICATION]: [RecoveryStrategy.REAUTHENTICATE],
  [ErrorCategory.AUTHORIZATION]: [RecoveryStrategy.NOTIFY_USER],
  [ErrorCategory.VALIDATION]: [RecoveryStrategy.NOTIFY_USER],
  [ErrorCategory.SERVER]: [RecoveryStrategy.RETRY, RecoveryStrategy.ESCALATE],
  [ErrorCategory.CLIENT]: [RecoveryStrategy.NOTIFY_USER],
  [ErrorCategory.UNKNOWN]: [RecoveryStrategy.RETRY, RecoveryStrategy.ESCALATE],
};

// Severity mapping
const CATEGORY_SEVERITY = {
  [ErrorCategory.NETWORK]: ErrorSeverity.MEDIUM,
  [ErrorCategory.TIMEOUT]: ErrorSeverity.MEDIUM,
  [ErrorCategory.ABORTED]: ErrorSeverity.LOW, // User cancelled, not a real error
  [ErrorCategory.RATE_LIMIT]: ErrorSeverity.LOW,
  [ErrorCategory.AUTHENTICATION]: ErrorSeverity.HIGH,
  [ErrorCategory.AUTHORIZATION]: ErrorSeverity.HIGH,
  [ErrorCategory.VALIDATION]: ErrorSeverity.LOW,
  [ErrorCategory.SERVER]: ErrorSeverity.HIGH,
  [ErrorCategory.CLIENT]: ErrorSeverity.MEDIUM,
  [ErrorCategory.UNKNOWN]: ErrorSeverity.MEDIUM,
};

/**
 * Classify error based on message and context
 */
export function classifyError(error: Error | string, context?: any): ErrorCategory {
  const message = error instanceof Error ? error.message : error;

  for (const [category, patterns] of Object.entries(ERROR_PATTERNS)) {
    for (const pattern of patterns) {
      if (pattern.test(message)) {
        return category as ErrorCategory;
      }
    }
  }

  // Check context for additional clues
  if (context?.status) {
    const status = context.status;
    if (status === 401) return ErrorCategory.AUTHENTICATION;
    if (status === 403) return ErrorCategory.AUTHORIZATION;
    if (status === 429) return ErrorCategory.RATE_LIMIT;
    if (status >= 400 && status < 500) return ErrorCategory.CLIENT;
    if (status >= 500) return ErrorCategory.SERVER;
  }

  return ErrorCategory.UNKNOWN;
}

/**
 * Determine error severity
 */
export function getErrorSeverity(category: ErrorCategory): ErrorSeverity {
  return CATEGORY_SEVERITY[category] || ErrorSeverity.MEDIUM;
}

/**
 * Get user-friendly message
 */
export function getUserMessage(category: ErrorCategory): string {
  return USER_MESSAGES[category] || USER_MESSAGES[ErrorCategory.UNKNOWN];
}

/**
 * Get recovery strategies for error category
 */
export function getRecoveryStrategies(category: ErrorCategory): RecoveryStrategy[] {
  return RECOVERY_STRATEGIES[category] || [RecoveryStrategy.RETRY];
}

/**
 * Create enhanced AppError from raw error
 */
export function createAppError(
  error: Error | ApiError | string,
  context?: Record<string, any>,
  retryCount = 0
): AppError {
  const originalError = error instanceof Error ? error : new Error(String(error));
  const category = classifyError(originalError, context);
  const severity = getErrorSeverity(category);
  const recoveryStrategies = getRecoveryStrategies(category);
  const userMessage = getUserMessage(category);

  // Determine if error is recoverable
  const isRecoverable = recoveryStrategies.includes(RecoveryStrategy.RETRY) ||
                       recoveryStrategies.includes(RecoveryStrategy.RECONNECT);

  // Set max retries based on category
  const maxRetries = category === ErrorCategory.RATE_LIMIT ? 1 :
                    category === ErrorCategory.AUTHENTICATION ? 0 :
                    category === ErrorCategory.AUTHORIZATION ? 0 : 3;

  return {
    id: crypto.randomUUID(),
    category,
    severity,
    message: originalError.message,
    originalError,
    context,
    timestamp: new Date().toISOString(),
    recoveryStrategies,
    userMessage,
    technicalDetails: process.env.NODE_ENV === 'development' ? originalError.stack : undefined,
    retryCount,
    maxRetries,
    isRecoverable,
  };
}

/**
 * Error recovery manager
 */
export class ErrorRecoveryManager {
  private config: ErrorRecoveryConfig;
  private recoveryHandlers: Map<RecoveryStrategy, (error: AppError) => Promise<void>>;
  private activeRecoveries: Set<string>;

  constructor(config: Partial<ErrorRecoveryConfig> = {}) {
    this.config = { ...DEFAULT_RECOVERY_CONFIG, ...config };
    this.recoveryHandlers = new Map();
    this.activeRecoveries = new Set();
    this.setupDefaultHandlers();
  }

  /**
   * Register a recovery handler
   */
  registerHandler(strategy: RecoveryStrategy, handler: (error: AppError) => Promise<void>) {
    this.recoveryHandlers.set(strategy, handler);
  }

  /**
   * Attempt to recover from an error
   */
  async attemptRecovery(error: AppError): Promise<boolean> {
    if (!error.isRecoverable || !this.config.enableAutoRecovery) {
      return false;
    }

    // Prevent duplicate recovery attempts
    if (this.activeRecoveries.has(error.id)) {
      return false;
    }

    this.activeRecoveries.add(error.id);

    try {
      for (const strategy of error.recoveryStrategies) {
        const handler = this.recoveryHandlers.get(strategy);
        if (handler) {
          console.log(`Attempting recovery strategy: ${strategy} for error: ${error.id}`);
          await handler(error);
          this.activeRecoveries.delete(error.id);
          return true;
        }
      }

      this.activeRecoveries.delete(error.id);
      return false;
    } catch (recoveryError) {
      console.error(`Recovery failed for error ${error.id}:`, recoveryError);
      this.activeRecoveries.delete(error.id);
      return false;
    }
  }

  /**
   * Calculate retry delay with exponential backoff
   */
  calculateRetryDelay(retryCount: number): number {
    const delay = this.config.retryDelay * Math.pow(this.config.backoffMultiplier, retryCount);
    return Math.min(delay, this.config.maxBackoffDelay);
  }

  /**
   * Set up default recovery handlers
   */
  private setupDefaultHandlers() {
    // Retry handler
    this.registerHandler(RecoveryStrategy.RETRY, async (error) => {
      if (error.retryCount >= error.maxRetries) {
        throw new Error('Max retries exceeded');
      }

      const delay = this.calculateRetryDelay(error.retryCount);
      await new Promise(resolve => setTimeout(resolve, delay));

      // This would trigger a retry in the calling context
      throw new Error('RETRY_REQUESTED');
    });

    // Reconnect handler
    this.registerHandler(RecoveryStrategy.RECONNECT, async (error) => {
      // Trigger reconnection in WebSocket/SSE hooks
      window.dispatchEvent(new CustomEvent('reconnect-requested', { detail: error }));
    });

    // Refresh handler
    this.registerHandler(RecoveryStrategy.REFRESH, async (error) => {
      // Trigger page refresh
      window.location.reload();
    });

    // Reauthenticate handler
    this.registerHandler(RecoveryStrategy.REAUTHENTICATE, async (error) => {
      // Redirect to login
      window.location.href = '/login';
    });

    // Notify user handler
    this.registerHandler(RecoveryStrategy.NOTIFY_USER, async (error) => {
      // Show user notification
      window.dispatchEvent(new CustomEvent('error-notification', { detail: error }));
    });

    // Escalate handler
    this.registerHandler(RecoveryStrategy.ESCALATE, async (error) => {
      // Log to monitoring service
      console.error('Escalating critical error:', error);

      // Could send to monitoring service here
      // await monitoringService.reportError(error);
    });
  }
}

// Global error recovery manager instance
let errorRecoveryManager: ErrorRecoveryManager | null = null;

export function getErrorRecoveryManager(): ErrorRecoveryManager {
  if (!errorRecoveryManager) {
    errorRecoveryManager = new ErrorRecoveryManager();
  }
  return errorRecoveryManager;
}

/**
 * React hook for error handling
 */
export function useErrorHandler() {
  const recoveryManager = getErrorRecoveryManager();

  const handleError = async (error: Error | ApiError | string, context?: Record<string, any>) => {
    const appError = createAppError(error, context);

    console.error(`[${appError.category}] ${appError.message}`, {
      error: appError,
      context,
    });

    // Dispatch error event for UI components
    window.dispatchEvent(new CustomEvent('app-error', { detail: appError }));

    // Attempt automatic recovery
    const recovered = await recoveryManager.attemptRecovery(appError);

    if (!recovered) {
      // Show user notification for unrecoverable errors
      window.dispatchEvent(new CustomEvent('error-notification', { detail: appError }));
    }

    return appError;
  };

  const clearError = (errorId: string) => {
    window.dispatchEvent(new CustomEvent('clear-error', { detail: { errorId } }));
  };

  return {
    handleError,
    clearError,
    recoveryManager,
  };
}

/**
 * Error boundary component
 */
// Error boundary utilities (implement ErrorBoundary in .tsx files)
export function handleErrorBoundaryError(error: Error): AppError {
  return createAppError(error);
}

// Error notification utilities
export function createErrorNotification(error: AppError, showTechnicalDetails = false) {
  return {
    error,
    showTechnicalDetails,
    title: "Something went wrong",
    message: error.userMessage,
    isCritical: error.severity === ErrorSeverity.CRITICAL,
    criticalMessage: "This appears to be a critical error. Please contact support.",
    technicalDetails: showTechnicalDetails ? JSON.stringify(error, null, 2) : undefined,
  };
}

/**
 * Standardized API route error handler
 * Provides consistent error responses across all API endpoints
 */
export function handleApiRouteError(error: any, context?: string) {
  const appError = createAppError(error, { context });

  // Log the error
  console.error(`[${appError.category}] API Route Error:`, {
    message: appError.message,
    context,
    timestamp: appError.timestamp,
    id: appError.id,
  });

  // Determine HTTP status code
  const statusCode = getHttpStatusCode(appError.category);

  return {
    success: false,
    error: {
      code: appError.category.toLowerCase(),
      message: appError.userMessage,
      timestamp: appError.timestamp,
      retryable: appError.isRecoverable,
      id: appError.id,
      details: process.env.NODE_ENV === 'development' ? {
        technicalMessage: appError.message,
        originalError: appError.originalError?.message,
        context,
      } : undefined,
    },
  } as const;
}

/**
 * Map error categories to HTTP status codes for API routes
 */
export function getHttpStatusCode(category: ErrorCategory): number {
  switch (category) {
    case ErrorCategory.NETWORK:
    case ErrorCategory.TIMEOUT:
    case ErrorCategory.ABORTED:
      return 503; // Service Unavailable

    case ErrorCategory.RATE_LIMIT:
      return 429; // Too Many Requests

    case ErrorCategory.AUTHENTICATION:
      return 401; // Unauthorized

    case ErrorCategory.AUTHORIZATION:
      return 403; // Forbidden

    case ErrorCategory.VALIDATION:
      return 400; // Bad Request

    case ErrorCategory.SERVER:
      return 500; // Internal Server Error

    case ErrorCategory.CLIENT:
      return 400; // Bad Request

    case ErrorCategory.UNKNOWN:
    default:
      return 500; // Internal Server Error
  }
}
