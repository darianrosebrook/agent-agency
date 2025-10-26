/**
 * Vector Database Page
 * Main page for vector database analytics and management
 *
 * @author @darianrosebrook
 */

'use client';

import { Suspense } from 'react';
import DashboardLayout from '@/components/shared/DashboardLayout';
import ConnectionBanner from '@/components/shared/ConnectionBanner';
import { VectorDatabaseDashboard } from '@/components/vector-database/VectorDatabaseDashboard';
import styles from './page.module.scss';

// Loading component for Suspense
const LoadingSkeleton = () => (
  <div className={styles.loadingSkeleton}>
    <div className={styles.spinner}></div>
    <p>Loading vector database dashboard...</p>
  </div>
);

export default function VectorDatabasePage() {
  return (
    <DashboardLayout>
      <ConnectionBanner />
      
      <Suspense fallback={<LoadingSkeleton />}>
        <VectorDatabaseDashboard />
      </Suspense>
    </DashboardLayout>
  );
}
