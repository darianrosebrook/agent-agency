/**
 * Webhook Handler Hook
 * Manages webhook connections with rate limiting and connection pooling
 *
 * @author @darianrosebrook
 */

"use client";

import { useEffect, useRef, useState, useCallback } from 'react';

export interface WebhookMessage {
  id: string;
  type: string;
  payload: any;
  timestamp: string;
  source: string;
  signature?: string;
}

export interface WebhookConfig {
  url: string;
  secret?: string;
  retryAttempts: number;
  retryDelay: number;
  rateLimit: {
    maxRequests: number;
    windowMs: number;
  };
}

export interface WebhookState {
  isConnected: boolean;
  connectionState: 'idle' | 'connecting' | 'connected' | 'disconnected' | 'error';
  lastMessage?: WebhookMessage;
  error?: string;
  messageCount: number;
  rateLimited: boolean;
  retryCount: number;
}

export interface UseWebhookHandlerReturn extends WebhookState {
  connect: () => void;
  disconnect: () => void;
  sendWebhook: (message: Partial<WebhookMessage>) => Promise<boolean>;
  clearError: () => void;
}

/**
 * Webhook handler with rate limiting and connection pooling
 * Prevents DDoS attacks on webhook endpoints
 */
export function useWebhookHandler(
  config: Partial<WebhookConfig> = {}
): UseWebhookHandlerReturn {
  const defaultConfig: WebhookConfig = {
    url: '/api/webhooks',
    retryAttempts: 3,
    retryDelay: 1000,
    rateLimit: {
      maxRequests: 50, // 50 requests per window
      windowMs: 60000, // 1 minute window
    },
    ...config,
  };

  const [state, setState] = useState<WebhookState>({
    isConnected: false,
    connectionState: 'idle',
    messageCount: 0,
    rateLimited: false,
    retryCount: 0,
  });

  const webhookRef = useRef<{
    config: WebhookConfig;
    requestQueue: Array<{ message: Partial<WebhookMessage>; resolve: (value: boolean) => void; reject: (error: Error) => void }>;
    isProcessing: boolean;
    abortController?: AbortController;
  }>({
    config: defaultConfig,
    requestQueue: [],
    isProcessing: false,
  });

  // Rate limiting state
  const rateLimitRef = useRef<{
    requests: number[];
    blockedUntil?: number;
  }>({
    requests: [],
  });

  // Check if rate limited
  const isRateLimited = useCallback(() => {
    const now = Date.now();
    const { requests, blockedUntil } = rateLimitRef.current;

    // Check if currently blocked
    if (blockedUntil && now < blockedUntil) {
      return true;
    }

    // Clean old requests
    const windowStart = now - defaultConfig.rateLimit.windowMs;
    rateLimitRef.current.requests = requests.filter(time => time > windowStart);

    // Check if over limit
    if (rateLimitRef.current.requests.length >= defaultConfig.rateLimit.maxRequests) {
      rateLimitRef.current.blockedUntil = now + defaultConfig.rateLimit.windowMs;
      setState(prev => ({ ...prev, rateLimited: true }));
      return true;
    }

    return false;
  }, []);

  // Record a request for rate limiting
  const recordRequest = useCallback(() => {
    const now = Date.now();
    rateLimitRef.current.requests.push(now);
    setState(prev => ({ ...prev, rateLimited: false }));
  }, []);

  // Process queued webhook requests
  const processQueue = useCallback(async () => {
    if (webhookRef.current.isProcessing || webhookRef.current.requestQueue.length === 0) {
      return;
    }

    webhookRef.current.isProcessing = true;

    while (webhookRef.current.requestQueue.length > 0) {
      const { message, resolve, reject } = webhookRef.current.requestQueue.shift()!;

      try {
        if (isRateLimited()) {
          reject(new Error('Rate limit exceeded'));
          continue;
        }

        const success = await sendWebhookRequest(message);
        resolve(success);
      } catch (error) {
        reject(error as Error);
      }

      // Small delay between requests to prevent overwhelming
      await new Promise(resolve => setTimeout(resolve, 100));
    }

    webhookRef.current.isProcessing = false;
  }, [isRateLimited]);

  // Send individual webhook request
  const sendWebhookRequest = useCallback(async (message: Partial<WebhookMessage>): Promise<boolean> => {
    const controller = new AbortController();
    webhookRef.current.abortController = controller;

    const fullMessage: WebhookMessage = {
      id: message.id || crypto.randomUUID(),
      type: message.type || 'unknown',
      payload: message.payload || {},
      timestamp: message.timestamp || new Date().toISOString(),
      source: message.source || 'web-dashboard',
      signature: message.signature,
      ...message,
    };

    // Add signature if secret is configured
    if (defaultConfig.secret && !fullMessage.signature) {
      const encoder = new TextEncoder();
      const data = encoder.encode(JSON.stringify(fullMessage));
      const key = await crypto.subtle.importKey(
        'raw',
        encoder.encode(defaultConfig.secret),
        { name: 'HMAC', hash: 'SHA-256' },
        false,
        ['sign']
      );
      const signature = await crypto.subtle.sign('HMAC', key, data);
      fullMessage.signature = btoa(String.fromCharCode(...new Uint8Array(signature)));
    }

    let lastError: Error | null = null;

    for (let attempt = 0; attempt <= defaultConfig.retryAttempts; attempt++) {
      try {
        recordRequest();

        const response = await fetch(defaultConfig.url, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'X-Webhook-Source': fullMessage.source,
            'X-Webhook-ID': fullMessage.id,
            ...(fullMessage.signature && { 'X-Webhook-Signature': fullMessage.signature }),
          },
          body: JSON.stringify(fullMessage),
          signal: controller.signal,
        });

        if (response.ok) {
          setState(prev => ({
            ...prev,
            lastMessage: fullMessage,
            messageCount: prev.messageCount + 1,
            connectionState: 'connected',
            isConnected: true,
            error: undefined,
            retryCount: 0,
          }));
          return true;
        } else {
          throw new Error(`Webhook failed: ${response.status} ${response.statusText}`);
        }

      } catch (error) {
        lastError = error as Error;

        if (controller.signal.aborted) {
          throw new Error('Webhook request aborted');
        }

        // Don't retry on client errors (4xx)
        if (lastError.message.includes('4')) {
          break;
        }

        // Exponential backoff for retries
        if (attempt < defaultConfig.retryAttempts) {
          const delay = defaultConfig.retryDelay * Math.pow(2, attempt);
          await new Promise(resolve => setTimeout(resolve, delay));
        }
      }
    }

    setState(prev => ({
      ...prev,
      connectionState: 'error',
      error: lastError?.message || 'Webhook request failed',
      retryCount: prev.retryCount + 1,
    }));

    throw lastError || new Error('Webhook request failed');
  }, [defaultConfig, recordRequest]);

  // Public API methods
  const connect = useCallback(() => {
    setState(prev => ({ ...prev, connectionState: 'connecting' }));

    // Test connection with a ping
    sendWebhook({
      type: 'ping',
      payload: { timestamp: Date.now() },
    }).then(success => {
      setState(prev => ({
        ...prev,
        connectionState: success ? 'connected' : 'error',
        isConnected: success,
      }));
    }).catch(() => {
      setState(prev => ({
        ...prev,
        connectionState: 'error',
        isConnected: false,
      }));
    });
  }, []);

  const disconnect = useCallback(() => {
    if (webhookRef.current.abortController) {
      webhookRef.current.abortController.abort();
    }

    webhookRef.current.requestQueue = [];
    webhookRef.current.isProcessing = false;

    setState(prev => ({
      ...prev,
      connectionState: 'disconnected',
      isConnected: false,
    }));
  }, []);

  const sendWebhook = useCallback(async (message: Partial<WebhookMessage>): Promise<boolean> => {
    return new Promise((resolve, reject) => {
      webhookRef.current.requestQueue.push({ message, resolve, reject });
      processQueue();
    });
  }, [processQueue]);

  const clearError = useCallback(() => {
    setState(prev => ({ ...prev, error: undefined }));
  }, []);

  // Auto-cleanup on unmount
  useEffect(() => {
    return () => {
      disconnect();
    };
  }, [disconnect]);

  return {
    ...state,
    connect,
    disconnect,
    sendWebhook,
    clearError,
  };
}

