/**
 * Connection System Usage Examples
 * Demonstrates how to use the new API client and real-time connection hooks
 *
 * @author @darianrosebrook
 */

"use client";

import React, { useState, useEffect } from 'react';
import { ApiClient, getApiClient } from '@/lib/api-client';
import { useTaskWebSocket } from '@/hooks/useTaskWebSocket';
import { useSSEConnection } from '@/hooks/useSSEConnection';
import { useWebhookHandler } from '@/hooks/useWebhookHandler';
import { useErrorHandler } from '@/lib/error-handling';
import { Task, TaskSubmissionRequest } from '@/types/tasks';

// Example 1: Basic API Client Usage
export function TaskManager() {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [loading, setLoading] = useState(false);
  const { handleError } = useErrorHandler();
  const apiClient = getApiClient();

  const loadTasks = async () => {
    setLoading(true);
    try {
      const response = await apiClient.getTasks();
      setTasks(response.data.tasks || []);
    } catch (error) {
      handleError(error);
    } finally {
      setLoading(false);
    }
  };

  const createTask = async (taskData: TaskSubmissionRequest) => {
    try {
      const response = await apiClient.createTask(taskData);
      // Add new task to the list
      setTasks(prev => [...prev, response.data as Task]);
    } catch (error) {
      handleError(error);
    }
  };

  useEffect(() => {
    loadTasks();
  }, []);

  return (
    <div>
      <h2>Task Manager</h2>
      <button onClick={loadTasks} disabled={loading}>
        {loading ? 'Loading...' : 'Refresh Tasks'}
      </button>

      <div className="tasks-list">
        {tasks.map(task => (
          <div key={task.task_id} className="task-item">
            <h3>{task.description}</h3>
            <p>Status: {task.status}</p>
            <p>Progress: {task.progress_percentage}%</p>
          </div>
        ))}
      </div>
    </div>
  );
}

// Example 2: WebSocket Real-Time Updates
export function RealTimeTaskMonitor({ taskId }: { taskId?: string }) {
  const {
    isConnected,
    connectionStatus,
    taskUpdates,
    subscribeToTask,
    unsubscribeFromTask
  } = useTaskWebSocket(taskId);

  useEffect(() => {
    if (taskId) {
      subscribeToTask(taskId);
      return () => unsubscribeFromTask(taskId);
    }
  }, [taskId, subscribeToTask, unsubscribeFromTask]);

  return (
    <div className="realtime-monitor">
      <div className="connection-status">
        Status: {connectionStatus}
        {isConnected ? '🟢' : '🔴'}
      </div>

      <div className="task-updates">
        <h3>Real-Time Updates</h3>
        {taskUpdates.map((update, index) => (
          <div key={index} className="update-item">
            <span>{new Date(update.timestamp).toLocaleTimeString()}</span>
            <span>Task {update.task_id}: {update.status}</span>
            <span>Progress: {update.progress_percentage}%</span>
          </div>
        ))}
      </div>
    </div>
  );
}

