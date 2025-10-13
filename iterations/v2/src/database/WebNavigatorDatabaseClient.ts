/**
 * @fileoverview Web Navigator Database Client for ARBITER-008
 *
 * Handles database persistence for web content, traversal sessions,
 * cache management, and rate limit tracking.
 *
 * Uses centralized ConnectionPoolManager for connection sharing and multi-tenant support.
 *
 * @author @darianrosebrook
 */

import {
  DomainRateLimit,
  RateLimitStatus,
  TraversalStrategy,
  WebContent,
  WebContentRecord,
} from "../types/web";
import { ConnectionPoolManager } from "./ConnectionPoolManager";

/**
 * Web Navigator Database Client
 *
 * Provides database persistence for web content, traversals, and cache.
 * Implements graceful degradation - operations continue if database is unavailable.
 * Uses centralized ConnectionPoolManager for connection sharing.
 */
export class WebNavigatorDatabaseClient {
  private poolManager: ConnectionPoolManager;
  private available = false;

  constructor() {
    // Use centralized pool manager
    this.poolManager = ConnectionPoolManager.getInstance();
  }

  /**
   * Initialize database connection (verify pool is available)
   */
  async initialize(): Promise<void> {
    try {
      // Verify pool is initialized and accessible
      const client = await this.poolManager.getPool().connect();
      await client.query("SELECT 1");
      client.release();

      this.available = true;
      console.log("Web Navigator database client initialized successfully");
    } catch (error) {
      console.warn("Failed to initialize web navigator database:", error);
      this.available = false;
      // Graceful degradation - continue without database
    }
  }

  /**
   * Check if database is available
   */
  isAvailable(): boolean {
    return this.available && this.poolManager.isInitialized();
  }

  /**
   * Store extracted web content
   */
  async storeContent(
    content: WebContent,
    _tenantId?: string
  ): Promise<string | null> {
    if (!this.isAvailable()) {
      return null;
    }

    try {
      const result = await this.poolManager.getPool().query(
        `INSERT INTO web_content (
          id, url, title, content, html, content_hash, quality, metadata,
          extraction_type, extracted_at, cached_until, extraction_time_ms, content_size_bytes
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        ON CONFLICT (content_hash) DO UPDATE SET
          extracted_at = EXCLUDED.extracted_at,
          cached_until = EXCLUDED.cached_until
        RETURNING id`,
        [
          content.id,
          content.url,
          content.title,
          content.content,
          content.html || null,
          content.contentHash,
          content.quality,
          JSON.stringify(content.metadata),
          "main_content", // Default extraction type
          content.extractedAt,
          new Date(content.extractedAt.getTime() + 24 * 60 * 60 * 1000), // 24h TTL
          0, // extraction_time_ms - would be provided separately
          Buffer.byteLength(content.content, "utf8"),
        ]
      );

      return result.rows[0]?.id || null;
    } catch (error) {
      console.error("Failed to store web content:", error);
      return null;
    }
  }

  /**
   * Retrieve content by URL from cache
   */
  async getContentByUrl(
    url: string,
    _tenantId?: string
  ): Promise<WebContent | null> {
    if (!this.isAvailable()) {
      return null;
    }

    try {
      const result = await this.poolManager.getPool().query(
        `SELECT wc.* 
         FROM web_content wc
         INNER JOIN web_cache cache ON wc.id = cache.content_id
         WHERE cache.url = $1 AND cache.expires_at > NOW()
         ORDER BY wc.extracted_at DESC
         LIMIT 1`,
        [url]
      );

      if (result.rows.length === 0) {
        return null;
      }

      // Update cache hit count
      await this.poolManager.getPool().query(
        `UPDATE web_cache 
         SET hit_count = hit_count + 1, last_accessed = NOW()
         WHERE url = $1`,
        [url]
      );

      return this.rowToWebContent(result.rows[0]);
    } catch (error) {
      console.error("Failed to retrieve cached content:", error);
      return null;
    }
  }

