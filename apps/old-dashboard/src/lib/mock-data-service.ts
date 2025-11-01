/**
 * Mock Data Service
 * Provides realistic mock data for development and testing
 *
 * @author @darianrosebrook
 */

import mockData from '@/mock-data/vector-database-api-mock.json';
import { VectorEmbedding, VectorSearchResult, VectorCluster, VectorAnalytics } from './vector-database-api';

export class MockDataService {
  private static instance: MockDataService;
  private data = mockData;

  static getInstance(): MockDataService {
    if (!MockDataService.instance) {
      MockDataService.instance = new MockDataService();
    }
    return MockDataService.instance;
  }

  /**
   * Get all embeddings with 3D projections
   */
  getEmbeddingsWithProjections(): Array<{
    id: string;
    position: [number, number, number];
    embedding: number[];
    metadata: VectorEmbedding['metadata'];
  }> {
    return this.data.embeddings.map((embedding, index) => ({
      id: embedding.id,
      position: this.projectTo3D(embedding.embedding, index),
      embedding: embedding.embedding,
      metadata: {
        ...embedding.metadata,
        timestamp: new Date(embedding.metadata.timestamp)
      }
    }));
  }

  /**
   * Get search results with similarity scores
   */
  getSearchResults(): VectorSearchResult[] {
    return this.data.searchResults.map(result => ({
      id: result.id,
      embedding: result.embedding,
      similarity: result.similarity,
      metadata: {
        ...result.metadata,
        timestamp: new Date(result.metadata.timestamp)
      }
    }));
  }

  /**
   * Get clusters with 3D centers
   */
  getClustersWith3DCenters(): Array<{
    id: string;
    center: [number, number, number];
    radius: number;
    label: string;
    color: string;
    vectorCount: number;
    averageSimilarity: number;
  }> {
    return this.data.clusters.map((cluster, index) => ({
      id: cluster.id,
      center: this.projectTo3D(cluster.center, index),
      radius: cluster.radius,
      label: cluster.label,
      color: cluster.color,
      vectorCount: cluster.vectorCount,
      averageSimilarity: cluster.averageSimilarity
    }));
  }

  /**
   * Get analytics data
   */
  getAnalytics(): VectorAnalytics {
    return {
      totalVectors: this.data.analytics.totalVectors,
      averageDimensions: this.data.analytics.averageDimensions,
      clusterCount: this.data.analytics.clusterCount,
      averageSimilarity: this.data.analytics.averageSimilarity,
      topCategories: this.data.analytics.topCategories,
      performanceMetrics: {
        searchLatency: this.data.analytics.performanceMetrics.averageSearchTime,
        indexingTime: 0, // Not in mock data
        memoryUsage: this.data.analytics.performanceMetrics.memoryUsage,
        queryThroughput: this.data.analytics.performanceMetrics.queryThroughput
      }
    };
  }

  /**
   * Get performance metrics over time
   */
  getPerformanceMetrics(timeRange: '1h' | '6h' | '24h' | '7d' = '24h'): {
    searchLatency: number[];
    indexingTime: number[];
    memoryUsage: number[];
    queryThroughput: number[];
    timestamps: Date[];
  } {
    const now = new Date();
    const hours = timeRange === '1h' ? 1 : timeRange === '6h' ? 6 : timeRange === '24h' ? 24 : 168;
    const points = Math.min(hours * 4, 100); // 4 points per hour, max 100 points

    const searchLatency: number[] = [];
    const indexingTime: number[] = [];
    const memoryUsage: number[] = [];
    const queryThroughput: number[] = [];
    const timestamps: Date[] = [];

    for (let i = 0; i < points; i++) {
      const timestamp = new Date(now.getTime() - (i * (hours * 60 * 60 * 1000) / points));
      
      // Generate realistic time series data
      searchLatency.push(8 + Math.random() * 8 + Math.sin(i * 0.1) * 2);
      indexingTime.push(50 + Math.random() * 100 + Math.cos(i * 0.05) * 20);
      memoryUsage.push(1.5 + Math.random() * 0.8 + Math.sin(i * 0.08) * 0.2);
      queryThroughput.push(40 + Math.random() * 20 + Math.cos(i * 0.12) * 5);
      timestamps.push(timestamp);
    }

    return {
      searchLatency: searchLatency.reverse(),
      indexingTime: indexingTime.reverse(),
      memoryUsage: memoryUsage.reverse(),
      queryThroughput: queryThroughput.reverse(),
      timestamps: timestamps.reverse()
    };
  }

