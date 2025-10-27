/**
 * Standardized Error Handling System
 * Implements error handling patterns as specified in planning document
 *
 * @author @darianrosebrook
 */

// Standardized error codes aligned with planning document
export enum ErrorCode {
  // Network and API errors
  NETWORK_ERROR = 'NETWORK_ERROR',
  TIMEOUT_ERROR = 'TIMEOUT_ERROR',
  API_UNAVAILABLE = 'API_UNAVAILABLE',

  // Authentication and authorization
  AUTHENTICATION_ERROR = 'AUTHENTICATION_ERROR',
  AUTHORIZATION_ERROR = 'AUTHORIZATION_ERROR',
  TOKEN_EXPIRED = 'TOKEN_EXPIRED',
  INSUFFICIENT_PERMISSIONS = 'INSUFFICIENT_PERMISSIONS',

  // Validation errors
  VALIDATION_ERROR = 'VALIDATION_ERROR',
  MISSING_REQUIRED_FIELD = 'MISSING_REQUIRED_FIELD',
  INVALID_FORMAT = 'INVALID_FORMAT',
  INVALID_VALUE = 'INVALID_VALUE',

  // Resource errors
  NOT_FOUND = 'NOT_FOUND',
  ALREADY_EXISTS = 'ALREADY_EXISTS',
  CONFLICT = 'CONFLICT',
  RESOURCE_LOCKED = 'RESOURCE_LOCKED',

  // Business logic errors
  INVALID_OPERATION = 'INVALID_OPERATION',
  OPERATION_FAILED = 'OPERATION_FAILED',
  EXTERNAL_SERVICE_ERROR = 'EXTERNAL_SERVICE_ERROR',

  // System errors
  INTERNAL_ERROR = 'INTERNAL_ERROR',
  DATABASE_ERROR = 'DATABASE_ERROR',
  CONFIGURATION_ERROR = 'CONFIGURATION_ERROR',

  // Council-specific errors
  VERDICT_NOT_FOUND = 'VERDICT_NOT_FOUND',
  VERDICT_ALREADY_OVERRIDDEN = 'VERDICT_ALREADY_OVERRIDDEN',
  JUDGE_UNAVAILABLE = 'JUDGE_UNAVAILABLE',

  // Apple Silicon-specific errors
  HARDWARE_UNAVAILABLE = 'HARDWARE_UNAVAILABLE',
  THERMAL_LIMIT_EXCEEDED = 'THERMAL_LIMIT_EXCEEDED',
  MODEL_LOAD_FAILED = 'MODEL_LOAD_FAILED'
}

// Standardized error severity levels
export enum ErrorSeverity {
  LOW = 'low',
  MEDIUM = 'medium',
  HIGH = 'high',
  CRITICAL = 'critical'
}

// Main API Error interface aligned with planning document
export interface ApiError {
  code: ErrorCode;
  message: string;          // User-friendly message
  details?: any;           // Additional error details
  retryable: boolean;      // Whether operation can be retried
  severity: ErrorSeverity;
  timestamp: Date;
  requestId?: string;      // For tracking specific requests
  stack?: string;          // For debugging (not exposed to users)
}

// Error response interface for API routes
export interface ErrorResponse {
  success: false;
  error: {
    code: ErrorCode;
    message: string;
    details?: any;
    retryable: boolean;
    severity: ErrorSeverity;
    timestamp: string;
  };
}

// Success response interface
export interface SuccessResponse<T = any> {
  success: true;
  data: T;
  timestamp: string;
}

/**
 * Create a standardized API error
 */
export function createApiError(
  code: ErrorCode,
  message: string,
  options: {
    details?: any;
    retryable?: boolean;
    severity?: ErrorSeverity;
    requestId?: string;
    stack?: string;
  } = {}
): ApiError {
  const {
    details,
    retryable = false,
    severity = ErrorSeverity.MEDIUM,
    requestId,
    stack
  } = options;

  return {
    code,
    message,
    details,
    retryable,
    severity,
    timestamp: new Date(),
    requestId,
    stack
  };
}

