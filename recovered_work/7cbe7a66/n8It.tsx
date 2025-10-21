"use client";

import React, { useEffect, useState } from "react";
import Header from "@/components/shared/Header";
import Navigation from "@/components/shared/Navigation";
import ChatInterface from "@/components/chat/ChatInterface";
import TaskList from "@/components/tasks/TaskList";
import SystemHealthMonitoring from "@/components/monitoring/SystemHealthOverview";
import MetricsDashboard from "@/components/metrics/MetricsDashboard";
import RealTimeMetricsStream from "@/components/monitoring/RealTimeMetricsStream";
import DatabaseExplorer from "@/components/database/DatabaseExplorer";
import AnalyticsDashboard from "@/components/analytics/AnalyticsDashboard";
import TTSSettingsPanel from "@/components/shared/TTSSettings";
import { useTTS } from "@/hooks/useTTS";

interface HealthStatus {
  status: "healthy" | "degraded" | "unhealthy" | "unknown";
  timestamp: string;
  version?: string;
  uptime?: number;
}

interface HealthResponse {
  status: "healthy" | "degraded" | "unhealthy" | "unknown";
  timestamp: string;
  dashboard: {
    status: string;
    version: string;
    uptime: number;
    node_version?: string;
  };
  backend: {
    status: string;
    url: string;
    response_time_ms: number;
    error?: string;
    raw_response?: string;
  };
}

