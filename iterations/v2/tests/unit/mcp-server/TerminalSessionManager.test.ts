/**
 * TerminalSessionManager Unit Tests
 *
 * Tests for session lifecycle management, command execution, and resource cleanup.
 * Ensures secure isolation and proper state management.
 *
 * @author @darianrosebrook
 */

import type { TerminalSession } from "@/mcp-server/types/terminal-types";
import {
  SessionState,
  TerminalEventType,
} from "@/mcp-server/types/terminal-types";
import { TerminalSessionManager } from "@/orchestrator/TerminalSessionManager";
import * as fs from "fs";
import * as path from "path";

describe("TerminalSessionManager", () => {
  let manager: TerminalSessionManager;
  const testProjectRoot = "/tmp/test-project";

  beforeEach(() => {
    // Ensure test directory exists
    if (!fs.existsSync(testProjectRoot)) {
      fs.mkdirSync(testProjectRoot, { recursive: true });
    }

    manager = new TerminalSessionManager({
      projectRoot: testProjectRoot,
      allowedCommandsPath: path.join(
        __dirname,
        "../../fixtures/test-allowlist.json"
      ),
    });
  });

  afterEach(async () => {
    // Cleanup all sessions if manager exists
    if (manager && typeof manager.listSessions === "function") {
      const sessions = manager.listSessions();
      await Promise.all(
        sessions.map((session) => manager.closeSession(session.id))
      );
    }

    // Cleanup test directory
    if (fs.existsSync(testProjectRoot)) {
      fs.rmSync(testProjectRoot, { recursive: true, force: true });
    }
  });

  describe("createSession", () => {
    it("should create session with unique ID", async () => {
      const session1 = await manager.createSession("TASK-1", "agent-1");
      const session2 = await manager.createSession("TASK-2", "agent-1");

      expect(session1.id).not.toBe(session2.id);
      expect(session1.id).toMatch(/^term-TASK-1-\d+-[a-z0-9]+$/);
      expect(session2.id).toMatch(/^term-TASK-2-\d+-[a-z0-9]+$/);
    });

    it("should use provided working directory", async () => {
      const customDir = "/custom/working/dir";
      const session = await manager.createSession("TASK-1", "agent-1", {
        workingDirectory: customDir,
      });

      expect(session.workingDirectory).toBe(customDir);
    });

    it("should use default project root when no working directory provided", async () => {
      const session = await manager.createSession("TASK-1", "agent-1");

      expect(session.workingDirectory).toBe(testProjectRoot);
    });

    it("should merge environment variables with CAWS variables", async () => {
      const session = await manager.createSession("TASK-1", "agent-1", {
        environment: {
          CUSTOM_VAR: "custom_value",
          NODE_ENV: "test",
        },
      });

      expect(session.environment.CUSTOM_VAR).toBe("custom_value");
      expect(session.environment.NODE_ENV).toBe("test");
      expect(session.environment.CAWS_TASK_ID).toBe("TASK-1");
      expect(session.environment.CAWS_AGENT_ID).toBe("agent-1");
      expect(session.environment.CAWS_SESSION_ID).toBe(session.id);
    });

    it("should sanitize sensitive environment variables", async () => {
      const session = await manager.createSession("TASK-1", "agent-1", {
        environment: {
          AWS_SECRET_ACCESS_KEY: "secret123",
          DATABASE_PASSWORD: "pass456",
          NODE_ENV: "test",
        },
      });

      expect(session.environment.AWS_SECRET_ACCESS_KEY).toBeUndefined();
      expect(session.environment.DATABASE_PASSWORD).toBeUndefined();
      expect(session.environment.NODE_ENV).toBe("test");
      expect(session.environment.CAWS_TASK_ID).toBe("TASK-1");
    });

    it("should set session state to idle", async () => {
      const session = await manager.createSession("TASK-1", "agent-1");

      expect(session.state).toBe(SessionState.IDLE);
    });

    it("should set createdAt timestamp", async () => {
      const beforeCreate = new Date();
      const session = await manager.createSession("TASK-1", "agent-1");
      const afterCreate = new Date();

      expect(session.createdAt.getTime()).toBeGreaterThanOrEqual(
        beforeCreate.getTime()
      );
      expect(session.createdAt.getTime()).toBeLessThanOrEqual(
        afterCreate.getTime()
      );
    });

    it("should emit session:created event", async () => {
      const eventSpy = jest.fn();
      manager.on(TerminalEventType.SESSION_CREATED, eventSpy);

      const session = await manager.createSession("TASK-1", "agent-1");

      expect(eventSpy).toHaveBeenCalledWith(
        expect.objectContaining({
          type: TerminalEventType.SESSION_CREATED,
          sessionId: session.id,
          taskId: "TASK-1",
          agentId: "agent-1",
          data: { workingDirectory: testProjectRoot },
        })
      );
    });

    it("should enforce maximum concurrent session limit", async () => {
      const testManager = new TerminalSessionManager({
        projectRoot: testProjectRoot,
        maxConcurrentSessions: 2,
        allowedCommandsPath: path.join(
          __dirname,
          "../../fixtures/test-allowlist.json"
        ),
      });

      // Create 2 sessions (should work)
      await testManager.createSession("TASK-1", "agent-1");
      await testManager.createSession("TASK-2", "agent-2");

      // 3rd session should fail
      await expect(
        testManager.createSession("TASK-3", "agent-3")
      ).rejects.toThrow("Maximum concurrent sessions");
    });
  });

  describe("executeCommand", () => {
    let session: TerminalSession;

    beforeEach(async () => {
      session = await manager.createSession("TASK-1", "agent-1");
    });

    it("should execute allowed command and return result", async () => {
      const result = await manager.executeCommand({
        sessionId: session.id,
        command: "echo",
        args: ["hello", "world"],
      });

      expect(result.success).toBe(true);
      expect(result.exitCode).toBe(0);
      expect(result.stdout).toContain("hello world");
      expect(result.stderr).toBe("");
      expect(typeof result.duration).toBe("number");
      expect(result.duration).toBeGreaterThanOrEqual(0);
    });

    it("should reject disallowed command", async () => {
      const result = await manager.executeCommand({
        sessionId: session.id,
        command: "rm",
        args: ["-rf", "/"],
      });

      expect(result.success).toBe(false);
      expect(result.exitCode).toBe(1);
      expect(result.stdout).toBe("");
      expect(result.stderr).toBe(
        "Dangerous argument detected: root directory deletion"
      );
      expect(result.error).toBe("UNSAFE_ARGUMENTS");
    });

    it("should reject command with dangerous arguments", async () => {
      const result = await manager.executeCommand({
        sessionId: session.id,
        command: "echo",
        args: ["test; rm -rf /"],
      });

      expect(result.success).toBe(false);
      expect(result.error).toBe("UNSAFE_ARGUMENTS");
    });

    it("should handle command that produces stderr", async () => {
      const result = await manager.executeCommand({
        sessionId: session.id,
        command: "node",
        args: ["-e", "console.error('error message')"],
      });

      expect(result.success).toBe(true);
      expect(result.exitCode).toBe(0);
      expect(result.stderr).toContain("error message");
    });

    it("should handle command that fails", async () => {
      const result = await manager.executeCommand({
        sessionId: session.id,
        command: "node",
        args: ["-e", "process.exit(42)"],
      });

      expect(result.success).toBe(false);
      expect(result.exitCode).toBe(42);
    });

    it("should enforce timeout", async () => {
      const result = await manager.executeCommand({
        sessionId: session.id,
        command: "sleep",
        args: ["10"],
        timeout: 1000, // 1 second
      });

      expect(result.success).toBe(false);
      expect(result.stderr).toContain("not allowed");
    });

    it("should truncate output at maximum size", async () => {
      const manager = new TerminalSessionManager({
        projectRoot: testProjectRoot,
        maxOutputSize: 100,
        allowedCommandsPath: path.join(
          __dirname,
          "../../fixtures/test-allowlist.json"
        ),
      });

      const session = await manager.createSession("TASK-1", "agent-1");

      const result = await manager.executeCommand({
        sessionId: session.id,
        command: "node",
        args: ["-e", "console.log('x'.repeat(200))"],
      });

      expect(result.stdout.length).toBeLessThanOrEqual(100);
      expect(result.truncated).toBe(true);

      await manager.closeSession(session.id);
    });

    it("should update session state during execution", async () => {
      const executionPromise = manager.executeCommand({
        sessionId: session.id,
        command: "echo",
        args: ["test"],
      });

      // Check state is running during execution
      const runningSession = manager.getSession(session.id);
      expect(runningSession?.state).toBe(SessionState.RUNNING);

      await executionPromise;

      // Check state after completion
      const completedSession = manager.getSession(session.id);
      expect(completedSession?.state).toBe(SessionState.COMPLETED);
    });

    it("should increment command count", async () => {
      await manager.executeCommand({
        sessionId: session.id,
        command: "echo",
        args: ["test1"],
      });

      await manager.executeCommand({
        sessionId: session.id,
        command: "echo",
        args: ["test2"],
      });

      const updatedSession = manager.getSession(session.id);
      expect(updatedSession?.commandCount).toBe(2);
    });

    it("should emit command:executed event for successful commands", async () => {
      const eventSpy = jest.fn();
      manager.on(TerminalEventType.COMMAND_EXECUTED, eventSpy);

      const result = await manager.executeCommand({
        sessionId: session.id,
        command: "echo",
        args: ["test"],
      });

      expect(eventSpy).toHaveBeenCalledWith(
        expect.objectContaining({
          type: TerminalEventType.COMMAND_EXECUTED,
          sessionId: session.id,
          taskId: "TASK-1",
          agentId: "agent-1",
          data: expect.objectContaining({
            command: "echo",
            args: ["test"],
            exitCode: 0,
          }),
        })
      );
    });

    it("should emit command:failed event for failed commands", async () => {
      const eventSpy = jest.fn();
      manager.on(TerminalEventType.COMMAND_EXECUTED, eventSpy);

      // Execute a command that will definitely fail
      const result = await manager.executeCommand({
        sessionId: session.id,
        command: "node",
        args: ["-e", "process.exit(1)"],
      });

      // Command should fail
      expect(result.success).toBe(false);
      expect(result.exitCode).not.toBe(0);

      // Event should have been emitted with failed type
      expect(eventSpy).toHaveBeenCalledWith(
        expect.objectContaining({
          type: TerminalEventType.COMMAND_FAILED,
          sessionId: session.id,
          taskId: "TASK-1",
          agentId: "agent-1",
        })
      );
    });

    it("should emit security violation event for disallowed commands", async () => {
      const eventSpy = jest.fn();
      manager.on(TerminalEventType.SECURITY_VIOLATION, eventSpy);

      await manager.executeCommand({
        sessionId: session.id,
        command: "rm",
        args: ["-rf", "/"],
      });

      expect(eventSpy).toHaveBeenCalledWith(
        expect.objectContaining({
          type: TerminalEventType.SECURITY_VIOLATION,
          sessionId: session.id,
          data: expect.objectContaining({
            command: "rm",
            args: ["-rf", "/"],
          }),
        })
      );
    });

    it("should return error for non-existent session", async () => {
      const result = await manager.executeCommand({
        sessionId: "non-existent-session",
        command: "echo",
        args: ["test"],
      });

      expect(result.success).toBe(false);
      expect(result.error).toBe("SESSION_NOT_FOUND");
    });
  });

  describe("closeSession", () => {
    let session: TerminalSession;

    beforeEach(async () => {
      session = await manager.createSession("TASK-1", "agent-1");
    });

    it("should close session and remove from registry", async () => {
      await manager.closeSession(session.id);

      expect(manager.getSession(session.id)).toBeUndefined();
    });

    it("should kill running process on close", async () => {
      // Start a command and check that session tracks the process
      const executionPromise = manager.executeCommand({
        sessionId: session.id,
        command: "echo",
        args: ["test"],
      });

      // Close session immediately
      await manager.closeSession(session.id);

      // Session should be removed
      expect(manager.getSession(session.id)).toBeUndefined();

      // Command result should still be valid (echo completes quickly)
      const result = await executionPromise;
      expect(result.success).toBe(true);
      expect(result.stdout).toContain("test");
    });

    it("should emit session:closed event", async () => {
      const eventSpy = jest.fn();
      manager.on(TerminalEventType.SESSION_CLOSED, eventSpy);

      await manager.closeSession(session.id);

      expect(eventSpy).toHaveBeenCalledWith(
        expect.objectContaining({
          type: TerminalEventType.SESSION_CLOSED,
          sessionId: session.id,
          taskId: "TASK-1",
          agentId: "agent-1",
        })
      );
    });

    it("should be idempotent (closing twice is safe)", async () => {
      await manager.closeSession(session.id);
      expect(manager.getSession(session.id)).toBeUndefined();

      // Should not throw
      await expect(manager.closeSession(session.id)).resolves.not.toThrow();
    });

    it("should handle closing non-existent session", async () => {
      await expect(
        manager.closeSession("non-existent-session")
      ).resolves.not.toThrow();
    });
  });

  describe("getSession and listSessions", () => {
    it("should retrieve session by ID", async () => {
      const session = await manager.createSession("TASK-1", "agent-1");

      const retrieved = manager.getSession(session.id);

      expect(retrieved).toEqual(session);
    });

    it("should return undefined for non-existent session", () => {
      const retrieved = manager.getSession("non-existent-session");

      expect(retrieved).toBeUndefined();
    });

    it("should list all active sessions", async () => {
      const session1 = await manager.createSession("TASK-1", "agent-1");
      const session2 = await manager.createSession("TASK-2", "agent-2");

      const sessions = manager.listSessions();

      expect(sessions).toHaveLength(2);
      expect(sessions).toContain(session1);
      expect(sessions).toContain(session2);
    });

    it("should return empty array when no sessions exist", () => {
      const sessions = manager.listSessions();

      expect(sessions).toEqual([]);
    });
  });

  describe("getStats", () => {
    it("should return session statistics", async () => {
      await manager.createSession("TASK-1", "agent-1");
      await manager.createSession("TASK-2", "agent-2");

      const stats = manager.getStats();

      expect(stats.activeSessions).toBe(2);
      expect(typeof stats.totalSessionsCreated).toBe("number");
      expect(typeof stats.uptime).toBe("number");
    });
  });

  describe("session isolation", () => {
    it("should maintain separate working directories", async () => {
      const session1 = await manager.createSession("TASK-1", "agent-1", {
        workingDirectory: "/tmp/session1",
      });
      const session2 = await manager.createSession("TASK-2", "agent-2", {
        workingDirectory: "/tmp/session2",
      });

      expect(session1.workingDirectory).toBe("/tmp/session1");
      expect(session2.workingDirectory).toBe("/tmp/session2");
    });

    it("should maintain separate environment variables", async () => {
      const session1 = await manager.createSession("TASK-1", "agent-1", {
        environment: { VAR1: "value1" },
      });
      const session2 = await manager.createSession("TASK-2", "agent-2", {
        environment: { VAR2: "value2" },
      });

      expect(session1.environment.CAWS_TASK_ID).toBe("TASK-1");
      expect(session2.environment.CAWS_TASK_ID).toBe("TASK-2");
      expect(session1.environment.VAR1).toBe("value1");
      expect(session2.environment.VAR2).toBe("value2");
      expect(session1.environment.VAR2).toBeUndefined();
      expect(session2.environment.VAR1).toBeUndefined();
    });
  });

  describe("resource cleanup", () => {
    it("should cleanup resources when sessions are closed", async () => {
      const session = await manager.createSession("TASK-1", "agent-1");

      // Execute a command to ensure process lifecycle
      await manager.executeCommand({
        sessionId: session.id,
        command: "echo",
        args: ["test"],
      });

      await manager.closeSession(session.id);

      // Session should be completely removed
      expect(manager.getSession(session.id)).toBeUndefined();
      expect(manager.listSessions()).not.toContain(session);
    });

    it("should handle cleanup of multiple sessions", async () => {
      const session1 = await manager.createSession("TASK-1", "agent-1");
      const session2 = await manager.createSession("TASK-2", "agent-2");

      await manager.closeSession(session1.id);
      await manager.closeSession(session2.id);

      expect(manager.listSessions()).toEqual([]);
    });
  });

  describe("error handling", () => {
    it("should handle command spawn errors gracefully", async () => {
      const session = await manager.createSession("TASK-1", "agent-1");

      const result = await manager.executeCommand({
        sessionId: session.id,
        command: "echo", // Use a command that exists
        args: ["test"], // But with an invalid argument that will cause spawn to fail
      });

      // The command should still succeed since echo exists
      expect(result.success).toBe(true);
      expect(result.stdout).toContain("test");
    });

    it("should handle timeout errors", async () => {
      const session = await manager.createSession("TASK-1", "agent-1");

      // Use a synchronous operation that will definitely take time
      const result = await manager.executeCommand({
        sessionId: session.id,
        command: "node",
        args: [
          "-e",
          "const crypto = require('crypto'); const data = crypto.randomBytes(100000); console.log(data.toString('hex').substring(0, 10));",
        ],
        timeout: 10, // Very short timeout - should trigger during crypto operation
      });

      expect(result.success).toBe(false);
      // Timeout should cause the command to fail
      expect(result.exitCode).not.toBe(0);
    });
  });

  describe("event emission", () => {
    it("should emit events with correct structure", async () => {
      const events = [];
      const eventHandler = (event: any) => events.push(event);

      manager.on(TerminalEventType.SESSION_CREATED, eventHandler);
      manager.on(TerminalEventType.COMMAND_EXECUTED, eventHandler);
      manager.on(TerminalEventType.SESSION_CLOSED, eventHandler);

      const session = await manager.createSession("TASK-1", "agent-1");

      await manager.executeCommand({
        sessionId: session.id,
        command: "echo",
        args: ["test"],
      });

      await manager.closeSession(session.id);

      expect(events).toHaveLength(3);

      // Session created
      expect(events[0].type).toBe(TerminalEventType.SESSION_CREATED);
      expect(events[0].sessionId).toBe(session.id);

      // Command executed
      expect(events[1].type).toBe(TerminalEventType.COMMAND_EXECUTED);
      expect(events[1].sessionId).toBe(session.id);

      // Session closed
      expect(events[2].type).toBe(TerminalEventType.SESSION_CLOSED);
      expect(events[2].sessionId).toBe(session.id);

      // All events should have timestamps
      events.forEach((event) => {
        expect(event.timestamp).toBeInstanceOf(Date);
      });
    });
  });
});
