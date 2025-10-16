/**
 * @fileoverview Type definitions for Adaptive Resource Manager (INFRA-004)
 *
 * Defines types for resource monitoring, load balancing, and dynamic
 * resource allocation across agent pools.
 *
 * @author @darianrosebrook
 */

/**
 * Resource types tracked by the system
 */
export enum ResourceType {
  _CPU = "cpu",
  _MEMORY = "memory",
  _NETWORK = "network",
  _DISK_IO = "disk_io",
  _AGENT_CAPACITY = "agent_capacity",
}

/**
 * Resource usage snapshot
 */
export interface ResourceUsage {
  /** Resource type */
  type: ResourceType;

  /** Current usage value */
  current: number;

  /** Maximum available */
  maximum: number;

  /** Usage percentage (0-100) */
  usagePercent: number;

  /** Unit of measurement */
  unit: string;

  /** Timestamp of measurement */
  timestamp: Date;

  /** Source component or agent */
  source: string;
}

/**
 * Agent resource profile
 */
export interface AgentResourceProfile {
  /** Agent identifier */
  agentId: string;

  /** CPU usage */
  cpuUsage: ResourceUsage;

  /** Memory usage */
  memoryUsage: ResourceUsage;

  /** Network usage */
  networkUsage: ResourceUsage;

  /** Current task count */
  currentTaskCount: number;

  /** Maximum task capacity */
  maxTaskCapacity: number;

  /** Agent health status */
  healthStatus: "healthy" | "degraded" | "unhealthy";

  /** Last updated timestamp */
  lastUpdated: Date;

  /** Average task completion time (ms) */
  avgTaskCompletionMs?: number;
}

/**
 * Task priority levels
 */
export enum TaskPriority {
  _LOW = 0,
  _MEDIUM = 50,
  _HIGH = 75,
  _CRITICAL = 100,
}

/**
 * Load balancing strategy
 */
export enum LoadBalancingStrategy {
  _ROUND_ROBIN = "round_robin",
  _LEAST_LOADED = "least_loaded",
  _WEIGHTED = "weighted",
  _RANDOM = "random",
  _PRIORITY_BASED = "priority_based",
}

/**
 * Load balancing decision
 */
export interface LoadBalancingDecision {
  /** Selected agent ID */
  selectedAgentId: string;

  /** Strategy used */
  strategy: LoadBalancingStrategy;

  /** Agent load at time of decision */
  agentLoad: number;

  /** Decision timestamp */
  timestamp: Date;

  /** Decision rationale */
  rationale: string;

  /** Alternative agents considered */
  alternativesConsidered: string[];

  /** Decision duration (ms) */
  decisionDurationMs: number;
}

/**
 * Resource allocation request
 */
export interface ResourceAllocationRequest {
  /** Request identifier */
  requestId: string;

  /** Task identifier */
  taskId: string;

  /** Task priority */
  priority: TaskPriority;

  /** Required resources */
  requiredResources: {
    cpuPercent?: number;
    memoryMb?: number;
    networkMbps?: number;
  };

  /** Requested timestamp */
  requestedAt: Date;

  /** Timeout for allocation (ms) */
  timeoutMs: number;

  /** Additional metadata */
  metadata?: Record<string, unknown>;
}

/**
 * Resource allocation result
 */
export interface ResourceAllocationResult {
  /** Request identifier */
  requestId: string;

  /** Allocation success */
  success: boolean;

  /** Assigned agent ID */
  assignedAgentId?: string;

  /** Allocated resources */
  allocatedResources?: {
    cpuPercent: number;
    memoryMb: number;
    networkMbps: number;
  };

  /** Allocation timestamp */
  allocatedAt?: Date;

  /** Failure reason */
  failureReason?: string;

  /** Wait time (ms) */
  waitTimeMs: number;
}

/**
 * Rate limiting configuration
 */
export interface RateLimitConfig {
  /** Maximum requests per window */
  maxRequests: number;

  /** Time window (ms) */
  windowMs: number;

  /** Current request count */
  currentCount: number;

  /** Window start time */
  windowStart: Date;

  /** Enable dynamic adjustment */
  dynamicAdjustment: boolean;
}

/**
 * Capacity analysis result
 */
export interface CapacityAnalysis {
  /** Analysis timestamp */
  timestamp: Date;

  /** Analysis window (ms) */
  windowMs: number;

  /** Total system capacity */
  totalCapacity: {
    cpuPercent: number;
    memoryMb: number;
    agentCount: number;
  };

  /** Used capacity */
  usedCapacity: {
    cpuPercent: number;
    memoryMb: number;
    activeAgents: number;
  };

  /** Available capacity */
  availableCapacity: {
    cpuPercent: number;
    memoryMb: number;
    idleAgents: number;
  };

  /** Capacity utilization percentage */
  utilizationPercent: number;

  /** Predicted capacity needs (next hour) */
  predictedNeeds?: {
    cpuPercent: number;
    memoryMb: number;
    agentCount: number;
  };

  /** Scaling recommendation */
  scalingRecommendation?: "scale_up" | "scale_down" | "maintain";

  /** Recommendation rationale */
  recommendationRationale?: string;
}

/**
 * Failover event
 */
export interface FailoverEvent {
  /** Event identifier */
  eventId: string;

  /** Failed agent ID */
  failedAgentId: string;

  /** Backup agent ID */
  backupAgentId: string;

