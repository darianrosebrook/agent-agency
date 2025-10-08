# Agent Memory System - Technical Architecture

## Architecture Overview

The Agent Memory System is built on a microservices architecture that extends the existing Agent Agency platform with sophisticated memory and learning capabilities. The system is designed to be scalable, maintainable, and extensible.

## System Components

### 1. Memory Service Layer

#### AgentMemoryManager
```typescript
/**
 * Core memory management service for agent experiences and knowledge
 * @author @darianrosebrook
 */
export class AgentMemoryManager {
  private knowledgeGraph: KnowledgeGraphEngine;
  private embeddingService: EmbeddingService;
  private temporalReasoning: TemporalReasoningEngine;
  private contextManager: ContextManager;
  private cache: MemoryCache;

  constructor(config: MemoryConfig) {
    this.knowledgeGraph = new KnowledgeGraphEngine(config.database);
    this.embeddingService = new EmbeddingService(config.ollama);
    this.temporalReasoning = new TemporalReasoningEngine(config.database);
    this.contextManager = new ContextManager(config.database);
    this.cache = new MemoryCache(config.redis);
  }

  /**
   * Store agent experience from task execution
   */
  async storeExperience(
    agentId: string,
    taskId: string,
    outcome: TaskOutcome,
    context: ExecutionContext
  ): Promise<void> {
    // Extract entities and relationships
    const entities = await this.extractEntities(agentId, taskId, context);
    const relationships = await this.extractRelationships(entities);
    
    // Update knowledge graph
    await this.knowledgeGraph.updateGraph(entities, relationships);
    
    // Update agent capabilities
    await this.updateAgentCapabilities(agentId, outcome, context);
    
    // Store temporal event
    await this.temporalReasoning.recordEvent(agentId, taskId, outcome);
    
    // Update cache
    await this.cache.invalidateAgentCache(agentId);
  }

  /**
   * Retrieve relevant memories for current context
   */
  async retrieveMemories(
    agentId: string,
    context: MemoryContext,
    options: RetrievalOptions = {}
  ): Promise<RelevantMemory[]> {
    // Check cache first
    const cacheKey = this.generateCacheKey(agentId, context);
    const cached = await this.cache.get(cacheKey);
    if (cached) return cached;

    // Generate context embedding
    const contextEmbedding = await this.embeddingService.embed(context.query);
    
    // Retrieve similar experiences
    const similarExperiences = await this.knowledgeGraph.findSimilarExperiences(
      agentId,
      contextEmbedding,
      options.similarityThreshold || 0.7
    );

    // Retrieve relevant conversations
    const relevantConversations = await this.findRelevantConversations(
      agentId,
      contextEmbedding
    );

    // Retrieve capability-relevant memories
    const capabilityMemories = await this.findCapabilityMemories(
      agentId,
      context.requiredCapabilities
    );

    // Combine and rank memories
    const memories = this.combineAndRankMemories([
      ...similarExperiences,
      ...relevantConversations,
      ...capabilityMemories
    ]);

    // Cache results
    await this.cache.set(cacheKey, memories, options.ttl || 3600);
    
    return memories;
  }
}
```

