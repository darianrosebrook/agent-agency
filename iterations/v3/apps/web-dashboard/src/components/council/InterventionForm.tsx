/**
 * InterventionForm Component
 * Form for requesting manual intervention in council verdicts
 *
 * @author @darianrosebrook
 */

'use client';

import { useState } from 'react';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import { Textarea } from '@/design-system/primitives';
import { Select } from '@/design-system/primitives';
import { Checkbox } from '@/design-system/primitives';
import { AlertTriangle, Clock, Users } from 'lucide-react';
import { Verdict } from './VerdictList';
import styles from './InterventionForm.module.scss';

interface InterventionFormProps {
  verdict: Verdict;
  onSubmit: (intervention: InterventionRequest) => void;
  onCancel: () => void;
}

export interface InterventionRequest {
  reason: string;
  priority: 'low' | 'medium' | 'high' | 'critical';
  requestedBy: string;
  reviewDeadline?: Date;
  additionalReviewers: string[];
  notes: string;
  overrideDecision?: 'approve' | 'reject';
  justification?: string;
}

const PRIORITY_OPTIONS = [
  { value: 'low', label: 'Low - Review within 24 hours' },
  { value: 'medium', label: 'Medium - Review within 4 hours' },
  { value: 'high', label: 'High - Review within 1 hour' },
  { value: 'critical', label: 'Critical - Immediate review required' },
];

const REVIEWER_OPTIONS = [
  'Senior AI Ethics Officer',
  'Chief Technology Officer',
  'Legal Counsel',
  'Product Manager',
  'External Auditor',
];

