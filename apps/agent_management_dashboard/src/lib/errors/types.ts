/**
 * Error types and utilities for consistent error handling
 *
 * Provides standardized error types and error response formats
 * for API communication and user-facing error messages.
 *
 * @author @darianrosebrook
 */

/**
 * Standard error response format from API
 * Matches backend ErrorResponse format
 */
export interface ApiErrorResponse {
  error?:
    | string
    | { code?: string; message?: string; details?: Record<string, unknown> }; // Human-readable error message or error object
  code?: string; // Machine-readable error code
  status?: number; // HTTP status code
  details?: Record<string, unknown>; // Additional error details
  request_id?: string; // Request ID for correlation
  detail?: string | { msg: string; type: string }[]; // FastAPI-style error
  message?: string; // Standard Error message property
}

/**
 * Error codes for different error types
 */
export enum ErrorCode {
  // Network errors
  NETWORK_ERROR = "NETWORK_ERROR",
  TIMEOUT = "TIMEOUT",
  CONNECTION_FAILED = "CONNECTION_FAILED",

  // API errors
  BAD_REQUEST = "BAD_REQUEST",
  UNAUTHORIZED = "UNAUTHORIZED",
  FORBIDDEN = "FORBIDDEN",
  NOT_FOUND = "NOT_FOUND",
  CONFLICT = "CONFLICT",
  VALIDATION_ERROR = "VALIDATION_ERROR",
  RATE_LIMIT = "RATE_LIMIT",
  SERVER_ERROR = "SERVER_ERROR",

  // Application errors
  INVALID_STATE = "INVALID_STATE",
  OPERATION_FAILED = "OPERATION_FAILED",
  RESOURCE_NOT_FOUND = "RESOURCE_NOT_FOUND",
  PERMISSION_DENIED = "PERMISSION_DENIED",

  // Streaming errors
  STREAM_ERROR = "STREAM_ERROR",
  STREAM_CLOSED = "STREAM_CLOSED",
  STREAM_TIMEOUT = "STREAM_TIMEOUT",
}

/**
 * User-friendly error messages mapped to error codes
 */
export const ErrorMessages: Record<ErrorCode, string> = {
  [ErrorCode.NETWORK_ERROR]:
    "Unable to connect to the server. Please check your internet connection.",
  [ErrorCode.TIMEOUT]: "The request took too long. Please try again.",
  [ErrorCode.CONNECTION_FAILED]:
    "Failed to connect to the server. Please try again later.",
  [ErrorCode.BAD_REQUEST]:
    "Invalid request. Please check your input and try again.",
  [ErrorCode.UNAUTHORIZED]:
    "You are not authorized. Please log in and try again.",
  [ErrorCode.FORBIDDEN]: "You do not have permission to perform this action.",
  [ErrorCode.NOT_FOUND]: "The requested resource was not found.",
  [ErrorCode.CONFLICT]:
    "This action conflicts with existing data. Please refresh and try again.",
  [ErrorCode.VALIDATION_ERROR]:
    "Invalid data provided. Please check your input.",
  [ErrorCode.RATE_LIMIT]:
    "Too many requests. Please wait a moment and try again.",
  [ErrorCode.SERVER_ERROR]: "Server error occurred. Please try again later.",
  [ErrorCode.INVALID_STATE]:
    "Invalid application state. Please refresh the page.",
  [ErrorCode.OPERATION_FAILED]: "Operation failed. Please try again.",
  [ErrorCode.RESOURCE_NOT_FOUND]: "Resource not found.",
  [ErrorCode.PERMISSION_DENIED]: "Permission denied.",
  [ErrorCode.STREAM_ERROR]: "Error occurred while streaming response.",
  [ErrorCode.STREAM_CLOSED]: "Stream connection closed unexpectedly.",
  [ErrorCode.STREAM_TIMEOUT]: "Stream timeout. Please try again.",
};

/**
 * Custom error class for application errors
 */
export class AppError extends Error {
  constructor(
    public code: ErrorCode,
    message?: string,
    public details?: Record<string, unknown>
  ) {
    super(message ?? ErrorMessages[code]);
    this.name = "AppError";
  }

  /**
   * Get user-friendly error message
   */
  getUserMessage(): string {
    return this.message ?? ErrorMessages[this.code];
  }

  /**
   * Check if error is retryable
   */
  isRetryable(): boolean {
    return [
      ErrorCode.NETWORK_ERROR,
      ErrorCode.TIMEOUT,
      ErrorCode.CONNECTION_FAILED,
      ErrorCode.SERVER_ERROR,
      ErrorCode.STREAM_ERROR,
      ErrorCode.STREAM_TIMEOUT,
    ].includes(this.code);
  }
}

/**
 * Map backend error codes to frontend ErrorCode enum
 */
