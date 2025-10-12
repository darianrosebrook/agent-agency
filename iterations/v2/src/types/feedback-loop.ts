export enum FeedbackSource {
  PERFORMANCE_TRACKER = "performance_tracker",
  TASK_OUTCOME = "task_outcome",
  AGENT_BEHAVIOR = "agent_behavior",
  SYSTEM_METRICS = "system_metrics",
  CONSTITUTIONAL_VIOLATIONS = "constitutional_violations",
  USER_FEEDBACK = "user_feedback",
  HEALTH_MONITORING = "health_monitoring",
}

export enum FeedbackType {
  METRIC = "metric",
  EVENT = "event",
  TREND = "trend",
  ANOMALY = "anomaly",
  RECOMMENDATION = "recommendation",
  OPTIMIZATION = "optimization",
}

export interface FeedbackData {
  id: string;
  source: FeedbackSource;
  type: FeedbackType;
  timestamp: string;
  data: Record<string, any>;
  confidence?: number; // 0-1, how confident we are in this feedback
  priority?: number; // 0-10, importance level
  context?: Record<string, any>; // Additional context
}

export interface TrendAnalysis {
  metric: string;
  trend: "increasing" | "decreasing" | "stable" | "volatile";
  slope: number; // Rate of change
  confidence: number; // 0-1
  dataPoints: number;
  timeWindow: string; // e.g., "1h", "24h", "7d"
  anomalies: AnomalyDetection[];
}

export interface AnomalyDetection {
  id: string;
  metric: string;
  value: number;
  expectedValue: number;
  deviation: number; // Standard deviations from mean
  severity: "low" | "medium" | "high" | "critical";
  timestamp: string;
  context?: Record<string, any>;
}

export interface InsightGeneration {
  id: string;
  title: string;
  description: string;
  category: "performance" | "reliability" | "efficiency" | "scalability" | "security";
  confidence: number; // 0-1
  recommendations: Recommendation[];
  supportingData: FeedbackData[];
  timestamp: string;
  priority: "low" | "medium" | "high" | "urgent";
}

export interface Recommendation {
  id: string;
  type: "parameter_tuning" | "scaling" | "routing_adjustment" | "health_check" | "alert";
  target: string; // Component or parameter to adjust
  action: string;
  parameters?: Record<string, any>;
  expectedImpact: "low" | "medium" | "high";
  riskLevel: "low" | "medium" | "high";
  justification: string;
}

export interface BayesianOptimization {
  parameter: string;
  currentValue: any;
  suggestedValue: any;
  confidence: number; // 0-1
  expectedImprovement: number;
  explorationVsExploitation: number; // 0 (pure exploitation) to 1 (pure exploration)
  trials: number;
  bestValue: any;
  searchSpace: {
    type: "continuous" | "discrete" | "categorical";
    bounds?: [number, number];
    values?: any[];
  };
}

export interface ContinuousImprovementSignal {
  id: string;
  component: string;
  signalType: "performance" | "reliability" | "efficiency" | "adaptation";
  strength: number; // 0-1, strength of the signal
  direction: "increase" | "decrease" | "maintain" | "adapt";
  targetParameter: string;
  currentValue: any;
  recommendedValue: any;
  reasoning: string;
  confidence: number;
  timestamp: string;
  implemented: boolean;
  implementationTimestamp?: string;
  outcome?: "success" | "failure" | "pending";
}

export interface FeedbackLoopConfig {
  collection: {
    enabledSources: FeedbackSource[];
    samplingRate: number; // 0-1, what percentage of events to sample
    maxBufferSize: number;
    retentionPeriodMs: number;
  };
  analysis: {
    trendWindowSizes: string[]; // e.g., ["1h", "24h", "7d"]
    anomalyThreshold: number; // Standard deviations
    minDataPoints: number;
    analysisIntervalMs: number;
  };
  optimization: {
    enabled: boolean;
    parameters: Record<string, BayesianOptimization["searchSpace"]>;
    explorationRate: number; // 0-1
    maxTrials: number;
    confidenceThreshold: number; // Minimum confidence to apply suggestion
  };
  improvement: {
    signalThreshold: number; // Minimum strength to emit signal
    maxConcurrentSignals: number;
    implementationTimeoutMs: number;
    evaluationPeriodMs: number;
  };
}

export interface FeedbackLoopStats {
  totalFeedbackCollected: number;
  feedbackBySource: Record<FeedbackSource, number>;
  insightsGenerated: number;
  recommendationsApplied: number;
  optimizationTrials: number;
  activeSignals: number;
  averageProcessingTime: number;
  lastAnalysisTimestamp?: string;
  bufferUtilization: number; // 0-1
  errorRate: number;
}

export interface FeedbackProcessingResult {
  processed: number;
  insights: InsightGeneration[];
  signals: ContinuousImprovementSignal[];
  errors: string[];
  processingTimeMs: number;
}
