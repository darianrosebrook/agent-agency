/**
 * @fileoverview
 * Type definitions for LLM-as-judge evaluation system.
 * Defines interfaces for subjective quality assessment with confidence scoring.
 *
 * @author @darianrosebrook
 */

import { loadLLMConfig } from "@/utils/llm-config";

/**
 * Evaluation criteria types
 */
export enum EvaluationCriterion {
  /** Factual accuracy and truthfulness */
  FAITHFULNESS = "faithfulness",
  /** Task alignment and appropriateness */
  RELEVANCE = "relevance",
  /** Solution elegance and minimalism */
  MINIMALITY = "minimality",
  /** Safety and potential for harm */
  SAFETY = "safety",
}

/**
 * Single criterion assessment result
 */
export interface CriterionAssessment {
  /** Criterion being evaluated */
  criterion: EvaluationCriterion;
  /** Score (0-1) where higher is better */
  score: number;
  /** Confidence in the assessment (0-1) */
  confidence: number;
  /** Reasoning for the score */
  reasoning: string;
  /** Whether this criterion passes threshold */
  passes: boolean;
}

/**
 * Complete judgment result for all criteria
 */
export interface JudgmentResult {
  /** Assessments for each criterion */
  assessments: CriterionAssessment[];
  /** Overall score (average of criterion scores) */
  overallScore: number;
  /** Overall confidence (average of criterion confidences) */
  overallConfidence: number;
  /** Whether all criteria pass thresholds */
  allCriteriaPass: boolean;
  /** Evaluation timestamp */
  evaluatedAt: Date;
  /** Evaluation duration in milliseconds */
  evaluationTimeMs: number;
}

/**
 * Input for judgment evaluation
 */
export interface JudgmentInput {
  /** Task/prompt that was given */
  task: string;
  /** Agent's response/output */
  output: string;
  /** Optional expected/reference output */
  expectedOutput?: string;
  /** Optional context information */
  context?: Record<string, unknown>;
}

/**
 * LLM provider configuration
 */
export interface LLMConfig {
  /** LLM provider name */
  provider: "openai" | "anthropic" | "mock" | "model-registry" | "ollama";
  /** Model identifier */
  model: string;
  /** Temperature (0 for deterministic) */
  temperature: number;
  /** Maximum tokens for response */
  maxTokens: number;
  /** API key (optional, can use env var) */
  apiKey?: string;
}

/**
 * Judge configuration
 */
export interface JudgeConfig {
  /** LLM configuration */
  llm: LLMConfig;
  /** Thresholds for each criterion (0-1) */
  thresholds: Record<EvaluationCriterion, number>;
  /** Enable fallback to rule-based evaluation */
  enableFallback: boolean;
  /** Timeout for LLM calls in milliseconds */
  timeoutMs: number;
  /** Enable caching of identical judgments */
  enableCaching: boolean;
}

/**
 * Default judge configuration
 */
export const DEFAULT_JUDGE_CONFIG: JudgeConfig = {
  llm: loadLLMConfig(),
  thresholds: {
    [EvaluationCriterion.FAITHFULNESS]: 0.7,
    [EvaluationCriterion.RELEVANCE]: 0.7,
    [EvaluationCriterion.MINIMALITY]: 0.7,
    [EvaluationCriterion.SAFETY]: 0.8,
  },
  enableFallback: true,
  timeoutMs: 5000,
  enableCaching: false,
};

/**
 * LLM response format
 */
export interface LLMResponse {
  /** Criterion being evaluated */
  criterion: string;
  /** Score (0-1) */
  score: number;
  /** Confidence (0-1) */
  confidence: number;
  /** Reasoning */
  reasoning: string;
}

/**
 * Cached judgment entry
 */
export interface CachedJudgment {
  /** Input hash (for cache key) */
  inputHash: string;
  /** Cached judgment result */
  result: JudgmentResult;
  /** Cache timestamp */
  cachedAt: Date;
}

/**
 * Judge metrics for monitoring
 */
export interface JudgeMetrics {
  /** Total judgments made */
  totalJudgments: number;
  /** Judgments by criterion */
  judgmentsByCriterion: Record<EvaluationCriterion, number>;
  /** Average evaluation time */
  averageEvaluationTimeMs: number;
  /** Cache hit rate (0-1) */
  cacheHitRate: number;
  /** Fallback trigger rate (0-1) */
  fallbackRate: number;
  /** Average confidence score */
  averageConfidence: number;
}
