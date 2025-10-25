'use client';

/**
 * System Status Card - Client Component
 * 
 * @author @darianrosebrook
 * 
 * Displays real-time system health indicators.
 * Updates automatically based on connection state.
 */

import { memo } from 'react';
import { CheckCircle, XCircle, AlertCircle, Loader2, Database, Cpu, Activity, AlertTriangle, CheckCircle2 } from 'lucide-react';
import { DashboardCard } from '@/design-system/composers';
// import { Text, StatusBadge } from '@/design-system';
import { useConnectionContext, OfflineOnly } from '@/components/providers/ConnectionProvider';
import styles from '@/app/page.module.scss';

function SystemStatusCard() {
  const { connection } = useConnectionContext();

  return (
    <DashboardCard
      title="System Status"
      description="Real-time health monitoring"
      className={styles.card || ''}
    >
        <div className={styles.status}>
          <div className={styles.statusItem}>
            <span className={styles.statusLabel}>API Server</span>
            <div className={styles.statusValue}>
              {connection.state === "online" ? (
                <div className={styles.statusIndicator}>
                  <CheckCircle size={16} className={styles.statusIconOnline} />
                  <span>Connected</span>
                </div>
              ) : connection.state === "offline" ? (
                <div className={styles.statusIndicator}>
                  <XCircle size={16} className={styles.statusIconOffline} />
                  <span>Disconnected</span>
                </div>
              ) : connection.state === "degraded" ? (
                <div className={styles.statusIndicator}>
                  <AlertCircle size={16} className={styles.statusIconDegraded} />
                  <span>Degraded</span>
                </div>
              ) : (
                <div className={styles.statusIndicator}>
                  <Loader2 size={16} className={`${styles.statusIconChecking} ${styles.spinning}`} />
                  <span>Checking</span>
                </div>
              )}
            </div>
          </div>
          <div className={styles.statusItem}>
            <span className={styles.statusLabel}>Database</span>
            <div className={styles.statusValue}>
              {connection.apiAvailable ? (
                <div className={styles.statusIndicator}>
                  <Database size={16} className={styles.statusIconOnline} />
                  <span>Available</span>
                </div>
              ) : (
                <div className={styles.statusIndicator}>
                  <Database size={16} className={styles.statusIconDegraded} />
                  <span>Cached</span>
                </div>
              )}
            </div>
          </div>
          <div className={styles.statusItem}>
            <span className={styles.statusLabel}>Workers</span>
            <div className={styles.statusValue}>
              {connection.apiAvailable ? (
                <div className={styles.statusIndicator}>
                  <Cpu size={16} className={styles.statusIconOnline} />
                  <span>Active</span>
                </div>
              ) : (
                <div className={styles.statusIndicator}>
                  <Cpu size={16} className={styles.statusIconDegraded} />
                  <span>Limited</span>
                </div>
              )}
            </div>
          </div>
          <div className={styles.statusItem}>
            <span className={styles.statusLabel}>Health Monitor</span>
            <div className={styles.statusValue}>
              {connection.apiAvailable ? (
                <div className={styles.statusIndicator}>
                  <Activity size={16} className={styles.statusIconOnline} />
                  <span>Active</span>
                </div>
              ) : (
                <div className={styles.statusIndicator}>
                  <Activity size={16} className={styles.statusIconDegraded} />
                  <span>Cached</span>
                </div>
              )}
            </div>
          </div>
        </div>

        <OfflineOnly>
          <div className={styles.statusNote}>
            <div className={styles.statusNoteItem}>
              <AlertTriangle size={16} className={styles.statusNoteIcon} />
              <span className={styles.warningText}>Running in offline mode. Real-time features are limited.</span>
            </div>
            <div className={styles.statusNoteItem}>
              <CheckCircle2 size={16} className={styles.statusNoteIcon} />
              <span className={styles.successText}>Cached data and local features remain available.</span>
            </div>
          </div>
        </OfflineOnly>
    </DashboardCard>
  );
}

export default memo(SystemStatusCard);

