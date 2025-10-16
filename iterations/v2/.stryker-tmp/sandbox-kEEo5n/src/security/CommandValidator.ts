/**
 * Command Validator
 *
 * Validates commands and arguments against security policies before execution.
 * Implements allowlist-based command validation and argument sanitization.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import * as fs from "fs";
import type {
  CommandValidatorConfig,
  SecurityValidationResult,
} from "../mcp-server/types/terminal-types";

/**
 * Patterns that indicate dangerous shell metacharacters or constructs
 */
const SHELL_METACHARACTERS = [
  ";", // Command chaining
  "|", // Pipe
  "&", // Background execution
  ">", // Redirection
  "<", // Redirection
  "{", // Brace expansion
  "}", // Brace expansion
  "[", // Character classes
  "]", // Character classes
  "?", // Wildcard
  "*", // Wildcard (when not properly escaped)
  "~", // Home directory expansion
  "\n", // Newline injection
  "\r", // Carriage return injection
  "\x00", // Null byte
];

/**
 * Patterns for command substitution
 */
const COMMAND_SUBSTITUTION_PATTERNS = [
  /\$\(.*?\)/g, // $(command) - match any content between $( and )
  /`.*?`/g, // `command` (backtick command substitution - match any content between backticks)
];

/**
 * Patterns for variable expansion
 */
const VARIABLE_EXPANSION_PATTERNS = [
  /\$\{[^}]*\}/g, // ${VAR}
  /\$[A-Z_a-z][A-Z_a-z0-9]*/g, // $VAR (fixed: lowercase allowed at start)
];

/**
 * Sensitive environment variable patterns to filter
 */
const SENSITIVE_ENV_PATTERNS = [
  /password/i,
  /secret/i,
  /key/i,
  /token/i,
  /credential/i,
  /auth/i,
  /api[_-]?key/i,
  /aws[_-]?access/i,
  /aws[_-]?secret/i,
  /database[_-]?url/i,
  /db[_-]?password/i,
  /private[_-]?key/i,
];

/**
 * Maximum lengths for security
 */
const MAX_COMMAND_LENGTH = 1000;
const MAX_ARG_LENGTH = 5000;

/**
 * CommandValidator
 *
 * Validates commands and arguments for security before terminal execution.
 */
export class CommandValidator {
  private allowedCommands: Set<string>;
  private config: CommandValidatorConfig;

  constructor(config: CommandValidatorConfig) {
    this.config = {
      allowRelativePaths: true,
      maxCommandLength: MAX_COMMAND_LENGTH,
      maxArgLength: MAX_ARG_LENGTH,
      sensitiveEnvPatterns: SENSITIVE_ENV_PATTERNS.map((p) => p.source),
      ...config,
    };

    this.allowedCommands = this.loadAllowlist(config.allowlistPath);
  }

  /**
   * Check if a command is in the allowlist
   *
   * @param command - Command to check
   * @returns True if command is allowed
   */
  isCommandAllowed(command: string): boolean {
    if (!command || typeof command !== "string") {
      return false;
    }

    // Trim whitespace
    command = command.trim();

    if (command.length === 0) {
      return false;
    }

    // Check length limit
    if (command.length > this.config.maxCommandLength!) {
      return false;
    }

    // Extract base command from path
    const baseCommand = this.extractBaseCommand(command);

    // Check against allowlist
    return this.allowedCommands.has(baseCommand);
  }

