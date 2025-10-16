/**
 * Agent Registry Type Definitions
 *
 * @author @darianrosebrook
 * @module agent-registry
 *
 * Type definitions for the Agent Registry Manager component (ARBITER-001).
 * Provides capability tracking, performance history, and agent profile management.
 */
// @ts-nocheck


// Re-export commonly used types from verification
export { VerificationPriority } from "./verification";

/**
 * Unique identifier for an agent in the registry.
 */
export type AgentId = string;

/**
 * Timestamp in ISO 8601 format with millisecond precision.
 */
export type Timestamp = string;

/**
 * Agent capability categories for task routing.
 */
export type TaskType =
  | "code-editing"
  | "research"
  | "code-review"
  | "documentation"
  | "testing"
  | "debugging"
  | "refactoring"
  | "api-design";

/**
 * Programming languages an agent can work with.
 */
export type ProgrammingLanguage =
  | "TypeScript"
  | "JavaScript"
  | "Python"
  | "Rust"
  | "Go"
  | "Java"
  | "C++"
  | "C#";

/**
 * Specialized capabilities beyond basic task types.
 */
export type Specialization =
  | "AST analysis"
  | "API design"
  | "Performance optimization"
  | "Security audit"
  | "Database design"
  | "Frontend architecture"
  | "Backend architecture"
  | "DevOps";

/**
 * Expertise levels for specializations.
 */
export type ExpertiseLevel = "novice" | "intermediate" | "expert" | "master";

/**
 * Enhanced specialization with expertise level and performance metrics.
 */
export interface SpecializationProfile {
  /**
   * The specialization type.
   */
  type: Specialization;

  /**
   * Expertise level in this specialization.
   */
  level: ExpertiseLevel;

  /**
   * Success rate specifically for this specialization (0.0 - 1.0).
   */
  successRate: number;

  /**
   * Number of tasks completed in this specialization.
   */
  taskCount: number;

  /**
   * Average quality score for this specialization.
   */
  averageQuality: number;

  /**
   * Last time this specialization was used.
   */
  lastUsed?: Timestamp;
}

/**
 * Model family identifier for the underlying AI model.
 */
export type ModelFamily =
  | "gpt-4"
  | "gpt-3.5-turbo"
  | "claude-3"
  | "claude-3.5"
  | "gemini-pro"
  | "llama-3"
  | "mixtral";

/**
 * Agent capability profile defining what tasks an agent can handle.
 */
export interface AgentCapabilities {
  /**
   * Types of tasks this agent can perform.
   */
  taskTypes: TaskType[];

  /**
   * Programming languages this agent is proficient in.
   */
  languages: ProgrammingLanguage[];

  /**
   * Specialized skills beyond basic capabilities with expertise levels.
   * @deprecated Use specializationsV2 for enhanced specialization tracking
   */
  specializations?: Specialization[];

  /**
   * Enhanced specialization profiles with expertise levels and metrics.
   */
  specializationsV2?: SpecializationProfile[];
}

/**
 * Historical performance metrics for an agent.
 * Uses running averages to avoid storing all historical data.
 */
export interface PerformanceHistory {
  /**
   * Success rate as a ratio (0.0 - 1.0).
   * Represents the percentage of tasks completed successfully.
   */
  successRate: number;

  /**
   * Average quality score (0.0 - 1.0) from evaluations.
   */
  averageQuality: number;

  /**
   * Average task completion latency in milliseconds.
   */
  averageLatency: number;

  /**
   * Total number of tasks completed by this agent.
   * Used for computing running averages and confidence intervals.
   */
  taskCount: number;
}

/**
 * Current load and utilization metrics for an agent.
 */
export interface CurrentLoad {
  /**
   * Number of tasks currently being executed by this agent.
   */
  activeTasks: number;

  /**
   * Number of tasks queued for this agent.
   */
  queuedTasks: number;

  /**
   * Utilization as a percentage (0-100).
   * Computed as (activeTasks / maxConcurrentTasks) * 100.
   */
  utilizationPercent: number;
}

/**
 * Complete agent profile stored in the registry.
 * Includes identity, capabilities, performance history, and current state.
 */
export interface AgentProfile {
  /**
   * Unique identifier for this agent.
   */
  id: AgentId;

  /**
   * Human-readable name for this agent.
   */
  name: string;

  /**
   * Model family this agent is based on.
   */
  modelFamily: ModelFamily;

  /**
   * Capability profile defining what this agent can do.
   */
  capabilities: AgentCapabilities;

  /**
   * Historical performance metrics (running averages).
   */
  performanceHistory: PerformanceHistory;

  /**
   * Current load and utilization state.
   */
  currentLoad: CurrentLoad;

  /**
   * Timestamp when this agent was registered.
   */
  registeredAt: Timestamp;

  /**
   * Timestamp of the most recent activity.
   */
  lastActiveAt: Timestamp;
}

/**
 * New performance metrics from a completed task.
 * Used to update the agent's running average performance history.
 */
export interface PerformanceMetrics {
  /**
   * Whether the task was completed successfully.
   */
  success: boolean;

  /**
   * Quality score from evaluation (0.0 - 1.0).
   */
  qualityScore: number;

  /**
   * Task completion time in milliseconds.
   */
  latencyMs: number;

  /**
   * Optional: Number of tokens consumed.
   */
  tokensUsed?: number;

  /**
   * Optional: Specific task type completed.
   */
  taskType?: TaskType;
}

/**
 * Query parameters for finding agents by capability.
 */
export interface AgentQuery {
  /**
   * Required task type.
   */
  taskType: TaskType;

  /**
   * Optional: Required programming languages.
   */
  languages?: ProgrammingLanguage[];

