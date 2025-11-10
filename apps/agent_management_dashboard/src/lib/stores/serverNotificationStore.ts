/**
 * Server-Side Notification Store
 * 
 * Temporary in-memory store for notifications received via API.
 * Client-side code polls this to retrieve and store notifications locally.
 * 
 * @author @darianrosebrook
 */

import type { NotificationType } from './notificationStore';

export interface ServerNotification {
  id: string;
  type: NotificationType;
  message: string;
  timestamp: number;
  errorCode?: string;
  errorDetails?: Record<string, unknown>;
  actionUrl?: string;
  actionLabel?: string;
}

// In-memory store (in production, this should be replaced with a database)
const notifications: Map<string, ServerNotification> = new Map();
const MAX_NOTIFICATIONS = 1000;

/**
 * Add a notification to the server store
 */
export function addServerNotification(notification: Omit<ServerNotification, 'id' | 'timestamp'>): string {
  const id = `server-notification-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  const serverNotification: ServerNotification = {
    ...notification,
    id,
    timestamp: Date.now(),
  };

  notifications.set(id, serverNotification);

  // Keep only the most recent notifications
  if (notifications.size > MAX_NOTIFICATIONS) {
    const sorted = Array.from(notifications.entries())
      .sort((a, b) => b[1].timestamp - a[1].timestamp);
    const toRemove = sorted.slice(MAX_NOTIFICATIONS);
    toRemove.forEach(([id]) => notifications.delete(id));
  }

  return id;
}

/**
 * Get notifications since a given timestamp
 */
export function getNotificationsSince(timestamp: number): ServerNotification[] {
  return Array.from(notifications.values())
    .filter(n => n.timestamp > timestamp)
    .sort((a, b) => b.timestamp - a.timestamp);
}

/**
 * Get all notifications
 */
export function getAllNotifications(): ServerNotification[] {
  return Array.from(notifications.values())
    .sort((a, b) => b.timestamp - a.timestamp);
}

/**
 * Delete a notification
 */
export function deleteServerNotification(id: string): boolean {
  return notifications.delete(id);
}

/**
 * Clear all notifications
 */
export function clearAllNotifications(): void {
  notifications.clear();
}

