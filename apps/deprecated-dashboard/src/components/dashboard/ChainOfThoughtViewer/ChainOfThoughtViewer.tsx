'use client';

import React, { useState } from 'react';
import { Card } from '@/components/ui';
import { formatDateTime } from '@/lib/utils';
import type { ChainOfThought } from '@/types';
import styles from './ChainOfThoughtViewer.module.scss';

export interface ChainOfThoughtViewerProps {
  chainOfThought: ChainOfThought;
}

export const ChainOfThoughtViewer: React.FC<ChainOfThoughtViewerProps> = ({
  chainOfThought,
}) => {
  const [expandedEntries, setExpandedEntries] = useState<Set<number>>(new Set());

  const toggleEntry = (index: number) => {
    const newExpanded = new Set(expandedEntries);
    if (newExpanded.has(index)) {
      newExpanded.delete(index);
    } else {
      newExpanded.add(index);
    }
    setExpandedEntries(newExpanded);
  };

  return (
    <Card>
      <div className={styles.header}>
        <h2>Chain of Thought</h2>
        <span className={styles.count}>{chainOfThought.entries.length} entries</span>
      </div>
      <div className={styles.entries}>
        {chainOfThought.entries.length === 0 ? (
          <p className={styles.empty}>No chain of thought entries available</p>
        ) : (
          chainOfThought.entries.map((entry, index) => {
            const isExpanded = expandedEntries.has(index);
            return (
              <div key={index} className={styles.entry}>
                <button
                  className={styles.entryHeader}
                  onClick={() => toggleEntry(index)}
                >
                  <div className={styles.entryMeta}>
                    <span className={styles.actor}>{entry.actor}</span>
                    <span className={styles.timestamp}>
                      {formatDateTime(entry.timestamp)}
                    </span>
                  </div>
                  <span className={styles.toggle}>
                    {isExpanded ? '−' : '+'}
                  </span>
                </button>
                {isExpanded && (
                  <div className={styles.entryContent}>
                    <p className={styles.thought}>{entry.thought}</p>
                    {entry.context && Object.keys(entry.context).length > 0 && (
                      <details className={styles.context}>
                        <summary>Context</summary>
                        <pre>{JSON.stringify(entry.context, null, 2)}</pre>
                      </details>
                    )}
                  </div>
                )}
              </div>
            );
          })
        )}
      </div>
    </Card>
  );
};

