/**
 * @fileoverview Unit Tests for RuleEngine
 * Comprehensive test suite for CAWS policy rule evaluation
 * @module tests/unit/caws-validator
 */

import { beforeEach, describe, expect, it } from "@jest/globals";
import { RuleEngine } from "../../../src/caws-validator/validation/RuleEngine";
import type { CAWSPolicy, WorkingSpec } from "../../../src/types/caws-types";

describe("RuleEngine", () => {
  let ruleEngine: RuleEngine;
  let mockPolicy: CAWSPolicy;

  const createMockPolicy = (): CAWSPolicy => ({
    version: "3.1.0",
    risk_tiers: {
      1: {
        max_files: 25,
        max_loc: 1000,
        coverage_threshold: 90,
        mutation_threshold: 70,
        contracts_required: true,
        manual_review_required: true,
      },
      2: {
        max_files: 50,
        max_loc: 2000,
        coverage_threshold: 80,
        mutation_threshold: 50,
        contracts_required: true,
        manual_review_required: false,
      },
      3: {
        max_files: 100,
        max_loc: 5000,
        coverage_threshold: 70,
        mutation_threshold: 30,
        contracts_required: false,
        manual_review_required: false,
      },
    },
  });

  const createValidSpec = (
    overrides: Partial<WorkingSpec> = {}
  ): WorkingSpec => ({
    id: "FEAT-001",
    title: "Test Feature",
    risk_tier: 2,
    mode: "feature",
    blast_radius: {
      modules: ["ui", "api"],
      data_migration: false,
    },
    operational_rollback_slo: "5m",
    scope: {
      in: ["src/features/", "tests/"],
      out: ["src/legacy/", "docs/"],
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
      perf: { api_p95_ms: 250 },
      security: ["input-validation"],
    },
    contracts: [
      {
        type: "openapi",
        path: "docs/api.yaml",
      },
    ],
    ...overrides,
  });

  beforeEach(() => {
    mockPolicy = createMockPolicy();
    ruleEngine = new RuleEngine(mockPolicy);
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe("evaluateRules", () => {
    it("should pass with valid spec", () => {
      const spec = createValidSpec();
      const result = ruleEngine.evaluateRules(spec);

      expect(result.passed).toBe(true);
      expect(result.violations).toHaveLength(0);
      expect(result.evaluations.length).toBeGreaterThan(0);
    });

    it("should include evaluation metadata", () => {
      const spec = createValidSpec();
      const result = ruleEngine.evaluateRules(spec);

      expect(result.evaluatedAt).toBeDefined();
      expect(typeof result.evaluatedAt).toBe("string");
      expect(result.evaluations.length).toBeGreaterThan(0);

      // Check that evaluations have required fields
      for (const evaluation of result.evaluations) {
        expect(evaluation.ruleId).toBeDefined();
        expect(evaluation.description).toBeDefined();
        expect(typeof evaluation.passed).toBe("boolean");
      }
    });
  });

  describe("scope rules", () => {
    it("should fail with empty scope.in", () => {
      const spec = createValidSpec({
        scope: { in: [], out: [] },
      });

      const result = ruleEngine.evaluateRules(spec);

      expect(result.passed).toBe(false);
      const violation = result.violations.find((v) => v.ruleId === "SCOPE-001");
      expect(violation).toBeDefined();
      expect(violation?.field).toBe("scope.in");
      expect(violation?.severity).toBe("high");
    });

    it("should warn about critical directories in scope.out", () => {
      const spec = createValidSpec({
        scope: {
          in: ["src/"],
          out: ["node_modules/", "src/"],
        },
      });

      const result = ruleEngine.evaluateRules(spec);

      const violation = result.violations.find(
        (v) =>
          v.ruleId === "SCOPE-001" && v.message.includes("critical directory")
      );
      expect(violation).toBeDefined();
      expect(violation?.severity).toBe("medium");
    });

    it("should fail with overly broad scope.in", () => {
      const spec = createValidSpec({
        scope: {
          in: ["/"],
          out: [],
        },
      });

      const result = ruleEngine.evaluateRules(spec);

      const violation = result.violations.find(
        (v) => v.ruleId === "SCOPE-001" && v.message.includes("too broad")
      );
      expect(violation).toBeDefined();
      expect(violation?.severity).toBe("high");
    });
  });

  describe("blast radius rules", () => {
    it("should fail with empty blast radius modules", () => {
      const spec = createValidSpec({
        blast_radius: {
          modules: [],
          data_migration: false,
        },
      });

      const result = ruleEngine.evaluateRules(spec);

      expect(result.passed).toBe(false);
      const violation = result.violations.find((v) => v.ruleId === "BLAST-001");
      expect(violation).toBeDefined();
      expect(violation?.field).toBe("blast_radius.modules");
    });

    it("should warn about missing data migration flag", () => {
      const spec = createValidSpec({
        blast_radius: {
          modules: ["auth"],
          data_migration: false,
        },
      });
      // Remove the data_migration property to test undefined check
      const specWithoutMigration = {
        ...spec,
        blast_radius: {
          ...spec.blast_radius,
          data_migration: undefined as any,
        },
      };

      const result = ruleEngine.evaluateRules(specWithoutMigration);

      const violation = result.violations.find(
        (v) =>
          v.ruleId === "BLAST-001" && v.message.includes("explicitly stated")
      );
      expect(violation).toBeDefined();
      expect(violation?.severity).toBe("medium");
    });

    it("should warn about high-risk modules without justification", () => {
      const spec = createValidSpec({
        blast_radius: {
          modules: ["auth"],
          data_migration: false,
        },
        invariants: ["Some other invariant"],
      });

      const result = ruleEngine.evaluateRules(spec);

      const violation = result.violations.find(
        (v) =>
          v.ruleId === "BLAST-001" &&
          v.message.includes("requires explicit justification")
      );
      expect(violation).toBeDefined();
      expect(violation?.severity).toBe("medium");
    });

    it("should pass with justified high-risk modules", () => {
      const spec = createValidSpec({
        blast_radius: {
          modules: ["auth"],
          data_migration: false,
        },
        invariants: ["Auth module change is approved and justified"],
      });

      const result = ruleEngine.evaluateRules(spec);

      const violation = result.violations.find(
        (v) =>
          v.ruleId === "BLAST-001" &&
          v.message.includes("requires explicit justification")
      );
      expect(violation).toBeUndefined();
    });
  });

  describe("rollback rules", () => {
    it("should fail with invalid rollback SLO", () => {
      const spec = createValidSpec({
        operational_rollback_slo: "invalid",
      });

      const result = ruleEngine.evaluateRules(spec);

      const violation = result.violations.find(
        (v) => v.ruleId === "ROLLBACK-001"
      );
      expect(violation).toBeDefined();
      expect(violation?.field).toBe("operational_rollback_slo");
    });

    it("should accept valid rollback SLOs", () => {
      const validSLOs = ["1m", "5m", "15m", "1h", "4h", "24h"];

      for (const slo of validSLOs) {
        const spec = createValidSpec({
          operational_rollback_slo: slo,
        });

        const result = ruleEngine.evaluateRules(spec);
        const violation = result.violations.find(
          (v) =>
            v.ruleId === "ROLLBACK-001" &&
            v.field === "operational_rollback_slo"
        );
        expect(violation).toBeUndefined();
      }
    });

    it("should require rollback procedures for Tier 1", () => {
      const spec = createValidSpec({
        risk_tier: 1,
        rollback: [],
      });

      const result = ruleEngine.evaluateRules(spec);

      const violation = result.violations.find(
        (v) => v.ruleId === "ROLLBACK-001" && v.message.includes("Tier 1")
      );
      expect(violation).toBeDefined();
      expect(violation?.severity).toBe("high");
    });

    it("should warn about non-testable rollback procedures", () => {
      const spec = createValidSpec({
        rollback: ["Revert the changes", "Restart the service"],
      });

      const result = ruleEngine.evaluateRules(spec);

      const violation = result.violations.find(
        (v) =>
          v.ruleId === "ROLLBACK-001" &&
          v.message.includes("verification steps")
      );
      expect(violation).toBeDefined();
      expect(violation?.severity).toBe("low");
    });
  });

  describe("acceptance criteria rules", () => {
    it("should fail with no acceptance criteria", () => {
      const spec = createValidSpec({
        acceptance: [],
      });

      const result = ruleEngine.evaluateRules(spec);

      const violation = result.violations.find(
        (v) => v.ruleId === "ACCEPTANCE-001"
      );
      expect(violation).toBeDefined();
      expect(violation?.severity).toBe("high");
    });

    it("should fail with missing required fields in acceptance criteria", () => {
      const spec = createValidSpec({
        acceptance: [
          { id: "A1", given: "test", when: "test" }, // missing "then"
        ] as any,
      });

      const result = ruleEngine.evaluateRules(spec);

      const violation = result.violations.find(
        (v) =>
          v.ruleId === "ACCEPTANCE-001" &&
          v.message.includes("missing required field")
      );
      expect(violation).toBeDefined();
    });

    it("should fail with duplicate acceptance IDs", () => {
      const spec = createValidSpec({
        acceptance: [
          { id: "A1", given: "test", when: "test", then: "test" },
          { id: "A1", given: "test2", when: "test2", then: "test2" },
        ],
      });

      const result = ruleEngine.evaluateRules(spec);

      const violation = result.violations.find(
        (v) => v.ruleId === "ACCEPTANCE-001" && v.message.includes("unique")
      );
      expect(violation).toBeDefined();
    });

    it("should warn about vague acceptance criteria", () => {
      const spec = createValidSpec({
        acceptance: [
          {
            id: "A1",
            given: "User is logged in",
            when: "User clicks submit",
            then: "It works correctly",
          },
        ],
      });

      const result = ruleEngine.evaluateRules(spec);

      const violation = result.violations.find(
        (v) => v.ruleId === "ACCEPTANCE-001" && v.message.includes("vague term")
      );
      expect(violation).toBeDefined();
      expect(violation?.severity).toBe("low");
    });
  });

  describe("contract rules", () => {
    it("should require contracts for Tier 1", () => {
      const spec = createValidSpec({
        risk_tier: 1,
        contracts: [],
      });

      const result = ruleEngine.evaluateRules(spec);

      const violation = result.violations.find(
        (v) => v.ruleId === "CONTRACT-001" && v.message.includes("Tier 1")
      );
      expect(violation).toBeDefined();
      expect(violation?.severity).toBe("high");
    });

    it("should require contracts for Tier 2", () => {
      const spec = createValidSpec({
        risk_tier: 2,
        contracts: [],
      });

      const result = ruleEngine.evaluateRules(spec);

      const violation = result.violations.find(
        (v) => v.ruleId === "CONTRACT-001" && v.message.includes("Tier 2")
      );
      expect(violation).toBeDefined();
      expect(violation?.severity).toBe("high");
    });

    it("should not require contracts for Tier 3", () => {
      const spec = createValidSpec({
        risk_tier: 3,
        contracts: [],
      });

      const result = ruleEngine.evaluateRules(spec);

      const violation = result.violations.find(
        (v) =>
          v.ruleId === "CONTRACT-001" && v.message.includes("requires contract")
      );
      expect(violation).toBeUndefined();
    });

    it("should fail with missing contract fields", () => {
      const spec = createValidSpec({
        contracts: [
          { path: "docs/api.yaml" }, // missing type
        ] as any,
      });

      const result = ruleEngine.evaluateRules(spec);

      const violation = result.violations.find(
        (v) =>
          v.ruleId === "CONTRACT-001" &&
          v.message.includes("missing type field")
      );
      expect(violation).toBeDefined();
    });

    it("should warn about unknown contract types", () => {
      const spec = createValidSpec({
        contracts: [{ type: "invalidtype" as any, path: "docs/api.yaml" }],
      });

      const result = ruleEngine.evaluateRules(spec);

      const violation = result.violations.find(
        (v) =>
          v.ruleId === "CONTRACT-001" &&
          v.message.includes("Unknown contract type")
      );
      expect(violation).toBeDefined();
      expect(violation?.severity).toBe("medium");
    });
  });

  describe("observability rules", () => {
    it("should require observability for Tier 1", () => {
      const spec = createValidSpec({
        risk_tier: 1,
        observability: undefined,
      });

      const result = ruleEngine.evaluateRules(spec);

      const violation = result.violations.find(
        (v) => v.ruleId === "OBSERVABILITY-001" && v.message.includes("Tier 1")
      );
      expect(violation).toBeDefined();
      expect(violation?.severity).toBe("high");
    });

    it("should check for required observability fields", () => {
      const spec = createValidSpec({
        risk_tier: 1,
        observability: {
          logs: "enabled",
          // missing metrics and alerts
        } as any,
      });

      const result = ruleEngine.evaluateRules(spec);

      const violations = result.violations.filter(
        (v) => v.ruleId === "OBSERVABILITY-001" && v.message.includes("missing")
      );
      expect(violations.length).toBeGreaterThan(0);
    });
  });

  describe("non-functional rules", () => {
    it("should warn about unrealistic performance requirements", () => {
      const spec = createValidSpec({
        non_functional: {
          perf: {
            api_p95_ms: 5, // unrealistically low
          },
        },
      });

      const result = ruleEngine.evaluateRules(spec);

      const violation = result.violations.find(
        (v) =>
          v.ruleId === "NONFUNC-001" && v.message.includes("too aggressive")
      );
      expect(violation).toBeDefined();
      expect(violation?.severity).toBe("medium");
    });

    it("should require comprehensive security for Tier 1", () => {
      const spec = createValidSpec({
        risk_tier: 1,
        non_functional: {
          security: ["input-validation"], // missing auth and authorization
        },
      });

      const result = ruleEngine.evaluateRules(spec);

      const violations = result.violations.filter(
        (v) =>
          v.ruleId === "NONFUNC-001" &&
          v.message.includes("security requirements missing")
      );
      expect(violations.length).toBeGreaterThan(0);
    });

    it("should suggest specific accessibility requirements", () => {
      const spec = createValidSpec({
        non_functional: {
          a11y: ["accessibility"], // too vague
        },
      });

      const result = ruleEngine.evaluateRules(spec);

      const violation = result.violations.find(
        (v) =>
          v.ruleId === "NONFUNC-001" && v.message.includes("should be specific")
      );
      expect(violation).toBeDefined();
      expect(violation?.severity).toBe("low");
    });
  });

  describe("violation severity handling", () => {
    it("should classify violations by severity", () => {
      const spec = createValidSpec({
        acceptance: [], // high severity
        non_functional: {
          perf: { api_p95_ms: 5 }, // medium severity
        },
      });

      const result = ruleEngine.evaluateRules(spec);

      const highSeverity = result.violations.filter(
        (v) => v.severity === "high"
      );
      const mediumSeverity = result.violations.filter(
        (v) => v.severity === "medium"
      );

      expect(highSeverity.length).toBeGreaterThan(0);
      expect(mediumSeverity.length).toBeGreaterThan(0);
    });

    it("should include suggestions for all violations", () => {
      const spec = createValidSpec({
        scope: { in: [], out: [] }, // triggers SCOPE-001 violation
      });

      const result = ruleEngine.evaluateRules(spec);

      for (const violation of result.violations) {
        expect(violation.suggestion).toBeDefined();
        expect(typeof violation.suggestion).toBe("string");
        expect(violation.suggestion!.length).toBeGreaterThan(0);
      }
    });
  });
});
