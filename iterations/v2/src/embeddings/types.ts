/**
 * @fileoverview Embedding Infrastructure Types
 *
 * Type definitions for embedding service, vector storage, and semantic search.
 * Integrates with existing Ollama infrastructure and database schema.
 *
 * @author @darianrosebrook
 */

/**
 * Configuration for EmbeddingService
 */
export interface EmbeddingConfig {
  ollamaEndpoint: string;
  cacheSize?: number;
  model?: string;
  timeout?: number;
  rateLimitPerSecond?: number;
}

/**
 * Embedding generation request
 */
export interface EmbeddingRequest {
  text: string;
  model?: string;
}

/**
 * Embedding generation response
 */
export interface EmbeddingResponse {
  embedding: number[];
  model: string;
  usage?: {
    prompt_tokens: number;
    total_tokens: number;
  };
}

/**
 * Batch embedding request
 */
export interface BatchEmbeddingRequest {
  texts: string[];
  model?: string;
}

/**
 * Batch embedding response
 */
export interface BatchEmbeddingResponse {
  embeddings: number[][];
  model: string;
  usage?: {
    prompt_tokens: number;
    total_tokens: number;
  };
}

/**
 * Similarity search result
 */
export interface SimilarityResult {
  id: string;
  entity_type: string;
  name: string;
  similarity_score: number;
  confidence: number;
  metadata: Record<string, any>;
  source: string;
  hop_distance?: number;
}

/**
 * Vector store configuration
 */
export interface VectorStoreConfig {
  tableName: string;
  embeddingDimension: number;
  indexType: "hnsw" | "ivfflat";
  similarityMetric: "cosine" | "euclidean";
}

/**
 * Vector store search options
 */
export interface VectorSearchOptions {
  threshold: number;
  limit: number;
  tenantId?: string;
  filters?: Record<string, any>;
  entityTypes?: string[];
}

/**
 * Workspace file metadata for embedding
 */
export interface WorkspaceFileMetadata {
  filePath: string;
  content: string;
  fileType: string;
  size: number;
  lastModified: Date;
  hash: string;
}

/**
 * Wikidata lexeme structure
 */
export interface WikidataLexeme {
  id: string;
  lemma: string;
  language: string;
  lexicalCategory: string;
  forms: string[];
  senses?: Array<{
    glosses: Record<string, string>;
    examples: string[];
  }>;
}

/**
 * WordNet synset structure
 */
export interface WordNetSynset {
  id: string;
  lemmas: string[];
  definition: string;
  examples: string[];
  partOfSpeech: string;
  gloss: string;
  relations?: Array<{
    type: string;
    target: string;
  }>;
}

/**
 * Knowledge update request
 */
export interface KnowledgeUpdateRequest {
  entityId: string;
  content: string;
  source: "wikidata" | "wordnet" | "agent_memory";
  confidence?: number;
  metadata?: Record<string, any>;
}

/**
 * Confidence reinforcement request
 */
export interface ConfidenceReinforcementRequest {
  entityId: string;
  successful: boolean;
  context?: string;
}

/**
 * Embedding generation error
 */
export class EmbeddingError extends Error {
  constructor(
    message: string,
    public code: string,
    public originalError?: Error
  ) {
    super(message);
    this.name = "EmbeddingError";
  }
}

/**
 * Vector store error
 */
export class VectorStoreError extends Error {
  constructor(
    message: string,
    public code: string,
    public originalError?: Error
  ) {
    super(message);
    this.name = "VectorStoreError";
  }
}

/**
 * Embedding service interface
 */
export interface IEmbeddingService {
  generateEmbedding(text: string): Promise<number[]>;
  generateBatch(texts: string[]): Promise<number[][]>;
  isAvailable(): Promise<boolean>;
}

/**
 * Vector store interface
 */
export interface IVectorStore {
  storeEmbedding(entry: {
    id: string;
    embedding: number[];
    metadata: Record<string, any>;
    tenantId?: string;
  }): Promise<void>;

  findSimilar(
    queryEmbedding: number[],
    options: VectorSearchOptions
  ): Promise<SimilarityResult[]>;

  updateEmbedding(id: string, embedding: number[]): Promise<void>;
  deleteEmbedding(id: string): Promise<void>;
  bulkInsert(
    entries: Array<{
      id: string;
      embedding: number[];
      metadata: Record<string, any>;
      tenantId?: string;
    }>
  ): Promise<void>;
}

/**
 * Knowledge indexer interface
 */
export interface IKnowledgeIndexer {
  index(source: string): Promise<void>;
  isIndexed(source: string): Promise<boolean>;
  getIndexStats(source: string): Promise<{
    totalEntries: number;
    lastIndexed: Date;
    size: number;
  }>;
}

/**
 * Confidence manager interface
 */
export interface IConfidenceManager {
  updateKnowledge(request: KnowledgeUpdateRequest): Promise<void>;
  reinforceKnowledge(request: ConfidenceReinforcementRequest): Promise<void>;
  applyDecay(): Promise<void>;
  getConfidence(entityId: string): Promise<number>;
}
