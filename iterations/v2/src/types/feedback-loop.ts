import { TaskOutcome } from "./agentic-rl";
import { ConstitutionalViolation } from "./caws-constitutional";
import { ComponentHealth, FailureEvent } from "./coordinator";

export enum FeedbackSource {
  PERFORMANCE_METRICS = "performance_metrics",
  TASK_OUTCOMES = "task_outcomes",
  USER_RATINGS = "user_ratings",
  SYSTEM_EVENTS = "system_events",
  CONSTITUTIONAL_VIOLATIONS = "constitutional_violations",
  COMPONENT_HEALTH = "component_health",
  ROUTING_DECISIONS = "routing_decisions",
  AGENT_FEEDBACK = "agent_feedback",
}

export enum FeedbackType {
  NUMERIC_METRIC = "numeric_metric",
  CATEGORICAL_EVENT = "categorical_event",
  TEXT_FEEDBACK = "text_feedback",
  RATING_SCALE = "rating_scale",
  BINARY_OUTCOME = "binary_outcome",
}

export interface FeedbackEvent {
  id: string;
  source: FeedbackSource;
  type: FeedbackType;
  entityId: string; // Agent ID, Task ID, Component ID, etc.
  entityType: string; // "agent", "task", "component", etc.
  timestamp: string;
  value: any; // Flexible value based on type
  context: Record<string, any>;
  metadata?: Record<string, any>;
}

export interface PerformanceFeedback extends FeedbackEvent {
  source: FeedbackSource.PERFORMANCE_METRICS;
  metrics: {
    latencyMs?: number;
    throughput?: number;
    errorRate?: number;
    resourceUsage?: {
      cpuPercent?: number;
      memoryMb?: number;
      networkMbps?: number;
    };
    qualityScore?: number;
  };
}

export interface TaskOutcomeFeedback extends FeedbackEvent {
  source: FeedbackSource.TASK_OUTCOMES;
  outcome: TaskOutcome;
  executionTimeMs: number;
  retryCount: number;
  errorDetails?: string;
}

export interface UserRatingFeedback extends FeedbackEvent {
  source: FeedbackSource.USER_RATINGS;
  rating: number; // 1-5 scale
  comments?: string;
  criteria: {
    accuracy: number;
    speed: number;
    reliability: number;
    communication: number;
  };
}

export interface SystemEventFeedback extends FeedbackEvent {
  source: FeedbackSource.SYSTEM_EVENTS;
  eventType: string;
  severity: "low" | "medium" | "high" | "critical";
  description: string;
  impact: {
    affectedComponents: string[];
    estimatedDowntimeMinutes?: number;
    userImpact: "none" | "minor" | "major" | "critical";
  };
}

export interface ConstitutionalViolationFeedback extends FeedbackEvent {
  source: FeedbackSource.CONSTITUTIONAL_VIOLATIONS;
  violation: ConstitutionalViolation;
  policyImpact: {
    affectedTasks: number;
    complianceScoreDelta: number;
    riskLevel: "low" | "medium" | "high";
  };
}

export interface ComponentHealthFeedback extends FeedbackEvent {
  source: FeedbackSource.COMPONENT_HEALTH;
  health: ComponentHealth;
  previousStatus?: ComponentHealth["status"];
  statusChangeReason?: string;
}

export interface RoutingDecisionFeedback extends FeedbackEvent {
  source: FeedbackSource.ROUTING_DECISIONS;
  decision: {
    taskId: string;
    selectedAgentId: string;
    routingStrategy: string;
    confidence: number;
    alternativesCount: number;
    routingTimeMs: number;
  };
  outcome?: {
    success: boolean;
    executionTimeMs?: number;
    qualityScore?: number;
  };
}

export interface AgentFeedback extends FeedbackEvent {
  source: FeedbackSource.AGENT_FEEDBACK;
  feedback: {
    agentId: string;
    feedbackType: "performance" | "capability" | "reliability" | "communication";
    rating: number;
    details?: string;
    suggestedImprovements?: string[];
  };
}

