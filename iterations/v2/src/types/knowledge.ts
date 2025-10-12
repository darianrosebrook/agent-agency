/**
 * @fileoverview Knowledge Types for ARBITER-006
 *
 * Type definitions for the Knowledge Seeker component providing
 * intelligent information gathering and research capabilities.
 *
 * @author @darianrosebrook
 */

/**
 * Knowledge query types
 */
export enum QueryType {
  FACTUAL = "factual",
  EXPLANATORY = "explanatory",
  COMPARATIVE = "comparative",
  TREND = "trend",
  TECHNICAL = "technical",
}

/**
 * Information source types
 */
export type SourceType =
  | "web"
  | "academic"
  | "news"
  | "documentation"
  | "social"
  | "internal";

/**
 * Search result quality levels
 */
export enum ResultQuality {
  HIGH = "high",
  MEDIUM = "medium",
  LOW = "low",
  UNRELIABLE = "unreliable",
}

/**
 * Search provider types
 */
export enum SearchProviderType {
  WEB_SEARCH = "web_search",
  ACADEMIC_SEARCH = "academic_search",
  NEWS_SEARCH = "news_search",
  CODE_SEARCH = "code_search",
  DOCUMENTATION_SEARCH = "documentation_search",
}

/**
 * Knowledge query interface
 */
export interface KnowledgeQuery {
  /** Unique query identifier */
  id: string;

  /** Human-readable query text */
  query: string;

  /** Type of query */
  queryType: QueryType;

  /** Preferred source types (optional filter) */
  preferredSources?: SourceType[];

  /** Maximum results to return */
  maxResults: number;

  /** Minimum relevance score threshold (0-1) */
  relevanceThreshold: number;

  /** Query timeout in milliseconds */
  timeoutMs: number;

  /** Additional context for the query */
  context?: Record<string, any>;

  /** Query metadata */
  metadata: {
    requesterId: string;
    priority: number;
    createdAt: Date;
    tags?: string[];
  };
}

/**
 * Search result interface
 */
export interface SearchResult {
  /** Unique result identifier */
  id: string;

  /** Original query ID */
  queryId: string;

  /** Title of the result */
  title: string;

  /** Content/snippet of the result */
  content: string;

  /** Full URL of the result */
  url: string;

  /** Source domain */
  domain: string;

  /** Type of source */
  sourceType: SourceType;

  /** Relevance score (0-1) */
  relevanceScore: number;

  /** Credibility score (0-1) */
  credibilityScore: number;

  /** Quality assessment */
  quality: ResultQuality;

  /** Publication date if available */
  publishedAt?: Date;

  /** Search provider that found this result */
  provider: string;

  /** Raw metadata from the provider */
  providerMetadata: Record<string, any>;

  /** Processing timestamp */
  processedAt: Date;

  /** Retrieved at timestamp (when result was fetched) */
  retrievedAt: Date;

  /** Content hash for duplicate detection */
  contentHash: string;
}

/**
 * Aggregated knowledge response
 */
export interface KnowledgeResponse {
  /** Original query */
  query: KnowledgeQuery;

  /** Search results */
  results: SearchResult[];

  /** Summary of findings */
  summary: string;

  /** Confidence in the response (0-1) */
  confidence: number;

  /** Sources used */
  sourcesUsed: string[];

  /** Processing metadata */
  metadata: {
    totalResultsFound: number;
    resultsFiltered: number;
    processingTimeMs: number;
    cacheUsed: boolean;
    providersQueried: string[];
  };

  /** Response timestamp */
  respondedAt: Date;
}

/**
 * Search provider configuration
 */
export interface SearchProviderConfig {
  /** Provider name */
  name: string;

  /** Provider type */
  type: SearchProviderType;

  /** API endpoint */
  endpoint: string;

  /** API key or authentication */
  apiKey?: string;

  /** Rate limit configuration */
  rateLimit: {
    requestsPerMinute: number;
    requestsPerHour: number;
  };

  /** Result limits */
  limits: {
    maxResultsPerQuery: number;
    maxConcurrentQueries: number;
  };

  /** Provider-specific options */
  options: Record<string, any>;
}

/**
 * Search provider interface
 */
export interface ISearchProvider {
  /** Provider name */
  readonly name: string;

  /** Provider type */
  readonly type: SearchProviderType;

  /** Check if provider is available */
  isAvailable(): Promise<boolean>;

  /** Execute search query */
  search(query: KnowledgeQuery): Promise<SearchResult[]>;

  /** Get provider health status */
  getHealthStatus(): Promise<ProviderHealthStatus>;
}

/**
 * Provider health status
 */
export interface ProviderHealthStatus {
  available: boolean;
  responseTimeMs: number;
  errorRate: number;
  lastError?: string;
  requestsThisMinute: number;
  requestsThisHour: number;
}

/**
 * Information processing configuration
 */
export interface InformationProcessorConfig {
  /** Minimum relevance score */
  minRelevanceScore: number;

