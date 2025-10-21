"use client";

import { useEffect, useRef, useCallback } from "react";
import { SSEClient } from "@/lib/sse/SSEClient";
import {
  RealTimeMetricsStreamProps,
  MetricsStreamEvent,
} from "@/types/metrics";

// Mock agent definitions with realistic data
const MOCK_AGENTS = [
  {
    id: "agent_1",
    name: "CodeAnalyzer",
    capabilities: {
      languages: ["rust", "typescript", "python"],
      domains: ["code-analysis", "security", "performance"],
      frameworks: ["tokio", "react", "django"],
    },
    performance: {
      task_completion_rate: 0.92,
      average_response_time_ms: 1200,
      error_rate: 0.034,
      active_tasks: 2,
    },
  },
  {
    id: "agent_2",
    name: "TaskCoordinator",
    capabilities: {
      languages: ["typescript", "go"],
      domains: ["orchestration", "task-management"],
      frameworks: ["node", "express", "kubernetes"],
    },
    performance: {
      task_completion_rate: 0.89,
      average_response_time_ms: 950,
      error_rate: 0.021,
      active_tasks: 0,
    },
  },
  {
    id: "agent_3",
    name: "SecurityAuditor",
    capabilities: {
      languages: ["rust", "python"],
      domains: ["security", "compliance", "audit"],
      frameworks: ["openssl", "cryptography"],
    },
    performance: {
      task_completion_rate: 0.95,
      average_response_time_ms: 1800,
      error_rate: 0.012,
      active_tasks: 1,
    },
  },
  {
    id: "agent_4",
    name: "DataProcessor",
    capabilities: {
      languages: ["python", "sql"],
      domains: ["data-processing", "analytics"],
      frameworks: ["pandas", "numpy", "postgresql"],
    },
    performance: {
      task_completion_rate: 0.87,
      average_response_time_ms: 2100,
      error_rate: 0.045,
      active_tasks: 3,
    },
  },
  {
    id: "agent_5",
    name: "APIDesigner",
    capabilities: {
      languages: ["typescript", "rust"],
      domains: ["api-design", "microservices"],
      frameworks: ["graphql", "rest", "axum"],
    },
    performance: {
      task_completion_rate: 0.91,
      average_response_time_ms: 1350,
      error_rate: 0.028,
      active_tasks: 0,
    },
  },
  {
    id: "agent_6",
    name: "FrontendArchitect",
    capabilities: {
      languages: ["typescript", "javascript"],
      domains: ["frontend", "ui-ux", "accessibility"],
      frameworks: ["react", "nextjs", "tailwind"],
    },
    performance: {
      task_completion_rate: 0.88,
      average_response_time_ms: 1600,
      error_rate: 0.039,
      active_tasks: 1,
    },
  },
  {
    id: "agent_7",
    name: "DatabaseExpert",
    capabilities: {
      languages: ["sql", "python"],
      domains: ["database-design", "optimization"],
      frameworks: ["postgresql", "mongodb", "redis"],
    },
    performance: {
      task_completion_rate: 0.94,
      average_response_time_ms: 1100,
      error_rate: 0.018,
      active_tasks: 0,
    },
  },
  {
    id: "agent_8",
    name: "TestAutomation",
    capabilities: {
      languages: ["typescript", "python"],
      domains: ["testing", "qa", "automation"],
      frameworks: ["jest", "cypress", "pytest"],
    },
    performance: {
      task_completion_rate: 0.85,
      average_response_time_ms: 1900,
      error_rate: 0.062,
      active_tasks: 2,
    },
  },
  {
    id: "agent_9",
    name: "DevOpsEngineer",
    capabilities: {
      languages: ["yaml", "bash", "python"],
      domains: ["infrastructure", "deployment", "monitoring"],
      frameworks: ["docker", "kubernetes", "terraform"],
    },
    performance: {
      task_completion_rate: 0.90,
      average_response_time_ms: 1450,
      error_rate: 0.031,
      active_tasks: 1,
    },
  },
  {
    id: "agent_10",
    name: "ResearchAnalyst",
    capabilities: {
      languages: ["python", "r"],
      domains: ["research", "analysis", "machine-learning"],
      frameworks: ["scikit-learn", "tensorflow", "pandas"],
    },
    performance: {
      task_completion_rate: 0.82,
      average_response_time_ms: 2800,
      error_rate: 0.078,
      active_tasks: 1,
    },
  },
];

