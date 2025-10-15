/**
 * Terminal Session Manager
 *
 * Manages isolated terminal sessions for secure command execution.
 * Provides session lifecycle management, command validation, and resource cleanup.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { spawn } from "child_process";
import { EventEmitter } from "events";
import type {
  CommandExecutionRequest,
  CommandExecutionResult,
  SessionCreateOptions,
  TerminalEvent,
  TerminalSession,
  TerminalSessionManagerConfig,
} from "../mcp-server/types/terminal-types";
import {
  SessionState,
  TerminalErrorCode,
  TerminalEventType,
} from "../mcp-server/types/terminal-types";
import { CommandValidator } from "../security/CommandValidator";

/**
 * Maximum output size per command (1MB)
 */
const MAX_OUTPUT_SIZE = 1024 * 1024;

/**
 * Default command timeout (60 seconds)
 */
const DEFAULT_TIMEOUT = 60000;

/**
 * Maximum command timeout (5 minutes)
 */
const MAX_TIMEOUT = 300000;

/**
 * Maximum concurrent sessions
 */
const MAX_CONCURRENT_SESSIONS = 50;

/**
 * TerminalSessionManager
 *
 * Manages terminal sessions for secure command execution with comprehensive
 * security controls and resource management.
 */
export class TerminalSessionManager extends EventEmitter {
  private sessions: Map<string, TerminalSession> = new Map();
  private config: TerminalSessionManagerConfig;
  private validator: CommandValidator;

  constructor(config: Partial<TerminalSessionManagerConfig> = {}) {
    super();

    this.config = {
      projectRoot: process.cwd(),
      allowedCommandsPath: "./apps/tools/caws/tools-allow.json",
      defaultTimeout: DEFAULT_TIMEOUT,
      maxTimeout: MAX_TIMEOUT,
      maxOutputSize: MAX_OUTPUT_SIZE,
      maxConcurrentSessions: MAX_CONCURRENT_SESSIONS,
      enableAuditLog: true,
      ...config,
    };

    this.validator = new CommandValidator({
      allowlistPath:
        this.config.allowedCommandsPath || "./apps/tools/caws/tools-allow.json",
    });
  }

  /**
   * Create a new terminal session
   *
   * @param taskId - Task ID this session belongs to
   * @param agentId - Agent ID creating the session
   * @param options - Session creation options
   * @returns Created terminal session
   */
  async createSession(
    taskId: string,
    agentId: string,
    options: SessionCreateOptions = {}
  ): Promise<TerminalSession> {
    // Check concurrent session limit
    if (this.sessions.size >= this.config.maxConcurrentSessions!) {
      throw new Error(
        `Maximum concurrent sessions (${this.config.maxConcurrentSessions}) exceeded`
      );
    }

    // Generate unique session ID
    const sessionId = `term-${taskId}-${Date.now()}-${Math.random()
      .toString(36)
      .substr(2, 9)}`;

    // Merge environment variables
    const environment = {
      ...process.env,
      CAWS_TASK_ID: taskId,
      CAWS_AGENT_ID: agentId,
      CAWS_SESSION_ID: sessionId,
      ...this.validator.sanitizeEnvironment(options.environment),
    };

    const session: TerminalSession = {
      id: sessionId,
      taskId,
      agentId,
      workingDirectory: options.workingDirectory || this.config.projectRoot!,
      environment,
      state: SessionState.IDLE,
      createdAt: new Date(),
    };

    this.sessions.set(sessionId, session);

    // Emit session created event
    this.emit(TerminalEventType.SESSION_CREATED, {
      type: TerminalEventType.SESSION_CREATED,
      timestamp: new Date(),
      sessionId,
      taskId,
      agentId,
      data: { workingDirectory: session.workingDirectory },
    } as TerminalEvent);

    return session;
  }

  /**
   * Execute a command in an existing session
   *
   * @param request - Command execution request
   * @returns Command execution result
   */
  async executeCommand(
    request: CommandExecutionRequest
  ): Promise<CommandExecutionResult> {
    const session = this.sessions.get(request.sessionId);
    if (!session) {
      return {
        success: false,
        exitCode: 1,
        stdout: "",
        stderr: "",
        duration: 0,
        error: TerminalErrorCode.SESSION_NOT_FOUND,
      };
    }

    // Validate command and arguments
    const validation = this.validator.validateCommand(
      request.command,
      request.args
    );
    if (!validation.valid) {
      this.emit(TerminalEventType.SECURITY_VIOLATION, {
        type: TerminalEventType.SECURITY_VIOLATION,
        timestamp: new Date(),
        sessionId: request.sessionId,
        taskId: session.taskId,
        agentId: session.agentId,
        data: {
          command: request.command,
          args: request.args,
          issues: validation.issues,
        },
      } as TerminalEvent);

      return {
        success: false,
        exitCode: 1,
        stdout: "",
        stderr: validation.error || "Command validation failed",
        duration: 0,
        error: TerminalErrorCode.UNSAFE_ARGUMENTS,
      };
    }

    // Update session state
    session.state = SessionState.RUNNING;
    session.lastCommandAt = new Date();

    const startTime = Date.now();
    const timeout = Math.min(
      request.timeout || this.config.defaultTimeout!,
      this.config.maxTimeout!
    );

    try {
      const result = await this.runCommandWithTimeout(
        request.command,
        request.args || [],
        {
          cwd: session.workingDirectory,
          env: session.environment,
          timeout,
        }
      );

      // Update session state and command count
      session.state = result.success
        ? SessionState.COMPLETED
        : SessionState.FAILED;
      session.commandCount = (session.commandCount || 0) + 1;

      // Emit command executed event
      this.emit(TerminalEventType.COMMAND_EXECUTED, {
        type: result.success
          ? TerminalEventType.COMMAND_EXECUTED
          : TerminalEventType.COMMAND_FAILED,
        timestamp: new Date(),
        sessionId: request.sessionId,
        taskId: session.taskId,
        agentId: session.agentId,
        data: {
          command: request.command,
          args: request.args,
          exitCode: result.exitCode,
          duration: result.duration,
          truncated: result.truncated,
        },
      } as TerminalEvent);

      return result;
    } catch (error) {
      session.state = SessionState.FAILED;

      this.emit(TerminalEventType.COMMAND_FAILED, {
        type: TerminalEventType.COMMAND_FAILED,
        timestamp: new Date(),
        sessionId: request.sessionId,
        taskId: session.taskId,
        agentId: session.agentId,
        data: {
          command: request.command,
          args: request.args,
          error: error instanceof Error ? error.message : String(error),
        },
      } as TerminalEvent);

      return {
        success: false,
        exitCode: 1,
        stdout: "",
        stderr: error instanceof Error ? error.message : String(error),
        duration: Date.now() - startTime,
        error: TerminalErrorCode.EXECUTION_ERROR,
      };
    }
  }

