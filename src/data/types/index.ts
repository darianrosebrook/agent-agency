/**
 * @fileoverview Data Layer Types and Interfaces
 * @author @darianrosebrook
 *
 * Core type definitions for the unified data access layer.
 * Provides type safety for database operations, caching, and data validation.
 */

export interface DatabaseConfig {
  host: string;
  port: number;
  database: string;
  username: string;
  password: string;
  ssl?: boolean | object;
  maxConnections?: number;
  idleTimeoutMillis?: number;
  connectionTimeoutMillis?: number;
  query_timeout?: number;
  statement_timeout?: number;
}

export interface CacheConfig {
  host: string;
  port: number;
  password?: string;
  db?: number;
  keyPrefix?: string;
  ttl?: number;
  maxRetries?: number;
  lazyConnect?: boolean;
}

export interface DataLayerConfig {
  database: DatabaseConfig;
  cache: CacheConfig;
  migrationPath?: string;
  enableCache?: boolean;
  enableMetrics?: boolean;
  queryTimeout?: number;
}

export interface QueryOptions {
  timeout?: number;
  cache?: boolean;
  cacheTtl?: number;
  transaction?: boolean;
}

export interface QueryResult<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  duration: number;
  queryId: string;
}

export interface CacheResult<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  hit: boolean;
  duration: number;
}

export interface MigrationResult {
  success: boolean;
  migrations?: string[];
  error?: string;
  duration: number;
}

export interface HealthCheckResult {
  status: "healthy" | "unhealthy" | "degraded";
  database?: {
    connected: boolean;
    latency?: number;
    error?: string;
  };
  cache?: {
    connected: boolean;
    latency?: number;
    error?: string;
  };
  details?: Record<string, any>;
}

export interface DataOperationMetrics {
  operation: string;
  entity: string;
  duration: number;
  success: boolean;
  cacheHit?: boolean;
  queryCount?: number;
  errorType?: string;
}

export interface VectorSearchOptions {
  limit?: number;
  threshold?: number;
  includeMetadata?: boolean;
  filter?: Record<string, any>;
  vectorWeight?: number;
  metadataWeight?: number;
}

export interface VectorSearchResult {
  id: string;
  score: number;
  metadata?: Record<string, any>;
  vector?: number[];
}

export interface BulkOperationOptions {
  batchSize?: number;
  concurrency?: number;
  continueOnError?: boolean;
  transaction?: boolean;
}

export interface BulkOperationResult<T = any> {
  success: boolean;
  processed: number;
  successful: number;
  failed: number;
  errors?: Array<{ index: number; error: string; data: T }>;
  duration: number;
}

// Entity-specific types for the core data model
export interface AgentEntity {
  id: string;
  tenantId: string;
  name: string;
  type: string;
  capabilities: string[];
  status: "active" | "inactive" | "maintenance";
  config: Record<string, any>;
  metadata: Record<string, any>;
  createdAt: Date;
  updatedAt: Date;
}

export interface TaskEntity {
  id: string;
  tenantId: string;
  agentId: string;
  type: string;
  status: "pending" | "running" | "completed" | "failed" | "cancelled";
  description: string;
  priority: "low" | "normal" | "high";
  payload: Record<string, any>;
  result?: Record<string, any>;
  error?: string;
  requirements?: string[];
  maxRetries?: number;
  retryCount?: number;
  timeout?: number;
  metadata: Record<string, any>;
  createdAt: Date;
  updatedAt: Date;
  startedAt?: Date;
  completedAt?: Date;
}

export interface ExperienceEntity {
  id: string;
  tenantId: string;
  agentId: string;
  taskId: string;
  type: string;
  content: Record<string, any>;
  outcome: "success" | "failure" | "partial";
  relevanceScore: number;
  contextMatch: {
    similarityScore: number;
    keywordMatches: string[];
    semanticMatches: string[];
    temporalAlignment: number;
  };
  reasoningPath?: {
    steps: Array<{
      action: string;
      reasoning: string;
      confidence: number;
      timestamp: Date;
    }>;
    confidence: number;
  };
  temporalRelevance: {
    lastAccessed: Date;
    decayRate: number;
    accessCount: number;
  };
  weight: number;
  embedding?: number[];
  metadata: Record<string, any>;
  createdAt: Date;
  updatedAt: Date;
}

export interface EntityEntity {
  id: string;
  tenantId: string;
  type: string;
  name: string;
  properties: Record<string, any>;
  relationships: Array<{
    targetId: string;
    type: string;
    properties: Record<string, any>;
    weight: number;
  }>;
  embedding?: number[];
  metadata: Record<string, any>;
  createdAt: Date;
  updatedAt: Date;
}

