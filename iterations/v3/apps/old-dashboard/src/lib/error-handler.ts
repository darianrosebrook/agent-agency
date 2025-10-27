/**
 * Error Handling Utilities for Agent Agency V3 Dashboard
 * 
 * @author @darianrosebrook
 * 
 * Centralized error handling with user-friendly messages,
 * logging, and recovery strategies.
 */

export interface ErrorContext {
  component?: string;
  action?: string;
  userId?: string;
  timestamp?: Date;
  metadata?: Record<string, unknown>;
}

export interface ErrorInfo {
  message: string;
  userMessage: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  recoverable: boolean;
  context: ErrorContext;
}

export class DashboardError extends Error {
  public readonly userMessage: string;
  public readonly severity: ErrorInfo['severity'];
  public readonly recoverable: boolean;
  public readonly context: ErrorContext;
  public readonly timestamp: Date;

  constructor(
    message: string,
    userMessage: string,
    severity: ErrorInfo['severity'] = 'medium',
    recoverable: boolean = true,
    context: ErrorContext = {}
  ) {
    super(message);
    this.name = 'DashboardError';
    this.userMessage = userMessage;
    this.severity = severity;
    this.recoverable = recoverable;
    this.context = context;
    this.timestamp = new Date();
  }
}

/**
 * Error severity levels with corresponding user messages
 */
export const ERROR_SEVERITY_MESSAGES = {
  low: {
    title: 'Minor Issue',
    description: 'A minor issue occurred but the system is still functioning normally.',
    action: 'You can continue using the dashboard. The issue will be automatically resolved.',
  },
  medium: {
    title: 'Temporary Issue',
    description: 'A temporary issue occurred that may affect some features.',
    action: 'Please try refreshing the page or try again in a few moments.',
  },
  high: {
    title: 'Service Disruption',
    description: 'Some features are currently unavailable due to a service issue.',
    action: 'Please try again later or contact support if the issue persists.',
  },
  critical: {
    title: 'System Error',
    description: 'A critical error occurred that requires immediate attention.',
    action: 'Please contact support immediately with the error details below.',
  },
} as const;

/**
 * Common error types with predefined user messages
 */
export const COMMON_ERRORS = {
  NETWORK_ERROR: {
    userMessage: 'Unable to connect to the server. Please check your internet connection.',
    severity: 'medium' as const,
    recoverable: true,
  },
  AUTHENTICATION_ERROR: {
    userMessage: 'Your session has expired. Please log in again.',
    severity: 'high' as const,
    recoverable: true,
  },
  PERMISSION_ERROR: {
    userMessage: 'You do not have permission to perform this action.',
    severity: 'medium' as const,
    recoverable: false,
  },
  VALIDATION_ERROR: {
    userMessage: 'Please check your input and try again.',
    severity: 'low' as const,
    recoverable: true,
  },
  SERVER_ERROR: {
    userMessage: 'The server encountered an error. Please try again later.',
    severity: 'high' as const,
    recoverable: true,
  },
  TIMEOUT_ERROR: {
    userMessage: 'The request timed out. Please try again.',
    severity: 'medium' as const,
    recoverable: true,
  },
  UNKNOWN_ERROR: {
    userMessage: 'An unexpected error occurred. Please try again or contact support.',
    severity: 'medium' as const,
    recoverable: true,
  },
} as const;

/**
 * Error handler class for centralized error management
 */
export class ErrorHandler {
  private static instance: ErrorHandler;
  private errorLog: ErrorInfo[] = [];
  private maxLogSize = 100;

  private constructor() {}

  public static getInstance(): ErrorHandler {
    if (!ErrorHandler.instance) {
      ErrorHandler.instance = new ErrorHandler();
    }
    return ErrorHandler.instance;
  }

  /**
   * Handle and categorize errors
   */
  public handleError(
    error: Error | unknown,
    context: ErrorContext = {}
  ): ErrorInfo {
    const errorInfo = this.categorizeError(error, context);
    this.logError(errorInfo);
    return errorInfo;
  }

