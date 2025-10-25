'use client';

/**
 * Quick Actions Card - Client Component
 * 
 * @author @darianrosebrook
 * 
 * Provides quick navigation to key dashboard features.
 * Adapts based on connection status.
 */

import { memo } from 'react';
import { ClipboardList, MessageSquare, BarChart3, Settings } from 'lucide-react';
import { DashboardCard } from '@/design-system/composers';
import { Text } from '@/design-system/primitives';
import { ConnectionAware } from '@/components/providers/ConnectionProvider';
import styles from '@/app/page.module.scss';

function QuickActions() {
  return (
    <DashboardCard
      title="Quick Actions"
      description="Navigate to key features"
      className={styles.card}
    >
        <div className={styles.actions}>
          <ConnectionAware
            online={
              <>
                <a href="/tasks" className={styles.actionButton}>
                  <ClipboardList className={styles.actionIcon} size={20} />
                  <span className={styles.actionText}>View Tasks</span>
                </a>
                <a href="/chat" className={styles.actionButton}>
                  <MessageSquare className={styles.actionIcon} size={20} />
                  <span className={styles.actionText}>Start Chat</span>
                </a>
              </>
            }
            offline={
              <div className={styles.offlineActions}>
                <span className={styles.offlineNote}>Some features require connection:</span>
                <button disabled className={`${styles.actionButton} ${styles.disabled}`}>
                  <ClipboardList className={styles.actionIcon} size={20} />
                  <span className={styles.actionText}>View Tasks</span>
                </button>
                <button disabled className={`${styles.actionButton} ${styles.disabled}`}>
                  <MessageSquare className={styles.actionIcon} size={20} />
                  <span className={styles.actionText}>Start Chat</span>
                </button>
              </div>
            }
          />
          <a href="/metrics" className={styles.actionButton}>
            <BarChart3 className={styles.actionIcon} size={20} />
            <span className={styles.actionText}>View Metrics</span>
          </a>
          <a href="/settings" className={styles.actionButton}>
            <Settings className={styles.actionIcon} size={20} />
            <span className={styles.actionText}>Settings</span>
          </a>
        </div>
    </DashboardCard>
  );
}

export default memo(QuickActions);

