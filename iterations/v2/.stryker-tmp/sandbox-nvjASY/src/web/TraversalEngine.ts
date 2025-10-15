/**
 * @fileoverview Traversal Engine for ARBITER-008
 *
 * Handles link traversal and exploration with configurable strategies,
 * depth limits, and respectful crawling practices.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import crypto from "crypto";
import {
  ContentExtractionConfig,
  TraversalConfig,
  TraversalResult,
  TraversalStatistics,
  TraversalStrategy,
  WebContent,
} from "../types/web";
import { ContentExtractor } from "./ContentExtractor";

/**
 * Traversal node for internal tracking
 */
interface TraversalNode {
  url: string;
  depth: number;
  parentUrl?: string;
  linkText?: string;
  status: "pending" | "visiting" | "visited" | "skipped" | "error";
  content?: WebContent;
  error?: string;
  visitedAt?: Date;
}

/**
 * Traversal Engine
 *
 * Implements link traversal with configurable strategies and limits.
 */
export class TraversalEngine {
  private nodes: Map<string, TraversalNode>;
  private visitQueue: string[];
  private visiting: Set<string>;
  private rateLimitDelays: Map<string, number>;
  private statistics: TraversalStatistics;

  private readonly contentExtractor: ContentExtractor;

  constructor(private readonly config: TraversalConfig) {
    // Create content extractor with default config
    const extractionConfig: ContentExtractionConfig = {
      includeImages: false,
      includeLinks: true,
      includeMetadata: true,
      stripNavigation: true,
      stripAds: true,
      maxContentLength: 100000,
      security: {
        verifySsl: true,
        sanitizeHtml: true,
        detectMalicious: true,
        followRedirects: true,
        maxRedirects: 5,
        userAgent: (config as any).userAgent || "TraversalEngine/1.0",
        respectRobotsTxt: config.respectRobotsTxt,
      },
    };

    this.contentExtractor = new ContentExtractor(extractionConfig);

    this.nodes = new Map();
    this.visitQueue = [];
    this.visiting = new Set();
    this.rateLimitDelays = new Map();
    this.statistics = {
      pagesVisited: 0,
      pagesSkipped: 0,
      errorsEncountered: 0,
      maxDepthReached: 0,
      processingTimeMs: 0,
      totalContentBytes: 0,
      avgPageLoadTimeMs: 0,
      rateLimitEncounters: 0,
    };
  }

  /**
   * Execute link traversal starting from seed URL
   */
  async traverse(
    startUrl: string,
    extractionConfig?: ContentExtractionConfig
  ): Promise<TraversalResult> {
    const startTime = Date.now();
    const sessionId = crypto.randomUUID();

    // Use provided extraction config or default
    const config = extractionConfig || {
      includeImages: false,
      includeLinks: true,
      includeMetadata: true,
      stripNavigation: true,
      stripAds: true,
      maxContentLength: 100000,
      security: {
        verifySsl: true,
        sanitizeHtml: true,
        detectMalicious: true,
        followRedirects: true,
        maxRedirects: 5,
        userAgent: this.config.userAgent || "TraversalEngine/1.0",
        respectRobotsTxt: this.config.respectRobotsTxt,
      },
    };

    // Initialize root node
    this.addNode(startUrl, 0);

    // Process queue based on strategy
    while (this.visitQueue.length > 0 && this.canContinue()) {
      const url = this.getNextUrl();
      if (!url) {
        break;
      }

      await this.visitNode(url, config);

      // Apply delay for rate limiting
      await this.applyRateLimit(url);
    }

    // Calculate final statistics
    this.statistics.processingTimeMs = Date.now() - startTime;
    this.statistics.avgPageLoadTimeMs =
      this.statistics.pagesVisited > 0
        ? this.statistics.processingTimeMs / this.statistics.pagesVisited
        : 0;

    // Build result
    const pages = this.getVisitedPages();
    const result: TraversalResult = {
      sessionId,
      startUrl,
      pages,
      nodes: pages, // Alias for pages
      statistics: this.statistics,
      graph: this.buildTraversalGraph(),
      completedAt: new Date(),
      depthDistribution: this.calculateDepthDistribution(),
    };

    return result;
  }

