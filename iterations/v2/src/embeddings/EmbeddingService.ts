/**
 * @fileoverview Embedding Service using Ollama embeddinggemma
 *
 * Provides embedding generation using the existing Ollama infrastructure.
 * Integrates with gemma3n:e2b ecosystem for consistent vector space.
 *
 * @author @darianrosebrook
 */

import { createHash } from "crypto";
import { PerformanceLogger } from "../logging/StructuredLogger.js";
import { CircuitBreaker, CircuitBreakerFactory } from "./CircuitBreaker.js";
import {
  ConfigurationError,
  EmbeddingConfigValidator,
} from "./ConfigValidation.js";
import { EmbeddingHealthCheck, HealthStatus } from "./HealthCheck.js";
import { EmbeddingRateLimiter, RateLimitError } from "./RateLimiter.js";
import { RetryPresets, retryWithBackoff } from "./RetryUtils.js";
import {
  BatchEmbeddingRequest,
  BatchEmbeddingResponse,
  EmbeddingConfig,
  EmbeddingError,
  EmbeddingRequest,
  EmbeddingResponse,
  IEmbeddingService,
} from "./types.js";

/**
 * Embedding service implementation using Ollama embeddinggemma
 */
export class EmbeddingService implements IEmbeddingService {
  private endpoint: string;
  private model: string;
  private timeout: number;
  private cache: Map<string, number[]>;
  private maxCacheSize: number;
  private circuitBreaker: CircuitBreaker;
  private logger: PerformanceLogger;
  private rateLimiter: EmbeddingRateLimiter;
  private healthCheck: EmbeddingHealthCheck;

  constructor(config: EmbeddingConfig) {
    // Validate configuration at startup
    const validation = EmbeddingConfigValidator.validate(config);
    if (!validation.isValid) {
      throw new ConfigurationError(
        "Invalid embedding configuration",
        validation.errors,
        validation.warnings
      );
    }

    // Log warnings
    for (const warning of validation.warnings) {
      console.warn(`Configuration warning: ${warning.message}`);
    }

    this.endpoint = config.ollamaEndpoint || "http://localhost:11434";
    this.model = config.model || "embeddinggemma";
    this.timeout = config.timeout || 30000;
    this.maxCacheSize = config.cacheSize || 1000;
    this.cache = new Map();
    this.circuitBreaker = CircuitBreakerFactory.createForOllama(
      `${this.model}-service`
    );
    this.logger = new PerformanceLogger("EmbeddingService");
    this.rateLimiter = new EmbeddingRateLimiter(
      config.rateLimitPerSecond || 50
    );
    this.healthCheck = new EmbeddingHealthCheck("1.0.0");
  }

  /**
   * Generate embedding for a single text
   */
  async generateEmbedding(text: string): Promise<number[]> {
    if (!text || text.trim().length === 0) {
      throw new EmbeddingError("Text cannot be empty", "EMPTY_TEXT");
    }

    // Check rate limit
    const rateLimitResult = this.rateLimiter.checkLimit("embeddings");
    if (!rateLimitResult.allowed) {
      throw new RateLimitError(rateLimitResult);
    }

    const cacheKey = this.hashText(text);
    const cached = this.cache.get(cacheKey);
    if (cached) {
      return cached;
    }

    try {
      const response = await this.callEmbeddingAPI({
        text: text.trim(),
        model: this.model,
      });

      const embedding = response.embedding;
      this.validateEmbedding(embedding);

      // Cache the result
      this.cacheResult(cacheKey, embedding);

      return embedding;
    } catch (error) {
      if (error instanceof EmbeddingError) {
        throw error;
      }
      throw new EmbeddingError(
        `Failed to generate embedding: ${error.message}`,
        "API_ERROR",
        error as Error
      );
    }
  }

