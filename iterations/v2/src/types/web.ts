/**
 * @fileoverview Web Navigator Types for ARBITER-008
 *
 * Type definitions for the Web Navigator component providing
 * web content extraction, DOM parsing, and link traversal capabilities.
 *
 * @author @darianrosebrook
 */

// Re-export SearchEngine types for convenience
export type {
  EnrichedSearchResult,
  SearchEngineConfig,
  SearchResults,
} from "../web/SearchEngine";

/**
 * Search query parameters
 */
export interface SearchQuery {
  query: string;
  maxResults?: number;
  language?: string;
  region?: string;
  safeSearch?: boolean;
  excludeDomains?: string[];
}

/**
 * Web content extraction types
 */
export enum ExtractionType {
  FULL_PAGE = "full_page",
  MAIN_CONTENT = "main_content",
  SPECIFIC_ELEMENT = "specific_element",
  METADATA = "metadata",
}

/**
 * Web traversal strategies
 */
export enum TraversalStrategy {
  BREADTH_FIRST = "breadth_first",
  DEPTH_FIRST = "depth_first",
  RELEVANCE_BASED = "relevance_based",
}

/**
 * Content quality levels
 */
export enum ContentQuality {
  HIGH = "high",
  MEDIUM = "medium",
  LOW = "low",
  UNKNOWN = "unknown",
}

/**
 * Rate limit status
 */
export enum RateLimitStatus {
  OK = "ok",
  THROTTLED = "throttled",
  BLOCKED = "blocked",
}

/**
 * Web navigation query interface
 */
export interface WebNavigationQuery {
  /** Unique query identifier */
  id: string;

  /** Target URL to extract or starting point for traversal */
  url: string;

  /** Type of extraction to perform */
  extractionType: ExtractionType;

  /** Whether to traverse links */
  enableTraversal: boolean;

  /** Traversal configuration if enabled */
  traversalConfig?: TraversalConfig;

  /** Content extraction configuration */
  extractionConfig: ContentExtractionConfig;

  /** Query timeout in milliseconds */
  timeoutMs: number;

  /** Query metadata */
  metadata: {
    requesterId: string;
    priority: number;
    createdAt: Date;
    tags?: string[];
  };
}

/**
 * Content extraction configuration
 */
export interface ContentExtractionConfig {
  /** Include images */
  includeImages: boolean;

  /** Include links */
  includeLinks: boolean;

  /** Include metadata */
  includeMetadata: boolean;

  /** Strip navigation elements */
  stripNavigation: boolean;

  /** Strip ads and promotional content */
  stripAds: boolean;

  /** Maximum content length in characters */
  maxContentLength: number;

  /** CSS selector for specific element extraction */
  selector?: string;

  /** Security context */
  security: WebSecurityContext;
}

/**
 * Web security context
 */
export interface WebSecurityContext {
  /** Verify SSL certificates */
  verifySsl: boolean;

  /** Sanitize HTML content */
  sanitizeHtml: boolean;

  /** Detect malicious content */
  detectMalicious: boolean;

  /** Follow redirects */
  followRedirects: boolean;

  /** Maximum redirect count */
  maxRedirects: number;

  /** User agent string */
  userAgent: string;

  /** Respect robots.txt */
  respectRobotsTxt: boolean;
}

/**
 * Link traversal configuration
 */
export interface TraversalConfig {
  /** Maximum depth to traverse */
  maxDepth: number;

  /** Maximum pages to visit */
  maxPages: number;

  /** Traversal strategy */
  strategy: TraversalStrategy;

  /** Only traverse same domain */
  sameDomainOnly: boolean;

  /** Respect robots.txt */
  respectRobotsTxt: boolean;

  /** Delay between requests (ms) */
  delayMs: number;

  /** Link filter patterns (regex) */
  linkFilters?: string[];

  /** Excluded URL patterns (regex) */
  excludePatterns?: string[];
}

/**
 * Extracted web content
 */
export interface WebContent {
  /** Unique content identifier */
  id: string;

  /** Source URL */
  url: string;

  /** Page title */
  title: string;

  /** Main content text */
  content: string;

  /** HTML content (if requested) */
  html?: string;

  /** Extracted links */
  links: ExtractedLink[];

  /** Extracted images */
  images: ExtractedImage[];

  /** Page metadata */
  metadata: WebContentMetadata;

  /** Content quality assessment */
  quality: ContentQuality;

  /** Content hash for duplicate detection */
  contentHash: string;

