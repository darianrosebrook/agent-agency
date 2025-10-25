'use client';

/**
 * Error boundary for dashboard page
 * 
 * @author @darianrosebrook
 * 
 * Catches and displays errors in a user-friendly way while maintaining
 * layout stability to prevent CLS issues.
 */

import { useEffect } from 'react';
import { AlertTriangle, RefreshCw, Home } from 'lucide-react';
import styles from './page.module.scss';

export default function Error({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  useEffect(() => {
    console.error('Dashboard error:', error);
  }, [error]);

  return (
    <div className={styles.page}>
      <div className={styles.mainContent}>
        <div className={styles.container}>
          <div className={styles.error} role="alert" aria-live="assertive">
            <AlertTriangle className={styles.errorIcon} size={48} />
            <div className={styles.errorContent}>
              <h2>Something went wrong</h2>
              <p>{error.message || 'An unexpected error occurred while loading the dashboard.'}</p>
              {error.digest && (
                <p className={styles.errorDigest}>
                  Error ID: <code>{error.digest}</code>
                </p>
              )}
              <div className={styles.errorActions}>
                <button
                  onClick={reset}
                  className={styles.retryButton}
                  aria-label="Try again to reload dashboard"
                >
                  <RefreshCw size={16} />
                  Try Again
                </button>
                <a href="/" className={styles.homeButton}>
                  <Home size={16} />
                  Return Home
                </a>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

