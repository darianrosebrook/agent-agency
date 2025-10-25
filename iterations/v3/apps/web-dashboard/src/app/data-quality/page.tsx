/**
 * Data Quality Dashboard Page
 * Comprehensive database monitoring and data quality metrics
 * 
 * @author @darianrosebrook
 */

'use client';

import { useState, useEffect, Suspense, useRef, lazy } from 'react';
import DashboardLayout from '@/components/shared/DashboardLayout';
import ConnectionBanner from '@/components/shared/ConnectionBanner';
import { OnlineOnly } from '@/components/providers/ConnectionProvider';
import { Text } from '@/design-system/primitives';
import { Database, AlertTriangle } from 'lucide-react';
import styles from './page.module.scss';

// Lazy load heavy components for better performance
const DataQualityDashboard = lazy(() => import('@/components/database/DataQualityDashboard'));
const DatabaseConnections = lazy(() => import('@/components/database/DatabaseConnections'));
const DataIntegrityMonitor = lazy(() => import('@/components/database/DataIntegrityMonitor'));

/**
 * Loading skeleton for data quality cards
 */
function DataQualitySkeleton() {
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
        <span className="sr-only">Loading data quality metrics...</span>
      </div>
    </div>
  );
}

/**
 * Loading skeleton for database connections
 */
function DatabaseConnectionsSkeleton() {
  return (
    <div 
      className={styles.card}
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
        <span className="sr-only">Loading database connections...</span>
      </div>
    </div>
  );
}

/**
 * Main Data Quality Dashboard Page Component
 * Uses Suspense boundaries to prevent layout shifts
 */
export default function DataQualityPage() {
  const [metrics, setMetrics] = useState<any>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Animation refs
  const headerRef = useRef<HTMLElement>(null);
  const overviewRef = useRef<HTMLElement>(null);
  const connectionsRef = useRef<HTMLElement>(null);
  const integrityRef = useRef<HTMLElement>(null);

  // Enhanced animations will be applied via GSAP in useEffect

  // Mock data for demonstration
  useEffect(() => {
    const loadMockData = async () => {
      setLoading(true);
      try {
        // Simulate API call
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        setMetrics({
          dataQuality: {
            overallScore: 94.2,
            completeness: 98.5,
            accuracy: 96.8,
            consistency: 91.2,
            timeliness: 89.7,
            validity: 95.3
          },
          databaseHealth: {
            connections: 12,
            activeQueries: 8,
            avgResponseTime: 45,
            errorRate: 0.2,
            uptime: 99.8
          },
          dataIntegrity: {
            totalRecords: 1250000,
            validRecords: 1235000,
            duplicateRecords: 15000,
            orphanedRecords: 0,
            corruptedRecords: 0
          }
        });
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load data quality metrics');
      } finally {
        setLoading(false);
      }
    };

    loadMockData();
  }, []);

  return (
    <DashboardLayout>
      <main role="main" aria-label="Data Quality Dashboard" className={styles.container}>
        {/* Page Header - Enhanced with animations */}
        <header ref={headerRef} className={styles.header}>
          <div className={styles.headerContent}>
            <div className={styles.headerText}>
              <Text variant="display-2" className={styles.title} id="page-title">
                Data Quality Dashboard
              </Text>
              <Text variant="paragraph-large" color="secondary" className={styles.subtitle}>
                Monitor database health, data integrity, and quality metrics
              </Text>
            </div>
            
            <div className={styles.headerActions}>
              <div className={styles.statusIndicator}>
                <Database className={styles.statusIcon} />
                <Text variant="paragraph-medium" color="secondary">
                  Database Monitoring Active
                </Text>
              </div>
            </div>
          </div>
        </header>

        {/* Connection Status */}
        <section aria-label="Connection status" role="status">
          <ConnectionBanner />
        </section>

        {/* Data Quality Overview */}
        <section ref={overviewRef} aria-labelledby="overview-heading" role="region">
          <h2 id="overview-heading" className="sr-only">Data Quality Overview</h2>
          <Suspense fallback={<DataQualitySkeleton />}>
            <div className={styles.overviewSection}>
              <OnlineOnly fallback={<DataQualitySkeleton />}>
                <DataQualityDashboard
                  metrics={metrics}
                  isLoading={loading}
                  error={error}
                  onRefresh={() => {
                    console.log('Refreshing data quality metrics...');
                  }}
                />
              </OnlineOnly>
            </div>
          </Suspense>
        </section>

        {/* Database Connections */}
        <section ref={connectionsRef} aria-labelledby="connections-heading" role="region">
          <h2 id="connections-heading" className="sr-only">Database Connections</h2>
          <Suspense fallback={<DatabaseConnectionsSkeleton />}>
            <div className={styles.connectionsSection}>
              <OnlineOnly fallback={<DatabaseConnectionsSkeleton />}>
                <DatabaseConnections />
              </OnlineOnly>
            </div>
          </Suspense>
        </section>

        {/* Data Integrity Monitor */}
        <section ref={integrityRef} aria-labelledby="integrity-heading" role="region">
          <h2 id="integrity-heading" className="sr-only">Data Integrity Monitor</h2>
          <Suspense fallback={<DataQualitySkeleton />}>
            <div className={styles.integritySection}>
              <OnlineOnly fallback={<DataQualitySkeleton />}>
                <DataIntegrityMonitor />
              </OnlineOnly>
            </div>
          </Suspense>
        </section>

        {/* Error State */}
        {error && (
          <div role="alert" className={styles.error}>
            <AlertTriangle className={styles.errorIcon} />
            <Text variant="paragraph-medium" color="error">
              {error}
            </Text>
          </div>
        )}
      </main>
    </DashboardLayout>
  );
}
