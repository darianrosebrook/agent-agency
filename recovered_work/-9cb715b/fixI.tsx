"use client";

import React, { useState, useEffect } from "react";
import { NotificationAlert } from "@/types/tts";
import { useNotificationAlerts, triggerGlobalAlert } from "@/hooks/useTTS";
import styles from "./VoicemailHistory.module.scss";

interface VoicemailHistoryProps {
  className?: string;
}

export default function VoicemailHistory({
  className = "",
}: VoicemailHistoryProps) {
  const { alerts, playAlert, dismissAlert, clearPlayedAlerts } =
    useNotificationAlerts();

  // Filter to only show voicemail-type alerts
  const voicemails = alerts.filter((alert) => alert.type === "voicemail");

  // Add some demo voicemails for testing
  useEffect(() => {
    if (process.env.NODE_ENV === "development" && voicemails.length === 0) {
      // Add some sample voicemails after a delay
      const timer1 = setTimeout(() => {
        triggerGlobalAlert({
          type: "voicemail",
          message: "Security scan completed. Found 2 low-risk issues that have been auto-resolved.",
          priority: "medium",
        });
      }, 5000);

      const timer2 = setTimeout(() => {
        triggerGlobalAlert({
          type: "voicemail",
          message: "Database optimization finished. Performance improved by 15%.",
          priority: "low",
        });
      }, 15000);

      const timer3 = setTimeout(() => {
        triggerGlobalAlert({
          type: "voicemail",
          message: "Code review completed. All critical issues resolved. Ready for deployment.",
          priority: "high",
        });
      }, 25000);

      return () => {
        clearTimeout(timer1);
        clearTimeout(timer2);
        clearTimeout(timer3);
      };
    }
  }, [voicemails.length]);

  const getPriorityBadge = (priority: NotificationAlert["priority"]) => {
    const classes = [styles.priorityBadge];

    switch (priority) {
      case "high":
        classes.push(styles.high);
        return { className: classes.join(" "), label: "High" };
      case "medium":
        classes.push(styles.medium);
        return { className: classes.join(" "), label: "Medium" };
      case "low":
        classes.push(styles.low);
        return { className: classes.join(" "), label: "Low" };
      default:
        return { className: classes.join(" "), label: "Normal" };
    }
  };

  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / (1000 * 60));
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffMins < 1) return "Just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;

    return date.toLocaleDateString();
  };

  if (voicemails.length === 0) {
    return (
      <div className={`${styles.container} ${className}`}>
        <div className={styles.header}>
          <h3>📬 Voicemail Inbox</h3>
        </div>
        <div className={styles.emptyState}>
          <div className={styles.emptyIcon}>📭</div>
          <h4>No voicemails yet</h4>
          <p>
            System notifications and task completion messages will appear here as
            voicemails.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className={`${styles.container} ${className}`}>
      <div className={styles.header}>
        <h3>📬 Voicemail Inbox ({voicemails.length})</h3>
        <div className={styles.actions}>
          <button
            className={styles.clearButton}
            onClick={clearPlayedAlerts}
            title="Clear all played voicemails"
          >
            🧹 Clear Played
          </button>
        </div>
      </div>

      <div className={styles.voicemailList}>
        {voicemails.map((voicemail) => {
          const priorityBadge = getPriorityBadge(voicemail.priority);

          return (
            <div
              key={voicemail.id}
              className={`${styles.voicemailItem} ${
                voicemail.played ? styles.played : styles.unplayed
              }`}
            >
              <div className={styles.voicemailContent}>
                <div className={styles.voicemailHeader}>
                  <span className={priorityBadge.className}>
                    {priorityBadge.label} Priority
                  </span>
                  <span className={styles.timestamp}>
                    {formatTimestamp(voicemail.timestamp)}
                  </span>
                </div>

                <div className={styles.message}>
                  {voicemail.message}
                </div>

                <div className={styles.playbackStatus}>
                  {voicemail.played ? (
                    <span className={styles.playedStatus}>
                      ✅ Played {formatTimestamp(voicemail.timestamp)}
                    </span>
                  ) : (
                    <span className={styles.unplayedStatus}>
                      🔴 New voicemail - Click play to listen
                    </span>
                  )}
                </div>
              </div>

              <div className={styles.voicemailActions}>
                {!voicemail.played && (
                  <button
                    className={styles.playButton}
                    onClick={() => playAlert(voicemail.id)}
                    title="Play voicemail audio"
                  >
                    🔊 Play
                  </button>
                )}

                <button
                  className={styles.dismissButton}
                  onClick={() => dismissAlert(voicemail.id)}
                  title="Delete voicemail"
                >
                  🗑️ Delete
                </button>
              </div>
            </div>
          );
        })}
      </div>

      {/* Development test buttons */}
      {process.env.NODE_ENV === "development" && (
        <div className={styles.testSection}>
          <h4>Test Voicemails</h4>
          <div className={styles.testButtons}>
            <button
              onClick={() =>
                triggerGlobalAlert({
                  type: "voicemail",
                  message: "System backup completed successfully. All data is now secure.",
                  priority: "low",
                })
              }
            >
              Add Backup Complete Voicemail
            </button>
            <button
              onClick={() =>
                triggerGlobalAlert({
                  type: "voicemail",
                  message: "Performance optimization finished. Response time improved by 25%.",
                  priority: "medium",
                })
              }
            >
              Add Performance Voicemail
            </button>
            <button
              onClick={() =>
                triggerGlobalAlert({
                  type: "voicemail",
                  message: "Critical security update applied. System is now protected against recent threats.",
                  priority: "high",
                })
              }
            >
              Add Security Update Voicemail
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
