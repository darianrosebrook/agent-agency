"use client";

import React, { useState } from "react";
import { AuditLogEntry } from "@/types/tasks";
import styles from "./AuditTrailViewer.module.scss";

interface AuditTrailViewerProps {
  auditTrail: AuditLogEntry[];
  taskId: string;
  onEntryClick?: (entry: AuditLogEntry) => void;
  showFullTrail?: boolean;
}

export default function AuditTrailViewer({
  auditTrail,
  taskId,
  onEntryClick,
  showFullTrail = false,
}: AuditTrailViewerProps) {
  const [expandedEntries, setExpandedEntries] = useState<Set<string>>(new Set());
  const [filter, setFilter] = useState<string>("all");
  const [sortOrder, setSortOrder] = useState<"asc" | "desc">("desc");

  const getAuditIcon = (action: string) => {
    switch (action.toLowerCase()) {
      case "task_created":
        return "🆕";
      case "task_started":
        return "▶️";
      case "task_paused":
        return "⏸️";
      case "task_resumed":
        return "▶️";
      case "task_completed":
        return "✅";
      case "task_failed":
        return "❌";
      case "task_cancelled":
        return "🛑";
      case "task_state_change":
        return "🔄";
      case "waiver_created":
        return "📋";
      case "waiver_approved":
        return "✅";
      case "waiver_expired":
        return "⏰";
      case "quality_gate_passed":
        return "✅";
      case "quality_gate_failed":
        return "❌";
      case "quality_gate_waived":
        return "⚠️";
      case "worker_assigned":
        return "👷";
      case "worker_completed":
        return "🏁";
      case "model_switched":
        return "🔄";
      case "iteration_started":
        return "🔄";
      case "iteration_completed":
        return "✅";
      default:
        return "📝";
    }
  };

  const getActionColor = (action: string) => {
    switch (action.toLowerCase()) {
      case "task_completed":
      case "quality_gate_passed":
      case "waiver_approved":
        return styles.success;
      case "task_failed":
      case "quality_gate_failed":
        return styles.error;
      case "task_paused":
      case "waiver_expired":
        return styles.warning;
      case "task_state_change":
      case "model_switched":
      case "iteration_started":
        return styles.info;
      default:
        return styles.neutral;
    }
  };

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleString("en-US", {
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  };

  const toggleEntryExpansion = (entryId: string) => {
    const newExpanded = new Set(expandedEntries);
    if (newExpanded.has(entryId)) {
      newExpanded.delete(entryId);
    } else {
      newExpanded.add(entryId);
    }
    setExpandedEntries(newExpanded);
  };

  const filteredAndSortedTrail = auditTrail
    .filter((entry) => {
      if (filter === "all") return true;
      return entry.action.toLowerCase().includes(filter.toLowerCase());
    })
    .sort((a, b) => {
      const dateA = new Date(a.created_at).getTime();
      const dateB = new Date(b.created_at).getTime();
      return sortOrder === "asc" ? dateA - dateB : dateB - dateA;
    });

  const displayedTrail = showFullTrail 
    ? filteredAndSortedTrail 
    : filteredAndSortedTrail.slice(0, 10);

  return (
    <div className={styles.auditTrailViewer}>
      <div className={styles.header}>
        <h3>Audit Trail</h3>
        <div className={styles.controls}>
          <select
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            className={styles.filterSelect}
          >
            <option value="all">All Actions</option>
            <option value="task">Task Actions</option>
            <option value="quality">Quality Gates</option>
            <option value="waiver">Waivers</option>
            <option value="worker">Worker Actions</option>
            <option value="model">Model Actions</option>
          </select>
          <button
            onClick={() => setSortOrder(sortOrder === "asc" ? "desc" : "asc")}
            className={styles.sortButton}
            title={`Sort ${sortOrder === "asc" ? "newest first" : "oldest first"}`}
          >
            {sortOrder === "asc" ? "⬆️" : "⬇️"}
          </button>
        </div>
      </div>

      <div className={styles.timeline}>
        {displayedTrail.map((entry, index) => {
          const isExpanded = expandedEntries.has(entry.id);
          const hasChangeSummary = entry.change_summary && 
            Object.keys(entry.change_summary).length > 0;

          return (
            <div
              key={entry.id}
              className={`${styles.timelineItem} ${getActionColor(entry.action)}`}
              onClick={() => onEntryClick?.(entry)}
            >
              <div className={styles.timelineMarker}>
                <span className={styles.timelineIcon}>
                  {getAuditIcon(entry.action)}
                </span>
              </div>
              
              <div className={styles.timelineContent}>
                <div className={styles.timelineHeader}>
                  <div className={styles.actionInfo}>
                    <span className={styles.action}>{entry.action}</span>
                    {entry.actor && (
                      <span className={styles.actor}>by {entry.actor}</span>
                    )}
                  </div>
                  <div className={styles.timelineMeta}>
                    <span className={styles.timestamp}>
                      {formatDate(entry.created_at)}
                    </span>
                    {hasChangeSummary && (
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          toggleEntryExpansion(entry.id);
                        }}
                        className={styles.expandButton}
                        title={isExpanded ? "Collapse details" : "Expand details"}
                      >
                        {isExpanded ? "▼" : "▶"}
                      </button>
                    )}
                  </div>
                </div>

                {entry.resource_type && (
                  <div className={styles.resourceInfo}>
                    <span className={styles.resourceType}>
                      {entry.resource_type}
                    </span>
                    {entry.resource_id && (
                      <span className={styles.resourceId}>
                        {entry.resource_id}
                      </span>
                    )}
                  </div>
                )}

                {isExpanded && hasChangeSummary && (
                  <div className={styles.changeSummary}>
                    <h5>Change Details:</h5>
                    <pre className={styles.changeDetails}>
                      {JSON.stringify(entry.change_summary, null, 2)}
                    </pre>
                  </div>
                )}
              </div>
            </div>
          );
        })}
      </div>

      {!showFullTrail && auditTrail.length > 10 && (
        <div className={styles.moreEntries}>
          <button className={styles.showMoreButton}>
            Show {auditTrail.length - 10} more entries
          </button>
        </div>
      )}

      {auditTrail.length === 0 && (
        <div className={styles.emptyState}>
          <p>No audit trail entries found for this task.</p>
        </div>
      )}
    </div>
  );
}