  /**
   * Generate embeddings for multiple texts in batch
   */
  async generateBatch(texts: string[]): Promise<number[][]> {
    if (!texts || texts.length === 0) {
      throw new EmbeddingError("Texts array cannot be empty", "EMPTY_TEXTS");
    }

    if (texts.length > 100) {
      throw new EmbeddingError(
        "Batch size cannot exceed 100 texts",
        "BATCH_SIZE_EXCEEDED"
      );
    }

    // Check rate limit for batch operations
    const rateLimitResult = this.rateLimiter.checkLimit("batch-embeddings");
    if (!rateLimitResult.allowed) {
      throw new RateLimitError(rateLimitResult);
    }

    const results: number[][] = [];
    const uncachedTexts: string[] = [];
    const uncachedIndices: number[] = [];

    // Check cache first
    for (let i = 0; i < texts.length; i++) {
      const text = texts[i];
      const cacheKey = this.hashText(text);
      const cached = this.cache.get(cacheKey);

      if (cached) {
        results[i] = cached;
      } else {
        results[i] = null as any; // Placeholder
        uncachedTexts.push(text);
        uncachedIndices.push(i);
      }
    }

    // Generate embeddings for uncached texts
    if (uncachedTexts.length > 0) {
      try {
        const batchResponse = await this.callBatchEmbeddingAPI({
          texts: uncachedTexts,
          model: this.model,
        });

        // Fill in the results
        for (let i = 0; i < uncachedTexts.length; i++) {
          const embedding = batchResponse.embeddings[i];
          this.validateEmbedding(embedding);

          const originalIndex = uncachedIndices[i];
          results[originalIndex] = embedding;

          // Cache the result
          const cacheKey = this.hashText(uncachedTexts[i]);
          this.cacheResult(cacheKey, embedding);
        }
      } catch (error) {
        throw new EmbeddingError(
          `Failed to generate batch embeddings: ${error.message}`,
          "BATCH_API_ERROR",
          error as Error
        );
      }
    }

    return results;
  }

  /**
   * Check if the embedding service is available
   */
  async isAvailable(): Promise<boolean> {
    try {
      const response = await fetch(`${this.endpoint}/api/tags`, {
        method: "GET",
        signal: AbortSignal.timeout(5000),
      });

      if (!response.ok) {
        return false;
      }

      const data = await response.json();
      return (
        data.models?.some(
          (model: any) =>
            model.name === this.model || model.name.includes("embeddinggemma")
        ) || false
      );
    } catch (error) {
      return false;
    }
  }

  /**
   * Get cache statistics
   */
  getCacheStats(): { size: number; maxSize: number; hitRate: number } {
    return {
      size: this.cache.size,
      maxSize: this.maxCacheSize,
      hitRate: 0, // Would need to track hits/misses for accurate rate
    };
  }

  /**
   * Clear the embedding cache
   */
  clearCache(): void {
    this.cache.clear();
  }

  /**
   * Call the Ollama embeddings API with circuit breaker protection and retry logic
   */
  private async callEmbeddingAPI(
    request: EmbeddingRequest
  ): Promise<EmbeddingResponse> {
    return retryWithBackoff(
      () =>
        this.circuitBreaker.execute(async () => {
          const controller = new AbortController();
          const timeoutId = setTimeout(() => controller.abort(), this.timeout);

          try {
            const response = await fetch(`${this.endpoint}/api/embeddings`, {
              method: "POST",
              headers: {
                "Content-Type": "application/json",
              },
              body: JSON.stringify({
                model: request.model || this.model,
                prompt: request.text,
              }),
              signal: controller.signal,
            });

            clearTimeout(timeoutId);

            if (!response.ok) {
              const errorText = await response.text();
              throw new EmbeddingError(
                `Ollama API error: ${response.status} ${response.statusText} - ${errorText}`,
                "API_ERROR"
              );
            }

            const data = await response.json();

            if (!data.embedding || !Array.isArray(data.embedding)) {
              throw new EmbeddingError(
                "Invalid embedding response format",
                "INVALID_RESPONSE"
              );
            }

            return {
              embedding: data.embedding,
              model: data.model || request.model || this.model,
              usage: data.usage,
            };
          } catch (error) {
            clearTimeout(timeoutId);

            if (error.name === "AbortError") {
              throw new EmbeddingError(
                `Embedding generation timeout after ${this.timeout}ms`,
                "TIMEOUT"
              );
            }

            throw error;
          }
        }),
      RetryPresets.STANDARD
    );
  }

