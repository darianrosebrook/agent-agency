/**
 * Workspace Management Page
 * Main workspace management dashboard page
 *
 * @author @darianrosebrook
 */

'use client';

import { Suspense } from 'react';
import DashboardLayout from '@/components/shared/DashboardLayout';
import ConnectionBanner from '@/components/shared/ConnectionBanner';
import { WorkspaceManagementDashboard } from '@/components/workspace/WorkspaceManagementDashboard';
import styles from './page.module.scss';

// Loading component for Suspense
const LoadingSkeleton = () => (
  <div className={styles.loadingSkeleton}>
    <div className={styles.spinner}></div>
    <p>Loading workspace management dashboard...</p>
  </div>
);

export default function WorkspaceManagementPage() {
  return (
    <DashboardLayout>
      <ConnectionBanner />

      <Suspense fallback={<LoadingSkeleton />}>
        <WorkspaceManagementDashboard />
      </Suspense>
    </DashboardLayout>
  );
}