  /** Tasks transferred */
  tasksTransferred: number;

  /** Failover timestamp */
  timestamp: Date;

  /** Failover duration (ms) */
  durationMs: number;

  /** Failure reason */
  failureReason: string;

  /** Success status */
  success: boolean;
}

/**
 * Adaptive Resource Manager configuration
 */
export interface AdaptiveResourceManagerConfig {
  /** Enable/disable resource manager */
  enabled: boolean;

  /** Resource monitoring interval (ms) */
  monitoringIntervalMs: number;

  /** Load balancing strategy */
  loadBalancingStrategy: LoadBalancingStrategy;

  /** Enable dynamic rate limiting */
  enableDynamicRateLimiting: boolean;

  /** Enable automatic failover */
  enableAutoFailover: boolean;

  /** Resource utilization thresholds */
  thresholds: {
    cpuWarning: number;
    cpuCritical: number;
    memoryWarning: number;
    memoryCritical: number;
  };

  /** Maximum allocation decision time (ms) */
  maxAllocationDecisionMs: number;

  /** Enable capacity planning */
  enableCapacityPlanning: boolean;

  /** Capacity analysis interval (ms) */
  capacityAnalysisIntervalMs: number;
}

/**
 * Resource pool statistics
 */
export interface ResourcePoolStats {
  /** Total agents in pool */
  totalAgents: number;

  /** Active agents */
  activeAgents: number;

  /** Idle agents */
  idleAgents: number;

  /** Unhealthy agents */
  unhealthyAgents: number;

  /** Total CPU capacity (%) */
  totalCpuCapacity: number;

  /** Used CPU capacity (%) */
  usedCpuCapacity: number;

  /** Total memory capacity (MB) */
  totalMemoryCapacity: number;

  /** Used memory capacity (MB) */
  usedMemoryCapacity: number;

  /** Total tasks in progress */
  tasksInProgress: number;

  /** Average task completion time (ms) */
  avgTaskCompletionMs: number;

  /** Last updated timestamp */
  lastUpdated: Date;
}

/**
 * Resource monitor interface
 */
export interface IResourceMonitor {
  /**
   * Start resource monitoring
   */
  start(): Promise<void>;

  /**
   * Stop resource monitoring
   */
  stop(): Promise<void>;

  /**
   * Get current resource usage for an agent
   */
  getAgentResources(_agentId: string): Promise<AgentResourceProfile | null>;

  /**
   * Get all agent resource profiles
   */
  getAllAgentResources(): Promise<AgentResourceProfile[]>;

  /**
   * Record resource usage
   */
  recordUsage(_agentId: string, _usage: ResourceUsage): Promise<void>;

  /**
   * Get resource pool statistics
   */
  getPoolStats(): Promise<ResourcePoolStats>;
}

/**
 * Load balancer interface
 */
export interface ILoadBalancer {
  /**
   * Select agent for task assignment
   */
  selectAgent(
    _request: ResourceAllocationRequest,
    _availableAgents: string[]
  ): Promise<LoadBalancingDecision>;

  /**
   * Update load balancing strategy
   */
  setStrategy(_strategy: LoadBalancingStrategy): void;

  /**
   * Get current strategy
   */
  getStrategy(): LoadBalancingStrategy;

  /**
   * Get load distribution statistics
   */
  getLoadDistribution(): Promise<Map<string, number>>;
}

/**
 * Resource allocator interface
 */
export interface IResourceAllocator {
  /**
   * Allocate resources for a task
   */
  allocate(
    _request: ResourceAllocationRequest
  ): Promise<ResourceAllocationResult>;

  /**
   * Release allocated resources
   */
  release(_requestId: string): Promise<void>;

  /**
   * Get allocation statistics
   */
  getAllocationStats(): {
    totalRequests: number;
    successfulAllocations: number;
    failedAllocations: number;
    avgAllocationTimeMs: number;
  };

  /**
   * Update rate limits
   */
  updateRateLimits(_config: RateLimitConfig): void;
}

/**
 * Adaptive Resource Manager interface
 */
export interface IAdaptiveResourceManager {
  /**
   * Initialize the resource manager
   */
  initialize(): Promise<void>;

  /**
   * Start resource management
   */
  start(): Promise<void>;

  /**
   * Stop resource management
   */
  stop(): Promise<void>;

  /**
   * Allocate resources for a task
   */
  allocateResources(
    _request: ResourceAllocationRequest
  ): Promise<ResourceAllocationResult>;

  /**
   * Release resources for a completed task
   */
  releaseResources(_requestId: string): Promise<void>;

  /**
   * Perform capacity analysis
   */
  analyzeCapacity(): Promise<CapacityAnalysis>;

  /**
   * Get resource pool statistics
   */
  getPoolStatistics(): Promise<ResourcePoolStats>;

  /**
   * Get current configuration
   */
  getConfig(): AdaptiveResourceManagerConfig;

  /**
   * Update configuration
   */
  updateConfig(_config: Partial<AdaptiveResourceManagerConfig>): void;

  /**
   * Get health status
   */
  getHealthStatus(): {
    isRunning: boolean;
    lastMonitoringTime?: Date;
    activeAllocations: number;
    failoverEvents: number;
  };

  /**
   * Handle agent failover
   */
  handleFailover(
    _failedAgentId: string,
    _backupAgentId: string
  ): Promise<FailoverEvent>;
}