  /**
   * Call the Ollama API for batch embeddings (sequential for now)
   */
  private async callBatchEmbeddingAPI(
    request: BatchEmbeddingRequest
  ): Promise<BatchEmbeddingResponse> {
    const embeddings: number[][] = [];
    const usage = {
      prompt_tokens: 0,
      total_tokens: 0,
    };

    // Process sequentially to avoid overwhelming the API
    for (const text of request.texts) {
      const response = await this.callEmbeddingAPI({
        text,
        model: request.model || this.model,
      });

      embeddings.push(response.embedding);

      if (response.usage) {
        usage.prompt_tokens += response.usage.prompt_tokens;
        usage.total_tokens += response.usage.total_tokens;
      }
    }

    return {
      embeddings,
      model: request.model || this.model,
      usage,
    };
  }

  /**
   * Validate embedding vector
   */
  private validateEmbedding(embedding: number[]): void {
    if (!Array.isArray(embedding)) {
      throw new EmbeddingError(
        "Embedding must be an array",
        "INVALID_EMBEDDING_TYPE"
      );
    }

    if (embedding.length !== 768) {
      throw new EmbeddingError(
        `Expected embedding dimension 768, got ${embedding.length}`,
        "INVALID_EMBEDDING_DIMENSION"
      );
    }

    // Check for NaN or infinite values
    for (let i = 0; i < embedding.length; i++) {
      const value = embedding[i];
      if (typeof value !== "number" || !isFinite(value)) {
        throw new EmbeddingError(
          `Invalid embedding value at index ${i}: ${value}`,
          "INVALID_EMBEDDING_VALUE"
        );
      }
    }
  }

  /**
   * Hash text for cache key
   */
  private hashText(text: string): string {
    return createHash("sha256")
      .update(text)
      .update(this.model) // Include model in hash to avoid conflicts
      .digest("hex");
  }

  /**
   * Cache result with LRU eviction
   */
  private cacheResult(key: string, embedding: number[]): void {
    // Simple LRU: if cache is full, remove oldest entry
    if (this.cache.size >= this.maxCacheSize) {
      const firstKey = this.cache.keys().next().value;
      this.cache.delete(firstKey);
    }

    this.cache.set(key, embedding);
  }

  /**
   * Get circuit breaker statistics for monitoring
   */
  getCircuitBreakerStats() {
    return this.circuitBreaker.getStats();
  }

  /**
   * Get circuit breaker state
   */
  getCircuitBreakerState() {
    return this.circuitBreaker.getState();
  }

  /**
   * Reset circuit breaker (for recovery/testing)
   */
  resetCircuitBreaker(): void {
    this.circuitBreaker.reset();
  }

  /**
   * Get rate limiter statistics
   */
  getRateLimiterStats(key?: string) {
    return this.rateLimiter.getStats(key);
  }

  /**
   * Reset rate limiter for a key
   */
  resetRateLimiter(key?: string): void {
    this.rateLimiter.reset(key);
  }

  /**
   * Cleanup old rate limit data
   */
  cleanupRateLimiter(): void {
    this.rateLimiter.cleanup();
  }

  /**
   * Perform comprehensive health check
   */
  async performHealthCheck(): Promise<HealthStatus> {
    return this.healthCheck.checkHealth(this, {
      ollamaEndpoint: this.endpoint,
      circuitBreaker: this.circuitBreaker,
      rateLimiter: this.rateLimiter,
      includeDeepChecks: true,
    });
  }

  /**
   * Perform basic health check (for load balancers)
   */
  async performBasicHealthCheck(): Promise<{
    status: string;
    timestamp: string;
  }> {
    const health = await this.healthCheck.checkHealth(this, {
      ollamaEndpoint: this.endpoint,
      circuitBreaker: this.circuitBreaker,
      rateLimiter: this.rateLimiter,
      includeDeepChecks: false,
    });

    return {
      status: health.status,
      timestamp: health.timestamp,
    };
  }

  /**
   * Graceful shutdown - cleanup resources
   */
  async shutdown(): Promise<void> {
    this.logger.info("Shutting down EmbeddingService");
    this.circuitBreaker.destroy();
    this.cleanupRateLimiter();
    this.clearCache();
    this.logger.info("EmbeddingService shutdown complete");
  }
}
