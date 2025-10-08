/**
 * @fileoverview VectorDAO Unit Tests
 * @author @darianrosebrook
 */

import { VectorDAO } from '../../../src/data/dao/VectorDAO';
import { DataLayer } from '../../../src/data/DataLayer';
import { Logger } from '../../../src/utils/Logger';
import { ConnectionPool, CacheProvider } from '../../../src/data/types';

// Concrete implementation for testing
class TestVectorEntity {
  id: string;
  tenantId: string;
  embedding?: number[];
  createdAt: Date;
  updatedAt: Date;
  name: string;

  constructor(id: string, tenantId: string, embedding?: number[], name: string = 'Test') {
    this.id = id;
    this.tenantId = tenantId;
    this.embedding = embedding;
    this.name = name;
    this.createdAt = new Date();
    this.updatedAt = new Date();
  }
}

class TestVectorDAO extends VectorDAO<TestVectorEntity> {
  protected getColumns(): string[] {
    return ['name'];
  }

  protected getValues(entity: Omit<TestVectorEntity, 'id' | 'createdAt' | 'updatedAt'>): any[] {
    return [entity.name];
  }

  protected mapRowToEntity(row: any): TestVectorEntity {
    return new TestVectorEntity(
      row.id,
      row.tenant_id,
      row.embedding,
      row.name
    );
  }

  protected mapFieldToColumn(field: string): string {
    const fieldMap: Record<string, string> = {
      tenantId: 'tenant_id',
      createdAt: 'created_at',
      updatedAt: 'updated_at'
    };
    return fieldMap[field] || field;
  }
}

