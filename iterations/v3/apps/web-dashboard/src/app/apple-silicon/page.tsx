/**
 * Apple Silicon Page
 * Main Apple Silicon performance monitoring dashboard page
 *
 * @author @darianrosebrook
 */

'use client';

import { Suspense } from 'react';
import DashboardLayout from '@/components/shared/DashboardLayout';
import ConnectionBanner from '@/components/shared/ConnectionBanner';
import { AppleSiliconDashboard } from '@/components/apple-silicon/AppleSiliconDashboard';
import styles from './page.module.scss';

// Loading component for Suspense
const LoadingSkeleton = () => (
  <div className={styles.loadingSkeleton}>
    <div className={styles.spinner}></div>
    <p>Loading Apple Silicon monitoring dashboard...</p>
  </div>
);

export default function AppleSiliconPage() {
  return (
    <DashboardLayout>
      <ConnectionBanner />

      <Suspense fallback={<LoadingSkeleton />}>
        <AppleSiliconDashboard />
      </Suspense>
    </DashboardLayout>
  );
}