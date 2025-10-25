/**
 * Dashboard Page - Next.js 16 Server Component
 * 
 * @author @darianrosebrook
 * 
 * Modernized dashboard using Server Components, Suspense boundaries,
 * and FlowPress design system. Optimized for minimal CLS.
 */

'use client';

import { Suspense, useEffect, useRef } from 'react';
import { gsap } from 'gsap';
import DashboardLayout from '@/components/shared/DashboardLayout';
import ConnectionBanner from '@/components/shared/ConnectionBanner';
import MetricsSection from '@/components/shared/MetricsSection';
import QuickActions from '@/components/shared/QuickActions';
import SystemStatusCard from '@/components/shared/SystemStatusCard';
import RecentTasksCard from '@/components/shared/RecentTasksCard';
import SLODashboard from '@/components/monitoring/SLODashboard';
import SLOAlertsDashboard from '@/components/monitoring/SLOAlertsDashboard';
import { OnlineOnly } from '@/components/providers/ConnectionProvider';
import { Text } from '@/design-system/primitives';
import { useStaggerAnimation } from '@/interactions';
import styles from './page.module.scss';

// Development utilities (tree-shaken in production)
if (process.env.NODE_ENV === 'development') {
  import('@/utils/responsive-test').then(() => {
    // Layout testing utilities available in dev mode
  });
}

/**
 * Skeleton for dashboard cards
 * Maintains exact dimensions to prevent CLS
 */
function CardSkeleton() {
  return (
    <div 
      className={styles.card} 
      style={{ 
        minHeight: '200px',
        height: '200px',
        maxHeight: '200px',
        contain: 'layout style paint',
      }} 
      role="status" 
      aria-live="polite" 
      aria-busy="true"
    >
      <div className={styles.loading}>
        <div className={styles.spinner} aria-hidden="true"></div>
        <span className="sr-only">Loading card content...</span>
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
    <div 
      className={styles.metricsSection} 
      style={{ 
        minHeight: '400px',
        height: '400px',
        maxHeight: '400px',
        contain: 'layout style paint',
      }} 
      role="status" 
      aria-live="polite" 
      aria-busy="true"
    >
      <div className={styles.loading}>
        <div className={styles.spinner} aria-hidden="true"></div>
        <p aria-live="polite">Loading metrics...</p>
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
    <div 
      className={styles.sloSection} 
      style={{ 
        minHeight: '300px',
        height: '300px',
        maxHeight: '300px',
        contain: 'layout style paint',
      }} 
      role="status" 
      aria-live="polite" 
      aria-busy="true"
    >
      <div className={styles.loading}>
        <div className={styles.spinner} aria-hidden="true"></div>
        <p aria-live="polite">Loading service level objectives...</p>
      </div>
    </div>
  );
}

/**
 * Main Dashboard Page Component
 * Uses Suspense boundaries and GSAP animations
 */
export default function DashboardPage() {
  const headerRef = useRef<HTMLElement>(null);
  
  // Immediate fade-in animation for header (always visible at top)
  useEffect(() => {
    const header = headerRef.current;
    if (!header) return;

    // Set initial state
    gsap.set(header, { opacity: 0, y: -20 });

    // Animate in immediately
    gsap.to(header, {
      opacity: 1,
      y: 0,
      duration: 0.6,
      delay: 0.1,
      ease: 'power3.out',
    });
  }, []);

  const metricsRef = useRef<HTMLElement>(null);
  const sloRef = useRef<HTMLElement>(null);

  // Immediate animations for above-the-fold sections
  useEffect(() => {
    const metrics = metricsRef.current;
    const slo = sloRef.current;

    if (metrics) {
      gsap.set(metrics, { opacity: 0, y: 30 });
      gsap.to(metrics, {
        opacity: 1,
        y: 0,
        duration: 0.6,
        delay: 0.2,
        ease: 'power3.out',
      });
    }

    if (slo) {
      gsap.set(slo, { opacity: 0, y: 30 });
      gsap.to(slo, {
        opacity: 1,
        y: 0,
        duration: 0.6,
        delay: 0.3,
        ease: 'power3.out',
      });
    }
  }, []);
  
  // Stagger animation for card grid
  const { ref: cardsGridRef } = useStaggerAnimation<HTMLDivElement>({
    delay: 0.4,
    stagger: 0.1,
    duration: 0.5,
    type: 'slideUp',
  });

  return (
    <DashboardLayout>
      <main role="main" aria-label="Dashboard" className={styles.container}>
        {/* Page Header - Immediate fade in animation with bold typography */}
        <header ref={headerRef} className={styles.header}>
          <Text variant="display-2" align="center" className={styles.title} id="page-title">
            Dashboard
          </Text>
          <Text variant="paragraph-large" color="secondary" align="center" className={styles.subtitle}>
            Welcome to Agent Agency V3. Monitor task execution and system health.
          </Text>
        </header>

        {/* Connection Status - Client-only */}
        <section aria-label="Connection status" role="status">
          <ConnectionBanner />
        </section>

        {/* Metrics Section with Suspense and animation */}
        <section ref={metricsRef} aria-labelledby="metrics-heading" role="region">
          <h2 id="metrics-heading" className="sr-only">Task Metrics</h2>
          <Suspense fallback={<MetricsSkeleton />}>
            <MetricsSection />
          </Suspense>
        </section>

        {/* SLO Dashboard with Suspense and animation */}
        <section ref={sloRef} aria-labelledby="slo-heading" role="region">
          <h2 id="slo-heading" className="sr-only">Service Level Objectives</h2>
          <Suspense fallback={<SLOSkeleton />}>
            <div className={styles.sloSection}>
              <OnlineOnly fallback={<SLODashboard />}>
                <SLODashboard />
              </OnlineOnly>
            </div>
          </Suspense>
        </section>

        {/* Alerts Dashboard with Suspense */}
        <section aria-labelledby="alerts-heading" role="region">
          <h2 id="alerts-heading" className="sr-only">Active Alerts</h2>
          <Suspense fallback={<SLOSkeleton />}>
            <div className={styles.alertsSection}>
              <OnlineOnly fallback={<SLOAlertsDashboard />}>
                <SLOAlertsDashboard />
              </OnlineOnly>
            </div>
          </Suspense>
        </section>

        {/* Dashboard Cards Grid with stagger animation */}
        <section aria-labelledby="overview-heading" role="region">
          <h2 id="overview-heading" className="sr-only">Dashboard Overview</h2>
          <div ref={cardsGridRef} className={styles.content}>
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
        </section>
      </main>
    </DashboardLayout>
  );
}

