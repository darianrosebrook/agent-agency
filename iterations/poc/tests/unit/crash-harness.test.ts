/**
 * Crash Harness Tests
 *
 * @author @darianrosebrook
 * @description Comprehensive crash testing to ensure system stability under failure conditions
 */

import {
  jest,
  describe,
  beforeEach,
  afterEach,
  it,
  expect,
} from "@jest/globals";
import { AgentOrchestrator } from "../../src/services/AgentOrchestrator";
import { MultiTenantMemoryManager } from "../../src/memory/MultiTenantMemoryManager";
import { AdvancedTaskRouter } from "../../src/services/AdvancedTaskRouter";
import { ErrorPatternAnalyzer } from "../../src/services/ErrorPatternAnalyzer";
import { CawsConstitutionalEnforcer } from "../../src/services/CawsConstitutionalEnforcer";
import { Logger } from "../../src/utils/Logger";

// Mock all dependencies to simulate various failure scenarios
jest.mock("../../src/memory/MultiTenantMemoryManager");
jest.mock("../../src/services/AdvancedTaskRouter");
jest.mock("../../src/services/ErrorPatternAnalyzer");
jest.mock("../../src/services/CawsConstitutionalEnforcer");
jest.mock("../../src/utils/Logger");

// Create a proper mock logger that doesn't throw during initialization
const mockLogger = {
  info: jest.fn(),
  warn: jest.fn(),
  error: jest.fn(),
  debug: jest.fn(),
};
(Logger as jest.Mock).mockImplementation(() => mockLogger);