#### KnowledgeGraphEngine
```typescript
/**
 * Knowledge graph management for agent relationships and experiences
 */
export class KnowledgeGraphEngine {
  private pool: Pool;
  private entityExtractor: EntityExtractor;
  private relationshipBuilder: RelationshipBuilder;

  constructor(pool: Pool) {
    this.pool = pool;
    this.entityExtractor = new EntityExtractor();
    this.relationshipBuilder = new RelationshipBuilder();
  }

  /**
   * Update knowledge graph with new entities and relationships
   */
  async updateGraph(
    entities: AgentEntity[],
    relationships: AgentRelationship[]
  ): Promise<void> {
    const client = await this.pool.connect();
    try {
      await client.query('BEGIN');

      // Process entities with deduplication
      const processedEntities = await this.processEntities(entities, client);
      
      // Process relationships
      await this.processRelationships(relationships, processedEntities, client);
      
      // Update entity embeddings
      await this.updateEntityEmbeddings(processedEntities, client);

      await client.query('COMMIT');
    } catch (error) {
      await client.query('ROLLBACK');
      throw error;
    } finally {
      client.release();
    }
  }

  /**
   * Find similar experiences using vector similarity
   */
  async findSimilarExperiences(
    agentId: string,
    queryEmbedding: number[],
    threshold: number
  ): Promise<AgentExperience[]> {
    const query = `
      SELECT 
        ae.*,
        (ae.embedding <=> $2::vector) as similarity
      FROM agent_entities ae
      WHERE ae.agent_id = $1
        AND ae.entity_type = 'task'
        AND ae.embedding IS NOT NULL
        AND (ae.embedding <=> $2::vector) < $3
      ORDER BY ae.embedding <=> $2::vector
      LIMIT 20
    `;

    const result = await this.pool.query(query, [
      agentId,
      `[${queryEmbedding.join(',')}]`,
      1 - threshold
    ]);

    return result.rows.map(row => this.mapToAgentExperience(row));
  }
}
```

### 2. Vector Embedding Service

```typescript
/**
 * Vector embedding service using Ollama embedding models
 */
export class EmbeddingService {
  private ollamaClient: Ollama;
  private model: string;
  private dimension: number;
  private cache: Map<string, number[]>;

  constructor(config: EmbeddingConfig) {
    this.ollamaClient = new Ollama({ host: config.host });
    this.model = config.model || 'embeddinggemma';
    this.dimension = config.dimension || 768;
    this.cache = new Map();
  }

  /**
   * Generate embedding for text content
   */
  async embed(text: string): Promise<number[]> {
    // Check cache first
    const cacheKey = this.hashText(text);
    if (this.cache.has(cacheKey)) {
      return this.cache.get(cacheKey)!;
    }

    try {
      const response = await this.ollamaClient.embeddings({
        model: this.model,
        prompt: text
      });

      const embedding = response.embedding;
      
      // Validate dimension
      if (embedding.length !== this.dimension) {
        throw new Error(`Expected embedding dimension ${this.dimension}, got ${embedding.length}`);
      }

      // Cache result
      this.cache.set(cacheKey, embedding);
      
      return embedding;
    } catch (error) {
      console.error('Embedding generation failed:', error);
      throw new Error(`Failed to generate embedding: ${error.message}`);
    }
  }

  /**
   * Generate embeddings for multiple texts in batch
   */
  async embedBatch(texts: string[]): Promise<number[][]> {
    const embeddings: number[][] = [];
    
    for (const text of texts) {
      const embedding = await this.embed(text);
      embeddings.push(embedding);
    }
    
    return embeddings;
  }

  /**
   * Calculate cosine similarity between two embeddings
   */
  calculateSimilarity(embedding1: number[], embedding2: number[]): number {
    if (embedding1.length !== embedding2.length) {
      throw new Error('Embedding dimensions must match');
    }

    let dotProduct = 0;
    let norm1 = 0;
    let norm2 = 0;

    for (let i = 0; i < embedding1.length; i++) {
      dotProduct += embedding1[i] * embedding2[i];
      norm1 += embedding1[i] * embedding1[i];
      norm2 += embedding2[i] * embedding2[i];
    }

    const similarity = dotProduct / (Math.sqrt(norm1) * Math.sqrt(norm2));
    return Math.max(0, Math.min(1, similarity)); // Clamp to [0, 1]
  }
}
```

### 3. Temporal Reasoning Engine

