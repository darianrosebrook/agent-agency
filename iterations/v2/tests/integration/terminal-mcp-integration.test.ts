/**
 * Terminal MCP Integration Tests
 *
 * Tests the integration between MCP server and terminal tools.
 * Verifies end-to-end functionality from MCP tool calls to command execution.
 *
 * @author @darianrosebrook
 */

import { ArbiterMCPServer, callTool } from "@/mcp-server/ArbiterMCPServer";
import * as fs from "fs";
import * as path from "path";

describe("Terminal MCP Integration", () => {
  let mcpServer: ArbiterMCPServer;
  const testProjectRoot = "/tmp/test-mcp-integration";

  beforeEach(async () => {
    // Ensure test directory exists
    if (!fs.existsSync(testProjectRoot)) {
      fs.mkdirSync(testProjectRoot, { recursive: true });
    }

    // Create test allowlist
    const allowlistPath = path.join(
      testProjectRoot,
      "apps/tools/caws/tools-allow.json"
    );
    if (!fs.existsSync(path.dirname(allowlistPath))) {
      fs.mkdirSync(path.dirname(allowlistPath), { recursive: true });
    }
    fs.writeFileSync(allowlistPath, JSON.stringify(["echo", "node"]));

    // Initialize MCP server
    mcpServer = new ArbiterMCPServer(testProjectRoot);
  });

  afterEach(async () => {
    // Cleanup test directory
    if (fs.existsSync(testProjectRoot)) {
      fs.rmSync(testProjectRoot, { recursive: true, force: true });
    }
  });

  describe("Tool Registration", () => {
    it("should register all terminal tools", async () => {
      // Mock the listTools method since it's not directly available on ArbiterMCPServer
      const tools = { tools: mcpServer["tools"] };

      const terminalToolNames = [
        "terminal_create_session",
        "terminal_execute_command",
        "terminal_close_session",
        "terminal_get_status",
      ];

      terminalToolNames.forEach((toolName) => {
        expect(tools.tools).toContainEqual(
          expect.objectContaining({ name: toolName })
        );
      });
    });

    it("should have correct tool schemas", async () => {
      // Mock the listTools method since it's not directly available on ArbiterMCPServer
      const tools = { tools: mcpServer["tools"] };

      const createSessionTool = tools.tools.find(
        (t: any) => t.name === "terminal_create_session"
      );

      expect(createSessionTool).toBeDefined();
      expect(createSessionTool.inputSchema.required).toEqual([
        "taskId",
        "agentId",
      ]);
      expect(createSessionTool.inputSchema.properties.taskId.type).toBe(
        "string"
      );
      expect(createSessionTool.inputSchema.properties.agentId.type).toBe(
        "string"
      );
    });
  });

  describe("End-to-End Terminal Workflow", () => {
    it("should create session, execute command, and close session", async () => {
      // Create session
      const createResponse = await callTool(mcpServer, {
        name: "terminal_create_session",
        arguments: {
          taskId: "MCP-INT-001",
          agentId: "test-agent",
        },
      });

      expect(createResponse.content[0].type).toBe("text");
      const createResult = JSON.parse(createResponse.content[0].text);
      expect(createResult.success).toBe(true);
      expect(createResult.sessionId).toBeDefined();
      const sessionId = createResult.sessionId;

      // Execute command
      const execResponse = await callTool(mcpServer, {
        name: "terminal_execute_command",
        arguments: {
          sessionId,
          command: "echo",
          args: ["hello", "world"],
        },
      });

      const execResult = JSON.parse(execResponse.content[0].text);
      expect(execResult.success).toBe(true);
      expect(execResult.exitCode).toBe(0);
      expect(execResult.stdout).toContain("hello world");
      expect(execResult.stderr).toBe("");

      // Get status
      const statusResponse = await callTool(mcpServer, {
        name: "terminal_get_status",
        arguments: {
          sessionId,
        },
      });

      const statusResult = JSON.parse(statusResponse.content[0].text);
      expect(statusResult.success).toBe(true);
      expect(statusResult.session.id).toBe(sessionId);
      expect(statusResult.session.taskId).toBe("MCP-INT-001");
      expect(statusResult.session.agentId).toBe("test-agent");
      expect(statusResult.session.commandCount).toBe(1);

      // Close session
      const closeResponse = await callTool(mcpServer, {
        name: "terminal_close_session",
        arguments: {
          sessionId,
        },
      });

      const closeResult = JSON.parse(closeResponse.content[0].text);
      expect(closeResult.success).toBe(true);

      // Verify session is gone
      const finalStatusResponse = await callTool(mcpServer, {
        name: "terminal_get_status",
        arguments: {
          sessionId,
        },
      });

      const finalStatusResult = JSON.parse(finalStatusResponse.content[0].text);
      expect(finalStatusResult.success).toBe(false);
      expect(finalStatusResult.error).toBe("SESSION_NOT_FOUND");
    });

    it("should reject disallowed commands", async () => {
      // Create session
      const createResponse = await callTool(mcpServer, {
        name: "terminal_create_session",
        arguments: {
          taskId: "MCP-INT-002",
          agentId: "test-agent",
        },
      });

      const createResult = JSON.parse(createResponse.content[0].text);
      const sessionId = createResult.sessionId;

      // Try to execute disallowed command
      const execResponse = await callTool(mcpServer, {
        name: "terminal_execute_command",
        arguments: {
          sessionId,
          command: "rm",
          args: ["-rf", "/"],
        },
      });

      const execResult = JSON.parse(execResponse.content[0].text);
      expect(execResult.success).toBe(false);
      expect(execResult.error).toBe("UNSAFE_ARGUMENTS");
    });

    it("should handle session not found errors", async () => {
      const response = await callTool(mcpServer, {
        name: "terminal_execute_command",
        arguments: {
          sessionId: "non-existent-session",
          command: "echo",
          args: ["test"],
        },
      });

      const result = JSON.parse(response.content[0].text);
      expect(result.success).toBe(false);
      expect(result.error).toBe("SESSION_NOT_FOUND");
    });

    it("should handle invalid parameters", async () => {
      // Missing taskId
      const response = await callTool(mcpServer, {
        name: "terminal_create_session",
        arguments: {
          agentId: "test-agent",
        },
      });

      const result = JSON.parse(response.content[0].text);
      expect(result.success).toBe(false);
      expect(result.error).toBe("INVALID_PARAMETERS");
    });
  });

  describe("Concurrent Sessions", () => {
    it("should support multiple concurrent sessions", async () => {
      // Create multiple sessions
      const sessions = [];
      for (let i = 0; i < 3; i++) {
        const response = await callTool(mcpServer, {
          name: "terminal_create_session",
          arguments: {
            taskId: `MCP-CONC-${i}`,
            agentId: `agent-${i}`,
          },
        });

        const result = JSON.parse(response.content[0].text);
        sessions.push(result.sessionId);
      }

      expect(sessions).toHaveLength(3);
      sessions.forEach((sessionId) => {
        expect(sessionId).toBeDefined();
      });

      // Execute commands in different sessions
      const results = await Promise.all(
        sessions.map((sessionId, i) =>
          callTool(mcpServer, {
            name: "terminal_execute_command",
            arguments: {
              sessionId,
              command: "echo",
              args: [`session-${i}`],
            },
          })
        )
      );

      results.forEach((response: any, i: number) => {
        const result = JSON.parse(response.content[0].text);
        expect(result.success).toBe(true);
        expect(result.stdout).toContain(`session-${i}`);
      });

      // Cleanup all sessions
      await Promise.all(
        sessions.map((sessionId) =>
          callTool(mcpServer, {
            name: "terminal_close_session",
            arguments: { sessionId },
          })
        )
      );
    });
  });

  describe("Error Handling", () => {
    it("should handle MCP server errors gracefully", async () => {
      // Try to execute without creating session first
      const response = await callTool(mcpServer, {
        name: "terminal_execute_command",
        arguments: {
          sessionId: "invalid-session",
          command: "echo",
          args: ["test"],
        },
      });

      const result = JSON.parse(response.content[0].text);
      expect(result.success).toBe(false);
      expect(result.error).toBe("SESSION_NOT_FOUND");
    });

    it("should validate tool arguments", async () => {
      const response = await callTool(mcpServer, {
        name: "terminal_create_session",
        arguments: {
          // Missing required fields
          workingDirectory: "/tmp",
        },
      });

      const result = JSON.parse(response.content[0].text);
      expect(result.success).toBe(false);
      expect(result.error).toBe("INVALID_PARAMETERS");
    });
  });
});