// Example 3: Server-Sent Events for System Monitoring
export function SystemHealthMonitor() {
  const { isConnected, healthData, alerts, latestHealth } = useSSEConnection('/api/health/stream');

  return (
    <div className="system-monitor">
      <div className="connection-status">
        SSE Status: {isConnected ? '🟢 Connected' : '🔴 Disconnected'}
      </div>

      {latestHealth && (
        <div className="health-summary">
          <h3>System Health</h3>
          <p>Status: {latestHealth.status}</p>
          <p>CPU: {latestHealth.cpu_usage}%</p>
          <p>Memory: {latestHealth.memory_usage}%</p>
          <p>Response Time: {latestHealth.avg_response_time}ms</p>
        </div>
      )}

      <div className="alerts-list">
        <h3>Active Alerts</h3>
        {alerts.map((alert, index) => (
          <div key={index} className={`alert ${alert.severity}`}>
            <span className="severity">{alert.severity.toUpperCase()}</span>
            <span className="message">{alert.message}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

// Example 4: Webhook Integration
export function TaskNotifications() {
  const webhookHandler = useWebhookHandler({
    url: '/api/webhooks/tasks',
    rateLimit: { maxRequests: 30, windowMs: 60000 },
  });

  const [notifications, setNotifications] = useState<any[]>([]);

  const sendTaskNotification = async (taskId: string, message: string) => {
    try {
      const success = await webhookHandler.sendWebhook({
        type: 'task_notification',
        payload: { taskId, message, timestamp: new Date().toISOString() },
      });

      if (success) {
        setNotifications(prev => [...prev.slice(-9), {
          type: 'sent',
          taskId,
          message,
          timestamp: new Date().toISOString(),
        }]);
      }
    } catch (error) {
      console.error('Failed to send webhook:', error);
    }
  };

  return (
    <div className="webhook-demo">
      <div className="connection-info">
        <p>Webhook Status: {webhookHandler.connectionState}</p>
        <p>Messages Sent: {webhookHandler.messageCount}</p>
        <p>Rate Limited: {webhookHandler.rateLimited ? 'Yes' : 'No'}</p>
      </div>

      <button
        onClick={() => sendTaskNotification('task-123', 'Task completed successfully!')}
        disabled={webhookHandler.rateLimited}
      >
        Send Notification
      </button>

      <div className="notification-log">
        <h3>Recent Notifications</h3>
        {notifications.map((notif, index) => (
          <div key={index} className="notification-item">
            <span>{notif.type}</span>
            <span>{notif.taskId}</span>
            <span>{notif.message}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

// Example 5: Error Boundary with Recovery
export function ResilientTaskDashboard() {
  const [retryCount, setRetryCount] = useState(0);
  const { handleError } = useErrorHandler();

  const handleTaskError = async (error: Error) => {
    const appError = await handleError(error);

    if (appError.isRecoverable && retryCount < 3) {
      setTimeout(() => {
        setRetryCount(prev => prev + 1);
        // Retry the failed operation
        loadTasksWithRetry();
      }, 1000 * Math.pow(2, retryCount)); // Exponential backoff
    }
  };

  const loadTasksWithRetry = async () => {
    try {
      const apiClient = getApiClient();
      await apiClient.getTasks();
      setRetryCount(0); // Reset on success
    } catch (error) {
      handleTaskError(error as Error);
    }
  };

  return (
    <div className="resilient-dashboard">
      <div className="retry-info">
        <p>Retry attempts: {retryCount}</p>
        {retryCount > 0 && <p>Recovering from error...</p>}
      </div>

      <TaskManager />
    </div>
  );
}

// Example 6: Connection Pooling Demo
export function ConnectionPoolMonitor() {
  const [activeConnections, setActiveConnections] = useState(0);
  const apiClient = getApiClient();

  useEffect(() => {
    const interval = setInterval(() => {
      setActiveConnections(apiClient.getActiveConnections());
    }, 1000);

    return () => clearInterval(interval);
  }, [apiClient]);

  const makeConcurrentRequests = async () => {
    const promises = [];
    for (let i = 0; i < 10; i++) {
      promises.push(apiClient.getTasks());
    }

    try {
      await Promise.all(promises);
      console.log('All requests completed successfully');
    } catch (error) {
      console.error('Some requests failed:', error);
    }
  };

  return (
    <div className="connection-monitor">
      <h3>Connection Pool Monitor</h3>
      <p>Active Connections: {activeConnections}</p>

      <button onClick={makeConcurrentRequests}>
        Make 10 Concurrent Requests
      </button>

      <div className="pool-info">
        <p>The connection pool prevents overwhelming the server</p>
        <p>Requests are queued and rate-limited automatically</p>
      </div>
    </div>
  );
}

// Main Example Component
export default function ConnectionExamples() {
  return (
    <div className="connection-examples">
      <h1>API Connection System Examples</h1>

      <section>
        <h2>1. Basic API Usage</h2>
        <TaskManager />
      </section>

      <section>
        <h2>2. Real-Time WebSocket Updates</h2>
        <RealTimeTaskMonitor taskId="task-123" />
      </section>

      <section>
        <h2>3. Server-Sent Events</h2>
        <SystemHealthMonitor />
      </section>

      <section>
        <h2>4. Webhook Integration</h2>
        <TaskNotifications />
      </section>

      <section>
        <h2>5. Error Recovery</h2>
        <ResilientTaskDashboard />
      </section>

      <section>
        <h2>6. Connection Pooling</h2>
        <ConnectionPoolMonitor />
      </section>
    </div>
  );
}
