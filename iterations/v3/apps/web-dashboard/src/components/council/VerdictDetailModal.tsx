/**
 * VerdictDetailModal Component
 * Comprehensive modal displaying full verdict details, judge rationales, and evidence
 *
 * @author @darianrosebrook
 */

'use client';

import { useState, useEffect } from 'react';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import { Badge } from '@/design-system/primitives';
import { Progress } from '@/design-system/primitives';
import {
  X,
  Clock,
  CheckCircle,
  XCircle,
  AlertCircle,
  Users,
  Shield,
  FileText,
  BarChart3,
  AlertTriangle,
  ExternalLink,
  ChevronDown,
  ChevronUp
} from 'lucide-react';
import { Verdict } from './VerdictList';
import { InterventionForm } from './InterventionForm';
import styles from './VerdictDetailModal.module.scss';

interface VerdictDetailModalProps {
  verdict: Verdict;
  onClose: () => void;
  onIntervention?: (verdictId: string, intervention: any) => void;
}

export function VerdictDetailModal({
  verdict,
  onClose,
  onIntervention
}: VerdictDetailModalProps) {
  const [showInterventionForm, setShowInterventionForm] = useState(false);
  const [expandedSections, setExpandedSections] = useState<Set<string>>(
    new Set(['overview', 'judges'])
  );

  // Handle escape key
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    };

    document.addEventListener('keydown', handleEscape);
    return () => document.removeEventListener('keydown', handleEscape);
  }, [onClose]);

  // Handle backdrop click
  const handleBackdropClick = (e: React.MouseEvent) => {
    if (e.target === e.currentTarget) {
      onClose();
    }
  };

  // Toggle section expansion
  const toggleSection = (section: string) => {
    const newExpanded = new Set(expandedSections);
    if (newExpanded.has(section)) {
      newExpanded.delete(section);
    } else {
      newExpanded.add(section);
    }
    setExpandedSections(newExpanded);
  };

  // Status configuration
  const getStatusConfig = (status: string) => {
    switch (status) {
      case 'approved':
        return {
          icon: CheckCircle,
          color: 'success' as const,
          label: 'Approved',
          bgColor: 'var(--color-success-light)',
        };
      case 'rejected':
        return {
          icon: XCircle,
          color: 'error' as const,
          label: 'Rejected',
          bgColor: 'var(--color-error-light)',
        };
      case 'intervened':
        return {
          icon: AlertCircle,
          color: 'warning' as const,
          label: 'Intervened',
          bgColor: 'var(--color-warning-light)',
        };
      default:
        return {
          icon: Clock,
          color: 'info' as const,
          label: 'Pending',
          bgColor: 'var(--color-info-light)',
        };
    }
  };

  const statusConfig = getStatusConfig(verdict.status);
  const StatusIcon = statusConfig.icon;

  // Judge verdict summary
  const judgeSummary = verdict.judges.reduce(
    (acc, judge) => {
      acc[judge.verdict] = (acc[judge.verdict] || 0) + 1;
      return acc;
    },
    {} as Record<string, number>
  );

  // Consensus visualization
  const getConsensusLevel = (score: number) => {
    if (score >= 0.8) return { level: 'High', color: 'success' };
    if (score >= 0.6) return { level: 'Medium', color: 'warning' };
    return { level: 'Low', color: 'error' };
  };

  const consensusInfo = getConsensusLevel(verdict.consensusScore);

  // Format timestamps
  const formatDateTime = (date: Date) => {
    return new Intl.DateTimeFormat('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    }).format(date);
  };

  return (
    <div
      className={styles.overlay}
      onClick={handleBackdropClick}
      role="dialog"
      aria-modal="true"
      aria-labelledby="verdict-modal-title"
    >
      <div className={styles.modal}>
        {/* Header */}
        <div className={styles.header}>
          <div className={styles.titleSection}>
            <h2 id="verdict-modal-title" className={styles.title}>
              {verdict.title}
            </h2>
            <div className={styles.subtitle}>
              <Text variant="paragraph-small" color="secondary">
                Task ID: {verdict.taskId}
              </Text>
              <Badge variant={statusConfig.color} size="sm">
                <StatusIcon size={14} />
                <span>{statusConfig.label}</span>
              </Badge>
            </div>
          </div>

          <Button
            variant="ghost"
            size="sm"
            onClick={onClose}
            aria-label="Close modal"
            className={styles.closeButton}
          >
            <X size={20} />
          </Button>
        </div>

        {/* Content */}
        <div className={styles.content}>
          {/* Overview Section */}
          <section className={styles.section}>
            <button
              className={styles.sectionHeader}
              onClick={() => toggleSection('overview')}
              aria-expanded={expandedSections.has('overview')}
            >
              <div className={styles.sectionTitle}>
                <BarChart3 size={18} />
                <Text variant="h4">Overview</Text>
              </div>
              {expandedSections.has('overview') ? (
                <ChevronUp size={18} />
              ) : (
                <ChevronDown size={18} />
              )}
            </button>

            {expandedSections.has('overview') && (
              <div className={styles.sectionContent}>
                <div className={styles.overviewGrid}>
                  <div className={styles.overviewItem}>
                    <Text variant="paragraph-small" color="secondary">Summary</Text>
                    <Text variant="paragraph-medium">{verdict.summary}</Text>
                  </div>

                  <div className={styles.overviewItem}>
                    <Text variant="paragraph-small" color="secondary">Status</Text>
                    <Badge variant={statusConfig.color}>
                      <StatusIcon size={14} />
                      <span>{statusConfig.label}</span>
                    </Badge>
                  </div>

                  <div className={styles.overviewItem}>
                    <Text variant="paragraph-small" color="secondary">Consensus Score</Text>
                    <div className={styles.consensusDisplay}>
                      <Progress
                        value={verdict.consensusScore * 100}
                        className={styles.consensusBar}
                        variant={consensusInfo.color as any}
                      />
                      <Text variant="paragraph-small" className={styles.consensusText}>
                        {Math.round(verdict.consensusScore * 100)}% ({consensusInfo.level})
                      </Text>
                    </div>
                  </div>

                  <div className={styles.overviewItem}>
                    <Text variant="paragraph-small" color="secondary">Judges</Text>
                    <div className={styles.judgeSummary}>
                      <Users size={16} />
                      <Text variant="paragraph-small">{verdict.judgeCount} total</Text>
                    </div>
                  </div>

                  <div className={styles.overviewItem}>
                    <Text variant="paragraph-small" color="secondary">Created</Text>
                    <Text variant="paragraph-small">{formatDateTime(verdict.createdAt)}</Text>
                  </div>

                  <div className={styles.overviewItem}>
                    <Text variant="paragraph-small" color="secondary">Last Updated</Text>
                    <Text variant="paragraph-small">{formatDateTime(verdict.updatedAt)}</Text>
                  </div>

                  {verdict.ethicalConcerns > 0 && (
                    <div className={styles.overviewItem}>
                      <Text variant="paragraph-small" color="secondary">Ethical Concerns</Text>
                      <div className={styles.ethicalIndicator}>
                        <Shield size={16} />
                        <Text variant="paragraph-small">{verdict.ethicalConcerns} flagged</Text>
                      </div>
                    </div>
                  )}
                </div>
              </div>
            )}
          </section>

          {/* Judges Section */}
          <section className={styles.section}>
            <button
              className={styles.sectionHeader}
              onClick={() => toggleSection('judges')}
              aria-expanded={expandedSections.has('judges')}
            >
              <div className={styles.sectionTitle}>
                <Users size={18} />
                <Text variant="h4">Judge Analysis</Text>
                <Text variant="paragraph-small" color="secondary">
                  {verdict.judgeCount} judges participated
                </Text>
              </div>
              {expandedSections.has('judges') ? (
                <ChevronUp size={18} />
              ) : (
                <ChevronDown size={18} />
              )}
            </button>

            {expandedSections.has('judges') && (
              <div className={styles.sectionContent}>
                {/* Judge Summary */}
                <div className={styles.judgeSummaryGrid}>
                  {Object.entries(judgeSummary).map(([verdict, count]) => {
                    const config = getStatusConfig(verdict);
                    const Icon = config.icon;
                    return (
                      <div key={verdict} className={styles.judgeSummaryItem}>
                        <Icon size={16} className={styles[`statusIcon${verdict.charAt(0).toUpperCase() + verdict.slice(1)}`]} />
                        <Text variant="paragraph-small">{count} {verdict}</Text>
                      </div>
                    );
                  })}
                </div>

                {/* Individual Judges */}
                <div className={styles.judgesList}>
                  {verdict.judges.map((judge, index) => {
                    const judgeStatus = getStatusConfig(judge.verdict);
                    const JudgeIcon = judgeStatus.icon;

                    return (
                      <div key={`${judge.id}-${index}`} className={styles.judgeCard}>
                        <div className={styles.judgeHeader}>
                          <div className={styles.judgeInfo}>
                            <Text variant="h5" className={styles.judgeName}>
                              {judge.name}
                            </Text>
                            <Badge variant={judgeStatus.color as any} size="sm">
                              <JudgeIcon size={12} />
                              <span>{judgeStatus.label}</span>
                            </Badge>
                          </div>
                          <div className={styles.judgeConfidence}>
                            <Text variant="paragraph-small" color="secondary">
                              {Math.round(judge.confidence * 100)}% confidence
                            </Text>
                            <Progress
                              value={judge.confidence * 100}
                              size="sm"
                              variant={judge.confidence >= 0.8 ? 'success' : judge.confidence >= 0.6 ? 'warning' : 'error'}
                            />
                          </div>
                        </div>

                        <div className={styles.judgeReasoning}>
                          <Text variant="paragraph-medium" color="secondary">
                            {judge.reasoning}
                          </Text>
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            )}
          </section>

          {/* Evidence Section */}
          {verdict.evidence.length > 0 && (
            <section className={styles.section}>
              <button
                className={styles.sectionHeader}
                onClick={() => toggleSection('evidence')}
                aria-expanded={expandedSections.has('evidence')}
              >
                <div className={styles.sectionTitle}>
                  <FileText size={18} />
                  <Text variant="h4">Evidence</Text>
                  <Text variant="paragraph-small" color="secondary">
                    {verdict.evidence.length} items
                  </Text>
                </div>
                {expandedSections.has('evidence') ? (
                  <ChevronUp size={18} />
                ) : (
                  <ChevronDown size={18} />
                )}
              </button>

              {expandedSections.has('evidence') && (
                <div className={styles.sectionContent}>
                  <div className={styles.evidenceList}>
                    {verdict.evidence.map((evidence, index) => (
                      <div key={`${evidence.id}-${index}`} className={styles.evidenceItem}>
                        <div className={styles.evidenceHeader}>
                          <div className={styles.evidenceInfo}>
                            <Text variant="h5" className={styles.evidenceTitle}>
                              {evidence.title}
                            </Text>
                            <Badge variant="secondary" size="sm">
                              {evidence.type}
                            </Badge>
                          </div>
                          <div className={styles.evidenceMeta}>
                            <Text variant="paragraph-small" color="secondary">
                              Relevance: {Math.round(evidence.relevance * 100)}%
                            </Text>
                            <Button variant="ghost" size="sm" aria-label="View evidence">
                              <ExternalLink size={14} />
                            </Button>
                          </div>
                        </div>

                        <div className={styles.evidenceSource}>
                          <Text variant="paragraph-small" color="secondary">
                            Source: {evidence.source}
                          </Text>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </section>
          )}
        </div>

        {/* Footer */}
        <div className={styles.footer}>
          <div className={styles.footerLeft}>
            {verdict.status === 'pending' && (
              <Button
                variant="outline"
                size="sm"
                onClick={() => setShowInterventionForm(true)}
                className={styles.interventionButton}
              >
                <AlertTriangle size={16} />
                <span>Request Intervention</span>
              </Button>
            )}
          </div>

          <div className={styles.footerRight}>
            <Button variant="secondary" onClick={onClose}>
              Close
            </Button>
          </div>
        </div>

        {/* Intervention Form Modal */}
        {showInterventionForm && (
          <InterventionForm
            verdict={verdict}
            onSubmit={(intervention) => {
              onIntervention?.(verdict.id, intervention);
              setShowInterventionForm(false);
            }}
            onCancel={() => setShowInterventionForm(false)}
          />
        )}
      </div>
    </div>
  );
}
