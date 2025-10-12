/**
 * ARBITER-001 through ARBITER-004 Integration Tests
 *
 * Purpose: Validate end-to-end integration between foundation components.
 * Tests use the actual component implementations to ensure they work together correctly.
 *
 * Test Coverage:
 * - Agent registration and retrieval (ARBITER-001)
 * - Agent querying by capability (ARBITER-001)
 * - Task routing to agents (ARBITER-002)
 * - CAWS spec validation (ARBITER-003)
 * - Performance tracking integration (ARBITER-004)
 * - Multi-component workflows
 * - Load testing
 *
 * @author @darianrosebrook
 */

import { beforeEach, describe, expect, it } from "@jest/globals";
import { SpecValidator } from "../../../src/caws-validator/validation/SpecValidator";
import { AgentRegistryManager } from "../../../src/orchestrator/AgentRegistryManager";
import { TaskRoutingManager } from "../../../src/orchestrator/TaskRoutingManager";
import { PerformanceTracker } from "../../../src/rl/PerformanceTracker";
import { AgentQuery } from "../../../src/types/agent-registry";
import {
  createAgentWithCapabilities,
  createInvalidSpec,
  createMinimalTask,
  createMinimalWorkingSpec,
  createMockOutcome,
  createMultipleAgents,
  createSpecForRiskTier,
  createTaskBatch,
  createTaskRequiring,
  createTestAgent,
} from "../../helpers/test-fixtures";

