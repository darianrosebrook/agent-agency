/**
 * @fileoverview
 * Abstract base class for local model providers.
 * Supports Ollama, custom trained models, and hardware-optimized models.
 *
 * @author @darianrosebrook
 */

import type {
  LocalComputeCost,
  LocalModelConfig,
  PerformanceCharacteristics,
} from "@/types/model-registry";

/**
 * Model generation request
 */
export interface ModelGenerationRequest {
  /** Input prompt */
  prompt: string;

  /** Maximum tokens to generate */
  maxTokens?: number;

  /** Temperature (0-1) */
  temperature?: number;

  /** Top-p sampling */
  topP?: number;

  /** Stop sequences */
  stopSequences?: string[];

  /** Stream responses */
  stream?: boolean;

  /** Request ID for tracing */
  requestId?: string;
}

/**
 * Model generation response
 */
export interface ModelGenerationResponse {
  /** Generated text */
  text: string;

  /** Input tokens */
  inputTokens: number;

  /** Output tokens */
  outputTokens: number;

  /** Generation time in milliseconds */
  generationTimeMs: number;

  /** Tokens per second */
  tokensPerSecond: number;

  /** Request ID */
  requestId?: string;

  /** Compute cost details */
  computeCost?: LocalComputeCost;
}

/**
 * Model health status
 */
export interface ModelHealthStatus {
  /** Model ID */
  modelId: string;

  /** Is healthy */
  healthy: boolean;

  /** Health check timestamp */
  checkedAt: Date;

  /** Response time in milliseconds */
  responseTimeMs?: number;

  /** Error message if unhealthy */
  error?: string;
}

/**
 * Abstract local model provider
 *
 * Base class for all local model providers (Ollama, custom, hardware-optimized).
 * Provides common interface for model operations.
 */
export abstract class LocalModelProvider {
  protected modelConfig: LocalModelConfig;

  constructor(modelConfig: LocalModelConfig) {
    this.modelConfig = modelConfig;
  }

  /**
   * Get model configuration
   */
  getConfig(): LocalModelConfig {
    return this.modelConfig;
  }

  /**
   * Generate text from prompt
   *
   * @param request Generation request
   * @returns Generation response
   */
  abstract generate(
    request: ModelGenerationRequest
  ): Promise<ModelGenerationResponse>;

  /**
   * Check model health
   *
   * @returns Health status
   */
  abstract checkHealth(): Promise<ModelHealthStatus>;

  /**
   * Load model into memory (warm-up)
   *
   * @returns Performance characteristics after warm-up
   */
  abstract load(): Promise<PerformanceCharacteristics>;

  /**
   * Unload model from memory
   */
  abstract unload(): Promise<void>;

  /**
   * Get current performance characteristics
   *
   * @returns Current performance metrics
   */
  abstract getPerformance(): Promise<PerformanceCharacteristics>;

  /**
   * Estimate compute cost for a request
   *
   * @param request Generation request
   * @returns Estimated cost
   */
  estimateCost(request: ModelGenerationRequest): LocalComputeCost {
    // Base estimation (override in subclasses for accuracy)
    const estimatedInputTokens = Math.ceil(request.prompt.length / 4);
    const estimatedOutputTokens = request.maxTokens ?? 100;

    return {
      modelId: this.modelConfig.id,
      operationId: request.requestId ?? "unknown",
      timestamp: new Date(),
      wallClockMs: 0,
      cpuTimeMs: 0,
      peakMemoryMB: 0,
      avgMemoryMB: 0,
      cpuUtilization: 0,
      inputTokens: estimatedInputTokens,
      outputTokens: estimatedOutputTokens,
      tokensPerSecond: 0,
    };
  }

  /**
   * Validate if provider can handle request
   *
   * @param request Generation request
   * @returns True if can handle
   */
  canHandle(request: ModelGenerationRequest): boolean {
    // Check if prompt fits in context window
    const estimatedTokens = Math.ceil(request.prompt.length / 4);
    const maxTokens = request.maxTokens ?? 100;

    return estimatedTokens + maxTokens <= this.modelConfig.contextWindow;
  }

  /**
   * Get provider type
   */
  getType(): string {
    return this.modelConfig.type;
  }

  /**
   * Get model ID
   */
  getModelId(): string {
    return this.modelConfig.id;
  }

  /**
   * Get model name
   */
  getModelName(): string {
    return this.modelConfig.name;
  }
}
