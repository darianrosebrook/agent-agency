/**
 * Terminal Access Type Definitions
 *
 * TypeScript type definitions for MCP terminal access layer.
 * Provides type safety for terminal session management and command execution.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


/**
 * Terminal session state
 */
export enum SessionState {
  /** Session created but no commands executed yet */
  IDLE = "idle",
  /** Command currently executing */
  RUNNING = "running",
  /** Last command completed successfully */
  COMPLETED = "completed",
  /** Last command or session failed */
  FAILED = "failed",
}

/**
 * Terminal session representing an isolated command execution environment
 */
export interface TerminalSession {
  /** Unique session identifier */
  id: string;

  /** Task ID this session is associated with */
  taskId: string;

  /** Agent ID that created this session */
  agentId: string;

  /** Working directory for command execution */
  workingDirectory: string;

  /** Environment variables for this session */
  environment: Record<string, string>;

  /** Current session state */
  state: SessionState;

  /** Session creation timestamp */
  createdAt: Date;

  /** Last command execution timestamp (optional) */
  lastCommandAt?: Date;

  /** Child process if command is running (optional) */
  process?: any; // ChildProcess type from 'child_process'

  /** Number of commands executed in this session */
  commandCount?: number;
}

/**
 * Request to execute a command in a terminal session
 */
export interface CommandExecutionRequest {
  /** Session ID to execute command in */
  sessionId: string;

  /** Command to execute (must be in allowlist) */
  command: string;

  /** Command arguments (optional) */
  args?: string[];

  /** Execution timeout in milliseconds (optional, default: 60000) */
  timeout?: number;

  /** Allowed commands for validation (optional, uses default allowlist if not provided) */
  allowedCommands?: string[];
}

/**
 * Result of command execution
 */
export interface CommandExecutionResult {
  /** Whether command executed successfully (exit code 0) */
  success: boolean;

  /** Process exit code */
  exitCode: number;

  /** Standard output from command */
  stdout: string;

  /** Standard error output from command */
  stderr: string;

  /** Execution duration in milliseconds */
  duration: number;

  /** Error code if execution failed (optional) */
  error?: TerminalErrorCode;

  /** Whether output was truncated due to size limit */
  truncated?: boolean;
}

/**
 * Terminal error codes
 */
export enum TerminalErrorCode {
  /** Command not in allowlist */
  COMMAND_NOT_ALLOWED = "COMMAND_NOT_ALLOWED",

  /** Command exceeded timeout */
  TIMEOUT_EXCEEDED = "TIMEOUT_EXCEEDED",

  /** Session not found */
  SESSION_NOT_FOUND = "SESSION_NOT_FOUND",

  /** Invalid request parameters */
  INVALID_PARAMETERS = "INVALID_PARAMETERS",

  /** Command execution error */
  EXECUTION_ERROR = "EXECUTION_ERROR",

  /** Maximum concurrent sessions exceeded */
  MAX_SESSIONS_EXCEEDED = "MAX_SESSIONS_EXCEEDED",

  /** Unsafe arguments detected */
  UNSAFE_ARGUMENTS = "UNSAFE_ARGUMENTS",
}

/**
 * Session creation options
 */
export interface SessionCreateOptions {
  /** Working directory (optional, defaults to project root) */
  workingDirectory?: string;

  /** Additional environment variables (optional) */
  environment?: Record<string, string>;

  /** Session-specific timeout override (optional) */
  defaultTimeout?: number;
}

/**
 * Terminal session manager configuration
 */
export interface TerminalSessionManagerConfig {
  /** Project root directory */
  projectRoot: string;

  /** Path to tools-allow.json (optional) */
  allowedCommandsPath?: string;

  /** Default command timeout in milliseconds */
  defaultTimeout?: number;

  /** Maximum command timeout in milliseconds */
  maxTimeout?: number;

  /** Maximum output size in bytes */
  maxOutputSize?: number;

