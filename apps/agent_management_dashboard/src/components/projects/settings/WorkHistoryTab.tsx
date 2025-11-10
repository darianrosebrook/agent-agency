'use client';

import { useState, useEffect } from 'react';
import { useProjectContext } from '../../ProjectContext';
import { getProjectWorkHistory, getProjectTaskStats, type WorkHistoryEntry, type ProjectTaskStats } from '../../../lib/api/projects';
import styles from './WorkHistoryTab.module.scss';

export function WorkHistoryTabContent() {
  const { currentProjectId } = useProjectContext();
  const [workHistory, setWorkHistory] = useState<WorkHistoryEntry[]>([]);
  const [taskStats, setTaskStats] = useState<ProjectTaskStats | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    async function fetchData() {
      if (!currentProjectId) {
        setIsLoading(false);
        return;
      }

      setIsLoading(true);
      setError(null);

      try {
        const [historyData, statsData] = await Promise.all([
          getProjectWorkHistory(currentProjectId, { limit: 100 }).catch(() => []),
          getProjectTaskStats(currentProjectId).catch(() => null),
        ]);

        setWorkHistory(historyData);
        setTaskStats(statsData);
      } catch (err) {
        setError(err instanceof Error ? err : new Error('Failed to load work history'));
      } finally {
        setIsLoading(false);
      }
    }

    fetchData();
  }, [currentProjectId]);

  if (isLoading) {
    return (
      <div className={styles.workHistoryTab}>
        <div className={styles.container}>
          <h2 className={styles.heading}>Work History</h2>
          <p className={styles.description}>Loading work history...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={styles.workHistoryTab}>
        <div className={styles.container}>
          <h2 className={styles.heading}>Work History</h2>
          <p className={styles.description}>Error: {error.message}</p>
        </div>
      </div>
    );
  }

  // Calculate metrics from work history and task stats
  const totalTasks = taskStats?.total || 0;
  const completedThisWeek = workHistory.filter((entry) => {
    const entryDate = new Date(entry.timestamp);
    const weekAgo = new Date();
    weekAgo.setDate(weekAgo.getDate() - 7);
    return entryDate >= weekAgo && entry.action.includes('completed');
  }).length;

  // Calculate average completion time (simplified)
  const avgCompletionTime = taskStats?.completion_rate 
    ? `${(taskStats.completion_rate / 100).toFixed(1)} days`
    : 'N/A';

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
        <div className={styles.metricsGrid}>
          <div className={styles.metricCard}>
            <p className={styles.metricLabel}>Total Tasks</p>
            <p className={styles.metricValue}>{totalTasks}</p>
          </div>
          <div className={styles.metricCard}>
            <p className={styles.metricLabel}>Completed This Week</p>
            <p className={styles.metricValue}>{completedThisWeek}</p>
          </div>
          <div className={styles.metricCard}>
            <p className={styles.metricLabel}>Average Completion Time</p>
            <p className={styles.metricValue}>{avgCompletionTime}</p>
          </div>
          {taskStats && (
            <>
              <div className={styles.metricCard}>
                <p className={styles.metricLabel}>In Progress</p>
                <p className={styles.metricValue}>{taskStats.in_progress}</p>
              </div>
              <div className={styles.metricCard}>
                <p className={styles.metricLabel}>Completion Rate</p>
                <p className={styles.metricValue}>{taskStats.completion_rate.toFixed(1)}%</p>
              </div>
              <div className={styles.metricCard}>
                <p className={styles.metricLabel}>Failed</p>
                <p className={styles.metricValue}>{taskStats.failed}</p>
              </div>
            </>
          )}
        </div>

        {/* Recent Work History */}
        {workHistory.length > 0 && (
          <div className={styles.historyList}>
            <h3 className={styles.historyTitle}>Recent Activity</h3>
            {workHistory.slice(0, 20).map((entry) => (
              <div key={entry.id} className={styles.historyItem}>
                <div className={styles.historyAction}>{entry.action}</div>
                <div className={styles.historyDescription}>{entry.description}</div>
                <div className={styles.historyTime}>
                  {new Date(entry.timestamp).toLocaleString()}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