export default function Dashboard() {
  const [healthStatus, setHealthStatus] = useState<HealthStatus | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeSection, setActiveSection] = useState<
    "overview" | "metrics" | "chat" | "tasks" | "database" | "analytics" | "settings"
  >("overview");

  const { settings: ttsSettings, setSettings: setTtsSettings, voices, isServiceAvailable } = useTTS();

  // Warn about unimplemented features
  React.useEffect(() => {
    if (activeSection === "metrics") {
      console.warn(
        "Metrics dashboard API integration partially implemented - requires V3 backend metrics endpoints"
      );
      // TODO: Centralized Metrics Dashboard (PARTIALLY COMPLETE)
      // - [x] Implement V3 metrics API proxy endpoints (/api/metrics, /api/metrics/stream)
      // - [x] Add real-time metrics streaming via SSE (/api/metrics/stream)
      // - [x] Implement metrics filtering and time range sync
      // - [x] Add metric aggregation and alerting capabilities
      // - [x] Integrate with task execution metrics
      // - [ ] Implement actual V3 backend metrics endpoints when available
      // - [ ] Add metric export capabilities (CSV, JSON)
      // - [ ] Implement custom dashboard layouts
      // - [ ] Add performance trend analysis and comparison tools
      // - [ ] Add real-time metric subscriptions with advanced filtering
    }

    if (activeSection === "chat") {
      console.warn(
        "Chat interface connecting to V3 backend - WebSocket integration partially implemented"
      );
      // TODO: Milestone 1 - Conversational Interface (PARTIALLY COMPLETE)
      // - [x] Implement V3 chat WebSocket endpoint proxy (/api/chat/ws/:session_id)
      // - [x] Create chat session management proxy (/api/proxy/chat/session)
      // - [x] Add chat message history API proxy (/api/proxy/chat/history/:session_id)
      // - [x] Build ChatInterface, MessageList, MessageInput, ContextPanel components
      // - [x] Implement WebSocket client with auto-reconnect and heartbeat
      // - [ ] Implement actual V3 backend chat endpoints when available
      // - [ ] Implement intent parsing and routing in V3 chat_handler.rs
      // - [ ] Integrate with planning agent for task initiation
      // - [ ] Add progress query and guidance request handling
      // - [ ] Test end-to-end chat flow with task creation
    }

    if (activeSection === "tasks") {
      console.warn(
        "Task monitoring API integration partially implemented - requires V3 backend task endpoints"
      );
      // TODO: Milestone 2 - Task Monitoring & Visualization (PARTIALLY COMPLETE)
      // - [x] Implement V3 task API proxy endpoints (list, detail, actions, events)
      // - [x] Add SSE stream proxy for task events with real-time updates
      // - [x] Create TaskList, TaskCard, TaskTimeline components with real API integration
      // - [x] Implement task filtering, sorting, pagination via API
      // - [x] Add task action controls (pause, resume, cancel, restart) via API
      // - [x] Implement SSE client for real-time task event streaming
      // - [ ] Implement V3 backend task endpoints when available
      // - [ ] Add real-time task events streaming from V3 backend
      // - [ ] Integrate task artifacts and quality reports
      // - [ ] Add comprehensive task metrics and performance tracking
      // - [ ] Test end-to-end task lifecycle with V3 backend
    }

    if (activeSection === "database") {
      console.warn(
        "Database explorer API integration partially implemented - requires V3 backend database endpoints"
      );
      // TODO: Milestone 4 - Database Explorer & Vector Tools (PARTIALLY COMPLETE)
      // - [x] Implement V3 database API proxy routes (/api/database/connections, /api/database/tables, /api/database/query, /api/database/vector-search)
      // - [x] Add safety constraints: SQL injection protection, timeout limits, row limits
      // - [x] Create DatabaseExplorer, TableViewer, VectorSearchPanel, QueryBuilder components
      // - [x] Implement database connection management and table schema inspection
      // - [x] Add vector similarity search with configurable parameters
      // - [ ] Implement actual V3 backend database endpoints when available
      // - [ ] Add DataQualityMetrics component integration
      // - [ ] Implement column allowlists with redactor middleware
      // - [ ] Add IP/session rate limiting
      // - [ ] Test end-to-end database operations with V3 backend
    }

    if (activeSection === "analytics") {
      console.warn(
        "Analytics dashboard API integration partially implemented - requires V3 backend analytics endpoints"
      );
      // TODO: Milestone 5 - Analytics & Insights (PARTIALLY COMPLETE)
      // - [x] Implement V3 analytics API proxy routes (/api/analytics)
      // - [x] Add anomaly detection and statistical analysis capabilities
      // - [x] Create AnalyticsDashboard, AnomalyDetector, TrendAnalyzer, PerformancePredictor components
      // - [x] Implement ForecastingChart with confidence intervals
      // - [x] Add CorrelationMatrix for metric relationships
      // - [x] Integrate time-series data and trend analysis
      // - [ ] Implement actual V3 backend analytics endpoints when available
      // - [ ] Add real-time analytics updates and alerting
      // - [ ] Implement export functionality (CSV, JSON)
      // - [ ] Test end-to-end analytics pipeline with V3 backend
    }
  }, [activeSection]);

  const checkHealth = async () => {
    try {
      setError(null);
      const response = await fetch("/api/health");

      if (!response.ok) {
        throw new Error(`Health check failed: ${response.status}`);
      }

      const healthResponse: HealthResponse = await response.json();

      // Create a simplified HealthStatus for the Header component
      const headerHealthStatus: HealthStatus = {
        status: healthResponse.status,
        timestamp: healthResponse.timestamp,
        version: healthResponse.dashboard.version,
        uptime: healthResponse.dashboard.uptime,
      };

      setHealthStatus(headerHealthStatus);

      // Log backend health details for debugging
      console.log("V3 Backend Health:", {
        status: healthResponse.backend.status,
        url: healthResponse.backend.url,
        responseTime: `${healthResponse.backend.response_time_ms}ms`,
        error: healthResponse.backend.error,
      });

      // Set error if backend is unhealthy
      if (healthResponse.backend.status !== "healthy") {
        const backendError =
          healthResponse.backend.error ??
          `Backend status: ${healthResponse.backend.status}`;
        setError(`V3 Backend: ${backendError}`);
      }
    } catch (err) {
      console.error("Health check error:", err);
      const errorMessage =
        err instanceof Error ? err.message : "Health check failed";
      setError(errorMessage);
      setHealthStatus({
        status: "unhealthy",
        timestamp: new Date().toISOString(),
      });
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    checkHealth();
    // Check health every 30 seconds
    const interval = setInterval(checkHealth, 30000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="dashboard">
      <Header
        healthStatus={healthStatus}
        isLoading={isLoading}
        error={error}
        onRetryHealthCheck={checkHealth}
      />

      <Navigation
        activeSection={activeSection}
        onSectionChange={setActiveSection}
      />

      <main className="dashboard-main">
        <div className="dashboard-content">
          {activeSection === "overview" && (
            <div className="overview-section">
              <h1 className="section-title">System Overview</h1>

              <div className="overview-grid">
                <SystemHealthMonitoring
                  onRetry={() => {
                    console.log("Retrying health check...");
                    checkHealth();
                    // TODO: Milestone 3 - System Health Monitoring (PARTIALLY COMPLETE)
                    // - [x] Implement V3 /health endpoint proxy with component status
                    // - [x] Add system uptime and version tracking
                    // - [x] Handle backend connectivity and error scenarios
                    // - [ ] Add detailed component health checks (database, agents, coordination)
                    // - [ ] Implement alert system integration
                    // - [ ] Add health metrics visualization and trends
                  }}
                />

                <div className="welcome-card">
                  <h2>Welcome to Agent Agency V3 Dashboard</h2>
                  <p>
                    This dashboard provides real-time monitoring and
                    conversational interaction with the autonomous agent system.
                    Use the navigation above to explore different aspects of the
                    system.
                  </p>

                  <div className="quick-actions">
                    <button
                      className="action-button primary"
                      onClick={() => setActiveSection("chat")}
                    >
                      Start Conversation
                    </button>
                    <button
                      className="action-button secondary"
                      onClick={() => setActiveSection("tasks")}
                    >
                      View Tasks
                    </button>
                  </div>
                </div>
              </div>

              {/* Real-time metrics stream for the overview */}
              <RealTimeMetricsStream
                onMetricsUpdate={(event) => {
                  console.log("Metrics update:", event);
                }}
                onError={(error) => {
                  console.error("Metrics stream error:", error);
                }}
                enabled={true}
              />
            </div>
          )}

          {activeSection === "metrics" && <MetricsDashboard />}

          {activeSection === "chat" && (
            <ChatInterface
              onSessionCreate={(session) => {
                console.log("Chat session created:", session.id);
              }}
              onSessionUpdate={(session) => {
                console.log(
                  "Chat session updated:",
                  session.id,
                  session.status
                );
              }}
              onError={(error) => {
                console.error("Chat interface error:", error);
              }}
            />
          )}

          {activeSection === "tasks" && (
            <TaskList
              onTaskSelect={(task) => {
                console.log("Task selected:", task.id);
              }}
              onTaskFilter={(filters) => {
                console.log("Task filters changed:", filters);
              }}
            />
          )}

          {activeSection === "database" && (
            <DatabaseExplorer
              onConnectionSelect={(connectionId) => {
                console.log("Database connection selected:", connectionId);
              }}
              onConnectionCreate={() => {
                console.log("Create new database connection");
                // TODO: Milestone 4 - Database Connection Management UI
                // - [ ] Implement connection creation dialog
                // - [ ] Add connection validation
                // - [ ] Support multiple database types (PostgreSQL, MySQL, SQLite)
                // - [ ] Implement connection testing before saving
              }}
            />
          )}

          {activeSection === "analytics" && (
            <AnalyticsDashboard
              onRefresh={() => {
                console.log("Refreshing analytics data");
                // TODO: Milestone 5 - Analytics Data Refresh
                // - [ ] Implement analytics data cache invalidation
                // - [ ] Add selective refresh for specific metrics
                // - [ ] Include real-time data streaming updates
              }}
            />
          )}

          {activeSection === "settings" && (
            <div className="settings-section">
              <h1 className="section-title">Settings</h1>

              <div className="settings-grid">
                <TTSSettingsPanel
                  settings={ttsSettings}
                  voices={voices}
                  isServiceAvailable={isServiceAvailable}
                  onSettingsChange={setTtsSettings}
                  onTestVoice={(voice) => {
                    console.log("Testing voice:", voice);
                    // TODO: Implement voice testing functionality
                  }}
                />

                <div className="system-settings-placeholder">
                  <h3>System Settings</h3>
                  <p>Additional system configuration options will be available here.</p>
                  <ul>
                    <li>Theme preferences</li>
                    <li>Notification settings</li>
                    <li>Performance tuning</li>
                    <li>Security options</li>
                  </ul>
                </div>
              </div>
            </div>
          )}
        </div>
      </main>
    </div>
  );
}