/**
 * Convert various error types to standardized ApiError
 */
export function normalizeError(error: any, context?: string): ApiError {
  // If already an ApiError, return as-is
  if (isApiError(error)) {
    return error;
  }

  // Handle fetch/network errors
  if (error instanceof TypeError && error.message.includes('fetch')) {
    return createApiError(
      ErrorCode.NETWORK_ERROR,
      'Unable to connect to the server. Please check your internet connection.',
      {
        details: { originalError: error.message },
        retryable: true,
        severity: ErrorSeverity.MEDIUM
      }
    );
  }

  // Handle HTTP response errors
  if (error.status) {
    const httpStatus = error.status;
    const responseText = error.message || error.text || '';

    switch (httpStatus) {
      case 400:
        return createApiError(
          ErrorCode.VALIDATION_ERROR,
          'The request contains invalid data. Please check your input.',
          {
            details: { responseText },
            retryable: false,
            severity: ErrorSeverity.LOW
          }
        );

      case 401:
        return createApiError(
          ErrorCode.AUTHENTICATION_ERROR,
          'Your session has expired. Please log in again.',
          {
            details: { responseText },
            retryable: false,
            severity: ErrorSeverity.MEDIUM
          }
        );

      case 403:
        return createApiError(
          ErrorCode.AUTHORIZATION_ERROR,
          'You do not have permission to perform this action.',
          {
            details: { responseText },
            retryable: false,
            severity: ErrorSeverity.MEDIUM
          }
        );

      case 404:
        return createApiError(
          ErrorCode.NOT_FOUND,
          'The requested resource was not found.',
          {
            details: { responseText },
            retryable: false,
            severity: ErrorSeverity.LOW
          }
        );

      case 409:
        return createApiError(
          ErrorCode.CONFLICT,
          'This action conflicts with the current state.',
          {
            details: { responseText },
            retryable: false,
            severity: ErrorSeverity.LOW
          }
        );

      case 429:
        return createApiError(
          ErrorCode.API_UNAVAILABLE,
          'Too many requests. Please try again in a moment.',
          {
            details: { responseText },
            retryable: true,
            severity: ErrorSeverity.LOW
          }
        );

      case 500:
      case 502:
      case 503:
      case 504:
        return createApiError(
          ErrorCode.INTERNAL_ERROR,
          'The server encountered an error. Please try again later.',
          {
            details: { responseText, httpStatus },
            retryable: true,
            severity: ErrorSeverity.HIGH
          }
        );

      default:
        return createApiError(
          ErrorCode.INTERNAL_ERROR,
          `An unexpected error occurred (${httpStatus}).`,
          {
            details: { responseText, httpStatus },
            retryable: true,
            severity: ErrorSeverity.MEDIUM
          }
        );
    }
  }

  // Handle timeout errors
  if (error.name === 'AbortError' || error.message?.includes('timeout')) {
    return createApiError(
      ErrorCode.TIMEOUT_ERROR,
      'The request timed out. Please try again.',
      {
        details: { originalError: error.message },
        retryable: true,
        severity: ErrorSeverity.MEDIUM
      }
    );
  }

  // Handle generic errors
  const errorMessage = error.message || 'An unexpected error occurred.';
  return createApiError(
    ErrorCode.INTERNAL_ERROR,
    errorMessage,
    {
      details: { originalError: error, context },
      retryable: false,
      severity: ErrorSeverity.MEDIUM,
      stack: error.stack
    }
  );
}

/**
 * Type guard for ApiError
 */
export function isApiError(error: any): error is ApiError {
  return error !== null &&
         typeof error === 'object' &&
         'code' in error &&
         'message' in error &&
         'retryable' in error &&
         'severity' in error &&
         'timestamp' in error &&
         Object.values(ErrorCode).includes(error.code);
}