  /**
   * Close a terminal session and cleanup resources
   *
   * @param sessionId - Session ID to close
   */
  async closeSession(sessionId: string): Promise<void> {
    const session = this.sessions.get(sessionId);
    if (!session) {
      return; // Idempotent - closing non-existent session is safe
    }

    // Kill any running process
    if (session.process && !session.process.killed) {
      session.process.kill("SIGTERM");

      // Give process time to cleanup, then force kill if needed
      setTimeout(() => {
        if (session.process && !session.process.killed) {
          session.process.kill("SIGKILL");
        }
      }, 5000);
    }

    // Remove from registry
    this.sessions.delete(sessionId);

    // Emit session closed event
    this.emit(TerminalEventType.SESSION_CLOSED, {
      type: TerminalEventType.SESSION_CLOSED,
      timestamp: new Date(),
      sessionId,
      taskId: session.taskId,
      agentId: session.agentId,
      data: { commandCount: session.commandCount },
    } as TerminalEvent);
  }

  /**
   * Get session by ID
   *
   * @param sessionId - Session ID
   * @returns Terminal session or undefined
   */
  getSession(sessionId: string): TerminalSession | undefined {
    return this.sessions.get(sessionId);
  }

  /**
   * List all active sessions
   *
   * @returns Array of active terminal sessions
   */
  listSessions(): TerminalSession[] {
    return Array.from(this.sessions.values());
  }

  /**
   * Get session statistics
   *
   * @returns Session statistics
   */
  getStats(): {
    activeSessions: number;
    totalSessionsCreated: number;
    uptime: number;
  } {
    return {
      activeSessions: this.sessions.size,
      totalSessionsCreated: this.sessions.size, // TODO: Track historical total
      uptime: Date.now() - (this as any).startTime || 0,
    };
  }

  /**
   * Execute command with timeout and output size limits
   *
   * @param command - Command to execute
   * @param args - Command arguments
   * @param options - Execution options
   * @returns Command execution result
   */
  private async runCommandWithTimeout(
    command: string,
    args: string[],
    options: {
      cwd: string;
      env: Record<string, string>;
      timeout: number;
    }
  ): Promise<CommandExecutionResult> {
    return new Promise((resolve, reject) => {
      let stdout = "";
      let stderr = "";
      let truncated = false;

      const child = spawn(command, args, {
        cwd: options.cwd,
        env: options.env,
        shell: false, // Disable shell to prevent injection
      });

      // Store process reference for cleanup
      const session = this.sessions.get((this as any).currentSessionId);
      if (session) {
        session.process = child;
      }

      const timeoutHandle = setTimeout(() => {
        if (!child.killed) {
          child.kill("SIGTERM");
          reject(new Error(`Command timeout after ${options.timeout}ms`));
        }
      }, options.timeout);

      // Handle stdout with size limits
      child.stdout?.on("data", (data: Buffer) => {
        const chunk = data.toString();
        if (stdout.length + chunk.length > this.config.maxOutputSize!) {
          truncated = true;
          stdout = stdout.substring(0, this.config.maxOutputSize!);
          child.kill("SIGTERM"); // Stop the process
        } else {
          stdout += chunk;
        }
      });

      // Handle stderr with size limits
      child.stderr?.on("data", (data: Buffer) => {
        const chunk = data.toString();
        if (stderr.length + chunk.length > this.config.maxOutputSize!) {
          stderr = stderr.substring(0, this.config.maxOutputSize!);
          child.kill("SIGTERM"); // Stop the process
        } else {
          stderr += chunk;
        }
      });

      child.on("close", (exitCode) => {
        clearTimeout(timeoutHandle);

        // Clean up process reference
        if (session) {
          session.process = undefined;
        }

        resolve({
          success: exitCode === 0,
          exitCode: exitCode || 0,
          stdout,
          stderr,
          duration: 0, // Will be set by caller
          truncated,
        });
      });

      child.on("error", (error) => {
        clearTimeout(timeoutHandle);

        // Clean up process reference
        if (session) {
          session.process = undefined;
        }

        reject(error);
      });
    });
  }

  /**
   * Set current session ID for process tracking (internal use)
   */
  private setCurrentSessionId(sessionId: string): void {
    (this as any).currentSessionId = sessionId;
  }

  /**
   * Execute command in session context (internal method)
   */
  async executeCommandInSession(
    sessionId: string,
    request: Omit<CommandExecutionRequest, "sessionId">
  ): Promise<CommandExecutionResult> {
    this.setCurrentSessionId(sessionId);
    return this.executeCommand({ ...request, sessionId });
  }
}