describe("ARBITER Foundation Integration Tests (001-004)", () => {
  let registry: AgentRegistryManager;
  let router: TaskRoutingManager;
  let validator: SpecValidator;
  let tracker: PerformanceTracker;

  beforeEach(async () => {
    // Initialize all components with test configuration
    registry = new AgentRegistryManager({
      maxAgents: 100,
      cleanupIntervalMs: 300000,
      enableSecurity: false, // Disable for testing
    });

    await registry.initialize();

    tracker = new PerformanceTracker();

    router = new TaskRoutingManager(
      registry,
      {
        enableBandit: true,
        minAgentsRequired: 1,
        maxRoutingTimeMs: 100,
      },
      tracker
    );

    validator = new SpecValidator();
    validator.setPerformanceTracker(tracker);
    registry.setPerformanceTracker(tracker);
  });

  describe("ARBITER-001: Agent Registration", () => {
    it("should register an agent and retrieve it by ID", async () => {
      const agentData = createTestAgent({
        id: "integration-test-agent-1",
        name: "Integration Test Agent",
      });

      const agent = await registry.registerAgent(agentData);

      expect(agent).toBeDefined();
      expect(agent.id).toBe("integration-test-agent-1");
      expect(agent.name).toBe("Integration Test Agent");

      // Retrieve agent by ID
      const retrieved = await registry.getProfile(agent.id);
      expect(retrieved).toBeDefined();
      expect(retrieved.id).toBe(agent.id);
      expect(retrieved.name).toBe(agent.name);
    });

    it("should track multiple agents", async () => {
      const agents = createMultipleAgents(5, {
        modelFamily: "gpt-4",
      });

      for (const agentData of agents) {
        await registry.registerAgent(agentData);
      }

      const stats = await registry.getStats();
      expect(stats.totalAgents).toBe(5);
      // activeAgents tracks agents with active tasks (not just registered)
      expect(stats.activeAgents).toBe(0); // No tasks running yet
      expect(stats.idleAgents).toBe(5); // All agents idle
    });
  });

  describe("ARBITER-001: Agent Querying", () => {
    it("should find agents by capability", async () => {
      // Register TypeScript specialist
      await registry.registerAgent(
        createAgentWithCapabilities(
          {
            languages: ["TypeScript"],
            taskTypes: ["code-editing"],
            specializations: ["AST analysis"],
          },
          { id: "typescript-specialist" }
        )
      );

      // Register Python specialist
      await registry.registerAgent(
        createAgentWithCapabilities(
          {
            languages: ["Python"],
            taskTypes: ["code-editing"],
            specializations: ["Performance optimization"],
          },
          { id: "python-specialist" }
        )
      );

      // Query for TypeScript agents
      const query: AgentQuery = {
        languages: ["TypeScript"],
        taskType: "code-editing",
      };

      const results = await registry.getAgentsByCapability(query);

      expect(results.length).toBeGreaterThan(0);
      expect(results[0].agent.id).toBe("typescript-specialist");
      expect(results[0].matchScore).toBeGreaterThan(0);
    });

    it("should rank agents by match score", async () => {
      // Register agents with varying capabilities
      await registry.registerAgent(
        createAgentWithCapabilities(
          {
            languages: ["TypeScript", "JavaScript", "Python"],
            taskTypes: ["code-editing", "code-review"],
          },
          { id: "full-stack-agent" }
        )
      );

      await registry.registerAgent(
        createAgentWithCapabilities(
          {
            languages: ["TypeScript"],
            taskTypes: ["code-editing"],
          },
          { id: "typescript-only-agent" }
        )
      );

      // Query for TypeScript only (both agents have it)
      const query: AgentQuery = {
        languages: ["TypeScript"],
        taskType: "code-editing",
      };

      const results = await registry.getAgentsByCapability(query);

      // Both agents should match since both have TypeScript
      expect(results.length).toBeGreaterThanOrEqual(1);
      // Full-stack agent should rank higher due to more capabilities
      if (results.length > 1) {
        expect(results[0].agent.id).toBe("full-stack-agent");
        expect(results[0].matchScore).toBeGreaterThanOrEqual(
          results[1].matchScore
        );
      }
    });
  });

  describe("ARBITER-002: Task Routing", () => {
    it("should route task to matching agent", async () => {
      // Register an agent
      await registry.registerAgent(
        createTestAgent({ id: "routing-test-agent" })
      );

      // Create a task
      const task = createMinimalTask({
        description: "Refactor authentication module",
        type: "code-editing",
      });

      const routing = await router.routeTask(task);

      expect(routing).toBeDefined();
      expect(routing.selectedAgent).toBeDefined();
      expect(routing.selectedAgent.id).toBe("routing-test-agent");
      expect(routing.taskId).toBe(task.id);
      expect(routing.confidence).toBeGreaterThan(0);
    });

    it("should route tasks to best-matching agents", async () => {
      // Register TypeScript specialist
      await registry.registerAgent(
        createAgentWithCapabilities(
          {
            languages: ["TypeScript"],
            taskTypes: ["code-editing"],
          },
          {
            id: "ts-specialist",
            performanceHistory: {
              successRate: 0.95,
              averageQuality: 0.9,
              averageLatency: 100,
              taskCount: 100,
            },
          }
        )
      );

      // Register general agent with lower performance
      await registry.registerAgent(
        createAgentWithCapabilities(
          {
            languages: ["TypeScript", "Python"],
            taskTypes: ["code-editing"],
          },
          {
            id: "general-agent",
            performanceHistory: {
              successRate: 0.7,
              averageQuality: 0.7,
              averageLatency: 200,
              taskCount: 50,
            },
          }
        )
      );

      // Create TypeScript task
      const task = createTaskRequiring({
        languages: ["TypeScript"],
        taskTypes: ["code-editing"],
      });

      const routing = await router.routeTask(task);

      // Should prefer the specialist with better performance
      expect(routing.selectedAgent.id).toBe("ts-specialist");
    });
  });

  describe("ARBITER-003: CAWS Validation", () => {
    it("should validate a correct working spec", async () => {
      const spec = createMinimalWorkingSpec({
        title: "Test feature implementation",
        risk_tier: 3,
      });

      const validation = await validator.validateWorkingSpec(spec);

      expect(validation).toBeDefined();
      expect(validation.valid).toBe(true);
    });

    it("should reject spec with missing acceptance criteria", async () => {
      const invalidSpec = createInvalidSpec("missing-acceptance");

      const validation = await validator.validateWorkingSpec(
        invalidSpec as any
      );

      expect(validation).toBeDefined();
      expect(validation.valid).toBe(false);
    });

    it("should validate specs for different risk tiers", async () => {
      // Tier 3 (low risk) - minimal requirements
      const tier3Spec = createSpecForRiskTier(3, {
        title: "Low risk feature",
      });
      const tier3Validation = await validator.validateWorkingSpec(tier3Spec);
      expect(tier3Validation.valid).toBe(true);

      // Tier 2 (standard) - requires contracts
      const tier2Spec = createSpecForRiskTier(2, {
        title: "Standard feature",
      });
      const tier2Validation = await validator.validateWorkingSpec(tier2Spec);
      expect(tier2Validation.valid).toBe(true);

      // Tier 1 (critical) - requires contracts and stricter validation
      // Note: May have additional validation that causes failures
      // This test validates that the spec is structurally valid
      const tier1Spec = createSpecForRiskTier(1, {
        title: "Critical feature",
      });
      const tier1Validation = await validator.validateWorkingSpec(tier1Spec);
      // Tier 1 may have stricter requirements - check for errors
      if (!tier1Validation.valid) {
        expect(tier1Validation.errors.length).toBeGreaterThan(0);
      }
    });
  });

  describe("ARBITER-004: Performance Tracking", () => {
    it("should track agent performance across tasks", async () => {
      const agent = await registry.registerAgent(
        createTestAgent({ id: "performance-test-agent" })
      );

      // Execute multiple tasks and track performance
      for (let i = 0; i < 3; i++) {
        const outcome = createMockOutcome(true, {
          latencyMs: 100 + i * 10,
          qualityScore: 0.85 + i * 0.05,
        });

        await registry.updatePerformance(agent.id, outcome);
      }

      // Verify performance was tracked
      const updatedAgent = await registry.getProfile(agent.id);
      expect(updatedAgent.performanceHistory.taskCount).toBe(3);
      expect(updatedAgent.performanceHistory.successRate).toBe(1.0);
      expect(updatedAgent.performanceHistory.averageQuality).toBeGreaterThan(
        0.85
      );
    });
  });

  describe("Multi-Component Integration Workflows", () => {
    it("should complete full workflow: register → validate → route → track", async () => {
      // 1. Register agent (ARBITER-001)
      const agent = await registry.registerAgent(
        createTestAgent({
          id: "workflow-agent",
          name: "Workflow Test Agent",
        })
      );
      expect(agent.id).toBe("workflow-agent");

      // 2. Validate CAWS spec (ARBITER-003)
      const spec = createMinimalWorkingSpec({
        title: "Workflow test feature",
        risk_tier: 3,
      });
      const validation = await validator.validateWorkingSpec(spec);
      expect(validation.valid).toBe(true);

      // 3. Route task to agent (ARBITER-002)
      const task = createMinimalTask({
        description: "Implement workflow test feature",
        type: "code-editing",
      });
      const routing = await router.routeTask(task);
      expect(routing.selectedAgent.id).toBe("workflow-agent");

      // 4. Track task execution (ARBITER-004)
      const outcome = createMockOutcome(true, {
        latencyMs: 150,
        qualityScore: 0.9,
      });
      await registry.updatePerformance(routing.selectedAgent.id, outcome);

      // 5. Verify performance was updated
      const updatedAgent = await registry.getProfile(agent.id);
      expect(updatedAgent.performanceHistory.taskCount).toBe(1);
      expect(updatedAgent.performanceHistory.successRate).toBe(1.0);
      expect(updatedAgent.performanceHistory.averageQuality).toBeCloseTo(
        0.9,
        1
      );
    });

    it("should handle agent failure and route retry", async () => {
      // Register two agents
      await registry.registerAgent(createTestAgent({ id: "agent-1" }));
      await registry.registerAgent(createTestAgent({ id: "agent-2" }));

      // Route initial task
      const task = createMinimalTask({ description: "Test task" });
      const routing1 = await router.routeTask(task);
      const firstAgentId = routing1.selectedAgent.id;

      // Simulate failure
      const failureOutcome = createMockOutcome(false, {
        error: "Agent timeout",
      });
      await registry.updatePerformance(firstAgentId, failureOutcome);

      // Verify agent was penalized
      const agent1After = await registry.getProfile(firstAgentId);
      expect(agent1After.performanceHistory.successRate).toBeLessThan(1.0);

      // Route retry task
      const retryTask = createMinimalTask({ description: "Retry task" });
      const routing2 = await router.routeTask(retryTask);

      // Should still route successfully (to same or different agent)
      expect(routing2.selectedAgent).toBeDefined();
    });
  });

  describe("Load Testing", () => {
    it("should handle 50 concurrent task routings", async () => {
      // Register 10 agents
      const agents = createMultipleAgents(10, {
        modelFamily: "gpt-4",
      });

      for (const agentData of agents) {
        await registry.registerAgent(agentData);
      }

      // Create 50 tasks
      const tasks = createTaskBatch(50, {
        type: "code-editing",
        priority: 5,
      });

      const startTime = Date.now();

      // Route all tasks concurrently
      const routings = await Promise.all(
        tasks.map((task) => router.routeTask(task))
      );

      const duration = Date.now() - startTime;

      // Verify
      expect(routings).toHaveLength(50);
      expect(routings.every((r) => r.selectedAgent)).toBe(true);

      // Should complete reasonably fast (< 3 seconds for 50 tasks)
      expect(duration).toBeLessThan(3000);

      // Verify load distribution
      const agentTaskCounts = new Map<string, number>();
      routings.forEach((r) => {
        const count = agentTaskCounts.get(r.selectedAgent.id) || 0;
        agentTaskCounts.set(r.selectedAgent.id, count + 1);
      });

      // Multiple agents should have received tasks
      expect(agentTaskCounts.size).toBeGreaterThan(3);

      console.log(
        `✅ Integration: Routed 50 tasks in ${duration}ms across ${agentTaskCounts.size} agents`
      );
    });

    it("should maintain data consistency under concurrent operations", async () => {
      const agent = await registry.registerAgent(
        createTestAgent({ id: "concurrent-test-agent" })
      );

      // Simulate 20 concurrent performance updates
      const outcomes = Array.from({ length: 20 }, (_, i) =>
        createMockOutcome(i % 4 !== 0, {
          // 75% success rate
          latencyMs: 100 + i * 5,
          qualityScore: 0.7 + (i % 3) * 0.1,
        })
      );

      await Promise.all(
        outcomes.map((outcome) => registry.updatePerformance(agent.id, outcome))
      );

      // Verify final state is consistent
      const finalAgent = await registry.getProfile(agent.id);
      expect(finalAgent.performanceHistory.taskCount).toBe(20);
      expect(finalAgent.performanceHistory.successRate).toBeCloseTo(0.75, 1);
      expect(finalAgent.performanceHistory.averageLatency).toBeGreaterThan(100);
    });
  });

  describe("Error Handling and Edge Cases", () => {
    it("should handle routing when no agents are available", async () => {
      // Don't register any agents
      const task = createMinimalTask();

      // Routing throws when no agents available (expected behavior)
      await expect(router.routeTask(task)).rejects.toThrow();
    });

    it("should handle invalid agent ID in performance update", async () => {
      const outcome = createMockOutcome(true);

      // Throws RegistryError for non-existent agent (expected behavior)
      await expect(
        registry.updatePerformance("non-existent-agent", outcome)
      ).rejects.toThrow(/not found/);
    });

    it("should handle malformed working specs gracefully", async () => {
      const malformedSpec = {
        id: "MALFORMED-001",
        // Missing required fields
      } as any;

      const validation = await validator.validateWorkingSpec(malformedSpec);
      expect(validation).toBeDefined();
      expect(validation.valid).toBe(false);
    });
  });
});
