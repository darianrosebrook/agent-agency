/**
 * Agent Memory API Client
 * API client for agent memory management, context preservation, and knowledge graph operations
 *
 * @author @darianrosebrook
 */

import { ApiClient } from './api-client';

export interface MemoryEntry {
  id: string;
  agentId: string;
  type: 'conversation' | 'fact' | 'experience' | 'knowledge' | 'context' | 'decision' | 'error' | 'other';
  content: string;
  metadata: {
    timestamp: Date;
    importance: number; // 0-1
    confidence: number; // 0-1
    source: string;
    tags: string[];
    category?: string;
    entities?: string[];
    sentiment?: number; // -1 to 1
    embedding?: number[]; // vector embedding
  };
  relationships: MemoryRelationship[];
  accessCount: number;
  lastAccessed: Date;
  createdAt: Date;
  updatedAt: Date;
  ttl?: number; // time to live in seconds
  compressed: boolean;
  size: number; // bytes
}

export interface MemoryRelationship {
  id: string;
  type: 'similar' | 'related' | 'contradicts' | 'supports' | 'causes' | 'follows' | 'parent' | 'child';
  targetMemoryId: string;
  strength: number; // 0-1
  bidirectional: boolean;
  metadata: Record<string, any>;
}

export interface AgentMemory {
  agentId: string;
  name: string;
  type: 'council_judge' | 'task_executor' | 'chat_agent' | 'analysis_agent' | 'other';
  memoryStats: {
    totalEntries: number;
    activeEntries: number;
    compressedEntries: number;
    totalSize: number; // bytes
    averageImportance: number;
    averageConfidence: number;
    oldestEntry: Date;
    newestEntry: Date;
  };
  health: {
    status: 'healthy' | 'warning' | 'critical';
    fragmentation: number; // 0-1
    memoryPressure: number; // 0-1
    accessEfficiency: number; // 0-1
    consistencyScore: number; // 0-1
  };
  configuration: {
    maxMemorySize: number;
    compressionEnabled: boolean;
    autoCleanup: boolean;
    retentionPolicy: {
      type: 'time_based' | 'importance_based' | 'size_based';
      maxAge?: number; // seconds
      minImportance?: number;
      maxSize?: number;
    };
  };
  lastUpdated: Date;
}

export interface MemoryQuery {
  agentId?: string;
  type?: MemoryEntry['type'][];
  content?: string; // search query
  tags?: string[];
  category?: string;
  entities?: string[];
  importance?: {
    min?: number;
    max?: number;
  };
  confidence?: {
    min?: number;
    max?: number;
  };
  timestamp?: {
    start?: Date;
    end?: Date;
  };
  limit?: number;
  offset?: number;
  sortBy?: 'timestamp' | 'importance' | 'confidence' | 'access_count' | 'size';
  sortOrder?: 'asc' | 'desc';
}

export interface MemorySearchResult {
  entries: MemoryEntry[];
  total: number;
  query: MemoryQuery;
  searchTime: number; // milliseconds
  relevanceScores?: Record<string, number>;
}

export interface KnowledgeGraph {
  nodes: KnowledgeNode[];
  edges: KnowledgeEdge[];
  metadata: {
    totalNodes: number;
    totalEdges: number;
    averageConnectivity: number;
    clusteringCoefficient: number;
    generatedAt: Date;
    agentId?: string;
  };
}

export interface KnowledgeNode {
  id: string;
  memoryId: string;
  label: string;
  type: MemoryEntry['type'];
  importance: number;
  confidence: number;
  position: { x: number; y: number; z?: number };
  size: number;
  color: string;
  metadata: Record<string, any>;
}

export interface KnowledgeEdge {
  id: string;
  source: string;
  target: string;
  type: MemoryRelationship['type'];
  strength: number;
  weight: number;
  color: string;
  metadata: Record<string, any>;
}

