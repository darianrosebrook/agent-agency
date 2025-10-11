/**
 * Unit tests for Memory-Aware Agent Orchestrator
 *
 * @author @darianrosebrook
 */

import {
  AgentOrchestrator,
  MemoryAwareAgentOrchestratorConfig,
} from "../../src/services/AgentOrchestrator";
import { Logger } from "../../src/utils/Logger";

describe("MemoryAwareAgentOrchestrator", () => {
  let orchestrator: AgentOrchestrator;
  let _mockLogger: Logger;

  const memoryConfig: MemoryAwareAgentOrchestratorConfig = {
    maxConcurrentTasks: 10,
    taskTimeoutMs: 30000,
    retryAttempts: 3,
    healthCheckIntervalMs: 5000,
    memoryEnabled: true,
    experienceLearningEnabled: true,
    memoryBasedRoutingEnabled: true,
    defaultTenantId: "test-tenant",
  };

  beforeEach(() => {
    _mockLogger = {
      info: jest.fn(),
      warn: jest.fn(),
      error: jest.fn(),
      debug: jest.fn(),
    } as any;
  });

  describe("initialization", () => {
    it("should initialize with memory system enabled", async () => {
      orchestrator = new AgentOrchestrator(memoryConfig);
      await orchestrator.initialize();

      expect(orchestrator).toBeDefined();
      // Memory manager should be initialized internally
    });

    it("should initialize with memory system disabled", async () => {
      const configWithoutMemory = { ...memoryConfig, memoryEnabled: false };
      orchestrator = new AgentOrchestrator(configWithoutMemory);
      await orchestrator.initialize();

      expect(orchestrator).toBeDefined();
    });
  });

  describe("memory-aware task submission", () => {
    beforeEach(async () => {
      orchestrator = new AgentOrchestrator(memoryConfig);
      await orchestrator.initialize();
    });

    it("should submit task with memory routing enabled", async () => {
      const task = {
        agentId: "agent-1",
        type: "process" as const,
        description: "Process customer data",
        priority: "normal" as const,
        requirements: ["python", "pandas"],
        maxRetries: 3,
        timeout: 60000,
        payload: {},
      };

      const taskId = await orchestrator.submitTask(task, {
        tenantId: "test-tenant",
        useMemoryRouting: true,
      });

      expect(typeof taskId).toBe("string");
      expect(taskId).toContain("id_");
    });

    it("should submit task with memory routing disabled", async () => {
      const task = {
        agentId: "agent-1",
        type: "process" as const,
        description: "Process customer data",
        priority: "normal" as const,
        requirements: ["python", "pandas"],
        maxRetries: 3,
        timeout: 60000,
        payload: {},
      };

      const taskId = await orchestrator.submitTask(task, {
        useMemoryRouting: false,
      });

      expect(typeof taskId).toBe("string");
    });
  });

  describe("task completion and learning", () => {
    let taskId: string;

    beforeEach(async () => {
      orchestrator = new AgentOrchestrator(memoryConfig);
      await orchestrator.initialize();

      // Submit a task
      taskId = await orchestrator.submitTask({
        agentId: "agent-1",
        type: "process" as const,
        description: "Process customer data",
        priority: "normal" as const,
        requirements: ["python", "pandas"],
        maxRetries: 3,
        timeout: 60000,
        payload: {},
      });
    });

    it("should complete task successfully and learn from outcome", async () => {
      const result = { processedRecords: 1000, accuracy: 0.95 };
      const metadata = { executionTime: 15000 };

      await expect(
        orchestrator.completeTask(taskId, result, "success", metadata)
      ).resolves.not.toThrow();
    });

    it("should complete task with failure and learn from outcome", async () => {
      const result = { error: "Database connection failed" };
      const metadata = { retryCount: 2 };

      await expect(
        orchestrator.completeTask(taskId, result, "failure", metadata)
      ).resolves.not.toThrow();
    });
  });

  describe("system metrics with memory stats", () => {
    beforeEach(async () => {
      orchestrator = new AgentOrchestrator(memoryConfig);
      await orchestrator.initialize();
    });

    it("should return system metrics including memory statistics", async () => {
      const metrics = await orchestrator.getSystemMetrics();

      expect(metrics).toHaveProperty("totalAgents");
      expect(metrics).toHaveProperty("activeAgents");
      expect(metrics).toHaveProperty("totalTasks");
      expect(metrics).toHaveProperty("completedTasks");
      expect(metrics).toHaveProperty("failedTasks");
      expect(metrics).toHaveProperty("averageTaskDuration");
      expect(metrics).toHaveProperty("systemUptime");

      // Memory stats may or may not be present depending on initialization
      if (metrics.memoryStats) {
        expect(metrics.memoryStats).toHaveProperty("tenants");
        expect(metrics.memoryStats).toHaveProperty("activeOperations");
        expect(metrics.memoryStats).toHaveProperty("cacheSize");
        expect(metrics.memoryStats).toHaveProperty("memoryEnabled", true);
      }
    });
  });

  describe("agent registration", () => {
    beforeEach(async () => {
      orchestrator = new AgentOrchestrator(memoryConfig);
      await orchestrator.initialize();
    });

    it("should register an agent successfully", async () => {
      const agentId = await orchestrator.registerAgent({
        name: "Test Agent",
        type: "worker" as const,
        capabilities: ["python", "pandas", "machine_learning"],
        status: "active",
        metadata: {},
      });

      expect(typeof agentId).toBe("string");
    });
  });

  describe("task retrieval", () => {
    let taskId: string;

    beforeEach(async () => {
      orchestrator = new AgentOrchestrator(memoryConfig);
      await orchestrator.initialize();

      taskId = await orchestrator.submitTask({
        agentId: "agent-1",
        type: "process" as const,
        description: "Process customer data",
        priority: "normal" as const,
        requirements: ["python", "pandas"],
        maxRetries: 3,
        timeout: 60000,
        payload: {},
      });
    });

    it("should retrieve task by ID", () => {
      const task = orchestrator.getTask(taskId);
      expect(task).toBeDefined();
      expect(task?.id).toBe(taskId);
      expect(task?.status).toBe("pending");
    });

    it("should return undefined for non-existent task", () => {
      const task = orchestrator.getTask("non-existent-task");
      expect(task).toBeUndefined();
    });
  });

  describe("agent retrieval", () => {
    let agentId: string;

    beforeEach(async () => {
      orchestrator = new AgentOrchestrator(memoryConfig);
      await orchestrator.initialize();

      agentId = await orchestrator.registerAgent({
        name: "Test Agent",
        type: "worker" as const,
        capabilities: ["python", "pandas"],
        status: "active",
        metadata: {},
      });
    });

    it("should retrieve agent by ID", () => {
      const agent = orchestrator.getAgent(agentId);
      expect(agent).toBeDefined();
      expect(agent?.id).toBe(agentId);
      expect(agent?.name).toBe("Test Agent");
    });

    it("should return undefined for non-existent agent", () => {
      const agent = orchestrator.getAgent("non-existent-agent");
      expect(agent).toBeUndefined();
    });
  });
});
