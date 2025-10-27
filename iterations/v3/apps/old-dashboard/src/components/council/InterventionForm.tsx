/**
 * Intervention Form
 * Manual override and escalation interface for verdicts
 *
 * @author @darianrosebrook
 */

'use client';

import { useState } from 'react';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import {
  AlertTriangle,
  Shield,
  User,
  MessageSquare,
  Send,
  X,
  CheckCircle,
  Clock
} from 'lucide-react';
import { Verdict } from '@/lib/council-api';
import { councilApiClient } from '@/lib/council-api';
import { useCouncilActions } from '@/stores/council';
import styles from './InterventionForm.module.scss';

interface InterventionFormProps {
  verdict: Verdict;
  onClose: () => void;
}

export function InterventionForm({ verdict, onClose }: InterventionFormProps) {
  const [interventionType, setInterventionType] = useState<'override' | 'escalate'>('override');
  const [decision, setDecision] = useState<'approve' | 'reject' | 'escalate'>('approve');
  const [reason, setReason] = useState('');
  const [operator, setOperator] = useState('admin'); // In real app, get from auth context
  const [priority, setPriority] = useState<'low' | 'medium' | 'high' | 'critical'>('medium');
  const [additionalNotes, setAdditionalNotes] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const actions = useCouncilActions();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!reason.trim()) {
      setError('Reason is required');
      return;
    }

    setIsSubmitting(true);
    setError(null);

    try {
      if (interventionType === 'override') {
        const updatedVerdict = await councilApiClient.overrideVerdict(verdict.id, {
          decision,
          reason: reason.trim(),
          operator
        });
        actions.updateVerdict(verdict.id, updatedVerdict);
      } else {
        const updatedVerdict = await councilApiClient.escalateVerdict(verdict.id, {
          reason: reason.trim(),
          priority,
          operator
        });
        actions.updateVerdict(verdict.id, updatedVerdict);
      }

      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Intervention failed');
    } finally {
      setIsSubmitting(false);
    }
  };

  const getInterventionIcon = (type: 'override' | 'escalate') => {
    return type === 'override' ? <Shield size={20} /> : <AlertTriangle size={20} />;
  };

  const getDecisionIcon = (decision: 'approve' | 'reject' | 'escalate') => {
    switch (decision) {
      case 'approve':
        return <CheckCircle size={16} className={styles.decisionApprove} />;
      case 'reject':
        return <X size={16} className={styles.decisionReject} />;
      case 'escalate':
        return <AlertTriangle size={16} className={styles.decisionEscalate} />;
    }
  };

  return (
    <div className={styles.interventionForm}>
      {/* Header */}
      <div className={styles.formHeader}>
        <div className={styles.headerIcon}>
          {getInterventionIcon(interventionType)}
        </div>
        <div className={styles.headerContent}>
          <Text variant="h4">
            {interventionType === 'override' ? 'Manual Override' : 'Escalate for Review'}
          </Text>
          <Text variant="paragraph-medium" color="secondary">
            Verdict: {verdict.id} • Task: {verdict.taskId}
          </Text>
        </div>
      </div>

      {/* Intervention Type Selector */}
      <div className={styles.typeSelector}>
        <div className={styles.typeOptions}>
          <label className={styles.typeOption}>
            <input
              type="radio"
              name="interventionType"
              value="override"
              checked={interventionType === 'override'}
              onChange={(e) => setInterventionType(e.target.value as 'override')}
            />
            <div className={styles.typeContent}>
              <Shield size={16} />
              <div>
                <Text variant="paragraph-medium">Override Decision</Text>
                <Text variant="paragraph-small" color="secondary">
                  Manually set the final decision
                </Text>
              </div>
            </div>
          </label>

          <label className={styles.typeOption}>
            <input
              type="radio"
              name="interventionType"
              value="escalate"
              checked={interventionType === 'escalate'}
              onChange={(e) => setInterventionType(e.target.value as 'escalate')}
            />
            <div className={styles.typeContent}>
              <AlertTriangle size={16} />
              <div>
                <Text variant="paragraph-medium">Escalate for Review</Text>
                <Text variant="paragraph-small" color="secondary">
                  Flag for human review
                </Text>
              </div>
            </div>
          </label>
        </div>
      </div>

      {/* Form */}
      <form onSubmit={handleSubmit} className={styles.form}>
        {/* Decision Selection (for override) */}
        {interventionType === 'override' && (
          <div className={styles.formGroup}>
            <Text variant="label">Final Decision</Text>
            <div className={styles.decisionOptions}>
              {(['approve', 'reject', 'escalate'] as const).map((option) => (
                <label key={option} className={styles.decisionOption}>
                  <input
                    type="radio"
                    name="decision"
                    value={option}
                    checked={decision === option}
                    onChange={(e) => setDecision(e.target.value as typeof decision)}
                  />
                  <div className={styles.decisionContent}>
                    {getDecisionIcon(option)}
                    <span>{option.toUpperCase()}</span>
                  </div>
                </label>
              ))}
            </div>
          </div>
        )}

        {/* Priority Selection (for escalation) */}
        {interventionType === 'escalate' && (
          <div className={styles.formGroup}>
            <Text variant="label">Priority Level</Text>
            <select
              value={priority}
              onChange={(e) => setPriority(e.target.value as typeof priority)}
              className={styles.prioritySelect}
              required
            >
              <option value="low">Low - Standard review timeline</option>
              <option value="medium">Medium - Within 24 hours</option>
              <option value="high">High - Within 4 hours</option>
              <option value="critical">Critical - Immediate attention</option>
            </select>
          </div>
        )}

        {/* Reason */}
        <div className={styles.formGroup}>
          <Text variant="label">
            Reason for Intervention <span className={styles.required}>*</span>
          </Text>
          <textarea
            value={reason}
            onChange={(e) => setReason(e.target.value)}
            placeholder="Explain the reasoning for this intervention..."
            className={styles.reasonTextarea}
            rows={4}
            required
          />
          <Text variant="paragraph-small" color="secondary">
            This will be logged and may be reviewed for compliance purposes.
          </Text>
        </div>

        {/* Additional Notes */}
        <div className={styles.formGroup}>
          <Text variant="label">Additional Notes</Text>
          <textarea
            value={additionalNotes}
            onChange={(e) => setAdditionalNotes(e.target.value)}
            placeholder="Any additional context or notes..."
            className={styles.notesTextarea}
            rows={3}
          />
        </div>

        {/* Operator */}
        <div className={styles.formGroup}>
          <Text variant="label">Operator</Text>
          <div className={styles.operatorField}>
            <User size={16} />
            <input
              type="text"
              value={operator}
              onChange={(e) => setOperator(e.target.value)}
              className={styles.operatorInput}
              placeholder="Your identifier"
              required
            />
          </div>
        </div>

        {/* Error Display */}
        {error && (
          <div className={styles.errorMessage}>
            <AlertTriangle size={16} />
            <Text variant="paragraph-medium">{error}</Text>
          </div>
        )}

        {/* Actions */}
        <div className={styles.formActions}>
          <Button
            type="button"
            variant="secondary"
            onClick={onClose}
            disabled={isSubmitting}
          >
            Cancel
          </Button>
          <Button
            type="submit"
            variant={interventionType === 'override' ? 'danger' : 'warning'}
            disabled={isSubmitting}
          >
            {isSubmitting ? (
              <>
                <Clock size={16} className={styles.spinning} />
                Processing...
              </>
            ) : (
              <>
                <Send size={16} />
                {interventionType === 'override' ? 'Override Decision' : 'Escalate for Review'}
              </>
            )}
          </Button>
        </div>
      </form>

      {/* Warning */}
      <div className={styles.warning}>
        <AlertTriangle size={16} />
        <div>
          <Text variant="paragraph-medium">Intervention Notice</Text>
          <Text variant="paragraph-small" color="secondary">
            {interventionType === 'override'
              ? 'Manual overrides bypass the AI decision process and may impact system learning. Ensure proper documentation.'
              : 'Escalations will notify human reviewers and pause automated processing until reviewed.'
            }
          </Text>
        </div>
      </div>
    </div>
  );
}