  /**
   * Add node to traversal queue
   */
  private addNode(
    url: string,
    depth: number,
    parentUrl?: string,
    linkText?: string
  ): void {
    // Skip if already exists
    if (this.nodes.has(url)) {
      return;
    }

    // Check depth limit
    if (depth > this.config.maxDepth) {
      return;
    }

    // Check same domain restriction
    if (this.config.sameDomainOnly && parentUrl) {
      const parentDomain = new URL(parentUrl).hostname;
      const urlDomain = new URL(url).hostname;
      if (parentDomain !== urlDomain) {
        return;
      }
    }

    // Check exclude patterns
    if (this.config.excludePatterns) {
      for (const pattern of this.config.excludePatterns) {
        const regex = new RegExp(pattern);
        if (regex.test(url)) {
          return;
        }
      }
    }

    // Check link filters (if provided, must match at least one)
    if (this.config.linkFilters && this.config.linkFilters.length > 0) {
      let matches = false;
      for (const pattern of this.config.linkFilters) {
        const regex = new RegExp(pattern);
        if (regex.test(url)) {
          matches = true;
          break;
        }
      }
      if (!matches) {
        return;
      }
    }

    // Add node
    const node: TraversalNode = {
      url,
      depth,
      parentUrl,
      linkText,
      status: "pending",
    };

    this.nodes.set(url, node);
    this.visitQueue.push(url);
  }

  /**
   * Get next URL to visit based on strategy
   */
  private getNextUrl(): string | undefined {
    if (this.visitQueue.length === 0) {
      return undefined;
    }

    switch (this.config.strategy) {
      case TraversalStrategy.BREADTH_FIRST:
        // FIFO - visit nodes at same depth before going deeper
        return this.visitQueue.shift();

      case TraversalStrategy.DEPTH_FIRST:
        // LIFO - go as deep as possible before backtracking
        return this.visitQueue.pop();

      case TraversalStrategy.RELEVANCE_BASED:
        // Sort by relevance (using link text relevance as proxy)
        return this.getNextRelevantUrl();

      default:
        return this.visitQueue.shift();
    }
  }

  /**
   * Get next URL based on relevance scoring
   */
  private getNextRelevantUrl(): string | undefined {
    if (this.visitQueue.length === 0) {
      return undefined;
    }

    // Simple relevance scoring based on link text and depth
    let bestUrl: string | undefined;
    let bestScore = -1;
    let bestIndex = 0;

    for (let i = 0; i < this.visitQueue.length; i++) {
      const url = this.visitQueue[i];
      const node = this.nodes.get(url);
      if (!node) {
        continue;
      }

      // Score based on link text quality and shallow depth
      let score = 0;

      // Prefer shallower depths
      score += (this.config.maxDepth - node.depth) * 2;

      // Prefer descriptive link text
      if (node.linkText && node.linkText.length > 10) {
        score += 3;
      }

      // Prefer non-navigation links
      const navPatterns = [
        "home",
        "about",
        "contact",
        "privacy",
        "terms",
        "sitemap",
      ];
      const hasNavPattern = navPatterns.some((pattern) =>
        url.toLowerCase().includes(pattern)
      );
      if (!hasNavPattern) {
        score += 2;
      }

      if (score > bestScore) {
        bestScore = score;
        bestUrl = url;
        bestIndex = i;
      }
    }

    if (bestUrl) {
      this.visitQueue.splice(bestIndex, 1);
    }

    return bestUrl;
  }

  /**
   * Visit node and extract content
   */
  private async visitNode(
    url: string,
    extractionConfig: ContentExtractionConfig
  ): Promise<void> {
    const node = this.nodes.get(url);
    if (!node || node.status !== "pending") {
      return;
    }

    // Mark as visiting
    node.status = "visiting";
    this.visiting.add(url);

    try {
      // Extract content
      const content = await this.contentExtractor.extractContent(
        url,
        extractionConfig
      );

      // Update node
      node.status = "visited";
      node.content = content;
      node.visitedAt = new Date();

      // Update statistics
      this.statistics.pagesVisited++;
      this.statistics.totalContentBytes += Buffer.byteLength(
        content.content,
        "utf8"
      );
      this.statistics.maxDepthReached = Math.max(
        this.statistics.maxDepthReached,
        node.depth
      );

      // Add links from this page to queue
      if (node.depth < this.config.maxDepth) {
        for (const link of content.links) {
          if (link.type === "internal" || !this.config.sameDomainOnly) {
            this.addNode(link.url, node.depth + 1, url, link.text);
          }
        }
      }
    } catch (error: any) {
      // Mark as error
      node.status = "error";
      node.error = error.message;
      this.statistics.errorsEncountered++;

      // Check if rate limit error
      if (
        error.message.includes("429") ||
        error.message.includes("rate limit")
      ) {
        this.statistics.rateLimitEncounters++;
        this.handleRateLimit(url);
      }
    } finally {
      this.visiting.delete(url);
    }
  }