  /** Extraction timestamp */
  extractedAt: Date;
}

/**
 * Extracted link information
 */
export interface ExtractedLink {
  /** Link URL */
  url: string;

  /** Link text/anchor */
  text: string;

  /** Link type (internal/external) */
  type: "internal" | "external";

  /** Link relevance score (0-1) */
  relevance: number;
}

/**
 * Extracted image information
 */
export interface ExtractedImage {
  /** Image URL */
  url: string;

  /** Alt text */
  alt: string;

  /** Image dimensions if available */
  dimensions?: {
    width: number;
    height: number;
  };
}

/**
 * Web content metadata
 */
export interface WebContentMetadata {
  /** HTTP status code */
  statusCode: number;

  /** Content type */
  contentType: string;

  /** Content length in bytes */
  contentLength: number;

  /** Last modified date */
  lastModified?: Date;

  /** Cache control headers */
  cacheControl?: string;

  /** Meta tags */
  metaTags: Record<string, string>;

  /** Open Graph data */
  openGraph?: Record<string, string>;

  /** Language */
  language?: string;

  /** Author */
  author?: string;

  /** Publication date */
  publishedAt?: Date;

  /** Domain */
  domain: string;

  /** Is HTTPS */
  isSecure: boolean;
}

/**
 * Link traversal result
 */
export interface TraversalResult {
  /** Unique traversal session ID */
  sessionId: string;

  /** Starting URL */
  startUrl: string;

  /** All visited pages */
  pages: WebContent[];

  /** Traversal statistics */
  statistics: TraversalStatistics;

  /** Traversal graph (URL relationships) */
  graph: TraversalGraph;

  /** Completed timestamp */
  completedAt: Date;
}

/**
 * Traversal statistics
 */
export interface TraversalStatistics {
  /** Total pages visited */
  pagesVisited: number;

  /** Total pages skipped */
  pagesSkipped: number;

  /** Total errors encountered */
  errorsEncountered: number;

  /** Maximum depth reached */
  maxDepthReached: number;

  /** Total processing time (ms) */
  processingTimeMs: number;

  /** Total content extracted (bytes) */
  totalContentBytes: number;

  /** Average page load time (ms) */
  avgPageLoadTimeMs: number;

  /** Rate limit encounters */
  rateLimitEncounters: number;
}

/**
 * Traversal graph structure
 */
export interface TraversalGraph {
  /** Graph nodes (URLs) */
  nodes: TraversalNode[];

  /** Graph edges (links between pages) */
  edges: TraversalEdge[];
}

/**
 * Traversal graph node
 */
export interface TraversalNode {
  /** URL */
  url: string;

  /** Depth level */
  depth: number;

  /** Visit status */
  status: "visited" | "pending" | "skipped" | "error";

  /** Content ID if visited */
  contentId?: string;
}

/**
 * Traversal graph edge
 */
export interface TraversalEdge {
  /** Source URL */
  from: string;

  /** Target URL */
  to: string;

  /** Link text */
  linkText: string;
}

/**
 * Web Navigator status
 */
export interface WebNavigatorStatus {
  /** Service enabled */
  enabled: boolean;

  /** Active extractions */
  activeExtractions: number;

  /** Active traversals */
  activeTraversals: number;

  /** Cache statistics */
  cacheStats: CacheStatistics;

  /** Rate limit status by domain */
  rateLimits: Map<string, DomainRateLimit>;

  /** Health status */
  health: WebNavigatorHealth;
}

/**
 * Cache statistics
 */
export interface CacheStatistics {
  /** Total cached pages */
  totalPages: number;

  /** Cache size in bytes */
  cacheSizeBytes: number;

  /** Cache hit rate */
  hitRate: number;

  /** Cache entries by age */
  ageDistribution: {
    under1Hour: number;
    under6Hours: number;
    under12Hours: number;
    under24Hours: number;
  };
}

/**
 * Domain rate limit tracking
 */
export interface DomainRateLimit {
  /** Domain name */
  domain: string;

  /** Current status */
  status: RateLimitStatus;

  /** Requests made in current window */
  requestsInWindow: number;

  /** Window reset time */
  windowResetAt: Date;

  /** Backoff until time (if throttled) */
  backoffUntil?: Date;

  /** Last request time */
  lastRequestAt: Date;
}

/**
 * Web Navigator health
 */
export interface WebNavigatorHealth {
  /** Overall health status */
  status: "healthy" | "degraded" | "unhealthy";

  /** HTTP client availability */
  httpClientAvailable: boolean;

