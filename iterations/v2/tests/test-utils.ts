/**
 * Test Utilities for V2
 *
 * Shared utilities for testing across unit, integration, and E2E tests.
 *
 * @author @darianrosebrook
 */

import { ConnectionPoolManager } from "@/database/ConnectionPoolManager";
import { jest } from "@jest/globals";
import type { Pool, PoolClient } from "pg";

// Database test utilities
export class DatabaseTestUtils {
  private static mockData: Map<string, any> = new Map();

  /**
   * Setup mock database for testing
   */
  static setupMockDatabase(): void {
    // Clear previous mock data
    this.mockData.clear();

    // Simple mock setup - Jest typing issues prevent complex mocks
    // Tests will use manual mocks instead
  }

  /**
   * Mock database query implementation
   */
  private static mockQuery(sql: string, params: any[]): Promise<any> {
    // Simple mock implementation for basic operations
    if (sql.includes("SELECT 1")) {
      return Promise.resolve({ rows: [{ "?column?": 1 }], rowCount: 1 });
    }

    if (sql.includes("INSERT INTO agent_profiles")) {
      const agentId = params[0];
      const name = params[1];
      const modelFamily = params[2];
      const activeTasks = params[3];
      const queuedTasks = params[4];
      const utilizationPercent = params[5];

      this.mockData.set(`agent:${agentId}`, {
        id: agentId,
        name,
        model_family: modelFamily,
        active_tasks: activeTasks,
        queued_tasks: queuedTasks,
        utilization_percent: utilizationPercent,
        registered_at: new Date().toISOString(),
        last_active_at: new Date().toISOString(),
      });
      return Promise.resolve({ rows: [{ id: agentId }], rowCount: 1 });
    }

    if (sql.includes("SELECT * FROM agent_profiles WHERE id = $1")) {
      const agentId = params[0];
      const agent = this.mockData.get(`agent:${agentId}`);
      if (agent) {
        return Promise.resolve({ rows: [agent], rowCount: 1 });
      }
      return Promise.resolve({ rows: [], rowCount: 0 });
    }

    if (sql.includes("DELETE FROM agent_profiles WHERE id = $1")) {
      const agentId = params[0];
      this.mockData.delete(`agent:${agentId}`);
      return Promise.resolve({ rowCount: 1 });
    }

    // Default mock response
    return Promise.resolve({ rows: [], rowCount: 0 });
  }

  /**
   * Mock successful database operations
   */
  static mockSuccessfulOperations(): void {
    // This will be expanded as needed for specific tests
  }

  /**
   * Mock database connection failure
   * Note: Disabled due to Jest typing issues. Tests should use manual mocks if needed.
   */
  static mockConnectionFailure(): void {
    // Implementation disabled - use manual mocks in tests
    console.warn("mockConnectionFailure is not implemented - use manual mocks");
  }

  /**
   * Mock database query failure
   * Note: Disabled due to Jest typing issues. Tests should use manual mocks if needed.
   */
  static mockQueryFailure(): void {
    // Implementation disabled - use manual mocks in tests
    console.warn("mockQueryFailure is not implemented - use manual mocks");
  }

  /**
   * Get centralized database pool for tests
   * Ensures pool is initialized before use
   */
  static getPool(): Pool {
    const manager = ConnectionPoolManager.getInstance();
    if (!manager.isInitialized()) {
      throw new Error(
        "Database pool not initialized. Ensure tests/setup.ts has run or call setupTestDatabase()"
      );
    }
    return manager.getPool();
  }

  /**
   * Setup test database with centralized pool
   * Call this in beforeAll/beforeEach if tests/setup.ts hasn't run
   */
  static async setupTestDatabase(): Promise<void> {
    const manager = ConnectionPoolManager.getInstance();

    if (!manager.isInitialized()) {
      manager.initialize({
        host: process.env.DB_HOST || "localhost",
        port: parseInt(process.env.DB_PORT || "5432", 10),
        database: process.env.DB_NAME || "agent_agency_v2_test",
        user: process.env.DB_USER || "postgres",
        password: process.env.DB_PASSWORD || "",
        min: 2,
        max: 10,
        idleTimeoutMs: 10000,
        connectionTimeoutMs: 5000,
        statementTimeoutMs: 30000,
        applicationName: "v2-arbiter-test",
      });
    }

    // Verify database is accessible
    const isHealthy = await manager.healthCheck();
    if (!isHealthy) {
      throw new Error(
        "Database health check failed. Ensure PostgreSQL is running and accessible."
      );
    }
  }