/**
 * Create Next.js API response from ApiError
 */
export function createErrorResponse(error: ApiError) {
  const statusCode = getHttpStatusCode(error.code);

  return {
    success: false,
    error: {
      code: error.code,
      message: error.message,
      details: error.details,
      retryable: error.retryable,
      severity: error.severity,
      timestamp: error.timestamp.toISOString()
    }
  } as ErrorResponse;
}

/**
 * Map error codes to HTTP status codes
 */
function getHttpStatusCode(errorCode: ErrorCode): number {
  switch (errorCode) {
    case ErrorCode.VALIDATION_ERROR:
    case ErrorCode.MISSING_REQUIRED_FIELD:
    case ErrorCode.INVALID_FORMAT:
    case ErrorCode.INVALID_VALUE:
      return 400;

    case ErrorCode.AUTHENTICATION_ERROR:
    case ErrorCode.TOKEN_EXPIRED:
      return 401;

    case ErrorCode.AUTHORIZATION_ERROR:
    case ErrorCode.INSUFFICIENT_PERMISSIONS:
      return 403;

    case ErrorCode.NOT_FOUND:
    case ErrorCode.VERDICT_NOT_FOUND:
      return 404;

    case ErrorCode.CONFLICT:
    case ErrorCode.ALREADY_EXISTS:
    case ErrorCode.VERDICT_ALREADY_OVERRIDDEN:
    case ErrorCode.RESOURCE_LOCKED:
      return 409;

    case ErrorCode.API_UNAVAILABLE:
      return 429;

    case ErrorCode.INTERNAL_ERROR:
    case ErrorCode.DATABASE_ERROR:
    case ErrorCode.CONFIGURATION_ERROR:
    case ErrorCode.OPERATION_FAILED:
    case ErrorCode.EXTERNAL_SERVICE_ERROR:
    case ErrorCode.JUDGE_UNAVAILABLE:
    case ErrorCode.HARDWARE_UNAVAILABLE:
    case ErrorCode.THERMAL_LIMIT_EXCEEDED:
    case ErrorCode.MODEL_LOAD_FAILED:
      return 500;

    case ErrorCode.NETWORK_ERROR:
    case ErrorCode.TIMEOUT_ERROR:
      return 503;

    case ErrorCode.INVALID_OPERATION:
      return 422;

    default:
      return 500;
  }
}

/**
 * Error logging utility
 */
export function logError(error: ApiError, context?: string) {
  const logData = {
    code: error.code,
    message: error.message,
    severity: error.severity,
    timestamp: error.timestamp.toISOString(),
    requestId: error.requestId,
    context,
    details: error.details,
    stack: error.stack
  };

  // Log based on severity
  switch (error.severity) {
    case ErrorSeverity.CRITICAL:
      console.error('🚨 CRITICAL ERROR:', logData);
      // Could send to error monitoring service
      break;

    case ErrorSeverity.HIGH:
      console.error('❌ HIGH ERROR:', logData);
      break;

    case ErrorSeverity.MEDIUM:
      console.warn('⚠️ MEDIUM ERROR:', logData);
      break;

    case ErrorSeverity.LOW:
    default:
      console.info('ℹ️ LOW ERROR:', logData);
      break;
  }
}

/**
 * React hook for error handling
 */
export function useErrorHandler() {
  const handleError = (error: ApiError | any, context?: string) => {
    const normalizedError = normalizeError(error, context);
    logError(normalizedError, context);

    // Could integrate with toast notifications or error UI
    // For now, just log the error
    return normalizedError;
  };

  const createSuccessResponse = <T>(data: T): SuccessResponse<T> => ({
    success: true,
    data,
    timestamp: new Date().toISOString()
  });

  return {
    handleError,
    createSuccessResponse,
    normalizeError,
    isApiError
  };
}
