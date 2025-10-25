'use client';

/**
 * Dashboard Layout - Client Component
 * 
 * @author @darianrosebrook
 * 
 * Wraps dashboard content with unified header and breadcrumbs.
 * Maintains fixed layout to prevent CLS.
 */

import { ReactNode } from 'react';
import UnifiedHeader from './UnifiedHeader';
import Breadcrumbs from '@/components/ui/Breadcrumbs';
import styles from '@/app/page.module.scss';

interface DashboardLayoutProps {
  children: ReactNode;
}

export default function DashboardLayout({ children }: DashboardLayoutProps) {
  return (
    <section className={styles.page}>
      {/* Skip to main content link for keyboard navigation */}
      <a href="#main-content" className="skip-link">
        Skip to main content
      </a>
      <main role="main" aria-label="Dashboard" className={styles.mainContent}>
        <UnifiedHeader />
        <Breadcrumbs />
        {children} 
      </main>
    </section>
  );
}