```typescript
/**
 * Temporal reasoning and causality detection for agent memory
 */
export class TemporalReasoningEngine {
  private pool: Pool;
  private causalityEngine: CausalityDetectionEngine;
  private trendAnalyzer: TrendAnalysisEngine;

  constructor(pool: Pool) {
    this.pool = pool;
    this.causalityEngine = new CausalityDetectionEngine(pool);
    this.trendAnalyzer = new TrendAnalysisEngine(pool);
  }

  /**
   * Record temporal event for analysis
   */
  async recordEvent(
    agentId: string,
    taskId: string,
    outcome: TaskOutcome
  ): Promise<void> {
    const event = {
      entity_type: 'agent',
      entity_id: agentId,
      event_type: 'task_completion',
      event_data: {
        task_id: taskId,
        outcome: outcome.status,
        performance_score: outcome.score,
        execution_time: outcome.executionTime,
        capabilities_used: outcome.capabilitiesUsed,
        lessons_learned: outcome.lessonsLearned
      },
      timestamp: new Date()
    };

    await this.pool.query(
      `INSERT INTO temporal_events (entity_type, entity_id, event_type, event_data, timestamp)
       VALUES ($1, $2, $3, $4, $5)`,
      [event.entity_type, event.entity_id, event.event_type, 
       JSON.stringify(event.event_data), event.timestamp]
    );
  }

  /**
   * Analyze agent performance trends
   */
  async analyzeAgentTrends(
    agentId: string,
    timeWindow: TimeWindow
  ): Promise<TrendAnalysis> {
    const query = `
      SELECT 
        event_data->>'performance_score' as score,
        event_data->>'execution_time' as execution_time,
        timestamp
      FROM temporal_events
      WHERE entity_id = $1
        AND event_type = 'task_completion'
        AND timestamp BETWEEN $2 AND $3
      ORDER BY timestamp
    `;

    const result = await this.pool.query(query, [
      agentId,
      timeWindow.start,
      timeWindow.end
    ]);

    const scores = result.rows.map(row => parseFloat(row.score)).filter(s => !isNaN(s));
    const executionTimes = result.rows.map(row => parseInt(row.execution_time)).filter(t => !isNaN(t));

    return {
      agentId,
      timeWindow,
      performanceTrend: this.calculateTrend(scores),
      executionTimeTrend: this.calculateTrend(executionTimes),
      improvementRate: this.calculateImprovementRate(scores),
      stability: this.calculateStability(scores),
      confidence: this.calculateConfidence(result.rows.length)
    };
  }

  /**
   * Detect causality between agent actions and outcomes
   */
  async detectCausality(
    agentId: string,
    timeWindow: TimeWindow
  ): Promise<CausalityResult[]> {
    return await this.causalityEngine.analyzeCausality(agentId, timeWindow);
  }

  private calculateTrend(values: number[]): TrendDirection {
    if (values.length < 2) return 'insufficient_data';
    
    const slope = this.calculateLinearRegression(values).slope;
    const threshold = 0.05; // 5% change threshold
    
    if (slope > threshold) return 'increasing';
    if (slope < -threshold) return 'decreasing';
    return 'stable';
  }

  private calculateLinearRegression(values: number[]): { slope: number; intercept: number } {
    const n = values.length;
    const x = Array.from({ length: n }, (_, i) => i);
    
    const sumX = x.reduce((a, b) => a + b, 0);
    const sumY = values.reduce((a, b) => a + b, 0);
    const sumXY = x.reduce((sum, xi, i) => sum + xi * values[i], 0);
    const sumXX = x.reduce((sum, xi) => sum + xi * xi, 0);
    
    const slope = (n * sumXY - sumX * sumY) / (n * sumXX - sumX * sumX);
    const intercept = (sumY - slope * sumX) / n;
    
    return { slope, intercept };
  }
}
```

### 4. Context Manager

