/**
 * @fileoverview MultiLevelCache Unit Tests
 * @author @darianrosebrook
 */

import { MultiLevelCache } from '../../../src/data/cache/MultiLevelCache';
import { Logger } from '../../../src/utils/Logger';

describe('MultiLevelCache', () => {
  let cache: MultiLevelCache;
  let mockLogger: Logger;
  let mockRedisCache: any;

  const cacheConfig = {
    host: 'localhost',
    port: 6379,
    password: 'test_pass',
    keyPrefix: 'test:',
    ttl: 300,
    l1MaxSize: 1024 * 1024, // 1MB
    l1MaxEntries: 100,
    promotionThreshold: 2,
    demotionThreshold: 30000, // 30 seconds
    enableMetrics: true
  };

  beforeEach(() => {
    mockLogger = {
      info: jest.fn(),
      warn: jest.fn(),
      error: jest.fn(),
      debug: jest.fn(),
    } as any;

    mockRedisCache = {
      get: jest.fn(),
      set: jest.fn(),
      delete: jest.fn(),
      exists: jest.fn(),
      clear: jest.fn(),
      getStats: jest.fn(),
      close: jest.fn(),
      initialize: jest.fn().mockResolvedValue(undefined),
      on: jest.fn(),
      emit: jest.fn()
    };

    // Mock the RedisCache constructor
    jest.doMock('../../../src/data/cache/RedisCache', () => ({
      RedisCache: jest.fn().mockImplementation(() => mockRedisCache)
    }));

    // Create cache instance
    cache = new MultiLevelCache(cacheConfig, mockLogger);
  });

  afterEach(async () => {
    await cache.close();
    jest.clearAllMocks();
  });

  describe('initialization', () => {
    it('should initialize successfully', async () => {
      await expect(cache.initialize()).resolves.not.toThrow();
      expect(mockRedisCache.initialize).toHaveBeenCalled();
    });
  });

  describe('cache operations', () => {
    beforeEach(async () => {
      await cache.initialize();
    });

    describe('get operations', () => {
      it('should return L1 cache hit', async () => {
        const testData = { message: 'L1 hit data' };
        const key = 'test:l1_hit';

        // First set the data in L1
        await cache.set(key, testData, 300);

        // Now get it back
        const result = await cache.get(key);

        expect(result.success).toBe(true);
        expect(result.hit).toBe(true);
        expect(result.data).toEqual(testData);
      });

      it('should promote from L2 to L1 on access', async () => {
        const testData = { message: 'L2 data' };
        const key = 'test:l2_promotion';

        // Mock L2 hit
        mockRedisCache.get.mockResolvedValue({
          success: true,
          hit: true,
          data: testData,
          duration: 10
        });

        const result = await cache.get(key);

        expect(result.success).toBe(true);
        expect(result.hit).toBe(true);
        expect(result.data).toEqual(testData);
        expect(mockRedisCache.get).toHaveBeenCalledWith(key);
      });

      it('should handle cache miss', async () => {
        const key = 'test:cache_miss';

        // Mock L2 miss
        mockRedisCache.get.mockResolvedValue({
          success: true,
          hit: false,
          duration: 5
        });

        const result = await cache.get(key);

        expect(result.success).toBe(true);
        expect(result.hit).toBe(false);
        expect(result.data).toBeUndefined();
      });

      it('should handle L2 cache errors gracefully', async () => {
        const key = 'test:l2_error';

        mockRedisCache.get.mockResolvedValue({
          success: false,
          error: 'Redis connection failed',
          hit: false,
          duration: 5
        });

        const result = await cache.get(key);

        expect(result.success).toBe(false);
        expect(result.error).toBe('Redis connection failed');
      });
    });

    describe('set operations', () => {
      it('should set data in both L1 and L2 caches', async () => {
        const testData = { message: 'test data' };
        const key = 'test:set_both';

        mockRedisCache.set.mockResolvedValue({
          success: true,
          data: true,
          hit: false,
          duration: 8
        });

        const result = await cache.set(key, testData, 300);

        expect(result.success).toBe(true);
        expect(result.data).toBe(true);
        expect(mockRedisCache.set).toHaveBeenCalledWith(key, testData, 300);
      });

      it('should handle large objects by not caching in L1', async () => {
        // Create a large object (> 10% of L1 cache size)
        const largeData = { data: 'x'.repeat(200000) }; // ~200KB
        const key = 'test:large_object';

        mockRedisCache.set.mockResolvedValue({
          success: true,
          data: true,
          hit: false,
          duration: 8
        });

        const result = await cache.set(key, largeData, 300);

        expect(result.success).toBe(true);
        // Should still set in L2 but not L1
        expect(mockRedisCache.set).toHaveBeenCalledWith(key, largeData, 300);
      });

      it('should handle L2 set failures', async () => {
        const testData = { message: 'test data' };
        const key = 'test:set_failure';

        mockRedisCache.set.mockResolvedValue({
          success: false,
          error: 'Redis set failed',
          hit: false,
          duration: 5
        });

        const result = await cache.set(key, testData, 300);

        expect(result.success).toBe(false);
        expect(result.error).toBe('Redis set failed');
      });
    });

    describe('promotion and demotion', () => {
      it('should promote frequently accessed items to L2', async () => {
        const testData = { message: 'frequent access data' };
        const key = 'test:promotion';

        // Set initial data
        mockRedisCache.set.mockResolvedValue({
          success: true,
          data: true,
          hit: false,
          duration: 5
        });
        await cache.set(key, testData, 300);

        // Access multiple times to trigger promotion
        for (let i = 0; i < 3; i++) {
          await cache.get(key);
        }

        // Check if promotion occurred (additional L2 set call)
        expect(mockRedisCache.set).toHaveBeenCalledTimes(2); // Initial set + promotion
      });

      it('should evict L1 entries when cache is full', async () => {
        // Set cache config to very small size for testing
        const smallCache = new MultiLevelCache({
          ...cacheConfig,
          l1MaxEntries: 2,
          l1MaxSize: 100
        }, mockLogger);

        mockRedisCache.set.mockResolvedValue({
          success: true,
          data: true,
          hit: false,
          duration: 5
        });

        // Fill the cache
        await smallCache.set('key1', { data: 'small1' }, 300);
        await smallCache.set('key2', { data: 'small2' }, 300);
        await smallCache.set('key3', { data: 'small3' }, 300); // Should trigger eviction

        // key1 should have been evicted (LRU)
        const result1 = await smallCache.get('key1');
        expect(result1.hit).toBe(false); // Should miss L1, check L2

        await smallCache.close();
      });
    });

    describe('cache statistics', () => {
      it('should provide comprehensive statistics', async () => {
        const stats = await cache.getStats();

        expect(stats.success).toBe(true);
        expect(stats.data).toHaveProperty('l1');
        expect(stats.data).toHaveProperty('l2');
        expect(stats.data).toHaveProperty('overall');
        expect(stats.data!.overall).toHaveProperty('hitRate');
        expect(stats.data!.overall).toHaveProperty('totalRequests');
      });

      it('should calculate hit rates correctly', async () => {
        // Perform some cache operations
        const testData = { message: 'stats test' };

        // Set data
        mockRedisCache.set.mockResolvedValue({
          success: true,
          data: true,
          hit: false,
          duration: 5
        });
        await cache.set('stats_key', testData, 300);

        // Get data (L1 hit)
        await cache.get('stats_key');

        // Get non-existent data (miss)
        mockRedisCache.get.mockResolvedValue({
          success: true,
          hit: false,
          duration: 5
        });
        await cache.get('non_existent');

        const stats = await cache.getStats();

        expect(stats.data!.overall.totalRequests).toBeGreaterThan(0);
        expect(typeof stats.data!.overall.hitRate).toBe('string');
      });
    });

    describe('cache clearing', () => {
      it('should clear specific patterns', async () => {
        mockRedisCache.clear.mockResolvedValue({
          success: true,
          data: 5,
          hit: false,
          duration: 10
        });

        const result = await cache.clear('test:*');

        expect(result.success).toBe(true);
        expect(result.data).toBe(5);
        expect(mockRedisCache.clear).toHaveBeenCalledWith('test:*');
      });

      it('should clear all cache when no pattern specified', async () => {
        mockRedisCache.clear.mockResolvedValue({
          success: true,
          data: 10,
          hit: false,
          duration: 15
        });

        const result = await cache.clear();

        expect(result.success).toBe(true);
        expect(mockRedisCache.clear).toHaveBeenCalledWith('*');
      });
    });
  });

  describe('error handling', () => {
    it('should handle initialization failures', async () => {
      mockRedisCache.initialize.mockRejectedValue(new Error('Redis init failed'));

      await expect(cache.initialize()).rejects.toThrow('Redis init failed');
    });

    it('should handle cache operations after close', async () => {
      await cache.close();

      await expect(cache.get('test')).rejects.toThrow();
      await expect(cache.set('test', 'data')).rejects.toThrow();
    });
  });

  describe('memory management', () => {
    it('should track memory usage', async () => {
      const data1 = { content: 'x'.repeat(1000) }; // ~1KB
      const data2 = { content: 'y'.repeat(2000) }; // ~2KB

      mockRedisCache.set.mockResolvedValue({
        success: true,
        data: true,
        hit: false,
        duration: 5
      });

      await cache.set('key1', data1, 300);
      await cache.set('key2', data2, 300);

      // Check that memory usage is tracked (internal implementation)
      // This is testing the internal memory tracking
      expect((cache as any).l1Size).toBeGreaterThan(0);
    });

    it('should clean up expired entries', async () => {
      const expiredData = { message: 'expired' };
      const key = 'test:expired';

      // Set with very short TTL
      mockRedisCache.set.mockResolvedValue({
        success: true,
        data: true,
        hit: false,
        duration: 5
      });
      await cache.set(key, expiredData, 1); // 1 second TTL

      // Wait for expiration
      await new Promise(resolve => setTimeout(resolve, 1100));

      // Trigger cleanup (normally done by interval)
      (cache as any).cleanupExpiredL1Entries();

      // Should not find expired entry in L1
      const l1Entry = (cache as any).l1Cache.get(key);
      expect(l1Entry).toBeUndefined();
    });
  });
});
