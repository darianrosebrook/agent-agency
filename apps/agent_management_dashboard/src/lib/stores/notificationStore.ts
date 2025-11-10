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
 * Add a new notification
 */
export function addNotification(notification: Omit<Notification, 'id' | 'timestamp' | 'read'>): void {
  const notifications = getNotifications();
  
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