/**
 * Hook for handling external webhook callbacks
 */
export function useWebhookCallbacks() {
  const [callbacks, setCallbacks] = useState<Map<string, (message: WebhookMessage) => void>>(new Map());

  const registerCallback = useCallback((type: string, callback: (message: WebhookMessage) => void) => {
    setCallbacks(prev => {
      const newCallbacks = new Map(prev);
      newCallbacks.set(type, callback);
      return newCallbacks;
    });
  }, []);

  const unregisterCallback = useCallback((type: string) => {
    setCallbacks(prev => {
      const newCallbacks = new Map(prev);
      newCallbacks.delete(type);
      return newCallbacks;
    });
  }, []);

  const handleWebhook = useCallback((message: WebhookMessage) => {
    const callback = callbacks.get(message.type);
    if (callback) {
      callback(message);
    } else {
      console.warn(`No callback registered for webhook type: ${message.type}`);
    }
  }, [callbacks]);

  return {
    registerCallback,
    unregisterCallback,
    handleWebhook,
    registeredTypes: Array.from(callbacks.keys()),
  };
}

/**
 * Hook for webhook-based task notifications
 */
export function useTaskWebhooks() {
  const webhookHandler = useWebhookHandler({
    url: '/api/webhooks/tasks',
    rateLimit: { maxRequests: 30, windowMs: 60000 }, // 30 per minute for tasks
  });

  const [taskNotifications, setTaskNotifications] = useState<WebhookMessage[]>([]);

  const sendTaskUpdate = useCallback(async (taskId: string, update: any) => {
    return webhookHandler.sendWebhook({
      type: 'task_update',
      payload: { taskId, update },
    });
  }, [webhookHandler]);

  const sendTaskCompletion = useCallback(async (taskId: string, result: any) => {
    const success = await webhookHandler.sendWebhook({
      type: 'task_completion',
      payload: { taskId, result },
    });

    if (success) {
      setTaskNotifications(prev => [...prev.slice(-19), {
        id: crypto.randomUUID(),
        type: 'task_completion',
        payload: { taskId, result },
        timestamp: new Date().toISOString(),
        source: 'web-dashboard',
      }]);
    }

    return success;
  }, [webhookHandler]);

  const sendTaskError = useCallback(async (taskId: string, error: any) => {
    return webhookHandler.sendWebhook({
      type: 'task_error',
      payload: { taskId, error },
    });
  }, [webhookHandler]);

  return {
    ...webhookHandler,
    taskNotifications,
    sendTaskUpdate,
    sendTaskCompletion,
    sendTaskError,
    recentNotifications: taskNotifications.slice(-10),
  };
}

