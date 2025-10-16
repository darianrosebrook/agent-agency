/**
 * Enhanced Specialization Features Test
 *
 * @author @darianrosebrook
 * @module tests/unit/orchestrator/enhanced-specialization
 *
 * Tests for enhanced agent specialization features including expertise levels,
 * performance tracking, and advanced matching algorithms.
 */
// @ts-nocheck


import { AgentRegistryManager } from "@/orchestrator/AgentRegistryManager";
import type { AgentQuery, PerformanceMetrics } from "@/types/agent-registry";

describe("Enhanced Agent Specialization", () => {
  let registry: AgentRegistryManager;

  beforeEach(async () => {
    registry = new AgentRegistryManager({
      enablePersistence: false,
      enableSecurity: false,
    });
    await registry.initialize();

    // Register test agents with enhanced specializations
    await registry.registerAgent({
      id: "expert-frontend-dev",
      name: "Frontend Expert",
      modelFamily: "gpt-4",
      capabilities: {
        taskTypes: ["code-editing", "testing"],
        languages: ["TypeScript", "JavaScript"],
        specializationsV2: [
          {
            type: "Frontend architecture",
            level: "expert",
            successRate: 0.95,
            taskCount: 50,
            averageQuality: 0.92,
            lastUsed: "2025-10-15T10:00:00Z",
          },
          {
            type: "Performance optimization",
            level: "intermediate",
            successRate: 0.88,
            taskCount: 25,
            averageQuality: 0.85,
            lastUsed: "2025-10-14T15:00:00Z",
          },
        ],
      },
      performanceHistory: {
        successRate: 0.93,
        averageQuality: 0.89,
        averageLatency: 3500,
        taskCount: 75,
      },
      currentLoad: {
        activeTasks: 0,
        queuedTasks: 0,
        utilizationPercent: 0,
      },
    });

    await registry.registerAgent({
      id: "backend-specialist",
      name: "Backend Specialist",
      modelFamily: "claude-3.5",
      capabilities: {
        taskTypes: ["code-editing", "api-design"],
        languages: ["TypeScript", "Python"],
        specializationsV2: [
          {
            type: "Backend architecture",
            level: "expert",
            successRate: 0.96,
            taskCount: 60,
            averageQuality: 0.94,
            lastUsed: "2025-10-15T11:00:00Z",
          },
          {
            type: "Database design",
            level: "expert",
            successRate: 0.92,
            taskCount: 40,
            averageQuality: 0.9,
            lastUsed: "2025-10-15T09:00:00Z",
          },
        ],
      },
      performanceHistory: {
        successRate: 0.94,
        averageQuality: 0.91,
        averageLatency: 4200,
        taskCount: 100,
      },
      currentLoad: {
        activeTasks: 1,
        queuedTasks: 0,
        utilizationPercent: 15,
      },
    });
  });

  describe("Enhanced Specialization Queries", () => {
    it("should find agents with specific expertise levels", async () => {
      const query: AgentQuery = {
        taskType: "code-editing",
        specializationQuery: [
          {
            type: "Frontend architecture",
            minLevel: "expert",
            required: true,
          },
        ],
      };

      const results = await registry.getAgentsByCapability(query);

      expect(results).toHaveLength(1);
      expect(results[0].agent.id).toBe("expert-frontend-dev");
      expect(results[0].matchScore).toBeGreaterThan(0.8);
    });

    it("should handle optional specializations with lower weight", async () => {
      const query: AgentQuery = {
        taskType: "code-editing",
        specializationQuery: [
          {
            type: "Frontend architecture",
            minLevel: "expert",
            required: true,
          },
          {
            type: "Security audit",
            minLevel: "intermediate",
            required: false, // Optional
          },
        ],
      };

      const results = await registry.getAgentsByCapability(query);

      expect(results).toHaveLength(1);
      expect(results[0].agent.id).toBe("expert-frontend-dev");
      // Score should be good even without the optional security specialization
      expect(results[0].matchScore).toBeGreaterThan(0.7);
    });

    it("should filter agents below minimum success rate", async () => {
      const query: AgentQuery = {
        taskType: "code-editing",
        specializationQuery: [
          {
            type: "Frontend architecture",
            minSuccessRate: 0.97, // Higher than available
            required: true,
          },
        ],
      };

      const results = await registry.getAgentsByCapability(query);

      expect(results).toHaveLength(0);
    });

    it("should rank agents by specialization expertise", async () => {
      // Add a third agent with lower expertise
      await registry.registerAgent({
        id: "junior-frontend-dev",
        name: "Junior Frontend Dev",
        modelFamily: "gpt-3.5-turbo",
        capabilities: {
          taskTypes: ["code-editing"],
          languages: ["JavaScript"],
          specializationsV2: [
            {
              type: "Frontend architecture",
              level: "novice",
              successRate: 0.75,
              taskCount: 10,
              averageQuality: 0.7,
              lastUsed: "2025-10-10T10:00:00Z",
            },
          ],
        },
        performanceHistory: {
          successRate: 0.8,
          averageQuality: 0.75,
          averageLatency: 5500,
          taskCount: 25,
        },
        currentLoad: {
          activeTasks: 0,
          queuedTasks: 0,
          utilizationPercent: 0,
        },
      });

      const query: AgentQuery = {
        taskType: "code-editing",
        specializationQuery: [
          {
            type: "Frontend architecture",
            required: true,
          },
        ],
      };

      const results = await registry.getAgentsByCapability(query);

      expect(results).toHaveLength(2);
      // Expert should rank higher than novice
      expect(results[0].agent.id).toBe("expert-frontend-dev");
      expect(results[1].agent.id).toBe("junior-frontend-dev");
      expect(results[0].matchScore).toBeGreaterThan(results[1].matchScore);
    });
  });

  describe("Specialization Performance Tracking", () => {
    it("should update specialization performance metrics", async () => {
      const metrics: PerformanceMetrics = {
        success: true,
        qualityScore: 0.95,
        latencyMs: 3200,
      };

      const updatedProfile = await registry.updateSpecializationPerformance(
        "expert-frontend-dev",
        "Frontend architecture",
        metrics
      );

      const frontendSpec = updatedProfile.capabilities.specializationsV2?.find(
        (s) => s.type === "Frontend architecture"
      );

      expect(frontendSpec).toBeDefined();
      expect(frontendSpec!.taskCount).toBe(51); // Increased from 50
      expect(frontendSpec!.successRate).toBeGreaterThan(0.94); // Improved average
      expect(frontendSpec!.averageQuality).toBeGreaterThan(0.91); // Improved average
    });

    it("should create new specialization profile for first use", async () => {
      const metrics: PerformanceMetrics = {
        success: true,
        qualityScore: 0.88,
        latencyMs: 4100,
      };

      const updatedProfile = await registry.updateSpecializationPerformance(
        "expert-frontend-dev",
        "Security audit", // New specialization
        metrics
      );

      const securitySpec = updatedProfile.capabilities.specializationsV2?.find(
        (s) => s.type === "Security audit"
      );

      expect(securitySpec).toBeDefined();
      expect(securitySpec!.taskCount).toBe(1);
      expect(securitySpec!.successRate).toBe(1.0);
      expect(securitySpec!.averageQuality).toBe(0.88);
      expect(securitySpec!.level).toBe("novice"); // Starts as novice
    });

    it("should promote expertise levels based on performance", async () => {
      // Add multiple high-performance tasks to promote from novice to intermediate
      const metrics: PerformanceMetrics = {
        success: true,
        qualityScore: 0.9,
        latencyMs: 3500,
      };

      let updatedProfile = await registry.updateSpecializationPerformance(
        "expert-frontend-dev",
        "Security audit",
        metrics
      );

      // Simulate multiple successful tasks
      for (let i = 0; i < 20; i++) {
        updatedProfile = await registry.updateSpecializationPerformance(
          "expert-frontend-dev",
          "Security audit",
          { success: true, qualityScore: 0.95, latencyMs: 3200 }
        );
      }

      const securitySpec = updatedProfile.capabilities.specializationsV2?.find(
        (s) => s.type === "Security audit"
      );

      expect(securitySpec!.level).toBe("intermediate");
      expect(securitySpec!.taskCount).toBe(21);
    });
  });

  describe("Specialization Statistics", () => {
    it("should provide comprehensive specialization statistics", async () => {
      const stats = await registry.getSpecializationStats();

      expect(stats.length).toBeGreaterThan(0);

      const frontendStats = stats.find(
        (s) => s.specialization === "Frontend architecture"
      );
      expect(frontendStats).toBeDefined();
      expect(frontendStats!.totalAgents).toBe(1);
      expect(frontendStats!.averageSuccessRate).toBe(0.95);
      expect(frontendStats!.expertiseDistribution.expert).toBe(1);

      const backendStats = stats.find(
        (s) => s.specialization === "Backend architecture"
      );
      expect(backendStats).toBeDefined();
      expect(backendStats!.totalAgents).toBe(1);
      expect(backendStats!.expertiseDistribution.expert).toBe(1);
    });

    it("should filter statistics by specialization type", async () => {
      const stats = await registry.getSpecializationStats(
        "Frontend architecture"
      );

      expect(stats).toHaveLength(1);
      expect(stats[0].specialization).toBe("Frontend architecture");
      expect(stats[0].totalAgents).toBe(1);
    });
  });

  describe("Backward Compatibility", () => {
    it("should support legacy specialization queries", async () => {
      const query: AgentQuery = {
        taskType: "code-editing",
        specializations: ["Frontend architecture"], // Legacy format
      };

      const results = await registry.getAgentsByCapability(query);

      expect(results).toHaveLength(1);
      expect(results[0].agent.id).toBe("expert-frontend-dev");
    });
  });
});
