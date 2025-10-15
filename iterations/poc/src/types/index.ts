/**
 * Core Type Definitions for Agent Agency
 *
 * @author @darianrosebrook
 * @description Central type definitions for the agent agency system
 */

export interface Agent {
  id: string;
  name: string;
  type: AgentType;
  status: AgentStatus;
  capabilities: string[];
  metadata: Record<string, unknown>;
  createdAt: Date;
  updatedAt: Date;
}

export type AgentType = "orchestrator" | "worker" | "monitor" | "coordinator";

export type AgentStatus = "idle" | "active" | "busy" | "error" | "offline";

export interface Task {
  id: string;
  agentId: string;
  type: TaskType;
  status: TaskStatus;
  description: string;
  priority: "low" | "medium" | "high" | "critical";
  requirements?: string[];
  maxRetries?: number;
  timeout?: number;
  metadata?: Record<string, unknown>;
  payload: Record<string, unknown>;
  result?: Record<string, unknown>;
  error?: string;
  createdAt: Date;
  updatedAt: Date;
  completedAt?: Date;
}

export type TaskType = "process" | "analyze" | "coordinate" | "monitor";

export type TaskStatus =
  | "pending"
  | "running"
  | "completed"
  | "failed"
  | "cancelled";

export interface AgentOrchestratorConfig {
  maxConcurrentTasks: number;
  taskTimeoutMs: number;
  retryAttempts: number;
  healthCheckIntervalMs: number;
}

export interface SystemMetrics {
  totalAgents: number;
  activeAgents: number;
  totalTasks: number;
  completedTasks: number;
  failedTasks: number;
  averageTaskDuration: number;
  systemUptime: number;
}

// Multi-Tenant Memory System Types

export interface TenantConfig {
  tenantId: string;
  projectId: string;
  isolationLevel: "strict" | "shared" | "federated";
  accessPolicies: AccessPolicy[];
  sharingRules: SharingRule[];
  dataRetention: RetentionPolicy;
  encryptionEnabled: boolean;
  auditLogging: boolean;
}

export interface AccessPolicy {
  resourceType: "memory" | "entity" | "relationship" | "embedding";
  accessLevel: "read" | "write" | "share" | "federate";
  allowedTenants: string[];
  restrictions: AccessRestriction[];
  conditions?: AccessCondition[];
}

export interface AccessRestriction {
  type: "time_based" | "data_sensitivity" | "usage_limit";
  value: any;
  description: string;
}

export interface AccessCondition {
  type: "user_role" | "project_status" | "compliance_level";
  value: any;
  required: boolean;
}

export interface SharingRule {
  targetTenant: string;
  resourceTypes: string[];
  conditions: SharingCondition[];
  anonymizationLevel: "none" | "basic" | "full";
  retentionLimit?: number;
}

export interface SharingCondition {
  type: "similarity_threshold" | "performance_requirement" | "trust_level";
  value: any;
  operator: "gt" | "gte" | "lt" | "lte" | "eq" | "contains";
}

export interface RetentionPolicy {
  defaultRetentionDays: number;
  archivalPolicy: "delete" | "compress" | "archive";
  complianceRequirements: string[];
  backupFrequency: "daily" | "weekly" | "monthly";
}

export interface TenantContext {
  tenantId: string;
  projectId: string;
  isolationLevel: TenantConfig["isolationLevel"];
  permissions: TenantPermissions;
  metadata: Record<string, any>;
  createdAt: Date;
  lastAccessed: Date;
}

export interface TenantPermissions {
  canRead: boolean;
  canWrite: boolean;
  canShare: boolean;
  canFederate: boolean;
  allowedOperations: string[];
  resourceLimits: Record<string, number>;
}

export interface IsolationResult<T> {
  data: T | null;
  allowed: boolean;
  reason?: string;
  auditLog?: AuditEntry;
}

export interface AuditEntry {
  tenantId: string;
  operation: string;
  resourceType: string;
  resourceId?: string;
  timestamp: Date;
  success: boolean;
  details: Record<string, any>;
}

// Context Offloading Types