export interface FeedbackAnalysis {
  id: string;
  entityId: string;
  entityType: string;
  timeWindow: {
    start: string;
    end: string;
  };
  metrics: {
    totalFeedbackEvents: number;
    averageRating?: number;
    performanceTrend: "improving" | "stable" | "declining";
    anomalyCount: number;
    correlationStrength: number;
  };
  insights: FeedbackInsight[];
  recommendations: FeedbackRecommendation[];
  confidence: number;
  generatedAt: string;
}

export interface FeedbackInsight {
  type: "trend" | "anomaly" | "correlation" | "prediction";
  description: string;
  severity: "low" | "medium" | "high";
  evidence: {
    metric: string;
    value: any;
    baseline?: any;
    changePercent?: number;
  };
  impact: {
    affectedEntities: string[];
    estimatedImpact: "positive" | "negative" | "neutral";
    confidence: number;
  };
}

export interface FeedbackRecommendation {
  id: string;
  type: "agent_update" | "routing_adjustment" | "resource_allocation" | "policy_change" | "system_configuration";
  priority: "low" | "medium" | "high" | "critical";
  description: string;
  action: {
    targetEntity: string;
    operation: string;
    parameters: Record<string, any>;
  };
  expectedImpact: {
    metric: string;
    improvementPercent: number;
    timeToEffect: string;
  };
  riskAssessment: {
    riskLevel: "low" | "medium" | "high";
    rollbackPlan: string;
    monitoringRequired: boolean;
  };
  prerequisites?: string[];
  implementationStatus: "pending" | "in_progress" | "implemented" | "failed";
}

export interface FeedbackCollectionConfig {
  enabledSources: FeedbackSource[];
  batchSize: number;
  flushIntervalMs: number;
  retentionPeriodDays: number;
  samplingRate: number; // 0.0-1.0, for high-volume sources
  filters: {
    minSeverity?: string;
    excludeEntityTypes?: string[];
    includeOnlyRecent?: boolean; // Last 24h only
  };
}

export interface FeedbackAnalysisConfig {
  enabledAnalyzers: string[];
  analysisIntervalMs: number;
  anomalyThreshold: number; // Z-score threshold
  trendWindowHours: number;
  minDataPoints: number; // Minimum feedback events for analysis
  correlationThreshold: number; // Minimum correlation coefficient
  predictionHorizonHours: number;
}

export interface ImprovementEngineConfig {
  autoApplyThreshold: number; // Minimum recommendation confidence for auto-apply
  maxConcurrentImprovements: number;
  cooldownPeriodMs: number; // Minimum time between improvements to same entity
  improvementTimeoutMs: number; // Max time to wait for improvement effect
  rollbackOnFailure: boolean;
  monitoringPeriodMs: number; // How long to monitor improvement effects
}

export interface FeedbackPipelineConfig {
  batchSize: number;
  processingIntervalMs: number;
  dataQualityThreshold: number; // Minimum quality score for RL training
  anonymizationLevel: "none" | "partial" | "full";
  featureEngineering: {
    enabled: boolean;
    timeWindowFeatures: boolean;
    correlationFeatures: boolean;
    trendFeatures: boolean;
  };
  trainingDataFormat: "json" | "parquet" | "csv";
}

export interface FeedbackLoopConfig {
  collection: FeedbackCollectionConfig;
  analysis: FeedbackAnalysisConfig;
  improvements: ImprovementEngineConfig;
  pipeline: FeedbackPipelineConfig;
  enabled: boolean;
  debugMode: boolean;
}

export interface FeedbackStats {
  totalEvents: number;
  eventsBySource: Record<FeedbackSource, number>;
  eventsByType: Record<FeedbackType, number>;
  analysisCount: number;
  recommendationsGenerated: number;
  recommendationsApplied: number;
  averageProcessingTimeMs: number;
  dataQualityScore: number;
  lastAnalysisTime?: string;
  uptimeSeconds: number;
}

export interface FeedbackProcessingResult {
  success: boolean;
  processedEvents: number;
  analysisPerformed: boolean;
  recommendations: FeedbackRecommendation[];
  errors: string[];
  processingTimeMs: number;
  qualityScore: number;
}