  /**
   * Validate command arguments for security issues
   *
   * @param args - Arguments to validate
   * @returns Validation result
   */
  validateArguments(args?: string[]): SecurityValidationResult {
    // Empty or undefined is valid
    if (!args || args.length === 0) {
      return { valid: true };
    }

    const issues: string[] = [];

    for (const arg of args) {
      // Check argument length
      if (arg.length > this.config.maxArgLength!) {
        return {
          valid: false,
          error: `Argument too long (max ${this.config.maxArgLength} chars)`,
          reason: "Length limit exceeded",
        };
      }

      // Check for command substitution (more specific patterns first)
      for (const pattern of COMMAND_SUBSTITUTION_PATTERNS) {
        // Reset regex lastIndex to avoid global flag issues
        pattern.lastIndex = 0;
        if (pattern.test(arg)) {
          return {
            valid: false,
            error: "Dangerous command substitution detected",
            reason: "Potential command injection",
            issues: [`Matches pattern: ${pattern}`],
          };
        }
      }

      // Check for variable expansion (more specific patterns first)
      for (const pattern of VARIABLE_EXPANSION_PATTERNS) {
        // Reset regex lastIndex to avoid global flag issues
        pattern.lastIndex = 0;
        if (pattern.test(arg)) {
          return {
            valid: false,
            error: "Dangerous variable expansion detected",
            reason: "Potential variable injection",
            issues: [`Matches pattern: ${pattern}`],
          };
        }
      }

      // Check for dangerous arguments
      if (arg === "/" || arg === "/*" || arg === "/**") {
        return {
          valid: false,
          error: "Dangerous argument detected: root directory deletion",
          reason: "Unsafe file system operation",
          issues: ["Attempting to delete root directory"],
        };
      }

      // Check for shell metacharacters (general patterns last)
      for (const metachar of SHELL_METACHARACTERS) {
        if (arg.includes(metachar)) {
          return {
            valid: false,
            error: `Dangerous shell metacharacter detected: ${metachar}`,
            reason: "Shell injection attempt",
            issues: [`Contains metacharacter: ${metachar}`],
          };
        }
      }
    }

    return { valid: true };
  }

  /**
   * Sanitize environment variables by removing sensitive ones
   *
   * @param env - Environment variables to sanitize
   * @returns Sanitized environment variables
   */
  sanitizeEnvironment(env?: Record<string, string>): Record<string, string> {
    if (!env) {
      return {};
    }

    const sanitized: Record<string, string> = {};
    const patterns = this.config.sensitiveEnvPatterns!.map(
      (p) => new RegExp(p, "i")
    );

    for (const [key, value] of Object.entries(env)) {
      // Check if key matches any sensitive pattern
      const isSensitive = patterns.some((pattern) => pattern.test(key));

      // Preserve CAWS-specific variables
      const isCawsVar = key.startsWith("CAWS_");

      if (!isSensitive || isCawsVar) {
        sanitized[key] = value;
      }
    }

    return sanitized;
  }

  /**
   * Validate complete command with arguments
   *
   * @param command - Command to validate
   * @param args - Command arguments
   * @returns Validation result
   */
  validateCommand(command: string, args?: string[]): SecurityValidationResult {
    const issues: string[] = [];

    // Validate command
    if (!this.isCommandAllowed(command)) {
      return {
        valid: false,
        error: `Command '${command}' is not allowed`,
        reason: "Command not in allowlist",
        issues: ["Command not found in allowlist"],
      };
    }

    // Validate arguments
    const argsResult = this.validateArguments(args);
    if (!argsResult.valid) {
      return {
        valid: false,
        error: argsResult.error,
        reason: argsResult.reason,
        issues: ["Unsafe arguments detected"],
      };
    }

    return { valid: true };
  }

  /**
   * Load allowlist from file
   *
   * @param allowlistPath - Path to allowlist JSON file
   * @returns Set of allowed commands
   */
  private loadAllowlist(allowlistPath: string): Set<string> {
    try {
      if (!fs.existsSync(allowlistPath)) {
        throw new Error(`Allowlist file not found: ${allowlistPath}`);
      }

      const content = fs.readFileSync(allowlistPath, "utf-8");
      const commands = JSON.parse(content);

      if (!Array.isArray(commands)) {
        throw new Error("Allowlist must be an array of command names");
      }

      return new Set(commands);
    } catch (error) {
      if (error instanceof Error) {
        throw new Error(`Failed to load allowlist: ${error.message}`);
      }
      throw error;
    }
  }

  /**
   * Extract base command from path
   *
   * @param command - Command with potential path
   * @returns Base command name
   */
  private extractBaseCommand(command: string): string {
    // Handle full paths: /usr/bin/npm -> npm
    const parts = command.split("/");
    const basename = parts[parts.length - 1];

    // Handle relative paths: ./npm -> npm
    return basename.replace(/^\.\//, "");
  }
}