export interface ContextSnapshot {
  id: string;
  agentId: string;
  name: string;
  description?: string;
  timestamp: Date;
  context: {
    currentTask?: string;
    activeMemories: string[]; // memory IDs
    recentInteractions: Array<{
      type: string;
      content: string;
      timestamp: Date;
    }>;
    state: Record<string, any>;
  };
  size: number;
  compressed: boolean;
}

export interface MemoryHealthMetrics {
  agentId: string;
  timestamp: Date;
  metrics: {
    totalMemoryUsage: number;
    activeMemoryUsage: number;
    fragmentationRatio: number;
    accessLatency: number; // milliseconds
    hitRate: number; // cache hit rate 0-1
    consistencyViolations: number;
    compressionRatio: number;
    cleanupOperations: number;
    memoryPressure: number; // 0-1
  };
  alerts: MemoryAlert[];
}

export interface MemoryAlert {
  id: string;
  agentId: string;
  type: 'memory_pressure' | 'consistency_violation' | 'fragmentation' | 'access_latency' | 'size_limit';
  severity: 'low' | 'medium' | 'high' | 'critical';
  message: string;
  value: number;
  threshold: number;
  timestamp: Date;
  acknowledged: boolean;
  resolved: boolean;
}

export interface MemoryOptimization {
  id: string;
  agentId: string;
  type: 'compression' | 'consolidation' | 'cleanup' | 'defragmentation' | 'reindexing';
  status: 'pending' | 'running' | 'completed' | 'failed';
  startedAt?: Date;
  completedAt?: Date;
  progress?: {
    current: number;
    total: number;
    message: string;
  };
  results?: {
    spaceSaved: number;
    entriesProcessed: number;
    performanceImprovement: number;
    errors: string[];
  };
  error?: string;
}

export interface AgentLearningMetrics {
  agentId: string;
  period: {
    start: Date;
    end: Date;
  };
  metrics: {
    newMemoriesLearned: number;
    memoriesConsolidated: number;
    knowledgeGrowth: number; // percentage
    learningEfficiency: number; // 0-1
    memoryRetention: number; // 0-1
    adaptationRate: number; // changes per day
  };
  insights: Array<{
    type: 'learning_pattern' | 'knowledge_gap' | 'adaptation_trend' | 'memory_issue';
    title: string;
    description: string;
    severity: 'low' | 'medium' | 'high';
    data: any;
  }>;
}

export class AgentMemoryApiClient {
  private apiClient: ApiClient;

  constructor(baseUrl: string = '/api/agent-memory') {
    this.apiClient = new ApiClient({ baseUrl });
  }

  /**
   * Agent Memory endpoints
   */
  async getAgentMemories(): Promise<AgentMemory[]> {
    const response = await this.apiClient.request<AgentMemory[]>('/agents');
    return response;
  }

  async getAgentMemory(agentId: string): Promise<AgentMemory> {
    const response = await this.apiClient.request<AgentMemory>(`/agents/${agentId}`);
    return response;
  }

  async updateAgentMemoryConfiguration(
    agentId: string,
    config: Partial<AgentMemory['configuration']>
  ): Promise<AgentMemory> {
    const response = await this.apiClient.request<AgentMemory>(
      `/agents/${agentId}/configuration`,
      {
        method: 'PATCH',
        body: JSON.stringify(config)
      }
    );
    return response;
  }

  /**
   * Memory Entry endpoints
   */
  async getMemoryEntry(memoryId: string, includeRelationships: boolean = false): Promise<MemoryEntry> {
    const response = await this.apiClient.request<MemoryEntry>(
      `/entries/${memoryId}?relationships=${includeRelationships}`
    );
    return response;
  }

  async searchMemories(query: MemoryQuery): Promise<MemorySearchResult> {
    const response = await this.apiClient.request<MemorySearchResult>('/entries/search', {
      method: 'POST',
      body: JSON.stringify(query)
    });
    return response;
  }

