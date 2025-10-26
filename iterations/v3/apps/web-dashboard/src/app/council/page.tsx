/**
 * Council Page
 * Main council oversight dashboard page
 *
 * @author @darianrosebrook
 */

'use client';

import { Suspense } from 'react';
import DashboardLayout from '@/components/shared/DashboardLayout';
import ConnectionBanner from '@/components/shared/ConnectionBanner';
import { CouncilOversightDashboard } from '@/components/council/CouncilOversightDashboard';
import styles from './page.module.scss';

// Loading component for Suspense
const LoadingSkeleton = () => (
  <div className={styles.loadingSkeleton}>
    <div className={styles.spinner}></div>
    <p>Loading council oversight dashboard...</p>
  </div>
);

export default function CouncilPage() {
  return (
    <DashboardLayout>
      <ConnectionBanner />

      <Suspense fallback={<LoadingSkeleton />}>
        <CouncilOversightDashboard />
      </Suspense>
    </DashboardLayout>
  );
}