/**
 * Main verification engine for information validation and fact-checking
 * @author @darianrosebrook
 */

import {
  EngineHealth,
  VerificationEngine as IVerificationEngine,
  MethodStatus,
  VerificationCacheEntry,
  VerificationEngineConfig,
  VerificationError,
  VerificationErrorCode,
  VerificationPriority,
  VerificationProvider,
  VerificationRequest,
  VerificationResult,
  VerificationType,
  VerificationVerdict,
} from "../types/verification";

export class VerificationEngine implements IVerificationEngine {
  private readonly config: VerificationEngineConfig;
  private readonly providers: Map<VerificationType, VerificationProvider>;
  private readonly cache = new Map<string, VerificationCacheEntry>();
  private readonly activeVerifications = new Set<string>();
  private cleanupTimer?: ReturnType<typeof setInterval>;

  constructor(
    config: Partial<VerificationEngineConfig> = {},
    providers?: VerificationProvider[]
  ) {
    this.config = {
      defaultTimeoutMs: 10000,
      maxConcurrentVerifications: 20,
      minConfidenceThreshold: 0.6,
      maxEvidencePerMethod: 5,
      methods: [],
      cacheEnabled: true,
      cacheTtlMs: 1800000, // 30 minutes
      retryAttempts: 2,
      retryDelayMs: 1000,
      ...config,
    };

    this.providers = new Map();
    if (providers) {
      providers.forEach((provider) => {
        this.providers.set(provider.type, provider);
      });
    }

    this.initializeProviders();
  }

  async verify(request: VerificationRequest): Promise<VerificationResult> {
    const startTime = Date.now();

    try {
      // Check cache first
      if (this.config.cacheEnabled) {
        const cached = this.checkCache(request);
        if (cached) {
          return {
            ...cached,
            processingTimeMs: Date.now() - startTime,
          };
        }
      }

      // Validate request
      this.validateRequest(request);

      // Check concurrency limits
      if (
        this.activeVerifications.size >= this.config.maxConcurrentVerifications
      ) {
        throw new VerificationError(
          "Maximum concurrent verifications exceeded",
          VerificationErrorCode.RATE_LIMIT_EXCEEDED,
          request.id
        );
      }

      this.activeVerifications.add(request.id);

      try {
        // Select verification methods
        const methods = this.selectMethods(request);

        // Execute verification methods in parallel
        const methodResults = await this.executeVerificationMethods(
          request,
          methods
        );

        // Aggregate results
        const result = this.aggregateResults(request, methodResults, startTime);

        // Cache the result
        if (this.config.cacheEnabled) {
          this.cacheResult(request, result);
        }

        return result;
      } finally {
        this.activeVerifications.delete(request.id);
      }
    } catch (error) {
      const processingTime = Date.now() - startTime;

      if (error instanceof VerificationError) {
        return {
          requestId: request.id,
          verdict: VerificationVerdict.UNVERIFIED,
          confidence: 0,
          reasoning: [error.message],
          supportingEvidence: [],
          contradictoryEvidence: [],
          verificationMethods: [],
          processingTimeMs: processingTime,
          error: error.message,
        };
      }

      throw error;
    }
  }

  async verifyBatch(
    requests: VerificationRequest[]
  ): Promise<VerificationResult[]> {
    // Prioritize requests and execute in batches
    const prioritizedRequests = this.prioritizeRequests(requests);
    const batches = this.createBatches(
      prioritizedRequests,
      this.config.maxConcurrentVerifications
    );

    const results: VerificationResult[] = [];

    for (const batch of batches) {
      const batchPromises = batch.map((request) => this.verify(request));
      const batchResults = await Promise.all(batchPromises);
      results.push(...batchResults);
    }

    return results;
  }

  getSupportedMethods(): VerificationType[] {
    return Array.from(this.providers.keys());
  }

  getMethodStatus(method: VerificationType): MethodStatus {
    const provider = this.providers.get(method);
    if (!provider) {
      return {
        type: method,
        enabled: false,
        healthy: false,
      };
    }

    // This would call provider.getHealth() in a real implementation
    return {
      type: method,
      enabled: true,
      healthy: true,
      successRate: 0.95, // Mock value
      averageProcessingTime: 2000, // Mock value
    };
  }

  async healthCheck(): Promise<EngineHealth> {
    const totalMethods = this.providers.size;
    const enabledMethods = Array.from(this.providers.values()).filter((p) =>
      this.isMethodEnabled(p.type)
    ).length;

    // Check health of each provider
    const healthPromises = Array.from(this.providers.values()).map(
      async (provider) => {
        try {
          const status = await provider.getHealth();
          return status.healthy;
        } catch {
          return false;
        }
      }
    );

    const healthResults = await Promise.all(healthPromises);
    const healthyMethods = healthResults.filter(Boolean).length;

    return {
      healthy: healthyMethods > 0,
      totalMethods,
      enabledMethods,
      healthyMethods,
      cacheSize: this.cache.size,
      activeVerifications: this.activeVerifications.size,
    };
  }