```typescript
/**
 * Context management for agent memory retrieval
 */
export class ContextManager {
  private pool: Pool;
  private embeddingService: EmbeddingService;

  constructor(pool: Pool) {
    this.pool = pool;
    this.embeddingService = new EmbeddingService();
  }

  /**
   * Build context-aware memory query
   */
  async buildContextQuery(
    agentId: string,
    taskContext: TaskContext
  ): Promise<ContextQuery> {
    // Extract key concepts from task context
    const concepts = await this.extractConcepts(taskContext);
    
    // Generate embeddings for concepts
    const conceptEmbeddings = await Promise.all(
      concepts.map(concept => this.embeddingService.embed(concept))
    );

    // Find related capabilities
    const relatedCapabilities = await this.findRelatedCapabilities(
      agentId,
      conceptEmbeddings
    );

    // Find similar past tasks
    const similarTasks = await this.findSimilarTasks(
      agentId,
      conceptEmbeddings,
      taskContext.taskType
    );

    return {
      agentId,
      concepts,
      conceptEmbeddings,
      relatedCapabilities,
      similarTasks,
      timeWindow: this.calculateTimeWindow(taskContext.priority),
      similarityThreshold: this.calculateSimilarityThreshold(taskContext.complexity)
    };
  }

  /**
   * Retrieve contextually relevant memories
   */
  async retrieveContextualMemories(
    contextQuery: ContextQuery
  ): Promise<ContextualMemory[]> {
    const memories: ContextualMemory[] = [];

    // Retrieve capability-based memories
    for (const capability of contextQuery.relatedCapabilities) {
      const capabilityMemories = await this.getCapabilityMemories(
        contextQuery.agentId,
        capability,
        contextQuery.timeWindow
      );
      memories.push(...capabilityMemories);
    }

    // Retrieve similar task memories
    for (const similarTask of contextQuery.similarTasks) {
      const taskMemories = await this.getTaskMemories(
        contextQuery.agentId,
        similarTask.taskId
      );
      memories.push(...taskMemories);
    }

    // Rank memories by contextual relevance
    const rankedMemories = await this.rankMemoriesByRelevance(
      memories,
      contextQuery
    );

    return rankedMemories;
  }

  private async extractConcepts(taskContext: TaskContext): Promise<string[]> {
    // Extract key concepts from task description, requirements, and context
    const text = [
      taskContext.description,
      taskContext.requirements?.join(' '),
      taskContext.context?.join(' ')
    ].filter(Boolean).join(' ');

    // Simple concept extraction - in production, use NLP libraries
    const words = text.toLowerCase()
      .replace(/[^\w\s]/g, ' ')
      .split(/\s+/)
      .filter(word => word.length > 3);

    // Remove common stop words
    const stopWords = new Set(['the', 'and', 'for', 'are', 'but', 'not', 'you', 'all', 'can', 'had', 'her', 'was', 'one', 'our', 'out', 'day', 'get', 'has', 'him', 'his', 'how', 'man', 'new', 'now', 'old', 'see', 'two', 'way', 'who', 'boy', 'did', 'its', 'let', 'put', 'say', 'she', 'too', 'use']);
    
    return [...new Set(words.filter(word => !stopWords.has(word)))];
  }
}
```

## Data Flow Architecture

```mermaid
sequenceDiagram
    participant AO as Agent Orchestrator
    participant AM as Agent Memory Manager
    participant KG as Knowledge Graph
    participant VE as Vector Embedding
    participant TR as Temporal Reasoning
    participant DB as Database

    Note over AO,DB: Task Execution Flow
    
    AO->>AM: submitTaskWithContext(task, context)
    AM->>VE: embed(context.query)
    VE-->>AM: contextEmbedding
    
    AM->>KG: retrieveRelevantMemories(agentId, contextEmbedding)
    KG-->>AM: relevantMemories
    
    AM->>AM: rankMemoriesByRelevance()
    AM-->>AO: contextualInsights
    
    Note over AO,DB: Experience Storage Flow
    
    AO->>AM: storeExperience(agentId, taskId, outcome)
    AM->>KG: extractEntities(agentId, taskId, context)
    KG-->>AM: entities
    
    AM->>KG: extractRelationships(entities)
    KG-->>AM: relationships
    
    AM->>KG: updateGraph(entities, relationships)
    KG->>DB: INSERT/UPDATE knowledge graph
    
    AM->>TR: recordEvent(agentId, taskId, outcome)
    TR->>DB: INSERT temporal event
    
    AM->>AM: updateAgentCapabilities(agentId, outcome)
    AM->>DB: UPDATE agent capabilities
```

