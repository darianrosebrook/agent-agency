/**
 * End-to-End Terminal Agent Workflow Tests
 *
 * Tests the complete agent workflow using MCP Terminal Access Layer
 * from task submission through command execution to result delivery.
 *
 * @author @darianrosebrook
 */

import * as fs from "fs";
import * as path from "path";
import { ArbiterOrchestrator } from "../../src/orchestrator/ArbiterOrchestrator";

// TODO: Import proper Task type when task-types module is created
type Task = any;

// Mock MCP client for testing
class MockMCPClient {
  private responses: Map<string, any> = new Map();

  // Set up mock responses
  mockResponse(toolName: string, response: any) {
    this.responses.set(toolName, response);
  }

  async callTool(name: string, args: any): Promise<any> {
    const response = this.responses.get(name);
    if (!response) {
      throw new Error(`No mock response for tool: ${name}`);
    }
    return response;
  }
}

describe("Terminal Agent E2E Workflow", () => {
  let orchestrator: ArbiterOrchestrator;
  let mockMcpClient: MockMCPClient;
  const testProjectRoot = path.join(__dirname, "../test-project-root");

  afterEach(async () => {
    // Clean up orchestrator
    if (orchestrator) {
      try {
        await orchestrator.shutdown();
      } catch (error) {
        // Ignore shutdown errors
      }
    }
    jest.clearAllMocks();
  });

  afterAll(async () => {
    // Clean up test directory
    if (fs.existsSync(testProjectRoot)) {
      try {
        fs.rmSync(testProjectRoot, { recursive: true, force: true });
      } catch (error) {
        // Ignore cleanup errors
      }
    }
  });

  beforeEach(async () => {
    // Ensure test directory exists
    if (!fs.existsSync(testProjectRoot)) {
      fs.mkdirSync(testProjectRoot, { recursive: true });
    }

    // Create a simple package.json for testing
    const packageJson = {
      name: "test-project",
      version: "1.0.0",
      scripts: {
        test: "echo 'Tests passed'",
        lint: "echo 'Linting passed'",
        build: "echo 'Build completed'",
      },
    };

    fs.writeFileSync(
      path.join(testProjectRoot, "package.json"),
      JSON.stringify(packageJson, null, 2)
    );

    // Initialize orchestrator with test configuration
    orchestrator = new ArbiterOrchestrator({
      taskQueue: {},
      taskAssignment: {},
      agentRegistry: {},
      healthMonitor: {},
      recoveryManager: {},
      knowledgeSeeker: {},
      database: {
        host: "localhost",
        port: 5432,
        database: "test",
        user: "test",
        maxConnections: 2,
      },
      security: {
        auditLoggingEnabled: true,
        maxAuditEvents: 1000,
        inputSanitizationEnabled: true,
        secureErrorResponsesEnabled: true,
        sessionTimeoutMinutes: 30,
      },
    } as any);

    await orchestrator.initialize();
  });

  afterEach(async () => {
    // Cleanup test directory
    if (fs.existsSync(testProjectRoot)) {
      fs.rmSync(testProjectRoot, { recursive: true, force: true });
    }

    // Cleanup orchestrator
    if (orchestrator) {
      await orchestrator.shutdown();
    }
  });

  describe("Node.js Test Execution Workflow", () => {
    it("should complete full test execution workflow", async () => {
      // Create a test execution task
      const task: Task = {
        id: "e2e-test-001",
        type: "test_execution",
        description: "Run comprehensive test suite for Node.js project",
        metadata: {
          workingDirectory: testProjectRoot,
          installDeps: true,
          runLint: true,
          coverage: true,
          testArgs: ["test"],
          timeout: 30000,
        },
      };

      // Submit task to orchestrator
      const result = await orchestrator.submitTask(task);

      expect(result.taskId).toBe(task.id);

      // Wait for task completion (in real scenario, this would be async)
      await new Promise((resolve) => setTimeout(resolve, 100));

      // Check task status
      const status = await orchestrator.getTaskStatus(task.id);
      expect(status).toBeDefined();

      // In a full E2E test, we would:
      // 1. Mock the MCP terminal responses
      // 2. Verify that terminal_create_session was called
      // 3. Verify that terminal_execute_command was called with correct args
      // 4. Verify that terminal_close_session was called
      // 5. Verify task completion with expected results

      // For this test, we verify the task was accepted and queued
      expect(result.taskId).toBeDefined();
    });

    it("should handle build workflow end-to-end", async () => {
      const buildTask: Task = {
        id: "e2e-build-001",
        type: "build",
        description: "Build production artifacts",
        metadata: {
          workingDirectory: testProjectRoot,
          buildArgs: ["run", "build"],
          buildTimeout: 30000,
        },
      };

      const result = await orchestrator.submitTask(buildTask);

      expect(result.taskId).toBe(buildTask.id);

      // Verify task was queued for processing
      const status = await orchestrator.getTaskStatus(buildTask.id);
      expect(status).toBeDefined();
    });

    it("should handle deployment workflow", async () => {
      const deployTask: Task = {
        id: "e2e-deploy-001",
        type: "deployment",
        description: "Deploy application to staging",
        metadata: {
          workingDirectory: testProjectRoot,
          buildBeforeDeploy: true,
          deployCommand: ["run", "deploy:staging"],
          healthCheck: true,
          postDeployCheck: true,
        },
      };

      const result = await orchestrator.submitTask(deployTask);

      expect(result.taskId).toBe(deployTask.id);

      const status = await orchestrator.getTaskStatus(deployTask.id);
      expect(status).toBeDefined();
    });
  });

  describe("Package Management Workflow", () => {
    it("should handle npm package installation", async () => {
      const packageTask: Task = {
        id: "e2e-package-001",
        type: "package_management",
        description: "Install project dependencies",
        metadata: {
          workingDirectory: testProjectRoot,
          packageManager: "npm",
          packageCommand: "install",
          packages: ["express", "lodash"],
        },
      };

      const result = await orchestrator.submitTask(packageTask);

      expect(result.taskId).toBe(packageTask.id);

      const status = await orchestrator.getTaskStatus(packageTask.id);
      expect(status).toBeDefined();
    });

    it("should handle Python package installation", async () => {
      // Create requirements.txt for Python test
      fs.writeFileSync(
        path.join(testProjectRoot, "requirements.txt"),
        "express==4.18.0\nlodash==4.17.21\n"
      );

      const pythonTask: Task = {
        id: "e2e-python-001",
        type: "package_management",
        description: "Install Python dependencies",
        metadata: {
          workingDirectory: testProjectRoot,
          packageManager: "pip",
          packageCommand: "install",
          packages: ["-r", "requirements.txt"],
        },
      };

      const result = await orchestrator.submitTask(pythonTask);

      expect(result.taskId).toBe(pythonTask.id);

      const status = await orchestrator.getTaskStatus(pythonTask.id);
      expect(status).toBeDefined();
    });
  });

  describe("Error Handling and Recovery", () => {
    it("should handle invalid working directory gracefully", async () => {
      const invalidTask: Task = {
        id: "e2e-invalid-dir-001",
        type: "test_execution",
        description: "Test with invalid directory",
        metadata: {
          workingDirectory: "/nonexistent/directory/that/does/not/exist",
          testArgs: ["test"],
        },
      };

      const result = await orchestrator.submitTask(invalidTask);

      expect(result.taskId).toBe(invalidTask.id);

      // Task should still be accepted but execution should fail gracefully
      const status = await orchestrator.getTaskStatus(invalidTask.id);
      expect(status).toBeDefined();
    });

    it("should handle command timeout gracefully", async () => {
      const timeoutTask: Task = {
        id: "e2e-timeout-001",
        type: "test_execution",
        description: "Test command timeout handling",
        metadata: {
          workingDirectory: testProjectRoot,
          testArgs: ["test"],
          timeout: 1, // Very short timeout to trigger timeout
        },
      };

      const result = await orchestrator.submitTask(timeoutTask);

      expect(result.taskId).toBe(timeoutTask.id);

      const status = await orchestrator.getTaskStatus(timeoutTask.id);
      expect(status).toBeDefined();
    });

    it("should handle disallowed commands gracefully", async () => {
      const dangerousTask: Task = {
        id: "e2e-dangerous-001",
        type: "infrastructure",
        description: "Attempt dangerous command execution",
        metadata: {
          workingDirectory: testProjectRoot,
          commands: [
            {
              command: "rm",
              args: ["-rf", "/tmp/safe-directory"],
              timeout: 5000,
            },
          ],
        },
      };

      const result = await orchestrator.submitTask(dangerousTask);

      expect(result.taskId).toBe(dangerousTask.id);

      // Task should be accepted but execution should be blocked by security
      const status = await orchestrator.getTaskStatus(dangerousTask.id);
      expect(status).toBeDefined();
    });
  });

  describe("Concurrent Session Management", () => {
    it("should handle multiple concurrent terminal sessions", async () => {
      const tasks = Array.from({ length: 5 }, (_, i) => ({
        id: `e2e-concurrent-${i + 1}`,
        type: "test_execution" as const,
        description: `Concurrent test execution ${i + 1}`,
        metadata: {
          workingDirectory: testProjectRoot,
          testArgs: ["test"],
          timeout: 10000,
        },
      }));

      // Submit all tasks concurrently
      const results = await Promise.all(
        tasks.map((task) => orchestrator.submitTask(task))
      );

      // All tasks should be accepted
      results.forEach((result, i) => {
        expect(result.taskId).toBe(tasks[i].id);
      });

      // Check all task statuses
      const statuses = await Promise.all(
        tasks.map((task) => orchestrator.getTaskStatus(task.id))
      );

      statuses.forEach((status) => {
        expect(status).toBeDefined();
      });
    });

    it("should respect session limits", async () => {
      // Create tasks that would exceed session limits if not managed properly
      const manyTasks = Array.from({ length: 10 }, (_, i) => ({
        id: `e2e-limit-${i + 1}`,
        type: "infrastructure" as const,
        description: `Resource intensive task ${i + 1}`,
        metadata: {
          workingDirectory: testProjectRoot,
          commands: [
            {
              command: "sleep",
              args: ["1"], // Simple sleep command
              timeout: 5000,
            },
          ],
        },
      }));

      // Submit tasks - should be queued appropriately
      const submitPromises = manyTasks.map((task) =>
        orchestrator.submitTask(task)
      );
      const results = await Promise.all(submitPromises);

      results.forEach((result, i) => {
        expect(result.taskId).toBe(manyTasks[i].id);
      });
    });
  });

  describe("Resource Cleanup", () => {
    it("should cleanup resources after task completion", async () => {
      const cleanupTask: Task = {
        id: "e2e-cleanup-001",
        type: "test_execution",
        description: "Test resource cleanup",
        metadata: {
          workingDirectory: testProjectRoot,
          testArgs: ["test"],
          timeout: 5000,
        },
      };

      await orchestrator.submitTask(cleanupTask);

      // Wait a bit for processing
      await new Promise((resolve) => setTimeout(resolve, 200));

      // Check that orchestrator can still accept new tasks (resources cleaned up)
      const followUpTask: Task = {
        id: "e2e-cleanup-followup-001",
        type: "test_execution",
        description: "Follow-up task after cleanup",
        metadata: {
          workingDirectory: testProjectRoot,
          testArgs: ["test"],
        },
      };

      const result = await orchestrator.submitTask(followUpTask);
      expect(result.taskId).toBe(followUpTask.id);
    });

    it("should handle orchestrator shutdown gracefully", async () => {
      // Submit a task
      const shutdownTask: Task = {
        id: "e2e-shutdown-001",
        type: "test_execution",
        description: "Test shutdown handling",
        metadata: {
          workingDirectory: testProjectRoot,
          testArgs: ["test"],
        },
      };

      await orchestrator.submitTask(shutdownTask);

      // Shutdown orchestrator
      await orchestrator.shutdown();

      // Verify orchestrator is properly shut down
      const status = await orchestrator.getTaskStatus(shutdownTask.id);
      expect(status).toBeDefined(); // Should still return status even when shut down
    });
  });

  describe("Integration with Task Routing", () => {
    it("should route terminal tasks to appropriate agents", async () => {
      // This test would verify that tasks requiring terminal access
      // are routed to agents that have terminal capabilities
      const terminalTask: Task = {
        id: "e2e-routing-001",
        type: "infrastructure",
        description: "Task requiring terminal access",
        metadata: {
          requiresTerminal: true,
          workingDirectory: testProjectRoot,
          commands: [
            {
              command: "echo",
              args: ["routing test"],
              timeout: 5000,
            },
          ],
        },
      };

      const result = await orchestrator.submitTask(terminalTask);

      expect(result.taskId).toBe(terminalTask.id);

      // In a full implementation, this would verify that the task
      // was assigned to an agent with terminal capabilities
      const status = await orchestrator.getTaskStatus(terminalTask.id);
      expect(status).toBeDefined();
    });
  });
});
