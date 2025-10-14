/**
 * CommandValidator Unit Tests
 *
 * Tests for command allowlist validation and security checks.
 * Ensures only safe, allowlisted commands can be executed.
 *
 * @author @darianrosebrook
 */

import { CommandValidator } from "@/security/CommandValidator";
import * as fs from "fs";
import * as path from "path";

describe("CommandValidator", () => {
  let validator: CommandValidator;
  const testAllowlistPath = path.join(
    __dirname,
    "../../fixtures/test-allowlist.json"
  );

  beforeEach(() => {
    // Create test allowlist
    const allowlist = ["npm", "git", "node", "echo", "cat", "ls"];
    fs.writeFileSync(testAllowlistPath, JSON.stringify(allowlist, null, 2));

    validator = new CommandValidator({
      allowlistPath: testAllowlistPath,
    });
  });

  afterEach(() => {
    // Cleanup
    if (fs.existsSync(testAllowlistPath)) {
      fs.unlinkSync(testAllowlistPath);
    }
  });

  describe("isCommandAllowed", () => {
    it("should allow commands in allowlist", () => {
      expect(validator.isCommandAllowed("npm")).toBe(true);
      expect(validator.isCommandAllowed("git")).toBe(true);
      expect(validator.isCommandAllowed("node")).toBe(true);
      expect(validator.isCommandAllowed("echo")).toBe(true);
    });

    it("should reject commands not in allowlist", () => {
      expect(validator.isCommandAllowed("rm")).toBe(false);
      expect(validator.isCommandAllowed("sudo")).toBe(false);
      expect(validator.isCommandAllowed("eval")).toBe(false);
      expect(validator.isCommandAllowed("chmod")).toBe(false);
    });

    it("should extract base command from full path", () => {
      expect(validator.isCommandAllowed("/usr/bin/npm")).toBe(true);
      expect(validator.isCommandAllowed("/usr/local/bin/git")).toBe(true);
      expect(validator.isCommandAllowed("./node_modules/.bin/npm")).toBe(true);
    });

    it("should extract base command from relative path", () => {
      expect(validator.isCommandAllowed("./npm")).toBe(true);
      expect(validator.isCommandAllowed("../bin/git")).toBe(true);
    });

    it("should handle empty string", () => {
      expect(validator.isCommandAllowed("")).toBe(false);
    });

    it("should handle whitespace", () => {
      expect(validator.isCommandAllowed("  ")).toBe(false);
      expect(validator.isCommandAllowed("\n")).toBe(false);
      expect(validator.isCommandAllowed("\t")).toBe(false);
    });

    it("should be case-sensitive", () => {
      expect(validator.isCommandAllowed("NPM")).toBe(false);
      expect(validator.isCommandAllowed("Git")).toBe(false);
    });
  });

  describe("validateArguments", () => {
    it("should allow safe arguments", () => {
      const result = validator.validateArguments(["test", "--coverage"]);
      expect(result.valid).toBe(true);
    });

    it("should allow arguments with dashes", () => {
      const result = validator.validateArguments([
        "--config",
        "jest.config.js",
        "-u",
      ]);
      expect(result.valid).toBe(true);
    });

    it("should allow file paths", () => {
      const result = validator.validateArguments([
        "src/index.ts",
        "./tests/unit.test.ts",
      ]);
      expect(result.valid).toBe(true);
    });

    it("should reject semicolon (command chaining)", () => {
      const result = validator.validateArguments(["test;rm -rf /"]);
      expect(result.valid).toBe(false);
      expect(result.error).toContain("shell metacharacter");
    });

    it("should reject pipe (command chaining)", () => {
      const result = validator.validateArguments(["test|grep secret"]);
      expect(result.valid).toBe(false);
      expect(result.error).toContain("shell metacharacter");
    });

    it("should reject ampersand (background execution)", () => {
      const result = validator.validateArguments(["test&"]);
      expect(result.valid).toBe(false);
    });

    it("should reject backticks (command substitution)", () => {
      const result = validator.validateArguments(["test`whoami`"]);
      expect(result.valid).toBe(false);
      expect(result.error).toContain("command substitution");
    });

    it("should reject $() command substitution", () => {
      const result = validator.validateArguments(["test$(echo bad)"]);
      expect(result.valid).toBe(false);
      expect(result.error).toContain("command substitution");
    });

    it("should reject environment variable expansion", () => {
      const result = validator.validateArguments(["test$PATH"]);
      expect(result.valid).toBe(false);
      expect(result.error).toContain("variable expansion");
    });

    it("should reject ${} variable expansion", () => {
      const result = validator.validateArguments(["test${HOME}"]);
      expect(result.valid).toBe(false);
    });

    it("should reject redirection operators", () => {
      expect(validator.validateArguments(["test>output.txt"]).valid).toBe(
        false
      );
      expect(validator.validateArguments(["test>>output.txt"]).valid).toBe(
        false
      );
      expect(validator.validateArguments(["test<input.txt"]).valid).toBe(false);
    });

    it("should reject wildcard expansion attempts", () => {
      // These are potentially dangerous and should be blocked for security
      const result = validator.validateArguments(["*.js", "test.*"]);
      expect(result.valid).toBe(false);
      expect(result.error).toContain("Dangerous shell metacharacter");
    });

    it("should handle empty array", () => {
      const result = validator.validateArguments([]);
      expect(result.valid).toBe(true);
    });

    it("should handle undefined", () => {
      const result = validator.validateArguments(undefined);
      expect(result.valid).toBe(true);
    });

    it("should reject null bytes", () => {
      const result = validator.validateArguments(["test\x00malicious"]);
      expect(result.valid).toBe(false);
    });

    it("should enforce maximum argument length", () => {
      const longArg = "a".repeat(10000);
      const result = validator.validateArguments([longArg]);
      expect(result.valid).toBe(false);
      expect(result.error).toContain("too long");
    });
  });

  describe("sanitizeEnvironment", () => {
    it("should preserve safe environment variables", () => {
      const env = {
        PATH: "/usr/bin",
        NODE_ENV: "test",
        HOME: "/home/user",
        USER: "testuser",
      };

      const sanitized = validator.sanitizeEnvironment(env);

      expect(sanitized.PATH).toBe("/usr/bin");
      expect(sanitized.NODE_ENV).toBe("test");
      expect(sanitized.HOME).toBe("/home/user");
      expect(sanitized.USER).toBe("testuser");
    });

    it("should remove AWS credentials", () => {
      const env = {
        AWS_ACCESS_KEY_ID: "AKIAIOSFODNN7EXAMPLE",
        AWS_SECRET_ACCESS_KEY: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
        NODE_ENV: "test",
      };

      const sanitized = validator.sanitizeEnvironment(env);

      expect(sanitized.AWS_ACCESS_KEY_ID).toBeUndefined();
      expect(sanitized.AWS_SECRET_ACCESS_KEY).toBeUndefined();
      expect(sanitized.NODE_ENV).toBe("test");
    });

    it("should remove database credentials", () => {
      const env = {
        DATABASE_URL: "postgresql://user:pass@localhost/db",
        DATABASE_PASSWORD: "secret123",
        DB_PASSWORD: "secret456",
        NODE_ENV: "test",
      };

      const sanitized = validator.sanitizeEnvironment(env);

      expect(sanitized.DATABASE_URL).toBeUndefined();
      expect(sanitized.DATABASE_PASSWORD).toBeUndefined();
      expect(sanitized.DB_PASSWORD).toBeUndefined();
      expect(sanitized.NODE_ENV).toBe("test");
    });

    it("should remove API keys and tokens", () => {
      const env = {
        API_KEY: "key123",
        AUTH_TOKEN: "token456",
        SECRET_KEY: "secret789",
        GITHUB_TOKEN: "ghp_abcd1234",
        NODE_ENV: "test",
      };

      const sanitized = validator.sanitizeEnvironment(env);

      expect(sanitized.API_KEY).toBeUndefined();
      expect(sanitized.AUTH_TOKEN).toBeUndefined();
      expect(sanitized.SECRET_KEY).toBeUndefined();
      expect(sanitized.GITHUB_TOKEN).toBeUndefined();
      expect(sanitized.NODE_ENV).toBe("test");
    });

    it("should use case-insensitive matching for sensitive patterns", () => {
      const env = {
        aws_secret: "secret",
        Api_Key: "key",
        DATABASE_password: "pass",
        NODE_ENV: "test",
      };

      const sanitized = validator.sanitizeEnvironment(env);

      expect(sanitized.aws_secret).toBeUndefined();
      expect(sanitized.Api_Key).toBeUndefined();
      expect(sanitized.DATABASE_password).toBeUndefined();
      expect(sanitized.NODE_ENV).toBe("test");
    });

    it("should handle empty environment", () => {
      const sanitized = validator.sanitizeEnvironment({});
      expect(sanitized).toEqual({});
    });

    it("should handle undefined", () => {
      const sanitized = validator.sanitizeEnvironment(undefined);
      expect(sanitized).toEqual({});
    });

    it("should preserve CAWS-specific variables", () => {
      const env = {
        CAWS_TASK_ID: "TASK-001",
        CAWS_AGENT_ID: "agent-1",
        CAWS_SESSION_ID: "session-123",
        NODE_ENV: "test",
      };

      const sanitized = validator.sanitizeEnvironment(env);

      expect(sanitized.CAWS_TASK_ID).toBe("TASK-001");
      expect(sanitized.CAWS_AGENT_ID).toBe("agent-1");
      expect(sanitized.CAWS_SESSION_ID).toBe("session-123");
    });
  });

  describe("validateCommand", () => {
    it("should validate complete command request", () => {
      const result = validator.validateCommand("npm", ["test", "--coverage"]);

      expect(result.valid).toBe(true);
    });

    it("should reject disallowed command", () => {
      const result = validator.validateCommand("rm", ["-rf", "/"]);

      expect(result.valid).toBe(false);
      expect(result.error).toContain("not allowed");
    });

    it("should reject allowed command with dangerous args", () => {
      const result = validator.validateCommand("npm", ["test;rm -rf /"]);

      expect(result.valid).toBe(false);
      expect(result.issues).toContain("Unsafe arguments detected");
    });

    it("should provide detailed validation results", () => {
      const result = validator.validateCommand("rm", ["test`whoami`"]);

      expect(result.valid).toBe(false);
      expect(result.issues).toBeDefined();
      expect(result.issues!.length).toBeGreaterThan(0);
    });
  });

  describe("allowlist loading", () => {
    it("should load allowlist from file", () => {
      expect(validator.isCommandAllowed("npm")).toBe(true);
      expect(validator.isCommandAllowed("git")).toBe(true);
    });

    it("should throw error if allowlist file not found", () => {
      expect(() => {
        new CommandValidator({
          allowlistPath: "/nonexistent/path.json",
        });
      }).toThrow("allowlist");
    });

    it("should throw error if allowlist is invalid JSON", () => {
      const invalidPath = path.join(__dirname, "../../fixtures/invalid.json");
      fs.writeFileSync(invalidPath, "not valid json");

      expect(() => {
        new CommandValidator({
          allowlistPath: invalidPath,
        });
      }).toThrow();

      fs.unlinkSync(invalidPath);
    });

    it("should throw error if allowlist is not an array", () => {
      const invalidPath = path.join(__dirname, "../../fixtures/invalid2.json");
      fs.writeFileSync(invalidPath, JSON.stringify({ commands: ["npm"] }));

      expect(() => {
        new CommandValidator({
          allowlistPath: invalidPath,
        });
      }).toThrow();

      fs.unlinkSync(invalidPath);
    });
  });

  describe("edge cases", () => {
    it("should handle very long command names", () => {
      const longCommand = "a".repeat(1000);
      expect(validator.isCommandAllowed(longCommand)).toBe(false);
    });

    it("should handle unicode characters", () => {
      expect(validator.isCommandAllowed("npm™")).toBe(false);
      expect(validator.isCommandAllowed("git∆")).toBe(false);
    });

    it("should handle special characters in paths", () => {
      expect(validator.isCommandAllowed("npm-cli")).toBe(false);
      expect(validator.isCommandAllowed("git_wrapper")).toBe(false);
    });

    it("should handle arguments with quotes", () => {
      const result = validator.validateArguments([
        '--message="test message"',
        "--name='value'",
      ]);
      expect(result.valid).toBe(true);
    });

    it("should handle mixed safe and unsafe arguments", () => {
      const result = validator.validateArguments([
        "test",
        "--coverage",
        ";rm -rf /",
      ]);
      expect(result.valid).toBe(false);
    });
  });
});
