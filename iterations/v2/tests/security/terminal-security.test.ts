/**
 * Terminal Security Tests
 *
 * Comprehensive security testing for the MCP Terminal Access Layer
 * covering command injection, shell escapes, and other attack vectors.
 *
 * @author @darianrosebrook
 */

import * as fs from "fs";
import * as path from "path";
import type { TerminalSession } from "../../src/mcp-server/types/terminal-types";
import { TerminalSessionManager } from "../../src/orchestrator/TerminalSessionManager";
import { CommandValidator } from "../../src/security/CommandValidator";

const testProjectRoot = path.join(__dirname, "../test-project-root");
let validator: CommandValidator;
let manager: TerminalSessionManager;

beforeEach(() => {
  // Ensure test directory exists
  if (!fs.existsSync(testProjectRoot)) {
    fs.mkdirSync(testProjectRoot, { recursive: true });
  }

  validator = new CommandValidator({
    allowlistPath: path.join(__dirname, "../../fixtures/test-allowlist.json"),
  });

  manager = new TerminalSessionManager({
    projectRoot: testProjectRoot,
    allowedCommandsPath: path.join(
      __dirname,
      "../../fixtures/test-allowlist.json"
    ),
  });
});

afterEach(async () => {
  // Cleanup test directory
  if (fs.existsSync(testProjectRoot)) {
    fs.rmSync(testProjectRoot, { recursive: true, force: true });
  }
});

describe("Command Injection Prevention", () => {
  describe("Shell Metacharacter Detection", () => {
    const _dangerousChars = [
      ";", // Command chaining
      "|", // Pipe
      "&", // Background execution
      ">", // Output redirection
      "<", // Input redirection
      "`", // Command substitution (backticks)
      "$", // Variable expansion (when not properly escaped)
      "(", // Command grouping
      ")", // Command grouping
      "{", // Brace expansion
      "}", // Brace expansion
      "[", // Character classes
      "]", // Character classes
      "?", // Wildcard
      "*", // Wildcard
      "~", // Home directory expansion
      "\n", // Newline injection
      "\r", // Carriage return injection
      "\x00", // Null byte
    ];

    // Characters that should be caught by command substitution patterns
    test("should block command substitution with backticks", () => {
      const args = ["`whoami`"];

      const result = validator.validateArguments(args);

      expect(result.valid).toBe(false);
      expect(result.error).toContain("Dangerous command substitution");
    });

    test("should block command substitution with parentheses", () => {
      const args = ["$(whoami)"];

      const result = validator.validateArguments(args);

      expect(result.valid).toBe(false);
      expect(result.error).toContain("Dangerous command substitution");
    });

    // Characters that should be caught by variable expansion patterns
    test("should block variable expansion with braces", () => {
      const args = ["${HOME}"];

      const result = validator.validateArguments(args);

      expect(result.valid).toBe(false);
      expect(result.error).toContain("Dangerous variable expansion");
    });

    // Characters that should be caught by shell metacharacter check
    const shellMetaChars = [
      ";",
      "|",
      "&",
      ">",
      "<",
      "{",
      "}",
      "[",
      "]",
      "?",
      "*",
      "~",
      "\n",
      "\r",
      "\x00",
    ];
    test.each(shellMetaChars)(
      "should block shell metacharacter: %s",
      (char) => {
        const args = [`test${char}command`];

        const result = validator.validateArguments(args);

        expect(result.valid).toBe(false);
        expect(result.error).toContain("Dangerous shell metacharacter");
        expect(result.issues).toContain(`Contains metacharacter: ${char}`);
      }
    );
  });

  describe("Command Substitution Attacks", () => {
    const substitutionAttacks = [
      "$(whoami)",
      "$(cat /etc/passwd)",
      "$(rm -rf /)",
      "`whoami`",
      "`cat /etc/passwd`",
      "`rm -rf /`",
    ];

    test.each(substitutionAttacks)(
      "should block command substitution: %s",
      (attack) => {
        const args = [attack];

        const result = validator.validateArguments(args);

        expect(result.valid).toBe(false);
        expect(result.error).toContain("Dangerous command substitution");
      }
    );
  });

  describe("Terminal Session Security", () => {
    let session: TerminalSession;

    beforeEach(async () => {
      session = await manager.createSession("SECURITY-TEST", "security-agent");
    });

    afterEach(async () => {
      await manager.closeSession(session.id);
    });

    test("should reject command injection via npm scripts", async () => {
      const result = await manager.executeCommand({
        sessionId: session.id,
        command: "npm",
        args: ["run", "test; rm -rf /"],
      });

      expect(result.success).toBe(false);
      expect(result.error).toBe("UNSAFE_ARGUMENTS");
      expect(result.stderr).toContain("Dangerous shell metacharacter detected");
    });

    test("should reject command injection via git commands", async () => {
      const result = await manager.executeCommand({
        sessionId: session.id,
        command: "git",
        args: ["log", "--oneline", "|", "cat"],
      });

      expect(result.success).toBe(false);
      expect(result.error).toBe("UNSAFE_ARGUMENTS");
      expect(result.stderr).toContain("Dangerous shell metacharacter detected");
    });

    test("should reject environment variable injection", async () => {
      const result = await manager.executeCommand({
        sessionId: session.id,
        command: "echo",
        args: ["$HOME"],
      });

      expect(result.success).toBe(false);
      expect(result.error).toBe("UNSAFE_ARGUMENTS");
      expect(result.stderr).toContain("Dangerous variable expansion detected");
    });
  });

  describe("Command Allowlist Security", () => {
    test("should only allow commands in the allowlist", () => {
      const allowedCommands = [
        "node",
        "npm",
        "git",
        "echo",
        "cat",
        "ls",
        "sleep",
      ];

      allowedCommands.forEach((command) => {
        const result = validator.isCommandAllowed(command);
        expect(result).toBe(true);
      });
    });

    test("should reject commands not in the allowlist", () => {
      const dangerousCommands = [
        "rm",
        "rmdir",
        "del",
        "format",
        "fdisk",
        "mkfs",
        "dd",
        "wget",
        "curl",
        "ssh",
        "scp",
        "sudo",
        "su",
        "chmod",
        "chown",
      ];

      dangerousCommands.forEach((command) => {
        const result = validator.isCommandAllowed(command);
        expect(result).toBe(false);
      });
    });
  });
});
