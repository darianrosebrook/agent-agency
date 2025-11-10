/**
 * Error Display Component
 *
 * Displays user-friendly error messages with retry functionality.
 * Inspired by Open-WebUI's error display patterns.
 *
 * @author @darianrosebrook
 */

"use client";

import { AlertCircle, RefreshCw, X } from "lucide-react";
import { Button } from "./primitives/button";
import { parseApiError, isRetryableError } from "../lib/errors";
import { cn } from "./primitives/utils";
import styles from "./ErrorDisplay.module.scss";

interface ErrorDisplayProps {
  error: unknown;
  onRetry?: () => void;
  onDismiss?: () => void;
  className?: string;
  showRetry?: boolean;
}

export function ErrorDisplay({
  error,
  onRetry,
  onDismiss,
  className = "",
  showRetry = true,
}: ErrorDisplayProps) {
  const appError = parseApiError(error);
  const retryable = isRetryableError(error);

  return (
    <div className={cn(styles.errorDisplay, className)}>
      <div className={styles.errorIcon}>
        <AlertCircle className={styles.errorIconSvg} />
      </div>

      <div className={styles.errorContent}>
        <div className={styles.errorMessage}>
          {appError.getUserMessage()}
        </div>
        {process.env.NODE_ENV === "development" && appError.details && (
          <div className={styles.errorDetails}>
            {JSON.stringify(appError.details, null, 2)}
          </div>
        )}
      </div>

      <div className={styles.errorActions}>
        {showRetry && retryable && onRetry && (
          <Button
            variant="ghost"
            size="sm"
            onClick={onRetry}
            className={styles.retryButton}
          >
            <RefreshCw className={styles.buttonIconWithMargin} />
            Retry
          </Button>
        )}
        {onDismiss && (
          <Button
            variant="ghost"
            size="sm"
            onClick={onDismiss}
            className={styles.dismissButton}
          >
            <X className={styles.buttonIcon} />
          </Button>
        )}
      </div>
    </div>
  );
}

/**
 * Inline error display (smaller, for forms)
 */
export function InlineError({ error }: { error: unknown }) {
  const appError = parseApiError(error);

  return (
    <div className={styles.inlineError}>
      <AlertCircle className={styles.inlineErrorIcon} />
      <span>{appError.getUserMessage()}</span>
    </div>
  );
}
