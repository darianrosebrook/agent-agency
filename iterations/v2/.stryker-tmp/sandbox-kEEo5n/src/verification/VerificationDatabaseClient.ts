/**
 * @fileoverview Verification Database Client (ARBITER-007)
 *
 * Handles all database operations for the verification engine including
 * request/result persistence, caching, method performance tracking,
 * and evidence storage.
 *
 * Uses centralized ConnectionPoolManager for connection sharing and multi-tenant support.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { PoolClient } from "pg";
import { ConnectionPoolManager } from "../database/ConnectionPoolManager";
import {
  VerificationRequest,
  VerificationResult,
  VerificationType,
  VerificationVerdict,
} from "../types/verification";

/**
 * Method performance statistics
 */
export interface MethodPerformanceStats {
  methodType: VerificationType;
  enabled: boolean;
  priority: number;
  totalRequests: number;
  successfulRequests: number;
  failedRequests: number;
  successRate: number;
  averageProcessingTimeMs: number;
  accuracyRate?: number;
  lastUsedAt?: Date;
  lastErrorAt?: Date;
  lastErrorMessage?: string;
}

/**
 * Evidence quality statistics
 */
export interface EvidenceQualityStats {
  verificationMethod: VerificationType;
  totalEvidence: number;
  avgRelevance: number;
  avgCredibility: number;
  supportingCount: number;
  contradictingCount: number;
  uniqueResults: number;
}

/**
 * Verification evidence for database storage
 */
export interface VerificationEvidence {
  resultId: string;
  requestId: string;
  sourceUrl?: string;
  sourceTitle?: string;
  sourcePublisher?: string;
  contentExcerpt?: string;
  contentHash?: string;
  relevanceScore: number;
  credibilityScore: number;
  supportingClaim: boolean;
  evidenceType: string;
  publishDate?: Date;
  verificationMethod: VerificationType;
  metadata?: Record<string, any>;
}

/**
 * Method statistics for updating
 */
export interface MethodStats {
  totalRequests: number;
  successfulRequests: number;
  failedRequests: number;
  averageProcessingTimeMs: number;
  lastErrorMessage?: string;
}

/**
 * Verification Database Client
 *
 * Manages all database interactions for the verification engine
 * including persistence, caching, and analytics.
 * Uses centralized ConnectionPoolManager for connection sharing.
 */
export class VerificationDatabaseClient {
  private poolManager: ConnectionPoolManager;
  private initialized = false;

  constructor() {
    // Use centralized pool manager
    this.poolManager = ConnectionPoolManager.getInstance();
  }

