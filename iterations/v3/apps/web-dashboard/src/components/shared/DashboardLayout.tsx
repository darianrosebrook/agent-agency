'use client';

/**
 * Dashboard Layout - Client Component
 * 
 * @author @darianrosebrook
 * 
 * Wraps dashboard content with Header and Navigation.
 * Maintains fixed layout to prevent CLS.
 */

import { ReactNode } from 'react';
import Header from './Header';
import Navigation from './Navigation';
import styles from '@/app/page.module.scss';

interface DashboardLayoutProps {
  children: ReactNode;
}

export default function DashboardLayout({ children }: DashboardLayoutProps) {
  return (
    <div className={styles.page}>
      {/* Skip to main content link for keyboard navigation */}
      <a href="#main-content" className="skip-link">
        Skip to main content
      </a>
      <div className={styles.mainContent}>
        <Header />
        <Navigation />
        <div id="main-content">
          {children}
        </div>
      </div>
    </div>
  );
}