## Performance Optimization

### 1. Caching Strategy

```typescript
/**
 * Multi-level caching system for memory operations
 */
export class MemoryCache {
  private redis: Redis;
  private localCache: Map<string, CacheEntry>;
  private readonly TTL = {
    AGENT_MEMORY: 3600,      // 1 hour
    SIMILARITY_RESULTS: 1800, // 30 minutes
    EMBEDDINGS: 86400,       // 24 hours
    CAPABILITY_MAP: 7200     // 2 hours
  };

  constructor(redisConfig: RedisConfig) {
    this.redis = new Redis(redisConfig);
    this.localCache = new Map();
  }

  async get(key: string): Promise<any> {
    // Check local cache first
    const localEntry = this.localCache.get(key);
    if (localEntry && !this.isExpired(localEntry)) {
      return localEntry.data;
    }

    // Check Redis cache
    const redisData = await this.redis.get(key);
    if (redisData) {
      const data = JSON.parse(redisData);
      this.localCache.set(key, { data, timestamp: Date.now() });
      return data;
    }

    return null;
  }

  async set(key: string, data: any, ttl?: number): Promise<void> {
    const serialized = JSON.stringify(data);
    
    // Set in Redis
    await this.redis.setex(key, ttl || 3600, serialized);
    
    // Set in local cache
    this.localCache.set(key, { data, timestamp: Date.now() });
  }

  private isExpired(entry: CacheEntry): boolean {
    return Date.now() - entry.timestamp > 300000; // 5 minutes local TTL
  }
}
```

### 2. Batch Processing

```typescript
/**
 * Batch processing for memory operations
 */
export class MemoryBatchProcessor {
  private batchQueue: MemoryOperation[];
  private batchSize: number;
  private flushInterval: number;
  private timer: NodeJS.Timeout;

  constructor(config: BatchConfig) {
    this.batchQueue = [];
    this.batchSize = config.batchSize || 100;
    this.flushInterval = config.flushInterval || 5000; // 5 seconds
    this.startBatchProcessor();
  }

  async addOperation(operation: MemoryOperation): Promise<void> {
    this.batchQueue.push(operation);
    
    if (this.batchQueue.length >= this.batchSize) {
      await this.flushBatch();
    }
  }

  private async flushBatch(): Promise<void> {
    if (this.batchQueue.length === 0) return;

    const batch = this.batchQueue.splice(0, this.batchSize);
    
    // Group operations by type for efficient processing
    const groupedOps = this.groupOperations(batch);
    
    // Process each group
    await Promise.all([
      this.processEntityOperations(groupedOps.entities),
      this.processRelationshipOperations(groupedOps.relationships),
      this.processExperienceOperations(groupedOps.experiences)
    ]);
  }

  private startBatchProcessor(): void {
    this.timer = setInterval(() => {
      this.flushBatch();
    }, this.flushInterval);
  }
}
```

## Monitoring and Observability

### 1. Metrics Collection

