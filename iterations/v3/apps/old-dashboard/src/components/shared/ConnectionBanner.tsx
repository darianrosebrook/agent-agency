'use client';

/**
 * Connection Status Banner - Client Component
 * 
 * @author @darianrosebrook
 * 
 * Displays connection status with retry functionality.
 * Uses ConnectionProvider context for real-time status updates.
 */

import { memo, useCallback } from 'react';
import { CheckCircle, XCircle, AlertCircle, Loader2, RefreshCw } from 'lucide-react';
import { useConnectionContext, ConnectionAware } from '@/components/providers/ConnectionProvider';
import EnhancedButton from '@/components/ui/EnhancedButton';
import styles from '@/app/page.module.scss';

function ConnectionBanner() {
  const { retryConnection } = useConnectionContext();
  
  const handleRetry = useCallback(() => {
    retryConnection();
  }, [retryConnection]);

  return (
    <ConnectionAware
      online={
        <div className={styles.statusBanner} role="status" aria-live="polite">
          <CheckCircle className={styles.statusIcon} size={20} />
          <span>Connected to Agent Agency API</span>
        </div>
      }
      offline={
        <div className={`${styles.statusBanner} ${styles.offlineBanner}`} role="status" aria-live="polite">
          <XCircle className={styles.statusIcon} size={20} />
          <span>Offline Mode - Using cached data</span>
          <EnhancedButton
            onClick={handleRetry}
            variant="secondary"
            size="sm"
            leftIcon={<RefreshCw size={16} />}
            aria-label="Retry connection to API server"
          >
            Retry Connection
          </EnhancedButton>
        </div>
      }
      degraded={
        <div className={`${styles.statusBanner} ${styles.degradedBanner}`} role="status" aria-live="polite">
          <AlertCircle className={styles.statusIcon} size={20} />
          <span>Limited connectivity - Some features unavailable</span>
          <EnhancedButton
            onClick={handleRetry}
            variant="secondary"
            size="sm"
            leftIcon={<RefreshCw size={16} />}
            aria-label="Retry connection to API server"
          >
            Retry Connection
          </EnhancedButton>
        </div>
      }
      checking={
        <div className={`${styles.statusBanner} ${styles.checkingBanner}`} role="status" aria-live="polite">
          <Loader2 className={`${styles.statusIcon} ${styles.spinning}`} size={20} />
          <span>Checking connection...</span>
        </div>
      }
    />
  );
}

export default memo(ConnectionBanner);