/**
 * Hook for webhook-based system alerts
 */
export function useAlertWebhooks() {
  const webhookHandler = useWebhookHandler({
    url: '/api/webhooks/alerts',
    rateLimit: { maxRequests: 60, windowMs: 60000 }, // 60 per minute for alerts
  });

  const [alerts, setAlerts] = useState<WebhookMessage[]>([]);

  const sendAlert = useCallback(async (alert: {
    level: 'info' | 'warning' | 'error' | 'critical';
    message: string;
    source: string;
    metadata?: any;
  }) => {
    const success = await webhookHandler.sendWebhook({
      type: 'system_alert',
      payload: alert,
    });

    if (success) {
      setAlerts(prev => [...prev.slice(-49), {
        id: crypto.randomUUID(),
        type: 'system_alert',
        payload: alert,
        timestamp: new Date().toISOString(),
        source: 'web-dashboard',
      }]);
    }

    return success;
  }, [webhookHandler]);

  return {
    ...webhookHandler,
    alerts,
    sendAlert,
    activeAlerts: alerts.filter(alert =>
      alert.payload.level === 'error' || alert.payload.level === 'critical'
    ),
    alertCountByLevel: alerts.reduce((acc, alert) => {
      const level = alert.payload.level;
      acc[level] = (acc[level] || 0) + 1;
      return acc;
    }, {} as Record<string, number>),
  };
}
