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
import { createClaimExtractor } from "./ClaimExtractor";
import { CredibilityScorer } from "./CredibilityScorer";
import { FactChecker } from "./FactChecker";
import { VerificationDatabaseClient } from "./VerificationDatabaseClient";
import { ConsistencyValidator } from "./validators/ConsistencyValidator";
import { CrossReferenceValidator } from "./validators/CrossReferenceValidator";
import { LogicalValidator } from "./validators/LogicalValidator";
import { StatisticalValidator } from "./validators/StatisticalValidator";
import type {
  ClaimBasedEvaluation,
  ConversationContext,
  EvidenceManifest,
  ExtractedClaim,
} from "./types";

/**
 * Main Verification Engine Implementation
 */
export class VerificationEngineImpl implements VerificationEngine {
  private config: VerificationEngineConfig;
  private factChecker: FactChecker;
  private credibilityScorer: CredibilityScorer;
  private crossReferenceValidator: CrossReferenceValidator;
  private consistencyValidator: ConsistencyValidator;
  private logicalValidator: LogicalValidator;
  private statisticalValidator: StatisticalValidator;
  private dbClient?: VerificationDatabaseClient;
  private activeRequests: Map<string, Promise<VerificationResult>> = new Map();
  private resultCache: Map<string, VerificationResult> = new Map();
  private readonly claimExtractor = createClaimExtractor();

  constructor(
    config: VerificationEngineConfig,
    dbClient?: VerificationDatabaseClient
  ) {
    this.config = config;
    this.factChecker = new FactChecker(config.methods);
    this.credibilityScorer = new CredibilityScorer(config.methods);

    // Initialize validators
    const crossRefConfig = config.methods.find(
      (m) => m.type === VerificationType.CROSS_REFERENCE
    );
    this.crossReferenceValidator = new CrossReferenceValidator(
      crossRefConfig?.config ?? {}
    );

    const consistencyConfig = config.methods.find(
      (m) => m.type === VerificationType.CONSISTENCY_CHECK
    );
    this.consistencyValidator = new ConsistencyValidator(
      consistencyConfig?.config ?? {}
    );

    const logicalConfig = config.methods.find(
      (m) => m.type === VerificationType.LOGICAL_VALIDATION
    );
    this.logicalValidator = new LogicalValidator(logicalConfig?.config ?? {});

    const statisticalConfig = config.methods.find(
      (m) => m.type === VerificationType.STATISTICAL_VALIDATION
    );
    this.statisticalValidator = new StatisticalValidator(
      statisticalConfig?.config ?? {}
    );

    this.dbClient = dbClient;
  }

