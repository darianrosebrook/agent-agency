/**
 * Agent Registry Manager Unit Tests
 *
 * @author @darianrosebrook
 * @spec ARBITER-001
 *
 * Tests for AgentRegistryManager mapping to acceptance criteria:
 * - A1: Agent registration with capability tracking
 * - A2: Query agents by capability sorted by performance
 * - A3: Performance metrics update with running average
 * - A4: Query agents filtered by utilization threshold
 * - A5: Registry backup and recovery
 */
// @ts-nocheck


import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";
import { AgentRegistryManager } from "../../../src/orchestrator/AgentRegistryManager";
import type {
  AgentProfile,
  AgentQuery,
  PerformanceMetrics,
} from "../../../src/types/agent-registry";
import {
  VerificationPriority,
  RegistryError,
  RegistryErrorType,
} from "../../../src/types/agent-registry";

describe("AgentRegistryManager", () => {
  let registry: AgentRegistryManager;

  beforeEach(() => {
    registry = new AgentRegistryManager({
      maxAgents: 10,
      enableAutoCleanup: false, // Disable for tests
      enableSecurity: false, // Disable security for unit tests
      enablePersistence: false, // Disable persistence for unit tests
    });
  });

  afterEach(async () => {
    await registry.shutdown();
  });

  describe("Agent Registration (A1)", () => {
    it("should register a new agent with capability tracking initialized", async () => {
      // GIVEN: A new agent with defined capabilities
      const newAgent: Partial<AgentProfile> = {
        id: "agent-001",
        name: "Test Agent 1",
        modelFamily: "claude-3.5",
        capabilities: {
          taskTypes: ["code-editing", "code-review"],
          languages: ["TypeScript", "JavaScript"],
          specializations: ["API design"],
        },
      };

      // WHEN: Agent registers with the registry
      const registered = await registry.registerAgent(newAgent);

      // THEN: Agent profile is created with capability tracking initialized and agent is queryable
      expect(registered.id).toBe("agent-001");
      expect(registered.name).toBe("Test Agent 1");
      expect(registered.capabilities.taskTypes).toContain("code-editing");
      expect(registered.performanceHistory).toBeDefined();
      expect(registered.performanceHistory.taskCount).toBe(0);
      expect(registered.currentLoad).toBeDefined();
      expect(registered.currentLoad.activeTasks).toBe(0);
      expect(registered.registeredAt).toBeDefined();
      expect(registered.lastActiveAt).toBeDefined();

      // Verify agent is queryable
      const retrieved = await registry.getProfile("agent-001");
      expect(retrieved.id).toBe("agent-001");
    });

    it("should reject duplicate agent registration", async () => {
      // GIVEN: An agent already registered
      await registry.registerAgent({
        id: "agent-001",
        name: "Test Agent",
        modelFamily: "claude-3.5",
        capabilities: {
          taskTypes: ["code-editing"],
          languages: ["TypeScript"],
          specializations: [],
        },
      });

      // WHEN: Attempting to register same agent again
      // THEN: Should throw AGENT_ALREADY_EXISTS error
      await expect(
        registry.registerAgent({
          id: "agent-001",
          name: "Duplicate Agent",
          modelFamily: "gpt-4",
          capabilities: {
            taskTypes: ["research"],
            languages: ["Python"],
            specializations: [],
          },
        })
      ).rejects.toThrow(RegistryError);

      await expect(
        registry.registerAgent({
          id: "agent-001",
          name: "Duplicate Agent",
          modelFamily: "gpt-4",
          capabilities: {
            taskTypes: ["research"],
            languages: ["Python"],
            specializations: [],
          },
        })
      ).rejects.toMatchObject({
        type: RegistryErrorType.AGENT_ALREADY_EXISTS,
      });
    });

    it("should reject registration when registry is full", async () => {
      // GIVEN: Registry at capacity (maxAgents = 10)
      for (let i = 0; i < 10; i++) {
        await registry.registerAgent({
          id: `agent-${i}`,
          name: `Agent ${i}`,
          modelFamily: "claude-3.5",
          capabilities: {
            taskTypes: ["code-editing"],
            languages: ["TypeScript"],
            specializations: [],
          },
        });
      }

      // WHEN: Attempting to register one more agent
      // THEN: Should throw REGISTRY_FULL error
      await expect(
        registry.registerAgent({
          id: "agent-overflow",
          name: "Overflow Agent",
          modelFamily: "gpt-4",
          capabilities: {
            taskTypes: ["research"],
            languages: ["Python"],
            specializations: [],
          },
        })
      ).rejects.toMatchObject({
        type: RegistryErrorType.REGISTRY_FULL,
      });
    });

    it("should validate required fields during registration", async () => {
      // GIVEN: Invalid agent data (missing required fields)
      // WHEN: Attempting to register
      // THEN: Should throw INVALID_AGENT_DATA error

      await expect(
        registry.registerAgent({
          id: "agent-001",
          // Missing name
          modelFamily: "claude-3.5",
          capabilities: {
            taskTypes: ["code-editing"],
            languages: ["TypeScript"],
            specializations: [],
          },
        } as Partial<AgentProfile>)
      ).rejects.toThrow();

      await expect(
        registry.registerAgent({
          id: "agent-001",
          name: "Test Agent",
          // Missing modelFamily
          capabilities: {
            taskTypes: ["code-editing"],
            languages: ["TypeScript"],
            specializations: [],
          },
        } as Partial<AgentProfile>)
      ).rejects.toThrow();
    });
  });

  describe("Query by Capability (A2)", () => {
    beforeEach(async () => {
      // Set up test agents with varying capabilities and performance
      await registry.registerAgent({
        id: "agent-typescript",
        name: "TypeScript Expert",
        modelFamily: "claude-3.5",
        capabilities: {
          taskTypes: ["code-editing", "code-review"],
          languages: ["TypeScript", "JavaScript"],
          specializations: ["API design"],
        },
        performanceHistory: {
          successRate: 0.95,
          averageQuality: 0.9,
          averageLatency: 3000,
          taskCount: 100,
        },
      });

      await registry.registerAgent({
        id: "agent-python",
        name: "Python Expert",
        modelFamily: "gpt-4",
        capabilities: {
          taskTypes: ["code-editing", "testing"],
          languages: ["Python", "JavaScript"],
          specializations: ["Database design"],
        },
        performanceHistory: {
          successRate: 0.85,
          averageQuality: 0.85,
          averageLatency: 4000,
          taskCount: 50,
        },
      });

      await registry.registerAgent({
        id: "agent-rust",
        name: "Rust Expert",
        modelFamily: "claude-3",
        capabilities: {
          taskTypes: ["code-editing", "refactoring"],
          languages: ["Rust", "C++"],
          specializations: ["Performance optimization"],
        },
        performanceHistory: {
          successRate: 0.9,
          averageQuality: 0.88,
          averageLatency: 5000,
          taskCount: 75,
        },
      });
    });

    it("should return agents matching task type sorted by success rate", async () => {
      // GIVEN: Multiple agents with code-editing capability
      const query: AgentQuery = {
        taskType: "code-editing",
      };

      // WHEN: Query for agents by task type
      const results = await registry.getAgentsByCapability(query);

      // THEN: Agents matching criteria returned sorted by performance history success rate
      expect(results.length).toBe(3);
      expect(results[0].agent.id).toBe("agent-typescript"); // 0.95 success rate
      expect(results[1].agent.id).toBe("agent-rust"); // 0.90 success rate
      expect(results[2].agent.id).toBe("agent-python"); // 0.85 success rate
    });

    it("should filter agents by required languages", async () => {
      // GIVEN: Query with language requirements
      const query: AgentQuery = {
        taskType: "code-editing",
        languages: ["TypeScript"],
      };

      // WHEN: Query for agents
      const results = await registry.getAgentsByCapability(query);

      // THEN: Only agents with TypeScript capability returned
      expect(results.length).toBe(1);
      expect(results[0].agent.id).toBe("agent-typescript");
    });

    it("should filter agents by required specializations", async () => {
      // GIVEN: Query with specialization requirements
      const query: AgentQuery = {
        taskType: "code-editing",
        specializations: ["API design"],
      };

      // WHEN: Query for agents
      const results = await registry.getAgentsByCapability(query);

      // THEN: Only agents with matching specialization returned
      expect(results.length).toBe(1);
      expect(results[0].agent.id).toBe("agent-typescript");
    });

    it("should return empty array when no agents match", async () => {
      // GIVEN: Query for non-existent capability combination
      const query: AgentQuery = {
        taskType: "code-editing",
        languages: ["Go"],
      };

      // WHEN: Query for agents
      const results = await registry.getAgentsByCapability(query);

      // THEN: Empty array returned
      expect(results.length).toBe(0);
    });

    it("should include match score and reason in results", async () => {
      // GIVEN: Query for agents
      const query: AgentQuery = {
        taskType: "code-editing",
        languages: ["TypeScript"],
      };

      // WHEN: Query for agents
      const results = await registry.getAgentsByCapability(query);

      // THEN: Results include match score and explanation
      expect(results[0].matchScore).toBeGreaterThan(0);
      expect(results[0].matchScore).toBeLessThanOrEqual(1);
      expect(results[0].matchReason).toContain("code-editing");
      expect(results[0].matchReason).toContain("success rate");
    });
  });

  describe("Performance Update (A3)", () => {
    beforeEach(async () => {
      await registry.registerAgent({
        id: "agent-001",
        name: "Test Agent",
        modelFamily: "claude-3.5",
        capabilities: {
          taskTypes: ["code-editing"],
          languages: ["TypeScript"],
          specializations: [],
        },
      });
    });

    it("should update running average performance history correctly", async () => {
      // GIVEN: An agent completes a task with measurable performance metrics
      const metrics: PerformanceMetrics = {
        success: true,
        qualityScore: 0.95,
        latencyMs: 3000,
      };

      // WHEN: Performance metrics are updated for the agent
      const updated = await registry.updatePerformance("agent-001", metrics);

      // THEN: Agent's running average performance history is computed correctly and persisted
      expect(updated.performanceHistory.taskCount).toBe(1);
      expect(updated.performanceHistory.successRate).toBeGreaterThan(0.8); // From optimistic init
      expect(updated.performanceHistory.averageQuality).toBeGreaterThan(0.7);
      expect(updated.performanceHistory.averageLatency).toBeLessThan(5000);

      // Verify persistence
      const retrieved = await registry.getProfile("agent-001");
      expect(retrieved.performanceHistory.taskCount).toBe(1);
    });

    it("should handle multiple performance updates with correct running average", async () => {
      // GIVEN: Multiple task completions
      const metrics1: PerformanceMetrics = {
        success: true,
        qualityScore: 0.9,
        latencyMs: 3000,
      };

      const metrics2: PerformanceMetrics = {
        success: true,
        qualityScore: 0.8,
        latencyMs: 4000,
      };

      const metrics3: PerformanceMetrics = {
        success: false,
        qualityScore: 0.5,
        latencyMs: 6000,
      };

      // WHEN: Performance updates applied sequentially
      await registry.updatePerformance("agent-001", metrics1);
      await registry.updatePerformance("agent-001", metrics2);
      const final = await registry.updatePerformance("agent-001", metrics3);

      // THEN: Running averages computed correctly
      expect(final.performanceHistory.taskCount).toBe(3);
      // Success rate should reflect 2/3 successes (plus optimistic init influence)
      expect(final.performanceHistory.successRate).toBeLessThan(0.8);
      expect(final.performanceHistory.successRate).toBeGreaterThan(0.5);
    });

    it("should throw error when updating non-existent agent", async () => {
      // GIVEN: Non-existent agent ID
      const metrics: PerformanceMetrics = {
        success: true,
        qualityScore: 0.9,
        latencyMs: 3000,
      };

      // WHEN: Attempting to update performance
      // THEN: Should throw AGENT_NOT_FOUND error
      await expect(
        registry.updatePerformance("non-existent", metrics)
      ).rejects.toMatchObject({
        type: RegistryErrorType.AGENT_NOT_FOUND,
      });
    });

    it("should update last active timestamp", async () => {
      // GIVEN: Agent with initial timestamp
      const before = await registry.getProfile("agent-001");
      const initialTimestamp = before.lastActiveAt;

      // Wait a bit to ensure timestamp difference
      await new Promise((resolve) => setTimeout(resolve, 10));

      // WHEN: Performance updated
      const metrics: PerformanceMetrics = {
        success: true,
        qualityScore: 0.9,
        latencyMs: 3000,
      };
      await registry.updatePerformance("agent-001", metrics);

      // THEN: Last active timestamp updated
      const after = await registry.getProfile("agent-001");
      expect(new Date(after.lastActiveAt).getTime()).toBeGreaterThan(
        new Date(initialTimestamp).getTime()
      );
    });
  });

  describe("Load Filtering (A4)", () => {
    beforeEach(async () => {
      // Set up agents with varying utilization
      await registry.registerAgent({
        id: "agent-low-util",
        name: "Low Utilization Agent",
        modelFamily: "claude-3.5",
        capabilities: {
          taskTypes: ["code-editing"],
          languages: ["TypeScript"],
          specializations: [],
        },
      });

      await registry.registerAgent({
        id: "agent-high-util",
        name: "High Utilization Agent",
        modelFamily: "gpt-4",
        capabilities: {
          taskTypes: ["code-editing"],
          languages: ["TypeScript"],
          specializations: [],
        },
      });

      // Update load to create utilization difference
      await registry.updateLoad("agent-low-util", 2, 0); // 20% utilization
      await registry.updateLoad("agent-high-util", 9, 5); // 90% utilization
    });

    it("should filter agents by utilization threshold", async () => {
      // GIVEN: Agent registry contains agents with current load information
      const query: AgentQuery = {
        taskType: "code-editing",
        maxUtilization: 50, // 50% threshold
      };

      // WHEN: Query for agents filters by utilization threshold
      const results = await registry.getAgentsByCapability(query);

      // THEN: Only agents with utilization below threshold are returned
      expect(results.length).toBe(1);
      expect(results[0].agent.id).toBe("agent-low-util");
      expect(results[0].agent.currentLoad.utilizationPercent).toBeLessThan(50);
    });

    it("should update agent load correctly", async () => {
      // GIVEN: Agent with initial load
      const before = await registry.getProfile("agent-low-util");
      expect(before.currentLoad.activeTasks).toBe(2);

      // WHEN: Load updated
      await registry.updateLoad("agent-low-util", 5, 2);

      // THEN: Load state updated with correct utilization
      const after = await registry.getProfile("agent-low-util");
      expect(after.currentLoad.activeTasks).toBe(5);
      expect(after.currentLoad.queuedTasks).toBe(2);
      expect(after.currentLoad.utilizationPercent).toBe(50); // 5/10 * 100
    });
  });

  describe("Registry Statistics and Recovery (A5)", () => {
    beforeEach(async () => {
      // Set up multiple agents for statistics
      await registry.registerAgent({
        id: "agent-001",
        name: "Agent 1",
        modelFamily: "claude-3.5",
        capabilities: {
          taskTypes: ["code-editing"],
          languages: ["TypeScript"],
          specializations: [],
        },
        performanceHistory: {
          successRate: 0.9,
          averageQuality: 0.85,
          averageLatency: 3000,
          taskCount: 50,
        },
      });

      await registry.registerAgent({
        id: "agent-002",
        name: "Agent 2",
        modelFamily: "gpt-4",
        capabilities: {
          taskTypes: ["research"],
          languages: ["Python"],
          specializations: [],
        },
        performanceHistory: {
          successRate: 0.85,
          averageQuality: 0.8,
          averageLatency: 4000,
          taskCount: 30,
        },
      });

      await registry.updateLoad("agent-001", 5, 0);
      await registry.updateLoad("agent-002", 0, 0);
    });

    it("should return accurate registry statistics", async () => {
      // GIVEN: Agent registry is operational with active agents
      // WHEN: Registry statistics requested
      const stats = await registry.getStats();

      // THEN: Statistics reflect current state
      expect(stats.totalAgents).toBe(2);
      expect(stats.activeAgents).toBe(1); // agent-001 has active tasks
      expect(stats.idleAgents).toBe(1); // agent-002 is idle
      expect(stats.averageUtilization).toBeGreaterThan(0);
      expect(stats.averageSuccessRate).toBeGreaterThan(0.8);
      expect(stats.lastUpdated).toBeDefined();
    });

    it("should support agent unregistration", async () => {
      // GIVEN: Registered agents
      const beforeStats = await registry.getStats();
      expect(beforeStats.totalAgents).toBe(2);

      // WHEN: Agent is unregistered
      const removed = await registry.unregisterAgent("agent-001");

      // THEN: Agent removed from registry
      expect(removed).toBe(true);

      const afterStats = await registry.getStats();
      expect(afterStats.totalAgents).toBe(1);

      // Verify agent not found
      await expect(registry.getProfile("agent-001")).rejects.toMatchObject({
        type: RegistryErrorType.AGENT_NOT_FOUND,
      });
    });

    it("should handle unregistration of non-existent agent", async () => {
      // GIVEN: Non-existent agent ID
      // WHEN: Attempting to unregister
      const removed = await registry.unregisterAgent("non-existent");

      // THEN: Returns false without error
      expect(removed).toBe(false);
    });
  });

  describe("Performance and Concurrency", () => {
    it("should handle concurrent registration requests", async () => {
      // GIVEN: Multiple concurrent registration requests
      const registrations = Array.from({ length: 5 }, (_, i) =>
        registry.registerAgent({
          id: `agent-${i}`,
          name: `Agent ${i}`,
          modelFamily: "claude-3.5",
          capabilities: {
            taskTypes: ["code-editing"],
            languages: ["TypeScript"],
            specializations: [],
          },
        })
      );

      // WHEN: All registrations processed concurrently
      const results = await Promise.all(registrations);

      // THEN: All agents registered successfully
      expect(results).toHaveLength(5);
      results.forEach((result, i) => {
        expect(result.id).toBe(`agent-${i}`);
      });
    });

    it("should handle concurrent performance updates", async () => {
      // GIVEN: Registered agent
      await registry.registerAgent({
        id: "agent-001",
        name: "Test Agent",
        modelFamily: "claude-3.5",
        capabilities: {
          taskTypes: ["code-editing"],
          languages: ["TypeScript"],
          specializations: [],
        },
      });

      // WHEN: Multiple concurrent performance updates
      const updates = Array.from({ length: 10 }, () =>
        registry.updatePerformance("agent-001", {
          success: true,
          qualityScore: 0.9,
          latencyMs: 3000,
        })
      );

      await Promise.all(updates);

      // THEN: All updates applied correctly
      const final = await registry.getProfile("agent-001");
      expect(final.performanceHistory.taskCount).toBe(10);
    });
  });
});
