/**
 * Terminal Tools Definitions
 *
 * MCP tool definitions for terminal access functionality.
 * Defines the tools that agents can use to interact with terminal sessions.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


/**
 * Terminal tool definitions for MCP server registration
 */
export const TERMINAL_TOOLS = [
  {
    name: "terminal_create_session",
    description:
      "Create a new terminal session for task execution with isolated environment and working directory",
    inputSchema: {
      type: "object",
      required: ["taskId", "agentId"],
      properties: {
        taskId: {
          type: "string",
          description: "Task ID this session is associated with",
          example: "TASK-001",
        },
        agentId: {
          type: "string",
          description: "Agent ID creating the session",
          example: "agent-worker-1",
        },
        workingDirectory: {
          type: "string",
          description:
            "Working directory for command execution (optional, defaults to project root)",
          example: "/workspace/iterations/v2",
        },
        environment: {
          type: "object",
          additionalProperties: {
            type: "string",
          },
          description: "Additional environment variables (optional)",
          example: {
            NODE_ENV: "test",
            CI: "true",
          },
        },
      },
    },
  },
  {
    name: "terminal_execute_command",
    description:
      "Execute a validated command in an existing terminal session with timeout and output limits",
    inputSchema: {
      type: "object",
      required: ["sessionId", "command"],
      properties: {
        sessionId: {
          type: "string",
          description: "Terminal session ID from terminal_create_session",
          example: "term-TASK-001-1697123456789",
        },
        command: {
          type: "string",
          description:
            "Command to execute (must be in allowlist: npm, git, node, echo, cat, ls, sleep)",
          example: "npm",
        },
        args: {
          type: "array",
          items: {
            type: "string",
          },
          description: "Command arguments (optional, validated for security)",
          example: ["test", "--coverage"],
        },
        timeout: {
          type: "integer",
          description:
            "Execution timeout in milliseconds (optional, default: 60000, max: 300000)",
          minimum: 1000,
          maximum: 300000,
          example: 120000,
        },
      },
    },
  },
  {
    name: "terminal_close_session",
    description:
      "Close a terminal session and cleanup all resources, killing any running processes",
    inputSchema: {
      type: "object",
      required: ["sessionId"],
      properties: {
        sessionId: {
          type: "string",
          description: "Session ID to close",
          example: "term-TASK-001-1697123456789",
        },
      },
    },
  },
  {
    name: "terminal_get_status",
    description: "Get current status and metadata for a terminal session",
    inputSchema: {
      type: "object",
      required: ["sessionId"],
      properties: {
        sessionId: {
          type: "string",
          description: "Session ID to query",
          example: "term-TASK-001-1697123456789",
        },
      },
    },
  },
];

/**
 * Terminal tool names type for type safety
 */
export type TerminalToolName =
  | "terminal_create_session"
  | "terminal_execute_command"
  | "terminal_close_session"
  | "terminal_get_status";

/**
 * Validate that a tool name is a valid terminal tool
 */
export function isTerminalTool(name: string): name is TerminalToolName {
  return TERMINAL_TOOLS.some((tool) => tool.name === name);
}

/**
 * Get tool definition by name
 */
export function getTerminalToolDefinition(name: TerminalToolName) {
  return TERMINAL_TOOLS.find((tool) => tool.name === name);
}

/**
 * Get all terminal tool definitions
 */
export function getAllTerminalTools() {
  return TERMINAL_TOOLS;
}
