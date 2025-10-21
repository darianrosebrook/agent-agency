"use client";

import React, { useState, useEffect } from "react";
import { AuditLogEntry } from "@/types/tasks";
import styles from "./AuditTrailViewer.module.scss";

interface AuditTrailViewerProps {
  auditTrail: AuditLogEntry[];
  taskId: string;
  showFullTrail?: boolean;
  maxHeight?: string;
}

export default function AuditTrailViewer({
  auditTrail,
  taskId,
  showFullTrail = false,
  maxHeight = "400px",
}: AuditTrailViewerProps) {
  const [expandedEntries, setExpandedEntries] = useState<Set<string>>(new Set());
  const [filteredTrail, setFilteredTrail] = useState<AuditLogEntry[]>(auditTrail);
  const [filter, setFilter] = useState<string>("all");
  const [searchTerm, setSearchTerm] = useState<string>("");

  useEffect(() => {
    let filtered = auditTrail;

    // Apply filter
    if (filter !== "all") {
      filtered = filtered.filter(entry => entry.action === filter);
    }

    // Apply search
    if (searchTerm) {
      const term = searchTerm.toLowerCase();
      filtered = filtered.filter(entry =>
        entry.action.toLowerCase().includes(term) ||
        entry.actor?.toLowerCase().includes(term) ||
        entry.change_summary?.toLowerCase().includes(term)
      );
    }

    setFilteredTrail(filtered);
  }, [auditTrail, filter, searchTerm]);

  const toggleExpanded = (entryId: string) => {
    const newExpanded = new Set(expandedEntries);
    if (newExpanded.has(entryId)) {
      newExpanded.delete(entryId);
    } else {
      newExpanded.add(entryId);
    }
    setExpandedEntries(newExpanded);
  };

  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp);
    return date.toLocaleString("en-US", {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  };

  const getActionIcon = (action: string) => {
    switch (action.toLowerCase()) {
      case "task_created":
        return "📝";
      case "task_started":
        return "▶️";
      case "task_completed":
        return "✅";
      case "task_failed":
        return "❌";
      case "task_paused":
        return "⏸️";
      case "task_cancelled":
        return "🛑";
      case "task_retried":
        return "🔄";
      case "state_change":
        return "🔄";
      case "quality_check":
        return "🔍";
      case "validation":
        return "✅";
      case "error_occurred":
        return "⚠️";
      case "user_action":
        return "👤";
      case "system_action":
        return "🤖";
      default:
        return "📋";
    }
  };

  const getActionColor = (action: string) => {
    switch (action.toLowerCase()) {
      case "task_created":
      case "task_started":
        return styles.primary;
      case "task_completed":
      case "validation":
        return styles.success;
      case "task_failed":
      case "error_occurred":
        return styles.error;
      case "task_paused":
      case "task_cancelled":
        return styles.warning;
      case "task_retried":
      case "state_change":
        return styles.info;
      default:
        return styles.neutral;
    }
  };

  const getUniqueActions = () => {
    const actions = new Set(auditTrail.map(entry => entry.action));
    return Array.from(actions).sort();
  };

  const renderChangeSummary = (changeSummary: any) => {
    if (!changeSummary) return null;

    if (typeof changeSummary === "string") {
      return <span className={styles.changeSummary}>{changeSummary}</span>;
    }

    if (typeof changeSummary === "object") {
      return (
        <div className={styles.changeSummary}>
          <pre>{JSON.stringify(changeSummary, null, 2)}</pre>
        </div>
      );
    }

    return <span className={styles.changeSummary}>{String(changeSummary)}</span>;
  };

  if (auditTrail.length === 0) {
    return (
      <div className={styles.emptyState}>
        <div className={styles.emptyIcon}>📋</div>
        <h3>No Audit Trail</h3>
        <p>No audit entries found for this task.</p>
      </div>
    );
  }

  return (
    <div className={styles.auditTrailViewer}>
      <div className={styles.header}>
        <h3>Audit Trail</h3>
        <div className={styles.controls}>
          <div className={styles.searchBox}>
            <input
              type="text"
              placeholder="Search audit trail..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className={styles.searchInput}
            />
          </div>
          <select
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            className={styles.filterSelect}
          >
            <option value="all">All Actions</option>
            {getUniqueActions().map(action => (
              <option key={action} value={action}>
                {action.replace(/_/g, " ").replace(/\b\w/g, l => l.toUpperCase())}
              </option>
            ))}
          </select>
        </div>
      </div>

      <div 
        className={styles.timeline}
        style={{ maxHeight: showFullTrail ? "none" : maxHeight }}
      >
        {filteredTrail.map((entry, index) => {
          const isExpanded = expandedEntries.has(entry.id);
          const hasDetails = entry.change_summary && 
            (typeof entry.change_summary === "object" || 
             (typeof entry.change_summary === "string" && entry.change_summary.length > 100));

          return (
            <div key={entry.id} className={styles.timelineItem}>
              <div className={styles.timelineMarker}>
                <div className={`${styles.actionIcon} ${getActionColor(entry.action)}`}>
                  {getActionIcon(entry.action)}
                </div>
                {index < filteredTrail.length - 1 && (
                  <div className={styles.timelineLine}></div>
                )}
              </div>

              <div className={styles.timelineContent}>
                <div className={styles.entryHeader}>
                  <div className={styles.entryInfo}>
                    <h4 className={styles.actionTitle}>
                      {entry.action.replace(/_/g, " ").replace(/\b\w/g, l => l.toUpperCase())}
                    </h4>
                    <div className={styles.entryMeta}>
                      <span className={styles.timestamp}>
                        {formatTimestamp(entry.timestamp)}
                      </span>
                      {entry.actor && (
                        <span className={styles.actor}>
                          by {entry.actor}
                        </span>
                      )}
                    </div>
                  </div>
                  {hasDetails && (
                    <button
                      onClick={() => toggleExpanded(entry.id)}
                      className={styles.expandButton}
                    >
                      {isExpanded ? "▼" : "▶"}
                    </button>
                  )}
                </div>

                {entry.change_summary && (
                  <div className={`${styles.entryDetails} ${isExpanded ? styles.expanded : ""}`}>
                    {renderChangeSummary(entry.change_summary)}
                  </div>
                )}
              </div>
            </div>
          );
        })}
      </div>

      {!showFullTrail && filteredTrail.length > 0 && (
        <div className={styles.footer}>
          <p className={styles.summary}>
            Showing {filteredTrail.length} of {auditTrail.length} audit entries
            {filter !== "all" && ` (filtered by ${filter})`}
            {searchTerm && ` (search: "${searchTerm}")`}
          </p>
        </div>
      )}
    </div>
  );
}