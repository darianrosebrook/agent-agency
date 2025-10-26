/**
 * Agent Memory Management Page
 * Main agent memory management dashboard page
 *
 * @author @darianrosebrook
 */

'use client';

import { Suspense } from 'react';
import DashboardLayout from '@/components/shared/DashboardLayout';
import ConnectionBanner from '@/components/shared/ConnectionBanner';
import { AgentMemoryManagementDashboard } from '@/components/agent-memory/AgentMemoryManagementDashboard';
import styles from './page.module.scss';

// Loading component for Suspense
const LoadingSkeleton = () => (
  <div className={styles.loadingSkeleton}>
    <div className={styles.spinner} />
    <p>Loading agent memory management dashboard...</p>
  </div>
);

export default function AgentMemoryManagementPage() {
  return (
    <DashboardLayout>
      <ConnectionBanner />

      <Suspense fallback={<LoadingSkeleton />}>
        <AgentMemoryManagementDashboard />
      </Suspense>
    </DashboardLayout>
  );
}
