/**
 * Terminal Tool Handlers
 *
 * MCP tool handlers for terminal access functionality.
 * Provides secure terminal session management and command execution.
 *
 * @author @darianrosebrook
 */

import { TerminalSessionManager } from "../../orchestrator/TerminalSessionManager";
import type { MCPToolResponse } from "../types/mcp-types";
import type {
  MCPCloseSessionArgs,
  MCPCreateSessionArgs,
  MCPExecuteCommandArgs,
  MCPGetStatusArgs,
} from "../types/terminal-types";

/**
 * Helper function to wrap response data in MCP format
 */
function createMCPResponse(data: any): MCPToolResponse {
  return {
    content: [
      {
        type: "text",
        text: JSON.stringify(data),
      },
    ],
  };
}

/**
 * Handle terminal_create_session tool call
 *
 * Creates a new isolated terminal session for task execution.
 */
export async function handleTerminalCreateSession(
  sessionManager: TerminalSessionManager,
  args: MCPCreateSessionArgs
): Promise<MCPToolResponse> {
  try {
    // Validate required parameters
    if (!args.taskId || !args.agentId) {
      return createMCPResponse({
        success: false,
        error: "INVALID_PARAMETERS",
        message: "taskId and agentId are required",
      });
    }

    // Create the session
    const session = await sessionManager.createSession(
      args.taskId,
      args.agentId,
      {
        workingDirectory: args.workingDirectory,
        environment: args.environment,
      }
    );

    return createMCPResponse({
      success: true,
      sessionId: session.id,
      workingDirectory: session.workingDirectory,
      createdAt: session.createdAt.toISOString(),
    });
  } catch (error) {
    console.error("[Terminal MCP] Create session error:", error);

    return createMCPResponse({
      success: false,
      error: "EXECUTION_ERROR",
      message:
        error instanceof Error ? error.message : "Unknown error occurred",
    });
  }
}

/**
 * Handle terminal_execute_command tool call
 *
 * Executes a validated command in an existing terminal session.
 */
export async function handleTerminalExecuteCommand(
  sessionManager: TerminalSessionManager,
  args: MCPExecuteCommandArgs
): Promise<MCPToolResponse> {
  try {
    // Validate required parameters
    if (!args.sessionId || !args.command) {
      return createMCPResponse({
        success: false,
        error: "INVALID_PARAMETERS",
        message: "sessionId and command are required",
      });
    }

    // Execute the command
    const result = await sessionManager.executeCommand({
      sessionId: args.sessionId,
      command: args.command,
      args: args.args,
      timeout: args.timeout,
    });

    return createMCPResponse({
      success: result.success,
      exitCode: result.exitCode,
      stdout: result.stdout,
      stderr: result.stderr,
      duration: result.duration,
      truncated: result.truncated,
      error: result.error,
    });
  } catch (error) {
    console.error("[Terminal MCP] Execute command error:", error);

    return createMCPResponse({
      success: false,
      error: "EXECUTION_ERROR",
      message:
        error instanceof Error ? error.message : "Unknown error occurred",
    });
  }
}

/**
 * Handle terminal_close_session tool call
 *
 * Closes a terminal session and cleans up all resources.
 */
export async function handleTerminalCloseSession(
  sessionManager: TerminalSessionManager,
  args: MCPCloseSessionArgs
): Promise<MCPToolResponse> {
  try {
    // Validate required parameters
    if (!args.sessionId) {
      return createMCPResponse({
        success: false,
        error: "INVALID_PARAMETERS",
        message: "sessionId is required",
      });
    }

    // Close the session
    await sessionManager.closeSession(args.sessionId);

    return createMCPResponse({
      success: true,
      message: "Session closed and resources freed",
      sessionId: args.sessionId,
    });
  } catch (error) {
    console.error("[Terminal MCP] Close session error:", error);

    return createMCPResponse({
      success: false,
      error: "EXECUTION_ERROR",
      message:
        error instanceof Error ? error.message : "Unknown error occurred",
    });
  }
}

/**
 * Handle terminal_get_status tool call
 *
 * Retrieves current status and metadata for a terminal session.
 */
export async function handleTerminalGetStatus(
  sessionManager: TerminalSessionManager,
  args: MCPGetStatusArgs
): Promise<MCPToolResponse> {
  try {
    // Validate required parameters
    if (!args.sessionId) {
      return createMCPResponse({
        success: false,
        error: "INVALID_PARAMETERS",
        message: "sessionId is required",
      });
    }

    // Get the session
    const session = sessionManager.getSession(args.sessionId);

    if (!session) {
      return createMCPResponse({
        success: false,
        error: "SESSION_NOT_FOUND",
        message: `Session ${args.sessionId} not found`,
      });
    }

    return createMCPResponse({
      success: true,
      session: {
        id: session.id,
        taskId: session.taskId,
        agentId: session.agentId,
        workingDirectory: session.workingDirectory,
        state: session.state,
        createdAt: session.createdAt.toISOString(),
        lastCommandAt: session.lastCommandAt?.toISOString(),
        commandCount: session.commandCount,
      },
    });
  } catch (error) {
    console.error("[Terminal MCP] Get status error:", error);

    return createMCPResponse({
      success: false,
      error: "EXECUTION_ERROR",
      message:
        error instanceof Error ? error.message : "Unknown error occurred",
    });
  }
}

/**
 * Get terminal session statistics
 *
 * Provides operational metrics for monitoring.
 */
export async function handleTerminalGetStats(
  sessionManager: TerminalSessionManager
): Promise<{
  success: boolean;
  stats?: {
    activeSessions: number;
    totalSessionsCreated: number;
    uptime: number;
  };
  error?: string;
  message?: string;
}> {
  try {
    const stats = sessionManager.getStats();

    return {
      success: true,
      stats,
    };
  } catch (error) {
    console.error("[Terminal MCP] Get stats error:", error);

    return {
      success: false,
      error: "EXECUTION_ERROR",
      message:
        error instanceof Error ? error.message : "Unknown error occurred",
    };
  }
}

/**
 * List all active terminal sessions
 *
 * Provides overview of all running sessions (admin/debugging).
 */
export async function handleTerminalListSessions(
  sessionManager: TerminalSessionManager
): Promise<{
  success: boolean;
  sessions?: Array<{
    id: string;
    taskId: string;
    agentId: string;
    state: string;
    createdAt: string;
  }>;
  error?: string;
  message?: string;
}> {
  try {
    const sessions = sessionManager.listSessions();

    return {
      success: true,
      sessions: sessions.map((session) => ({
        id: session.id,
        taskId: session.taskId,
        agentId: session.agentId,
        state: session.state,
        createdAt: session.createdAt.toISOString(),
      })),
    };
  } catch (error) {
    console.error("[Terminal MCP] List sessions error:", error);

    return {
      success: false,
      error: "EXECUTION_ERROR",
      message:
        error instanceof Error ? error.message : "Unknown error occurred",
    };
  }
}