  async createMemoryEntry(entry: Omit<MemoryEntry, 'id' | 'accessCount' | 'lastAccessed' | 'createdAt' | 'updatedAt'>): Promise<MemoryEntry> {
    const response = await this.apiClient.request<MemoryEntry>('/entries', {
      method: 'POST',
      body: JSON.stringify(entry)
    });
    return response;
  }

  async updateMemoryEntry(memoryId: string, updates: Partial<MemoryEntry>): Promise<MemoryEntry> {
    const response = await this.apiClient.request<MemoryEntry>(`/entries/${memoryId}`, {
      method: 'PATCH',
      body: JSON.stringify(updates)
    });
    return response;
  }

  async deleteMemoryEntry(memoryId: string): Promise<void> {
    await this.apiClient.request<void>(`/entries/${memoryId}`, {
      method: 'DELETE'
    });
  }

  async getMemoryRelationships(memoryId: string): Promise<MemoryRelationship[]> {
    const response = await this.apiClient.request<MemoryRelationship[]>(`/entries/${memoryId}/relationships`);
    return response;
  }

  async addMemoryRelationship(
    memoryId: string,
    relationship: Omit<MemoryRelationship, 'id'>
  ): Promise<MemoryRelationship> {
    const response = await this.apiClient.request<MemoryRelationship>(`/entries/${memoryId}/relationships`, {
      method: 'POST',
      body: JSON.stringify(relationship)
    });
    return response;
  }

  async updateMemoryRelationship(
    memoryId: string,
    relationshipId: string,
    updates: Partial<MemoryRelationship>
  ): Promise<MemoryRelationship> {
    const response = await this.apiClient.request<MemoryRelationship>(
      `/entries/${memoryId}/relationships/${relationshipId}`,
      {
        method: 'PATCH',
        body: JSON.stringify(updates)
      }
    );
    return response;
  }

  async deleteMemoryRelationship(memoryId: string, relationshipId: string): Promise<void> {
    await this.apiClient.request<void>(
      `/entries/${memoryId}/relationships/${relationshipId}`,
      {
        method: 'DELETE'
      }
    );
  }

  /**
   * Knowledge Graph endpoints
   */
  async getKnowledgeGraph(agentId?: string, options?: {
    maxNodes?: number;
    minImportance?: number;
    includeTypes?: MemoryEntry['type'][];
    layout?: 'force' | 'hierarchical' | 'circular';
  }): Promise<KnowledgeGraph> {
    const params = new URLSearchParams();
    if (agentId) params.append('agentId', agentId);
    if (options?.maxNodes) params.append('maxNodes', options.maxNodes.toString());
    if (options?.minImportance) params.append('minImportance', options.minImportance.toString());
    if (options?.includeTypes) params.append('includeTypes', options.includeTypes.join(','));
    if (options?.layout) params.append('layout', options.layout);

    const query = params.toString() ? `?${params.toString()}` : '';
    const response = await this.apiClient.request<KnowledgeGraph>(`/knowledge-graph${query}`);
    return response;
  }

  async updateKnowledgeGraphLayout(
    agentId: string,
    layout: 'force' | 'hierarchical' | 'circular',
    positions?: Record<string, { x: number; y: number; z?: number }>
  ): Promise<KnowledgeGraph> {
    const response = await this.apiClient.request<KnowledgeGraph>(`/knowledge-graph/${agentId}/layout`, {
      method: 'POST',
      body: JSON.stringify({ layout, positions })
    });
    return response;
  }

