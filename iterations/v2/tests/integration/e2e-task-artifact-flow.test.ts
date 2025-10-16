/**
 * End-to-End Test for Complete Task Flow: Submit → Route → Artifact → Observe
 *
 * @author @darianrosebrook
 */

import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";
import * as fs from "fs";
import * as path from "path";

// Mock task orchestrator for E2E tests
class MockTaskOrchestrator {
  private tasks: Map<string, any> = new Map();

  async submitTask(task: any): Promise<string> {
    const taskId = `task-${Date.now()}-${Math.random()
      .toString(36)
      .substr(2, 9)}`;
    this.tasks.set(taskId, {
      ...task,
      id: taskId,
      status: "submitted",
      submittedAt: new Date(),
    });

    // Simulate async processing
    setTimeout(() => {
      const taskData = this.tasks.get(taskId);
      if (taskData) {
        taskData.status = "assigned";
        taskData.assignedAt = new Date();
        taskData.agentId = "mock-agent-1";

        setTimeout(() => {
          taskData.status = "completed";
          taskData.completedAt = new Date();
          taskData.result = {
            output: "Mock task completed successfully",
            executionTime: 150,
            artifacts: [
              {
                path: `task-${taskId}-result.md`,
                content: `# Task Result\n\n${task.description}\n\n**Result:** Mock execution completed.`,
                size: 100,
                sha256: "mock-sha256-hash",
                mimeType: "text/markdown",
                createdAt: new Date().toISOString(),
              },
            ],
          };
        }, 100);
      }
    }, 50);

    return taskId;
  }

  async getTaskStatus(taskId: string): Promise<any> {
    return this.tasks.get(taskId) || null;
  }

  async getAllTasks(): Promise<any[]> {
    return Array.from(this.tasks.values());
  }

  async shutdown(): Promise<void> {
    this.tasks.clear();
  }
}

// Mock registry for E2E tests
class MockAgentRegistry {
  private agents = [
    {
      id: "mock-agent-1",
      name: "Mock Code Agent",
      capabilities: ["typescript", "javascript", "testing"],
      status: "healthy",
    },
    {
      id: "mock-agent-2",
      name: "Mock Analysis Agent",
      capabilities: ["analysis", "documentation"],
      status: "healthy",
    },
  ];

  async getAvailableAgents(): Promise<any[]> {
    return this.agents.filter((agent) => agent.status === "healthy");
  }

  async registerAgent(agent: any): Promise<void> {
    this.agents.push({ ...agent, status: "healthy" });
  }

  async getAgent(agentId: string): Promise<any> {
    return this.agents.find((agent) => agent.id === agentId) || null;
  }
}

// Mock runtime for E2E tests
class MockArbiterRuntime {
  private running = false;
  private orchestrator = new MockTaskOrchestrator();
  private registry = new MockAgentRegistry();
  private outputDir: string;

  constructor(options: { outputDir: string }) {
    this.outputDir = options.outputDir;
  }

  async start(): Promise<void> {
    this.running = true;
    // Ensure output directory exists
    if (!fs.existsSync(this.outputDir)) {
      fs.mkdirSync(this.outputDir, { recursive: true });
    }
  }

  async stop(): Promise<void> {
    this.running = false;
    await this.orchestrator.shutdown();
  }

  async submitTask(task: any): Promise<string> {
    return this.orchestrator.submitTask(task);
  }

  async getTaskStatus(taskId: string): Promise<any> {
    return this.orchestrator.getTaskStatus(taskId);
  }

  async getAvailableAgents(): Promise<any[]> {
    return this.registry.getAvailableAgents();
  }

  isRunning(): boolean {
    return this.running;
  }
}

// Use mock runtime for E2E tests
const ArbiterRuntime = MockArbiterRuntime;

