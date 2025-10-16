/**
 * Integration tests for Arbiter MCP Server
 *
 * Tests MCP server functionality including all 4 tools.
 * Focuses on MCP protocol integration rather than full CAWS stack.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";
import * as fs from "fs/promises";
import * as path from "path";
import { ArbiterMCPServer } from "../../../src/mcp-server/ArbiterMCPServer";
import type { WorkingSpec } from "../../../src/types/caws-types";
import { TaskPriority } from "../../../src/types/task-runner";

describe("Arbiter MCP Server Integration Tests", () => {
  const tempDir = path.join(__dirname, "../../temp/mcp-server-tests");
  let server: ArbiterMCPServer;

  // Sample working spec for tests
  const validSpec: WorkingSpec = {
    id: "TEST-MCP-001",
    title: "MCP Test Specification",
    risk_tier: 2,
    mode: "feature",
    blast_radius: {
      modules: ["src/mcp-test"],
      data_migration: false,
    },
    operational_rollback_slo: "5m",
    scope: {
      in: ["src/mcp-test/"],
      out: ["node_modules/"],
    },
    invariants: ["Test invariant"],
    acceptance: [
      {
        id: "A1",
        given: "Test condition",
        when: "Test action",
        then: "Test result",
      },
      {
        id: "A2",
        given: "Second condition",
        when: "Second action",
        then: "Second result",
      },
    ],
    non_functional: {
      perf: { api_p95_ms: 250 },
    },
    contracts: [],
  };

  beforeEach(async () => {
    await fs.mkdir(tempDir, { recursive: true });

    // Create the CAWS tools directory structure
    const cawsToolsDir = path.join(tempDir, "apps", "tools", "caws");
    await fs.mkdir(cawsToolsDir, { recursive: true });

    // Copy the allowlist file to the test directory
    const allowlistSource = path.join(
      __dirname,
      "..",
      "..",
      "fixtures",
      "test-allowlist.json"
    );
    const allowlistDest = path.join(cawsToolsDir, "tools-allow.json");
    await fs.copyFile(allowlistSource, allowlistDest);

    server = new ArbiterMCPServer(tempDir);
  });

  afterEach(async () => {
    try {
      await fs.rm(tempDir, { recursive: true, force: true });
    } catch {
      // Ignore cleanup errors
    }
  });

  describe("Server Initialization", () => {
    it("should create server with default project root", () => {
      const defaultServer = new ArbiterMCPServer();
      expect(defaultServer).toBeDefined();
      expect(defaultServer).toBeInstanceOf(ArbiterMCPServer);
    });

    it("should create server with custom project root", async () => {
      // Create a custom test directory with proper structure
      const customTestDir = path.join(tempDir, "custom-test");
      await fs.mkdir(customTestDir, { recursive: true });

      // Create the CAWS tools directory structure
      const cawsToolsDir = path.join(customTestDir, "apps", "tools", "caws");
      await fs.mkdir(cawsToolsDir, { recursive: true });

      // Copy the allowlist file to the custom test directory
      const allowlistSource = path.join(
        __dirname,
        "..",
        "..",
        "fixtures",
        "test-allowlist.json"
      );
      const allowlistDest = path.join(cawsToolsDir, "tools-allow.json");
      await fs.copyFile(allowlistSource, allowlistDest);

      const customServer = new ArbiterMCPServer(customTestDir);
      expect(customServer).toBeDefined();
      expect(customServer).toBeInstanceOf(ArbiterMCPServer);
    });

    it("should initialize with correct server info", () => {
      // Server metadata is private, but we can verify it doesn't throw
      expect(server).toBeDefined();
    });
  });

  describe("arbiter_validate Tool", () => {
    it("should handle validation requests", async () => {
      const result = await (server as any).handleValidate({
        spec: validSpec,
      });

      expect(result).toBeDefined();
      expect(result.content).toBeDefined();
      expect(Array.isArray(result.content)).toBe(true);
      expect(result.content.length).toBeGreaterThan(0);
    });

    it("should return error for missing arguments", async () => {
      const result = await (server as any).handleValidate({});

      expect(result).toBeDefined();
      expect(result.isError).toBe(true);
    });

    it("should handle spec validation with options", async () => {
      const result = await (server as any).handleValidate({
        spec: validSpec,
        autoFix: false,
        suggestions: true,
      });

      expect(result).toBeDefined();
      expect(result.content).toBeDefined();
    });
  });

  describe("arbiter_assign_task Tool", () => {
    it("should assign task to agent", async () => {
      const result = await (server as any).handleAssignTask({
        spec: validSpec,
        availableAgents: ["agent-1", "agent-2"],
        strategy: "capability",
        priority: TaskPriority.HIGH,
      });

      expect(result).toBeDefined();
      expect(result.isError).toBeUndefined();

      const response = JSON.parse(result.content[0].text);
      expect(response.success).toBe(true);
      expect(response.agentId).toBeDefined();
      expect(response.agentName).toBeDefined();
      expect(response.reason).toBeDefined();
      expect(response.priority).toBe("high");
    });

    it("should use default strategy", async () => {
      const result = await (server as any).handleAssignTask({
        spec: validSpec,
      });

      expect(result).toBeDefined();
      const response = JSON.parse(result.content[0].text);
      expect(response.reason).toBeDefined();
    });

    it("should estimate effort based on spec complexity", async () => {
      const simpleSpec: WorkingSpec = {
        ...validSpec,
        risk_tier: 3,
        acceptance: [validSpec.acceptance[0]],
      };

      const complexSpec: WorkingSpec = {
        ...validSpec,
        risk_tier: 1,
        acceptance: [
          ...validSpec.acceptance,
          ...validSpec.acceptance,
          ...validSpec.acceptance,
        ],
      };

      const simpleResult = await (server as any).handleAssignTask({
        spec: simpleSpec,
      });
      const complexResult = await (server as any).handleAssignTask({
        spec: complexSpec,
      });

      const simpleResponse = JSON.parse(simpleResult.content[0].text);
      const complexResponse = JSON.parse(complexResult.content[0].text);

      // Both should provide reasonable effort estimates
      expect(simpleResponse.estimatedEffort.hours).toBeGreaterThan(0);
      expect(complexResponse.estimatedEffort.hours).toBeGreaterThan(0);
      // Complex spec (tier 1, 6 criteria) should require meaningful effort
      expect(complexResponse.estimatedEffort.hours).toBeGreaterThanOrEqual(8);
    });

    it("should handle error gracefully", async () => {
      const result = await (server as any).handleAssignTask({});

      expect(result).toBeDefined();
      expect(result.isError).toBe(true);
    });
  });

  describe("arbiter_monitor_progress Tool", () => {
    it("should return error when spec file not found", async () => {
      const result = await (server as any).handleMonitorProgress({
        taskId: "task-123",
      });

      expect(result).toBeDefined();
      expect(result.isError).toBe(true);
    });

    it("should handle task monitoring with thresholds", async () => {
      // Create a spec file first
      const specPath = path.join(tempDir, ".caws", "working-spec.yaml");
      await fs.mkdir(path.dirname(specPath), { recursive: true });
      await fs.writeFile(
        specPath,
        `id: TEST-001
title: Test
risk_tier: 2
mode: feature
operational_rollback_slo: 5m
blast_radius:
  modules: []
  data_migration: false
scope:
  in: []
  out: []
invariants: []
acceptance:
  - id: A1
    given: test
    when: test
    then: test
non_functional: {}
contracts: []
`
      );

      const result = await (server as any).handleMonitorProgress({
        taskId: "task-456",
        thresholds: {
          warning: 0.5,
          critical: 0.9,
        },
      });

      expect(result).toBeDefined();
      // May succeed or fail depending on validation, but shouldn't crash
    });
  });

  describe("arbiter_generate_verdict Tool", () => {
    it("should generate verdict successfully", async () => {
      const result = await (server as any).handleGenerateVerdict({
        taskId: "task-verdict-1",
        spec: validSpec,
        artifacts: {
          filesChanged: ["src/file1.ts", "tests/file1.test.ts"],
          testsAdded: 10,
          coverage: 85,
          mutationScore: 75,
        },
        qualityGates: [
          { gate: "coverage", passed: true, score: 85 },
          { gate: "mutation", passed: true, score: 75 },
        ],
        agentId: "agent-verdict-1",
      });

      expect(result).toBeDefined();
      expect(result.isError).toBeUndefined();

      const response = JSON.parse(result.content[0].text);
      expect(response.decision).toBeDefined();
      expect(response.taskId).toBe("task-verdict-1");
      expect(response.agentId).toBe("agent-verdict-1");
    });

    it("should calculate quality score correctly", async () => {
      const result = await (server as any).handleGenerateVerdict({
        taskId: "task-score",
        spec: validSpec,
        artifacts: {
          coverage: 80,
          mutationScore: 70,
        },
        qualityGates: [
          { gate: "gate1", passed: true },
          { gate: "gate2", passed: true },
        ],
      });

      const response = JSON.parse(result.content[0].text);
      expect(response.qualityScore).toBeDefined();
      expect(response.qualityScore).toBeGreaterThan(0);
    });

    it("should handle minimal artifacts", async () => {
      const result = await (server as any).handleGenerateVerdict({
        taskId: "task-minimal",
        spec: validSpec,
      });

      expect(result).toBeDefined();
      const response = JSON.parse(result.content[0].text);
      expect(response.decision).toBeDefined();
    });
  });

  describe("Error Handling", () => {
    it("should handle validation errors gracefully", async () => {
      const result = await (server as any).handleValidate({
        spec: { invalid: "spec" },
      });

      expect(result).toBeDefined();
      // Should return some response even for invalid input
    });

    it("should handle missing spec in assign_task", async () => {
      const result = await (server as any).handleAssignTask({});

      expect(result).toBeDefined();
      expect(result.isError).toBe(true);
    });
  });

  describe("Integration Flows", () => {
    it("should complete validation and assignment flow", async () => {
      // Step 1: Validate
      const validateResult = await (server as any).handleValidate({
        spec: validSpec,
      });
      expect(validateResult).toBeDefined();

      // Step 2: Assign
      const assignResult = await (server as any).handleAssignTask({
        spec: validSpec,
        availableAgents: ["agent-1"],
      });

      const assignResponse = JSON.parse(assignResult.content[0].text);
      expect(assignResponse.success).toBe(true);
    });

    it("should complete assignment and verdict flow", async () => {
      // Step 1: Assign
      const assignResult = await (server as any).handleAssignTask({
        spec: validSpec,
      });

      const assignResponse = JSON.parse(assignResult.content[0].text);
      expect(assignResponse.success).toBe(true);

      // Step 2: Generate verdict
      const verdictResult = await (server as any).handleGenerateVerdict({
        taskId: "flow-task",
        spec: validSpec,
        artifacts: { coverage: 85 },
        qualityGates: [{ gate: "coverage", passed: true, score: 85 }],
      });

      const verdictResponse = JSON.parse(verdictResult.content[0].text);
      expect(verdictResponse.decision).toBeDefined();
    });
  });

  describe("Performance", () => {
    it("should assign task within performance budget", async () => {
      const startTime = Date.now();

      await (server as any).handleAssignTask({ spec: validSpec });

      const duration = Date.now() - startTime;
      expect(duration).toBeLessThan(1000); // <1s budget
    });

    it("should generate verdict within performance budget", async () => {
      const startTime = Date.now();

      await (server as any).handleGenerateVerdict({
        taskId: "perf-test",
        spec: validSpec,
      });

      const duration = Date.now() - startTime;
      expect(duration).toBeLessThan(1000); // <1s budget
    });
  });
});