  /**
   * Handle rate limit for domain
   */
  private handleRateLimit(url: string): void {
    try {
      const domain = new URL(url).hostname;
      const currentDelay =
        this.rateLimitDelays.get(domain) || this.config.delayMs;

      // Exponential backoff (double the delay)
      const newDelay = Math.min(currentDelay * 2, 60000); // Max 60 seconds
      this.rateLimitDelays.set(domain, newDelay);
    } catch (error) {
      // Invalid URL, ignore
    }
  }

  /**
   * Apply rate limit delay for domain
   */
  private async applyRateLimit(url: string): Promise<void> {
    try {
      const domain = new URL(url).hostname;
      const delay = this.rateLimitDelays.get(domain) || this.config.delayMs;

      if (delay > 0) {
        await new Promise((resolve) => setTimeout(resolve, delay));
      }
    } catch (error) {
      // Invalid URL, ignore
    }
  }

  /**
   * Check if traversal can continue
   */
  private canContinue(): boolean {
    // Check page limit
    if (this.statistics.pagesVisited >= this.config.maxPages) {
      return false;
    }

    // Check if any nodes are still pending
    return this.visitQueue.length > 0;
  }

  /**
   * Get all visited pages
   */
  private getVisitedPages(): WebContent[] {
    const pages: WebContent[] = [];

    for (const node of this.nodes.values()) {
      if (node.status === "visited" && node.content) {
        pages.push(node.content);
      }
    }

    return pages;
  }

  /**
   * Build traversal graph
   */
  private buildTraversalGraph() {
    const nodes = Array.from(this.nodes.values()).map((node) => ({
      url: node.url,
      depth: node.depth,
      // Map internal status to exported status (exclude "visiting")
      status: (node.status === "visiting" ? "pending" : node.status) as
        | "pending"
        | "visited"
        | "skipped"
        | "error",
      contentId: node.content?.id,
    }));

    const edges = Array.from(this.nodes.values())
      .filter((node) => node.parentUrl)
      .map((node) => ({
        from: node.parentUrl!,
        to: node.url,
        linkText: node.linkText || "",
      }));

    return { nodes, edges };
  }

  /**
   * Check if URL is valid for traversal
   */
  isValidUrl(url: string): boolean {
    try {
      const urlObj = new URL(url);
      return urlObj.protocol === "http:" || urlObj.protocol === "https:";
    } catch {
      return false;
    }
  }

  /**
   * Check if domain is allowed for traversal
   */
  isDomainAllowed(url: string): boolean {
    if (!this.config.sameDomainOnly) {
      return true;
    }

    try {
      const urlObj = new URL(url);
      const startUrlObj = new URL(
        this.nodes.get(this.visitQueue[0])?.url || ""
      );
      return urlObj.hostname === startUrlObj.hostname;
    } catch {
      return false;
    }
  }

  /**
   * Normalize URL for consistent storage and comparison
   */
  normalizeUrl(url: string): string {
    try {
      const urlObj = new URL(url);
      // Remove fragment and normalize
      urlObj.hash = "";
      return urlObj.toString();
    } catch {
      return url;
    }
  }

  /**
   * Calculate distribution of pages by depth
   */
  private calculateDepthDistribution(): Record<number, number> {
    const distribution: Record<number, number> = {};

    for (const node of this.nodes.values()) {
      const depth = node.depth;
      distribution[depth] = (distribution[depth] || 0) + 1;
    }

    return distribution;
  }
}
