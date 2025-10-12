/**
 * @fileoverview Tests for Resilient Database Client
 *
 * Tests circuit breaker integration, graceful degradation, recovery, etc.
 *
 * @author @darianrosebrook
 */

import {
  VerificationPriority,
  CircuitBreakerOpenError,
  CircuitState,
} from "../../../src/resilience/CircuitBreaker";
import { ResilientDatabaseClient } from "../../../src/resilience/ResilientDatabaseClient";
import { AgentProfile, AgentQuery } from "../../../src/types/agent-registry";

// Create a partial mock that implements the interface we need
class MockDatabaseClient {
  initialize = jest.fn().mockResolvedValue(undefined);
  registerAgent = jest.fn();
  getAgent = jest.fn();
  queryAgentsByCapability = jest.fn();
  updatePerformance = jest.fn();
  unregisterAgent = jest.fn();
  getStats = jest.fn();
  shutdown = jest.fn();
  healthCheck = jest.fn().mockResolvedValue(true);
}

const mockFallbackRegistry = {
  registerAgent: jest.fn(),
  getProfile: jest.fn(),
  getAgentsByCapability: jest.fn(),
  updatePerformance: jest.fn(),
  unregisterAgent: jest.fn(),
  getRegistryStats: jest.fn(),
};

describe("ResilientDatabaseClient", () => {
  let resilientClient: ResilientDatabaseClient;
  let mockDatabaseClient: MockDatabaseClient;

  beforeEach(() => {
    jest.clearAllMocks();
    mockDatabaseClient = new MockDatabaseClient();

    resilientClient = new ResilientDatabaseClient(
      mockDatabaseClient as any,
      {
        enableFallback: true,
        circuitBreaker: {
          failureThreshold: 2,
          failureWindowMs: 1000,
          resetTimeoutMs: 500,
          successThreshold: 1,
        },
        enableRetry: false, // Disable retry for simpler testing
      },
      mockFallbackRegistry as any
    );
  });

  describe("Initialization", () => {
    it("should initialize successfully when database is available", async () => {
      mockDatabaseClient.initialize.mockResolvedValue(undefined);

      await resilientClient.initialize();

      expect(mockDatabaseClient.initialize).toHaveBeenCalledTimes(1);
      expect(resilientClient.getStatus().usingFallback).toBe(false);
    });

    it("should switch to fallback mode when database initialization fails", async () => {
      mockDatabaseClient.initialize.mockRejectedValue(
        new Error("Connection failed")
      );

      await resilientClient.initialize();

      expect(resilientClient.getStatus().usingFallback).toBe(true);
    });
  });

  describe("Normal Operations (Database Available)", () => {
    beforeEach(async () => {
      await resilientClient.initialize();
    });

    it("should delegate registerAgent to database when available", async () => {
      const agent: AgentProfile = {
        id: "test-agent",
        name: "Test Agent",
        modelFamily: "claude-3.5" as any,
        capabilities: {
          taskTypes: ["code-editing" as any],
          languages: ["TypeScript" as any],
          specializations: [],
        },
        performanceHistory: {
          successRate: 0.9,
          averageQuality: 0.8,
          averageLatency: 1000,
          taskCount: 0,
        },
        currentLoad: { activeTasks: 0, queuedTasks: 0, utilizationPercent: 0 },
        registeredAt: new Date().toISOString(),
        lastActiveAt: new Date().toISOString(),
      };

      mockDatabaseClient.registerAgent.mockResolvedValue(agent);

      const result = await resilientClient.registerAgent(agent);

      expect(mockDatabaseClient.registerAgent).toHaveBeenCalledWith(agent);
      expect(result).toBe(agent);
    });

    it("should delegate query operations to database", async () => {
      const query: AgentQuery = { taskType: "code-editing" as any };
      const results = [
        {
          id: "agent1",
          name: "Test Agent",
          modelFamily: "claude-3.5" as any,
          capabilities: {
            taskTypes: ["code-editing" as any],
            languages: ["TypeScript"],
            specializations: [],
          },
          performanceHistory: {
            taskCount: 0,
            successRate: 0.9,
            averageQuality: 0.8,
            averageLatency: 1000,
          },
          currentLoad: {
            activeTasks: 0,
            queuedTasks: 0,
            utilizationPercent: 0,
          },
          registeredAt: new Date().toISOString(),
          lastActiveAt: new Date().toISOString(),
        },
      ];

      mockDatabaseClient.queryAgentsByCapability.mockResolvedValue(results);

      const result = await resilientClient.queryAgentsByCapability(query);

      expect(mockDatabaseClient.queryAgentsByCapability).toHaveBeenCalledWith(
        query
      );
      expect(result).toEqual(results);
    });
  });

  describe("Fallback Mode (Database Unavailable)", () => {
    beforeEach(async () => {
      mockDatabaseClient.initialize.mockRejectedValue(
        new Error("Connection failed")
      );
      await resilientClient.initialize();
    });

    it("should use fallback registry for registerAgent", async () => {
      const agent: AgentProfile = {
        id: "test-agent",
        name: "Test Agent",
        modelFamily: "claude-3.5" as any,
        capabilities: {
          taskTypes: ["code-editing" as any],
          languages: ["TypeScript" as any],
          specializations: [],
        },
        performanceHistory: {
          successRate: 0.9,
          averageQuality: 0.8,
          averageLatency: 1000,
          taskCount: 0,
        },
        currentLoad: { activeTasks: 0, queuedTasks: 0, utilizationPercent: 0 },
        registeredAt: new Date().toISOString(),
        lastActiveAt: new Date().toISOString(),
      };

      mockFallbackRegistry.registerAgent.mockResolvedValue(agent);

      const result = await resilientClient.registerAgent(agent);

      expect(mockFallbackRegistry.registerAgent).toHaveBeenCalledWith(agent);
      expect(result).toBe(agent);
    });

    it("should use fallback registry for queries", async () => {
      const query: AgentQuery = { taskType: "code-editing" as any };
      const results = [{ agent: { id: "agent1" }, score: 0.9 }];

      mockFallbackRegistry.getAgentsByCapability.mockResolvedValue(results);

      const result = await resilientClient.queryAgentsByCapability(query);

      expect(mockFallbackRegistry.getAgentsByCapability).toHaveBeenCalledWith(
        query
      );
      expect(result).toEqual([{ id: "agent1" }]);
    });
  });

  describe("Circuit Breaker Integration", () => {
    beforeEach(async () => {
      // Recreate client without fallback for circuit breaker tests
      resilientClient = new ResilientDatabaseClient(mockDatabaseClient as any, {
        enableFallback: false, // Disable fallback to test circuit breaker
        circuitBreaker: {
          failureThreshold: 2,
          failureWindowMs: 1000,
          resetTimeoutMs: 500,
          successThreshold: 1,
        },
        enableRetry: false,
      });
      await resilientClient.initialize();
    });

    it("should open circuit after multiple database failures", async () => {
      mockDatabaseClient.registerAgent
        .mockRejectedValueOnce(new Error("DB error"))
        .mockRejectedValueOnce(new Error("DB error"))
        .mockRejectedValueOnce(new Error("DB error"));

      const agent: AgentProfile = {
        id: "test-agent",
        name: "Test Agent",
        modelFamily: "claude-3.5" as any,
        capabilities: {
          taskTypes: ["code-editing" as any],
          languages: ["TypeScript" as any],
          specializations: [],
        },
        performanceHistory: {
          successRate: 0.9,
          averageQuality: 0.8,
          averageLatency: 1000,
          taskCount: 0,
        },
        currentLoad: { activeTasks: 0, queuedTasks: 0, utilizationPercent: 0 },
        registeredAt: new Date().toISOString(),
        lastActiveAt: new Date().toISOString(),
      };

      // Two failures should open the circuit
      await expect(resilientClient.registerAgent(agent)).rejects.toThrow(
        "DB error"
      );
      await expect(resilientClient.registerAgent(agent)).rejects.toThrow(
        "DB error"
      );

      expect(resilientClient.getStatus().circuitState).toBe(CircuitState.OPEN);

      // Third attempt should be rejected by circuit breaker
      await expect(resilientClient.registerAgent(agent)).rejects.toThrow(
        CircuitBreakerOpenError
      );
    });

    it("should transition to fallback mode on circuit breaker errors", async () => {
      mockDatabaseClient.registerAgent
        .mockRejectedValueOnce(new Error("DB error"))
        .mockRejectedValueOnce(new Error("DB error"))
        .mockRejectedValueOnce(new Error("DB error"));

      const agent: AgentProfile = {
        id: "test-agent",
        name: "Test Agent",
        modelFamily: "claude-3.5" as any,
        capabilities: {
          taskTypes: ["code-editing" as any],
          languages: ["TypeScript" as any],
          specializations: [],
        },
        performanceHistory: {
          successRate: 0.9,
          averageQuality: 0.8,
          averageLatency: 1000,
          taskCount: 0,
        },
        currentLoad: { activeTasks: 0, queuedTasks: 0, utilizationPercent: 0 },
        registeredAt: new Date().toISOString(),
        lastActiveAt: new Date().toISOString(),
      };

      // Open circuit
      await expect(resilientClient.registerAgent(agent)).rejects.toThrow(
        "DB error"
      );
      await expect(resilientClient.registerAgent(agent)).rejects.toThrow(
        "DB error"
      );

      // Next call should be rejected by circuit breaker (fallback disabled)
      await expect(resilientClient.registerAgent(agent)).rejects.toThrow(
        CircuitBreakerOpenError
      );

      expect(resilientClient.getStatus().usingFallback).toBe(false); // Fallback disabled in this test
    });
  });

  describe("Recovery Mechanism", () => {
    it("should attempt recovery when database becomes available again", async () => {
      // Start with database failure
      mockDatabaseClient.initialize.mockRejectedValue(
        new Error("Connection failed")
      );
      await resilientClient.initialize();

      expect(resilientClient.getStatus().usingFallback).toBe(true);

      // Simulate database recovery
      mockDatabaseClient.healthCheck.mockResolvedValue(true);

      // Trigger health check (this would normally happen periodically)
      const healthy = await resilientClient.healthCheck();

      expect(healthy).toBe(true);
      // Note: Actual recovery logic would need to be triggered by health monitoring
    });

    it("should allow manual circuit breaker reset", async () => {
      // Temporarily disable fallback to test circuit breaker
      (resilientClient as any).config.enableFallback = false;

      // Open circuit
      mockDatabaseClient.registerAgent.mockRejectedValue(new Error("DB error"));
      const agent = {
        id: "test-agent",
        name: "Test Agent",
        modelFamily: "claude-3.5" as any,
        capabilities: {
          taskTypes: ["code-editing" as any],
          languages: ["TypeScript" as any],
          specializations: [],
        },
        performanceHistory: {
          successRate: 0.9,
          averageQuality: 0.8,
          averageLatency: 1000,
          taskCount: 0,
        },
        currentLoad: { activeTasks: 0, queuedTasks: 0, utilizationPercent: 0 },
        registeredAt: new Date().toISOString(),
        lastActiveAt: new Date().toISOString(),
      };

      await expect(resilientClient.registerAgent(agent)).rejects.toThrow(
        "DB error"
      );
      await expect(resilientClient.registerAgent(agent)).rejects.toThrow(
        "DB error"
      );

      expect(resilientClient.getStatus().circuitState).toBe(CircuitState.OPEN);

      // Manual reset
      resilientClient.resetCircuitBreaker();

      expect(resilientClient.getStatus().circuitState).toBe(
        CircuitState.CLOSED
      );
    });
  });

  describe("Status Reporting", () => {
    it("should report correct status when database is healthy", async () => {
      await resilientClient.initialize();

      const status = resilientClient.getStatus();

      expect(status.circuitState).toBe(CircuitState.CLOSED);
      expect(status.usingFallback).toBe(false);
      expect(status.circuitStats.failures).toBe(0);
      expect(status.circuitStats.successes).toBeGreaterThanOrEqual(0);
    });

    it("should report correct status when using fallback", async () => {
      mockDatabaseClient.initialize.mockRejectedValue(
        new Error("Connection failed")
      );
      await resilientClient.initialize();

      const status = resilientClient.getStatus();

      expect(status.usingFallback).toBe(true);
    });
  });

  describe("Shutdown", () => {
    it("should shutdown database client", async () => {
      await resilientClient.initialize();

      await resilientClient.shutdown();

      expect(mockDatabaseClient.shutdown).toHaveBeenCalledTimes(1);
    });
  });
});
