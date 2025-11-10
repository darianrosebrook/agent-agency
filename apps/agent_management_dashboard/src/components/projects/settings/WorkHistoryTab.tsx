'use client';

import styles from './WorkHistoryTab.module.scss';

export function WorkHistoryTabContent() {
  return (
    <div className={styles.workHistoryTab}>
      <div className={styles.container}>
        <h2 className={styles.heading}>
          Work History
        </h2>
        <p className={styles.description}>
          View and analyze your team&apos;s work history, time tracking, and
          productivity metrics.
        </p>
        {/* TODO: Replace hardcoded work history metrics with data from v3 database with the following requirements:
        // 1. Work history data fetching: Load team work history and productivity metrics
        //    - Data source: GET /api/projects/:projectId/work-history endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
        //    - Database tables: PostgreSQL `tasks`, `worker_assignments`, and `telemetry` tables
        //    - Aggregate task completion statistics, time tracking, and productivity metrics
        // 2. Time tracking: Calculate time spent on tasks
        //    - Aggregate time from task timestamps (created_at, updated_at, completed_at)
        //    - Calculate average completion time per task
        //    - Track time spent by worker/agent
        // 3. Productivity metrics: Calculate team productivity indicators
        //    - Total tasks completed
        //    - Tasks completed this week/month
        //    - Average completion time
        //    - Task completion rate trends
        // 4. Data visualization: Display metrics in charts and graphs
        //    - Time-series charts showing productivity over time
        //    - Bar charts comparing team member productivity
        //    - Pie charts showing task distribution by status */}
        <div className={styles.metricsGrid}>
          {[
            'Total Tasks',
            'Completed This Week',
            'Average Completion Time',
          ].map((metric, i) => (
            <div
              key={i}
              className={styles.metricCard}
            >
              <p className={styles.metricLabel}>
                {metric}
              </p>
              <p className={styles.metricValue}>
                {i === 0 ? '127' : i === 1 ? '23' : '2.3 days'}
              </p>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

