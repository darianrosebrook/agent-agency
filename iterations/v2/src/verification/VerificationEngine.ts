/**
 * @fileoverview Verification Engine Core Component (ARBITER-007)
 *
 * Main orchestrator for information validation and fact-checking,
 * coordinating multiple verification methods and aggregating results.
 *
 * @author @darianrosebrook
 */

import { events } from "../orchestrator/EventEmitter";
import { EventTypes } from "../orchestrator/OrchestratorEvents";
import {
  EngineHealth,
  Evidence,
  MethodStatus,
  VerificationEngine,
  VerificationEngineConfig,
  VerificationError,
  VerificationErrorCode,
  VerificationMethodResult,
  VerificationRequest,
  VerificationResult,
  VerificationType,
  VerificationVerdict,
} from "../types/verification";
import { CredibilityScorer } from "./CredibilityScorer";
import { FactChecker } from "./FactChecker";

/**
 * Main Verification Engine Implementation
 */
export class VerificationEngineImpl implements VerificationEngine {
  private config: VerificationEngineConfig;
  private factChecker: FactChecker;
  private credibilityScorer: CredibilityScorer;
  private activeRequests: Map<string, Promise<VerificationResult>> = new Map();
  private resultCache: Map<string, VerificationResult> = new Map();

  constructor(config: VerificationEngineConfig) {
    this.config = config;
    this.factChecker = new FactChecker(config.methods);
    this.credibilityScorer = new CredibilityScorer(config.methods);
  }