function mapErrorCode(backendCode: string): ErrorCode {
  const codeMap: Record<string, ErrorCode> = {
    DATABASE_ERROR: ErrorCode.SERVER_ERROR,
    NOT_FOUND: ErrorCode.NOT_FOUND,
    TASK_NOT_FOUND: ErrorCode.RESOURCE_NOT_FOUND,
    INVALID_OPERATION: ErrorCode.INVALID_STATE,
    INVALID_REQUEST: ErrorCode.BAD_REQUEST,
    EXECUTION_ERROR: ErrorCode.OPERATION_FAILED,
    VALIDATION_ERROR: ErrorCode.VALIDATION_ERROR,
    AUTHENTICATION_ERROR: ErrorCode.UNAUTHORIZED,
    AUTHORIZATION_ERROR: ErrorCode.FORBIDDEN,
    RATE_LIMIT_EXCEEDED: ErrorCode.RATE_LIMIT,
    INTERNAL_ERROR: ErrorCode.SERVER_ERROR,
    BAD_REQUEST: ErrorCode.BAD_REQUEST,
  };

  return codeMap[backendCode] || ErrorCode.OPERATION_FAILED;
}

/**
 * Safely create an AppError instance
 * Handles cases where AppError might not be available (module loading issues)
 */
function createAppError(
  code: ErrorCode,
  message?: string,
  details?: Record<string, unknown>
): AppError {
  try {
    return new AppError(code, message, details);
  } catch (err) {
    // Fallback if AppError constructor fails (should not happen, but safety check)
    const fallbackError = new Error(
      message ?? ErrorMessages[code]
    ) as unknown as AppError;
    fallbackError.code = code;
    fallbackError.details = details;
    fallbackError.getUserMessage = () => message ?? ErrorMessages[code];
    fallbackError.isRetryable = () =>
      [
        ErrorCode.NETWORK_ERROR,
        ErrorCode.TIMEOUT,
        ErrorCode.CONNECTION_FAILED,
        ErrorCode.SERVER_ERROR,
        ErrorCode.STREAM_ERROR,
        ErrorCode.STREAM_TIMEOUT,
      ].includes(code);
    return fallbackError;
  }
}

/**
 * Parse API error response to AppError
 * Handles both new standardized format and legacy formats
 */
export function parseApiError(error: unknown): AppError {
  // Handle fetch errors
  if (error instanceof TypeError && error.message.includes("fetch")) {
    return createAppError(ErrorCode.NETWORK_ERROR, "Network request failed");
  }

  // Handle API error response (new standardized format)
  if (typeof error === "object" && error !== null) {
    const apiError = error as ApiErrorResponse;

    // New format: { error: string, code: string, status: number, details?, request_id? }
    if (
      "error" in apiError &&
      "code" in apiError &&
      "status" in apiError &&
      typeof apiError.error === "string" &&
      typeof apiError.code === "string" &&
      typeof apiError.status === "number"
    ) {
      const code = mapErrorCode(apiError.code);
      return createAppError(code, apiError.error, apiError.details);
    }

    // Legacy format: { error: { code, message, details } }
    if (
      "error" in apiError &&
      typeof apiError.error === "object" &&
      apiError.error !== null &&
      !Array.isArray(apiError.error)
    ) {
      const legacyError = apiError.error as {
        code?: string;
        message?: string;
        details?: Record<string, unknown>;
      };
      if (legacyError.code && typeof legacyError.code === "string") {
        const code = mapErrorCode(legacyError.code);
        return createAppError(code, legacyError.message, legacyError.details);
      }
    }

    // Handle FastAPI-style errors
    if ("detail" in apiError) {
      return createAppError(ErrorCode.BAD_REQUEST, String(apiError.detail));
    }

    // Handle standard Error objects
    if ("message" in apiError) {
      const message = String(apiError.message);
      if (message.includes("401") || message.includes("Unauthorized")) {
        return createAppError(ErrorCode.UNAUTHORIZED);
      }
      if (message.includes("403") || message.includes("Forbidden")) {
        return createAppError(ErrorCode.FORBIDDEN);
      }
      if (message.includes("404") || message.includes("Not Found")) {
        return createAppError(ErrorCode.NOT_FOUND);
      }
      if (message.includes("429") || message.includes("Rate Limit")) {
        return createAppError(ErrorCode.RATE_LIMIT);
      }
      if (message.includes("500") || message.includes("Server Error")) {
        return createAppError(ErrorCode.SERVER_ERROR);
      }
      return createAppError(ErrorCode.OPERATION_FAILED, message);
    }
  }

  // Handle string errors
  if (typeof error === "string") {
    return createAppError(ErrorCode.OPERATION_FAILED, error);
  }

  // Default fallback
  return createAppError(
    ErrorCode.OPERATION_FAILED,
    "An unexpected error occurred"
  );
}

/**
 * Check if error is a network error
 */
export function isNetworkError(error: unknown): boolean {
  if (error instanceof AppError) {
    return [
      ErrorCode.NETWORK_ERROR,
      ErrorCode.TIMEOUT,
      ErrorCode.CONNECTION_FAILED,
    ].includes(error.code);
  }
  return false;
}

/**
 * Check if error is retryable
 */
export function isRetryableError(error: unknown): boolean {
  if (error instanceof AppError) {
    return error.isRetryable();
  }
  return false;
}