export interface ContextualMemory {
  memoryId: string;
  relevanceScore: number;
  contextMatch: ContextMatch;
  reasoningPath?: ReasoningPath;
  temporalRelevance?: TemporalRelevance;
  content: any;
  weight?: number; // For federated learning aggregation
}

export interface ContextMatch {
  similarityScore: number;
  keywordMatches: string[];
  semanticMatches: string[];
  temporalAlignment: number;
}

export interface ReasoningPath {
  steps: ReasoningStep[];
  confidence: number;
  depth: number;
}

export interface ReasoningStep {
  entityId: string;
  relationship: string;
  confidence: number;
  reasoning: string;
}

export interface TemporalRelevance {
  recencyScore: number;
  frequencyScore: number;
  trendAlignment: number;
  decayFactor: number;
}

export interface OffloadedContext {
  id: string;
  tenantId: string;
  originalContext: TaskContext;
  summarizedContext: SummarizedContext;
  embedding: number[];
  compressionRatio: number;
  retrievalMetadata: RetrievalMetadata;
  createdAt: Date;
  lastAccessed?: Date;
  accessCount: number;
}

export interface SummarizedContext {
  coreTask: string;
  keyRequirements: string[];
  criticalConstraints: Record<string, any>;
  essentialEntities: string[];
  summary: string;
  compressionLevel: "minimal" | "moderate" | "aggressive";
}

export interface RetrievalMetadata {
  relevanceThreshold: number;
  retrievalStrategy: "semantic" | "temporal" | "hybrid";
  contextQuarantine: boolean;
  summarizationApplied: boolean;
  expectedRetrievalTime: number;
}

export interface ReconstructedContext {
  context: TaskContext | null;
  relevanceScore: number;
  reconstructionMethod: "direct" | "summarized" | "hybrid";
  confidence: number;
  metadata: RetrievalMetadata;
}

export interface ContextOffloadingConfig {
  maxContextSize: number;
  compressionThreshold: number;
  relevanceThreshold: number;
  quarantineEnabled: boolean;
  summarizationEnabled: boolean;
  temporalDecayEnabled: boolean;
  embeddingDimensions: number;
}

// Multi-Tenant Memory Manager Types

export interface MultiTenantMemoryConfig {
  tenantIsolation: {
    enabled: boolean;
    defaultIsolationLevel: "strict" | "shared" | "federated";
    auditLogging: boolean;
    maxTenants: number;
  };
  contextOffloading: {
    enabled: boolean;
    maxContextSize: number;
    compressionThreshold: number;
    relevanceThreshold: number;
    embeddingDimensions: number;
  };
  federatedLearning: FederatedLearningConfig;
  performance: {
    cacheEnabled: boolean;
    cacheSize: number;
    batchProcessing: boolean;
    asyncOperations: boolean;
  };
}

export interface MemoryOperationResult<T> {
  success: boolean;
  data?: T;
  error?: string;
  tenantId: string;
  operationId: string;
  performance: {
    duration: number;
    cacheHit: boolean;
    offloaded: boolean;
  };
}

export interface FederatedInsights {
  insights: ContextualMemory[];
  confidence: number;
  sourceTenants: string[];
  aggregationMethod: "weighted" | "consensus" | "hybrid";
  privacyPreserved: boolean;
}

// Federated Learning Types

export interface FederatedLearningConfig {
  enabled: boolean;
  privacyLevel: "basic" | "differential" | "secure";
  aggregationFrequency: number;
  minParticipants: number;
  maxParticipants: number;
  privacyBudget: number;
  aggregationMethod: "weighted" | "consensus" | "hybrid";
  learningRate: number;
  convergenceThreshold: number;
}

export interface FederatedParticipant {
  tenantId: string;
  contributionWeight: number;
  privacyLevel: "basic" | "differential" | "secure";
  lastContribution: Date;
  reputationScore: number;
  active: boolean;
}

export interface FederatedSession {
  sessionId: string;
  topic: string;
  participants: FederatedParticipant[];
  status: "forming" | "active" | "aggregating" | "completed" | "failed";
  startTime: Date;
  endTime?: Date;
  aggregatedInsights: ContextualMemory[];
  privacyMetrics: PrivacyMetrics;
  convergenceScore: number;
}