  /**
   * Verify a single request
   */
  async verify(request: VerificationRequest): Promise<VerificationResult> {
    const startTime = Date.now();

    // Validate request
    this.validateRequest(request);

    // Check cache first
    if (this.config.cacheEnabled) {
      const cached = this.checkCache(request);
      if (cached) {
        events.emit({
          id: `event-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
          type: EventTypes.TASK_ASSIGNMENT_ACKNOWLEDGED,
          timestamp: new Date(),
          severity: "info" as any,
          source: "VerificationEngine",
          taskId: request.id,
          metadata: { cacheHit: true },
        });
        return cached;
      }
    }

    // Check if request is already being processed
    if (this.activeRequests.has(request.id)) {
      return this.activeRequests.get(request.id)!;
    }

    // Create processing promise
    const processingPromise = this.processVerification(request, startTime);
    this.activeRequests.set(request.id, processingPromise);

    try {
      const result = await processingPromise;
      return result;
    } finally {
      this.activeRequests.delete(request.id);
    }
  }

  /**
   * Verify multiple requests in batch
   */
  async verifyBatch(
    requests: VerificationRequest[]
  ): Promise<VerificationResult[]> {
    // Process requests in parallel with concurrency control
    const batches = this.createBatches(
      requests,
      this.config.maxConcurrentVerifications
    );
    const results: VerificationResult[] = [];

    for (const batch of batches) {
      const batchPromises = batch.map((request) => this.verify(request));
      const batchResults = await Promise.allSettled(batchPromises);

      for (const result of batchResults) {
        if (result.status === "fulfilled") {
          results.push(result.value);
        } else {
          // Create error result for failed verification
          const errorResult: VerificationResult = {
            requestId: "unknown", // Would need to track which request failed
            verdict: VerificationVerdict.UNVERIFIED,
            confidence: 0,
            reasoning: [`Verification failed: ${result.reason}`],
            supportingEvidence: [],
            contradictoryEvidence: [],
            verificationMethods: [],
            processingTimeMs: 0,
            error:
              result.reason instanceof Error
                ? result.reason.message
                : String(result.reason),
          };
          results.push(errorResult);
        }
      }
    }

    return results;
  }

  /**
   * Get supported verification methods
   */
  getSupportedMethods(): VerificationType[] {
    return this.config.methods
      .filter((method) => method.enabled)
      .map((method) => method.type);
  }

  /**
   * Get status of a specific verification method
   */
  getMethodStatus(method: VerificationType): MethodStatus {
    const methodConfig = this.config.methods.find((m) => m.type === method);

    if (!methodConfig) {
      return {
        type: method,
        enabled: false,
        healthy: false,
      };
    }

    // In a real implementation, this would check actual method health
    return {
      type: method,
      enabled: methodConfig.enabled,
      healthy: true, // Simplified - would check actual health
      lastUsed: new Date(),
      successRate: 0.95, // Mock values
      averageProcessingTime: methodConfig.timeoutMs * 0.8,
    };
  }

  /**
   * Perform health check on the verification engine
   */
  async healthCheck(): Promise<EngineHealth> {
    const methodStatuses = this.config.methods.map((method) =>
      this.getMethodStatus(method.type)
    );
    const enabledMethods = methodStatuses.filter((status) => status.enabled);
    const healthyMethods = methodStatuses.filter((status) => status.healthy);

    return {
      healthy: healthyMethods.length > 0,
      totalMethods: methodStatuses.length,
      enabledMethods: enabledMethods.length,
      healthyMethods: healthyMethods.length,
      cacheSize: this.resultCache.size,
      activeVerifications: this.activeRequests.size,
    };
  }

  /**
   * Process a verification request
   */
  private async processVerification(
    request: VerificationRequest,
    startTime: number
  ): Promise<VerificationResult> {
    try {
      // Emit verification started event
      events.emit({
        id: `event-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        type: EventTypes.TASK_ASSIGNMENT_ACKNOWLEDGED,
        timestamp: new Date(),
        severity: "info" as any,
        source: "VerificationEngine",
        taskId: request.id,
        metadata: {
          contentLength: request.content.length,
          verificationTypes: request.verificationTypes,
        },
      });

      // Determine which methods to use
      const methodsToUse = this.selectVerificationMethods(request);

      // Execute verification methods in parallel
      const methodPromises = methodsToUse.map(async (methodType) => {
        try {
          return await this.executeVerificationMethod(methodType, request);
        } catch (error) {
          console.warn(`Verification method ${methodType} failed:`, error);
          return this.createFailedMethodResult(methodType, error);
        }
      });

      const methodResults = await Promise.all(methodPromises);

      // Aggregate results
      const aggregatedResult = this.aggregateResults(
        request,
        methodResults,
        startTime
      );

      // Cache result if enabled
      if (this.config.cacheEnabled) {
        this.cacheResult(request, aggregatedResult);
      }

      // Emit verification completed event
      events.emit({
        id: `event-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        type: EventTypes.TASK_ASSIGNMENT_ACKNOWLEDGED,
        timestamp: new Date(),
        severity: "info" as any,
        source: "VerificationEngine",
        taskId: request.id,
        metadata: {
          verdict: aggregatedResult.verdict,
          confidence: aggregatedResult.confidence,
          methodsUsed: methodResults.length,
          processingTimeMs: aggregatedResult.processingTimeMs,
        },
      });

      return aggregatedResult;
    } catch (error) {
      const processingTimeMs = Date.now() - startTime;

      // Emit verification error event
      events.emit({
        id: `event-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        type: EventTypes.TASK_ASSIGNMENT_ACKNOWLEDGED,
        timestamp: new Date(),
        severity: "error" as any,
        source: "VerificationEngine",
        taskId: request.id,
        metadata: {
          error: error instanceof Error ? error.message : String(error),
          processingTimeMs,
        },
      });

      throw error;
    }
  }

  /**
   * Validate verification request
   */
  private validateRequest(request: VerificationRequest): void {
    if (!request.id || request.id.trim().length === 0) {
      throw new VerificationError(
        "Request ID is required",
        VerificationErrorCode.INVALID_REQUEST
      );
    }

    if (!request.content || request.content.trim().length === 0) {
      throw new VerificationError(
        "Content is required",
        VerificationErrorCode.INVALID_REQUEST,
        request.id
      );
    }

    if (request.content.length > 10000) {
      // Arbitrary limit
      throw new VerificationError(
        "Content too long (max 10000 characters)",
        VerificationErrorCode.INVALID_REQUEST,
        request.id
      );
    }

    if (
      request.timeoutMs &&
      (request.timeoutMs <= 0 || request.timeoutMs > 300000)
    ) {
      throw new VerificationError(
        "Invalid timeout (must be 1-300000ms)",
        VerificationErrorCode.INVALID_REQUEST,
        request.id
      );
    }
  }

  /**
   * Select verification methods to use for the request
   */
  private selectVerificationMethods(
    request: VerificationRequest
  ): VerificationType[] {
    // Use requested methods or fall back to all enabled methods
    const requestedMethods = request.verificationTypes || [];

    if (requestedMethods.length > 0) {
      // Filter to only enabled methods
      return requestedMethods.filter((method) =>
        this.config.methods.some((m) => m.type === method && m.enabled)
      );
    }

    // Use all enabled methods by priority
    return this.config.methods
      .filter((method) => method.enabled)
      .sort((a, b) => a.priority - b.priority)
      .map((method) => method.type);
  }

  /**
   * Execute a single verification method
   */
  private async executeVerificationMethod(
    methodType: VerificationType,
    request: VerificationRequest
  ): Promise<VerificationMethodResult> {
    const methodConfig = this.config.methods.find((m) => m.type === methodType);

    if (!methodConfig) {
      throw new Error(`Method configuration not found: ${methodType}`);
    }

    const methodStartTime = Date.now();

    try {
      let result: VerificationMethodResult;

      switch (methodType) {
        case VerificationType.FACT_CHECKING:
          result = await this.factChecker.verify(request);
          break;

        case VerificationType.SOURCE_CREDIBILITY:
          result = await this.credibilityScorer.verify(request);
          break;

        case VerificationType.CROSS_REFERENCE:
          result = await this.executeCrossReference();
          break;

        case VerificationType.CONSISTENCY_CHECK:
          result = await this.executeConsistencyCheck();
          break;

        case VerificationType.LOGICAL_VALIDATION:
          result = await this.executeLogicalValidation();
          break;

        case VerificationType.STATISTICAL_VALIDATION:
          result = await this.executeStatisticalValidation();
          break;

        default:
          throw new Error(`Unsupported verification method: ${methodType}`);
      }

      const processingTimeMs = Date.now() - methodStartTime;

      // Update method result with timing
      return {
        ...result,
        processingTimeMs,
      };
    } catch (error) {
      const processingTimeMs = Date.now() - methodStartTime;

      return {
        method: methodType,
        verdict: VerificationVerdict.UNVERIFIED,
        confidence: 0,
        reasoning: [
          `Method execution failed: ${
            error instanceof Error ? error.message : String(error)
          }`,
        ],
        processingTimeMs,
        evidenceCount: 0,
      };
    }
  }

  /**
   * Create failed method result
   */
  private createFailedMethodResult(
    methodType: VerificationType,
    error: any
  ): VerificationMethodResult {
    return {
      method: methodType,
      verdict: VerificationVerdict.UNVERIFIED,
      confidence: 0,
      reasoning: [
        `Method failed: ${
          error instanceof Error ? error.message : String(error)
        }`,
      ],
      processingTimeMs: 0,
      evidenceCount: 0,
    };
  }

  /**
   * Aggregate results from multiple verification methods
   */
  private aggregateResults(
    request: VerificationRequest,
    methodResults: VerificationMethodResult[],
    startTime: number
  ): VerificationResult {
    const processingTimeMs = Date.now() - startTime;

    // Filter out failed methods
    const validResults = methodResults.filter(
      (result) => result.confidence > 0
    );

    if (validResults.length === 0) {
      return {
        requestId: request.id,
        verdict: VerificationVerdict.INSUFFICIENT_DATA,
        confidence: 0,
        reasoning: ["No verification methods produced valid results"],
        supportingEvidence: [],
        contradictoryEvidence: [],
        verificationMethods: methodResults,
        processingTimeMs,
      };
    }

    // Calculate consensus verdict
    const verdictCounts = new Map<VerificationVerdict, number>();
    let totalConfidence = 0;

    for (const result of validResults) {
      verdictCounts.set(
        result.verdict,
        (verdictCounts.get(result.verdict) || 0) + 1
      );
      totalConfidence += result.confidence;
    }

    const averageConfidence = totalConfidence / validResults.length;

    // Find most common verdict
    let consensusVerdict = VerificationVerdict.UNVERIFIED;
    let maxCount = 0;

    for (const [verdict, count] of verdictCounts.entries()) {
      if (count > maxCount) {
        maxCount = count;
        consensusVerdict = verdict;
      }
    }

    // Adjust confidence based on consensus strength
    const consensusRatio = maxCount / validResults.length;
    const adjustedConfidence = averageConfidence * (0.5 + 0.5 * consensusRatio);

    // Collect reasoning from all methods
    const allReasoning = validResults.flatMap((result) =>
      Array.isArray(result.reasoning) ? result.reasoning : [result.reasoning]
    );

    // Create aggregated evidence (simplified)
    const supportingEvidence: Evidence[] = [];
    const contradictoryEvidence: Evidence[] = [];

    // In a real implementation, this would aggregate evidence from all methods
    // For now, we create placeholder evidence

    return {
      requestId: request.id,
      verdict: consensusVerdict,
      confidence: Math.min(1.0, adjustedConfidence),
      reasoning: allReasoning,
      supportingEvidence,
      contradictoryEvidence,
      verificationMethods: methodResults,
      processingTimeMs,
    };
  }

  private async executeCrossReference(): Promise<VerificationMethodResult> {
    // Simplified cross-reference implementation
    // In production, this would search multiple sources and check for consistency
    return {
      method: VerificationType.CROSS_REFERENCE,
      verdict: VerificationVerdict.UNVERIFIED, // Placeholder
      confidence: 0.5,
      reasoning: ["Cross-reference verification not fully implemented"],
      processingTimeMs: 100,
      evidenceCount: 0,
    };
  }

  private async executeConsistencyCheck(): Promise<VerificationMethodResult> {
    // Simplified consistency check
    return {
      method: VerificationType.CONSISTENCY_CHECK,
      verdict: VerificationVerdict.UNVERIFIED,
      confidence: 0.5,
      reasoning: ["Consistency check not fully implemented"],
      processingTimeMs: 50,
      evidenceCount: 0,
    };
  }

  private async executeLogicalValidation(): Promise<VerificationMethodResult> {
    // Simplified logical validation
    return {
      method: VerificationType.LOGICAL_VALIDATION,
      verdict: VerificationVerdict.UNVERIFIED,
      confidence: 0.5,
      reasoning: ["Logical validation not fully implemented"],
      processingTimeMs: 75,
      evidenceCount: 0,
    };
  }

  private async executeStatisticalValidation(): Promise<VerificationMethodResult> {
    // Simplified statistical validation
    return {
      method: VerificationType.STATISTICAL_VALIDATION,
      verdict: VerificationVerdict.UNVERIFIED,
      confidence: 0.5,
      reasoning: ["Statistical validation not fully implemented"],
      processingTimeMs: 60,
      evidenceCount: 0,
    };
  }

  /**
   * Check cache for existing result
   */
  private checkCache(request: VerificationRequest): VerificationResult | null {
    const cacheKey = this.generateCacheKey(request);
    const cached = this.resultCache.get(cacheKey);

    if (cached && this.isCacheValid(cached)) {
      return cached;
    }

    return null;
  }

  /**
   * Cache verification result
   */
  private cacheResult(
    request: VerificationRequest,
    result: VerificationResult
  ): void {
    const cacheKey = this.generateCacheKey(request);

    // Store with TTL
    this.resultCache.set(cacheKey, result);

    // Clean up expired entries periodically
    if (this.resultCache.size > 100) {
      this.cleanupExpiredCache();
    }
  }

  /**
   * Generate cache key for request
   */
  private generateCacheKey(request: VerificationRequest): string {
    const keyData = {
      content: request.content,
      source: request.source,
      verificationTypes: request.verificationTypes?.sort(),
    };
    return Buffer.from(JSON.stringify(keyData)).toString("base64");
  }

  /**
   * Check if cached result is still valid
   */
  private isCacheValid(result: VerificationResult): boolean {
    // Cache for configured TTL
    const cacheAge = Date.now() - result.processingTimeMs;
    return cacheAge < this.config.cacheTtlMs;
  }

  /**
   * Clean up expired cache entries
   */
  private cleanupExpiredCache(): void {
    const now = Date.now();
    const maxAge = this.config.cacheTtlMs;

    for (const [key, result] of this.resultCache.entries()) {
      const cacheAge = now - result.processingTimeMs;
      if (cacheAge > maxAge) {
        this.resultCache.delete(key);
      }
    }
  }

  /**
   * Create batches for concurrent processing
   */
  private createBatches<T>(items: T[], batchSize: number): T[][] {
    const batches: T[][] = [];
    for (let i = 0; i < items.length; i += batchSize) {
      batches.push(items.slice(i, i + batchSize));
    }
    return batches;
  }
}