  /** Maximum concurrent sessions */
  maxConcurrentSessions?: number;

  /** Whether to enable audit logging */
  enableAuditLog?: boolean;
}

/**
 * Command validator configuration
 */
export interface CommandValidatorConfig {
  /** Path to tools-allow.json */
  allowlistPath: string;

  /** Whether to allow relative paths in commands */
  allowRelativePaths?: boolean;

  /** Maximum command length */
  maxCommandLength?: number;

  /** Maximum argument length */
  maxArgLength?: number;

  /** List of sensitive environment variable patterns to filter */
  sensitiveEnvPatterns?: string[];
}

/**
 * Terminal session event types
 */
export enum TerminalEventType {
  SESSION_CREATED = "session:created",
  SESSION_CLOSED = "session:closed",
  COMMAND_EXECUTED = "command:executed",
  COMMAND_FAILED = "command:failed",
  SECURITY_VIOLATION = "security:violation",
}

/**
 * Terminal event data
 */
export interface TerminalEvent {
  /** Event type */
  type: TerminalEventType;

  /** Timestamp */
  timestamp: Date;

  /** Session ID */
  sessionId: string;

  /** Task ID */
  taskId: string;

  /** Agent ID */
  agentId: string;

  /** Additional event data */
  data: Record<string, any>;
}

/**
 * Security validation result
 */
export interface SecurityValidationResult {
  /** Whether validation passed */
  valid: boolean;

  /** Validation error message (if invalid) */
  error?: string;

  /** Detailed reason for failure (optional) */
  reason?: string;

  /** Detected security issues (optional) */
  issues?: string[];
}

/**
 * Terminal session statistics
 */
export interface TerminalSessionStats {
  /** Total sessions created */
  totalSessions: number;

  /** Currently active sessions */
  activeSessions: number;

  /** Total commands executed */
  totalCommands: number;

  /** Commands failed */
  failedCommands: number;

  /** Security violations detected */
  securityViolations: number;

  /** Average command duration (ms) */
  avgCommandDuration: number;

  /** Uptime since manager started (ms) */
  uptime: number;
}

/**
 * MCP tool call arguments for terminal_create_session
 */
export interface MCPCreateSessionArgs {
  taskId: string;
  agentId: string;
  workingDirectory?: string;
  environment?: Record<string, string>;
}

/**
 * MCP tool call arguments for terminal_execute_command
 */
export interface MCPExecuteCommandArgs {
  sessionId: string;
  command: string;
  args?: string[];
  timeout?: number;
}

/**
 * MCP tool call arguments for terminal_close_session
 */
export interface MCPCloseSessionArgs {
  sessionId: string;
}

/**
 * MCP tool call arguments for terminal_get_status
 */
export interface MCPGetStatusArgs {
  sessionId: string;
}

/**
 * MCP tool response for terminal_create_session
 */
export interface MCPCreateSessionResponse {
  success: boolean;
  sessionId?: string;
  workingDirectory?: string;
  createdAt?: string;
  error?: string;
  message?: string;
}

/**
 * MCP tool response for terminal_execute_command
 */
export interface MCPExecuteCommandResponse {
  success: boolean;
  exitCode?: number;
  stdout?: string;
  stderr?: string;
  duration?: number;
  truncated?: boolean;
  error?: string;
  message?: string;
}

/**
 * MCP tool response for terminal_close_session
 */
export interface MCPCloseSessionResponse {
  success: boolean;
  message?: string;
  sessionId?: string;
  error?: string;
}

/**
 * MCP tool response for terminal_get_status
 */
export interface MCPGetStatusResponse {
  success: boolean;
  session?: {
    id: string;
    taskId: string;
    agentId: string;
    workingDirectory: string;
    state: SessionState;
    createdAt: string;
    lastCommandAt?: string;
    commandCount?: number;
  };
  error?: string;
  message?: string;
}