  private validateRequest(request: VerificationRequest): void {
    if (!request.content || request.content.trim().length === 0) {
      throw new VerificationError(
        "Verification request content cannot be empty",
        VerificationErrorCode.INVALID_REQUEST,
        request.id
      );
    }

    if (request.content.length > 10000) {
      throw new VerificationError(
        "Verification request content too long (max 10000 characters)",
        VerificationErrorCode.INVALID_REQUEST,
        request.id
      );
    }

    if (request.verificationTypes && request.verificationTypes.length === 0) {
      throw new VerificationError(
        "At least one verification type must be specified",
        VerificationErrorCode.INVALID_REQUEST,
        request.id
      );
    }
  }

  private selectMethods(request: VerificationRequest): VerificationType[] {
    let candidates: VerificationType[];

    if (request.verificationTypes && request.verificationTypes.length > 0) {
      // Use specified methods if provided
      candidates = request.verificationTypes;
    } else {
      // Use all available methods
      candidates = this.getSupportedMethods();
    }

    // Filter to enabled methods
    candidates = candidates.filter((method) => this.isMethodEnabled(method));

    // Sort by priority (this would be configurable)
    const priorityOrder = [
      VerificationType.FACT_CHECKING,
      VerificationType.SOURCE_CREDIBILITY,
      VerificationType.CROSS_REFERENCE,
      VerificationType.CONSISTENCY_CHECK,
      VerificationType.LOGICAL_VALIDATION,
      VerificationType.STATISTICAL_VALIDATION,
    ];

    candidates.sort(
      (a, b) => priorityOrder.indexOf(a) - priorityOrder.indexOf(b)
    );

    return candidates;
  }

  private async executeVerificationMethods(
    request: VerificationRequest,
    methods: VerificationType[]
  ): Promise<any[]> {
    const methodPromises = methods.map(async (method) => {
      const provider = this.providers.get(method);
      if (!provider) {
        throw new VerificationError(
          `No provider available for method: ${method}`,
          VerificationErrorCode.METHOD_UNAVAILABLE,
          request.id,
          method
        );
      }

      try {
        const timeoutMs = request.timeoutMs || this.config.defaultTimeoutMs;
        const result = await this.executeWithTimeout(
          provider.verify(request),
          timeoutMs
        );
        return result;
      } catch (error) {
        // Return failed result instead of throwing
        return {
          method,
          verdict: VerificationVerdict.UNVERIFIED,
          confidence: 0,
          reasoning: `Method failed: ${
            error instanceof Error ? error.message : "Unknown error"
          }`,
          processingTimeMs: 0,
          evidenceCount: 0,
        };
      }
    });

    return Promise.all(methodPromises);
  }

  private aggregateResults(
    request: VerificationRequest,
    methodResults: any[],
    startTime: number
  ): VerificationResult {
    // Filter out failed methods
    const validResults = methodResults.filter(
      (result) => result.verdict !== VerificationVerdict.UNVERIFIED
    );

    if (validResults.length === 0) {
      return {
        requestId: request.id,
        verdict: VerificationVerdict.UNVERIFIED,
        confidence: 0,
        reasoning: ["No verification methods succeeded"],
        supportingEvidence: [],
        contradictoryEvidence: [],
        verificationMethods: methodResults,
        processingTimeMs: Date.now() - startTime,
      };
    }

    // Aggregate verdicts
    const verdictCounts = this.countVerdicts(validResults);
    const consensusVerdict = this.determineConsensusVerdict(verdictCounts);

    // Calculate overall confidence
    const avgConfidence =
      validResults.reduce((sum, r) => sum + r.confidence, 0) /
      validResults.length;
    const consensusConfidence = this.calculateConsensusConfidence(
      verdictCounts,
      validResults.length
    );

    const overallConfidence = Math.min(
      avgConfidence * consensusConfidence,
      1.0
    );

    // Generate reasoning
    const reasoning = this.generateReasoning(consensusVerdict, validResults);

    return {
      requestId: request.id,
      verdict: consensusVerdict,
      confidence: overallConfidence,
      reasoning,
      supportingEvidence: [], // Would be populated from method results
      contradictoryEvidence: [], // Would be populated from method results
      verificationMethods: methodResults,
      processingTimeMs: Date.now() - startTime,
    };
  }

  private countVerdicts(results: any[]): Map<VerificationVerdict, number> {
    const counts = new Map<VerificationVerdict, number>();

    results.forEach((result) => {
      const count = counts.get(result.verdict) || 0;
      counts.set(result.verdict, count + 1);
    });

    return counts;
  }