export default function RealTimeMetricsStream({
  onMetricsUpdate,
  onError,
  enabled = true,
}: RealTimeMetricsStreamProps) {
  const sseClientRef = useRef<SSEClient | null>(null);
  const enabledRef = useRef(enabled);

  // Update enabled ref when prop changes
  useEffect(() => {
    enabledRef.current = enabled;
  }, [enabled]);

  const handleMetricsEvent = useCallback(
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (event: any) => {
      if (!enabledRef.current) return;

      try {
        const metricsEvent: MetricsStreamEvent = {
          type: event.type,
          timestamp: event.timestamp || new Date().toISOString(),
          data: event.data,
          event_id: event.event_id || `event_${Date.now()}`,
        };

        onMetricsUpdate?.(metricsEvent);
      } catch (error) {
        console.error("Failed to process metrics event:", error, event);
        onError?.(error as Event);
      }
    },
    [onMetricsUpdate, onError]
  );

  const handleSSEError = useCallback(
    (error: Event) => {
      console.error("SSE connection error:", error);
      onError?.(error);
    },
    [onError]
  );

  const handleSSEOpen = useCallback(() => {
    console.log("Metrics SSE connection opened");
  }, []);

  const handleSSEClose = useCallback(() => {
    console.log("Metrics SSE connection closed");
  }, []);

  // Initialize SSE connection
  useEffect(() => {
    if (!enabled) return;

    console.warn(
      "Using mock real-time metrics stream - V3 SSE endpoint not available"
    );

    // Mock SSE simulation for development
    let eventCount = 0;
    const maxEvents = 100; // Prevent infinite simulation

    const mockEventTypes = [
      "health_update",
      "agent_performance",
      "coordination_update",
      "business_metrics",
      "system_load",
      "error_rate",
      "throughput",
    ];

    const mockEventGenerator = () => {
      if (eventCount >= maxEvents) return;

      eventCount++;

      const eventType =
        mockEventTypes[Math.floor(Math.random() * mockEventTypes.length)];
      const timestamp = new Date().toISOString();

      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const mockData: any = {
        event_id: `evt_${Date.now()}_${Math.random()
          .toString(36)
          .substr(2, 9)}`,
        timestamp,
        event_type: eventType,
      };

      // Generate mock data based on event type
      switch (eventType) {
        case "health_update":
          mockData.data = {
            service: ["orchestrator", "worker", "database", "api"][
              Math.floor(Math.random() * 4)
            ],
            status: ["healthy", "degraded", "unhealthy"][
              Math.floor(Math.random() * 3)
            ],
            response_time_ms: Math.floor(Math.random() * 1000) + 50,
            uptime_seconds: Math.floor(Math.random() * 86400),
          };
          break;

        case "agent_performance": {
          // Use real agent data with slight variations for realism
          const agent = MOCK_AGENTS[Math.floor(Math.random() * MOCK_AGENTS.length)];
          const variation = 0.05; // ±5% variation for realism

          mockData.data = {
            agent_id: agent.id,
            agent_name: agent.name,
            task_completion_rate: Math.max(0, Math.min(1,
              agent.performance.task_completion_rate +
              (Math.random() - 0.5) * variation * 2
            )),
            average_response_time_ms: Math.max(100, Math.floor(
              agent.performance.average_response_time_ms +
              (Math.random() - 0.5) * agent.performance.average_response_time_ms * variation
            )),
            error_rate: Math.max(0, Math.min(0.1,
              agent.performance.error_rate +
              (Math.random() - 0.5) * variation * 2
            )),
            active_tasks: Math.max(0, Math.floor(
              agent.performance.active_tasks +
              (Math.random() - 0.5) * 2
            )),
            capabilities: agent.capabilities,
          };
          break;
        }

        case "coordination_update":
          mockData.data = {
            coordination_type: [
              "task_assignment",
              "resource_allocation",
              "conflict_resolution",
            ][Math.floor(Math.random() * 3)],
            participants: Math.floor(Math.random() * 5) + 1,
            success: Math.random() > 0.1,
            duration_ms: Math.floor(Math.random() * 1000) + 50,
          };
          break;

        case "business_metrics":
          mockData.data = {
            metric_type: [
              "user_engagement",
              "task_completion",
              "system_efficiency",
            ][Math.floor(Math.random() * 3)],
            value: Math.floor(Math.random() * 1000),
            trend: ["up", "down", "stable"][Math.floor(Math.random() * 3)],
            period: "1h",
          };
          break;

        case "system_load":
          mockData.data = {
            cpu_usage_percent: Math.floor(Math.random() * 100),
            memory_usage_percent: Math.floor(Math.random() * 100),
            disk_usage_percent: Math.floor(Math.random() * 100),
            network_throughput_mbps: Math.floor(Math.random() * 1000),
          };
          break;

        case "error_rate":
          mockData.data = {
            service: ["api", "worker", "database", "coordinator"][
              Math.floor(Math.random() * 4)
            ],
            error_count: Math.floor(Math.random() * 10),
            total_requests: Math.floor(Math.random() * 1000) + 100,
            error_rate_percent: Math.random() * 5,
          };
          break;

        case "throughput":
          mockData.data = {
            service: ["api", "worker", "database"][
              Math.floor(Math.random() * 3)
            ],
            requests_per_second: Math.floor(Math.random() * 100) + 10,
            average_latency_ms: Math.floor(Math.random() * 500) + 50,
            success_rate_percent: 95 + Math.random() * 5,
          };
          break;
      }

      // Simulate SSE message handling
      handleMetricsEvent({ data: JSON.stringify(mockData) });
    };

    // Start mock event stream
    const intervalId = setInterval(
      mockEventGenerator,
      1000 + Math.random() * 2000
    ); // Random interval 1-3 seconds

    return () => {
      clearInterval(intervalId);
    };

    // Future implementation:
    /*
    const streamUrl = metricsApiClient.getMetricsStreamUrl();

    sseClientRef.current = new SSEClient({
      url: streamUrl,
      onMessage: handleMetricsEvent,
      onError: handleSSEError,
      onOpen: handleSSEOpen,
      onClose: handleSSEClose,
    });

    return () => {
      if (sseClientRef.current) {
        sseClientRef.current.destroy();
        sseClientRef.current = null;
      }
    };
    */
  }, [
    enabled,
    handleMetricsEvent,
    handleSSEError,
    handleSSEOpen,
    handleSSEClose,
  ]);

  // Handle enabled/disabled changes
  useEffect(() => {
    if (!sseClientRef.current) return;

    if (enabled) {
      // Reconnect if enabled
      sseClientRef.current.reconnect();
    } else {
      // Disconnect if disabled
      sseClientRef.current.disconnect();
    }
  }, [enabled]);

  // This component doesn't render anything visible - it's a data provider
  return null;
}
