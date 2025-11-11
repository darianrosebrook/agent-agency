/**
 * Server-Side Notification Store
 *
 * Temporary in-memory store for notifications received via API.
 * Client-side code polls this to retrieve and store notifications locally.
 *
 * @author @darianrosebrook
 */

import type { NotificationType } from "./notificationStore";

export interface ServerNotification {
  id: string;
  type: NotificationType;
  message: string;
  timestamp: number;
  errorCode?: string;
  errorDetails?: Record<string, unknown>;
  actionUrl?: string;
  actionLabel?: string;
  voicemailAudioUrl?: string; // URL to voicemail audio file
  voicemailTranscription?: string; // Transcription of the voicemail
}

// In-memory store (in production, this should be replaced with a database)
const notifications: Map<string, ServerNotification> = new Map();
const MAX_NOTIFICATIONS = 1000;
const DEDUPLICATION_WINDOW_MS = 30000; // 30 seconds - longer window for server-side

/**
 * Generate a deduplication key for a notification
 */
function getDeduplicationKey(
  notification: Omit<ServerNotification, "id" | "timestamp">
): string {
  return `${notification.type}:${notification.message}:${
    notification.errorCode || ""
  }`;
}

/**
 * Check if a notification is a duplicate
 */
function isDuplicateNotification(
  notification: Omit<ServerNotification, "id" | "timestamp">,
  existingNotifications: ServerNotification[]
): boolean {
  const now = Date.now();
  const key = getDeduplicationKey(notification);

  return existingNotifications.some((existing) => {
    const existingKey = getDeduplicationKey(existing);

    // Check if keys match
    if (existingKey !== key) {
      return false;
    }

    // Check if within deduplication window
    const timeDiff = now - existing.timestamp;
    return timeDiff <= DEDUPLICATION_WINDOW_MS;
  });
}

/**
 * Add a notification to the server store with deduplication
 */
export function addServerNotification(
  notification: Omit<ServerNotification, "id" | "timestamp">
): string {
  const allNotifications = Array.from(notifications.values());

  // Check for duplicates before adding
  if (isDuplicateNotification(notification, allNotifications)) {
    // Find the existing duplicate and return its ID
    const duplicate = allNotifications.find((n) => {
      const key = getDeduplicationKey(notification);
      const existingKey = getDeduplicationKey(n);
      const timeDiff = Date.now() - n.timestamp;

      return existingKey === key && timeDiff <= DEDUPLICATION_WINDOW_MS;
    });

    if (duplicate) {
      // Update timestamp to keep it fresh, but don't create a new notification
      const updated: ServerNotification = {
        ...duplicate,
        timestamp: Date.now(),
      };
      notifications.set(duplicate.id, updated);
      return duplicate.id;
    }
  }

  const id = `server-notification-${Date.now()}-${Math.random()
    .toString(36)
    .substr(2, 9)}`;
  const serverNotification: ServerNotification = {
    ...notification,
    id,
    timestamp: Date.now(),
  };

  notifications.set(id, serverNotification);

  // Keep only the most recent notifications
  if (notifications.size > MAX_NOTIFICATIONS) {
    const sorted = Array.from(notifications.entries()).sort(
      (a, b) => b[1].timestamp - a[1].timestamp
    );
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
    .filter((n) => n.timestamp > timestamp)
    .sort((a, b) => b.timestamp - a.timestamp);
}

/**
 * Get all notifications
 */
export function getAllNotifications(): ServerNotification[] {
  return Array.from(notifications.values()).sort(
    (a, b) => b.timestamp - a.timestamp
  );
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
