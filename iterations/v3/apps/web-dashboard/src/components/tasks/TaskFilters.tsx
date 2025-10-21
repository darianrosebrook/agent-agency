"use client";

import React, { useState } from "react";
import { TaskListFilters } from "@/types/tasks";
import styles from "./TaskFilters.module.scss";

interface TaskFiltersProps {
  filters: TaskListFilters;
  onFiltersChange: (filters: TaskListFilters) => void;
}

export default function TaskFilters({ filters, onFiltersChange }: TaskFiltersProps) {
  const [localFilters, setLocalFilters] = useState<TaskListFilters>(filters);

  const handleFilterChange = (key: keyof TaskListFilters, value: any) => {
    const newFilters = { ...localFilters, [key]: value };
    setLocalFilters(newFilters);
  };

  const handleApplyFilters = () => {
    onFiltersChange(localFilters);
  };

  const handleClearFilters = () => {
    const clearedFilters: TaskListFilters = {};
    setLocalFilters(clearedFilters);
    onFiltersChange(clearedFilters);
  };

  const handleSearchChange = (value: string) => {
    handleFilterChange("search", value || undefined);
  };

  const handleStatusChange = (status: string, checked: boolean) => {
    const currentStatuses = localFilters.status || [];
    let newStatuses;
    
    if (checked) {
      newStatuses = [...currentStatuses, status];
    } else {
      newStatuses = currentStatuses.filter(s => s !== status);
    }
    
    handleFilterChange("status", newStatuses.length > 0 ? newStatuses : undefined);
  };

  const handlePhaseChange = (phase: string, checked: boolean) => {
    const currentPhases = localFilters.phase || [];
    let newPhases;
    
    if (checked) {
      newPhases = [...currentPhases, phase];
    } else {
      newPhases = currentPhases.filter(p => p !== phase);
    }
    
    handleFilterChange("phase", newPhases.length > 0 ? newPhases : undefined);
  };

  const handlePriorityChange = (priority: string, checked: boolean) => {
    const currentPriorities = localFilters.priority || [];
    let newPriorities;
    
    if (checked) {
      newPriorities = [...currentPriorities, priority];
    } else {
      newPriorities = currentPriorities.filter(p => p !== priority);
    }
    
    handleFilterChange("priority", newPriorities.length > 0 ? newPriorities : undefined);
  };

  const statusOptions = [
    { value: "pending", label: "Pending" },
    { value: "running", label: "Running" },
    { value: "completed", label: "Completed" },
    { value: "failed", label: "Failed" },
    { value: "paused", label: "Paused" },
    { value: "cancelled", label: "Cancelled" },
  ];

  const phaseOptions = [
    { value: "planning", label: "Planning" },
    { value: "analysis", label: "Analysis" },
    { value: "execution", label: "Execution" },
    { value: "validation", label: "Validation" },
    { value: "refinement", label: "Refinement" },
    { value: "qa", label: "QA" },
    { value: "finalization", label: "Finalization" },
  ];

  const priorityOptions = [
    { value: "low", label: "Low" },
    { value: "medium", label: "Medium" },
    { value: "high", label: "High" },
    { value: "critical", label: "Critical" },
  ];

  return (
    <div className={styles.filters}>
      <div className={styles.header}>
        <h3>Filter Tasks</h3>
        <div className={styles.actions}>
          <button
            className={styles.clearButton}
            onClick={handleClearFilters}
          >
            Clear All
          </button>
          <button
            className={styles.applyButton}
            onClick={handleApplyFilters}
          >
            Apply Filters
          </button>
        </div>
      </div>

      <div className={styles.filtersGrid}>
        <div className={styles.filterGroup}>
          <label className={styles.filterLabel}>Search</label>
          <input
            type="text"
            placeholder="Search tasks..."
            value={localFilters.search || ""}
            onChange={(e) => handleSearchChange(e.target.value)}
            className={styles.searchInput}
          />
        </div>

        <div className={styles.filterGroup}>
          <label className={styles.filterLabel}>Status</label>
          <div className={styles.checkboxGroup}>
            {statusOptions.map((option) => (
              <label key={option.value} className={styles.checkboxLabel}>
                <input
                  type="checkbox"
                  checked={localFilters.status?.includes(option.value) || false}
                  onChange={(e) => handleStatusChange(option.value, e.target.checked)}
                  className={styles.checkbox}
                />
                <span className={styles.checkboxText}>{option.label}</span>
              </label>
            ))}
          </div>
        </div>

        <div className={styles.filterGroup}>
          <label className={styles.filterLabel}>Phase</label>
          <div className={styles.checkboxGroup}>
            {phaseOptions.map((option) => (
              <label key={option.value} className={styles.checkboxLabel}>
                <input
                  type="checkbox"
                  checked={localFilters.phase?.includes(option.value) || false}
                  onChange={(e) => handlePhaseChange(option.value, e.target.checked)}
                  className={styles.checkbox}
                />
                <span className={styles.checkboxText}>{option.label}</span>
              </label>
            ))}
          </div>
        </div>

        <div className={styles.filterGroup}>
          <label className={styles.filterLabel}>Priority</label>
          <div className={styles.checkboxGroup}>
            {priorityOptions.map((option) => (
              <label key={option.value} className={styles.checkboxLabel}>
                <input
                  type="checkbox"
                  checked={localFilters.priority?.includes(option.value) || false}
                  onChange={(e) => handlePriorityChange(option.value, e.target.checked)}
                  className={styles.checkbox}
                />
                <span className={styles.checkboxText}>{option.label}</span>
              </label>
            ))}
          </div>
        </div>

        <div className={styles.filterGroup}>
          <label className={styles.filterLabel}>Date Range</label>
          <div className={styles.dateRange}>
            <input
              type="date"
              value={localFilters.created_after || ""}
              onChange={(e) => handleFilterChange("created_after", e.target.value || undefined)}
              className={styles.dateInput}
            />
            <span className={styles.dateSeparator}>to</span>
            <input
              type="date"
              value={localFilters.created_before || ""}
              onChange={(e) => handleFilterChange("created_before", e.target.value || undefined)}
              className={styles.dateInput}
            />
          </div>
        </div>
      </div>
    </div>
  );
}
