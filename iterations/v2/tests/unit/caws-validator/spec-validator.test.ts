/**
 * @fileoverview Unit Tests for SpecValidator
 * Comprehensive test suite for working spec validation
 * @module tests/unit/caws-validator
 */

import { beforeEach, describe, expect, it } from "@jest/globals";
import { SpecValidator } from "../../../src/caws-validator/validation/SpecValidator";
import type { WorkingSpec } from "../../../src/types/caws-types";

describe("SpecValidator", () => {
  let validator: SpecValidator;

  beforeEach(() => {
    validator = new SpecValidator();
  });

  // Helper to create valid spec
  const createValidSpec = (): WorkingSpec => ({
    id: "FEAT-001",
    title: "Test Feature",
    risk_tier: 2,
    mode: "feature",
    blast_radius: {
      modules: ["src/features"],
      data_migration: false,
    },
    operational_rollback_slo: "5m",
    scope: {
      in: ["src/features/", "tests/"],
      out: ["node_modules/", "dist/"],
    },
    invariants: ["System remains stable", "Data consistency maintained"],
    acceptance: [
      {
        id: "A1",
        given: "User is logged in",
        when: "User clicks submit",
        then: "Data is saved",
      },
    ],
    non_functional: {
      a11y: ["keyboard-navigation"],
      perf: {
        api_p95_ms: 250,
      },
      security: ["input-validation"],
    },
    contracts: [
      {
        type: "openapi",
        path: "docs/api.yaml",
      },
    ],
  });

  describe("validateWorkingSpec", () => {
    it("should pass with valid spec", async () => {
      const spec = createValidSpec();
      const result = await validator.validateWorkingSpec(spec);

      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it("should fail with missing required fields", async () => {
      const spec = {
        title: "Test",
      } as WorkingSpec;

      const result = await validator.validateWorkingSpec(spec);

      expect(result.valid).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors.some((e) => e.field === "id")).toBe(true);
      expect(result.errors.some((e) => e.field === "risk_tier")).toBe(true);
    });

    it("should provide suggestions for missing fields", () => {
      const spec = {
        title: "Test",
      } as WorkingSpec;

      const result = validator.validateWorkingSpec(spec);

      const idError = result.errors.find((e) => e.field === "id");
      expect(idError).toBeDefined();
      expect(idError?.suggestion).toBeDefined();
      expect(idError?.suggestion).toContain("PROJ-001");
    });

    it("should fail with invalid ID format", () => {
      const spec = createValidSpec();
      spec.id = "invalid";

      const result = validator.validateWorkingSpec(spec);

      expect(result.valid).toBe(false);
      expect(result.errors.some((e) => e.field === "id")).toBe(true);
    });

    it("should accept valid ID formats", () => {
      const validIds = ["FEAT-001", "FIX-123", "REFACTOR-999", "DOC-042"];

      for (const id of validIds) {
        const spec = createValidSpec();
        spec.id = id;
        const result = validator.validateWorkingSpec(spec);

        expect(result.valid).toBe(true);
      }
    });

    it("should fail with invalid risk tier", () => {
      const spec = createValidSpec();
      spec.risk_tier = 5;

      const result = validator.validateWorkingSpec(spec);

      expect(result.valid).toBe(false);
      expect(result.errors.some((e) => e.field === "risk_tier")).toBe(true);
    });

    it("should suggest auto-fix for invalid risk tier", () => {
      const spec = createValidSpec();
      spec.risk_tier = 5;

      const result = validator.validateWorkingSpec(spec);

      expect(result.fixes).toBeDefined();
      expect(result.fixes?.length).toBeGreaterThan(0);
      const fix = result.fixes?.find((f) => f.field === "risk_tier");
      expect(fix).toBeDefined();
      expect(fix?.value).toBe(3);
    });

    it("should accept valid risk tiers", () => {
      const tiers = [1, 2, 3];

      for (const tier of tiers) {
        const spec = createValidSpec();
        spec.risk_tier = tier;
        const result = validator.validateWorkingSpec(spec);

        expect(result.valid).toBe(true);
      }
    });

    it("should fail with invalid mode", () => {
      const spec = createValidSpec();
      spec.mode = "invalid" as any;

      const result = validator.validateWorkingSpec(spec);

      expect(result.valid).toBe(false);
      expect(result.errors.some((e) => e.field === "mode")).toBe(true);
    });

    it("should accept valid modes", () => {
      const modes: Array<"feature" | "refactor" | "fix" | "doc" | "chore"> = [
        "feature",
        "refactor",
        "fix",
        "doc",
        "chore",
      ];

      for (const mode of modes) {
        const spec = createValidSpec();
        spec.mode = mode;
        const result = validator.validateWorkingSpec(spec);

        expect(result.valid).toBe(true);
      }
    });

    it("should fail with empty scope.in", () => {
      const spec = createValidSpec();
      spec.scope.in = [];

      const result = validator.validateWorkingSpec(spec);

      expect(result.valid).toBe(false);
      expect(result.errors.some((e) => e.field === "scope.in")).toBe(true);
    });

    it("should fail with missing scope", () => {
      const spec = createValidSpec();
      delete (spec as any).scope;

      const result = validator.validateWorkingSpec(spec);

      expect(result.valid).toBe(false);
      expect(result.errors.some((e) => e.field === "scope")).toBe(true);
    });

    it("should require contracts for Tier 1", () => {
      const spec = createValidSpec();
      spec.risk_tier = 1;
      spec.contracts = [];

      const result = validator.validateWorkingSpec(spec);

      expect(result.valid).toBe(false);
      expect(result.errors.some((e) => e.field === "contracts")).toBe(true);
    });

    it("should require contracts for Tier 2", () => {
      const spec = createValidSpec();
      spec.risk_tier = 2;
      spec.contracts = [];

      const result = validator.validateWorkingSpec(spec);

      expect(result.valid).toBe(false);
      expect(result.errors.some((e) => e.field === "contracts")).toBe(true);
    });

    it("should not require contracts for Tier 3", () => {
      const spec = createValidSpec();
      spec.risk_tier = 3;
      spec.contracts = [];

      const result = validator.validateWorkingSpec(spec);

      expect(result.valid).toBe(true);
    });

    it("should require observability for Tier 1", () => {
      const spec = createValidSpec();
      spec.risk_tier = 1;
      delete spec.observability;

      const result = validator.validateWorkingSpec(spec);

      expect(result.valid).toBe(false);
      expect(result.errors.some((e) => e.field === "observability")).toBe(true);
    });

    it("should require rollback procedures for Tier 1", () => {
      const spec = createValidSpec();
      spec.risk_tier = 1;
      spec.rollback = [];

      const result = validator.validateWorkingSpec(spec);

      expect(result.valid).toBe(false);
      expect(result.errors.some((e) => e.field === "rollback")).toBe(true);
    });

    it("should require security requirements for Tier 1", () => {
      const spec = createValidSpec();
      spec.risk_tier = 1;
      spec.non_functional.security = [];

      const result = validator.validateWorkingSpec(spec);

      expect(result.valid).toBe(false);
      expect(
        result.errors.some((e) => e.field === "non_functional.security")
      ).toBe(true);
    });

    it("should warn about missing invariants", () => {
      const spec = createValidSpec();
      spec.invariants = [];

      const result = validator.validateWorkingSpec(spec);

      expect(result.warnings.length).toBeGreaterThan(0);
      expect(result.warnings.some((w) => w.field === "invariants")).toBe(true);
    });

    it("should warn about missing acceptance criteria", () => {
      const spec = createValidSpec();
      spec.acceptance = [];

      const result = validator.validateWorkingSpec(spec);

      expect(result.warnings.length).toBeGreaterThan(0);
      expect(result.warnings.some((w) => w.field === "acceptance")).toBe(true);
    });
  });

  describe("validateWithSuggestions", () => {
    it("should apply auto-fixes when enabled", () => {
      const spec = createValidSpec();
      spec.risk_tier = 5;

      const result = validator.validateWithSuggestions(spec, { autoFix: true });

      expect(spec.risk_tier).toBe(3);
    });

    it("should not apply auto-fixes when disabled", () => {
      const spec = createValidSpec();
      const originalTier = spec.risk_tier;
      spec.risk_tier = 5;

      validator.validateWithSuggestions(spec, { autoFix: false });

      expect(spec.risk_tier).toBe(5);
    });
  });

  describe("experimental mode validation", () => {
    it("should require all experimental mode fields", () => {
      const spec = createValidSpec();
      spec.experimental_mode = {
        enabled: true,
      } as any;

      const result = validator.validateWorkingSpec(spec);

      expect(result.valid).toBe(false);
      expect(
        result.errors.some((e) => e.field.startsWith("experimental_mode"))
      ).toBe(true);
    });

    it("should only allow experimental mode for Tier 3", () => {
      const spec = createValidSpec();
      spec.risk_tier = 2;
      spec.experimental_mode = {
        enabled: true,
        rationale: "Testing new feature",
        expires_at: "2025-12-31",
      };

      const result = validator.validateWorkingSpec(spec);

      expect(result.valid).toBe(false);
      expect(result.errors.some((e) => e.field === "experimental_mode")).toBe(
        true
      );
    });

    it("should allow experimental mode for Tier 3", () => {
      const spec = createValidSpec();
      spec.risk_tier = 3;
      spec.experimental_mode = {
        enabled: true,
        rationale: "Testing new feature",
        expires_at: "2025-12-31",
      };

      const result = validator.validateWorkingSpec(spec);

      expect(result.valid).toBe(true);
    });

    it("should fail with expired experimental mode", () => {
      const spec = createValidSpec();
      spec.risk_tier = 3;
      spec.experimental_mode = {
        enabled: true,
        rationale: "Testing new feature",
        expires_at: "2020-01-01",
      };

      const result = validator.validateWorkingSpec(spec);

      expect(result.valid).toBe(false);
      expect(
        result.errors.some((e) => e.field === "experimental_mode.expires_at")
      ).toBe(true);
    });
  });
});
