"use client";

/**
 * Error Page - Global Error Boundary
 * 
 * This page is displayed when an unhandled error occurs in the application.
 */

import { useEffect } from "react";
import Link from "next/link";
import { AlertCircle, Home, RefreshCw } from "lucide-react";

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
    <div className="min-h-screen flex items-center justify-center bg-[#0d0d0d] p-8">
      <div className="text-center max-w-2xl">
        <div className="mb-8">
          <div className="inline-flex items-center justify-center w-20 h-20 bg-red-500/20 rounded-full mb-6">
            <AlertCircle className="w-10 h-10 text-red-500" />
          </div>
          <h1 className="text-3xl font-bold text-white mb-4">Something went wrong!</h1>
          <p className="text-gray-400 text-lg mb-2">
            An unexpected error occurred. We&apos;ve been notified and are working on a fix.
          </p>
          {error.digest && (
            <p className="text-gray-500 text-sm mt-2">
              Error ID: {error.digest}
            </p>
          )}
        </div>

        <div className="flex items-center justify-center gap-4 mb-8">
          <button
            onClick={reset}
            className="flex items-center gap-2 px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors font-medium"
          >
            <RefreshCw className="w-4 h-4" />
            Try Again
          </button>
          <Link
            href="/"
            className="flex items-center gap-2 px-6 py-3 bg-[#1a1a1a] border border-gray-800 text-gray-300 rounded-lg hover:bg-gray-800 transition-colors"
          >
            <Home className="w-4 h-4" />
            Go to Dashboard
          </Link>
        </div>

        {/* Error Details (only in development) */}
        {process.env.NODE_ENV === "development" && (
          <div className="mt-8 text-left bg-[#1a1a1a] border border-red-500/50 rounded-lg p-6">
            <h3 className="text-lg font-semibold text-red-500 mb-4">Error Details</h3>
            <pre className="text-xs text-gray-300 overflow-auto">
              {error.message}
              {error.stack && `\n\n${error.stack}`}
            </pre>
          </div>
        )}
      </div>
    </div>
  );
}

