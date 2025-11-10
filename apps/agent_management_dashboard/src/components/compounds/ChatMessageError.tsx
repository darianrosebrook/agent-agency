/**
 * Chat Message Error Component
 *
 * Displays error messages within chat messages with retry functionality.
 * Handles various error formats gracefully, matching open-webui patterns.
 *
 * @author @darianrosebrook
 */

"use client";

import React from "react";
import { AlertCircle, RefreshCw } from "lucide-react";
import { Button } from "../ui/button";
import { parseApiError, isRetryableError } from "../../lib/errors";
import type { AppError } from "../../lib/errors";
import { cn } from "../ui/utils";
import styles from "./ChatMessageError.module.scss";

interface ChatMessageErrorProps {
  error: unknown;
  onRetry?: () => void | Promise<void>;
  className?: string;
}

/**
 * Extracts error message from various error formats
 * Handles string, object, nested error structures
 */
function extractErrorMessage(error: unknown): string {
  if (typeof error === "string") {
    return error;
  }

  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === "object" && error !== null) {
    const errorObj = error as Record<string, unknown>;

    // Handle nested error structures
    if (errorObj.error) {
      const nestedError = errorObj.error;
      if (typeof nestedError === "string") {
        return nestedError;
      }
      if (
        typeof nestedError === "object" &&
        nestedError !== null &&
        "message" in nestedError
      ) {
        return String((nestedError as { message: unknown }).message);
      }
    }

    // Handle common error fields
    if (errorObj.detail) {
      return String(errorObj.detail);
    }

    if (errorObj.message) {
      return String(errorObj.message);
    }

    // Fallback to JSON stringify for complex objects
    if (process.env.NODE_ENV === "development") {
      return JSON.stringify(errorObj, null, 2);
    }
  }

  return "An error occurred";
}

export function ChatMessageError({
  error,
  onRetry,
  className = "",
}: ChatMessageErrorProps) {
  const appError = parseApiError(error);
  const retryable = isRetryableError(error);
  const errorMessage = extractErrorMessage(error);

  // Log error for debugging
  React.useEffect(() => {
    if (process.env.NODE_ENV === "development") {
      console.error("Chat message error:", error);
    }
  }, [error]);

  return (
    <div className={cn(styles.chatMessageError, className)}>
      <div className={styles.chatMessageErrorIcon}>
        <AlertCircle className={styles.chatMessageErrorIconSvg} />
      </div>

      <div className={styles.chatMessageErrorContent}>
        <div className={styles.chatMessageErrorTitle}>
          {appError.getUserMessage() || errorMessage}
        </div>
        {process.env.NODE_ENV === "development" && appError.details && (
          <div className={styles.chatMessageErrorDetails}>
            {JSON.stringify(appError.details, null, 2)}
          </div>
        )}
      </div>

      {retryable && onRetry && (
        <div className={styles.chatMessageErrorRetry}>
          <Button
            variant="ghost"
            size="sm"
            onClick={onRetry}
            className={styles.retryButton}
          >
            <RefreshCw className={styles.retryButtonIcon} />
            Retry
          </Button>
        </div>
      )}
    </div>
  );
}
