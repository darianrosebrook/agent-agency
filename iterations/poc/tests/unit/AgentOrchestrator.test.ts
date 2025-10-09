/**
 * Agent Orchestrator Unit Tests
 *
 * @author @darianrosebrook
 * @description Unit tests for the AgentOrchestrator service
 */

import { AgentOrchestrator } from "../../src/services/AgentOrchestrator";

describe("AgentOrchestrator", () => {
  let orchestrator: AgentOrchestrator;

  beforeEach(() => {
    orchestrator = new AgentOrchestrator();
  });

  describe("initialization", () => {
    it("should initialize successfully", async () => {
      await expect(orchestrator.initialize()).resolves.not.toThrow();
    });

    it("should not initialize twice", async () => {
      await orchestrator.initialize();
      await expect(orchestrator.initialize()).resolves.not.toThrow();
    });
  });

  describe("agent registration", () => {
    beforeEach(async () => {
      await orchestrator.initialize();
    });

    it("should register a new agent", async () => {
      const agentData = {
        name: "Test Agent",
        type: "worker" as const,
        status: "idle" as const,
        capabilities: ["process"],
        metadata: {},
      };

      const agentId = await orchestrator.registerAgent(agentData);

      expect(agentId).toBeDefined();
      expect(agentId).toMatch(/^id_\d+_[a-z0-9]+$/);

      const agent = orchestrator.getAgent(agentId);
      expect(agent).toBeDefined();
      expect(agent?.name).toBe("Test Agent");
      expect(agent?.type).toBe("worker");
    });

    it("should generate unique agent IDs", async () => {
      const agentData = {
        name: "Test Agent",
        type: "worker" as const,
        status: "idle" as const,
        capabilities: ["process"],
        metadata: {},
      };

      const id1 = await orchestrator.registerAgent(agentData);
      const id2 = await orchestrator.registerAgent(agentData);

      expect(id1).not.toBe(id2);
    });
  });

  describe("task submission", () => {
    let agentId: string;

    beforeEach(async () => {
      await orchestrator.initialize();
      agentId = await orchestrator.registerAgent({
        name: "Test Agent",
        type: "worker",
        status: "idle",
        capabilities: ["process"],
        metadata: {},
      });
    });

    it("should submit a task successfully", async () => {
      const taskData = {
        agentId,
        type: "process" as const,
        description: "Test task",
        priority: "normal" as const,
        payload: { data: "test" },
      };

      const taskId = await orchestrator.submitTask(taskData);

      expect(taskId).toBeDefined();
      expect(taskId).toMatch(/^id_\d+_[a-z0-9]+$/);

      const task = orchestrator.getTask(taskId);
      expect(task).toBeDefined();
      expect(task?.agentId).toBe(agentId);
      expect(task?.status).toBe("pending");
    });
  });

  describe("system metrics", () => {
    beforeEach(async () => {
      await orchestrator.initialize();
    });

    it("should return system metrics", async () => {
      const metrics = await orchestrator.getSystemMetrics();

      expect(metrics).toBeDefined();
      expect(metrics.totalAgents).toBe(0);
      expect(metrics.activeAgents).toBe(0);
      expect(metrics.totalTasks).toBe(0);
      expect(metrics.completedTasks).toBe(0);
      expect(metrics.failedTasks).toBe(0);
      expect(metrics.systemUptime).toBeGreaterThan(0);
    });

    it("should update metrics after agent registration", async () => {
      await orchestrator.registerAgent({
        name: "Test Agent",
        type: "worker",
        status: "active",
        capabilities: ["process"],
        metadata: {},
      });

      const metrics = await orchestrator.getSystemMetrics();
      expect(metrics.totalAgents).toBe(1);
      expect(metrics.activeAgents).toBe(1);
    });
  });
});