  /**
   * Cache web content with TTL
   */
  async cacheContent(
    url: string,
    contentId: string,
    ttlHours: number = 24,
    _tenantId?: string
  ): Promise<void> {
    if (!this.isAvailable()) {
      return;
    }

    try {
      const expiresAt = new Date(Date.now() + ttlHours * 60 * 60 * 1000);

      // Get content size for cache entry
      const sizeResult = await this.poolManager
        .getPool()
        .query(`SELECT content_size_bytes FROM web_content WHERE id = $1`, [
          contentId,
        ]);

      const cacheSize = sizeResult.rows[0]?.content_size_bytes || 0;

      await this.poolManager.getPool().query(
        `INSERT INTO web_cache (url, content_id, cached_at, expires_at, cache_size_bytes)
         VALUES ($1, $2, NOW(), $3, $4)
         ON CONFLICT (url) DO UPDATE SET
           content_id = EXCLUDED.content_id,
           cached_at = EXCLUDED.cached_at,
           expires_at = EXCLUDED.expires_at,
           cache_size_bytes = EXCLUDED.cache_size_bytes,
           hit_count = 0`,
        [url, contentId, expiresAt, cacheSize]
      );
    } catch (error) {
      console.error("Failed to cache content:", error);
    }
  }

  /**
   * Clean up expired cache entries
   */
  async cleanupExpiredCache(_tenantId?: string): Promise<number> {
    if (!this.isAvailable()) {
      return 0;
    }

    try {
      const result = await this.poolManager
        .getPool()
        .query(`SELECT cleanup_expired_web_cache() as deleted_count`);
      return result.rows[0]?.deleted_count || 0;
    } catch (error) {
      console.error("Failed to cleanup expired cache:", error);
      return 0;
    }
  }

  /**
   * Get rate limit status for domain
   */
  async getRateLimit(
    domain: string,
    _tenantId?: string
  ): Promise<DomainRateLimit | null> {
    if (!this.isAvailable()) {
      return null;
    }

    try {
      const result = await this.poolManager
        .getPool()
        .query(`SELECT * FROM web_rate_limits WHERE domain = $1`, [domain]);

      if (result.rows.length === 0) {
        return null;
      }

      const row = result.rows[0];
      return {
        domain: row.domain,
        status: row.status as RateLimitStatus,
        requestsInWindow: row.requests_in_window,
        windowResetAt: row.window_end,
        backoffUntil: row.backoff_until || undefined,
        lastRequestAt: row.last_request,
      };
    } catch (error) {
      console.error("Failed to get rate limit:", error);
      return null;
    }
  }

  /**
   * Update rate limit for domain
   */
  async updateRateLimit(
    rateLimit: DomainRateLimit,
    _tenantId?: string
  ): Promise<void> {
    if (!this.isAvailable()) {
      return;
    }

    try {
      await this.poolManager.getPool().query(
        `INSERT INTO web_rate_limits (
          domain, status, requests_in_window, window_start, window_end,
          backoff_until, last_request
        ) VALUES ($1, $2, $3, NOW(), $4, $5, $6)
        ON CONFLICT (domain) DO UPDATE SET
          status = EXCLUDED.status,
          requests_in_window = EXCLUDED.requests_in_window,
          window_end = EXCLUDED.window_end,
          backoff_until = EXCLUDED.backoff_until,
          last_request = EXCLUDED.last_request,
          total_requests = web_rate_limits.total_requests + 1`,
        [
          rateLimit.domain,
          rateLimit.status,
          rateLimit.requestsInWindow,
          rateLimit.windowResetAt,
          rateLimit.backoffUntil || null,
          rateLimit.lastRequestAt,
        ]
      );
    } catch (error) {
      console.error("Failed to update rate limit:", error);
    }
  }

