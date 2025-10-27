/**
 * Data Integrity Monitor Component
 * Monitors data integrity and quality metrics
 * 
 * @author @darianrosebrook
 */

"use client";

import { useState, useEffect } from "react";
import { Shield, AlertTriangle, CheckCircle, Database, TrendingUp } from "lucide-react";
import { Text } from "@/design-system/primitives";
import styles from "./DataIntegrityMonitor.module.scss";

interface IntegrityMetric {
  name: string;
  value: number;
  total: number;
  percentage: number;
  status: 'excellent' | 'good' | 'warning' | 'critical';
  trend: 'up' | 'down' | 'stable';
}

export default function DataIntegrityMonitor() {
  const [metrics, setMetrics] = useState<IntegrityMetric[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const loadMetrics = async () => {
      setLoading(true);
      try {
        // Mock data for demonstration
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        setMetrics([
          {
            name: 'Valid Records',
            value: 1235000,
            total: 1250000,
            percentage: 98.8,
            status: 'excellent',
            trend: 'up'
          },
          {
            name: 'Duplicate Records',
            value: 15000,
            total: 1250000,
            percentage: 1.2,
            status: 'good',
            trend: 'down'
          },
          {
            name: 'Orphaned Records',
            value: 0,
            total: 1250000,
            percentage: 0,
            status: 'excellent',
            trend: 'stable'
          },
          {
            name: 'Corrupted Records',
            value: 0,
            total: 1250000,
            percentage: 0,
            status: 'excellent',
            trend: 'stable'
          }
        ]);
      } catch (error) {
        console.error('Failed to load integrity metrics:', error);
      } finally {
        setLoading(false);
      }
    };

    loadMetrics();
  }, []);

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'excellent':
        return <CheckCircle className={styles.statusIcon} />;
      case 'good':
        return <CheckCircle className={styles.statusIcon} />;
      case 'warning':
        return <AlertTriangle className={styles.statusIcon} />;
      case 'critical':
        return <AlertTriangle className={styles.statusIcon} />;
      default:
        return <Database className={styles.statusIcon} />;
    }
  };

  const getStatusClass = (status: string) => {
    switch (status) {
      case 'excellent':
        return styles.excellent;
      case 'good':
        return styles.good;
      case 'warning':
        return styles.warning;
      case 'critical':
        return styles.critical;
      default:
        return styles.neutral;
    }
  };

  const getTrendIcon = (trend: string) => {
    switch (trend) {
      case 'up':
        return <TrendingUp className={styles.trendIcon} />;
      case 'down':
        return <TrendingUp className={`${styles.trendIcon} ${styles.trendDown}`} />;
      default:
        return null;
    }
  };

  if (loading) {
    return (
      <div className={styles.loading}>
        <div className={styles.spinner}></div>
        <Text variant="paragraph-medium" color="secondary">
          Loading data integrity metrics...
        </Text>
      </div>
    );
  }

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <Shield className={styles.headerIcon} />
        <Text variant="h3" className={styles.title}>
          Data Integrity Monitor
        </Text>
        <Text variant="paragraph-medium" color="secondary">
          Real-time data quality and integrity monitoring
        </Text>
      </div>

      <div className={styles.metricsGrid}>
        {metrics.map((metric, index) => (
          <div key={index} className={styles.metricCard}>
            <div className={styles.metricHeader}>
              <div className={styles.metricTitle}>
                {getStatusIcon(metric.status)}
                <Text variant="paragraph-medium" weight="medium">
                  {metric.name}
                </Text>
              </div>
              <div className={`${styles.statusIndicator} ${getStatusClass(metric.status)}`}>
                <Text variant="paragraph-small" weight="medium">
                  {metric.status.toUpperCase()}
                </Text>
              </div>
            </div>

            <div className={styles.metricContent}>
              <div className={styles.metricValue}>
                <Text variant="display-1" weight="semibold">
                  {metric.value.toLocaleString()}
                </Text>
                <Text variant="paragraph-small" color="secondary">
                  of {metric.total.toLocaleString()}
                </Text>
              </div>

              <div className={styles.metricPercentage}>
                <Text variant="paragraph-large" weight="medium">
                  {metric.percentage.toFixed(1)}%
                </Text>
                {getTrendIcon(metric.trend)}
              </div>
            </div>

            <div className={styles.progressBar}>
              <div 
                className={`${styles.progressFill} ${getStatusClass(metric.status)}`}
                style={{ width: `${metric.percentage}%` }}
              />
            </div>
          </div>
        ))}
      </div>

      <div className={styles.summary}>
        <div className={styles.summaryItem}>
          <Database className={styles.summaryIcon} />
          <div className={styles.summaryContent}>
            <Text variant="paragraph-medium" weight="medium">
              Total Records
            </Text>
            <Text variant="paragraph-large" weight="semibold">
              1,250,000
            </Text>
          </div>
        </div>
        <div className={styles.summaryItem}>
          <CheckCircle className={styles.summaryIcon} />
          <div className={styles.summaryContent}>
            <Text variant="paragraph-medium" weight="medium">
              Data Quality Score
            </Text>
            <Text variant="paragraph-large" weight="semibold">
              98.8%
            </Text>
          </div>
        </div>
      </div>
    </div>
  );
}
