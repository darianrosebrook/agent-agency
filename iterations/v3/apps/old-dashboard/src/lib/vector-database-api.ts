/**
 * Vector Database API Client
 * API client for vector database operations and analytics
 *
 * @author @darianrosebrook
 */

import { ApiClient } from './api-client';

export interface VectorEmbedding {
  id: string;
  embedding: number[];
  metadata: {
    text?: string;
    category?: string;
    model?: string;
    timestamp: Date;
    source?: string;
    dimensions: number;
  };
}

export interface VectorSearchResult {
  id: string;
  embedding: number[];
  similarity: number;
  metadata: VectorEmbedding['metadata'];
}

export interface VectorCluster {
  id: string;
  center: number[];
  radius: number;
  label: string;
  color: string;
  vectorCount: number;
  averageSimilarity: number;
}

export interface VectorAnalytics {
  totalVectors: number;
  averageDimensions: number;
  clusterCount: number;
  averageSimilarity: number;
  topCategories: Array<{
    category: string;
    count: number;
    percentage: number;
  }>;
  performanceMetrics: {
    searchLatency: number;
    indexingTime: number;
    memoryUsage: number;
    queryThroughput: number;
  };
}

export interface VectorSearchParams {
  query: string;
  limit?: number;
  threshold?: number;
  filters?: {
    category?: string;
    model?: string;
    dateRange?: {
      start: Date;
      end: Date;
    };
  };
}

export interface VectorProjectionParams {
  method: 'pca' | 'tsne' | 'umap' | 'custom';
  dimensions: 2 | 3;
  perplexity?: number;
  learningRate?: number;
  iterations?: number;
}

export class VectorDatabaseApiClient {
  private apiClient: ApiClient;

  constructor(baseUrl: string = '/api/vector-database') {
    this.apiClient = new ApiClient({ baseUrl });
  }

  /**
   * Search for similar vectors
   */
  async searchVectors(params: VectorSearchParams): Promise<VectorSearchResult[]> {
    const response = await this.apiClient.request<VectorSearchResult[]>('/search', {
      method: 'POST',
      body: JSON.stringify(params)
    });
    return response;
  }

  /**
   * Get vector by ID
   */
  async getVector(id: string): Promise<VectorEmbedding> {
    const response = await this.apiClient.request<VectorEmbedding>(`/vectors/${id}`);
    return response;
  }

  /**
   * Get multiple vectors by IDs
   */
  async getVectors(ids: string[]): Promise<VectorEmbedding[]> {
    const response = await this.apiClient.request<VectorEmbedding[]>('/vectors/batch', {
      method: 'POST',
      body: JSON.stringify({ ids })
    });
    return response;
  }

  /**
   * Add new vector embedding
   */
  async addVector(vector: Omit<VectorEmbedding, 'id'>): Promise<VectorEmbedding> {
    const response = await this.apiClient.request<VectorEmbedding>('/vectors', {
      method: 'POST',
      body: JSON.stringify(vector)
    });
    return response;
  }

