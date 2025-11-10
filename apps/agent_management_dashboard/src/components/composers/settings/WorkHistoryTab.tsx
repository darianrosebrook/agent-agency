'use client';

import { useState, useEffect, useMemo } from "react";
import { KanbanHeading } from "../../primitives/kanban/KanbanHeading";
import { KanbanText } from "../../primitives/kanban/KanbanText";
import { getTasksStats, type TasksStats } from "../../lib/api/tasks";
import { listTasks, type Task } from "../../lib/api/tasks";
import styles from "./WorkHistoryTab.module.scss";

export function WorkHistoryTabContent() {
  const [taskStats, setTaskStats] = useState<TasksStats | null>(null);
  const [allTasks, setAllTasks] = useState<Task[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    async function fetchData() {
      setIsLoading(true);
      setError(null);
      try {
        const [statsData, tasksData] = await Promise.all([
          getTasksStats(),
          listTasks(),
        ]);
        setTaskStats(statsData);
        setAllTasks(tasksData.tasks);
      } catch (err) {
        console.error("Failed to fetch work history data:", err);
        setError(err instanceof Error ? err : new Error("Failed to load work history"));
      } finally {
        setIsLoading(false);
      }
    }

    fetchData();
    // Refresh every 30 seconds
    const interval = setInterval(fetchData, 30000);
    return () => clearInterval(interval);
  }, []);

  // Calculate completed this week
  const completedThisWeek = useMemo(() => {
    const now = new Date();
    const weekAgo = new Date(now);
    weekAgo.setDate(weekAgo.getDate() - 7);
    
    return allTasks.filter((task) => {
      if (task.status !== "completed" || !task.completed_at) return false;
      const completedDate = new Date(task.completed_at);
      return completedDate >= weekAgo && completedDate <= now;
    }).length;
  }, [allTasks]);

  // Calculate average completion time
  const averageCompletionTime = useMemo(() => {
    const completedTasks = allTasks.filter(
      (task) => task.status === "completed" && task.completed_at && task.created_at
    );

    if (completedTasks.length === 0) return 0;

    const totalDays = completedTasks.reduce((sum, task) => {
      const created = new Date(task.created_at);
      const completed = new Date(task.completed_at!);
      const days = (completed.getTime() - created.getTime()) / (1000 * 60 * 60 * 24);
      return sum + days;
    }, 0);

    return totalDays / completedTasks.length;
  }, [allTasks]);

  const metrics = [
    {
      label: 'Total Tasks',
      value: taskStats?.total ?? 0,
      isLoading: isLoading && !taskStats,
    },
    {
      label: 'Completed This Week',
      value: completedThisWeek,
      isLoading: isLoading && allTasks.length === 0,
    },
    {
      label: 'Average Completion Time',
      value: averageCompletionTime > 0 
        ? `${averageCompletionTime.toFixed(1)} days`
        : 'N/A',
      isLoading: isLoading && allTasks.length === 0,
    },
  ];

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
        {error ? (
          <div className={styles.errorState}>
            <KanbanText size="sm">Error: {error.message}</KanbanText>
          </div>
        ) : (
          <div className={styles.metricsGrid}>
            {metrics.map((metric, i) => (
              <div key={i} className={styles.metricCard}>
                <KanbanText size="sm" className={styles.metricLabel}>
                  {metric.label}
                </KanbanText>
                <KanbanText size="xl" className={styles.metricValue}>
                  {metric.isLoading ? '...' : typeof metric.value === 'number' ? metric.value.toLocaleString() : metric.value}
                </KanbanText>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
