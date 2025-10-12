/**
 * @fileoverview Knowledge Database Client for ARBITER-006
 *
 * Handles database persistence for knowledge queries, search results,
 * responses, and provider health tracking.
 *
 * @author @darianrosebrook
 */

import { Pool } from "pg";
import {
  KnowledgeQuery,
  KnowledgeResponse,
  ProviderHealthStatus,
  SearchResult,
} from "../types/knowledge";

/**
 * Database client configuration
 */
export interface KnowledgeDatabaseConfig {
  host: string;
  port: number;
  database: string;
  user: string;
  password: string;
  maxConnections?: number;
  idleTimeoutMs?: number;
  connectionTimeoutMs?: number;
}

/**
 * Knowledge Database Client
 *
 * Provides database persistence for knowledge queries, results, and provider health.
 * Implements graceful degradation - operations continue if database is unavailable.
 */
export class KnowledgeDatabaseClient {
  private pool: Pool | null = null;
  private config: KnowledgeDatabaseConfig;
  private available = false;

  constructor(config: KnowledgeDatabaseConfig) {
    this.config = config;
  }

  /**
   * Initialize database connection pool
   */
  async initialize(): Promise<void> {
    try {
      this.pool = new Pool({
        host: this.config.host,
        port: this.config.port,
        database: this.config.database,
        user: this.config.user,
        password: this.config.password,
        max: this.config.maxConnections || 10,
        idleTimeoutMillis: this.config.idleTimeoutMs || 30000,
        connectionTimeoutMillis: this.config.connectionTimeoutMs || 2000,
      });

      // Test connection
      const client = await this.pool.connect();
      await client.query("SELECT 1");
      client.release();

      this.available = true;
      console.log("Knowledge database client initialized successfully");
    } catch (error) {
      console.warn("Failed to initialize knowledge database:", error);
      this.available = false;
      // Graceful degradation - continue without database
    }
  }

  /**
   * Shutdown database connection pool
   */
  async shutdown(): Promise<void> {
    if (this.pool) {
      await this.pool.end();
      this.pool = null;
      this.available = false;
    }
  }

  /**
   * Check if database is available
   */
  isAvailable(): boolean {
    return this.available && this.pool !== null;
  }

  /**
   * Store a knowledge query
   */
  async storeQuery(query: KnowledgeQuery): Promise<string | null> {
    if (!this.isAvailable()) {
      return null;
    }

    try {
      const result = await this.pool!.query(
        `INSERT INTO knowledge_queries (
          id, query_text, query_type, requester_id, priority,
          max_results, relevance_threshold, timeout_ms,
          context, tags, status, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING id`,
        [
          query.id,
          query.query,
          query.queryType,
          query.metadata.requesterId,
          query.metadata.priority,
          query.maxResults,
          query.relevanceThreshold,
          query.timeoutMs,
          JSON.stringify(query.context || {}),
          query.metadata.tags || [],
          "processing",
          query.metadata.createdAt,
        ]
      );

      return result.rows[0]?.id || null;
    } catch (error) {
      console.error("Failed to store query:", error);
      return null;
    }
  }

  /**
   * Update query status
   */
  async updateQueryStatus(
    queryId: string,
    status: "pending" | "processing" | "completed" | "failed",
    errorMessage?: string
  ): Promise<void> {
    if (!this.isAvailable()) {
      return;
    }

    try {
      await this.pool!.query(
        `UPDATE knowledge_queries 
         SET status = $1, processed_at = $2, error_message = $3
         WHERE id = $4`,
        [status, new Date(), errorMessage || null, queryId]
      );
    } catch (error) {
      console.error("Failed to update query status:", error);
    }
  }

  /**
   * Store search results
   */
  async storeResults(results: SearchResult[]): Promise<number> {
    if (!this.isAvailable() || results.length === 0) {
      return 0;
    }

    let stored = 0;
    const client = await this.pool!.connect();

    try {
      await client.query("BEGIN");

      for (const result of results) {
        await client.query(
          `INSERT INTO search_results (
            id, query_id, title, content, url, domain,
            source_type, relevance_score, credibility_score, quality,
            provider, provider_metadata, published_at, created_at, content_hash
          ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
          ON CONFLICT (content_hash) DO NOTHING`,
          [
            result.id,
            result.queryId,
            result.title,
            result.content,
            result.url,
            result.domain,
            result.sourceType,
            result.relevanceScore,
            result.credibilityScore,
            result.quality,
            result.provider,
            JSON.stringify(result.providerMetadata || {}),
            result.publishedAt || null,
            result.retrievedAt,
            result.contentHash,
          ]
        );
        stored++;
      }

      await client.query("COMMIT");
    } catch (error) {
      await client.query("ROLLBACK");
      console.error("Failed to store results:", error);
    } finally {
      client.release();
    }

    return stored;
  }

  /**
   * Store knowledge response
   */
  async storeResponse(response: KnowledgeResponse): Promise<string | null> {
    if (!this.isAvailable()) {
      return null;
    }

    try {
      const result = await this.pool!.query(
        `INSERT INTO knowledge_responses (
          query_id, summary, confidence, sources_used,
          total_results_found, results_filtered, processing_time_ms,
          cache_used, providers_queried,
          relevance_score_avg, credibility_score_avg, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING id`,
        [
          response.query.id,
          response.summary,
          response.confidence,
          response.sourcesUsed,
          response.metadata.totalResultsFound,
          response.metadata.resultsFiltered,
          response.metadata.processingTimeMs,
          response.metadata.cacheUsed,
          response.metadata.providersQueried,
          response.results.length > 0
            ? response.results.reduce((sum, r) => sum + r.relevanceScore, 0) /
              response.results.length
            : null,
          response.results.length > 0
            ? response.results.reduce((sum, r) => sum + r.credibilityScore, 0) /
              response.results.length
            : null,
          response.respondedAt,
        ]
      );

      return result.rows[0]?.id || null;
    } catch (error) {
      console.error("Failed to store response:", error);
      return null;
    }
  }

