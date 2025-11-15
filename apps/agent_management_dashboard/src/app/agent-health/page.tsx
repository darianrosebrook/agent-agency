"use client";

/**
 * Agent Health Page
 * 
 * Monitors the health, status, and operational metrics of AI agents,
 * including system resources, error rates, and performance indicators.
 * 
 * @author @darianrosebrook
 */

import { useState, useEffect } from "react";
import styles from "./page.module.scss";
import {
  getAgents,
  getAgentHealth,
  getAgentMetrics,
  getAgentLogs,
  restartAgent,
  stopAgent,
  type Agent,
  type AgentHealth,
  type AgentMetrics,
  type AgentLog,
} from "../../lib/api/agents";
import { getAlerts, getSystemMetrics, type Alert, type SystemMetrics } from "../../lib/api/observability";
import { getSystemHealth } from "../../lib/api/system";
import { ErrorDisplay } from "../../components/ErrorDisplay";

function getStatusColor(status: AgentHealth['status']): string {
  switch (status) {
    case 'healthy':
      return '#10b981'; // green
    case 'warning':
      return '#f59e0b'; // yellow
    case 'critical':
      return '#ef4444'; // red
    case 'offline':
      return '#6b7280'; // gray
    default:
      return '#6b7280';
  }
}

function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  
  if (days > 0) return `${days}d ${hours}h`;
  if (hours > 0) return `${hours}h ${minutes}m`;
  return `${minutes}m`;
}