  /**
   * Update vector metadata
   */
  async updateVector(id: string, updates: Partial<VectorEmbedding>): Promise<VectorEmbedding> {
    const response = await this.apiClient.request<VectorEmbedding>(`/vectors/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(updates)
    });
    return response;
  }

  /**
   * Delete vector
   */
  async deleteVector(id: string): Promise<void> {
    await this.apiClient.request<void>(`/vectors/${id}`, {
      method: 'DELETE'
    });
  }

  /**
   * Get vector clusters
   */
  async getClusters(): Promise<VectorCluster[]> {
    const response = await this.apiClient.request<VectorCluster[]>('/clusters');
    return response;
  }

  /**
   * Create new cluster
   */
  async createCluster(cluster: Omit<VectorCluster, 'id'>): Promise<VectorCluster> {
    const response = await this.apiClient.request<VectorCluster>('/clusters', {
      method: 'POST',
      body: JSON.stringify(cluster)
    });
    return response;
  }

  /**
   * Update cluster
   */
  async updateCluster(id: string, updates: Partial<VectorCluster>): Promise<VectorCluster> {
    const response = await this.apiClient.request<VectorCluster>(`/clusters/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(updates)
    });
    return response;
  }

  /**
   * Delete cluster
   */
  async deleteCluster(id: string): Promise<void> {
    await this.apiClient.request<void>(`/clusters/${id}`, {
      method: 'DELETE'
    });
  }

  /**
   * Get vector analytics
   */
  async getAnalytics(): Promise<VectorAnalytics> {
    const response = await this.apiClient.request<VectorAnalytics>('/analytics');
    return response;
  }

  /**
   * Get performance metrics
   */
  async getPerformanceMetrics(timeRange: '1h' | '6h' | '24h' | '7d' = '24h'): Promise<{
    searchLatency: number[];
    indexingTime: number[];
    memoryUsage: number[];
    queryThroughput: number[];
    timestamps: Date[];
  }> {
    const response = await this.apiClient.request<{
      searchLatency: number[];
      indexingTime: number[];
      memoryUsage: number[];
      queryThroughput: number[];
      timestamps: Date[];
    }>(`/performance?timeRange=${timeRange}`);
    return response;
  }

  /**
   * Project high-dimensional vectors to 2D/3D
   */
  async projectVectors(
    vectors: VectorEmbedding[],
    params: VectorProjectionParams
  ): Promise<Array<{
    id: string;
    position: [number, number, number];
    embedding: number[];
    metadata: VectorEmbedding['metadata'];
  }>> {
    const response = await this.apiClient.request<Array<{
      id: string;
      position: [number, number, number];
      embedding: number[];
      metadata: VectorEmbedding['metadata'];
    }>>('/project', {
      method: 'POST',
      body: JSON.stringify({
        vectors: vectors.map(v => ({
          id: v.id,
          embedding: v.embedding,
          metadata: v.metadata
        })),
        ...params
      })
    });
    return response;
  }

  /**
   * Get similar vectors for a given vector
   */
  async getSimilarVectors(
    vectorId: string,
    limit: number = 10,
    threshold: number = 0.7
  ): Promise<VectorSearchResult[]> {
    const response = await this.apiClient.request<VectorSearchResult[]>(
      `/vectors/${vectorId}/similar?limit=${limit}&threshold=${threshold}`
    );
    return response;
  }

  /**
   * Batch search for multiple queries
   */
  async batchSearch(queries: VectorSearchParams[]): Promise<VectorSearchResult[][]> {
    const response = await this.apiClient.request<VectorSearchResult[][]>('/search/batch', {
      method: 'POST',
      body: JSON.stringify({ queries })
    });
    return response;
  }

  /**
   * Get vector statistics
   */
  async getVectorStats(): Promise<{
    totalVectors: number;
    averageDimensions: number;
    dimensionDistribution: Record<number, number>;
    categoryDistribution: Record<string, number>;
    modelDistribution: Record<string, number>;
    recentActivity: Array<{
      timestamp: Date;
      action: 'add' | 'update' | 'delete' | 'search';
      count: number;
    }>;
  }> {
    const response = await this.apiClient.request<{
      totalVectors: number;
      averageDimensions: number;
      dimensionDistribution: Record<number, number>;
      categoryDistribution: Record<string, number>;
      modelDistribution: Record<string, number>;
      recentActivity: Array<{
        timestamp: Date;
        action: 'add' | 'update' | 'delete' | 'search';
        count: number;
      }>;
    }>('/stats');
    return response;
  }

  /**
   * Optimize vector database
   */
  async optimizeDatabase(): Promise<{
    message: string;
    improvements: string[];
    performanceGain: number;
  }> {
    const response = await this.apiClient.request<{
      message: string;
      improvements: string[];
      performanceGain: number;
    }>('/optimize', {
      method: 'POST'
    });
    return response;
  }

  /**
   * Export vectors
   */
  async exportVectors(format: 'json' | 'csv' | 'parquet' = 'json'): Promise<Blob> {
    const response = await fetch(`${this.apiClient['config'].baseUrl}/export?format=${format}`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${this.apiClient['config'].authToken}`
      }
    });
    
    if (!response.ok) {
      throw new Error(`Export failed: ${response.statusText}`);
    }
    
    return response.blob();
  }

  /**
   * Import vectors
   */
  async importVectors(file: File): Promise<{
    imported: number;
    errors: string[];
    warnings: string[];
  }> {
    const formData = new FormData();
    formData.append('file', file);
    
    const response = await fetch(`${this.apiClient['config'].baseUrl}/import`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.apiClient['config'].authToken}`
      },
      body: formData
    });
    
    if (!response.ok) {
      throw new Error(`Import failed: ${response.statusText}`);
    }
    
    return response.json();
  }
}

// Export singleton instance
export const vectorDatabaseApiClient = new VectorDatabaseApiClient();
