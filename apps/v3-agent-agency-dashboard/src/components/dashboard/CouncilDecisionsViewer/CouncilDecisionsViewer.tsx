'use client';

import React from 'react';
import { Card, Badge } from '@/components/ui';
import { formatDateTime } from '@/lib/utils';
import type { CouncilDecisions } from '@/types';
import styles from './CouncilDecisionsViewer.module.scss';

export interface CouncilDecisionsViewerProps {
  decisions: CouncilDecisions;
}

export const CouncilDecisionsViewer: React.FC<CouncilDecisionsViewerProps> = ({
  decisions,
}) => {
  const getVerdictVariant = (verdict: string) => {
    const lower = verdict.toLowerCase();
    if (lower.includes('approve') || lower.includes('pass')) return 'success';
    if (lower.includes('reject') || lower.includes('fail')) return 'error';
    if (lower.includes('review') || lower.includes('pending')) return 'warning';
    return 'default';
  };

  return (
    <Card>
      <div className={styles.header}>
        <h2>Council Decisions</h2>
        {decisions.final_decision && (
          <Badge variant={getVerdictVariant(decisions.final_decision)}>
            {decisions.final_decision}
          </Badge>
        )}
      </div>
      <div className={styles.timeline}>
        {decisions.verdicts.length === 0 ? (
          <p className={styles.empty}>No council decisions available</p>
        ) : (
          decisions.verdicts.map((verdict, index) => (
            <div key={verdict.id} className={styles.verdict}>
              <div className={styles.verdictHeader}>
                <div className={styles.verdictMeta}>
                  <span className={styles.judgeName}>Judge: {verdict.judge_id}</span>
                  <span className={styles.timestamp}>
                    {formatDateTime(verdict.created_at)}
                  </span>
                </div>
                <Badge variant={getVerdictVariant(verdict.verdict)}>
                  {verdict.verdict}
                </Badge>
              </div>
              <div className={styles.verdictContent}>
                <p className={styles.reasoning}>{verdict.reasoning}</p>
                {verdict.confidence !== undefined && (
                  <div className={styles.confidence}>
                    <span>Confidence: </span>
                    <strong>{verdict.confidence.toFixed(2)}</strong>
                  </div>
                )}
              </div>
            </div>
          ))
        )}
      </div>
    </Card>
  );
};

