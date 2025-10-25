/**
 * Metrics Page - Next.js 16 Server Component
 * 
 * @author @darianrosebrook
 * 
 * Comprehensive metrics dashboard with real-time data visualization
 * and performance monitoring capabilities.
 */

'use client';

import { Suspense, useRef, lazy } from 'react';
import DashboardLayout from '@/components/shared/DashboardLayout';
import ConnectionBanner from '@/components/shared/ConnectionBanner';
import { OnlineOnly } from '@/components/providers/ConnectionProvider';
import { Text } from '@/design-system/primitives';
import styles from './page.module.scss';

// Lazy load heavy components for better performance
const MetricsSection = lazy(() => import('@/components/shared/MetricsSection'));
const SystemStatusCard = lazy(() => import('@/components/shared/SystemStatusCard'));
const RecentTasksCard = lazy(() => import('@/components/shared/RecentTasksCard'));
const QuickActions = lazy(() => import('@/components/shared/QuickActions'));

/**
 * Skeleton for metrics cards
 * Maintains exact dimensions to prevent CLS
 */
function MetricsCardSkeleton() {
  return (
    <div 
      className={styles.metricsCard} 
      style={{ 
        minHeight: '300px',
        height: '300px',
        maxHeight: '300px',
        contain: 'layout style paint',
      }} 
      role="status" 
      aria-live="polite" 
      aria-busy="true"
    >
      <div className={styles.loading}>
        <div className={styles.spinner} aria-hidden="true"></div>
        <span className="sr-only">Loading metrics...</span>
      </div>
    </div>
  );
}

/**
 * Skeleton for metrics section
 * Reserves space for metric tiles
 */
function MetricsSkeleton() {
  return (
    <div className={styles.metricsGrid}>
      {Array.from({ length: 6 }).map((_, i) => (
        <div key={i} className={styles.metricTile}>
          <div className={styles.skeletonMetric} />
        </div>
      ))}
    </div>
  );
}

/**
 * Main Metrics Page Component
 * Uses Suspense boundaries to prevent layout shifts
 */
export default function MetricsPage() {
  const headerRef = useRef<HTMLElement>(null);
  const metricsRef = useRef<HTMLElement>(null);
  const systemHealthRef = useRef<HTMLElement>(null);
  const performanceRef = useRef<HTMLElement>(null);
  const businessRef = useRef<HTMLElement>(null);

  return (
    <DashboardLayout>
      <main role="main" aria-label="Metrics Dashboard" className={styles.container}>
        {/* Page Header - Immediate fade in animation with bold typography */}
        <header ref={headerRef} className={styles.header}>
          <Text variant="display-2" align="center" className={styles.title} id="page-title">
            Metrics Dashboard
          </Text>
          <Text variant="paragraph-large" color="secondary" align="center" className={styles.subtitle}>
            Comprehensive system metrics and performance monitoring
          </Text>
        </header>

        {/* Connection Status - Client-only */}
        <section aria-label="Connection status" role="status">
          <ConnectionBanner />
        </section>

        {/* System Status */}
        <section ref={systemHealthRef} aria-labelledby="system-status-heading" role="region">
          <h2 id="system-status-heading" className="sr-only">System Status</h2>
          <Suspense fallback={<MetricsCardSkeleton />}>
            <div className={styles.systemStatusSection}>
              <OnlineOnly fallback={<SystemStatusCard />}>
                <SystemStatusCard />
              </OnlineOnly>
            </div>
          </Suspense>
        </section>

        {/* Recent Tasks */}
        <section ref={performanceRef} aria-labelledby="recent-tasks-heading" role="region">
          <h2 id="recent-tasks-heading" className="sr-only">Recent Tasks</h2>
          <Suspense fallback={<MetricsCardSkeleton />}>
            <div className={styles.recentTasksSection}>
              <OnlineOnly fallback={<RecentTasksCard />}>
                <RecentTasksCard />
              </OnlineOnly>
            </div>
          </Suspense>
        </section>

        {/* Quick Actions */}
        <section ref={businessRef} aria-labelledby="quick-actions-heading" role="region">
          <h2 id="quick-actions-heading" className="sr-only">Quick Actions</h2>
          <Suspense fallback={<MetricsCardSkeleton />}>
            <div className={styles.quickActionsSection}>
              <OnlineOnly fallback={<QuickActions />}>
                <QuickActions />
              </OnlineOnly>
            </div>
          </Suspense>
        </section>

        {/* Task Metrics with Suspense and animation */}
        <section ref={metricsRef} aria-labelledby="metrics-heading" role="region">
          <h2 id="metrics-heading" className="sr-only">Task Metrics</h2>
          <Suspense fallback={<MetricsSkeleton />}>
            <MetricsSection />
          </Suspense>
        </section>
      </main>
    </DashboardLayout>
  );
}
