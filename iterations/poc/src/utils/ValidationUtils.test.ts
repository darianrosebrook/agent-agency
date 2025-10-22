/**
 * Tests for central validation utilities
 *
 * @author @darianrosebrook
 */

import { ValidationUtils } from "./ValidationUtils.js";

describe("ValidationUtils", () => {
  let validator: ValidationUtils;

  beforeEach(() => {
    validator = new ValidationUtils();
  });

  describe("validateJsonAgainstSchema", () => {
    test("should validate correct JSON against schema", () => {
      const schema = {
        type: "object",
        properties: {
          name: { type: "string" },
          age: { type: "number" },
        },
        required: ["name"],
      };

      const validData = { name: "John", age: 30 };
      const result = validator.validateJsonAgainstSchema(validData, schema);

      expect(result.passed).toBe(true);
      expect(result.score).toBe(1);
    });

    test("should reject invalid JSON against schema", () => {
      const schema = {
        type: "object",
        properties: {
          name: { type: "string" },
        },
        required: ["name"],
      };

      const invalidData = { age: 30 }; // Missing required name
      const result = validator.validateJsonAgainstSchema(invalidData, schema);

      expect(result.passed).toBe(false);
      expect(result.score).toBe(0);
      expect(result.errors).toBeDefined();
    });
  });

  describe("calculateTrustScore", () => {
    test("should calculate trust score correctly", () => {
      const components = {
        coverage_branch: 85,
        mutation_score: 65,
        contracts_consumer: true,
        contracts_provider: true,
        a11y_passed: true,
        perf_within_budget: true,
        flake_rate: 0.05,
      };

      const result = validator.calculateTrustScore(components);

      expect(result).toBeDefined();
      expect(result.total_score).toBeGreaterThan(0);
      expect(result.tier).toBeDefined();
      expect(result.breakdown).toBeDefined();
      expect(result.recommendations).toBeDefined();
    });

    test("should assign appropriate tiers", () => {
      const excellentComponents = {
        coverage_branch: 95,
        mutation_score: 80,
        contracts_consumer: true,
        contracts_provider: true,
        a11y_passed: true,
        perf_within_budget: true,
        flake_rate: 0.01,
      };

      const excellentResult =
        validator.calculateTrustScore(excellentComponents);
      expect(excellentResult.tier).toBe("platinum");

      const poorComponents = {
        coverage_branch: 60,
        mutation_score: 20,
        contracts_consumer: false,
        contracts_provider: false,
        a11y_passed: false,
        perf_within_budget: false,
        flake_rate: 0.3,
      };

      const poorResult = validator.calculateTrustScore(poorComponents);
      expect(poorResult.tier).toBe("needs-improvement");
    });
  });

  describe("validateContractResults", () => {
    test("should validate successful contract tests", () => {
      const consumerResults = { numPassed: 95, numTotal: 100 };
      const providerResults = { numPassed: 90, numTotal: 100 };

      const result = validator.validateContractResults(
        consumerResults,
        providerResults
      );

      expect(result.passed).toBe(true);
      expect(result.score).toBe(1);
      expect(result.coverage.endpointsTested).toBe(200);
    });

    test("should reject failing contract tests", () => {
      const consumerResults = { numPassed: 70, numTotal: 100 }; // Below 90%

      const result = validator.validateContractResults(consumerResults);

      expect(result.passed).toBe(false);
      expect(result.score).toBe(0); // Consumer failed, so score is 0
      expect(result.errors).toHaveLength(1);
    });
  });

  describe("validateFileExists", () => {
    test("should validate existing file", () => {
      // Use a file we know exists
      const result = validator.validateFileExists("../../package.json");

      expect(result.passed).toBe(true);
      expect(result.score).toBe(1);
    });

    test("should reject non-existent file", () => {
      const result = validator.validateFileExists("non-existent-file.txt");

      expect(result.passed).toBe(false);
      expect(result.score).toBe(0);
      expect(result.errors).toBeDefined();
    });
  });
});
