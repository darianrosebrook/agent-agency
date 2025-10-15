/**
 * TypeScript client for DSPy Integration Service
 *
 * Provides type-safe interface to Python-based DSPy service for
 * rubric optimization and model-based judge evaluation.
 *
 * @author @darianrosebrook
 */

import { Logger } from "@/observability/Logger";
import axios, { AxiosError, AxiosInstance } from "axios";

/**
 * Configuration for DSPy client
 */
export interface DSPyClientConfig {
  /** Base URL of DSPy service */
  baseUrl: string;
  /** Request timeout in milliseconds */
  timeout?: number;
  /** Maximum retry attempts */
  maxRetries?: number;
  /** Retry delay in milliseconds */
  retryDelay?: number;
}

/**
 * Request for rubric optimization
 */
export interface RubricOptimizationRequest {
  taskContext: string;
  agentOutput: string;
  evaluationCriteria: string;
}

/**
 * Response from rubric optimization
 */
export interface RubricOptimizationResponse {
  rewardScore: number;
  reasoning: string;
  improvementSuggestions: string;
  metadata: Record<string, unknown>;
}

/**
 * Request for judge evaluation
 */
export interface JudgeEvaluationRequest {
  judgeType: "relevance" | "faithfulness" | "minimality" | "safety";
  artifact: string;
  groundTruth: string;
  context: string;
}

/**
 * Response from judge evaluation
 */
export interface JudgeEvaluationResponse {
  judgment: string;
  confidence: number;
  reasoning: string;
  metadata: Record<string, unknown>;
}

/**
 * Request for signature optimization
 */
export interface SignatureOptimizationRequest {
  signatureId: string;
  evalData: Array<Record<string, unknown>>;
  optimizer: "MIPROv2" | "SIMBA" | "BootstrapFewShot";
}

/**
 * Response from signature optimization
 */
export interface SignatureOptimizationResponse {
  optimizedSignatureId: string;
  improvementMetrics: Record<string, number>;
  metadata: Record<string, unknown>;
}

/**
 * Health check response
 */
export interface HealthResponse {
  status: string;
  version: string;
  dspyConfigured: boolean;
}

/**
 * Client for DSPy Integration Service
 *
 * Provides methods for:
 * - Rubric optimization
 * - Model-based judge evaluation
 * - Signature optimization
 */
export class DSPyClient {
  private readonly client: AxiosInstance;
  private readonly logger: Logger;
  private readonly maxRetries: number;
  private readonly retryDelay: number;

  constructor(config: DSPyClientConfig) {
    this.client = axios.create({
      baseURL: config.baseUrl,
      timeout: config.timeout ?? 30000,
      headers: {
        "Content-Type": "application/json",
      },
    });

    this.maxRetries = config.maxRetries ?? 3;
    this.retryDelay = config.retryDelay ?? 1000;
    this.logger = new Logger("DSPyClient");
  }

  /**
   * Check health of DSPy service
   *
   * @returns Health status
   * @throws Error if service is unhealthy
   */
  async health(): Promise<HealthResponse> {
    try {
      const response = await this.client.get<HealthResponse>("/health");
      return response.data;
    } catch (error) {
      this.logger.error("DSPy service health check failed", { error });
      throw new Error(
        `DSPy service unavailable: ${this.getErrorMessage(error)}`
      );
    }
  }

  /**
   * Optimize rubric computation using DSPy
   *
   * Uses DSPy's signature-based programming to systematically optimize
   * reward computation prompts.
   *
   * @param request - Rubric optimization request
   * @returns Optimized rubric evaluation
   * @throws Error if optimization fails
   */
  async optimizeRubric(
    request: RubricOptimizationRequest
  ): Promise<RubricOptimizationResponse> {
    return this.retryRequest(async () => {
      this.logger.info("Optimizing rubric", {
        taskContext: request.taskContext.substring(0, 100),
      });

      const response = await this.client.post<RubricOptimizationResponse>(
        "/api/v1/rubric/optimize",
        {
          task_context: request.taskContext,
          agent_output: request.agentOutput,
          evaluation_criteria: request.evaluationCriteria,
        }
      );

      return {
        rewardScore: response.data.rewardScore,
        reasoning: response.data.reasoning,
        improvementSuggestions: response.data.improvementSuggestions,
        metadata: response.data.metadata,
      };
    });
  }

