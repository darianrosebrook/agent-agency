"use client";

import { ObserverApiClient } from "@/lib/api-client";
import { useEffect, useState } from "react";

interface AlertManagerProps {
  apiClient: ObserverApiClient;
}

interface Alert {
  id: string;
  type: "error" | "warning" | "info";
  severity: "low" | "medium" | "high" | "critical";
  title: string;
  message: string;
  source: string;
  timestamp: string;
  acknowledged: boolean;
  resolved: boolean;
  tags: string[];
  metadata?: Record<string, any>;
}

interface AlertRule {
  id: string;
  name: string;
  description: string;
  condition: string;
  threshold: number;
  severity: "low" | "medium" | "high" | "critical";
  enabled: boolean;
  cooldownMs: number;
  lastTriggered?: string;
}

export default function AlertManager({ apiClient: _apiClient }: AlertManagerProps) {
  const [alerts, setAlerts] = useState<Alert[]>([]);
  const [rules, setRules] = useState<AlertRule[]>([]);
  const [loading, setLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<"active" | "history" | "rules">("active");
  const [filterSeverity, setFilterSeverity] = useState<string>("all");
  const [filterSource, setFilterSource] = useState<string>("all");

  useEffect(() => {
    loadAlerts();
    loadRules();

    // Refresh alerts every 15 seconds
    const interval = setInterval(loadAlerts, 15000);
    return () => clearInterval(interval);
  }, []);

  const loadAlerts = async () => {
    try {
      // Mock alerts data - would come from backend alert API
      const mockAlerts: Alert[] = [
        {
          id: "alert-001",
          type: "error",
          severity: "high",
          title: "Task Processing Failed",
          message: "Task intake validation failed for 3 consecutive tasks",
          source: "TaskOrchestrator",
          timestamp: new Date(Date.now() - 300000).toISOString(),
          acknowledged: false,
          resolved: false,
          tags: ["task", "processing", "validation"],
          metadata: {
            failedTasks: 3,
            lastError: "MISSING_REQUIRED_FIELD: description",
          },
        },
        {
          id: "alert-002",
          type: "warning",
          severity: "medium",
          title: "High Agent Load",
          message: "Agent runtime-refactorer load at 75%",
          source: "AgentRegistry",
          timestamp: new Date(Date.now() - 180000).toISOString(),
          acknowledged: true,
          resolved: false,
          tags: ["agent", "load", "performance"],
          metadata: {
            agentId: "runtime-refactorer",
            currentLoad: 75,
            threshold: 70,
          },
        },
        {
          id: "alert-003",
          type: "info",
          severity: "low",
          title: "System Health Check",
          message: "Daily health check completed successfully",
          source: "HealthMonitor",
          timestamp: new Date(Date.now() - 3600000).toISOString(),
          acknowledged: false,
          resolved: true,
          tags: ["health", "system", "maintenance"],
        },
        {
          id: "alert-004",
          type: "error",
          severity: "critical",
          title: "Database Connection Lost",
          message: "Lost connection to PostgreSQL database",
          source: "DatabasePool",
          timestamp: new Date(Date.now() - 120000).toISOString(),
          acknowledged: false,
          resolved: false,
          tags: ["database", "connection", "critical"],
          metadata: {
            connectionAttempts: 5,
            lastError: "ECONNREFUSED",
          },
        },
      ];

      setAlerts(mockAlerts);
    } catch (err) {
      console.error("Failed to load alerts:", err);
    } finally {
      setLoading(false);
    }
  };

  const loadRules = async () => {
    try {
      // Mock alert rules - would come from backend configuration API
      const mockRules: AlertRule[] = [
        {
          id: "rule-001",
          name: "Task Failure Rate",
          description: "Alert when task failure rate exceeds threshold",
          condition: "taskFailureRate > 0.1",
          threshold: 10,
          severity: "high",
          enabled: true,
          cooldownMs: 300000,
          lastTriggered: new Date(Date.now() - 600000).toISOString(),
        },
        {
          id: "rule-002",
          name: "Agent Load Threshold",
          description: "Alert when agent utilization exceeds 70%",
          condition: "agentLoad > 0.7",
          threshold: 70,
          severity: "medium",
          enabled: true,
          cooldownMs: 600000,
        },
        {
          id: "rule-003",
          name: "Memory Usage Alert",
          description: "Alert when system memory usage exceeds 80%",
          condition: "memoryUsage > 0.8",
          threshold: 80,
          severity: "high",
          enabled: true,
          cooldownMs: 300000,
        },
        {
          id: "rule-004",
          name: "Database Connection",
          description: "Alert when database connections fail",
          condition: "dbConnectionStatus == 'failed'",
          threshold: 0,
          severity: "critical",
          enabled: true,
          cooldownMs: 60000,
        },
      ];

      setRules(mockRules);
    } catch (err) {
      console.error("Failed to load alert rules:", err);
    }
  };

  const acknowledgeAlert = async (alertId: string) => {
    try {
      // Would call backend API to acknowledge alert
      setAlerts(alerts.map(alert =>
        alert.id === alertId
          ? { ...alert, acknowledged: true }
          : alert
      ));
    } catch (err) {
      console.error("Failed to acknowledge alert:", err);
    }
  };

  const resolveAlert = async (alertId: string) => {
    try {
      // Would call backend API to resolve alert
      setAlerts(alerts.map(alert =>
        alert.id === alertId
          ? { ...alert, resolved: true }
          : alert
      ));
    } catch (err) {
      console.error("Failed to resolve alert:", err);
    }
  };

  const toggleRule = async (ruleId: string, enabled: boolean) => {
    try {
      // Would call backend API to update rule
      setRules(rules.map(rule =>
        rule.id === ruleId
          ? { ...rule, enabled }
          : rule
      ));
    } catch (err) {
      console.error("Failed to update rule:", err);
    }
  };

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case "critical":
        return "bg-red-100 text-red-800 border-red-200";
      case "high":
        return "bg-orange-100 text-orange-800 border-orange-200";
      case "medium":
        return "bg-yellow-100 text-yellow-800 border-yellow-200";
      case "low":
        return "bg-blue-100 text-blue-800 border-blue-200";
      default:
        return "bg-gray-100 text-gray-800 border-gray-200";
    }
  };

  const getSeverityIcon = (severity: string) => {
    switch (severity) {
      case "critical":
        return "🚨";
      case "high":
        return "⚠️";
      case "medium":
        return "⚡";
      case "low":
        return "ℹ️";
      default:
        return "📝";
    }
  };

  const filteredAlerts = alerts.filter(alert => {
    const matchesSeverity = filterSeverity === "all" || alert.severity === filterSeverity;
    const matchesSource = filterSource === "all" || alert.source === filterSource;
    return matchesSeverity && matchesSource;
  });

  const activeAlerts = filteredAlerts.filter(alert => !alert.resolved);
  const resolvedAlerts = filteredAlerts.filter(alert => alert.resolved);

  const uniqueSources = Array.from(new Set(alerts.map(alert => alert.source)));
  const uniqueSeverities = Array.from(new Set(alerts.map(alert => alert.severity)));

  if (loading) {
    return (
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">
          Alert Manager
        </h2>
        <div className="animate-pulse">
          <div className="space-y-4">
            {Array.from({ length: 3 }).map((_, i) => (
              <div key={i} className="h-16 bg-gray-200 rounded"></div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg shadow p-6">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-lg font-semibold text-gray-900">
          Alert Manager
        </h2>
        <div className="flex items-center space-x-4">
          <div className="flex items-center space-x-2">
            <span className="text-sm text-gray-500">Active Alerts:</span>
            <span className="text-lg font-bold text-red-600">
              {activeAlerts.length}
            </span>
          </div>
          <div className="flex items-center space-x-2">
            <span className="text-sm text-gray-500">Resolved:</span>
            <span className="text-lg font-bold text-green-600">
              {resolvedAlerts.length}
            </span>
          </div>
        </div>
      </div>

      {/* Filters */}
      <div className="flex items-center space-x-4 mb-6">
        <div className="flex items-center space-x-2">
          <label className="text-sm font-medium text-gray-700">
            Severity:
          </label>
          <select
            value={filterSeverity}
            onChange={(e) => setFilterSeverity(e.target.value)}
            className="text-sm border border-gray-300 rounded px-2 py-1"
          >
            <option value="all">All</option>
            {uniqueSeverities.map(severity => (
              <option key={severity} value={severity}>
                {severity.charAt(0).toUpperCase() + severity.slice(1)}
              </option>
            ))}
          </select>
        </div>

        <div className="flex items-center space-x-2">
          <label className="text-sm font-medium text-gray-700">
            Source:
          </label>
          <select
            value={filterSource}
            onChange={(e) => setFilterSource(e.target.value)}
            className="text-sm border border-gray-300 rounded px-2 py-1"
          >
            <option value="all">All</option>
            {uniqueSources.map(source => (
              <option key={source} value={source}>{source}</option>
            ))}
          </select>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex space-x-1 mb-6">
        {[
          { id: "active", label: "Active Alerts", count: activeAlerts.length },
          { id: "history", label: "Alert History", count: resolvedAlerts.length },
          { id: "rules", label: "Alert Rules", count: rules.length },
        ].map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id as any)}
            className={`px-4 py-2 text-sm font-medium rounded-md ${
              activeTab === tab.id
                ? "bg-blue-100 text-blue-700 border border-blue-200"
                : "text-gray-600 hover:text-gray-900 hover:bg-gray-100"
            }`}
          >
            {tab.label} ({tab.count})
          </button>
        ))}
      </div>

      {/* Active Alerts Tab */}
      {activeTab === "active" && (
        <div className="space-y-4">
          {activeAlerts.length === 0 ? (
            <div className="text-center py-8">
              <div className="text-green-500 text-4xl mb-4">✅</div>
              <h3 className="text-lg font-medium text-gray-900 mb-2">
                All Clear
              </h3>
              <p className="text-gray-600">
                No active alerts at this time.
              </p>
            </div>
          ) : (
            activeAlerts.map((alert) => (
              <div
                key={alert.id}
                className={`border rounded-lg p-4 ${
                  alert.severity === "critical"
                    ? "border-red-200 bg-red-50"
                    : alert.severity === "high"
                    ? "border-orange-200 bg-orange-50"
                    : alert.severity === "medium"
                    ? "border-yellow-200 bg-yellow-50"
                    : "border-blue-200 bg-blue-50"
                }`}
              >
                <div className="flex items-start justify-between mb-3">
                  <div className="flex items-start space-x-3">
                    <span className="text-2xl">
                      {getSeverityIcon(alert.severity)}
                    </span>
                    <div>
                      <h3 className="font-medium text-gray-900">
                        {alert.title}
                      </h3>
                      <p className="text-sm text-gray-600 mt-1">
                        {alert.message}
                      </p>
                      <div className="flex items-center space-x-4 mt-2 text-xs text-gray-500">
                        <span>Source: {alert.source}</span>
                        <span>
                          {new Date(alert.timestamp).toLocaleString()}
                        </span>
                        {!alert.acknowledged && (
                          <span className="text-red-600 font-medium">
                            Unacknowledged
                          </span>
                        )}
                      </div>
                    </div>
                  </div>

                  <div className="flex items-center space-x-2">
                    <span
                      className={`px-2 py-1 rounded-full text-xs font-medium border ${getSeverityColor(
                        alert.severity
                      )}`}
                    >
                      {alert.severity.toUpperCase()}
                    </span>
                  </div>
                </div>

                {alert.tags.length > 0 && (
                  <div className="flex flex-wrap gap-1 mb-3">
                    {alert.tags.map((tag, i) => (
                      <span
                        key={i}
                        className="px-2 py-1 bg-gray-100 text-gray-700 text-xs rounded"
                      >
                        {tag}
                      </span>
                    ))}
                  </div>
                )}

                <div className="flex items-center justify-between">
                  <div className="text-xs text-gray-500">
                    Alert ID: {alert.id}
                  </div>
                  <div className="flex space-x-2">
                    {!alert.acknowledged && (
                      <button
                        onClick={() => acknowledgeAlert(alert.id)}
                        className="px-3 py-1 bg-blue-600 text-white text-xs rounded hover:bg-blue-700 transition-colors"
                      >
                        Acknowledge
                      </button>
                    )}
                    <button
                      onClick={() => resolveAlert(alert.id)}
                      className="px-3 py-1 bg-green-600 text-white text-xs rounded hover:bg-green-700 transition-colors"
                    >
                      Resolve
                    </button>
                  </div>
                </div>
              </div>
            ))
          )}
        </div>
      )}

      {/* Alert History Tab */}
      {activeTab === "history" && (
        <div className="space-y-4">
          {resolvedAlerts.map((alert) => (
            <div
              key={alert.id}
              className="border border-gray-200 rounded-lg p-4 bg-gray-50"
            >
              <div className="flex items-start justify-between mb-2">
                <div className="flex items-start space-x-3">
                  <span className="text-xl">📋</span>
                  <div>
                    <h3 className="font-medium text-gray-900">
                      {alert.title}
                    </h3>
                    <p className="text-sm text-gray-600 mt-1">
                      {alert.message}
                    </p>
                  </div>
                </div>
                <span className="text-xs text-green-600 font-medium">
                  RESOLVED
                </span>
              </div>

              <div className="text-xs text-gray-500">
                Resolved: {new Date(alert.timestamp).toLocaleString()} •
                Source: {alert.source}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Alert Rules Tab */}
      {activeTab === "rules" && (
        <div className="space-y-4">
          {rules.map((rule) => (
            <div
              key={rule.id}
              className="border border-gray-200 rounded-lg p-4"
            >
              <div className="flex items-start justify-between mb-3">
                <div className="flex-1">
                  <div className="flex items-center space-x-3 mb-2">
                    <h3 className="font-medium text-gray-900">
                      {rule.name}
                    </h3>
                    <span
                      className={`px-2 py-1 rounded-full text-xs font-medium border ${getSeverityColor(
                        rule.severity
                      )}`}
                    >
                      {rule.severity.toUpperCase()}
                    </span>
                    {!rule.enabled && (
                      <span className="px-2 py-1 bg-gray-100 text-gray-600 text-xs rounded">
                        DISABLED
                      </span>
                    )}
                  </div>
                  <p className="text-sm text-gray-600 mb-2">
                    {rule.description}
                  </p>
                  <div className="text-xs text-gray-500 space-y-1">
                    <div>Condition: {rule.condition}</div>
                    <div>Threshold: {rule.threshold}</div>
                    <div>Cooldown: {rule.cooldownMs / 1000}s</div>
                    {rule.lastTriggered && (
                      <div>
                        Last Triggered:{" "}
                        {new Date(rule.lastTriggered).toLocaleString()}
                      </div>
                    )}
                  </div>
                </div>

                <div className="ml-4">
                  <label className="flex items-center space-x-2">
                    <input
                      type="checkbox"
                      checked={rule.enabled}
                      onChange={(e) => toggleRule(rule.id, e.target.checked)}
                      className="rounded"
                    />
                    <span className="text-sm text-gray-700">Enabled</span>
                  </label>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
