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
import { Button } from "./ui/button";
import { parseApiError, isRetryableError } from "../lib/errors";
import type { AppError } from "../lib/errors";

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
    <div
      className={`flex gap-3 border border-red-600/20 bg-red-600/10 rounded-lg p-4 ${className}`}
    >
      <div className="flex-shrink-0 mt-0.5">
        <AlertCircle className="w-5 h-5 text-red-500" />
      </div>

      <div className="flex-1 min-w-0">
        <div className="text-sm text-red-400 font-medium mb-1">
          {appError.getUserMessage()}
        </div>
        {process.env.NODE_ENV === "development" && appError.details && (
          <div className="text-xs text-red-500/70 mt-2 font-mono">
            {JSON.stringify(appError.details, null, 2)}
          </div>
        )}
      </div>

      <div className="flex items-start gap-2">
        {showRetry && retryable && onRetry && (
          <Button
            variant="ghost"
            size="sm"
            onClick={onRetry}
            className="text-red-400 hover:text-red-300 hover:bg-red-500/20"
          >
            <RefreshCw className="w-4 h-4 mr-1" />
            Retry
          </Button>
        )}
        {onDismiss && (
          <Button
            variant="ghost"
            size="sm"
            onClick={onDismiss}
            className="text-gray-400 hover:text-gray-300"
          >
            <X className="w-4 h-4" />
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
    <div className="flex gap-2 items-start text-sm text-red-400 mt-1">
      <AlertCircle className="w-4 h-4 mt-0.5 flex-shrink-0" />
      <span>{appError.getUserMessage()}</span>
    </div>
  );
}

