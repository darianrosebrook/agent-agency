/**
 * Types for System Health Monitor
 *
 * Defines the data structures and interfaces for system health monitoring,
 * metrics collection, and health assessment.
 *
 * @author @darianrosebrook
 */

export interface SystemMetrics {
  /** CPU usage percentage (0-100) */
  cpuUsage: number;
  /** Memory usage percentage (0-100) */
  memoryUsage: number;
  /** Available memory in MB */
  availableMemoryMB: number;
  /** Total memory in MB */
  totalMemoryMB: number;
  /** Disk usage percentage (0-100) */
  diskUsage: number;
  /** Available disk space in GB */
  availableDiskGB: number;
  /** Network I/O in bytes per second */
  networkIO: {
    bytesInPerSecond: number;
    bytesOutPerSecond: number;
  };
  /** System load average (1, 5, 15 minute averages) */
  loadAverage: [number, number, number];
  /** Timestamp of metrics collection */
  timestamp: Date;
}

export interface AgentHealthMetrics {
  /** Agent ID */
  agentId: string;
  /** Agent health score (0-1, higher is healthier) */
  healthScore: number;
  /** Reliability score based on recent performance (0-1) */
  reliabilityScore: number;
  /** Error rate (errors per minute) */
  errorRate: number;
  /** Response time in milliseconds (P95) */
  responseTimeP95: number;
  /** Current task load */
  currentLoad: number;
  /** Maximum allowed load */
  maxLoad: number;
  /** Success rate for recent tasks (0-1) */
  successRate: number;
  /** Last activity timestamp */
  lastActivity: Date;
  /** Circuit breaker status */
  circuitBreakerStatus: "closed" | "open" | "half-open";
}

export interface HealthMetrics {
  /** Overall system health score (0-1, higher is healthier) */
  overallHealth: number;
  /** System-wide metrics */
  system: SystemMetrics;
  /** Agent-specific health metrics */
  agents: Map<string, AgentHealthMetrics>;
  /** System error rate (errors per minute) */
  errorRate: number;
  /** Queue depth (pending tasks) */
  queueDepth: number;
  /** Circuit breaker status for the system */
  circuitBreakerOpen: boolean;
  /** Timestamp of health assessment */
  timestamp: Date;
}

export interface HealthThresholds {
  /** CPU usage threshold (percentage) */
  cpuWarningThreshold: number;
  cpuCriticalThreshold: number;

  /** Memory usage threshold (percentage) */
  memoryWarningThreshold: number;
  memoryCriticalThreshold: number;

  /** Disk usage threshold (percentage) */
  diskWarningThreshold: number;
  diskCriticalThreshold: number;

  /** Agent error rate threshold (errors per minute) */
  agentErrorRateThreshold: number;

  /** Agent response time threshold (milliseconds) */
  agentResponseTimeThreshold: number;

  /** System error rate threshold (errors per minute) */
  systemErrorRateThreshold: number;

  /** Queue depth threshold */
  queueDepthThreshold: number;
}

export interface SystemHealthMonitorConfig {
  /** Metrics collection interval in milliseconds */
  collectionIntervalMs: number;
  /** Health assessment interval in milliseconds */
  healthCheckIntervalMs: number;
  /** Metrics retention period in milliseconds */
  retentionPeriodMs: number;
  /** Health thresholds for alerts */
  thresholds: HealthThresholds;
  /** Enable circuit breaker pattern */
  enableCircuitBreaker: boolean;
  /** Circuit breaker failure threshold */
  circuitBreakerFailureThreshold: number;
  /** Circuit breaker recovery timeout in milliseconds */
  circuitBreakerRecoveryTimeoutMs: number;
}

export interface HealthAlert {
  id: string;
  level: "info" | "warning" | "error" | "critical";
  component: "system" | "agent";
  componentId?: string; // Agent ID if agent-specific
  metric: string;
  value: number;
  threshold: number;
  message: string;
  timestamp: Date;
  acknowledged: boolean;
  resolved: boolean;
  resolvedAt?: Date;
}

export interface HealthReport {
  /** Report period start */
  periodStart: Date;
  /** Report period end */
  periodEnd: Date;
  /** Overall system health trend */
  healthTrend: "improving" | "stable" | "degrading";
  /** System availability percentage */
  availability: number;
  /** Top health issues */
  topIssues: HealthAlert[];
  /** Agent performance summary */
  agentSummary: {
    totalAgents: number;
    healthyAgents: number;
    degradedAgents: number;
    unhealthyAgents: number;
  };
  /** Resource utilization trends */
  resourceTrends: {
    cpuAverage: number;
    memoryAverage: number;
    diskAverage: number;
  };
}