// DAO interfaces
export interface BaseDAO<T> {
  create(
    entity: Omit<T, "id" | "createdAt" | "updatedAt">,
    options?: QueryOptions
  ): Promise<QueryResult<T>>;
  findById(
    id: string,
    tenantId: string,
    options?: QueryOptions
  ): Promise<QueryResult<T>>;
  findMany(
    filter: Partial<T>,
    options?: QueryOptions
  ): Promise<QueryResult<T[]>>;
  update(
    id: string,
    tenantId: string,
    updates: Partial<T>,
    options?: QueryOptions
  ): Promise<QueryResult<T>>;
  delete(
    id: string,
    tenantId: string,
    options?: QueryOptions
  ): Promise<QueryResult<boolean>>;
  exists(
    id: string,
    tenantId: string,
    options?: QueryOptions
  ): Promise<QueryResult<boolean>>;
  count(
    filter: Partial<T>,
    options?: QueryOptions
  ): Promise<QueryResult<number>>;
}

export interface VectorDAO<T> extends BaseDAO<T> {
  findSimilar(
    vector: number[],
    options?: VectorSearchOptions & QueryOptions
  ): Promise<QueryResult<VectorSearchResult[]>>;
  findSimilarById(
    id: string,
    tenantId: string,
    options?: VectorSearchOptions & QueryOptions
  ): Promise<QueryResult<VectorSearchResult[]>>;
  bulkInsert(
    entities: Array<Omit<T, "id" | "createdAt" | "updatedAt">>,
    options?: BulkOperationOptions
  ): Promise<BulkOperationResult<T>>;
  bulkUpdate(
    updates: Array<{ id: string; tenantId: string; data: Partial<T> }>,
    options?: BulkOperationOptions
  ): Promise<BulkOperationResult<T>>;
}

// Cache interfaces
export interface CacheProvider {
  get<T>(key: string): Promise<CacheResult<T>>;
  set<T>(key: string, value: T, ttl?: number): Promise<CacheResult<boolean>>;
  delete(key: string): Promise<CacheResult<boolean>>;
  exists(key: string): Promise<CacheResult<boolean>>;
  clear(pattern?: string): Promise<CacheResult<number>>;
  getStats(): Promise<CacheResult<Record<string, any>>>;
  close(): Promise<void>;
}

// Connection pool interfaces
export interface ConnectionPool {
  connect(): Promise<any>;
  query<T = any>(
    text: string,
    params?: any[],
    options?: QueryOptions
  ): Promise<QueryResult<T>>;
  transaction<T>(
    callback: (client: any) => Promise<T>
  ): Promise<QueryResult<T>>;
  healthCheck(): Promise<HealthCheckResult>;
  getStats(): Promise<Record<string, any>>;
  close(): Promise<void>;
}

// Migration interfaces
export interface Migration {
  id: string;
  name: string;
  up: (pool: ConnectionPool) => Promise<void>;
  down: (pool: ConnectionPool) => Promise<void>;
  depends?: string[];
}

export interface MigrationManager {
  migrate(): Promise<MigrationResult>;
  rollback(steps?: number): Promise<MigrationResult>;
  status(): Promise<MigrationResult>;
  create(name: string): Promise<string>;
}

// Error types
export class DataLayerError extends Error {
  constructor(
    message: string,
    public code: string,
    public operation: string,
    public entity?: string,
    public originalError?: Error
  ) {
    super(message);
    this.name = "DataLayerError";
  }
}

export class ConnectionError extends DataLayerError {
  constructor(message: string, operation: string, originalError?: Error) {
    super(message, "CONNECTION_ERROR", operation, undefined, originalError);
    this.name = "ConnectionError";
  }
}

export class ValidationError extends DataLayerError {
  constructor(
    message: string,
    operation: string,
    entity: string,
    originalError?: Error
  ) {
    super(message, "VALIDATION_ERROR", operation, entity, originalError);
    this.name = "ValidationError";
  }
}

export class NotFoundError extends DataLayerError {
  constructor(message: string, operation: string, entity: string) {
    super(message, "NOT_FOUND", operation, entity);
    this.name = "NotFoundError";
  }
}

export class ConcurrencyError extends DataLayerError {
  constructor(message: string, operation: string, entity: string) {
    super(message, "CONCURRENCY_ERROR", operation, entity);
    this.name = "ConcurrencyError";
  }
}