  /**
   * Optional: Required specializations (legacy - prefer specializationQuery).
   * @deprecated Use specializationQuery for enhanced matching
   */
  specializations?: Specialization[];

  /**
   * Optional: Enhanced specialization query with expertise requirements.
   */
  specializationQuery?: SpecializationRequirement[];

  /**
   * Optional: Maximum utilization threshold (0-100).
   * Only return agents below this utilization level.
   */
  maxUtilization?: number;

  /**
   * Optional: Minimum success rate threshold (0.0 - 1.0).
   */
  minSuccessRate?: number;
}

/**
 * Requirements for specialization matching.
 */
export interface SpecializationRequirement {
  /**
   * Required specialization type.
   */
  type: Specialization;

  /**
   * Minimum expertise level required.
   */
  minLevel?: ExpertiseLevel;

  /**
   * Minimum success rate for this specialization.
   */
  minSuccessRate?: number;

  /**
   * Whether this specialization is required (true) or preferred (false).
   */
  required?: boolean;
}

/**
 * Result from querying agents with scoring information.
 */
export interface AgentQueryResult {
  /**
   * Matching agent profile.
   */
  agent: AgentProfile;

  /**
   * Score indicating how well this agent matches the query (0.0 - 1.0).
   * Higher scores indicate better matches.
   */
  matchScore: number;

  /**
   * Human-readable explanation of the match score.
   */
  matchReason: string;
}

/**
 * Database connection configuration.
 */
export interface DatabaseConfig {
  /**
   * Database host address.
   */
  host: string;

  /**
   * Database port number.
   */
  port: number;

  /**
   * Database name.
   */
  database: string;

  /**
   * Database username.
   */
  username: string;

  /**
   * Database password.
   */
  password: string;

  /**
   * Whether to use SSL for database connections.
   */
  ssl?: boolean;
}

/**
 * Configuration for the agent registry.
 */
export interface AgentRegistryConfig {
  /**
   * Maximum number of agents that can be registered.
   */
  maxAgents: number;

  /**
   * Time in milliseconds before an inactive agent is considered stale.
   */
  staleAgentThresholdMs: number;

  /**
   * Whether to enable automatic cleanup of stale agents.
   */
  enableAutoCleanup: boolean;

  /**
   * Interval in milliseconds for running cleanup operations.
   */
  cleanupIntervalMs: number;

  /**
   * Database configuration for persistence.
   */
  database?: DatabaseConfig;

  /**
   * Whether to enable database persistence.
   */
  enablePersistence: boolean;

  /**
   * Security configuration.
   */
  security?: any; // Will be imported from security module

  /**
   * Whether to enable security controls.
   */
  enableSecurity: boolean;
}

/**
 * Statistics about the agent registry state.
 */
export interface RegistryStats {
  /**
   * Total number of registered agents.
   */
  totalAgents: number;

  /**
   * Number of currently active agents.
   */
  activeAgents: number;

  /**
   * Number of idle agents (no active tasks).
   */
  idleAgents: number;

  /**
   * Average utilization across all agents (0-100).
   */
  averageUtilization: number;

  /**
   * Average success rate across all agents (0.0 - 1.0).
   */
  averageSuccessRate: number;

  /**
   * Timestamp of the last registry update.
   */
  lastUpdated: Timestamp;
}

/**
 * Error types that can occur in registry operations.
 */
export enum RegistryErrorType {
  AGENT_NOT_FOUND = "AGENT_NOT_FOUND",
  AGENT_ALREADY_EXISTS = "AGENT_ALREADY_EXISTS",
  REGISTRY_FULL = "REGISTRY_FULL",
  INVALID_AGENT_DATA = "INVALID_AGENT_DATA",
  QUERY_FAILED = "QUERY_FAILED",
  UPDATE_FAILED = "UPDATE_FAILED",
  DATABASE_ERROR = "DATABASE_ERROR",
}

/**
 * Registry operation error with context.
 */
export class RegistryError extends Error {
  constructor(
    public readonly type: RegistryErrorType,
    message: string,
    public readonly context?: Record<string, unknown>
  ) {
    super(message);
    this.name = "RegistryError";
  }
}

/**
 * Core interface for agent registry operations.
 * Allows dependency inversion and easier testing/mocking.
 */
export interface AgentRegistry {
  /**
   * Initialize the registry with optional seeding data.
   * @param options - Initialization options including seeds and mode
   */
  initialize(options?: RegistryInitOptions): Promise<void>;

  /**
   * Query agents by capability requirements.
   * @param query - Capability query parameters
   * @returns Array of matching agents with scores
   */
  getAgentsByCapability(query: AgentQuery): Promise<AgentQueryResult[]>;

  /**
   * Update performance metrics for an agent after task completion.
   * @param agentId - ID of the agent to update
   * @param metrics - Performance metrics from completed task
   */
  updatePerformance(
    agentId: AgentId,
    metrics: PerformanceMetrics
  ): Promise<AgentProfile>;

  /**
   * Get current registry statistics.
   * @returns Registry statistics including counts and averages
   */
  getStats(): Promise<RegistryStats>;

  /**
   * Get agent profile by ID.
   * @param agentId - ID of the agent to retrieve
   * @returns Agent profile
   */
  getProfile(agentId: AgentId): Promise<AgentProfile>;
}

/**
 * Options for registry initialization.
 */
export interface RegistryInitOptions {
  /**
   * Seed data for initializing the registry.
   */
  seeds?: Partial<AgentProfile>[];

  /**
   * Initialization mode - idempotent allows re-initialization without errors.
   */
  mode?: "idempotent" | "strict";

  /**
   * Whether to emit registry.ready event on completion.
   */
  emitReady?: boolean;
}
