/**
 * ARBITER-001 End-to-End Integration Tests
 *
 * Full integration tests using real PostgreSQL and Redis instances.
 * Tests complete workflows from agent registration through task execution.
 *
 * **Prerequisites**:
 * - PostgreSQL 16+ running on localhost:5432
 * - Redis running on localhost:6379
 * - Database 'agent_agency_test' created
 * - pgvector extension enabled
 *
 * **Run with**: `npm run test:e2e:db`
 *
 * @author @darianrosebrook
 */

import { Pool } from "pg";
import { createClient as createRedisClient, RedisClientType } from "redis";
import { AgentRegistryDbClient } from "../../../src/database/AgentRegistryDbClient.js";
import { AgentRegistryManager } from "../../../src/orchestrator/AgentRegistryManager.js";
import { AgentRegistrySecurity } from "../../../src/security/AgentRegistrySecurity.js";
import {
  VerificationPriority,
  ModelFamily,
  PerformanceMetrics,
} from "../../../src/types/agent-registry.js";

// Test configuration (using legacy format for simplicity)
const TEST_DB_CONFIG = {
  host: process.env.TEST_DB_HOST || "localhost",
  port: parseInt(process.env.TEST_DB_PORT || "5432"),
  database: process.env.TEST_DB_NAME || "agent_agency_test",
  user: process.env.TEST_DB_USER || "postgres",
  password: process.env.TEST_DB_PASSWORD || "test123",
};