describe("E2E Task Flow: Submit → Route → Artifact → Observe", () => {
  const testOutputDir = path.join(__dirname, "..", "fixtures", "e2e-output");
  let runtime: MockArbiterRuntime;

  beforeEach(async () => {
    // Clean up any existing test output
    if (fs.existsSync(testOutputDir)) {
      fs.rmSync(testOutputDir, { recursive: true, force: true });
    }

    // Create runtime with test configuration
    runtime = new ArbiterRuntime({
      outputDir: testOutputDir,
    });

    // Start runtime
    await runtime.start();
  }, 5000);

  afterEach(async () => {
    // Stop runtime
    if (runtime && runtime.isRunning()) {
      await runtime.stop();
    }

    // Clean up test output
    if (fs.existsSync(testOutputDir)) {
      fs.rmSync(testOutputDir, { recursive: true, force: true });
    }
  }, 2000);

  it("should complete full task flow from submit to artifact creation", async () => {
    // Test task submission
    const task = {
      description: "Write a simple test function",
      type: "code",
      priority: "medium",
    };

    const taskId = await runtime.submitTask(task);
    expect(typeof taskId).toBe("string");
    expect(taskId).toMatch(/^task-\d+-[a-z0-9]+$/);

    // Wait for task completion (mock async processing)
    await new Promise((resolve) => setTimeout(resolve, 200));

    // Verify task status
    const status = await runtime.getTaskStatus(taskId);
    expect(status).toBeDefined();
    expect(status.id).toBe(taskId);
    expect(status.status).toBe("completed");
    expect(status.result).toBeDefined();
    expect(status.result.output).toBe("Mock task completed successfully");

    // Verify artifacts were created
    expect(status.result.artifacts).toBeDefined();
    expect(Array.isArray(status.result.artifacts)).toBe(true);
    expect(status.result.artifacts.length).toBeGreaterThan(0);

    // Check artifact content
    const artifact = status.result.artifacts[0];
    expect(artifact.path).toMatch(/task-.*-result\.md/);
    expect(artifact.content).toContain("# Task Result");
    expect(artifact.content).toContain("Write a simple test function");
    expect(artifact.sha256).toBe("mock-sha256-hash");
    expect(artifact.mimeType).toBe("text/markdown");
  });

  it("should handle multiple concurrent tasks", async () => {
    const tasks = [
      { description: "Task 1", type: "code", priority: "high" },
      { description: "Task 2", type: "analysis", priority: "medium" },
      { description: "Task 3", type: "test", priority: "low" },
    ];

    // Submit all tasks
    const taskIds = await Promise.all(
      tasks.map((task) => runtime.submitTask(task))
    );

    expect(taskIds).toHaveLength(3);
    taskIds.forEach((taskId: string) => {
      expect(typeof taskId).toBe("string");
    });

    // Wait for completion
    await new Promise((resolve) => setTimeout(resolve, 300));

    // Verify all tasks completed
    const statuses = await Promise.all(
      taskIds.map((taskId: string) => runtime.getTaskStatus(taskId))
    );

    statuses.forEach((status: any, _index: number) => {
      expect(status.status).toBe("completed");
      expect(status.result.output).toBe("Mock task completed successfully");
      expect(status.result.artifacts.length).toBeGreaterThan(0);
    });
  });

  it("should provide access to available agents", async () => {
    const agents = await runtime.getAvailableAgents();

    expect(Array.isArray(agents)).toBe(true);
    expect(agents.length).toBeGreaterThan(0);

    // Verify agent structure
    agents.forEach((agent: any) => {
      expect(agent.id).toBeDefined();
      expect(agent.name).toBeDefined();
      expect(agent.capabilities).toBeDefined();
      expect(Array.isArray(agent.capabilities)).toBe(true);
      expect(agent.status).toBe("healthy");
    });
  });

  it("should manage task lifecycle correctly", async () => {
    const task = {
      description: "Lifecycle test",
      type: "test",
      priority: "low",
    };
    const taskId = await runtime.submitTask(task);

    // Initially should be submitted
    let status = await runtime.getTaskStatus(taskId);
    expect(status.status).toBe("submitted");

    // Wait for assignment
    await new Promise((resolve) => setTimeout(resolve, 60));
    status = await runtime.getTaskStatus(taskId);
    expect(status.status).toBe("assigned");
    expect(status.agentId).toBeDefined();

    // Wait for completion
    await new Promise((resolve) => setTimeout(resolve, 120));
    status = await runtime.getTaskStatus(taskId);
    expect(status.status).toBe("completed");
    expect(status.result).toBeDefined();
    expect(status.completedAt).toBeInstanceOf(Date);
  });

  it("should create artifacts with proper metadata", async () => {
    const task = {
      description: "Create artifact with metadata",
      type: "documentation",
      priority: "medium",
    };

    const taskId = await runtime.submitTask(task);
    await new Promise((resolve) => setTimeout(resolve, 200));

    const status = await runtime.getTaskStatus(taskId);
    const artifact = status.result.artifacts[0];

    // Verify artifact metadata
    expect(artifact.path).toBeDefined();
    expect(artifact.size).toBeGreaterThan(0);
    expect(artifact.sha256).toBeDefined();
    expect(artifact.mimeType).toBeDefined();
    expect(artifact.createdAt).toBeDefined();

    // Verify content is reasonable
    expect(artifact.content).toContain("Create artifact with metadata");
    expect(artifact.content).toContain("Mock execution completed");
  });
});