  private determineConsensusVerdict(
    counts: Map<VerificationVerdict, number>
  ): VerificationVerdict {
    let maxCount = 0;
    let consensusVerdict = VerificationVerdict.UNVERIFIED;

    for (const [verdict, count] of counts) {
      if (count > maxCount) {
        maxCount = count;
        consensusVerdict = verdict;
      }
    }

    // Check for contradictory results
    if (counts.size > 1 && maxCount < counts.size) {
      return VerificationVerdict.CONTRADICTORY;
    }

    return consensusVerdict;
  }

  private calculateConsensusConfidence(
    counts: Map<VerificationVerdict, number>,
    totalResults: number
  ): number {
    if (totalResults === 0) return 0;

    const maxCount = Math.max(...counts.values());
    const consensusRatio = maxCount / totalResults;

    // Boost confidence for strong consensus
    if (consensusRatio >= 0.8) return 1.0;
    if (consensusRatio >= 0.6) return 0.8;
    if (consensusRatio >= 0.4) return 0.6;

    return 0.4; // Low confidence for weak consensus
  }

  private generateReasoning(
    verdict: VerificationVerdict,
    results: any[]
  ): string[] {
    const reasoning: string[] = [];

    reasoning.push(`Consensus verdict: ${verdict}`);

    const methodCount = results.length;
    reasoning.push(
      `${methodCount} verification method${
        methodCount === 1 ? "" : "s"
      } applied`
    );

    // Add method-specific reasoning
    results.forEach((result) => {
      if (result.reasoning) {
        reasoning.push(`${result.method}: ${result.reasoning}`);
      }
    });

    return reasoning;
  }

  private checkCache(request: VerificationRequest): VerificationResult | null {
    const cacheKey = this.createCacheKey(request);
    const entry = this.cache.get(cacheKey);

    if (!entry) return null;

    // Check if expired
    if (Date.now() - entry.timestamp.getTime() > entry.ttlMs) {
      this.cache.delete(cacheKey);
      return null;
    }

    // Update access statistics
    entry.accessCount++;
    entry.lastAccessed = new Date();

    return entry.result;
  }

  private createCacheKey(request: VerificationRequest): string {
    // Create a deterministic key based on request content
    const keyData = {
      content: request.content,
      source: request.source,
      context: request.context,
      verificationTypes: request.verificationTypes?.sort(),
    };

    return `verification:${JSON.stringify(keyData)}`;
  }

  private cacheResult(
    request: VerificationRequest,
    result: VerificationResult
  ): void {
    const cacheKey = this.createCacheKey(request);
    const ttlMs =
      request.priority === VerificationPriority.CRITICAL
        ? this.config.cacheTtlMs * 2
        : this.config.cacheTtlMs;

    const entry: VerificationCacheEntry = {
      key: cacheKey,
      result,
      timestamp: new Date(),
      ttlMs,
      accessCount: 1,
      lastAccessed: new Date(),
    };

    this.cache.set(cacheKey, entry);
  }

  private prioritizeRequests(
    requests: VerificationRequest[]
  ): VerificationRequest[] {
    const priorityOrder = {
      [VerificationPriority.CRITICAL]: 4,
      [VerificationPriority.HIGH]: 3,
      [VerificationPriority.MEDIUM]: 2,
      [VerificationPriority.LOW]: 1,
    };

    return requests.sort(
      (a, b) => priorityOrder[b.priority] - priorityOrder[a.priority]
    );
  }

  private createBatches<T>(items: T[], batchSize: number): T[][] {
    const batches: T[][] = [];
    for (let i = 0; i < items.length; i += batchSize) {
      batches.push(items.slice(i, i + batchSize));
    }
    return batches;
  }

  private isMethodEnabled(method: VerificationType): boolean {
    // Check if method is configured and enabled
    const methodConfig = this.config.methods.find((m) => m.type === method);
    return methodConfig?.enabled ?? false;
  }

  private async executeWithTimeout<T>(
    promise: Promise<T>,
    timeoutMs: number
  ): Promise<T> {
    const timeoutPromise = new Promise<never>((_, reject) =>
      setTimeout(() => reject(new Error("Operation timeout")), timeoutMs)
    );

    return Promise.race([promise, timeoutPromise]);
  }

  private initializeProviders(): void {
    // Set up cleanup timer
    this.cleanupTimer = setInterval(() => {
      this.performCacheCleanup();
    }, 300000); // Clean every 5 minutes
  }

  private performCacheCleanup(): void {
    const now = Date.now();
    const expiredKeys: string[] = [];

    for (const [key, entry] of this.cache.entries()) {
      if (now - entry.timestamp.getTime() > entry.ttlMs) {
        expiredKeys.push(key);
      }
    }

    expiredKeys.forEach((key) => this.cache.delete(key));
  }

  destroy(): void {
    if (this.cleanupTimer) {
      clearInterval(this.cleanupTimer);
      this.cleanupTimer = undefined;
    }
    this.cache.clear();
  }
}