  /** Minimum credibility score */
  minCredibilityScore: number;

  /** Maximum results to process */
  maxResultsToProcess: number;

  /** Diversity requirements */
  diversity: {
    minSources: number;
    minSourceTypes: number;
    maxResultsPerDomain: number;
  };

  /** Quality filtering */
  quality: {
    enableCredibilityScoring: boolean;
    enableRelevanceFiltering: boolean;
    enableDuplicateDetection: boolean;
  };

  /** Caching configuration */
  caching: {
    enableResultCaching: boolean;
    cacheTtlMs: number;
    maxCacheSize: number;
  };
}

/**
 * Information processor interface
 */
export interface IInformationProcessor {
  /** Process search results */
  processResults(
    query: KnowledgeQuery,
    results: SearchResult[]
  ): Promise<SearchResult[]>;

  /** Score result relevance */
  scoreRelevance(query: KnowledgeQuery, result: SearchResult): number;

  /** Assess result credibility */
  assessCredibility(result: SearchResult): number;

  /** Detect duplicate results */
  detectDuplicates(results: SearchResult[]): SearchResult[];

  /** Generate response summary */
  generateSummary(query: KnowledgeQuery, results: SearchResult[]): string;
}

/**
 * Knowledge seeker configuration
 */
export interface KnowledgeSeekerConfig {
  /** Enabled state */
  enabled: boolean;

  /** Search providers */
  providers: SearchProviderConfig[];

  /** Information processor config */
  processor: InformationProcessorConfig;

  /** Query processing */
  queryProcessing: {
    maxConcurrentQueries: number;
    defaultTimeoutMs: number;
    retryAttempts: number;
  };

  /** Caching */
  caching: {
    enableQueryCaching: boolean;
    enableResultCaching: boolean;
    cacheTtlMs: number;
  };

  /** Observability */
  observability: {
    enableMetrics: boolean;
    enableTracing: boolean;
    logLevel: "debug" | "info" | "warn" | "error";
  };
}

/**
 * Knowledge seeker interface
 */
export interface IKnowledgeSeeker {
  /** Process knowledge query */
  processQuery(query: KnowledgeQuery): Promise<KnowledgeResponse>;

  /** Get seeker status */
  getStatus(): Promise<KnowledgeSeekerStatus>;

  /** Clear caches */
  clearCaches(): Promise<void>;
}

/**
 * Knowledge seeker status
 */
export interface KnowledgeSeekerStatus {
  enabled: boolean;
  providers: {
    name: string;
    available: boolean;
    health: ProviderHealthStatus;
  }[];
  cacheStats: {
    queryCacheSize: number;
    resultCacheSize: number;
    hitRate: number;
  };
  processingStats: {
    activeQueries: number;
    queuedQueries: number;
    completedQueries: number;
    failedQueries: number;
  };
}

/**
 * Knowledge database entities
 */
export interface KnowledgeQueryRecord {
  id: string;
  query_text: string;
  query_type: QueryType;
  requester_id: string;
  priority: number;
  max_results: number;
  relevance_threshold: number;
  timeout_ms: number;
  context: Record<string, any>;
  tags: string[];
  created_at: Date;
  processed_at?: Date;
  status: "pending" | "processing" | "completed" | "failed";
}

export interface SearchResultRecord {
  id: string;
  query_id: string;
  title: string;
  content: string;
  url: string;
  domain: string;
  source_type: SourceType;
  relevance_score: number;
  credibility_score: number;
  quality: ResultQuality;
  published_at?: Date;
  provider: string;
  provider_metadata: Record<string, any>;
  created_at: Date;
}

export interface KnowledgeResponseRecord {
  id: string;
  query_id: string;
  summary: string;
  confidence: number;
  sources_used: string[];
  total_results_found: number;
  results_filtered: number;
  processing_time_ms: number;
  cache_used: boolean;
  providers_queried: string[];
  created_at: Date;
}

/**
 * Knowledge event types for observability
 */
export enum KnowledgeEventType {
  QUERY_RECEIVED = "knowledge.query.received",
  QUERY_PROCESSING_STARTED = "knowledge.query.processing_started",
  SEARCH_EXECUTED = "knowledge.search.executed",
  RESULTS_PROCESSED = "knowledge.results.processed",
  RESPONSE_GENERATED = "knowledge.response.generated",
  CACHE_HIT = "knowledge.cache.hit",
  CACHE_MISS = "knowledge.cache.miss",
  PROVIDER_ERROR = "knowledge.provider.error",
  PROCESSING_ERROR = "knowledge.processing.error",
  QUERY_TIMEOUT = "knowledge.query.timeout",
}

/**
 * Knowledge event data
 */
export interface KnowledgeEventData {
  queryId: string;
  providerName?: string;
  resultCount?: number;
  processingTimeMs?: number;
  error?: string;
  cacheHit?: boolean;
}
