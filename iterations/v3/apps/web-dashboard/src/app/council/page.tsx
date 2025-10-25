/**
 * Council Oversight Dashboard
 * Main page for monitoring AI judge decision-making and ethical assessments
 *
 * @author @darianrosebrook
 */

'use client';

import { useState, useEffect, Suspense, useRef, lazy } from 'react';
import { gsap } from 'gsap';
import DashboardLayout from '@/components/shared/DashboardLayout';
import ConnectionBanner from '@/components/shared/ConnectionBanner';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import { RefreshCw, Shield, Scale, Users, AlertTriangle } from 'lucide-react';
import { useStaggerAnimation } from '@/interactions';
import styles from './page.module.scss';

// Lazy load heavy components for better performance
const VerdictList = lazy(() => import('@/components/council/VerdictList'));
const EthicalDashboard = lazy(() => import('@/components/council/EthicalDashboard'));
const JudgeMetricsDashboard = lazy(() => import('@/components/council/JudgeMetricsDashboard'));
const DecisionFlowDiagram = lazy(() => import('@/components/council/DecisionFlowDiagram'));

// Council status types
type CouncilTab = 'verdicts' | 'ethics' | 'judges' | 'decisions';

interface CouncilStats {
  totalVerdicts: number;
  pendingVerdicts: number;
  ethicalConcerns: number;
  activeJudges: number;
  interventions: number;
}

/**
 * Loading skeleton for stats cards
 */
function StatsSkeleton() {
  return (
    <div className={styles.statsGrid}>
      {[...Array(4)].map((_, i) => (
        <div key={i} className={styles.statCard}>
          <div className={styles.loading}>
            <div className={styles.skeletonIcon}></div>
            <div className={styles.skeletonText}></div>
            <div className={styles.skeletonNumber}></div>
          </div>
        </div>
      ))}
    </div>
  );
}

/**
 * Loading skeleton for main content
 */
function ContentSkeleton() {
  return (
    <div className={styles.content}>
      <div className={styles.loading}>
        <div className={styles.skeletonBlock}></div>
        <div className={styles.skeletonBlock}></div>
        <div className={styles.skeletonBlock}></div>
      </div>
    </div>
  );
}

