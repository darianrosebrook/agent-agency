"use client";

import React, { useEffect, useState } from "react";
import Header from "@/components/shared/Header";
import Navigation from "@/components/shared/Navigation";
import SystemHealthOverview from "@/components/shared/SystemHealthOverview";
import ChatInterface from "@/components/chat/ChatInterface";
import { ChatSession } from "@/types/chat";

interface HealthStatus {
  status: "healthy" | "degraded" | "unhealthy" | "unknown";
  timestamp: string;
  version?: string;
  uptime?: number;
}

export default function Dashboard() {
  const [healthStatus, setHealthStatus] = useState<HealthStatus | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeSection, setActiveSection] = useState<
    "overview" | "chat" | "tasks" | "database" | "analytics"
  >("overview");

  // Warn about unimplemented features
  React.useEffect(() => {
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

      const data: HealthStatus = await response.json();
      setHealthStatus(data);
    } catch (err) {
      console.error("Health check error:", err);
      setError(err instanceof Error ? err.message : "Health check failed");
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
                <SystemHealthOverview
                  healthStatus={healthStatus}
                  isLoading={isLoading}
                  error={error}
                  onRetry={checkHealth}
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
            </div>
          )}

          {activeSection === "chat" && (
            <ChatInterface
              onSessionCreate={(session) => {
                console.log('Chat session created:', session.id);
              }}
              onSessionUpdate={(session) => {
                console.log('Chat session updated:', session.id, session.status);
              }}
              onError={(error) => {
                console.error('Chat interface error:', error);
              }}
            />
          )}

          {activeSection === "tasks" && (
            <div className="empty-state">
              <div className="empty-state-content">
                <div className="empty-state-icon">📋</div>
                <h1 className="empty-state-title">Task Monitoring</h1>
                <p className="empty-state-description">
                  Monitor autonomous task execution in real-time. View planning
                  phases, execution progress, quality checks, and refinement
                  cycles. Requires SSE connection to V3 task streaming API.
                </p>
                <div className="empty-state-actions">
                  <button className="action-button secondary" disabled>
                    Connect to Task Stream
                  </button>
                </div>
              </div>
            </div>
          )}

          {activeSection === "database" && (
            <div className="empty-state">
              <div className="empty-state-content">
                <div className="empty-state-icon">🗄️</div>
                <h1 className="empty-state-title">Database Explorer</h1>
                <p className="empty-state-description">
                  Safely inspect database state for efficiency and robustness
                  research. Query tables, run vector searches, and analyze data
                  quality. Requires read-only database API access.
                </p>
                <div className="empty-state-actions">
                  <button className="action-button secondary" disabled>
                    Connect to Database API
                  </button>
                </div>
              </div>
            </div>
          )}

          {activeSection === "analytics" && (
            <div className="empty-state">
              <div className="empty-state-content">
                <div className="empty-state-icon">📈</div>
                <h1 className="empty-state-title">Analytics & Insights</h1>
                <p className="empty-state-description">
                  Analyze system performance trends, detect anomalies, and get
                  optimization recommendations. Requires metrics aggregation and
                  historical data access.
                </p>
                <div className="empty-state-actions">
                  <button className="action-button secondary" disabled>
                    Connect to Analytics API
                  </button>
                </div>
              </div>
            </div>
          )}
        </div>
      </main>
    </div>
  );
}