  /**
   * Cleanup test database
   * Call this in afterAll to gracefully close connections
   */
  static async cleanupTestDatabase(): Promise<void> {
    const manager = ConnectionPoolManager.getInstance();
    if (manager.isInitialized()) {
      await manager.shutdown();
    }
  }

  /**
   * Execute query with automatic tenant context (for RLS testing)
   */
  static async queryWithTenantContext<T = any>(
    tenantId: string,
    sql: string,
    params: any[] = []
  ): Promise<{ rows: T[]; rowCount: number }> {
    const manager = ConnectionPoolManager.getInstance();
    return manager.queryWithTenantContext<T>(tenantId, sql, params);
  }

  /**
   * Get client with tenant context for multi-query operations
   * IMPORTANT: Must call client.release() in finally block
   */
  static async getClientWithTenantContext(
    tenantId: string,
    context?: { userId?: string; sessionId?: string }
  ): Promise<PoolClient> {
    const manager = ConnectionPoolManager.getInstance();
    return manager.getClientWithTenantContext(tenantId, context);
  }

  /**
   * Begin transaction for test isolation
   * Returns client that must be released after test
   */
  static async beginTestTransaction(): Promise<PoolClient> {
    const pool = this.getPool();
    const client = await pool.connect();
    await client.query("BEGIN");
    return client;
  }

  /**
   * Rollback and release client (for test cleanup)
   */
  static async rollbackTestTransaction(client: PoolClient): Promise<void> {
    try {
      await client.query("ROLLBACK");
    } finally {
      client.release();
    }
  }

  /**
   * Seed test data into database
   * Useful for integration tests
   */
  static async seedTestData(data: {
    agents?: any[];
    tasks?: any[];
    tenants?: any[];
  }): Promise<void> {
    const pool = this.getPool();

    // Seed tenants
    if (data.tenants) {
      for (const tenant of data.tenants) {
        await pool.query(
          `INSERT INTO tenants (id, name, isolation_level, created_at)
           VALUES ($1, $2, $3, NOW())
           ON CONFLICT (id) DO NOTHING`,
          [tenant.id, tenant.name, tenant.isolationLevel || "project"]
        );
      }
    }

    // Seed agents
    if (data.agents) {
      for (const agent of data.agents) {
        await pool.query(
          `INSERT INTO agent_profiles (
            agent_id, name, model_family, capabilities, config,
            active_tasks, queued_tasks, utilization_percent,
            created_at, updated_at
          )
          VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
          ON CONFLICT (agent_id) DO UPDATE SET
            name = EXCLUDED.name,
            updated_at = NOW()`,
          [
            agent.id,
            agent.name,
            agent.modelFamily,
            JSON.stringify(agent.capabilities || []),
            JSON.stringify(agent.config || {}),
            agent.activeTasks || 0,
            agent.queuedTasks || 0,
            agent.utilizationPercent || 0,
          ]
        );
      }
    }

    // Seed tasks
    if (data.tasks) {
      for (const task of data.tasks) {
        await pool.query(
          `INSERT INTO tasks (
            id, description, status, priority, assigned_agent_id,
            created_at, updated_at
          )
          VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
          ON CONFLICT (id) DO NOTHING`,
          [
            task.id,
            task.description,
            task.status || "pending",
            task.priority || 5,
            task.assignedAgentId,
          ]
        );
      }
    }
  }

  /**
   * Clear test data from database
   * Be careful - this deletes data!
   */
  static async clearTestData(options: {
    agents?: boolean;
    tasks?: boolean;
    tenants?: boolean;
    all?: boolean;
  }): Promise<void> {
    const pool = this.getPool();

    if (options.all || options.tasks) {
      await pool.query("DELETE FROM tasks WHERE id LIKE 'test-%'");
    }

    if (options.all || options.agents) {
      await pool.query(
        "DELETE FROM agent_profiles WHERE agent_id LIKE 'test-%'"
      );
    }

    if (options.all || options.tenants) {
      await pool.query("DELETE FROM tenants WHERE id LIKE 'test-%'");
    }
  }
}

