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
import { Button } from '@/design-system/primitives';
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
                <Button
                  variant="primary"
                  size="md"
                  onClick={reset}
                  leftIcon={<RefreshCw size={16} />}
                  aria-label="Try again to reload dashboard"
                >
                  Try Again
                </Button>
                <Button
                  variant="secondary"
                  size="md"
                  onClick={() => window.location.href = '/'}
                  leftIcon={<Home size={16} />}
                  aria-label="Return to dashboard home"
                >
                  Return Home
                </Button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

