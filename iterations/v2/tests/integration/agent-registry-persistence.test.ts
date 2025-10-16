/**
 * Agent Registry Persistence Integration Tests
 *
 * Tests database persistence and retrieval for ARBITER-001.
 * Validates ACID compliance, data integrity, and recovery scenarios.
 *
 * @author @darianrosebrook
 */

import { AgentRegistryDbClient } from "../../src/database/AgentRegistryDbClient.js";
import { AgentRegistryManager } from "../../src/orchestrator/AgentRegistryManager.js";
import {
  RegistryError,
  RegistryErrorType,
} from "../../src/types/agent-registry.js";
import { DatabaseTestUtils } from "../test-utils.js";

describe("Agent Registry Persistence Integration", () => {
  let registry: AgentRegistryManager;
  let dbClient: AgentRegistryDbClient;

  beforeAll(async () => {
    // Setup database for testing
    DatabaseTestUtils.setupMockDatabase();
    DatabaseTestUtils.mockSuccessfulOperations();

    // Create registry with database persistence enabled
    registry = new AgentRegistryManager({
      enablePersistence: true,
      database: {
        host: "localhost",
        port: 5432,
        database: "agent_agency_test",
        username: "postgres",
        password: "test123",
      },
      maxAgents: 100,
    });

    // Get direct access to database client for testing
    dbClient = (registry as any).dbClient;

    // Initialize registry (this should initialize the database)
    await registry.initialize();
  });

  afterAll(async () => {
    if (registry) {
      await registry.shutdown();
    }
    jest.clearAllMocks();
  });

  beforeEach(() => {
    // Reset mocks before each test
    jest.clearAllMocks();
  });

  describe("Database Persistence", () => {
    it("should persist agent registration to database", async () => {
      // Arrange
      const agentData = {
        id: "test-agent-1",
        name: "Test Agent",
        modelFamily: "gpt-4" as const,
        capabilities: {
          taskTypes: ["code-editing" as const, "debugging" as const],
          languages: ["TypeScript" as const],
          specializations: [],
        },
      };

      // Mock successful database registration
      const mockRegister = jest
        .spyOn(dbClient, "registerAgent")
        .mockResolvedValue("test-agent-1");

      // Act
      const result = await registry.registerAgent(agentData);

      // Assert
      expect(result.id).toBe("test-agent-1");
      expect(result.name).toBe("Test Agent");
      expect(mockRegister).toHaveBeenCalledWith(
        expect.objectContaining({
          id: "test-agent-1",
          name: "Test Agent",
          modelFamily: "gpt-4",
        })
      );
    });

    it("should rollback in-memory storage on database failure", async () => {
      // Arrange
      const agentData = {
        id: "test-agent-fail",
        name: "Test Agent Fail",
        modelFamily: "gpt-4" as const,
        capabilities: {
          taskTypes: [],
          languages: [],
          specializations: [],
        },
      };

      // Mock database failure
      const _mockRegister = jest
        .spyOn(dbClient, "registerAgent")
        .mockRejectedValue(new Error("Database connection failed"));

      // Act & Assert
      await expect(registry.registerAgent(agentData)).rejects.toThrow(
        RegistryError
      );
      await expect(registry.registerAgent(agentData)).rejects.toMatchObject({
        type: RegistryErrorType.DATABASE_ERROR,
      });

      // Verify agent was not stored in memory
      await expect(registry.getProfile("test-agent-fail")).rejects.toThrow(
        RegistryError
      );
    });

    it("should load agent from database when not in memory cache", async () => {
      // Arrange
      const agentId = "cached-agent";
      const mockAgent = {
        id: agentId,
        name: "Cached Agent",
        modelFamily: "gpt-4" as const,
        capabilities: {
          taskTypes: ["testing" as const],
          languages: ["TypeScript" as const],
          specializations: [],
        },
        performanceHistory: {
          successRate: 0.85,
          averageLatency: 150,
          averageQuality: 0.8,
          taskCount: 10,
        },
        currentLoad: { activeTasks: 0, queuedTasks: 0, utilizationPercent: 0 },
        registeredAt: new Date().toISOString(),
        lastActiveAt: new Date().toISOString(),
        createdAt: new Date(),
        updatedAt: new Date(),
      };

      // Mock database retrieval
      const mockGet = jest
        .spyOn(dbClient, "getAgent")
        .mockResolvedValue(mockAgent);

      // Act
      const result = await registry.getProfile(agentId);

      // Assert
      expect(result.id).toBe(agentId);
      expect(result.name).toBe("Cached Agent");
      expect(mockGet).toHaveBeenCalledWith(agentId);

      // Verify agent is now cached in memory
      const cachedResult = await registry.getProfile(agentId);
      expect(cachedResult).toEqual(result);
      // getAgent should only be called once (first time)
      expect(mockGet).toHaveBeenCalledTimes(1);
    });

    it("should record performance metrics to database", async () => {
      // Arrange
      const agentId = "perf-agent";
      const metrics = {
        success: true,
        qualityScore: 0.9,
        latencyMs: 150,
        taskType: "code-editing" as const,
        taskId: "task-123",
      };

      // Mock agent exists in memory
      const mockAgent = {
        id: agentId,
        name: "Performance Agent",
        modelFamily: "gpt-4",
        capabilities: [],
        performanceHistory: [],
        registeredAt: new Date().toISOString(),
        lastActiveAt: new Date().toISOString(),
        activeTasks: 0,
        queuedTasks: 0,
        utilizationPercent: 0,
        createdAt: new Date(),
        updatedAt: new Date(),
      };

      (registry as any).agents.set(agentId, mockAgent);

      // Mock performance recording
      const mockRecord = jest
        .spyOn(dbClient, "recordPerformance")
        .mockResolvedValue(undefined);

      // Act
      await registry.updatePerformance(agentId, metrics);

      // Assert
      expect(mockRecord).toHaveBeenCalledWith(
        agentId,
        expect.objectContaining({
          taskType: "code-editing",
          successRate: 1.0, // true converted to 1.0
          averageLatency: 150,
          qualityScore: 0.9,
          confidenceScore: 0.85,
          metadata: expect.objectContaining({
            taskId: "task-123",
          }),
        })
      );
    });

    it("should continue operation when performance recording fails", async () => {
      // Arrange
      const agentId = "perf-fail-agent";
      const metrics = {
        success: false,
        qualityScore: 0.5,
        latencyMs: 200,
        taskType: "debugging" as const,
      };

      // Mock agent exists
      const mockAgent = {
        id: agentId,
        name: "Performance Fail Agent",
        modelFamily: "gpt-4",
        capabilities: [],
        performanceHistory: [],
        registeredAt: new Date().toISOString(),
        lastActiveAt: new Date().toISOString(),
        activeTasks: 0,
        queuedTasks: 0,
        utilizationPercent: 0,
        createdAt: new Date(),
        updatedAt: new Date(),
      };

      (registry as any).agents.set(agentId, mockAgent);

      // Mock database failure for performance recording
      const _mockRecord = jest
        .spyOn(dbClient, "recordPerformance")
        .mockRejectedValue(new Error("Database write failed"));

      // Mock console.error to verify it's called
      const consoleErrorSpy = jest.spyOn(console, "error").mockImplementation();

      // Act
      const result = await registry.updatePerformance(agentId, metrics);

      // Assert
      expect(result.id).toBe(agentId);
      expect(consoleErrorSpy).toHaveBeenCalledWith(
        expect.stringContaining("Failed to record performance to database"),
        expect.any(Error)
      );

      // Cleanup
      consoleErrorSpy.mockRestore();
    });

    it("should load agents from database on initialization", async () => {
      // Arrange
      const mockAgents = [
        {
          id: "loaded-agent-1",
          name: "Loaded Agent 1",
          modelFamily: "gpt-4" as const,
          capabilities: {
            taskTypes: ["code-editing" as const],
            languages: ["TypeScript" as const],
            specializations: [],
          },
          performanceHistory: {
            successRate: 0.9,
            averageLatency: 100,
            averageQuality: 0.85,
            taskCount: 5,
          },
          currentLoad: {
            activeTasks: 0,
            queuedTasks: 0,
            utilizationPercent: 0,
          },
          registeredAt: new Date().toISOString(),
          lastActiveAt: new Date().toISOString(),
          createdAt: new Date(),
          updatedAt: new Date(),
        },
        {
          id: "loaded-agent-2",
          name: "Loaded Agent 2",
          modelFamily: "claude-3" as const,
          capabilities: {
            taskTypes: ["debugging" as const],
            languages: ["Python" as const],
            specializations: [],
          },
          performanceHistory: {
            successRate: 0.8,
            averageLatency: 120,
            averageQuality: 0.75,
            taskCount: 3,
          },
          currentLoad: {
            activeTasks: 0,
            queuedTasks: 0,
            utilizationPercent: 0,
          },
          registeredAt: new Date().toISOString(),
          lastActiveAt: new Date().toISOString(),
          createdAt: new Date(),
          updatedAt: new Date(),
        },
      ];

      // Mock database query
      const mockQuery = jest.spyOn(dbClient, "queryAgents").mockResolvedValue(
        mockAgents.map((agent) => ({
          agent,
          matchScore: 0.9,
          matchReason: "Mock match",
        }))
      );

      // Create new registry instance to test initialization
      const newRegistry = new AgentRegistryManager({
        enablePersistence: true,
        database: {
          host: "localhost",
          port: 5432,
          database: "agent_agency_test",
          username: "postgres",
          password: "test123",
        },
        maxAgents: 100,
      });

      // Act
      await newRegistry.initialize();

      // Assert
      expect(mockQuery).toHaveBeenCalledWith({
        limit: 100,
        offset: 0,
      });

      // Verify agents were loaded into memory
      const agent1 = await newRegistry.getProfile("loaded-agent-1");
      const agent2 = await newRegistry.getProfile("loaded-agent-2");

      expect(agent1.name).toBe("Loaded Agent 1");
      expect(agent2.name).toBe("Loaded Agent 2");

      // Cleanup
      await newRegistry.shutdown();
    });
  });

  describe("Database Error Handling", () => {
    it("should handle database connection failures gracefully", async () => {
      // Arrange
      const agentData = {
        id: "connection-fail-agent",
        name: "Connection Fail Agent",
        modelFamily: "gpt-4" as const,
        capabilities: {
          taskTypes: [],
          languages: [],
          specializations: [],
        },
      };

      // Mock database connection failure
      const _mockRegister = jest
        .spyOn(dbClient, "registerAgent")
        .mockRejectedValue(new Error("Connection timeout"));

      // Act & Assert
      await expect(registry.registerAgent(agentData)).rejects.toThrow(
        RegistryError
      );
      await expect(registry.registerAgent(agentData)).rejects.toMatchObject({
        type: RegistryErrorType.DATABASE_ERROR,
        message: expect.stringContaining("Failed to persist agent to database"),
      });
    });

    it("should handle database retrieval failures gracefully", async () => {
      // Arrange
      const agentId = "retrieve-fail-agent";

      // Mock database retrieval failure
      const _mockGet = jest
        .spyOn(dbClient, "getAgent")
        .mockRejectedValue(new Error("Query failed"));

      // Act & Assert
      await expect(registry.getProfile(agentId)).rejects.toThrow(RegistryError);
      await expect(registry.getProfile(agentId)).rejects.toMatchObject({
        type: RegistryErrorType.DATABASE_ERROR,
        message: expect.stringContaining(
          "Failed to retrieve agent from database"
        ),
      });
    });
  });

  describe("ACID Compliance", () => {
    it("should maintain atomicity - all operations succeed or all fail", async () => {
      // Arrange
      const agentData = {
        id: "acid-test-agent",
        name: "ACID Test Agent",
        modelFamily: "gpt-4" as const,
        capabilities: {
          taskTypes: ["code-editing" as const, "testing" as const],
          languages: ["TypeScript" as const],
          specializations: [],
        },
      };

      // Mock database failure after partial operations
      let callCount = 0;
      const _mockRegister = jest
        .spyOn(dbClient, "registerAgent")
        .mockImplementation(async () => {
          callCount++;
          if (callCount === 1) {
            throw new Error("Simulated database failure");
          }
          return "acid-test-agent";
        });

      // Act & Assert - First attempt should fail and rollback
      await expect(registry.registerAgent(agentData)).rejects.toThrow(
        RegistryError
      );

      // Verify agent was not stored in memory (rollback)
      await expect(registry.getProfile("acid-test-agent")).rejects.toThrow(
        RegistryError
      );

      // Second attempt should succeed
      const result = await registry.registerAgent(agentData);
      expect(result.id).toBe("acid-test-agent");
    });
  });
});