export interface PrivacyMetrics {
  epsilonSpent: number;
  noiseLevel: number;
  informationLeakage: number;
  anonymizationStrength: number;
}

// Enhanced agent types for advanced POC features
export interface EnhancedAgent extends Agent {
  thinkingProfile?: ThinkingProfile;
  rlConfig?: RLConfig;
}

// New types for agentic RL and thinking budgets
export interface ThinkingProfile {
  budgetTokens: number;
  adaptationEnabled: boolean;
  complexityThreshold: number;
  fallbackStrategy: FallbackStrategy;
}

export interface RLConfig {
  algorithm: RLAlgorithm;
  rewardFunction: RewardFunction;
  explorationRate: number;
  learningRate: number;
  discountFactor: number;
}

export interface ThinkingBudget {
  taskId: string;
  allocatedTokens: number;
  usedTokens: number;
  complexity: TaskComplexity;
  adaptationHistory: AdaptationRecord[];
}

export interface AdaptationRecord {
  timestamp: string;
  complexityEstimate: number;
  tokenUsage: number;
  performanceScore: number;
  adjustment: number;
}

export interface TaskContext {
  tenantId?: string;
  userId?: string;
  sessionId?: string;
  agentId?: string;
  taskId?: string;
  type?: string;
  description?: string;
  requirements?: string[];
  constraints?: Record<string, any>;
  metadata?: Record<string, any>;
  historicalContext?: any[];
}

// Evaluation Report contracts
export interface EvaluationReport {
  taskId: string;
  artifactPaths: string[];
  status: "pass" | "iterate" | "fail";
  score: number;
  thresholdsMet: string[];
  thresholdsMissed: string[];
  criteria: EvalCriterion[];
  iterations: number;
  stopReason?:
    | "satisficed"
    | "max-iterations"
    | "quality-ceiling"
    | "failed-gates";
  nextActions?: string[];
  logs?: string[];
  timestamp: string;
}

export interface EvalCriterion {
  id: string;
  description: string;
  weight: number;
  passed: boolean;
  score: number;
  notes?: string;
}

export interface EnhancedEvaluationReport extends EvaluationReport {
  thinkingMetrics: ThinkingMetrics;
  rlMetrics: RLMetrics;
  minimalDiffScore: number;
  rewardHackingPenalty: number;
}

export interface ThinkingMetrics {
  totalTokens: number;
  reasoningDepth: number;
  adaptationCount: number;
  budgetEfficiency: number;
}

export interface RLMetrics {
  creditAssignment: number;
  toolUtilityScore: number;
  rewardDiversity: number;
  learningProgress: number;
}

// Enhanced task types
export interface EnhancedTask extends Task {
  thinkingBudget?: ThinkingBudget;
  rlContext?: RLContext;
  evaluationCriteria: EnhancedEvaluationCriteria;
}

export interface RLContext {
  algorithm: RLAlgorithm;
  episodeId: string;
  stepCount: number;
  accumulatedReward: number;
}

export interface EnhancedEvaluationCriteria {
  minimalDiff: boolean;
  thinkingEfficiency: boolean;
  toolOptimization: boolean;
  rewardHackingPrevention: boolean;
  thresholds: EvaluationThresholds;
}

export interface EvaluationThresholds {
  minScore: number;
  maxRewardHackingPenalty: number;
  minThinkingEfficiency: number;
  maxDiffSize: number;
}

// Enums for enhanced features
export type RLAlgorithm = "ppo" | "grpo" | "dqn" | "sac";
export type RewardFunction =
  | "task_completion"
  | "minimal_diff"
  | "tool_efficiency"
  | "thinking_optimization";
export type FallbackStrategy =
  | "reduce_budget"
  | "simplify_task"
  | "delegate"
  | "fail_fast";
export type TaskComplexity =
  | "trivial"
  | "simple"
  | "moderate"
  | "complex"
  | "extreme";
