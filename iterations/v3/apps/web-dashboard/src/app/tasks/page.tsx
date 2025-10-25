/**
 * Tasks List Page
 * Displays all tasks with filtering, sorting, and real-time updates
 * 
 * @author @darianrosebrook
 */

"use client";

import { useState, useEffect, Suspense, useRef, lazy } from "react";
import DashboardLayout from "@/components/shared/DashboardLayout";
import { TaskApiClient } from "@/lib/task-api";
import { Task, TaskListFilters, TaskMetrics as TaskMetricsType } from "@/types/tasks";
import { Text } from "@/design-system/primitives";
import { RefreshCw, Filter, X } from "lucide-react";
import styles from "./page.module.scss";

// Lazy load heavy components for better performance
const TaskListLazy = lazy(() => import("@/components/tasks/TaskList"));
const TaskFiltersLazy = lazy(() => import("@/components/tasks/TaskFilters"));
const TaskMetricsLazy = lazy(() => import("@/components/tasks/TaskMetrics"));

/**
 * Loading skeleton for metrics section
 */
function MetricsSkeleton() {
  return (
    <div 
      className={styles.metricsSection}
      style={{ 
        minHeight: '160px',
        height: '160px',
        maxHeight: '160px',
        contain: 'layout style paint',
      }}
      role="status"
      aria-live="polite"
      aria-busy="true"
    >
      <div className={styles.loading}>
        <div className={styles.spinner} aria-hidden="true"></div>
        <span className="sr-only">Loading metrics...</span>
      </div>
    </div>
  );
}

/**
 * Loading skeleton for task list
 */
function TaskListSkeleton() {
  return (
    <div 
      className={styles.taskListSection}
      style={{ 
        minHeight: '400px',
        contain: 'layout style paint',
      }}
      role="status"
      aria-live="polite"
      aria-busy="true"
    >
      <div className={styles.loading}>
        <div className={styles.spinner} aria-hidden="true"></div>
        <span className="sr-only">Loading tasks...</span>
      </div>
    </div>
  );
}

export default function TasksPage() {
  // State management
  const [tasks, setTasks] = useState<Task[]>([]);
  const [metrics, setMetrics] = useState<TaskMetricsType | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filters, setFilters] = useState<TaskListFilters>({});
  const [showFilters, setShowFilters] = useState(false);
  const [refreshing, setRefreshing] = useState(false);

  // Enhanced GSAP animations with refs
  const headerRef = useRef<HTMLElement>(null);
  const metricsRef = useRef<HTMLElement>(null);
  const filtersRef = useRef<HTMLElement>(null);
  const taskListRef = useRef<HTMLElement>(null);

  // Enhanced animations will be applied via GSAP in useEffect

  const taskApi = new TaskApiClient();

  /**
   * Fetch tasks from API
   */

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

  /**
   * Fetch metrics from API
   */
  const fetchMetrics = async () => {
    try {
      const metricsData = await taskApi.getTaskMetrics();
      setMetrics(metricsData);
    } catch (err) {
      console.error("Failed to fetch metrics:", err);
      // Don't set error for metrics failure
    }
  };

  /**
   * Refresh all data
   */
  const handleRefresh = async () => {
    setRefreshing(true);
    try {
      await Promise.all([fetchTasks(), fetchMetrics()]);
    } finally {
      setRefreshing(false);
    }
  };

  /**
   * Handle filter changes
   */
  const handleFiltersChange = async (newFilters: TaskListFilters) => {
    setFilters(newFilters);
    await fetchTasks(newFilters);
  };

  /**
   * Handle task actions (pause, resume, cancel, retry)
   */
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
        // Refresh to show updated status
        await handleRefresh();
      }
    } catch (err) {
      console.error(`Failed to ${action} task:`, err);
      setError(err instanceof Error ? err.message : `Failed to ${action} task`);
    }
  };

  /**
   * Initial data load
   */
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
    
    // Set up polling for real-time updates (every 30 seconds)
    const interval = setInterval(() => {
      fetchTasks();
      fetchMetrics();
    }, 30000);

    return () => clearInterval(interval);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <DashboardLayout>
      <main role="main" aria-label="Tasks" className={styles.container}>
        {/* Page Header - Bold typography with enhanced animations */}
        <header ref={headerRef} className={styles.header}>
          <div className={styles.headerContent}>
            <div>
              <Text variant="display-3" className={styles.title} id="page-title">
                Tasks
              </Text>
              <Text variant="paragraph-large" color="secondary" className={styles.subtitle}>
                Monitor and manage all task executions
              </Text>
            </div>
            
            <div className={styles.headerActions}>
              <button
                onClick={() => setShowFilters(!showFilters)}
                className={styles.filterButton}
                aria-expanded={showFilters}
                aria-label="Toggle filters"
              >
                {showFilters ? <X size={20} /> : <Filter size={20} />}
                <span>Filters</span>
              </button>
              
              <button
                onClick={handleRefresh}
                className={styles.refreshButton}
                disabled={refreshing}
                aria-label="Refresh tasks"
              >
                <RefreshCw 
                  size={20} 
                  className={refreshing ? styles.spinning : ''}
                  aria-hidden="true"
                />
                <span>Refresh</span>
              </button>
            </div>
          </div>
        </header>

        {/* Metrics Section */}
        <section 
          ref={metricsRef}
          aria-labelledby="metrics-heading"
          role="region"
        >
          <h2 id="metrics-heading" className="sr-only">Task Metrics</h2>
          <Suspense fallback={<MetricsSkeleton />}>
            {loading ? (
              <MetricsSkeleton />
            ) : metrics ? (
              <TaskMetricsLazy metrics={metrics} />
            ) : null}
          </Suspense>
        </section>

        {/* Filters Section (Collapsible) with enhanced animations */}
        {showFilters && (
          <section
            ref={filtersRef}
            aria-labelledby="filters-heading"
            role="region"
            className={styles.filtersSection}
          >
            <h2 id="filters-heading" className="sr-only">Task Filters</h2>
            <Suspense fallback={<div className={styles.loading}>Loading filters...</div>}>
              <TaskFiltersLazy
                filters={filters}
                onFiltersChange={handleFiltersChange}
              />
            </Suspense>
          </section>
        )}

        {/* Error State */}
        {error && (
          <div role="alert" className={styles.error}>
            <Text variant="paragraph-medium" color="error">
              ⚠️ {error}
            </Text>
          </div>
        )}

        {/* Task List Section with enhanced animations */}
        <section
          ref={taskListRef}
          aria-labelledby="tasks-heading"
          role="region"
          className={styles.taskListSection}
        >
          <h2 id="tasks-heading" className="sr-only">Task List</h2>
          <Suspense fallback={<TaskListSkeleton />}>
            {loading ? (
              <TaskListSkeleton />
            ) : (
              <TaskListLazy
                tasks={tasks}
                onTaskAction={handleTaskAction}
                loading={loading}
              />
            )}
          </Suspense>
        </section>
      </main>
    </DashboardLayout>
  );
}