export default function AgentHealthPage() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [agentHealth, setAgentHealth] = useState<Record<string, AgentHealth>>({});
  const [agentMetrics, setAgentMetrics] = useState<Record<string, AgentMetrics>>({});
  const [agentLogs, setAgentLogs] = useState<Record<string, AgentLog[]>>({});
  const [alerts, setAlerts] = useState<Alert[]>([]);
  const [systemMetrics, setSystemMetrics] = useState<SystemMetrics | null>(null);
  const [systemHealth, setSystemHealth] = useState<{
    status: string;
    database?: { status: string; error?: string };
    timestamp?: string;
  } | null>(null);
  const [selectedAgentId, setSelectedAgentId] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const [isRestarting, setIsRestarting] = useState<string | null>(null);
  const [isStopping, setIsStopping] = useState<string | null>(null);

  useEffect(() => {
    async function fetchData() {
      setIsLoading(true);
      setError(null);

      try {
        const [agentsData, alertsData, systemMetricsData, systemHealthData] = await Promise.all([
          getAgents(),
          getAlerts({ resolved: false }), // Already has graceful degradation
          getSystemMetrics(), // Now returns null on error
          getSystemHealth(), // Use the API client function instead of direct apiGet
        ]);

        // Ensure agents is an array
        const agentsArray = Array.isArray(agentsData) 
          ? agentsData 
          : (agentsData?.agents || []);
        setAgents(agentsArray);
        // Ensure alerts is an array
        setAlerts(Array.isArray(alertsData) ? alertsData : []);
        setSystemMetrics(systemMetricsData);
        setSystemHealth(systemHealthData);

        // Fetch health and metrics for each agent
        const healthPromises = agentsArray.map(async (agent) => {
          try {
            const [health, metrics, logs] = await Promise.all([
              getAgentHealth(agent.id).catch(() => null),
              getAgentMetrics(agent.id).catch(() => null),
              getAgentLogs(agent.id, { limit: 10, level: 'error' }).catch(() => []),
            ]);

            if (health) {
              setAgentHealth((prev) => ({ ...prev, [agent.id]: health }));
            }
            if (metrics) {
              setAgentMetrics((prev) => ({ ...prev, [agent.id]: metrics }));
            }
            if (logs.length > 0) {
              setAgentLogs((prev) => ({ ...prev, [agent.id]: logs }));
            }
          } catch (err) {
            console.error(`Failed to fetch data for agent ${agent.id}:`, err);
          }
        });

        await Promise.all(healthPromises);
      } catch (err) {
        setError(err instanceof Error ? err : new Error("Failed to load agent health data"));
      } finally {
        setIsLoading(false);
      }
    }

    fetchData();
    // Refresh every 30 seconds
    const interval = setInterval(fetchData, 30000);
    return () => clearInterval(interval);
  }, []);

  const handleRestart = async (agentId: string) => {
    if (!confirm(`Are you sure you want to restart agent ${agentId}?`)) {
      return;
    }

    setIsRestarting(agentId);
    try {
      await restartAgent(agentId);
      alert("Agent restart initiated successfully");
      // Refresh data after restart
      setTimeout(() => {
        window.location.reload();
      }, 2000);
    } catch (err) {
      alert(`Failed to restart agent: ${err instanceof Error ? err.message : "Unknown error"}`);
    } finally {
      setIsRestarting(null);
    }
  };

  const handleStop = async (agentId: string) => {
    if (!confirm(`Are you sure you want to stop agent ${agentId}? This will halt all operations.`)) {
      return;
    }

    setIsStopping(agentId);
    try {
      await stopAgent(agentId);
      alert("Agent stop initiated successfully");
      // Refresh data after stop
      setTimeout(() => {
        window.location.reload();
      }, 2000);
    } catch (err) {
      alert(`Failed to stop agent: ${err instanceof Error ? err.message : "Unknown error"}`);
    } finally {
      setIsStopping(null);
    }
  };

  if (isLoading) {
    return (
      <div className={styles.agentHealthPage}>
        <div className={styles.agentHealthHeader}>
          <h1 className={styles.agentHealthTitle}>Agent Health</h1>
          <p className={styles.agentHealthDescription}>Loading agent health data...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={styles.agentHealthPage}>
        <div className={styles.agentHealthHeader}>
          <h1 className={styles.agentHealthTitle}>Agent Health</h1>
        </div>
        <ErrorDisplay error={error} />
      </div>
    );
  }

  const selectedAgent = selectedAgentId ? agents.find((a) => a.id === selectedAgentId) : null;
  const selectedHealth = selectedAgentId ? agentHealth[selectedAgentId] : null;
  const selectedMetrics = selectedAgentId ? agentMetrics[selectedAgentId] : null;
  const selectedLogs = selectedAgentId ? agentLogs[selectedAgentId] || [] : [];

  return (
    <div className={styles.agentHealthPage}>
      <div className={styles.agentHealthHeader}>
        <h1 className={styles.agentHealthTitle}>Agent Health</h1>
        <p className={styles.agentHealthDescription}>
          Monitor agent status, system health, and operational metrics
        </p>
      </div>

      {/* Services Status */}
      <div className={styles.servicesSection}>
        <h2 className={styles.sectionTitle}>Infrastructure Services</h2>
        <div className={styles.servicesGrid}>
          {/* API Server Status */}
          <div className={styles.serviceCard}>
            <div className={styles.serviceHeader}>
              <div className={styles.serviceName}>API Server</div>
              <div
                className={styles.statusIndicator}
                style={{ backgroundColor: '#10b981' }} // Always healthy if we can call it
                title="API Server is responding"
              />
            </div>
            <div className={styles.serviceInfo}>
              <div className={styles.serviceType}>Rust API Server</div>
              <div className={styles.serviceStatus}>Status: Healthy</div>
              <div className={styles.servicePort}>Port: 8080</div>
            </div>
            <div className={styles.serviceNote}>
              Service is running and responding to requests
            </div>
          </div>

          {/* Database Status */}
          {systemHealth && (
            <div className={styles.serviceCard}>
              <div className={styles.serviceHeader}>
                <div className={styles.serviceName}>Database</div>
                <div
                  className={styles.statusIndicator}
                  style={{
                    backgroundColor:
                      systemHealth.database?.status === 'healthy'
                        ? '#10b981'
                        : systemHealth.database?.status === 'unhealthy'
                        ? '#ef4444'
                        : '#6b7280',
                  }}
                  title={systemHealth.database?.status ?? 'Unknown'}
                />
              </div>
              <div className={styles.serviceInfo}>
                <div className={styles.serviceType}>PostgreSQL</div>
                <div className={styles.serviceStatus}>
                  Status: {systemHealth.database?.status === 'healthy' ? 'Healthy' : systemHealth.database?.status === 'unhealthy' ? 'Unhealthy' : 'Unknown'}
                </div>
                {systemHealth.database?.error && (
                  <div className={styles.serviceError}>{systemHealth.database.error}</div>
                )}
              </div>
              {systemHealth.database?.status === 'unhealthy' && (
                <div className={styles.serviceWarning}>
                  Database connection pool timeout. Check database is running and connection settings.
                </div>
              )}
            </div>
          )}
        </div>
      </div>

      {/* System Metrics Summary */}
      {systemMetrics && (
        <div className={styles.systemMetrics}>
          <div className={styles.metricCard}>
            <div className={styles.metricLabel}>CPU Usage</div>
            <div className={styles.metricValue}>{(systemMetrics?.cpu_usage_percent || 0).toFixed(1)}%</div>
          </div>
          <div className={styles.metricCard}>
            <div className={styles.metricLabel}>Memory Usage</div>
            <div className={styles.metricValue}>{((systemMetrics?.memory_usage_mb || 0) / 1024).toFixed(1)} GB</div>
          </div>
          <div className={styles.metricCard}>
            <div className={styles.metricLabel}>Disk Usage</div>
            <div className={styles.metricValue}>{(systemMetrics?.disk_usage_percent || 0).toFixed(1)}%</div>
          </div>
          <div className={styles.metricCard}>
            <div className={styles.metricLabel}>Network I/O</div>
            <div className={styles.metricValue}>{(systemMetrics?.network_io_mbps || 0).toFixed(2)} Mbps</div>
          </div>
        </div>
      )}

      {/* Active Alerts */}
      {alerts.length > 0 && (
        <div className={styles.alertsSection}>
          <h2 className={styles.sectionTitle}>Active Alerts</h2>
          <div className={styles.alertsList}>
            {alerts.map((alert) => (
              <div
                key={alert.id}
                className={styles.alertItem}
                style={{
                  borderLeftColor:
                    alert.severity === 'critical' ? '#ef4444' : alert.severity === 'warning' ? '#f59e0b' : '#3b82f6',
                }}
              >
                <div className={styles.alertSeverity}>{alert.severity.toUpperCase()}</div>
                <div className={styles.alertContent}>
                  <div className={styles.alertTitle}>{alert.title}</div>
                  <div className={styles.alertMessage}>{alert.message}</div>
                  <div className={styles.alertSource}>Source: {alert.source}</div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Agent List */}
      <div className={styles.agentsSection}>
        <h2 className={styles.sectionTitle}>Agents ({agents.length})</h2>
        <div className={styles.agentsGrid}>
          {agents.map((agent) => {
            const health = agentHealth[agent.id];
            const metrics = agentMetrics[agent.id];
            const status = health?.status || 'offline';
            const statusColor = getStatusColor(status);

            return (
              <div
                key={agent.id}
                className={styles.agentCard}
                onClick={() => setSelectedAgentId(agent.id === selectedAgentId ? null : agent.id)}
              >
                <div className={styles.agentCardHeader}>
                  <div className={styles.agentName}>{agent.name}</div>
                  <div
                    className={styles.statusIndicator}
                    style={{ backgroundColor: statusColor }}
                  />
                </div>
                <div className={styles.agentInfo}>
                  <div className={styles.agentType}>{agent.worker_type}</div>
                  {agent.specialty && <div className={styles.agentSpecialty}>{agent.specialty}</div>}
                </div>
                {health && (
                  <div className={styles.agentMetrics}>
                    <div className={styles.metricRow}>
                      <span>Uptime:</span>
                      <span>{formatUptime(health.uptime_seconds)}</span>
                    </div>
                    <div className={styles.metricRow}>
                      <span>Health Score:</span>
                      <span>{(health?.health_score || 0).toFixed(0)}%</span>
                    </div>
                    <div className={styles.metricRow}>
                      <span>Errors:</span>
                      <span>{health.error_count}</span>
                    </div>
                    {metrics && metrics.cpu_usage_percent !== undefined && (
                      <>
                        <div className={styles.metricRow}>
                          <span>CPU:</span>
                          <span>{(metrics.cpu_usage_percent || 0).toFixed(1)}%</span>
                        </div>
                        <div className={styles.metricRow}>
                          <span>Memory:</span>
                          <span>{((metrics.memory_usage_mb || 0) / 1024).toFixed(1)} GB</span>
                        </div>
                        <div className={styles.metricRow}>
                          <span>Response Time (P95):</span>
                          <span>{(metrics.response_time_p95_ms || 0).toFixed(0)}ms</span>
                        </div>
                      </>
                    )}
                  </div>
                )}
                <div className={styles.agentActions}>
                  <button
                    className={styles.actionButton}
                    onClick={(e) => {
                      e.stopPropagation();
                      handleRestart(agent.id);
                    }}
                    disabled={isRestarting === agent.id || isStopping === agent.id}
                  >
                    {isRestarting === agent.id ? "Restarting..." : "Restart"}
                  </button>
                  <button
                    className={styles.actionButton}
                    onClick={(e) => {
                      e.stopPropagation();
                      handleStop(agent.id);
                    }}
                    disabled={isRestarting === agent.id || isStopping === agent.id}
                  >
                    {isStopping === agent.id ? "Stopping..." : "Stop"}
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Agent Details */}
      {selectedAgent && (
        <div className={styles.agentDetails}>
          <h2 className={styles.sectionTitle}>Agent Details: {selectedAgent.name}</h2>
          
          {selectedHealth && (
            <div className={styles.detailsSection}>
              <h3>Health Status</h3>
              <div className={styles.detailsGrid}>
                <div>Status: <strong>{selectedHealth.status}</strong></div>
                <div>Uptime: {formatUptime(selectedHealth.uptime_seconds)}</div>
                <div>Last Seen: {new Date(selectedHealth.last_seen).toLocaleString()}</div>
                <div>Error Count: {selectedHealth.error_count}</div>
                <div>Response Time: {selectedHealth.response_time_ms.toFixed(0)}ms</div>
                <div>Health Score: {selectedHealth.health_score.toFixed(0)}%</div>
              </div>
            </div>
          )}

          {selectedMetrics && (
            <div className={styles.detailsSection}>
              <h3>Performance Metrics</h3>
              <div className={styles.detailsGrid}>
                <div>CPU Usage: {selectedMetrics.cpu_usage_percent.toFixed(1)}%</div>
                <div>Memory Usage: {(selectedMetrics.memory_usage_mb / 1024).toFixed(1)} GB</div>
                <div>Response Time (P50): {selectedMetrics.response_time_p50_ms.toFixed(0)}ms</div>
                <div>Response Time (P95): {selectedMetrics.response_time_p95_ms.toFixed(0)}ms</div>
                <div>Response Time (P99): {selectedMetrics.response_time_p99_ms.toFixed(0)}ms</div>
                <div>Requests/sec: {selectedMetrics.requests_per_second.toFixed(2)}</div>
                <div>Error Rate: {(selectedMetrics.error_rate * 100).toFixed(2)}%</div>
                <div>Timestamp: {new Date(selectedMetrics.timestamp).toLocaleString()}</div>
              </div>
            </div>
          )}

          {selectedLogs.length > 0 && (
            <div className={styles.detailsSection}>
              <h3>Recent Error Logs</h3>
              <div className={styles.logsList}>
                {selectedLogs.map((log) => (
                  <div key={log.id} className={styles.logEntry}>
                    <div className={styles.logHeader}>
                      <span className={styles.logLevel}>{log.level.toUpperCase()}</span>
                      <span className={styles.logTime}>
                        {new Date(log.timestamp).toLocaleString()}
                      </span>
                    </div>
                    <div className={styles.logMessage}>{log.message}</div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