export default function CouncilDashboard() {
  // State management
  const [activeTab, setActiveTab] = useState<CouncilTab>('verdicts');
  const [stats, setStats] = useState<CouncilStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [refreshing, setRefreshing] = useState(false);

  // Animation refs
  const headerRef = useRef<HTMLElement>(null);
  const statsRef = useRef<HTMLElement>(null);
  const contentRef = useRef<HTMLElement>(null);

  // GSAP animations
  useEffect(() => {
    const header = headerRef.current;
    if (!header) return;

    gsap.set(header, { opacity: 0, y: -20 });
    gsap.to(header, {
      opacity: 1,
      y: 0,
      duration: 0.6,
      delay: 0.1,
      ease: 'power3.out',
    });
  }, []);

  // Stats animation
  useEffect(() => {
    const statsElement = statsRef.current;
    if (!statsElement || !stats) return;

    gsap.set(statsElement, { opacity: 0, y: 20 });
    gsap.to(statsElement, {
      opacity: 1,
      y: 0,
      duration: 0.6,
      delay: 0.3,
      ease: 'power3.out',
    });
  }, [stats]);

  // Content animation
  const { ref: contentAnimationRef } = useStaggerAnimation<HTMLDivElement>({
    delay: 0.5,
    stagger: 0.1,
    duration: 0.5,
    type: 'slideUp',
  });

  // Fetch council statistics
  const fetchStats = async () => {
    try {
      // TODO: Replace with actual API call
      await new Promise(resolve => setTimeout(resolve, 1000));

      setStats({
        totalVerdicts: 2847,
        pendingVerdicts: 12,
        ethicalConcerns: 3,
        activeJudges: 8,
        interventions: 2,
      });
    } catch (err) {
      console.error('Failed to fetch council stats:', err);
      setError(err instanceof Error ? err.message : 'Failed to load council statistics');
    }
  };

  // Handle refresh
  const handleRefresh = async () => {
    setRefreshing(true);
    try {
      await fetchStats();
    } finally {
      setRefreshing(false);
    }
  };

  // Initial data load
  useEffect(() => {
    const loadData = async () => {
      setLoading(true);
      try {
        await fetchStats();
      } finally {
        setLoading(false);
      }
    };

    loadData();

    // Set up polling for real-time updates (every 30 seconds)
    const interval = setInterval(() => {
      fetchStats();
    }, 30000);

    return () => clearInterval(interval);
  }, []);

  // Tab configuration
  const tabs = [
    { id: 'verdicts' as CouncilTab, label: 'Verdicts', icon: Scale, count: stats?.pendingVerdicts },
    { id: 'ethics' as CouncilTab, label: 'Ethics', icon: Shield, count: stats?.ethicalConcerns },
    { id: 'judges' as CouncilTab, label: 'Judges', icon: Users, count: stats?.activeJudges },
    { id: 'decisions' as CouncilTab, label: 'Decisions', icon: AlertTriangle, count: stats?.interventions },
  ];

  // Render tab content
  const renderTabContent = () => {
    switch (activeTab) {
      case 'verdicts':
        return <VerdictList />;
      case 'ethics':
        return <EthicalDashboard />;
      case 'judges':
        return <JudgeMetricsDashboard />;
      case 'decisions':
        return <DecisionFlowDiagram />;
      default:
        return <VerdictList />;
    }
  };

  return (
    <DashboardLayout>
      <main role="main" aria-label="Council Dashboard" className={styles.container}>
        {/* Page Header */}
        <header ref={headerRef} className={styles.header}>
          <div className={styles.headerContent}>
            <div>
              <Text variant="display-3" className={styles.title} id="page-title">
                Council Oversight
              </Text>
              <Text variant="paragraph-large" color="secondary" className={styles.subtitle}>
                Monitor AI judge decision-making and ethical assessments
              </Text>
            </div>

            <div className={styles.headerActions}>
              <Button
                variant="secondary"
                size="sm"
                onClick={handleRefresh}
                disabled={refreshing}
                aria-label="Refresh council data"
              >
                <RefreshCw
                  size={16}
                  className={refreshing ? styles.spinning : ''}
                  aria-hidden="true"
                />
                <span>Refresh</span>
              </Button>
            </div>
          </div>
        </header>

        {/* Connection Status */}
        <section aria-label="Connection status" role="status">
          <ConnectionBanner />
        </section>

        {/* Error State */}
        {error && (
          <div role="alert" className={styles.error}>
            <Text variant="paragraph-medium" color="error">
              {error}
            </Text>
          </div>
        )}

        {/* Council Statistics */}
        <section
          ref={statsRef}
          aria-labelledby="stats-heading"
          role="region"
          className={styles.statsSection}
        >
          <h2 id="stats-heading" className="sr-only">Council Statistics</h2>
          {loading ? (
            <StatsSkeleton />
          ) : stats ? (
            <div className={styles.statsGrid}>
              <div className={styles.statCard}>
                <Scale size={24} className={styles.statIcon} />
                <div className={styles.statContent}>
                  <Text variant="h4" className={styles.statValue}>
                    {stats.totalVerdicts.toLocaleString()}
                  </Text>
                  <Text variant="paragraph-small" color="secondary">
                    Total Verdicts
                  </Text>
                </div>
              </div>

              <div className={styles.statCard}>
                <AlertTriangle size={24} className={styles.statIcon} />
                <div className={styles.statContent}>
                  <Text variant="h4" className={styles.statValue}>
                    {stats.pendingVerdicts}
                  </Text>
                  <Text variant="paragraph-small" color="secondary">
                    Pending Review
                  </Text>
                </div>
              </div>

              <div className={styles.statCard}>
                <Shield size={24} className={styles.statIcon} />
                <div className={styles.statContent}>
                  <Text variant="h4" className={styles.statValue}>
                    {stats.ethicalConcerns}
                  </Text>
                  <Text variant="paragraph-small" color="secondary">
                    Ethical Concerns
                  </Text>
                </div>
              </div>

              <div className={styles.statCard}>
                <Users size={24} className={styles.statIcon} />
                <div className={styles.statContent}>
                  <Text variant="h4" className={styles.statValue}>
                    {stats.activeJudges}
                  </Text>
                  <Text variant="paragraph-small" color="secondary">
                    Active Judges
                  </Text>
                </div>
              </div>
            </div>
          ) : null}
        </section>

        {/* Tab Navigation */}
        <nav aria-label="Council sections" className={styles.tabNavigation}>
          <div className={styles.tabs} role="tablist">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                role="tab"
                aria-selected={activeTab === tab.id}
                aria-controls={`tabpanel-${tab.id}`}
                onClick={() => setActiveTab(tab.id)}
                className={`${styles.tab} ${activeTab === tab.id ? styles.active : ''}`}
              >
                <tab.icon size={18} aria-hidden="true" />
                <span>{tab.label}</span>
                {tab.count !== undefined && tab.count > 0 && (
                  <span className={styles.tabCount} aria-label={`${tab.count} items`}>
                    {tab.count}
                  </span>
                )}
              </button>
            ))}
          </div>
        </nav>

        {/* Main Content */}
        <section
          ref={contentAnimationRef}
          id={`tabpanel-${activeTab}`}
          role="tabpanel"
          aria-labelledby={`tab-${activeTab}`}
          className={styles.mainContent}
        >
          <Suspense fallback={<ContentSkeleton />}>
            {renderTabContent()}
          </Suspense>
        </section>
      </main>
    </DashboardLayout>
  );
}