  /** Database availability */
  databaseAvailable: boolean;

  /** Cache availability */
  cacheAvailable: boolean;

  /** Average response time (ms) */
  avgResponseTimeMs: number;

  /** Error rate (0-1) */
  errorRate: number;

  /** Last health check */
  lastCheckAt: Date;
}

/**
 * Web Navigator configuration
 */
export interface WebNavigatorConfig {
  /** Enabled state */
  enabled: boolean;

  /** HTTP client configuration */
  http: {
    /** Request timeout (ms) */
    timeoutMs: number;

    /** Maximum concurrent requests */
    maxConcurrentRequests: number;

    /** Retry attempts */
    retryAttempts: number;

    /** Retry delay (ms) */
    retryDelayMs: number;

    /** User agent */
    userAgent: string;

    /** Follow redirects */
    followRedirects: boolean;

    /** Maximum redirects */
    maxRedirects: number;
  };

  /** Cache configuration */
  cache: {
    /** Enable caching */
    enabled: boolean;

    /** Cache TTL (hours) */
    ttlHours: number;

    /** Maximum cache size (MB) */
    maxSizeMb: number;
  };

  /** Rate limiting configuration */
  rateLimit: {
    /** Enable rate limiting */
    enabled: boolean;

    /** Requests per minute per domain */
    requestsPerMinute: number;

    /** Backoff multiplier */
    backoffMultiplier: number;

    /** Maximum backoff (ms) */
    maxBackoffMs: number;
  };

  /** Security configuration */
  security: {
    /** Verify SSL */
    verifySsl: boolean;

    /** Enable content sanitization */
    sanitizeContent: boolean;

    /** Detect malicious content */
    detectMalicious: boolean;

    /** Respect robots.txt */
    respectRobotsTxt: boolean;
  };

  /** Performance limits */
  limits: {
    /** Maximum content size (MB) */
    maxContentSizeMb: number;

    /** Maximum extraction time (ms) */
    maxExtractionTimeMs: number;

    /** Maximum traversal depth */
    maxTraversalDepth: number;

    /** Maximum pages per traversal */
    maxPagesPerTraversal: number;
  };

  /** Observability */
  observability: {
    /** Enable metrics */
    enableMetrics: boolean;

    /** Enable tracing */
    enableTracing: boolean;

    /** Log level */
    logLevel: "debug" | "info" | "warn" | "error";
  };
}

/**
 * Web Navigator database records
 */
export interface WebContentRecord {
  id: string;
  url: string;
  title: string;
  content: string;
  html: string | null;
  content_hash: string;
  quality: ContentQuality;
  metadata: Record<string, any>;
  extracted_at: Date;
  cached_until: Date;
}

export interface WebTraversalRecord {
  id: string;
  session_id: string;
  start_url: string;
  max_depth: number;
  max_pages: number;
  strategy: TraversalStrategy;
  status: "pending" | "running" | "completed" | "failed";
  pages_visited: number;
  errors_encountered: number;
  started_at: Date;
  completed_at: Date | null;
}

export interface WebCacheRecord {
  url: string;
  content_id: string;
  cached_at: Date;
  expires_at: Date;
  hit_count: number;
  last_accessed: Date;
}

export interface WebRateLimitRecord {
  domain: string;
  status: RateLimitStatus;
  requests_in_window: number;
  window_start: Date;
  window_end: Date;
  backoff_until: Date | null;
  last_request: Date;
}

/**
 * Web Navigator events
 */
export enum WebNavigatorEventType {
  EXTRACTION_STARTED = "web.extraction.started",
  EXTRACTION_COMPLETED = "web.extraction.completed",
  EXTRACTION_FAILED = "web.extraction.failed",
  TRAVERSAL_STARTED = "web.traversal.started",
  TRAVERSAL_COMPLETED = "web.traversal.completed",
  TRAVERSAL_FAILED = "web.traversal.failed",
  RATE_LIMIT_HIT = "web.rate_limit.hit",
  CACHE_HIT = "web.cache.hit",
  CACHE_MISS = "web.cache.miss",
  CONTENT_SANITIZED = "web.content.sanitized",
  MALICIOUS_CONTENT_DETECTED = "web.content.malicious",
}

/**
 * Web Navigator event data
 */
export interface WebNavigatorEventData {
  queryId?: string;
  url?: string;
  domain?: string;
  statusCode?: number;
  contentLength?: number;
  processingTimeMs?: number;
  error?: string;
  cacheHit?: boolean;
}
