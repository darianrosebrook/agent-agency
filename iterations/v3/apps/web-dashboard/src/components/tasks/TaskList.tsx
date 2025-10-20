"use client";

import React, { useState, useEffect, useCallback } from "react";
import { TaskListProps, Task, TaskFilters } from "@/types/tasks";
import { taskApiClient, TaskApiError } from "@/lib/task-api";
import TaskCard from "./TaskCard";
import TaskFiltersBar from "./TaskFiltersBar";
import styles from "./TaskList.module.scss";

interface TaskListState {
  tasks: Task[];
  isLoading: boolean;
  error: string | null;
  filters: TaskFilters;
  pagination: {
    page: number;
    pageSize: number;
    totalCount: number;
    filteredCount: number;
  };
}

export default function TaskList({
  tasks,
  isLoading: externalLoading,
  onTaskSelect,
  onTaskFilter,
  selectedTaskId,
}: TaskListProps) {
  const [state, setState] = useState<TaskListState>({
    tasks: tasks ?? [],
    isLoading: externalLoading ?? !tasks,
    error: null,
    filters: {},
    pagination: {
      page: 1,
      pageSize: 20,
      totalCount: 0,
      filteredCount: 0,
    },
  });

  // Load tasks from API if not provided externally
  const loadTasks = useCallback(
    async (filters: TaskFilters = {}, page: number = 1) => {
      if (tasks) return; // Use external tasks if provided

      try {
        setState((prev) => ({ ...prev, isLoading: true, error: null }));

        const response = await taskApiClient.getTasks(
          filters,
          page,
          state.pagination.pageSize
        );

        setState((prev) => ({
          ...prev,
          tasks: response.tasks,
          isLoading: false,
          filters,
          pagination: {
            ...prev.pagination,
            page,
            totalCount: response.total_count,
            filteredCount: response.filtered_count,
          },
        }));

        onTaskFilter?.(filters);
      } catch (error) {
        const errorMessage =
          error instanceof TaskApiError
            ? error.message
            : "Failed to load tasks";

        setState((prev) => ({
          ...prev,
          isLoading: false,
          error: errorMessage,
        }));

        console.error("Failed to load tasks:", error);
      }
    },
    [tasks, state.pagination.pageSize, onTaskFilter]
  );

  // Initial load
  useEffect(() => {
    if (!tasks) {
      loadTasks();
    } else {
      setState((prev) => ({
        ...prev,
        tasks: tasks,
        isLoading: false,
      }));
    }
  }, [tasks, loadTasks]);

  // Handle filter changes
  const handleFiltersChange = useCallback(
    (newFilters: TaskFilters) => {
      setState((prev) => ({ ...prev, filters: newFilters }));
      loadTasks(newFilters, 1); // Reset to first page
    },
    [loadTasks]
  );

  // Handle page changes
  const handlePageChange = useCallback(
    (newPage: number) => {
      loadTasks(state.filters, newPage);
    },
    [loadTasks, state.filters]
  );

  // Handle task selection
  const handleTaskSelect = useCallback(
    (task: Task) => {
      onTaskSelect?.(task);
    },
    [onTaskSelect]
  );

  if (state.isLoading && !tasks) {
    return (
      <div className={styles.taskList}>
        <div className={styles.loading}>
          <div className={styles.spinner}></div>
          <p>Loading tasks...</p>
        </div>
      </div>
    );
  }

  if (state.error) {
    return (
      <div className={styles.taskList}>
        <div className={styles.error}>
          <h3>Failed to load tasks</h3>
          <p>{state.error}</p>
          <button
            onClick={() => loadTasks(state.filters, state.pagination.page)}
          >
            Retry
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.taskList}>
      <div className={styles.header}>
        <h2>Task Monitoring</h2>
        <div className={styles.stats}>
          <span className={styles.stat}>
            {state.pagination.filteredCount} of {state.pagination.totalCount}{" "}
            tasks
          </span>
        </div>
      </div>

      <TaskFiltersBar
        filters={state.filters}
        onFiltersChange={handleFiltersChange}
      />

      <div className={styles.taskGrid}>
        {state.tasks.length === 0 ? (
          <div className={styles.emptyState}>
            <div className={styles.emptyIcon}>📋</div>
            <h3>No tasks found</h3>
            <p>
              {Object.keys(state.filters).length > 0
                ? "Try adjusting your filters to see more tasks."
                : "No tasks are currently running or available."}
            </p>
          </div>
        ) : (
          state.tasks.map((task) => (
            <TaskCard
              key={task.id}
              task={task}
              isSelected={selectedTaskId === task.id}
              onClick={() => handleTaskSelect(task)}
            />
          ))
        )}
      </div>

      {/* Pagination */}
      {state.pagination.totalCount > state.pagination.pageSize && (
        <div className={styles.pagination}>
          <button
            className={styles.pageButton}
            disabled={state.pagination.page <= 1}
            onClick={() => handlePageChange(state.pagination.page - 1)}
          >
            Previous
          </button>

          <span className={styles.pageInfo}>
            Page {state.pagination.page} of{" "}
            {Math.ceil(
              state.pagination.filteredCount / state.pagination.pageSize
            )}
          </span>

          <button
            className={styles.pageButton}
            disabled={
              state.pagination.page >=
              Math.ceil(
                state.pagination.filteredCount / state.pagination.pageSize
              )
            }
            onClick={() => handlePageChange(state.pagination.page + 1)}
          >
            Next
          </button>
        </div>
      )}
    </div>
  );
}