  /**
   * Search for similar vectors
   */
  searchVectors(query: string, limit: number = 10, threshold: number = 0.7): VectorSearchResult[] {
    // Simulate search by returning filtered results
    const results = this.getSearchResults();
    return results
      .filter(result => result.similarity >= threshold)
      .slice(0, limit);
  }

  /**
   * Get vector by ID
   */
  getVector(id: string): VectorEmbedding | null {
    const embedding = this.data.embeddings.find(e => e.id === id);
    if (!embedding) return null;

    return {
      id: embedding.id,
      embedding: embedding.embedding,
      metadata: {
        ...embedding.metadata,
        timestamp: new Date(embedding.metadata.timestamp)
      }
    };
  }

  /**
   * Get multiple vectors by IDs
   */
  getVectors(ids: string[]): VectorEmbedding[] {
    return ids
      .map(id => this.getVector(id))
      .filter((vector): vector is VectorEmbedding => vector !== null);
  }

  /**
   * Get vector statistics
   */
  getVectorStats() {
    return {
      totalVectors: this.data.analytics.totalVectors,
      averageDimensions: this.data.analytics.averageDimensions,
      dimensionDistribution: {
        1536: this.data.analytics.totalVectors * 0.8,
        768: this.data.analytics.totalVectors * 0.15,
        384: this.data.analytics.totalVectors * 0.05
      },
      categoryDistribution: this.data.analytics.topCategories.reduce((acc, cat) => {
        acc[cat.category] = cat.count;
        return acc;
      }, {} as Record<string, number>),
      modelDistribution: {
        'text-embedding-ada-002': this.data.analytics.totalVectors * 0.7,
        'text-embedding-3-small': this.data.analytics.totalVectors * 0.2,
        'text-embedding-3-large': this.data.analytics.totalVectors * 0.1
      },
      recentActivity: [
        {
          timestamp: new Date(Date.now() - 5 * 60 * 1000),
          action: 'add' as const,
          count: 12
        },
        {
          timestamp: new Date(Date.now() - 15 * 60 * 1000),
          action: 'search' as const,
          count: 8
        },
        {
          timestamp: new Date(Date.now() - 30 * 60 * 1000),
          action: 'update' as const,
          count: 3
        }
      ]
    };
  }

  /**
   * Project high-dimensional vector to 3D space
   */
  private projectTo3D(embedding: number[], index: number): [number, number, number] {
    // Simple PCA-like projection for demonstration
    // In production, you'd use proper dimensionality reduction
    
    if (embedding.length < 3) {
      return [embedding[0] || 0, embedding[1] || 0, 0];
    }

    // Use first 3 dimensions with some variation based on index
    const x = embedding[0] * 2 + (index % 3) * 0.5;
    const y = embedding[1] * 2 + (index % 2) * 0.3;
    const z = embedding[2] * 2 + (index % 4) * 0.2;

    return [x, y, z];
  }

  /**
   * Generate mock time series data for charts
   */
  generateTimeSeriesData(points: number = 20, type: 'latency' | 'throughput' | 'memory' | 'accuracy' = 'latency') {
    const data = [];
    const now = new Date();

    for (let i = 0; i < points; i++) {
      const timestamp = new Date(now.getTime() - (points - i) * 5 * 60 * 1000); // 5 minute intervals
      let value: number;

      switch (type) {
        case 'latency':
          value = 10 + Math.random() * 15 + Math.sin(i * 0.3) * 5;
          break;
        case 'throughput':
          value = 30 + Math.random() * 20 + Math.cos(i * 0.2) * 8;
          break;
        case 'memory':
          value = 1.2 + Math.random() * 0.8 + Math.sin(i * 0.4) * 0.3;
          break;
        case 'accuracy':
          value = 0.85 + Math.random() * 0.1 + Math.cos(i * 0.15) * 0.03;
          break;
        default:
          value = Math.random() * 100;
      }

      data.push({
        x: i,
        y: value,
        timestamp,
        category: type
      });
    }

    return data;
  }

  /**
   * Generate mock scatter plot data
   */
  generateScatterData(points: number = 50) {
    const data = [];
    const categories = ['neural', 'transformer', 'cnn', 'rnn', 'attention', 'embedding'];

    for (let i = 0; i < points; i++) {
      data.push({
        x: Math.random() * 100,
        y: Math.random() * 100,
        size: 3 + Math.random() * 8,
        category: categories[Math.floor(Math.random() * categories.length)]
      });
    }

    return data;
  }
}

// Export singleton instance
export const mockDataService = MockDataService.getInstance();