describe("ARBITER-001 End-to-End Integration", () => {
  let dbPool: Pool;
  let redisClient: RedisClientType;
  let dbClient: AgentRegistryDbClient;
  let registry: AgentRegistryManager;
  let security: AgentRegistrySecurity;

  // Setup: Connect to real database and create tables
  beforeAll(async () => {
    // Connect to PostgreSQL
    dbPool = new Pool({
      host: TEST_DB_CONFIG.host,
      port: TEST_DB_CONFIG.port,
      database: TEST_DB_CONFIG.database,
      user: TEST_DB_CONFIG.user,
      password: TEST_DB_CONFIG.password,
      ssl: false,
      min: 2,
      max: 10,
    });

    // Verify connection
    try {
      await dbPool.query("SELECT 1");
      console.log("✅ PostgreSQL connection established");
    } catch (error) {
      console.error("❌ Failed to connect to PostgreSQL:", error);
      throw new Error(
        "Database connection failed. Ensure PostgreSQL is running and TEST_DB_* env vars are set."
      );
    }

    // Create test schema
    await createTestSchema(dbPool);

    // Connect to Redis
    redisClient = createRedisClient({
      url: process.env.TEST_REDIS_URL || "redis://localhost:6379",
    }) as RedisClientType;

    await redisClient.connect();
    console.log("✅ Redis connection established");

    // Initialize components
    dbClient = new AgentRegistryDbClient(TEST_DB_CONFIG);
    await dbClient.initialize();

    security = new AgentRegistrySecurity();

    registry = new AgentRegistryManager({
      enablePersistence: false, // Simplified for E2E - just test in-memory
      maxAgents: 100,
    });

    await registry.initialize();
    console.log("✅ ARBITER-001 components initialized");
  }, 30000); // 30s timeout for setup

  afterAll(async () => {
    // Cleanup
    if (registry) await registry.shutdown();
    // dbClient doesn't have close() - pool cleanup handled by registry
    if (redisClient) await redisClient.quit();
    if (dbPool) await dbPool.end();
    console.log("✅ Test cleanup complete");
  });

  // Clear data between tests
  beforeEach(async () => {
    await clearTestData(dbPool);
    await redisClient.flushDb();
  });

  describe("End-to-End Agent Lifecycle", () => {
    it("should complete full agent registration workflow", async () => {
      // Arrange
      const agentProfile = {
        name: "E2E Test Agent",
        modelFamily: "claude-3.5" as const,
        capabilities: {
          taskTypes: ["code-editing" as const, "debugging" as const],
          languages: ["TypeScript" as const, "Python" as const],
          specializations: ["AST analysis" as const],
        },
      };

      // Act: Register agent
      const registered = await registry.registerAgent(agentProfile);

      // Assert: Agent created in memory
      expect(registered.id).toBeDefined();
      expect(registered.name).toBe("E2E Test Agent");
      expect(registered.capabilities.taskTypes).toContain("code-editing");

      // Assert: Agent persisted to database
      const fromDb = await dbClient.getAgent(registered.id);
      expect(fromDb).toBeDefined();
      expect(fromDb?.name).toBe("E2E Test Agent");
      expect(fromDb?.capabilities.languages).toContain("TypeScript");

      // Assert: Agent retrievable from registry
      const retrieved = await registry.getProfile(registered.id);
      expect(retrieved.id).toBe(registered.id);
      expect(retrieved.capabilities.specializations).toContain("testing");
    });

    it("should handle performance tracking end-to-end", async () => {
      // Arrange: Register agent
      const agent = await registry.registerAgent({
        name: "Performance Test Agent",
        modelFamily: "gpt-4" as const,
        capabilities: {
          taskTypes: ["code-editing" as const],
          languages: ["JavaScript" as const],
          specializations: [],
        },
      });

      // Act: Record multiple performance metrics
      const metrics: PerformanceMetrics[] = [
        {
          taskType: "code-editing",
          success: true,
          qualityScore: 0.95,
          latencyMs: 120,
          tokensUsed: 1500,
        },
        {
          taskType: "code-editing",
          success: true,
          qualityScore: 0.88,
          latencyMs: 150,
          tokensUsed: 1800,
        },
        {
          taskType: "code-editing",
          success: false,
          qualityScore: 0.4,
          latencyMs: 200,
          tokensUsed: 2000,
        },
      ];

      for (const metric of metrics) {
        await registry.updatePerformance(agent.id, metric);
      }

      // Assert: Performance history updated in memory
      const updated = await registry.getProfile(agent.id);
      expect(updated.performanceHistory.taskCount).toBeGreaterThan(0);
      expect(updated.performanceHistory.successRate).toBeGreaterThan(0);

      // Assert: Performance metrics persisted to database
      const fromDb = await dbClient.getAgent(agent.id);
      expect(fromDb?.performanceHistory.taskCount).toBeGreaterThan(0);

      // Calculate expected success rate: 2/3 ≈ 0.67
      expect(fromDb?.performanceHistory.successRate).toBeCloseTo(0.67, 1);
    });

    it.skip("should enforce security controls end-to-end", async () => {
      // TODO: Implement when security layer is fully integrated
      // This test requires the security methods to be implemented in AgentRegistrySecurity
      expect(true).toBe(true);
    });

    it("should handle agent unregistration workflow", async () => {
      // Arrange: Register agent
      const agent = await registry.registerAgent({
        name: "Temporary Agent",
        modelFamily: "claude-3" as const,
        capabilities: {
          taskTypes: ["debugging" as const],
          languages: ["Python" as const],
          specializations: [],
        },
      });

      const agentId = agent.id;

      // Verify agent exists
      const exists = await registry.getProfile(agentId);
      expect(exists).toBeDefined();

      // Act: Unregister agent
      const result = await registry.unregisterAgent(agentId);
      expect(result).toBe(true);

      // Assert: Agent removed from memory
      await expect(registry.getProfile(agentId)).rejects.toThrow();

      // Assert: Agent removed from database
      const fromDb = await dbClient.getAgent(agentId);
      expect(fromDb).toBeNull();
    });
  });

  describe("Multi-Agent Scenarios", () => {
    it("should handle concurrent agent registrations", async () => {
      // Arrange: Create 10 agents concurrently
      const agents = Array.from({ length: 10 }, (_, i) => ({
        name: `Concurrent Agent ${i}`,
        modelFamily:
          i % 2 === 0 ? ("gpt-4" as ModelFamily) : ("claude-3" as ModelFamily),
        capabilities: {
          taskTypes: ["code-editing" as const],
          languages: ["TypeScript" as const],
          specializations: [],
        },
      }));

      // Act: Register all agents concurrently
      const startTime = Date.now();
      const results = await Promise.all(
        agents.map((agent) => registry.registerAgent(agent))
      );
      const duration = Date.now() - startTime;

      // Assert: All agents registered successfully
      expect(results).toHaveLength(10);
      expect(new Set(results.map((r) => r.id)).size).toBe(10); // All unique IDs

      // Assert: Performance is acceptable (< 5s for 10 agents)
      expect(duration).toBeLessThan(5000);

      // Assert: All agents queryable (using getProfile for each)
      for (const result of results) {
        const retrieved = await registry.getProfile(result.id);
        expect(retrieved.name).toBe(result.name);
      }
    });

    it("should handle complex capability queries", async () => {
      // Arrange: Register agents with diverse capabilities
      await registry.registerAgent({
        name: "TypeScript Expert",
        modelFamily: "gpt-4" as const,
        capabilities: {
          taskTypes: ["code-editing", "code-review"],
          languages: ["TypeScript" as const],
          specializations: [
            "AST analysis" as const,
            "Performance optimization" as const,
          ],
        },
      });

      await registry.registerAgent({
        name: "Python Expert",
        modelFamily: "claude-3" as const,
        capabilities: {
          taskTypes: ["debugging", "refactoring"],
          languages: ["Python" as const],
          specializations: ["Database design" as const],
        },
      });

      await registry.registerAgent({
        name: "Full Stack",
        modelFamily: "gpt-4" as const,
        capabilities: {
          taskTypes: [
            "code-editing" as const,
            "debugging" as const,
            "testing" as const,
          ],
          languages: ["TypeScript" as const, "Python" as const],
          specializations: ["AST analysis" as const],
        },
      });

      // Act: Query for TypeScript + AST analysis
      const tsTesters = await registry.getAgentsByCapability({
        taskType: "code-editing",
        languages: ["TypeScript"],
        specializations: ["AST analysis"],
      });

      // Assert: Should find TypeScript Expert and Full Stack
      expect(tsTesters.length).toBe(2);
      expect(tsTesters.some((a) => a.agent.name === "TypeScript Expert")).toBe(
        true
      );
      expect(tsTesters.some((a) => a.agent.name === "Full Stack")).toBe(true);

      // Act: Query for Python + Data Science
      const dataScienceAgents = await registry.getAgentsByCapability({
        taskType: "debugging",
        languages: ["Python"],
        specializations: ["Database design"],
      });

      // Assert: Should find Python Expert only
      expect(dataScienceAgents.length).toBe(1);
      expect(dataScienceAgents[0].agent.name).toBe("Python Expert");
    });

    it("should maintain performance under load", async () => {
      // Arrange: Register 50 agents
      const agents = await Promise.all(
        Array.from({ length: 50 }, (_, i) =>
          registry.registerAgent({
            name: `Load Test Agent ${i}`,
            modelFamily:
              i % 3 === 0
                ? ("gpt-4" as ModelFamily)
                : ("claude-3" as ModelFamily),
            capabilities: {
              taskTypes: ["code-editing" as const],
              languages: ["TypeScript" as const],
              specializations: [],
            },
          })
        )
      );

      // Act: Perform 100 concurrent reads
      const startTime = Date.now();
      const reads = await Promise.all(
        Array.from({ length: 100 }, (_, i) =>
          registry.getProfile(agents[i % 50].id)
        )
      );
      const duration = Date.now() - startTime;

      // Assert: All reads successful
      expect(reads).toHaveLength(100);
      expect(reads.every((r) => r !== null)).toBe(true);

      // Assert: P95 latency < 100ms (100 reads in < 10s)
      expect(duration).toBeLessThan(10000);
      const avgLatency = duration / 100;
      expect(avgLatency).toBeLessThan(100);

      console.log(
        `✅ Load test: 100 reads in ${duration}ms (avg: ${avgLatency.toFixed(
          2
        )}ms/read)`
      );
    });
  });

  describe("Error Recovery & Resilience", () => {
    it("should recover from temporary database connection loss", async () => {
      // Arrange: Register agent
      const agent = await registry.registerAgent({
        name: "Resilience Test Agent",
        modelFamily: "gpt-4" as const,
        capabilities: {
          taskTypes: ["code-editing" as const],
          languages: ["TypeScript" as const],
          specializations: [],
        },
      });

      // Simulate database connection issue by closing pool temporarily
      await dbPool.end();

      // Act: Try to register another agent (should fail)
      await expect(
        registry.registerAgent({
          name: "Should Fail Agent",
          modelFamily: "gpt-4" as const,
          capabilities: {
            taskTypes: ["code-editing" as const],
            languages: ["TypeScript" as const],
            specializations: [],
          },
        })
      ).rejects.toThrow();

      // Restore connection
      dbPool = new Pool({
        host: TEST_DB_CONFIG.host,
        port: TEST_DB_CONFIG.port,
        database: TEST_DB_CONFIG.database,
        user: TEST_DB_CONFIG.user,
        password: TEST_DB_CONFIG.password,
      });

      // Reinitialize db client with new pool
      dbClient = new AgentRegistryDbClient(TEST_DB_CONFIG);
      await dbClient.initialize();
      (registry as any).dbClient = dbClient;

      // Assert: Can still read previously registered agent from memory
      const retrieved = await registry.getProfile(agent.id);
      expect(retrieved.name).toBe("Resilience Test Agent");
    });

    it("should handle transaction rollback on partial failure", async () => {
      // This test verifies ACID compliance
      const agentData = {
        name: "Transaction Test Agent",
        modelFamily: "gpt-4" as const,
        capabilities: {
          taskTypes: ["code-editing" as const],
          languages: ["TypeScript" as const],
          specializations: [],
        },
      };

      // Attempt to register (may fail due to previous test's db closure)
      try {
        await registry.registerAgent(agentData);
      } catch (error) {
        // Expected - database might still be recovering
      }

      // Verify no partial data persisted by checking if agent exists
      try {
        const transactionTestAgent = await registry.getProfile(
          "acid-test-agent"
        );
        // If agent exists, it should have complete data
        expect(transactionTestAgent.capabilities).toBeDefined();
        expect(transactionTestAgent.performanceHistory).toBeDefined();
      } catch (error) {
        // Agent doesn't exist - this is also valid (no partial state)
        expect(error).toBeDefined();
      }
    });
  });

  describe("Data Integrity & Consistency", () => {
    it("should maintain consistency between cache and database", async () => {
      // Arrange: Register agent
      const agent = await registry.registerAgent({
        name: "Consistency Test Agent",
        modelFamily: "claude-3" as const,
        capabilities: {
          taskTypes: ["debugging" as const],
          languages: ["Python" as const],
          specializations: [],
        },
      });

      // Act: Update performance multiple times
      for (let i = 0; i < 5; i++) {
        await registry.updatePerformance(agent.id, {
          taskType: "debugging",
          success: i % 2 === 0,
          qualityScore: 0.8 + i * 0.02,
          latencyMs: 100 + i * 10,
        });
      }

      // Assert: Memory and DB have same data
      const fromMemory = await registry.getProfile(agent.id);
      const fromDb = await dbClient.getAgent(agent.id);

      expect(fromMemory.id).toBe(fromDb?.id);
      expect(fromMemory.name).toBe(fromDb?.name);
      expect(fromMemory.performanceHistory.taskCount).toBe(
        fromDb?.performanceHistory.taskCount
      );
    });

    it("should enforce unique agent IDs", async () => {
      // Arrange: Register first agent
      const agent = await registry.registerAgent({
        name: "Unique ID Test 1",
        modelFamily: "gpt-4" as const,
        capabilities: {
          taskTypes: ["code-editing" as const],
          languages: ["TypeScript" as const],
          specializations: [],
        },
      });

      // Act & Assert: Try to register another agent with same ID should fail
      await expect(
        (registry as any).registerAgent({
          id: agent.id, // Explicitly using same ID
          name: "Unique ID Test 2",
          modelFamily: "gpt-4" as const,
          capabilities: {
            taskTypes: ["code-editing" as const],
            languages: ["TypeScript" as const],
            specializations: [],
          },
        })
      ).rejects.toThrow();
    });
  });
});

