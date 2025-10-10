/**
 * Unit tests for Knowledge Seeker component
 * @author @darianrosebrook
 */

import { KnowledgeSeeker } from '../../../src/knowledge/KnowledgeSeeker';
import { SearchProviderFactory } from '../../../src/knowledge/SearchProvider';
import { InformationProcessor } from '../../../src/knowledge/InformationProcessor';
import {
  KnowledgeQuery,
  KnowledgeResponse,
  SearchResult,
  QueryPriority,
  SearchProvider,
  KnowledgeSeekerError,
  KnowledgeSeekerErrorCode,
  ContentType
} from '../../../src/types/knowledge';

describe('KnowledgeSeeker', () => {
  let knowledgeSeeker: KnowledgeSeeker;
  let mockSearchProvider: any;
  let mockProcessor: any;

  beforeEach(() => {
    // Create mocks
    mockSearchProvider = {
      search: jest.fn(),
      isAvailable: jest.fn().mockResolvedValue(true),
      getRateLimitStatus: jest.fn().mockResolvedValue({
        remainingRequests: 100,
        resetTime: new Date(Date.now() + 60000),
        isLimited: false
      }),
      name: SearchProvider.DUCKDUCKGO,
      config: {
        name: SearchProvider.DUCKDUCKGO,
        baseUrl: 'https://duckduckgo.com',
        rateLimit: { requestsPerMinute: 30, requestsPerHour: 100, burstLimit: 10 },
        enabled: true,
        priority: 1
      }
    };

    mockProcessor = {
      processResults: jest.fn(),
      calculateRelevance: jest.fn(),
      deduplicateResults: jest.fn(),
      rankResults: jest.fn()
    };

    knowledgeSeeker = new KnowledgeSeeker(
      {
        cacheEnabled: false, // Disable cache for testing
        minRelevanceThreshold: 0.3
      },
      [mockSearchProvider],
      mockProcessor
    );
  });

  afterEach(() => {
    knowledgeSeeker.destroy();
  });

  describe('search', () => {
    const validQuery: KnowledgeQuery = {
      id: 'test-query-123',
      query: 'TypeScript best practices',
      priority: QueryPriority.MEDIUM,
      maxResults: 10
    };

    const mockResults: SearchResult[] = [
      {
        id: 'result-1',
        title: 'TypeScript Best Practices Guide',
        url: 'https://example.com/ts-guide',
        snippet: 'Learn TypeScript best practices for scalable applications',
        provider: SearchProvider.DUCKDUCKGO,
        relevanceScore: 0.9,
        credibilityScore: 0.8,
        contentType: ContentType.ARTICLE,
        metadata: { domain: 'example.com' }
      },
      {
        id: 'result-2',
        title: 'Advanced TypeScript Patterns',
        url: 'https://example.com/ts-patterns',
        snippet: 'Explore advanced patterns in TypeScript development',
        provider: SearchProvider.DUCKDUCKGO,
        relevanceScore: 0.7,
        credibilityScore: 0.7,
        contentType: ContentType.ARTICLE,
        metadata: { domain: 'example.com' }
      }
    ];

    beforeEach(() => {
      mockSearchProvider.search.mockResolvedValue(mockResults);
      mockProcessor.processResults.mockResolvedValue(mockResults);
    });

    it('should successfully execute a search and return results', async () => {
      const response = await knowledgeSeeker.search(validQuery);

      expect(response).toBeDefined();
      expect(response.queryId).toBe(validQuery.id);
      expect(response.results).toHaveLength(2);
      expect(response.confidence).toBeGreaterThan(0);
      expect(response.processingTimeMs).toBeGreaterThan(0);
      expect(response.sourcesUsed).toContain(SearchProvider.DUCKDUCKGO);
      expect(response.cacheHit).toBe(false);
    });

    it('should filter results below relevance threshold', async () => {
      const lowRelevanceResults = [
        {
          ...mockResults[0],
          relevanceScore: 0.1 // Below threshold
        },
        {
          ...mockResults[1],
          relevanceScore: 0.8 // Above threshold
        }
      ];

      mockProcessor.processResults.mockResolvedValue(lowRelevanceResults);

      const response = await knowledgeSeeker.search(validQuery);

      expect(response.results).toHaveLength(1);
      expect(response.results[0].relevanceScore).toBe(0.8);
    });

    it('should handle empty query', async () => {
      const invalidQuery = { ...validQuery, query: '' };

      await expect(knowledgeSeeker.search(invalidQuery))
        .rejects
        .toThrow(KnowledgeSeekerError);
    });

    it('should handle query too long', async () => {
      const longQuery = 'a'.repeat(1001);
      const invalidQuery = { ...validQuery, query: longQuery };

      await expect(knowledgeSeeker.search(invalidQuery))
        .rejects
        .toThrow(KnowledgeSeekerError);
    });

    it('should handle too many results requested', async () => {
      const invalidQuery = { ...validQuery, maxResults: 200 };

      await expect(knowledgeSeeker.search(invalidQuery))
        .rejects
        .toThrow(KnowledgeSeekerError);
    });

    it('should handle provider search failure gracefully', async () => {
      mockSearchProvider.search.mockRejectedValue(new Error('Network error'));

      const response = await knowledgeSeeker.search(validQuery);

      expect(response.results).toHaveLength(0);
      expect(response.error).toContain('Network error');
    });

    it('should respect timeout configuration', async () => {
      mockSearchProvider.search.mockImplementation(
        () => new Promise(resolve => setTimeout(() => resolve(mockResults), 100))
      );

      const timeoutQuery = { ...validQuery, timeoutMs: 50 };

      const response = await knowledgeSeeker.search(timeoutQuery);

      expect(response.results).toHaveLength(0);
      expect(response.error).toBeDefined();
    });
  });

  describe('searchMultiple', () => {
    const queries: KnowledgeQuery[] = [
      {
        id: 'query-1',
        query: 'React hooks',
        priority: QueryPriority.HIGH
      },
      {
        id: 'query-2',
        query: 'Node.js performance',
        priority: QueryPriority.MEDIUM
      },
      {
        id: 'query-3',
        query: 'Docker best practices',
        priority: QueryPriority.LOW
      }
    ];

    beforeEach(() => {
      mockSearchProvider.search.mockResolvedValue([]);
      mockProcessor.processResults.mockResolvedValue([]);
    });

    it('should process multiple queries', async () => {
      const responses = await knowledgeSeeker.searchMultiple(queries);

      expect(responses).toHaveLength(3);
      expect(responses.every(r => r.queryId)).toBe(true);
    });

    it('should prioritize queries by priority', async () => {
      const responses = await knowledgeSeeker.searchMultiple(queries);

      // Should maintain priority order (HIGH, MEDIUM, LOW)
      expect(responses[0].queryId).toBe('query-1');
      expect(responses[1].queryId).toBe('query-2');
      expect(responses[2].queryId).toBe('query-3');
    });
  });

  describe('caching', () => {
    let cacheEnabledSeeker: KnowledgeSeeker;

    beforeEach(() => {
      cacheEnabledSeeker = new KnowledgeSeeker(
        { cacheEnabled: true, cacheTtlMs: 60000 },
        [mockSearchProvider],
        mockProcessor
      );
    });

    afterEach(() => {
      cacheEnabledSeeker.destroy();
    });

    it('should cache and return cached results', async () => {
      const query: KnowledgeQuery = {
        id: 'cache-test',
        query: 'test query',
        priority: QueryPriority.MEDIUM
      };

      mockSearchProvider.search.mockResolvedValue([]);
      mockProcessor.processResults.mockResolvedValue([]);

      // First search
      const response1 = await cacheEnabledSeeker.search(query);
      expect(response1.cacheHit).toBe(false);

      // Second search with same query
      const response2 = await cacheEnabledSeeker.search(query);
      expect(response2.cacheHit).toBe(true);

      // Provider should only be called once
      expect(mockSearchProvider.search).toHaveBeenCalledTimes(1);
    });
  });

  describe('concurrency control', () => {
    it('should limit concurrent searches', async () => {
      const limitedSeeker = new KnowledgeSeeker(
        { maxConcurrentSearches: 1 },
        [mockSearchProvider],
        mockProcessor
      );

      const query: KnowledgeQuery = {
        id: 'concurrency-test',
        query: 'test',
        priority: QueryPriority.MEDIUM
      };

      mockSearchProvider.search.mockImplementation(
        () => new Promise(resolve => setTimeout(() => resolve([]), 100))
      );
      mockProcessor.processResults.mockResolvedValue([]);

      // Start multiple searches simultaneously
      const promises = [
        limitedSeeker.search(query),
        limitedSeeker.search({ ...query, id: 'query-2' }),
        limitedSeeker.search({ ...query, id: 'query-3' })
      ];

      const results = await Promise.all(promises);

      // All should complete eventually
      expect(results).toHaveLength(3);
      expect(results.every(r => r.queryId)).toBe(true);

      limitedSeeker.destroy();
    });

    it('should reject searches when at concurrency limit', async () => {
      const limitedSeeker = new KnowledgeSeeker(
        { maxConcurrentSearches: 0 },
        [mockSearchProvider],
        mockProcessor
      );

      const query: KnowledgeQuery = {
        id: 'limit-test',
        query: 'test',
        priority: QueryPriority.MEDIUM
      };

      await expect(limitedSeeker.search(query))
        .rejects
        .toThrow(KnowledgeSeekerError);

      limitedSeeker.destroy();
    });
  });

  describe('health check', () => {
    it('should report healthy when providers are available', async () => {
      const health = await knowledgeSeeker.healthCheck();

      expect(health.healthy).toBe(true);
      expect(health.details.totalProviders).toBe(1);
      expect(health.details.availableProviders).toBe(1);
    });

    it('should report unhealthy when no providers available', async () => {
      mockSearchProvider.isAvailable.mockResolvedValue(false);

      const health = await knowledgeSeeker.healthCheck();

      expect(health.healthy).toBe(false);
      expect(health.details.availableProviders).toBe(0);
    });
  });

  describe('cache management', () => {
    let cacheEnabledSeeker: KnowledgeSeeker;

    beforeEach(() => {
      cacheEnabledSeeker = new KnowledgeSeeker(
        { cacheEnabled: true },
        [mockSearchProvider],
        mockProcessor
      );
    });

    afterEach(() => {
      cacheEnabledSeeker.destroy();
    });

    it('should provide cache statistics', () => {
      const stats = cacheEnabledSeeker.getCacheStats();

      expect(stats).toHaveProperty('size');
      expect(stats).toHaveProperty('hitRate');
      expect(stats).toHaveProperty('totalAccesses');
    });

    it('should clear cache', () => {
      cacheEnabledSeeker.clearCache();
      const stats = cacheEnabledSeeker.getCacheStats();

      expect(stats.size).toBe(0);
    });
  });

  describe('active searches tracking', () => {
    it('should track active searches', async () => {
      const query: KnowledgeQuery = {
        id: 'tracking-test',
        query: 'test',
        priority: QueryPriority.MEDIUM
      };

      mockSearchProvider.search.mockImplementation(
        () => new Promise(resolve => setTimeout(() => resolve([]), 50))
      );
      mockProcessor.processResults.mockResolvedValue([]);

      const searchPromise = knowledgeSeeker.search(query);

      expect(knowledgeSeeker.getActiveSearches()).toContain('tracking-test');

      await searchPromise;

      expect(knowledgeSeeker.getActiveSearches()).not.toContain('tracking-test');
    });
  });

  describe('error handling', () => {
    it('should handle and classify different error types', async () => {
      const testCases = [
        {
          error: new KnowledgeSeekerError('Test error', KnowledgeSeekerErrorCode.INVALID_QUERY, 'test-id'),
          expectedCode: KnowledgeSeekerErrorCode.INVALID_QUERY
        },
        {
          error: new Error('Network failure'),
          expectedCode: KnowledgeSeekerErrorCode.NETWORK_ERROR
        }
      ];

      for (const testCase of testCases) {
        mockSearchProvider.search.mockRejectedValue(testCase.error);

        const query: KnowledgeQuery = {
          id: 'error-test',
          query: 'test',
          priority: QueryPriority.MEDIUM
        };

        const response = await knowledgeSeeker.search(query);

        expect(response.error).toBeDefined();
        expect(response.results).toHaveLength(0);
      }
    });
  });
});
