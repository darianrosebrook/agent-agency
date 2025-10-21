"use client";

import React, { useState, useEffect } from "react";
import Header from "@/components/shared/Header";
import Navigation from "@/components/shared/Navigation";
import TaskList from "@/components/tasks/TaskList";
import TaskFilters from "@/components/tasks/TaskFilters";
import TaskMetrics from "@/components/tasks/TaskMetrics";
import { TaskApiClient } from "@/lib/task-api";
import { Task, TaskListFilters, TaskMetrics as TaskMetricsType } from "@/types/tasks";
import styles from "./page.module.scss";

export default function TasksPage() {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [metrics, setMetrics] = useState<TaskMetricsType | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filters, setFilters] = useState<TaskListFilters>({});
  const [showFilters, setShowFilters] = useState(false);
  const [refreshing, setRefreshing] = useState(false);

  const taskApi = new TaskApiClient();

  const fetchTasks = async (currentFilters: TaskListFilters = filters) => {
    try {
      setError(null);
      const response = await taskApi.getTasks(currentFilters);
      setTasks(response.tasks);
    } catch (err) {
      console.error("Failed to fetch tasks:", err);
      setError(err instanceof Error ? err.message : "Failed to load tasks");
    }
  };

  const fetchMetrics = async () => {
    try {
      const metricsData = await taskApi.getTaskMetrics();
      setMetrics(metricsData);
    } catch (err) {
      console.error("Failed to fetch metrics:", err);
      // Don't set error for metrics failure, just log it
    }
  };

  const handleRefresh = async () => {
    setRefreshing(true);
    try {
      await Promise.all([fetchTasks(), fetchMetrics()]);
    } finally {
      setRefreshing(false);
    }
  };

  const handleFiltersChange = async (newFilters: TaskListFilters) => {
    setFilters(newFilters);
    await fetchTasks(newFilters);
  };

  const handleTaskAction = async (taskId: string, action: string) => {
    try {
      let result;
      switch (action) {
        case "pause":
          result = await taskApi.pauseTask(taskId);
          break;
        case "resume":
          result = await taskApi.resumeTask(taskId);
          break;
        case "cancel":
          result = await taskApi.cancelTask(taskId);
          break;
        case "retry":
          result = await taskApi.retryTask(taskId);
          break;
        default:
          throw new Error(`Unknown action: ${action}`);
      }

      if (result.success) {
        // Refresh the task list to show updated status
        await fetchTasks();
      } else {
        setError(result.message);
      }
    } catch (err) {
      console.error(`Failed to ${action} task ${taskId}:`, err);
      setError(err instanceof Error ? err.message : `Failed to ${action} task`);
    }
  };

  useEffect(() => {
    const loadData = async () => {
      setLoading(true);
      try {
        await Promise.all([fetchTasks(), fetchMetrics()]);
      } finally {
        setLoading(false);
      }
    };

    loadData();
  }, []);

  if (loading) {
    return (
      <div className={styles.page}>
        <Header />
        <Navigation />
        <div className={styles.loading}>
          <div className={styles.spinner}></div>
          <p>Loading tasks...</p>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.page}>
      <Header />
      <Navigation />
      
      <div className={styles.container}>
        <div className={styles.header}>
          <div className={styles.titleSection}>
            <h1 className={styles.title}>Tasks</h1>
            <p className={styles.subtitle}>
              Monitor and manage agent task execution
            </p>
          </div>
          
          <div className={styles.actions}>
            <button
              className={styles.filterButton}
              onClick={() => setShowFilters(!showFilters)}
            >
              {showFilters ? "Hide Filters" : "Show Filters"}
            </button>
            <button
              className={styles.refreshButton}
              onClick={handleRefresh}
              disabled={refreshing}
            >
              {refreshing ? "Refreshing..." : "Refresh"}
            </button>
          </div>
        </div>

        {error && (
          <div className={styles.error}>
            <p>{error}</p>
            <button onClick={() => setError(null)}>Dismiss</button>
          </div>
        )}

        {metrics && (
          <div className={styles.metricsSection}>
            <TaskMetrics metrics={metrics} />
          </div>
        )}

        {showFilters && (
          <div className={styles.filtersSection}>
            <TaskFilters
              filters={filters}
              onFiltersChange={handleFiltersChange}
            />
          </div>
        )}

        <div className={styles.content}>
          <TaskList
            tasks={tasks}
            onTaskAction={handleTaskAction}
            loading={refreshing}
          />
        </div>
      </div>
    </div>
  );
}