  async getKnowledgeGraphInsights(agentId: string): Promise<{
    clusters: Array<{
      id: string;
      nodes: string[];
      theme: string;
      importance: number;
    }>;
    paths: Array<{
      start: string;
      end: string;
      path: string[];
      strength: number;
    }>;
    anomalies: Array<{
      nodeId: string;
      type: 'isolated' | 'overconnected' | 'inconsistent';
      severity: number;
    }>;
  }> {
    const response = await this.apiClient.request<{
      clusters: Array<{
        id: string;
        nodes: string[];
        theme: string;
        importance: number;
      }>;
      paths: Array<{
        start: string;
        end: string;
        path: string[];
        strength: number;
      }>;
      anomalies: Array<{
        nodeId: string;
        type: 'isolated' | 'overconnected' | 'inconsistent';
        severity: number;
      }>;
    }>(`/knowledge-graph/${agentId}/insights`);
    return response;
  }

  /**
   * Context Management endpoints
   */
  async getContextSnapshots(agentId?: string): Promise<ContextSnapshot[]> {
    const params = agentId ? `?agentId=${agentId}` : '';
    const response = await this.apiClient.request<ContextSnapshot[]>(`/context/snapshots${params}`);
    return response;
  }

  async createContextSnapshot(
    agentId: string,
    snapshot: Omit<ContextSnapshot, 'id' | 'timestamp' | 'size' | 'compressed'>
  ): Promise<ContextSnapshot> {
    const response = await this.apiClient.request<ContextSnapshot>(`/context/snapshots`, {
      method: 'POST',
      body: JSON.stringify({ ...snapshot, agentId })
    });
    return response;
  }

  async restoreContextSnapshot(snapshotId: string): Promise<{
    success: boolean;
    restoredMemories: number;
    errors: string[];
  }> {
    const response = await this.apiClient.request<{
      success: boolean;
      restoredMemories: number;
      errors: string[];
    }>(`/context/snapshots/${snapshotId}/restore`, {
      method: 'POST'
    });
    return response;
  }

  async deleteContextSnapshot(snapshotId: string): Promise<void> {
    await this.apiClient.request<void>(`/context/snapshots/${snapshotId}`, {
      method: 'DELETE'
    });
  }

  /**
   * Memory Health endpoints
   */
  async getMemoryHealthMetrics(agentId?: string): Promise<MemoryHealthMetrics[]> {
    const params = agentId ? `?agentId=${agentId}` : '';
    const response = await this.apiClient.request<MemoryHealthMetrics[]>(`/health/metrics${params}`);
    return response;
  }

  async getMemoryAlerts(agentId?: string, status?: 'active' | 'acknowledged' | 'resolved'): Promise<MemoryAlert[]> {
    const params = new URLSearchParams();
    if (agentId) params.append('agentId', agentId);
    if (status) params.append('status', status);

    const query = params.toString() ? `?${params.toString()}` : '';
    const response = await this.apiClient.request<MemoryAlert[]>(`/health/alerts${query}`);
    return response;
  }

  async acknowledgeMemoryAlert(alertId: string): Promise<void> {
    await this.apiClient.request<void>(`/health/alerts/${alertId}/acknowledge`, {
      method: 'POST'
    });
  }

  async resolveMemoryAlert(alertId: string): Promise<void> {
    await this.apiClient.request<void>(`/health/alerts/${alertId}/resolve`, {
      method: 'POST'
    });
  }

  /**
   * Memory Optimization endpoints
   */
  async getMemoryOptimizations(agentId?: string): Promise<MemoryOptimization[]> {
    const params = agentId ? `?agentId=${agentId}` : '';
    const response = await this.apiClient.request<MemoryOptimization[]>(`/optimization/jobs${params}`);
    return response;
  }

  async startMemoryOptimization(
    agentId: string,
    optimizationType: MemoryOptimization['type'],
    options?: Record<string, any>
  ): Promise<MemoryOptimization> {
    const response = await this.apiClient.request<MemoryOptimization>('/optimization/jobs', {
      method: 'POST',
      body: JSON.stringify({ agentId, type: optimizationType, options })
    });
    return response;
  }

  async getMemoryOptimizationStatus(jobId: string): Promise<MemoryOptimization> {
    const response = await this.apiClient.request<MemoryOptimization>(`/optimization/jobs/${jobId}`);
    return response;
  }