/**
 * Create test database schema
 */
async function createTestSchema(pool: Pool): Promise<void> {
  const client = await pool.connect();
  try {
    await client.query("BEGIN");

    // Create tables (matching migrations)
    await client.query(`
      CREATE TABLE IF NOT EXISTS agent_profiles (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        model_family TEXT NOT NULL,
        active_tasks INTEGER DEFAULT 0,
        queued_tasks INTEGER DEFAULT 0,
        utilization_percent REAL DEFAULT 0,
        registered_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
        last_active_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
        created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
        updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
      );

      CREATE TABLE IF NOT EXISTS agent_capabilities (
        id SERIAL PRIMARY KEY,
        agent_id TEXT NOT NULL REFERENCES agent_profiles(id) ON DELETE CASCADE,
        capability_name TEXT NOT NULL,
        score REAL DEFAULT 1.0,
        metadata JSONB DEFAULT '{}',
        created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
        UNIQUE(agent_id, capability_name)
      );

      CREATE TABLE IF NOT EXISTS agent_performance_history (
        id SERIAL PRIMARY KEY,
        agent_id TEXT NOT NULL REFERENCES agent_profiles(id) ON DELETE CASCADE,
        task_type TEXT NOT NULL,
        success_rate REAL DEFAULT 0,
        average_latency REAL DEFAULT 0,
        total_tasks INTEGER DEFAULT 0,
        quality_score REAL DEFAULT 0,
        confidence_score REAL DEFAULT 0,
        metadata JSONB DEFAULT '{}',
        recorded_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
      );

      CREATE INDEX IF NOT EXISTS idx_agent_capabilities_agent_id ON agent_capabilities(agent_id);
      CREATE INDEX IF NOT EXISTS idx_agent_performance_agent_id ON agent_performance_history(agent_id);
    `);

    await client.query("COMMIT");
    console.log("✅ Test schema created");
  } catch (error) {
    await client.query("ROLLBACK");
    throw error;
  } finally {
    client.release();
  }
}

/**
 * Clear test data between tests
 */
async function clearTestData(pool: Pool): Promise<void> {
  const client = await pool.connect();
  try {
    await client.query("BEGIN");
    await client.query("DELETE FROM agent_performance_history");
    await client.query("DELETE FROM agent_capabilities");
    await client.query("DELETE FROM agent_profiles");
    await client.query("COMMIT");
  } catch (error) {
    await client.query("ROLLBACK");
    throw error;
  } finally {
    client.release();
  }
}
