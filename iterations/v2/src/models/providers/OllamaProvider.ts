/**
 * @fileoverview
 * Ollama model provider for local-first AI inference.
 * Integrates with existing Ollama setup from RL-011.
 *
 * @author @darianrosebrook
 */

import type {
  LocalComputeCost,
  OllamaModelConfig,
  PerformanceCharacteristics,
} from "@/types/model-registry";
import {
  LocalModelProvider,
  type ModelGenerationRequest,
  type ModelGenerationResponse,
  type ModelHealthStatus,
} from "./LocalModelProvider";

/**
 * Ollama API response format
 */
interface OllamaGenerateResponse {
  model: string;
  created_at: string;
  response: string;
  done: boolean;
  context?: number[];
  total_duration?: number;
  load_duration?: number;
  prompt_eval_count?: number;
  prompt_eval_duration?: number;
  eval_count?: number;
  eval_duration?: number;
}

/**
 * Ollama provider error
 */
export class OllamaProviderError extends Error {
  constructor(message: string, public code: string) {
    super(message);
    this.name = "OllamaProviderError";
  }
}

/**
 * Ollama model provider
 *
 * Integrates with local Ollama installation for zero-cost,
 * privacy-first AI inference.
 */
export class OllamaProvider extends LocalModelProvider {
  private endpoint: string;
  private isLoaded: boolean = false;
  private lastHealthCheck?: Date;

  constructor(modelConfig: OllamaModelConfig) {
    super(modelConfig);
    this.endpoint = modelConfig.endpoint ?? "http://localhost:11434";
  }