  /**
   * Evaluate artifact using self-improving model judge
   *
   * Uses DSPy-optimized prompts for consistent and accurate evaluation.
   *
   * @param request - Judge evaluation request
   * @returns Judge evaluation result
   * @throws Error if evaluation fails
   */
  async evaluateWithJudge(
    request: JudgeEvaluationRequest
  ): Promise<JudgeEvaluationResponse> {
    return this.retryRequest(async () => {
      this.logger.info("Evaluating with judge", {
        judgeType: request.judgeType,
      });

      const response = await this.client.post<JudgeEvaluationResponse>(
        "/api/v1/judge/evaluate",
        {
          judge_type: request.judgeType,
          artifact: request.artifact,
          ground_truth: request.groundTruth,
          context: request.context,
        }
      );

      return {
        judgment: response.data.judgment,
        confidence: response.data.confidence,
        reasoning: response.data.reasoning,
        metadata: response.data.metadata,
      };
    });
  }

  /**
   * Optimize DSPy signature using evaluation data
   *
   * Systematically improves prompts using evaluation-driven optimization.
   *
   * @param request - Signature optimization request
   * @returns Optimization results
   * @throws Error if optimization fails
   */
  async optimizeSignature(
    request: SignatureOptimizationRequest
  ): Promise<SignatureOptimizationResponse> {
    return this.retryRequest(async () => {
      this.logger.info("Optimizing signature", {
        signatureId: request.signatureId,
        optimizer: request.optimizer,
      });

      const response = await this.client.post<SignatureOptimizationResponse>(
        "/api/v1/optimize/signature",
        {
          signature_id: request.signatureId,
          eval_data: request.evalData,
          optimizer: request.optimizer,
        }
      );

      return {
        optimizedSignatureId: response.data.optimizedSignatureId,
        improvementMetrics: response.data.improvementMetrics,
        metadata: response.data.metadata,
      };
    });
  }

  /**
   * Retry request with exponential backoff
   *
   * @param fn - Function to retry
   * @returns Result from function
   * @throws Error if all retries fail
   */
  private async retryRequest<T>(fn: () => Promise<T>): Promise<T> {
    let lastError: Error | undefined;

    for (let attempt = 0; attempt < this.maxRetries; attempt++) {
      try {
        return await fn();
      } catch (error) {
        lastError = error instanceof Error ? error : new Error(String(error));

        if (attempt < this.maxRetries - 1) {
          const delay = this.retryDelay * Math.pow(2, attempt);
          this.logger.warn("Request failed, retrying", {
            attempt: attempt + 1,
            maxRetries: this.maxRetries,
            delay,
            error: this.getErrorMessage(error),
          });

          await this.sleep(delay);
        }
      }
    }

    this.logger.error("Request failed after all retries", {
      maxRetries: this.maxRetries,
      error: lastError,
    });

    throw lastError ?? new Error("Request failed after all retries");
  }

  /**
   * Sleep for specified milliseconds
   *
   * @param ms - Milliseconds to sleep
   */
  private sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  /**
   * Extract error message from error object
   *
   * @param error - Error object
   * @returns Error message
   */
  private getErrorMessage(error: unknown): string {
    if (axios.isAxiosError(error)) {
      const axiosError = error as AxiosError;
      return axiosError.response?.data
        ? JSON.stringify(axiosError.response.data)
        : axiosError.message;
    }

    if (error instanceof Error) {
      return error.message;
    }

    return String(error);
  }
}
