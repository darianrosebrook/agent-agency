/**
 * Webhook Notifications Component
 * Real-time notifications for task events and system alerts
 *
 * @author @darianrosebrook
 */

"use client";

import React, { useState, useEffect, useRef } from 'react';
import { useWebhookHandler } from '@/hooks/useWebhookHandler';
import { useErrorHandler } from '@/lib/error-handling';
import { Bell, X, CheckCircle, AlertCircle, Info, Clock } from 'lucide-react';
import styles from './WebhookNotifications.module.scss';

interface WebhookNotification {
  id: string;
  type: 'task_completed' | 'task_failed' | 'task_started' | 'system_alert' | 'info';
  title: string;
  message: string;
  timestamp: Date;
  taskId?: string;
  priority: 'low' | 'medium' | 'high' | 'critical';
  read: boolean;
  actionUrl?: string;
}

interface WebhookNotificationsProps {
  className?: string;
  maxNotifications?: number;
  autoHideDelay?: number;
  position?: 'top-right' | 'top-left' | 'bottom-right' | 'bottom-left';
}

export function WebhookNotifications({
  className = '',
  maxNotifications = 5,
  autoHideDelay = 5000,
  position = 'top-right'
}: WebhookNotificationsProps) {
  const [notifications, setNotifications] = useState<WebhookNotification[]>([]);
  const [isExpanded, setIsExpanded] = useState(false);
  const [unreadCount, setUnreadCount] = useState(0);

  const { handleError } = useErrorHandler();
  const webhookHandler = useWebhookHandler({
    url: '/api/webhooks/notifications',
    rateLimit: { maxRequests: 20, windowMs: 60000 },
  });

  const timeoutsRef = useRef<Map<string, NodeJS.Timeout>>(new Map());

  // Handle incoming webhooks
  useEffect(() => {
    const handleWebhook = async (webhookData: any) => {
      try {
        const notification = createNotificationFromWebhook(webhookData);
        addNotification(notification);
      } catch (error) {
        handleError(error);
      }
    };

    // Simulate webhook events for demonstration
    const simulateWebhook = (type: string, data: any) => {
      handleWebhook({
        id: `webhook-${Date.now()}`,
        type,
        payload: data,
        timestamp: new Date().toISOString(),
      });
    };

    // Simulate some initial notifications
    setTimeout(() => simulateWebhook('task_completed', {
      taskId: 'task-001',
      title: 'Dashboard Implementation',
      message: 'Apple Silicon monitoring dashboard completed successfully'
    }), 2000);

    setTimeout(() => simulateWebhook('task_started', {
      taskId: 'task-002',
      title: 'API Optimization',
      message: 'Starting API response time optimization'
    }), 5000);

    setTimeout(() => simulateWebhook('system_alert', {
      title: 'High Memory Usage',
      message: 'System memory usage is above 80%',
      priority: 'high'
    }), 8000);

  }, [handleError]);

  const createNotificationFromWebhook = (webhookData: any): WebhookNotification => {
    const { type, payload, timestamp, id: webhookId } = webhookData;

    // Generate unique ID using webhook ID and timestamp for better uniqueness
    const uniqueId = webhookId || `notification-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

    switch (type) {
      case 'task_completed':
        return {
          id: uniqueId,
          type: 'task_completed',
          title: 'Task Completed',
          message: `${payload.title} has been completed`,
          timestamp: new Date(timestamp),
          taskId: payload.taskId,
          priority: 'medium',
          read: false,
          actionUrl: `/tasks/${payload.taskId}`,
        };

      case 'task_failed':
        return {
          id: uniqueId,
          type: 'task_failed',
          title: 'Task Failed',
          message: `${payload.title} failed to complete`,
          timestamp: new Date(timestamp),
          taskId: payload.taskId,
          priority: 'high',
          read: false,
          actionUrl: `/tasks/${payload.taskId}`,
        };

      case 'task_started':
        return {
          id: uniqueId,
          type: 'task_started',
          title: 'Task Started',
          message: `${payload.title} has begun execution`,
          timestamp: new Date(timestamp),
          taskId: payload.taskId,
          priority: 'low',
          read: false,
          actionUrl: `/tasks/${payload.taskId}`,
        };

      case 'system_alert':
        return {
          id: uniqueId,
          type: 'system_alert',
          title: payload.title,
          message: payload.message,
          timestamp: new Date(timestamp),
          priority: payload.priority || 'medium',
          read: false,
        };

      default:
        return {
          id: uniqueId,
          type: 'info',
          title: 'Notification',
          message: payload.message || 'New notification received',
          timestamp: new Date(timestamp),
          priority: 'low',
          read: false,
        };
    }
  };

  const addNotification = (notification: WebhookNotification) => {
    setNotifications(prev => {
      const updated = [notification, ...prev].slice(0, maxNotifications);
      return updated;
    });

    // Auto-hide after delay for non-critical notifications
    if (notification.priority !== 'critical') {
      const timeout = setTimeout(() => {
        dismissNotification(notification.id);
      }, autoHideDelay);

      timeoutsRef.current.set(notification.id, timeout);
    }
  };

  const dismissNotification = (id: string) => {
    setNotifications(prev => prev.filter(n => n.id !== id));

    // Clear timeout if it exists
    const timeout = timeoutsRef.current.get(id);
    if (timeout) {
      clearTimeout(timeout);
      timeoutsRef.current.delete(id);
    }
  };

  const markAsRead = (id: string) => {
    setNotifications(prev =>
      prev.map(n =>
        n.id === id ? { ...n, read: true } : n
      )
    );
  };

  const clearAllNotifications = () => {
    setNotifications([]);
    // Clear all timeouts
    timeoutsRef.current.forEach(timeout => clearTimeout(timeout));
    timeoutsRef.current.clear();
  };

  // Update unread count
  useEffect(() => {
    const count = notifications.filter(n => !n.read).length;
    setUnreadCount(count);
  }, [notifications]);

  // Cleanup timeouts on unmount
  useEffect(() => {
    return () => {
      timeoutsRef.current.forEach(timeout => clearTimeout(timeout));
    };
  }, []);

  const getNotificationIcon = (type: WebhookNotification['type']) => {
    switch (type) {
      case 'task_completed':
        return <CheckCircle size={20} className={styles.successIcon} />;
      case 'task_failed':
        return <AlertCircle size={20} className={styles.errorIcon} />;
      case 'task_started':
        return <Clock size={20} className={styles.infoIcon} />;
      case 'system_alert':
        return <AlertCircle size={20} className={styles.warningIcon} />;
      default:
        return <Info size={20} className={styles.infoIcon} />;
    }
  };

  const getPriorityClass = (priority: WebhookNotification['priority']) => {
    switch (priority) {
      case 'critical':
        return styles.critical;
      case 'high':
        return styles.high;
      case 'medium':
        return styles.medium;
      case 'low':
        return styles.low;
      default:
        return styles.medium;
    }
  };

  const visibleNotifications = isExpanded ? notifications : notifications.slice(0, 3);

  return (
    <div className={`${styles.webhookNotifications} ${styles[position]} ${className}`}>
      {/* Notification Bell */}
      <div className={styles.notificationBell}>
        <button
          onClick={() => setIsExpanded(!isExpanded)}
          className={`${styles.bellButton} ${unreadCount > 0 ? styles.hasUnread : ''}`}
          aria-label={`${unreadCount} unread notifications`}
        >
          <Bell size={20} />
          {unreadCount > 0 && (
            <span className={styles.unreadBadge}>
              {unreadCount > 99 ? '99+' : unreadCount}
            </span>
          )}
        </button>

        {notifications.length > 0 && (
          <div className={styles.bellActions}>
            <button
              onClick={clearAllNotifications}
              className={styles.clearAllButton}
              aria-label="Clear all notifications"
            >
              Clear All
            </button>
          </div>
        )}
      </div>

      {/* Notifications Panel */}
      {visibleNotifications.length > 0 && (
        <div className={`${styles.notificationsPanel} ${isExpanded ? styles.expanded : ''}`}>
          <div className={styles.notificationsHeader}>
            <h3>Notifications</h3>
            {notifications.length > visibleNotifications.length && (
              <button
                onClick={() => setIsExpanded(true)}
                className={styles.expandButton}
              >
                +{notifications.length - visibleNotifications.length} more
              </button>
            )}
          </div>

          <div className={styles.notificationsList}>
            {visibleNotifications.map((notification) => (
              <div
                key={notification.id}
                className={`${styles.notificationItem} ${getPriorityClass(notification.priority)} ${notification.read ? styles.read : styles.unread}`}
                onClick={() => markAsRead(notification.id)}
              >
                <div className={styles.notificationIcon}>
                  {getNotificationIcon(notification.type)}
                </div>

                <div className={styles.notificationContent}>
                  <div className={styles.notificationTitle}>
                    {notification.title}
                  </div>
                  <div className={styles.notificationMessage}>
                    {notification.message}
                  </div>
                  <div className={styles.notificationMeta}>
                    <span className={styles.timestamp}>
                      {notification.timestamp.toLocaleTimeString()}
                    </span>
                    {notification.taskId && (
                      <span className={styles.taskId}>
                        Task {notification.taskId}
                      </span>
                    )}
                  </div>
                </div>

                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    dismissNotification(notification.id);
                  }}
                  className={styles.dismissButton}
                  aria-label="Dismiss notification"
                >
                  <X size={16} />
                </button>
              </div>
            ))}
          </div>

          {isExpanded && (
            <div className={styles.notificationsFooter}>
              <button
                onClick={() => setIsExpanded(false)}
                className={styles.collapseButton}
              >
                Show Less
              </button>
            </div>
          )}
        </div>
      )}

      {/* Webhook Connection Status */}
      <div className={styles.webhookStatus}>
        <div className={`${styles.statusIndicator} ${webhookHandler.isConnected ? styles.connected : styles.disconnected}`}>
          {webhookHandler.isConnected ? '🟢' : '🔴'}
        </div>
        <div className={styles.statusText}>
          {webhookHandler.isConnected ? 'Live' : 'Offline'}
        </div>
        {webhookHandler.rateLimited && (
          <div className={styles.rateLimitWarning}>
            Rate limited
          </div>
        )}
      </div>
    </div>
  );
}

// Compact version for embedding in headers
export function NotificationBell({ className = '' }: { className?: string }) {
  return (
    <WebhookNotifications
      className={`${styles.compact} ${className}`}
      maxNotifications={3}
      autoHideDelay={3000}
      position="top-right"
    />
  );
}
