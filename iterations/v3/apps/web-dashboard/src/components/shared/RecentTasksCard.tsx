'use client';

/**
 * Recent Tasks Card - Client Component
 * 
 * @author @darianrosebrook
 * 
 * Displays recent tasks with offline support.
 * Maintains fixed height to prevent layout shift.
 */

import { memo } from 'react';
import { DashboardCard } from '@/design-system/composers';
import { Text, StatusBadge } from '@/design-system';
import { ConnectionAware } from '@/components/providers/ConnectionProvider';
import { useOfflineTasks } from '@/hooks/useOfflineData';
import styles from '@/app/page.module.scss';

function RecentTasksCard() {
  const { data: tasks, isLoading } = useOfflineTasks();

  return (
    <DashboardCard
      title="Recent Tasks"
      description="Latest task activity"
      isLoading={isLoading}
      className={styles.card}
    >
        {tasks && tasks.length > 0 ? (
          <div className={styles.taskList}>
            {tasks.slice(0, 5).map((task: any) => (
              <div key={task.id} className={styles.taskItem}>
                <Text variant="paragraph-medium" weight="medium">
                  {task.title}
                </Text>
                <StatusBadge 
                  status={task.status as any} 
                  size="sm"
                />
              </div>
            ))}
            <a href="/tasks" className={styles.viewAllLink}>
              <Text variant="paragraph-medium" weight="medium" color="primary">
                View all tasks →
              </Text>
            </a>
          </div>
        ) : (
          <div className={styles.emptyTasks}>
            <Text variant="paragraph-medium" color="secondary">
              No tasks available
            </Text>
            <ConnectionAware
              offline={
                <Text variant="paragraph-small" color="muted" style={{ marginTop: "var(--spacing-2)" }}>
                  Tasks will sync when connection is restored
                </Text>
              }
            />
          </div>
        )}
    </DashboardCard>
  );
}

export default memo(RecentTasksCard);

