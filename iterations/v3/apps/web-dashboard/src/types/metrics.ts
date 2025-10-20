// Metrics and Observability Types
// Defines data structures for system health, agent performance, and business metrics

export interface SystemHealth {
  status: "healthy" | "degraded" | "unhealthy" | "unknown";
  timestamp: string;
  version: string;
  uptime_seconds: number;
  components: ComponentHealth[];
  alerts: HealthAlert[];
}

export interface ComponentHealth {
  name: string;
  status: "healthy" | "degraded" | "unhealthy" | "unknown";
  response_time_ms?: number;
  error_rate?: number;
  last_check: string;
  details?: Record<string, any>;
}

export interface HealthAlert {
  id: string;
  severity: "info" | "warning" | "error" | "critical";
  title: string;
  description: string;
  timestamp: string;
  component?: string;
  resolved: boolean;
  acknowledged: boolean;
}

// Agent Performance Metrics
export interface AgentPerformance {
  agent_id: string;
  name: string;
  type: "planning" | "execution" | "coordination" | "validation" | "specialized";
  status: "active" | "idle" | "error" | "maintenance";

  // Performance metrics
  tasks_completed: number;
  tasks_failed: number;
  success_rate: number;
  average_response_time_ms: number;
  throughput_per_hour: number;

  // Resource utilization
  cpu_usage_percent: number;
  memory_usage_mb: number;
  active_connections: number;

  // Error tracking
  error_count: number;
  last_error?: string;
  error_rate_per_hour: number;

  // Business metrics
  cost_per_task: number;
  efficiency_score: number; // 0-100
}

// Coordination Metrics
export interface CoordinationMetrics {
  timestamp: string;

  // Council consensus
  consensus_rate: number; // 0-1
  average_decision_time_ms: number;
  total_decisions: number;
  failed_decisions: number;

  // Task coordination
  active_tasks: number;
  queued_tasks: number;
  completed_tasks_per_hour: number;
  average_task_duration_ms: number;

  // Communication patterns
  inter_agent_messages_per_minute: number;
  average_message_latency_ms: number;
  message_failure_rate: number;
}

// Business Intelligence Metrics
export interface BusinessMetrics {
  timestamp: string;

  // Task metrics
  total_tasks_created: number;
  tasks_completed_today: number;
  average_task_completion_time_ms: number;
  task_success_rate: number;

  // Quality metrics
  quality_checks_passed: number;
  quality_checks_failed: number;
  average_quality_score: number;

  // Cost and efficiency
  total_cost_today: number;
  cost_per_task: number;
  efficiency_trend: number; // percentage change

  // User engagement (if applicable)
  active_sessions: number;
  average_session_duration_ms: number;
}

// Real-time Metrics Stream
export interface MetricsStreamEvent {
  type: "health_update" | "agent_performance" | "coordination_update" | "business_metrics";
  timestamp: string;
  data: any;
  event_id: string;
}

export interface HealthUpdateEvent {
  system_health: SystemHealth;
}

export interface AgentPerformanceEvent {
  agent_performance: AgentPerformance;
}

export interface CoordinationUpdateEvent {
  coordination_metrics: CoordinationMetrics;
}

export interface BusinessMetricsEvent {
  business_metrics: BusinessMetrics;
}

// Time Series Data
export interface TimeSeriesPoint {
  timestamp: string;
  value: number;
  metadata?: Record<string, any>;
}

export interface TimeSeriesData {
  metric_name: string;
  points: TimeSeriesPoint[];
  interval_seconds: number;
  aggregation: "sum" | "avg" | "min" | "max" | "count";
}

// Alert and Anomaly Detection
export interface AnomalyAlert {
  id: string;
  metric_name: string;
  severity: "low" | "medium" | "high" | "critical";
  message: string;
  current_value: number;
  expected_value: number;
  deviation_percent: number;
  timestamp: string;
  acknowledged: boolean;
  resolved: boolean;
}

export interface AnomalyConfig {
  metric_name: string;
  enabled: boolean;
  threshold_deviation_percent: number;
  min_samples: number;
  cooldown_minutes: number;
  severity_mapping: {
    low: number;
    medium: number;
    high: number;
    critical: number;
  };
}

// Component Props Types
export interface SystemHealthOverviewProps {
  healthStatus?: SystemHealth;
  isLoading?: boolean;
  error?: string | null;
  onRetry?: () => void;
}

export interface AgentPerformanceGridProps {
  agents: AgentPerformance[];
  isLoading?: boolean;
  error?: string | null;
  onAgentSelect?: (agentId: string) => void;
  selectedAgentId?: string;
}

export interface CoordinationMetricsProps {
  metrics?: CoordinationMetrics;
  isLoading?: boolean;
  error?: string | null;
}

export interface BusinessIntelligenceProps {
  metrics?: BusinessMetrics;
  isLoading?: boolean;
  error?: string | null;
  timeRange?: "1h" | "6h" | "24h" | "7d" | "30d";
  onTimeRangeChange?: (range: string) => void;
}

export interface RealTimeMetricsStreamProps {
  onMetricsUpdate?: (event: MetricsStreamEvent) => void;
  onError?: (error: Event) => void;
  enabled?: boolean;
}

export interface MetricTileProps {
  title: string;
  value: string | number;
  change?: number;
  changeLabel?: string;
  status?: "success" | "warning" | "error" | "neutral";
  icon?: string;
  trend?: "up" | "down" | "stable";
  loading?: boolean;
  format?: "number" | "percentage" | "currency" | "duration" | "bytes";
}

export interface AlertPanelProps {
  alerts: HealthAlert[];
  onAcknowledge?: (alertId: string) => void;
  onResolve?: (alertId: string) => void;
  maxItems?: number;
}

// API Response Types
export interface GetSystemHealthResponse {
  health: SystemHealth;
}

export interface GetAgentPerformanceResponse {
  agents: AgentPerformance[];
  timestamp: string;
}

export interface GetCoordinationMetricsResponse {
  metrics: CoordinationMetrics;
}

export interface GetBusinessMetricsResponse {
  metrics: BusinessMetrics;
  time_series?: TimeSeriesData[];
}

export interface GetAlertsResponse {
  alerts: HealthAlert[];
  total_count: number;
  acknowledged_count: number;
  resolved_count: number;
}

// Error Types
export interface MetricsError {
  code: "metrics_unavailable" | "agent_not_found" | "stream_error" | "server_error";
  message: string;
  metric_name?: string;
  timestamp: string;
  retryable: boolean;
}