```typescript
/**
 * Metrics collection for memory system performance
 */
export class MemoryMetrics {
  private metrics: Map<string, MetricValue>;
  private exporters: MetricExporter[];

  constructor() {
    this.metrics = new Map();
    this.exporters = [];
  }

  recordMemoryRetrieval(agentId: string, duration: number, resultCount: number): void {
    this.incrementCounter('memory.retrievals.total');
    this.recordHistogram('memory.retrieval.duration', duration);
    this.recordHistogram('memory.retrieval.result_count', resultCount);
    this.incrementCounter(`memory.retrievals.agent.${agentId}`);
  }

  recordKnowledgeGraphUpdate(operation: string, duration: number, entityCount: number): void {
    this.incrementCounter(`knowledge_graph.${operation}.total`);
    this.recordHistogram(`knowledge_graph.${operation}.duration`, duration);
    this.recordHistogram(`knowledge_graph.${operation}.entity_count`, entityCount);
  }

  recordEmbeddingGeneration(count: number, duration: number): void {
    this.incrementCounter('embeddings.generated', count);
    this.recordHistogram('embeddings.generation.duration', duration);
  }

  private incrementCounter(name: string, value: number = 1): void {
    const current = this.metrics.get(name) || { type: 'counter', value: 0 };
    current.value += value;
    this.metrics.set(name, current);
  }

  private recordHistogram(name: string, value: number): void {
    const current = this.metrics.get(name) || { 
      type: 'histogram', 
      values: [],
      count: 0,
      sum: 0
    };
    current.values.push(value);
    current.count++;
    current.sum += value;
    this.metrics.set(name, current);
  }
}
```

### 2. Health Checks

```typescript
/**
 * Health monitoring for memory system components
 */
export class MemoryHealthMonitor {
  private checks: HealthCheck[];
  private status: HealthStatus;

  constructor() {
    this.checks = [
      new DatabaseHealthCheck(),
      new RedisHealthCheck(),
      new EmbeddingServiceHealthCheck(),
      new KnowledgeGraphHealthCheck()
    ];
    this.status = { status: 'unknown', checks: {} };
  }

  async performHealthCheck(): Promise<HealthStatus> {
    const results: Record<string, HealthCheckResult> = {};

    for (const check of this.checks) {
      try {
        const result = await check.execute();
        results[check.name] = result;
      } catch (error) {
        results[check.name] = {
          status: 'error',
          message: error.message,
          timestamp: new Date()
        };
      }
    }

    const overallStatus = this.calculateOverallStatus(results);
    
    this.status = {
      status: overallStatus,
      checks: results,
      timestamp: new Date()
    };

    return this.status;
  }

  private calculateOverallStatus(checks: Record<string, HealthCheckResult>): HealthStatus['status'] {
    const statuses = Object.values(checks).map(c => c.status);
    
    if (statuses.includes('error')) return 'error';
    if (statuses.includes('warning')) return 'warning';
    return 'healthy';
  }
}
```

## Configuration

### Environment Variables

```bash
# Database Configuration
DATABASE_URL=postgresql://user:pass@localhost:5432/agent_agency
REDIS_URL=redis://localhost:6379

# Embedding Service
OLLAMA_HOST=http://localhost:11434
EMBEDDING_MODEL=embeddinggemma
EMBEDDING_DIMENSION=768

# Memory System Configuration
MEMORY_CACHE_TTL=3600
BATCH_SIZE=100
BATCH_FLUSH_INTERVAL=5000
SIMILARITY_THRESHOLD=0.7

# Performance Tuning
MAX_CONCURRENT_EMBEDDINGS=10
VECTOR_INDEX_LISTS=100
MEMORY_CACHE_SIZE=1000
```

### Configuration Schema

```typescript
interface MemorySystemConfig {
  database: {
    url: string;
    poolSize: number;
    connectionTimeout: number;
  };
  redis: {
    url: string;
    keyPrefix: string;
    defaultTTL: number;
  };
  embedding: {
    ollamaHost: string;
    model: string;
    dimension: number;
    batchSize: number;
    timeout: number;
  };
  knowledgeGraph: {
    similarityThreshold: number;
    maxEntitiesPerBatch: number;
    enableDeduplication: boolean;
    deduplicationThreshold: number;
  };
  temporal: {
    maxAnalysisWindow: number; // in milliseconds
    minDataPoints: number;
    confidenceThreshold: number;
  };
  performance: {
    enableCaching: boolean;
    cacheSize: number;
    batchProcessing: boolean;
    batchSize: number;
    flushInterval: number;
  };
}
```

This technical architecture provides the foundation for implementing a sophisticated agent memory system that can scale with the Agent Agency platform while maintaining high performance and reliability.