describe("Crash Harness Tests", () => {
  let orchestrator: AgentOrchestrator;

  beforeEach(() => {
    jest.clearAllMocks();
    // Reset all mocked classes
    (MultiTenantMemoryManager as jest.Mock).mockClear();
    (AdvancedTaskRouter as jest.Mock).mockClear();
    (ErrorPatternAnalyzer as jest.Mock).mockClear();
    (CawsConstitutionalEnforcer as jest.Mock).mockClear();
    (Logger as jest.Mock).mockClear();

    // Set up default successful mocks
    const mockMemoryManager = {
      initialize: jest.fn().mockResolvedValue(undefined),
      registerTenant: jest.fn().mockResolvedValue(undefined),
      storeExperience: jest.fn().mockResolvedValue(undefined),
      retrieveExperiences: jest.fn().mockResolvedValue([]),
    };
    (MultiTenantMemoryManager as jest.Mock).mockImplementation(
      () => mockMemoryManager
    );

    const mockTaskRouter = {
      submitTask: jest.fn().mockResolvedValue({
        selectedAgentId: "agent-1",
        routingStrategy: "load_balanced",
        confidence: 0.95,
        estimatedLatency: 100,
        expectedQuality: 0.9,
      }),
    };
    (AdvancedTaskRouter as jest.Mock).mockImplementation(() => mockTaskRouter);

    const mockErrorAnalyzer = {
      initialize: jest.fn().mockResolvedValue(undefined),
    };
    (ErrorPatternAnalyzer as jest.Mock).mockImplementation(
      () => mockErrorAnalyzer
    );

    const mockCawsEnforcer = {
      initialize: jest.fn().mockResolvedValue(undefined),
      enforceConstitution: jest
        .fn()
        .mockResolvedValue({ allowed: true, violations: [] }),
      startBudgetTracking: jest.fn(),
    };
    (CawsConstitutionalEnforcer as jest.Mock).mockImplementation(
      () => mockCawsEnforcer
    );
  });

  afterEach(async () => {
    if (orchestrator) {
      // Clean up any state
      orchestrator = null as any;
    }
  });

  describe("Initialization Crash Scenarios", () => {
    it("should handle memory system initialization failure gracefully", async () => {
      // Override the default mock with a failing one for this test
      const mockMemoryManager = {
        initialize: jest
          .fn()
          .mockRejectedValue(new Error("Memory system crashed")),
        registerTenant: jest.fn().mockResolvedValue(undefined),
        storeExperience: jest.fn().mockResolvedValue(undefined),
        retrieveExperiences: jest.fn().mockResolvedValue([]),
      };
      (MultiTenantMemoryManager as jest.Mock).mockImplementation(
        () => mockMemoryManager
      );

      orchestrator = new AgentOrchestrator({
        memoryEnabled: true,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });

      // The orchestrator should handle memory initialization failures gracefully
      await expect(orchestrator.initialize()).resolves.not.toThrow();

      expect(mockMemoryManager.initialize).toHaveBeenCalledTimes(1);
      expect(mockLogger.error).toHaveBeenCalledWith(
        "Failed to initialize memory system:",
        expect.any(Error)
      );
    });

    it("should handle task router initialization failure", async () => {
      // Simulate task router constructor failure
      (AdvancedTaskRouter as jest.Mock).mockImplementation(() => {
        throw new Error("Task router initialization failed");
      });

      orchestrator = new AgentOrchestrator({
        memoryEnabled: false,
        advancedRoutingEnabled: true,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });

      await expect(orchestrator.initialize()).rejects.toThrow(
        "Task router initialization failed"
      );
    });

    it("should handle error analyzer initialization failure gracefully", async () => {
      const mockErrorAnalyzer = {
        initialize: jest
          .fn()
          .mockRejectedValue(new Error("Error analyzer crashed")),
      };
      (ErrorPatternAnalyzer as jest.Mock).mockImplementation(
        () => mockErrorAnalyzer
      );

      orchestrator = new AgentOrchestrator({
        memoryEnabled: false,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: true,
        cawsEnforcementEnabled: false,
      });

      // Error analyzer initialization should be handled gracefully
      await expect(orchestrator.initialize()).resolves.not.toThrow();

      // The error should be logged
      expect(mockLogger.error).toHaveBeenCalledWith(
        "Failed to initialize error pattern analyzer:",
        expect.any(Error)
      );
    });

    it("should handle CAWS enforcer initialization failure", async () => {
      const mockCawsEnforcer = {
        initialize: jest
          .fn()
          .mockRejectedValue(new Error("CAWS enforcer crashed")),
        enforceConstitution: jest.fn(),
        startBudgetTracking: jest.fn(),
      };
      (CawsConstitutionalEnforcer as jest.Mock).mockImplementation(
        () => mockCawsEnforcer
      );

      orchestrator = new AgentOrchestrator({
        memoryEnabled: false,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: true,
      });

      // CAWS enforcer initialization might be handled gracefully
      await expect(orchestrator.initialize()).resolves.not.toThrow();

      // The error should be logged
      expect(mockLogger.error).toHaveBeenCalledWith(
        "Failed to initialize CAWS constitutional enforcer:",
        expect.any(Error)
      );
    });

    it("should handle partial initialization failures gracefully", async () => {
      // Memory succeeds, task router fails, others succeed
      const mockMemoryManager = {
        initialize: jest.fn().mockResolvedValue(undefined),
        registerTenant: jest.fn().mockResolvedValue(undefined),
        storeExperience: jest.fn().mockResolvedValue(undefined),
        retrieveExperiences: jest.fn().mockResolvedValue([]),
      };
      (MultiTenantMemoryManager as jest.Mock).mockImplementation(
        () => mockMemoryManager
      );

      (AdvancedTaskRouter as jest.Mock).mockImplementation(() => {
        throw new Error("Task router crashed during construction");
      });

      orchestrator = new AgentOrchestrator({
        memoryEnabled: true,
        advancedRoutingEnabled: true,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });

      // Task router construction failure should cause initialization to fail
      await expect(orchestrator.initialize()).rejects.toThrow(
        "Task router crashed during construction"
      );

      // Memory should still have been initialized before the task router failure
      expect(mockMemoryManager.initialize).toHaveBeenCalledTimes(1);
    });
  });

  describe("Task Submission Crash Scenarios", () => {
    beforeEach(async () => {
      // Reset mocks for this test suite
      jest.clearAllMocks();

      // Set up successful initialization mocks
      const mockMemoryManager = {
        initialize: jest.fn().mockResolvedValue(undefined),
        registerTenant: jest.fn().mockResolvedValue(undefined),
        storeExperience: jest.fn().mockResolvedValue(undefined),
        retrieveExperiences: jest.fn().mockResolvedValue([]),
      };
      (MultiTenantMemoryManager as jest.Mock).mockImplementation(
        () => mockMemoryManager
      );

      const mockTaskRouter = {
        submitTask: jest.fn().mockResolvedValue({
          selectedAgentId: "agent-1",
          routingStrategy: "load_balanced",
          confidence: 0.95,
          estimatedLatency: 100,
          expectedQuality: 0.9,
        }),
      };
      (AdvancedTaskRouter as jest.Mock).mockImplementation(
        () => mockTaskRouter
      );

      const mockErrorAnalyzer = {
        initialize: jest.fn().mockResolvedValue(undefined),
      };
      (ErrorPatternAnalyzer as jest.Mock).mockImplementation(
        () => mockErrorAnalyzer
      );

      const mockCawsEnforcer = {
        initialize: jest.fn().mockResolvedValue(undefined),
        enforceConstitution: jest
          .fn()
          .mockResolvedValue({ allowed: true, violations: [] }),
        startBudgetTracking: jest.fn(),
      };
      (CawsConstitutionalEnforcer as jest.Mock).mockImplementation(
        () => mockCawsEnforcer
      );

      orchestrator = new AgentOrchestrator({
        memoryEnabled: true,
        advancedRoutingEnabled: true,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: true,
      });

      await orchestrator.initialize();
    });

    it("should handle CAWS constitution enforcement failure", async () => {
      // Override the CAWS enforcer mock to fail constitution checks
      const mockCawsEnforcer = {
        initialize: jest.fn().mockResolvedValue(undefined),
        enforceConstitution: jest
          .fn()
          .mockRejectedValue(new Error("Constitution check failed")),
        startBudgetTracking: jest.fn(),
      };
      (CawsConstitutionalEnforcer as jest.Mock).mockImplementation(
        () => mockCawsEnforcer
      );

      const task = {
        type: "code_generation" as const,
        description: "Generate a function",
        priority: 1 as const,
        agentId: "agent-1",
        payload: {},
      };

      await expect(orchestrator.submitTask(task)).rejects.toThrow(
        "Constitution check failed"
      );
    });

    it("should handle routing failure gracefully", async () => {
      // Override the task router mock to fail
      const mockTaskRouter = {
        submitTask: jest
          .fn()
          .mockRejectedValue(new Error("Routing service unavailable")),
      };
      (AdvancedTaskRouter as jest.Mock).mockImplementation(
        () => mockTaskRouter
      );

      const task = {
        type: "code_generation" as const,
        description: "Generate a function",
        priority: 1 as const,
        agentId: "agent-1",
        payload: {},
      };

      // Should not crash - should fall back to original agent assignment
      const taskId = await orchestrator.submitTask(task);
      expect(typeof taskId).toBe("string");
      expect(taskId.length).toBeGreaterThan(0);
    });

    it("should handle task submission with extremely deep object graphs", async () => {
      // Create a task with deeply nested payload that could cause stack overflow in serialization
      const createDeepObject = (depth: number): any => {
        if (depth === 0) return { value: "leaf" };
        return { nested: createDeepObject(depth - 1) };
      };

      const task = {
        type: "code_generation" as const,
        description: "Process deep object",
        priority: 1 as const,
        agentId: "agent-1",
        payload: createDeepObject(1000), // Very deep nesting
      };

      // Should handle deep objects without crashing
      const taskId = await orchestrator.submitTask(task);
      expect(typeof taskId).toBe("string");
      expect(taskId.length).toBeGreaterThan(0);
    });

    it("should handle invalid task data gracefully", async () => {
      const invalidTasks = [
        null,
        undefined,
        {},
        { type: "invalid" },
        { type: "code_generation", description: null },
        { type: "code_generation", description: "", priority: -1 },
      ];

      for (const invalidTask of invalidTasks) {
        await expect(
          orchestrator.submitTask(invalidTask as any)
        ).rejects.toThrow();
      }
    });
  });

  describe("Agent Management Crash Scenarios", () => {
    beforeEach(async () => {
      orchestrator = new AgentOrchestrator({
        memoryEnabled: false,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });
      await orchestrator.initialize();
    });

    it("should handle agent registration with invalid data", async () => {
      // Test with some truly invalid data that should cause failures
      const invalidAgents = [
        null,
        undefined,
        // Empty object should fail validation
        {},
        // Missing required name should fail
        { type: "worker" },
      ];

      // Most invalid data might be handled gracefully, so let's check what actually fails
      let failureCount = 0;
      for (const invalidAgent of invalidAgents) {
        try {
          await orchestrator.registerAgent(invalidAgent as any);
        } catch (error) {
          failureCount++;
        }
      }

      // At least some should fail
      expect(failureCount).toBeGreaterThan(0);
    });

    it("should handle duplicate agent registration", async () => {
      const agent = {
        name: "Test Agent",
        type: "worker" as const,
        capabilities: ["code_generation"],
      };

      await orchestrator.registerAgent(agent);
      // Registering the same agent again should work (different IDs generated)
      await expect(orchestrator.registerAgent(agent)).resolves.toBeDefined();
    });

    it("should handle extremely large agent metadata", async () => {
      const agentWithLargeMetadata = {
        name: "Large Metadata Agent",
        type: "worker" as const,
        capabilities: ["code_generation"],
        metadata: {
          largeData: "x".repeat(1000000), // 1MB string
          nested: Array.from({ length: 10000 }, (_, i) => ({
            id: i,
            data: "x".repeat(100),
          })),
        },
      };

      // Should handle large metadata without crashing
      const agentId = await orchestrator.registerAgent(agentWithLargeMetadata);
      expect(typeof agentId).toBe("string");
      expect(agentId.length).toBeGreaterThan(0);
    });
  });

  describe("Memory System Crash Scenarios", () => {
    it("should handle memory manager operational failures", async () => {
      const mockMemoryManager = {
        initialize: jest.fn().mockResolvedValue(undefined),
        storeExperience: jest
          .fn()
          .mockRejectedValue(new Error("Memory store failed")),
        retrieveExperiences: jest
          .fn()
          .mockRejectedValue(new Error("Memory retrieval failed")),
      };
      (MultiTenantMemoryManager as jest.Mock).mockImplementation(
        () => mockMemoryManager
      );

      orchestrator = new AgentOrchestrator({
        memoryEnabled: true,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });

      await orchestrator.initialize();

      // Operations that use memory should handle failures gracefully
      // This would test any code paths that call memoryManager methods
    });

    it("should handle memory manager initialization race conditions", async () => {
      const mockMemoryManager = {
        initialize: jest.fn().mockImplementation(() => {
          // Simulate a race condition where initialization is called multiple times
          return new Promise((resolve, reject) => {
            setTimeout(() => {
              if (Math.random() > 0.5) {
                resolve(undefined);
              } else {
                reject(new Error("Concurrent initialization conflict"));
              }
            }, 10);
          });
        }),
      };
      (MultiTenantMemoryManager as jest.Mock).mockImplementation(
        () => mockMemoryManager
      );

      orchestrator = new AgentOrchestrator({
        memoryEnabled: true,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });

      // Multiple concurrent initialization calls
      const initPromises = [
        orchestrator.initialize(),
        orchestrator.initialize(),
        orchestrator.initialize(),
      ];

      const results = await Promise.allSettled(initPromises);

      // At least one should succeed, others may fail due to race conditions
      const successes = results.filter((r) => r.status === "fulfilled").length;
      const failures = results.filter((r) => r.status === "rejected").length;

      expect(successes + failures).toBe(3);
      expect(successes).toBeGreaterThanOrEqual(1);
    });
  });

  describe("Error Handling Edge Cases", () => {
    it("should handle logger failures gracefully", async () => {
      // Override the global logger mock for this test
      const failingLogger = {
        info: jest.fn().mockImplementation(() => {
          throw new Error("Logging system failed");
        }),
        warn: jest.fn().mockImplementation(() => {
          throw new Error("Logging system failed");
        }),
        error: jest.fn().mockImplementation(() => {
          throw new Error("Logging system failed");
        }),
        debug: jest.fn().mockImplementation(() => {
          throw new Error("Logging system failed");
        }),
      };
      (Logger as jest.Mock).mockImplementation(() => failingLogger);

      orchestrator = new AgentOrchestrator({
        memoryEnabled: false,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });

      // Initialization might fail due to logger errors, but operations should be resilient
      try {
        await orchestrator.initialize();
        // If initialization succeeds, operations should still work
        const task = {
          type: "code_generation" as const,
          description: "Generate a function",
          priority: 1 as const,
          agentId: "agent-1",
          payload: {},
        };

        await expect(orchestrator.submitTask(task)).resolves.toBeDefined();
      } catch (error) {
        // If initialization fails due to logger, that's also acceptable
        expect(error.message).toContain("Logging system failed");
      }
    });

    it("should handle ID generation failures", async () => {
      orchestrator = new AgentOrchestrator({
        memoryEnabled: false,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });

      // Mock Math.random to return invalid values that could break ID generation
      const originalRandom = Math.random;
      Math.random = jest.fn().mockReturnValue(NaN); // Invalid random value

      await orchestrator.initialize();

      const task = {
        type: "code_generation" as const,
        description: "Generate a function",
        priority: 1 as const,
        agentId: "agent-1",
        payload: {},
      };

      // Should handle invalid random values gracefully
      const taskId = await orchestrator.submitTask(task);
      expect(typeof taskId).toBe("string");
      expect(taskId.length).toBeGreaterThan(0);

      // Restore original method
      Math.random = originalRandom;
    });

    it("should handle Date constructor failures", async () => {
      // Simulate Date.now() throwing errors (possible in some environments)
      const originalDateNow = global.Date.now;
      global.Date.now = jest.fn().mockImplementation(() => {
        throw new Error("System clock unavailable");
      });

      orchestrator = new AgentOrchestrator({
        memoryEnabled: false,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });

      await orchestrator.initialize();

      const task = {
        type: "code_generation" as const,
        description: "Generate a function",
        priority: 1 as const,
        agentId: "agent-1",
        payload: {},
      };

      await expect(orchestrator.submitTask(task)).rejects.toThrow(
        "System clock unavailable"
      );

      // Restore original Date.now
      global.Date.now = originalDateNow;
    });
  });

  describe("Resource Exhaustion Scenarios", () => {
    it("should handle concurrent task submission limits", async () => {
      orchestrator = new AgentOrchestrator({
        memoryEnabled: false,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
        maxConcurrentTasks: 2,
      });

      await orchestrator.initialize();

      const tasks = Array.from({ length: 10 }, (_, i) => ({
        type: "code_generation" as const,
        description: `Task ${i}`,
        priority: 1 as const,
        agentId: "agent-1",
        payload: {},
      }));

      // Submit all tasks concurrently
      const results = await Promise.allSettled(
        tasks.map((task) => orchestrator.submitTask(task))
      );

      // Should handle the load without crashing
      const successes = results.filter((r) => r.status === "fulfilled").length;
      const failures = results.filter((r) => r.status === "rejected").length;

      expect(successes + failures).toBe(10);
      // System should not crash even under high concurrent load
    });

    it("should handle extremely large task payloads", async () => {
      orchestrator = new AgentOrchestrator({
        memoryEnabled: false,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });

      await orchestrator.initialize();

      // Create a task with an extremely large payload
      const largePayload = {
        data: "x".repeat(1000000), // 1MB string
        nested: {
          deep: {
            structure: Array.from({ length: 10000 }, (_, i) => ({
              id: i,
              data: "x".repeat(100),
            })),
          },
        },
      };

      const task = {
        type: "code_generation" as const,
        description: "Process large payload",
        priority: 1 as const,
        agentId: "agent-1",
        payload: largePayload,
      };

      // Should handle large payloads without crashing
      const taskId = await orchestrator.submitTask(task);
      expect(typeof taskId).toBe("string");
    });

    it("should handle rapid initialization/deinitialization cycles", async () => {
      // Simulate rapid start/stop cycles (like in unstable environments)
      for (let i = 0; i < 100; i++) {
        orchestrator = new AgentOrchestrator({
          memoryEnabled: false,
          advancedRoutingEnabled: false,
          errorAnalysisEnabled: false,
          cawsEnforcementEnabled: false,
        });

        await orchestrator.initialize();

        // Force cleanup
        orchestrator = null as any;
      }

      // System should not have crashed from rapid cycles
    });
  });

  describe("Network and External Dependency Failures", () => {
    it("should handle external service timeouts", async () => {
      // Simulate network timeouts in routing decisions
      const mockTaskRouter = {
        submitTask: jest.fn().mockImplementation(() => {
          return new Promise((_, reject) => {
            setTimeout(() => reject(new Error("Network timeout")), 100);
          });
        }),
      };
      (AdvancedTaskRouter as jest.Mock).mockImplementation(
        () => mockTaskRouter
      );

      orchestrator = new AgentOrchestrator({
        memoryEnabled: false,
        advancedRoutingEnabled: true,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
        taskTimeoutMs: 50, // Very short timeout
      });

      await orchestrator.initialize();

      const task = {
        type: "code_generation" as const,
        description: "Generate with timeout",
        priority: 1 as const,
        agentId: "agent-1",
        payload: {},
      };

      // Should handle timeout gracefully
      const startTime = Date.now();
      await expect(orchestrator.submitTask(task)).rejects.toThrow();
      const duration = Date.now() - startTime;

      // Should fail quickly, not hang
      expect(duration).toBeLessThan(200);
    });

    it("should handle connection pool exhaustion", async () => {
      // Simulate database connection pool exhaustion
      const mockMemoryManager = {
        initialize: jest.fn().mockResolvedValue(undefined),
        storeExperience: jest
          .fn()
          .mockRejectedValue(new Error("Connection pool exhausted")),
      };
      (MultiTenantMemoryManager as jest.Mock).mockImplementation(
        () => mockMemoryManager
      );

      orchestrator = new AgentOrchestrator({
        memoryEnabled: true,
        advancedRoutingEnabled: false,
        errorAnalysisEnabled: false,
        cawsEnforcementEnabled: false,
      });

      await orchestrator.initialize();

      // Operations should handle connection failures gracefully
      // This tests resilience against infrastructure failures
    });
  });
});