export function InterventionForm({ verdict, onSubmit, onCancel }: InterventionFormProps) {
  const [formData, setFormData] = useState<Partial<InterventionRequest>>({
    reason: '',
    priority: 'medium',
    requestedBy: '', // Would be populated from user context
    additionalReviewers: [],
    notes: '',
    reviewDeadline: undefined,
  });

  const [overrideRequested, setOverrideRequested] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [errors, setErrors] = useState<Record<string, string>>({});

  // Handle form field changes
  const handleFieldChange = (field: keyof InterventionRequest, value: any) => {
    setFormData(prev => ({ ...prev, [field]: value }));
    // Clear error when field is modified
    if (errors[field]) {
      setErrors(prev => ({ ...prev, [field]: undefined }));
    }
  };

  // Handle reviewer selection
  const handleReviewerToggle = (reviewer: string) => {
    setFormData(prev => ({
      ...prev,
      additionalReviewers: prev.additionalReviewers?.includes(reviewer)
        ? prev.additionalReviewers.filter(r => r !== reviewer)
        : [...(prev.additionalReviewers || []), reviewer]
    }));
  };

  // Validate form
  const validateForm = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!formData.reason?.trim()) {
      newErrors.reason = 'Intervention reason is required';
    }

    if (!formData.priority) {
      newErrors.priority = 'Priority level is required';
    }

    if (!formData.requestedBy?.trim()) {
      newErrors.requestedBy = 'Requester name is required';
    }

    if (overrideRequested && !formData.overrideDecision) {
      newErrors.overrideDecision = 'Override decision is required when override is requested';
    }

    if (overrideRequested && !formData.justification?.trim()) {
      newErrors.justification = 'Justification is required when override is requested';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  // Handle form submission
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!validateForm()) {
      return;
    }

    setIsSubmitting(true);
    try {
      const interventionRequest: InterventionRequest = {
        reason: formData.reason!,
        priority: formData.priority!,
        requestedBy: formData.requestedBy!,
        reviewDeadline: formData.reviewDeadline,
        additionalReviewers: formData.additionalReviewers || [],
        notes: formData.notes || '',
        ...(overrideRequested && {
          overrideDecision: formData.overrideDecision!,
          justification: formData.justification!,
        }),
      };

      onSubmit(interventionRequest);
    } catch (error) {
      console.error('Failed to submit intervention request:', error);
    } finally {
      setIsSubmitting(false);
    }
  };

  // Calculate review deadline based on priority
  const getReviewDeadline = (priority: string): Date => {
    const now = new Date();
    switch (priority) {
      case 'critical':
        return new Date(now.getTime() + 30 * 60 * 1000); // 30 minutes
      case 'high':
        return new Date(now.getTime() + 60 * 60 * 1000); // 1 hour
      case 'medium':
        return new Date(now.getTime() + 4 * 60 * 60 * 1000); // 4 hours
      case 'low':
      default:
        return new Date(now.getTime() + 24 * 60 * 60 * 1000); // 24 hours
    }
  };

  // Update deadline when priority changes
  const handlePriorityChange = (priority: string) => {
    handleFieldChange('priority', priority);
    handleFieldChange('reviewDeadline', getReviewDeadline(priority));
  };

  return (
    <div className={styles.overlay} role="dialog" aria-modal="true">
      <div className={styles.modal}>
        {/* Header */}
        <div className={styles.header}>
          <div className={styles.titleSection}>
            <AlertTriangle size={24} className={styles.warningIcon} />
            <div>
              <h2 className={styles.title}>Request Intervention</h2>
              <Text variant="paragraph-small" color="secondary">
                Manual review for verdict: {verdict.title}
              </Text>
            </div>
          </div>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className={styles.form}>
          {/* Reason */}
          <div className={styles.field}>
            <label htmlFor="reason" className={styles.label}>
              Reason for Intervention *
            </label>
            <Textarea
              id="reason"
              value={formData.reason || ''}
              onChange={(e) => handleFieldChange('reason', e.target.value)}
              placeholder="Explain why manual intervention is needed..."
              rows={3}
              className={errors.reason ? styles.error : ''}
            />
            {errors.reason && (
              <Text variant="paragraph-small" color="error" className={styles.errorText}>
                {errors.reason}
              </Text>
            )}
          </div>

          {/* Priority */}
          <div className={styles.field}>
            <label htmlFor="priority" className={styles.label}>
              Priority Level *
            </label>
            <Select
              id="priority"
              value={formData.priority || 'medium'}
              onChange={handlePriorityChange}
              options={PRIORITY_OPTIONS}
            />
            {formData.reviewDeadline && (
              <Text variant="paragraph-small" color="secondary" className={styles.deadlineText}>
                Review deadline: {formData.reviewDeadline.toLocaleString()}
              </Text>
            )}
          </div>

          {/* Requester */}
          <div className={styles.field}>
            <label htmlFor="requester" className={styles.label}>
              Requested By *
            </label>
            <input
              id="requester"
              type="text"
              value={formData.requestedBy || ''}
              onChange={(e) => handleFieldChange('requestedBy', e.target.value)}
              placeholder="Your name or identifier"
              className={`${styles.input} ${errors.requestedBy ? styles.error : ''}`}
            />
            {errors.requestedBy && (
              <Text variant="paragraph-small" color="error" className={styles.errorText}>
                {errors.requestedBy}
              </Text>
            )}
          </div>

          {/* Additional Reviewers */}
          <div className={styles.field}>
            <label className={styles.label}>Additional Reviewers</label>
            <Text variant="paragraph-small" color="secondary" className={styles.fieldHelp}>
              Select team members who should be notified for review
            </Text>
            <div className={styles.reviewerGrid}>
              {REVIEWER_OPTIONS.map((reviewer) => (
                <label key={reviewer} className={styles.reviewerOption}>
                  <Checkbox
                    checked={formData.additionalReviewers?.includes(reviewer) || false}
                    onChange={() => handleReviewerToggle(reviewer)}
                  />
                  <Text variant="paragraph-small">{reviewer}</Text>
                </label>
              ))}
            </div>
          </div>

          {/* Override Request */}
          <div className={styles.field}>
            <label className={styles.checkboxLabel}>
              <Checkbox
                checked={overrideRequested}
                onChange={setOverrideRequested}
              />
              <Text variant="paragraph-medium">Request immediate override</Text>
            </label>
            <Text variant="paragraph-small" color="secondary" className={styles.fieldHelp}>
              Check this if you need to immediately override the current verdict decision
            </Text>
          </div>

          {/* Override Details */}
          {overrideRequested && (
            <>
              <div className={styles.field}>
                <label htmlFor="overrideDecision" className={styles.label}>
                  Override Decision *
                </label>
                <Select
                  id="overrideDecision"
                  value={formData.overrideDecision || ''}
                  onChange={(value) => handleFieldChange('overrideDecision', value)}
                  options={[
                    { value: 'approve', label: 'Approve the action' },
                    { value: 'reject', label: 'Reject the action' },
                  ]}
                  className={errors.overrideDecision ? styles.error : ''}
                />
                {errors.overrideDecision && (
                  <Text variant="paragraph-small" color="error" className={styles.errorText}>
                    {errors.overrideDecision}
                  </Text>
                )}
              </div>

              <div className={styles.field}>
                <label htmlFor="justification" className={styles.label}>
                  Override Justification *
                </label>
                <Textarea
                  id="justification"
                  value={formData.justification || ''}
                  onChange={(e) => handleFieldChange('justification', e.target.value)}
                  placeholder="Explain the reasoning for this override decision..."
                  rows={3}
                  className={errors.justification ? styles.error : ''}
                />
                {errors.justification && (
                  <Text variant="paragraph-small" color="error" className={styles.errorText}>
                    {errors.justification}
                  </Text>
                )}
              </div>
            </>
          )}

          {/* Additional Notes */}
          <div className={styles.field}>
            <label htmlFor="notes" className={styles.label}>
              Additional Notes
            </label>
            <Textarea
              id="notes"
              value={formData.notes || ''}
              onChange={(e) => handleFieldChange('notes', e.target.value)}
              placeholder="Any additional context or information..."
              rows={2}
            />
          </div>

          {/* Verdict Summary */}
          <div className={styles.verdictSummary}>
            <Text variant="h5" className={styles.summaryTitle}>
              <Clock size={18} />
              Current Verdict Status
            </Text>
            <div className={styles.summaryDetails}>
              <div className={styles.summaryItem}>
                <Text variant="paragraph-small" color="secondary">Verdict</Text>
                <Text variant="paragraph-medium">{verdict.title}</Text>
              </div>
              <div className={styles.summaryItem}>
                <Text variant="paragraph-small" color="secondary">Status</Text>
                <Text variant="paragraph-medium">{verdict.status}</Text>
              </div>
              <div className={styles.summaryItem}>
                <Text variant="paragraph-small" color="secondary">Judges</Text>
                <div className={styles.judgeCount}>
                  <Users size={14} />
                  <Text variant="paragraph-small">{verdict.judgeCount}</Text>
                </div>
              </div>
              <div className={styles.summaryItem}>
                <Text variant="paragraph-small" color="secondary">Consensus</Text>
                <Text variant="paragraph-medium">{Math.round(verdict.consensusScore * 100)}%</Text>
              </div>
            </div>
          </div>

          {/* Actions */}
          <div className={styles.actions}>
            <Button
              type="button"
              variant="secondary"
              onClick={onCancel}
              disabled={isSubmitting}
            >
              Cancel
            </Button>
            <Button
              type="submit"
              variant="primary"
              disabled={isSubmitting}
              className={styles.submitButton}
            >
              {isSubmitting ? 'Submitting...' : 'Submit Intervention Request'}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
}
