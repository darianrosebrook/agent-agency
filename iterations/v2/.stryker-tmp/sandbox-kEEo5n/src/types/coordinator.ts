/**
 * System Coordinator Types
 *
 * Types for centralized system coordination, health monitoring,
 * load balancing, and failure management.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


export enum ComponentType {
  AGENT_REGISTRY = "agent_registry",
  TASK_ROUTER = "task_router",
  CAWS_VALIDATOR = "caws_validator",
  PERFORMANCE_TRACKER = "performance_tracker",
  TASK_ORCHESTRATOR = "task_orchestrator",
  CONSTITUTIONAL_RUNTIME = "constitutional_runtime",
}

export enum HealthStatus {
  HEALTHY = "healthy",
  DEGRADED = "degraded",
  UNHEALTHY = "unhealthy",
  UNKNOWN = "unknown",
}

export enum FailureType {
  HEALTH_CHECK_FAILURE = "health_check_failure",
  CONNECTION_FAILURE = "connection_failure",
  TIMEOUT_FAILURE = "timeout_failure",
  INTERNAL_ERROR = "internal_error",
  DEPENDENCY_FAILURE = "dependency_failure",
}

export enum RecoveryStatus {
  IN_PROGRESS = "in_progress",
  SUCCESSFUL = "successful",
  FAILED = "failed",
  TIMEOUT = "timeout",
}

export interface ComponentRegistration {
  id: string;
  name: string;
  type: ComponentType;
  endpoint: string;
  healthCheck: HealthCheckConfig;
  capabilities: ComponentCapabilities;
  dependencies: string[];
  metadata: Record<string, any>;
}

export interface ComponentCapabilities {
  maxConcurrentTasks?: number;
  supportedTaskTypes?: string[];
  performanceMetrics?: boolean;
  healthMonitoring?: boolean;
  constitutionalCompliance?: boolean;
}

export interface HealthCheckConfig {
  endpoint: string;
  method: "GET" | "POST";
  timeout: number;
  interval: number;
  retries: number;
}

export interface ComponentHealth {
  id: string;
  status: HealthStatus;
  lastCheck: Date;
  responseTime: number;
  errorCount: number;
  details?: Record<string, any>;
}

export interface HealthIssue {
  componentId: string;
  type: string;
  severity: "low" | "medium" | "high" | "critical";
  message: string;
  timestamp: Date;
}

export interface SystemHealth {
  status: HealthStatus;
  components: {
    total: number;
    healthy: number;
    degraded: number;
    unhealthy: number;
  };
  lastUpdate: Date;
  issues: HealthIssue[];
}

export interface RoutingPreferences {
  preferredComponent?: string;
  avoidComponents?: string[];
  maxLoad?: number;
  location?: string;
  capabilities?: string[];
}

export interface LoadDistribution {
  componentId: string;
  loadPercentage: number;
  activeConnections: number;
  queueDepth: number;
}

export interface FailureEvent {
  componentId: string;
  failureType: FailureType;
  error: any;
  timestamp: Date;
  context?: Record<string, any>;
}

export interface RecoveryAction {
  type: "restart" | "switchover" | "scale_up" | "alert" | "isolate";
  target: string;
  parameters?: Record<string, any>;
  executed?: boolean;
  executionTime?: number;
  error?: any;
}

export interface FailureRecovery {
  failure: FailureEvent;
  actions: RecoveryAction[];
  status: RecoveryStatus;
  startTime: Date;
  endTime?: Date;
  success: boolean;
}

export interface SystemCoordinatorConfig {
  healthCheckInterval: number;
  failureThreshold: number;
  recoveryTimeout: number;
  loadBalancingEnabled: boolean;
  autoScalingEnabled: boolean;
  maxComponentsPerType: number;
}

export interface CoordinatorStats {
  components: {
    total: number;
    byType: Record<ComponentType, number>;
  };
  health: {
    healthy: number;
    degraded: number;
    unhealthy: number;
  };
  load: {
    totalRequests: number;
    averageResponseTime: number;
  };
  failures: {
    total: number;
    activeRecoveries: number;
    recentFailures: number;
  };
}