  /**
   * Categorize error and determine user message
   */
  private categorizeError(
    error: Error | unknown,
    context: ErrorContext
  ): ErrorInfo {
    // Handle known error types
    if (error instanceof DashboardError) {
      return {
        message: error.message,
        userMessage: error.userMessage,
        severity: error.severity,
        recoverable: error.recoverable,
        context: { ...error.context, ...context },
      };
    }

    // Handle network errors
    if (error instanceof TypeError && error.message.includes('fetch')) {
      return {
        message: error.message,
        userMessage: COMMON_ERRORS.NETWORK_ERROR.userMessage,
        severity: COMMON_ERRORS.NETWORK_ERROR.severity,
        recoverable: COMMON_ERRORS.NETWORK_ERROR.recoverable,
        context,
      };
    }

    // Handle timeout errors
    if (error instanceof Error && error.name === 'AbortError') {
      return {
        message: error.message,
        userMessage: COMMON_ERRORS.TIMEOUT_ERROR.userMessage,
        severity: COMMON_ERRORS.TIMEOUT_ERROR.severity,
        recoverable: COMMON_ERRORS.TIMEOUT_ERROR.recoverable,
        context,
      };
    }

    // Handle HTTP errors
    if (typeof Response !== 'undefined' && error instanceof Response) {
      const severity = error.status >= 500 ? 'high' : 'medium';
      return {
        message: `HTTP ${error.status}: ${error.statusText}`,
        userMessage: COMMON_ERRORS.SERVER_ERROR.userMessage,
        severity,
        recoverable: true,
        context,
      };
    }

    // Default to unknown error
    return {
      message: error instanceof Error ? error.message : 'Unknown error',
      userMessage: COMMON_ERRORS.UNKNOWN_ERROR.userMessage,
      severity: COMMON_ERRORS.UNKNOWN_ERROR.severity,
      recoverable: COMMON_ERRORS.UNKNOWN_ERROR.recoverable,
      context,
    };
  }

  /**
   * Log error for debugging and monitoring
   */
  private logError(errorInfo: ErrorInfo): void {
    // Add to internal log
    this.errorLog.push(errorInfo);
    
    // Maintain log size
    if (this.errorLog.length > this.maxLogSize) {
      this.errorLog = this.errorLog.slice(-this.maxLogSize);
    }

    // Log to console in development
    if (process.env.NODE_ENV === 'development') {
      console.error('Dashboard Error:', {
        message: errorInfo.message,
        userMessage: errorInfo.userMessage,
        severity: errorInfo.severity,
        context: errorInfo.context,
        timestamp: new Date().toISOString(),
      });
    }

    // TODO: Send to error tracking service in production
    // Example: Sentry, LogRocket, etc.
  }

  /**
   * Get error log for debugging
   */
  public getErrorLog(): ErrorInfo[] {
    return [...this.errorLog];
  }

  /**
   * Clear error log
   */
  public clearErrorLog(): void {
    this.errorLog = [];
  }

  /**
   * Create a user-friendly error message
   */
  public createUserMessage(errorInfo: ErrorInfo): string {
    const severityInfo = ERROR_SEVERITY_MESSAGES[errorInfo.severity];
    return `${severityInfo.title}: ${errorInfo.userMessage}`;
  }

  /**
   * Check if error is recoverable
   */
  public isRecoverable(error: Error | unknown): boolean {
    if (error instanceof DashboardError) {
      return error.recoverable;
    }
    return true; // Default to recoverable for unknown errors
  }

  /**
   * Get recovery suggestions based on error type
   */
  public getRecoverySuggestions(errorInfo: ErrorInfo): string[] {
    const suggestions: string[] = [];

    switch (errorInfo.severity) {
      case 'low':
        suggestions.push('Continue using the dashboard normally');
        break;
      case 'medium':
        suggestions.push('Try refreshing the page');
        suggestions.push('Check your internet connection');
        break;
      case 'high':
        suggestions.push('Try again in a few minutes');
        suggestions.push('Contact support if the issue persists');
        break;
      case 'critical':
        suggestions.push('Contact support immediately');
        suggestions.push('Include the error details in your message');
        break;
    }

    return suggestions;
  }
}

// Export singleton instance
export const errorHandler = ErrorHandler.getInstance();

/**
 * Utility function for quick error handling
 */
export function handleError(
  error: Error | unknown,
  context: ErrorContext = {}
): ErrorInfo {
  return errorHandler.handleError(error, context);
}

/**
 * Utility function for creating user-friendly error messages
 */
export function createUserErrorMessage(error: Error | unknown): string {
  const errorInfo = errorHandler.handleError(error);
  return errorHandler.createUserMessage(errorInfo);
}

/**
 * Utility function for checking if error is recoverable
 */
export function isErrorRecoverable(error: Error | unknown): boolean {
  return errorHandler.isRecoverable(error);
}
