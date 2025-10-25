/**
 * Streaming Dashboard - Next.js 16 Streaming SSR
 * Demonstrates real-time updates with server-side streaming
 */

import { Suspense } from 'react';
import { getBusinessMetrics, getTasks, getAlerts } from '@/lib/server/data-fetchers';
import DashboardLayout from '@/components/shared/DashboardLayout';
import ConnectionBanner from '@/components/shared/ConnectionBanner';
import { Text } from '@/design-system/primitives';
import StreamingMetrics from '@/components/streaming/StreamingMetrics';
import styles from './page.module.scss';

// Streaming components with individual loading states
async function MetricsSection() {
  const metrics = await getBusinessMetrics();
  
  return (
    <div className={styles.metricsSection}>
      <Text variant="h2" color="primary">Real-time Metrics</Text>
      {metrics ? (
        <div className={styles.metricsGrid}>
          <div className={styles.metricCard}>
            <Text variant="h3" color="primary">{(metrics as any)?.total_tasks_created || 0}</Text>
            <Text variant="caption" color="secondary">Total Tasks</Text>
          </div>
          <div className={styles.metricCard}>
            <Text variant="h3" color="primary">{(metrics as any)?.tasks_completed_today || 0}</Text>
            <Text variant="caption" color="secondary">Completed Today</Text>
          </div>
          <div className={styles.metricCard}>
            <Text variant="h3" color="primary">{(metrics as any)?.active_sessions || 0}</Text>
            <Text variant="caption" color="secondary">Active Sessions</Text>
          </div>
        </div>
      ) : (
        <div className={styles.loading}>Loading metrics...</div>
      )}
    </div>
  );
}

async function TasksSection() {
  const tasks = await getTasks({ limit: 5 });
  
  return (
    <div className={styles.tasksSection}>
      <Text variant="h2" color="primary">Recent Tasks</Text>
      {tasks ? (
        <div className={styles.tasksList}>
          {(tasks as any)?.tasks?.slice(0, 5).map((task: any) => (
            <div key={task.id} className={styles.taskItem}>
              <Text variant="paragraph-medium" color="primary">{task.name || 'Unnamed Task'}</Text>
              <Text variant="caption" color="secondary">{task.status || 'Unknown'}</Text>
            </div>
          ))}
        </div>
      ) : (
        <div className={styles.loading}>Loading tasks...</div>
      )}
    </div>
  );
}

async function AlertsSection() {
  const alerts = await getAlerts({ limit: 3 });
  
  return (
    <div className={styles.alertsSection}>
      <Text variant="h2" color="primary">Active Alerts</Text>
      {alerts ? (
        <div className={styles.alertsList}>
          {(alerts as any)?.alerts?.slice(0, 3).map((alert: any) => (
            <div key={alert.id} className={styles.alertItem}>
              <Text variant="paragraph-medium" color="primary">{alert.message || 'No message'}</Text>
              <Text variant="caption" color="secondary">{alert.severity || 'Unknown'}</Text>
            </div>
          ))}
        </div>
      ) : (
        <div className={styles.loading}>Loading alerts...</div>
      )}
    </div>
  );
}

// Loading components
function MetricsSkeleton() {
  return (
    <div className={styles.metricsSection}>
      <div className={styles.skeletonHeader} />
      <div className={styles.skeletonGrid}>
        <div className={styles.skeletonCard} />
        <div className={styles.skeletonCard} />
        <div className={styles.skeletonCard} />
      </div>
    </div>
  );
}

function TasksSkeleton() {
  return (
    <div className={styles.tasksSection}>
      <div className={styles.skeletonHeader} />
      <div className={styles.skeletonList}>
        <div className={styles.skeletonItem} />
        <div className={styles.skeletonItem} />
        <div className={styles.skeletonItem} />
      </div>
    </div>
  );
}

function AlertsSkeleton() {
  return (
    <div className={styles.alertsSection}>
      <div className={styles.skeletonHeader} />
      <div className={styles.skeletonList}>
        <div className={styles.skeletonItem} />
        <div className={styles.skeletonItem} />
      </div>
    </div>
  );
}

export default function StreamingDashboardPage() {
  return (
    <DashboardLayout>
      <div className={styles.streamingDashboard}>
        <ConnectionBanner />
        
        {/* Real-time streaming component */}
        <div className={styles.streamingSection}>
          <Text variant="h1" color="primary">Streaming Dashboard</Text>
          <StreamingMetrics />
        </div>

        {/* Server-side rendered sections with streaming */}
        <div className={styles.contentGrid}>
          <Suspense fallback={<MetricsSkeleton />}>
            <MetricsSection />
          </Suspense>
          
          <Suspense fallback={<TasksSkeleton />}>
            <TasksSection />
          </Suspense>
          
          <Suspense fallback={<AlertsSkeleton />}>
            <AlertsSection />
          </Suspense>
        </div>
      </div>
    </DashboardLayout>
  );
}
