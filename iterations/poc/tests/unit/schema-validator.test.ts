/**
 * Tests for JSON Schema Validator
 *
 * Ensures working spec validation works correctly
 *
 * @author @darianrosebrook
 */

import { beforeAll, describe, expect, it } from "@jest/globals";
import {
  SchemaValidator,
  ValidationResult,
} from "../../src/utils/schema-validator";

describe("SchemaValidator", () => {
  let validator: SchemaValidator;

  beforeAll(() => {
    validator = new SchemaValidator();
  });

  describe("Working Spec Validation", () => {
    it("should validate a correct working spec", () => {
      const validSpec = {
        id: "FEAT-0001",
        title: "Test Feature Implementation",
        risk_tier: 2,
        mode: "feature",
        change_budget: {
          max_files: 10,
          max_loc: 500,
        },
        blast_radius: {
          modules: ["test"],
          data_migration: false,
        },
        operational_rollback_slo: "5m",
        scope: {
          in: ["src/test/"],
          out: ["node_modules/", "dist/"],
        },
        invariants: ["Test invariant"],
        acceptance: [
          {
            id: "A1",
            given: "test condition",
            when: "test action",
            then: "test outcome",
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
            path: "docs/api/test.yaml",
          },
        ],
      };

      const result: ValidationResult = validator.validateWorkingSpec(validSpec);

      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it("should reject spec with missing required fields", () => {
      const invalidSpec = {
        title: "Test Feature",
        // Missing id, risk_tier, etc.
      };

      const result: ValidationResult =
        validator.validateWorkingSpec(invalidSpec);

      expect(result.valid).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
      expect(
        result.errors.some(
          (error) => error.includes("id") || error.includes("required")
        )
      ).toBe(true);
    });

    it("should generate warnings for Tier 1 specs with insufficient acceptance criteria", () => {
      const tier1Spec = {
        id: "TIER1-001",
        title: "Tier 1 Feature",
        risk_tier: 1,
        mode: "feature",
        change_budget: {
          max_files: 25,
          max_loc: 1000,
        },
        blast_radius: {
          modules: ["critical"],
          data_migration: false,
        },
        operational_rollback_slo: "5m",
        scope: {
          in: ["src/critical/"],
          out: ["node_modules/"],
        },
        invariants: ["Critical invariant"],
        acceptance: [
          {
            id: "TIER1-001-A1",
            given: "critical condition",
            when: "critical action",
            then: "critical outcome",
          },
          // Only 1 acceptance criterion for Tier 1 - should warn
        ],
        non_functional: {
          perf: { api_p95_ms: 250 },
          security: ["input-validation"],
        },
        contracts: [
          {
            type: "typescript",
            path: "src/types/critical.ts",
            version: "1.0.0",
          },
        ],
      };

      const result: ValidationResult = validator.validateWorkingSpec(tier1Spec);

      expect(result.warnings).toContain(
        "Tier 1 spec should have at least 5 acceptance criteria, found 1"
      );
    });

    it("should generate warnings for missing contracts", () => {
      const specWithoutContracts = {
        id: "NOCONTRACT-001",
        title: "Feature Without Contracts",
        risk_tier: 2,
        mode: "feature",
        change_budget: {
          max_files: 10,
          max_loc: 500,
        },
        blast_radius: {
          modules: ["test"],
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
            id: "NOCONTRACT-001-A1",
            given: "test condition",
            when: "test action",
            then: "test outcome",
          },
        ],
        non_functional: {
          perf: { api_p95_ms: 250 },
          security: ["input-validation"],
        },
        // Missing contracts section
      };

      const result: ValidationResult =
        validator.validateWorkingSpec(specWithoutContracts);

      expect(result.warnings).toContain(
        "Working spec should define contracts for API compatibility"
      );
    });

    it("should generate warnings for missing non-functional requirements", () => {
      const specWithoutNonFunctional = {
        id: "NONF-001",
        title: "Feature Without Non-Functional",
        risk_tier: 2,
        mode: "feature",
        change_budget: {
          max_files: 10,
          max_loc: 500,
        },
        blast_radius: {
          modules: ["test"],
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
            id: "NONF-001-A1",
            given: "test condition",
            when: "test action",
            then: "test outcome",
          },
        ],
        contracts: [
          {
            type: "typescript",
            path: "src/types/test.ts",
            version: "1.0.0",
          },
        ],
        // Missing non_functional section
      };

      const result: ValidationResult = validator.validateWorkingSpec(
        specWithoutNonFunctional
      );

      expect(result.warnings).toContain(
        "Working spec should define non-functional requirements"
      );
    });

    it("should validate contract types correctly", () => {
      const specWithVariousContracts = {
        id: "FEAT-0003",
        title: "Feature With Various Contracts",
        risk_tier: 2,
        mode: "feature",
        change_budget: {
          max_files: 15,
          max_loc: 750,
        },
        blast_radius: {
          modules: ["contracts"],
          data_migration: false,
        },
        operational_rollback_slo: "5m",
        scope: {
          in: ["src/contracts/"],
          out: ["node_modules/"],
        },
        invariants: ["Contract invariant"],
        acceptance: [
          {
            id: "A1",
            given: "contracts defined",
            when: "validation runs",
            then: "all contracts validate",
          },
        ],
        non_functional: {
          perf: { api_p95_ms: 250 },
          security: ["input-validation"],
        },
        contracts: [
          {
            type: "typescript",
            path: "src/types/index.ts",
            version: "1.0.0",
          },
          {
            type: "openapi",
            path: "docs/api/test.yaml",
            version: "1.0.0",
          },
          {
            type: "sql",
            path: "migrations/001_create_test.sql",
            version: "1.0.0",
          },
          {
            type: "json-schema",
            path: "schemas/test.schema.json",
            version: "1.0.0",
          },
        ],
      };

      const result: ValidationResult = validator.validateWorkingSpec(
        specWithVariousContracts
      );

      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it("should validate scope boundaries correctly", () => {
      const specWithGoodScope = {
        id: "FEAT-0002",
        title: "Feature With Good Scope",
        risk_tier: 2,
        mode: "feature",
        change_budget: {
          max_files: 10,
          max_loc: 500,
        },
        blast_radius: {
          modules: ["scoped"],
          data_migration: false,
        },
        operational_rollback_slo: "5m",
        scope: {
          in: ["src/scoped/", "tests/scoped/"],
          out: ["node_modules/", "dist/", ".git/"],
        },
        invariants: ["Scope invariant"],
        acceptance: [
          {
            id: "A1",
            given: "scope defined",
            when: "changes made",
            then: "scope respected",
          },
        ],
        non_functional: {
          perf: { api_p95_ms: 250 },
          security: ["input-validation"],
        },
        contracts: [
          {
            type: "typescript",
            path: "src/types/scoped.ts",
            version: "1.0.0",
          },
        ],
      };

      const result: ValidationResult =
        validator.validateWorkingSpec(specWithGoodScope);

      expect(result.valid).toBe(true);
      expect(result.warnings).not.toContain(
        "Scope.out should include standard exclusions"
      );
    });

    it("should warn about missing operational rollback SLO", () => {
      const specWithoutSLO = {
        id: "NOSLO-001",
        title: "Feature Without SLO",
        risk_tier: 2,
        mode: "feature",
        change_budget: {
          max_files: 10,
          max_loc: 500,
        },
        blast_radius: {
          modules: ["test"],
          data_migration: false,
        },
        // Missing operational_rollback_slo
        scope: {
          in: ["src/test/"],
          out: ["node_modules/"],
        },
        invariants: ["Test invariant"],
        acceptance: [
          {
            id: "NOSLO-001-A1",
            given: "no SLO defined",
            when: "validation runs",
            then: "warning generated",
          },
        ],
        non_functional: {
          perf: { api_p95_ms: 250 },
          security: ["input-validation"],
        },
        contracts: [
          {
            type: "typescript",
            path: "src/types/test.ts",
            version: "1.0.0",
          },
        ],
      };

      const result: ValidationResult =
        validator.validateWorkingSpec(specWithoutSLO);

      expect(result.warnings).toContain(
        "Working spec should define operational rollback SLO"
      );
    });
  });

  describe("Schema Loading", () => {
    it("should load working spec schema successfully", () => {
      // Test that schemas were loaded
      const testSpec = {
        id: "SCHEMA-TEST",
        title: "Schema Test",
        risk_tier: 2,
        mode: "feature",
        change_budget: { max_files: 1, max_loc: 10 },
        blast_radius: { modules: [], data_migration: false },
        operational_rollback_slo: "1m",
        scope: { in: [], out: [] },
        invariants: [],
        acceptance: [],
        non_functional: { perf: {}, security: [] },
        contracts: [],
      };

      const result = validator.validateWorkingSpec(testSpec);
      // Should not fail due to missing schema
      expect(result.errors).not.toContain("Schema working-spec not found");
    });
  });

  describe("Error Formatting", () => {
    it("should format validation errors clearly", () => {
      const invalidSpec = {
        // Missing required fields
        title: "Invalid Spec",
      };

      const result = validator.validateWorkingSpec(invalidSpec);

      expect(result.valid).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);

      // Check that errors are properly formatted
      result.errors.forEach((error) => {
        expect(typeof error).toBe("string");
        expect(error.length).toBeGreaterThan(0);
      });
    });
  });
});