  /**
   * Generate text using Ollama model
   *
   * @param request Generation request
   * @returns Generation response with compute costs
   */
  async generate(
    request: ModelGenerationRequest
  ): Promise<ModelGenerationResponse> {
    const config = this.modelConfig as OllamaModelConfig;

    if (!this.canHandle(request)) {
      throw new OllamaProviderError(
        "Request exceeds model context window",
        "CONTEXT_OVERFLOW"
      );
    }

    const startTime = Date.now();
    const startMemory = this.getMemoryUsage();

    try {
      // Call Ollama API
      const response = await fetch(`${this.endpoint}/api/generate`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          model: config.ollamaName,
          prompt: request.prompt,
          stream: request.stream ?? false,
          options: {
            temperature: request.temperature ?? 0.7,
            top_p: request.topP ?? 0.9,
            stop: request.stopSequences,
            num_predict: request.maxTokens,
          },
        }),
      });

      if (!response.ok) {
        throw new OllamaProviderError(
          `Ollama API error: ${response.statusText}`,
          "API_ERROR"
        );
      }

      const data: OllamaGenerateResponse = await response.json();

      const endTime = Date.now();
      const generationTimeMs = endTime - startTime;
      const endMemory = this.getMemoryUsage();

      // Calculate tokens
      const inputTokens =
        data.prompt_eval_count ?? Math.ceil(request.prompt.length / 4);
      const outputTokens =
        data.eval_count ?? Math.ceil(data.response.length / 4);
      const tokensPerSecond = outputTokens / (generationTimeMs / 1000);

      // Build compute cost
      const computeCost: LocalComputeCost = {
        modelId: this.modelConfig.id,
        operationId: request.requestId ?? `ollama-${Date.now()}`,
        timestamp: new Date(),
        wallClockMs: generationTimeMs,
        cpuTimeMs: data.total_duration
          ? data.total_duration / 1_000_000
          : generationTimeMs,
        peakMemoryMB: endMemory,
        avgMemoryMB: (startMemory + endMemory) / 2,
        cpuUtilization: this.estimateCPUUtilization(data),
        inputTokens,
        outputTokens,
        tokensPerSecond,
      };

      return {
        text: data.response,
        inputTokens,
        outputTokens,
        generationTimeMs,
        tokensPerSecond,
        requestId: request.requestId,
        computeCost,
      };
    } catch (error) {
      if (error instanceof OllamaProviderError) {
        throw error;
      }

      throw new OllamaProviderError(
        `Failed to generate: ${
          error instanceof Error ? error.message : "Unknown error"
        }`,
        "GENERATION_FAILED"
      );
    }
  }

  /**
   * Check Ollama model health
   *
   * @returns Health status
   */
  async checkHealth(): Promise<ModelHealthStatus> {
    const config = this.modelConfig as OllamaModelConfig;
    const startTime = Date.now();

    try {
      // Ping Ollama API
      const response = await fetch(`${this.endpoint}/api/tags`, {
        method: "GET",
      });

      if (!response.ok) {
        return {
          modelId: this.modelConfig.id,
          healthy: false,
          checkedAt: new Date(),
          error: `Ollama not responding: ${response.statusText}`,
        };
      }

      const data = await response.json();
      const models = data.models || [];

      // Check if our model is available
      const modelExists = models.some(
        (m: { name: string }) => m.name === config.ollamaName
      );

      if (!modelExists) {
        return {
          modelId: this.modelConfig.id,
          healthy: false,
          checkedAt: new Date(),
          error: `Model ${config.ollamaName} not found in Ollama`,
        };
      }

      const responseTimeMs = Date.now() - startTime;
      this.lastHealthCheck = new Date();

      return {
        modelId: this.modelConfig.id,
        healthy: true,
        checkedAt: this.lastHealthCheck,
        responseTimeMs,
      };
    } catch (error) {
      return {
        modelId: this.modelConfig.id,
        healthy: false,
        checkedAt: new Date(),
        error: error instanceof Error ? error.message : "Unknown error",
      };
    }
  }

  /**
   * Load model into memory (warm-up)
   *
   * @returns Performance characteristics
   */
  async load(): Promise<PerformanceCharacteristics> {
    const config = this.modelConfig as OllamaModelConfig;

    // Warm up with a small test prompt
    const warmupPrompt = "Hello";
    const startTime = Date.now();
    const startMemory = this.getMemoryUsage();

    try {
      await this.generate({
        prompt: warmupPrompt,
        maxTokens: 5,
        requestId: "warmup",
      });

      const endTime = Date.now();
      const endMemory = this.getMemoryUsage();

      this.isLoaded = true;

      const performance: PerformanceCharacteristics = {
        avgLatencyMs: endTime - startTime,
        p95LatencyMs: endTime - startTime, // Initial estimate
        tokensPerSec: config.tokensPerSec ?? 50, // Use measured or default
        memoryUsageMB: endMemory,
        cpuUtilization: 50, // Estimate
      };

      return performance;
    } catch (error) {
      throw new OllamaProviderError(
        `Failed to load model: ${
          error instanceof Error ? error.message : "Unknown error"
        }`,
        "LOAD_FAILED"
      );
    }
  }

  /**
   * Unload model from memory
   */
  async unload(): Promise<void> {
    // Ollama manages its own memory
    // We just mark as unloaded
    this.isLoaded = false;
  }

  /**
   * Get current performance characteristics
   *
   * @returns Performance metrics
   */
  async getPerformance(): Promise<PerformanceCharacteristics> {
    const config = this.modelConfig as OllamaModelConfig;

    // Return measured performance if available
    return {
      avgLatencyMs: 1000, // Placeholder
      p95LatencyMs: 1500, // Placeholder
      tokensPerSec: config.tokensPerSec ?? 50,
      memoryUsageMB: config.memoryUsageMB ?? 1024,
      cpuUtilization: 50, // Estimate
    };
  }

  /**
   * Check if model is loaded
   */
  isModelLoaded(): boolean {
    return this.isLoaded;
  }

  /**
   * Get Ollama endpoint
   */
  getEndpoint(): string {
    return this.endpoint;
  }

  /**
   * Get memory usage (placeholder - would use actual OS metrics)
   */
  private getMemoryUsage(): number {
    // In real implementation, would use process.memoryUsage()
    // or OS-specific APIs
    return 1024; // MB
  }

  /**
   * Estimate CPU utilization from Ollama response
   */
  private estimateCPUUtilization(data: OllamaGenerateResponse): number {
    // Rough estimate based on duration
    // In real implementation, would measure actual CPU usage
    if (data.total_duration && data.eval_duration) {
      const evalRatio = data.eval_duration / data.total_duration;
      return Math.min(100, evalRatio * 100);
    }

    return 50; // Default estimate
  }
}
