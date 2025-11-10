/**
 * Notification Store
 * 
 * Manages persistent storage and retrieval of toast notifications
 * for viewing in the notifications/activity log page.
 * 
 * @author @darianrosebrook
 */

export type NotificationType = 'error' | 'warning' | 'info' | 'success';

export interface Notification {
  id: string;
  type: NotificationType;
  message: string;
  timestamp: number;
  read: boolean;
  errorCode?: string;
  errorDetails?: Record<string, unknown>;
  actionUrl?: string;
  actionLabel?: string;
}

const STORAGE_KEY = 'agent-agency-notifications';
const MAX_NOTIFICATIONS = 500; // Keep last 500 notifications

/**
 * Get all notifications from storage
 */
export function getNotifications(): Notification[] {
  if (typeof window === 'undefined') {
    return [];
  }

  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (!stored) return [];
    
    const notifications = JSON.parse(stored) as Notification[];
    return notifications.sort((a, b) => b.timestamp - a.timestamp);
  } catch (error) {
    console.error('Failed to load notifications:', error);
    return [];
  }
}

/**
 * Save notifications to storage
 */
function saveNotifications(notifications: Notification[]): void {
  if (typeof window === 'undefined') {
    return;
  }

  try {
    // Keep only the most recent notifications
    const limited = notifications.slice(0, MAX_NOTIFICATIONS);
    localStorage.setItem(STORAGE_KEY, JSON.stringify(limited));
  } catch (error) {
    console.error('Failed to save notifications:', error);
  }
}

/**
 * Check if a notification is a duplicate of an existing one
 * A notification is considered a duplicate if it has the same:
 * - message
 * - type
 * - errorCode (if present)
 * And was created within the last 5 seconds
 */
function isDuplicateNotification(
  newNotification: Omit<Notification, 'id' | 'timestamp' | 'read'>,
  existingNotifications: Notification[]
): boolean {
  const now = Date.now();
  const DEDUPLICATION_WINDOW_MS = 5000; // 5 seconds

  return existingNotifications.some((existing) => {
    // Check if within deduplication window
    const timeDiff = now - existing.timestamp;
    if (timeDiff > DEDUPLICATION_WINDOW_MS) {
      return false; // Too old to be a duplicate
    }

    // Check if message, type, and errorCode match
    if (
      existing.message === newNotification.message &&
      existing.type === newNotification.type &&
      existing.errorCode === newNotification.errorCode
    ) {
      return true; // It's a duplicate
    }

    return false;
  });
}

/**
 * Add a new notification with deduplication
 * Prevents storing duplicate notifications within a 5-second window
 */
export function addNotification(notification: Omit<Notification, 'id' | 'timestamp' | 'read'>): void {
  const notifications = getNotifications();
  
  // Check for duplicates before adding
  if (isDuplicateNotification(notification, notifications)) {
    // Update the count on the existing notification if it's a duplicate
    // Find the most recent matching notification
    const duplicate = notifications.find(
      (n) =>
        n.message === notification.message &&
        n.type === notification.type &&
        n.errorCode === notification.errorCode &&
        Date.now() - n.timestamp < 5000
    );

    if (duplicate) {
      // Update the existing notification's timestamp to keep it fresh
      // and increment a count if we track it
      const updated = notifications.map((n) =>
        n.id === duplicate.id
          ? { ...n, timestamp: Date.now() } // Refresh timestamp
          : n
      );
      saveNotifications(updated);
    }
    return; // Don't add duplicate
  }
  
  const newNotification: Notification = {
    ...notification,
    id: `notification-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
    timestamp: Date.now(),
    read: false,
  };

  notifications.unshift(newNotification);
  saveNotifications(notifications);
}

/**
 * Mark notification as read
 */
export function markNotificationAsRead(id: string): void {
  const notifications = getNotifications();
  const updated = notifications.map(n => 
    n.id === id ? { ...n, read: true } : n
  );
  saveNotifications(updated);
}

/**
 * Mark all notifications as read
 */
export function markAllAsRead(): void {
  const notifications = getNotifications();
  const updated = notifications.map(n => ({ ...n, read: true }));
  saveNotifications(updated);
}

/**
 * Delete a notification
 */
export function deleteNotification(id: string): void {
  const notifications = getNotifications();
  const filtered = notifications.filter(n => n.id !== id);
  saveNotifications(filtered);
}

/**
 * Delete all notifications
 */
export function deleteAllNotifications(): void {
  if (typeof window === 'undefined') {
    return;
  }
  localStorage.removeItem(STORAGE_KEY);
}

/**
 * Get unread notification count
 */
export function getUnreadCount(): number {
  const notifications = getNotifications();
  return notifications.filter(n => !n.read).length;
}

/**
 * Get notifications filtered by type
 */
export function getNotificationsByType(type: NotificationType): Notification[] {
  const notifications = getNotifications();
  return notifications.filter(n => n.type === type);
}

/**
 * Get notifications filtered by read status
 */
export function getNotificationsByReadStatus(read: boolean): Notification[] {
  const notifications = getNotifications();
  return notifications.filter(n => n.read === read);
}

/**
 * Remove duplicate notifications from storage
 * Keeps only the most recent occurrence of each unique notification
 * (based on message, type, and errorCode)
 */
export function deduplicateNotifications(): number {
  const notifications = getNotifications();
  const seen = new Map<string, Notification>();
  const DEDUPLICATION_WINDOW_MS = 5000; // 5 seconds
  
  // Process notifications in reverse chronological order (newest first)
  // Keep the first (newest) occurrence of each duplicate
  const deduplicated: Notification[] = [];
  
  for (const notification of notifications) {
    const key = `${notification.type}:${notification.message}:${notification.errorCode || ''}`;
    const existing = seen.get(key);
    
    if (!existing) {
      // First occurrence - keep it
      seen.set(key, notification);
      deduplicated.push(notification);
    } else {
      // Check if it's within the deduplication window
      const timeDiff = Math.abs(notification.timestamp - existing.timestamp);
      if (timeDiff > DEDUPLICATION_WINDOW_MS) {
        // Outside window - treat as separate occurrence
        seen.set(key, notification);
        deduplicated.push(notification);
      }
      // Otherwise, skip this duplicate
    }
  }
  
  const removedCount = notifications.length - deduplicated.length;
  
  if (removedCount > 0) {
    // Sort by timestamp descending (newest first)
    deduplicated.sort((a, b) => b.timestamp - a.timestamp);
    saveNotifications(deduplicated);
  }
  
  return removedCount;
}