  /**
   * Verify a single request
   */
  async verify(request: VerificationRequest): Promise<VerificationResult> {
    const startTime = Date.now();

    // Validate request
    this.validateRequest(request);

    // Save request to database if available
    if (this.dbClient) {
      try {
        await this.dbClient.saveRequest(request);
      } catch (error) {
        console.warn("Failed to save verification request to database:", error);
      }
    }

    // Check database cache first if available
    if (this.config.cacheEnabled && this.dbClient) {
      try {
        const cacheKey = this.generateCacheKey(request);
        const cached = await this.dbClient.getCachedResult(cacheKey);
        if (cached) {
          events.emit({
            id: `event-${Date.now()}-${Math.random()
              .toString(36)
              .substring(2, 9)}`,
            type: EventTypes.TASK_ASSIGNMENT_ACKNOWLEDGED,
            timestamp: new Date(),
            severity: "info" as any,
            source: "VerificationEngine",
            taskId: request.id,
            metadata: { cacheHit: true, source: "database" },
          });
          return cached;
        }
      } catch (error) {
        console.warn("Failed to check database cache:", error);
      }
    }

    // Check in-memory cache
    if (this.config.cacheEnabled) {
      const cached = this.checkCache(request);
      if (cached) {
        events.emit({
          id: `event-${Date.now()}-${Math.random()
            .toString(36)
            .substring(2, 9)}`,
          type: EventTypes.TASK_ASSIGNMENT_ACKNOWLEDGED,
          timestamp: new Date(),
          severity: "info" as any,
          source: "VerificationEngine",
          taskId: request.id,
          metadata: { cacheHit: true, source: "memory" },
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

      // Save result to database if available
      if (this.dbClient) {
        try {
          await this.dbClient.saveResult(result);

          // Cache result in database
          if (this.config.cacheEnabled) {
            await this.dbClient.cacheResult(
              request,
              result,
              this.config.cacheTtlMs
            );
          }
        } catch (error) {
          console.warn(
            "Failed to save verification result to database:",
            error
          );
        }
      }

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

    // Get real health data from the specific method
    let healthData: {
      available: boolean;
      responseTime: number;
      errorRate: number;
    };
    let successRate: number;
    let averageProcessingTime: number;

    switch (method) {
      case VerificationType.FACT_CHECKING:
        healthData = this.factChecker.getHealth();
        successRate = 1 - healthData.errorRate;
        averageProcessingTime = healthData.responseTime;
        break;
      case VerificationType.CREDIBILITY_SCORING:
        healthData = this.credibilityScorer.getHealth();
        successRate = 1 - healthData.errorRate;
        averageProcessingTime = healthData.responseTime;
        break;
      case VerificationType.LOGICAL_VALIDATION:
        healthData = this.logicalValidator.getHealth();
        successRate = 1 - healthData.errorRate;
        averageProcessingTime = healthData.responseTime;
        break;
      case VerificationType.CROSS_REFERENCE:
        healthData = this.crossReferenceValidator.getHealth();
        successRate = 1 - healthData.errorRate;
        averageProcessingTime = healthData.responseTime;
        break;
      case VerificationType.STATISTICAL_VALIDATION:
        healthData = this.statisticalValidator.getHealth();
        successRate = 1 - healthData.errorRate;
        averageProcessingTime = healthData.responseTime;
        break;
      case VerificationType.CONSISTENCY_VALIDATION:
        healthData = this.consistencyValidator.getHealth();
        successRate = 1 - healthData.errorRate;
        averageProcessingTime = healthData.responseTime;
        break;
      default:
        healthData = { available: false, responseTime: 0, errorRate: 1 };
        successRate = 0;
        averageProcessingTime = 0;
    }

    return {
      type: method,
      enabled: methodConfig.enabled,
      healthy: methodConfig.enabled && healthData.available,
      lastUsed: new Date(), // Would track actual last usage
      successRate: Math.max(0, Math.min(1, successRate)),
      averageProcessingTime:
        averageProcessingTime || methodConfig.timeoutMs * 0.8,
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
        id: `event-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`,
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

      // Enrich request with claim extraction prior to running methods
      await this.enrichRequestWithClaims(request);

      // Determine which methods to use (skip when no claims extracted)
      const methodsToUse =
        Array.isArray(request.claims) && request.claims.length === 0
          ? []
          : this.selectVerificationMethods(request);

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
        id: `event-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`,
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
        id: `event-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`,
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

    if (!request.verificationTypes || request.verificationTypes.length === 0) {
      throw new VerificationError(
        "At least one verification type is required",
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
          result = await this.crossReferenceValidator.verify(request);
          break;

        case VerificationType.CONSISTENCY_CHECK:
          result = await this.consistencyValidator.verify(request);
          break;

        case VerificationType.LOGICAL_VALIDATION:
          result = await this.logicalValidator.verify(request);
          break;

        case VerificationType.STATISTICAL_VALIDATION:
          result = await this.statisticalValidator.verify(request);
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
    const noClaimsExtracted =
      Array.isArray(request.claims) && request.claims.length === 0;

    if (noClaimsExtracted) {
      return {
        requestId: request.id,
        verdict: VerificationVerdict.INSUFFICIENT_DATA,
        confidence: 0,
        reasoning: [
          "No verifiable claims extracted; treat content as an unverified document requiring additional evidence.",
        ],
        supportingEvidence: [],
        contradictoryEvidence: [],
        verificationMethods: methodResults,
        processingTimeMs,
        claimEvaluation: request.claimEvaluation,
        claims: request.claims ?? [],
      };
    }

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
        claimEvaluation: request.claimEvaluation,
        claims: request.claims ?? [],
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

    // Aggregate evidence from all verification methods
    const supportingEvidence: Evidence[] = [];
    const contradictoryEvidence: Evidence[] = [];
    const evidenceMap = new Map<string, Evidence>();

    // Collect all evidence from verification methods
    for (const result of validResults) {
      if (result.supportingEvidence) {
        for (const evidence of result.supportingEvidence) {
          const key = this.generateEvidenceKey(evidence);
          if (!evidenceMap.has(key)) {
            evidenceMap.set(key, evidence);
            supportingEvidence.push(evidence);
          }
        }
      }

      if (result.contradictoryEvidence) {
        for (const evidence of result.contradictoryEvidence) {
          const key = this.generateEvidenceKey(evidence);
          if (!evidenceMap.has(key)) {
            evidenceMap.set(key, evidence);
            contradictoryEvidence.push(evidence);
          }
        }
      }
    }

    // Resolve conflicts between supporting and contradictory evidence
    const resolvedEvidence = this.resolveEvidenceConflicts(
      supportingEvidence,
      contradictoryEvidence
    );

    return {
      requestId: request.id,
      verdict: consensusVerdict,
      confidence: Math.min(1.0, adjustedConfidence),
      reasoning: allReasoning,
      supportingEvidence: resolvedEvidence.supporting,
      contradictoryEvidence: resolvedEvidence.contradictory,
      verificationMethods: methodResults,
      methodResults: methodResults, // Alias for backward compatibility
      processingTimeMs,
      claimEvaluation: request.claimEvaluation,
      claims: request.claims ?? [],
    };
  }

  /**
   * Check cache for existing result
   */
  private async enrichRequestWithClaims(
    request: VerificationRequest
  ): Promise<void> {
    const conversationContext = this.createConversationContext(request);
    const evidenceManifest = this.createEvidenceManifest(request);

    const evaluation: ClaimBasedEvaluation =
      await this.claimExtractor.evaluateWithClaims(request.content, {
        conversationContext,
        evidenceManifest,
      });

    request.claims = this.mergeClaims(request.claims, evaluation.extractedClaims);
    request.claimEvaluation = evaluation;
    request.conversationContext = conversationContext;
    request.evidenceManifest = evidenceManifest;
  }

  private createConversationContext(
    request: VerificationRequest
  ): ConversationContext {
    const provided =
      request.conversationContext ||
      (request.metadata?.conversationContext as ConversationContext | undefined);

    if (provided) {
      return {
        conversationId: provided.conversationId,
        tenantId: provided.tenantId,
        previousMessages: [...(provided.previousMessages ?? [])],
        metadata: {
          ...(provided.metadata ?? {}),
          ...(request.metadata ?? {}),
        },
      };
    }

    const previousMessages = [
      ...this.extractStringArray(request.context),
      ...this.extractStringArray(request.metadata?.previousMessages),
      ...this.extractStringArray(request.metadata?.history),
    ];

    const resolvedConversationId =
      (request.metadata?.conversationId as string) ??
      request.id ??
      `verification-${Date.now()}`;
    const resolvedTenantId =
      (request.metadata?.tenantId as string) ??
      (request.metadata?.requesterId as string) ??
      "arbiter";

    return {
      conversationId: String(resolvedConversationId),
      tenantId: String(resolvedTenantId),
      previousMessages,
      metadata: { ...(request.metadata ?? {}) },
    };
  }

  private createEvidenceManifest(
    request: VerificationRequest
  ): EvidenceManifest {
    if (request.evidenceManifest) {
      return request.evidenceManifest;
    }

    const fromMetadata = request.metadata?.evidenceManifest as
      | EvidenceManifest
      | undefined;

    if (fromMetadata) {
      return fromMetadata;
    }

    return {
      sources: [],
      evidence: [],
      quality: 0,
      cawsCompliant: false,
    };
  }

  private mergeClaims(
    existing: ExtractedClaim[] | undefined,
    generated: ExtractedClaim[]
  ): ExtractedClaim[] {
    const combined = [...(existing ?? []), ...generated];
    const unique = new Map<string, ExtractedClaim>();

    for (const claim of combined) {
      const key = (claim.id || claim.statement).toLowerCase();
      const current = unique.get(key);

      if (!current || (claim.confidence ?? 0) > (current.confidence ?? 0)) {
        unique.set(key, claim);
      }
    }

    return Array.from(unique.values());
  }

  private extractStringArray(value: unknown): string[] {
    if (!value) {
      return [];
    }

    if (Array.isArray(value)) {
      return value
        .map((item) => (typeof item === "string" ? item.trim() : ""))
        .filter((item) => item.length > 0);
    }

    if (typeof value === "string") {
      const trimmed = value.trim();
      return trimmed.length > 0 ? [trimmed] : [];
    }

    return [];
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
      conversationId: request.conversationContext?.conversationId,
      claimSignatures: request.claims
        ?.map((claim) => claim.statement.toLowerCase())
        .sort(),
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

  /**
   * Get method performance statistics from database
   */
  async getMethodPerformance() {
    if (!this.dbClient) {
      return [];
    }

    try {
      return await this.dbClient.getMethodPerformance();
    } catch (error) {
      console.warn("Failed to get method performance from database:", error);
      return [];
    }
  }

  /**
   * Get evidence quality statistics from database
   */
  async getEvidenceQualityStats() {
    if (!this.dbClient) {
      return [];
    }

    try {
      return await this.dbClient.getEvidenceQualityStats();
    } catch (error) {
      console.warn(
        "Failed to get evidence quality stats from database:",
        error
      );
      return [];
    }
  }

  /**
   * Generate a unique key for evidence deduplication
   */
  private generateEvidenceKey(evidence: Evidence): string {
    // Create a key based on evidence content and source
    const content = JSON.stringify(evidence.content);
    const source = evidence.source || "unknown";
    const type = evidence.type || "unknown";

    // Use a simple hash of the content for deduplication
    return `${type}:${source}:${this.simpleHash(content)}`;
  }

  /**
   * Simple hash function for evidence deduplication
   */
  private simpleHash(str: string): string {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = (hash << 5) - hash + char;
      hash = hash & hash; // Convert to 32-bit integer
    }
    return Math.abs(hash).toString(36);
  }

  /**
   * Check if two pieces of evidence are conflicting
   */
  private isEvidenceConflicting(
    evidence1: Evidence,
    evidence2: Evidence
  ): boolean {
    // Simple conflict detection based on content similarity
    // In a real implementation, this would use more sophisticated NLP
    const content1 = JSON.stringify(evidence1.content).toLowerCase();
    const content2 = JSON.stringify(evidence2.content).toLowerCase();

    // Check for significant overlap in content
    const words1 = content1.split(/\s+/);
    const words2 = content2.split(/\s+/);

    const commonWords = words1.filter((word) => words2.includes(word));
    const similarity =
      commonWords.length / Math.max(words1.length, words2.length);

    // Consider it conflicting if similarity is above threshold
    return similarity > 0.3;
  }

  /**
   * Resolve a conflict between two pieces of evidence
   */
  private resolveEvidenceConflict(
    supporting: Evidence,
    contradictory: Evidence
  ): { winner: "supporting" | "contradictory"; reasoning: string } {
    // Simple resolution based on evidence strength and recency
    const supportStrength = this.calculateEvidenceStrength(supporting);
    const contraStrength = this.calculateEvidenceStrength(contradictory);

    // If strengths are similar, prefer more recent evidence
    if (Math.abs(supportStrength - contraStrength) < 0.1) {
      const supportTime = supporting.timestamp?.getTime() || 0;
      const contraTime = contradictory.timestamp?.getTime() || 0;

      if (supportTime > contraTime) {
        return {
          winner: "supporting",
          reasoning: "Supporting evidence is more recent",
        };
      } else {
        return {
          winner: "contradictory",
          reasoning: "Contradictory evidence is more recent",
        };
      }
    }

    // Otherwise, prefer stronger evidence
    if (supportStrength > contraStrength) {
      return {
        winner: "supporting",
        reasoning: `Supporting evidence is stronger (${supportStrength.toFixed(
          2
        )} vs ${contraStrength.toFixed(2)})`,
      };
    } else {
      return {
        winner: "contradictory",
        reasoning: `Contradictory evidence is stronger (${contraStrength.toFixed(
          2
        )} vs ${supportStrength.toFixed(2)})`,
      };
    }
  }

  /**
   * Calculate the strength of a piece of evidence
   */
  private calculateEvidenceStrength(evidence: Evidence): number {
    let strength = 0.5; // Base strength

    // Boost strength based on evidence type
    switch (evidence.type) {
      case "factual":
        strength += 0.3;
        break;
      case "statistical":
        strength += 0.2;
        break;
      case "testimonial":
        strength += 0.1;
        break;
      case "circumstantial":
        strength -= 0.1;
        break;
    }

    // Boost strength based on source reliability
    if (evidence.source) {
      if (
        evidence.source.includes("official") ||
        evidence.source.includes("government")
      ) {
        strength += 0.2;
      } else if (
        evidence.source.includes("peer-reviewed") ||
        evidence.source.includes("academic")
      ) {
        strength += 0.15;
      } else if (
        evidence.source.includes("news") ||
        evidence.source.includes("media")
      ) {
        strength += 0.05;
      }
    }

    // Boost strength based on recency (if timestamp available)
    if (evidence.timestamp) {
      const age = Date.now() - evidence.timestamp.getTime();
      const ageInDays = age / (1000 * 60 * 60 * 24);

      if (ageInDays < 1) {
        strength += 0.1; // Very recent
      } else if (ageInDays < 7) {
        strength += 0.05; // Recent
      } else if (ageInDays > 365) {
        strength -= 0.1; // Old
      }
    }

    return Math.max(0, Math.min(1, strength));
  }

  /**
   * Resolve conflicts between supporting and contradictory evidence
   * by comparing credibility scores and removing duplicates
   */
  private resolveEvidenceConflicts(
    supportingEvidence: Evidence[],
    contradictoryEvidence: Evidence[]
  ): { supporting: Evidence[]; contradictory: Evidence[] } {
    const supportingMap = new Map<string, Evidence>();
    const contradictoryMap = new Map<string, Evidence>();

    // Process supporting evidence
    for (const evidence of supportingEvidence) {
      const key = this.generateEvidenceKey(evidence);
      const existing = supportingMap.get(key);

      if (!existing) {
        supportingMap.set(key, evidence);
      } else {
        // If we have duplicate evidence, keep the one with higher credibility
        if (
          this.calculateEvidenceStrength(evidence) >
          this.calculateEvidenceStrength(existing)
        ) {
          supportingMap.set(key, evidence);
        }
      }
    }

    // Process contradictory evidence
    for (const evidence of contradictoryEvidence) {
      const key = this.generateEvidenceKey(evidence);
      const existing = contradictoryMap.get(key);

      if (!existing) {
        contradictoryMap.set(key, evidence);
      } else {
        // If we have duplicate evidence, keep the one with higher credibility
        if (
          this.calculateEvidenceStrength(evidence) >
          this.calculateEvidenceStrength(existing)
        ) {
          contradictoryMap.set(key, evidence);
        }
      }
    }

    // Remove contradictory evidence that has stronger supporting evidence
    const finalSupporting: Evidence[] = [];
    const finalContradictory: Evidence[] = [];

    for (const [key, supporting] of supportingMap.entries()) {
      const contradictory = contradictoryMap.get(key);

      if (contradictory) {
        // Compare evidence strength
        const supportingStrength = this.calculateEvidenceStrength(supporting);
        const contradictoryStrength =
          this.calculateEvidenceStrength(contradictory);

        if (supportingStrength > contradictoryStrength) {
          // Supporting evidence is stronger, keep it
          finalSupporting.push(supporting);
        } else if (contradictoryStrength > supportingStrength) {
          // Contradictory evidence is stronger, keep it
          finalContradictory.push(contradictory);
        } else {
          // Equal strength - keep both but mark as conflicting
          supporting.metadata = {
            ...supporting.metadata,
            conflicting: true,
            conflictStrength: contradictoryStrength,
          };
          contradictory.metadata = {
            ...contradictory.metadata,
            conflicting: true,
            conflictStrength: supportingStrength,
          };
          finalSupporting.push(supporting);
          finalContradictory.push(contradictory);
        }

        // Remove from contradictory map since we've processed it
        contradictoryMap.delete(key);
      } else {
        // No conflict, add to final supporting evidence
        finalSupporting.push(supporting);
      }
    }

    // Add remaining contradictory evidence that wasn't in conflict
    for (const contradictory of contradictoryMap.values()) {
      finalContradictory.push(contradictory);
    }

    return {
      supporting: finalSupporting,
      contradictory: finalContradictory,
    };
  }
}
