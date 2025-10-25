/**
 * JudgeMetricsDashboard Component
 * Dashboard for monitoring judge performance and analytics
 *
 * @author @darianrosebrook
 */

'use client';

import { Text } from '@/design-system/primitives';
import { Badge } from '@/design-system/primitives';
import { Progress } from '@/design-system/primitives';
import { Users, TrendingUp, Clock, Target, AlertCircle, CheckCircle } from 'lucide-react';
import styles from './JudgeMetricsDashboard.module.scss';

interface JudgeMetrics {
  id: string;
  name: string;
  type: 'ethical' | 'safety' | 'context' | 'compliance';
  totalVerdicts: number;
  accuracy: number;
  averageResponseTime: number;
  consensusRate: number;
  ethicalConcernsFlagged: number;
  lastActive: Date;
  status: 'active' | 'idle' | 'error';
}

interface SystemMetrics {
  totalJudges: number;
  activeJudges: number;
  averageAccuracy: number;
  averageResponseTime: number;
  totalVerdictsToday: number;
  consensusRate: number;
}

export function JudgeMetricsDashboard() {
  // Mock data - replace with real API calls
  const systemMetrics: SystemMetrics = {
    totalJudges: 8,
    activeJudges: 6,
    averageAccuracy: 94.2,
    averageResponseTime: 1.8,
    totalVerdictsToday: 247,
    consensusRate: 87.3,
  };

  const judgeMetrics: JudgeMetrics[] = [
    {
      id: 'judge-ethical-1',
      name: 'Ethical Judge Alpha',
      type: 'ethical',
      totalVerdicts: 1250,
      accuracy: 96.8,
      averageResponseTime: 1.2,
      consensusRate: 92.1,
      ethicalConcernsFlagged: 23,
      lastActive: new Date(Date.now() - 1000 * 60 * 5), // 5 minutes ago
      status: 'active',
    },
    {
      id: 'judge-safety-1',
      name: 'Safety Judge Beta',
      type: 'safety',
      totalVerdicts: 1180,
      accuracy: 95.4,
      averageResponseTime: 1.5,
      consensusRate: 89.7,
      ethicalConcernsFlagged: 18,
      lastActive: new Date(Date.now() - 1000 * 60 * 15), // 15 minutes ago
      status: 'active',
    },
    {
      id: 'judge-context-1',
      name: 'Context Judge Gamma',
      type: 'context',
      totalVerdicts: 1320,
      accuracy: 93.2,
      averageResponseTime: 2.1,
      consensusRate: 85.6,
      ethicalConcernsFlagged: 12,
      lastActive: new Date(Date.now() - 1000 * 60 * 30), // 30 minutes ago
      status: 'active',
    },
    {
      id: 'judge-compliance-1',
      name: 'Compliance Judge Delta',
      type: 'compliance',
      totalVerdicts: 980,
      accuracy: 97.1,
      averageResponseTime: 1.8,
      consensusRate: 94.2,
      ethicalConcernsFlagged: 8,
      lastActive: new Date(Date.now() - 1000 * 60 * 60), // 1 hour ago
      status: 'idle',
    },
  ];

  const getTypeConfig = (type: string) => {
    switch (type) {
      case 'ethical':
        return { color: 'primary', icon: '🛡️' };
      case 'safety':
        return { color: 'error', icon: '🚨' };
      case 'context':
        return { color: 'warning', icon: '📝' };
      case 'compliance':
        return { color: 'success', icon: '⚖️' };
      default:
        return { color: 'secondary', icon: '🤖' };
    }
  };

  const getStatusConfig = (status: string) => {
    switch (status) {
      case 'active':
        return { color: 'success', icon: CheckCircle, label: 'Active' };
      case 'idle':
        return { color: 'warning', icon: Clock, label: 'Idle' };
      case 'error':
        return { color: 'error', icon: AlertCircle, label: 'Error' };
      default:
        return { color: 'secondary', icon: Clock, label: 'Unknown' };
    }
  };

  const getAccuracyColor = (accuracy: number) => {
    if (accuracy >= 95) return 'success';
    if (accuracy >= 90) return 'warning';
    return 'error';
  };

  const getResponseTimeColor = (time: number) => {
    if (time <= 2) return 'success';
    if (time <= 3) return 'warning';
    return 'error';
  };

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <Text variant="h3">Judge Performance Metrics</Text>
        <Text variant="paragraph-small" color="secondary">
          Monitor AI judge accuracy, performance, and reliability
        </Text>
      </div>

      {/* System Overview */}
      <div className={styles.systemOverview}>
        <div className={styles.overviewGrid}>
          <div className={styles.overviewCard}>
            <Users className={styles.overviewIcon} />
            <div className={styles.overviewContent}>
              <Text variant="h4" className={styles.overviewValue}>
                {systemMetrics.activeJudges}/{systemMetrics.totalJudges}
              </Text>
              <Text variant="paragraph-small" color="secondary">
                Active Judges
              </Text>
            </div>
          </div>

          <div className={styles.overviewCard}>
            <Target className={styles.overviewIcon} />
            <div className={styles.overviewContent}>
              <Text variant="h4" className={styles.overviewValue}>
                {systemMetrics.averageAccuracy}%
              </Text>
              <Text variant="paragraph-small" color="secondary">
                Average Accuracy
              </Text>
            </div>
          </div>

          <div className={styles.overviewCard}>
            <Clock className={styles.overviewIcon} />
            <div className={styles.overviewContent}>
              <Text variant="h4" className={styles.overviewValue}>
                {systemMetrics.averageResponseTime}s
              </Text>
              <Text variant="paragraph-small" color="secondary">
                Avg Response Time
              </Text>
            </div>
          </div>

          <div className={styles.overviewCard}>
            <TrendingUp className={styles.overviewIcon} />
            <div className={styles.overviewContent}>
              <Text variant="h4" className={styles.overviewValue}>
                {systemMetrics.consensusRate}%
              </Text>
              <Text variant="paragraph-small" color="secondary">
                Consensus Rate
              </Text>
            </div>
          </div>
        </div>
      </div>

      {/* Judge Performance Table */}
      <div className={styles.judgesSection}>
        <Text variant="h4" className={styles.sectionTitle}>Individual Judge Performance</Text>

        <div className={styles.judgesTable}>
          <div className={styles.tableHeader}>
            <div className={styles.colName}>Judge</div>
            <div className={styles.colType}>Type</div>
            <div className={styles.colAccuracy}>Accuracy</div>
            <div className={styles.colResponseTime}>Response Time</div>
            <div className={styles.colConsensus}>Consensus</div>
            <div className={styles.colVerdicts}>Verdicts</div>
            <div className={styles.colStatus}>Status</div>
          </div>

          {judgeMetrics.map((judge) => {
            const typeConfig = getTypeConfig(judge.type);
            const statusConfig = getStatusConfig(judge.status);
            const StatusIcon = statusConfig.icon;

            return (
              <div key={judge.id} className={styles.tableRow}>
                <div className={styles.colName}>
                  <div className={styles.judgeInfo}>
                    <Text variant="h5" className={styles.judgeName}>
                      {judge.name}
                    </Text>
                    <Text variant="paragraph-small" color="secondary">
                      Last active: {judge.lastActive.toLocaleString()}
                    </Text>
                  </div>
                </div>

                <div className={styles.colType}>
                  <Badge variant={typeConfig.color as any} size="sm">
                    <span className={styles.typeIcon}>{typeConfig.icon}</span>
                    <span>{judge.type}</span>
                  </Badge>
                </div>

                <div className={styles.colAccuracy}>
                  <div className={styles.metricDisplay}>
                    <Text variant="paragraph-medium" className={styles.metricValue}>
                      {judge.accuracy}%
                    </Text>
                    <Progress
                      value={judge.accuracy}
                      size="sm"
                      variant={getAccuracyColor(judge.accuracy) as any}
                      className={styles.metricBar}
                    />
                  </div>
                </div>

                <div className={styles.colResponseTime}>
                  <div className={styles.metricDisplay}>
                    <Text variant="paragraph-medium" className={styles.metricValue}>
                      {judge.averageResponseTime}s
                    </Text>
                    <div className={`${styles.timeIndicator} ${styles[getResponseTimeColor(judge.averageResponseTime)]}`} />
                  </div>
                </div>

                <div className={styles.colConsensus}>
                  <Text variant="paragraph-medium">
                    {judge.consensusRate}%
                  </Text>
                </div>

                <div className={styles.colVerdicts}>
                  <Text variant="paragraph-medium">
                    {judge.totalVerdicts.toLocaleString()}
                  </Text>
                  <Text variant="paragraph-small" color="secondary">
                    {judge.ethicalConcernsFlagged} concerns
                  </Text>
                </div>

                <div className={styles.colStatus}>
                  <Badge variant={statusConfig.color as any} size="sm">
                    <StatusIcon size={12} />
                    <span>{statusConfig.label}</span>
                  </Badge>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Performance Trends Placeholder */}
      <div className={styles.placeholder}>
        <TrendingUp size={48} className={styles.placeholderIcon} />
        <Text variant="h4">Performance Trend Analysis</Text>
        <Text variant="paragraph-medium" color="secondary">
          Historical performance trends and predictive analytics coming soon
        </Text>
      </div>
    </div>
  );
}
