/**
 * EthicalDashboard Component
 * Dashboard for monitoring ethical concerns and assessments
 *
 * @author @darianrosebrook
 */

'use client';

import { Text } from '@/design-system/primitives';
import { Badge } from '@/design-system/primitives';
import { Shield, AlertTriangle, CheckCircle, Clock, TrendingUp } from 'lucide-react';
import styles from './EthicalDashboard.module.scss';

interface EthicalConcern {
  id: string;
  title: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  category: string;
  description: string;
  status: 'active' | 'resolved' | 'escalated';
  reportedAt: Date;
  resolvedAt?: Date;
  judgeId?: string;
}

interface EthicalMetrics {
  totalConcerns: number;
  activeConcerns: number;
  resolvedToday: number;
  criticalConcerns: number;
  averageResolutionTime: number;
  topCategories: Array<{ category: string; count: number }>;
}

export function EthicalDashboard() {
  // Mock data - replace with real API calls
  const metrics: EthicalMetrics = {
    totalConcerns: 47,
    activeConcerns: 8,
    resolvedToday: 3,
    criticalConcerns: 1,
    averageResolutionTime: 4.2,
    topCategories: [
      { category: 'Privacy', count: 18 },
      { category: 'Bias', count: 12 },
      { category: 'Safety', count: 9 },
      { category: 'Transparency', count: 8 },
    ],
  };

  const recentConcerns: EthicalConcern[] = [
    {
      id: 'eth-001',
      title: 'Potential privacy violation in user data processing',
      severity: 'high',
      category: 'Privacy',
      description: 'Automated analysis detected potential PII exposure in processing pipeline',
      status: 'active',
      reportedAt: new Date(Date.now() - 1000 * 60 * 45), // 45 minutes ago
      judgeId: 'judge-privacy-1',
    },
    {
      id: 'eth-002',
      title: 'Algorithmic bias in content recommendation',
      severity: 'medium',
      category: 'Bias',
      description: 'Statistical analysis shows demographic bias in recommendation algorithm',
      status: 'active',
      reportedAt: new Date(Date.now() - 1000 * 60 * 120), // 2 hours ago
      judgeId: 'judge-bias-1',
    },
    {
      id: 'eth-003',
      title: 'Unclear decision rationale in automated moderation',
      severity: 'low',
      category: 'Transparency',
      description: 'Content moderation decision lacks sufficient explanation for human review',
      status: 'resolved',
      reportedAt: new Date(Date.now() - 1000 * 60 * 240), // 4 hours ago
      resolvedAt: new Date(Date.now() - 1000 * 60 * 30), // 30 minutes ago
    },
  ];

  const getSeverityConfig = (severity: string) => {
    switch (severity) {
      case 'critical':
        return { color: 'error', icon: AlertTriangle };
      case 'high':
        return { color: 'error', icon: AlertTriangle };
      case 'medium':
        return { color: 'warning', icon: AlertTriangle };
      default:
        return { color: 'info', icon: Clock };
    }
  };

  const getStatusConfig = (status: string) => {
    switch (status) {
      case 'resolved':
        return { color: 'success', icon: CheckCircle, label: 'Resolved' };
      case 'escalated':
        return { color: 'error', icon: AlertTriangle, label: 'Escalated' };
      default:
        return { color: 'warning', icon: Clock, label: 'Active' };
    }
  };

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <Text variant="h3">Ethical Assessment Dashboard</Text>
        <Text variant="paragraph-small" color="secondary">
          Monitor and manage ethical concerns in AI decision-making
        </Text>
      </div>

      {/* Metrics Overview */}
      <div className={styles.metricsGrid}>
        <div className={styles.metricCard}>
          <div className={styles.metricHeader}>
            <Shield className={styles.metricIcon} />
            <Text variant="paragraph-small" color="secondary">Total Concerns</Text>
          </div>
          <Text variant="h3" className={styles.metricValue}>
            {metrics.totalConcerns}
          </Text>
        </div>

        <div className={styles.metricCard}>
          <div className={styles.metricHeader}>
            <AlertTriangle className={styles.metricIcon} />
            <Text variant="paragraph-small" color="secondary">Active</Text>
          </div>
          <Text variant="h3" className={styles.metricValue}>
            {metrics.activeConcerns}
          </Text>
        </div>

        <div className={styles.metricCard}>
          <div className={styles.metricHeader}>
            <CheckCircle className={styles.metricIcon} />
            <Text variant="paragraph-small" color="secondary">Resolved Today</Text>
          </div>
          <Text variant="h3" className={styles.metricValue}>
            {metrics.resolvedToday}
          </Text>
        </div>

        <div className={styles.metricCard}>
          <div className={styles.metricHeader}>
            <Clock className={styles.metricIcon} />
            <Text variant="paragraph-small" color="secondary">Avg Resolution</Text>
          </div>
          <Text variant="h3" className={styles.metricValue}>
            {metrics.averageResolutionTime}h
          </Text>
        </div>
      </div>

      {/* Top Categories */}
      <div className={styles.categoriesSection}>
        <Text variant="h4" className={styles.sectionTitle}>Top Ethical Concern Categories</Text>
        <div className={styles.categoriesGrid}>
          {metrics.topCategories.map((category, index) => (
            <div key={category.category} className={styles.categoryCard}>
              <div className={styles.categoryRank}>#{index + 1}</div>
              <div className={styles.categoryInfo}>
                <Text variant="h5" className={styles.categoryName}>
                  {category.category}
                </Text>
                <Text variant="paragraph-small" color="secondary">
                  {category.count} concerns
                </Text>
              </div>
              <div className={styles.categoryBar}>
                <div
                  className={styles.categoryProgress}
                  style={{ width: `${(category.count / metrics.totalConcerns) * 100}%` }}
                />
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Recent Concerns */}
      <div className={styles.concernsSection}>
        <Text variant="h4" className={styles.sectionTitle}>Recent Ethical Concerns</Text>
        <div className={styles.concernsList}>
          {recentConcerns.map((concern) => {
            const severityConfig = getSeverityConfig(concern.severity);
            const statusConfig = getStatusConfig(concern.status);
            const SeverityIcon = severityConfig.icon;
            const StatusIcon = statusConfig.icon;

            return (
              <div key={concern.id} className={styles.concernCard}>
                <div className={styles.concernHeader}>
                  <div className={styles.concernTitle}>
                    <Text variant="h5">{concern.title}</Text>
                    <div className={styles.concernBadges}>
                      <Badge variant={severityConfig.color as any} size="sm">
                        <SeverityIcon size={12} />
                        <span>{concern.severity}</span>
                      </Badge>
                      <Badge variant="secondary" size="sm">
                        {concern.category}
                      </Badge>
                      <Badge variant={statusConfig.color as any} size="sm">
                        <StatusIcon size={12} />
                        <span>{statusConfig.label}</span>
                      </Badge>
                    </div>
                  </div>
                  <div className={styles.concernMeta}>
                    <Text variant="paragraph-small" color="secondary">
                      {concern.reportedAt.toLocaleString()}
                    </Text>
                    {concern.judgeId && (
                      <Text variant="paragraph-small" color="secondary">
                        Judge: {concern.judgeId}
                      </Text>
                    )}
                  </div>
                </div>

                <div className={styles.concernDescription}>
                  <Text variant="paragraph-medium" color="secondary">
                    {concern.description}
                  </Text>
                </div>

                {concern.status === 'resolved' && concern.resolvedAt && (
                  <div className={styles.concernResolution}>
                    <Text variant="paragraph-small" color="success">
                      Resolved: {concern.resolvedAt.toLocaleString()}
                    </Text>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </div>

      {/* Coming Soon Placeholder */}
      <div className={styles.placeholder}>
        <TrendingUp size={48} className={styles.placeholderIcon} />
        <Text variant="h4">Ethical Trend Analysis</Text>
        <Text variant="paragraph-medium" color="secondary">
          Advanced analytics for ethical concern patterns and predictive insights coming soon
        </Text>
      </div>
    </div>
  );
}
