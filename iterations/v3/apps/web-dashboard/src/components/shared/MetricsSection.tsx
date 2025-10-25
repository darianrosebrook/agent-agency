'use client';

/**
 * Metrics Section - Client Component
 * 
 * @author @darianrosebrook
 * 
 * Displays task metrics with offline support.
 * Uses suspense-compatible data fetching.
 */

import { memo, useCallback } from 'react';
import { Clock, RefreshCw, Activity } from 'lucide-react';
import TaskMetrics from '@/components/tasks/TaskMetrics';
import EnhancedButton from '@/components/ui/EnhancedButton';
import { useOfflineMetrics } from '@/hooks/useOfflineData';
import styles from '@/app/page.module.scss';

function MetricsSection() {
  const {
    data: metrics,
    isLoading,
    error,
    isStale,
    refresh: refreshMetrics
  } = useOfflineMetrics();

  const handleRefresh = useCallback(() => {
    refreshMetrics();
  }, [refreshMetrics]);

  if (error && !metrics) {
    return (
      <div className={styles.metricsSection}>
        <div className={styles.emptyState}>
          <Activity className={styles.emptyIcon} size={48} />
          <h3>No Metrics Available</h3>
          <p>Unable to load task metrics at this time.</p>
          <EnhancedButton
            onClick={handleRefresh}
            variant="primary"
            size="md"
            leftIcon={<RefreshCw size={16} />}
            className={styles.connectButton}
          >
            Try Again
          </EnhancedButton>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.metricsSection}>
      {metrics && (
        <>
          <TaskMetrics metrics={metrics} />
          {isStale && (
            <div className={styles.staleData}>
              <Clock className={styles.staleIcon} size={16} />
              <span>Data may be outdated</span>
              <EnhancedButton
                onClick={handleRefresh}
                variant="ghost"
                size="sm"
                leftIcon={<RefreshCw size={14} />}
                className={styles.refreshButton}
              >
                Refresh
              </EnhancedButton>
            </div>
          )}
        </>
      )}
    </div>
  );
}

export default memo(MetricsSection);

