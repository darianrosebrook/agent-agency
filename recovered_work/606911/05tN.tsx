"use client";

import React, { useEffect } from "react";
import { NotificationAlert } from "@/types/tts";
import { useNotificationAlerts } from "@/hooks/useTTS";
import styles from "./AttentionAlerts.module.scss";

interface AttentionAlertsProps {
  userName?: string;
  className?: string;
}

export default function AttentionAlerts({
  userName,
  className = "",
}: AttentionAlertsProps) {
  const { alerts, addAlert, playAlert, dismissAlert, clearPlayedAlerts } =
    useNotificationAlerts();

  // Demo: Add some sample alerts for testing (remove in production)
  useEffect(() => {
    if (process.env.NODE_ENV === "development") {
      // Add a sample attention alert after 10 seconds
      const timer = setTimeout(() => {
        addAlert({
          type: "attention",
          message: "System needs your attention",
          userName,
          priority: "medium",
        });
      }, 10000);

      // Add a sample task completion alert after 20 seconds
      const timer2 = setTimeout(() => {
        addAlert({
          type: "task_complete",
          message: "Task processing completed successfully",
          priority: "low",
        });
      }, 20000);

      // Add voicemail-style notifications
      const timer3 = setTimeout(() => {
        addAlert({
          type: "voicemail",
          message: "Code review completed. Found 3 minor issues, all resolved.",
          priority: "medium",
        });
      }, 30000);

      const timer4 = setTimeout(() => {
        addAlert({
          type: "voicemail",
          message: "Security scan finished. No critical vulnerabilities detected.",
          priority: "low",
        });
      }, 45000);

      return () => {
        clearTimeout(timer);
        clearTimeout(timer2);
        clearTimeout(timer3);
        clearTimeout(timer4);
      };
    }
  }, [addAlert, userName]);

  const getAlertIcon = (type: NotificationAlert["type"]) => {
    switch (type) {
      case "attention":
        return "🚨";
      case "voicemail":
        return "📬";
      case "task_complete":
        return "✅";
      case "error":
        return "❌";
      default:
        return "🔔";
    }
  };

  const getAlertTitle = (type: NotificationAlert["type"]) => {
    switch (type) {
      case "attention":
        return "Attention Required";
      case "voicemail":
        return "Voicemail Message";
      case "task_complete":
        return "Task Completed";
      case "error":
        return "System Error";
      default:
        return "Notification";
    }
  };

  const getAlertClass = (
    priority: NotificationAlert["priority"],
    played: boolean
  ) => {
    const classes = [styles.alert];

    if (played) {
      classes.push(styles.played);
    } else {
      switch (priority) {
        case "high":
          classes.push(styles.high);
          break;
        case "medium":
          classes.push(styles.medium);
          break;
        case "low":
          classes.push(styles.low);
          break;
      }
    }

    return classes.join(" ");
  };

  if (alerts.length === 0) {
    return null;
  }

  return (
    <div className={`${styles.container} ${className}`}>
      <div className={styles.header}>
        <h3>Notifications</h3>
        <button
          className={styles.clearButton}
          onClick={clearPlayedAlerts}
          title="Clear played alerts"
        >
          🧹
        </button>
      </div>

      <div className={styles.alertsList}>
        {alerts.map((alert) => (
          <div
            key={alert.id}
            className={getAlertClass(alert.priority, alert.played)}
          >
            <div className={styles.alertContent}>
              <span className={styles.icon}>{getAlertIcon(alert.type)}</span>
              <div className={styles.message}>
                <div className={styles.title}>{getAlertTitle(alert.type)}</div>
                <div className={styles.text}>{alert.message}</div>
                <div className={styles.timestamp}>
                  {new Date(alert.timestamp).toLocaleTimeString()}
                </div>
              </div>
            </div>

            <div className={styles.actions}>
              {!alert.played && (
                <button
                  className={styles.playButton}
                  onClick={() => playAlert(alert.id)}
                  title="Play audio alert"
                >
                  🔊
                </button>
              )}

              <button
                className={styles.dismissButton}
                onClick={() => dismissAlert(alert.id)}
                title="Dismiss alert"
              >
                ✕
              </button>
            </div>
          </div>
        ))}
      </div>

      {/* Global alert trigger buttons for testing */}
      {process.env.NODE_ENV === "development" && (
        <div className={styles.testButtons}>
          <button
            onClick={() =>
              addAlert({
                type: "attention",
                message: "Hey there! System needs attention.",
                userName,
                priority: "high",
              })
            }
          >
            Test Attention Alert
          </button>
          <button
            onClick={() =>
              addAlert({
                type: "voicemail",
                message: "Code review completed. Found 3 minor issues, all resolved.",
                priority: "medium",
              })
            }
          >
            Test Voicemail Alert
          </button>
          <button
            onClick={() =>
              addAlert({
                type: "task_complete",
                message: "Background task completed successfully",
                priority: "low",
              })
            }
          >
            Test Task Alert
          </button>
        </div>
      )}
    </div>
  );
}
