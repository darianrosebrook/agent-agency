'use client';

import { KanbanHeading } from "../../primitives/kanban/KanbanHeading";
import { KanbanText } from "../../primitives/kanban/KanbanText";
import styles from "./WorkHistoryTab.module.scss";

export function WorkHistoryTabContent() {
  return (
    <div className={styles.workHistoryTab}>
      <div className={styles.workHistoryCard}>
        <KanbanHeading size="lg" className={styles.cardTitle}>
          Work History
        </KanbanHeading>
        <KanbanText size="sm" className={styles.cardDescription}>
          View and analyze your team&apos;s work history, time tracking, and
          productivity metrics.
        </KanbanText>
        {/* TODO: Replace hardcoded work history metrics with data from v3 database */}
        <div className={styles.metricsGrid}>
          {[
            { label: 'Total Tasks', value: '127' },
            { label: 'Completed This Week', value: '23' },
            { label: 'Average Completion Time', value: '2.3 days' },
          ].map((metric, i) => (
            <div key={i} className={styles.metricCard}>
              <KanbanText size="sm" className={styles.metricLabel}>
                {metric.label}
              </KanbanText>
              <KanbanText size="xl" className={styles.metricValue}>
                {metric.value}
              </KanbanText>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
