/**
 * Notification Sync Hook
 *
 * Polls server for new notifications and syncs them to client-side store.
 * Triggers toast notifications for new notifications.
 *
 * @author @darianrosebrook
 */

import { useEffect, useRef } from 'react';
import { addNotification } from '@/lib/stores/notificationStore';
import { toastError, toastWarning, toastInfo, toastSuccess } from '@/lib/utils/toast';
import { apiGet } from '@/lib/utils/api';

interface ServerNotification {
  id: string;
  type: 'error' | 'warning' | 'info' | 'success';
  message: string;
  timestamp: number;
  errorCode?: string;
  errorDetails?: Record<string, unknown>;
  actionUrl?: string;
  actionLabel?: string;
}

interface PollResponse {
  success: boolean;
  notifications: ServerNotification[];
  count: number;
}

/**
 * Hook to sync server notifications to client store
 * Polls for new notifications and triggers toasts
 */
export function useNotificationSync(enabled: boolean = true) {
  const lastSyncTimestamp = useRef<number>(Date.now());
  const processedIds = useRef<Set<string>>(new Set());

  useEffect(() => {
    if (!enabled) return;

    const syncNotifications = async () => {
      try {
        const response = await apiGet<PollResponse>(
          `/api/notifications/poll?since=${lastSyncTimestamp.current}`,
          { showToast: false } // Don't show toast for polling errors
        );

        if (response.success && response.notifications.length > 0) {
          // Process new notifications
          response.notifications.forEach((notification) => {
            // Skip if already processed
            if (processedIds.current.has(notification.id)) {
              return;
            }

            // Add to client-side store
            addNotification({
              type: notification.type,
              message: notification.message,
              errorCode: notification.errorCode,
              errorDetails: notification.errorDetails,
              actionUrl: notification.actionUrl,
              actionLabel: notification.actionLabel,
            });

            // Trigger toast notification
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

            // Mark as processed
            processedIds.current.add(notification.id);

            // Update last sync timestamp
            if (notification.timestamp > lastSyncTimestamp.current) {
              lastSyncTimestamp.current = notification.timestamp;
            }
          });
        }
      } catch (error) {
        // Silently fail - polling errors shouldn't disrupt the UI
        console.debug('Notification sync error:', error);
      }
    };

    // Initial sync
    syncNotifications();

    // Poll every 3 seconds for new notifications
    const interval = setInterval(syncNotifications, 3000);

    return () => clearInterval(interval);
  }, [enabled]);
}

