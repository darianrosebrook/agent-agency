"use client";

import React, { useState } from "react";
import { TaskFilters } from "@/types/tasks";
import styles from "./TaskFiltersBar.module.scss";

interface TaskFiltersBarProps {
  filters: TaskFilters;
  onFiltersChange: (filters: TaskFilters) => void;
}

export default function TaskFiltersBar({
  filters,
  onFiltersChange,
}: TaskFiltersBarProps) {
  const [isExpanded, setIsExpanded] = useState(false);

  const updateFilters = (updates: Partial<TaskFilters>) => {
    const newFilters = { ...filters, ...updates };
    onFiltersChange(newFilters);
  };

  const toggleFilter = (filterType: keyof TaskFilters, value: string) => {
    const currentValues = (filters[filterType] as string[] | undefined) ?? [];
    const newValues = currentValues.includes(value)
      ? currentValues.filter((v) => v !== value)
      : [...currentValues, value];

    updateFilters({
      [filterType]: newValues.length > 0 ? newValues : undefined,
    });
  };

  const clearFilters = () => {
    onFiltersChange({});
  };

  const hasActiveFilters = Object.keys(filters).some((key) => {
    const value = filters[key as keyof TaskFilters];
    return (
      value !== undefined &&
      value !== null &&
      (Array.isArray(value) ? value.length > 0 : true)
    );
  });

  return (
    <div className={styles.filtersBar}>
      <div className={styles.filtersHeader}>
        <button
          onClick={() => setIsExpanded(!isExpanded)}
          className={styles.toggleButton}
          aria-expanded={isExpanded}
        >
          <span className={styles.toggleIcon}>{isExpanded ? "−" : "+"}</span>
          Filters
          {hasActiveFilters && (
            <span className={styles.activeCount}>
              ({Object.keys(filters).length})
            </span>
          )}
        </button>

        {hasActiveFilters && (
          <button onClick={clearFilters} className={styles.clearButton}>
            Clear All
          </button>
        )}
      </div>

      {isExpanded && (
        <div className={styles.filtersContent}>
          {/* Status Filter */}
          <div className={styles.filterGroup}>
            <label className={styles.filterLabel}>Status</label>
            <div className={styles.filterOptions}>
              {(
                [
                  "pending",
                  "running",
                  "paused",
                  "completed",
                  "failed",
                  "cancelled",
                ] as const
              ).map((status) => (
                <label key={status} className={styles.checkboxLabel}>
                  <input
                    type="checkbox"
                    checked={filters.status?.includes(status) ?? false}
                    onChange={() => toggleFilter("status", status)}
                  />
                  <span className={styles.checkboxText}>
                    {status.charAt(0).toUpperCase() + status.slice(1)}
                  </span>
                </label>
              ))}
            </div>
          </div>

          {/* Phase Filter */}
          <div className={styles.filterGroup}>
            <label className={styles.filterLabel}>Phase</label>
            <div className={styles.filterOptions}>
              {(
                [
                  "planning",
                  "analysis",
                  "execution",
                  "validation",
                  "refinement",
                  "qa",
                  "finalization",
                ] as const
              ).map((phase) => (
                <label key={phase} className={styles.checkboxLabel}>
                  <input
                    type="checkbox"
                    checked={filters.phase?.includes(phase) ?? false}
                    onChange={() => toggleFilter("phase", phase)}
                  />
                  <span className={styles.checkboxText}>
                    {phase.charAt(0).toUpperCase() + phase.slice(1)}
                  </span>
                </label>
              ))}
            </div>
          </div>

          {/* Priority Filter */}
          <div className={styles.filterGroup}>
            <label className={styles.filterLabel}>Priority</label>
            <div className={styles.filterOptions}>
              {(["low", "medium", "high", "critical"] as const).map(
                (priority) => (
                  <label key={priority} className={styles.checkboxLabel}>
                    <input
                      type="checkbox"
                      checked={filters.priority?.includes(priority) ?? false}
                      onChange={() => toggleFilter("priority", priority)}
                    />
                    <span className={styles.checkboxText}>
                      {priority.charAt(0).toUpperCase() + priority.slice(1)}
                    </span>
                  </label>
                )
              )}
            </div>
          </div>

          {/* Working Spec ID Filter */}
          <div className={styles.filterGroup}>
            <label className={styles.filterLabel}>Working Spec ID</label>
            <input
              type="text"
              value={filters.working_spec_id ?? ""}
              onChange={(e) =>
                updateFilters({ working_spec_id: e.target.value || undefined })
              }
              placeholder="Enter working spec ID..."
              className={styles.textInput}
            />
          </div>

          {/* Date Range Filter */}
          <div className={styles.filterGroup}>
            <label className={styles.filterLabel}>Date Range</label>
            <div className={styles.dateInputs}>
              <input
                type="date"
                value={filters.date_range?.start ?? ""}
                onChange={(e) =>
                  updateFilters({
                    date_range: {
                      start: e.target.value,
                      end: filters.date_range?.end ?? "",
                    },
                  })
                }
                className={styles.dateInput}
              />
              <span className={styles.dateSeparator}>to</span>
              <input
                type="date"
                value={filters.date_range?.end ?? ""}
                onChange={(e) =>
                  updateFilters({
                    date_range: {
                      start: filters.date_range?.start ?? "",
                      end: e.target.value,
                    },
                  })
                }
                className={styles.dateInput}
              />
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
