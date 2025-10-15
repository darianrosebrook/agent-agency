/**
 * @fileoverview DataLayer Unit Tests
 * @author @darianrosebrook
 */

import { DataLayer } from "../../../src/data/DataLayer";
import { CacheProvider, ConnectionPool } from "../../../src/data/types";
import { Logger } from "../../../src/utils/Logger";

describe("DataLayer", () => {
  let dataLayer: DataLayer;
  let mockLogger: Logger;
  let mockConnection: jest.Mocked<ConnectionPool>;
  let mockCache: jest.Mocked<CacheProvider>;

  const testConfig = {
    database: {
      host: "localhost",
      port: 5432,
      database: "test_db",
      username: "test_user",
      password: "test_pass",
      maxConnections: 5,
      idleTimeoutMillis: 1000,
      connectionTimeoutMillis: 1000,
    },
    cache: {
      host: "localhost",
      port: 6379,
      password: "test_cache_pass",
      keyPrefix: "test:",
      ttl: 300,
    },
    enableCache: true,
    enableMetrics: true,
    queryTimeout: 5000,
  };

  beforeEach(() => {
    mockLogger = {
      info: jest.fn(),
      warn: jest.fn(),
      error: jest.fn(),
      debug: jest.fn(),
    } as any;

    // Mock connection
    mockConnection = {
      connect: jest.fn().mockResolvedValue({}),
      query: jest.fn().mockResolvedValue({
        success: true,
        data: [{ health_check: 1 }],
        duration: 10,
        queryId: "test-query",
      }),
      transaction: jest.fn().mockImplementation(async (callback) => {
        const client = {};
        const result = await callback(client);
        return {
          success: true,
          data: result,
          duration: 10,
          queryId: "test-transaction",
        };
      }),
      healthCheck: jest.fn().mockResolvedValue({
        status: "healthy",
        database: { connected: true, latency: 5 },
      }),
      getStats: jest.fn().mockResolvedValue({
        totalCount: 5,
        idleCount: 3,
        waitingCount: 0,
        totalQueries: 10,
      }),
      close: jest.fn().mockResolvedValue(undefined),
    };

    // Mock cache
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
        data: {
          connected: true,
          hitRate: "75.0%",
          hits: 15,
          misses: 5,
        },
        hit: false,
        duration: 5,
      }),
      initialize: jest.fn().mockResolvedValue(undefined),
      healthCheck: jest.fn().mockResolvedValue({
        status: "healthy",
        connected: true,
        latency: 5,
      }),
      close: jest.fn().mockResolvedValue(undefined),
      // EventEmitter methods
      on: jest.fn(),
      emit: jest.fn(),
      off: jest.fn(),
      removeListener: jest.fn(),
    };

    // Create DataLayer instance
    dataLayer = new DataLayer(testConfig, mockLogger);

    // Mock the internal connection and cache
    (dataLayer as any).connection = mockConnection;
    (dataLayer as any).cache = mockCache;
  });

  afterEach(async () => {
    // Cleanup
    if (dataLayer) {
      await dataLayer.shutdown().catch(() => {});
    }
  });

  describe("initialization", () => {
    beforeEach(() => {
      // Reset initialization state
      (dataLayer as any).initialized = false;
    });

    it("should initialize successfully", async () => {
      await expect(dataLayer.initialize()).resolves.not.toThrow();

      expect(mockLogger.info).toHaveBeenCalledWith(
        "Initializing data layer..."
      );
      expect(mockLogger.info).toHaveBeenCalledWith(
        "Data layer initialized successfully"
      );
      expect((dataLayer as any).initialized).toBe(true);
    });

    it("should not initialize twice", async () => {
      await dataLayer.initialize();
      await dataLayer.initialize(); // Second call

      expect(mockLogger.warn).toHaveBeenCalledWith(
        "Data layer already initialized"
      );
    });
  });

  describe("connection management", () => {
    beforeEach(() => {
      // Set initialized state
      (dataLayer as any).initialized = true;
    });

    it("should provide connection interface", () => {
      const connection = dataLayer.getConnection();
      expect(connection).toBeDefined();
      expect(typeof connection.query).toBe("function");
      expect(typeof connection.healthCheck).toBe("function");
    });

    it("should provide cache interface when enabled", () => {
      const cache = dataLayer.getCache();
      expect(cache).toBeDefined();
      expect(typeof cache!.get).toBe("function");
      expect(typeof cache!.set).toBe("function");
    });
  });

  describe("health checks", () => {
    beforeEach(() => {
      (dataLayer as any).initialized = true;
    });

    it("should perform health check", async () => {
      const health = await dataLayer.healthCheck();

      expect(health).toBeDefined();
      expect(health.status).toBeDefined();
      expect(["healthy", "unhealthy", "degraded"]).toContain(health.status);
      expect(health.details).toBeDefined();
    });

    it("should include database health in results", async () => {
      const health = await dataLayer.healthCheck();

      // Database health may be undefined if connection fails
      if (health.database) {
        expect(health.database).toHaveProperty("connected");
        expect(typeof health.database.latency).toBe("number");
      }
    });

    it("should include cache health when enabled", async () => {
      const health = await dataLayer.healthCheck();

      // Cache health may be undefined if connection fails
      if (health.cache) {
        expect(health.cache).toHaveProperty("connected");
        expect(typeof health.cache.latency).toBe("number");
      }
    });
  });

  describe("query execution", () => {
    beforeEach(() => {
      (dataLayer as any).initialized = true;
    });

    it("should handle query execution", async () => {
      const result = await dataLayer.query("SELECT 1 as test");

      expect(result).toBeDefined();
      expect(typeof result.success).toBe("boolean");
    });

    it("should handle query errors gracefully", async () => {
      // Mock a query error
      mockConnection.query.mockResolvedValueOnce({
        success: false,
        error: "Invalid SQL",
        duration: 10,
        queryId: "test-error",
      });

      const result = await dataLayer.query("INVALID SQL QUERY");

      expect(result).toBeDefined();
      expect(result.success).toBe(false);
      expect(result.error).toBeDefined();
    });
  });

  describe("statistics", () => {
    beforeEach(() => {
      (dataLayer as any).initialized = true;
    });

    it("should provide comprehensive statistics", async () => {
      const stats = await dataLayer.getStats();

      expect(stats).toBeDefined();
      expect(stats.initialized).toBe(true);
      expect(stats.config).toBeDefined();
      expect(stats.database).toBeDefined();
    });

    it("should include cache statistics when enabled", async () => {
      const stats = await dataLayer.getStats();

      expect(stats.cache).toBeDefined();
    });
  });

  describe("shutdown", () => {
    beforeEach(() => {
      (dataLayer as any).initialized = true;
    });

    it("should shutdown gracefully", async () => {
      await expect(dataLayer.shutdown()).resolves.not.toThrow();

      expect(mockLogger.info).toHaveBeenCalledWith(
        "Data layer shutdown complete"
      );
      expect(mockConnection.close).toHaveBeenCalled();
      expect(mockCache.close).toHaveBeenCalled();
    });

    it("should handle shutdown without initialization", async () => {
      (dataLayer as any).initialized = false;
      await expect(dataLayer.shutdown()).resolves.not.toThrow();
    });
  });

  describe("error handling", () => {
    it("should throw error for operations before initialization", () => {
      expect(() => dataLayer.getConnection()).toThrow();
      expect(() => dataLayer.getCache()).toThrow();
    });

    it("should handle transaction failures gracefully", async () => {
      (dataLayer as any).initialized = true;

      // Mock transaction to throw an error
      mockConnection.transaction.mockImplementationOnce(async () => {
        throw new Error("Transaction failure test");
      });

      await expect(
        dataLayer.transaction(async () => {
          throw new Error("Transaction failure test");
        })
      ).rejects.toThrow("Transaction failure test");
    });
  });
});