  async cancelMemoryOptimization(jobId: string): Promise<void> {
    await this.apiClient.request<void>(`/optimization/jobs/${jobId}/cancel`, {
      method: 'POST'
    });
  }

  /**
   * Learning Metrics endpoints
   */
  async getAgentLearningMetrics(agentId: string, period: { start: Date; end: Date }): Promise<AgentLearningMetrics> {
    const params = new URLSearchParams({
      start: period.start.toISOString(),
      end: period.end.toISOString()
    });

    const response = await this.apiClient.request<AgentLearningMetrics>(
      `/learning/metrics/${agentId}?${params.toString()}`
    );
    return response;
  }

  async getLearningInsights(agentId: string): Promise<AgentLearningMetrics['insights']> {
    const response = await this.apiClient.request<AgentLearningMetrics['insights']>(
      `/learning/insights/${agentId}`
    );
    return response;
  }

  /**
   * Memory Export/Import endpoints
   */
  async exportAgentMemory(
    agentId: string,
    format: 'json' | 'csv' | 'xml' = 'json',
    options?: {
      types?: MemoryEntry['type'][];
      dateRange?: { start: Date; end: Date };
      includeRelationships?: boolean;
    }
  ): Promise<Blob> {
    const params = new URLSearchParams({ format });
    if (options?.types) params.append('types', options.types.join(','));
    if (options?.dateRange) {
      params.append('startDate', options.dateRange.start.toISOString());
      params.append('endDate', options.dateRange.end.toISOString());
    }
    if (options?.includeRelationships !== undefined) {
      params.append('includeRelationships', options.includeRelationships.toString());
    }

    const response = await fetch(
      `${this.apiClient['config'].baseUrl}/export/${agentId}?${params.toString()}`,
      {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${this.apiClient['config'].authToken}`
        }
      }
    );

    if (!response.ok) {
      throw new Error(`Export failed: ${response.statusText}`);
    }

    return response.blob();
  }

  async importAgentMemory(
    agentId: string,
    file: File,
    options?: {
      mergeStrategy?: 'replace' | 'merge' | 'skip_duplicates';
      validateBeforeImport?: boolean;
    }
  ): Promise<{
    success: boolean;
    importedEntries: number;
    skippedEntries: number;
    errors: string[];
  }> {
    const formData = new FormData();
    formData.append('file', file);
    if (options) {
      formData.append('options', JSON.stringify(options));
    }

    const response = await this.apiClient.request<{
      success: boolean;
      importedEntries: number;
      skippedEntries: number;
      errors: string[];
    }>(`/import/${agentId}`, {
      method: 'POST',
      body: formData
    });
    return response;
  }

  /**
   * Memory Analytics endpoints
   */
  async getMemoryAnalytics(agentId?: string): Promise<{
    overview: {
      totalAgents: number;
      totalMemories: number;
      totalSize: number;
      averageHealth: number;
    };
    typeDistribution: Record<MemoryEntry['type'], number>;
    temporalPatterns: Array<{
      period: string;
      memoriesCreated: number;
      memoriesAccessed: number;
    }>;
    performanceMetrics: {
      averageAccessTime: number;
      cacheHitRate: number;
      compressionRatio: number;
    };
  }> {
    const params = agentId ? `?agentId=${agentId}` : '';
    const response = await this.apiClient.request<{
      overview: {
        totalAgents: number;
        totalMemories: number;
        totalSize: number;
        averageHealth: number;
      };
      typeDistribution: Record<MemoryEntry['type'], number>;
      temporalPatterns: Array<{
        period: string;
        memoriesCreated: number;
        memoriesAccessed: number;
      }>;
      performanceMetrics: {
        averageAccessTime: number;
        cacheHitRate: number;
        compressionRatio: number;
      };
    }>(`/analytics/overview${params}`);
    return response;
  }
}

// Export singleton instance
export const agentMemoryApiClient = new AgentMemoryApiClient();