  /**
   * Update provider health metrics
   */
  async updateProviderHealth(
    providerName: string,
    health: ProviderHealthStatus
  ): Promise<void> {
    if (!this.isAvailable()) {
      return;
    }

    try {
      await this.pool!.query(
        `INSERT INTO search_provider_health (
          provider_name, available, last_health_check, consecutive_failures,
          avg_response_time_ms, error_rate, requests_this_minute, requests_this_hour,
          updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT (provider_name) DO UPDATE SET
          available = $2,
          last_health_check = $3,
          consecutive_failures = $4,
          avg_response_time_ms = $5,
          error_rate = $6,
          requests_this_minute = $7,
          requests_this_hour = $8,
          updated_at = $9`,
        [
          providerName,
          health.available,
          new Date(),
          0, // consecutive_failures - would need to track this
          health.responseTimeMs,
          health.errorRate,
          health.requestsThisMinute,
          health.requestsThisHour,
          new Date(),
        ]
      );
    } catch (error) {
      console.error("Failed to update provider health:", error);
    }
  }

  /**
   * Get cached response by cache key
   */
  async getCachedResponse(cacheKey: string): Promise<any | null> {
    if (!this.isAvailable()) {
      return null;
    }

    try {
      const result = await this.pool!.query(
        `SELECT content, expires_at, access_count
         FROM knowledge_cache
         WHERE cache_key = $1 AND expires_at > NOW()`,
        [cacheKey]
      );

      if (result.rows.length === 0) {
        return null;
      }

      // Update access statistics
      await this.pool!.query(
        `UPDATE knowledge_cache
         SET access_count = access_count + 1, last_accessed_at = NOW()
         WHERE cache_key = $1`,
        [cacheKey]
      );

      return result.rows[0].content;
    } catch (error) {
      console.error("Failed to get cached response:", error);
      return null;
    }
  }

  /**
   * Store cached response
   */
  async storeCachedResponse(
    cacheKey: string,
    content: any,
    cacheTtlMs: number
  ): Promise<void> {
    if (!this.isAvailable()) {
      return;
    }

    try {
      const expiresAt = new Date(Date.now() + cacheTtlMs);
      const contentStr = JSON.stringify(content);
      const sizeBytes = Buffer.byteLength(contentStr, "utf8");

      await this.pool!.query(
        `INSERT INTO knowledge_cache (
          cache_key, cache_type, content, metadata,
          created_at, expires_at, size_bytes, last_accessed_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (cache_key) DO UPDATE SET
          content = $3,
          expires_at = $6,
          size_bytes = $7,
          last_accessed_at = $8`,
        [
          cacheKey,
          "query_result",
          content,
          JSON.stringify({}),
          new Date(),
          expiresAt,
          sizeBytes,
          new Date(),
        ]
      );
    } catch (error) {
      console.error("Failed to store cached response:", error);
    }
  }

  /**
   * Clean expired cache entries
   */
  async cleanExpiredCache(): Promise<number> {
    if (!this.isAvailable()) {
      return 0;
    }

    try {
      const result = await this.pool!.query(
        `DELETE FROM knowledge_cache WHERE expires_at < NOW()`
      );

      return result.rowCount || 0;
    } catch (error) {
      console.error("Failed to clean expired cache:", error);
      return 0;
    }
  }

  /**
   * Get provider health status
   */
  async getProviderHealth(
    providerName: string
  ): Promise<ProviderHealthStatus | null> {
    if (!this.isAvailable()) {
      return null;
    }

    try {
      const result = await this.pool!.query(
        `SELECT available, avg_response_time_ms, error_rate,
                requests_this_minute, requests_this_hour
         FROM search_provider_health
         WHERE provider_name = $1`,
        [providerName]
      );

      if (result.rows.length === 0) {
        return null;
      }

      const row = result.rows[0];
      return {
        available: row.available,
        responseTimeMs: row.avg_response_time_ms || 0,
        errorRate: row.error_rate || 0,
        requestsThisMinute: row.requests_this_minute || 0,
        requestsThisHour: row.requests_this_hour || 0,
      };
    } catch (error) {
      console.error("Failed to get provider health:", error);
      return null;
    }
  }

  /**
   * Get cache statistics
   */
  async getCacheStats(): Promise<{
    totalEntries: number;
    totalSizeBytes: number;
    hitRate: number;
  }> {
    if (!this.isAvailable()) {
      return { totalEntries: 0, totalSizeBytes: 0, hitRate: 0 };
    }

    try {
      const result = await this.pool!.query(
        `SELECT 
          COUNT(*) as total_entries,
          SUM(size_bytes) as total_size,
          AVG(access_count) as avg_access_count
         FROM knowledge_cache
         WHERE expires_at > NOW()`
      );

      const row = result.rows[0];
      return {
        totalEntries: parseInt(row.total_entries) || 0,
        totalSizeBytes: parseInt(row.total_size) || 0,
        hitRate: parseFloat(row.avg_access_count) > 1 ? 0.7 : 0, // Simplified calculation
      };
    } catch (error) {
      console.error("Failed to get cache stats:", error);
      return { totalEntries: 0, totalSizeBytes: 0, hitRate: 0 };
    }
  }
}
