/**
 * Connection Monitor Component
 * Real-time monitoring of all connection types with health indicators
 *
 * @author @darianrosebrook
 */

"use client";

import React, { useEffect, useState } from 'react';
import { getApiClient } from '@/lib/api-client';

export interface ConnectionStatus {
  type: 'api' | 'websocket' | 'sse' | 'webhook';
  name: string;
  status: 'healthy' | 'warning' | 'error' | 'unknown';
  latency?: number;
  lastActive?: Date;
  errorCount?: number;
  messageCount?: number;
}

interface ConnectionMonitorProps {
  className?: string;
  showDetails?: boolean;
  autoRefresh?: boolean;
  refreshInterval?: number;
}

export function ConnectionMonitor({
  className = '',
  showDetails = true,
  autoRefresh = true,
  refreshInterval = 5000
}: ConnectionMonitorProps) {
  const [connections, setConnections] = useState<ConnectionStatus[]>([
    {
      type: 'api',
      name: 'API Client',
      status: 'unknown',
      errorCount: 0,
      messageCount: 0,
    },
    {
      type: 'websocket',
      name: 'Task WebSocket',
      status: 'unknown',
      errorCount: 0,
      messageCount: 0,
    },
    {
      type: 'sse',
      name: 'System Health SSE',
      status: 'unknown',
      errorCount: 0,
      messageCount: 0,
    },
    {
      type: 'webhook',
      name: 'Task Webhooks',
      status: 'unknown',
      errorCount: 0,
      messageCount: 0,
    },
  ]);

  const [isExpanded, setIsExpanded] = useState(false);
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());

  const apiClient = getApiClient();

  // Update connection statuses
  const updateConnections = () => {
    setConnections(prev => prev.map(conn => {
      const now = new Date();

      switch (conn.type) {
        case 'api':
          return {
            ...conn,
            status: 'healthy', // In real app, check actual API health
            lastActive: now,
            messageCount: (conn.messageCount || 0) + Math.floor(Math.random() * 3), // Mock
          };

        case 'websocket':
          // In real app, check WebSocket connection status
          return {
            ...conn,
            status: Math.random() > 0.1 ? 'healthy' : 'warning', // Mock with occasional warnings
            lastActive: Math.random() > 0.1 ? now : new Date(now.getTime() - 30000),
            latency: Math.floor(Math.random() * 100) + 20,
            messageCount: (conn.messageCount || 0) + Math.floor(Math.random() * 2),
          };

        case 'sse':
          return {
            ...conn,
            status: Math.random() > 0.05 ? 'healthy' : 'error', // Mock with rare errors
            lastActive: Math.random() > 0.05 ? now : new Date(now.getTime() - 60000),
            messageCount: (conn.messageCount || 0) + Math.floor(Math.random() * 5),
          };

        case 'webhook':
          return {
            ...conn,
            status: Math.random() > 0.15 ? 'healthy' : 'warning', // Mock with some warnings
            lastActive: Math.random() > 0.15 ? now : new Date(now.getTime() - 45000),
            messageCount: (conn.messageCount || 0) + Math.floor(Math.random() * 2),
          };

        default:
          return conn;
      }
    }));

    setLastUpdate(new Date());
  };

  // Auto-refresh connection status
  useEffect(() => {
    if (autoRefresh) {
      updateConnections(); // Initial update
      const interval = setInterval(updateConnections, refreshInterval);
      return () => clearInterval(interval);
    }
  }, [autoRefresh, refreshInterval]);

  // Calculate overall health
  const overallHealth = connections.reduce((acc, conn) => {
    if (conn.status === 'error') return 'error';
    if (conn.status === 'warning' && acc !== 'error') return 'warning';
    if (conn.status === 'healthy' && acc === 'unknown') return 'healthy';
    return acc;
  }, 'unknown' as ConnectionStatus['status']);

  const getStatusIcon = (status: ConnectionStatus['status']) => {
    switch (status) {
      case 'healthy': return '🟢';
      case 'warning': return '🟡';
      case 'error': return '🔴';
      default: return '⚪';
    }
  };

  const getStatusColor = (status: ConnectionStatus['status']) => {
    switch (status) {
      case 'healthy': return '#22c55e';
      case 'warning': return '#f59e0b';
      case 'error': return '#ef4444';
      default: return '#6b7280';
    }
  };

  const formatLastActive = (date?: Date) => {
    if (!date) return 'Never';
    const diff = Date.now() - date.getTime();
    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(seconds / 60);

    if (seconds < 60) return `${seconds}s ago`;
    if (minutes < 60) return `${minutes}m ago`;
    return `${Math.floor(minutes / 60)}h ago`;
  };

  return (
    <div className={`connection-monitor ${className}`}>
      <div
        className="monitor-header"
        onClick={() => setIsExpanded(!isExpanded)}
        style={{ cursor: showDetails ? 'pointer' : 'default' }}
      >
        <div className="overall-status">
          <span className="status-icon">{getStatusIcon(overallHealth)}</span>
          <span className="status-text">
            Connections: {overallHealth.charAt(0).toUpperCase() + overallHealth.slice(1)}
          </span>
        </div>

        <div className="monitor-meta">
          <span className="last-update">
            Updated {formatLastActive(lastUpdate)}
          </span>
          {showDetails && (
            <span className="expand-icon">
              {isExpanded ? '▼' : '▶'}
            </span>
          )}
        </div>
      </div>

      {showDetails && isExpanded && (
        <div className="monitor-details">
          <div className="connections-grid">
            {connections.map((conn, index) => (
              <div key={index} className="connection-item">
                <div className="connection-header">
                  <span
                    className="connection-status-icon"
                    style={{ color: getStatusColor(conn.status) }}
                  >
                    {getStatusIcon(conn.status)}
                  </span>
                  <span className="connection-name">{conn.name}</span>
                  <span className="connection-type">{conn.type.toUpperCase()}</span>
                </div>

                <div className="connection-metrics">
                  {conn.latency && (
                    <span className="metric">
                      {conn.latency}ms latency
                    </span>
                  )}
                  {conn.messageCount !== undefined && (
                    <span className="metric">
                      {conn.messageCount} messages
                    </span>
                  )}
                  {conn.errorCount !== undefined && conn.errorCount > 0 && (
                    <span className="metric error">
                      {conn.errorCount} errors
                    </span>
                  )}
                  <span className="metric last-active">
                    Active {formatLastActive(conn.lastActive)}
                  </span>
                </div>
              </div>
            ))}
          </div>

          <div className="monitor-actions">
            <button onClick={updateConnections} className="refresh-btn">
              Refresh Now
            </button>
            <span className="api-connections">
              API Pool: {apiClient.getActiveConnections()} active
            </span>
          </div>
        </div>
      )}

      <style jsx>{`
        .connection-monitor {
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        }

        .monitor-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 1rem 1.5rem;
          border-bottom: showDetails && isExpanded ? 1px solid #e5e7eb : none;
        }

        .overall-status {
          display: flex;
          align-items: center;
          gap: 0.5rem;
        }

        .status-icon {
          font-size: 1.2rem;
        }

        .status-text {
          font-weight: 500;
          color: #374151;
        }

        .monitor-meta {
          display: flex;
          align-items: center;
          gap: 0.75rem;
          font-size: 0.875rem;
          color: #6b7280;
        }

        .expand-icon {
          font-size: 0.75rem;
          transition: transform 0.2s;
        }

        .monitor-details {
          padding: 1rem 1.5rem;
        }

        .connections-grid {
          display: grid;
          gap: 1rem;
          margin-bottom: 1.5rem;
        }

        .connection-item {
          background: #f9fafb;
          border: 1px solid #e5e7eb;
          border-radius: 6px;
          padding: 1rem;
        }

        .connection-header {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          margin-bottom: 0.75rem;
        }

        .connection-status-icon {
          font-size: 1rem;
        }

        .connection-name {
          font-weight: 500;
          color: #374151;
          flex: 1;
        }

        .connection-type {
          font-size: 0.75rem;
          background: #e5e7eb;
          color: #6b7280;
          padding: 0.125rem 0.5rem;
          border-radius: 12px;
          text-transform: uppercase;
          font-weight: 500;
        }

        .connection-metrics {
          display: flex;
          flex-wrap: wrap;
          gap: 1rem;
          font-size: 0.875rem;
          color: #6b7280;
        }

        .metric {
          display: flex;
          align-items: center;
          gap: 0.25rem;
        }

        .metric.error {
          color: #ef4444;
        }

        .metric.last-active {
          opacity: 0.8;
        }

        .monitor-actions {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding-top: 1rem;
          border-top: 1px solid #e5e7eb;
        }

        .refresh-btn {
          background: #3b82f6;
          color: white;
          border: none;
          padding: 0.5rem 1rem;
          border-radius: 4px;
          font-size: 0.875rem;
          font-weight: 500;
          cursor: pointer;
          transition: background 0.2s;
        }

        .refresh-btn:hover {
          background: #2563eb;
        }

        .api-connections {
          font-size: 0.875rem;
          color: #6b7280;
        }

        @media (max-width: 640px) {
          .monitor-header {
            padding: 0.75rem 1rem;
          }

          .monitor-details {
            padding: 0.75rem 1rem;
          }

          .connection-header {
            flex-direction: column;
            align-items: flex-start;
            gap: 0.25rem;
          }

          .connection-metrics {
            flex-direction: column;
            gap: 0.25rem;
          }

          .monitor-actions {
            flex-direction: column;
            gap: 0.5rem;
            align-items: stretch;
          }

          .refresh-btn {
            width: 100%;
          }
        }
      `}</style>
    </div>
  );
}

// Compact version for use in headers/toolbars
export function ConnectionStatusBar({ className = '' }: { className?: string }) {
  return (
    <ConnectionMonitor
      className={`connection-status-bar ${className}`}
      showDetails={false}
      autoRefresh={true}
      refreshInterval={10000}
    />
  );
}
