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
    "overview" | "metrics" | "chat" | "tasks" | "database" | "analytics"
  >("overview");

  // Warn about unimplemented features
  React.useEffect(() => {
    if (activeSection === "metrics") {
      console.warn(
        "Centralized metrics dashboard not implemented - requires V3 metrics aggregation"
      );
      // TODO: Centralized Metrics Dashboard
      // - [ ] Implement V3 metrics aggregation endpoints
      // - [ ] Add cross-component metric correlation
      // - [ ] Implement metrics filtering and time range sync
      // - [ ] Add metric export capabilities (CSV, JSON)
      // - [ ] Implement custom dashboard layouts
      // - [ ] Add metric alerting and threshold monitoring
      // - [ ] Integrate with task execution metrics
      // - [ ] Add performance trend analysis
      // - [ ] Implement metrics comparison tools
      // - [ ] Add real-time metric subscriptions
    }

    if (activeSection === "chat") {
      console.warn(
        "Chat interface not implemented - requires V3 chat WebSocket API"
      );
      // TODO: Milestone 1 - Conversational Interface
      // - [ ] Implement V3 chat WebSocket endpoint (/api/v1/chat/ws/:session_id)
      // - [ ] Create chat session management (/api/v1/chat/session)
      // - [ ] Add chat message history API (/api/v1/chat/history/:session_id)
      // - [ ] Build ChatInterface, MessageList, MessageInput, ContextPanel components
      // - [ ] Implement intent parsing and routing in V3 chat_handler.rs
      // - [ ] Add WebSocket client with auto-reconnect and heartbeat
      // - [ ] Integrate with planning agent for task initiation
      // - [ ] Add progress query and guidance request handling
      // - [ ] Test end-to-end chat flow with task creation
    }

    if (activeSection === "tasks") {
      console.warn(
        "Task monitoring not implemented - requires V3 task streaming API"
      );
      // TODO: Milestone 2 - Task Monitoring & Visualization
      // - [ ] Implement V3 task API endpoints (list, detail, events, artifacts, quality)
      // - [ ] Add SSE stream for task events with event_seq ordering
      // - [ ] Create TaskList, TaskCard, TaskTimeline components
      // - [ ] Build ExecutionPhaseViewer and WorkingSpecViewer
      // - [ ] Implement SSE client for real-time task updates
      // - [ ] Add event deduplication using task_id:event_seq
      // - [ ] Integrate pause/resume/context endpoints for research
      // - [ ] Test real-time phase changes and artifact updates
    }

    if (activeSection === "database") {
      console.warn(
        "Database explorer not implemented - requires V3 database API"
      );
      // TODO: Milestone 4 - Database Explorer & Vector Tools
      // - [ ] Implement V3 database query service (query_service.rs)
      // - [ ] Add database API endpoints (tables, schema, query, vector search, metrics)
      // - [ ] Create DatabaseExplorer, TableViewer, VectorSearchPanel components
      // - [ ] Implement QueryBuilder for visual query construction
      // - [ ] Add DataQualityMetrics component
      // - [ ] Enforce safety constraints: 1000 row limit, 5s timeout, parameterized queries
      // - [ ] Add IP/session rate limiting
      // - [ ] Implement column allowlists with redactor middleware
      // - [ ] Test vector similarity search and provenance links
    }

    if (activeSection === "analytics") {
      console.warn(
        "Analytics not implemented - requires V3 metrics aggregation"
      );
      // TODO: Milestone 5 - Analytics & Insights
      // - [ ] Implement V3 analytics aggregation endpoints
      // - [ ] Add anomaly detection with Z-score/EWMA (pluggable)
      // - [ ] Create PerformanceCharts with time-series visualizations
      // - [ ] Build TrendAnalysis and AnomalyDetection components
      // - [ ] Implement PredictiveInsights with actionable recommendations
      // - [ ] Add deploy window tracking to reduce false positives
      // - [ ] Implement export functionality (CSV, JSON)
      // - [ ] Test anomaly detection and trend analysis accuracy
      // - [ ] Validate export generates valid CSV/JSON files
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
        const backendError = healthResponse.backend.error ||
          `Backend status: ${healthResponse.backend.status}`;
        setError(`V3 Backend: ${backendError}`);
      }

    } catch (err) {
      console.error("Health check error:", err);
      const errorMessage = err instanceof Error ? err.message : "Health check failed";
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
                    // TODO: Milestone 3 - System Health Monitoring
                    // - [ ] Implement V3 /health endpoint with component status
                    // - [ ] Add system uptime and version tracking
                    // - [ ] Implement component health checks (database, agents, coordination)
                    // - [ ] Add alert system integration
                    // - [ ] Test health endpoint with various failure scenarios
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
        </div>
      </main>
    </div>
  );
}
