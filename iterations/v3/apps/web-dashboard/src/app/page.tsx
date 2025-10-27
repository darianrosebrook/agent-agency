/**
 * Dashboard Page - Next.js 16 Server Component
 * 
 * @author @darianrosebrook
 * 
 * Modernized dashboard using Server Components, Suspense boundaries,
 * and FlowPress design system. Optimized for minimal CLS.
 */

'use client';

import { Suspense, useEffect, useRef, lazy } from 'react';
import { gsap } from 'gsap';
import DashboardLayout from '@/components/shared/DashboardLayout';
import ConnectionBanner from '@/components/shared/ConnectionBanner';
import { OnlineOnly } from '@/components/providers/ConnectionProvider';
import { Text } from '@/design-system/primitives';
import { useStaggerAnimation } from '@/interactions';
import styles from './page.module.scss';

// Lazy load heavy components for better performance
const MetricsSection = lazy(() => import('@/components/shared/MetricsSection'));
const QuickActions = lazy(() => import('@/components/shared/QuickActions'));
const SystemStatusCard = lazy(() => import('@/components/shared/SystemStatusCard'));
const RecentTasksCard = lazy(() => import('@/components/shared/RecentTasksCard'));
const SLODashboard = lazy(() => import('@/components/monitoring/SLODashboard'));
const SLOAlertsDashboard = lazy(() => import('@/components/monitoring/SLOAlertsDashboard'));

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
        <div className={styles.spinner} aria-hidden="true" />
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
        <div className={styles.spinner} aria-hidden="true" />
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
        <div className={styles.spinner} aria-hidden="true" />
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

        {/* Connection Banner - only shown when offline */}
        <OnlineOnly>
          <ConnectionBanner />
        </OnlineOnly>

        {/* Metrics Section - Above the fold for immediate visibility */}
        <section ref={metricsRef} className={styles.metricsSection} aria-labelledby="metrics-heading">
          <Suspense fallback={<MetricsSkeleton />}>
            <MetricsSection />
          </Suspense>
        </section>

        {/* Service Level Objectives Dashboard */}
        <section ref={sloRef} className={styles.sloSection} aria-labelledby="slo-heading">
          <Suspense fallback={<SLOSkeleton />}>
            <SLODashboard />
          </Suspense>
        </section>

        {/* Card Grid - Staggered animation for engaging entry */}
        <section
          ref={cardsGridRef}
          className={styles.cardsGrid}
          aria-labelledby="cards-heading"
          role="region"
        >
          <Suspense fallback={<CardSkeleton />}>
            <QuickActions />
          </Suspense>

          <Suspense fallback={<CardSkeleton />}>
            <SystemStatusCard />
          </Suspense>

          <Suspense fallback={<CardSkeleton />}>
            <RecentTasksCard />
          </Suspense>

          <Suspense fallback={<CardSkeleton />}>
            <SLOAlertsDashboard />
          </Suspense>
        </section>
      </main>
    </DashboardLayout>
  );
}

