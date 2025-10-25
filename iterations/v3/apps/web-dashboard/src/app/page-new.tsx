/**
 * Dashboard Page - Next.js 16 Server Component
 * 
 * @author @darianrosebrook
 * 
 * Modernized dashboard using Server Components, Suspense boundaries,
 * and FlowPress design system. Optimized for minimal CLS.
 */

import { Suspense } from 'react';
import DashboardLayout from '@/components/shared/DashboardLayout';
import ConnectionBanner from '@/components/shared/ConnectionBanner';
import MetricsSection from '@/components/shared/MetricsSection';
import QuickActions from '@/components/shared/QuickActions';
import SystemStatusCard from '@/components/shared/SystemStatusCard';
import RecentTasksCard from '@/components/shared/RecentTasksCard';
import SLODashboard from '@/components/monitoring/SLODashboard';
import SLOAlertsDashboard from '@/components/monitoring/SLOAlertsDashboard';
import { OnlineOnly } from '@/components/providers/ConnectionProvider';
import styles from './page.module.scss';

/**
 * Skeleton for dashboard cards
 * Maintains exact dimensions to prevent CLS
 */
function CardSkeleton() {
  return (
    <div className={styles.card} style={{ minHeight: '200px' }}>
      <div className={styles.loading}>
        <div className={styles.spinner}></div>
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
    <div className={styles.metricsSection} style={{ minHeight: '400px' }}>
      <div className={styles.loading}>
        <div className={styles.spinner}></div>
        <p>Loading metrics...</p>
      </div>
    </div>
  );
}

/**
 * Skeleton for SLO section
 * Maintains consistent height
 */
function SLOSkeleton() {
  return (
    <div className={styles.sloSection} style={{ minHeight: '300px' }}>
      <div className={styles.loading}>
        <div className={styles.spinner}></div>
        <p>Loading SLOs...</p>
      </div>
    </div>
  );
}

/**
 * Main Dashboard Page Component
 * Uses Suspense boundaries to prevent layout shifts
 */
export default function DashboardPage() {
  return (
    <DashboardLayout>
      <div className={styles.container}>
        {/* Page Header - Static, no shift */}
        <div className={styles.header}>
          <h1 className={styles.title}>Dashboard</h1>
          <p className={styles.subtitle}>
            Welcome to Agent Agency V3. Monitor task execution and system health.
          </p>
        </div>

        {/* Connection Status - Client-only */}
        <ConnectionBanner />

        {/* Metrics Section with Suspense */}
        <Suspense fallback={<MetricsSkeleton />}>
          <MetricsSection />
        </Suspense>

        {/* SLO Dashboard with Suspense */}
        <Suspense fallback={<SLOSkeleton />}>
          <div className={styles.sloSection}>
            <OnlineOnly fallback={<SLODashboard />}>
              <SLODashboard />
            </OnlineOnly>
          </div>
        </Suspense>

        {/* Alerts Dashboard with Suspense */}
        <Suspense fallback={<SLOSkeleton />}>
          <div className={styles.alertsSection}>
            <OnlineOnly fallback={<SLOAlertsDashboard />}>
              <SLOAlertsDashboard />
            </OnlineOnly>
          </div>
        </Suspense>

        {/* Dashboard Cards Grid */}
        <div className={styles.content}>
          <Suspense fallback={<CardSkeleton />}>
            <RecentTasksCard />
          </Suspense>

          <Suspense fallback={<CardSkeleton />}>
            <QuickActions />
          </Suspense>

          <Suspense fallback={<CardSkeleton />}>
            <SystemStatusCard />
          </Suspense>
        </div>
      </div>
    </DashboardLayout>
  );
}