// Redis test utilities
export class RedisTestUtils {
  /**
   * Setup mock Redis for testing
   */
  static setupMockRedis(): void {
    // Mock redis client
    jest.mock("redis", () => ({
      createClient: jest.fn().mockReturnValue({
        connect: jest.fn(),
        disconnect: jest.fn(),
        get: jest.fn(),
        set: jest.fn(),
        del: jest.fn(),
        expire: jest.fn(),
        on: jest.fn(),
      }),
    }));
  }
}

// General test environment utilities
export class TestEnvironment {
  private static originalEnv: Record<string, string | undefined>;

  /**
   * Setup test environment
   */
  static setup(): TestEnvironment {
    // Save original environment
    this.originalEnv = { ...process.env };

    // Set test environment variables
    process.env.NODE_ENV = "test";
    process.env.DB_HOST = "localhost";
    process.env.DB_PORT = "5432";
    process.env.DB_NAME = "agent_agency_test";
    process.env.DB_USER = "postgres";
    process.env.DB_PASSWORD = "test123";
    process.env.REDIS_HOST = "localhost";
    process.env.REDIS_PORT = "6379";

    return new TestEnvironment();
  }

  /**
   * Cleanup test environment
   */
  cleanup(): void {
    // Restore original environment
    process.env = TestEnvironment.originalEnv;
  }
}

// Mock factory utilities
export class MockFactory {
  /**
   * Create mock agent profile
   */
  static createMockAgent(overrides: Partial<any> = {}): any {
    return {
      id: "mock-agent-id",
      name: "Mock Agent",
      modelFamily: "gpt-4",
      capabilities: [
        { name: "code-editing", score: 0.9 },
        { name: "debugging", score: 0.8 },
      ],
      performanceHistory: [],
      registeredAt: new Date().toISOString(),
      lastActiveAt: new Date().toISOString(),
      activeTasks: 0,
      queuedTasks: 0,
      utilizationPercent: 0,
      createdAt: new Date(),
      updatedAt: new Date(),
      ...overrides,
    };
  }

  /**
   * Create mock performance metrics
   */
  static createMockPerformanceMetrics(overrides: Partial<any> = {}): any {
    return {
      success: true,
      latency: 150,
      quality: 0.85,
      confidence: 0.9,
      taskType: "code-editing",
      taskId: "task-123",
      ...overrides,
    };
  }

  /**
   * Create mock database result
   */
  static createMockDatabaseResult(overrides: Partial<any> = {}): any {
    return {
      agents: [],
      total: 0,
      hasMore: false,
      query: { limit: 50, offset: 0 },
      ...overrides,
    };
  }
}

// Assertion helpers
export class AssertionHelpers {
  /**
   * Assert agent profile matches expected structure
   */
  static assertValidAgentProfile(profile: any): void {
    expect(profile).toHaveProperty("id");
    expect(profile).toHaveProperty("name");
    expect(profile).toHaveProperty("modelFamily");
    expect(profile).toHaveProperty("capabilities");
    expect(profile).toHaveProperty("performanceHistory");
    expect(profile).toHaveProperty("registeredAt");
    expect(profile).toHaveProperty("lastActiveAt");
    expect(profile).toHaveProperty("activeTasks");
    expect(profile).toHaveProperty("queuedTasks");
    expect(profile).toHaveProperty("utilizationPercent");
    expect(profile).toHaveProperty("createdAt");
    expect(profile).toHaveProperty("updatedAt");

    expect(Array.isArray(profile.capabilities)).toBe(true);
    expect(Array.isArray(profile.performanceHistory)).toBe(true);
    expect(typeof profile.activeTasks).toBe("number");
    expect(typeof profile.queuedTasks).toBe("number");
    expect(typeof profile.utilizationPercent).toBe("number");
  }

  /**
   * Assert performance metrics are valid
   */
  static assertValidPerformanceMetrics(metrics: any): void {
    expect(metrics).toHaveProperty("success");
    expect(metrics).toHaveProperty("latency");
    expect(metrics).toHaveProperty("quality");
    expect(metrics).toHaveProperty("confidence");

    expect(typeof metrics.success).toBe("boolean");
    expect(typeof metrics.latency).toBe("number");
    expect(typeof metrics.quality).toBe("number");
    expect(typeof metrics.confidence).toBe("number");

    if (metrics.taskType) {
      expect(typeof metrics.taskType).toBe("string");
    }
    if (metrics.taskId) {
      expect(typeof metrics.taskId).toBe("string");
    }
  }
}