describe('VectorDAO', () => {
  let vectorDAO: TestVectorDAO;
  let dataLayer: DataLayer;
  let mockConnection: jest.Mocked<ConnectionPool>;
  let mockCache: jest.Mocked<CacheProvider>;
  let mockLogger: Logger;

  const testConfig = {
    database: {
      host: 'localhost',
      port: 5432,
      database: 'test_db',
      username: 'test_user',
      password: 'test_pass',
      maxConnections: 5,
      idleTimeoutMillis: 1000,
      connectionTimeoutMillis: 1000,
    },
    cache: {
      host: 'localhost',
      port: 6379,
      password: 'test_cache_pass',
      keyPrefix: 'test:',
      ttl: 300,
    },
    enableCache: true,
    enableMetrics: false,
    queryTimeout: 5000,
  };

  beforeEach(() => {
    mockLogger = {
      info: jest.fn(),
      warn: jest.fn(),
      error: jest.fn(),
      debug: jest.fn(),
    } as any;

    mockConnection = {
      connect: jest.fn().mockResolvedValue({}),
      query: jest.fn().mockResolvedValue({
        success: true,
        data: [],
        duration: 10,
        queryId: 'test-query',
      }),
      transaction: jest.fn().mockImplementation(async (callback) => {
        const client = {};
        const result = await callback(client);
        return {
          success: true,
          data: result,
          duration: 10,
          queryId: 'test-transaction',
        };
      }),
      healthCheck: jest.fn().mockResolvedValue({
        status: 'healthy',
        database: { connected: true, latency: 5 },
      }),
      getStats: jest.fn().mockResolvedValue({}),
      close: jest.fn().mockResolvedValue(undefined),
    };

    mockCache = {
      get: jest.fn().mockResolvedValue({
        success: true,
        hit: false,
        duration: 5,
      }),
      set: jest.fn().mockResolvedValue({
        success: true,
        data: true,
        hit: false,
        duration: 5,
      }),
      delete: jest.fn().mockResolvedValue({
        success: true,
        data: true,
        hit: false,
        duration: 5,
      }),
      exists: jest.fn().mockResolvedValue({
        success: true,
        data: false,
        hit: false,
        duration: 5,
      }),
      clear: jest.fn().mockResolvedValue({
        success: true,
        data: 0,
        hit: false,
        duration: 5,
      }),
      getStats: jest.fn().mockResolvedValue({
        success: true,
        data: {},
        hit: false,
        duration: 5,
      }),
      close: jest.fn().mockResolvedValue(undefined),
    };

    dataLayer = new DataLayer(testConfig, mockLogger);
    (dataLayer as any).connection = mockConnection;
    (dataLayer as any).cache = mockCache;
    (dataLayer as any).initialized = true;

    vectorDAO = new TestVectorDAO(dataLayer, 'test_vectors', 'TestVector', mockLogger);
  });

  describe('vector validation', () => {
    it('should validate correct vectors', () => {
      const validVector = [0.1, 0.2, 0.3, 0.4, 0.5];
      expect(() => (vectorDAO as any).validateVector(validVector)).not.toThrow();
    });

    it('should reject empty vectors', () => {
      expect(() => (vectorDAO as any).validateVector([])).toThrow('Vector must be a non-empty array');
    });

    it('should reject non-array inputs', () => {
      expect(() => (vectorDAO as any).validateVector('not an array')).toThrow('Vector must be a non-empty array');
    });

    it('should reject vectors with non-numeric values', () => {
      expect(() => (vectorDAO as any).validateVector([0.1, 'invalid', 0.3])).toThrow('All vector values must be valid numbers');
    });

    it('should reject NaN values', () => {
      expect(() => (vectorDAO as any).validateVector([0.1, NaN, 0.3])).toThrow('All vector values must be valid numbers');
    });

    it('should reject vectors that are too large', () => {
      const largeVector = new Array(5000).fill(0.1);
      expect(() => (vectorDAO as any).validateVector(largeVector)).toThrow('Vector dimension cannot exceed 4096');
    });
  });

  describe('similarity search', () => {
    const testVector = [0.1, 0.2, 0.3];
    const mockResults = [
      { id: 'vec1', score: 0.95, metadata: { type: 'test' } },
      { id: 'vec2', score: 0.89, metadata: { type: 'test' } }
    ];

    beforeEach(() => {
      mockConnection.query.mockResolvedValue({
        success: true,
        data: mockResults,
        duration: 15,
        queryId: 'similarity-search',
      });
    });

    it('should perform similarity search with valid vector', async () => {
      const result = await vectorDAO.findSimilar(testVector);

      expect(result.success).toBe(true);
      expect(result.data).toHaveLength(2);
      expect(result.data![0]).toEqual(mockResults[0]);
      expect(result.data![1]).toEqual(mockResults[1]);
      expect(mockConnection.query).toHaveBeenCalled();
    });

    it('should apply default limits and thresholds', async () => {
      await vectorDAO.findSimilar(testVector);

      const queryCall = mockConnection.query.mock.calls[0][0];
      expect(queryCall).toContain('LIMIT 10');
      expect(queryCall).toContain('> $2'); // threshold parameter
    });

    it('should use custom search options', async () => {
      await vectorDAO.findSimilar(testVector, {
        limit: 5,
        threshold: 0.8,
        cache: true,
        timeout: 1000
      });

      const queryCall = mockConnection.query.mock.calls[0][0];
      expect(queryCall).toContain('LIMIT 5');
      expect(mockConnection.query).toHaveBeenCalledWith(
        expect.any(String),
        [testVector, 0.8, 5],
        expect.objectContaining({ cache: true, timeout: 1000 })
      );
    });

    it('should handle empty results', async () => {
      mockConnection.query.mockResolvedValue({
        success: true,
        data: [],
        duration: 10,
        queryId: 'empty-search',
      });

      const result = await vectorDAO.findSimilar(testVector);
      expect(result.success).toBe(true);
      expect(result.data).toEqual([]);
    });

    it('should handle search errors', async () => {
      mockConnection.query.mockResolvedValue({
        success: false,
        error: 'Database error',
        duration: 10,
        queryId: 'error-search',
      });

      await expect(vectorDAO.findSimilar(testVector)).rejects.toThrow('Failed to perform vector similarity search');
    });
  });

  describe('similarity search by ID', () => {
    const testId = 'test-entity-123';
    const testTenantId = 'tenant-456';
    const entityWithEmbedding = {
      id: testId,
      tenantId: testTenantId,
      embedding: [0.1, 0.2, 0.3],
      name: 'Test Entity'
    };

    beforeEach(() => {
      // Mock finding the entity first
      mockConnection.query
        .mockResolvedValueOnce({
          success: true,
          data: [entityWithEmbedding],
          duration: 5,
          queryId: 'find-entity',
        })
        .mockResolvedValueOnce({
          success: true,
          data: [
            { id: 'similar1', score: 0.92, metadata: {} },
            { id: 'similar2', score: 0.87, metadata: {} }
          ],
          duration: 12,
          queryId: 'similarity-search',
        });
    });

    it('should find similar entities by ID', async () => {
      const result = await vectorDAO.findSimilarById(testId, testTenantId);

      expect(result.success).toBe(true);
      expect(result.data).toHaveLength(2);
      expect(result.data![0].id).toBe('similar1');
      expect(result.data![1].id).toBe('similar2');
    });

    it('should exclude the original entity from results', async () => {
      await vectorDAO.findSimilarById(testId, testTenantId);

      const similarityQuery = mockConnection.query.mock.calls[1][0];
      expect(similarityQuery).toContain(`id != $2`);
      expect(mockConnection.query.mock.calls[1][1]).toEqual([entityWithEmbedding.embedding, testId, 0.1, 11]); // +1 for exclusion
    });

    it('should handle entity without embedding', async () => {
      mockConnection.query.mockResolvedValueOnce({
        success: true,
        data: [{ id: testId, tenantId: testTenantId, embedding: null }],
        duration: 5,
        queryId: 'find-entity-no-embedding',
      });

      await expect(vectorDAO.findSimilarById(testId, testTenantId))
        .rejects.toThrow('Entity test-entity-123 not found or has no embedding');
    });

    it('should handle entity not found', async () => {
      mockConnection.query.mockResolvedValueOnce({
        success: true,
        data: [],
        duration: 5,
        queryId: 'entity-not-found',
      });

      await expect(vectorDAO.findSimilarById(testId, testTenantId))
        .rejects.toThrow('Entity test-entity-123 not found or has no embedding');
    });
  });

  describe('hybrid search', () => {
    const testVector = [0.1, 0.2, 0.3];
    const metadataFilter = { type: 'document', category: 'technical' };

    beforeEach(() => {
      mockConnection.query.mockResolvedValue({
        success: true,
        data: [
          { id: 'hybrid1', score: 0.88, metadata: { type: 'document', relevance: 0.9 } },
          { id: 'hybrid2', score: 0.76, metadata: { type: 'document', relevance: 0.7 } }
        ],
        duration: 20,
        queryId: 'hybrid-search',
      });
    });

    it('should perform hybrid search combining vector and metadata', async () => {
      const result = await vectorDAO.hybridSearch(testVector, metadataFilter);

      expect(result.success).toBe(true);
      expect(result.data).toHaveLength(2);
      expect(mockConnection.query).toHaveBeenCalled();
    });

    it('should build correct WHERE clause for metadata filters', async () => {
      await vectorDAO.hybridSearch(testVector, metadataFilter);

      const queryCall = mockConnection.query.mock.calls[0][0];
      expect(queryCall).toContain(`metadata->>'type' = $5`);
      expect(queryCall).toContain(`metadata->>'category' = $6`);
    });

    it('should use default vector and metadata weights', async () => {
      await vectorDAO.hybridSearch(testVector, metadataFilter);

      const queryCall = mockConnection.query.mock.calls[0][0];
      expect(queryCall).toContain(`$3 * (1 - (embedding <=> $1::vector))`); // 0.7 weight for vector
      expect(queryCall).toContain(`$4 * 0.8`); // 0.3 weight for metadata
    });

    it('should allow custom weights', async () => {
      const result = await vectorDAO.hybridSearch(testVector, metadataFilter, {
        vectorWeight: 0.8,
        metadataWeight: 0.2
      });

      expect(result.success).toBe(true);
    });

    it('should handle empty metadata filters', async () => {
      const result = await vectorDAO.hybridSearch(testVector, {});

      expect(result.success).toBe(true);
      const queryCall = mockConnection.query.mock.calls[0][0];
      expect(queryCall).not.toContain(`metadata->>`);
    });
  });

  describe('bulk operations', () => {
    const testEntities = [
      { id: 'bulk1', tenantId: 'tenant1', embedding: [0.1, 0.2, 0.3], name: 'Entity 1' },
      { id: 'bulk2', tenantId: 'tenant1', embedding: [0.4, 0.5, 0.6], name: 'Entity 2' },
      { id: 'bulk3', tenantId: 'tenant1', embedding: [0.7, 0.8, 0.9], name: 'Entity 3' }
    ];

    it('should handle bulk embedding updates', async () => {
      const updates = testEntities.map(entity => ({
        id: entity.id,
        tenantId: entity.tenantId,
        embedding: entity.embedding
      }));

      mockConnection.query.mockResolvedValue({
        success: true,
        data: [],
        duration: 25,
        queryId: 'bulk-update',
      });

      const result = await vectorDAO.updateEmbeddings(updates);

      expect(result.success).toBe(true);
      expect(result.processed).toBe(3);
      expect(result.successful).toBe(3);
      expect(result.failed).toBe(0);
    });

    it('should handle bulk update errors with continueOnError', async () => {
      const updates = testEntities.map(entity => ({
        id: entity.id,
        tenantId: entity.tenantId,
        embedding: entity.embedding
      }));

      // Mock alternating success/failure
      let callCount = 0;
      mockConnection.query.mockImplementation(() => {
        callCount++;
        if (callCount % 2 === 0) {
          return Promise.resolve({
            success: false,
            error: 'Update failed',
            duration: 5,
            queryId: `failed-update-${callCount}`,
          });
        }
        return Promise.resolve({
          success: true,
          data: [],
          duration: 5,
          queryId: `success-update-${callCount}`,
        });
      });

      const result = await vectorDAO.updateEmbeddings(updates, { continueOnError: true });

      expect(result.success).toBe(false); // Some failed
      expect(result.processed).toBe(3);
      expect(result.successful).toBe(2); // bulk2 and bulk3 succeed
      expect(result.failed).toBe(1); // bulk1 fails
    });
  });

  describe('statistics', () => {
    it('should get vector statistics', async () => {
      mockConnection.query.mockResolvedValue({
        success: true,
        data: [{
          total_vectors: 150,
          avg_dimensions: 384,
          min_dimensions: 384,
          max_dimensions: 384,
          null_embeddings: 5
        }],
        duration: 8,
        queryId: 'vector-stats',
      });

      const result = await vectorDAO.getVectorStats('tenant-123');

      expect(result.success).toBe(true);
      expect(result.data).toEqual({
        totalVectors: 150,
        avgDimensions: 384,
        minDimensions: 384,
        maxDimensions: 384,
        nullEmbeddings: 5
      });
    });

    it('should handle statistics query errors', async () => {
      mockConnection.query.mockResolvedValue({
        success: false,
        error: 'Stats query failed',
        duration: 5,
        queryId: 'stats-error',
      });

      await expect(vectorDAO.getVectorStats('tenant-123'))
        .rejects.toThrow('Failed to get vector statistics');
    });
  });
});