  /**
   * Initialize the database client and verify connection
   */
  async initialize(): Promise<void> {
    if (this.initialized) {
      return;
    }

    try {
      const client = await this.poolManager.getPool().connect();
      await client.query("SELECT 1");
      client.release();
      this.initialized = true;
    } catch (error) {
      throw new Error(
        `Failed to initialize verification database client: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  /**
   * Save a verification request to the database
   */
  async saveRequest(
    request: VerificationRequest,
    _tenantId?: string
  ): Promise<void> {
    const query = `
      INSERT INTO verification_requests (
        id, content, source, context, priority,
        timeout_ms, verification_types, metadata,
        status, cache_key
      ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
      ON CONFLICT (id) DO UPDATE SET
        status = EXCLUDED.status,
        started_at = CASE WHEN EXCLUDED.status = 'processing' THEN NOW() ELSE verification_requests.started_at END
    `;

    const cacheKey = this.generateCacheKey(request);

    await this.poolManager
      .getPool()
      .query(query, [
        request.id,
        request.content,
        request.source ?? null,
        request.context ?? null,
        request.priority,
        request.timeoutMs ?? 30000,
        request.verificationTypes ?? [],
        JSON.stringify(request.metadata ?? {}),
        "pending",
        cacheKey,
      ]);
  }

  /**
   * Update request status
   */
  async updateRequestStatus(
    requestId: string,
    status: string,
    processingTimeMs?: number,
    errorMessage?: string,
    errorCode?: string,
    _tenantId?: string
  ): Promise<void> {
    const query = `
      UPDATE verification_requests
      SET status = $2,
          completed_at = CASE WHEN $2 IN ('completed', 'failed', 'cancelled') THEN NOW() ELSE completed_at END,
          processing_time_ms = $3,
          error_message = $4,
          error_code = $5
      WHERE id = $1
    `;

    await this.poolManager
      .getPool()
      .query(query, [
        requestId,
        status,
        processingTimeMs ?? null,
        errorMessage ?? null,
        errorCode ?? null,
      ]);
  }

  /**
   * Save a verification result to the database
   */
  async saveResult(
    result: VerificationResult,
    _tenantId?: string
  ): Promise<void> {
    const client = await this.poolManager.getPool().connect();

    try {
      await client.query("BEGIN");

      // Update request status
      await client.query(
        `
        UPDATE verification_requests
        SET status = $2,
            completed_at = NOW(),
            processing_time_ms = $3
        WHERE id = $1
      `,
        [
          result.requestId,
          result.error ? "failed" : "completed",
          result.processingTimeMs,
        ]
      );

      // Insert result
      const resultQuery = `
        INSERT INTO verification_results (
          request_id, verdict, confidence, reasoning,
          supporting_evidence, contradictory_evidence,
          verification_methods, processing_time_ms,
          error_message, error_code
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id
      `;

      const resultData = await client.query(resultQuery, [
        result.requestId,
        result.verdict,
        result.confidence,
        result.reasoning,
        JSON.stringify(result.supportingEvidence),
        JSON.stringify(result.contradictoryEvidence),
        JSON.stringify(result.verificationMethods),
        result.processingTimeMs,
        result.error ?? null,
        null, // error_code - would need to be added to VerificationResult type
      ]);

      const resultId = resultData.rows[0].id;

      // Save evidence
      const allEvidence = [
        ...result.supportingEvidence.map((e) => ({ ...e, supporting: true })),
        ...result.contradictoryEvidence.map((e) => ({
          ...e,
          supporting: false,
        })),
      ];

      for (const evidence of allEvidence) {
        await this.saveEvidenceInternal(client, {
          resultId,
          requestId: result.requestId,
          sourceUrl: evidence.source,
          sourceTitle: evidence.metadata?.title,
          sourcePublisher: evidence.metadata?.publisher,
          contentExcerpt: evidence.content,
          contentHash: this.hashContent(evidence.content),
          relevanceScore: evidence.relevance,
          credibilityScore: evidence.credibility,
          supportingClaim: evidence.supporting,
          evidenceType: evidence.metadata?.type ?? "general",
          publishDate: evidence.metadata?.publishDate,
          verificationMethod:
            evidence.metadata?.method ?? VerificationType.FACT_CHECKING,
          metadata: evidence.metadata,
        });
      }

      await client.query("COMMIT");
    } catch (error) {
      await client.query("ROLLBACK");
      throw error;
    } finally {
      client.release();
    }
  }

  /**
   * Get a verification result by request ID
   */
  async getResult(
    requestId: string,
    _tenantId?: string
  ): Promise<VerificationResult | null> {
    const query = `
      SELECT
        vr.request_id,
        vr.verdict,
        vr.confidence,
        vr.reasoning,
        vr.supporting_evidence,
        vr.contradictory_evidence,
        vr.verification_methods,
        vr.processing_time_ms,
        vr.error_message
      FROM verification_results vr
      WHERE vr.request_id = $1
      ORDER BY vr.created_at DESC
      LIMIT 1
    `;

    const result = await this.poolManager.getPool().query(query, [requestId]);

    if (result.rows.length === 0) {
      return null;
    }

    const row = result.rows[0];

    return {
      requestId: row.request_id,
      verdict: row.verdict as VerificationVerdict,
      confidence: parseFloat(row.confidence),
      reasoning: row.reasoning,
      supportingEvidence:
        typeof row.supporting_evidence === "string"
          ? JSON.parse(row.supporting_evidence)
          : row.supporting_evidence,
      contradictoryEvidence:
        typeof row.contradictory_evidence === "string"
          ? JSON.parse(row.contradictory_evidence)
          : row.contradictory_evidence,
      verificationMethods:
        typeof row.verification_methods === "string"
          ? JSON.parse(row.verification_methods)
          : row.verification_methods,
      processingTimeMs: parseInt(row.processing_time_ms, 10),
      error: row.error_message ?? undefined,
    };
  }

  /**
   * Get cached result by cache key
   */
  async getCachedResult(
    cacheKey: string,
    _tenantId?: string
  ): Promise<VerificationResult | null> {
    const query = `
      SELECT
        result_data,
        expires_at
      FROM verification_cache
      WHERE cache_key = $1
        AND expires_at > NOW()
      LIMIT 1
    `;

    const result = await this.poolManager.getPool().query(query, [cacheKey]);

    if (result.rows.length === 0) {
      return null;
    }

    // Update access statistics
    await this.poolManager.getPool().query(
      `
      UPDATE verification_cache
      SET access_count = access_count + 1,
          last_accessed_at = NOW()
      WHERE cache_key = $1
    `,
      [cacheKey]
    );

    return result.rows[0].result_data as VerificationResult;
  }

  /**
   * Cache a verification result
   */
  async cacheResult(
    request: VerificationRequest,
    result: VerificationResult,
    ttlMs: number,
    _tenantId?: string
  ): Promise<void> {
    const cacheKey = this.generateCacheKey(request);
    const requestHash = this.hashContent(
      `${request.id}:${JSON.stringify(request)}`
    );
    const resultHash = this.hashContent(JSON.stringify(result));
    const expiresAt = new Date(Date.now() + ttlMs);
    const sizeBytes = JSON.stringify(result).length;

    const query = `
      INSERT INTO verification_cache (
        cache_key, request_hash, result_data, result_hash,
        expires_at, size_bytes
      ) VALUES ($1, $2, $3, $4, $5, $6)
      ON CONFLICT (cache_key) DO UPDATE SET
        result_data = EXCLUDED.result_data,
        result_hash = EXCLUDED.result_hash,
        expires_at = EXCLUDED.expires_at,
        access_count = verification_cache.access_count + 1,
        last_accessed_at = NOW()
    `;

    await this.poolManager
      .getPool()
      .query(query, [
        cacheKey,
        requestHash,
        JSON.stringify(result),
        resultHash,
        expiresAt,
        sizeBytes,
      ]);
  }

  /**
   * Clean up expired cache entries
   */
  async cleanupExpiredCache(_tenantId?: string): Promise<number> {
    const query = `
      DELETE FROM verification_cache
      WHERE expires_at < NOW()
    `;

    const result = await this.poolManager.getPool().query(query);
    return result.rowCount ?? 0;
  }

  /**
   * Update method statistics
   */
  async updateMethodStats(
    methodType: VerificationType,
    stats: MethodStats,
    _tenantId?: string
  ): Promise<void> {
    const query = `
      UPDATE verification_methods
      SET total_requests = $2,
          successful_requests = $3,
          failed_requests = $4,
          average_processing_time_ms = $5,
          last_used_at = NOW(),
          last_error_message = $6,
          last_error_at = CASE WHEN $6 IS NOT NULL THEN NOW() ELSE last_error_at END
      WHERE method_type = $1
    `;

    await this.poolManager
      .getPool()
      .query(query, [
        methodType,
        stats.totalRequests,
        stats.successfulRequests,
        stats.failedRequests,
        stats.averageProcessingTimeMs,
        stats.lastErrorMessage ?? null,
      ]);
  }

  /**
   * Get method performance statistics
   */
  async getMethodPerformance(
    _tenantId?: string
  ): Promise<MethodPerformanceStats[]> {
    const query = `
      SELECT
        method_type,
        enabled,
        priority,
        total_requests,
        successful_requests,
        failed_requests,
        average_processing_time_ms,
        accuracy_rate,
        last_used_at,
        last_error_at,
        last_error_message
      FROM verification_methods
      ORDER BY priority ASC
    `;

    const result = await this.poolManager.getPool().query(query);

    return result.rows.map((row) => ({
      methodType: row.method_type as VerificationType,
      enabled: row.enabled,
      priority: row.priority,
      totalRequests: parseInt(row.total_requests, 10),
      successfulRequests: parseInt(row.successful_requests, 10),
      failedRequests: parseInt(row.failed_requests, 10),
      successRate:
        row.total_requests > 0
          ? (row.successful_requests / row.total_requests) * 100
          : 0,
      averageProcessingTimeMs: parseFloat(
        row.average_processing_time_ms ?? "0"
      ),
      accuracyRate: row.accuracy_rate
        ? parseFloat(row.accuracy_rate)
        : undefined,
      lastUsedAt: row.last_used_at,
      lastErrorAt: row.last_error_at,
      lastErrorMessage: row.last_error_message,
    }));
  }

  /**
   * Save evidence (internal with client)
   */
  private async saveEvidenceInternal(
    client: PoolClient,
    evidence: VerificationEvidence
  ): Promise<void> {
    const query = `
      INSERT INTO verification_evidence (
        result_id, request_id, source_url, source_title,
        source_publisher, content_excerpt, content_hash,
        relevance_score, credibility_score, supporting_claim,
        evidence_type, publish_date, verification_method, metadata
      ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
    `;

    await client.query(query, [
      evidence.resultId,
      evidence.requestId,
      evidence.sourceUrl ?? null,
      evidence.sourceTitle ?? null,
      evidence.sourcePublisher ?? null,
      evidence.contentExcerpt ?? null,
      evidence.contentHash ?? null,
      evidence.relevanceScore,
      evidence.credibilityScore,
      evidence.supportingClaim,
      evidence.evidenceType,
      evidence.publishDate ?? null,
      evidence.verificationMethod,
      JSON.stringify(evidence.metadata ?? {}),
    ]);
  }

  /**
   * Save evidence (public)
   */
  async saveEvidence(
    evidence: VerificationEvidence,
    _tenantId?: string
  ): Promise<void> {
    const client = await this.poolManager.getPool().connect();
    try {
      await this.saveEvidenceInternal(client, evidence);
    } finally {
      client.release();
    }
  }

  /**
   * Get evidence quality statistics
   */
  async getEvidenceQualityStats(
    _tenantId?: string
  ): Promise<EvidenceQualityStats[]> {
    const query = `
      SELECT
        verification_method,
        COUNT(*) as total_evidence,
        AVG(relevance_score) as avg_relevance,
        AVG(credibility_score) as avg_credibility,
        COUNT(CASE WHEN supporting_claim THEN 1 END) as supporting_count,
        COUNT(CASE WHEN NOT supporting_claim THEN 1 END) as contradicting_count,
        COUNT(DISTINCT result_id) as unique_results
      FROM verification_evidence
      GROUP BY verification_method
      ORDER BY total_evidence DESC
    `;

    const result = await this.poolManager.getPool().query(query);

    return result.rows.map((row) => ({
      verificationMethod: row.verification_method as VerificationType,
      totalEvidence: parseInt(row.total_evidence, 10),
      avgRelevance: parseFloat(row.avg_relevance),
      avgCredibility: parseFloat(row.avg_credibility),
      supportingCount: parseInt(row.supporting_count, 10),
      contradictingCount: parseInt(row.contradicting_count, 10),
      uniqueResults: parseInt(row.unique_results, 10),
    }));
  }

  /**
   * Generate cache key for request
   * Uses request ID to ensure uniqueness per request
   */
  private generateCacheKey(request: VerificationRequest): string {
    // Use request ID as the primary cache key to ensure uniqueness
    // This prevents duplicate key violations while still allowing caching
    return `req_${request.id}`;
  }

  /**
   * Hash content for deduplication
   */
  private hashContent(content: string): string {
    const crypto = require("crypto");
    return crypto.createHash("sha256").update(content).digest("hex");
  }

  /**
   * Check if client is initialized
   */
  isInitialized(): boolean {
    return this.initialized;
  }
}
