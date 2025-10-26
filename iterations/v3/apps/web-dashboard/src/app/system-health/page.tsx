/**
 * System Health Page
 * Main system health monitoring dashboard page
 *
 * @author @darianrosebrook
 */

'use client';

import { Suspense } from 'react';
import DashboardLayout from '@/components/shared/DashboardLayout';
import ConnectionBanner from '@/components/shared/ConnectionBanner';
import { SystemHealthDashboard } from '@/components/system-health/SystemHealthDashboard';
import styles from './page.module.scss';

// Loading component for Suspense
const LoadingSkeleton = () => (
  <div className={styles.loadingSkeleton}>
    <div className={styles.spinner}></div>
    <p>Loading system health monitoring dashboard...</p>
  </div>
);

export default function SystemHealthPage() {
  return (
    <DashboardLayout>
      <ConnectionBanner />

      <Suspense fallback={<LoadingSkeleton />}>
        <SystemHealthDashboard />
      </Suspense>
    </DashboardLayout>
  );
}
