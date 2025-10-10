/**
 * Knowledge seeking and information processing types
 * @author @darianrosebrook
 */

export interface KnowledgeQuery {
  id: string;
  query: string;
  context?: string;
  priority: QueryPriority;
  timeoutMs?: number;
  maxResults?: number;
  sources?: SearchProvider[];
  filters?: QueryFilters;
  metadata?: Record<string, any>;
}

export enum QueryPriority {
  LOW = "low",
  MEDIUM = "medium",
  HIGH = "high",
  CRITICAL = "critical",
}

export interface QueryFilters {
  dateRange?: DateRange;
  language?: string;
  contentType?: ContentType[];
  credibilityMinimum?: number;
  excludeDomains?: string[];
  includeDomains?: string[];
}

export interface DateRange {
  start: Date;
  end: Date;
}

export enum ContentType {
  ARTICLE = "article",
  BLOG_POST = "blog_post",
  NEWS = "news",
  ACADEMIC_PAPER = "academic_paper",
  DOCUMENTATION = "documentation",
  BOOK = "book",
  VIDEO = "video",
  PODCAST = "podcast",
}

export enum SearchProvider {
  GOOGLE = "google",
  BING = "bing",
  DUCKDUCKGO = "duckduckgo",
  WIKIPEDIA = "wikipedia",
  SCHOLAR = "scholar",
  GITHUB = "github",
  STACK_OVERFLOW = "stackoverflow",
}

export interface SearchResult {
  id: string;
  title: string;
  url: string;
  snippet: string;
  content?: string;
  provider: SearchProvider;
  relevanceScore: number;
  credibilityScore: number;
  publishedDate?: Date;
  author?: string;
  contentType: ContentType;
  metadata: ResultMetadata;
}

export interface ResultMetadata {
  wordCount?: number;
  language?: string;
  domain: string;
  lastModified?: Date;
  cacheTimestamp?: Date;
  providerSpecificData?: Record<string, any>;
}

export interface KnowledgeResponse {
  queryId: string;
  results: SearchResult[];
  summary?: KnowledgeSummary;
  confidence: number;
  processingTimeMs: number;
  sourcesUsed: SearchProvider[];
  cacheHit: boolean;
  error?: string;
}

export interface KnowledgeSummary {
  keyPoints: string[];
  confidence: number;
  contradictoryInfo: boolean;
  sources: SourceAttribution[];
  generatedAt: Date;
}

export interface SourceAttribution {
  url: string;
  title: string;
  relevance: number;
  credibility: number;
}

export interface SearchProviderConfig {
  name: SearchProvider;
  apiKey?: string;
  baseUrl: string;
  rateLimit: RateLimit;
  enabled: boolean;
  priority: number;
}

export interface RateLimit {
  requestsPerMinute: number;
  requestsPerHour: number;
  burstLimit: number;
}

export interface CacheEntry {
  key: string;
  data: KnowledgeResponse;
  timestamp: Date;
  ttlMs: number;
  accessCount: number;
  lastAccessed: Date;
}

export interface KnowledgeSeekerConfig {
  defaultTimeoutMs: number;
  maxConcurrentSearches: number;
  cacheEnabled: boolean;
  cacheTtlMs: number;
  minRelevanceThreshold: number;
  maxResultsPerProvider: number;
  providers: SearchProviderConfig[];
  circuitBreakerEnabled: boolean;
  retryAttempts: number;
  retryDelayMs: number;
}

// Information processing interfaces
export interface InformationProcessor {
  processResults(
    query: KnowledgeQuery,
    rawResults: SearchResult[]
  ): Promise<SearchResult[]>;
  calculateRelevance(query: string, result: SearchResult): number;
  deduplicateResults(results: SearchResult[]): SearchResult[];
  rankResults(results: SearchResult[]): SearchResult[];
}

export interface RelevanceScorer {
  score(query: string, result: SearchResult): number;
  getFactors(query: string, result: SearchResult): RelevanceFactor[];
}

export interface RelevanceFactor {
  name: string;
  weight: number;
  score: number;
  explanation: string;
}

// Search provider abstraction
export interface SearchProviderInterface {
  readonly name: SearchProvider;
  search(query: KnowledgeQuery): Promise<SearchResult[]>;
  isAvailable(): Promise<boolean>;
  getRateLimitStatus(): Promise<RateLimitStatus>;
}

export interface RateLimitStatus {
  remainingRequests: number;
  resetTime: Date;
  isLimited: boolean;
}

// Error handling
export class KnowledgeSeekerError extends Error {
  constructor(
    message: string,
    public code: KnowledgeSeekerErrorCode,
    public queryId?: string,
    public provider?: SearchProvider
  ) {
    super(message);
    this.name = "KnowledgeSeekerError";
  }
}

export enum KnowledgeSeekerErrorCode {
  PROVIDER_UNAVAILABLE = "PROVIDER_UNAVAILABLE",
  RATE_LIMIT_EXCEEDED = "RATE_LIMIT_EXCEEDED",
  INVALID_QUERY = "INVALID_QUERY",
  TIMEOUT = "TIMEOUT",
  PARSING_ERROR = "PARSING_ERROR",
  NETWORK_ERROR = "NETWORK_ERROR",
  CONFIGURATION_ERROR = "CONFIGURATION_ERROR",
}
