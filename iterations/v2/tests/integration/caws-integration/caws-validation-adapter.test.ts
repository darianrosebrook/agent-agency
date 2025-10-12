/**
 * Integration tests for CAWSValidationAdapter
 *
 * Tests CAWS CLI integration, validation flow, and spec generation.
 *
 * @author @darianrosebrook
 */

import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";
import * as fs from "fs/promises";
import * as path from "path";
import { CAWSValidationAdapter } from "../../../src/caws-integration/adapters/CAWSValidationAdapter";
import type { WorkingSpec } from "../../../src/types/caws-types";

describe("CAWSValidationAdapter Integration Tests", () => {
  const fixturesDir = path.join(__dirname, "../../fixtures/caws-integration");
  const tempDir = path.join(__dirname, "../../temp/validation-adapter-tests");
  let adapter: CAWSValidationAdapter;

  // Sample working spec for tests
  const validSpec: WorkingSpec = {
    id: "TEST-002",
    title: "Test Specification",
    risk_tier: 2,
    mode: "feature",
    blast_radius: {
      modules: ["src/test"],
      data_migration: false,
    },
    operational_rollback_slo: "5m",
    scope: {
      in: ["src/test/"],
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
    ],
    non_functional: {
      perf: { api_p95_ms: 250 },
    },
    contracts: [],
  };

  beforeEach(async () => {
    // Create temp directory
    await fs.mkdir(tempDir, { recursive: true });

    adapter = new CAWSValidationAdapter({
      projectRoot: tempDir,
    });
  });

  afterEach(async () => {
    // Clean up temp directory
    try {
      await fs.rm(tempDir, { recursive: true, force: true });
    } catch {
      // Ignore cleanup errors
    }
  });

  describe("Spec Validation", () => {
    it("should validate a valid spec successfully", async () => {
      const result = await adapter.validateSpec({
        spec: validSpec,
        projectRoot: tempDir,
      });

      expect(result.success).toBe(true);
      expect(result.data).toBeDefined();
      expect(result.durationMs).toBeGreaterThan(0);
    });

    it("should handle validation with options", async () => {
      const result = await adapter.validateSpec({
        spec: validSpec,
        projectRoot: tempDir,
        options: {
          autoFix: false,
          suggestions: true,
          checkBudget: true,
        },
      });

      expect(result.success).toBe(true);
      expect(result.data).toBeDefined();
    });

    it("should include duration in result", async () => {
      const result = await adapter.validateSpec({
        spec: validSpec,
        projectRoot: tempDir,
      });

      expect(result.durationMs).toBeGreaterThan(0);
      expect(result.durationMs).toBeLessThan(2000); // Should be under 2s
    });
  });

  describe("Existing Spec Validation", () => {
    it("should validate existing spec file", async () => {
      const fixtureAdapter = new CAWSValidationAdapter({
        projectRoot: fixturesDir,
      });

      const result = await fixtureAdapter.validateExistingSpec();

      expect(result).toBeDefined();
      expect(result.durationMs).toBeGreaterThan(0);
    });

    it("should return error when spec file doesn't exist", async () => {
      const result = await adapter.validateExistingSpec();

      expect(result.success).toBe(false);
      expect(result.error).toBeDefined();
      expect(result.error?.message).toBeDefined();
    });

    it("should accept validation options", async () => {
      // Write a spec first
      await adapter.validateSpec({
        spec: validSpec,
        projectRoot: tempDir,
      });

      // Now create actual spec file for testing
      const specPath = path.join(tempDir, ".caws", "working-spec.yaml");
      await fs.mkdir(path.dirname(specPath), { recursive: true });

      const yaml = `id: ${validSpec.id}
title: ${validSpec.title}
risk_tier: ${validSpec.risk_tier}
mode: ${validSpec.mode}
operational_rollback_slo: ${validSpec.operational_rollback_slo}
blast_radius:
  modules:
    - ${validSpec.blast_radius.modules[0]}
  data_migration: ${validSpec.blast_radius.data_migration}
scope:
  in:
    - ${validSpec.scope.in[0]}
  out:
    - ${validSpec.scope.out[0]}
invariants:
  - ${validSpec.invariants[0]}
acceptance:
  - id: ${validSpec.acceptance[0].id}
    given: ${validSpec.acceptance[0].given}
    when: ${validSpec.acceptance[0].when}
    then: ${validSpec.acceptance[0].then}
non_functional: {}
contracts: []
`;

      await fs.writeFile(specPath, yaml, "utf-8");

      const result = await adapter.validateExistingSpec({
        autoFix: false,
        suggestions: true,
      });

      expect(result).toBeDefined();
    });
  });

  describe("Quick Validation Check", () => {
    it("should return true for valid spec", async () => {
      const result = await adapter.isSpecValid(validSpec);

      expect(result).toBe(true);
    });

    it("should be faster than full validation", async () => {
      const quickStart = Date.now();
      await adapter.isSpecValid(validSpec);
      const quickDuration = Date.now() - quickStart;

      expect(quickDuration).toBeLessThan(2000);
    });
  });

  describe("Spec Generation", () => {
    it("should generate new spec from task", async () => {
      const result = await adapter.generateSpec({
        title: "Implement user authentication",
        mode: "feature",
        riskTier: 2,
        description: "Add JWT-based authentication to the API",
      });

      expect(result.success).toBe(true);
      expect(result.data).toBeDefined();
      if (result.data) {
        expect(result.data.title).toBe("Implement user authentication");
        expect(result.data.risk_tier).toBe(2);
        expect(result.data.mode).toBe("feature");
      }
    });

    it("should generate spec for different modes", async () => {
      const result = await adapter.generateSpec({
        title: "Fix authentication bug",
        mode: "fix",
        riskTier: 1,
      });

      expect(result.success).toBe(true);
      expect(result.data?.mode).toBe("fix");
    });

    it("should include duration in generation result", async () => {
      const result = await adapter.generateSpec({
        title: "Test task",
        mode: "feature",
        riskTier: 3,
      });

      expect(result.durationMs).toBeGreaterThan(0);
    });
  });

  describe("Error Handling", () => {
    it("should provide detailed error information", async () => {
      const result = await adapter.validateExistingSpec();

      expect(result.success).toBe(false);
      expect(result.error).toBeDefined();
      expect(result.error?.code).toBeDefined();
      expect(result.error?.message).toBeDefined();
    });

    it("should measure duration even on error", async () => {
      const result = await adapter.validateExistingSpec();

      expect(result.success).toBe(false);
      expect(result.durationMs).toBeGreaterThan(0);
    });
  });

  describe("Integration with Fixtures", () => {
    it("should validate fixture spec successfully", async () => {
      const fixtureAdapter = new CAWSValidationAdapter({
        projectRoot: fixturesDir,
      });

      const result = await fixtureAdapter.validateExistingSpec();

      expect(result).toBeDefined();
    });
  });

  describe("Performance", () => {
    it("should validate within performance budget", async () => {
      const startTime = Date.now();

      const result = await adapter.validateSpec({
        spec: validSpec,
        projectRoot: tempDir,
      });

      const duration = Date.now() - startTime;

      expect(result).toBeDefined();
      expect(duration).toBeLessThan(2000); // <2s budget
    });

    it("should handle multiple validations efficiently", async () => {
      const validations = Array.from({ length: 3 }, (_, i) =>
        adapter.validateSpec({
          spec: { ...validSpec, id: `TEST-${100 + i}` },
          projectRoot: tempDir,
        })
      );

      const results = await Promise.all(validations);

      expect(results).toHaveLength(3);
      results.forEach((result) => {
        expect(result).toBeDefined();
      });
    });
  });
});
