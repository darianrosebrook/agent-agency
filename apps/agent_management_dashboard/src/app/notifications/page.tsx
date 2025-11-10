"use client";

/**
 * Notifications Page - Activity Log
 *
 * Displays all system notifications, errors, and activity logs.
 * Allows users to review past notifications, mark as read, and take actions.
 *
 * @author @darianrosebrook
 */

import { useState, useEffect, useMemo } from "react";
import {
  Bell,
  AlertCircle,
  AlertTriangle,
  Info,
  CheckCircle,
  X,
  CheckCheck,
  Trash2,
  Filter,
} from "lucide-react";
import {
  getNotifications,
  markNotificationAsRead,
  markAllAsRead,
  deleteNotification,
  deleteAllNotifications,
  getUnreadCount,
  deduplicateNotifications,
  type Notification,
  type NotificationType,
} from "@/lib/stores/notificationStore";
import { toastError, toastWarning, toastInfo, toastSuccess } from "@/lib/utils/toast";
import { cn } from "@/components/primitives/utils";
import styles from "./page.module.scss";

export default function NotificationsPage() {
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [filter, setFilter] = useState<NotificationType | "all">("all");
  const [showRead, setShowRead] = useState(true);
  const [lastNotificationId, setLastNotificationId] = useState<string | null>(null);

  useEffect(() => {
    loadNotifications();
    // Refresh every 5 seconds to catch new notifications
    const interval = setInterval(loadNotifications, 5000);
    return () => clearInterval(interval);
  }, []);

  const loadNotifications = () => {
    const all = getNotifications();
    
    // Check for new notifications and trigger toasts
    if (all.length > 0) {
      const lastNotification = all.find(n => n.id === lastNotificationId);
      const lastTimestamp = lastNotification?.timestamp ?? 0;
      
      const newNotifications = all.filter(n => 
        !n.read && 
        n.timestamp > lastTimestamp
      );
      
      newNotifications.forEach(notification => {
        // Trigger toast for new unread notifications
        switch (notification.type) {
          case 'error':
            toastError(notification.message);
            break;
          case 'warning':
            toastWarning(notification.message);
            break;
          case 'info':
            toastInfo(notification.message);
            break;
          case 'success':
            toastSuccess(notification.message);
            break;
        }
      });
      
      // Update last notification ID to the most recent one
      if (all[0] && (!lastNotificationId || all[0].timestamp > lastTimestamp)) {
        setLastNotificationId(all[0].id);
      }
    }
    
    setNotifications(all);
  };

  const filteredNotifications = useMemo(() => {
    let filtered = notifications;

    if (filter !== "all") {
      filtered = filtered.filter((n) => n.type === filter);
    }

    if (!showRead) {
      filtered = filtered.filter((n) => !n.read);
    }

    return filtered;
  }, [notifications, filter, showRead]);

  const unreadCount = useMemo(() => getUnreadCount(), [notifications]);

  const handleMarkAsRead = (id: string) => {
    markNotificationAsRead(id);
    loadNotifications();
  };

  const handleMarkAllAsRead = () => {
    markAllAsRead();
    loadNotifications();
  };

  const handleDelete = (id: string) => {
    deleteNotification(id);
    loadNotifications();
  };

  const handleDeleteAll = () => {
    if (confirm("Are you sure you want to delete all notifications?")) {
      deleteAllNotifications();
      loadNotifications();
    }
  };

  const handleDeduplicate = () => {
    const removedCount = deduplicateNotifications();
    if (removedCount > 0) {
      toastSuccess(`Removed ${removedCount} duplicate notification${removedCount === 1 ? '' : 's'}`);
      loadNotifications();
    } else {
      toastInfo("No duplicates found");
    }
  };

  const getIcon = (type: NotificationType) => {
    switch (type) {
      case "error":
        return <AlertCircle className={styles.notificationIcon} />;
      case "warning":
        return <AlertTriangle className={styles.notificationIcon} />;
      case "info":
        return <Info className={styles.notificationIcon} />;
      case "success":
        return <CheckCircle className={styles.notificationIcon} />;
    }
  };

  const formatTimestamp = (timestamp: number) => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return "Just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    return date.toLocaleDateString();
  };

  const typeCounts = useMemo(() => {
    return {
      all: notifications.length,
      error: notifications.filter((n) => n.type === "error").length,
      warning: notifications.filter((n) => n.type === "warning").length,
      info: notifications.filter((n) => n.type === "info").length,
      success: notifications.filter((n) => n.type === "success").length,
    };
  }, [notifications]);

  return (
    <div className={styles.notificationsPage}>
      <div className={styles.notificationsHeader}>
        <div className={styles.headerContent}>
          <div className={styles.headerTitleSection}>
            <Bell className={styles.headerIcon} />
            <div>
              <h1 className={styles.notificationsTitle}>Notifications</h1>
              <p className={styles.notificationsDescription}>
                View system notifications, errors, and activity logs
              </p>
            </div>
          </div>
          {unreadCount > 0 && (
            <div className={styles.unreadBadge}>
              {unreadCount} unread
            </div>
          )}
        </div>

        <div className={styles.headerActions}>
          {notifications.length > 0 && (
            <button
              onClick={handleDeduplicate}
              className={styles.actionButton}
              type="button"
              title="Remove duplicate notifications"
            >
              <Filter className={styles.actionButtonIcon} />
              Remove duplicates
            </button>
          )}
          {unreadCount > 0 && (
            <button
              onClick={handleMarkAllAsRead}
              className={styles.actionButton}
              type="button"
            >
              <CheckCheck className={styles.actionButtonIcon} />
              Mark all as read
            </button>
          )}
          {notifications.length > 0 && (
            <button
              onClick={handleDeleteAll}
              className={cn(styles.actionButton, styles.actionButtonDanger)}
              type="button"
            >
              <Trash2 className={styles.actionButtonIcon} />
              Clear all
            </button>
          )}
        </div>
      </div>

      <div className={styles.notificationsContent}>
        {/* Filters */}
        <div className={styles.filters}>
          <div className={styles.filterGroup}>
            <Filter className={styles.filterIcon} />
            <span className={styles.filterLabel}>Type:</span>
            <button
              onClick={() => setFilter("all")}
              className={cn(
                styles.filterButton,
                filter === "all" && styles.filterButtonActive
              )}
              type="button"
            >
              All ({typeCounts.all})
            </button>
            <button
              onClick={() => setFilter("error")}
              className={cn(
                styles.filterButton,
                filter === "error" && styles.filterButtonActive,
                styles.filterButtonError
              )}
              type="button"
            >
              Errors ({typeCounts.error})
            </button>
            <button
              onClick={() => setFilter("warning")}
              className={cn(
                styles.filterButton,
                filter === "warning" && styles.filterButtonActive,
                styles.filterButtonWarning
              )}
              type="button"
            >
              Warnings ({typeCounts.warning})
            </button>
            <button
              onClick={() => setFilter("info")}
              className={cn(
                styles.filterButton,
                filter === "info" && styles.filterButtonActive,
                styles.filterButtonInfo
              )}
              type="button"
            >
              Info ({typeCounts.info})
            </button>
            <button
              onClick={() => setFilter("success")}
              className={cn(
                styles.filterButton,
                filter === "success" && styles.filterButtonActive,
                styles.filterButtonSuccess
              )}
              type="button"
            >
              Success ({typeCounts.success})
            </button>
          </div>
          <div className={styles.filterGroup}>
            <button
              onClick={() => setShowRead(!showRead)}
              className={cn(
                styles.filterButton,
                !showRead && styles.filterButtonActive
              )}
              type="button"
            >
              {showRead ? "Hide read" : "Show read"}
            </button>
          </div>
        </div>

        {/* Notifications List */}
        <div className={styles.notificationsList}>
          {filteredNotifications.length === 0 ? (
            <div className={styles.emptyState}>
              <Bell className={styles.emptyStateIcon} />
              <h3 className={styles.emptyStateTitle}>No notifications</h3>
              <p className={styles.emptyStateDescription}>
                {filter !== "all" || !showRead
                  ? "No notifications match your filters"
                  : "You're all caught up! No notifications to display."}
              </p>
            </div>
          ) : (
            filteredNotifications.map((notification) => (
              <div
                key={notification.id}
                className={cn(
                  styles.notificationItem,
                  !notification.read && styles.notificationItemUnread,
                  styles[`notificationItem${notification.type.charAt(0).toUpperCase() + notification.type.slice(1)}`]
                )}
              >
                <div className={styles.notificationContent}>
                  <div className={styles.notificationIconContainer}>
                    {getIcon(notification.type)}
                  </div>
                  <div className={styles.notificationText}>
                    <p className={styles.notificationMessage}>
                      {notification.message}
                    </p>
                    <div className={styles.notificationMeta}>
                      <span className={styles.notificationTimestamp}>
                        {formatTimestamp(notification.timestamp)}
                      </span>
                      {notification.errorCode && (
                        <span className={styles.notificationErrorCode}>
                          {notification.errorCode}
                        </span>
                      )}
                    </div>
                    {notification.errorDetails && (
                      <details className={styles.notificationDetails}>
                        <summary className={styles.notificationDetailsSummary}>
                          View details
                        </summary>
                        <pre className={styles.notificationDetailsPre}>
                          {JSON.stringify(notification.errorDetails, null, 2)}
                        </pre>
                      </details>
                    )}
                  </div>
                </div>
                <div className={styles.notificationActions}>
                  {!notification.read && (
                    <button
                      onClick={() => handleMarkAsRead(notification.id)}
                      className={styles.notificationActionButton}
                      type="button"
                      title="Mark as read"
                    >
                      <CheckCheck className={styles.notificationActionIcon} />
                    </button>
                  )}
                  <button
                    onClick={() => handleDelete(notification.id)}
                    className={styles.notificationActionButton}
                    type="button"
                    title="Delete"
                  >
                    <X className={styles.notificationActionIcon} />
                  </button>
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}

