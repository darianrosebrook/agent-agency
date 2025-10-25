/**
 * VerdictCard Component
 * Displays a summary of a council verdict with key metrics and status
 *
 * @author @darianrosebrook
 */

'use client';

import { Text } from '@/design-system/primitives';
import { Badge } from '@/design-system/primitives';
import {
  Clock,
  CheckCircle,
  XCircle,
  AlertCircle,
  Users,
  Shield,
  ChevronRight,
  Calendar
} from 'lucide-react';
import { Verdict, VerdictStatus } from './VerdictList';
import styles from './VerdictCard.module.scss';

interface VerdictCardProps {
  verdict: Verdict;
  onClick: () => void;
  className?: string;
}

export function VerdictCard({ verdict, onClick, className }: VerdictCardProps) {
  // Status configuration
  const getStatusConfig = (status: VerdictStatus) => {
    switch (status) {
      case 'approved':
        return {
          icon: CheckCircle,
          color: 'success',
          label: 'Approved',
          bgColor: 'var(--color-success-light)',
        };
      case 'rejected':
        return {
          icon: XCircle,
          color: 'error',
          label: 'Rejected',
          bgColor: 'var(--color-error-light)',
        };
      case 'intervened':
        return {
          icon: AlertCircle,
          color: 'warning',
          label: 'Intervened',
          bgColor: 'var(--color-warning-light)',
        };
      default:
        return {
          icon: Clock,
          color: 'info',
          label: 'Pending',
          bgColor: 'var(--color-info-light)',
        };
    }
  };

  const statusConfig = getStatusConfig(verdict.status);
  const StatusIcon = statusConfig.icon;

  // Format relative time
  const formatRelativeTime = (date: Date) => {
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMinutes = Math.floor(diffMs / (1000 * 60));
    const diffHours = Math.floor(diffMinutes / 60);
    const diffDays = Math.floor(diffHours / 24);

    if (diffMinutes < 1) return 'Just now';
    if (diffMinutes < 60) return `${diffMinutes}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;

    return date.toLocaleDateString();
  };

  // Consensus score color
  const getConsensusColor = (score: number) => {
    if (score >= 0.8) return 'success';
    if (score >= 0.6) return 'warning';
    return 'error';
  };

  // Ethical concerns indicator
  const getEthicalIndicator = (concerns: number) => {
    if (concerns === 0) return null;
    if (concerns === 1) return '⚠️';
    if (concerns === 2) return '⚠️⚠️';
    return '🚨';
  };

  return (
    <div
      className={`${styles.card} ${className || ''}`}
      onClick={onClick}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          onClick();
        }
      }}
      aria-label={`Verdict for task ${verdict.taskId}: ${verdict.title}`}
    >
      {/* Status Badge */}
      <div className={styles.statusBadge}>
        <Badge variant={statusConfig.color as any} size="sm">
          <StatusIcon size={12} />
          <span>{statusConfig.label}</span>
        </Badge>
      </div>

      {/* Main Content */}
      <div className={styles.content}>
        {/* Title and Task ID */}
        <div className={styles.header}>
          <Text variant="h4" className={styles.title}>
            {verdict.title}
          </Text>
          <Text variant="paragraph-small" color="secondary" className={styles.taskId}>
            Task: {verdict.taskId}
          </Text>
        </div>

        {/* Summary */}
        <Text variant="paragraph-medium" color="secondary" className={styles.summary}>
          {verdict.summary}
        </Text>

        {/* Metrics Row */}
        <div className={styles.metrics}>
          <div className={styles.metric}>
            <Users size={14} className={styles.metricIcon} />
            <Text variant="paragraph-small">
              {verdict.judgeCount} judges
            </Text>
          </div>

          <div className={styles.metric}>
            <CheckCircle
              size={14}
              className={`${styles.metricIcon} ${styles[getConsensusColor(verdict.consensusScore)]}`}
            />
            <Text variant="paragraph-small">
              {Math.round(verdict.consensusScore * 100)}% consensus
            </Text>
          </div>

          {verdict.ethicalConcerns > 0 && (
            <div className={styles.metric}>
              <Shield size={14} className={styles.metricIcon} />
              <Text variant="paragraph-small">
                {getEthicalIndicator(verdict.ethicalConcerns)} {verdict.ethicalConcerns} concerns
              </Text>
            </div>
          )}

          <div className={styles.metric}>
            <Calendar size={14} className={styles.metricIcon} />
            <Text variant="paragraph-small">
              {formatRelativeTime(verdict.createdAt)}
            </Text>
          </div>
        </div>

        {/* Evidence Preview */}
        {verdict.evidence.length > 0 && (
          <div className={styles.evidence}>
            <Text variant="paragraph-small" color="secondary">
              Key evidence: {verdict.evidence.slice(0, 2).map(e => e.title).join(', ')}
              {verdict.evidence.length > 2 && ` +${verdict.evidence.length - 2} more`}
            </Text>
          </div>
        )}
      </div>

      {/* Action Indicator */}
      <div className={styles.action}>
        <ChevronRight size={16} className={styles.chevron} />
      </div>
    </div>
  );
}
