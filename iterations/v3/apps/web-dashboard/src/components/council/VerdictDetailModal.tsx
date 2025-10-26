/**
 * Verdict Detail Modal
 * Comprehensive verdict information display
 *
 * @author @darianrosebrook
 */

'use client';

import { useState } from 'react';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import {
  X,
  Gavel,
  Users,
  Shield,
  FileText,
  Clock,
  CheckCircle,
  AlertTriangle,
  XCircle,
  Info,
  ChevronRight,
  Download,
  ExternalLink
} from 'lucide-react';
import { Verdict } from '@/lib/council-api';
import { EvidenceViewer } from './EvidenceViewer';
import { InterventionForm } from './InterventionForm';
import styles from './VerdictDetailModal.module.scss';

interface VerdictDetailModalProps {
  verdict: Verdict;
  onClose: () => void;
}

export function VerdictDetailModal({ verdict, onClose }: VerdictDetailModalProps) {
  const [activeTab, setActiveTab] = useState<'overview' | 'judges' | 'evidence' | 'ethics' | 'intervention'>('overview');

  const getStatusConfig = (status: Verdict['status']) => {
    switch (status) {
      case 'completed':
        return {
          icon: <CheckCircle size={20} className={styles.statusCompleted} />,
          color: 'success',
          text: 'Completed'
        };
      case 'escalated':
        return {
          icon: <AlertTriangle size={20} className={styles.statusEscalated} />,
          color: 'error',
          text: 'Escalated'
        };
      case 'in_progress':
        return {
          icon: <Clock size={20} className={styles.statusInProgress} />,
          color: 'warning',
          text: 'In Progress'
        };
      case 'pending':
        return {
          icon: <Clock size={20} className={styles.statusPending} />,
          color: 'neutral',
          text: 'Pending'
        };
      case 'overridden':
        return {
          icon: <XCircle size={20} className={styles.statusOverridden} />,
          color: 'error',
          text: 'Overridden'
        };
      default:
        return {
          icon: <Gavel size={20} />,
          color: 'neutral',
          text: status
        };
    }
  };

  const getRiskConfig = (risk: Verdict['ethicalAssessment']['overallRisk']) => {
    switch (risk) {
      case 'critical':
        return { color: 'error', text: 'Critical', severity: 4 };
      case 'high':
        return { color: 'error', text: 'High', severity: 3 };
      case 'medium':
        return { color: 'warning', text: 'Medium', severity: 2 };
      case 'low':
        return { color: 'success', text: 'Low', severity: 1 };
      default:
        return { color: 'neutral', text: risk, severity: 1 };
    }
  };

  const formatDate = (date: Date) => {
    return new Intl.DateTimeFormat('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    }).format(new Date(date));
  };

  const statusConfig = getStatusConfig(verdict.status);
  const riskConfig = getRiskConfig(verdict.ethicalAssessment.overallRisk);

  const tabs = [
    { id: 'overview', label: 'Overview', icon: <Info size={16} /> },
    { id: 'judges', label: 'Judges', icon: <Users size={16} /> },
    { id: 'evidence', label: 'Evidence', icon: <FileText size={16} /> },
    { id: 'ethics', label: 'Ethics', icon: <Shield size={16} /> },
    { id: 'intervention', label: 'Intervention', icon: <AlertTriangle size={16} />, show: verdict.status === 'escalated' || verdict.intervention }
  ].filter(tab => tab.show !== false);

  return (
    <div className={styles.modalOverlay} onClick={onClose}>
      <div className={styles.modalContent} onClick={(e) => e.stopPropagation()}>
        {/* Header */}
        <div className={styles.modalHeader}>
          <div className={styles.headerLeft}>
            <div className={styles.statusIndicator}>
              {statusConfig.icon}
            </div>
            <div className={styles.headerInfo}>
              <Text variant="h3">Verdict {verdict.id}</Text>
              <Text variant="paragraph-medium" color="secondary">
                Task: {verdict.taskId}
              </Text>
            </div>
          </div>

          <div className={styles.headerRight}>
            <div className={styles.riskIndicator}>
              <Shield size={16} />
              <span className={styles[`risk${riskConfig.color}`]}>
                {riskConfig.text} Risk
              </span>
            </div>

            <Button variant="secondary" size="sm">
              <Download size={16} />
              Export
            </Button>

            <button className={styles.closeButton} onClick={onClose}>
              <X size={20} />
            </button>
          </div>
        </div>

        {/* Tabs */}
        <div className={styles.modalTabs}>
          {tabs.map((tab) => (
            <button
              key={tab.id}
              className={`${styles.tabButton} ${activeTab === tab.id ? styles.active : ''}`}
              onClick={() => setActiveTab(tab.id as any)}
            >
              {tab.icon}
              <span>{tab.label}</span>
            </button>
          ))}
        </div>

        {/* Content */}
        <div className={styles.modalBody}>
          {activeTab === 'overview' && (
            <div className={styles.overviewTab}>
              {/* Key Metrics */}
              <div className={styles.metricsGrid}>
                <div className={styles.metricCard}>
                  <Text variant="label">Decision</Text>
                  <Text variant="h4">{verdict.consensus.finalDecision.toUpperCase()}</Text>
                  <Text variant="paragraph-small" color="secondary">
                    Confidence: {Math.round(verdict.consensus.confidence * 100)}%
                  </Text>
                </div>

                <div className={styles.metricCard}>
                  <Text variant="label">Algorithm</Text>
                  <Text variant="h4">{verdict.consensus.algorithm.replace('_', ' ').toUpperCase()}</Text>
                  <Text variant="paragraph-small" color="secondary">
                    {verdict.consensus.participatingJudges} judges participated
                  </Text>
                </div>

                <div className={styles.metricCard}>
                  <Text variant="label">Evidence</Text>
                  <Text variant="h4">{verdict.evidence.length}</Text>
                  <Text variant="paragraph-small" color="secondary">
                    Items reviewed
                  </Text>
                </div>

                <div className={styles.metricCard}>
                  <Text variant="label">Duration</Text>
                  <Text variant="h4">
                    {verdict.completedAt
                      ? `${Math.round((new Date(verdict.completedAt).getTime() - new Date(verdict.createdAt).getTime()) / 1000 / 60)}m`
                      : 'In Progress'
                    }
                  </Text>
                  <Text variant="paragraph-small" color="secondary">
                    Created: {formatDate(verdict.createdAt)}
                  </Text>
                </div>
              </div>

              {/* Consensus Rationale */}
              <div className={styles.rationaleSection}>
                <Text variant="h4">Consensus Rationale</Text>
                <div className={styles.rationaleContent}>
                  <Text variant="paragraph-medium">
                    {verdict.consensus.rationale}
                  </Text>
                </div>
              </div>

              {/* Timeline */}
              <div className={styles.timelineSection}>
                <Text variant="h4">Decision Timeline</Text>
                <div className={styles.timeline}>
                  <div className={styles.timelineItem}>
                    <div className={styles.timelineDot}></div>
                    <div className={styles.timelineContent}>
                      <Text variant="label">Created</Text>
                      <Text variant="paragraph-small">{formatDate(verdict.createdAt)}</Text>
                    </div>
                  </div>

                  {verdict.completedAt && (
                    <div className={styles.timelineItem}>
                      <div className={styles.timelineDot}></div>
                      <div className={styles.timelineContent}>
                        <Text variant="label">Completed</Text>
                        <Text variant="paragraph-small">{formatDate(verdict.completedAt)}</Text>
                      </div>
                    </div>
                  )}

                  {verdict.intervention && (
                    <div className={styles.timelineItem}>
                      <div className={styles.timelineDot}></div>
                      <div className={styles.timelineContent}>
                        <Text variant="label">Intervention</Text>
                        <Text variant="paragraph-small">{verdict.intervention.reason}</Text>
                        <Text variant="paragraph-small" color="secondary">
                          By: {verdict.intervention.operator}
                        </Text>
                      </div>
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}

          {activeTab === 'judges' && (
            <div className={styles.judgesTab}>
              <Text variant="h4">Judge Assignments & Verdicts</Text>
              <div className={styles.judgesList}>
                {verdict.judges.map((assignment, index) => (
                  <div key={assignment.judgeId} className={styles.judgeCard}>
                    <div className={styles.judgeHeader}>
                      <Text variant="h5">Judge {assignment.judgeId}</Text>
                      <div className={styles.judgeRole}>
                        {assignment.role.replace('_', ' ').toUpperCase()}
                      </div>
                    </div>

                    {assignment.verdict && (
                      <div className={styles.judgeVerdict}>
                        <div className={styles.verdictDecision}>
                          <Text variant="label">Decision</Text>
                          <Text variant="paragraph-medium">
                            {assignment.verdict.decision.toUpperCase()}
                          </Text>
                        </div>

                        <div className={styles.verdictConfidence}>
                          <Text variant="label">Confidence</Text>
                          <Text variant="paragraph-medium">
                            {Math.round(assignment.verdict.confidence * 100)}%
                          </Text>
                        </div>

                        <div className={styles.verdictRationale}>
                          <Text variant="label">Rationale</Text>
                          <Text variant="paragraph-small">
                            {assignment.verdict.rationale}
                          </Text>
                        </div>
                      </div>
                    )}

                    <div className={styles.judgeStatus}>
                      <Text variant="paragraph-small" color="secondary">
                        Status: {assignment.status.replace('_', ' ').toUpperCase()}
                      </Text>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {activeTab === 'evidence' && (
            <EvidenceViewer evidence={verdict.evidence} />
          )}

          {activeTab === 'ethics' && (
            <div className={styles.ethicsTab}>
              <Text variant="h4">Ethical Assessment</Text>

              {/* Risk Overview */}
              <div className={styles.riskOverview}>
                <div className={styles.riskScore}>
                  <Text variant="h3">{verdict.ethicalAssessment.overallRisk.toUpperCase()}</Text>
                  <Text variant="paragraph-medium" color="secondary">Overall Risk Level</Text>
                </div>

                <div className={styles.stakeholderImpact}>
                  <Text variant="label">Stakeholder Impact</Text>
                  <div className={styles.impactGrid}>
                    <div className={styles.impactItem}>
                      <Text variant="paragraph-medium">{verdict.ethicalAssessment.stakeholderImpact.individuals}</Text>
                      <Text variant="paragraph-small" color="secondary">Individuals</Text>
                    </div>
                    <div className={styles.impactItem}>
                      <Text variant="paragraph-medium">{verdict.ethicalAssessment.stakeholderImpact.organizations}</Text>
                      <Text variant="paragraph-small" color="secondary">Organizations</Text>
                    </div>
                    <div className={styles.impactItem}>
                      <Text variant="paragraph-medium">{verdict.ethicalAssessment.stakeholderImpact.society}</Text>
                      <Text variant="paragraph-small" color="secondary">Society</Text>
                    </div>
                  </div>
                </div>
              </div>

              {/* Concerns */}
              <div className={styles.concernsSection}>
                <Text variant="h5">Ethical Concerns</Text>
                <div className={styles.concernsList}>
                  {verdict.ethicalAssessment.concerns.map((concern, index) => (
                    <div key={index} className={styles.concernItem}>
                      <div className={styles.concernHeader}>
                        <Text variant="paragraph-medium">{concern.description}</Text>
                        <div className={styles.concernMeta}>
                          <span className={styles.concernCategory}>
                            {concern.category.toUpperCase()}
                          </span>
                          <span className={styles.concernSeverity}>
                            {concern.severity.toUpperCase()}
                          </span>
                        </div>
                      </div>

                      <div className={styles.concernDetails}>
                        <Text variant="paragraph-small" color="secondary">
                          Affected: {concern.affectedParties.join(', ')}
                        </Text>
                        {concern.mitigation && (
                          <Text variant="paragraph-small">
                            Mitigation: {concern.mitigation}
                          </Text>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Recommendations */}
              <div className={styles.recommendationsSection}>
                <Text variant="h5">Recommendations</Text>
                <ul className={styles.recommendationsList}>
                  {verdict.ethicalAssessment.recommendations.map((rec, index) => (
                    <li key={index}>
                      <Text variant="paragraph-medium">{rec}</Text>
                    </li>
                  ))}
                </ul>
              </div>
            </div>
          )}

          {activeTab === 'intervention' && (
            <InterventionForm verdict={verdict} onClose={onClose} />
          )}
        </div>
      </div>
    </div>
  );
}