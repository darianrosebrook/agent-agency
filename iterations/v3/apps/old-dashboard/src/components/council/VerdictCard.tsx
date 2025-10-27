/**
 * Verdict Card
 * Compact display of council verdict information
 *
 * @author @darianrosebrook
 */

'use client';

import { Text } from '@/design-system/primitives';
import {
  Gavel,
  Clock,
  CheckCircle,
  XCircle,
  AlertTriangle,
  Users,
  Shield,
  Eye,
  MoreHorizontal
} from 'lucide-react';
import { Verdict } from '@/lib/council-api';
import styles from './VerdictCard.module.scss';

interface VerdictCardProps {
  verdict: Verdict;
  onClick?: () => void;
  compact?: boolean;
}

export function VerdictCard({ verdict, onClick, compact = false }: VerdictCardProps) {
  const getStatusConfig = (status: Verdict['status']) => {
    switch (status) {
      case 'completed':
        return {
          icon: <CheckCircle size={16} />,
          color: 'success',
          text: 'Completed'
        };
      case 'escalated':
        return {
          icon: <AlertTriangle size={16} />,
          color: 'error',
          text: 'Escalated'
        };
      case 'in_progress':
        return {
          icon: <Clock size={16} />,
          color: 'warning',
          text: 'In Progress'
        };
      case 'pending':
        return {
          icon: <Clock size={16} />,
          color: 'neutral',
          text: 'Pending'
        };
      case 'overridden':
        return {
          icon: <XCircle size={16} />,
          color: 'error',
          text: 'Overridden'
        };
      default:
        return {
          icon: <Gavel size={16} />,
          color: 'neutral',
          text: status
        };
    }
  };

  const getRiskConfig = (risk: Verdict['ethicalAssessment']['overallRisk']) => {
    switch (risk) {
      case 'critical':
        return { color: 'error', text: 'Critical' };
      case 'high':
        return { color: 'error', text: 'High' };
      case 'medium':
        return { color: 'warning', text: 'Medium' };
      case 'low':
        return { color: 'success', text: 'Low' };
      default:
        return { color: 'neutral', text: risk };
    }
  };

  const statusConfig = getStatusConfig(verdict.status);
  const riskConfig = getRiskConfig(verdict.ethicalAssessment.overallRisk);

  const formatDate = (date: Date) => {
    return new Intl.DateTimeFormat('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    }).format(new Date(date));
  };

  const cardClasses = [
    styles.verdictCard,
    compact && styles.compact,
    styles[`status${statusConfig.color}`],
    onClick && styles.clickable
  ].filter(Boolean).join(' ');

  return (
    <div className={cardClasses} onClick={onClick}>
      {/* Header */}
      <div className={styles.cardHeader}>
        <div className={styles.statusBadge}>
          {statusConfig.icon}
          <span>{statusConfig.text}</span>
        </div>

        <div className={styles.cardActions}>
          <button className={styles.actionButton}>
            <Eye size={14} />
          </button>
          <button className={styles.actionButton}>
            <MoreHorizontal size={14} />
          </button>
        </div>
      </div>

      {/* Main Content */}
      <div className={styles.cardContent}>
        <div className={styles.primaryInfo}>
          <Text variant="h4" className={styles.taskId}>
            {verdict.taskId}
          </Text>

          <div className={styles.decisionInfo}>
            <Text variant="paragraph-medium" className={styles.decision}>
              Decision: <strong>{verdict.consensus.finalDecision.toUpperCase()}</strong>
            </Text>

            {!compact && (
              <Text variant="paragraph-small" color="secondary">
                Confidence: {Math.round(verdict.consensus.confidence * 100)}%
              </Text>
            )}
          </div>
        </div>

        <div className={styles.secondaryInfo}>
          {/* Risk Level */}
          <div className={styles.riskBadge}>
            <Shield size={12} />
            <span className={styles[`risk${riskConfig.color}`]}>
              {riskConfig.text} Risk
            </span>
          </div>

          {/* Judge Count */}
          <div className={styles.judgeInfo}>
            <Users size={12} />
            <span>{verdict.judges.length} judges</span>
          </div>

          {/* Evidence Count */}
          {!compact && (
            <div className={styles.evidenceInfo}>
              <span>{verdict.evidence.length} evidence items</span>
            </div>
          )}
        </div>
      </div>

      {/* Footer */}
      <div className={styles.cardFooter}>
        <div className={styles.timestamp}>
          <Text variant="paragraph-small" color="secondary">
            Created: {formatDate(verdict.createdAt)}
          </Text>

          {verdict.completedAt && (
            <Text variant="paragraph-small" color="secondary">
              Completed: {formatDate(verdict.completedAt)}
            </Text>
          )}
        </div>

        {!compact && verdict.intervention && (
          <div className={styles.interventionBadge}>
            <AlertTriangle size={12} />
            <span>Intervention Required</span>
          </div>
        )}
      </div>
    </div>
  );
}