  /**
   * Increment rate limit counter for domain
   */
  async incrementRateLimitCounter(
    domain: string,
    _tenantId?: string
  ): Promise<number> {
    if (!this.isAvailable()) {
      return 0;
    }

    try {
      // Initialize if not exists
      await this.poolManager.getPool().query(
        `INSERT INTO web_rate_limits (domain, requests_in_window, window_start, window_end)
         VALUES ($1, 0, NOW(), NOW() + INTERVAL '1 minute')
         ON CONFLICT (domain) DO NOTHING`,
        [domain]
      );

      // Reset window if expired
      await this.poolManager.getPool().query(
        `UPDATE web_rate_limits
         SET requests_in_window = 0,
             window_start = NOW(),
             window_end = NOW() + INTERVAL '1 minute'
         WHERE domain = $1 AND window_end < NOW()`,
        [domain]
      );

      // Increment counter
      const result = await this.poolManager.getPool().query(
        `UPDATE web_rate_limits
         SET requests_in_window = requests_in_window + 1,
             last_request = NOW(),
             total_requests = total_requests + 1
         WHERE domain = $1
         RETURNING requests_in_window`,
        [domain]
      );

      return result.rows[0]?.requests_in_window || 0;
    } catch (error) {
      console.error("Failed to increment rate limit counter:", error);
      return 0;
    }
  }

  /**
   * Create traversal session
   */
  async createTraversal(
    sessionId: string,
    startUrl: string,
    maxDepth: number,
    maxPages: number,
    strategy: TraversalStrategy,
    _tenantId?: string
  ): Promise<string | null> {
    if (!this.isAvailable()) {
      return null;
    }

    try {
      const result = await this.poolManager.getPool().query(
        `INSERT INTO web_traversals (
          session_id, start_url, max_depth, max_pages, strategy, status, started_at
        ) VALUES ($1, $2, $3, $4, $5, 'running', NOW())
        RETURNING id`,
        [sessionId, startUrl, maxDepth, maxPages, strategy]
      );

      return result.rows[0]?.id || null;
    } catch (error) {
      console.error("Failed to create traversal:", error);
      return null;
    }
  }

  /**
   * Update traversal status
   */
  async updateTraversalStatus(
    sessionId: string,
    status: "pending" | "running" | "completed" | "failed",
    errorMessage?: string,
    _tenantId?: string
  ): Promise<void> {
    if (!this.isAvailable()) {
      return;
    }

    try {
      await this.poolManager.getPool().query(
        `UPDATE web_traversals
         SET status = $1, completed_at = NOW(), error_message = $2
         WHERE session_id = $3`,
        [status, errorMessage || null, sessionId]
      );
    } catch (error) {
      console.error("Failed to update traversal status:", error);
    }
  }

  /**
   * Add node to traversal
   */
  async addTraversalNode(
    traversalId: string,
    url: string,
    depth: number,
    parentId?: string,
    linkText?: string,
    _tenantId?: string
  ): Promise<string | null> {
    if (!this.isAvailable()) {
      return null;
    }

    try {
      const result = await this.poolManager.getPool().query(
        `INSERT INTO web_traversal_nodes (
          traversal_id, url, depth, status, parent_id, link_text
        ) VALUES ($1, $2, $3, 'pending', $4, $5)
        RETURNING id`,
        [traversalId, url, depth, parentId || null, linkText || null]
      );

      return result.rows[0]?.id || null;
    } catch (error) {
      console.error("Failed to add traversal node:", error);
      return null;
    }
  }

  /**
   * Update traversal node status
   */
  async updateTraversalNode(
    nodeId: string,
    status: "pending" | "visited" | "skipped" | "error",
    contentId?: string,
    errorMessage?: string,
    _tenantId?: string
  ): Promise<void> {
    if (!this.isAvailable()) {
      return;
    }

    try {
      await this.poolManager.getPool().query(
        `UPDATE web_traversal_nodes
         SET status = $1, visited_at = NOW(), content_id = $2, error_message = $3
         WHERE id = $4`,
        [status, contentId || null, errorMessage || null, nodeId]
      );
    } catch (error) {
      console.error("Failed to update traversal node:", error);
    }
  }

