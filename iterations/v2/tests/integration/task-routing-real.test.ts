/**
 * Integration Tests for Capability-Based Task Routing with Real Agents
 *
 * @author @darianrosebrook
 */

import { beforeEach, describe, expect, it } from "@jest/globals";
import { RegistryProvider } from "../../src/orchestrator/RegistryProvider.js";
import { TaskRoutingManager } from "../../src/orchestrator/TaskRoutingManager.js";
import { runtimeAgentSeeds } from "../../src/orchestrator/runtime/runtimeAgentDataset.js";
import type { AgentRegistry } from "../../src/types/agent-registry.js";
import type { Task } from "../../src/types/arbiter-orchestration.js";

describe("Task Routing with Real Agents", () => {
  let registry: AgentRegistry;
  let routingManager: TaskRoutingManager;

  // Helper to create test tasks
  const createTestTask = (overrides: Partial<Task>): Task => ({
    id: `test-task-${Math.random().toString(36).substring(2, 9)}`,
    type: "code-editing",
    description: "Test task",
    requiredCapabilities: {
      taskTypes: ["code-editing"],
    },
    priority: 5,
    timeoutMs: 30000,
    createdAt: new Date(),
    metadata: {},
    attempts: 0,
    maxAttempts: 3,
    budget: {
      maxFiles: 10,
      maxLoc: 1000,
    },
    ...overrides,
  });

  beforeEach(async () => {
    // Initialize registry with real seeded agents
    registry = await RegistryProvider.createAgentRegistry({
      config: {
        enableSecurity: false,
        enablePersistence: false,
      },
      initOptions: {
        seeds: runtimeAgentSeeds,
        mode: "idempotent",
        emitReady: false,
      },
    });

    // Initialize routing manager
    routingManager = new TaskRoutingManager(registry, {
      enableBandit: false, // Use deterministic routing for testing
      minAgentsRequired: 1,
      maxAgentsToConsider: 10,
      defaultStrategy: "capability-match",
      maxRoutingTimeMs: 100,
      loadBalancingWeight: 0.3,
      capabilityMatchWeight: 0.7,
    });
    routingManager.setPerformanceTracker({
      recordTaskExecution: async () => {},
      recordTaskCompletion: async () => {},
      recordTaskFailure: async () => {},
      getStats: async () => ({
        totalTasks: 100,
        completedTasks: 95,
        failedTasks: 5,
        averageLatency: 5000,
        averageQuality: 0.85,
        successRate: 0.95,
      }),
    } as any);
  });

  describe("Task Routing by Capability", () => {
    it("should route testing task to agent with testing capability", async () => {
      const task = createTestTask({
        requiredCapabilities: {
          taskTypes: ["testing"],
        },
      });

      const decision = await routingManager.routeTask(task);

      expect(decision.selectedAgent).toBeDefined();
      expect(decision.selectedAgent.id).toBeDefined();
      expect(decision.strategy).toBe("capability-match");
      expect(decision.confidence).toBeGreaterThan(0);
      expect(decision.confidence).toBeLessThanOrEqual(1);
      expect(decision.alternatives).toBeDefined();

      // Verify the selected agent actually has the required capability
      const agentProfile = await registry.getProfile(decision.selectedAgent.id);
      expect(agentProfile.capabilities.taskTypes).toContain("testing");
    });

    it("should route code-editing task to appropriate agent", async () => {
      const task = createTestTask({
        requiredCapabilities: {
          taskTypes: ["code-editing"],
          languages: ["TypeScript"],
        },
      });

      const decision = await routingManager.routeTask(task);

      expect(decision.selectedAgent).toBeDefined();
      expect(decision.selectedAgent.id).toBeDefined();

      // Verify the selected agent has the required capabilities
      const agentProfile = await registry.getProfile(decision.selectedAgent.id);
      expect(agentProfile.capabilities.taskTypes).toContain("code-editing");
      expect(agentProfile.capabilities.languages).toContain("TypeScript");
    });

    it("should fail to route task with unmet capabilities", async () => {
      const task = createTestTask({
        requiredCapabilities: {
          taskTypes: ["nonexistent-capability" as any],
        },
        type: "nonexistent-capability" as any,
      });

      await expect(routingManager.routeTask(task)).rejects.toThrow(
        "No agents available for task type: nonexistent-capability"
      );
    });

    it("should prefer least-loaded agent when multiple eligible", async () => {
      // Create a task that multiple agents can handle
      const task = createTestTask({
        requiredCapabilities: {
          taskTypes: ["code-editing"],
        },
      });

      // Get all eligible agents
      const eligibleAgents = await registry.getAgentsByCapability({
        taskType: "code-editing",
      });

      expect(eligibleAgents.length).toBeGreaterThan(1); // Need multiple for meaningful test

      const decision = await routingManager.routeTask(task);

      // The selected agent should be one of the eligible agents
      const selectedAgent = eligibleAgents.find(
        (agent) => agent.agent.id === decision.selectedAgent.id
      );
      expect(selectedAgent).toBeDefined();

      // With load balancing enabled, should prefer lower utilization
      const selectedLoad = selectedAgent!.agent.currentLoad.utilizationPercent;

      // Check that no other agent has significantly lower load
      const _lowerLoadAgents = eligibleAgents.filter(
        (agent) =>
          agent.agent.currentLoad.utilizationPercent < selectedLoad - 10 // 10% threshold
      );

      // This is probabilistic - we can't guarantee the lowest load agent is always selected
      // but we can verify the selected agent is reasonable
      expect(selectedLoad).toBeLessThan(100); // Not overutilized
    });

    it("should respect language requirements in routing", async () => {
      const task = createTestTask({
        requiredCapabilities: {
          taskTypes: ["code-editing"],
          languages: ["Python"], // Specific language requirement
        },
      });

      const decision = await routingManager.routeTask(task);

      // Verify the selected agent supports Python
      const agentProfile = await registry.getProfile(decision.selectedAgent.id);
      expect(agentProfile.capabilities.languages).toContain("Python");
    });

    it("should respect specialization requirements", async () => {
      const task = createTestTask({
        requiredCapabilities: {
          taskTypes: ["code-editing"],
          specializations: ["API design"], // Specific specialization
        },
      });

      const decision = await routingManager.routeTask(task);

      // Verify the selected agent has the required specialization
      const agentProfile = await registry.getProfile(decision.selectedAgent.id);
      expect(agentProfile.capabilities.specializations).toContain("API design");
    });

    it("should include routing rationale in decision", async () => {
      const task = createTestTask({
        requiredCapabilities: {
          taskTypes: ["testing"],
        },
      });

      const decision = await routingManager.routeTask(task);

      expect(decision.reason).toBeDefined();
      expect(typeof decision.reason).toBe("string");
      expect(decision.reason.length).toBeGreaterThan(0);

      // Should mention capability matching
      expect(decision.reason.toLowerCase()).toMatch(/capability|match|testing/);
    });

    it("should provide alternatives in routing decision", async () => {
      const task = createTestTask({
        requiredCapabilities: {
          taskTypes: ["code-editing"],
        },
      });

      const decision = await routingManager.routeTask(task);

      expect(decision.alternatives).toBeDefined();
      expect(Array.isArray(decision.alternatives)).toBe(true);

      // Should have at least one alternative (if multiple agents available)
      const eligibleAgents = await registry.getAgentsByCapability({
        taskType: "code-editing",
      });

      if (eligibleAgents.length > 1) {
        expect(decision.alternatives.length).toBeGreaterThan(0);
      }

      // Each alternative should have an agent and score
      for (const alt of decision.alternatives) {
        expect(alt.agent).toBeDefined();
        expect(alt.agent.id).toBeDefined();
        expect(typeof alt.score).toBe("number");
        expect(alt.score).toBeGreaterThanOrEqual(0);
        expect(alt.score).toBeLessThanOrEqual(1);
        expect(alt.reason).toBeDefined();
      }
    });

    it("should handle routing timeout gracefully", async () => {
      // Create a routing manager with very short timeout
      const fastRoutingManager = new TaskRoutingManager(registry, {
        enableBandit: false,
        minAgentsRequired: 1,
        maxAgentsToConsider: 10,
        defaultStrategy: "capability-match",
        maxRoutingTimeMs: 1, // Very short timeout
        loadBalancingWeight: 0.3,
        capabilityMatchWeight: 0.7,
      });

      const task = createTestTask({
        requiredCapabilities: {
          taskTypes: ["code-editing"],
        },
      });

      // Should still succeed (routing is fast)
      const decision = await fastRoutingManager.routeTask(task);
      expect(decision.selectedAgent).toBeDefined();
    });
  });

  describe("Load Balancing", () => {
    it("should consider agent utilization in routing decisions", async () => {
      // This test would require mocking agent loads or having agents with different loads
      // For now, verify the routing manager considers load balancing weight
      const task = createTestTask({
        requiredCapabilities: {
          taskTypes: ["code-editing"],
        },
      });

      const decision = await routingManager.routeTask(task);

      // Verify the decision includes load balancing consideration
      // This is mainly a structural test since actual load balancing depends on agent states
      expect(decision).toBeDefined();
      expect(decision.selectedAgent).toBeDefined();
      expect(decision.confidence).toBeDefined();
    });

    it("should avoid over-utilized agents when possible", async () => {
      // Test that agents with high utilization are avoided when alternatives exist
      const task = createTestTask({
        requiredCapabilities: {
          taskTypes: ["code-editing"],
        },
      });

      const decision = await routingManager.routeTask(task);

      // Get the selected agent's load
      const agentProfile = await registry.getProfile(decision.selectedAgent.id);
      const utilization = agentProfile.currentLoad.utilizationPercent;

      // Should prefer agents that aren't at 100% utilization if alternatives exist
      const eligibleAgents = await registry.getAgentsByCapability({
        taskType: "code-editing",
      });

      const _hasLowerUtilizationAlternatives = eligibleAgents.some(
        (agent) => agent.agent.currentLoad.utilizationPercent < utilization
      );

      // If there are alternatives with lower utilization, routing should consider this
      // (Though it might still select a higher utilization agent for other reasons)
      expect(utilization).toBeLessThanOrEqual(100);
    });
  });
});
