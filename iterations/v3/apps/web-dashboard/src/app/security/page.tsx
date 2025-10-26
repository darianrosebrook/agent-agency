/**
 * Security Page
 * Main security monitoring and access control dashboard page
 *
 * @author @darianrosebrook
 */

'use client';

import { Suspense } from 'react';
import DashboardLayout from '@/components/shared/DashboardLayout';
import ConnectionBanner from '@/components/shared/ConnectionBanner';
import { SecurityDashboard } from '@/components/security/SecurityDashboard';
import styles from './page.module.scss';

// Loading component for Suspense
const LoadingSkeleton = () => (
  <div className={styles.loadingSkeleton}>
    <div className={styles.spinner}></div>
    <p>Loading security dashboard...</p>
  </div>
);

export default function SecurityPage() {
  return (
    <DashboardLayout>
      <ConnectionBanner />

      <Suspense fallback={<LoadingSkeleton />}>
        <SecurityDashboard />
      </Suspense>
    </DashboardLayout>
  );
}
