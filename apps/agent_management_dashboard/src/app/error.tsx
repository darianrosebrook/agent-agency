"use client";

/**
 * Error Page - Global Error Boundary
 * 
 * This page is displayed when an unhandled error occurs in the application.
 */

import { useEffect } from "react";
import Link from "next/link";
import { AlertCircle, Home, RefreshCw } from "lucide-react";
import styles from "./error.module.scss";

interface ErrorProps {
  error: Error & { digest?: string };
  reset: () => void;
}

export default function Error({ error, reset }: ErrorProps) {
  useEffect(() => {
    // TODO: Replace console.error with error logging service with the following requirements:
    // 1. Error logging: Send errors to error tracking service
    //    - Data source: POST /api/errors endpoint or external service (Sentry, LogRocket, etc.)
    //    - Include error message, stack trace, user context, and digest
    //    - Store error metadata in PostgreSQL `error_logs` table via `iterations/v3/data-infrastructure`
    // 2. Error context: Include additional context information
    //    - User ID, session ID, page URL, browser info
    //    - Request ID for tracing
    //    - Timestamp and environment (dev/staging/prod)
    // 3. Error aggregation: Group similar errors for analysis
    //    - Use error digest or fingerprinting
    //    - Track error frequency and trends
    //    - Alert on critical errors
    console.error("Application error:", error);
  }, [error]);

  return (
    <div className={styles.errorPage}>
      <div className={styles.errorContent}>
        <div className={styles.errorHeader}>
          <div className={styles.errorIconContainer}>
            <AlertCircle className={styles.errorIcon} />
          </div>
          <h1 className={styles.errorTitle}>Something went wrong!</h1>
          <p className={styles.errorMessage}>
            An unexpected error occurred. We&apos;ve been notified and are working on a fix.
          </p>
          {error.digest && (
            <p className={styles.errorDigest}>
              Error ID: {error.digest}
            </p>
          )}
        </div>

        <div className={styles.errorActions}>
          <button
            onClick={reset}
            className={styles.errorButton}
          >
            <RefreshCw className={styles.errorButtonIcon} />
            Try Again
          </button>
          <Link
            href="/"
            className={styles.errorLink}
          >
            <Home className={styles.errorLinkIcon} />
            Go to Dashboard
          </Link>
        </div>

        {/* Error Details (only in development) */}
        {process.env.NODE_ENV === "development" && (
          <div className={styles.errorDetails}>
            <h3 className={styles.errorDetailsTitle}>Error Details</h3>
            <pre className={styles.errorDetailsPre}>
              {error.message}
              {error.stack && `\n\n${error.stack}`}
            </pre>
          </div>
        )}
      </div>
    </div>
  );
}