  /**
   * Get cache statistics
   */
  async getCacheStats(_tenantId?: string): Promise<{
    totalPages: number;
    cacheSizeBytes: number;
    hitRate: number;
    ageDistribution: {
      under1Hour: number;
      under6Hours: number;
      under12Hours: number;
      under24Hours: number;
    };
  }> {
    if (!this.isAvailable()) {
      return {
        totalPages: 0,
        cacheSizeBytes: 0,
        hitRate: 0,
        ageDistribution: {
          under1Hour: 0,
          under6Hours: 0,
          under12Hours: 0,
          under24Hours: 0,
        },
      };
    }

    try {
      const result = await this.poolManager
        .getPool()
        .query(`SELECT * FROM web_cache_performance`);
      const row = result.rows[0] || {};

      return {
        totalPages: row.total_entries || 0,
        cacheSizeBytes: row.total_cache_size_bytes || 0,
        hitRate: row.total_hits / Math.max(row.total_entries, 1),
        ageDistribution: {
          under1Hour: row.entries_accessed_last_hour || 0,
          under6Hours: row.entries_accessed_last_6hours || 0,
          under12Hours: row.entries_accessed_last_12hours || 0,
          under24Hours: row.entries_accessed_last_24hours || 0,
        },
      };
    } catch (error) {
      console.error("Failed to get cache stats:", error);
      return {
        totalPages: 0,
        cacheSizeBytes: 0,
        hitRate: 0,
        ageDistribution: {
          under1Hour: 0,
          under6Hours: 0,
          under12Hours: 0,
          under24Hours: 0,
        },
      };
    }
  }

  /**
   * Store extraction metrics
   */
  async storeExtractionMetrics(
    contentId: string,
    metrics: {
      totalTimeMs: number;
      fetchTimeMs: number;
      parseTimeMs: number;
      sanitizeTimeMs: number;
      statusCode: number;
      contentType?: string;
      contentLength?: number;
      redirectCount: number;
      sslVerified: boolean;
      maliciousDetected: boolean;
      sanitizationApplied: boolean;
    },
    _tenantId?: string
  ): Promise<void> {
    if (!this.isAvailable()) {
      return;
    }

    try {
      await this.poolManager.getPool().query(
        `INSERT INTO web_extraction_metrics (
          content_id, total_time_ms, fetch_time_ms, parse_time_ms, sanitize_time_ms,
          status_code, content_type, content_length, redirect_count,
          ssl_verified, malicious_detected, sanitization_applied
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)`,
        [
          contentId,
          metrics.totalTimeMs,
          metrics.fetchTimeMs,
          metrics.parseTimeMs,
          metrics.sanitizeTimeMs,
          metrics.statusCode,
          metrics.contentType || null,
          metrics.contentLength || null,
          metrics.redirectCount,
          metrics.sslVerified,
          metrics.maliciousDetected,
          metrics.sanitizationApplied,
        ]
      );
    } catch (error) {
      console.error("Failed to store extraction metrics:", error);
    }
  }

  /**
   * Convert database row to WebContent
   */
  private rowToWebContent(row: WebContentRecord): WebContent {
    const metadata =
      typeof row.metadata === "string"
        ? JSON.parse(row.metadata)
        : row.metadata;

    return {
      id: row.id,
      url: row.url,
      title: row.title,
      content: row.content,
      html: row.html || undefined,
      links: [], // Would need separate query to fetch links
      images: [], // Would need separate query to fetch images
      metadata: metadata,
      quality: row.quality,
      contentHash: row.content_hash,
      extractedAt: row.extracted_at,
    };
  }
}
